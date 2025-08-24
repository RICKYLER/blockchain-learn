//! LedgerDB - A high-performance blockchain implementation in Rust
//!
//! This is the main entry point for the LedgerDB blockchain application.
//! It initializes the blockchain, starts the HTTP API server, and handles
//! WebSocket connections for real-time updates.

use axum::{
    extract::ws::{WebSocket, WebSocketUpgrade},
    extract::{Path, Query, State},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::Arc,
    time::Duration,
};
use tokio::time::sleep;
use tower::ServiceBuilder;
use tower_http::{
    cors::CorsLayer,
    trace::TraceLayer,
};

// Import our modules
mod api;
mod config;
mod core;
mod crypto;
mod error;
mod storage;
mod utils;

use api::{
    handlers::*,
    middleware::*,
    responses::*,
    websocket::*,
};
use crate::core::{
    blockchain::Blockchain,
    block::Block,
    transaction::Transaction,
};
use crate::crypto::{
    pow::{ProofOfWorkMiner, MiningProgress},
    Hash256,
    Address,
    PublicKey,
    SignatureAlgorithm,
};
use crate::storage::PersistentStorage;
use crate::utils::{
    time::current_timestamp,
    format::format_hash,
};

// Use the AppState from the api module
use api::AppState;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    utils::init_logging();
    
    println!("🚀 Starting LedgerDB blockchain...");
    
    // Initialize storage
    let storage = Arc::new(PersistentStorage::new("./data".to_string()).expect("Failed to initialize storage"));

    // Create a genesis address
    let genesis_public_key = PublicKey::new(
        SignatureAlgorithm::EcdsaSecp256k1,
        vec![0u8; 33] // Placeholder public key
    );
    let genesis_address = Address::from_public_key(&genesis_public_key);
    
    // Create blockchain config
    let config = crate::core::blockchain::BlockchainConfig::default();
    
    // Initialize blockchain
    let blockchain = Arc::new(tokio::sync::RwLock::new(
        Blockchain::new(config, genesis_address).expect("Failed to create blockchain")
    ));

    // Initialize mining progress broadcaster
    let (mining_progress_tx, _) = tokio::sync::broadcast::channel::<MiningProgress>(100);

    // Initialize miner
    let miner = Arc::new(tokio::sync::RwLock::new(None::<ProofOfWorkMiner>));

    // Create API config
    let config = api::ApiConfig::default();

    // Create application state
    let app_state = api::AppState {
        blockchain: blockchain.clone(),
        storage: storage.clone(),
        mining_progress_tx,
        miner,
        config,
    };
    
    // The blockchain is already initialized with genesis block in Blockchain::new()
    println!("📦 Genesis block created successfully!");
    
    // Build the router with all endpoints
    let app = Router::new()
        // API routes
        .route("/api/blocks", get(get_blocks))
        .route("/api/blocks/:hash", get(get_block_by_hash))
        .route("/api/transactions", get(get_transactions))
        .route("/api/transactions/:hash", get(get_transaction_by_hash))
        .route("/api/mine", post(mine_block))
        .route("/api/submit_transaction", post(submit_transaction))
        .route("/api/balance/:address", get(get_balance))
        .route("/api/stats", get(get_network_stats))
        .route("/api/health", get(health_check))
        
        // WebSocket endpoint
        .route("/ws", get(websocket_handler))
        
        // Static file serving (for frontend)
        .route("/", get(serve_index))
        .route("/static/*file", get(serve_static))
        
        // Add middleware
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(CorsLayer::permissive())
                .layer(axum::middleware::from_fn(request_logging_middleware))
                .layer(axum::middleware::from_fn(security_headers_middleware))
        )
        .with_state(app_state);
    
    // Start the server
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("🌐 LedgerDB API server starting on http://{}", addr);
    println!("📊 WebSocket endpoint available at ws://{}/ws", addr);
    println!("🔗 Blockchain explorer UI at http://{}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    
    Ok(())
}

/// Serve the main index.html file
async fn serve_index() -> impl IntoResponse {
    // Serve embedded HTML since static file doesn't exist yet
    Html(
        r#"<!DOCTYPE html>
<html>
<head>
    <title>LedgerDB Blockchain Explorer</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 40px; background: #f5f5f5; }
        .container { max-width: 800px; margin: 0 auto; background: white; padding: 30px; border-radius: 8px; box-shadow: 0 2px 10px rgba(0,0,0,0.1); }
        h1 { color: #333; text-align: center; }
        .status { background: #e8f5e8; padding: 15px; border-radius: 5px; margin: 20px 0; }
        .endpoint { background: #f8f9fa; padding: 10px; margin: 10px 0; border-left: 4px solid #007bff; }
        code { background: #f1f1f1; padding: 2px 6px; border-radius: 3px; }
    </style>
</head>
<body>
    <div class="container">
        <h1>🚀 LedgerDB Blockchain Explorer</h1>
        <div class="status">
            <h3>✅ Blockchain is running!</h3>
            <p>The LedgerDB blockchain node is operational and ready to process transactions.</p>
        </div>
        
        <h3>📡 API Endpoints</h3>
        <div class="endpoint"><strong>GET /api/blocks</strong> - Get all blocks</div>
        <div class="endpoint"><strong>GET /api/blocks/:hash</strong> - Get block by hash</div>
        <div class="endpoint"><strong>GET /api/transactions</strong> - Get all transactions</div>
        <div class="endpoint"><strong>GET /api/transactions/:hash</strong> - Get transaction by hash</div>
        <div class="endpoint"><strong>POST /api/mine</strong> - Mine a new block</div>
        <div class="endpoint"><strong>POST /api/submit_transaction</strong> - Submit a transaction</div>
        <div class="endpoint"><strong>GET /api/balance/:address</strong> - Get address balance</div>
        <div class="endpoint"><strong>GET /api/stats</strong> - Get network statistics</div>
        <div class="endpoint"><strong>GET /api/health</strong> - Health check</div>
        
        <h3>🔌 WebSocket</h3>
        <div class="endpoint"><strong>WS /ws</strong> - Real-time blockchain updates</div>
        
        <p style="text-align: center; margin-top: 30px; color: #666;">
            Built with ❤️ using Rust, Axum, and Tokio
        </p>
    </div>
</body>
</html>"#
    )
}

/// Serve static files
async fn serve_static(Path(file): Path<String>) -> impl IntoResponse {
    // In a real application, you'd serve actual static files
    // For now, return a simple response
    match file.as_str() {
        "style.css" => (
            StatusCode::OK,
            [("content-type", "text/css")],
            "/* LedgerDB Styles */\nbody { font-family: 'Segoe UI', sans-serif; }"
        ).into_response(),
        _ => (
            StatusCode::NOT_FOUND,
            "File not found"
        ).into_response()
    }
}

/// WebSocket handler
async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> Response {
    ws.on_upgrade(|socket| handle_websocket(socket, state))
}

/// Handle WebSocket connections
async fn handle_websocket(socket: WebSocket, state: AppState) {

    let connection_id = manager.add_connection();
    drop(manager);
    
    println!("🔌 New WebSocket connection: {}", connection_id);
    
    // Handle the WebSocket connection
    if let Err(e) = handle_mining_progress_websocket(socket, state.clone()).await {
        eprintln!("❌ WebSocket error: {}", e);
    }
    
    // Clean up connection

    manager.remove_connection(&connection_id);
    println!("🔌 WebSocket connection closed: {}", connection_id);
}
