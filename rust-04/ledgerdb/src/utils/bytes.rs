//! Byte manipulation utilities for the LedgerDB blockchain.
//!
//! This module provides utilities for working with byte arrays,
//! serialization, deserialization, and byte-level operations.

use crate::error::LedgerError;
use std::convert::TryInto;

/// Byte utilities
pub struct ByteUtils;

impl ByteUtils {
    /// Convert u16 to bytes (big-endian)
    pub fn u16_to_bytes(value: u16) -> [u8; 2] {
        value.to_be_bytes()
    }
    
    /// Convert bytes to u16 (big-endian)
    pub fn bytes_to_u16(bytes: &[u8]) -> Result<u16, LedgerError> {
        if bytes.len() < 2 {
            return Err(LedgerError::Serialization(
                "Not enough bytes for u16".to_string()
            ));
        }
        Ok(u16::from_be_bytes([bytes[0], bytes[1]]))
    }
    
    /// Convert u32 to bytes (big-endian)
    pub fn u32_to_bytes(value: u32) -> [u8; 4] {
        value.to_be_bytes()
    }
    
    /// Convert bytes to u32 (big-endian)
    pub fn bytes_to_u32(bytes: &[u8]) -> Result<u32, LedgerError> {
        if bytes.len() < 4 {
            return Err(LedgerError::Serialization(
                "Not enough bytes for u32".to_string()
            ));
        }
        let array: [u8; 4] = bytes[0..4].try_into().map_err(|_| {
            LedgerError::Serialization("Failed to convert bytes to u32".to_string())
        })?;
        Ok(u32::from_be_bytes(array))
    }
    
    /// Convert u64 to bytes (big-endian)
    pub fn u64_to_bytes(value: u64) -> [u8; 8] {
        value.to_be_bytes()
    }
    
    /// Convert bytes to u64 (big-endian)
    pub fn bytes_to_u64(bytes: &[u8]) -> Result<u64, LedgerError> {
        if bytes.len() < 8 {
            return Err(LedgerError::Serialization(
                "Not enough bytes for u64".to_string()
            ));
        }
        let array: [u8; 8] = bytes[0..8].try_into().map_err(|_| {
            LedgerError::Serialization("Failed to convert bytes to u64".to_string())
        })?;
        Ok(u64::from_be_bytes(array))
    }
    
    /// Convert u128 to bytes (big-endian)
    pub fn u128_to_bytes(value: u128) -> [u8; 16] {
        value.to_be_bytes()
    }
    
    /// Convert bytes to u128 (big-endian)
    pub fn bytes_to_u128(bytes: &[u8]) -> Result<u128, LedgerError> {
        if bytes.len() < 16 {
            return Err(LedgerError::Serialization(
                "Not enough bytes for u128".to_string()
            ));
        }
        let array: [u8; 16] = bytes[0..16].try_into().map_err(|_| {
            LedgerError::Serialization("Failed to convert bytes to u128".to_string())
        })?;
        Ok(u128::from_be_bytes(array))
    }
    
    /// Convert string to bytes (UTF-8)
    pub fn string_to_bytes(s: &str) -> Vec<u8> {
        s.as_bytes().to_vec()
    }
    
    /// Convert bytes to string (UTF-8)
    pub fn bytes_to_string(bytes: &[u8]) -> Result<String, LedgerError> {
        String::from_utf8(bytes.to_vec()).map_err(|e| {
            LedgerError::Serialization(format!("Invalid UTF-8 bytes: {}", e))
        })
    }
    
    /// Convert hex string to bytes
    pub fn hex_to_bytes(hex: &str) -> Result<Vec<u8>, LedgerError> {
        // Remove 0x prefix if present
        let hex = hex.strip_prefix("0x").unwrap_or(hex);
        
        if hex.len() % 2 != 0 {
            return Err(LedgerError::Serialization(
                "Hex string must have even length".to_string()
            ));
        }
        
        hex::decode(hex).map_err(|e| {
            LedgerError::Serialization(format!("Invalid hex string: {}", e))
        })
    }
    
    /// Convert bytes to hex string
    pub fn bytes_to_hex(bytes: &[u8]) -> String {
        hex::encode(bytes)
    }
    
    /// Convert bytes to hex string with 0x prefix
    pub fn bytes_to_hex_prefixed(bytes: &[u8]) -> String {
        format!("0x{}", hex::encode(bytes))
    }
    
    /// XOR two byte arrays
    pub fn xor_bytes(a: &[u8], b: &[u8]) -> Result<Vec<u8>, LedgerError> {
        if a.len() != b.len() {
            return Err(LedgerError::Serialization(
                "Byte arrays must have same length for XOR".to_string()
            ));
        }
        
        Ok(a.iter().zip(b.iter()).map(|(x, y)| x ^ y).collect())
    }
    
    /// Pad bytes to specified length with zeros
    pub fn pad_bytes(bytes: &[u8], target_len: usize) -> Vec<u8> {
        let mut padded = bytes.to_vec();
        if padded.len() < target_len {
            padded.resize(target_len, 0);
        }
        padded
    }
    
    /// Truncate bytes to specified length
    pub fn truncate_bytes(bytes: &[u8], max_len: usize) -> Vec<u8> {
        if bytes.len() <= max_len {
            bytes.to_vec()
        } else {
            bytes[0..max_len].to_vec()
        }
    }
    
    /// Reverse byte order
    pub fn reverse_bytes(bytes: &[u8]) -> Vec<u8> {
        let mut reversed = bytes.to_vec();
        reversed.reverse();
        reversed
    }
    
    /// Check if bytes are all zeros
    pub fn is_zero_bytes(bytes: &[u8]) -> bool {
        bytes.iter().all(|&b| b == 0)
    }
    
    /// Generate random bytes
    pub fn random_bytes(len: usize) -> Vec<u8> {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        use std::time::SystemTime;
        
        let mut bytes = Vec::with_capacity(len);
        let mut hasher = DefaultHasher::new();
        
        for i in 0..len {
            SystemTime::now().hash(&mut hasher);
            i.hash(&mut hasher);
            let hash = hasher.finish();
            bytes.push((hash & 0xFF) as u8);
        }
        
        bytes
    }
    
    /// Compare bytes in constant time (to prevent timing attacks)
    pub fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
        if a.len() != b.len() {
            return false;
        }
        
        let mut result = 0u8;
        for (x, y) in a.iter().zip(b.iter()) {
            result |= x ^ y;
        }
        
        result == 0
    }
    
    /// Calculate Hamming distance between two byte arrays
    pub fn hamming_distance(a: &[u8], b: &[u8]) -> Result<u32, LedgerError> {
        if a.len() != b.len() {
            return Err(LedgerError::Serialization(
                "Byte arrays must have same length for Hamming distance".to_string()
            ));
        }
        
        let mut distance = 0;
        for (x, y) in a.iter().zip(b.iter()) {
            distance += (x ^ y).count_ones();
        }
        
        Ok(distance)
    }
    
    /// Find pattern in bytes
    pub fn find_pattern(haystack: &[u8], needle: &[u8]) -> Option<usize> {
        if needle.is_empty() {
            return Some(0);
        }
        
        if needle.len() > haystack.len() {
            return None;
        }
        
        for i in 0..=(haystack.len() - needle.len()) {
            if &haystack[i..i + needle.len()] == needle {
                return Some(i);
            }
        }
        
        None
    }
    
    /// Replace pattern in bytes
    pub fn replace_pattern(
        haystack: &[u8],
        needle: &[u8],
        replacement: &[u8],
    ) -> Vec<u8> {
        if needle.is_empty() {
            return haystack.to_vec();
        }
        
        let mut result = Vec::new();
        let mut i = 0;
        
        while i < haystack.len() {
            if i + needle.len() <= haystack.len() && &haystack[i..i + needle.len()] == needle {
                result.extend_from_slice(replacement);
                i += needle.len();
            } else {
                result.push(haystack[i]);
                i += 1;
            }
        }
        
        result
    }
}

/// Variable-length integer encoding (VarInt)
pub struct VarInt;

impl VarInt {
    /// Encode u64 as variable-length integer
    pub fn encode(mut value: u64) -> Vec<u8> {
        let mut bytes = Vec::new();
        
        while value >= 0x80 {
            bytes.push((value & 0x7F) as u8 | 0x80);
            value >>= 7;
        }
        bytes.push(value as u8);
        
        bytes
    }
    
    /// Decode variable-length integer to u64
    pub fn decode(bytes: &[u8]) -> Result<(u64, usize), LedgerError> {
        let mut value = 0u64;
        let mut shift = 0;
        let mut pos = 0;
        
        for &byte in bytes {
            if shift >= 64 {
                return Err(LedgerError::Serialization(
                    "VarInt overflow".to_string()
                ));
            }
            
            value |= ((byte & 0x7F) as u64) << shift;
            pos += 1;
            
            if byte & 0x80 == 0 {
                return Ok((value, pos));
            }
            
            shift += 7;
        }
        
        Err(LedgerError::Serialization(
            "Incomplete VarInt".to_string()
        ))
    }
    
    /// Get encoded size of a value
    pub fn encoded_size(mut value: u64) -> usize {
        let mut size = 1;
        while value >= 0x80 {
            size += 1;
            value >>= 7;
        }
        size
    }
}

/// Byte buffer for reading/writing
#[derive(Debug, Clone)]
pub struct ByteBuffer {
    data: Vec<u8>,
    position: usize,
}

impl ByteBuffer {
    /// Create new byte buffer
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            position: 0,
        }
    }
    
    /// Create byte buffer from existing data
    pub fn from_bytes(data: Vec<u8>) -> Self {
        Self {
            data,
            position: 0,
        }
    }
    
    /// Get buffer data
    pub fn data(&self) -> &[u8] {
        &self.data
    }
    
    /// Get buffer length
    pub fn len(&self) -> usize {
        self.data.len()
    }
    
    /// Check if buffer is empty
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
    
    /// Get current position
    pub fn position(&self) -> usize {
        self.position
    }
    
    /// Set position
    pub fn set_position(&mut self, pos: usize) -> Result<(), LedgerError> {
        if pos > self.data.len() {
            return Err(LedgerError::Serialization(
                "Position exceeds buffer length".to_string()
            ));
        }
        self.position = pos;
        Ok(())
    }
    
    /// Reset position to beginning
    pub fn reset(&mut self) {
        self.position = 0;
    }
    
    /// Check if at end of buffer
    pub fn is_at_end(&self) -> bool {
        self.position >= self.data.len()
    }
    
    /// Remaining bytes
    pub fn remaining(&self) -> usize {
        if self.position < self.data.len() {
            self.data.len() - self.position
        } else {
            0
        }
    }
    
    /// Write bytes
    pub fn write_bytes(&mut self, bytes: &[u8]) {
        self.data.extend_from_slice(bytes);
    }
    
    /// Write u8
    pub fn write_u8(&mut self, value: u8) {
        self.data.push(value);
    }
    
    /// Write u16 (big-endian)
    pub fn write_u16(&mut self, value: u16) {
        self.data.extend_from_slice(&value.to_be_bytes());
    }
    
    /// Write u32 (big-endian)
    pub fn write_u32(&mut self, value: u32) {
        self.data.extend_from_slice(&value.to_be_bytes());
    }
    
    /// Write u64 (big-endian)
    pub fn write_u64(&mut self, value: u64) {
        self.data.extend_from_slice(&value.to_be_bytes());
    }
    
    /// Write string (length-prefixed)
    pub fn write_string(&mut self, s: &str) {
        let bytes = s.as_bytes();
        self.write_u32(bytes.len() as u32);
        self.write_bytes(bytes);
    }
    
    /// Read bytes
    pub fn read_bytes(&mut self, len: usize) -> Result<Vec<u8>, LedgerError> {
        if self.position + len > self.data.len() {
            return Err(LedgerError::Serialization(
                "Not enough bytes to read".to_string()
            ));
        }
        
        let bytes = self.data[self.position..self.position + len].to_vec();
        self.position += len;
        Ok(bytes)
    }
    
    /// Read u8
    pub fn read_u8(&mut self) -> Result<u8, LedgerError> {
        if self.position >= self.data.len() {
            return Err(LedgerError::Serialization(
                "Not enough bytes to read u8".to_string()
            ));
        }
        
        let value = self.data[self.position];
        self.position += 1;
        Ok(value)
    }
    
    /// Read u16 (big-endian)
    pub fn read_u16(&mut self) -> Result<u16, LedgerError> {
        let bytes = self.read_bytes(2)?;
        ByteUtils::bytes_to_u16(&bytes)
    }
    
    /// Read u32 (big-endian)
    pub fn read_u32(&mut self) -> Result<u32, LedgerError> {
        let bytes = self.read_bytes(4)?;
        ByteUtils::bytes_to_u32(&bytes)
    }
    
    /// Read u64 (big-endian)
    pub fn read_u64(&mut self) -> Result<u64, LedgerError> {
        let bytes = self.read_bytes(8)?;
        ByteUtils::bytes_to_u64(&bytes)
    }
    
    /// Read string (length-prefixed)
    pub fn read_string(&mut self) -> Result<String, LedgerError> {
        let len = self.read_u32()? as usize;
        let bytes = self.read_bytes(len)?;
        ByteUtils::bytes_to_string(&bytes)
    }
    
    /// Clear buffer
    pub fn clear(&mut self) {
        self.data.clear();
        self.position = 0;
    }
    
    /// Truncate buffer
    pub fn truncate(&mut self, len: usize) {
        self.data.truncate(len);
        if self.position > len {
            self.position = len;
        }
    }
}

impl Default for ByteBuffer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_integer_conversions() {
        // u16
        let value = 0x1234u16;
        let bytes = ByteUtils::u16_to_bytes(value);
        assert_eq!(ByteUtils::bytes_to_u16(&bytes).unwrap(), value);
        
        // u32
        let value = 0x12345678u32;
        let bytes = ByteUtils::u32_to_bytes(value);
        assert_eq!(ByteUtils::bytes_to_u32(&bytes).unwrap(), value);
        
        // u64
        let value = 0x123456789ABCDEFu64;
        let bytes = ByteUtils::u64_to_bytes(value);
        assert_eq!(ByteUtils::bytes_to_u64(&bytes).unwrap(), value);
    }
    
    #[test]
    fn test_hex_conversions() {
        let bytes = vec![0x12, 0x34, 0xAB, 0xCD];
        let hex = ByteUtils::bytes_to_hex(&bytes);
        assert_eq!(hex, "1234abcd");
        
        let decoded = ByteUtils::hex_to_bytes(&hex).unwrap();
        assert_eq!(decoded, bytes);
        
        // Test with 0x prefix
        let hex_prefixed = "0x1234abcd";
        let decoded = ByteUtils::hex_to_bytes(hex_prefixed).unwrap();
        assert_eq!(decoded, bytes);
    }
    
    #[test]
    fn test_string_conversions() {
        let text = "Hello, World!";
        let bytes = ByteUtils::string_to_bytes(text);
        let decoded = ByteUtils::bytes_to_string(&bytes).unwrap();
        assert_eq!(decoded, text);
    }
    
    #[test]
    fn test_xor_bytes() {
        let a = vec![0x12, 0x34, 0x56];
        let b = vec![0xAB, 0xCD, 0xEF];
        let result = ByteUtils::xor_bytes(&a, &b).unwrap();
        let expected = vec![0x12 ^ 0xAB, 0x34 ^ 0xCD, 0x56 ^ 0xEF];
        assert_eq!(result, expected);
    }
    
    #[test]
    fn test_pad_and_truncate() {
        let bytes = vec![1, 2, 3];
        
        let padded = ByteUtils::pad_bytes(&bytes, 5);
        assert_eq!(padded, vec![1, 2, 3, 0, 0]);
        
        let truncated = ByteUtils::truncate_bytes(&padded, 2);
        assert_eq!(truncated, vec![1, 2]);
    }
    
    #[test]
    fn test_constant_time_eq() {
        let a = vec![1, 2, 3, 4];
        let b = vec![1, 2, 3, 4];
        let c = vec![1, 2, 3, 5];
        
        assert!(ByteUtils::constant_time_eq(&a, &b));
        assert!(!ByteUtils::constant_time_eq(&a, &c));
    }
    
    #[test]
    fn test_hamming_distance() {
        let a = vec![0b10101010];
        let b = vec![0b01010101];
        let distance = ByteUtils::hamming_distance(&a, &b).unwrap();
        assert_eq!(distance, 8); // All bits are different
    }
    
    #[test]
    fn test_find_pattern() {
        let haystack = vec![1, 2, 3, 4, 5, 3, 4, 6];
        let needle = vec![3, 4];
        
        let pos = ByteUtils::find_pattern(&haystack, &needle);
        assert_eq!(pos, Some(2)); // First occurrence
    }
    
    #[test]
    fn test_varint() {
        let values = vec![0, 127, 128, 255, 256, 16383, 16384, u64::MAX];
        
        for value in values {
            let encoded = VarInt::encode(value);
            let (decoded, len) = VarInt::decode(&encoded).unwrap();
            assert_eq!(decoded, value);
            assert_eq!(len, encoded.len());
            assert_eq!(len, VarInt::encoded_size(value));
        }
    }
    
    #[test]
    fn test_byte_buffer() {
        let mut buffer = ByteBuffer::new();
        
        // Write data
        buffer.write_u8(0x12);
        buffer.write_u16(0x3456);
        buffer.write_u32(0x789ABCDE);
        buffer.write_string("test");
        
        // Reset position for reading
        buffer.reset();
        
        // Read data
        assert_eq!(buffer.read_u8().unwrap(), 0x12);
        assert_eq!(buffer.read_u16().unwrap(), 0x3456);
        assert_eq!(buffer.read_u32().unwrap(), 0x789ABCDE);
        assert_eq!(buffer.read_string().unwrap(), "test");
        
        assert!(buffer.is_at_end());
    }
}