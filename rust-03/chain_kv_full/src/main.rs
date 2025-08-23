use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};
use chrono::Utc;
use ed25519_dalek::{SigningKey, VerifyingKey, Signature, Signer, Verifier};
use rand_core::OsRng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::HashMap,
    fs,
    io::{self, Write},
    path::Path as FsPath,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use tokio::task;

/* ---------------- Domain Types ---------------- */

#[derive(Debug, Clone, Serialize, Deserialize)]
enum Op {
    Put { key: String, value: String },
    Del { key: String },
}

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
                h.update(pair[0].as_bytes()); // duplicate last if odd
            }
            next.push(hex::encode(h.finalize()));
        }
        hashes = next;
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

    fn mine_with_progress<F: Fn(u64, &str, f64)>(
        index: u64,
        timestamp: i64,
        merkle_root: &str,
        prev_hash: &str,
        difficulty: usize,
        progress: Option<F>,
    ) -> (u64, String) {
        let target = "0".repeat(difficulty);
        let start = Instant::now();
        let mut last_report = Instant::now();
        let mut nonce = 0u64;

        loop {
            let candidate = Self::compute_hash(index, timestamp, merkle_root, prev_hash, nonce);
            if candidate.starts_with(&target) {
                // final progress report
                if let Some(ref cb) = progress {
                    let elapsed = start.elapsed().as_secs_f64();
                    let hps = (nonce as f64 + 1.0) / elapsed.max(1e-6);
                    cb(nonce, &candidate, hps);
                }
                return (nonce, candidate);
            }
            nonce = nonce.wrapping_add(1);

            if let Some(ref cb) = progress {
                if last_report.elapsed() >= Duration::from_millis(500) {
                    let elapsed = start.elapsed().as_secs_f64();
                    let hps = (nonce as f64 + 1.0) / elapsed.max(1e-6);
                    cb(nonce, &candidate, hps);
                    last_report = Instant::now();
                }
            }
        }
    }

    fn new(
        index: u64,
        ops: Vec<Op>,
        prev_hash: String,
        difficulty: usize,
        keypair: &SigningKey,
        with_progress: bool,
    ) -> Self {
        let timestamp = Utc::now().timestamp();
        let merkle_root = merkle_root(&ops);

        let (nonce, hash) = if with_progress {
            Self::mine_with_progress(
                index,
                timestamp,
                &merkle_root,
                &prev_hash,
                difficulty,
                Some(|nonce, cand: &str, hps| {
                    eprint!("\r⛏️  mining… nonce={:<12} rate={:.0} H/s last={}", nonce, hps, &cand[..8]);
                }),
            )
        } else {
            Self::mine_with_progress(index, timestamp, &merkle_root, &prev_hash, difficulty, Option::<fn(u64, &str, f64)>::None)
        };
        eprintln!();

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
        if self.prev_hash != prev_hash {
            return Err("prev_hash mismatch".into());
        }
        let recomputed = Self::compute_hash(self.index, self.timestamp, &self.merkle_root, &self.prev_hash, self.nonce);
        if recomputed != self.hash {
            return Err("hash mismatch".into());
        }
        if !self.hash.starts_with(&"0".repeat(difficulty)) {
            return Err("insufficient PoW".into());
        }
        if let (Some(sig_hex), Some(pub_hex)) = (&self.signature, &self.signer_pubkey) {
            let sig_bytes = hex::decode(sig_hex).map_err(|_| "bad signature hex")?;
            if sig_bytes.len() != 64 {
                return Err("signature must be 64 bytes".into());
            }
            let mut sig_array = [0u8; 64];
            sig_array.copy_from_slice(&sig_bytes);
            let sig = Signature::try_from(&sig_array[..]).map_err(|_| "bad signature bytes")?;
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
    // batching
    batch_active: bool,
    batch_ops: Vec<Op>,
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
        Self {
            blocks: vec![genesis],
            difficulty,
            batch_active: false,
            batch_ops: Vec::new(),
        }
    }

    fn last_hash(&self) -> String {
        self.blocks.last().map(|b| b.hash.clone()).unwrap_or_else(|| "0".into())
    }

    fn next_index(&self) -> u64 {
        self.blocks.last().map(|b| b.index + 1).unwrap_or(0)
    }

    fn append_signed(&mut self, ops: Vec<Op>, keypair: &SigningKey, with_progress: bool) {
        let blk = Block::new(self.next_index(), ops, self.last_hash(), self.difficulty, keypair, with_progress);
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

    // batching
    fn begin_batch(&mut self) -> Result<(), String> {
        if self.batch_active {
            return Err("batch already active".into());
        }
        self.batch_active = true;
        self.batch_ops.clear();
        Ok(())
    }
    fn add_put(&mut self, key: String, value: String) -> Result<(), String> {
        if !self.batch_active {
            return Err("no active batch".into());
        }
        self.batch_ops.push(Op::Put { key, value });
        Ok(())
    }
    fn add_del(&mut self, key: String) -> Result<(), String> {
        if !self.batch_active {
            return Err("no active batch".into());
        }
        self.batch_ops.push(Op::Del { key });
        Ok(())
    }
    fn abort_batch(&mut self) {
        self.batch_active = false;
        self.batch_ops.clear();
    }
    fn commit_batch(&mut self, keypair: &SigningKey, with_progress: bool) -> Result<usize, String> {
        if !self.batch_active {
            return Err("no active batch".into());
        }
        let count = self.batch_ops.len();
        let ops = std::mem::take(&mut self.batch_ops);
        self.batch_active = false;
        self.append_signed(ops, keypair, with_progress);
        Ok(count)
    }
}

/* ---------------- Key Management ---------------- */

#[derive(Serialize, Deserialize)]
struct KeyFile {
    keypair_hex: String, // 64-byte secret||public hex
    public_hex: String,  // convenience copy
}

fn keygen_to_file(path: &str) -> io::Result<()> {
    let mut csprng = OsRng;
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

/* ---------------- RPC Types ---------------- */

#[derive(Deserialize)]
struct SetReq { key: String, value: String }

#[derive(Deserialize)]
struct DelReq { key: String }

#[derive(Deserialize)]
struct DifficultyReq { n: usize }

#[derive(Serialize)]
struct VerifyResp { ok: bool, error: Option<String> }

#[derive(Clone)]
struct AppState {
    chain: Arc<Mutex<Chain>>,
    keypair: Arc<Mutex<Option<SigningKey>>>,
}

/* ---------------- RPC Server ---------------- */

async fn router(state: AppState) -> Router {
    Router::new()
        .route("/get/:key", get(http_get))
        .route("/state", get(http_state))
        .route("/verify", get(http_verify))
        .route("/set", post(http_set))
        .route("/del", post(http_del))
        .route("/begin", post(http_begin))
        .route("/addput", post(http_addput))
        .route("/adddel", post(http_adddel))
        .route("/commit", post(http_commit))
        .route("/abort", post(http_abort))
        .route("/difficulty", post(http_difficulty))
        .with_state(state)
}

async fn http_get(Path(key): Path<String>, State(state): State<AppState>) -> Json<Option<String>> {
    let chain = state.chain.lock().unwrap();
    let s = chain.materialize();
    Json(s.get(&key).cloned())
}

async fn http_state(State(state): State<AppState>) -> Json<HashMap<String, String>> {
    let chain = state.chain.lock().unwrap();
    Json(chain.materialize())
}

async fn http_verify(State(state): State<AppState>) -> Json<VerifyResp> {
    let chain = state.chain.lock().unwrap();
    match chain.verify_all() {
        Ok(_) => Json(VerifyResp { ok: true, error: None }),
        Err(e) => Json(VerifyResp { ok: false, error: Some(e) }),
    }
}

async fn http_set(State(state): State<AppState>, Json(req): Json<SetReq>) -> Json<String> {
    let maybe_kp = state.keypair.lock().unwrap().clone();
    if let Some(kp) = maybe_kp {
        // mine without chatty progress in HTTP
        let mut chain = state.chain.lock().unwrap();
        chain.append_signed(vec![Op::Put { key: req.key, value: req.value }], &kp, false);
        Json("ok".into())
    } else {
        Json("no signing key loaded".into())
    }
}

async fn http_del(State(state): State<AppState>, Json(req): Json<DelReq>) -> Json<String> {
    let maybe_kp = state.keypair.lock().unwrap().clone();
    if let Some(kp) = maybe_kp {
        let mut chain = state.chain.lock().unwrap();
        chain.append_signed(vec![Op::Del { key: req.key }], &kp, false);
        Json("ok".into())
    } else {
        Json("no signing key loaded".into())
    }
}

async fn http_begin(State(state): State<AppState>) -> Json<String> {
    let mut chain = state.chain.lock().unwrap();
    match chain.begin_batch() {
        Ok(_) => Json("batch begun".into()),
        Err(e) => Json(format!("error: {e}")),
    }
}

#[derive(Deserialize)]
struct AddPutReq { key: String, value: String }

async fn http_addput(State(state): State<AppState>, Json(req): Json<AddPutReq>) -> Json<String> {
    let mut chain = state.chain.lock().unwrap();
    match chain.add_put(req.key, req.value) {
        Ok(_) => Json("added".into()),
        Err(e) => Json(format!("error: {e}")),
    }
}

#[derive(Deserialize)]
struct AddDelReq { key: String }

async fn http_adddel(State(state): State<AppState>, Json(req): Json<AddDelReq>) -> Json<String> {
    let mut chain = state.chain.lock().unwrap();
    match chain.add_del(req.key) {
        Ok(_) => Json("added".into()),
        Err(e) => Json(format!("error: {e}")),
    }
}

async fn http_commit(State(state): State<AppState>) -> Json<String> {
    let maybe_kp = state.keypair.lock().unwrap().clone();
    if let Some(kp) = maybe_kp {
        let mut chain = state.chain.lock().unwrap();
        match chain.commit_batch(&kp, false) {
            Ok(n) => Json(format!("committed {n} ops")),
            Err(e) => Json(format!("error: {e}")),
        }
    } else {
        Json("no signing key loaded".into())
    }
}

async fn http_abort(State(state): State<AppState>) -> Json<String> {
    let mut chain = state.chain.lock().unwrap();
    chain.abort_batch();
    Json("aborted".into())
}

async fn http_difficulty(State(state): State<AppState>, Json(body): Json<DifficultyReq>) -> Json<String> {
    let mut chain = state.chain.lock().unwrap();
    if body.n == 0 || body.n > 9 {
        return Json("choose 1..9".into());
    }
    chain.difficulty = body.n;
    Json(format!("difficulty set to {}", body.n))
}

/* ---------------- CLI ---------------- */

fn prompt() -> io::Result<String> {
    print!("chain-kv> ");
    io::stdout().flush()?;
    let mut s = String::new();
    std::io::stdin().read_line(&mut s)?;
    Ok(s.trim().to_string())
}

fn print_help() {
    println!("Commands:");
    println!("  set <key> <value...>      - mine+sign single-op block (shows PoW progress)");
    println!("  del <key>                 - mine+sign single-op block");
    println!("  begin                     - begin batch");
    println!("  addput <key> <value...>   - add op to batch");
    println!("  adddel <key>              - add op to batch");
    println!("  commit                    - mine+sign a multi-op block");
    println!("  abort                     - drop current batch");
    println!("  get <key>                 - read value from materialized state");
    println!("  state                     - dump state");
    println!("  verify                    - verify PoW, signatures, and links");
    println!("  save <file>               - save chain JSON");
    println!("  load <file>               - load chain JSON");
    println!("  keygen <file>             - generate Ed25519 keypair JSON");
    println!("  loadkey <file>            - load signing key");
    println!("  whoami                    - show loaded public key");
    println!("  difficulty <n>            - set PoW difficulty (1..9)");
    println!("  serve <port>              - start Axum server on port");
    println!("  help                      - show this help");
    println!("  exit                      - quit");
}

#[tokio::main]
async fn main() {
    let chain = Arc::new(Mutex::new(Chain::genesis(3)));
    let keypair: Arc<Mutex<Option<SigningKey>>> = Arc::new(Mutex::new(None));

    println!("🔗 ChainKV — PoW + Signatures + Merkle + Batching + RPC");
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
                let kp = { keypair.lock().unwrap().clone() };
                if let Some(kp) = kp {
                    let key = parts[1].to_string();
                    let value = parts[2..].join(" ");
                    chain.lock().unwrap().append_signed(vec![Op::Put { key, value }], &kp, true);
                } else {
                    println!("❌ no signing key loaded. Use: loadkey <file>");
                }
            }
            "del" if parts.len() == 2 => {
                let kp = { keypair.lock().unwrap().clone() };
                if let Some(kp) = kp {
                    let key = parts[1].to_string();
                    chain.lock().unwrap().append_signed(vec![Op::Del { key }], &kp, true);
                } else {
                    println!("❌ no signing key loaded. Use: loadkey <file>");
                }
            }
            "begin" => match chain.lock().unwrap().begin_batch() {
                Ok(_) => println!("🧺 batch started"),
                Err(e) => println!("❌ {e}"),
            },
            "addput" if parts.len() >= 3 => {
                let key = parts[1].to_string();
                let value = parts[2..].join(" ");
                match chain.lock().unwrap().add_put(key, value) {
                    Ok(_) => println!("➕ added put"),
                    Err(e) => println!("❌ {e}"),
                }
            }
            "adddel" if parts.len() == 2 => {
                let key = parts[1].to_string();
                match chain.lock().unwrap().add_del(key) {
                    Ok(_) => println!("➖ added del"),
                    Err(e) => println!("❌ {e}"),
                }
            }
            "commit" => {
                let kp = { keypair.lock().unwrap().clone() };
                if let Some(kp) = kp {
                    match chain.lock().unwrap().commit_batch(&kp, true) {
                        Ok(n) => println!("✅ committed {n} ops"),
                        Err(e) => println!("❌ {e}"),
                    }
                } else {
                    println!("❌ no signing key loaded. Use: loadkey <file>");
                }
            }
            "abort" => {
                chain.lock().unwrap().abort_batch();
                println!("🧹 batch aborted");
            }
            "get" if parts.len() == 2 => {
                let state = chain.lock().unwrap().materialize();
                match state.get(parts[1]) {
                    Some(v) => println!("🔎 {}", v),
                    None => println!("❌ Not found"),
                }
            }
            "state" => {
                let state = chain.lock().unwrap().materialize();
                if state.is_empty() {
                    println!("(empty)");
                } else {
                    for (k, v) in state {
                        println!("{k} = {v}");
                    }
                }
            }
            "verify" => match chain.lock().unwrap().verify_all() {
                Ok(_) => println!("✅ chain ok ({} blocks, difficulty {})", chain.lock().unwrap().blocks.len(), chain.lock().unwrap().difficulty),
                Err(e) => println!("❌ verify failed: {e}"),
            },
            "save" if parts.len() == 2 => match chain.lock().unwrap().save(parts[1]) {
                Ok(_) => println!("💾 saved {}", parts[1]),
                Err(e) => println!("❌ save error: {e}"),
            },
            "load" if parts.len() == 2 => match Chain::load(parts[1]) {
                Ok(loaded) => {
                    match loaded.verify_all() {
                        Ok(_) => {
                            *chain.lock().unwrap() = loaded;
                            println!("📥 loaded chain ({} blocks) | difficulty={}", chain.lock().unwrap().blocks.len(), chain.lock().unwrap().difficulty);
                        }
                        Err(e) => println!("❌ load verify failed: {e}"),
                    }
                }
                Err(e) => println!("❌ load error: {e}"),
            },
            "keygen" if parts.len() == 2 => {
                let path = parts[1];
                if FsPath::new(path).exists() {
                    println!("⚠️ file exists; will overwrite.");
                }
                match keygen_to_file(path) {
                    Ok(_) => println!("🔐 keypair saved to {}", path),
                    Err(e) => println!("❌ keygen error: {e}"),
                }
            }
            "loadkey" if parts.len() == 2 => match load_key_from_file(parts[1]) {
                Ok(kp) => {
                    let pub_hex = hex::encode(kp.verifying_key().to_bytes());
                    *keypair.lock().unwrap() = Some(kp);
                    println!("🔓 loaded key. pubkey={}", pub_hex);
                }
                Err(e) => println!("❌ loadkey error: {e}"),
            },
            "whoami" => {
                if let Some(kp) = &*keypair.lock().unwrap() {
                    println!("🪪 pubkey={}", hex::encode(kp.verifying_key().to_bytes()));
                } else {
                    println!("(no key loaded)");
                }
            }
            "difficulty" if parts.len() == 2 => {
                match parts[1].parse::<usize>() {
                    Ok(n) if (1..=9).contains(&n) => {
                        chain.lock().unwrap().difficulty = n;
                        println!("⛏️ difficulty set to {}", n);
                    }
                    _ => println!("⚠️ choose 1..9"),
                }
            }
            "serve" if parts.len() == 2 => {
                let port = parts[1].parse::<u16>().unwrap_or(3000);
                let state = AppState {
                    chain: chain.clone(),
                    keypair: keypair.clone(),
                };
                println!("🌐 starting server on 0.0.0.0:{port}");
                // run server in background task
                task::spawn(async move {
                    let app = router(state).await;
                    let listener = tokio::net::TcpListener::bind((std::net::Ipv4Addr::new(0, 0, 0, 0), port)).await.unwrap();
                    axum::serve(listener, app).await.ok();
                });
            }
            "help" => print_help(),
            "exit" => break,
            _ => println!("⚠️ unknown command. type: help"),
        }
    }
}
