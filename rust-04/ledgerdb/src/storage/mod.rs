//! Persistent storage layer for the blockchain.
//!
//! This module provides persistent storage capabilities using the `sled` embedded database,
//! including block storage, transaction indexing, and UTXO set persistence.

use crate::core::{Block, Transaction, UtxoEntry, UtxoId};
use crate::crypto::Hash256;
use crate::error::{Result, StorageError};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sled::{Db, Tree};
use std::path::Path;
use std::sync::Arc;

/// Storage keys for different data types
mod keys {
    pub const BLOCKS: &[u8] = b"blocks";
    pub const TRANSACTIONS: &[u8] = b"transactions";
    pub const UTXOS: &[u8] = b"utxos";
    pub const METADATA: &[u8] = b"metadata";
    pub const JOURNAL: &[u8] = b"journal";
    pub const BLOCK_INDEX: &[u8] = b"block_index";
    pub const TX_INDEX: &[u8] = b"tx_index";
    pub const ADDRESS_INDEX: &[u8] = b"address_index";
}

/// Blockchain metadata stored in the database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockchainMetadata {
    /// Current blockchain height
    pub height: u64,
    /// Hash of the latest block
    pub latest_block_hash: Hash256,
    /// Total number of transactions
    pub total_transactions: u64,
    /// Database version for migrations
    pub db_version: u32,
    /// Last updated timestamp
    pub last_updated: DateTime<Utc>,
    /// Genesis block hash
    pub genesis_hash: Hash256,
    /// Total supply
    pub total_supply: u64,
}

impl Default for BlockchainMetadata {
    fn default() -> Self {
        Self {
            height: 0,
            latest_block_hash: Hash256::zero(),
            total_transactions: 0,
            db_version: 1,
            last_updated: Utc::now(),
            genesis_hash: Hash256::zero(),
            total_supply: 0,
        }
    }
}

/// Journal entry for atomic operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JournalEntry {
    /// Unique journal entry ID
    pub id: u64,
    /// Timestamp of the operation
    pub timestamp: DateTime<Utc>,
    /// Type of operation
    pub operation: JournalOperation,
    /// Whether the operation was committed
    pub committed: bool,
    /// Block height at the time of operation
    pub block_height: u64,
}

/// Types of journal operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum JournalOperation {
    /// Block addition
    AddBlock {
        block_hash: Hash256,
        block_index: u64,
    },
    /// Block removal (for rollbacks)
    RemoveBlock {
        block_hash: Hash256,
        block_index: u64,
    },
    /// UTXO creation
    CreateUtxo {
        utxo_id: UtxoId,
        utxo_entry: UtxoEntry,
    },
    /// UTXO spending
    SpendUtxo {
        utxo_id: UtxoId,
        spent_at_height: u64,
    },
    /// Transaction addition
    AddTransaction {
        tx_hash: Hash256,
        block_hash: Hash256,
    },
    /// Metadata update
    UpdateMetadata {
        old_metadata: BlockchainMetadata,
        new_metadata: BlockchainMetadata,
    },
}

/// Persistent storage implementation
#[derive(Debug)]
pub struct PersistentStorage {
    /// Main database instance
    db: Db,
    /// Blocks tree
    blocks: Tree,
    /// Transactions tree
    transactions: Tree,
    /// UTXOs tree
    utxos: Tree,
    /// Metadata tree
    metadata: Tree,
    /// Journal tree for atomic operations
    journal: Tree,
    /// Block index (hash -> height)
    block_index: Tree,
    /// Transaction index (hash -> block_hash)
    tx_index: Tree,
    /// Address index (address -> [utxo_ids])
    address_index: Tree,
    /// Next journal ID
    next_journal_id: u64,
}

impl PersistentStorage {
    /// Create a new persistent storage instance
    pub fn new<P: AsRef<Path>>(db_path: P) -> Result<Self> {
        let db = sled::open(db_path)
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;
        
        let blocks = db.open_tree(keys::BLOCKS)
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;
        let transactions = db.open_tree(keys::TRANSACTIONS)
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;
        let utxos = db.open_tree(keys::UTXOS)
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;
        let metadata = db.open_tree(keys::METADATA)
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;
        let journal = db.open_tree(keys::JOURNAL)
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;
        let block_index = db.open_tree(keys::BLOCK_INDEX)
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;
        let tx_index = db.open_tree(keys::TX_INDEX)
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;
        let address_index = db.open_tree(keys::ADDRESS_INDEX)
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;
        
        // Get next journal ID
        let next_journal_id = journal.len() as u64;
        
        Ok(Self {
            db,
            blocks,
            transactions,
            utxos,
            metadata,
            journal,
            block_index,
            tx_index,
            address_index,
            next_journal_id,
        })
    }

    /// Load or create blockchain metadata
    pub fn load_or_create_blockchain(&self) -> Result<BlockchainMetadata> {
        match self.load_metadata() {
            Ok(metadata) => Ok(metadata),
            Err(StorageError::NotFound(_)) => {
                let metadata = BlockchainMetadata::default();
                self.store_metadata(&metadata)?;
                Ok(metadata)
            }
            Err(e) => Err(e),
        }
    }

    /// Load blockchain metadata
    pub fn load_metadata(&self) -> Result<BlockchainMetadata> {
        let key = b"blockchain_metadata";
        
        match self.metadata.get(key)
            .map_err(|e| StorageError::DatabaseError(e.to_string()))? {
            Some(data) => {
                bincode::deserialize(data.as_ref())
                    .map_err(|e| StorageError::SerializationError(e.to_string()))
            }
            None => Err(StorageError::NotFound("blockchain metadata".to_string())),
        }
    }

    /// Store blockchain metadata
    pub fn store_metadata(&self, metadata: &BlockchainMetadata) -> Result<()> {
        let key = b"blockchain_metadata";
        let data = bincode::serialize(metadata)
            .map_err(|e| StorageError::SerializationError(e.to_string()))?;
        
        self.metadata.insert(key, data)
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;
        
        self.db.flush()
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;
        
        Ok(())
    }

    /// Store a block
    pub fn store_block(&self, block: &Block) -> Result<()> {
        let block_hash = block.hash();
        let block_key = block_hash.to_hex();
        
        // Start journal entry
        let journal_entry = self.create_journal_entry(JournalOperation::AddBlock {
            block_hash: block_hash.clone(),
            block_index: block.index,
        })?;
        
        // Serialize block
        let block_data = bincode::serialize(block)
            .map_err(|e| StorageError::SerializationError(e.to_string()))?;
        
        // Store block
        self.blocks.insert(block_key.as_bytes(), block_data)
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;
        
        // Update block index
        let height_key = block.index.to_be_bytes();
        self.block_index.insert(&height_key, block_hash.to_hex().as_bytes())
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;
        
        // Store transactions
        for tx in &block.transactions {
            self.store_transaction(tx, &block_hash)?;
        }
        
        // Commit journal entry
        self.commit_journal_entry(journal_entry.id)?;
        
        // Flush to disk
        self.db.flush()
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;
        
        Ok(())
    }

    /// Load a block by hash
    pub fn load_block_by_hash(&self, block_hash: &Hash256) -> Result<Block> {
        let block_key = block_hash.to_hex();
        
        match self.blocks.get(block_key.as_bytes())
            .map_err(|e| StorageError::DatabaseError(e.to_string()))? {
            Some(data) => {
                bincode::deserialize(data.as_ref())
                    .map_err(|e| StorageError::SerializationError(e.to_string()))
            }
            None => Err(StorageError::NotFound(format!("block {}", block_hash.to_hex()))),
        }
    }

    /// Load a block by height
    pub fn load_block_by_height(&self, height: u64) -> Result<Block> {
        let height_key = height.to_be_bytes();
        
        match self.block_index.get(&height_key)
            .map_err(|e| StorageError::DatabaseError(e.to_string()))? {
            Some(hash_data) => {
                let hash_str = String::from_utf8(hash_data.to_vec())
                    .map_err(|e| StorageError::SerializationError(e.to_string()))?;
                let block_hash = Hash256::from_hex(&hash_str)
                    .map_err(|e| StorageError::SerializationError(e.to_string()))?;
                self.load_block_by_hash(&block_hash)
            }
            None => Err(StorageError::NotFound(format!("block at height {}", height))),
        }
    }

    /// Load all blocks (for blockchain reconstruction)
    pub fn load_all_blocks(&self) -> Result<Vec<Block>> {
        let mut blocks = Vec::new();
        
        // Iterate through block index in order
        for result in self.block_index.iter() {
            let (height_bytes, hash_bytes) = result
                .map_err(|e| StorageError::DatabaseError(e.to_string()))?;
            
            let hash_str = String::from_utf8(hash_bytes.to_vec())
                .map_err(|e| StorageError::SerializationError(e.to_string()))?;
            let block_hash = Hash256::from_hex(&hash_str)
                .map_err(|e| StorageError::SerializationError(e.to_string()))?;
            
            let block = self.load_block_by_hash(&block_hash)?;
            blocks.push(block);
        }
        
        // Sort by height to ensure correct order
        blocks.sort_by_key(|b| b.index);
        
        Ok(blocks)
    }

    /// Store a transaction
    pub fn store_transaction(&self, transaction: &Transaction, block_hash: &Hash256) -> Result<()> {
        let tx_hash = transaction.hash();
        let tx_key = tx_hash.to_hex();
        
        // Serialize transaction
        let tx_data = bincode::serialize(transaction)
            .map_err(|e| StorageError::SerializationError(e.to_string()))?;
        
        // Store transaction
        self.transactions.insert(tx_key.as_bytes(), tx_data)
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;
        
        // Update transaction index
        self.tx_index.insert(tx_hash.to_hex().as_bytes(), block_hash.to_hex().as_bytes())
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;
        
        Ok(())
    }

    /// Load a transaction by hash
    pub fn load_transaction(&self, tx_hash: &Hash256) -> Result<Transaction> {
        let tx_key = tx_hash.to_hex();
        
        match self.transactions.get(tx_key.as_bytes())
            .map_err(|e| StorageError::DatabaseError(e.to_string()))? {
            Some(data) => {
                bincode::deserialize(data.as_ref())
                    .map_err(|e| StorageError::SerializationError(e.to_string()))
            }
            None => Err(StorageError::NotFound(format!("transaction {}", tx_hash.to_hex()))),
        }
    }

    /// Store UTXO
    pub fn store_utxo(&self, utxo_id: &UtxoId, utxo_entry: &UtxoEntry) -> Result<()> {
        let utxo_key = utxo_id.to_string();
        
        // Create journal entry
        let _journal_entry = self.create_journal_entry(JournalOperation::CreateUtxo {
            utxo_id: utxo_id.clone(),
            utxo_entry: utxo_entry.clone(),
        })?;
        
        // Serialize UTXO
        let utxo_data = bincode::serialize(utxo_entry)
            .map_err(|e| StorageError::SerializationError(e.to_string()))?;
        
        // Store UTXO
        self.utxos.insert(utxo_key.as_bytes(), utxo_data)
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;
        
        // Update address index
        self.update_address_index(&utxo_entry.output.recipient, utxo_id, true)?;
        
        Ok(())
    }

    /// Load UTXO
    pub fn load_utxo(&self, utxo_id: &UtxoId) -> Result<UtxoEntry> {
        let utxo_key = utxo_id.to_string();
        
        match self.utxos.get(utxo_key.as_bytes())
            .map_err(|e| StorageError::DatabaseError(e.to_string()))? {
            Some(data) => {
                bincode::deserialize(data.as_ref())
                    .map_err(|e| StorageError::SerializationError(e.to_string()))
            }
            None => Err(StorageError::NotFound(format!("UTXO {}", utxo_key))),
        }
    }

    /// Remove UTXO (when spent)
    pub fn remove_utxo(&self, utxo_id: &UtxoId, spent_at_height: u64) -> Result<()> {
        let utxo_key = utxo_id.to_string();
        
        // Load UTXO first to get address for index update
        let utxo_entry = self.load_utxo(utxo_id)?;
        
        // Create journal entry
        let _journal_entry = self.create_journal_entry(JournalOperation::SpendUtxo {
            utxo_id: utxo_id.clone(),
            spent_at_height,
        })?;
        
        // Remove UTXO
        self.utxos.remove(utxo_key.as_bytes())
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;
        
        // Update address index
        self.update_address_index(&utxo_entry.output.recipient_address, utxo_id, false)?;
        
        Ok(())
    }

    /// Load all UTXOs for an address
    pub fn load_utxos_for_address(&self, address: &crate::crypto::Address) -> Result<Vec<UtxoEntry>> {
        let address_key = address.to_string();
        
        match self.address_index.get(address_key.as_bytes())
            .map_err(|e| StorageError::DatabaseError(e.to_string()))? {
            Some(data) => {
                let utxo_ids: Vec<UtxoId> = bincode::deserialize(data.as_ref())
                    .map_err(|e| StorageError::SerializationError(e.to_string()))?;
                
                let mut utxos = Vec::new();
                for utxo_id in utxo_ids {
                    if let Ok(utxo) = self.load_utxo(&utxo_id) {
                        utxos.push(utxo);
                    }
                }
                
                Ok(utxos)
            }
            None => Ok(Vec::new()),
        }
    }

    /// Update address index
    fn update_address_index(
        &self,
        address: &crate::crypto::Address,
        utxo_id: &UtxoId,
        add: bool,
    ) -> Result<()> {
        let address_key = address.to_string();
        
        let mut utxo_ids: Vec<UtxoId> = match self.address_index.get(address_key.as_bytes())
            .map_err(|e| StorageError::DatabaseError(e.to_string()))? {
            Some(data) => bincode::deserialize(data.as_ref())
                .map_err(|e| StorageError::SerializationError(e.to_string()))?,
            None => Vec::new(),
        };
        
        if add {
            if !utxo_ids.contains(utxo_id) {
                utxo_ids.push(utxo_id.clone());
            }
        } else {
            utxo_ids.retain(|id| id != utxo_id);
        }
        
        let data = bincode::serialize(&utxo_ids)
            .map_err(|e| StorageError::SerializationError(e.to_string()))?;
        
        self.address_index.insert(address_key.as_bytes(), data)
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;
        
        Ok(())
    }

    /// Create a journal entry
    fn create_journal_entry(&self, operation: JournalOperation) -> Result<JournalEntry> {
        let entry = JournalEntry {
            id: self.next_journal_id,
            timestamp: Utc::now(),
            operation,
            committed: false,
            block_height: 0, // Will be updated when committed
        };
        
        let entry_data = bincode::serialize(&entry)
            .map_err(|e| StorageError::SerializationError(e.to_string()))?;
        
        self.journal.insert(&entry.id.to_be_bytes(), entry_data)
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;
        
        Ok(entry)
    }

    /// Commit a journal entry
    fn commit_journal_entry(&self, journal_id: u64) -> Result<()> {
        let key = journal_id.to_be_bytes();
        
        if let Some(data) = self.journal.get(&key)
            .map_err(|e| StorageError::DatabaseError(e.to_string()))? {
            let mut entry: JournalEntry = bincode::deserialize(data.as_ref())
                .map_err(|e| StorageError::SerializationError(e.to_string()))?;
            
            entry.committed = true;
            
            let updated_data = bincode::serialize(&entry)
                .map_err(|e| StorageError::SerializationError(e.to_string()))?;
            
            self.journal.insert(&key, updated_data)
                .map_err(|e| StorageError::DatabaseError(e.to_string()))?;
        }
        
        Ok(())
    }

    /// Get database statistics
    pub fn get_stats(&self) -> Result<StorageStats> {
        let blocks_count = self.blocks.len();
        let transactions_count = self.transactions.len();
        let utxos_count = self.utxos.len();
        let journal_entries = self.journal.len();
        
        // Calculate approximate database size
        let db_size = self.db.size_on_disk()
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;
        
        Ok(StorageStats {
            blocks_count,
            transactions_count,
            utxos_count,
            journal_entries,
            database_size: db_size,
            last_updated: Utc::now(),
        })
    }

    /// Compact the database
    pub fn compact(&self) -> Result<()> {
        // Clean up old journal entries
        let mut to_remove = Vec::new();
        
        for result in self.journal.iter() {
            let (key, data) = result
                .map_err(|e| StorageError::DatabaseError(e.to_string()))?;
            
            let entry: JournalEntry = bincode::deserialize(&data)
                .map_err(|e| StorageError::SerializationError(e.to_string()))?;
            
            // Remove committed entries older than 1 day
            if entry.committed && entry.timestamp < Utc::now() - chrono::Duration::days(1) {
                to_remove.push(key.to_vec());
            }
        }
        
        for key in to_remove {
            self.journal.remove(&key)
                .map_err(|e| StorageError::DatabaseError(e.to_string()))?;
        }
        
        // Flush changes
        self.db.flush()
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;
        
        Ok(())
    }

    /// Close the database
    pub fn close(&self) -> Result<()> {
        self.db.flush()
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;
        Ok(())
    }
}

/// Storage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageStats {
    /// Number of blocks stored
    pub blocks_count: usize,
    /// Number of transactions stored
    pub transactions_count: usize,
    /// Number of UTXOs stored
    pub utxos_count: usize,
    /// Number of journal entries
    pub journal_entries: usize,
    /// Database size on disk in bytes
    pub database_size: u64,
    /// Last updated timestamp
    pub last_updated: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::TransactionOutput;
    use crate::crypto::{Address, PublicKey, SignatureAlgorithm};
    use tempfile::TempDir;

    fn create_test_address() -> Address {
        let public_key = PublicKey::new(SignatureAlgorithm::EcdsaSecp256k1, vec![1, 2, 3, 4, 5]);
        Address::from_public_key(&public_key)
    }

    fn create_test_storage() -> (PersistentStorage, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let storage = PersistentStorage::new(temp_dir.path()).unwrap();
        (storage, temp_dir)
    }

    #[test]
    fn test_storage_creation() {
        let (_storage, _temp_dir) = create_test_storage();
        // Storage should be created successfully
    }

    #[test]
    fn test_metadata_storage() {
        let (storage, _temp_dir) = create_test_storage();
        
        let metadata = BlockchainMetadata {
            height: 100,
            latest_block_hash: Hash256::from_hex("1234567890abcdef").unwrap(),
            total_transactions: 500,
            db_version: 1,
            last_updated: Utc::now(),
            genesis_hash: Hash256::zero(),
            total_supply: 1000000,
        };
        
        storage.store_metadata(&metadata).unwrap();
        let loaded_metadata = storage.load_metadata().unwrap();
        
        assert_eq!(metadata.height, loaded_metadata.height);
        assert_eq!(metadata.latest_block_hash, loaded_metadata.latest_block_hash);
        assert_eq!(metadata.total_transactions, loaded_metadata.total_transactions);
    }

    #[test]
    fn test_utxo_storage() {
        let (storage, _temp_dir) = create_test_storage();
        
        let tx_hash = Hash256::from_hex("abcdef1234567890").unwrap();
        let utxo_id = UtxoId::new(tx_hash.clone(), 0);
        let output = TransactionOutput::new(1000, create_test_address());
        let utxo_entry = UtxoEntry::new(output, 1, tx_hash, 0);
        
        storage.store_utxo(&utxo_id, &utxo_entry).unwrap();
        let loaded_utxo = storage.load_utxo(&utxo_id).unwrap();
        
        assert_eq!(utxo_entry.output.amount, loaded_utxo.output.amount);
        assert_eq!(utxo_entry.block_height, loaded_utxo.block_height);
    }

    #[test]
    fn test_utxo_id_string_conversion() {
        let tx_hash = Hash256::from_hex("1234567890abcdef").unwrap();
        let utxo_id = UtxoId::new(tx_hash, 5);
        
        let string_repr = utxo_id.to_string();
        let parsed_id = UtxoId::from_string(&string_repr).unwrap();
        
        assert_eq!(utxo_id, parsed_id);
    }

    #[test]
    fn test_storage_stats() {
        let (storage, _temp_dir) = create_test_storage();
        
        let stats = storage.get_stats().unwrap();
        assert_eq!(stats.blocks_count, 0);
        assert_eq!(stats.transactions_count, 0);
        assert_eq!(stats.utxos_count, 0);
    }
}