//! Cryptographic utilities for the LedgerDB blockchain system.
//!
//! This module provides hashing, digital signatures, Merkle trees,
//! and proof-of-work algorithms required for blockchain operations.

pub mod hash;
pub mod keys;
pub mod merkle;
pub mod pow;

// Re-export commonly used types
pub use hash::*;
pub use keys::*;
pub use merkle::*;
pub use pow::*;

use crate::error::{CryptoError, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fmt;

/// A 256-bit hash value
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Hash256([u8; 32]);

impl Hash256 {
    /// Create a new hash from bytes
    pub fn new(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// Create a hash from a slice
    pub fn from_slice(slice: &[u8]) -> Result<Self> {
        if slice.len() != 32 {
            return Err(CryptoError::InvalidFormat(
                "Hash must be exactly 32 bytes".to_string()
            ));
        }
        let mut bytes = [0u8; 32];
        bytes.copy_from_slice(slice);
        Ok(Self(bytes))
    }

    /// Get the hash as bytes
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// Get the hash as a slice
    pub fn as_slice(&self) -> &[u8] {
        &self.0
    }

    /// Convert to hex string
    pub fn to_hex(&self) -> String {
        hex::encode(self.0)
    }

    /// Create from hex string
    pub fn from_hex(hex_str: &str) -> Result<Self> {
        let bytes = hex::decode(hex_str)
            .map_err(|e| CryptoError::InvalidFormat(format!("Invalid hex: {}", e)))?;
        
        if bytes.len() != 32 {
            return Err(CryptoError::InvalidFormat(
                "Hash must be exactly 32 bytes".to_string()
            ));
        }
        
        let mut hash_bytes = [0u8; 32];
        hash_bytes.copy_from_slice(&bytes);
        Ok(Self(hash_bytes))
    }

    /// Create a zero hash
    pub fn zero() -> Self {
        Self([0u8; 32])
    }

    /// Check if this is a zero hash
    pub fn is_zero(&self) -> bool {
        self.0.iter().all(|&b| b == 0)
    }
}

impl Default for Hash256 {
    fn default() -> Self {
        Self::zero()
    }
}

impl fmt::Display for Hash256 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

impl From<[u8; 32]> for Hash256 {
    fn from(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }
}

impl AsRef<[u8]> for Hash256 {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

/// Digital signature representation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Signature {
    /// The signature algorithm used
    pub algorithm: SignatureAlgorithm,
    /// The signature bytes
    pub data: Vec<u8>,
}

/// Supported signature algorithms
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SignatureAlgorithm {
    /// ECDSA with secp256k1 curve
    EcdsaSecp256k1,
    /// Ed25519 signature scheme
    Ed25519,
}



/// Public key representation
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PublicKey {
    /// The signature algorithm
    pub algorithm: SignatureAlgorithm,
    /// The public key bytes
    pub data: Vec<u8>,
}

impl PublicKey {
    /// Create a new public key
    pub fn new(algorithm: SignatureAlgorithm, data: Vec<u8>) -> Self {
        Self { algorithm, data }
    }

    /// Get the key as bytes
    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }

    /// Convert to hex string
    pub fn to_hex(&self) -> String {
        hex::encode(&self.data)
    }
}

/// Blockchain address derived from public key
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Address(Hash256);

impl Address {
    /// Create address from public key
    pub fn from_public_key(public_key: &PublicKey) -> Self {
        let hash = hash_data(&public_key.data);
        Self(hash)
    }

    /// Get the address as a hash
    pub fn as_hash(&self) -> &Hash256 {
        &self.0
    }

    /// Convert to hex string
    pub fn to_hex(&self) -> String {
        self.0.to_hex()
    }

    /// Create from hex string
    pub fn from_hex(hex_str: &str) -> Result<Self> {
        let hash = Hash256::from_hex(hex_str)?;
        Ok(Self(hash))
    }
}

impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}



/// Hash arbitrary data using SHA-256
pub fn hash_data(data: &[u8]) -> Hash256 {
    let mut hasher = Sha256::new();
    hasher.update(data);
    Hash256(hasher.finalize().into())
}

/// Double SHA-256 hash (Bitcoin-style)
pub fn double_hash(data: &[u8]) -> Hash256 {
    let first_hash = hash_data(data);
    hash_data(first_hash.as_slice())
}

/// Hash multiple pieces of data together
pub fn hash_multiple(data_pieces: &[&[u8]]) -> Hash256 {
    let mut hasher = Sha256::new();
    for piece in data_pieces {
        hasher.update(piece);
    }
    Hash256(hasher.finalize().into())
}



/// Verify a signature (placeholder implementation)
pub fn verify_signature(
    _message: &[u8],
    _signature: &Signature,
    _public_key: &PublicKey,
) -> Result<bool> {
    // TODO: Implement actual signature verification
    // This would require integrating with cryptographic libraries
    // like secp256k1 or ed25519-dalek
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash256_creation() {
        let bytes = [1u8; 32];
        let hash = Hash256::new(bytes);
        assert_eq!(hash.as_bytes(), &bytes);
    }

    #[test]
    fn test_hash256_hex() {
        let hash = Hash256::zero();
        let hex = hash.to_hex();
        assert_eq!(hex.len(), 64);
        assert!(hex.chars().all(|c| c == '0'));

        let parsed = Hash256::from_hex(&hex).unwrap();
        assert_eq!(hash, parsed);
    }

    #[test]
    fn test_hash256_zero() {
        let zero_hash = Hash256::zero();
        assert_eq!(zero_hash.as_bytes(), &[0u8; 32]);
    }

    #[test]
    fn test_hash_data() {
        let data = b"hello world";
        let hash1 = hash_data(data);
        let hash2 = hash_data(data);
        assert_eq!(hash1, hash2); // Same input should produce same hash
        
        let different_data = b"hello world!";
        let hash3 = hash_data(different_data);
        assert_ne!(hash1, hash3); // Different input should produce different hash
    }

    #[test]
    fn test_double_hash() {
        let data = b"test data";
        let single_hash = hash_data(data);
        let double_hash_result = double_hash(data);
        
        assert_ne!(single_hash, double_hash_result);
    }

    #[test]
    fn test_public_key() {
        let key_data = vec![1, 2, 3, 4, 5];
        let public_key = PublicKey::new(SignatureAlgorithm::EcdsaSecp256k1, key_data.clone());
        
        assert_eq!(public_key.as_bytes(), &key_data);
        assert_eq!(public_key.algorithm, SignatureAlgorithm::EcdsaSecp256k1);
    }

    #[test]
    fn test_address_from_public_key() {
        let key_data = vec![1, 2, 3, 4, 5];
        let public_key = PublicKey::new(SignatureAlgorithm::EcdsaSecp256k1, key_data);
        
        let address1 = Address::from_public_key(&public_key);
        let address2 = Address::from_public_key(&public_key);
        
        assert_eq!(address1, address2); // Same key should produce same address
    }

    #[test]
    fn test_address_hex() {
        let key_data = vec![1, 2, 3, 4, 5];
        let public_key = PublicKey::new(SignatureAlgorithm::EcdsaSecp256k1, key_data);
        let address = Address::from_public_key(&public_key);
        
        let hex = address.to_hex();
        let parsed = Address::from_hex(&hex).unwrap();
        
        assert_eq!(address, parsed);
    }

    #[test]
    fn test_hash_multiple() {
        let data1 = b"hello";
        let data2 = b"world";
        let combined_hash = hash_multiple(&[data1, data2]);
        let single_hash = hash_data(b"helloworld");
        
        assert_eq!(combined_hash, single_hash);
    }
}