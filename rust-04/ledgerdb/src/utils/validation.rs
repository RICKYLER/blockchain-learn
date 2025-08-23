//! Validation utilities for the LedgerDB blockchain.
//!
//! This module provides validation functions for various blockchain
//! data types and structures.

use crate::crypto::Hash256;
use crate::error::LedgerError;
use std::collections::HashSet;

/// Validate a hash string (hex format)
pub fn validate_hash_string(hash_str: &str) -> Result<(), LedgerError> {
    // Remove 0x prefix if present
    let hash_str = hash_str.strip_prefix("0x").unwrap_or(hash_str);
    
    // Check length (64 characters for 32 bytes)
    if hash_str.len() != 64 {
        return Err(LedgerError::Validation(format!(
            "Invalid hash length: expected 64, got {}",
            hash_str.len()
        )));
    }
    
    // Check if all characters are valid hex
    if !hash_str.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(LedgerError::Validation(
            "Hash contains invalid hex characters".to_string()
        ));
    }
    
    Ok(())
}

/// Validate an address string
pub fn validate_address(address: &str) -> Result<(), LedgerError> {
    if address.is_empty() {
        return Err(LedgerError::Validation("Address cannot be empty".to_string()));
    }
    
    // Basic length check (Bitcoin addresses are typically 26-35 characters)
    if address.len() < 26 || address.len() > 35 {
        return Err(LedgerError::Validation(format!(
            "Invalid address length: expected 26-35, got {}",
            address.len()
        )));
    }
    
    // Check for valid base58 characters (simplified)
    let valid_chars = "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";
    if !address.chars().all(|c| valid_chars.contains(c)) {
        return Err(LedgerError::Validation(
            "Address contains invalid characters".to_string()
        ));
    }
    
    Ok(())
}

/// Validate a transaction amount
pub fn validate_amount(amount: u64) -> Result<(), LedgerError> {
    const MAX_MONEY: u64 = 21_000_000 * 100_000_000; // 21M BTC in satoshis
    const DUST_THRESHOLD: u64 = 546; // Minimum output value
    
    if amount == 0 {
        return Err(LedgerError::Validation("Amount cannot be zero".to_string()));
    }
    
    if amount < DUST_THRESHOLD {
        return Err(LedgerError::Validation(format!(
            "Amount {} is below dust threshold {}",
            amount, DUST_THRESHOLD
        )));
    }
    
    if amount > MAX_MONEY {
        return Err(LedgerError::Validation(format!(
            "Amount {} exceeds maximum money supply {}",
            amount, MAX_MONEY
        )));
    }
    
    Ok(())
}

/// Validate a transaction fee
pub fn validate_fee(fee: u64, transaction_size: usize) -> Result<(), LedgerError> {
    const MIN_FEE: u64 = 1000; // Minimum fee in satoshis
    const MAX_FEE_RATE: u64 = 1000; // Maximum fee rate (sat/byte)
    
    if fee < MIN_FEE {
        return Err(LedgerError::Validation(format!(
            "Fee {} is below minimum {}",
            fee, MIN_FEE
        )));
    }
    
    if transaction_size > 0 {
        let fee_rate = fee / transaction_size as u64;
        if fee_rate > MAX_FEE_RATE {
            return Err(LedgerError::Validation(format!(
                "Fee rate {} sat/byte exceeds maximum {}",
                fee_rate, MAX_FEE_RATE
            )));
        }
    }
    
    Ok(())
}

/// Validate a block height
pub fn validate_block_height(height: u64, current_height: u64) -> Result<(), LedgerError> {
    if height > current_height + 1 {
        return Err(LedgerError::Validation(format!(
            "Block height {} is too far in the future (current: {})",
            height, current_height
        )));
    }
    
    Ok(())
}

/// Validate a timestamp
pub fn validate_timestamp(timestamp: u64) -> Result<(), LedgerError> {
    use std::time::{SystemTime, UNIX_EPOCH};
    
    const MAX_FUTURE_TIME: u64 = 2 * 60 * 60; // 2 hours
    const MIN_TIMESTAMP: u64 = 1640995200; // 2022-01-01 00:00:00 UTC
    
    if timestamp < MIN_TIMESTAMP {
        return Err(LedgerError::Validation(format!(
            "Timestamp {} is too far in the past",
            timestamp
        )));
    }
    
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    
    if timestamp > now + MAX_FUTURE_TIME {
        return Err(LedgerError::Validation(format!(
            "Timestamp {} is too far in the future",
            timestamp
        )));
    }
    
    Ok(())
}

/// Validate a difficulty value
pub fn validate_difficulty(difficulty: u32) -> Result<(), LedgerError> {
    const MIN_DIFFICULTY: u32 = 1;
    const MAX_DIFFICULTY: u32 = u32::MAX;
    
    if difficulty < MIN_DIFFICULTY {
        return Err(LedgerError::Validation(format!(
            "Difficulty {} is below minimum {}",
            difficulty, MIN_DIFFICULTY
        )));
    }
    
    if difficulty > MAX_DIFFICULTY {
        return Err(LedgerError::Validation(format!(
            "Difficulty {} exceeds maximum {}",
            difficulty, MAX_DIFFICULTY
        )));
    }
    
    Ok(())
}

/// Validate a nonce value
pub fn validate_nonce(nonce: u64) -> Result<(), LedgerError> {
    // Nonce can be any u64 value, so this is mostly a placeholder
    // In practice, you might want to check for specific patterns or ranges
    Ok(())
}

/// Validate transaction inputs for double spending
pub fn validate_no_double_spending(transaction_hashes: &[Hash256]) -> Result<(), LedgerError> {
    let mut seen = HashSet::new();
    
    for hash in transaction_hashes {
        if !seen.insert(hash) {
            return Err(LedgerError::Validation(format!(
                "Double spending detected: transaction {} appears multiple times",
                hex::encode(hash.as_bytes())
            )));
        }
    }
    
    Ok(())
}

/// Validate that a collection is not empty
pub fn validate_not_empty<T>(collection: &[T], name: &str) -> Result<(), LedgerError> {
    if collection.is_empty() {
        return Err(LedgerError::Validation(format!(
            "{} cannot be empty",
            name
        )));
    }
    Ok(())
}

/// Validate collection size limits
pub fn validate_collection_size<T>(
    collection: &[T],
    name: &str,
    max_size: usize,
) -> Result<(), LedgerError> {
    if collection.len() > max_size {
        return Err(LedgerError::Validation(format!(
            "{} size {} exceeds maximum {}",
            name,
            collection.len(),
            max_size
        )));
    }
    Ok(())
}

/// Validate string length
pub fn validate_string_length(
    string: &str,
    name: &str,
    min_len: usize,
    max_len: usize,
) -> Result<(), LedgerError> {
    let len = string.len();
    
    if len < min_len {
        return Err(LedgerError::Validation(format!(
            "{} length {} is below minimum {}",
            name, len, min_len
        )));
    }
    
    if len > max_len {
        return Err(LedgerError::Validation(format!(
            "{} length {} exceeds maximum {}",
            name, len, max_len
        )));
    }
    
    Ok(())
}

/// Validate numeric range
pub fn validate_range<T>(
    value: T,
    name: &str,
    min: T,
    max: T,
) -> Result<(), LedgerError>
where
    T: PartialOrd + std::fmt::Display + Copy,
{
    if value < min {
        return Err(LedgerError::Validation(format!(
            "{} {} is below minimum {}",
            name, value, min
        )));
    }
    
    if value > max {
        return Err(LedgerError::Validation(format!(
            "{} {} exceeds maximum {}",
            name, value, max
        )));
    }
    
    Ok(())
}

/// Validate that a value is positive
pub fn validate_positive<T>(value: T, name: &str) -> Result<(), LedgerError>
where
    T: PartialOrd + std::fmt::Display + Default,
{
    if value <= T::default() {
        return Err(LedgerError::Validation(format!(
            "{} must be positive, got {}",
            name, value
        )));
    }
    Ok(())
}

/// Validate email format (basic)
pub fn validate_email(email: &str) -> Result<(), LedgerError> {
    if email.is_empty() {
        return Err(LedgerError::Validation("Email cannot be empty".to_string()));
    }
    
    if !email.contains('@') {
        return Err(LedgerError::Validation("Email must contain @".to_string()));
    }
    
    let parts: Vec<&str> = email.split('@').collect();
    if parts.len() != 2 {
        return Err(LedgerError::Validation("Email format is invalid".to_string()));
    }
    
    let (local, domain) = (parts[0], parts[1]);
    
    if local.is_empty() {
        return Err(LedgerError::Validation("Email local part cannot be empty".to_string()));
    }
    
    if domain.is_empty() {
        return Err(LedgerError::Validation("Email domain cannot be empty".to_string()));
    }
    
    if !domain.contains('.') {
        return Err(LedgerError::Validation("Email domain must contain a dot".to_string()));
    }
    
    Ok(())
}

/// Validate URL format (basic)
pub fn validate_url(url: &str) -> Result<(), LedgerError> {
    if url.is_empty() {
        return Err(LedgerError::Validation("URL cannot be empty".to_string()));
    }
    
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return Err(LedgerError::Validation(
            "URL must start with http:// or https://".to_string()
        ));
    }
    
    Ok(())
}

/// Validate IP address format (basic IPv4)
pub fn validate_ipv4(ip: &str) -> Result<(), LedgerError> {
    let parts: Vec<&str> = ip.split('.').collect();
    
    if parts.len() != 4 {
        return Err(LedgerError::Validation(
            "IPv4 address must have 4 parts separated by dots".to_string()
        ));
    }
    
    for part in parts {
        let num: u8 = part.parse().map_err(|_| {
            LedgerError::Validation(format!("Invalid IPv4 part: {}", part))
        })?;
        
        // u8 automatically ensures 0-255 range
    }
    
    Ok(())
}

/// Validate port number
pub fn validate_port(port: u16) -> Result<(), LedgerError> {
    if port == 0 {
        return Err(LedgerError::Validation("Port cannot be 0".to_string()));
    }
    
    // Well-known ports (1-1023) might be restricted in some contexts
    // but we'll allow them for now
    
    Ok(())
}

/// Comprehensive validation result
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl ValidationResult {
    /// Create a new validation result
    pub fn new() -> Self {
        Self {
            is_valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }
    
    /// Add an error
    pub fn add_error(&mut self, error: String) {
        self.errors.push(error);
        self.is_valid = false;
    }
    
    /// Add a warning
    pub fn add_warning(&mut self, warning: String) {
        self.warnings.push(warning);
    }
    
    /// Check if validation passed
    pub fn is_valid(&self) -> bool {
        self.is_valid
    }
    
    /// Get all errors
    pub fn errors(&self) -> &[String] {
        &self.errors
    }
    
    /// Get all warnings
    pub fn warnings(&self) -> &[String] {
        &self.warnings
    }
    
    /// Merge another validation result
    pub fn merge(&mut self, other: ValidationResult) {
        self.errors.extend(other.errors);
        self.warnings.extend(other.warnings);
        self.is_valid = self.is_valid && other.is_valid;
    }
}

impl Default for ValidationResult {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_validate_hash_string() {
        // Valid hash
        assert!(validate_hash_string("0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef").is_ok());
        
        // Valid hash with 0x prefix
        assert!(validate_hash_string("0x0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef").is_ok());
        
        // Invalid length
        assert!(validate_hash_string("0123456789abcdef").is_err());
        
        // Invalid characters
        assert!(validate_hash_string("0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdefg").is_err());
    }
    
    #[test]
    fn test_validate_address() {
        // Valid address
        assert!(validate_address("1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa").is_ok());
        
        // Empty address
        assert!(validate_address("").is_err());
        
        // Too short
        assert!(validate_address("1A1zP1eP5QGefi2DMPTfTL5SL").is_err());
        
        // Too long
        assert!(validate_address("1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa1234567890").is_err());
    }
    
    #[test]
    fn test_validate_amount() {
        // Valid amount
        assert!(validate_amount(100_000_000).is_ok()); // 1 BTC
        
        // Zero amount
        assert!(validate_amount(0).is_err());
        
        // Below dust threshold
        assert!(validate_amount(500).is_err());
        
        // Above maximum money supply
        assert!(validate_amount(22_000_000 * 100_000_000).is_err());
    }
    
    #[test]
    fn test_validate_fee() {
        // Valid fee
        assert!(validate_fee(5000, 250).is_ok());
        
        // Below minimum
        assert!(validate_fee(500, 250).is_err());
        
        // Too high fee rate
        assert!(validate_fee(250_000, 250).is_err());
    }
    
    #[test]
    fn test_validate_timestamp() {
        use std::time::{SystemTime, UNIX_EPOCH};
        
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        // Current time should be valid
        assert!(validate_timestamp(now).is_ok());
        
        // Too far in the past
        assert!(validate_timestamp(1000000000).is_err());
        
        // Too far in the future
        assert!(validate_timestamp(now + 10 * 60 * 60).is_err());
    }
    
    #[test]
    fn test_validate_email() {
        assert!(validate_email("user@example.com").is_ok());
        assert!(validate_email("").is_err());
        assert!(validate_email("invalid-email").is_err());
        assert!(validate_email("@example.com").is_err());
        assert!(validate_email("user@").is_err());
        assert!(validate_email("user@example").is_err());
    }
    
    #[test]
    fn test_validate_url() {
        assert!(validate_url("https://example.com").is_ok());
        assert!(validate_url("http://example.com").is_ok());
        assert!(validate_url("").is_err());
        assert!(validate_url("ftp://example.com").is_err());
        assert!(validate_url("example.com").is_err());
    }
    
    #[test]
    fn test_validate_ipv4() {
        assert!(validate_ipv4("192.168.1.1").is_ok());
        assert!(validate_ipv4("0.0.0.0").is_ok());
        assert!(validate_ipv4("255.255.255.255").is_ok());
        assert!(validate_ipv4("192.168.1").is_err());
        assert!(validate_ipv4("192.168.1.256").is_err());
        assert!(validate_ipv4("192.168.1.a").is_err());
    }
    
    #[test]
    fn test_validation_result() {
        let mut result = ValidationResult::new();
        assert!(result.is_valid());
        
        result.add_warning("This is a warning".to_string());
        assert!(result.is_valid());
        
        result.add_error("This is an error".to_string());
        assert!(!result.is_valid());
        
        assert_eq!(result.errors().len(), 1);
        assert_eq!(result.warnings().len(), 1);
    }
}