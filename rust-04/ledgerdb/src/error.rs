use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;
use serde::{Serialize, Deserialize};
use std::fmt;

// Core error types
#[derive(Debug, thiserror::Error)]
pub enum LedgerError {
    #[error("Validation error: {0}")]
    Validation(String),
    #[error("IO error: {0}")]
    Io(String),
    #[error("Serialization error: {0}")]
    Serialization(String),
    #[error("Network error: {0}")]
    Network(String),
    #[error("Configuration error: {0}")]
    Config(String),
    #[error("Internal error: {0}")]
    Internal(String),
    #[error("Database error: {0}")]
    Database(String),
    #[error("Parse error: {0}")]
    Parse(String),
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Invalid input: {0}")]
    InvalidInput(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValidationError {
    InvalidHash(String),
    InvalidSignature(String),
    InvalidTimestamp(String),
    InvalidDifficulty(String),
    InvalidMerkleRoot(String),
    InvalidProofOfWork(String),
    InvalidTransactionCount(String),
    MiningTimeout,
    InvalidNonce(String),
    InvalidPreviousHash(String),
    InvalidIndex(String),
    ArithmeticOverflow(String),
    OutputNotFound(String),
    InsufficientFunds(String),
    InvalidUtxoId(String),
    UtxoNotFound(String),
    EmptyOutputs,
    InvalidCoinbase(String),
    OutputAlreadySpent(String),
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::InvalidHash(msg) => write!(f, "Invalid hash: {}", msg),
            ValidationError::InvalidSignature(msg) => write!(f, "Invalid signature: {}", msg),
            ValidationError::InvalidTimestamp(msg) => write!(f, "Invalid timestamp: {}", msg),
            ValidationError::InvalidDifficulty(msg) => write!(f, "Invalid difficulty: {}", msg),
            ValidationError::InvalidMerkleRoot(msg) => write!(f, "Invalid merkle root: {}", msg),
            ValidationError::InvalidProofOfWork(msg) => write!(f, "Invalid proof of work: {}", msg),
            ValidationError::InvalidTransactionCount(msg) => write!(f, "Invalid transaction count: {}", msg),
            ValidationError::MiningTimeout => write!(f, "Mining timeout"),
            ValidationError::InvalidNonce(msg) => write!(f, "Invalid nonce: {}", msg),
            ValidationError::InvalidPreviousHash(msg) => write!(f, "Invalid previous hash: {}", msg),
            ValidationError::InvalidIndex(msg) => write!(f, "Invalid index: {}", msg),
            ValidationError::ArithmeticOverflow(msg) => write!(f, "Arithmetic overflow: {}", msg),
            ValidationError::OutputNotFound(msg) => write!(f, "Output not found: {}", msg),
            ValidationError::InsufficientFunds(msg) => write!(f, "Insufficient funds: {}", msg),
            ValidationError::InvalidUtxoId(msg) => write!(f, "Invalid UTXO ID: {}", msg),
            ValidationError::UtxoNotFound(msg) => write!(f, "UTXO not found: {}", msg),
            ValidationError::EmptyOutputs => write!(f, "Empty outputs"),
            ValidationError::InvalidCoinbase(msg) => write!(f, "Invalid coinbase: {}", msg),
            ValidationError::OutputAlreadySpent(msg) => write!(f, "Output already spent: {}", msg),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum BlockchainError {
    #[error("Block not found: {0}")]
    BlockNotFound(String),
    #[error("Transaction not found: {0}")]
    TransactionNotFound(String),
    #[error("Invalid chain: {0}")]
    InvalidChain(String),
    #[error("Consensus error: {0}")]
    ConsensusError(String),
    #[error("Storage error: {0}")]
    StorageError(String),
    #[error("Invalid genesis block")]
    InvalidGenesisBlock,
}

#[derive(Debug, thiserror::Error)]
pub enum CryptoError {
    #[error("Key generation error: {0}")]
    KeyGeneration(String),
    #[error("Signature error: {0}")]
    Signature(String),
    #[error("Hash error: {0}")]
    Hash(String),
    #[error("Encryption error: {0}")]
    Encryption(String),
    #[error("Empty merkle tree")]
    EmptyMerkleTree,
    #[error("Leaf not found: index {index}")]
    LeafNotFound { index: usize },
    #[error("Serialization error")]
    SerializationError { source: bincode::Error },
    #[error("Invalid merkle proof")]
    InvalidMerkleProof,
    #[error("Invalid key format: {0}")]
    InvalidKeyFormat(String),
    #[error("Invalid hex string: {hex_str}")]
    InvalidHexString { hex_str: String },
    #[error("Invalid format: {0}")]
    InvalidFormat(String),
    #[error("Key not found: {hash}")]
    KeyNotFound { hash: String },
    #[error("Invalid leaf index: {index}")]
    InvalidLeafIndex { index: usize },
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Missing configuration: {0}")]
    Missing(String),
    #[error("Invalid configuration: {0}")]
    Invalid(String),
    #[error("Parse error: {0}")]
    Parse(String),
}

// Convert between error types
impl From<ValidationError> for LedgerError {
    fn from(err: ValidationError) -> Self {
        LedgerError::Validation(err.to_string())
    }
}

impl From<BlockchainError> for LedgerError {
    fn from(err: BlockchainError) -> Self {
        LedgerError::Validation(err.to_string())
    }
}

impl From<CryptoError> for LedgerError {
    fn from(err: CryptoError) -> Self {
        LedgerError::Validation(err.to_string())
    }
}

impl From<ValidationError> for BlockchainError {
    fn from(err: ValidationError) -> Self {
        BlockchainError::InvalidChain(err.to_string())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("Internal Server Error: {0}")]
    InternalServerError(String),
    #[error("Bad Request: {0}")]
    BadRequest(String),
    #[error("Unauthorized: {0}")]
    Unauthorized(String),
    #[error("Not Found: {0}")]
    NotFound(String),
    #[error("Conflict: {0}")]
    Conflict(String),
    #[error("Service Unavailable: {0}")]
    ServiceUnavailable(String),
    #[error("Validation Error: {0}")]
    ValidationError(String),
    #[error("Blockchain Error: {0}")]
    BlockchainError(String),
    #[error("Storage Error: {0}")]
    StorageError(String),
    #[error("Mining Error: {0}")]
    MiningError(String),
    #[error("Network Error: {0}")]
    NetworkError(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            ApiError::InternalServerError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            ApiError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            ApiError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg),
            ApiError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            ApiError::Conflict(msg) => (StatusCode::CONFLICT, msg),
            ApiError::ServiceUnavailable(msg) => (StatusCode::SERVICE_UNAVAILABLE, msg),
            ApiError::ValidationError(msg) => (StatusCode::BAD_REQUEST, msg),
            ApiError::BlockchainError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            ApiError::StorageError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            ApiError::MiningError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            ApiError::NetworkError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        };

        let body = json!({ "error": error_message });
        (status, Json(body)).into_response()
    }
}

// Result type aliases
pub type Result<T> = std::result::Result<T, LedgerError>;
pub type ApiResult<T> = std::result::Result<T, ApiError>;