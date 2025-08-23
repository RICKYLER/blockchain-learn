# LedgerDB - High-Performance Blockchain Ledger Database

A production-ready, high-performance blockchain implementation in Rust featuring an embedded database, WebSocket real-time updates, and enterprise-grade architecture. LedgerDB represents the most advanced implementation in this learning series, designed for scalable, real-world blockchain applications.

## 🚀 Features

### Core Blockchain Features
- ✅ **Advanced Proof-of-Work**: Configurable difficulty with optimized mining
- ✅ **Cryptographic Security**: Multiple signature algorithms (ECDSA secp256k1, Ed25519)
- ✅ **Merkle Tree Verification**: Efficient transaction integrity validation
- ✅ **Transaction Processing**: High-throughput transaction handling
- ✅ **Chain Validation**: Comprehensive blockchain integrity checks

### Performance & Storage
- ✅ **Embedded Database**: High-performance Sled database for persistence
- ✅ **Async Operations**: Tokio-based asynchronous processing
- ✅ **Memory Optimization**: Efficient data structures and caching
- ✅ **Concurrent Mining**: Multi-threaded proof-of-work computation
- ✅ **Batch Processing**: Optimized transaction batching

### API & Real-time Features
- ✅ **REST API**: Full HTTP API with Axum framework
- ✅ **WebSocket Support**: Real-time blockchain updates
- ✅ **CORS Enabled**: Cross-origin resource sharing
- ✅ **Request Tracing**: Comprehensive logging and monitoring
- ✅ **Error Handling**: Production-grade error management

### Enterprise Features
- ✅ **Modular Architecture**: Clean separation of concerns
- ✅ **Configuration Management**: Environment-based configuration
- ✅ **Comprehensive Logging**: Structured logging with tracing
- ✅ **Input Validation**: Robust data validation utilities
- ✅ **Network Utilities**: Advanced networking capabilities

## 🏗️ Architecture

### Project Structure
```
src/
├── api/                    # HTTP API and WebSocket handlers
│   ├── handlers.rs        # REST API endpoint handlers
│   ├── middleware.rs      # Custom middleware components
│   ├── responses.rs       # API response structures
│   └── websocket.rs       # WebSocket connection management
├── config/                # Configuration management
│   └── mod.rs            # Application configuration
├── core/                  # Core blockchain logic
│   ├── block.rs          # Block structure and operations
│   ├── blockchain.rs     # Blockchain management
│   └── transaction.rs    # Transaction handling
├── crypto/                # Cryptographic operations
│   ├── hash.rs           # Hashing utilities
│   ├── keys.rs           # Key management
│   ├── merkle.rs         # Merkle tree implementation
│   └── pow.rs            # Proof-of-Work algorithm
├── storage/               # Data persistence layer
│   └── mod.rs            # Database operations
├── utils/                 # Utility modules
│   ├── bytes.rs          # Byte manipulation
│   ├── collections.rs    # Collection utilities
│   ├── format.rs         # Data formatting
│   ├── fs.rs             # File system operations
│   ├── math.rs           # Mathematical utilities
│   ├── network.rs        # Network utilities
│   ├── random.rs         # Random number generation
│   ├── time.rs           # Time utilities
│   └── validation.rs     # Input validation
└── main.rs               # Application entry point
```

### Key Components

#### Core Blockchain (`core/`)
- **Block**: Immutable block structure with cryptographic linking
- **Blockchain**: Chain management with validation and consensus
- **Transaction**: Secure transaction processing and verification

#### Cryptography (`crypto/`)
- **Hash**: SHA-256 and other cryptographic hash functions
- **Keys**: Public/private key management and operations
- **Merkle**: Merkle tree construction and verification
- **PoW**: Configurable Proof-of-Work implementation

#### API Layer (`api/`)
- **Handlers**: RESTful API endpoints for blockchain operations
- **WebSocket**: Real-time updates and bidirectional communication
- **Middleware**: Authentication, logging, and request processing
- **Responses**: Standardized API response formats

#### Storage (`storage/`)
- **Sled Database**: High-performance embedded key-value store
- **Persistence**: Blockchain state and transaction storage
- **Caching**: Optimized data retrieval and storage

## 🛠️ Prerequisites

- **Rust 1.70+**: Latest stable Rust toolchain
- **System Requirements**: 4GB+ RAM, multi-core CPU recommended
- **Network**: For API server functionality

## 🚀 Quick Start

### Installation
```bash
# Clone the repository
git clone https://github.com/rjaysolamo/blockchain-learn.git
cd rust-04/ledgerdb

# Build the project
cargo build --release
```

### Running the Application
```bash
# Start the blockchain server
cargo run

# Or run in release mode for better performance
cargo run --release
```

### Development Mode
```bash
# Run with debug logging
RUST_LOG=debug cargo run

# Run tests
cargo test

# Run with specific features
cargo run --features "websocket-support"
```

## 📡 API Endpoints

### Blockchain Operations
- `GET /api/blockchain/info` - Get blockchain information
- `GET /api/blockchain/blocks` - List all blocks
- `GET /api/blockchain/blocks/{id}` - Get specific block
- `POST /api/blockchain/mine` - Mine a new block

### Transaction Management
- `POST /api/transactions` - Submit new transaction
- `GET /api/transactions/{id}` - Get transaction details
- `GET /api/transactions/pending` - List pending transactions

### Real-time Updates
- `WS /ws` - WebSocket connection for real-time updates

## 🔧 Configuration

### Environment Variables
```bash
# Server configuration
SERVER_HOST=0.0.0.0
SERVER_PORT=3000

# Database configuration
DB_PATH=./data
DB_CACHE_SIZE=1024

# Mining configuration
MINING_DIFFICULTY=4
MINING_THREADS=4

# Logging
RUST_LOG=info
```

### Configuration File
Create a `config.toml` file:
```toml
[server]
host = "0.0.0.0"
port = 3000

[database]
path = "./data"
cache_size = 1024

[mining]
difficulty = 4
threads = 4

[logging]
level = "info"
```

## 🧪 Testing

### Unit Tests
```bash
# Run all tests
cargo test

# Run specific test module
cargo test core::

# Run with output
cargo test -- --nocapture
```

### Integration Tests
```bash
# Run integration tests
cargo test --test integration

# Test API endpoints
cargo test api::
```

### Performance Testing
```bash
# Benchmark mining performance
cargo bench

# Load testing (requires additional tools)
# Use tools like wrk or artillery for API load testing
```

## 🔒 Security Features

### Cryptographic Security
- **Multiple Signature Algorithms**: ECDSA secp256k1, Ed25519
- **Secure Hashing**: SHA-256 for all cryptographic operations
- **Key Management**: Secure key generation and storage
- **Address Generation**: Cryptographically secure address derivation

### Network Security
- **CORS Protection**: Configurable cross-origin policies
- **Input Validation**: Comprehensive request validation
- **Rate Limiting**: Protection against spam and DoS attacks
- **Secure Headers**: Security-focused HTTP headers

### Data Integrity
- **Chain Validation**: Continuous blockchain integrity checks
- **Transaction Verification**: Cryptographic transaction validation
- **Merkle Proofs**: Efficient transaction inclusion proofs
- **Immutable Storage**: Tamper-evident data storage

## 🚀 Production Deployment

### Docker Deployment
```dockerfile
FROM rust:1.70 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates
COPY --from=builder /app/target/release/ledgerdb /usr/local/bin/
EXPOSE 3000
CMD ["ledgerdb"]
```

### Kubernetes Deployment
```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: ledgerdb
spec:
  replicas: 3
  selector:
    matchLabels:
      app: ledgerdb
  template:
    metadata:
      labels:
        app: ledgerdb
    spec:
      containers:
      - name: ledgerdb
        image: ledgerdb:latest
        ports:
        - containerPort: 3000
        env:
        - name: RUST_LOG
          value: "info"
```

### Performance Optimization
- **Database Tuning**: Optimize Sled configuration for your workload
- **Connection Pooling**: Configure appropriate connection limits
- **Caching Strategy**: Implement Redis for distributed caching
- **Load Balancing**: Use nginx or similar for request distribution

## 📊 Monitoring & Observability

### Metrics
- **Blockchain Metrics**: Block height, transaction throughput
- **Performance Metrics**: Response times, database operations
- **System Metrics**: CPU, memory, disk usage
- **Custom Metrics**: Application-specific measurements

### Logging
- **Structured Logging**: JSON-formatted logs with tracing
- **Log Levels**: Configurable logging levels
- **Request Tracing**: End-to-end request tracking
- **Error Tracking**: Comprehensive error logging

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes with tests
4. Run the test suite (`cargo test`)
5. Commit your changes (`git commit -m 'Add amazing feature'`)
6. Push to the branch (`git push origin feature/amazing-feature`)
7. Open a Pull Request

### Development Guidelines
- Follow Rust best practices and idioms
- Write comprehensive tests for new features
- Update documentation for API changes
- Use `cargo fmt` and `cargo clippy` for code quality

## 📖 Learning Resources

### Blockchain Concepts
- [Bitcoin Whitepaper](https://bitcoin.org/bitcoin.pdf)
- [Ethereum Yellow Paper](https://ethereum.github.io/yellowpaper/paper.pdf)
- [Consensus Algorithms](https://en.wikipedia.org/wiki/Consensus_algorithm)

### Rust Resources
- [The Rust Book](https://doc.rust-lang.org/book/)
- [Async Programming in Rust](https://rust-lang.github.io/async-book/)
- [Rust Performance Book](https://nnethercote.github.io/perf-book/)

### Web3 Development
- [Web3 Developer Stack](https://ethereum.org/en/developers/)
- [Blockchain Development Best Practices](https://consensys.github.io/smart-contract-best-practices/)

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙋‍♂️ Support

For questions, issues, or contributions:
- Open an issue on GitHub
- Check the [documentation](./docs/)
- Review the code comments for implementation details
- Join our community discussions

---

**Built with ❤️ in Rust for the Web3 future**

LedgerDB represents the pinnacle of blockchain development in this learning series, combining performance, security, and scalability for real-world applications.