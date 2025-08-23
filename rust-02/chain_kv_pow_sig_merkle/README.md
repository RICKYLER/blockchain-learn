# ChainKV - Enhanced CLI Blockchain

An advanced blockchain implementation in Rust featuring an interactive CLI, key management, chain persistence, and comprehensive verification system.

## Features

- **Interactive CLI**: Full command-line interface for blockchain operations
- **Key Management**: Generate, save, and load Ed25519 keypairs
- **Chain Persistence**: Save and load blockchain state to/from JSON files
- **Proof-of-Work Mining**: Configurable difficulty with mining progress display
- **Digital Signatures**: Ed25519 cryptographic signatures for all blocks
- **Merkle Trees**: Efficient data integrity verification
- **State Materialization**: Query current key-value state from blockchain history
- **Complete Verification**: Validate PoW, signatures, and chain integrity

## Architecture

### Enhanced Components

- **Block**: Extended with signature verification and mining progress
- **Chain**: Advanced chain management with persistence and verification
- **Key Management**: Secure keypair generation and storage
- **CLI Interface**: Interactive command system for all operations
- **State Engine**: Materializes current state from operation history

### Security Enhancements

- **Persistent Key Storage**: Secure keypair management with JSON serialization
- **Signature Verification**: All blocks must be properly signed (except genesis)
- **Chain Validation**: Complete integrity checking on load
- **Mining Progress**: Real-time PoW mining feedback

## Getting Started

### Prerequisites

- Rust 2024 edition or later
- Cargo package manager

### Installation

```bash
cd rust-02/chain_kv_pow_sig_merkle
cargo build --release
```

### Running the Application

```bash
cargo run
```

## CLI Commands

### Key Management

```bash
# Generate a new keypair
keygen mykey.json

# Load an existing keypair
loadkey mykey.json

# Show current public key
whoami
```

### Blockchain Operations

```bash
# Set a key-value pair (requires loaded key)
set username Alice
set role admin

# Delete a key (requires loaded key)
del role

# Get current value
get username

# Show complete state
state
```

### Chain Management

```bash
# Verify blockchain integrity
verify

# Save chain to file
save mychain.json

# Load chain from file
load mychain.json

# Adjust mining difficulty (1-9)
difficulty 4
```

### Utility Commands

```bash
# Show help
help

# Exit application
exit
```

## Dependencies

```toml
sha2 = "0.10.9"                    # SHA-256 hashing
serde = { version = "1.0.219", features = ["derive"] }  # Serialization
serde_json = "1.0.143"             # JSON serialization
hex = "0.4.3"                      # Hexadecimal encoding
chrono = { version = "0.4.41", default-features = false, features = ["clock"] }  # Timestamps
ed25519-dalek = { version = "2.2.0", features = ["std", "rand_core"] }  # Digital signatures
rand = "0.8.5"                     # Random number generation
```

## Code Structure

```
src/
└── main.rs                 # Complete CLI application with all features
```

### Key Functions

- `merkle_root(ops: &[Op])`: Computes Merkle root from operations
- `Block::new()`: Creates and mines new block with progress display
- `Block::verify()`: Comprehensive block validation
- `Chain::genesis()`: Creates genesis block
- `Chain::append_signed()`: Mines and adds signed blocks
- `Chain::materialize()`: Builds current state from operations
- `Chain::save()/load()`: Blockchain persistence
- `generate_keypair()/load_keypair()`: Key management utilities

## Example Session

```bash
🔗 ChainKV — PoW + Signatures + Merkle

Commands:
  set <key> <value...>      - mine+sign single-op block
  del <key>                 - mine+sign single-op block
  get <key>                 - read value from materialized state
  state                     - dump state
  verify                    - verify PoW, signatures, and links
  save <file>               - save chain JSON
  load <file>               - load chain JSON
  keygen <file>             - generate Ed25519 keypair JSON
  loadkey <file>            - load signing key
  whoami                    - show loaded public key
  difficulty <n>            - set PoW difficulty (1..9)
  help                      - show this help
  exit                      - quit

chain-kv> keygen mykey.json
🔑 generated keypair → mykey.json

chain-kv> loadkey mykey.json
🔑 loaded signing key from mykey.json

chain-kv> set username Alice
⛏️  mining… nonce=12847       rate=2847 H/s last=00012a4f
✅ mined block 1 (nonce 12847)

chain-kv> get username
🔎 Alice

chain-kv> verify
✅ chain ok (2 blocks, difficulty 3)

chain-kv> save mychain.json
💾 saved chain to mychain.json
```

## File Formats

### Keypair JSON Format

```json
{
  "secret_key": "hex-encoded-64-byte-secret-key",
  "public_key": "hex-encoded-32-byte-public-key"
}
```

### Chain JSON Format

```json
{
  "blocks": [
    {
      "index": 0,
      "timestamp": 0,
      "ops": [{"Put": {"key": "__genesis__", "value": "ok"}}],
      "prev_hash": "0",
      "merkle_root": "GENESIS",
      "nonce": 0,
      "hash": "GENESIS",
      "signature": null,
      "signer_pubkey": null
    }
  ],
  "difficulty": 3
}
```

## Security Features

### Cryptographic Security
- **Ed25519 Signatures**: Industry-standard elliptic curve signatures
- **SHA-256 Hashing**: Secure cryptographic hash function
- **Merkle Tree Verification**: Efficient operation integrity checking
- **Proof-of-Work**: Computational difficulty prevents spam

### Key Management Security
- **Secure Key Generation**: Uses OS random number generator
- **Key Persistence**: Safely store and load keypairs
- **Public Key Verification**: Validate signatures against stored public keys
- **Genesis Block Exception**: Only genesis block lacks signature requirement

### Chain Integrity
- **Hash Chain Linking**: Each block references previous block hash
- **Complete Verification**: Validates entire chain on load
- **Operation Ordering**: Maintains chronological operation sequence
- **State Consistency**: Materializes consistent state from operation history

## Advanced Usage

### Custom Difficulty

Adjust mining difficulty for different security/performance trade-offs:

```bash
chain-kv> difficulty 5  # Requires 5 leading zeros (much harder)
```

### Batch Operations

While this version doesn't support batching, you can quickly execute multiple operations:

```bash
chain-kv> set user1 Alice
chain-kv> set user2 Bob
chain-kv> set user3 Charlie
```

### Chain Analysis

Use the verification system to analyze chain properties:

```bash
chain-kv> verify
✅ chain ok (4 blocks, difficulty 3)

chain-kv> state
user1 = Alice
user2 = Bob
user3 = Charlie
```

## Development

### Testing

```bash
cargo test
```

### Building for Production

```bash
cargo build --release
```

### Debugging

The application provides detailed error messages and mining progress:

```bash
❌ no signing key loaded. Use: loadkey <file>
❌ verify failed: signature verify failed
⛏️  mining… nonce=45123       rate=3421 H/s last=0001a2b3
```

## Migration from rust-01

This version extends the basic blockchain with:
- Interactive CLI instead of hardcoded operations
- Persistent key management
- Chain save/load functionality
- Real-time mining progress
- Enhanced error handling
- State querying capabilities

## Next Steps

For even more advanced features, see:
- `rust-03/chain_kv_full`: Adds HTTP API server, batching, and web interface

## Troubleshooting

### Common Issues

1. **"no signing key loaded"**: Generate and load a keypair first
2. **"verify failed"**: Chain integrity compromised, check file corruption
3. **"bad signature hex"**: Keypair file may be corrupted
4. **Mining too slow**: Reduce difficulty level

### Performance Tips

- Lower difficulty for faster mining during development
- Use SSD storage for better I/O performance
- Monitor system resources during intensive mining

## License

This project is part of a blockchain learning series demonstrating advanced concepts in Rust.