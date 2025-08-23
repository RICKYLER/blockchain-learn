//! Time utilities for the LedgerDB blockchain.
//!
//! This module provides time-related functions and utilities for working
//! with timestamps, durations, and time formatting.

use std::time::{Duration, SystemTime, UNIX_EPOCH};
use crate::error::LedgerError;

/// Get current Unix timestamp in seconds
pub fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// Get current Unix timestamp in milliseconds
pub fn current_timestamp_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

/// Get current Unix timestamp in microseconds
pub fn current_timestamp_micros() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_micros() as u64
}

/// Convert Unix timestamp to SystemTime
pub fn timestamp_to_system_time(timestamp: u64) -> SystemTime {
    UNIX_EPOCH + Duration::from_secs(timestamp)
}

/// Convert SystemTime to Unix timestamp
pub fn system_time_to_timestamp(time: SystemTime) -> Result<u64, LedgerError> {
    time.duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .map_err(|e| LedgerError::Internal(format!("Invalid system time: {}", e)))
}

/// Format timestamp as ISO 8601 string
pub fn format_timestamp(timestamp: u64) -> String {
    let _system_time = timestamp_to_system_time(timestamp);
    // Simple formatting - in a real implementation you'd use chrono
    format!("timestamp:{}", timestamp)
}

/// Parse ISO 8601 string to timestamp
pub fn parse_timestamp(timestamp_str: &str) -> Result<u64, LedgerError> {
    // Simple parsing - in a real implementation you'd use chrono
    if let Some(ts_str) = timestamp_str.strip_prefix("timestamp:") {
        ts_str.parse::<u64>()
            .map_err(|e| LedgerError::Internal(format!("Invalid timestamp format: {}", e)))
    } else {
        Err(LedgerError::Internal("Invalid timestamp format".to_string()))
    }
}

/// Check if timestamp is within acceptable range (not too far in past/future)
pub fn is_timestamp_valid(timestamp: u64, max_drift: Duration) -> bool {
    let now = current_timestamp();
    let max_drift_secs = max_drift.as_secs();
    
    // Allow some drift in both directions
    timestamp >= now.saturating_sub(max_drift_secs) && 
    timestamp <= now.saturating_add(max_drift_secs)
}

/// Calculate time difference between two timestamps
pub fn time_diff(timestamp1: u64, timestamp2: u64) -> Duration {
    let diff = if timestamp1 > timestamp2 {
        timestamp1 - timestamp2
    } else {
        timestamp2 - timestamp1
    };
    Duration::from_secs(diff)
}

/// Format duration as human-readable string
pub fn format_duration(duration: Duration) -> String {
    let total_seconds = duration.as_secs();
    
    if total_seconds < 60 {
        format!("{}s", total_seconds)
    } else if total_seconds < 3600 {
        let minutes = total_seconds / 60;
        let seconds = total_seconds % 60;
        format!("{}m {}s", minutes, seconds)
    } else if total_seconds < 86400 {
        let hours = total_seconds / 3600;
        let minutes = (total_seconds % 3600) / 60;
        format!("{}h {}m", hours, minutes)
    } else {
        let days = total_seconds / 86400;
        let hours = (total_seconds % 86400) / 3600;
        format!("{}d {}h", days, hours)
    }
}

/// Sleep for specified duration (async)
pub async fn sleep(duration: Duration) {
    tokio::time::sleep(duration).await;
}

/// Sleep for specified number of seconds (async)
pub async fn sleep_secs(seconds: u64) {
    sleep(Duration::from_secs(seconds)).await;
}

/// Sleep for specified number of milliseconds (async)
pub async fn sleep_millis(millis: u64) {
    sleep(Duration::from_millis(millis)).await;
}

/// Create a timeout future
pub async fn timeout<F, T>(duration: Duration, future: F) -> Result<T, LedgerError>
where
    F: std::future::Future<Output = T>,
{
    tokio::time::timeout(duration, future)
        .await
        .map_err(|_| LedgerError::Internal("Operation timed out".to_string()))
}

/// Measure execution time of an async operation
pub async fn measure_time<F, T>(future: F) -> (T, Duration)
where
    F: std::future::Future<Output = T>,
{
    let start = std::time::Instant::now();
    let result = future.await;
    let elapsed = start.elapsed();
    (result, elapsed)
}

/// Rate limiter based on time windows
pub struct RateLimiter {
    max_requests: u32,
    window_duration: Duration,
    requests: Vec<u64>,
}

impl RateLimiter {
    /// Create a new rate limiter
    pub fn new(max_requests: u32, window_duration: Duration) -> Self {
        Self {
            max_requests,
            window_duration,
            requests: Vec::new(),
        }
    }
    
    /// Check if a request is allowed
    pub fn is_allowed(&mut self) -> bool {
        let now = current_timestamp_millis();
        let window_start = now.saturating_sub(self.window_duration.as_millis() as u64);
        
        // Remove old requests outside the window
        self.requests.retain(|&timestamp| timestamp > window_start);
        
        if self.requests.len() < self.max_requests as usize {
            self.requests.push(now);
            true
        } else {
            false
        }
    }
    
    /// Get remaining requests in current window
    pub fn remaining_requests(&self) -> u32 {
        self.max_requests.saturating_sub(self.requests.len() as u32)
    }
    
    /// Get time until window resets
    pub fn time_until_reset(&self) -> Duration {
        if let Some(&oldest) = self.requests.first() {
            let now = current_timestamp_millis();
            let window_end = oldest + self.window_duration.as_millis() as u64;
            if window_end > now {
                Duration::from_millis(window_end - now)
            } else {
                Duration::from_secs(0)
            }
        } else {
            Duration::from_secs(0)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    
    #[test]
    fn test_current_timestamp() {
        let timestamp = current_timestamp();
        assert!(timestamp > 0);
        
        let timestamp_millis = current_timestamp_millis();
        assert!(timestamp_millis > timestamp * 1000);
    }
    
    #[test]
    fn test_timestamp_conversion() {
        let timestamp = 1640995200; // 2022-01-01 00:00:00 UTC
        let system_time = timestamp_to_system_time(timestamp);
        let converted_back = system_time_to_timestamp(system_time).unwrap();
        assert_eq!(timestamp, converted_back);
    }
    
    #[test]
    fn test_timestamp_validation() {
        let now = current_timestamp();
        let max_drift = Duration::from_secs(300); // 5 minutes
        
        assert!(is_timestamp_valid(now, max_drift));
        assert!(is_timestamp_valid(now - 100, max_drift));
        assert!(is_timestamp_valid(now + 100, max_drift));
        assert!(!is_timestamp_valid(now - 400, max_drift));
        assert!(!is_timestamp_valid(now + 400, max_drift));
    }
    
    #[test]
    fn test_time_diff() {
        let diff = time_diff(1000, 500);
        assert_eq!(diff, Duration::from_secs(500));
        
        let diff = time_diff(500, 1000);
        assert_eq!(diff, Duration::from_secs(500));
    }
    
    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(Duration::from_secs(30)), "30s");
        assert_eq!(format_duration(Duration::from_secs(90)), "1m 30s");
        assert_eq!(format_duration(Duration::from_secs(3661)), "1h 1m");
        assert_eq!(format_duration(Duration::from_secs(90061)), "1d 1h");
    }
    
    #[test]
    fn test_format_parse_timestamp() {
        let timestamp = 1640995200;
        let formatted = format_timestamp(timestamp);
        let parsed = parse_timestamp(&formatted).unwrap();
        assert_eq!(timestamp, parsed);
    }
    
    #[test]
    fn test_rate_limiter() {
        let mut limiter = RateLimiter::new(3, Duration::from_secs(1));
        
        // First 3 requests should be allowed
        assert!(limiter.is_allowed());
        assert!(limiter.is_allowed());
        assert!(limiter.is_allowed());
        
        // 4th request should be denied
        assert!(!limiter.is_allowed());
        
        assert_eq!(limiter.remaining_requests(), 0);
    }
    
    #[tokio::test]
    async fn test_timeout() {
        // Test successful operation
        let result = timeout(Duration::from_millis(100), async { 42 }).await;
        assert_eq!(result.unwrap(), 42);
        
        // Test timeout
        let result = timeout(Duration::from_millis(10), async {
            tokio::time::sleep(Duration::from_millis(50)).await;
            42
        }).await;
        assert!(result.is_err());
    }
    
    #[tokio::test]
    async fn test_measure_time() {
        let (result, elapsed) = measure_time(async {
            tokio::time::sleep(Duration::from_millis(10)).await;
            42
        }).await;
        
        assert_eq!(result, 42);
        assert!(elapsed >= Duration::from_millis(10));
    }
}