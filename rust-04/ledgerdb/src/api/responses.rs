//! Standardized API response structures.
//!
//! This module defines common response formats, error handling, and serialization
//! for the HTTP API endpoints.

use crate::core::{Block, Transaction};
use crate::crypto::{Address, Hash256};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Health check response
#[derive(Debug, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub timestamp: DateTime<Utc>,
    pub version: String,
    pub uptime: u64,
}

/// Blockchain info response
#[derive(Debug, Serialize, Deserialize)]
pub struct BlockchainInfoResponse {
    pub height: u64,
    pub latest_block_hash: Hash256,
    pub total_transactions: u64,
    pub total_supply: u64,
    pub difficulty: u32,
    pub network_hash_rate: f64,
}

/// Mining status response
#[derive(Debug, Serialize, Deserialize)]
pub struct MiningStatusResponse {
    pub is_mining: bool,
    pub current_block_height: u64,
    pub difficulty: u32,
    pub hash_rate: f64,
}

/// Address balance response
#[derive(Debug, Serialize, Deserialize)]
pub struct AddressBalanceResponse {
    pub address: Address,
    pub balance: u64,
    pub utxo_count: usize,
}

/// UTXO response
#[derive(Debug, Serialize, Deserialize)]
pub struct UtxoResponse {
    pub utxo_id: String,
    pub tx_hash: Hash256,
    pub output_index: u32,
    pub amount: u64,
    pub address: Address,
    pub recipient: Address,
    pub block_height: u64,
    pub is_spent: bool,
}

/// Network status response
#[derive(Debug, Serialize, Deserialize)]
pub struct NetworkStatusResponse {
    pub peer_count: u32,
    pub is_synced: bool,
    pub sync_progress: f64,
}

/// System metrics response
#[derive(Debug, Serialize, Deserialize)]
pub struct SystemMetricsResponse {
    pub memory_usage: u64,
    pub cpu_usage: f64,
    pub disk_usage: u64,
    pub network_io: NetworkIoMetrics,
}

/// Network IO metrics
#[derive(Debug, Serialize, Deserialize)]
pub struct NetworkIoMetrics {
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub packets_sent: u64,
    pub packets_received: u64,
}

/// Create transaction request
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateTransactionRequest {
    pub from: Address,
    pub to: Address,
    pub amount: u64,
    pub fee: Option<u64>,
}

/// Start mining request
#[derive(Debug, Serialize, Deserialize)]
pub struct StartMiningRequest {
    pub address: Address,
    pub threads: Option<u32>,
}

/// Paginated response wrapper
#[derive(Debug, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub total: u64,
    pub page: u32,
    pub per_page: u32,
    pub total_pages: u32,
}

/// Pagination parameters
#[derive(Debug, Serialize, Deserialize)]
pub struct PaginationParams {
    pub page: Option<u32>,
    pub per_page: Option<u32>,
}

/// Standard API response wrapper
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    /// Response data
    pub data: T,
    /// Response metadata
    pub meta: ResponseMeta,
}

/// Response metadata
#[derive(Debug, Serialize, Deserialize)]
pub struct ResponseMeta {
    /// Request ID for tracing
    pub request_id: Option<String>,
    /// Response timestamp
    pub timestamp: DateTime<Utc>,
    /// API version
    pub version: String,
    /// Processing time in milliseconds
    pub processing_time_ms: Option<u64>,
    /// Additional metadata
    pub extra: Option<HashMap<String, serde_json::Value>>,
}

impl ResponseMeta {
    /// Create new response metadata
    pub fn new() -> Self {
        Self {
            request_id: None,
            timestamp: Utc::now(),
            version: "1.0.0".to_string(),
            processing_time_ms: None,
            extra: None,
        }
    }
    
    /// Set request ID
    pub fn with_request_id(mut self, request_id: String) -> Self {
        self.request_id = Some(request_id);
        self
    }
    
    /// Set processing time
    pub fn with_processing_time(mut self, processing_time_ms: u64) -> Self {
        self.processing_time_ms = Some(processing_time_ms);
        self
    }
    
    /// Add extra metadata
    pub fn with_extra(mut self, key: String, value: serde_json::Value) -> Self {
        if self.extra.is_none() {
            self.extra = Some(HashMap::new());
        }
        self.extra.as_mut().unwrap().insert(key, value);
        self
    }
}

impl Default for ResponseMeta {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a successful API response
pub fn success<T>(data: T) -> ApiResponse<T> {
    ApiResponse {
        data,
        meta: ResponseMeta::new(),
    }
}

/// Create a successful API response with metadata
pub fn success_with_meta<T>(data: T, meta: ResponseMeta) -> ApiResponse<T> {
    ApiResponse { data, meta }
}

/// Block response with additional metadata
#[derive(Debug, Serialize, Deserialize)]
pub struct BlockResponse {
    /// Block data
    #[serde(flatten)]
    pub block: Block,
    /// Block size in bytes
    pub size: usize,
    /// Number of confirmations
    pub confirmations: u64,
    /// Block reward
    pub reward: u64,
    /// Transaction fees
    pub total_fees: u64,
    /// Block difficulty
    pub difficulty: u32,
    /// Time since previous block
    pub time_since_previous: Option<u64>,
}

impl BlockResponse {
    /// Create a block response from a block
    pub fn from_block(block: Block, current_height: u64) -> Self {
        let size = bincode::serialize(&block).map(|b| b.len()).unwrap_or(0);
        let confirmations = current_height.saturating_sub(block.index);
        let total_fees = block.transactions.iter()
            .map(|tx| tx.fee.base_fee + tx.fee.per_byte_fee * tx.size.unwrap_or(0) as u64)
            .sum();
        
        Self {
            block,
            size,
            confirmations,
            reward: 50_000_000, // TODO: Calculate actual block reward
            total_fees,
            difficulty: 0, // TODO: Get from block header
            time_since_previous: None, // TODO: Calculate from previous block
        }
    }
}

/// Transaction response with additional metadata
#[derive(Debug, Serialize, Deserialize)]
pub struct TransactionResponse {
    /// Transaction data
    #[serde(flatten)]
    pub transaction: Transaction,
    /// Transaction size in bytes
    pub size: usize,
    /// Block height (if confirmed)
    pub block_height: Option<u64>,
    /// Block hash (if confirmed)
    pub block_hash: Option<Hash256>,
    /// Number of confirmations
    pub confirmations: Option<u64>,
    /// Transaction status
    pub status: TransactionStatus,
    /// Fee rate (satoshis per byte)
    pub fee_rate: Option<f64>,
}

/// Transaction status
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TransactionStatus {
    /// Transaction is in mempool
    Pending,
    /// Transaction is confirmed in a block
    Confirmed,
    /// Transaction failed validation
    Failed,
    /// Transaction was replaced
    Replaced,
}

impl TransactionResponse {
    /// Create a transaction response from a transaction
    pub fn from_transaction(
        transaction: Transaction,
        block_height: Option<u64>,
        block_hash: Option<Hash256>,
        current_height: u64,
    ) -> Self {
        let size = bincode::serialize(&transaction).map(|b| b.len()).unwrap_or(0);
        let confirmations = block_height.map(|h| current_height.saturating_sub(h));
        let status = if block_height.is_some() {
            TransactionStatus::Confirmed
        } else {
            TransactionStatus::Pending
        };
        
        let fee_rate = if size > 0 {
            let total_fee = transaction.fee.base_fee + transaction.fee.per_byte_fee * transaction.size.unwrap_or(0) as u64;
            Some(total_fee as f64 / size as f64)
        } else {
            None
        };
        
        Self {
            transaction,
            size,
            block_height,
            block_hash,
            confirmations,
            status,
            fee_rate,
        }
    }
}

/// Address information response
#[derive(Debug, Serialize, Deserialize)]
pub struct AddressInfoResponse {
    /// Address
    pub address: Address,
    /// Current balance
    pub balance: u64,
    /// Number of UTXOs
    pub utxo_count: usize,
    /// Total received
    pub total_received: u64,
    /// Total sent
    pub total_sent: u64,
    /// Transaction count
    pub transaction_count: u64,
    /// First seen (first transaction)
    pub first_seen: Option<DateTime<Utc>>,
    /// Last activity
    pub last_activity: Option<DateTime<Utc>>,
}

/// UTXO information response
#[derive(Debug, Serialize, Deserialize)]
pub struct UtxoInfoResponse {
    /// UTXO identifier
    pub utxo_id: String,
    /// Transaction hash
    pub tx_hash: Hash256,
    /// Output index
    pub output_index: u32,
    /// Amount in satoshis
    pub amount: u64,
    /// Recipient address
    pub address: Address,
    /// Block height where UTXO was created
    pub block_height: u64,
    /// Block hash where UTXO was created
    pub block_hash: Hash256,
    /// Number of confirmations
    pub confirmations: u64,
    /// Whether the UTXO is spent
    pub is_spent: bool,
    /// Script type
    pub script_type: String,
    /// UTXO age in blocks
    pub age: u64,
}

/// Mining information response
#[derive(Debug, Serialize, Deserialize)]
pub struct MiningInfoResponse {
    /// Current mining status
    pub is_mining: bool,
    /// Current block height
    pub block_height: u64,
    /// Current difficulty
    pub difficulty: u32,
    /// Target hash
    pub target: String,
    /// Network hash rate (hashes per second)
    pub network_hashrate: f64,
    /// Estimated time to next block (seconds)
    pub estimated_time: Option<u64>,
    /// Mining pool information
    pub pool_info: Option<PoolInfo>,
    /// Next difficulty adjustment
    pub next_difficulty_adjustment: DifficultyAdjustment,
}

/// Mining pool information
#[derive(Debug, Serialize, Deserialize)]
pub struct PoolInfo {
    /// Pool name
    pub name: String,
    /// Pool hash rate
    pub hashrate: f64,
    /// Pool share percentage
    pub share_percentage: f64,
    /// Last block found
    pub last_block: Option<u64>,
}

/// Difficulty adjustment information
#[derive(Debug, Serialize, Deserialize)]
pub struct DifficultyAdjustment {
    /// Blocks until next adjustment
    pub blocks_remaining: u64,
    /// Estimated time until adjustment
    pub estimated_time: u64,
    /// Current difficulty
    pub current_difficulty: u32,
    /// Estimated next difficulty
    pub estimated_next_difficulty: Option<u32>,
    /// Difficulty change percentage
    pub change_percentage: Option<f64>,
}

/// Network statistics response
#[derive(Debug, Serialize, Deserialize)]
pub struct NetworkStatsResponse {
    /// Total network hash rate
    pub total_hashrate: f64,
    /// Number of active miners
    pub active_miners: u32,
    /// Average block time (seconds)
    pub average_block_time: f64,
    /// Blocks in last 24 hours
    pub blocks_last_24h: u64,
    /// Network difficulty
    pub difficulty: u32,
    /// Total supply
    pub total_supply: u64,
    /// Circulating supply
    pub circulating_supply: u64,
    /// Market cap (if available)
    pub market_cap: Option<f64>,
}

/// Mempool information response
#[derive(Debug, Serialize, Deserialize)]
pub struct MempoolInfoResponse {
    /// Number of transactions in mempool
    pub transaction_count: u64,
    /// Total size of mempool (bytes)
    pub total_size: u64,
    /// Total fees in mempool
    pub total_fees: u64,
    /// Average fee rate
    pub average_fee_rate: f64,
    /// Minimum fee rate
    pub min_fee_rate: f64,
    /// Maximum fee rate
    pub max_fee_rate: f64,
    /// Fee rate percentiles
    pub fee_percentiles: FeePercentiles,
}

/// Fee rate percentiles
#[derive(Debug, Serialize, Deserialize)]
pub struct FeePercentiles {
    /// 10th percentile
    pub p10: f64,
    /// 25th percentile
    pub p25: f64,
    /// 50th percentile (median)
    pub p50: f64,
    /// 75th percentile
    pub p75: f64,
    /// 90th percentile
    pub p90: f64,
    /// 95th percentile
    pub p95: f64,
    /// 99th percentile
    pub p99: f64,
}

/// Search results response
#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResponse {
    /// Search query
    pub query: String,
    /// Search results
    pub results: Vec<SearchResult>,
    /// Total number of results
    pub total_results: u64,
    /// Search took (milliseconds)
    pub search_time_ms: u64,
}

/// Individual search result
#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResult {
    /// Result type
    pub result_type: SearchResultType,
    /// Result data
    pub data: serde_json::Value,
    /// Relevance score
    pub score: f64,
}

/// Search result types
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum SearchResultType {
    /// Block result
    Block,
    /// Transaction result
    Transaction,
    /// Address result
    Address,
    /// UTXO result
    Utxo,
}

/// Error response structure
#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    /// Error code
    pub error: ErrorInfo,
    /// Request metadata
    pub meta: ResponseMeta,
}

/// Error information
#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorInfo {
    /// Error code
    pub code: String,
    /// Error message
    pub message: String,
    /// Error details
    pub details: Option<serde_json::Value>,
    /// Error context
    pub context: Option<HashMap<String, serde_json::Value>>,
    /// Suggested actions
    pub suggestions: Option<Vec<String>>,
}

/// Create an error response
pub fn error_response(
    code: impl Into<String>,
    message: impl Into<String>,
) -> ErrorResponse {
    ErrorResponse {
        error: ErrorInfo {
            code: code.into(),
            message: message.into(),
            details: None,
            context: None,
            suggestions: None,
        },
        meta: ResponseMeta::new(),
    }
}

/// Create a detailed error response
pub fn detailed_error_response(
    code: impl Into<String>,
    message: impl Into<String>,
    details: Option<serde_json::Value>,
    context: Option<HashMap<String, serde_json::Value>>,
    suggestions: Option<Vec<String>>,
) -> ErrorResponse {
    ErrorResponse {
        error: ErrorInfo {
            code: code.into(),
            message: message.into(),
            details,
            context,
            suggestions,
        },
        meta: ResponseMeta::new(),
    }
}

/// Validation error response
#[derive(Debug, Serialize, Deserialize)]
pub struct ValidationErrorResponse {
    /// Field-specific errors
    pub field_errors: HashMap<String, Vec<String>>,
    /// General validation errors
    pub general_errors: Vec<String>,
    /// Request metadata
    pub meta: ResponseMeta,
}

/// Create a validation error response
pub fn validation_error_response(
    field_errors: HashMap<String, Vec<String>>,
    general_errors: Vec<String>,
) -> ValidationErrorResponse {
    ValidationErrorResponse {
        field_errors,
        general_errors,
        meta: ResponseMeta::new(),
    }
}

/// Health check response
#[derive(Debug, Serialize, Deserialize)]
pub struct HealthCheckResponse {
    /// Service status
    pub status: HealthStatus,
    /// Service version
    pub version: String,
    /// Uptime in seconds
    pub uptime: u64,
    /// Component health checks
    pub components: HashMap<String, ComponentHealth>,
    /// System information
    pub system: SystemInfo,
}

/// Health status
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum HealthStatus {
    /// All systems operational
    Healthy,
    /// Some non-critical issues
    Degraded,
    /// Critical issues present
    Unhealthy,
}

/// Component health information
#[derive(Debug, Serialize, Deserialize)]
pub struct ComponentHealth {
    /// Component status
    pub status: HealthStatus,
    /// Status message
    pub message: Option<String>,
    /// Last check time
    pub last_check: DateTime<Utc>,
    /// Response time (milliseconds)
    pub response_time_ms: Option<u64>,
}

/// System information
#[derive(Debug, Serialize, Deserialize)]
pub struct SystemInfo {
    /// Memory usage (bytes)
    pub memory_usage: u64,
    /// CPU usage percentage
    pub cpu_usage: f64,
    /// Disk usage (bytes)
    pub disk_usage: u64,
    /// Network connections
    pub active_connections: u32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{Block, BlockHeader, Transaction};
    use crate::crypto::Hash256;
    
    #[test]
    fn test_response_meta_creation() {
        let meta = ResponseMeta::new()
            .with_request_id("test-123".to_string())
            .with_processing_time(100)
            .with_extra("custom".to_string(), serde_json::json!("value"));
        
        assert_eq!(meta.request_id, Some("test-123".to_string()));
        assert_eq!(meta.processing_time_ms, Some(100));
        assert!(meta.extra.is_some());
    }
    
    #[test]
    fn test_success_response() {
        let data = "test data";
        let response = success(data);
        
        assert_eq!(response.data, "test data");
        assert_eq!(response.meta.version, "1.0.0");
    }
    
    #[test]
    fn test_error_response() {
        let error = error_response("TEST_ERROR", "Test error message");
        
        assert_eq!(error.error.code, "TEST_ERROR");
        assert_eq!(error.error.message, "Test error message");
        assert!(error.error.details.is_none());
    }
    
    #[test]
    fn test_transaction_status_serialization() {
        let status = TransactionStatus::Confirmed;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, "\"confirmed\"");
        
        let deserialized: TransactionStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, TransactionStatus::Confirmed);
    }
    
    #[test]
    fn test_health_status_serialization() {
        let status = HealthStatus::Healthy;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, "\"healthy\"");
        
        let deserialized: HealthStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, HealthStatus::Healthy);
    }
}