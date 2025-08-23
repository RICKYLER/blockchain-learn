# ChainKV - Basic Blockchain Implementation

A simple blockchain implementation in Rust featuring Proof-of-Work (PoW), digital signatures, and Merkle trees for data integrity.

## Features

- **Proof-of-Work Mining**: Configurable difficulty level (default: 3 leading zeros)
- **Digital Signatures**: Ed25519 cryptographic signatures for block authentication
- **Merkle Trees**: Efficient data integrity verification for operations
- **Key-Value Operations**: Support for PUT and DELETE operations
- **Chain Verification**: Complete blockchain integrity validation

## Architecture

### Core Components

- **Block**: Contains index, timestamp, operations, previous hash, Merkle root, nonce, hash, and signature
- **Chain**: Manages the blockchain with automatic keypair generation and block validation
- **Operations**: PUT and DELETE operations for key-value storage
- **Merkle Root**: Cryptographic hash tree for efficient operation verification

### Security Features

- **Ed25519 Signatures**: Each block is cryptographically signed
- **SHA-256 Hashing**: Secure hash function for block integrity
- **Proof-of-Work**: Mining requirement prevents spam and ensures consensus
- **Chain Linking**: Each block references the previous block's hash

## Getting Started

### Prerequisites

- Rust 2024 edition or later
- Cargo package manager

### Installation

```bash
cd rust-01/chain_kv
cargo build --release
```

### Running the Application

```bash
cargo run
```

The application will:
1. Create a genesis block
2. Mine a block with PUT operations (user=Alice, role=admin)
3. Mine another block with a DELETE operation (removing role)
4. Verify the entire blockchain integrity

## Dependencies

```toml
sha2 = "0.10.9"                    # SHA-256 hashing
serde = { version = "1.0.219", features = ["derive"] }  # Serialization
serde_json = "1.0.143"             # JSON serialization
hex = "0.4.3"                      # Hexadecimal encoding
chrono = { version = "0.4.41", default-features = false, features = ["clock"] }  # Timestamps
ed25519-dalek = { version = "2.2.0", features = ["std"] }  # Digital signatures
rand = "0.9.2"                     # Random number generation
```

## Code Structure

```
src/
└── main.rs                 # Main application with blockchain logic
```

### Key Functions

- `merkle_root(ops: &[Op])`: Computes Merkle root from operations
- `Block::new()`: Creates and mines a new block with PoW
- `Block::verify()`: Verifies block integrity and signature
- `Chain::new()`: Initializes blockchain with genesis block
- `Chain::add_block()`: Mines and adds new block to chain
- `Chain::verify_chain()`: Validates entire blockchain

## Example Output

```
✅ Mined block 1 with nonce 12847
✅ Mined block 2 with nonce 8392
Verify chain: true
```

## Customization

### Adjusting Difficulty

Modify the difficulty parameter in `main()` function:

```rust
let mut chain = Chain::new(4); // Requires 4 leading zeros (harder)
```

### Adding Custom Operations

Extend the `Op` enum to support additional operation types:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
enum Op {
    Put { key: String, value: String },
    Del { key: String },
    // Add your custom operations here
}
```

## Security Considerations

- **Private Key Management**: Keypairs are generated automatically but not persisted
- **Mining Difficulty**: Higher difficulty increases security but requires more computational power
- **Signature Verification**: All blocks (except genesis) must have valid signatures
- **Hash Chain Integrity**: Any tampering breaks the chain verification

## Development

### Testing

```bash
cargo test
```

### Building for Production

```bash
cargo build --release
```

## Next Steps

This is a foundational implementation. For more advanced features, see:
- `rust-02/chain_kv_pow_sig_merkle`: Enhanced CLI with key management and persistence
- `rust-03/chain_kv_full`: Full-featured blockchain with HTTP API and batching

## License

This project is part of a blockchain learning series demonstrating core concepts in Rust.