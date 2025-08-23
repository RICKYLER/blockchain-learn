//! Mathematical utilities for the LedgerDB blockchain.
//!
//! This module provides mathematical functions for blockchain operations,
//! including difficulty calculations, statistical functions, and numerical utilities.

use crate::error::LedgerError;
use std::cmp::Ordering;

/// Mathematical utilities
pub struct MathUtils;

impl MathUtils {
    /// Calculate the average of a slice of u64 values
    pub fn average_u64(values: &[u64]) -> Option<u64> {
        if values.is_empty() {
            return None;
        }
        
        let sum: u128 = values.iter().map(|&x| x as u128).sum();
        Some((sum / values.len() as u128) as u64)
    }
    
    /// Calculate the median of a slice of u64 values
    pub fn median_u64(values: &[u64]) -> Option<u64> {
        if values.is_empty() {
            return None;
        }
        
        let mut sorted = values.to_vec();
        sorted.sort_unstable();
        
        let len = sorted.len();
        if len % 2 == 0 {
            // Even number of elements - average of middle two
            let mid1 = sorted[len / 2 - 1] as u128;
            let mid2 = sorted[len / 2] as u128;
            Some(((mid1 + mid2) / 2) as u64)
        } else {
            // Odd number of elements - middle element
            Some(sorted[len / 2])
        }
    }
    
    /// Calculate the standard deviation of u64 values
    pub fn std_deviation_u64(values: &[u64]) -> Option<f64> {
        if values.len() < 2 {
            return None;
        }
        
        let mean = Self::average_u64(values)? as f64;
        let variance: f64 = values
            .iter()
            .map(|&x| {
                let diff = x as f64 - mean;
                diff * diff
            })
            .sum::<f64>() / (values.len() - 1) as f64;
        
        Some(variance.sqrt())
    }
    
    /// Calculate percentile of u64 values
    pub fn percentile_u64(values: &[u64], percentile: f64) -> Option<u64> {
        if values.is_empty() || percentile < 0.0 || percentile > 100.0 {
            return None;
        }
        
        let mut sorted = values.to_vec();
        sorted.sort_unstable();
        
        let index = (percentile / 100.0) * (sorted.len() - 1) as f64;
        let lower = index.floor() as usize;
        let upper = index.ceil() as usize;
        
        if lower == upper {
            Some(sorted[lower])
        } else {
            let weight = index - lower as f64;
            let interpolated = sorted[lower] as f64 * (1.0 - weight) + sorted[upper] as f64 * weight;
            Some(interpolated as u64)
        }
    }
    
    /// Calculate moving average
    pub fn moving_average(values: &[u64], window_size: usize) -> Vec<u64> {
        if window_size == 0 || window_size > values.len() {
            return Vec::new();
        }
        
        let mut result = Vec::new();
        
        for i in 0..=(values.len() - window_size) {
            let window = &values[i..i + window_size];
            if let Some(avg) = Self::average_u64(window) {
                result.push(avg);
            }
        }
        
        result
    }
    
    /// Calculate exponential moving average
    pub fn exponential_moving_average(values: &[u64], alpha: f64) -> Vec<f64> {
        if values.is_empty() || alpha <= 0.0 || alpha > 1.0 {
            return Vec::new();
        }
        
        let mut result = Vec::with_capacity(values.len());
        let mut ema = values[0] as f64;
        result.push(ema);
        
        for &value in &values[1..] {
            ema = alpha * value as f64 + (1.0 - alpha) * ema;
            result.push(ema);
        }
        
        result
    }
    
    /// Calculate greatest common divisor
    pub fn gcd(mut a: u64, mut b: u64) -> u64 {
        while b != 0 {
            let temp = b;
            b = a % b;
            a = temp;
        }
        a
    }
    
    /// Calculate least common multiple
    pub fn lcm(a: u64, b: u64) -> u64 {
        if a == 0 || b == 0 {
            return 0;
        }
        (a / Self::gcd(a, b)) * b
    }
    
    /// Check if a number is prime
    pub fn is_prime(n: u64) -> bool {
        if n < 2 {
            return false;
        }
        if n == 2 {
            return true;
        }
        if n % 2 == 0 {
            return false;
        }
        
        let sqrt_n = (n as f64).sqrt() as u64;
        for i in (3..=sqrt_n).step_by(2) {
            if n % i == 0 {
                return false;
            }
        }
        
        true
    }
    
    /// Calculate factorial
    pub fn factorial(n: u64) -> Result<u64, LedgerError> {
        if n > 20 {
            return Err(LedgerError::Validation(
                "Factorial overflow for values > 20".to_string()
            ));
        }
        
        let mut result = 1u64;
        for i in 2..=n {
            result = result.checked_mul(i).ok_or_else(|| {
                LedgerError::Validation("Factorial overflow".to_string())
            })?;
        }
        
        Ok(result)
    }
    
    /// Calculate power with modulo (for cryptographic operations)
    pub fn mod_pow(mut base: u64, mut exp: u64, modulus: u64) -> u64 {
        if modulus == 1 {
            return 0;
        }
        
        let mut result = 1;
        base %= modulus;
        
        while exp > 0 {
            if exp % 2 == 1 {
                result = (result as u128 * base as u128 % modulus as u128) as u64;
            }
            exp >>= 1;
            base = (base as u128 * base as u128 % modulus as u128) as u64;
        }
        
        result
    }
    
    /// Calculate logarithm base 2 (integer)
    pub fn log2(n: u64) -> Option<u32> {
        if n == 0 {
            return None;
        }
        Some(63 - n.leading_zeros())
    }
    
    /// Check if a number is a power of 2
    pub fn is_power_of_2(n: u64) -> bool {
        n != 0 && (n & (n - 1)) == 0
    }
    
    /// Round up to next power of 2
    pub fn next_power_of_2(n: u64) -> u64 {
        if n == 0 {
            return 1;
        }
        if Self::is_power_of_2(n) {
            return n;
        }
        
        1u64 << (64 - (n - 1).leading_zeros())
    }
    
    /// Calculate Hamming weight (number of 1 bits)
    pub fn hamming_weight(n: u64) -> u32 {
        n.count_ones()
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
    
    /// Map value from one range to another
    pub fn map_range(value: f64, from_min: f64, from_max: f64, to_min: f64, to_max: f64) -> f64 {
        let normalized = (value - from_min) / (from_max - from_min);
        Self::lerp(to_min, to_max, normalized)
    }
}

/// Difficulty calculation utilities
pub struct DifficultyUtils;

impl DifficultyUtils {
    /// Calculate target from difficulty bits (Bitcoin-style)
    pub fn bits_to_target(bits: u32) -> Result<[u8; 32], LedgerError> {
        let exponent = (bits >> 24) as usize;
        let mantissa = bits & 0x00FFFFFF;
        
        if exponent > 32 {
            return Err(LedgerError::Validation(
                "Invalid difficulty bits: exponent too large".to_string()
            ));
        }
        
        let mut target = [0u8; 32];
        
        if exponent <= 3 {
            let mantissa_bytes = mantissa.to_be_bytes();
            let start_idx = 32 - exponent;
            for i in 0..exponent {
                if i < 3 {
                    target[start_idx + i] = mantissa_bytes[1 + i];
                }
            }
        } else {
            let mantissa_bytes = mantissa.to_be_bytes();
            let start_idx = 32 - exponent;
            target[start_idx] = mantissa_bytes[1];
            target[start_idx + 1] = mantissa_bytes[2];
            target[start_idx + 2] = mantissa_bytes[3];
        }
        
        Ok(target)
    }
    
    /// Calculate difficulty bits from target
    pub fn target_to_bits(target: &[u8; 32]) -> u32 {
        // Find the first non-zero byte
        let mut exponent = 32;
        for (i, &byte) in target.iter().enumerate() {
            if byte != 0 {
                exponent = 32 - i;
                break;
            }
        }
        
        if exponent == 0 {
            return 0;
        }
        
        let start_idx = 32 - exponent;
        let mut mantissa = 0u32;
        
        // Extract up to 3 bytes for mantissa
        for i in 0..3.min(exponent) {
            mantissa = (mantissa << 8) | target[start_idx + i] as u32;
        }
        
        // Adjust if mantissa is too large
        if mantissa > 0x7FFFFF {
            mantissa >>= 8;
            exponent += 1;
        }
        
        ((exponent as u32) << 24) | mantissa
    }
    
    /// Calculate difficulty from target
    pub fn target_to_difficulty(target: &[u8; 32]) -> f64 {
        // Maximum target (difficulty 1)
        let max_target = {
            let mut max = [0u8; 32];
            max[0] = 0x1d;
            max[1] = 0x00;
            max[2] = 0xff;
            max[3] = 0xff;
            max
        };
        
        let max_target_num = Self::target_to_number(&max_target);
        let target_num = Self::target_to_number(target);
        
        if target_num == 0.0 {
            return f64::INFINITY;
        }
        
        max_target_num / target_num
    }
    
    /// Convert target bytes to number for calculation
    fn target_to_number(target: &[u8; 32]) -> f64 {
        let mut result = 0.0;
        let mut multiplier = 1.0;
        
        for &byte in target.iter().rev() {
            result += byte as f64 * multiplier;
            multiplier *= 256.0;
        }
        
        result
    }
    
    /// Adjust difficulty based on time taken
    pub fn adjust_difficulty(
        current_difficulty: u32,
        target_time: u64,
        actual_time: u64,
        max_adjustment: f64,
    ) -> u32 {
        if actual_time == 0 {
            return current_difficulty;
        }
        
        let ratio = target_time as f64 / actual_time as f64;
        let clamped_ratio = MathUtils::clamp(ratio, 1.0 / max_adjustment, max_adjustment);
        
        let new_difficulty = current_difficulty as f64 * clamped_ratio;
        new_difficulty.round() as u32
    }
    
    /// Check if hash meets difficulty target
    pub fn meets_target(hash: &[u8; 32], target: &[u8; 32]) -> bool {
        for (h, t) in hash.iter().zip(target.iter()) {
            match h.cmp(t) {
                Ordering::Less => return true,
                Ordering::Greater => return false,
                Ordering::Equal => continue,
            }
        }
        true // Equal is considered meeting the target
    }
    
    /// Calculate work from target
    pub fn target_to_work(target: &[u8; 32]) -> [u8; 32] {
        // Work = 2^256 / (target + 1)
        // This is a simplified implementation
        let mut work = [0xFFu8; 32];
        
        // Find first non-zero byte in target
        for (i, &byte) in target.iter().enumerate() {
            if byte != 0 {
                // Approximate work calculation
                work[i] = 0xFF - byte;
                break;
            }
        }
        
        work
    }
}

/// Statistical utilities for blockchain metrics
pub struct StatsUtils;

impl StatsUtils {
    /// Calculate hash rate from difficulty and block time
    pub fn calculate_hash_rate(difficulty: f64, block_time_seconds: f64) -> f64 {
        if block_time_seconds <= 0.0 {
            return 0.0;
        }
        
        // Simplified hash rate calculation
        // Hash rate = difficulty * 2^32 / block_time
        difficulty * 4_294_967_296.0 / block_time_seconds
    }
    
    /// Calculate network security (total work)
    pub fn calculate_total_work(difficulties: &[f64], block_times: &[f64]) -> f64 {
        if difficulties.len() != block_times.len() {
            return 0.0;
        }
        
        difficulties
            .iter()
            .zip(block_times.iter())
            .map(|(&diff, &time)| Self::calculate_hash_rate(diff, time) * time)
            .sum()
    }
    
    /// Calculate transaction throughput (TPS)
    pub fn calculate_tps(transaction_count: u64, time_period_seconds: f64) -> f64 {
        if time_period_seconds <= 0.0 {
            return 0.0;
        }
        
        transaction_count as f64 / time_period_seconds
    }
    
    /// Calculate fee statistics
    pub fn calculate_fee_stats(fees: &[u64]) -> FeeStats {
        if fees.is_empty() {
            return FeeStats::default();
        }
        
        let mut sorted_fees = fees.to_vec();
        sorted_fees.sort_unstable();
        
        FeeStats {
            min: sorted_fees[0],
            max: sorted_fees[sorted_fees.len() - 1],
            average: MathUtils::average_u64(&sorted_fees).unwrap_or(0),
            median: MathUtils::median_u64(&sorted_fees).unwrap_or(0),
            percentile_25: MathUtils::percentile_u64(&sorted_fees, 25.0).unwrap_or(0),
            percentile_75: MathUtils::percentile_u64(&sorted_fees, 75.0).unwrap_or(0),
            percentile_90: MathUtils::percentile_u64(&sorted_fees, 90.0).unwrap_or(0),
            percentile_95: MathUtils::percentile_u64(&sorted_fees, 95.0).unwrap_or(0),
            std_deviation: MathUtils::std_deviation_u64(&sorted_fees).unwrap_or(0.0),
        }
    }
}

/// Fee statistics structure
#[derive(Debug, Clone, Default)]
pub struct FeeStats {
    pub min: u64,
    pub max: u64,
    pub average: u64,
    pub median: u64,
    pub percentile_25: u64,
    pub percentile_75: u64,
    pub percentile_90: u64,
    pub percentile_95: u64,
    pub std_deviation: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_average_and_median() {
        let values = vec![1, 2, 3, 4, 5];
        assert_eq!(MathUtils::average_u64(&values), Some(3));
        assert_eq!(MathUtils::median_u64(&values), Some(3));
        
        let values = vec![1, 2, 3, 4];
        assert_eq!(MathUtils::average_u64(&values), Some(2));
        assert_eq!(MathUtils::median_u64(&values), Some(2));
    }
    
    #[test]
    fn test_percentile() {
        let values = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        assert_eq!(MathUtils::percentile_u64(&values, 50.0), Some(5));
        assert_eq!(MathUtils::percentile_u64(&values, 90.0), Some(9));
    }
    
    #[test]
    fn test_gcd_lcm() {
        assert_eq!(MathUtils::gcd(12, 8), 4);
        assert_eq!(MathUtils::lcm(12, 8), 24);
    }
    
    #[test]
    fn test_is_prime() {
        assert!(!MathUtils::is_prime(1));
        assert!(MathUtils::is_prime(2));
        assert!(MathUtils::is_prime(3));
        assert!(!MathUtils::is_prime(4));
        assert!(MathUtils::is_prime(17));
        assert!(!MathUtils::is_prime(25));
    }
    
    #[test]
    fn test_factorial() {
        assert_eq!(MathUtils::factorial(0).unwrap(), 1);
        assert_eq!(MathUtils::factorial(5).unwrap(), 120);
        assert!(MathUtils::factorial(25).is_err());
    }
    
    #[test]
    fn test_power_of_2() {
        assert!(MathUtils::is_power_of_2(1));
        assert!(MathUtils::is_power_of_2(2));
        assert!(MathUtils::is_power_of_2(8));
        assert!(!MathUtils::is_power_of_2(3));
        assert!(!MathUtils::is_power_of_2(6));
        
        assert_eq!(MathUtils::next_power_of_2(5), 8);
        assert_eq!(MathUtils::next_power_of_2(8), 8);
        assert_eq!(MathUtils::next_power_of_2(9), 16);
    }
    
    #[test]
    fn test_clamp() {
        assert_eq!(MathUtils::clamp(5, 1, 10), 5);
        assert_eq!(MathUtils::clamp(0, 1, 10), 1);
        assert_eq!(MathUtils::clamp(15, 1, 10), 10);
    }
    
    #[test]
    fn test_lerp() {
        assert_eq!(MathUtils::lerp(0.0, 10.0, 0.5), 5.0);
        assert_eq!(MathUtils::lerp(0.0, 10.0, 0.0), 0.0);
        assert_eq!(MathUtils::lerp(0.0, 10.0, 1.0), 10.0);
    }
    
    #[test]
    fn test_difficulty_bits() {
        let bits = 0x1d00ffff; // Bitcoin genesis block difficulty
        let target = DifficultyUtils::bits_to_target(bits).unwrap();
        let back_to_bits = DifficultyUtils::target_to_bits(&target);
        
        // Should be approximately equal (some precision loss is expected)
        assert!((bits as i64 - back_to_bits as i64).abs() < 256);
    }
    
    #[test]
    fn test_hash_rate_calculation() {
        let difficulty = 1000.0;
        let block_time = 600.0; // 10 minutes
        let hash_rate = StatsUtils::calculate_hash_rate(difficulty, block_time);
        assert!(hash_rate > 0.0);
    }
    
    #[test]
    fn test_fee_stats() {
        let fees = vec![100, 200, 150, 300, 250, 180, 220];
        let stats = StatsUtils::calculate_fee_stats(&fees);
        
        assert_eq!(stats.min, 100);
        assert_eq!(stats.max, 300);
        assert!(stats.average > 0);
        assert!(stats.median > 0);
    }
    
    #[test]
    fn test_moving_average() {
        let values = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let ma = MathUtils::moving_average(&values, 3);
        
        assert_eq!(ma.len(), 8); // 10 - 3 + 1
        assert_eq!(ma[0], 2); // (1+2+3)/3
        assert_eq!(ma[1], 3); // (2+3+4)/3
    }
}