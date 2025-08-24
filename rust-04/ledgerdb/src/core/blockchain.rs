//! Blockchain core logic and state management.
//!
//! This module implements the main blockchain structure, including block validation,
//! chain management, UTXO tracking, and consensus rules.

use crate::core::{Block, Transaction, TransactionInput, TransactionOutput};
use crate::crypto::{Hash256, MerkleTree};
use crate::error::{Result, BlockchainError, ValidationError};
use crate::storage::PersistentStorage;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::{Arc, RwLock};

/// UTXO (Unspent Transaction Output) identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UtxoId {
    /// Transaction hash containing the output
    pub tx_hash: Hash256,
    /// Output index within the transaction
    pub output_index: u32,
}

impl UtxoId {
    /// Create a new UTXO identifier
    pub fn new(tx_hash: Hash256, output_index: u32) -> Self {
        Self { tx_hash, output_index }
    }

    /// Convert to string representation
    pub fn to_string(&self) -> String {
        format!("{}:{}", self.tx_hash.to_hex(), self.output_index)
    }

    /// Parse from string representation
    pub fn from_string(s: &str) -> Result<Self> {
        let parts: Vec<&str> = s.split(':').collect();
        if parts.len() != 2 {
            return Err(ValidationError::InvalidUtxoId(s.to_string()).into());
        }
        
        let tx_hash = Hash256::from_hex(parts[0])
            .map_err(|_| ValidationError::InvalidUtxoId(s.to_string()))?;
        let output_index = parts[1].parse::<u32>()
            .map_err(|_| ValidationError::InvalidUtxoId(s.to_string()))?;
        
        Ok(Self::new(tx_hash, output_index))
    }
}

/// UTXO set entry
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UtxoEntry {
    /// The transaction output
    pub output: TransactionOutput,
    /// Block height where this UTXO was created
    pub block_height: u64,
    /// Transaction hash that created this UTXO
    pub tx_hash: Hash256,
    /// Output index within the transaction
    pub output_index: u32,
    /// Whether this UTXO is spent (for tracking)
    pub is_spent: bool,
    /// Block height where this UTXO was spent (if applicable)
    pub spent_at_height: Option<u64>,
}

impl UtxoEntry {
    /// Create a new UTXO entry
    pub fn new(
        output: TransactionOutput,
        block_height: u64,
        tx_hash: Hash256,
        output_index: u32,
    ) -> Self {
        Self {
            output,
            block_height,
            tx_hash,
            output_index,
            is_spent: false,
            spent_at_height: None,
        }
    }

    /// Mark this UTXO as spent
    pub fn mark_spent(&mut self, spent_at_height: u64) {
        self.is_spent = true;
        self.spent_at_height = Some(spent_at_height);
    }

    /// Get the UTXO identifier
    pub fn id(&self) -> UtxoId {
        UtxoId::new(self.tx_hash.clone(), self.output_index)
    }
}

/// Blockchain statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockchainStats {
    /// Current blockchain height (number of blocks)
    pub height: u64,
    /// Hash of the latest block
    pub latest_block_hash: Hash256,
    /// Total number of transactions
    pub total_transactions: u64,
    /// Total number of UTXOs
    pub total_utxos: u64,
    /// Total supply of coins
    pub total_supply: u64,
    /// Current difficulty
    pub current_difficulty: u32,
    /// Average block time in seconds
    pub average_block_time: f64,
    /// Total network hash rate (estimated)
    pub estimated_hash_rate: f64,
    /// Blockchain size in bytes
    pub blockchain_size: u64,
    /// Last updated timestamp
    pub last_updated: DateTime<Utc>,
}

impl Default for BlockchainStats {
    fn default() -> Self {
        Self {
            height: 0,
            latest_block_hash: Hash256::zero(),
            total_transactions: 0,
            total_utxos: 0,
            total_supply: 0,
            current_difficulty: 1,
            average_block_time: 600.0, // 10 minutes
            estimated_hash_rate: 0.0,
            blockchain_size: 0,
            last_updated: Utc::now(),
        }
    }
}

/// Blockchain configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockchainConfig {
    /// Target block time in seconds
    pub target_block_time: u64,
    /// Difficulty adjustment interval (in blocks)
    pub difficulty_adjustment_interval: u64,
    /// Maximum block size in bytes
    pub max_block_size: u64,
    /// Block reward amount
    pub block_reward: u64,
    /// Halving interval (in blocks)
    pub halving_interval: u64,
    /// Maximum transactions per block
    pub max_transactions_per_block: u32,
    /// Minimum transaction fee
    pub min_transaction_fee: u64,
    /// Genesis block timestamp
    pub genesis_timestamp: DateTime<Utc>,
    /// Initial difficulty
    pub initial_difficulty: u32,
}

impl Default for BlockchainConfig {
    fn default() -> Self {
        Self {
            target_block_time: 600, // 10 minutes
            difficulty_adjustment_interval: 2016, // ~2 weeks
            max_block_size: 1_000_000, // 1MB
            block_reward: 5_000_000_000, // 50 units
            halving_interval: 210_000, // ~4 years
            max_transactions_per_block: 1000,
            min_transaction_fee: 1000, // 0.00001 units
            genesis_timestamp: DateTime::parse_from_rfc3339("2024-01-01T00:00:00Z")
                .unwrap()
                .with_timezone(&Utc),
            initial_difficulty: 1,
        }
    }
}

/// Main blockchain structure
#[derive(Debug)]
pub struct Blockchain {
    /// Blockchain configuration
    pub config: BlockchainConfig,
    /// Chain of blocks (in memory cache)
    blocks: Vec<Block>,
    /// UTXO set for fast transaction validation
    utxo_set: HashMap<UtxoId, UtxoEntry>,
    /// Transaction pool for pending transactions
    transaction_pool: HashMap<Hash256, Transaction>,
    /// Block index for fast lookup by hash
    block_index: HashMap<Hash256, u64>,
    /// Persistent storage backend
    storage: Option<Arc<PersistentStorage>>,
    /// Blockchain statistics
    stats: BlockchainStats,
    /// Orphaned blocks (blocks without valid parent)
    orphaned_blocks: HashMap<Hash256, Block>,
    /// Recent block times for difficulty adjustment
    recent_block_times: VecDeque<DateTime<Utc>>,
}

impl Blockchain {
    /// Create a new blockchain with genesis block
    pub fn new(config: BlockchainConfig, genesis_address: crate::crypto::Address) -> Result<Self> {
        let mut blockchain = Self {
            config: config.clone(),
            blocks: Vec::new(),
            utxo_set: HashMap::new(),
            transaction_pool: HashMap::new(),
            block_index: HashMap::new(),
            storage: None,
            stats: BlockchainStats::default(),
            orphaned_blocks: HashMap::new(),
            recent_block_times: VecDeque::new(),
        };
        
        // Create and add genesis block
        let genesis_block = Block::genesis(genesis_address, config.block_reward);
        blockchain.add_genesis_block(genesis_block)?;
        
        Ok(blockchain)
    }

    /// Create blockchain with persistent storage
    pub fn with_storage(
        config: BlockchainConfig,
        storage: Arc<PersistentStorage>,
        genesis_address: crate::crypto::Address,
    ) -> Result<Self> {
        let mut blockchain = Self::new(config, genesis_address)?;
        blockchain.storage = Some(storage);
        
        // Load existing blockchain from storage if available
        if let Some(ref storage) = blockchain.storage {
            blockchain.load_from_storage()?;
        }
        
        Ok(blockchain)
    }

    /// Load blockchain state from persistent storage
    fn load_from_storage(&mut self) -> Result<()> {
        if let Some(ref storage) = self.storage {
            // Load blocks from storage
            let stored_blocks = storage.load_all_blocks()?;
            
            for block in stored_blocks {
                self.add_block_internal(block, false)?;
            }
            
            // Rebuild UTXO set
            self.rebuild_utxo_set()?;
            
            // Update statistics
            self.update_stats();
        }
        
        Ok(())
    }

    /// Add the genesis block
    fn add_genesis_block(&mut self, genesis_block: Block) -> Result<()> {
        if !genesis_block.is_genesis() {
            return Err(BlockchainError::InvalidGenesisBlock.into());
        }
        
        // Add genesis block
        self.add_block_internal(genesis_block, true)?;
        
        Ok(())
    }

    /// Add a new block to the blockchain
    pub fn add_block(&mut self, mut block: Block) -> Result<()> {
        // Validate the block
        self.validate_block(&block)?;
        
        // Mine the block if not already mined
        if !block.header.meets_difficulty_target() {
            block.mine(None)?;
        }
        
        // Add to blockchain
        self.add_block_internal(block, true)?;
        
        Ok(())
    }

    /// Internal method to add a block
    fn add_block_internal(&mut self, block: Block, update_utxo: bool) -> Result<()> {
        let block_hash = block.hash();
        let block_height = block.index;
        
        // Update UTXO set if requested
        if update_utxo {
            self.apply_block_to_utxo_set(&block)?;
        }
        
        // Remove transactions from pool
        for tx in &block.transactions {
            self.transaction_pool.remove(&tx.hash());
        }
        
        // Add to block index
        self.block_index.insert(block_hash.clone(), block_height);
        
        // Add to blocks
        self.blocks.push(block);
        
        // Update recent block times
        if let Some(latest_block) = self.blocks.last() {
            self.recent_block_times.push_back(latest_block.header.timestamp);
            if self.recent_block_times.len() > 10 {
                self.recent_block_times.pop_front();
            }
        }
        
        // Persist to storage
        if let Some(ref storage) = self.storage {
            if let Some(latest_block) = self.blocks.last() {
                storage.store_block(latest_block)?;
            }
        }
        
        // Update statistics
        self.update_stats();
        
        Ok(())
    }

    /// Validate a block before adding it to the chain
    pub fn validate_block(&self, block: &Block) -> Result<()> {
        // Get previous block for validation
        let previous_block = if block.index == 0 {
            None
        } else {
            self.get_block_by_index(block.index - 1)
        };
        
        // Convert UTXO set to the format expected by block validation
        let utxo_map: HashMap<String, TransactionOutput> = self.utxo_set
            .iter()
            .map(|(id, entry)| (id.to_string(), entry.output.clone()))
            .collect();
        
        // Validate the block
        block.validate(previous_block, &utxo_map)?;
        
        // Additional blockchain-specific validations
        self.validate_block_difficulty(block)?;
        self.validate_block_timestamp(block)?;
        
        Ok(())
    }

    /// Validate block difficulty
    fn validate_block_difficulty(&self, block: &Block) -> Result<()> {
        let expected_difficulty = self.calculate_next_difficulty();
        
        if block.header.difficulty != expected_difficulty {
            return Err(ValidationError::InvalidDifficulty(
                format!("Expected {}, got {}", expected_difficulty, block.header.difficulty)
            ).into());
        }
        
        Ok(())
    }

    /// Validate block timestamp
    fn validate_block_timestamp(&self, block: &Block) -> Result<()> {
        let now = Utc::now();
        
        // Block timestamp cannot be too far in the future
        if block.header.timestamp > now + chrono::Duration::hours(2) {
            return Err(ValidationError::InvalidTimestamp(
                "Block timestamp too far in future".to_string()
            ).into());
        }
        
        // Block timestamp must be after previous block
        if let Some(previous_block) = self.get_latest_block() {
            if block.header.timestamp <= previous_block.header.timestamp {
                return Err(ValidationError::InvalidTimestamp(
                    "Block timestamp must be after previous block".to_string()
                ).into());
            }
        }
        
        Ok(())
    }

    /// Apply block transactions to UTXO set
    fn apply_block_to_utxo_set(&mut self, block: &Block) -> Result<()> {
        for tx in &block.transactions {
            // Remove spent UTXOs
            for input in &tx.inputs {
                if !input.is_coinbase() {
                    let utxo_id = UtxoId::new(input.previous_tx_hash.clone(), input.output_index);
                    if let Some(mut utxo_entry) = self.utxo_set.remove(&utxo_id) {
                        utxo_entry.mark_spent(block.index);
                        // Optionally keep spent UTXOs for historical tracking
                    } else {
                        return Err(ValidationError::UtxoNotFound(utxo_id.to_string()).into());
                    }
                }
            }
            
            // Add new UTXOs
            for (output_index, output) in tx.outputs.iter().enumerate() {
                let utxo_id = UtxoId::new(tx.hash(), output_index as u32);
                let utxo_entry = UtxoEntry::new(
                    output.clone(),
                    block.index,
                    tx.hash(),
                    output_index as u32,
                );
                self.utxo_set.insert(utxo_id, utxo_entry);
            }
        }
        
        Ok(())
    }

    /// Rebuild UTXO set from scratch
    fn rebuild_utxo_set(&mut self) -> Result<()> {
        self.utxo_set.clear();
        
        // Clone the blocks to avoid borrowing conflicts
        let blocks = self.blocks.clone();
        for block in &blocks {
            self.apply_block_to_utxo_set(block)?;
        }
        
        Ok(())
    }

    /// Calculate the next difficulty based on recent block times
    pub fn calculate_next_difficulty(&self) -> u32 {
        if self.blocks.len() < self.config.difficulty_adjustment_interval as usize {
            return self.config.initial_difficulty;
        }
        
        let adjustment_interval = self.config.difficulty_adjustment_interval as usize;
        let current_height = self.blocks.len();
        
        // Only adjust at specific intervals
        if current_height % adjustment_interval != 0 {
            return self.get_latest_block()
                .map(|b| b.header.difficulty)
                .unwrap_or(self.config.initial_difficulty);
        }
        
        // Calculate time taken for the last interval
        let start_block = &self.blocks[current_height - adjustment_interval];
        let end_block = &self.blocks[current_height - 1];
        
        let time_taken = end_block.header.timestamp
            .signed_duration_since(start_block.header.timestamp)
            .num_seconds() as f64;
        
        let expected_time = (adjustment_interval as f64) * (self.config.target_block_time as f64);
        let ratio = time_taken / expected_time;
        
        // Limit adjustment to prevent extreme changes
        let adjustment_factor = ratio.max(0.25).min(4.0);
        
        let current_difficulty = end_block.header.difficulty as f64;
        let new_difficulty = (current_difficulty / adjustment_factor).round() as u32;
        
        // Ensure minimum difficulty
        new_difficulty.max(1)
    }

    /// Get block by hash
    pub fn get_block_by_hash(&self, hash: &Hash256) -> Option<&Block> {
        if let Some(&index) = self.block_index.get(hash) {
            self.blocks.get(index as usize)
        } else {
            None
        }
    }

    /// Get block by index
    pub fn get_block_by_index(&self, index: u64) -> Option<&Block> {
        self.blocks.get(index as usize)
    }

    /// Get the latest block
    pub fn get_latest_block(&self) -> Option<&Block> {
        self.blocks.last()
    }

    /// Get blockchain height
    pub fn height(&self) -> u64 {
        self.blocks.len() as u64
    }

    /// Get blockchain statistics
    pub fn get_stats(&self) -> &BlockchainStats {
        &self.stats
    }

    /// Update blockchain statistics
    fn update_stats(&mut self) {
        self.stats.height = self.blocks.len() as u64;
        
        if let Some(latest_block) = self.get_latest_block() {
            let latest_hash = latest_block.hash();
            let latest_difficulty = latest_block.header.difficulty;
            self.stats.latest_block_hash = latest_hash;
            self.stats.current_difficulty = latest_difficulty;
        }
        
        self.stats.total_transactions = self.blocks.iter()
            .map(|b| b.transactions.len() as u64)
            .sum();
        
        self.stats.total_utxos = self.utxo_set.len() as u64;
        
        self.stats.total_supply = self.utxo_set.values()
            .map(|utxo| utxo.output.amount)
            .sum();
        
        // Calculate average block time
        if self.recent_block_times.len() > 1 {
            let times: Vec<_> = self.recent_block_times.iter().collect();
            let total_time: i64 = times.windows(2)
                .map(|window| {
                    window[1].signed_duration_since(*window[0]).num_seconds()
                })
                .sum();
            
            self.stats.average_block_time = total_time as f64 / (self.recent_block_times.len() - 1) as f64;
        }
        
        // Estimate hash rate (simplified)
        if self.stats.average_block_time > 0.0 {
            let difficulty = self.stats.current_difficulty as f64;
            self.stats.estimated_hash_rate = difficulty * 2_f64.powi(32) / self.stats.average_block_time;
        }
        
        self.stats.last_updated = Utc::now();
    }

    /// Add transaction to the pool
    pub fn add_transaction_to_pool(&mut self, transaction: Transaction) -> Result<()> {
        // Validate transaction
        let utxo_map: HashMap<String, TransactionOutput> = self.utxo_set
            .iter()
            .map(|(id, entry)| (id.to_string(), entry.output.clone()))
            .collect();
        
        transaction.validate(&utxo_map)?;
        
        // Check for double spending
        for input in &transaction.inputs {
            if !input.is_coinbase() {
                let utxo_id = UtxoId::new(input.previous_tx_hash.clone(), input.output_index);
                if !self.utxo_set.contains_key(&utxo_id) {
                    return Err(ValidationError::UtxoNotFound(utxo_id.to_string()).into());
                }
            }
        }
        
        // Add to pool
        let tx_hash = transaction.hash();
        self.transaction_pool.insert(tx_hash, transaction);
        
        Ok(())
    }

    /// Get pending transactions from pool
    pub fn get_pending_transactions(&self) -> Vec<&Transaction> {
        self.transaction_pool.values().collect()
    }

    /// Get transaction by hash (from blockchain or pool)
    pub fn get_transaction(&self, tx_hash: &Hash256) -> Option<&Transaction> {
        // First check transaction pool
        if let Some(tx) = self.transaction_pool.get(tx_hash) {
            return Some(tx);
        }
        
        // Then check blockchain
        for block in &self.blocks {
            if let Some(tx) = block.get_transaction(tx_hash) {
                return Some(tx);
            }
        }
        
        None
    }

    /// Create a new block with pending transactions
    pub fn create_block(&mut self, miner_address: crate::crypto::Address) -> Result<Block> {
        let previous_hash = self.get_latest_block()
            .map(|b| b.hash())
            .unwrap_or_else(Hash256::zero);
        
        let next_index = self.height();
        let difficulty = self.calculate_next_difficulty();
        
        // Select transactions from pool
        let mut transactions = Vec::new();
        
        // Add coinbase transaction
        let block_reward = self.calculate_block_reward(next_index);
        let coinbase_tx = Transaction::coinbase(miner_address, block_reward, next_index);
        transactions.push(coinbase_tx);
        
        // Add pending transactions (up to limit)
        let max_tx = (self.config.max_transactions_per_block - 1) as usize; // -1 for coinbase
        for tx in self.transaction_pool.values().take(max_tx) {
            transactions.push(tx.clone());
        }
        
        // Create block
        let block = Block::new(next_index, previous_hash, transactions, difficulty);
        
        Ok(block)
    }

    /// Calculate block reward for given height
    fn calculate_block_reward(&self, height: u64) -> u64 {
        let halvings = height / self.config.halving_interval;
        let reward = self.config.block_reward >> halvings; // Halve for each halving period
        reward.max(1) // Minimum reward of 1 unit
    }

    /// Get UTXO by ID
    pub fn get_utxo(&self, utxo_id: &UtxoId) -> Option<&UtxoEntry> {
        self.utxo_set.get(utxo_id)
    }

    /// Get all UTXOs for an address
    pub fn get_utxos_for_address(&self, address: &crate::crypto::Address) -> Vec<&UtxoEntry> {
        self.utxo_set.values()
            .filter(|utxo| utxo.output.recipient == *address)
            .collect()
    }

    /// Get balance for an address
    pub fn get_balance(&self, address: &crate::crypto::Address) -> u64 {
        self.get_utxos_for_address(address)
            .iter()
            .map(|utxo| utxo.output.amount)
            .sum()
    }

    /// Get the current difficulty
    pub fn get_current_difficulty(&self) -> u32 {
        self.stats.current_difficulty
    }

    /// Get blocks until next difficulty adjustment
    pub fn blocks_until_difficulty_adjustment(&self) -> u64 {
        let current_height = self.height();
        let interval = self.config.difficulty_adjustment_interval;
        interval - (current_height % interval)
    }

    /// Get all UTXOs
    pub fn get_all_utxos(&self) -> Vec<&UtxoEntry> {
        self.utxo_set.values().collect()
    }

    /// Find transaction in blockchain and return block with transaction index
    pub fn find_transaction_in_block(&self, tx_hash: &Hash256) -> Option<(&Block, usize)> {
        for block in &self.blocks {
            for (index, tx) in block.transactions.iter().enumerate() {
                if &tx.hash() == tx_hash {
                    return Some((block, index));
                }
            }
        }
        None
    }

    /// Verify the entire blockchain
    pub fn verify_chain(&self) -> Result<()> {
        for (i, block) in self.blocks.iter().enumerate() {
            let previous_block = if i == 0 { None } else { Some(&self.blocks[i - 1]) };
            
            let utxo_map: HashMap<String, TransactionOutput> = self.utxo_set
                .iter()
                .map(|(id, entry)| (id.to_string(), entry.output.clone()))
                .collect();
            
            block.validate(previous_block, &utxo_map)?;
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::{Address, PublicKey, SignatureAlgorithm};

    fn create_test_address() -> Address {
        let public_key = PublicKey::new(SignatureAlgorithm::EcdsaSecp256k1, vec![1, 2, 3, 4, 5]);
        Address::from_public_key(&public_key)
    }

    #[test]
    fn test_blockchain_creation() {
        let config = BlockchainConfig::default();
        let genesis_address = create_test_address();
        let blockchain = Blockchain::new(config, genesis_address).unwrap();
        
        assert_eq!(blockchain.height(), 1); // Genesis block
        assert!(blockchain.get_latest_block().unwrap().is_genesis());
    }

    #[test]
    fn test_utxo_id_string_conversion() {
        let tx_hash = Hash256::from_hex("1234567890abcdef").unwrap();
        let utxo_id = UtxoId::new(tx_hash.clone(), 0);
        
        let string_repr = utxo_id.to_string();
        let parsed_id = UtxoId::from_string(&string_repr).unwrap();
        
        assert_eq!(utxo_id, parsed_id);
    }

    #[test]
    fn test_difficulty_calculation() {
        let config = BlockchainConfig::default();
        let genesis_address = create_test_address();
        let blockchain = Blockchain::new(config, genesis_address).unwrap();
        
        let difficulty = blockchain.calculate_next_difficulty();
        assert_eq!(difficulty, 1); // Should return initial difficulty
    }

    #[test]
    fn test_balance_calculation() {
        let config = BlockchainConfig::default();
        let genesis_address = create_test_address();
        let blockchain = Blockchain::new(config.clone(), genesis_address.clone()).unwrap();
        
        let balance = blockchain.get_balance(&genesis_address);
        assert_eq!(balance, config.block_reward); // Genesis block reward
    }

    #[test]
    fn test_block_reward_halving() {
        let mut config = BlockchainConfig::default();
        config.halving_interval = 10; // Small interval for testing
        
        let genesis_address = create_test_address();
        let blockchain = Blockchain::new(config.clone(), genesis_address).unwrap();
        
        // Test rewards at different heights
        assert_eq!(blockchain.calculate_block_reward(0), config.block_reward);
        assert_eq!(blockchain.calculate_block_reward(10), config.block_reward / 2);
        assert_eq!(blockchain.calculate_block_reward(20), config.block_reward / 4);
    }

    #[test]
    fn test_transaction_pool() {
        let config = BlockchainConfig::default();
        let genesis_address = create_test_address();
        let mut blockchain = Blockchain::new(config, genesis_address).unwrap();
        
        // Create a test transaction
        let input = TransactionInput::new(Hash256::zero(), 0, None, None);
        let output = TransactionOutput::new(1000, create_test_address());
        let tx = Transaction::new(vec![input], vec![output]);
        
        // Note: This will fail validation due to missing UTXO, but tests the pool mechanism
        assert!(blockchain.add_transaction_to_pool(tx).is_err());
    }
}