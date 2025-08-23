//! Utility functions and helpers for the LedgerDB blockchain.
//!
//! This module provides various utility functions, constants, and helper
//! types used throughout the application.

use crate::crypto::Hash256;
use crate::error::LedgerError;
use serde::{Deserialize, Serialize};
use std::{
    fmt,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

/// Time utilities
pub mod time;

/// Formatting utilities
pub mod format;

/// Validation utilities
pub mod validation;

/// Network utilities
pub mod network;

/// File system utilities
pub mod fs;

/// Re-export submodule contents
pub use time::*;
pub use format::*;
pub use validation::*;
pub use network::*;
pub use fs::*;

/// Common constants
pub mod constants {
    /// Maximum block size in bytes (1MB)
    pub const MAX_BLOCK_SIZE: usize = 1_000_000;
    
    /// Maximum transaction size in bytes (100KB)
    pub const MAX_TRANSACTION_SIZE: usize = 100_000;
    
    /// Maximum number of transactions per block
    pub const MAX_TRANSACTIONS_PER_BLOCK: usize = 10_000;
    
    /// Minimum transaction fee (satoshis)
    pub const MIN_TRANSACTION_FEE: u64 = 1000;
    
    /// Block reward (satoshis)
    pub const BLOCK_REWARD: u64 = 50_000_000;
    
    /// Target block time (seconds)
    pub const TARGET_BLOCK_TIME: u64 = 600; // 10 minutes
    
    /// Difficulty adjustment interval (blocks)
    pub const DIFFICULTY_ADJUSTMENT_INTERVAL: u64 = 2016;
    
    /// Maximum difficulty adjustment factor
    pub const MAX_DIFFICULTY_ADJUSTMENT: f64 = 4.0;
    
    /// Minimum difficulty adjustment factor
    pub const MIN_DIFFICULTY_ADJUSTMENT: f64 = 0.25;
    
    /// Genesis block timestamp
    pub const GENESIS_TIMESTAMP: u64 = 1640995200; // 2022-01-01 00:00:00 UTC
    
    /// Maximum nonce value
    pub const MAX_NONCE: u64 = u64::MAX;
    
    /// Hash length in bytes
    pub const HASH_LENGTH: usize = 32;
    
    /// Address length in bytes
    pub const ADDRESS_LENGTH: usize = 20;
    
    /// Signature length in bytes
    pub const SIGNATURE_LENGTH: usize = 64;
    
    /// Public key length in bytes
    pub const PUBLIC_KEY_LENGTH: usize = 33;
    
    /// Private key length in bytes
    pub const PRIVATE_KEY_LENGTH: usize = 32;
    
    /// Maximum script length
    pub const MAX_SCRIPT_LENGTH: usize = 10_000;
    
    /// Maximum number of inputs per transaction
    pub const MAX_TRANSACTION_INPUTS: usize = 1000;
    
    /// Maximum number of outputs per transaction
    pub const MAX_TRANSACTION_OUTPUTS: usize = 1000;
    
    /// Dust threshold (minimum output value)
    pub const DUST_THRESHOLD: u64 = 546;
    
    /// Maximum money supply (21 million coins)
    pub const MAX_MONEY: u64 = 21_000_000 * 100_000_000; // 21M * 1 BTC in satoshis
    
    /// Coinbase maturity (blocks)
    pub const COINBASE_MATURITY: u64 = 100;
    
    /// Maximum orphan transactions
    pub const MAX_ORPHAN_TRANSACTIONS: usize = 10_000;
    
    /// Maximum peer connections
    pub const MAX_PEER_CONNECTIONS: usize = 125;
    
    /// Default P2P port
    pub const DEFAULT_P2P_PORT: u16 = 8333;
    
    /// Default RPC port
    pub const DEFAULT_RPC_PORT: u16 = 8332;
    
    /// Default WebSocket port
    pub const DEFAULT_WS_PORT: u16 = 8334;
    
    /// Maximum message size for P2P
    pub const MAX_P2P_MESSAGE_SIZE: usize = 32 * 1024 * 1024; // 32MB
    
    /// Connection timeout (seconds)
    pub const CONNECTION_TIMEOUT: u64 = 30;
    
    /// Ping interval (seconds)
    pub const PING_INTERVAL: u64 = 60;
    
    /// Maximum ban time (seconds)
    pub const MAX_BAN_TIME: u64 = 24 * 60 * 60; // 24 hours
}

/// Result type alias for convenience
pub type Result<T> = std::result::Result<T, LedgerError>;

/// Byte utilities
pub mod bytes {
    use super::*;
    
    /// Convert bytes to hex string
    pub fn to_hex(bytes: &[u8]) -> String {
        hex::encode(bytes)
    }
    
    /// Convert hex string to bytes
    pub fn from_hex(hex: &str) -> Result<Vec<u8>> {
        hex::decode(hex).map_err(|e| LedgerError::Internal(format!("Invalid hex: {}", e)))
    }
    
    /// Convert bytes to base58 string
    pub fn to_base58(bytes: &[u8]) -> String {
        bs58::encode(bytes).into_string()
    }
    
    /// Convert base58 string to bytes
    pub fn from_base58(base58: &str) -> Result<Vec<u8>> {
        bs58::decode(base58)
            .into_vec()
            .map_err(|e| LedgerError::Internal(format!("Invalid base58: {}", e)))
    }
    
    /// XOR two byte arrays
    pub fn xor(a: &[u8], b: &[u8]) -> Vec<u8> {
        a.iter().zip(b.iter()).map(|(x, y)| x ^ y).collect()
    }
    
    /// Reverse byte order
    pub fn reverse(bytes: &[u8]) -> Vec<u8> {
        bytes.iter().rev().cloned().collect()
    }
    
    /// Pad bytes to specified length
    pub fn pad_left(bytes: &[u8], length: usize, pad_byte: u8) -> Vec<u8> {
        if bytes.len() >= length {
            bytes.to_vec()
        } else {
            let mut padded = vec![pad_byte; length - bytes.len()];
            padded.extend_from_slice(bytes);
            padded
        }
    }
    
    /// Pad bytes to specified length (right)
    pub fn pad_right(bytes: &[u8], length: usize, pad_byte: u8) -> Vec<u8> {
        if bytes.len() >= length {
            bytes.to_vec()
        } else {
            let mut padded = bytes.to_vec();
            padded.resize(length, pad_byte);
            padded
        }
    }
    
    #[cfg(test)]
    mod tests {
        use super::*;
        
        #[test]
        fn test_hex_conversion() {
            let bytes = vec![0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef];
            let hex = to_hex(&bytes);
            assert_eq!(hex, "0123456789abcdef");
            
            let decoded = from_hex(&hex).unwrap();
            assert_eq!(decoded, bytes);
        }
        
        #[test]
        fn test_base58_conversion() {
            let bytes = vec![0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef];
            let base58 = to_base58(&bytes);
            let decoded = from_base58(&base58).unwrap();
            assert_eq!(decoded, bytes);
        }
        
        #[test]
        fn test_xor() {
            let a = vec![0x01, 0x02, 0x03];
            let b = vec![0x04, 0x05, 0x06];
            let result = xor(&a, &b);
            assert_eq!(result, vec![0x05, 0x07, 0x05]);
        }
        
        #[test]
        fn test_reverse() {
            let bytes = vec![0x01, 0x02, 0x03, 0x04];
            let reversed = reverse(&bytes);
            assert_eq!(reversed, vec![0x04, 0x03, 0x02, 0x01]);
        }
        
        #[test]
        fn test_padding() {
            let bytes = vec![0x01, 0x02];
            let padded_left = pad_left(&bytes, 5, 0x00);
            assert_eq!(padded_left, vec![0x00, 0x00, 0x00, 0x01, 0x02]);
            
            let padded_right = pad_right(&bytes, 5, 0xff);
            assert_eq!(padded_right, vec![0x01, 0x02, 0xff, 0xff, 0xff]);
        }
    }
}

/// Math utilities
pub mod math {
    use super::*;
    
    /// Calculate percentage change
    pub fn percentage_change(old_value: f64, new_value: f64) -> f64 {
        if old_value == 0.0 {
            if new_value == 0.0 {
                0.0
            } else {
                f64::INFINITY
            }
        } else {
            ((new_value - old_value) / old_value) * 100.0
        }
    }
    
    /// Calculate moving average
    pub fn moving_average(values: &[f64], window_size: usize) -> Vec<f64> {
        if values.is_empty() || window_size == 0 {
            return Vec::new();
        }
        
        let mut averages = Vec::new();
        for i in 0..values.len() {
            let start = if i >= window_size - 1 { i - window_size + 1 } else { 0 };
            let window = &values[start..=i];
            let avg = window.iter().sum::<f64>() / window.len() as f64;
            averages.push(avg);
        }
        averages
    }
    
    /// Calculate exponential moving average
    pub fn exponential_moving_average(values: &[f64], alpha: f64) -> Vec<f64> {
        if values.is_empty() {
            return Vec::new();
        }
        
        let mut ema = Vec::with_capacity(values.len());
        ema.push(values[0]);
        
        for i in 1..values.len() {
            let new_ema = alpha * values[i] + (1.0 - alpha) * ema[i - 1];
            ema.push(new_ema);
        }
        
        ema
    }
    
    /// Calculate standard deviation
    pub fn standard_deviation(values: &[f64]) -> f64 {
        if values.len() <= 1 {
            return 0.0;
        }
        
        let mean = values.iter().sum::<f64>() / values.len() as f64;
        let variance = values.iter()
            .map(|x| (x - mean).powi(2))
            .sum::<f64>() / (values.len() - 1) as f64;
        
        variance.sqrt()
    }
    
    /// Calculate median
    pub fn median(values: &mut [f64]) -> f64 {
        if values.is_empty() {
            return 0.0;
        }
        
        values.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let len = values.len();
        
        if len % 2 == 0 {
            (values[len / 2 - 1] + values[len / 2]) / 2.0
        } else {
            values[len / 2]
        }
    }
    
    /// Clamp value between min and max
    pub fn clamp<T: PartialOrd>(value: T, min: T, max: T) -> T {
        if value < min {
            min
        } else if value > max {
            max
        } else {
            value
        }
    }
    
    /// Linear interpolation
    pub fn lerp(a: f64, b: f64, t: f64) -> f64 {
        a + t * (b - a)
    }
    
    #[cfg(test)]
    mod tests {
        use super::*;
        
        #[test]
        fn test_percentage_change() {
            assert_eq!(percentage_change(100.0, 110.0), 10.0);
            assert_eq!(percentage_change(100.0, 90.0), -10.0);
            assert_eq!(percentage_change(0.0, 10.0), f64::INFINITY);
            assert_eq!(percentage_change(0.0, 0.0), 0.0);
        }
        
        #[test]
        fn test_moving_average() {
            let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
            let ma = moving_average(&values, 3);
            assert_eq!(ma, vec![1.0, 1.5, 2.0, 3.0, 4.0]);
        }
        
        #[test]
        fn test_standard_deviation() {
            let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
            let std_dev = standard_deviation(&values);
            assert!((std_dev - 1.5811388300841898).abs() < 1e-10);
        }
        
        #[test]
        fn test_median() {
            let mut values = vec![3.0, 1.0, 4.0, 1.0, 5.0];
            assert_eq!(median(&mut values), 3.0);
            
            let mut values = vec![3.0, 1.0, 4.0, 1.0];
            assert_eq!(median(&mut values), 2.0);
        }
        
        #[test]
        fn test_clamp() {
            assert_eq!(clamp(5, 1, 10), 5);
            assert_eq!(clamp(-5, 1, 10), 1);
            assert_eq!(clamp(15, 1, 10), 10);
        }
        
        #[test]
        fn test_lerp() {
            assert_eq!(lerp(0.0, 10.0, 0.5), 5.0);
            assert_eq!(lerp(10.0, 20.0, 0.25), 12.5);
        }
    }
}

/// Collection utilities
pub mod collections {
    use std::collections::{HashMap, HashSet};
    use std::hash::Hash;
    
    /// Merge two HashMaps
    pub fn merge_hashmaps<K, V>(mut map1: HashMap<K, V>, map2: HashMap<K, V>) -> HashMap<K, V>
    where
        K: Eq + Hash,
    {
        for (key, value) in map2 {
            map1.insert(key, value);
        }
        map1
    }
    
    /// Get intersection of two HashSets
    pub fn intersection<T>(set1: &HashSet<T>, set2: &HashSet<T>) -> HashSet<T>
    where
        T: Clone + Eq + Hash,
    {
        set1.intersection(set2).cloned().collect()
    }
    
    /// Get union of two HashSets
    pub fn union<T>(set1: &HashSet<T>, set2: &HashSet<T>) -> HashSet<T>
    where
        T: Clone + Eq + Hash,
    {
        set1.union(set2).cloned().collect()
    }
    
    /// Get difference of two HashSets (set1 - set2)
    pub fn difference<T>(set1: &HashSet<T>, set2: &HashSet<T>) -> HashSet<T>
    where
        T: Clone + Eq + Hash,
    {
        set1.difference(set2).cloned().collect()
    }
    
    /// Chunk a vector into smaller vectors of specified size
    pub fn chunk<T: Clone>(vec: Vec<T>, chunk_size: usize) -> Vec<Vec<T>> {
        if chunk_size == 0 {
            return vec![];
        }
        
        vec.chunks(chunk_size)
            .map(|chunk| chunk.to_vec())
            .collect()
    }
    
    /// Deduplicate a vector while preserving order
    pub fn deduplicate<T>(vec: Vec<T>) -> Vec<T>
    where
        T: Clone + Eq + Hash,
    {
        let mut seen = HashSet::new();
        let mut result = Vec::new();
        
        for item in vec {
            if seen.insert(item.clone()) {
                result.push(item);
            }
        }
        
        result
    }
    
    #[cfg(test)]
    mod tests {
        use super::*;
        
        #[test]
        fn test_merge_hashmaps() {
            let mut map1 = HashMap::new();
            map1.insert("a", 1);
            map1.insert("b", 2);
            
            let mut map2 = HashMap::new();
            map2.insert("c", 3);
            map2.insert("b", 4); // Should overwrite
            
            let merged = merge_hashmaps(map1, map2);
            assert_eq!(merged.len(), 3);
            assert_eq!(merged["a"], 1);
            assert_eq!(merged["b"], 4);
            assert_eq!(merged["c"], 3);
        }
        
        #[test]
        fn test_set_operations() {
            let set1: HashSet<i32> = [1, 2, 3, 4].iter().cloned().collect();
            let set2: HashSet<i32> = [3, 4, 5, 6].iter().cloned().collect();
            
            let intersect = intersection(&set1, &set2);
            assert_eq!(intersect, [3, 4].iter().cloned().collect());
            
            let union_set = union(&set1, &set2);
            assert_eq!(union_set, [1, 2, 3, 4, 5, 6].iter().cloned().collect());
            
            let diff = difference(&set1, &set2);
            assert_eq!(diff, [1, 2].iter().cloned().collect());
        }
        
        #[test]
        fn test_chunk() {
            let vec = vec![1, 2, 3, 4, 5, 6, 7];
            let chunks = chunk(vec, 3);
            assert_eq!(chunks, vec![vec![1, 2, 3], vec![4, 5, 6], vec![7]]);
        }
        
        #[test]
        fn test_deduplicate() {
            let vec = vec![1, 2, 2, 3, 1, 4, 3];
            let deduped = deduplicate(vec);
            assert_eq!(deduped, vec![1, 2, 3, 4]);
        }
    }
}

/// Random utilities
pub mod random {
    use rand::{thread_rng, Rng};

    
    /// Generate random bytes
    pub fn random_bytes(length: usize) -> Vec<u8> {
        let mut rng = thread_rng();
        (0..length).map(|_| rng.random()).collect()
    }
    
    /// Generate random u64
    pub fn random_u64() -> u64 {
        thread_rng().random()
    }
    
    /// Generate random u32
    pub fn random_u32() -> u32 {
        thread_rng().random()
    }
    
    /// Generate random f64 between 0.0 and 1.0
    pub fn random_f64() -> f64 {
        thread_rng().random()
    }
    
    /// Generate random boolean
    pub fn random_bool() -> bool {
        thread_rng().random()
    }
    
    /// Generate random string of specified length
    pub fn random_string(length: usize) -> String {
        const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
        let mut rng = thread_rng();
        
        (0..length)
            .map(|_| {
                let idx = rng.gen_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect()
    }
    
    /// Shuffle a vector in place
    pub fn shuffle<T>(vec: &mut Vec<T>) {
        use rand::seq::SliceRandom;
        vec.shuffle(&mut thread_rng());
    }
    
    /// Choose a random element from a slice
    pub fn choose<T>(slice: &[T]) -> Option<&T> {
        use rand::seq::SliceRandom;
        slice.choose(&mut thread_rng())
    }
    
    #[cfg(test)]
    mod tests {
        use super::*;
        
        #[test]
        fn test_random_bytes() {
            let bytes = random_bytes(10);
            assert_eq!(bytes.len(), 10);
        }
        
        #[test]
        fn test_random_string() {
            let s = random_string(20);
            assert_eq!(s.len(), 20);
            assert!(s.chars().all(|c| c.is_ascii_alphanumeric()));
        }
        
        #[test]
        fn test_shuffle() {
            let mut vec = vec![1, 2, 3, 4, 5];
            let original = vec.clone();
            shuffle(&mut vec);
            // Note: There's a small chance this could fail if shuffle returns the same order
            assert_eq!(vec.len(), original.len());
            for item in &original {
                assert!(vec.contains(item));
            }
        }
        
        #[test]
        fn test_choose() {
            let slice = [1, 2, 3, 4, 5];
            let chosen = choose(&slice);
            assert!(chosen.is_some());
            assert!(slice.contains(chosen.unwrap()));
            
            let empty: &[i32] = &[];
            assert!(choose(empty).is_none());
        }
    }
}

/// Logging utilities
pub mod logging {
    /// Initialize logging with default configuration
    pub fn init_logging() {
        init_logging_with_level("info")
    }
    
    /// Initialize logging with specified level
    pub fn init_logging_with_level(level: &str) {
        // Simple logging initialization
        // In a real implementation, you would use tracing-subscriber
        println!("Initializing logging with level: {}", level);
    }
    
    /// Initialize JSON logging for production
    pub fn init_json_logging() {
        // Simple JSON logging initialization
        // In a real implementation, you would use tracing-subscriber with JSON format
        println!("Initializing JSON logging");
    }
}

/// Performance measurement utilities
pub mod perf {
    use std::time::{Duration, Instant};
    
    /// Simple timer for measuring execution time
    pub struct Timer {
        start: Instant,
        name: String,
    }
    
    impl Timer {
        /// Create a new timer
        pub fn new(name: impl Into<String>) -> Self {
            Self {
                start: Instant::now(),
                name: name.into(),
            }
        }
        
        /// Get elapsed time
        pub fn elapsed(&self) -> Duration {
            self.start.elapsed()
        }
        
        /// Stop timer and log elapsed time
        pub fn stop(self) -> Duration {
            let elapsed = self.elapsed();
            tracing::info!("{} took {:?}", self.name, elapsed);
            elapsed
        }
    }
    
    /// Measure execution time of a closure
    pub fn measure<F, R>(name: &str, f: F) -> (R, Duration)
    where
        F: FnOnce() -> R,
    {
        let start = Instant::now();
        let result = f();
        let elapsed = start.elapsed();
        tracing::info!("{} took {:?}", name, elapsed);
        (result, elapsed)
    }
    
    /// Measure execution time of an async closure
    pub async fn measure_async<F, Fut, R>(name: &str, f: F) -> (R, Duration)
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = R>,
    {
        let start = Instant::now();
        let result = f().await;
        let elapsed = start.elapsed();
        tracing::info!("{} took {:?}", name, elapsed);
        (result, elapsed)
    }
    
    #[cfg(test)]
    mod tests {
        use super::*;
        use std::thread;
        
        #[test]
        fn test_timer() {
            let timer = Timer::new("test");
            thread::sleep(Duration::from_millis(10));
            let elapsed = timer.elapsed();
            assert!(elapsed >= Duration::from_millis(10));
        }
        
        #[test]
        fn test_measure() {
            let (result, elapsed) = measure("test", || {
                thread::sleep(Duration::from_millis(10));
                42
            });
            assert_eq!(result, 42);
            assert!(elapsed >= Duration::from_millis(10));
        }
    }
}