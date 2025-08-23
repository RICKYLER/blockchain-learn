//! Transaction data structures and validation logic.
//!
//! This module defines the transaction types used in the LedgerDB blockchain,
//! including input/output structures, validation, and serialization.

use crate::crypto::{Address, Hash256, PublicKey, Signature};
use crate::error::{Result, ValidationError};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Transaction input referencing a previous output
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransactionInput {
    /// Hash of the transaction containing the output being spent
    pub previous_tx_hash: Hash256,
    /// Index of the output in the previous transaction
    pub output_index: u32,
    /// Script or signature proving ownership
    pub signature: Option<Signature>,
    /// Public key of the spender
    pub public_key: Option<PublicKey>,
    /// Sequence number for transaction ordering
    pub sequence: u32,
}

impl TransactionInput {
    /// Create a new transaction input
    pub fn new(
        previous_tx_hash: Hash256,
        output_index: u32,
        signature: Option<Signature>,
        public_key: Option<PublicKey>,
    ) -> Self {
        Self {
            previous_tx_hash,
            output_index,
            signature,
            public_key,
            sequence: u32::MAX, // Default to maximum sequence
        }
    }

    /// Create a coinbase input (for mining rewards)
    pub fn coinbase(block_height: u64) -> Self {
        Self {
            previous_tx_hash: Hash256::zero(),
            output_index: u32::MAX,
            signature: None,
            public_key: None,
            sequence: block_height as u32,
        }
    }

    /// Check if this is a coinbase input
    pub fn is_coinbase(&self) -> bool {
        self.previous_tx_hash == Hash256::zero() && self.output_index == u32::MAX
    }

    /// Validate the input structure
    pub fn validate(&self) -> Result<()> {
        if !self.is_coinbase() {
            if self.signature.is_none() {
                return Err(ValidationError::MissingSignature.into());
            }
            if self.public_key.is_none() {
                return Err(ValidationError::MissingPublicKey.into());
            }
        }
        Ok(())
    }
}

/// Transaction output defining where funds are sent
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransactionOutput {
    /// Amount being transferred (in smallest unit)
    pub amount: u64,
    /// Recipient address
    pub recipient: Address,
    /// Optional script for complex spending conditions
    pub script: Option<Vec<u8>>,
    /// Whether this output has been spent
    pub spent: bool,
    /// Block height when this output was created
    pub created_at_height: Option<u64>,
}

impl TransactionOutput {
    /// Create a new transaction output
    pub fn new(amount: u64, recipient: Address) -> Self {
        Self {
            amount,
            recipient,
            script: None,
            spent: false,
            created_at_height: None,
        }
    }

    /// Create an output with a custom script
    pub fn with_script(amount: u64, recipient: Address, script: Vec<u8>) -> Self {
        Self {
            amount,
            recipient,
            script: Some(script),
            spent: false,
            created_at_height: None,
        }
    }

    /// Mark this output as spent
    pub fn mark_spent(&mut self) {
        self.spent = true;
    }

    /// Check if this output can be spent
    pub fn is_spendable(&self) -> bool {
        !self.spent
    }

    /// Validate the output structure
    pub fn validate(&self) -> Result<()> {
        if self.amount == 0 {
            return Err(ValidationError::InvalidAmount("Amount cannot be zero".to_string()).into());
        }
        Ok(())
    }
}

/// Transaction fee calculation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransactionFee {
    /// Base fee per transaction
    pub base_fee: u64,
    /// Fee per byte of transaction data
    pub per_byte_fee: u64,
    /// Priority multiplier (1.0 = normal, 2.0 = high priority)
    pub priority_multiplier: f64,
}

impl Default for TransactionFee {
    fn default() -> Self {
        Self {
            base_fee: 1000,      // 0.001 units
            per_byte_fee: 10,    // 0.00001 units per byte
            priority_multiplier: 1.0,
        }
    }
}

/// Main transaction structure
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Transaction {
    /// Unique transaction identifier
    pub id: String,
    /// Transaction version for future upgrades
    pub version: u32,
    /// Transaction inputs
    pub inputs: Vec<TransactionInput>,
    /// Transaction outputs
    pub outputs: Vec<TransactionOutput>,
    /// Transaction fee information
    pub fee: TransactionFee,
    /// Lock time (block height or timestamp)
    pub lock_time: u64,
    /// Transaction timestamp
    pub timestamp: DateTime<Utc>,
    /// Optional transaction memo/data
    pub data: Option<Vec<u8>>,
    /// Transaction size in bytes (calculated)
    pub size: Option<usize>,
}

impl Transaction {
    /// Create a new transaction
    pub fn new(
        inputs: Vec<TransactionInput>,
        outputs: Vec<TransactionOutput>,
    ) -> Self {
        let id = Uuid::new_v4().to_string();
        let mut tx = Self {
            id,
            version: 1,
            inputs,
            outputs,
            fee: TransactionFee::default(),
            lock_time: 0,
            timestamp: Utc::now(),
            data: None,
            size: None,
        };
        tx.calculate_size();
        tx
    }

    /// Create a coinbase transaction (mining reward)
    pub fn coinbase(recipient: Address, amount: u64, block_height: u64) -> Self {
        let input = TransactionInput::coinbase(block_height);
        let output = TransactionOutput::new(amount, recipient);
        
        let mut tx = Self {
            id: format!("coinbase_{}", block_height),
            version: 1,
            inputs: vec![input],
            outputs: vec![output],
            fee: TransactionFee {
                base_fee: 0,
                per_byte_fee: 0,
                priority_multiplier: 1.0,
            },
            lock_time: 0,
            timestamp: Utc::now(),
            data: Some(format!("Block {} mining reward", block_height).into_bytes()),
            size: None,
        };
        tx.calculate_size();
        tx
    }

    /// Calculate and set the transaction size
    pub fn calculate_size(&mut self) {
        let serialized = bincode::serialize(self).unwrap_or_default();
        self.size = Some(serialized.len());
    }

    /// Get the transaction hash
    pub fn hash(&self) -> Hash256 {
        let mut tx_for_hash = self.clone();
        // Remove signatures for hash calculation
        for input in &mut tx_for_hash.inputs {
            input.signature = None;
        }
        
        let serialized = bincode::serialize(&tx_for_hash).unwrap_or_default();
        crate::crypto::hash_data(&serialized)
    }

    /// Get total input amount
    pub fn total_input_amount(&self, utxo_set: &HashMap<String, TransactionOutput>) -> u64 {
        self.inputs.iter()
            .filter_map(|input| {
                if input.is_coinbase() {
                    None
                } else {
                    let key = format!("{}:{}", input.previous_tx_hash, input.output_index);
                    utxo_set.get(&key).map(|output| output.amount)
                }
            })
            .sum()
    }

    /// Get total output amount
    pub fn total_output_amount(&self) -> u64 {
        self.outputs.iter().map(|output| output.amount).sum()
    }

    /// Calculate transaction fee
    pub fn calculate_fee(&self, utxo_set: &HashMap<String, TransactionOutput>) -> u64 {
        if self.is_coinbase() {
            return 0;
        }

        let input_amount = self.total_input_amount(utxo_set);
        let output_amount = self.total_output_amount();
        
        if input_amount > output_amount {
            input_amount - output_amount
        } else {
            0
        }
    }

    /// Check if this is a coinbase transaction
    pub fn is_coinbase(&self) -> bool {
        self.inputs.len() == 1 && self.inputs[0].is_coinbase()
    }

    /// Validate the transaction
    pub fn validate(&self, utxo_set: &HashMap<String, TransactionOutput>) -> Result<()> {
        // Basic structure validation
        if self.inputs.is_empty() {
            return Err(ValidationError::EmptyInputs.into());
        }
        if self.outputs.is_empty() {
            return Err(ValidationError::EmptyOutputs.into());
        }

        // Validate inputs and outputs
        for input in &self.inputs {
            input.validate()?;
        }
        for output in &self.outputs {
            output.validate()?;
        }

        // Special validation for coinbase transactions
        if self.is_coinbase() {
            if self.inputs.len() != 1 {
                return Err(ValidationError::InvalidCoinbase("Coinbase must have exactly one input".to_string()).into());
            }
            return Ok(()); // Coinbase transactions don't need further validation
        }

        // Validate input amounts and availability
        let mut total_input = 0u64;
        for input in &self.inputs {
            let key = format!("{}:{}", input.previous_tx_hash, input.output_index);
            match utxo_set.get(&key) {
                Some(output) => {
                    if !output.is_spendable() {
                        return Err(ValidationError::OutputAlreadySpent(key).into());
                    }
                    total_input = total_input.checked_add(output.amount)
                        .ok_or_else(|| ValidationError::ArithmeticOverflow)?;
                }
                None => {
                    return Err(ValidationError::OutputNotFound(key).into());
                }
            }
        }

        let total_output = self.total_output_amount();
        if total_input < total_output {
            return Err(ValidationError::InsufficientFunds {
                required: total_output,
                available: total_input,
            }.into());
        }

        Ok(())
    }

    /// Sign the transaction (placeholder implementation)
    pub fn sign(&mut self, _private_key: &[u8]) -> Result<()> {
        // TODO: Implement actual transaction signing
        // This would involve:
        // 1. Creating a signature hash
        // 2. Signing with the private key
        // 3. Adding signatures to inputs
        Ok(())
    }

    /// Verify transaction signatures
    pub fn verify_signatures(&self) -> Result<bool> {
        for input in &self.inputs {
            if !input.is_coinbase() {
                if let (Some(signature), Some(public_key)) = (&input.signature, &input.public_key) {
                    let tx_hash = self.hash();
                    if !crate::crypto::verify_signature(tx_hash.as_slice(), signature, public_key)? {
                        return Ok(false);
                    }
                } else {
                    return Ok(false);
                }
            }
        }
        Ok(true)
    }
}

/// Transaction pool for managing pending transactions
#[derive(Debug, Clone, Default)]
pub struct TransactionPool {
    /// Pending transactions by hash
    pub transactions: HashMap<Hash256, Transaction>,
    /// Transaction priority queue (hash -> priority score)
    pub priority_queue: HashMap<Hash256, f64>,
    /// Maximum pool size
    pub max_size: usize,
}

impl TransactionPool {
    /// Create a new transaction pool
    pub fn new(max_size: usize) -> Self {
        Self {
            transactions: HashMap::new(),
            priority_queue: HashMap::new(),
            max_size,
        }
    }

    /// Add a transaction to the pool
    pub fn add_transaction(&mut self, transaction: Transaction) -> Result<()> {
        let tx_hash = transaction.hash();
        
        // Check if pool is full
        if self.transactions.len() >= self.max_size {
            self.evict_lowest_priority();
        }

        // Calculate priority score (higher fee = higher priority)
        let priority = transaction.fee.base_fee as f64 * transaction.fee.priority_multiplier;
        
        self.transactions.insert(tx_hash.clone(), transaction);
        self.priority_queue.insert(tx_hash, priority);
        
        Ok(())
    }

    /// Remove a transaction from the pool
    pub fn remove_transaction(&mut self, tx_hash: &Hash256) -> Option<Transaction> {
        self.priority_queue.remove(tx_hash);
        self.transactions.remove(tx_hash)
    }

    /// Get transactions sorted by priority
    pub fn get_transactions_by_priority(&self, limit: usize) -> Vec<Transaction> {
        let mut sorted_hashes: Vec<_> = self.priority_queue.iter()
            .map(|(hash, priority)| (hash.clone(), *priority))
            .collect();
        
        sorted_hashes.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        
        sorted_hashes.into_iter()
            .take(limit)
            .filter_map(|(hash, _)| self.transactions.get(&hash).cloned())
            .collect()
    }

    /// Evict the lowest priority transaction
    fn evict_lowest_priority(&mut self) {
        if let Some((lowest_hash, _)) = self.priority_queue.iter()
            .min_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(k, v)| (k.clone(), *v)) {
            self.remove_transaction(&lowest_hash);
        }
    }

    /// Get pool statistics
    pub fn stats(&self) -> TransactionPoolStats {
        let total_fees: u64 = self.transactions.values()
            .map(|tx| tx.fee.base_fee)
            .sum();
        
        let avg_fee = if !self.transactions.is_empty() {
            total_fees / self.transactions.len() as u64
        } else {
            0
        };

        TransactionPoolStats {
            total_transactions: self.transactions.len(),
            total_fees,
            average_fee: avg_fee,
            pool_utilization: (self.transactions.len() as f64 / self.max_size as f64) * 100.0,
        }
    }

    /// Clear all transactions
    pub fn clear(&mut self) {
        self.transactions.clear();
        self.priority_queue.clear();
    }

    /// Check if pool contains transaction
    pub fn contains(&self, tx_hash: &Hash256) -> bool {
        self.transactions.contains_key(tx_hash)
    }

    /// Get transaction by hash
    pub fn get_transaction(&self, tx_hash: &Hash256) -> Option<&Transaction> {
        self.transactions.get(tx_hash)
    }

    /// Get all transaction hashes
    pub fn get_all_hashes(&self) -> Vec<Hash256> {
        self.transactions.keys().cloned().collect()
    }
}

/// Transaction pool statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionPoolStats {
    /// Total number of transactions in pool
    pub total_transactions: usize,
    /// Total fees of all transactions
    pub total_fees: u64,
    /// Average transaction fee
    pub average_fee: u64,
    /// Pool utilization percentage
    pub pool_utilization: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::{SignatureAlgorithm};

    fn create_test_address() -> Address {
        let public_key = PublicKey::new(SignatureAlgorithm::EcdsaSecp256k1, vec![1, 2, 3, 4, 5]);
        Address::from_public_key(&public_key)
    }

    #[test]
    fn test_transaction_creation() {
        let input = TransactionInput::new(
            Hash256::zero(),
            0,
            None,
            None,
        );
        let output = TransactionOutput::new(1000, create_test_address());
        
        let tx = Transaction::new(vec![input], vec![output]);
        
        assert_eq!(tx.inputs.len(), 1);
        assert_eq!(tx.outputs.len(), 1);
        assert_eq!(tx.version, 1);
        assert!(tx.size.is_some());
    }

    #[test]
    fn test_coinbase_transaction() {
        let recipient = create_test_address();
        let tx = Transaction::coinbase(recipient, 5000, 1);
        
        assert!(tx.is_coinbase());
        assert_eq!(tx.inputs.len(), 1);
        assert_eq!(tx.outputs.len(), 1);
        assert_eq!(tx.outputs[0].amount, 5000);
        assert!(tx.inputs[0].is_coinbase());
    }

    #[test]
    fn test_transaction_hash() {
        let input = TransactionInput::new(Hash256::zero(), 0, None, None);
        let output = TransactionOutput::new(1000, create_test_address());
        let tx = Transaction::new(vec![input], vec![output]);
        
        let hash1 = tx.hash();
        let hash2 = tx.hash();
        
        assert_eq!(hash1, hash2); // Same transaction should produce same hash
    }

    #[test]
    fn test_transaction_amounts() {
        let input = TransactionInput::new(Hash256::zero(), 0, None, None);
        let output1 = TransactionOutput::new(500, create_test_address());
        let output2 = TransactionOutput::new(300, create_test_address());
        
        let tx = Transaction::new(vec![input], vec![output1, output2]);
        
        assert_eq!(tx.total_output_amount(), 800);
    }

    #[test]
    fn test_transaction_pool() {
        let mut pool = TransactionPool::new(10);
        
        let input = TransactionInput::new(Hash256::zero(), 0, None, None);
        let output = TransactionOutput::new(1000, create_test_address());
        let tx = Transaction::new(vec![input], vec![output]);
        
        let tx_hash = tx.hash();
        pool.add_transaction(tx).unwrap();
        
        assert!(pool.contains(&tx_hash));
        assert_eq!(pool.transactions.len(), 1);
        
        let removed = pool.remove_transaction(&tx_hash);
        assert!(removed.is_some());
        assert_eq!(pool.transactions.len(), 0);
    }

    #[test]
    fn test_transaction_validation() {
        let input = TransactionInput::new(Hash256::zero(), 0, None, None);
        let output = TransactionOutput::new(1000, create_test_address());
        let tx = Transaction::new(vec![input], vec![output]);
        
        let utxo_set = HashMap::new();
        
        // This should fail because the input doesn't exist in UTXO set
        assert!(tx.validate(&utxo_set).is_err());
    }

    #[test]
    fn test_coinbase_validation() {
        let recipient = create_test_address();
        let tx = Transaction::coinbase(recipient, 5000, 1);
        
        let utxo_set = HashMap::new();
        
        // Coinbase transactions should validate without UTXO checks
        assert!(tx.validate(&utxo_set).is_ok());
    }

    #[test]
    fn test_transaction_output_spending() {
        let mut output = TransactionOutput::new(1000, create_test_address());
        
        assert!(output.is_spendable());
        
        output.mark_spent();
        assert!(!output.is_spendable());
    }
}