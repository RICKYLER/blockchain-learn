//! Configuration management for the LedgerDB blockchain system.
//!
//! This module handles all configuration aspects including environment variables,
//! configuration files, and runtime settings with proper validation and defaults.

use crate::error::{ConfigError, Result};
use serde::{Deserialize, Serialize};
use std::env;
use std::path::PathBuf;
use tracing::Level;

/// Main application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Server configuration
    pub server: ServerConfig,
    /// Blockchain configuration
    pub blockchain: BlockchainConfig,
    /// Storage configuration
    pub storage: StorageConfig,
    /// Mining configuration
    pub mining: MiningConfig,
    /// Logging configuration
    pub logging: LoggingConfig,
    /// API configuration
    pub api: ApiConfig,
}

/// Server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// Server host address
    pub host: String,
    /// Server port
    pub port: u16,
    /// Maximum number of concurrent connections
    pub max_connections: usize,
    /// Request timeout in seconds
    pub request_timeout: u64,
    /// Enable CORS
    pub enable_cors: bool,
    /// Static files directory
    pub static_dir: Option<PathBuf>,
}

/// Blockchain configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockchainConfig {
    /// Initial mining difficulty
    pub initial_difficulty: u32,
    /// Mining reward in smallest units
    pub mining_reward: u64,
    /// Maximum transactions per block
    pub max_transactions_per_block: usize,
    /// Difficulty adjustment interval in blocks
    pub difficulty_adjustment_interval: u64,
    /// Target block time in seconds
    pub target_block_time: u64,
    /// Maximum block size in bytes
    pub max_block_size: usize,
    /// Transaction fee per byte
    pub transaction_fee_per_byte: u64,
}

/// Storage configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    /// Database file path
    pub db_path: PathBuf,
    /// Enable database compression
    pub enable_compression: bool,
    /// Cache size in MB
    pub cache_size_mb: usize,
    /// Backup directory
    pub backup_dir: Option<PathBuf>,
    /// Auto-backup interval in hours
    pub auto_backup_interval_hours: Option<u64>,
    /// Maximum number of backup files to keep
    pub max_backup_files: usize,
}

/// Mining configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MiningConfig {
    /// Enable mining
    pub enabled: bool,
    /// Number of mining threads
    pub threads: usize,
    /// Mining timeout in seconds
    pub timeout_seconds: u64,
    /// Progress update interval in milliseconds
    pub progress_update_interval_ms: u64,
    /// Maximum mining attempts before giving up
    pub max_attempts: Option<u64>,
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Log level
    pub level: String,
    /// Log format (json, pretty, compact)
    pub format: String,
    /// Log file path (None for stdout)
    pub file: Option<PathBuf>,
    /// Enable colored output
    pub colored: bool,
    /// Include timestamps
    pub timestamps: bool,
    /// Include thread IDs
    pub thread_ids: bool,
}

/// API configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    /// API rate limiting (requests per minute)
    pub rate_limit: Option<u32>,
    /// Enable API authentication
    pub enable_auth: bool,
    /// API key for authentication
    pub api_key: Option<String>,
    /// Maximum request body size in bytes
    pub max_request_size: usize,
    /// Enable request/response logging
    pub enable_request_logging: bool,
    /// WebSocket configuration
    pub websocket: WebSocketConfig,
}

/// WebSocket configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSocketConfig {
    /// Maximum number of WebSocket connections
    pub max_connections: usize,
    /// Ping interval in seconds
    pub ping_interval: u64,
    /// Connection timeout in seconds
    pub connection_timeout: u64,
    /// Message buffer size
    pub message_buffer_size: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server: ServerConfig::default(),
            blockchain: BlockchainConfig::default(),
            storage: StorageConfig::default(),
            mining: MiningConfig::default(),
            logging: LoggingConfig::default(),
            api: ApiConfig::default(),
        }
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: 3000,
            max_connections: 1000,
            request_timeout: 30,
            enable_cors: true,
            static_dir: Some(PathBuf::from("frontend/dist")),
        }
    }
}

impl Default for BlockchainConfig {
    fn default() -> Self {
        Self {
            initial_difficulty: 2,
            mining_reward: 50_000_000, // 50 tokens with 6 decimal places
            max_transactions_per_block: 100,
            difficulty_adjustment_interval: 10,
            target_block_time: 60, // 1 minute
            max_block_size: 1_048_576, // 1 MB
            transaction_fee_per_byte: 1,
        }
    }
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            db_path: PathBuf::from("ledgerdb.db"),
            enable_compression: true,
            cache_size_mb: 64,
            backup_dir: Some(PathBuf::from("backups")),
            auto_backup_interval_hours: Some(24),
            max_backup_files: 7,
        }
    }
}

impl Default for MiningConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            threads: num_cpus::get().max(1),
            timeout_seconds: 300, // 5 minutes
            progress_update_interval_ms: 1000,
            max_attempts: None,
        }
    }
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            format: "pretty".to_string(),
            file: None,
            colored: true,
            timestamps: true,
            thread_ids: false,
        }
    }
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            rate_limit: Some(100), // 100 requests per minute
            enable_auth: false,
            api_key: None,
            max_request_size: 1_048_576, // 1 MB
            enable_request_logging: true,
            websocket: WebSocketConfig::default(),
        }
    }
}

impl Default for WebSocketConfig {
    fn default() -> Self {
        Self {
            max_connections: 100,
            ping_interval: 30,
            connection_timeout: 60,
            message_buffer_size: 1024,
        }
    }
}

impl Config {
    /// Load configuration from environment variables and defaults
    pub fn from_env() -> Result<Self> {
        let mut config = Self::default();

        // Server configuration
        if let Ok(host) = env::var("LEDGER_HOST") {
            config.server.host = host;
        }
        if let Ok(port) = env::var("PORT").or_else(|_| env::var("LEDGER_PORT")) {
            config.server.port = port.parse().map_err(|_| ConfigError::InvalidConfig {
                field: "port".to_string(),
            })?;
        }
        if let Ok(max_conn) = env::var("LEDGER_MAX_CONNECTIONS") {
            config.server.max_connections = max_conn.parse().map_err(|_| {
                ConfigError::InvalidConfig {
                    field: "max_connections".to_string(),
                }
            })?;
        }

        // Blockchain configuration
        if let Ok(difficulty) = env::var("LEDGER_INITIAL_DIFFICULTY") {
            config.blockchain.initial_difficulty = difficulty.parse().map_err(|_| {
                ConfigError::InvalidConfig {
                    field: "initial_difficulty".to_string(),
                }
            })?;
        }
        if let Ok(reward) = env::var("LEDGER_MINING_REWARD") {
            config.blockchain.mining_reward = reward.parse().map_err(|_| {
                ConfigError::InvalidConfig {
                    field: "mining_reward".to_string(),
                }
            })?;
        }

        // Storage configuration
        if let Ok(db_path) = env::var("LEDGER_DB_PATH") {
            config.storage.db_path = PathBuf::from(db_path);
        }
        if let Ok(cache_size) = env::var("LEDGER_CACHE_SIZE_MB") {
            config.storage.cache_size_mb = cache_size.parse().map_err(|_| {
                ConfigError::InvalidConfig {
                    field: "cache_size_mb".to_string(),
                }
            })?;
        }

        // Mining configuration
        if let Ok(enabled) = env::var("LEDGER_MINING_ENABLED") {
            config.mining.enabled = enabled.parse().map_err(|_| ConfigError::InvalidConfig {
                field: "mining_enabled".to_string(),
            })?;
        }
        if let Ok(threads) = env::var("LEDGER_MINING_THREADS") {
            config.mining.threads = threads.parse().map_err(|_| ConfigError::InvalidConfig {
                field: "mining_threads".to_string(),
            })?;
        }

        // Logging configuration
        if let Ok(level) = env::var("LEDGER_LOG_LEVEL") {
            config.logging.level = level;
        }
        if let Ok(format) = env::var("LEDGER_LOG_FORMAT") {
            config.logging.format = format;
        }

        // API configuration
        if let Ok(api_key) = env::var("LEDGER_API_KEY") {
            config.api.api_key = Some(api_key);
            config.api.enable_auth = true;
        }
        if let Ok(rate_limit) = env::var("LEDGER_RATE_LIMIT") {
            config.api.rate_limit = Some(rate_limit.parse().map_err(|_| {
                ConfigError::InvalidConfig {
                    field: "rate_limit".to_string(),
                }
            })?);
        }

        config.validate()?;
        Ok(config)
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<()> {
        // Validate server config
        if self.server.port == 0 {
            return Err(ConfigError::ValueOutOfRange {
                field: "server.port".to_string(),
                value: "0".to_string(),
                range: "1-65535".to_string(),
            }
            .into());
        }

        // Validate blockchain config
        if self.blockchain.initial_difficulty == 0 {
            return Err(ConfigError::ValueOutOfRange {
                field: "blockchain.initial_difficulty".to_string(),
                value: "0".to_string(),
                range: "1+".to_string(),
            }
            .into());
        }

        if self.blockchain.max_transactions_per_block == 0 {
            return Err(ConfigError::ValueOutOfRange {
                field: "blockchain.max_transactions_per_block".to_string(),
                value: "0".to_string(),
                range: "1+".to_string(),
            }
            .into());
        }

        // Validate mining config
        if self.mining.threads == 0 {
            return Err(ConfigError::ValueOutOfRange {
                field: "mining.threads".to_string(),
                value: "0".to_string(),
                range: "1+".to_string(),
            }
            .into());
        }

        // Validate logging level
        match self.logging.level.to_lowercase().as_str() {
            "trace" | "debug" | "info" | "warn" | "error" => {}
            _ => {
                return Err(ConfigError::InvalidConfig {
                    field: format!("logging.level: {}", self.logging.level),
                }
                .into());
            }
        }

        Ok(())
    }

    /// Get the tracing level from the logging configuration
    pub fn tracing_level(&self) -> Level {
        match self.logging.level.to_lowercase().as_str() {
            "trace" => Level::TRACE,
            "debug" => Level::DEBUG,
            "info" => Level::INFO,
            "warn" => Level::WARN,
            "error" => Level::ERROR,
            _ => Level::INFO,
        }
    }

    /// Get the server address as a string
    pub fn server_address(&self) -> String {
        format!("{}:{}", self.server.host, self.server.port)
    }

    /// Check if development mode is enabled
    pub fn is_development(&self) -> bool {
        env::var("LEDGER_ENV").unwrap_or_default() == "development"
    }

    /// Check if production mode is enabled
    pub fn is_production(&self) -> bool {
        env::var("LEDGER_ENV").unwrap_or_default() == "production"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.server.port, 3000);
        assert_eq!(config.blockchain.initial_difficulty, 2);
        assert!(config.mining.enabled);
    }

    #[test]
    fn test_config_validation() {
        let mut config = Config::default();
        assert!(config.validate().is_ok());

        config.server.port = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_tracing_level() {
        let mut config = Config::default();
        config.logging.level = "debug".to_string();
        assert_eq!(config.tracing_level(), Level::DEBUG);
    }

    #[test]
    fn test_server_address() {
        let config = Config::default();
        assert_eq!(config.server_address(), "0.0.0.0:3000");
    }

    #[test]
    fn test_env_override() {
        env::set_var("PORT", "8080");
        let config = Config::from_env().unwrap();
        assert_eq!(config.server.port, 8080);
        env::remove_var("PORT");
    }
}