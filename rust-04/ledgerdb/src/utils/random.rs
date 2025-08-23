//! Random number generation utilities for the LedgerDB blockchain.
//!
//! This module provides cryptographically secure random number generation
//! and utilities for blockchain operations requiring randomness.

use crate::error::LedgerError;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::time::{SystemTime, UNIX_EPOCH};

/// Random number generator using system entropy
pub struct SecureRng {
    state: u64,
    counter: u64,
}

impl SecureRng {
    /// Create a new secure RNG
    pub fn new() -> Self {
        let mut rng = Self {
            state: 0,
            counter: 0,
        };
        rng.reseed();
        rng
    }
    
    /// Create RNG with specific seed (for testing)
    pub fn with_seed(seed: u64) -> Self {
        Self {
            state: seed,
            counter: 0,
        }
    }
    
    /// Reseed the RNG with system entropy
    pub fn reseed(&mut self) {
        let mut hasher = DefaultHasher::new();
        
        // Use system time as entropy source
        if let Ok(duration) = SystemTime::now().duration_since(UNIX_EPOCH) {
            duration.as_nanos().hash(&mut hasher);
        }
        
        // Add process-specific entropy
        std::process::id().hash(&mut hasher);
        
        // Add thread-specific entropy
        std::thread::current().id().hash(&mut hasher);
        
        // Mix with current state
        self.state.hash(&mut hasher);
        self.counter.hash(&mut hasher);
        
        self.state = hasher.finish();
        self.counter = 0;
    }
    
    /// Generate next random u64
    pub fn next_u64(&mut self) -> u64 {
        // Simple linear congruential generator with good constants
        self.counter = self.counter.wrapping_add(1);
        self.state = self.state.wrapping_mul(6364136223846793005).wrapping_add(self.counter);
        
        // XOR shift for better distribution
        let mut x = self.state;
        x ^= x >> 32;
        x = x.wrapping_mul(0xd6e8feb86659fd93);
        x ^= x >> 32;
        x = x.wrapping_mul(0xd6e8feb86659fd93);
        x ^= x >> 32;
        
        x
    }
    
    /// Generate random u32
    pub fn next_u32(&mut self) -> u32 {
        (self.next_u64() >> 32) as u32
    }
    
    /// Generate random u16
    pub fn next_u16(&mut self) -> u16 {
        (self.next_u64() >> 48) as u16
    }
    
    /// Generate random u8
    pub fn next_u8(&mut self) -> u8 {
        (self.next_u64() >> 56) as u8
    }
    
    /// Generate random bool
    pub fn next_bool(&mut self) -> bool {
        (self.next_u64() & 1) == 1
    }
    
    /// Generate random f64 in range [0.0, 1.0)
    pub fn next_f64(&mut self) -> f64 {
        let mantissa = self.next_u64() >> 12; // Use 52 bits for mantissa
        (mantissa as f64) / (1u64 << 52) as f64
    }
    
    /// Generate random f32 in range [0.0, 1.0)
    pub fn next_f32(&mut self) -> f32 {
        let mantissa = self.next_u32() >> 9; // Use 23 bits for mantissa
        (mantissa as f32) / (1u32 << 23) as f32
    }
    
    /// Generate random number in range [min, max)
    pub fn range_u64(&mut self, min: u64, max: u64) -> Result<u64, LedgerError> {
        if min >= max {
            return Err(LedgerError::Validation(
                "Invalid range: min must be less than max".to_string()
            ));
        }
        
        let range = max - min;
        let random = self.next_u64();
        Ok(min + (random % range))
    }
    
    /// Generate random number in range [min, max)
    pub fn range_i64(&mut self, min: i64, max: i64) -> Result<i64, LedgerError> {
        if min >= max {
            return Err(LedgerError::Validation(
                "Invalid range: min must be less than max".to_string()
            ));
        }
        
        let range = (max - min) as u64;
        let random = self.next_u64() % range;
        Ok(min + random as i64)
    }
    
    /// Generate random f64 in range [min, max)
    pub fn range_f64(&mut self, min: f64, max: f64) -> Result<f64, LedgerError> {
        if min >= max {
            return Err(LedgerError::Validation(
                "Invalid range: min must be less than max".to_string()
            ));
        }
        
        let random = self.next_f64();
        Ok(min + random * (max - min))
    }
    
    /// Fill byte array with random data
    pub fn fill_bytes(&mut self, bytes: &mut [u8]) {
        for chunk in bytes.chunks_mut(8) {
            let random = self.next_u64().to_le_bytes();
            for (i, &byte) in random.iter().enumerate() {
                if i < chunk.len() {
                    chunk[i] = byte;
                }
            }
        }
    }
    
    /// Generate random bytes
    pub fn bytes(&mut self, len: usize) -> Vec<u8> {
        let mut bytes = vec![0u8; len];
        self.fill_bytes(&mut bytes);
        bytes
    }
    
    /// Generate random hex string
    pub fn hex_string(&mut self, len: usize) -> String {
        let bytes = self.bytes((len + 1) / 2);
        let hex = hex::encode(bytes);
        hex[..len].to_string()
    }
    
    /// Shuffle slice in place
    pub fn shuffle<T>(&mut self, slice: &mut [T]) {
        for i in (1..slice.len()).rev() {
            let j = (self.next_u64() as usize) % (i + 1);
            slice.swap(i, j);
        }
    }
    
    /// Choose random element from slice
    pub fn choose<T>(&mut self, slice: &[T]) -> Option<&T> {
        if slice.is_empty() {
            None
        } else {
            let index = (self.next_u64() as usize) % slice.len();
            Some(&slice[index])
        }
    }
    
    /// Choose multiple random elements without replacement
    pub fn sample<T: Clone>(&mut self, slice: &[T], count: usize) -> Vec<T> {
        if count >= slice.len() {
            return slice.to_vec();
        }
        
        let mut indices: Vec<usize> = (0..slice.len()).collect();
        self.shuffle(&mut indices);
        
        indices[..count]
            .iter()
            .map(|&i| slice[i].clone())
            .collect()
    }
    
    /// Generate random weighted choice
    pub fn weighted_choice<T>(&mut self, items: &[(T, f64)]) -> Option<&T> 
    where 
        T: Clone,
    {
        if items.is_empty() {
            return None;
        }
        
        let total_weight: f64 = items.iter().map(|(_, weight)| weight).sum();
        if total_weight <= 0.0 {
            return None;
        }
        
        let mut random = self.next_f64() * total_weight;
        
        for (item, weight) in items {
            random -= weight;
            if random <= 0.0 {
                return Some(item);
            }
        }
        
        // Fallback to last item (shouldn't happen with proper weights)
        Some(&items.last()?.0)
    }
}

impl Default for SecureRng {
    fn default() -> Self {
        Self::new()
    }
}

/// Random utilities
pub struct RandomUtils;

impl RandomUtils {
    /// Generate cryptographically secure random bytes
    pub fn secure_bytes(len: usize) -> Vec<u8> {
        let mut rng = SecureRng::new();
        rng.bytes(len)
    }
    
    /// Generate random UUID-like string
    pub fn uuid() -> String {
        let mut rng = SecureRng::new();
        format!(
            "{:08x}-{:04x}-{:04x}-{:04x}-{:012x}",
            rng.next_u32(),
            rng.next_u16(),
            rng.next_u16(),
            rng.next_u16(),
            rng.next_u64() & 0xFFFFFFFFFFFF
        )
    }
    
    /// Generate random nonce for mining
    pub fn mining_nonce() -> u64 {
        let mut rng = SecureRng::new();
        rng.next_u64()
    }
    
    /// Generate random transaction ID
    pub fn transaction_id() -> String {
        let mut rng = SecureRng::new();
        rng.hex_string(64) // 32 bytes = 64 hex chars
    }
    
    /// Generate random address (simplified)
    pub fn address() -> String {
        let mut rng = SecureRng::new();
        format!("addr_{}", rng.hex_string(40)) // 20 bytes = 40 hex chars
    }
    
    /// Generate random private key (for testing only)
    pub fn private_key() -> [u8; 32] {
        let mut rng = SecureRng::new();
        let mut key = [0u8; 32];
        rng.fill_bytes(&mut key);
        key
    }
    
    /// Generate random salt for hashing
    pub fn salt(len: usize) -> Vec<u8> {
        Self::secure_bytes(len)
    }
    
    /// Generate random password
    pub fn password(length: usize) -> String {
        const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789!@#$%^&*";
        let mut rng = SecureRng::new();
        
        (0..length)
            .map(|_| {
                let idx = (rng.next_u64() as usize) % CHARS.len();
                CHARS[idx] as char
            })
            .collect()
    }
    
    /// Generate random alphanumeric string
    pub fn alphanumeric(length: usize) -> String {
        const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
        let mut rng = SecureRng::new();
        
        (0..length)
            .map(|_| {
                let idx = (rng.next_u64() as usize) % CHARS.len();
                CHARS[idx] as char
            })
            .collect()
    }
    
    /// Generate random numeric string
    pub fn numeric(length: usize) -> String {
        let mut rng = SecureRng::new();
        
        (0..length)
            .map(|_| {
                let digit = (rng.next_u64() % 10) as u8;
                (b'0' + digit) as char
            })
            .collect()
    }
}

/// Probability utilities
pub struct ProbabilityUtils;

impl ProbabilityUtils {
    /// Simulate coin flip with given probability of heads
    pub fn coin_flip(probability: f64) -> bool {
        let mut rng = SecureRng::new();
        rng.next_f64() < probability
    }
    
    /// Simulate dice roll (1-6)
    pub fn dice_roll() -> u8 {
        let mut rng = SecureRng::new();
        ((rng.next_u64() % 6) + 1) as u8
    }
    
    /// Simulate dice roll with n sides
    pub fn dice_roll_n(sides: u8) -> u8 {
        if sides == 0 {
            return 0;
        }
        let mut rng = SecureRng::new();
        ((rng.next_u64() % sides as u64) + 1) as u8
    }
    
    /// Generate random boolean with given probability of true
    pub fn random_bool(probability: f64) -> bool {
        let mut rng = SecureRng::new();
        rng.next_f64() < probability.clamp(0.0, 1.0)
    }
    
    /// Generate random number following normal distribution (Box-Muller)
    pub fn normal_distribution(mean: f64, std_dev: f64) -> f64 {
        static mut SPARE: Option<f64> = None;
        static mut HAS_SPARE: bool = false;
        
        unsafe {
            if HAS_SPARE {
                HAS_SPARE = false;
                return SPARE.unwrap() * std_dev + mean;
            }
        }
        
        let mut rng = SecureRng::new();
        let u1 = rng.next_f64();
        let u2 = rng.next_f64();
        
        let mag = std_dev * (-2.0 * u1.ln()).sqrt();
        let z0 = mag * (2.0 * std::f64::consts::PI * u2).cos() + mean;
        let z1 = mag * (2.0 * std::f64::consts::PI * u2).sin();
        
        unsafe {
            SPARE = Some(z1);
            HAS_SPARE = true;
        }
        
        z0
    }
    
    /// Generate random number following exponential distribution
    pub fn exponential_distribution(lambda: f64) -> f64 {
        let mut rng = SecureRng::new();
        let u = rng.next_f64();
        -u.ln() / lambda
    }
    
    /// Generate random number following Poisson distribution
    pub fn poisson_distribution(lambda: f64) -> u32 {
        if lambda <= 0.0 {
            return 0;
        }
        
        let mut rng = SecureRng::new();
        let l = (-lambda).exp();
        let mut k = 0;
        let mut p = 1.0;
        
        loop {
            k += 1;
            p *= rng.next_f64();
            if p <= l {
                break;
            }
        }
        
        k - 1
    }
}

/// Sampling utilities
pub struct SamplingUtils;

impl SamplingUtils {
    /// Reservoir sampling - select k random items from stream
    pub fn reservoir_sample<T: Clone>(items: &[T], k: usize) -> Vec<T> {
        if k >= items.len() {
            return items.to_vec();
        }
        
        let mut rng = SecureRng::new();
        let mut reservoir = Vec::with_capacity(k);
        
        // Fill reservoir with first k items
        for item in items.iter().take(k) {
            reservoir.push(item.clone());
        }
        
        // Process remaining items
        for (i, item) in items.iter().enumerate().skip(k) {
            let j = rng.range_u64(0, (i + 1) as u64).unwrap() as usize;
            if j < k {
                reservoir[j] = item.clone();
            }
        }
        
        reservoir
    }
    
    /// Stratified sampling
    pub fn stratified_sample<T, F, K>(
        items: &[T],
        strata_fn: F,
        samples_per_stratum: usize,
    ) -> Vec<T>
    where
        T: Clone,
        F: Fn(&T) -> K,
        K: Eq + std::hash::Hash,
    {
        use std::collections::HashMap;
        
        // Group items by strata
        let mut strata: HashMap<K, Vec<&T>> = HashMap::new();
        for item in items {
            let stratum = strata_fn(item);
            strata.entry(stratum).or_default().push(item);
        }
        
        let mut result = Vec::new();
        let mut rng = SecureRng::new();
        
        // Sample from each stratum
        for (_, stratum_items) in strata {
            let sample_size = samples_per_stratum.min(stratum_items.len());
            let sampled = rng.sample(&stratum_items, sample_size);
            result.extend(sampled.into_iter().map(|item| (*item).clone()));
        }
        
        result
    }
    
    /// Bootstrap sampling (sampling with replacement)
    pub fn bootstrap_sample<T: Clone>(items: &[T], size: usize) -> Vec<T> {
        let mut rng = SecureRng::new();
        let mut result = Vec::with_capacity(size);
        
        for _ in 0..size {
            if let Some(item) = rng.choose(items) {
                result.push(item.clone());
            }
        }
        
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_secure_rng_basic() {
        let mut rng = SecureRng::new();
        
        // Test different types
        let _u64_val = rng.next_u64();
        let _u32_val = rng.next_u32();
        let _u16_val = rng.next_u16();
        let _u8_val = rng.next_u8();
        let _bool_val = rng.next_bool();
        
        // Test ranges
        let f64_val = rng.next_f64();
        assert!(f64_val >= 0.0 && f64_val < 1.0);
        
        let f32_val = rng.next_f32();
        assert!(f32_val >= 0.0 && f32_val < 1.0);
    }
    
    #[test]
    fn test_rng_ranges() {
        let mut rng = SecureRng::new();
        
        // Test u64 range
        for _ in 0..100 {
            let val = rng.range_u64(10, 20).unwrap();
            assert!(val >= 10 && val < 20);
        }
        
        // Test i64 range
        for _ in 0..100 {
            let val = rng.range_i64(-10, 10).unwrap();
            assert!(val >= -10 && val < 10);
        }
        
        // Test f64 range
        for _ in 0..100 {
            let val = rng.range_f64(1.0, 2.0).unwrap();
            assert!(val >= 1.0 && val < 2.0);
        }
    }
    
    #[test]
    fn test_rng_bytes() {
        let mut rng = SecureRng::new();
        
        let bytes = rng.bytes(32);
        assert_eq!(bytes.len(), 32);
        
        let hex = rng.hex_string(16);
        assert_eq!(hex.len(), 16);
    }
    
    #[test]
    fn test_shuffle_and_choose() {
        let mut rng = SecureRng::new();
        let mut items = vec![1, 2, 3, 4, 5];
        let original = items.clone();
        
        rng.shuffle(&mut items);
        // Items should be same but potentially in different order
        items.sort();
        assert_eq!(items, original);
        
        let chosen = rng.choose(&original);
        assert!(chosen.is_some());
        assert!(original.contains(chosen.unwrap()));
    }
    
    #[test]
    fn test_weighted_choice() {
        let mut rng = SecureRng::new();
        let items = vec![
            ("rare", 0.1),
            ("common", 0.9),
        ];
        
        let mut rare_count = 0;
        let mut common_count = 0;
        
        for _ in 0..1000 {
            match rng.weighted_choice(&items) {
                Some(&"rare") => rare_count += 1,
                Some(&"common") => common_count += 1,
                _ => {},
            }
        }
        
        // Common should be much more frequent than rare
        assert!(common_count > rare_count);
    }
    
    #[test]
    fn test_random_utils() {
        let uuid = RandomUtils::uuid();
        assert_eq!(uuid.len(), 36); // UUID format: 8-4-4-4-12
        
        let nonce = RandomUtils::mining_nonce();
        assert!(nonce > 0);
        
        let tx_id = RandomUtils::transaction_id();
        assert_eq!(tx_id.len(), 64);
        
        let addr = RandomUtils::address();
        assert!(addr.starts_with("addr_"));
        
        let password = RandomUtils::password(12);
        assert_eq!(password.len(), 12);
        
        let alphanumeric = RandomUtils::alphanumeric(10);
        assert_eq!(alphanumeric.len(), 10);
        assert!(alphanumeric.chars().all(|c| c.is_alphanumeric()));
    }
    
    #[test]
    fn test_probability_utils() {
        // Test dice rolls
        for _ in 0..100 {
            let roll = ProbabilityUtils::dice_roll();
            assert!(roll >= 1 && roll <= 6);
            
            let roll_n = ProbabilityUtils::dice_roll_n(10);
            assert!(roll_n >= 1 && roll_n <= 10);
        }
        
        // Test normal distribution (basic sanity check)
        let normal = ProbabilityUtils::normal_distribution(0.0, 1.0);
        assert!(normal.is_finite());
        
        // Test exponential distribution
        let exp = ProbabilityUtils::exponential_distribution(1.0);
        assert!(exp >= 0.0);
        
        // Test Poisson distribution
        let poisson = ProbabilityUtils::poisson_distribution(2.0);
        assert!(poisson < 100); // Reasonable upper bound
    }
    
    #[test]
    fn test_sampling_utils() {
        let items = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        
        // Test reservoir sampling
        let sample = SamplingUtils::reservoir_sample(&items, 5);
        assert_eq!(sample.len(), 5);
        for item in &sample {
            assert!(items.contains(item));
        }
        
        // Test bootstrap sampling
        let bootstrap = SamplingUtils::bootstrap_sample(&items, 15);
        assert_eq!(bootstrap.len(), 15);
        for item in &bootstrap {
            assert!(items.contains(item));
        }
    }
    
    #[test]
    fn test_rng_deterministic() {
        let mut rng1 = SecureRng::with_seed(12345);
        let mut rng2 = SecureRng::with_seed(12345);
        
        // Same seed should produce same sequence
        for _ in 0..10 {
            assert_eq!(rng1.next_u64(), rng2.next_u64());
        }
    }
}