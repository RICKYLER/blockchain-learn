//! Hashing utilities for the LedgerDB blockchain system.
//!
//! This module provides various hashing functions and utilities used throughout
//! the blockchain system for data integrity and cryptographic operations.

use crate::crypto::Hash256;
use sha2::{Digest, Sha256};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// Trait for types that can be hashed
pub trait Hashable {
    /// Compute the cryptographic hash of this object
    fn hash(&self) -> Hash256;
    
    /// Compute the hash with additional context
    fn hash_with_context(&self, context: &[u8]) -> Hash256 {
        let self_hash = self.hash();
        let mut hasher = Sha256::new();
        hasher.update(context);
        hasher.update(self_hash.as_slice());
        Hash256::new(hasher.finalize().into())
    }
}

/// Hash builder for incremental hashing
pub struct HashBuilder {
    hasher: Sha256,
}

impl HashBuilder {
    /// Create a new hash builder
    pub fn new() -> Self {
        Self {
            hasher: Sha256::new(),
        }
    }

    /// Add data to the hash
    pub fn update(&mut self, data: &[u8]) -> &mut Self {
        self.hasher.update(data);
        self
    }

    /// Add a string to the hash
    pub fn update_str(&mut self, data: &str) -> &mut Self {
        self.update(data.as_bytes())
    }

    /// Add a number to the hash
    pub fn update_u64(&mut self, value: u64) -> &mut Self {
        self.update(&value.to_le_bytes())
    }

    /// Add a number to the hash
    pub fn update_u32(&mut self, value: u32) -> &mut Self {
        self.update(&value.to_le_bytes())
    }

    /// Add a hash to the hash
    pub fn update_hash(&mut self, hash: &Hash256) -> &mut Self {
        self.update(hash.as_slice())
    }

    /// Finalize the hash and return the result
    pub fn finalize(self) -> Hash256 {
        Hash256::new(self.hasher.finalize().into())
    }

    /// Reset the hash builder
    pub fn reset(&mut self) {
        self.hasher = Sha256::new();
    }
}

impl Default for HashBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Compute a fast non-cryptographic hash for data structures
pub fn fast_hash<T: Hash>(value: &T) -> u64 {
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish()
}

/// Compute hash of serializable data
pub fn hash_serializable<T: serde::Serialize>(value: &T) -> crate::error::Result<Hash256> {
    let bytes = bincode::serialize(value)
        .map_err(|e| crate::error::CryptoError::SerializationError { source: e })?;
    Ok(crate::crypto::sha256_hash(&bytes))
}

/// Compute hash with a salt
pub fn hash_with_salt(data: &[u8], salt: &[u8]) -> Hash256 {
    let mut hasher = Sha256::new();
    hasher.update(salt);
    hasher.update(data);
    Hash256::new(hasher.finalize().into())
}

/// Compute HMAC-SHA256
pub fn hmac_sha256(key: &[u8], message: &[u8]) -> Hash256 {
    use sha2::Sha256;
    
    const BLOCK_SIZE: usize = 64;
    const IPAD: u8 = 0x36;
    const OPAD: u8 = 0x5c;
    
    let mut key_padded = [0u8; BLOCK_SIZE];
    
    if key.len() > BLOCK_SIZE {
        let key_hash = crate::crypto::sha256_hash(key);
        key_padded[..32].copy_from_slice(key_hash.as_slice());
    } else {
        key_padded[..key.len()].copy_from_slice(key);
    }
    
    let mut ipad = [0u8; BLOCK_SIZE];
    let mut opad = [0u8; BLOCK_SIZE];
    
    for i in 0..BLOCK_SIZE {
        ipad[i] = key_padded[i] ^ IPAD;
        opad[i] = key_padded[i] ^ OPAD;
    }
    
    // Inner hash
    let mut inner_hasher = Sha256::new();
    inner_hasher.update(&ipad);
    inner_hasher.update(message);
    let inner_hash = inner_hasher.finalize();
    
    // Outer hash
    let mut outer_hasher = Sha256::new();
    outer_hasher.update(&opad);
    outer_hasher.update(&inner_hash);
    
    Hash256::new(outer_hasher.finalize().into())
}

/// Compute a hash chain (hash of hash of ... of data)
pub fn hash_chain(data: &[u8], iterations: usize) -> Hash256 {
    let mut result = crate::crypto::sha256_hash(data);
    for _ in 1..iterations {
        result = crate::crypto::sha256_hash(result.as_slice());
    }
    result
}

/// Compute a time-locked hash (for proof of sequential work)
pub fn time_locked_hash(data: &[u8], time_param: u32) -> Hash256 {
    hash_chain(data, time_param as usize)
}

/// Hash combiner for merging multiple hashes
pub struct HashCombiner {
    hashes: Vec<Hash256>,
}

impl HashCombiner {
    /// Create a new hash combiner
    pub fn new() -> Self {
        Self { hashes: Vec::new() }
    }

    /// Add a hash to combine
    pub fn add_hash(&mut self, hash: Hash256) -> &mut Self {
        self.hashes.push(hash);
        self
    }

    /// Add multiple hashes
    pub fn add_hashes(&mut self, hashes: &[Hash256]) -> &mut Self {
        self.hashes.extend_from_slice(hashes);
        self
    }

    /// Combine all hashes into a single hash
    pub fn combine(self) -> Hash256 {
        if self.hashes.is_empty() {
            return Hash256::zero();
        }

        let mut hasher = Sha256::new();
        for hash in &self.hashes {
            hasher.update(hash.as_slice());
        }
        Hash256::new(hasher.finalize().into())
    }

    /// Combine hashes in a tree structure (more efficient for large sets)
    pub fn combine_tree(mut self) -> Hash256 {
        if self.hashes.is_empty() {
            return Hash256::zero();
        }

        while self.hashes.len() > 1 {
            let mut next_level = Vec::new();
            
            for chunk in self.hashes.chunks(2) {
                if chunk.len() == 2 {
                    let combined = crate::crypto::hash_multiple(&[
                        chunk[0].as_slice(),
                        chunk[1].as_slice(),
                    ]);
                    next_level.push(combined);
                } else {
                    next_level.push(chunk[0].clone());
                }
            }
            
            self.hashes = next_level;
        }

        self.hashes.into_iter().next().unwrap_or_else(Hash256::zero)
    }

    /// Get the number of hashes
    pub fn len(&self) -> usize {
        self.hashes.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.hashes.is_empty()
    }
}

impl Default for HashCombiner {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_builder() {
        let mut builder = HashBuilder::new();
        let hash1 = builder
            .update(b"hello")
            .update(b"world")
            .finalize();

        let hash2 = crate::crypto::sha256_hash(b"helloworld");
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_hash_builder_numbers() {
        let mut builder = HashBuilder::new();
        let hash = builder
            .update_u64(12345)
            .update_u32(678)
            .finalize();
        
        assert!(!hash.is_zero());
    }

    #[test]
    fn test_fast_hash() {
        let value = "test string";
        let hash1 = fast_hash(&value);
        let hash2 = fast_hash(&value);
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_hash_with_salt() {
        let data = b"message";
        let salt = b"salt";
        let hash1 = hash_with_salt(data, salt);
        let hash2 = hash_with_salt(data, salt);
        let hash3 = hash_with_salt(data, b"different_salt");
        
        assert_eq!(hash1, hash2);
        assert_ne!(hash1, hash3);
    }

    #[test]
    fn test_hmac_sha256() {
        let key = b"secret_key";
        let message = b"message";
        let hmac1 = hmac_sha256(key, message);
        let hmac2 = hmac_sha256(key, message);
        let hmac3 = hmac_sha256(b"different_key", message);
        
        assert_eq!(hmac1, hmac2);
        assert_ne!(hmac1, hmac3);
    }

    #[test]
    fn test_hash_chain() {
        let data = b"test";
        let hash1 = hash_chain(data, 1);
        let hash2 = hash_chain(data, 2);
        let hash3 = hash_chain(data, 3);
        
        assert_ne!(hash1, hash2);
        assert_ne!(hash2, hash3);
        
        // Verify chain property
        let expected_hash2 = crate::crypto::sha256_hash(hash1.as_slice());
        assert_eq!(hash2, expected_hash2);
    }

    #[test]
    fn test_hash_combiner() {
        let hash1 = crate::crypto::sha256_hash(b"test1");
        let hash2 = crate::crypto::sha256_hash(b"test2");
        let hash3 = crate::crypto::sha256_hash(b"test3");
        
        let mut combiner = HashCombiner::new();
        let combined = combiner
            .add_hash(hash1)
            .add_hash(hash2)
            .add_hash(hash3)
            .combine();
        
        assert!(!combined.is_zero());
    }

    #[test]
    fn test_hash_combiner_tree() {
        let hashes: Vec<Hash256> = (0..7)
            .map(|i| crate::crypto::sha256_hash(&i.to_le_bytes()))
            .collect();
        
        let mut combiner = HashCombiner::new();
        combiner.add_hashes(&hashes);
        let tree_hash = combiner.combine_tree();
        
        assert!(!tree_hash.is_zero());
    }

    #[test]
    fn test_time_locked_hash() {
        let data = b"time_locked_data";
        let hash1 = time_locked_hash(data, 1);
        let hash10 = time_locked_hash(data, 10);
        
        assert_ne!(hash1, hash10);
    }
}