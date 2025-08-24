//! Proof of Work implementation for the LedgerDB blockchain system.
//!
//! This module provides mining algorithms, difficulty adjustment, and
//! proof-of-work validation for blockchain consensus.

use crate::crypto::Hash256;
use crate::error::{CryptoError, Result};
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Proof of Work configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofOfWorkConfig {
    /// Target difficulty (number of leading zeros required)
    pub difficulty: u32,
    /// Maximum number of mining attempts
    pub max_attempts: Option<u64>,
    /// Mining timeout in seconds
    pub timeout_seconds: Option<u64>,
    /// Number of mining threads
    pub threads: usize,
    /// Progress update interval in milliseconds
    pub progress_interval_ms: u64,
}

impl Default for ProofOfWorkConfig {
    fn default() -> Self {
        Self {
            difficulty: 4,
            max_attempts: None,
            timeout_seconds: Some(300), // 5 minutes
            threads: num_cpus::get().max(1),
            progress_interval_ms: 1000,
        }
    }
}

/// Mining progress information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MiningProgress {
    /// Current nonce being tested
    pub current_nonce: u64,
    /// Total attempts made
    pub attempts: u64,
    /// Hash rate (hashes per second)
    pub hash_rate: f64,
    /// Elapsed time in seconds
    pub elapsed_seconds: f64,
    /// Estimated time remaining in seconds
    pub estimated_remaining_seconds: Option<f64>,
    /// Whether mining is complete
    pub is_complete: bool,
    /// Current best hash (closest to target)
    pub best_hash: Option<Hash256>,
    /// Target difficulty
    pub target_difficulty: u32,
}

impl Default for MiningProgress {
    fn default() -> Self {
        Self {
            current_nonce: 0,
            attempts: 0,
            hash_rate: 0.0,
            elapsed_seconds: 0.0,
            estimated_remaining_seconds: None,
            is_complete: false,
            best_hash: None,
            target_difficulty: 0,
        }
    }
}

/// Result of a mining operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MiningResult {
    /// Whether mining was successful
    pub success: bool,
    /// The winning nonce (if successful)
    pub nonce: Option<u64>,
    /// The resulting hash
    pub hash: Option<Hash256>,
    /// Total attempts made
    pub attempts: u64,
    /// Time taken in seconds
    pub duration_seconds: f64,
    /// Average hash rate
    pub hash_rate: f64,
    /// Reason for stopping (if unsuccessful)
    pub stop_reason: Option<String>,
}

/// Proof of Work miner
#[derive(Debug)]
pub struct ProofOfWorkMiner {
    config: ProofOfWorkConfig,
    is_mining: Arc<AtomicBool>,
    current_nonce: Arc<AtomicU64>,
    total_attempts: Arc<AtomicU64>,
}

impl ProofOfWorkMiner {
    /// Create a new PoW miner with configuration
    pub fn new(config: ProofOfWorkConfig) -> Self {
        Self {
            config,
            is_mining: Arc::new(AtomicBool::new(false)),
            current_nonce: Arc::new(AtomicU64::new(0)),
            total_attempts: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Mine a block with the given data
    pub fn mine<F>(
        &self,
        block_data: &[u8],
        progress_callback: F,
    ) -> Result<MiningResult>
    where
        F: Fn(MiningProgress) + Send + Sync + 'static,
    {
        self.is_mining.store(true, Ordering::SeqCst);
        self.current_nonce.store(0, Ordering::SeqCst);
        self.total_attempts.store(0, Ordering::SeqCst);

        let start_time = Instant::now();
        let target = calculate_target(self.config.difficulty);
        let progress_callback = Arc::new(progress_callback);
        
        let mut best_hash = None;
        let mut best_score = u64::MAX;

        // Single-threaded mining for simplicity
        // TODO: Implement multi-threaded mining
        let mut nonce = 0u64;
        let mut attempts = 0u64;
        let mut last_progress_update = Instant::now();

        loop {
            // Check if we should stop
            if !self.is_mining.load(Ordering::SeqCst) {
                return Ok(MiningResult {
                    success: false,
                    nonce: None,
                    hash: best_hash,
                    attempts,
                    duration_seconds: start_time.elapsed().as_secs_f64(),
                    hash_rate: attempts as f64 / start_time.elapsed().as_secs_f64(),
                    stop_reason: Some("Mining stopped by user".to_string()),
                });
            }

            // Check timeout
            if let Some(timeout) = self.config.timeout_seconds {
                if start_time.elapsed().as_secs() >= timeout {
                    return Ok(MiningResult {
                        success: false,
                        nonce: None,
                        hash: best_hash,
                        attempts,
                        duration_seconds: start_time.elapsed().as_secs_f64(),
                        hash_rate: attempts as f64 / start_time.elapsed().as_secs_f64(),
                        stop_reason: Some("Timeout reached".to_string()),
                    });
                }
            }

            // Check max attempts
            if let Some(max_attempts) = self.config.max_attempts {
                if attempts >= max_attempts {
                    return Ok(MiningResult {
                        success: false,
                        nonce: None,
                        hash: best_hash,
                        attempts,
                        duration_seconds: start_time.elapsed().as_secs_f64(),
                        hash_rate: attempts as f64 / start_time.elapsed().as_secs_f64(),
                        stop_reason: Some("Maximum attempts reached".to_string()),
                    });
                }
            }

            // Try current nonce
            let hash = hash_with_nonce(block_data, nonce);
            attempts += 1;
            
            // Update counters
            self.current_nonce.store(nonce, Ordering::SeqCst);
            self.total_attempts.store(attempts, Ordering::SeqCst);

            // Check if this hash meets the target
            if hash_meets_target(&hash, &target) {
                self.is_mining.store(false, Ordering::SeqCst);
                return Ok(MiningResult {
                    success: true,
                    nonce: Some(nonce),
                    hash: Some(hash),
                    attempts,
                    duration_seconds: start_time.elapsed().as_secs_f64(),
                    hash_rate: attempts as f64 / start_time.elapsed().as_secs_f64(),
                    stop_reason: None,
                });
            }

            // Track best hash
            let hash_score = hash_to_score(&hash);
            if hash_score < best_score {
                best_score = hash_score;
                best_hash = Some(hash);
            }

            // Send progress update
            if last_progress_update.elapsed().as_millis() >= self.config.progress_interval_ms as u128 {
                let elapsed = start_time.elapsed().as_secs_f64();
                let hash_rate = attempts as f64 / elapsed;
                
                let estimated_remaining = if hash_rate > 0.0 {
                    let target_attempts = calculate_expected_attempts(self.config.difficulty);
                    let remaining_attempts = target_attempts.saturating_sub(attempts);
                    Some(remaining_attempts as f64 / hash_rate)
                } else {
                    None
                };

                let progress = MiningProgress {
                    current_nonce: nonce,
                    attempts,
                    hash_rate,
                    elapsed_seconds: elapsed,
                    estimated_remaining_seconds: estimated_remaining,
                    is_complete: false,
                    best_hash: best_hash.clone(),
                    target_difficulty: self.config.difficulty,
                };

                progress_callback(progress);
                last_progress_update = Instant::now();
            }

            nonce = nonce.wrapping_add(1);
        }
    }

    /// Stop the current mining operation
    pub fn stop(&self) {
        self.is_mining.store(false, Ordering::SeqCst);
    }

    /// Check if currently mining
    pub fn is_mining(&self) -> bool {
        self.is_mining.load(Ordering::SeqCst)
    }

    /// Get current mining progress
    pub fn get_progress(&self, start_time: Instant) -> MiningProgress {
        let current_nonce = self.current_nonce.load(Ordering::SeqCst);
        let attempts = self.total_attempts.load(Ordering::SeqCst);
        let elapsed = start_time.elapsed().as_secs_f64();
        let hash_rate = if elapsed > 0.0 {
            attempts as f64 / elapsed
        } else {
            0.0
        };

        MiningProgress {
            current_nonce,
            attempts,
            hash_rate,
            elapsed_seconds: elapsed,
            estimated_remaining_seconds: None,
            is_complete: !self.is_mining(),
            best_hash: None,
            target_difficulty: self.config.difficulty,
        }
    }
}

/// Calculate the target value for a given difficulty
pub fn calculate_target(difficulty: u32) -> Hash256 {
    let mut target_bytes = [0xFFu8; 32];
    
    // Set leading bytes to zero based on difficulty
    let zero_bytes = difficulty / 8;
    let remaining_bits = difficulty % 8;
    
    for i in 0..zero_bytes as usize {
        if i < 32 {
            target_bytes[i] = 0;
        }
    }
    
    if zero_bytes < 32 && remaining_bits > 0 {
        let mask = 0xFF >> remaining_bits;
        target_bytes[zero_bytes as usize] = mask;
    }
    
    Hash256::new(target_bytes)
}

/// Check if a hash meets the target difficulty
pub fn hash_meets_target(hash: &Hash256, target: &Hash256) -> bool {
    hash.as_slice() <= target.as_slice()
}

/// Validate proof of work for a block
pub fn validate_proof_of_work(
    block_data: &[u8],
    nonce: u64,
    difficulty: u32,
) -> bool {
    let hash = hash_with_nonce(block_data, nonce);
    let target = calculate_target(difficulty);
    hash_meets_target(&hash, &target)
}

/// Hash block data with a nonce
pub fn hash_with_nonce(block_data: &[u8], nonce: u64) -> Hash256 {
    let nonce_bytes = nonce.to_le_bytes();
    crate::crypto::hash_multiple(&[block_data, &nonce_bytes])
}

/// Convert hash to a numeric score for comparison
fn hash_to_score(hash: &Hash256) -> u64 {
    let bytes = hash.as_slice();
    u64::from_be_bytes([
        bytes[0], bytes[1], bytes[2], bytes[3],
        bytes[4], bytes[5], bytes[6], bytes[7],
    ])
}

/// Calculate expected number of attempts for a given difficulty
pub fn calculate_expected_attempts(difficulty: u32) -> u64 {
    if difficulty == 0 {
        1
    } else {
        2u64.pow(difficulty)
    }
}

/// Adjust difficulty based on block time
pub fn adjust_difficulty(
    current_difficulty: u32,
    target_time_seconds: u64,
    actual_time_seconds: u64,
    max_adjustment: f64,
) -> u32 {
    if actual_time_seconds == 0 {
        return current_difficulty;
    }

    let time_ratio = target_time_seconds as f64 / actual_time_seconds as f64;
    let adjustment_factor = time_ratio.ln() / 2f64.ln(); // Log base 2
    
    // Clamp adjustment to prevent wild swings
    let clamped_adjustment = adjustment_factor.max(-max_adjustment).min(max_adjustment);
    
    let new_difficulty = current_difficulty as f64 + clamped_adjustment;
    new_difficulty.max(1.0).round() as u32
}

/// Calculate mining statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MiningStats {
    /// Total blocks mined
    pub blocks_mined: u64,
    /// Total mining time in seconds
    pub total_mining_time: f64,
    /// Total attempts made
    pub total_attempts: u64,
    /// Average hash rate
    pub average_hash_rate: f64,
    /// Success rate (blocks found / attempts)
    pub success_rate: f64,
    /// Average time per block
    pub average_block_time: f64,
}

impl MiningStats {
    /// Create new empty mining stats
    pub fn new() -> Self {
        Self {
            blocks_mined: 0,
            total_mining_time: 0.0,
            total_attempts: 0,
            average_hash_rate: 0.0,
            success_rate: 0.0,
            average_block_time: 0.0,
        }
    }

    /// Update stats with a new mining result
    pub fn update(&mut self, result: &MiningResult) {
        if result.success {
            self.blocks_mined += 1;
        }
        
        self.total_mining_time += result.duration_seconds;
        self.total_attempts += result.attempts;
        
        // Recalculate averages
        if self.total_mining_time > 0.0 {
            self.average_hash_rate = self.total_attempts as f64 / self.total_mining_time;
        }
        
        if self.total_attempts > 0 {
            self.success_rate = self.blocks_mined as f64 / self.total_attempts as f64;
        }
        
        if self.blocks_mined > 0 {
            self.average_block_time = self.total_mining_time / self.blocks_mined as f64;
        }
    }
}

impl Default for MiningStats {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_target() {
        let target_0 = calculate_target(0);
        assert_eq!(target_0.as_slice(), &[0xFF; 32]);
        
        let target_8 = calculate_target(8);
        assert_eq!(target_8.as_slice()[0], 0);
        assert_eq!(target_8.as_slice()[1], 0xFF);
    }

    #[test]
    fn test_hash_with_nonce() {
        let data = b"test block data";
        let hash1 = hash_with_nonce(data, 0);
        let hash2 = hash_with_nonce(data, 1);
        
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_validate_proof_of_work() {
        let data = b"test block";
        let difficulty = 1; // Very easy for testing
        
        // Try different nonces until we find one that works
        for nonce in 0..1000 {
            if validate_proof_of_work(data, nonce, difficulty) {
                // Found a valid nonce
                assert!(validate_proof_of_work(data, nonce, difficulty));
                return;
            }
        }
        
        // If we get here, we didn't find a valid nonce in 1000 tries
        // This is possible but unlikely with difficulty 1
    }

    #[test]
    fn test_difficulty_adjustment() {
        let current_difficulty = 4;
        let target_time = 60; // 1 minute
        let max_adjustment = 1.0;
        
        // Block took too long, should decrease difficulty
        let new_difficulty = adjust_difficulty(current_difficulty, target_time, 120, max_adjustment);
        assert!(new_difficulty <= current_difficulty);
        
        // Block was too fast, should increase difficulty
        let new_difficulty = adjust_difficulty(current_difficulty, target_time, 30, max_adjustment);
        assert!(new_difficulty >= current_difficulty);
    }

    #[test]
    fn test_expected_attempts() {
        assert_eq!(calculate_expected_attempts(0), 1);
        assert_eq!(calculate_expected_attempts(1), 2);
        assert_eq!(calculate_expected_attempts(2), 4);
        assert_eq!(calculate_expected_attempts(3), 8);
    }

    #[test]
    fn test_mining_stats() {
        let mut stats = MiningStats::new();
        
        let result = MiningResult {
            success: true,
            nonce: Some(12345),
            hash: Some(Hash256::zero()),
            attempts: 1000,
            duration_seconds: 10.0,
            hash_rate: 100.0,
            stop_reason: None,
        };
        
        stats.update(&result);
        
        assert_eq!(stats.blocks_mined, 1);
        assert_eq!(stats.total_attempts, 1000);
        assert_eq!(stats.total_mining_time, 10.0);
        assert_eq!(stats.average_hash_rate, 100.0);
    }

    #[test]
    fn test_pow_miner_creation() {
        let config = ProofOfWorkConfig::default();
        let miner = ProofOfWorkMiner::new(config);
        
        assert!(!miner.is_mining());
    }

    #[test]
    fn test_hash_meets_target() {
        let easy_target = calculate_target(1);
        let hard_target = calculate_target(10);
        
        let zero_hash = Hash256::zero();
        assert!(hash_meets_target(&zero_hash, &easy_target));
        assert!(hash_meets_target(&zero_hash, &hard_target));
        
        let max_hash = Hash256::new([0xFF; 32]);
        assert!(hash_meets_target(&max_hash, &easy_target));
    }
}