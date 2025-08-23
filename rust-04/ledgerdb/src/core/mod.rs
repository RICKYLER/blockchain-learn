//! Core blockchain data structures and logic.
//!
//! This module contains the fundamental blockchain components including
//! blocks, transactions, and the main blockchain implementation.

pub mod block;
pub mod blockchain;
pub mod transaction;

// Re-export commonly used types
pub use block::*;
pub use blockchain::*;
pub use transaction::*;