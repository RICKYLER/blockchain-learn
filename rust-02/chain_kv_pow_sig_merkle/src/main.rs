use chrono::Utc;
use ed25519_dalek::{SigningKey, VerifyingKey, Signature, Signer, Verifier};
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::io::{self, Write};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
enum Op {
    Put { key: String, value: String },
    Del { key: String },
}

/* ---------------- Merkle Tree ---------------- */

fn merkle_root(ops: &[Op]) -> String {
    if ops.is_empty() {
        return "0".into();
    }
    let mut hashes: Vec<String> = ops
        .iter()
        .map(|op| {
            let mut h = Sha256::new();
            match op {
                Op::Put { key, value } => {
                    h.update(b"PUT");
                    h.update(key.as_bytes());
                    h.update(value.as_bytes());
                }
                Op::Del { key } => {
                    h.update(b"DEL");
                    h.update(key.as_bytes());
                }
            }
            hex::encode(h.finalize())
        })
        .collect();

    while hashes.len() > 1 {
        let mut next = Vec::with_capacity((hashes.len() + 1) / 2);
        for pair in hashes.chunks(2) {
            let mut h = Sha256::new();
            h.update(pair[0].as_bytes());
            if pair.len() == 2 {
                h.update(pair[1].as_bytes());
            } else {
                h.update(pair[0].as_bytes()); // duplicate if odd
            }
            next.push(hex::encode(h.finalize()));
        }
        hashes = next;
    }
    hashes[0].clone()
}

/* ---------------- Block & Chain ---------------- */

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Block {
    index: u64,
    timestamp: i64,
    ops: Vec<Op>,
    prev_hash: String,
    merkle_root: String,
    nonce: u64,
    hash: String,
    signature: Option<String>,     // hex-encoded signature over `hash`
    signer_pubkey: Option<String>, // hex-encoded 32-byte pubkey
}

impl Block {
    fn compute_hash(index: u64, timestamp: i64, merkle_root: &str, prev_hash: &str, nonce: u64) -> String {
        let mut hasher = Sha256::new();
        hasher.update(index.to_le_bytes());
        hasher.update(timestamp.to_le_bytes());
        hasher.update(merkle_root.as_bytes());
        hasher.update(prev_hash.as_bytes());
        hasher.update(nonce.to_le_bytes());
        hex::encode(hasher.finalize())
    }

    fn mine(index: u64, timestamp: i64, merkle_root: &str, prev_hash: &str, difficulty: usize) -> (u64, String) {
        let target_prefix = "0".repeat(difficulty);
        let mut nonce = 0u64;
        loop {
            let candidate = Self::compute_hash(index, timestamp, merkle_root, prev_hash, nonce);
            if candidate.starts_with(&target_prefix) {
                return (nonce, candidate);
            }
            nonce = nonce.wrapping_add(1);
        }
    }

    fn new(
        index: u64,
        ops: Vec<Op>,
        prev_hash: String,
        difficulty: usize,
        keypair: &SigningKey,
    ) -> Self {
        let timestamp = Utc::now().timestamp();
        let merkle_root = merkle_root(&ops);
        let (nonce, hash) = Self::mine(index, timestamp, &merkle_root, &prev_hash, difficulty);
        let sig = keypair.sign(hash.as_bytes());
        let sig_hex = hex::encode(sig.to_bytes());
        let pubkey_hex = hex::encode(keypair.verifying_key().to_bytes());

        Self {
            index,
            timestamp,
            ops,
            prev_hash,
            merkle_root,
            nonce,
            hash,
            signature: Some(sig_hex),
            signer_pubkey: Some(pubkey_hex),
        }
    }

    fn verify(&self, prev_hash: &str, difficulty: usize) -> Result<(), String> {
        // Link to previous
        if self.prev_hash != prev_hash {
            return Err("prev_hash mismatch".into());
        }
        // Recompute hash
        let recomputed = Self::compute_hash(self.index, self.timestamp, &self.merkle_root, &self.prev_hash, self.nonce);
        if recomputed != self.hash {
            return Err("hash mismatch".into());
        }
        // Check PoW
        if !self.hash.starts_with(&"0".repeat(difficulty)) {
            return Err("insufficient PoW".into());
        }
        // Verify signature (if present; genesis won't have one)
        if let (Some(sig_hex), Some(pub_hex)) = (&self.signature, &self.signer_pubkey) {
            let sig_bytes = hex::decode(sig_hex).map_err(|_| "bad signature hex")?;
            if sig_bytes.len() != 64 {
                return Err("signature must be 64 bytes".into());
            }
            let mut sig_array = [0u8; 64];
            sig_array.copy_from_slice(&sig_bytes);
            let sig = Signature::from_bytes(&sig_array);
            
            let pk_bytes = hex::decode(pub_hex).map_err(|_| "bad pubkey hex")?;
            if pk_bytes.len() != 32 {
                return Err("public key must be 32 bytes".into());
            }
            let mut pk_array = [0u8; 32];
            pk_array.copy_from_slice(&pk_bytes);
            let pk = VerifyingKey::from_bytes(&pk_array).map_err(|_| "bad pubkey bytes")?;
            pk.verify(self.hash.as_bytes(), &sig).map_err(|_| "signature verify failed")?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Chain {
    blocks: Vec<Block>,
    difficulty: usize,
}

impl Chain {
    fn genesis(difficulty: usize) -> Self {
        let genesis = Block {
            index: 0,
            timestamp: 0,
            ops: vec![Op::Put { key: "__genesis__".into(), value: "ok".into() }],
            prev_hash: "0".into(),
            merkle_root: "GENESIS".into(),
            nonce: 0,
            hash: "GENESIS".into(),
            signature: None,
            signer_pubkey: None,
        };
        Self { blocks: vec![genesis], difficulty }
    }

    fn last_hash(&self) -> String {
        self.blocks.last().map(|b| b.hash.clone()).unwrap_or_else(|| "0".into())
    }

    fn next_index(&self) -> u64 {
        self.blocks.last().map(|b| b.index + 1).unwrap_or(0)
    }

    fn append_signed(&mut self, ops: Vec<Op>, keypair: &SigningKey) {
        let blk = Block::new(self.next_index(), ops, self.last_hash(), self.difficulty, keypair);
        println!("✅ mined block {} (nonce {})", blk.index, blk.nonce);
        self.blocks.push(blk);
    }

    fn materialize(&self) -> HashMap<String, String> {
        let mut state = HashMap::new();
        for b in &self.blocks {
            for op in &b.ops {
                match op {
                    Op::Put { key, value } => {
                        if key != "__genesis__" {
                            state.insert(key.clone(), value.clone());
                        }
                    }
                    Op::Del { key } => {
                        state.remove(key);
                    }
                }
            }
        }
        state
    }

    fn verify_all(&self) -> Result<(), String> {
        if self.blocks.is_empty() {
            return Err("empty chain".into());
        }
        for i in 1..self.blocks.len() {
            let prev = &self.blocks[i - 1];
            let curr = &self.blocks[i];
            curr.verify(&prev.hash, self.difficulty)?;
        }
        Ok(())
    }

    fn save(&self, path: &str) -> io::Result<()> {
        let s = serde_json::to_string_pretty(self).unwrap();
        fs::write(path, s)
    }

    fn load(path: &str) -> io::Result<Self> {
        let s = fs::read_to_string(path)?;
        let c: Chain = serde_json::from_str(&s)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, format!("parse error: {e}")))?;
        Ok(c)
    }
}

/* ---------------- Key Management ---------------- */

#[derive(Serialize, Deserialize)]
struct KeyFile {
    /// 64-byte keypair (secret||public) as hex
    keypair_hex: String,
    /// 32-byte public key as hex (redundant, convenient)
    public_hex: String,
}

fn keygen_to_file(path: &str) -> io::Result<()> {
    let mut csprng = OsRng {};
    let kp = SigningKey::generate(&mut csprng);
    let keypair_hex = hex::encode(kp.to_bytes());
    let public_hex = hex::encode(kp.verifying_key().to_bytes());
    let data = KeyFile { keypair_hex, public_hex };
    let json = serde_json::to_string_pretty(&data).unwrap();
    fs::write(path, json)
}

fn load_key_from_file(path: &str) -> io::Result<SigningKey> {
    let s = fs::read_to_string(path)?;
    let kf: KeyFile = serde_json::from_str(&s)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, format!("key parse error: {e}")))?;
    let bytes = hex::decode(kf.keypair_hex)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "bad keypair hex"))?;
    if bytes.len() != 32 {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "expected 32-byte signing key"));
    }
    let mut arr = [0u8; 32];
    arr.copy_from_slice(&bytes);
    Ok(SigningKey::from_bytes(&arr))
}

/* ---------------- CLI ---------------- */

fn prompt() -> io::Result<String> {
    print!("chain-kv> ");
    io::stdout().flush()?;
    let mut s = String::new();
    io::stdin().read_line(&mut s)?;
    Ok(s.trim().to_string())
}

fn print_help() {
    println!("Commands:");
    println!("  set <key> <value...>   - add/overwrite a value (mined, signed)");
    println!("  del <key>              - delete a key (mined, signed)");
    println!("  get <key>              - read current value");
    println!("  state                  - dump all key/value pairs");
    println!("  verify                 - verify PoW, signatures, and links");
    println!("  save <file>            - save chain to JSON");
    println!("  load <file>            - load chain from JSON");
    println!("  keygen <file>          - generate & save an Ed25519 keypair");
    println!("  loadkey <file>         - load an Ed25519 keypair for signing");
    println!("  whoami                 - show loaded public key (if any)");
    println!("  difficulty <n>         - set PoW difficulty (current session)");
    println!("  help                   - show this help");
    println!("  exit                   - quit");
}

fn main() {
    let mut chain = Chain::genesis(3); // default difficulty: 3 leading zeros
    let mut current_keypair: Option<SigningKey> = None;

    println!("🔗 ChainKV — PoW + Signatures + Merkle");
    print_help();
    println!();

    loop {
        let line = match prompt() {
            Ok(s) => s,
            Err(_) => break,
        };
        if line.is_empty() {
            continue;
        }
        let parts: Vec<&str> = line.split_whitespace().collect();
        match parts[0] {
            "set" if parts.len() >= 3 => {
                if let Some(kp) = &current_keypair {
                    let key = parts[1].to_string();
                    let value = parts[2..].join(" ");
                    chain.append_signed(vec![Op::Put { key, value }], kp);
                } else {
                    println!("❌ no signing key loaded. Use: loadkey <file> (or keygen <file> then loadkey)");
                }
            }
            "del" if parts.len() == 2 => {
                if let Some(kp) = &current_keypair {
                    let key = parts[1].to_string();
                    chain.append_signed(vec![Op::Del { key }], kp);
                } else {
                    println!("❌ no signing key loaded. Use: loadkey <file>");
                }
            }
            "get" if parts.len() == 2 => {
                let state = chain.materialize();
                match state.get(parts[1]) {
                    Some(v) => println!("🔎 {}", v),
                    None => println!("❌ Not found"),
                }
            }
            "state" => {
                let state = chain.materialize();
                if state.is_empty() {
                    println!("(empty)");
                } else {
                    for (k, v) in state {
                        println!("{k} = {v}");
                    }
                }
            }
            "verify" => match chain.verify_all() {
                Ok(_) => println!("✅ chain ok ({} blocks, difficulty {})", chain.blocks.len(), chain.difficulty),
                Err(e) => println!("❌ verify failed: {e}"),
            },
            "save" if parts.len() == 2 => match chain.save(parts[1]) {
                Ok(_) => println!("💾 saved chain to {}", parts[1]),
                Err(e) => println!("❌ save error: {e}"),
            },
            "load" if parts.len() == 2 => match Chain::load(parts[1]) {
                Ok(loaded) => {
                    match loaded.verify_all() {
                        Ok(_) => {
                            chain = loaded;
                            println!("📥 loaded chain ({} blocks) | difficulty={}", chain.blocks.len(), chain.difficulty);
                        }
                        Err(e) => println!("❌ load verify failed: {e}"),
                    }
                }
                Err(e) => println!("❌ load error: {e}"),
            },
            "keygen" if parts.len() == 2 => {
                let path = parts[1];
                if Path::new(path).exists() {
                    println!("⚠️ file exists; will overwrite.");
                }
                match keygen_to_file(path) {
                    Ok(_) => println!("🔐 keypair generated & saved to {}", path),
                    Err(e) => println!("❌ keygen error: {e}"),
                }
            }
            "loadkey" if parts.len() == 2 => match load_key_from_file(parts[1]) {
                Ok(kp) => {
                    let pub_hex = hex::encode(kp.verifying_key().to_bytes());
                    current_keypair = Some(kp);
                    println!("🔓 loaded key. pubkey={}", pub_hex);
                }
                Err(e) => println!("❌ loadkey error: {e}"),
            },
            "whoami" => {
                if let Some(kp) = &current_keypair {
                    println!("🪪 pubkey={}", hex::encode(kp.verifying_key().to_bytes()));
                } else {
                    println!("(no key loaded)");
                }
            }
            "difficulty" if parts.len() == 2 => {
                match parts[1].parse::<usize>() {
                    Ok(n) if n > 0 && n < 10 => {
                        chain.difficulty = n;
                        println!("⛏️ difficulty set to {}", n);
                    }
                    _ => println!("⚠️ choose 1..9"),
                }
            }
            "help" => print_help(),
            "exit" => break,
            _ => println!("⚠️ unknown command. type: help"),
        }
    }
}
