//! HTTP API handlers for blockchain operations.
//!
//! This module contains all the handler functions for the REST API endpoints,
//! including block operations, transaction management, mining, and administrative functions.

use super::{
    types::*, ApiError, AppState, PaginatedResponse, PaginationParams,
};
use crate::core::{Block, Transaction};
use crate::crypto::{Address, Hash256};
use crate::error::Result;
use axum::{
    extract::{Path, Query, State, WebSocketUpgrade},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use chrono::Utc;
use serde_json::json;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// Health check endpoint
pub async fn health_check(State(state): State<AppState>) -> Result<Json<HealthResponse>, ApiError> {
    let uptime = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let response = HealthResponse {
        status: "healthy".to_string(),
        timestamp: Utc::now(),
        version: state.config.version.clone(),
        uptime,
    };

    Ok(Json(response))
}

/// Get API version
pub async fn get_api_version(State(state): State<AppState>) -> Json<serde_json::Value> {
    Json(json!({
        "version": state.config.version,
        "api_version": "v1",
        "build_time": option_env!("BUILD_TIME").unwrap_or("unknown"),
        "git_commit": option_env!("GIT_COMMIT").unwrap_or("unknown")
    }))
}

/// Get blockchain information
pub async fn get_blockchain_info(
    State(state): State<AppState>,
) -> Result<Json<BlockchainInfoResponse>, ApiError> {
    let blockchain = state.blockchain.read().await;
    let stats = blockchain.get_stats();
    
    let response = BlockchainInfoResponse {
        height: stats.height,
        latest_block_hash: blockchain.get_latest_block_hash(),
        total_transactions: stats.total_transactions,
        total_supply: stats.total_supply,
        difficulty: blockchain.get_current_difficulty(),
        network_hash_rate: calculate_network_hash_rate(&blockchain).await,
    };

    Ok(Json(response))
}

/// Get blockchain statistics
pub async fn get_blockchain_stats(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let blockchain = state.blockchain.read().await;
    let stats = blockchain.get_stats();
    let storage_stats = state.storage.get_stats().await.map_err(ApiError::from)?;

    let response = json!({
        "blockchain": {
            "height": stats.height,
            "total_blocks": stats.total_blocks,
            "total_transactions": stats.total_transactions,
            "total_supply": stats.total_supply,
            "average_block_time": stats.average_block_time,
            "difficulty": blockchain.get_current_difficulty(),
        },
        "storage": {
            "total_size": storage_stats.total_size,
            "block_count": storage_stats.block_count,
            "transaction_count": storage_stats.transaction_count,
            "utxo_count": storage_stats.utxo_count,
        },
        "network": {
            "hash_rate": calculate_network_hash_rate(&blockchain).await,
            "connected_peers": 0, // TODO: Implement peer management
        }
    });

    Ok(Json(response))
}

/// Get blocks with pagination
pub async fn get_blocks(
    State(state): State<AppState>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<PaginatedResponse<Block>>, ApiError> {
    let blockchain = state.blockchain.read().await;
    let page = params.page.unwrap_or(0);
    let limit = params.limit.unwrap_or(20).min(100); // Cap at 100
    
    let total_blocks = blockchain.get_height();
    let start_height = if page * limit > total_blocks {
        return Ok(Json(super::paginate(vec![], page, limit, total_blocks)));
    } else {
        total_blocks.saturating_sub((page + 1) * limit)
    };
    
    let mut blocks = Vec::new();
    for height in start_height..start_height + limit.min(total_blocks - start_height) {
        if let Some(block) = blockchain.get_block_by_height(height) {
            blocks.push(block);
        }
    }
    
    // Reverse for descending order (newest first)
    if params.order.as_deref() != Some("asc") {
        blocks.reverse();
    }
    
    Ok(Json(super::paginate(blocks, page, limit, total_blocks)))
}

/// Get latest block
pub async fn get_latest_block(
    State(state): State<AppState>,
) -> Result<Json<Block>, ApiError> {
    let blockchain = state.blockchain.read().await;
    
    blockchain
        .get_latest_block()
        .map(Json)
        .ok_or_else(|| ApiError::new("NOT_FOUND", "No blocks found"))
}

/// Get block by height
pub async fn get_block_by_height(
    State(state): State<AppState>,
    Path(height): Path<u64>,
) -> Result<Json<Block>, ApiError> {
    let blockchain = state.blockchain.read().await;
    
    blockchain
        .get_block_by_height(height)
        .map(Json)
        .ok_or_else(|| ApiError::new("NOT_FOUND", format!("Block at height {} not found", height)))
}

/// Get block by hash
pub async fn get_block_by_hash(
    State(state): State<AppState>,
    Path(hash): Path<String>,
) -> Result<Json<Block>, ApiError> {
    let hash = Hash256::from_hex(&hash)
        .map_err(|_| ApiError::new("INVALID_HASH", "Invalid block hash format"))?;
    
    let blockchain = state.blockchain.read().await;
    
    blockchain
        .get_block_by_hash(&hash)
        .map(Json)
        .ok_or_else(|| ApiError::new("NOT_FOUND", "Block not found"))
}

/// Get transactions in a block
pub async fn get_block_transactions(
    State(state): State<AppState>,
    Path(block_id): Path<String>,
) -> Result<Json<Vec<Transaction>>, ApiError> {
    let blockchain = state.blockchain.read().await;
    
    // Try to parse as height first, then as hash
    let block = if let Ok(height) = block_id.parse::<u64>() {
        blockchain.get_block_by_height(height)
    } else if let Ok(hash) = Hash256::from_hex(&block_id) {
        blockchain.get_block_by_hash(&hash)
    } else {
        return Err(ApiError::new("INVALID_BLOCK_ID", "Invalid block ID format"));
    };
    
    let block = block.ok_or_else(|| ApiError::new("NOT_FOUND", "Block not found"))?;
    
    Ok(Json(block.transactions))
}

/// Create a new transaction
pub async fn create_transaction(
    State(state): State<AppState>,
    Json(request): Json<CreateTransactionRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    // TODO: Implement transaction creation from request
    // This would involve:
    // 1. Validating inputs and outputs
    // 2. Creating the transaction
    // 3. Adding to transaction pool
    // 4. Broadcasting to network
    
    Err(ApiError::new("NOT_IMPLEMENTED", "Transaction creation not yet implemented"))
}

/// Get pending transactions
pub async fn get_pending_transactions(
    State(state): State<AppState>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<PaginatedResponse<Transaction>>, ApiError> {
    let blockchain = state.blockchain.read().await;
    let pending_txs = blockchain.get_pending_transactions();
    
    let page = params.page.unwrap_or(0);
    let limit = params.limit.unwrap_or(20).min(100);
    let total = pending_txs.len() as u64;
    
    let start = (page * limit) as usize;
    let end = ((page + 1) * limit).min(total) as usize;
    
    let transactions = if start < pending_txs.len() {
        pending_txs[start..end].to_vec()
    } else {
        vec![]
    };
    
    Ok(Json(super::paginate(transactions, page, limit, total)))
}

/// Get transaction by hash
pub async fn get_transaction_by_hash(
    State(state): State<AppState>,
    Path(hash): Path<String>,
) -> Result<Json<Transaction>, ApiError> {
    let hash = Hash256::from_hex(&hash)
        .map_err(|_| ApiError::new("INVALID_HASH", "Invalid transaction hash format"))?;
    
    let blockchain = state.blockchain.read().await;
    
    blockchain
        .get_transaction_by_hash(&hash)
        .map(Json)
        .ok_or_else(|| ApiError::new("NOT_FOUND", "Transaction not found"))
}

/// Get Merkle proof for a transaction
pub async fn get_transaction_merkle_proof(
    State(state): State<AppState>,
    Path(hash): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let hash = Hash256::from_hex(&hash)
        .map_err(|_| ApiError::new("INVALID_HASH", "Invalid transaction hash format"))?;
    
    let blockchain = state.blockchain.read().await;
    
    // Find the block containing this transaction
    let (block, tx_index) = blockchain
        .find_transaction_in_block(&hash)
        .ok_or_else(|| ApiError::new("NOT_FOUND", "Transaction not found in any block"))?;
    
    // Generate Merkle proof
    let proof = block.generate_merkle_proof(tx_index)
        .map_err(|e| ApiError::new("PROOF_GENERATION_FAILED", format!("Failed to generate proof: {}", e)))?;
    
    Ok(Json(json!({
        "transaction_hash": hash,
        "block_hash": block.hash(),
        "block_height": block.header.height,
        "transaction_index": tx_index,
        "merkle_proof": proof,
        "merkle_root": block.header.merkle_root
    })))
}

/// Validate a transaction
pub async fn validate_transaction(
    State(state): State<AppState>,
    Json(transaction): Json<Transaction>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let blockchain = state.blockchain.read().await;
    
    match blockchain.validate_transaction(&transaction) {
        Ok(_) => Ok(Json(json!({
            "valid": true,
            "message": "Transaction is valid"
        }))),
        Err(e) => Ok(Json(json!({
            "valid": false,
            "error": e.to_string()
        })))
    }
}

/// Start mining
pub async fn start_mining(
    State(state): State<AppState>,
    Json(request): Json<StartMiningRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    // TODO: Implement mining start
    Err(ApiError::new("NOT_IMPLEMENTED", "Mining start not yet implemented"))
}

/// Stop mining
pub async fn stop_mining(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, ApiError> {
    // TODO: Implement mining stop
    Err(ApiError::new("NOT_IMPLEMENTED", "Mining stop not yet implemented"))
}

/// Get mining status
pub async fn get_mining_status(
    State(state): State<AppState>,
) -> Result<Json<MiningStatusResponse>, ApiError> {
    let miner = state.miner.read().await;
    let blockchain = state.blockchain.read().await;
    
    let response = MiningStatusResponse {
        is_mining: miner.is_some(),
        current_block_height: Some(blockchain.get_height()),
        difficulty: Some(blockchain.get_current_difficulty()),
        hash_rate: None, // TODO: Calculate actual hash rate
        estimated_time: None, // TODO: Calculate estimated time
    };
    
    Ok(Json(response))
}

/// Get mining difficulty
pub async fn get_mining_difficulty(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let blockchain = state.blockchain.read().await;
    let difficulty = blockchain.get_current_difficulty();
    
    Ok(Json(json!({
        "difficulty": difficulty,
        "target": format!("{:064x}", u64::MAX >> difficulty.min(63)),
        "next_adjustment": blockchain.blocks_until_difficulty_adjustment(),
    })))
}

/// Get address balance
pub async fn get_address_balance(
    State(state): State<AppState>,
    Path(address): Path<String>,
) -> Result<Json<AddressBalanceResponse>, ApiError> {
    let address = Address::from_string(&address)
        .map_err(|_| ApiError::new("INVALID_ADDRESS", "Invalid address format"))?;
    
    let blockchain = state.blockchain.read().await;
    let utxos = blockchain.get_utxos_for_address(&address);
    let balance = utxos.iter().map(|utxo| utxo.amount).sum();
    
    let response = AddressBalanceResponse {
        address,
        balance,
        utxo_count: utxos.len(),
        last_updated: Utc::now(),
    };
    
    Ok(Json(response))
}

/// Get UTXOs for an address
pub async fn get_address_utxos(
    State(state): State<AppState>,
    Path(address): Path<String>,
) -> Result<Json<Vec<UtxoResponse>>, ApiError> {
    let address = Address::from_string(&address)
        .map_err(|_| ApiError::new("INVALID_ADDRESS", "Invalid address format"))?;
    
    let blockchain = state.blockchain.read().await;
    let utxos = blockchain.get_utxos_for_address(&address);
    
    let utxo_responses: Vec<UtxoResponse> = utxos
        .into_iter()
        .map(|utxo| UtxoResponse {
            utxo_id: format!("{}:{}", utxo.tx_hash, utxo.output_index),
            amount: utxo.amount,
            recipient_address: utxo.recipient_address,
            block_height: utxo.block_height,
            tx_hash: utxo.tx_hash,
            output_index: utxo.output_index,
            is_spent: false, // UTXOs in our set are unspent by definition
        })
        .collect();
    
    Ok(Json(utxo_responses))
}

/// Get transactions for an address
pub async fn get_address_transactions(
    State(state): State<AppState>,
    Path(address): Path<String>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<PaginatedResponse<Transaction>>, ApiError> {
    let address = Address::from_string(&address)
        .map_err(|_| ApiError::new("INVALID_ADDRESS", "Invalid address format"))?;
    
    // TODO: Implement address transaction history
    // This would require indexing transactions by address
    
    Err(ApiError::new("NOT_IMPLEMENTED", "Address transaction history not yet implemented"))
}

/// Get all UTXOs
pub async fn get_all_utxos(
    State(state): State<AppState>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<PaginatedResponse<UtxoResponse>>, ApiError> {
    let blockchain = state.blockchain.read().await;
    let all_utxos = blockchain.get_all_utxos();
    
    let page = params.page.unwrap_or(0);
    let limit = params.limit.unwrap_or(20).min(100);
    let total = all_utxos.len() as u64;
    
    let start = (page * limit) as usize;
    let end = ((page + 1) * limit).min(total) as usize;
    
    let utxos = if start < all_utxos.len() {
        all_utxos[start..end]
            .iter()
            .map(|utxo| UtxoResponse {
                utxo_id: format!("{}:{}", utxo.tx_hash, utxo.output_index),
                amount: utxo.amount,
                recipient_address: utxo.recipient_address.clone(),
                block_height: utxo.block_height,
                tx_hash: utxo.tx_hash,
                output_index: utxo.output_index,
                is_spent: false,
            })
            .collect()
    } else {
        vec![]
    };
    
    Ok(Json(super::paginate(utxos, page, limit, total)))
}

/// Get UTXO by ID
pub async fn get_utxo_by_id(
    State(state): State<AppState>,
    Path(utxo_id): Path<String>,
) -> Result<Json<UtxoResponse>, ApiError> {
    // Parse UTXO ID (format: "tx_hash:output_index")
    let parts: Vec<&str> = utxo_id.split(':').collect();
    if parts.len() != 2 {
        return Err(ApiError::new("INVALID_UTXO_ID", "UTXO ID must be in format 'tx_hash:output_index'"));
    }
    
    let tx_hash = Hash256::from_hex(parts[0])
        .map_err(|_| ApiError::new("INVALID_HASH", "Invalid transaction hash in UTXO ID"))?;
    let output_index: u32 = parts[1]
        .parse()
        .map_err(|_| ApiError::new("INVALID_INDEX", "Invalid output index in UTXO ID"))?;
    
    let blockchain = state.blockchain.read().await;
    
    if let Some(utxo) = blockchain.get_utxo(&tx_hash, output_index) {
        let response = UtxoResponse {
            utxo_id,
            amount: utxo.amount,
            recipient_address: utxo.recipient_address,
            block_height: utxo.block_height,
            tx_hash: utxo.tx_hash,
            output_index: utxo.output_index,
            is_spent: false,
        };
        Ok(Json(response))
    } else {
        Err(ApiError::new("NOT_FOUND", "UTXO not found"))
    }
}

/// Get network peers (placeholder)
pub async fn get_network_peers(
    State(_state): State<AppState>,
) -> Result<Json<Vec<serde_json::Value>>, ApiError> {
    // TODO: Implement peer management
    Ok(Json(vec![]))
}

/// Get network status
pub async fn get_network_status(
    State(state): State<AppState>,
) -> Result<Json<NetworkStatusResponse>, ApiError> {
    let blockchain = state.blockchain.read().await;
    
    let response = NetworkStatusResponse {
        connected_peers: 0, // TODO: Implement peer counting
        network_height: blockchain.get_height(),
        sync_status: "synced".to_string(), // TODO: Implement sync status
        last_sync: Utc::now(),
    };
    
    Ok(Json(response))
}

/// Compact database (admin endpoint)
pub async fn compact_database(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, ApiError> {
    // TODO: Implement database compaction
    Err(ApiError::new("NOT_IMPLEMENTED", "Database compaction not yet implemented"))
}

/// Create backup (admin endpoint)
pub async fn create_backup(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, ApiError> {
    // TODO: Implement backup creation
    Err(ApiError::new("NOT_IMPLEMENTED", "Backup creation not yet implemented"))
}

/// Get system metrics (admin endpoint)
pub async fn get_system_metrics(
    State(state): State<AppState>,
) -> Result<Json<SystemMetricsResponse>, ApiError> {
    // TODO: Implement system metrics collection
    let response = SystemMetricsResponse {
        memory_usage: 0,
        cpu_usage: 0.0,
        disk_usage: 0,
        network_io: NetworkIoMetrics {
            bytes_sent: 0,
            bytes_received: 0,
            requests_per_second: 0.0,
        },
        database_size: 0,
        active_connections: 0,
    };
    
    Ok(Json(response))
}

/// Helper function to calculate network hash rate
async fn calculate_network_hash_rate(blockchain: &crate::core::Blockchain) -> f64 {
    // TODO: Implement actual hash rate calculation based on recent blocks
    // This would involve analyzing the time between blocks and difficulty
    0.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::storage::PersistentStorage;
    use tokio::sync::broadcast;

    async fn create_test_state() -> AppState {
        let config = Config::default();
        let storage = Arc::new(PersistentStorage::new(":memory:").unwrap());
        let blockchain = Arc::new(RwLock::new(
            Blockchain::load_or_create(storage.clone(), config.blockchain.clone())
                .await
                .unwrap()
        ));
        let (mining_progress_tx, _) = broadcast::channel(100);
        
        AppState {
            blockchain,
            storage,
            mining_progress_tx,
            miner: Arc::new(RwLock::new(None)),
            config: super::ApiConfig::default(),
        }
    }

    #[tokio::test]
    async fn test_health_check() {
        let state = create_test_state().await;
        let result = health_check(State(state)).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_blockchain_info() {
        let state = create_test_state().await;
        let result = get_blockchain_info(State(state)).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_latest_block_empty_chain() {
        let state = create_test_state().await;
        let result = get_latest_block(State(state)).await;
        // Should return error for empty blockchain
        assert!(result.is_err());
    }
}