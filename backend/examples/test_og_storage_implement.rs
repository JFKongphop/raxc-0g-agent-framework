/*!
Test 0G Storage Loading - Simple test to verify exploit parsing

Flow:
  1. Load manifest.json
  2. Download and parse exploits concurrently (10 at a time)
  3. Print success/failure for each
  4. Show total loaded

Run: cargo run --example test_og_storage
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
const MANIFEST_PATH: &str = "../manifest.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ExploitMetadata {
  exploit_name: String,
  #[serde(default)]
  vuln_type: String,
  #[serde(default)]
  source: String,
  #[serde(default)]
  chain: String,
  #[serde(default)]
  date: String,
  #[serde(default)]
  total_lost: String,
  #[serde(default)]
  code_snippet: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ExploitData {
  embedding: Vec<f64>,
  metadata: ExploitMetadata,
}

fn find_bytes(haystack: &[u8], needle: &[u8]) -> Option<usize> {
  haystack.windows(needle.len()).position(|w| w == needle)
}

async fn download_and_parse(root_hash: &str, stream_id: &str, key: &str) -> Result<ExploitData> {
  // Use key name in temp path to avoid collisions in concurrent execution
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

#[tokio::main]
async fn main() -> Result<()> {
  println!("🧪 Testing 0G Storage Exploit Loading (Concurrent)\n");
  println!("CLI Path: {}", CLI_PATH);
  println!("Manifest: {}\n", MANIFEST_PATH);

  // Load manifest
  let raw = fs::read_to_string(MANIFEST_PATH)
    .context("manifest.json not found")?;
  let manifest: HashMap<String, HashMap<String, String>> =
    serde_json::from_str(&raw).context("Failed to parse manifest")?;

  let mut total_success = 0;
  let mut total_failed = 0;

  // Limit to 10 concurrent downloads
  let semaphore = Arc::new(Semaphore::new(10));

  for (stream_id, entries) in &manifest {
    println!("📂 Stream: {} ({} entries)", stream_id, entries.len());
    
    let test_limit = entries.len();
    let mut tasks = Vec::new();

    for (i, (key, root_hash)) in entries.iter().enumerate() {
      if i >= test_limit {
        break;
      }

      let sem = semaphore.clone();
      let stream_id = stream_id.clone();
      let key = key.clone();
      let root_hash = root_hash.clone();
      let idx = i + 1;

      let task = task::spawn(async move {
        let _permit = sem.acquire().await.unwrap();
        let result = download_and_parse(&root_hash, &stream_id, &key).await;
        (idx, key, result)
      });

      tasks.push(task);
    }

    // Collect results
    let mut success = 0;
    let mut failed = 0;

    for task in tasks {
      match task.await {
        Ok((idx, key, Ok(exploit))) => {
          println!("  [{}] {} ... ✅ OK (embedding_dim={})", idx, key, exploit.embedding.len());
          success += 1;
        }
        Ok((idx, key, Err(e))) => {
          println!("  [{}] {} ... ❌ FAILED: {}", idx, key, e);
          failed += 1;
        }
        Err(e) => {
          println!("  Task error: {}", e);
          failed += 1;
        }
      }
    }

    println!("  → Loaded: {}/{}\n", success, test_limit);
    total_success += success;
    total_failed += failed;
  }

  println!("{}", "=".repeat(60));
  println!("📊 SUMMARY:");
  println!("  ✅ Success: {}", total_success);
  println!("  ❌ Failed:  {}", total_failed);
  println!("  📈 Rate:    {:.1}%", 
    if (total_success + total_failed) > 0 {
      (total_success as f64 / (total_success + total_failed) as f64) * 100.0
    } else {
      0.0
    }
  );

  if total_failed == 0 {
    println!("\n🎉 All tests passed!");
  } else {
    println!("\n⚠️  Some exploits failed to load");
  }

  Ok(())
}
