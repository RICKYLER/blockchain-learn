//! HTTP API module for the LedgerDB blockchain.
//!
//! This module provides REST API endpoints for interacting with the blockchain,
//! including block retrieval, transaction management, mining operations, and WebSocket support.

mod handlers;
mod middleware;
mod responses;
mod websocket;

pub use handlers::*;
pub use middleware::*;
pub use responses::*;
pub use websocket::*;

use crate::core::Blockchain;
use crate::crypto::pow::{MiningProgress, ProofOfWorkMiner};
use crate::error::Result;
use crate::storage::PersistentStorage;
use axum::{
    extract::DefaultBodyLimit,
    http::{header, Method, StatusCode},
    middleware::from_fn,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use tower::ServiceBuilder;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};

/// Shared application state
#[derive(Debug, Clone)]
pub struct AppState {
    /// Blockchain instance
    pub blockchain: Arc<RwLock<Blockchain>>,
    /// Persistent storage
    pub storage: Arc<PersistentStorage>,
    /// Mining progress broadcaster
    pub mining_progress_tx: broadcast::Sender<MiningProgress>,
    /// Proof-of-work miner
    pub miner: Arc<RwLock<Option<ProofOfWorkMiner>>>,
    /// API configuration
    pub config: ApiConfig,
}

/// API configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    /// Maximum request body size
    pub max_body_size: usize,
    /// Request timeout in seconds
    pub request_timeout: u64,
    /// Rate limiting: requests per minute
    pub rate_limit: u32,
    /// Enable CORS
    pub enable_cors: bool,
    /// Enable request logging
    pub enable_logging: bool,
    /// WebSocket connection limit
    pub max_websocket_connections: usize,
    /// API version
    pub version: String,
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            max_body_size: 1024 * 1024, // 1MB
            request_timeout: 30,
            rate_limit: 100,
            enable_cors: true,
            enable_logging: true,
            max_websocket_connections: 100,
            version: "1.0.0".to_string(),
        }
    }
}

/// Create the main API router
pub fn create_router(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION])
        .allow_origin(Any);

    let middleware_stack = ServiceBuilder::new()
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .layer(DefaultBodyLimit::max(state.config.max_body_size))
        .layer(from_fn(request_logging_middleware))
        .layer(from_fn(rate_limiting_middleware));

    Router::new()
        // Health and info endpoints
        .route("/health", get(health_check))
        .route("/info", get(get_blockchain_info))
        .route("/stats", get(get_blockchain_stats))
        .route("/version", get(get_api_version))
        
        // Block endpoints
        .route("/blocks", get(get_blocks))
        .route("/blocks/latest", get(get_latest_block))
        .route("/blocks/height/:height", get(get_block_by_height))
        .route("/blocks/hash/:hash", get(get_block_by_hash))
        .route("/blocks/:block_id/transactions", get(get_block_transactions))
        
        // Transaction endpoints
        .route("/transactions", post(create_transaction))
        .route("/transactions", get(get_pending_transactions))
        .route("/transactions/:hash", get(get_transaction_by_hash))
        .route("/transactions/:hash/proof", get(get_transaction_merkle_proof))
        .route("/transactions/validate", post(validate_transaction))
        
        // Mining endpoints
        .route("/mining/start", post(start_mining))
        .route("/mining/stop", post(stop_mining))
        .route("/mining/status", get(get_mining_status))
        .route("/mining/difficulty", get(get_mining_difficulty))
        .route("/mining/progress", get(mining_progress_websocket))
        
        // Address endpoints
        .route("/addresses/:address/balance", get(get_address_balance))
        .route("/addresses/:address/utxos", get(get_address_utxos))
        .route("/addresses/:address/transactions", get(get_address_transactions))
        
        // UTXO endpoints
        .route("/utxos", get(get_all_utxos))
        .route("/utxos/:utxo_id", get(get_utxo_by_id))
        
        // Network endpoints
        .route("/network/peers", get(get_network_peers))
        .route("/network/status", get(get_network_status))
        
        // Admin endpoints (protected)
        .route("/admin/compact", post(compact_database))
        .route("/admin/backup", post(create_backup))
        .route("/admin/metrics", get(get_system_metrics))
        
        .layer(middleware_stack)
        .with_state(state)
}

/// API error response
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiError {
    /// Error code
    pub code: String,
    /// Error message
    pub message: String,
    /// Additional error details
    pub details: Option<serde_json::Value>,
    /// Request ID for tracking
    pub request_id: Option<String>,
}

impl ApiError {
    /// Create a new API error
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            details: None,
            request_id: None,
        }
    }

    /// Add details to the error
    pub fn with_details(mut self, details: serde_json::Value) -> Self {
        self.details = Some(details);
        self
    }

    /// Add request ID to the error
    pub fn with_request_id(mut self, request_id: impl Into<String>) -> Self {
        self.request_id = Some(request_id.into());
        self
    }
}

/// Convert internal errors to API errors
impl From<crate::error::LedgerError> for ApiError {
    fn from(error: crate::error::LedgerError) -> Self {
        match error {
            crate::error::LedgerError::Validation(e) => {
                ApiError::new("VALIDATION_ERROR", format!("Validation failed: {}", e))
            }
            crate::error::LedgerError::Blockchain(e) => {
                ApiError::new("BLOCKCHAIN_ERROR", format!("Blockchain error: {}", e))
            }
            crate::error::LedgerError::Storage(e) => {
                ApiError::new("STORAGE_ERROR", format!("Storage error: {}", e))
            }
            crate::error::LedgerError::Crypto(e) => {
                ApiError::new("CRYPTO_ERROR", format!("Cryptographic error: {}", e))
            }
            crate::error::LedgerError::Mining(e) => {
                ApiError::new("MINING_ERROR", format!("Mining error: {}", e))
            }
            crate::error::LedgerError::Api(e) => {
                ApiError::new("API_ERROR", format!("API error: {}", e))
            }
            crate::error::LedgerError::Config(e) => {
                ApiError::new("CONFIG_ERROR", format!("Configuration error: {}", e))
            }
            crate::error::LedgerError::Internal(e) => {
                ApiError::new("INTERNAL_ERROR", format!("Internal error: {}", e))
            }
            crate::error::LedgerError::Io(e) => {
                ApiError::new("IO_ERROR", format!("I/O error: {}", e))
            }
            crate::error::LedgerError::Serialization(e) => {
                ApiError::new("SERIALIZATION_ERROR", format!("Serialization error: {}", e))
            }
        }
    }
}

/// Convert API errors to HTTP responses
impl axum::response::IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let status = match self.code.as_str() {
            "VALIDATION_ERROR" => StatusCode::BAD_REQUEST,
            "NOT_FOUND" => StatusCode::NOT_FOUND,
            "UNAUTHORIZED" => StatusCode::UNAUTHORIZED,
            "FORBIDDEN" => StatusCode::FORBIDDEN,
            "RATE_LIMITED" => StatusCode::TOO_MANY_REQUESTS,
            "INTERNAL_ERROR" => StatusCode::INTERNAL_SERVER_ERROR,
            _ => StatusCode::BAD_REQUEST,
        };

        (status, Json(self)).into_response()
    }
}

/// Pagination parameters
#[derive(Debug, Deserialize)]
pub struct PaginationParams {
    /// Page number (0-based)
    pub page: Option<u64>,
    /// Number of items per page
    pub limit: Option<u64>,
    /// Sort order (asc/desc)
    pub order: Option<String>,
}

impl Default for PaginationParams {
    fn default() -> Self {
        Self {
            page: Some(0),
            limit: Some(20),
            order: Some("desc".to_string()),
        }
    }
}

/// Paginated response wrapper
#[derive(Debug, Serialize)]
pub struct PaginatedResponse<T> {
    /// Response data
    pub data: Vec<T>,
    /// Pagination metadata
    pub pagination: PaginationMeta,
}

/// Pagination metadata
#[derive(Debug, Serialize)]
pub struct PaginationMeta {
    /// Current page
    pub page: u64,
    /// Items per page
    pub limit: u64,
    /// Total number of items
    pub total: u64,
    /// Total number of pages
    pub total_pages: u64,
    /// Whether there's a next page
    pub has_next: bool,
    /// Whether there's a previous page
    pub has_prev: bool,
}

impl PaginationMeta {
    /// Create pagination metadata
    pub fn new(page: u64, limit: u64, total: u64) -> Self {
        let total_pages = (total + limit - 1) / limit; // Ceiling division
        
        Self {
            page,
            limit,
            total,
            total_pages,
            has_next: page + 1 < total_pages,
            has_prev: page > 0,
        }
    }
}

/// Create a paginated response
pub fn paginate<T>(
    data: Vec<T>,
    page: u64,
    limit: u64,
    total: u64,
) -> PaginatedResponse<T> {
    PaginatedResponse {
        data,
        pagination: PaginationMeta::new(page, limit, total),
    }
}

/// Request/Response types for API endpoints
pub mod types {
    use super::*;
    use crate::core::{Block, Transaction};
    use crate::crypto::{Address, Hash256};
    use chrono::{DateTime, Utc};

    /// Health check response
    #[derive(Debug, Serialize)]
    pub struct HealthResponse {
        pub status: String,
        pub timestamp: DateTime<Utc>,
        pub version: String,
        pub uptime: u64,
    }

    /// Blockchain info response
    #[derive(Debug, Serialize)]
    pub struct BlockchainInfoResponse {
        pub height: u64,
        pub latest_block_hash: Hash256,
        pub total_transactions: u64,
        pub total_supply: u64,
        pub difficulty: u32,
        pub network_hash_rate: f64,
    }

    /// Transaction creation request
    #[derive(Debug, Deserialize)]
    pub struct CreateTransactionRequest {
        pub inputs: Vec<TransactionInputRequest>,
        pub outputs: Vec<TransactionOutputRequest>,
        pub fee: Option<u64>,
    }

    /// Transaction input request
    #[derive(Debug, Deserialize)]
    pub struct TransactionInputRequest {
        pub previous_tx_hash: Hash256,
        pub output_index: u32,
        pub signature: Option<String>,
        pub public_key: Option<String>,
    }

    /// Transaction output request
    #[derive(Debug, Deserialize)]
    pub struct TransactionOutputRequest {
        pub amount: u64,
        pub recipient_address: Address,
    }

    /// Mining start request
    #[derive(Debug, Deserialize)]
    pub struct StartMiningRequest {
        pub miner_address: Address,
        pub max_iterations: Option<u64>,
    }

    /// Mining status response
    #[derive(Debug, Serialize)]
    pub struct MiningStatusResponse {
        pub is_mining: bool,
        pub current_block_height: Option<u64>,
        pub difficulty: Option<u32>,
        pub hash_rate: Option<f64>,
        pub estimated_time: Option<u64>,
    }

    /// Address balance response
    #[derive(Debug, Serialize)]
    pub struct AddressBalanceResponse {
        pub address: Address,
        pub balance: u64,
        pub utxo_count: usize,
        pub last_updated: DateTime<Utc>,
    }

    /// UTXO response
    #[derive(Debug, Serialize)]
    pub struct UtxoResponse {
        pub utxo_id: String,
        pub amount: u64,
        pub recipient_address: Address,
        pub block_height: u64,
        pub tx_hash: Hash256,
        pub output_index: u32,
        pub is_spent: bool,
    }

    /// Network status response
    #[derive(Debug, Serialize)]
    pub struct NetworkStatusResponse {
        pub connected_peers: u32,
        pub network_height: u64,
        pub sync_status: String,
        pub last_sync: DateTime<Utc>,
    }

    /// System metrics response
    #[derive(Debug, Serialize)]
    pub struct SystemMetricsResponse {
        pub memory_usage: u64,
        pub cpu_usage: f64,
        pub disk_usage: u64,
        pub network_io: NetworkIoMetrics,
        pub database_size: u64,
        pub active_connections: u32,
    }

    /// Network I/O metrics
    #[derive(Debug, Serialize)]
    pub struct NetworkIoMetrics {
        pub bytes_sent: u64,
        pub bytes_received: u64,
        pub requests_per_second: f64,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_error_creation() {
        let error = ApiError::new("TEST_ERROR", "Test message");
        assert_eq!(error.code, "TEST_ERROR");
        assert_eq!(error.message, "Test message");
        assert!(error.details.is_none());
    }

    #[test]
    fn test_pagination_meta() {
        let meta = PaginationMeta::new(0, 10, 25);
        assert_eq!(meta.page, 0);
        assert_eq!(meta.limit, 10);
        assert_eq!(meta.total, 25);
        assert_eq!(meta.total_pages, 3);
        assert!(meta.has_next);
        assert!(!meta.has_prev);
    }

    #[test]
    fn test_api_config_default() {
        let config = ApiConfig::default();
        assert_eq!(config.max_body_size, 1024 * 1024);
        assert_eq!(config.request_timeout, 30);
        assert_eq!(config.rate_limit, 100);
        assert!(config.enable_cors);
        assert!(config.enable_logging);
    }
}