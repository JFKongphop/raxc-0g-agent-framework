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

const CLI_PATH: &str = "../0g-cli";
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

  /// Store data to 0G Storage (for persistent memory)
  /// 
  /// Note: This is a mock implementation for hackathon demo.
  /// In production, this would use 0g-cli upload command or SDK.
  pub async fn put(&self, key: &str, value: &str) -> Result<()> {
    // For hackathon: store locally as demonstration
    // In production: upload to 0G Storage via 0g-cli or SDK
    
    let storage_dir = std::path::Path::new("/tmp/raxc_memory");
    std::fs::create_dir_all(storage_dir)?;
    
    let file_path = storage_dir.join(format!("{}.json", key.replace(":", "_")));
    std::fs::write(file_path, value)?;
    
    Ok(())
  }

  /// Search for similar past analyses from stored memory
  /// 
  /// Note: This is a mock implementation for hackathon demo.
  /// In production, this would query 0G Storage for similar contract analyses.
  pub async fn search_analyses(&self, _contract: &str) -> Result<Vec<String>> {
    // For hackathon: read from local storage
    // In production: query 0G Storage with semantic search
    
    let storage_dir = std::path::Path::new("/tmp/raxc_memory");
    
    if !storage_dir.exists() {
      return Ok(vec![]);
    }

    let mut analyses = Vec::new();
    
    for entry in std::fs::read_dir(storage_dir)? {
      let entry = entry?;
      let path = entry.path();
      
      if path.extension().and_then(|s| s.to_str()) == Some("json") {
        if let Ok(content) = std::fs::read_to_string(&path) {
          // Parse and extract key info for context
          if let Ok(output) = serde_json::from_str::<serde_json::Value>(&content) {
            let summary = format!(
              "Past Analysis: {} - {} (Confidence: {}%)",
              output.get("vulnerability_type").and_then(|v| v.as_str()).unwrap_or("Unknown"),
              output.get("risk_level").and_then(|v| v.as_str()).unwrap_or("Unknown"),
              output.get("confidence").and_then(|v| v.as_u64()).unwrap_or(0)
            );
            analyses.push(summary);
          }
        }
      }
      
      // Limit to top 3 past analyses
      if analyses.len() >= 3 {
        break;
      }
    }
    
    Ok(analyses)
  }
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
