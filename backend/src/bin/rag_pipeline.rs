/*!
RAG Pipeline — manifest → 0G Storage download → cosine similarity → 0G Compute

Flow:
  1. Load manifest.json  (stream → { key → rootHash })
  2. Download every file via `0g-cli download`
  3. Parse binary KV format → ExploitData { embedding, metadata }
  4. Cosine similarity search against all in-memory embeddings
  5. Augment prompt with top-K results → 0G Compute LLM inference

Run: cargo run --example rag_pipeline
*/

use anyhow::{Context, Result};
use base64::{engine::general_purpose, Engine as _};
use regex::Regex;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::process::Command;

// ─── Config ───────────────────────────────────────────────────────────────────

const CLI_PATH: &str = "../0g-cli";
const MANIFEST_PATH: &str = "../manifest.json";
const TOP_K: usize = 5;

// ─── Data types ───────────────────────────────────────────────────────────────

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

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct LoadedExploit {
  key: String,
  data: ExploitData,
}

// ─── 0G CLI Download + Binary Parse ──────────────────────────────────────────

fn find_bytes(haystack: &[u8], needle: &[u8]) -> Option<usize> {
  haystack.windows(needle.len()).position(|w| w == needle)
}

fn download_and_parse(root_hash: &str, stream_id: &str, key: &str) -> Result<ExploitData> {
  let temp_path = format!(
    "/tmp/og_rag_{}.bin",
    std::time::SystemTime::now()
      .duration_since(std::time::UNIX_EPOCH)
      .unwrap()
      .as_millis()
  );

  let indexer_rpc = std::env::var("OG_INDEXER_RPC")
    .unwrap_or_else(|_| "https://indexer-storage-testnet-turbo.0g.ai".to_string());
  let output = Command::new(CLI_PATH)
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

// ─── Load manifest → download all exploits ────────────────────────────────────

fn load_all_from_manifest() -> Result<Vec<LoadedExploit>> {
  println!("📂 Loading manifest: {}", MANIFEST_PATH);
  let raw = fs::read_to_string(MANIFEST_PATH).context(
    "manifest.json not found — run indexers first (npm run index:cases && npm run index:protocols)",
  )?;
  let manifest: HashMap<String, HashMap<String, String>> =
    serde_json::from_str(&raw).context("Failed to parse manifest.json")?;

  let mut all: Vec<LoadedExploit> = Vec::new();
  let mut failed = 0usize;

  for (stream_id, entries) in &manifest {
    println!("  Stream '{}': {} entries", stream_id, entries.len());
    for (key, root_hash) in entries {
      print!("    ↓ {} ... ", key);
      match download_and_parse(root_hash, stream_id, key) {
        Ok(data) => {
          println!("✅ ({} dims)", data.embedding.len());
          all.push(LoadedExploit {
            key: format!("{}:{}", stream_id, key),
            data,
          });
        }
        Err(e) => {
          println!("❌ {}", e);
          failed += 1;
        }
      }
    }
  }

  println!("\n✅ Loaded {} exploits ({} failed)\n", all.len(), failed);
  Ok(all)
}

// ─── Cosine similarity search ─────────────────────────────────────────────────

fn cosine_similarity(a: &[f64], b: &[f64]) -> f64 {
  if a.len() != b.len() || a.is_empty() {
    return 0.0;
  }
  let dot: f64 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
  let na: f64 = a.iter().map(|x| x * x).sum::<f64>().sqrt();
  let nb: f64 = b.iter().map(|x| x * x).sum::<f64>().sqrt();
  if na == 0.0 || nb == 0.0 {
    0.0
  } else {
    dot / (na * nb)
  }
}

fn top_k_similar<'a>(
  query_embedding: &[f64],
  exploits: &'a [LoadedExploit],
  k: usize,
) -> Vec<(f64, &'a LoadedExploit)> {
  let mut scored: Vec<(f64, &LoadedExploit)> = exploits
    .iter()
    .map(|e| (cosine_similarity(query_embedding, &e.data.embedding), e))
    .collect();
  scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
  scored.truncate(k);
  scored
}

// ─── 0G Compute LLM inference ─────────────────────────────────────────────────

#[derive(Serialize)]
struct ChatMsg {
  role: String,
  content: String,
}

#[derive(Serialize)]
struct ChatReq {
  model: String,
  messages: Vec<ChatMsg>,
}

#[derive(Deserialize)]
struct ChatChoice {
  message: ChatChoiceMsg,
}

#[derive(Deserialize)]
struct ChatChoiceMsg {
  content: String,
}

#[derive(Deserialize)]
struct ChatResp {
  choices: Vec<ChatChoice>,
}

async fn call_og_compute(prompt: &str) -> Result<String> {
  let endpoint = std::env::var("OG_COMPUTE_ENDPOINT").context("OG_COMPUTE_ENDPOINT not set in .env")?;
  let api_key = std::env::var("OG_COMPUTE_API_KEY").context("OG_COMPUTE_API_KEY not set in .env")?;
  let model = std::env::var("OG_COMPUTE_MODEL")
    .unwrap_or_else(|_| "qwen/qwen-2.5-7b-instruct".to_string());
  let resp: ChatResp = Client::new()
        .post(&endpoint)
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&ChatReq {
            model,
            messages: vec![
                ChatMsg {
                    role: "system".to_string(),
                    content: "You are a DeFi security expert. Analyze smart contracts for vulnerabilities based on provided exploit patterns.".to_string(),
                },
                ChatMsg {
                    role: "user".to_string(),
                    content: prompt.to_string(),
                },
            ],
        })
        .send()
        .await
        .context("Failed to call 0G Compute")?
        .error_for_status()
        .context("0G Compute error status")?
        .json()
        .await
        .context("Failed to parse 0G Compute response")?;

  resp
    .choices
    .into_iter()
    .next()
    .map(|c| c.message.content)
    .ok_or_else(|| anyhow::anyhow!("No choices in 0G Compute response"))
}

// ─── Build RAG prompt ─────────────────────────────────────────────────────────

fn build_rag_prompt(user_query: &str, top_matches: &[(f64, &LoadedExploit)]) -> String {
  let mut ctx = String::new();
  for (i, (score, e)) in top_matches.iter().enumerate() {
    let m = &e.data.metadata;
    ctx.push_str(&format!(
            "\n[Match {}] similarity={:.3}\n  Name: {}\n  Type: {}\n  Chain: {}\n  Lost: {}\n  Snippet: {}\n",
            i + 1, score,
            m.exploit_name, m.vuln_type, m.chain, m.total_lost,
            m.code_snippet.chars().take(300).collect::<String>(),
        ));
  }
  format!(
        "Similar exploit patterns from knowledge base:\n{}\n\nUser question: {}\n\nProvide a detailed security analysis.",
        ctx, user_query
    )
}

// ─── Demo embedding (replace with real embedding call in production) ──────────

fn demo_query_embedding(dims: usize) -> Vec<f64> {
  // In production: call an embedding API on the user query text
  (0..dims).map(|i| (i as f64 * 0.001) % 1.0).collect()
}

// ─── Main ─────────────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() -> Result<()> {
  dotenv::dotenv().ok();
  println!("=== RAXC RAG Pipeline (0G Storage + 0G Compute) ===\n");

  // Step 1: Load all exploit data from 0G Storage via manifest
  let exploits = load_all_from_manifest()?;

  if exploits.is_empty() {
    eprintln!(
      "❌ No exploits loaded. Run indexers first:\n  \
             cd indexer-ts && npm run index:cases && npm run index:protocols"
    );
    std::process::exit(1);
  }

  // Step 2: Query
  let user_query = "What are common reentrancy vulnerabilities in DeFi contracts?";
  println!("🔍 Query: {}\n", user_query);

  // Step 3: Query embedding  (use first exploit dims as reference)
  let dims = exploits[0].data.embedding.len();
  println!("🔢 Query embedding: {} dims (demo vector)\n", dims);
  let query_emb = demo_query_embedding(dims);

  // Step 4: Cosine similarity → top-K
  println!("📊 Top-{} similar exploits:", TOP_K);
  let top = top_k_similar(&query_emb, &exploits, TOP_K);
  for (score, e) in &top {
    println!(
      "  {:.3}  {}  [{}]",
      score, e.data.metadata.exploit_name, e.data.metadata.chain
    );
  }
  println!();

  // Step 5: Build augmented prompt
  let prompt = build_rag_prompt(user_query, &top);

  // Step 6: 0G Compute inference
  println!("🤖 Calling 0G Compute ({})...\n", std::env::var("OG_COMPUTE_MODEL").unwrap_or_else(|_| "qwen/qwen-2.5-7b-instruct".to_string()));
  let answer = call_og_compute(&prompt).await?;

  println!("─── 0G Compute Response ───");
  println!("{}", answer);
  println!("\n✅ RAG pipeline complete!");

  Ok(())
}
