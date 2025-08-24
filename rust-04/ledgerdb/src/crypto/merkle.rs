//! Merkle tree implementation for the LedgerDB blockchain system.
//!
//! This module provides efficient Merkle tree operations for transaction
//! verification, inclusion proofs, and data integrity validation.

use crate::crypto::Hash256;
use crate::error::{CryptoError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A node in the Merkle tree
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MerkleNode {
    /// Hash of this node
    pub hash: Hash256,
    /// Left child hash (if internal node)
    pub left: Option<Hash256>,
    /// Right child hash (if internal node)
    pub right: Option<Hash256>,
    /// Whether this is a leaf node
    pub is_leaf: bool,
}

impl MerkleNode {
    /// Create a new leaf node
    pub fn leaf(hash: Hash256) -> Self {
        Self {
            hash,
            left: None,
            right: None,
            is_leaf: true,
        }
    }

    /// Create a new internal node
    pub fn internal(left_hash: Hash256, right_hash: Hash256) -> Self {
        let combined_hash = crate::crypto::hash_multiple(&[
            left_hash.as_slice(),
            right_hash.as_slice(),
        ]);
        Self {
            hash: combined_hash,
            left: Some(left_hash),
            right: Some(right_hash),
            is_leaf: false,
        }
    }

    /// Get the hash of this node
    pub fn hash(&self) -> &Hash256 {
        &self.hash
    }
}

/// A Merkle tree for efficient data verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MerkleTree {
    /// All nodes in the tree, indexed by their hash
    nodes: HashMap<Hash256, MerkleNode>,
    /// Root hash of the tree
    root: Hash256,
    /// Leaf hashes in order
    leaves: Vec<Hash256>,
    /// Height of the tree
    height: usize,
}

impl MerkleTree {
    /// Create a new Merkle tree from leaf data
    pub fn new<T: AsRef<[u8]>>(leaf_data: &[T]) -> Result<Self> {
        if leaf_data.is_empty() {
            return Err(CryptoError::EmptyMerkleTree.into());
        }

        // Create leaf hashes
        let leaves: Vec<Hash256> = leaf_data
            .iter()
            .map(|data| crate::crypto::hash_data(data.as_ref()))
            .collect();

        Self::from_hashes(&leaves)
    }

    /// Create a Merkle tree from pre-computed hashes
    pub fn from_hashes(leaf_hashes: &[Hash256]) -> Result<Self> {
        if leaf_hashes.is_empty() {
            return Err(CryptoError::EmptyMerkleTree.into());
        }

        let mut nodes = HashMap::new();
        let leaves = leaf_hashes.to_vec();
        
        // Add leaf nodes
        for hash in &leaves {
            nodes.insert(hash.clone(), MerkleNode::leaf(hash.clone()));
        }

        // Build tree bottom-up
        let mut current_level = leaves.clone();
        let mut height = 0;

        while current_level.len() > 1 {
            let mut next_level = Vec::new();
            
            for chunk in current_level.chunks(2) {
                let left_hash = chunk[0].clone();
                let right_hash = if chunk.len() == 2 {
                    chunk[1].clone()
                } else {
                    // Duplicate the last hash if odd number of nodes
                    chunk[0].clone()
                };

                let internal_node = MerkleNode::internal(left_hash, right_hash);
                let node_hash = internal_node.hash.clone();
                nodes.insert(node_hash.clone(), internal_node);
                next_level.push(node_hash);
            }
            
            current_level = next_level;
            height += 1;
        }

        let root = current_level.into_iter().next().unwrap();

        Ok(Self {
            nodes,
            root,
            leaves,
            height,
        })
    }

    /// Get the root hash of the tree
    pub fn root(&self) -> &Hash256 {
        &self.root
    }

    /// Get the height of the tree
    pub fn height(&self) -> usize {
        self.height
    }

    /// Get the number of leaves
    pub fn leaf_count(&self) -> usize {
        self.leaves.len()
    }

    /// Get all leaf hashes
    pub fn leaves(&self) -> &[Hash256] {
        &self.leaves
    }

    /// Generate a Merkle proof for a specific leaf
    pub fn generate_proof(&self, leaf_hash: &Hash256) -> Result<MerkleProof> {
        let leaf_index = self
            .leaves
            .iter()
            .position(|h| h == leaf_hash)
            .ok_or_else(|| CryptoError::LeafNotFound {
                index: 0, // Will be updated with actual index if needed
            })?;

        self.generate_proof_by_index(leaf_index)
    }

    /// Generate a Merkle proof for a leaf at a specific index
    pub fn generate_proof_by_index(&self, leaf_index: usize) -> Result<MerkleProof> {
        if leaf_index >= self.leaves.len() {
            return Err(CryptoError::InvalidLeafIndex {
                index: leaf_index,
            }
            .into());
        }

        let mut proof_hashes = Vec::new();
        let mut proof_directions = Vec::new();
        let mut current_index = leaf_index;
        let mut current_level = self.leaves.clone();

        // Traverse up the tree
        while current_level.len() > 1 {
            let sibling_index = if current_index % 2 == 0 {
                // Current node is left child, sibling is right
                if current_index + 1 < current_level.len() {
                    current_index + 1
                } else {
                    // No right sibling, use self (odd number of nodes)
                    current_index
                }
            } else {
                // Current node is right child, sibling is left
                current_index - 1
            };

            let sibling_hash = current_level[sibling_index].clone();
            proof_hashes.push(sibling_hash);
            proof_directions.push(current_index % 2 == 0); // true if current is left

            // Move to next level
            let mut next_level = Vec::new();
            for chunk in current_level.chunks(2) {
                let left_hash = chunk[0].clone();
                let right_hash = if chunk.len() == 2 {
                    chunk[1].clone()
                } else {
                    chunk[0].clone()
                };
                let combined = crate::crypto::hash_multiple(&[
                    left_hash.as_slice(),
                    right_hash.as_slice(),
                ]);
                next_level.push(combined);
            }
            
            current_level = next_level;
            current_index /= 2;
        }

        Ok(MerkleProof {
            leaf_hash: self.leaves[leaf_index].clone(),
            leaf_index,
            proof_hashes,
            proof_directions,
            root_hash: self.root.clone(),
        })
    }

    /// Verify a Merkle proof against this tree
    pub fn verify_proof(&self, proof: &MerkleProof) -> bool {
        proof.verify(&self.root)
    }

    /// Get a node by its hash
    pub fn get_node(&self, hash: &Hash256) -> Option<&MerkleNode> {
        self.nodes.get(hash)
    }

    /// Check if the tree contains a specific leaf
    pub fn contains_leaf(&self, leaf_hash: &Hash256) -> bool {
        self.leaves.contains(leaf_hash)
    }

    /// Create a Merkle tree from transactions
    pub fn from_transactions(transactions: &[crate::core::Transaction]) -> Result<Self> {
        if transactions.is_empty() {
            return Err(CryptoError::EmptyMerkleTree.into());
        }

        // Create hashes from transaction IDs
        let tx_hashes: Vec<Hash256> = transactions
            .iter()
            .map(|tx| tx.hash())
            .collect();

        Self::from_hashes(&tx_hashes)
    }

    /// Get the path from root to a specific leaf
    pub fn get_path_to_leaf(&self, leaf_hash: &Hash256) -> Result<Vec<Hash256>> {
        let leaf_index = self
            .leaves
            .iter()
            .position(|h| h == leaf_hash)
            .ok_or_else(|| CryptoError::LeafNotFound {
                index: 0, // Will be updated with actual index if needed
            })?;

        let mut path = vec![leaf_hash.clone()];
        let mut current_index = leaf_index;
        let mut current_level = self.leaves.clone();

        while current_level.len() > 1 {
            let mut next_level = Vec::new();
            for chunk in current_level.chunks(2) {
                let left_hash = chunk[0].clone();
                let right_hash = if chunk.len() == 2 {
                    chunk[1].clone()
                } else {
                    chunk[0].clone()
                };
                let combined = crate::crypto::hash_multiple(&[
                    left_hash.as_slice(),
                    right_hash.as_slice(),
                ]);
                next_level.push(combined);
            }
            
            current_level = next_level;
            current_index /= 2;
            if current_index < current_level.len() {
                path.push(current_level[current_index].clone());
            }
        }

        Ok(path)
    }
}

/// A proof of inclusion for a leaf in a Merkle tree
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MerkleProof {
    /// The leaf hash being proven
    pub leaf_hash: Hash256,
    /// Index of the leaf in the tree
    pub leaf_index: usize,
    /// Sibling hashes along the path to root
    pub proof_hashes: Vec<Hash256>,
    /// Direction indicators (true = current node is left child)
    pub proof_directions: Vec<bool>,
    /// Expected root hash
    pub root_hash: Hash256,
}

impl MerkleProof {
    /// Verify this proof against a given root hash
    pub fn verify(&self, expected_root: &Hash256) -> bool {
        if self.root_hash != *expected_root {
            return false;
        }

        let mut current_hash = self.leaf_hash.clone();
        
        for (sibling_hash, is_left) in self.proof_hashes.iter().zip(&self.proof_directions) {
            current_hash = if *is_left {
                // Current node is left child
                crate::crypto::hash_multiple(&[
                    current_hash.as_slice(),
                    sibling_hash.as_slice(),
                ])
            } else {
                // Current node is right child
                crate::crypto::hash_multiple(&[
                    sibling_hash.as_slice(),
                    current_hash.as_slice(),
                ])
            };
        }

        current_hash == *expected_root
    }

    /// Get the size of this proof in bytes
    pub fn size(&self) -> usize {
        32 + // leaf_hash
        8 + // leaf_index
        (self.proof_hashes.len() * 32) + // proof_hashes
        self.proof_directions.len() + // proof_directions (1 byte each)
        32 // root_hash
    }

    /// Convert to bytes for serialization
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        bincode::serialize(self).map_err(|e| CryptoError::SerializationError { source: e }.into())
    }

    /// Create from bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        bincode::deserialize(bytes).map_err(|e| CryptoError::SerializationError { source: e }.into())
    }
}

/// Utility functions for Merkle tree operations
pub mod utils {
    use super::*;

    /// Compute Merkle root from a list of hashes
    pub fn compute_merkle_root(hashes: &[Hash256]) -> Result<Hash256> {
        if hashes.is_empty() {
            return Err(CryptoError::EmptyMerkleTree.into());
        }

        let tree = MerkleTree::from_hashes(hashes)?;
        Ok(tree.root().clone())
    }

    /// Verify multiple proofs against the same root
    pub fn verify_multiple_proofs(proofs: &[MerkleProof], root: &Hash256) -> bool {
        proofs.iter().all(|proof| proof.verify(root))
    }

    /// Create a Merkle tree from transaction IDs
    pub fn merkle_tree_from_transaction_ids(tx_ids: &[String]) -> Result<MerkleTree> {
        let hashes: Vec<Hash256> = tx_ids
            .iter()
            .map(|id| crate::crypto::sha256_hash(id.as_bytes()))
            .collect();
        MerkleTree::from_hashes(&hashes)
    }

    /// Batch verify proofs (more efficient for multiple proofs)
    pub fn batch_verify_proofs(proofs: &[MerkleProof]) -> HashMap<Hash256, bool> {
        let mut results = HashMap::new();
        
        for proof in proofs {
            let is_valid = proof.verify(&proof.root_hash);
            results.insert(proof.root_hash.clone(), is_valid);
        }
        
        results
    }

    /// Calculate the minimum tree height for a given number of leaves
    pub fn calculate_tree_height(leaf_count: usize) -> usize {
        if leaf_count <= 1 {
            0
        } else {
            (leaf_count as f64).log2().ceil() as usize
        }
    }

    /// Calculate the maximum number of leaves for a given tree height
    pub fn max_leaves_for_height(height: usize) -> usize {
        2_usize.pow(height as u32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merkle_tree_creation() {
        let data = vec!["tx1", "tx2", "tx3", "tx4"];
        let tree = MerkleTree::new(&data).unwrap();
        
        assert_eq!(tree.leaf_count(), 4);
        assert!(!tree.root().is_zero());
    }

    #[test]
    fn test_merkle_proof_generation_and_verification() {
        let data = vec!["tx1", "tx2", "tx3", "tx4"];
        let tree = MerkleTree::new(&data).unwrap();
        
        let leaf_hash = crate::crypto::sha256_hash(b"tx1");
        let proof = tree.generate_proof(&leaf_hash).unwrap();
        
        assert!(tree.verify_proof(&proof));
        assert!(proof.verify(tree.root()));
    }

    #[test]
    fn test_merkle_tree_odd_number_of_leaves() {
        let data = vec!["tx1", "tx2", "tx3"];
        let tree = MerkleTree::new(&data).unwrap();
        
        assert_eq!(tree.leaf_count(), 3);
        
        for i in 0..3 {
            let proof = tree.generate_proof_by_index(i).unwrap();
            assert!(tree.verify_proof(&proof));
        }
    }

    #[test]
    fn test_single_leaf_tree() {
        let data = vec!["single_tx"];
        let tree = MerkleTree::new(&data).unwrap();
        
        assert_eq!(tree.leaf_count(), 1);
        assert_eq!(tree.height(), 0);
        
        let leaf_hash = crate::crypto::sha256_hash(b"single_tx");
        assert_eq!(tree.root(), &leaf_hash);
    }

    #[test]
    fn test_empty_tree_error() {
        let data: Vec<&str> = vec![];
        let result = MerkleTree::new(&data);
        assert!(result.is_err());
    }

    #[test]
    fn test_merkle_proof_serialization() {
        let data = vec!["tx1", "tx2", "tx3", "tx4"];
        let tree = MerkleTree::new(&data).unwrap();
        
        let leaf_hash = crate::crypto::sha256_hash(b"tx1");
        let proof = tree.generate_proof(&leaf_hash).unwrap();
        
        let bytes = proof.to_bytes().unwrap();
        let restored_proof = MerkleProof::from_bytes(&bytes).unwrap();
        
        assert_eq!(proof, restored_proof);
        assert!(restored_proof.verify(tree.root()));
    }

    #[test]
    fn test_compute_merkle_root() {
        let hashes = vec![
            crate::crypto::sha256_hash(b"tx1"),
            crate::crypto::sha256_hash(b"tx2"),
            crate::crypto::sha256_hash(b"tx3"),
        ];
        
        let root = utils::compute_merkle_root(&hashes).unwrap();
        let tree = MerkleTree::from_hashes(&hashes).unwrap();
        
        assert_eq!(root, *tree.root());
    }

    #[test]
    fn test_tree_height_calculation() {
        assert_eq!(utils::calculate_tree_height(1), 0);
        assert_eq!(utils::calculate_tree_height(2), 1);
        assert_eq!(utils::calculate_tree_height(4), 2);
        assert_eq!(utils::calculate_tree_height(8), 3);
        assert_eq!(utils::calculate_tree_height(7), 3);
    }

    #[test]
    fn test_max_leaves_for_height() {
        assert_eq!(utils::max_leaves_for_height(0), 1);
        assert_eq!(utils::max_leaves_for_height(1), 2);
        assert_eq!(utils::max_leaves_for_height(2), 4);
        assert_eq!(utils::max_leaves_for_height(3), 8);
    }

    #[test]
    fn test_path_to_leaf() {
        let data = vec!["tx1", "tx2", "tx3", "tx4"];
        let tree = MerkleTree::new(&data).unwrap();
        
        let leaf_hash = crate::crypto::sha256_hash(b"tx1");
        let path = tree.get_path_to_leaf(&leaf_hash).unwrap();
        
        assert!(!path.is_empty());
        assert_eq!(path[0], leaf_hash);
        assert_eq!(path.last().unwrap(), tree.root());
    }
}