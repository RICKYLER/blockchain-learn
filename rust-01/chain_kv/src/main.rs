use chrono::Utc;
use ed25519_dalek::{Keypair, PublicKey, Signature, Signer, Verifier};
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize)]
enum Op {
    Put { key: String, value: String },
    Del { key: String },
}

/// Compute Merkle Root from operations
fn merkle_root(ops: &[Op]) -> String {
    if ops.is_empty() {
        return "0".into();
    }

    let mut hashes: Vec<String> = ops
        .iter()
        .map(|op| {
            let mut hasher = Sha256::new();
            match op {
                Op::Put { key, value } => {
                    hasher.update(b"PUT");
                    hasher.update(key.as_bytes());
                    hasher.update(value.as_bytes());
                }
                Op::Del { key } => {
                    hasher.update(b"DEL");
                    hasher.update(key.as_bytes());
                }
            }
            hex::encode(hasher.finalize())
        })
        .collect();

    while hashes.len() > 1 {
        let mut new_hashes = vec![];
        for pair in hashes.chunks(2) {
            let mut hasher = Sha256::new();
            hasher.update(pair[0].as_bytes());
            if pair.len() == 2 {
                hasher.update(pair[1].as_bytes());
            } else {
                hasher.update(pair[0].as_bytes()); // duplicate if odd
            }
            new_hashes.push(hex::encode(hasher.finalize()));
        }
        hashes = new_hashes;
    }
    hashes[0].clone()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Block {
    index: u64,
    timestamp: i64,
    ops: Vec<Op>,
    prev_hash: String,
    merkle_root: String,
    nonce: u64,
    hash: String,
    signature: Option<String>,
}

impl Block {
    fn new(index: u64, ops: Vec<Op>, prev_hash: String, keypair: &Keypair, difficulty: usize) -> Self {
        let timestamp = Utc::now().timestamp();
        let merkle_root = merkle_root(&ops);

        // Mining (Proof-of-Work)
        let mut nonce = 0;
        let hash;
        loop {
            let mut hasher = Sha256::new();
            hasher.update(index.to_le_bytes());
            hasher.update(timestamp.to_le_bytes());
            hasher.update(merkle_root.as_bytes());
            hasher.update(prev_hash.as_bytes());
            hasher.update(nonce.to_le_bytes());
            let candidate = hex::encode(hasher.finalize());

            if candidate.starts_with(&"0".repeat(difficulty)) {
                hash = candidate;
                break;
            }
            nonce += 1;
        }

        // Sign the block hash
        let signature = keypair.sign(hash.as_bytes());

        Block {
            index,
            timestamp,
            ops,
            prev_hash,
            merkle_root,
            nonce,
            hash,
            signature: Some(hex::encode(signature.to_bytes())),
        }
    }

    /// Verify block integrity
    fn verify(&self, prev_hash: &str, pubkey: &PublicKey, difficulty: usize) -> bool {
        if self.prev_hash != prev_hash {
            return false;
        }
        if !self.hash.starts_with(&"0".repeat(difficulty)) {
            return false;
        }
        // Verify signature
        if let Some(sig_hex) = &self.signature {
            let sig_bytes = hex::decode(sig_hex).unwrap();
            let sig = Signature::from_bytes(&sig_bytes).unwrap();
            pubkey.verify(self.hash.as_bytes(), &sig).is_ok()
        } else {
            false
        }
    }
}

struct Chain {
    blocks: Vec<Block>,
    keypair: Keypair,
    pubkey: PublicKey,
    difficulty: usize,
}

impl Chain {
    fn new(difficulty: usize) -> Self {
        let mut csprng = OsRng {};
        let keypair = Keypair::generate(&mut csprng);
        let pubkey = keypair.public.clone();

        let genesis = Block {
            index: 0,
            timestamp: 0,
            ops: vec![Op::Put {
                key: "__genesis__".into(),
                value: "ok".into(),
            }],
            prev_hash: "0".into(),
            merkle_root: "GENESIS".into(),
            nonce: 0,
            hash: "GENESIS".into(),
            signature: None,
        };

        Chain {
            blocks: vec![genesis],
            keypair,
            pubkey,
            difficulty,
        }
    }

    fn add_block(&mut self, ops: Vec<Op>) {
        let prev_hash = self.blocks.last().unwrap().hash.clone();
        let block = Block::new(
            self.blocks.len() as u64,
            ops,
            prev_hash,
            &self.keypair,
            self.difficulty,
        );
        println!("✅ Mined block {} with nonce {}", block.index, block.nonce);
        self.blocks.push(block);
    }

    fn verify_chain(&self) -> bool {
        for i in 1..self.blocks.len() {
            let prev = &self.blocks[i - 1];
            let curr = &self.blocks[i];
            if !curr.verify(&prev.hash, &self.pubkey, self.difficulty) {
                return false;
            }
        }
        true
    }
}

fn main() {
    let mut chain = Chain::new(3); // Difficulty = 3 (hash must start with 000)

    chain.add_block(vec![
        Op::Put {
            key: "user".into(),
            value: "Alice".into(),
        },
        Op::Put {
            key: "role".into(),
            value: "admin".into(),
        },
    ]);

    chain.add_block(vec![Op::Del { key: "role".into() }]);

    println!("Verify chain: {}", chain.verify_chain());
}
