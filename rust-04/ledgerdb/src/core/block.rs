//! Block data structures and validation logic.
//!
//! This module defines the block structure used in the LedgerDB blockchain,
//! including block headers, validation, and mining-related functionality.

use crate::core::Transaction;
use crate::crypto::{Hash256, MerkleTree};
use crate::error::{Result, ValidationError};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Block header containing metadata
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlockHeader {
    /// Block version for future upgrades
    pub version: u32,
    /// Hash of the previous block
    pub previous_hash: Hash256,
    /// Merkle root of all transactions in the block
    pub merkle_root: Hash256,
    /// Block creation timestamp
    pub timestamp: DateTime<Utc>,
    /// Mining difficulty target
    pub difficulty: u32,
    /// Nonce used for proof-of-work
    pub nonce: u64,
    /// Number of transactions in the block
    pub transaction_count: u32,
    /// Block size in bytes
    pub size: u64,
    /// Additional metadata hash (optional)
    pub metadata_hash: Option<Hash256>,
}

impl BlockHeader {
    /// Create a new block header
    pub fn new(
        version: u32,
        previous_hash: Hash256,
        merkle_root: Hash256,
        difficulty: u32,
        transaction_count: u32,
    ) -> Self {
        Self {
            version,
            previous_hash,
            merkle_root,
            timestamp: Utc::now(),
            difficulty,
            nonce: 0,
            transaction_count,
            size: 0,
            metadata_hash: None,
        }
    }

    /// Calculate the hash of this block header
    pub fn hash(&self) -> Hash256 {
        let serialized = bincode::serialize(self).unwrap_or_default();
        crate::crypto::hash_data(&serialized)
    }

    /// Validate the block header structure
    pub fn validate(&self) -> Result<()> {
        if self.version == 0 {
            return Err(ValidationError::InvalidVersion("Block version cannot be zero".to_string()).into());
        }
        
        if self.difficulty == 0 {
            return Err(ValidationError::InvalidDifficulty("Difficulty cannot be zero".to_string()).into());
        }
        
        // Check timestamp is not too far in the future (within 2 hours)
        let max_future_time = Utc::now() + chrono::Duration::hours(2);
        if self.timestamp > max_future_time {
            return Err(ValidationError::InvalidTimestamp("Block timestamp too far in future".to_string()).into());
        }
        
        Ok(())
    }

    /// Check if this header satisfies the proof-of-work requirement
    pub fn meets_difficulty_target(&self) -> bool {
        crate::crypto::validate_proof_of_work(
            &bincode::serialize(self).unwrap_or_default(),
            self.nonce,
            self.difficulty,
        )
    }
}

/// Block metadata for additional information
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlockMetadata {
    /// Block proposer/miner identifier
    pub proposer: Option<String>,
    /// Gas limit for the block
    pub gas_limit: Option<u64>,
    /// Gas used by all transactions
    pub gas_used: Option<u64>,
    /// Block reward amount
    pub block_reward: u64,
    /// Total transaction fees in the block
    pub total_fees: u64,
    /// Average transaction fee
    pub average_fee: u64,
    /// Block processing time in milliseconds
    pub processing_time_ms: Option<u64>,
    /// Additional arbitrary data
    pub extra_data: Option<Vec<u8>>,
}

impl Default for BlockMetadata {
    fn default() -> Self {
        Self {
            proposer: None,
            gas_limit: Some(21_000_000), // Default gas limit
            gas_used: Some(0),
            block_reward: 5_000_000_000, // 5 units in smallest denomination
            total_fees: 0,
            average_fee: 0,
            processing_time_ms: None,
            extra_data: None,
        }
    }
}

/// Complete block structure
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Block {
    /// Block header
    pub header: BlockHeader,
    /// List of transactions in the block
    pub transactions: Vec<Transaction>,
    /// Block metadata
    pub metadata: BlockMetadata,
    /// Block index/height in the chain
    pub index: u64,
    /// Cached block hash
    #[serde(skip)]
    pub cached_hash: Option<Hash256>,
}

impl Block {
    /// Create a new block
    pub fn new(
        index: u64,
        previous_hash: Hash256,
        transactions: Vec<Transaction>,
        difficulty: u32,
    ) -> Self {
        let merkle_tree = MerkleTree::from_transactions(&transactions);
        let merkle_root = merkle_tree.root();
        
        let header = BlockHeader::new(
            1, // version
            previous_hash,
            merkle_root,
            difficulty,
            transactions.len() as u32,
        );
        
        let mut metadata = BlockMetadata::default();
        metadata.total_fees = transactions.iter()
            .map(|tx| tx.fee.base_fee)
            .sum();
        
        if !transactions.is_empty() {
            metadata.average_fee = metadata.total_fees / transactions.len() as u64;
        }
        
        let mut block = Self {
            header,
            transactions,
            metadata,
            index,
            cached_hash: None,
        };
        
        block.calculate_size();
        block
    }

    /// Create the genesis block
    pub fn genesis(genesis_address: crate::crypto::Address, initial_supply: u64) -> Self {
        let genesis_tx = Transaction::coinbase(genesis_address, initial_supply, 0);
        let mut block = Self::new(0, Hash256::zero(), vec![genesis_tx], 1);
        
        // Set genesis block timestamp to a fixed value
        block.header.timestamp = DateTime::parse_from_rfc3339("2024-01-01T00:00:00Z")
            .unwrap()
            .with_timezone(&Utc);
        
        block.metadata.proposer = Some("genesis".to_string());
        block.metadata.extra_data = Some(b"LedgerDB Genesis Block".to_vec());
        
        block
    }

    /// Get the hash of this block
    pub fn hash(&self) -> Hash256 {
        if let Some(cached) = &self.cached_hash {
            return cached.clone();
        }
        
        let hash = self.header.hash();
        // Note: We can't cache here due to immutable reference
        hash
    }

    /// Calculate and cache the block hash
    pub fn calculate_and_cache_hash(&mut self) -> Hash256 {
        let hash = self.header.hash();
        self.cached_hash = Some(hash.clone());
        hash
    }

    /// Calculate and set the block size
    pub fn calculate_size(&mut self) {
        let serialized = bincode::serialize(self).unwrap_or_default();
        self.header.size = serialized.len() as u64;
    }

    /// Get the Merkle tree for this block's transactions
    pub fn merkle_tree(&self) -> MerkleTree {
        MerkleTree::from_transactions(&self.transactions)
    }

    /// Verify the Merkle root matches the transactions
    pub fn verify_merkle_root(&self) -> bool {
        let merkle_tree = self.merkle_tree();
        let calculated_root = merkle_tree.root();
        *calculated_root == self.header.merkle_root
    }

    /// Get a transaction by its hash
    pub fn get_transaction(&self, tx_hash: &Hash256) -> Option<&Transaction> {
        self.transactions.iter()
            .find(|tx| &tx.hash() == tx_hash)
    }

    /// Get transaction by index
    pub fn get_transaction_by_index(&self, index: usize) -> Option<&Transaction> {
        self.transactions.get(index)
    }

    /// Check if block contains a specific transaction
    pub fn contains_transaction(&self, tx_hash: &Hash256) -> bool {
        self.get_transaction(tx_hash).is_some()
    }

    /// Generate a Merkle proof for a transaction at a specific index
    pub fn generate_merkle_proof(&self, tx_index: usize) -> Result<crate::crypto::MerkleProof> {
        let merkle_tree = self.merkle_tree();
        merkle_tree.generate_proof_by_index(tx_index)
    }

    /// Get all transaction hashes in this block
    pub fn transaction_hashes(&self) -> Vec<Hash256> {
        self.transactions.iter()
            .map(|tx| tx.hash())
            .collect()
    }

    /// Validate the entire block
    pub fn validate(
        &self,
        previous_block: Option<&Block>,
        utxo_set: &HashMap<String, crate::core::TransactionOutput>,
    ) -> Result<()> {
        // Validate header
        self.header.validate()?;
        
        // Check index continuity
        if let Some(prev) = previous_block {
            if self.index != prev.index + 1 {
                return Err(ValidationError::InvalidBlockIndex {
                    expected: prev.index + 1,
                    actual: self.index,
                }.into());
            }
            
            if self.header.previous_hash != prev.hash() {
                return Err(ValidationError::InvalidPreviousHash.into());
            }
            
            // Check timestamp is after previous block
            if self.header.timestamp <= prev.header.timestamp {
                return Err(ValidationError::InvalidTimestamp(
                    "Block timestamp must be after previous block".to_string()
                ).into());
            }
        } else if self.index != 0 {
            return Err(ValidationError::InvalidBlockIndex {
                expected: 0,
                actual: self.index,
            }.into());
        }
        
        // Validate transactions
        if self.transactions.is_empty() {
            return Err(ValidationError::EmptyBlock.into());
        }
        
        // First transaction should be coinbase for non-genesis blocks
        if self.index > 0 && !self.transactions[0].is_coinbase() {
            return Err(ValidationError::MissingCoinbase.into());
        }
        
        // Only first transaction should be coinbase
        for (i, tx) in self.transactions.iter().enumerate() {
            if i == 0 && self.index > 0 {
                if !tx.is_coinbase() {
                    return Err(ValidationError::MissingCoinbase.into());
                }
            } else if tx.is_coinbase() {
                return Err(ValidationError::MultipleCoinbase.into());
            }
            
            // Validate each transaction
            tx.validate(utxo_set)?;
        }
        
        // Verify Merkle root
        if !self.verify_merkle_root() {
            return Err(ValidationError::InvalidMerkleRoot.into());
        }
        
        // Verify proof of work
        if !self.header.meets_difficulty_target() {
            return Err(ValidationError::InvalidProofOfWork.into());
        }
        
        // Validate transaction count
        if self.header.transaction_count != self.transactions.len() as u32 {
            return Err(ValidationError::InvalidTransactionCount(
                format!("Expected {} transactions, found {}", 
                    self.transactions.len(), 
                    self.header.transaction_count)
            ).into());
        }
        
        Ok(())
    }

    /// Mine this block by finding a valid nonce
    pub fn mine(&mut self, progress_callback: Option<Box<dyn Fn(u64, f64) + Send>>) -> Result<()> {
        use std::time::Instant;
        
        let start_time = Instant::now();
        let mut attempts = 0u64;
        
        loop {
            attempts += 1;
            
            // Check if current nonce satisfies difficulty
            if self.header.meets_difficulty_target() {
                self.calculate_and_cache_hash();
                return Ok(());
            }
            
            // Increment nonce
            self.header.nonce = self.header.nonce.wrapping_add(1);
            
            // Report progress every 100,000 attempts
            if attempts % 100_000 == 0 {
                if let Some(ref callback) = progress_callback {
                    let elapsed = start_time.elapsed().as_secs_f64();
                    let hash_rate = attempts as f64 / elapsed;
                    callback(attempts, hash_rate);
                }
            }
            
            // Prevent infinite loops in tests
            if attempts > 10_000_000 {
                return Err(ValidationError::MiningTimeout.into());
            }
        }
    }

    /// Get block statistics
    pub fn stats(&self) -> BlockStats {
        let total_tx_fees: u64 = self.transactions.iter()
            .filter(|tx| !tx.is_coinbase())
            .map(|tx| tx.fee.base_fee)
            .sum();
        
        let total_amount_transferred: u64 = self.transactions.iter()
            .map(|tx| tx.total_output_amount())
            .sum();
        
        let avg_tx_size = if !self.transactions.is_empty() {
            self.transactions.iter()
                .filter_map(|tx| tx.size)
                .sum::<usize>() / self.transactions.len()
        } else {
            0
        };
        
        BlockStats {
            index: self.index,
            hash: self.hash(),
            timestamp: self.header.timestamp,
            transaction_count: self.transactions.len(),
            total_fees: total_tx_fees,
            total_amount: total_amount_transferred,
            block_size: self.header.size,
            average_tx_size: avg_tx_size,
            difficulty: self.header.difficulty,
            nonce: self.header.nonce,
        }
    }

    /// Check if this is the genesis block
    pub fn is_genesis(&self) -> bool {
        self.index == 0 && self.header.previous_hash == Hash256::zero()
    }

    /// Get the coinbase transaction (if any)
    pub fn coinbase_transaction(&self) -> Option<&Transaction> {
        self.transactions.first().filter(|tx| tx.is_coinbase())
    }

    /// Get non-coinbase transactions
    pub fn regular_transactions(&self) -> Vec<&Transaction> {
        self.transactions.iter()
            .filter(|tx| !tx.is_coinbase())
            .collect()
    }
}

/// Block statistics for reporting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockStats {
    /// Block index/height
    pub index: u64,
    /// Block hash
    pub hash: Hash256,
    /// Block timestamp
    pub timestamp: DateTime<Utc>,
    /// Number of transactions
    pub transaction_count: usize,
    /// Total transaction fees
    pub total_fees: u64,
    /// Total amount transferred
    pub total_amount: u64,
    /// Block size in bytes
    pub block_size: u64,
    /// Average transaction size
    pub average_tx_size: usize,
    /// Mining difficulty
    pub difficulty: u32,
    /// Mining nonce
    pub nonce: u64,
}

/// Block validation context
#[derive(Debug, Clone)]
pub struct BlockValidationContext {
    /// Current blockchain height
    pub current_height: u64,
    /// Target block time in seconds
    pub target_block_time: u64,
    /// Maximum block size
    pub max_block_size: u64,
    /// Maximum transactions per block
    pub max_transactions: u32,
    /// Minimum difficulty
    pub min_difficulty: u32,
    /// Maximum difficulty adjustment
    pub max_difficulty_adjustment: f64,
}

impl Default for BlockValidationContext {
    fn default() -> Self {
        Self {
            current_height: 0,
            target_block_time: 600, // 10 minutes
            max_block_size: 1_000_000, // 1MB
            max_transactions: 1000,
            min_difficulty: 1,
            max_difficulty_adjustment: 4.0, // 4x max adjustment
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::TransactionInput;
    use crate::crypto::{Address, PublicKey, SignatureAlgorithm};

    fn create_test_address() -> Address {
        let public_key = PublicKey::new(SignatureAlgorithm::EcdsaSecp256k1, vec![1, 2, 3, 4, 5]);
        Address::from_public_key(&public_key)
    }

    fn create_test_transaction() -> Transaction {
        let input = TransactionInput::new(Hash256::zero(), 0, None, None);
        let output = crate::core::TransactionOutput::new(1000, create_test_address());
        Transaction::new(vec![input], vec![output])
    }

    #[test]
    fn test_block_creation() {
        let transactions = vec![create_test_transaction()];
        let block = Block::new(1, Hash256::zero(), transactions, 4);
        
        assert_eq!(block.index, 1);
        assert_eq!(block.header.previous_hash, Hash256::zero());
        assert_eq!(block.transactions.len(), 1);
        assert_eq!(block.header.difficulty, 4);
    }

    #[test]
    fn test_genesis_block() {
        let genesis_address = create_test_address();
        let genesis = Block::genesis(genesis_address, 1_000_000);
        
        assert!(genesis.is_genesis());
        assert_eq!(genesis.index, 0);
        assert_eq!(genesis.transactions.len(), 1);
        assert!(genesis.transactions[0].is_coinbase());
    }

    #[test]
    fn test_block_hash() {
        let transactions = vec![create_test_transaction()];
        let block = Block::new(1, Hash256::zero(), transactions, 4);
        
        let hash1 = block.hash();
        let hash2 = block.hash();
        
        assert_eq!(hash1, hash2); // Same block should produce same hash
    }

    #[test]
    fn test_merkle_root_verification() {
        let transactions = vec![create_test_transaction(), create_test_transaction()];
        let block = Block::new(1, Hash256::zero(), transactions, 4);
        
        assert!(block.verify_merkle_root());
    }

    #[test]
    fn test_block_header_validation() {
        let header = BlockHeader::new(1, Hash256::zero(), Hash256::zero(), 4, 1);
        assert!(header.validate().is_ok());
        
        let invalid_header = BlockHeader::new(0, Hash256::zero(), Hash256::zero(), 0, 1);
        assert!(invalid_header.validate().is_err());
    }

    #[test]
    fn test_transaction_lookup() {
        let tx = create_test_transaction();
        let tx_hash = tx.hash();
        let transactions = vec![tx];
        let block = Block::new(1, Hash256::zero(), transactions, 4);
        
        assert!(block.contains_transaction(&tx_hash));
        assert!(block.get_transaction(&tx_hash).is_some());
        assert!(block.get_transaction_by_index(0).is_some());
    }

    #[test]
    fn test_block_stats() {
        let transactions = vec![create_test_transaction()];
        let block = Block::new(1, Hash256::zero(), transactions, 4);
        
        let stats = block.stats();
        assert_eq!(stats.index, 1);
        assert_eq!(stats.transaction_count, 1);
        assert_eq!(stats.difficulty, 4);
    }

    #[test]
    fn test_coinbase_transaction_detection() {
        let genesis_address = create_test_address();
        let genesis = Block::genesis(genesis_address, 1_000_000);
        
        assert!(genesis.coinbase_transaction().is_some());
        assert_eq!(genesis.regular_transactions().len(), 0);
        
        let regular_tx = create_test_transaction();
        let block = Block::new(1, Hash256::zero(), vec![regular_tx], 4);
        assert!(block.coinbase_transaction().is_none());
        assert_eq!(block.regular_transactions().len(), 1);
    }
}