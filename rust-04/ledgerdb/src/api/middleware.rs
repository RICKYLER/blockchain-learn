//! Middleware functions for the HTTP API.
//!
//! This module provides middleware for request logging, rate limiting, authentication,
//! CORS handling, and other cross-cutting concerns.

use axum::{
    extract::Request,
    http::{HeaderMap, Method, StatusCode},
    middleware::Next,
    response::Response,
};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use tokio::time::sleep;
use tracing::{error, info, warn};
use uuid::Uuid;

/// Request logging middleware
pub async fn request_logging_middleware(
    request: Request,
    next: Next,
) -> Response {
    let start_time = Instant::now();
    let request_id = Uuid::new_v4().to_string();
    let method = request.method().clone();
    let uri = request.uri().clone();
    let headers = request.headers().clone();
    
    // Get user agent and other relevant headers
    let user_agent = headers
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown");
    
    let content_length = headers
        .get("content-length")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(0);
    
    info!(
        "[{}] {} {} - {} bytes",
        request_id, method, uri, content_length
    );
    
    // Process the request
    let response = next.run(request).await;
    
    let duration = start_time.elapsed();
    let status = response.status();
    
    // Log the response
    let log_level = match status.as_u16() {
        200..=299 => tracing::Level::INFO,
        300..=399 => tracing::Level::INFO,
        400..=499 => tracing::Level::WARN,
        500..=599 => tracing::Level::ERROR,
        _ => tracing::Level::INFO,
    };
    
    match log_level {
        tracing::Level::INFO => info!(
            "[{}] {} {} {} - {}ms - {}",
            request_id, method, uri, status, duration.as_millis(), user_agent
        ),
        tracing::Level::WARN => warn!(
            "[{}] {} {} {} - {}ms - {}",
            request_id, method, uri, status, duration.as_millis(), user_agent
        ),
        tracing::Level::ERROR => error!(
            "[{}] {} {} {} - {}ms - {}",
            request_id, method, uri, status, duration.as_millis(), user_agent
        ),
        _ => {},
    }
    
    response
}

/// Rate limiting middleware
pub async fn rate_limiting_middleware(
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Simple in-memory rate limiter
    // In production, you'd want to use Redis or a more sophisticated solution
    static RATE_LIMITER: Mutex<Option<Arc<RateLimiter>>> = Mutex::new(None);
    
    let rate_limiter = {
        let mut guard = RATE_LIMITER.lock().unwrap();
        if guard.is_none() {
            *guard = Some(Arc::new(RateLimiter::new(100, Duration::from_secs(60))));
        }
        guard.as_ref().unwrap().clone()
    };
    
    // For now, skip rate limiting since we can't easily extract IP from request
    // In production, you'd want to implement proper IP extraction
    // if !rate_limiter.check_rate_limit(addr.ip().to_string()).await {
    //     warn!("Rate limit exceeded for {}", addr.ip());
    //     return Err(StatusCode::TOO_MANY_REQUESTS);
    // }
    
    Ok(next.run(request).await)
}

/// Authentication middleware (placeholder)
pub async fn auth_middleware(
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Check for API key or JWT token
    let _auth_header = request.headers().get("authorization");
    
    // For now, we'll allow all requests
    // TODO: Implement proper authentication
    
    Ok(next.run(request).await)
}

/// CORS middleware (handled by tower-http, but this is a custom implementation)
pub async fn cors_middleware(
    request: Request,
    next: Next,
) -> Response {
    let method = request.method().clone();
    let mut response = next.run(request).await;
    
    // Add CORS headers
    let response_headers = response.headers_mut();
    
    response_headers.insert(
        "access-control-allow-origin",
        "*".parse().unwrap(),
    );
    
    response_headers.insert(
        "access-control-allow-methods",
        "GET, POST, PUT, DELETE, OPTIONS".parse().unwrap(),
    );
    
    response_headers.insert(
        "access-control-allow-headers",
        "content-type, authorization, x-request-id".parse().unwrap(),
    );
    
    response_headers.insert(
        "access-control-max-age",
        "86400".parse().unwrap(),
    );
    
    // Handle preflight requests
    if method == Method::OPTIONS {
        *response.status_mut() = StatusCode::NO_CONTENT;
    }
    
    response
}

/// Request timeout middleware
pub async fn timeout_middleware(
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let timeout_duration = Duration::from_secs(30); // 30 second timeout
    
    match tokio::time::timeout(timeout_duration, next.run(request)).await {
        Ok(response) => Ok(response),
        Err(_) => {
            error!("Request timed out after {:?}", timeout_duration);
            Err(StatusCode::REQUEST_TIMEOUT)
        }
    }
}

/// Request size limiting middleware
pub async fn request_size_middleware(
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    const MAX_REQUEST_SIZE: u64 = 10 * 1024 * 1024; // 10MB
    
    if let Some(content_length) = request.headers().get("content-length") {
        if let Ok(length_str) = content_length.to_str() {
            if let Ok(length) = length_str.parse::<u64>() {
                if length > MAX_REQUEST_SIZE {
                    warn!("Request too large: {} bytes", length);
                    return Err(StatusCode::PAYLOAD_TOO_LARGE);
                }
            }
        }
    }
    
    Ok(next.run(request).await)
}

/// Security headers middleware
pub async fn security_headers_middleware(
    request: Request,
    next: Next,
) -> Response {
    let mut response = next.run(request).await;
    
    let headers = response.headers_mut();
    
    // Add security headers
    headers.insert(
        "x-content-type-options",
        "nosniff".parse().unwrap(),
    );
    
    headers.insert(
        "x-frame-options",
        "DENY".parse().unwrap(),
    );
    
    headers.insert(
        "x-xss-protection",
        "1; mode=block".parse().unwrap(),
    );
    
    headers.insert(
        "strict-transport-security",
        "max-age=31536000; includeSubDomains".parse().unwrap(),
    );
    
    headers.insert(
        "referrer-policy",
        "strict-origin-when-cross-origin".parse().unwrap(),
    );
    
    response
}

/// Simple in-memory rate limiter
#[derive(Debug)]
pub struct RateLimiter {
    /// Maximum requests per window
    max_requests: u32,
    /// Time window duration
    window_duration: Duration,
    /// Request counts per client
    clients: Arc<Mutex<HashMap<String, ClientRateLimit>>>,
}

#[derive(Debug, Clone)]
struct ClientRateLimit {
    /// Number of requests in current window
    count: u32,
    /// Window start time
    window_start: Instant,
}

impl RateLimiter {
    /// Create a new rate limiter
    pub fn new(max_requests: u32, window_duration: Duration) -> Self {
        Self {
            max_requests,
            window_duration,
            clients: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    /// Check if a client is within rate limits
    pub async fn check_rate_limit(&self, client_id: String) -> bool {
        let now = Instant::now();
        let mut clients = self.clients.lock().unwrap();
        
        let client_limit = clients.entry(client_id.clone()).or_insert(ClientRateLimit {
            count: 0,
            window_start: now,
        });
        
        // Check if we need to reset the window
        if now.duration_since(client_limit.window_start) >= self.window_duration {
            client_limit.count = 0;
            client_limit.window_start = now;
        }
        
        // Check if client is within limits
        if client_limit.count >= self.max_requests {
            false
        } else {
            client_limit.count += 1;
            true
        }
    }
    
    /// Clean up expired entries (should be called periodically)
    pub async fn cleanup_expired(&self) {
        let now = Instant::now();
        let mut clients = self.clients.lock().unwrap();
        
        clients.retain(|_, client_limit| {
            now.duration_since(client_limit.window_start) < self.window_duration * 2
        });
    }
}

/// API key validation
#[derive(Debug, Clone)]
pub struct ApiKeyValidator {
    /// Valid API keys
    valid_keys: Arc<Mutex<HashMap<String, ApiKeyInfo>>>,
}

#[derive(Debug, Clone)]
pub struct ApiKeyInfo {
    /// Key name/description
    pub name: String,
    /// Rate limit for this key
    pub rate_limit: u32,
    /// Whether the key is active
    pub active: bool,
    /// Key creation time
    pub created_at: Instant,
    /// Last used time
    pub last_used: Option<Instant>,
}

impl ApiKeyValidator {
    /// Create a new API key validator
    pub fn new() -> Self {
        Self {
            valid_keys: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    /// Add an API key
    pub fn add_key(&self, key: String, info: ApiKeyInfo) {
        let mut keys = self.valid_keys.lock().unwrap();
        keys.insert(key, info);
    }
    
    /// Validate an API key
    pub fn validate_key(&self, key: &str) -> Option<ApiKeyInfo> {
        let mut keys = self.valid_keys.lock().unwrap();
        
        if let Some(info) = keys.get_mut(key) {
            if info.active {
                info.last_used = Some(Instant::now());
                Some(info.clone())
            } else {
                None
            }
        } else {
            None
        }
    }
    
    /// Revoke an API key
    pub fn revoke_key(&self, key: &str) -> bool {
        let mut keys = self.valid_keys.lock().unwrap();
        
        if let Some(info) = keys.get_mut(key) {
            info.active = false;
            true
        } else {
            false
        }
    }
}

/// Request metrics collector
#[derive(Debug, Default)]
pub struct RequestMetrics {
    /// Total requests
    pub total_requests: Arc<Mutex<u64>>,
    /// Requests by status code
    pub status_codes: Arc<Mutex<HashMap<u16, u64>>>,
    /// Average response time
    pub response_times: Arc<Mutex<Vec<Duration>>>,
    /// Requests by endpoint
    pub endpoints: Arc<Mutex<HashMap<String, u64>>>,
}

impl RequestMetrics {
    /// Create new request metrics collector
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Record a request
    pub fn record_request(
        &self,
        method: &Method,
        uri: &Uri,
        status: StatusCode,
        duration: Duration,
    ) {
        // Increment total requests
        {
            let mut total = self.total_requests.lock().unwrap();
            *total += 1;
        }
        
        // Record status code
        {
            let mut status_codes = self.status_codes.lock().unwrap();
            *status_codes.entry(status.as_u16()).or_insert(0) += 1;
        }
        
        // Record response time
        {
            let mut response_times = self.response_times.lock().unwrap();
            response_times.push(duration);
            
            // Keep only last 1000 response times
            if response_times.len() > 1000 {
                response_times.remove(0);
            }
        }
        
        // Record endpoint
        {
            let mut endpoints = self.endpoints.lock().unwrap();
            let endpoint = format!("{} {}", method, uri.path());
            *endpoints.entry(endpoint).or_insert(0) += 1;
        }
    }
    
    /// Get metrics summary
    pub fn get_summary(&self) -> serde_json::Value {
        let total_requests = *self.total_requests.lock().unwrap();
        let status_codes = self.status_codes.lock().unwrap().clone();
        let response_times = self.response_times.lock().unwrap();
        let endpoints = self.endpoints.lock().unwrap().clone();
        
        let avg_response_time = if !response_times.is_empty() {
            response_times.iter().sum::<Duration>().as_millis() as f64 / response_times.len() as f64
        } else {
            0.0
        };
        
        serde_json::json!({
            "total_requests": total_requests,
            "status_codes": status_codes,
            "average_response_time_ms": avg_response_time,
            "endpoints": endpoints
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    
    #[tokio::test]
    async fn test_rate_limiter() {
        let limiter = RateLimiter::new(2, Duration::from_secs(1));
        let client_id = "test_client".to_string();
        
        // First two requests should pass
        assert!(limiter.check_rate_limit(client_id.clone()).await);
        assert!(limiter.check_rate_limit(client_id.clone()).await);
        
        // Third request should fail
        assert!(!limiter.check_rate_limit(client_id.clone()).await);
        
        // Wait for window to reset
        tokio::time::sleep(Duration::from_secs(1)).await;
        
        // Should pass again
        assert!(limiter.check_rate_limit(client_id).await);
    }
    
    #[test]
    fn test_api_key_validator() {
        let validator = ApiKeyValidator::new();
        let key = "test_key".to_string();
        let info = ApiKeyInfo {
            name: "Test Key".to_string(),
            rate_limit: 100,
            active: true,
            created_at: Instant::now(),
            last_used: None,
        };
        
        // Add key
        validator.add_key(key.clone(), info);
        
        // Validate key
        assert!(validator.validate_key(&key).is_some());
        
        // Revoke key
        assert!(validator.revoke_key(&key));
        
        // Should no longer validate
        assert!(validator.validate_key(&key).is_none());
    }
    
    #[test]
    fn test_request_metrics() {
        let metrics = RequestMetrics::new();
        
        metrics.record_request(
            &Method::GET,
            &Uri::from_static("/test"),
            StatusCode::OK,
            Duration::from_millis(100),
        );
        
        let summary = metrics.get_summary();
        assert_eq!(summary["total_requests"], 1);
        assert_eq!(summary["status_codes"]["200"], 1);
    }
}