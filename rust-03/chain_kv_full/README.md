# ChainKV Full - Production-Ready Blockchain

A comprehensive blockchain implementation in Rust featuring HTTP API server, transaction batching, interactive CLI, and enterprise-grade features for production deployment.

## Features

### Core Blockchain
- **Proof-of-Work Mining**: Configurable difficulty with real-time progress monitoring
- **Digital Signatures**: Ed25519 cryptographic signatures for all transactions
- **Merkle Trees**: Efficient data integrity verification
- **Chain Persistence**: Save and load blockchain state
- **Complete Verification**: Validate PoW, signatures, and chain integrity

### Advanced Features
- **Transaction Batching**: Group multiple operations into single blocks
- **HTTP API Server**: RESTful API for blockchain interactions
- **Interactive CLI**: Full command-line interface
- **Key Management**: Secure keypair generation and storage
- **State Materialization**: Query current key-value state
- **Async Mining**: Non-blocking mining with progress updates

### Production Ready
- **Axum Web Framework**: High-performance async HTTP server
- **Tokio Runtime**: Async/await support for concurrent operations
- **JSON API**: Standard REST endpoints for integration
- **Error Handling**: Comprehensive error responses
- **Scalable Architecture**: Designed for high-throughput applications

## Architecture

### System Components

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   HTTP API      │    │   CLI Interface │    │  Blockchain     │
│   (Axum)        │    │   (Interactive) │    │  Core Engine    │
├─────────────────┤    ├─────────────────┤    ├─────────────────┤
│ GET /state      │    │ set <k> <v>     │    │ Block Mining    │
│ POST /set       │    │ get <k>         │    │ PoW Validation  │
│ POST /del       │    │ begin/commit    │    │ Signature Verify│
│ POST /verify    │    │ save/load       │    │ Merkle Trees    │
│ GET /verify     │    │ serve <port>    │    │ State Engine    │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

### Enhanced Security
- **Multi-layer Validation**: API, CLI, and core validation
- **Async-safe Operations**: Thread-safe blockchain operations
- **Request Validation**: Input sanitization and validation
- **Error Isolation**: Proper error handling and recovery

## Getting Started

### Prerequisites

- Rust 2024 edition or later
- Cargo package manager
- Network access for HTTP server (optional)

### Installation

```bash
cd rust-03/chain_kv_full
cargo build --release
```

### Running the Application

#### CLI Mode
```bash
cargo run
```

#### HTTP Server Mode
```bash
cargo run
# Then in CLI:
chain-kv> serve 3000
🌐 HTTP server running on http://localhost:3000
```

## CLI Commands

### Key Management
```bash
keygen mykey.json          # Generate new keypair
loadkey mykey.json         # Load existing keypair
whoami                     # Show current public key
```

### Single Operations
```bash
set username Alice         # Mine single-op block with progress
del username               # Delete key in single-op block
get username               # Query current value
state                      # Show complete state
```

### Batch Operations
```bash
begin                      # Start transaction batch
addput user1 Alice         # Add PUT to batch
addput user2 Bob           # Add PUT to batch
adddel old_user            # Add DELETE to batch
commit                     # Mine multi-op block
abort                      # Cancel current batch
```

### Chain Management
```bash
verify                     # Verify blockchain integrity
save mychain.json          # Save chain to file
load mychain.json          # Load chain from file
difficulty 4               # Set mining difficulty (1-9)
```

### Server Operations
```bash
serve 3000                 # Start HTTP server on port 3000
help                       # Show all commands
exit                       # Quit application
```

## HTTP API Reference

### Base URL
```
http://localhost:<port>
```

### Endpoints

#### GET /state
Retrieve current blockchain state

**Response:**
```json
{
  "user1": "Alice",
  "user2": "Bob",
  "role": "admin"
}
```

#### POST /set
Set a key-value pair

**Request:**
```json
{
  "key": "username",
  "value": "Alice"
}
```

**Response:**
```json
{
  "success": true,
  "block_index": 5,
  "nonce": 12847
}
```

#### POST /del
Delete a key

**Request:**
```json
{
  "key": "username"
}
```

**Response:**
```json
{
  "success": true,
  "block_index": 6,
  "nonce": 8392
}
```

#### GET /verify
Verify blockchain integrity

**Response:**
```json
{
  "ok": true,
  "error": null
}
```

#### POST /difficulty
Set mining difficulty

**Request:**
```json
{
  "n": 4
}
```

**Response:**
```json
{
  "success": true
}
```

### Error Responses

```json
{
  "error": "No signing key loaded"
}
```

## Dependencies

```toml
# Core blockchain
sha2 = "0.10.9"
serde = { version = "1.0.219", features = ["derive"] }
serde_json = "1.0.143"
hex = "0.4.3"
chrono = { version = "0.4.41", default-features = false, features = ["clock"] }
ed25519-dalek = { version = "2.2.0", features = ["std", "rand_core"] }
rand = "0.9.2"
rand_core = { version = "0.6", features = ["getrandom"] }

# HTTP server
axum = "0.8.4"
tokio = { version = "1.47.1", features = ["rt-multi-thread", "macros"] }
```

## Code Structure

```
src/
└── main.rs                 # Complete application with CLI and HTTP server
```

### Key Components

#### Blockchain Core
- `merkle_root()`: Merkle tree computation
- `Block::new()`: Mining with progress display
- `Block::verify()`: Comprehensive validation
- `Chain::genesis()`: Genesis block creation
- `Chain::append_signed()`: Block mining and addition

#### Batching System
- `Chain::begin_batch()`: Start transaction batch
- `Chain::add_put()/add_del()`: Add operations to batch
- `Chain::commit_batch()`: Mine multi-operation block
- `Chain::abort_batch()`: Cancel current batch

#### HTTP Server
- `app_state()`: Shared application state
- `get_state()`: State retrieval endpoint
- `set_key()`: Key-value setting endpoint
- `del_key()`: Key deletion endpoint
- `verify_chain()`: Chain verification endpoint

## Example Usage

### CLI Session with Batching

```bash
🔗 ChainKV — PoW + Signatures + Merkle

chain-kv> keygen mykey.json
🔑 generated keypair → mykey.json

chain-kv> loadkey mykey.json
🔑 loaded signing key from mykey.json

chain-kv> begin
📦 batch started

chain-kv> addput user1 Alice
📦 added PUT user1=Alice to batch (1 ops)

chain-kv> addput user2 Bob
📦 added PUT user2=Bob to batch (2 ops)

chain-kv> addput role admin
📦 added PUT role=admin to batch (3 ops)

chain-kv> commit
⛏️  mining… nonce=15234       rate=3421 H/s last=00089a2f
✅ mined block 1 (nonce 15234)
📦 committed batch with 3 operations

chain-kv> state
user1 = Alice
user2 = Bob
role = admin

chain-kv> serve 3000
🌐 HTTP server running on http://localhost:3000
```

### HTTP API Usage

```bash
# Set a value
curl -X POST http://localhost:3000/set \
  -H "Content-Type: application/json" \
  -d '{"key": "username", "value": "Charlie"}'

# Get current state
curl http://localhost:3000/state

# Delete a key
curl -X POST http://localhost:3000/del \
  -H "Content-Type: application/json" \
  -d '{"key": "username"}'

# Verify chain
curl http://localhost:3000/verify
```

## Advanced Features

### Transaction Batching

Batch multiple operations into a single block for efficiency:

```bash
chain-kv> begin
chain-kv> addput user1 Alice
chain-kv> addput user2 Bob
chain-kv> adddel old_user
chain-kv> commit  # All operations in one block
```

### Async Mining

Mining operations are non-blocking with real-time progress:

```bash
⛏️  mining… nonce=45123       rate=2847 H/s last=00012a4f
⛏️  mining… nonce=89456       rate=3021 H/s last=0001b5c2
✅ mined block 3 (nonce 89456)
```

### Concurrent HTTP Server

The HTTP server handles multiple concurrent requests:

```rust
// Server runs on separate async task
tokio::spawn(async move {
    axum::serve(listener, app).await.unwrap();
});
```

## Production Deployment

### Environment Setup

```bash
# Build optimized binary
cargo build --release

# Run with specific configuration
RUST_LOG=info ./target/release/chain_kv_full
```

### Docker Deployment

```dockerfile
FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates
COPY --from=builder /app/target/release/chain_kv_full /usr/local/bin/
EXPOSE 3000
CMD ["chain_kv_full"]
```

### Performance Tuning

1. **Mining Difficulty**: Balance security vs. performance
2. **Batch Size**: Optimize operations per block
3. **Server Threads**: Configure Tokio runtime
4. **Memory Usage**: Monitor chain size growth

## Security Considerations

### Cryptographic Security
- **Ed25519 Signatures**: Quantum-resistant signatures
- **SHA-256 Hashing**: Industry-standard hash function
- **Secure Random**: OS-level entropy for key generation
- **Merkle Tree Integrity**: Tamper-evident operation verification

### Network Security
- **Input Validation**: All API inputs validated
- **Error Handling**: No sensitive information in errors
- **Rate Limiting**: Consider implementing for production
- **HTTPS**: Use TLS in production environments

### Operational Security
- **Key Management**: Secure keypair storage
- **Chain Backup**: Regular blockchain state backups
- **Access Control**: Implement authentication for production
- **Monitoring**: Log all operations and errors

## Development

### Testing

```bash
# Run all tests
cargo test

# Test with logging
RUST_LOG=debug cargo test

# Integration tests
cargo test --test integration
```

### Debugging

```bash
# Debug mode with detailed logging
RUST_LOG=debug cargo run

# Profile performance
cargo build --release
perf record ./target/release/chain_kv_full
```

### Contributing

1. Fork the repository
2. Create feature branch
3. Add tests for new features
4. Ensure all tests pass
5. Submit pull request

## Migration Guide

### From rust-02

This version adds:
- HTTP API server functionality
- Transaction batching system
- Async/await support
- Enhanced error handling
- Production-ready features

### Upgrading Chain Files

Chain files from rust-02 are compatible:

```bash
chain-kv> load ../rust-02/chain_kv_pow_sig_merkle/mychain.json
📥 loaded chain (5 blocks) | difficulty=3
```

## API Integration Examples

### JavaScript/Node.js

```javascript
const axios = require('axios');
const BASE_URL = 'http://localhost:3000';

// Set a value
async function setValue(key, value) {
  const response = await axios.post(`${BASE_URL}/set`, { key, value });
  return response.data;
}

// Get current state
async function getState() {
  const response = await axios.get(`${BASE_URL}/state`);
  return response.data;
}
```

### Python

```python
import requests

BASE_URL = 'http://localhost:3000'

def set_value(key, value):
    response = requests.post(f'{BASE_URL}/set', json={'key': key, 'value': value})
    return response.json()

def get_state():
    response = requests.get(f'{BASE_URL}/state')
    return response.json()
```

### Go

```go
package main

import (
    "bytes"
    "encoding/json"
    "net/http"
)

type SetRequest struct {
    Key   string `json:"key"`
    Value string `json:"value"`
}

func setValue(key, value string) error {
    req := SetRequest{Key: key, Value: value}
    jsonData, _ := json.Marshal(req)
    
    _, err := http.Post("http://localhost:3000/set", 
                       "application/json", 
                       bytes.NewBuffer(jsonData))
    return err
}
```

## Troubleshooting

### Common Issues

1. **"No signing key loaded"**
   - Solution: Generate and load a keypair first
   ```bash
   keygen mykey.json
   loadkey mykey.json
   ```

2. **"Port already in use"**
   - Solution: Use a different port or kill existing process
   ```bash
   serve 3001  # Try different port
   ```

3. **"Batch not active"**
   - Solution: Start a batch before adding operations
   ```bash
   begin
   addput key value
   commit
   ```

4. **Mining too slow**
   - Solution: Reduce difficulty
   ```bash
   difficulty 2
   ```

### Performance Issues

- **High CPU usage**: Normal during mining, reduce difficulty if needed
- **Memory growth**: Chain size grows with blocks, implement pruning for production
- **Network timeouts**: Increase HTTP client timeouts for mining operations

### Debugging Tips

```bash
# Enable debug logging
RUST_LOG=debug cargo run

# Monitor system resources
top -p $(pgrep chain_kv_full)

# Check network connections
netstat -tlnp | grep 3000
```

## License

This project is part of a blockchain learning series demonstrating production-ready concepts in Rust.

## Acknowledgments

- Built with Rust's excellent async ecosystem
- Uses industry-standard cryptographic libraries
- Inspired by Bitcoin and Ethereum architectures
- Designed for educational and production use