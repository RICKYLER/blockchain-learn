//! WebSocket support for real-time blockchain updates.
//!
//! This module provides WebSocket endpoints for streaming real-time data
//! including mining progress, new blocks, transactions, and network status.

use crate::api::AppState;
use crate::core::{Block, Transaction};
use crate::crypto::pow::MiningProgress;
use crate::crypto::Hash256;
use axum::{
    extract::{ws::WebSocket, State, WebSocketUpgrade},
    response::Response,
};
use futures_util::{sink::SinkExt, stream::StreamExt};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use tokio::{
    sync::broadcast,
    time::{interval, timeout},
};
use tracing::{error, info, warn};
use uuid::Uuid;

/// WebSocket message types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum WsMessage {
    /// Mining progress update
    MiningProgress(MiningProgressData),
    /// New block notification
    NewBlock(NewBlockData),
    /// New transaction notification
    NewTransaction(NewTransactionData),
    /// Network status update
    NetworkStatus(NetworkStatusData),
    /// Mempool update
    MempoolUpdate(MempoolUpdateData),
    /// Difficulty adjustment
    DifficultyAdjustment(DifficultyAdjustmentData),
    /// Connection status
    ConnectionStatus(ConnectionStatusData),
    /// Error message
    Error(ErrorData),
    /// Ping/Pong for keepalive
    Ping(PingData),
    /// Pong response
    Pong(PongData),
    /// Subscription confirmation
    Subscribed(SubscriptionData),
    /// Unsubscription confirmation
    Unsubscribed(UnsubscriptionData),
}

/// Mining progress data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MiningProgressData {
    /// Current block height being mined
    pub block_height: u64,
    /// Current difficulty
    pub difficulty: u32,
    /// Current nonce
    pub nonce: u64,
    /// Hash rate (hashes per second)
    pub hash_rate: f64,
    /// Progress percentage (0-100)
    pub progress: f64,
    /// Estimated time remaining (seconds)
    pub estimated_time: Option<u64>,
    /// Number of attempts
    pub attempts: u64,
    /// Best hash found so far
    pub best_hash: Option<Hash256>,
}

/// New block data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewBlockData {
    /// Block hash
    pub hash: Hash256,
    /// Block height
    pub height: u64,
    /// Number of transactions
    pub transaction_count: usize,
    /// Block size in bytes
    pub size: usize,
    /// Block timestamp
    pub timestamp: u64,
    /// Miner address
    pub miner: Option<String>,
    /// Block reward
    pub reward: u64,
    /// Total fees
    pub total_fees: u64,
    /// Difficulty
    pub difficulty: u32,
}

/// New transaction data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewTransactionData {
    /// Transaction hash
    pub hash: Hash256,
    /// Transaction size
    pub size: usize,
    /// Fee amount
    pub fee: Option<u64>,
    /// Fee rate (satoshis per byte)
    pub fee_rate: Option<f64>,
    /// Input count
    pub input_count: usize,
    /// Output count
    pub output_count: usize,
    /// Total input amount
    pub total_input: u64,
    /// Total output amount
    pub total_output: u64,
}

/// Network status data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkStatusData {
    /// Current block height
    pub block_height: u64,
    /// Network hash rate
    pub hash_rate: f64,
    /// Connected peers
    pub connected_peers: u32,
    /// Sync status
    pub sync_status: String,
    /// Network difficulty
    pub difficulty: u32,
    /// Average block time
    pub average_block_time: f64,
}

/// Mempool update data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MempoolUpdateData {
    /// Number of transactions
    pub transaction_count: u64,
    /// Total mempool size (bytes)
    pub total_size: u64,
    /// Total fees
    pub total_fees: u64,
    /// Average fee rate
    pub average_fee_rate: f64,
    /// Recent transactions added
    pub recent_transactions: Vec<Hash256>,
}

/// Difficulty adjustment data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DifficultyAdjustmentData {
    /// Old difficulty
    pub old_difficulty: u32,
    /// New difficulty
    pub new_difficulty: u32,
    /// Change percentage
    pub change_percentage: f64,
    /// Block height of adjustment
    pub block_height: u64,
    /// Next adjustment in blocks
    pub next_adjustment: u64,
}

/// Connection status data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionStatusData {
    /// Connection ID
    pub connection_id: String,
    /// Connection status
    pub status: String,
    /// Connected at
    pub connected_at: u64,
    /// Subscriptions
    pub subscriptions: Vec<String>,
}

/// Error data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorData {
    /// Error code
    pub code: String,
    /// Error message
    pub message: String,
    /// Additional details
    pub details: Option<serde_json::Value>,
}

/// Ping data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PingData {
    /// Timestamp
    pub timestamp: u64,
    /// Optional message
    pub message: Option<String>,
}

/// Pong data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PongData {
    /// Original timestamp from ping
    pub timestamp: u64,
    /// Response timestamp
    pub response_timestamp: u64,
    /// Optional message
    pub message: Option<String>,
}

/// Subscription data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionData {
    /// Subscription topic
    pub topic: String,
    /// Subscription ID
    pub subscription_id: String,
    /// Success status
    pub success: bool,
    /// Optional message
    pub message: Option<String>,
}

/// Unsubscription data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnsubscriptionData {
    /// Subscription topic
    pub topic: String,
    /// Subscription ID
    pub subscription_id: String,
    /// Success status
    pub success: bool,
}

/// WebSocket client subscription request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionRequest {
    /// Action (subscribe/unsubscribe)
    pub action: String,
    /// Topic to subscribe to
    pub topic: String,
    /// Optional parameters
    pub params: Option<HashMap<String, serde_json::Value>>,
}

/// Available subscription topics
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SubscriptionTopic {
    /// Mining progress updates
    MiningProgress,
    /// New block notifications
    NewBlocks,
    /// New transaction notifications
    NewTransactions,
    /// Network status updates
    NetworkStatus,
    /// Mempool updates
    MempoolUpdates,
    /// Difficulty adjustments
    DifficultyAdjustments,
    /// All updates
    All,
}

impl SubscriptionTopic {
    /// Parse topic from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "mining_progress" => Some(Self::MiningProgress),
            "new_blocks" => Some(Self::NewBlocks),
            "new_transactions" => Some(Self::NewTransactions),
            "network_status" => Some(Self::NetworkStatus),
            "mempool_updates" => Some(Self::MempoolUpdates),
            "difficulty_adjustments" => Some(Self::DifficultyAdjustments),
            "all" => Some(Self::All),
            _ => None,
        }
    }
    
    /// Convert to string
    pub fn to_str(&self) -> &'static str {
        match self {
            Self::MiningProgress => "mining_progress",
            Self::NewBlocks => "new_blocks",
            Self::NewTransactions => "new_transactions",
            Self::NetworkStatus => "network_status",
            Self::MempoolUpdates => "mempool_updates",
            Self::DifficultyAdjustments => "difficulty_adjustments",
            Self::All => "all",
        }
    }
}

/// WebSocket connection manager
#[derive(Debug)]
pub struct WebSocketManager {
    /// Active connections
    connections: Arc<Mutex<HashMap<String, WebSocketConnection>>>,
    /// Broadcast channels for different topics
    channels: HashMap<SubscriptionTopic, broadcast::Sender<WsMessage>>,
}

/// WebSocket connection information
#[derive(Debug)]
pub struct WebSocketConnection {
    /// Connection ID
    pub id: String,
    /// Connection start time
    pub connected_at: Instant,
    /// Active subscriptions
    pub subscriptions: HashMap<SubscriptionTopic, String>,
    /// Last ping time
    pub last_ping: Option<Instant>,
    /// Message sender
    pub sender: tokio::sync::mpsc::UnboundedSender<WsMessage>,
}

impl WebSocketManager {
    /// Create a new WebSocket manager
    pub fn new() -> Self {
        let mut channels = HashMap::new();
        
        // Create broadcast channels for each topic
        for topic in [
            SubscriptionTopic::MiningProgress,
            SubscriptionTopic::NewBlocks,
            SubscriptionTopic::NewTransactions,
            SubscriptionTopic::NetworkStatus,
            SubscriptionTopic::MempoolUpdates,
            SubscriptionTopic::DifficultyAdjustments,
        ] {
            let (tx, _) = broadcast::channel(1000);
            channels.insert(topic, tx);
        }
        
        Self {
            connections: Arc::new(Mutex::new(HashMap::new())),
            channels,
        }
    }
    
    /// Add a new connection
    pub fn add_connection(&self, connection: WebSocketConnection) {
        let mut connections = self.connections.lock().unwrap();
        connections.insert(connection.id.clone(), connection);
    }
    
    /// Remove a connection
    pub fn remove_connection(&self, connection_id: &str) {
        let mut connections = self.connections.lock().unwrap();
        connections.remove(connection_id);
    }
    
    /// Broadcast message to topic subscribers
    pub fn broadcast_to_topic(&self, topic: SubscriptionTopic, message: WsMessage) {
        if let Some(sender) = self.channels.get(&topic) {
            if let Err(e) = sender.send(message) {
                warn!("Failed to broadcast message to topic {:?}: {}", topic, e);
            }
        }
    }
    
    /// Get connection count
    pub fn connection_count(&self) -> usize {
        self.connections.lock().unwrap().len()
    }
    
    /// Get subscriber for topic
    pub fn subscribe_to_topic(&self, topic: SubscriptionTopic) -> Option<broadcast::Receiver<WsMessage>> {
        self.channels.get(&topic).map(|sender| sender.subscribe())
    }
}

/// Mining progress WebSocket endpoint
pub async fn mining_progress_websocket(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> Response {
    ws.on_upgrade(|socket| handle_mining_progress_websocket(socket, state))
}

/// Handle mining progress WebSocket connection
async fn handle_mining_progress_websocket(socket: WebSocket, state: AppState) {
    let connection_id = Uuid::new_v4().to_string();
    info!("New mining progress WebSocket connection: {}", connection_id);
    
    let (mut sender, mut receiver) = socket.split();
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<WsMessage>();
    
    // Subscribe to mining progress updates
    let mut mining_progress_rx = state.mining_progress_tx.subscribe();
    
    // Send connection status
    let connection_status = WsMessage::ConnectionStatus(ConnectionStatusData {
        connection_id: connection_id.clone(),
        status: "connected".to_string(),
        connected_at: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
        subscriptions: vec!["mining_progress".to_string()],
    });
    
    if tx.send(connection_status).is_err() {
        error!("Failed to send connection status");
        return;
    }
    
    // Spawn task to handle outgoing messages
    let outgoing_task = tokio::spawn(async move {
        while let Some(message) = rx.recv().await {
            let json = match serde_json::to_string(&message) {
                Ok(json) => json,
                Err(e) => {
                    error!("Failed to serialize WebSocket message: {}", e);
                    continue;
                }
            };
            
            if sender.send(axum::extract::ws::Message::Text(json)).await.is_err() {
                break;
            }
        }
    });
    
    // Spawn task to handle mining progress updates
    let mining_progress_task = {
        let tx = tx.clone();
        tokio::spawn(async move {
            while let Ok(progress) = mining_progress_rx.recv().await {
                let message = WsMessage::MiningProgress(MiningProgressData {
                    block_height: progress.block_height,
                    difficulty: progress.difficulty,
                    nonce: progress.nonce,
                    hash_rate: progress.hash_rate,
                    progress: progress.progress,
                    estimated_time: progress.estimated_time,
                    attempts: progress.attempts,
                    best_hash: progress.best_hash,
                });
                
                if tx.send(message).is_err() {
                    break;
                }
            }
        })
    };
    
    // Spawn task to handle incoming messages
    let incoming_task = {
        let tx = tx.clone();
        tokio::spawn(async move {
            while let Some(msg) = receiver.next().await {
                match msg {
                    Ok(axum::extract::ws::Message::Text(text)) => {
                        // Handle client messages (ping, subscription requests, etc.)
                        if let Ok(request) = serde_json::from_str::<SubscriptionRequest>(&text) {
                            handle_subscription_request(request, &tx).await;
                        }
                    }
                    Ok(axum::extract::ws::Message::Close(_)) => {
                        info!("WebSocket connection closed: {}", connection_id);
                        break;
                    }
                    Err(e) => {
                        error!("WebSocket error: {}", e);
                        break;
                    }
                    _ => {}
                }
            }
        })
    };
    
    // Spawn keepalive task
    let keepalive_task = {
        let tx = tx.clone();
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(30));
            loop {
                interval.tick().await;
                
                let ping = WsMessage::Ping(PingData {
                    timestamp: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs(),
                    message: None,
                });
                
                if tx.send(ping).is_err() {
                    break;
                }
            }
        })
    };
    
    // Wait for any task to complete
    tokio::select! {
        _ = outgoing_task => {},
        _ = mining_progress_task => {},
        _ = incoming_task => {},
        _ = keepalive_task => {},
    }
    
    info!("Mining progress WebSocket connection closed: {}", connection_id);
}

/// Handle subscription request
async fn handle_subscription_request(
    request: SubscriptionRequest,
    tx: &tokio::sync::mpsc::UnboundedSender<WsMessage>,
) {
    let subscription_id = Uuid::new_v4().to_string();
    
    let response = match request.action.as_str() {
        "subscribe" => {
            if let Some(_topic) = SubscriptionTopic::from_str(&request.topic) {
                WsMessage::Subscribed(SubscriptionData {
                    topic: request.topic,
                    subscription_id,
                    success: true,
                    message: Some("Successfully subscribed".to_string()),
                })
            } else {
                WsMessage::Error(ErrorData {
                    code: "INVALID_TOPIC".to_string(),
                    message: format!("Invalid subscription topic: {}", request.topic),
                    details: None,
                })
            }
        }
        "unsubscribe" => {
            WsMessage::Unsubscribed(UnsubscriptionData {
                topic: request.topic,
                subscription_id,
                success: true,
            })
        }
        _ => {
            WsMessage::Error(ErrorData {
                code: "INVALID_ACTION".to_string(),
                message: format!("Invalid action: {}", request.action),
                details: None,
            })
        }
    };
    
    if tx.send(response).is_err() {
        error!("Failed to send subscription response");
    }
}

/// Convert mining progress to WebSocket message
impl From<MiningProgress> for MiningProgressData {
    fn from(progress: MiningProgress) -> Self {
        Self {
            block_height: progress.block_height,
            difficulty: progress.difficulty,
            nonce: progress.nonce,
            hash_rate: progress.hash_rate,
            progress: progress.progress,
            estimated_time: progress.estimated_time,
            attempts: progress.attempts,
            best_hash: progress.best_hash,
        }
    }
}

/// Convert block to WebSocket message
impl From<&Block> for NewBlockData {
    fn from(block: &Block) -> Self {
        let size = bincode::serialize(block).map(|b| b.len()).unwrap_or(0);
        let total_fees = block.transactions.iter()
            .map(|tx| tx.fee.unwrap_or_default())
            .sum();
        
        Self {
            hash: block.hash(),
            height: block.header.height,
            transaction_count: block.transactions.len(),
            size,
            timestamp: block.header.timestamp,
            miner: None, // TODO: Extract miner from coinbase transaction
            reward: 50_000_000, // TODO: Calculate actual block reward
            total_fees,
            difficulty: 0, // TODO: Get from block header
        }
    }
}

/// Convert transaction to WebSocket message
impl From<&Transaction> for NewTransactionData {
    fn from(transaction: &Transaction) -> Self {
        let size = bincode::serialize(transaction).map(|b| b.len()).unwrap_or(0);
        let total_input = transaction.inputs.iter()
            .map(|input| input.amount)
            .sum();
        let total_output = transaction.outputs.iter()
            .map(|output| output.amount)
            .sum();
        
        let fee_rate = transaction.fee.map(|fee| {
            if size > 0 {
                fee as f64 / size as f64
            } else {
                0.0
            }
        });
        
        Self {
            hash: transaction.hash(),
            size,
            fee: transaction.fee,
            fee_rate,
            input_count: transaction.inputs.len(),
            output_count: transaction.outputs.len(),
            total_input,
            total_output,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_subscription_topic_parsing() {
        assert_eq!(SubscriptionTopic::from_str("mining_progress"), Some(SubscriptionTopic::MiningProgress));
        assert_eq!(SubscriptionTopic::from_str("new_blocks"), Some(SubscriptionTopic::NewBlocks));
        assert_eq!(SubscriptionTopic::from_str("invalid"), None);
    }
    
    #[test]
    fn test_subscription_topic_to_string() {
        assert_eq!(SubscriptionTopic::MiningProgress.to_str(), "mining_progress");
        assert_eq!(SubscriptionTopic::NewBlocks.to_str(), "new_blocks");
    }
    
    #[test]
    fn test_websocket_message_serialization() {
        let message = WsMessage::Ping(PingData {
            timestamp: 1234567890,
            message: Some("test".to_string()),
        });
        
        let json = serde_json::to_string(&message).unwrap();
        let deserialized: WsMessage = serde_json::from_str(&json).unwrap();
        
        match deserialized {
            WsMessage::Ping(data) => {
                assert_eq!(data.timestamp, 1234567890);
                assert_eq!(data.message, Some("test".to_string()));
            }
            _ => panic!("Wrong message type"),
        }
    }
    
    #[test]
    fn test_websocket_manager_creation() {
        let manager = WebSocketManager::new();
        assert_eq!(manager.connection_count(), 0);
        assert!(manager.channels.contains_key(&SubscriptionTopic::MiningProgress));
        assert!(manager.channels.contains_key(&SubscriptionTopic::NewBlocks));
    }
}