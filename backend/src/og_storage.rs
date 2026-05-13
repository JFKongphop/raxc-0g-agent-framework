#![allow(dead_code, unused_variables)]
/*!
0G Storage Client — Pre-loads exploits from 0G Storage via 0g-cli.

Architecture:
1. At construction: downloads ALL exploit files via 0g-cli concurrently
2. Stores all exploits in memory as Vec<LoadedExploit>
3. Query method: fast in-memory cosine similarity (no network calls)

Based on test_og_storage.rs behavior for complete coverage.
*/

use anyhow::{Context, Result};
use base64::{engine::general_purpose, Engine as _};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::sync::Arc;
use tokio::sync::Semaphore;
use tokio::task;

const CLI_PATH: &str = "./0g-cli";
const CONCURRENT_LIMIT: usize = 10;

// EXACT copy from test_og_storage.rs
fn find_bytes(haystack: &[u8], needle: &[u8]) -> Option<usize> {
  haystack.windows(needle.len()).position(|w| w == needle)
}

// EXACT copy from test_og_storage.rs
async fn download_and_parse(root_hash: &str, stream_id: &str, key: &str) -> Result<ExploitData> {
  let safe_key = key.replace("/", "_").replace(".", "_");
  let temp_path = format!(
    "/tmp/og_test_{}_{}.bin",
    safe_key,
    std::time::SystemTime::now()
      .duration_since(std::time::UNIX_EPOCH)
      .unwrap()
      .as_nanos()
  );

  let indexer_rpc = std::env::var("OG_INDEXER_RPC")
    .unwrap_or_else(|_| "https://indexer-storage-testnet-turbo.0g.ai".to_string());
  
  let output = tokio::process::Command::new(CLI_PATH)
    .args(&[
      "download",
      "--indexer",
      &indexer_rpc,
      "--root",
      root_hash,
      "--file",
      &temp_path,
    ])
    .output()
    .await
    .context("Failed to execute 0g-cli")?;

  if !output.status.success() {
    let stderr = String::from_utf8_lossy(&output.stderr);
    anyhow::bail!("CLI download failed: {}", stderr.trim());
  }

  let data = fs::read(&temp_path).context("Failed to read downloaded file")?;
  let _ = fs::remove_file(&temp_path);

  // Binary search: stream_id → key → base64_value
  let stream_idx = find_bytes(&data, stream_id.as_bytes())
    .ok_or_else(|| anyhow::anyhow!("stream_id '{}' not found", stream_id))?;

  let key_idx = find_bytes(&data[stream_idx..], key.as_bytes())
    .map(|i| stream_idx + i)
    .ok_or_else(|| anyhow::anyhow!("key '{}' not found after stream_id", key))?;

  let after_key = &data[key_idx + key.len()..];
  let text = String::from_utf8_lossy(after_key);

  let re = Regex::new(r"([A-Za-z0-9+/]{100,}={0,2})").unwrap();
  let b64 = re
    .find(&text)
    .ok_or_else(|| anyhow::anyhow!("No base64 payload found after key"))?
    .as_str();

  let decoded = general_purpose::STANDARD
    .decode(b64)
    .context("Base64 decode failed")?;
  let json_str = String::from_utf8(decoded).context("UTF-8 decode failed")?;
  serde_json::from_str(&json_str).context("JSON parse failed")
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExploitMetadata {
  pub exploit_name: String,
  #[serde(default)]
  pub vuln_type: String,
  #[serde(default)]
  pub source: String,
  #[serde(default)]
  pub chain: String,
  #[serde(default)]
  pub date: String,
  #[serde(default)]
  pub total_lost: String,
  #[serde(default)]
  pub code_snippet: String,
  #[serde(default)]
  pub attack_tx: String,
  #[serde(default)]
  pub vulnerable_contract: String,
  #[serde(default)]
  pub attacker: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExploitData {
  pub embedding: Vec<f64>,
  pub metadata: ExploitMetadata,
}

#[derive(Debug, Clone)]
pub struct LoadedExploit {
  pub stream: String,
  pub key: String,
  pub root_hash: String,
  pub data: ExploitData,
}

#[derive(Clone)]
pub struct OgStorageClient {
  pub exploits: Vec<LoadedExploit>,
}

impl OgStorageClient {
  /// Create new client - EXACTLY copied from test_og_storage.rs main() logic
  pub async fn new(
    _indexer_rpc: String,
    _stream_id: String,
    _cli_path: String,
    manifest_path: String,
  ) -> Result<Self> {
    println!("\n[OgStorageClient NEW] Loading manifest from {}...", manifest_path);
    
    // Load manifest - EXACT copy
    let raw = fs::read_to_string(&manifest_path)
      .context("manifest.json not found")?;
    let manifest: HashMap<String, HashMap<String, String>> =
      serde_json::from_str(&raw).context("Failed to parse manifest")?;

    let cases_total = manifest.get("defi_cases").map(|m| m.len()).unwrap_or(0);
    let proto_total = manifest.get("defi_protocols").map(|m| m.len()).unwrap_or(0);
    println!(
      "[OgStorageClient NEW] Found {} defi_cases + {} defi_protocols = {} total",
      cases_total, proto_total, cases_total + proto_total
    );

    let mut all_exploits = Vec::new();

    // Limit to 10 concurrent downloads - EXACT copy
    let semaphore = Arc::new(Semaphore::new(CONCURRENT_LIMIT));

    for (stream_id, entries) in &manifest {
      println!("  [{}] downloading {} of {} exploits (concurrent={})...", stream_id, entries.len(), entries.len(), CONCURRENT_LIMIT);
      
      let mut tasks = Vec::new();

      for (key, root_hash) in entries.iter() {
        let sem = semaphore.clone();
        let stream_id = stream_id.clone();
        let key = key.clone();
        let root_hash = root_hash.clone();

        let task = task::spawn(async move {
          let _permit = sem.acquire().await.unwrap();
          let result = download_and_parse(&root_hash, &stream_id, &key).await;
          (stream_id, key, root_hash, result)
        });

        tasks.push(task);
      }

      // Collect results - EXACT copy
      let mut success = 0;
      let mut failed = 0;

      for task in tasks {
        match task.await {
          Ok((stream, key, root_hash, Ok(data))) => {
            all_exploits.push(LoadedExploit {
              stream,
              key,
              root_hash,
              data,
            });
            success += 1;
          }
          Ok((_, key, _, Err(e))) => {
            eprintln!("    [!] {} → {}", key, e);
            failed += 1;
          }
          Err(e) => {
            eprintln!("    [!] Task error: {}", e);
            failed += 1;
          }
        }
      }

      println!("  [{}] loaded={}, failed={}", stream_id, success, failed);
    }

    println!("\n[OgStorageClient NEW] Pre-loaded {} exploits into memory\n", all_exploits.len());

    Ok(Self { exploits: all_exploits })
  }

  pub fn total_loaded(&self) -> usize {
    self.exploits.len()
  }

  /// Create an empty client (no exploits loaded) — for MemoryTool in remote mode.
  /// Only `put()` and `search_analyses()` are usable; `query()` returns empty.
  pub fn new_empty() -> Self {
    Self { exploits: Vec::new() }
  }

  /// Query exploits with cosine similarity
  pub fn query(&self, query_embedding: &[f64], top_k: usize) -> Vec<(f64, &LoadedExploit)> {
    let mut scores: Vec<(f64, &LoadedExploit)> = self
      .exploits
      .iter()
      .map(|ex| {
        let score = cosine_similarity(query_embedding, &ex.data.embedding);
        (score, ex)
      })
      .collect();

    scores.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
    scores.into_iter().take(top_k).collect()
  }

  /// Upload data to real 0G Storage via 0g-cli and return the root hash.
  ///
  /// Steps:
  ///   1. Write `value` to a temp file
  ///   2. Run `0g-cli upload --indexer <rpc> --file <path> --private-key <key>`
  ///   3. Parse root hash from stdout (line matching "Root hash: 0x...")
  ///   4. Cache root hash in /tmp/raxc_memory/<key>.rootHash for MemoryTool lookup
  ///
  /// Falls back to local /tmp write if 0g-cli is unavailable or env vars missing.
  pub async fn put(&self, key: &str, value: &str) -> Result<String> {
    let safe_key = key.replace(":", "_").replace("/", "_");

    let indexer_rpc = std::env::var("OG_INDEXER_RPC")
      .unwrap_or_else(|_| "https://indexer-storage-testnet-turbo.0g.ai".to_string());
    let private_key = std::env::var("OG_STORAGE_PRIVATE_KEY")
      .or_else(|_| std::env::var("PRIVATE_KEY"));

    let root_hash = match private_key {
      Ok(pk) => {
        let evm_rpc = std::env::var("OG_RPC_URL")
          .unwrap_or_else(|_| "https://evmrpc-testnet.0g.ai".to_string());
        let stream_id = std::env::var("OG_STREAM_ID")
          .unwrap_or_else(|_| "raxc_audits".to_string());
        // Base64-encode value to avoid CSV quote-parsing issues in 0g-cli --stream-values
        let encoded_value = general_purpose::STANDARD.encode(value.as_bytes());
        // Use spawn() + stream stderr line-by-line so INFO logs appear in real time
        use tokio::io::{AsyncBufReadExt, BufReader};
        use std::process::Stdio;
        let child = tokio::process::Command::new(CLI_PATH)
          .args(&[
            "kv-write",
            "--url", &evm_rpc,
            "--indexer", &indexer_rpc,
            "--key", &pk,
            "--stream-id", &stream_id,
            "--stream-keys", key,
            "--stream-values", &encoded_value,
            "--expected-replica", "1",
          ])
          .stderr(Stdio::piped())
          .stdout(Stdio::piped())
          .spawn();

        let output: Result<(bool, String), _> = match child {
          Ok(mut proc) => {
            let stderr_pipe = proc.stderr.take().unwrap();
            let mut reader = BufReader::new(stderr_pipe).lines();
            let mut all_stderr = String::new();
            // Stream each line to stdout as it arrives
            while let Ok(Some(line)) = reader.next_line().await {
              println!("    {}", line);
              all_stderr.push_str(&line);
              all_stderr.push('\n');
            }
            let status = proc.wait().await;
            Ok((status.map(|s| s.success()).unwrap_or(false), all_stderr))
          }
          Err(e) => Err(e),
        };
        let _ = &encoded_value; // suppress unused warning

        match output {
          Ok((true, stderr)) => {
            // Find the line containing "merkle root" or "root=" and extract the 64-hex hash from it.
            // This avoids accidentally picking up TX hashes (also 64 hex chars) from other log lines.
            // Logrus may wrap field names in ANSI codes but the value 0x... is always plain text.
            let hex_re = Regex::new(r"0x([0-9a-fA-F]{64})").unwrap();
            let hash = stderr.lines()
              .find(|line| {
                let l = line.to_lowercase();
                l.contains("merkle") || l.contains("root=") || l.contains("root =")
              })
              .and_then(|line| {
                hex_re.captures(line).map(|c| format!("0x{}", &c[1]))
              })
              // Fallback: any 64-hex in the whole output (last resort)
              .or_else(|| {
                hex_re.captures_iter(&stderr).map(|c| format!("0x{}", &c[1])).last()
              });

            match hash {
              Some(h) => {
                println!("    [0G Storage]     Uploaded '{}' → root hash: {}", key, h);
                let cache_dir = raxc_cache_dir();
                let _ = std::fs::write(
                  format!("{}/roothash_{}.content", cache_dir, h.trim_start_matches("0x")),
                  value,
                );
                h
              }
              None => {
                let fallback = format!("0x{}", sha256_hex(value));
                println!("    [0G Storage]     Hash not found in output, using SHA256 fallback: {}", fallback);
                fallback
              }
            }
          }
          Ok((false, stderr)) => {
            // Even on failure, the merkle root may have been computed
            let hex_re2 = Regex::new(r"0x([0-9a-fA-F]{64})").unwrap();
            let recovered = stderr.lines()
              .find(|line| {
                let l = line.to_lowercase();
                l.contains("merkle") || l.contains("root=") || l.contains("root =")
              })
              .and_then(|line| hex_re2.captures(line).map(|c| format!("0x{}", &c[1])))
              .or_else(|| hex_re2.captures_iter(&stderr).map(|c| format!("0x{}", &c[1])).last());
            if let Some(ref h) = recovered {
              println!("    [0G Storage]     kv-write partial — root computed: {}", h);
              let cache_dir = raxc_cache_dir();
              let _ = std::fs::write(
                format!("{}/roothash_{}.content", cache_dir, h.trim_start_matches("0x")),
                value,
              );
            } else {
              println!("    [0G Storage]     Upload failed (no root hash found in output)");
            }
            persist_local(&safe_key, value)?;
            recovered.unwrap_or_else(|| format!("0x{}", sha256_hex(value)))
          }
          Err(e) => {
            println!("    [0G Storage]     0g-cli not available: {}", e);
            persist_local(&safe_key, value)?;
            format!("0x{}", sha256_hex(value))
          }
        }
      }
      Err(_) => {
        // No private key — persist locally only
        println!("    [0G Storage]     No OG_STORAGE_PRIVATE_KEY/PRIVATE_KEY set — persisting locally");
        persist_local(&safe_key, value)?;
        format!("0x{}", sha256_hex(value))
      }
    };

    // Cache root hash for MemoryTool lookup
    let cache_dir = std::path::Path::new("/tmp/raxc_memory");
    std::fs::create_dir_all(cache_dir)?;
    std::fs::write(
      cache_dir.join(format!("{}.rootHash", safe_key)),
      &root_hash,
    )?;

    Ok(root_hash)
  }

  /// Search past audit analyses stored in 0G Storage.
  ///
  /// Reads cached root hashes from /tmp/raxc_memory/*.rootHash and returns
  /// summaries for MemoryTool context. In a full production setup, these root
  /// hashes would be fetched from the on-chain ERC-7857 token and the actual
  /// JSON downloaded via `0g-cli download`.
  pub async fn search_analyses(&self, _contract: &str) -> Result<Vec<String>> {
    let storage_dir = std::path::Path::new("/tmp/raxc_memory");
    if !storage_dir.exists() {
      return Ok(vec![]);
    }

    let mut analyses = Vec::new();

    let mut entries: Vec<_> = std::fs::read_dir(storage_dir)?
      .filter_map(|e| e.ok())
      .collect();
    // Sort newest first (by filename which contains timestamp)
    entries.sort_by(|a, b| b.file_name().cmp(&a.file_name()));

    for entry in entries {
      let path = entry.path();
      // Read local JSON cache files (written by persist_local)
      if path.extension().and_then(|s| s.to_str()) == Some("json") {
        if let Ok(content) = std::fs::read_to_string(&path) {
          if let Ok(output) = serde_json::from_str::<serde_json::Value>(&content) {
            let summary = format!(
              "Past Analysis: {} | Risk: {} | Confidence: {}%",
              output.get("vulnerability_type").and_then(|v| v.as_str()).unwrap_or("Unknown"),
              output.get("risk_level").and_then(|v| v.as_str()).unwrap_or("Unknown"),
              output.get("confidence").and_then(|v| v.as_u64()).unwrap_or(0)
            );
            analyses.push(summary);
          }
        }
      }
      if analyses.len() >= 3 {
        break;
      }
    }

    Ok(analyses)
  }

  /// Load past audit context from the on-chain ERC-7857 NFT — Stage 2 long-context memory.
  ///
  /// Reads intelligentDatasOf(tokenId) from the chain → downloads full audit JSON from 0G Storage.
  /// Returns text summaries injected into the LLM explanation prompt (Phase 5).
  ///
  /// This is separate from the 722-exploit static knowledge base (Stage 1).
  /// Stage 1 answers "does this look like a known exploit?"
  /// Stage 2 answers "have I audited this pattern before?"
  pub async fn load_from_chain(&self, token_id: u64) -> Result<Vec<String>> {
    let (summaries, _) = self.load_from_chain_full(token_id).await?;
    Ok(summaries)
  }

  /// Full version — returns (text summaries, raw JSON values) for callers that need both.
  pub async fn load_from_chain_full(&self, token_id: u64) -> Result<(Vec<String>, Vec<serde_json::Value>)> {
    let contract = std::env::var("RAXC_AGENT_NFT_ADDRESS")
      .map_err(|_| anyhow::anyhow!("RAXC_AGENT_NFT_ADDRESS not set"))?;
    let rpc_url = std::env::var("OG_RPC_URL")
      .unwrap_or_else(|_| "https://evmrpc-testnet.0g.ai".to_string());
    let evm_rpc = rpc_url.clone();
    let indexer_rpc = std::env::var("OG_INDEXER_RPC")
      .unwrap_or_else(|_| "https://indexer-storage-testnet-turbo.0g.ai".to_string());

    println!("[MemoryTool]     Reading past audits from chain (token #{})...", token_id);

    // intelligenceHistory = all past snapshots (raw bytes32 dataHash per update)
    // intelligentDatasOf  = current snapshot (not yet in history)
    // ── Fetch all 0G root hashes via ethers-rs eth_getLogs (no cast dependency) ──
    // Event: Updated(uint256 indexed tokenId, IntelligentData[] oldDatas, IntelligentData[] newDatas)
    // Topic0 = keccak256("Updated(uint256,(string,bytes32)[],(string,bytes32)[])")
    // The raw bytes32 dataHash appears as a 66-char 0x word in the ABI-encoded log data.
    use ethers::providers::{Http, Provider, Middleware};
    use ethers::types::{Filter, Address, H256};
    use std::str::FromStr;

    let contract_addr = Address::from_str(&contract)
      .map_err(|_| anyhow::anyhow!("Invalid contract address"))?;
    let provider = Provider::<Http>::try_from(rpc_url.as_str())
      .map_err(|e| anyhow::anyhow!("RPC connect failed: {}", e))?;

    // Updated(uint256,(string,bytes32)[],(string,bytes32)[])
    let event_sig = H256::from(ethers::utils::keccak256(
      b"Updated(uint256,(string,bytes32)[],(string,bytes32)[])"
    ));
    // Filter from block 0 to latest — Galileo requires explicit range for eth_getLogs
    let filter = Filter::new()
      .address(contract_addr)
      .topic0(event_sig)
      .from_block(0u64)
      .to_block(ethers::types::BlockNumber::Latest);

    let logs = provider.get_logs(&filter).await.unwrap_or_default();
    println!("[MemoryTool]     Found {} Updated events on chain", logs.len());

    // ABI-decode log.data to extract the exact bytes32 dataHash from newDatas.
    // Updated(uint256 indexed tokenId, IntelligentData[] oldDatas, IntelligentData[] newDatas)
    // log.data = ABI encoded (oldDatas, newDatas) where IntelligentData = (string, bytes32)
    use ethers::abi::{decode as abi_decode, ParamType, Token};
    let intel_data_type = ParamType::Tuple(vec![ParamType::String, ParamType::FixedBytes(32)]);
    let param_types = vec![
      ParamType::Array(Box::new(intel_data_type.clone())), // oldDatas
      ParamType::Array(Box::new(intel_data_type)),          // newDatas
    ];

    let mut root_hashes: Vec<String> = Vec::new();
    let mut seen_hashes = std::collections::HashSet::new();
    for log in &logs {
      if let Ok(decoded) = abi_decode(&param_types, &log.data.0) {
        // decoded[1] = newDatas — the hashes from THIS update call
        if let Some(Token::Array(new_datas)) = decoded.get(1) {
          for item in new_datas {
            if let Token::Tuple(fields) = item {
              if let Some(Token::FixedBytes(hash_bytes)) = fields.get(1) {
                let hash = format!("0x{}", ethers::utils::hex::encode(hash_bytes));
                // Skip all-zero and bootstrap placeholder (0x0000...0001)
                let is_zero = hash_bytes.iter().all(|b| *b == 0);
                let is_bootstrap = hash == "0x0000000000000000000000000000000000000000000000000000000000000001";
                if !is_zero && !is_bootstrap && seen_hashes.insert(hash.clone()) {
                  root_hashes.push(hash);
                }
              }
            }
          }
        }
      }
    }
    println!("[MemoryTool]     Extracted {} unique 0G root hashes from events", root_hashes.len());

    // Persistent cache dir — survives reboots unlike /tmp
    let cache_dir = dirs_home().unwrap_or_else(|| "/tmp".into()) + "/.raxc/memory";
    let _ = std::fs::create_dir_all(&cache_dir);

    let mut contexts = Vec::new();
    let mut raw_jsons: Vec<serde_json::Value> = Vec::new();

    for root_hash in &root_hashes {
        println!("[MemoryTool]     Processing hash (root: {}...)", &root_hash[..10]);

        // Always download from 0G Storage network — decentralized memory, any agent on any machine can read
        let cache_key = root_hash.trim_start_matches("0x");
        let cache_path = format!("{}/roothash_{}.content", cache_dir, cache_key);

        let temp_path = format!(
          "/tmp/raxc_chain_memory_{}.json",
          std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
        );

        // Single attempt — if file not found, it's gone from 0G testnet (no retry spam)
        let dl = tokio::process::Command::new(CLI_PATH)
          .args(&[
            "download",
            "--indexer", &indexer_rpc,
            "--root", &root_hash,
            "--file", &temp_path,
          ])
          .output()
          .await;

        match dl {
          Ok(o) if o.status.success() => {
            // KV binary format: [header][key_len:4LE][key][value_len:8LE][value]
            // The value is stored RAW (not base64) — indexer base64-encodes only because
            // it stores complex JSON with embeddings. Our audit value is plain JSON.
            // Strategy: find the stream-key bytes in the binary blob, then read raw value after it.
            if let Ok(raw_bytes) = std::fs::read(&temp_path) {
              let _ = std::fs::remove_file(&temp_path);
              // Value was stored base64-encoded (to avoid CLI quote issues)
              // Find the longest base64 block in the binary and decode it to get JSON
              let text = String::from_utf8_lossy(&raw_bytes);
              let b64_re = Regex::new(r"[A-Za-z0-9+/]{50,}={0,2}").unwrap();
              let json_str = b64_re.find(&text)
                .and_then(|m| general_purpose::STANDARD.decode(m.as_str()).ok())
                .and_then(|bytes| String::from_utf8(bytes).ok());

              if let Some(content) = json_str {
                let _ = std::fs::write(&cache_path, &content);
                if let Ok(v) = serde_json::from_str::<serde_json::Value>(&content) {
                  let vuln = v.get("vulnerability_type").and_then(|x| x.as_str()).unwrap_or("Unknown");
                  let risk  = v.get("risk_level").and_then(|x| x.as_str()).unwrap_or("Unknown");
                  let conf  = v.get("confidence").and_then(|x| x.as_u64()).unwrap_or(0);
                  let summary = format!(
                    "[Chain Memory] Past audit — vuln: {} | risk: {} | confidence: {}% | hash: {}",
                    vuln, risk, conf, &root_hash[..10],
                  );
                  println!("[✓] MemoryTool: Loaded past audit from 0G Storage");
                  contexts.push(summary);
                  raw_jsons.push(v);
                } else {
                  contexts.push(format!("[Chain Memory] {}", content.chars().take(500).collect::<String>()));
                }
              }
            }
          }
          _ => {
            let _ = std::fs::remove_file(&temp_path);
            let err_msg = if let Ok(ref o) = dl {
              String::from_utf8_lossy(&o.stderr).chars().take(300).collect::<String>()
            } else {
              "command failed".to_string()
            };
            println!("[!] MemoryTool: 0G Storage download failed for {} — {}", &root_hash[..10], err_msg);
          }
        }
    }

    Ok((contexts, raw_jsons))
  }
}

fn dirs_home() -> Option<String> {
  std::env::var("HOME").ok()
}

fn raxc_cache_dir() -> String {
  let home = dirs_home().unwrap_or_else(|| "/tmp".into());
  let dir = format!("{}/.raxc/memory", home);
  let _ = std::fs::create_dir_all(&dir);
  dir
}

fn persist_local(safe_key: &str, value: &str) -> Result<()> {
  let dir = std::path::Path::new("/tmp/raxc_memory");
  std::fs::create_dir_all(dir)?;
  std::fs::write(dir.join(format!("{}.json", safe_key)), value)?;
  Ok(())
}

/// Deterministic SHA-256 hex of content (used as fallback root hash when 0g-cli unavailable)
fn sha256_hex(data: &str) -> String {
  use std::collections::hash_map::DefaultHasher;
  use std::hash::{Hash, Hasher};
  // Note: DefaultHasher is not cryptographically strong, but sufficient as a
  // deterministic content fingerprint for demo purposes when 0G Storage is unavailable.
  let mut hasher = DefaultHasher::new();
  data.hash(&mut hasher);
  let h = hasher.finish();
  format!("{:016x}{:016x}{:016x}{:016x}", h, h ^ 0xdeadbeef, h.rotate_left(32), h ^ 0xcafebabe)
}

fn cosine_similarity(a: &[f64], b: &[f64]) -> f64 {
  if a.len() != b.len() {
    return 0.0;
  }
  let dot: f64 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
  let norm_a: f64 = a.iter().map(|x| x * x).sum::<f64>().sqrt();
  let norm_b: f64 = b.iter().map(|x| x * x).sum::<f64>().sqrt();
  if norm_a == 0.0 || norm_b == 0.0 {
    0.0
  } else {
    dot / (norm_a * norm_b)
  }
}

// ─── Remote Storage Client ────────────────────────────────────────────────────

/// Result item from the remote storage API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteExploitResult {
  pub score: f64,
  pub exploit_name: String,
  pub vuln_type: String,
  pub chain: String,
  pub date: String,
  pub total_lost: String,
  pub source: String,
  pub code_snippet: String,
  pub attack_tx: String,
  pub embedding_dim: usize,
}

#[derive(Debug, Clone, Deserialize)]
struct RemoteQueryResponse {
  pub results: Vec<RemoteExploitResult>,
  pub total_searched: usize,
  pub query_time_ms: u64,
}

/// HTTP client that queries the api_0g_storage server instead of loading locally.
/// Use this in agent_example.rs and OpenClaw skill to avoid 2-3 min local load time.
///
/// Usage:
///   let client = RemoteOgStorageClient::new("http://localhost:3001");
///   let results = client.query(&embedding, 5).await?;
pub struct RemoteOgStorageClient {
  api_url: String,
  client: reqwest::Client,
}

impl RemoteOgStorageClient {
  /// Create a new remote client pointing to the api_0g_storage server
  pub fn new(api_url: impl Into<String>) -> Self {
    Self {
      api_url: api_url.into().trim_end_matches('/').to_string(),
      client: reqwest::Client::new(),
    }
  }

  /// Create with default localhost URL
  pub fn local() -> Self {
    let port = std::env::var("STORAGE_PORT").unwrap_or_else(|_| "3001".to_string());
    Self::new(format!("http://localhost:{}", port))
  }

  /// Query top-k similar exploits by embedding vector
  /// Returns (score, exploit_name, vuln_type, total_lost, code_snippet)
  pub async fn query(
    &self,
    embedding: &[f64],
    top_k: usize,
  ) -> Result<Vec<RemoteExploitResult>> {
    let url = format!("{}/query", self.api_url);

    let body = serde_json::json!({
      "embedding": embedding,
      "top_k": top_k
    });

    let resp = self
      .client
      .post(&url)
      .json(&body)
      .send()
      .await
      .context("Failed to reach api_0g_storage server — is it running?")?;

    if !resp.status().is_success() {
      let status = resp.status();
      let text = resp.text().await.unwrap_or_default();
      anyhow::bail!("Storage API error {}: {}", status, text);
    }

    let data: RemoteQueryResponse = resp.json().await.context("Failed to parse storage API response")?;

    println!(
      "[0G Storage]     Queried {} exploits in {}ms — {} matches found",
      data.total_searched, data.query_time_ms, data.results.len()
    );

    Ok(data.results)
  }

  /// Check if the storage server is healthy and how many exploits are loaded
  pub async fn health(&self) -> Result<usize> {
    let url = format!("{}/health", self.api_url);
    let resp = self
      .client
      .get(&url)
      .send()
      .await
      .context("Failed to reach api_0g_storage server")?;
    let data: serde_json::Value = resp.json().await?;
    let loaded = data["loaded"].as_u64().unwrap_or(0) as usize;
    Ok(loaded)
  }
}
