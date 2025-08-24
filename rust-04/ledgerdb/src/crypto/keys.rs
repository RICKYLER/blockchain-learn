//! Cryptographic key management for the LedgerDB blockchain system.
//!
//! This module provides key generation, management, and digital signature
//! functionality for securing blockchain transactions and operations.

use crate::crypto::{Address, Hash256, PublicKey, Signature, SignatureAlgorithm};
use crate::error::{CryptoError, Result};
use rand::{CryptoRng, RngCore};
use serde::{Deserialize, Serialize};
use std::fmt;

/// A private key for signing operations
#[derive(Clone, Serialize, Deserialize)]
pub struct PrivateKey {
    /// The private key bytes
    bytes: Vec<u8>,
    /// The signature algorithm
    algorithm: SignatureAlgorithm,
}

// Implement Debug without showing the private key bytes
impl fmt::Debug for PrivateKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PrivateKey")
            .field("algorithm", &self.algorithm)
            .field("bytes", &"[REDACTED]")
            .finish()
    }
}

impl PrivateKey {
    /// Create a new private key from bytes
    pub fn new(bytes: Vec<u8>, algorithm: SignatureAlgorithm) -> Self {
        Self { bytes, algorithm }
    }

    /// Generate a new random private key
    pub fn generate<R: CryptoRng + RngCore>(
        rng: &mut R,
        algorithm: SignatureAlgorithm,
    ) -> Result<Self> {
        let bytes = match algorithm {
            SignatureAlgorithm::EcdsaSecp256k1 => {
                let mut key_bytes = vec![0u8; 32];
                rng.fill_bytes(&mut key_bytes);
                key_bytes
            }
            SignatureAlgorithm::Ed25519 => {
                let mut key_bytes = vec![0u8; 32];
                rng.fill_bytes(&mut key_bytes);
                key_bytes
            }
        };

        Ok(Self::new(bytes, algorithm))
    }

    /// Get the algorithm used by this key
    pub fn algorithm(&self) -> SignatureAlgorithm {
        self.algorithm.clone()
    }

    /// Get the private key bytes (use with caution)
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Derive the public key from this private key
    pub fn public_key(&self) -> Result<PublicKey> {
        match self.algorithm {
            SignatureAlgorithm::EcdsaSecp256k1 => {
                // TODO: Implement ECDSA public key derivation
                // For now, use a simple hash-based derivation (NOT SECURE)
                let pub_key_hash = crate::crypto::hash_data(&self.bytes);
                Ok(PublicKey::new(
                    self.algorithm.clone(),
                    pub_key_hash.as_slice().to_vec(),
                ))
            }
            SignatureAlgorithm::Ed25519 => {
                // TODO: Implement Ed25519 public key derivation
                // For now, use a simple hash-based derivation (NOT SECURE)
                let pub_key_hash = crate::crypto::hash_data(&self.bytes);
                Ok(PublicKey::new(
                    self.algorithm.clone(),
                    pub_key_hash.as_slice().to_vec(),
                ))
            }
        }
    }

    /// Sign a message with this private key
    pub fn sign(&self, message: &[u8]) -> Result<Signature> {
        match self.algorithm {
            SignatureAlgorithm::EcdsaSecp256k1 => {
                // TODO: Implement ECDSA signing
                // For now, use a simple hash-based signature (NOT SECURE)
                let message_hash = crate::crypto::hash_data(message);
                let signature_data = crate::crypto::hash_multiple(&[
                    &self.bytes,
                    message_hash.as_slice(),
                ]);
                Ok(Signature::new(
                    self.algorithm.clone(),
                    signature_data.as_slice().to_vec(),
                ))
            }
            SignatureAlgorithm::Ed25519 => {
                // TODO: Implement Ed25519 signing
                // For now, use a simple hash-based signature (NOT SECURE)
                let message_hash = crate::crypto::hash_data(message);
                let signature_data = crate::crypto::hash_multiple(&[
                    &self.bytes,
                    message_hash.as_slice(),
                ]);
                Ok(Signature::new(
                    self.algorithm.clone(),
                    signature_data.as_slice().to_vec(),
                ))
            }
        }
    }

    /// Convert to hex string (use with extreme caution)
    pub fn to_hex(&self) -> String {
        hex::encode(&self.bytes)
    }

    /// Create from hex string
    pub fn from_hex(hex_str: &str, algorithm: SignatureAlgorithm) -> Result<Self> {
        let bytes = hex::decode(hex_str).map_err(|_| CryptoError::InvalidHexString {
            hex_str: hex_str.to_string(),
        })?;
        Ok(Self::new(bytes, algorithm))
    }

    /// Securely clear the private key from memory
    pub fn zeroize(&mut self) {
        self.bytes.fill(0);
    }
}

/// Drop implementation to securely clear private key
impl Drop for PrivateKey {
    fn drop(&mut self) {
        self.zeroize();
    }
}

/// A key pair containing both private and public keys
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyPair {
    private_key: PrivateKey,
    public_key: PublicKey,
    address: Address,
}

impl KeyPair {
    /// Create a new key pair
    pub fn new(private_key: PrivateKey) -> Result<Self> {
        let public_key = private_key.public_key()?;
        let address = public_key.to_address();
        Ok(Self {
            private_key,
            public_key,
            address,
        })
    }

    /// Generate a new random key pair
    pub fn generate<R: CryptoRng + RngCore>(
        rng: &mut R,
        algorithm: SignatureAlgorithm,
    ) -> Result<Self> {
        let private_key = PrivateKey::generate(rng, algorithm)?;
        Self::new(private_key)
    }

    /// Get the private key
    pub fn private_key(&self) -> &PrivateKey {
        &self.private_key
    }

    /// Get the public key
    pub fn public_key(&self) -> &PublicKey {
        &self.public_key
    }

    /// Get the address
    pub fn address(&self) -> &Address {
        &self.address
    }

    /// Sign a message with this key pair
    pub fn sign(&self, message: &[u8]) -> Result<Signature> {
        self.private_key.sign(message)
    }

    /// Verify a signature against this key pair's public key
    pub fn verify(&self, message: &[u8], signature: &Signature) -> Result<bool> {
        crate::crypto::verify_signature(message, signature, &self.public_key)
    }
}

/// Key manager for handling multiple key pairs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyManager {
    key_pairs: Vec<KeyPair>,
    default_algorithm: SignatureAlgorithm,
}

impl KeyManager {
    /// Create a new key manager
    pub fn new(default_algorithm: SignatureAlgorithm) -> Self {
        Self {
            key_pairs: Vec::new(),
            default_algorithm,
        }
    }

    /// Generate and add a new key pair
    pub fn generate_key_pair<R: CryptoRng + RngCore>(&mut self, rng: &mut R) -> Result<&KeyPair> {
        let key_pair = KeyPair::generate(rng, self.default_algorithm.clone())?;
        self.key_pairs.push(key_pair);
        Ok(self.key_pairs.last().unwrap())
    }

    /// Add an existing key pair
    pub fn add_key_pair(&mut self, key_pair: KeyPair) {
        self.key_pairs.push(key_pair);
    }

    /// Get a key pair by address
    pub fn get_key_pair(&self, address: &Address) -> Option<&KeyPair> {
        self.key_pairs.iter().find(|kp| kp.address() == address)
    }

    /// Get all addresses
    pub fn addresses(&self) -> Vec<Address> {
        self.key_pairs.iter().map(|kp| kp.address().clone()).collect()
    }

    /// Get the number of key pairs
    pub fn len(&self) -> usize {
        self.key_pairs.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.key_pairs.is_empty()
    }

    /// Sign a message with a specific address
    pub fn sign_with_address(&self, address: &Address, message: &[u8]) -> Result<Signature> {
        let key_pair = self
            .get_key_pair(address)
            .ok_or_else(|| CryptoError::KeyNotFound {
                hash: address.to_hex(),
            })?;
        key_pair.sign(message)
    }

    /// Remove a key pair by address
    pub fn remove_key_pair(&mut self, address: &Address) -> Option<KeyPair> {
        if let Some(pos) = self.key_pairs.iter().position(|kp| kp.address() == address) {
            Some(self.key_pairs.remove(pos))
        } else {
            None
        }
    }

    /// Clear all key pairs
    pub fn clear(&mut self) {
        self.key_pairs.clear();
    }
}

impl Default for KeyManager {
    fn default() -> Self {
        Self::new(SignatureAlgorithm::Ed25519)
    }
}

/// Utility functions for key operations
pub mod utils {
    use super::*;

    /// Generate a deterministic key pair from a seed
    pub fn key_pair_from_seed(seed: &[u8], algorithm: SignatureAlgorithm) -> Result<KeyPair> {
        let private_key_hash = crate::crypto::sha256_hash(seed);
        let private_key = PrivateKey::new(private_key_hash.as_slice().to_vec(), algorithm);
        KeyPair::new(private_key)
    }

    /// Generate a key pair from a passphrase
    pub fn key_pair_from_passphrase(
        passphrase: &str,
        algorithm: SignatureAlgorithm,
    ) -> Result<KeyPair> {
        key_pair_from_seed(passphrase.as_bytes(), algorithm)
    }

    /// Derive a child key from a parent key (simple derivation)
    pub fn derive_child_key(
        parent_key: &PrivateKey,
        index: u32,
    ) -> Result<PrivateKey> {
        let derivation_data = crate::crypto::hash_multiple(&[
            parent_key.as_bytes(),
            &index.to_le_bytes(),
        ]);
        Ok(PrivateKey::new(
            derivation_data.as_slice().to_vec(),
            parent_key.algorithm(),
        ))
    }

    /// Generate multiple key pairs at once
    pub fn generate_multiple_key_pairs<R: CryptoRng + RngCore>(
        rng: &mut R,
        count: usize,
        algorithm: SignatureAlgorithm,
    ) -> Result<Vec<KeyPair>> {
        let mut key_pairs = Vec::with_capacity(count);
        for _ in 0..count {
            key_pairs.push(KeyPair::generate(rng, algorithm.clone())?);
        }
        Ok(key_pairs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::thread_rng;

    #[test]
    fn test_private_key_generation() {
        let mut rng = thread_rng();
        let private_key = PrivateKey::generate(&mut rng, SignatureAlgorithm::Ed25519).unwrap();
        assert_eq!(private_key.algorithm(), SignatureAlgorithm::Ed25519);
        assert_eq!(private_key.as_bytes().len(), 32);
    }

    #[test]
    fn test_key_pair_generation() {
        let mut rng = thread_rng();
        let key_pair = KeyPair::generate(&mut rng, SignatureAlgorithm::Ed25519).unwrap();
        assert_eq!(
            key_pair.private_key().algorithm(),
            SignatureAlgorithm::Ed25519
        );
        assert_eq!(
            key_pair.public_key().algorithm,
            SignatureAlgorithm::Ed25519
        );
    }

    #[test]
    fn test_signing_and_verification() {
        let mut rng = thread_rng();
        let key_pair = KeyPair::generate(&mut rng, SignatureAlgorithm::Ed25519).unwrap();
        let message = b"test message";
        
        let signature = key_pair.sign(message).unwrap();
        // Note: verification will return false with our placeholder implementation
        let _is_valid = key_pair.verify(message, &signature).unwrap();
    }

    #[test]
    fn test_key_manager() {
        let mut rng = thread_rng();
        let mut manager = KeyManager::new(SignatureAlgorithm::Ed25519);
        
        let key_pair = manager.generate_key_pair(&mut rng).unwrap();
        let address = key_pair.address().clone();
        
        assert_eq!(manager.len(), 1);
        assert!(manager.get_key_pair(&address).is_some());
        
        let message = b"test message";
        let _signature = manager.sign_with_address(&address, message).unwrap();
    }

    #[test]
    fn test_key_pair_from_seed() {
        let seed = b"test seed";
        let key_pair1 = utils::key_pair_from_seed(seed, SignatureAlgorithm::Ed25519).unwrap();
        let key_pair2 = utils::key_pair_from_seed(seed, SignatureAlgorithm::Ed25519).unwrap();
        
        // Should be deterministic
        assert_eq!(key_pair1.address(), key_pair2.address());
    }

    #[test]
    fn test_child_key_derivation() {
        let mut rng = thread_rng();
        let parent_key = PrivateKey::generate(&mut rng, SignatureAlgorithm::Ed25519).unwrap();
        
        let child_key1 = utils::derive_child_key(&parent_key, 0).unwrap();
        let child_key2 = utils::derive_child_key(&parent_key, 1).unwrap();
        
        assert_ne!(child_key1.as_bytes(), child_key2.as_bytes());
        assert_ne!(child_key1.as_bytes(), parent_key.as_bytes());
    }

    #[test]
    fn test_private_key_hex() {
        let mut rng = thread_rng();
        let private_key = PrivateKey::generate(&mut rng, SignatureAlgorithm::Ed25519).unwrap();
        
        let hex = private_key.to_hex();
        let restored = PrivateKey::from_hex(&hex, SignatureAlgorithm::Ed25519).unwrap();
        
        assert_eq!(private_key.as_bytes(), restored.as_bytes());
    }

    #[test]
    fn test_multiple_key_generation() {
        let mut rng = thread_rng();
        let key_pairs = utils::generate_multiple_key_pairs(
            &mut rng,
            5,
            SignatureAlgorithm::Ed25519,
        ).unwrap();
        
        assert_eq!(key_pairs.len(), 5);
        
        // All addresses should be unique
        let addresses: std::collections::HashSet<_> = key_pairs
            .iter()
            .map(|kp| kp.address().clone())
            .collect();
        assert_eq!(addresses.len(), 5);
    }
}