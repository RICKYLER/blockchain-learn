//! Formatting utilities for the LedgerDB blockchain.
//!
//! This module provides functions for formatting various data types
//! into human-readable strings and parsing them back.

use crate::crypto::Hash256;
use crate::error::LedgerError;
use std::fmt;

/// Format a hash as a hex string with optional prefix
pub fn format_hash(hash: &Hash256, prefix: bool) -> String {
    let hex = hex::encode(hash.as_bytes());
    if prefix {
        format!("0x{}", hex)
    } else {
        hex
    }
}

/// Format a hash as a short hex string (first 8 characters)
pub fn format_hash_short(hash: &Hash256) -> String {
    let hex = hex::encode(hash.as_bytes());
    if hex.len() >= 8 {
        format!("{}...", &hex[..8])
    } else {
        hex
    }
}

/// Format bytes as human-readable size (B, KB, MB, GB)
pub fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB", "PB"];
    const THRESHOLD: f64 = 1024.0;
    
    if bytes == 0 {
        return "0 B".to_string();
    }
    
    let mut size = bytes as f64;
    let mut unit_index = 0;
    
    while size >= THRESHOLD && unit_index < UNITS.len() - 1 {
        size /= THRESHOLD;
        unit_index += 1;
    }
    
    if unit_index == 0 {
        format!("{} {}", bytes, UNITS[unit_index])
    } else {
        format!("{:.2} {}", size, UNITS[unit_index])
    }
}

/// Parse human-readable size back to bytes
pub fn parse_bytes(size_str: &str) -> Result<u64, LedgerError> {
    let size_str = size_str.trim().to_uppercase();
    
    let (number_part, unit_part) = if let Some(pos) = size_str.find(char::is_alphabetic) {
        (&size_str[..pos], &size_str[pos..])
    } else {
        (size_str.as_str(), "B")
    };
    
    let number: f64 = number_part.trim().parse()
        .map_err(|e| LedgerError::Internal(format!("Invalid number: {}", e)))?;
    
    let multiplier = match unit_part.trim() {
        "B" => 1,
        "KB" => 1024,
        "MB" => 1024 * 1024,
        "GB" => 1024 * 1024 * 1024,
        "TB" => 1024_u64.pow(4),
        "PB" => 1024_u64.pow(5),
        _ => return Err(LedgerError::Internal(format!("Unknown unit: {}", unit_part))),
    };
    
    Ok((number * multiplier as f64) as u64)
}

/// Format hash rate (H/s, KH/s, MH/s, GH/s, TH/s)
pub fn format_hash_rate(hash_rate: f64) -> String {
    const UNITS: &[&str] = &["H/s", "KH/s", "MH/s", "GH/s", "TH/s", "PH/s"];
    const THRESHOLD: f64 = 1000.0;
    
    if hash_rate == 0.0 {
        return "0 H/s".to_string();
    }
    
    let mut rate = hash_rate;
    let mut unit_index = 0;
    
    while rate >= THRESHOLD && unit_index < UNITS.len() - 1 {
        rate /= THRESHOLD;
        unit_index += 1;
    }
    
    if unit_index == 0 {
        format!("{:.0} {}", rate, UNITS[unit_index])
    } else {
        format!("{:.2} {}", rate, UNITS[unit_index])
    }
}

/// Format currency amount (satoshis to BTC)
pub fn format_currency(satoshis: u64, symbol: &str) -> String {
    const SATOSHIS_PER_BTC: u64 = 100_000_000;
    
    if satoshis == 0 {
        return format!("0 {}", symbol);
    }
    
    let btc = satoshis as f64 / SATOSHIS_PER_BTC as f64;
    
    if btc >= 1.0 {
        format!("{:.8} {}", btc, symbol)
    } else if btc >= 0.001 {
        format!("{:.6} {}", btc, symbol)
    } else {
        format!("{} sat", satoshis)
    }
}

/// Parse currency amount back to satoshis
pub fn parse_currency(amount_str: &str) -> Result<u64, LedgerError> {
    let amount_str = amount_str.trim().to_lowercase();
    
    if amount_str.ends_with(" sat") {
        let sat_str = amount_str.strip_suffix(" sat").unwrap();
        sat_str.parse::<u64>()
            .map_err(|e| LedgerError::Internal(format!("Invalid satoshi amount: {}", e)))
    } else {
        // Assume BTC
        let btc_str = amount_str.split_whitespace().next().unwrap_or(&amount_str);
        let btc: f64 = btc_str.parse()
            .map_err(|e| LedgerError::Internal(format!("Invalid BTC amount: {}", e)))?;
        
        Ok((btc * 100_000_000.0) as u64)
    }
}

/// Format percentage with specified decimal places
pub fn format_percentage(value: f64, decimal_places: usize) -> String {
    format!("{:.prec$}%", value, prec = decimal_places)
}

/// Format number with thousands separators
pub fn format_number(number: u64) -> String {
    let number_str = number.to_string();
    let mut result = String::new();
    
    for (i, ch) in number_str.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(ch);
    }
    
    result.chars().rev().collect()
}

/// Format floating point number with thousands separators
pub fn format_float(number: f64, decimal_places: usize) -> String {
    let formatted = format!("{:.prec$}", number, prec = decimal_places);
    
    if let Some(dot_pos) = formatted.find('.') {
        let integer_part = &formatted[..dot_pos];
        let decimal_part = &formatted[dot_pos..];
        
        if let Ok(integer) = integer_part.parse::<u64>() {
            format!("{}{}", format_number(integer), decimal_part)
        } else {
            formatted
        }
    } else if let Ok(integer) = formatted.parse::<u64>() {
        format_number(integer)
    } else {
        formatted
    }
}

/// Format address with optional prefix and suffix
pub fn format_address(address: &str, prefix_len: usize, suffix_len: usize) -> String {
    if address.len() <= prefix_len + suffix_len {
        address.to_string()
    } else {
        format!(
            "{}...{}",
            &address[..prefix_len],
            &address[address.len() - suffix_len..]
        )
    }
}

/// Format transaction ID with optional prefix
pub fn format_txid(txid: &str, short: bool) -> String {
    if short {
        format_address(txid, 8, 8)
    } else {
        txid.to_string()
    }
}

/// Format block height with ordinal suffix
pub fn format_block_height(height: u64) -> String {
    let suffix = match height % 100 {
        11..=13 => "th",
        _ => match height % 10 {
            1 => "st",
            2 => "nd",
            3 => "rd",
            _ => "th",
        },
    };
    
    format!("{}{}", format_number(height), suffix)
}

/// Format difficulty as a human-readable string
pub fn format_difficulty(difficulty: u32) -> String {
    if difficulty < 1000 {
        difficulty.to_string()
    } else if difficulty < 1_000_000 {
        format!("{:.1}K", difficulty as f64 / 1000.0)
    } else if difficulty < 1_000_000_000 {
        format!("{:.1}M", difficulty as f64 / 1_000_000.0)
    } else {
        format!("{:.1}B", difficulty as f64 / 1_000_000_000.0)
    }
}

/// Format network statistics
pub struct NetworkStatsFormatter {
    pub block_height: u64,
    pub hash_rate: f64,
    pub difficulty: u32,
    pub total_supply: u64,
    pub market_cap: Option<f64>,
}

impl fmt::Display for NetworkStatsFormatter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Network Statistics:")?;
        writeln!(f, "  Block Height: {}", format_block_height(self.block_height))?;
        writeln!(f, "  Hash Rate: {}", format_hash_rate(self.hash_rate))?;
        writeln!(f, "  Difficulty: {}", format_difficulty(self.difficulty))?;
        writeln!(f, "  Total Supply: {}", format_currency(self.total_supply, "BTC"))?;
        
        if let Some(market_cap) = self.market_cap {
            writeln!(f, "  Market Cap: ${}", format_float(market_cap, 2))?;
        }
        
        Ok(())
    }
}

/// Format table with aligned columns
pub struct TableFormatter {
    headers: Vec<String>,
    rows: Vec<Vec<String>>,
    column_widths: Vec<usize>,
}

impl TableFormatter {
    /// Create a new table formatter
    pub fn new(headers: Vec<String>) -> Self {
        let column_widths = headers.iter().map(|h| h.len()).collect();
        Self {
            headers,
            rows: Vec::new(),
            column_widths,
        }
    }
    
    /// Add a row to the table
    pub fn add_row(&mut self, row: Vec<String>) {
        // Update column widths
        for (i, cell) in row.iter().enumerate() {
            if i < self.column_widths.len() {
                self.column_widths[i] = self.column_widths[i].max(cell.len());
            }
        }
        self.rows.push(row);
    }
    
    /// Format the table as a string
    pub fn format(&self) -> String {
        let mut result = String::new();
        
        // Header
        for (i, header) in self.headers.iter().enumerate() {
            if i > 0 {
                result.push_str(" | ");
            }
            result.push_str(&format!("{:<width$}", header, width = self.column_widths[i]));
        }
        result.push('\n');
        
        // Separator
        for (i, &width) in self.column_widths.iter().enumerate() {
            if i > 0 {
                result.push_str("-+-");
            }
            result.push_str(&"-".repeat(width));
        }
        result.push('\n');
        
        // Rows
        for row in &self.rows {
            for (i, cell) in row.iter().enumerate() {
                if i > 0 {
                    result.push_str(" | ");
                }
                let width = self.column_widths.get(i).copied().unwrap_or(0);
                result.push_str(&format!("{:<width$}", cell, width = width));
            }
            result.push('\n');
        }
        
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::Hash256;
    
    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(0), "0 B");
        assert_eq!(format_bytes(512), "512 B");
        assert_eq!(format_bytes(1024), "1.00 KB");
        assert_eq!(format_bytes(1536), "1.50 KB");
        assert_eq!(format_bytes(1048576), "1.00 MB");
        assert_eq!(format_bytes(1073741824), "1.00 GB");
    }
    
    #[test]
    fn test_parse_bytes() {
        assert_eq!(parse_bytes("512 B").unwrap(), 512);
        assert_eq!(parse_bytes("1 KB").unwrap(), 1024);
        assert_eq!(parse_bytes("1.5 KB").unwrap(), 1536);
        assert_eq!(parse_bytes("1 MB").unwrap(), 1048576);
        assert_eq!(parse_bytes("1 GB").unwrap(), 1073741824);
    }
    
    #[test]
    fn test_format_hash_rate() {
        assert_eq!(format_hash_rate(0.0), "0 H/s");
        assert_eq!(format_hash_rate(500.0), "500 H/s");
        assert_eq!(format_hash_rate(1500.0), "1.50 KH/s");
        assert_eq!(format_hash_rate(1_500_000.0), "1.50 MH/s");
    }
    
    #[test]
    fn test_format_currency() {
        assert_eq!(format_currency(0, "BTC"), "0 BTC");
        assert_eq!(format_currency(100_000_000, "BTC"), "1.00000000 BTC");
        assert_eq!(format_currency(50_000_000, "BTC"), "0.500000 BTC");
        assert_eq!(format_currency(1000, "BTC"), "1000 sat");
    }
    
    #[test]
    fn test_parse_currency() {
        assert_eq!(parse_currency("1000 sat").unwrap(), 1000);
        assert_eq!(parse_currency("1.0").unwrap(), 100_000_000);
        assert_eq!(parse_currency("0.5").unwrap(), 50_000_000);
    }
    
    #[test]
    fn test_format_percentage() {
        assert_eq!(format_percentage(12.3456, 2), "12.35%");
        assert_eq!(format_percentage(100.0, 0), "100%");
    }
    
    #[test]
    fn test_format_number() {
        assert_eq!(format_number(1000), "1,000");
        assert_eq!(format_number(1234567), "1,234,567");
        assert_eq!(format_number(123), "123");
    }
    
    #[test]
    fn test_format_address() {
        let address = "1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa";
        assert_eq!(format_address(address, 6, 6), "1A1zP1...DivfNa");
        assert_eq!(format_address("short", 10, 10), "short");
    }
    
    #[test]
    fn test_format_block_height() {
        assert_eq!(format_block_height(1), "1st");
        assert_eq!(format_block_height(2), "2nd");
        assert_eq!(format_block_height(3), "3rd");
        assert_eq!(format_block_height(4), "4th");
        assert_eq!(format_block_height(11), "11th");
        assert_eq!(format_block_height(21), "21st");
        assert_eq!(format_block_height(1000), "1,000th");
    }
    
    #[test]
    fn test_format_difficulty() {
        assert_eq!(format_difficulty(500), "500");
        assert_eq!(format_difficulty(1500), "1.5K");
        assert_eq!(format_difficulty(1_500_000), "1.5M");
        assert_eq!(format_difficulty(1_500_000_000), "1.5B");
    }
    
    #[test]
    fn test_table_formatter() {
        let mut table = TableFormatter::new(vec!["Name".to_string(), "Age".to_string(), "City".to_string()]);
        table.add_row(vec!["Alice".to_string(), "30".to_string(), "New York".to_string()]);
        table.add_row(vec!["Bob".to_string(), "25".to_string(), "San Francisco".to_string()]);
        
        let formatted = table.format();
        assert!(formatted.contains("Alice"));
        assert!(formatted.contains("Bob"));
        assert!(formatted.contains("New York"));
        assert!(formatted.contains("San Francisco"));
    }
}