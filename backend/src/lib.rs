/*!
RAXC — RAG-based smart contract vulnerability analysis with 0G Storage + 0G Compute.

Simplified architecture matching full_rag_demo.rs:
1. Pre-load exploits at startup (OgStorageClient does this)
2. Embed contract code (OpenAI or hash-based demo)
3. Query in-memory exploits (fast cosine similarity)
4. Build RAG context and prompt
5. Send to 0G Compute for LLM analysis
*/

use anyhow::{Context, Result};
use chrono::Local;
use reqwest::Client;
use std::path::Path;

mod agent;
mod og_compute;
pub mod og_storage;
pub mod tools;
pub mod erc8183;

pub use agent::{
  Agent, AgentConfig, AgentOutput, Tool, RaxcAnalyzer, RaxcAnalyzerRemote, ToolSignal, DecisionResult,
  // Multi-Agent Framework
  ToolRegistry, AgentVote, ConsensusEngine, MemoryLayer, AgentCore,
  AnalysisResult, ReportEngine,
  // Production Hardening
  SignalNormalizer, SeverityLock,
  // Intelligence + Scoring Layer
  IntelligenceReport, RiskScoringEngine, ToolTrustWeighting, ExploitabilityEstimator,
  // Attack Simulation + Deterministic Exploit Execution Engine
  AttackSimulation, StateTransition, AttackerModel, ExploitVerdict, AttackSimulationEngine,
  DeterministicReplay, ExploitGraph, AttackerPersona, AttackerCapabilities,
  ConfidenceEngine, ExecutionStep, ToolSignalReference, AttackSuccessProbability, StateProof, SeverityProof,
};
pub use og_compute::OgComputeClient;
pub use og_storage::{ExploitData, ExploitMetadata, LoadedExploit, OgStorageClient, RemoteOgStorageClient, RemoteExploitResult};
pub use tools::{GasAnalyzerTool, PatternDetectorTool, FlashLoanTool, AccessControlTool, ReflectionTool, MemoryTool};
pub use erc8183::{create_audit_task, finalize_audit_task, hex_to_bytes32};

// ─── Constants ────────────────────────────────────────────────────────────────

pub const TOP_K: usize = 5;
pub const SIM_THRESHOLD: f64 = 0.01;  // Lowered to always trigger analysis even with poor similarity

// ─── Environment setup ────────────────────────────────────────────────────────

/// Load `.env` from the project root
pub fn load_env() {
  dotenv::dotenv().ok();
  let root = Path::new(env!("CARGO_MANIFEST_DIR"));
  dotenv::from_path(root.join(".env")).ok();
}

/// Build 0G Storage client (pre-loads exploits)
pub async fn build_og_storage() -> Result<OgStorageClient> {
  let indexer_rpc = std::env::var("OG_INDEXER_RPC")
    .unwrap_or_else(|_| "https://indexer-storage-testnet-turbo.0g.ai".to_string());
  let stream_id =
    std::env::var("OG_STORAGE_STREAM_ID").unwrap_or_else(|_| "defi_cases".to_string());
  let cli_path = std::env::var("OG_CLI_PATH").unwrap_or_else(|_| "./0g-cli".to_string());
  let manifest_path =
    std::env::var("OG_MANIFEST_PATH").unwrap_or_else(|_| "./manifest.json".to_string());

  OgStorageClient::new(indexer_rpc, stream_id, cli_path, manifest_path).await
}

/// Build 0G Compute client
pub fn build_og_compute() -> Result<OgComputeClient> {
  let endpoint =
    std::env::var("OG_COMPUTE_ENDPOINT").context("OG_COMPUTE_ENDPOINT not set in .env")?;
  let model =
    std::env::var("OG_COMPUTE_MODEL").unwrap_or_else(|_| "qwen/qwen-2.5-7b-instruct".to_string());

  match std::env::var("OG_COMPUTE_API_KEY") {
    Ok(api_key) => Ok(OgComputeClient::with_api_key(endpoint, model, api_key)),
    Err(_) => Ok(OgComputeClient::new(endpoint, model)),
  }
}

// ─── Embedding (0G Compute LLM → OpenAI fallback → hash demo) ────────────────

#[derive(serde::Serialize)]
struct EmbedRequest {
  input: String,
  model: String,
}

#[derive(serde::Deserialize)]
struct EmbedResponse {
  data: Vec<EmbedData>,
}

#[derive(serde::Deserialize)]
struct EmbedData {
  embedding: Vec<f64>,
}

/// Embed text — demo: OpenAI → hash-based fallback
/// Production: swap to embed_0g_compute() when exploit DB is re-indexed with 0G vectors
pub async fn embed(client: &Client, text: &str) -> Result<Vec<f64>> {
  let use_openai = std::env::var("USE_OPENAI_EMBEDDING")
    .map(|v| v == "true" || v == "1")
    .unwrap_or(false);

  if use_openai {
    embed_openai(client, text).await
  } else {
    Ok(embed_hash_demo(text, 1536))
  }
}

/// 0G Compute LLM-based embedding (production path — requires exploit DB re-indexed with these vectors).
/// Prompts the chat LLM to extract 32 semantic vulnerability concepts,
/// then deterministically hashes them into a 1536-dim vector.
/// Uses the same OG_COMPUTE_ENDPOINT + OG_COMPUTE_API_KEY already configured.
#[allow(dead_code)]
async fn embed_0g_compute(client: &Client, text: &str) -> Result<Vec<f64>> {
  use serde_json::Value;

  let endpoint = std::env::var("OG_COMPUTE_ENDPOINT")
    .context("OG_COMPUTE_ENDPOINT not set")?;
  let model = std::env::var("OG_COMPUTE_MODEL")
    .unwrap_or_else(|_| "qwen/qwen-2.5-7b-instruct".to_string());

  let snippet = text.chars().take(3000).collect::<String>();
  let prompt = format!(
    "Extract exactly 32 semantic security concepts from this smart contract code.\n\
     Return ONLY a JSON array of 32 short strings (1-3 words each). No explanation.\n\
     Example: [\"reentrancy\", \"access control\", \"flash loan\", ...]\n\n\
     Contract:\n{}", snippet
  );

  #[derive(serde::Serialize)]
  struct Msg { role: String, content: String }
  #[derive(serde::Serialize)]
  struct Req { model: String, messages: Vec<Msg>, max_tokens: u32 }

  let req = Req {
    model: model.clone(),
    messages: vec![
      Msg { role: "system".to_string(), content: "You are a smart contract security expert. Output only valid JSON.".to_string() },
      Msg { role: "user".to_string(), content: prompt },
    ],
    max_tokens: 256,
  };

  let mut http_req = client.post(&endpoint).json(&req);
  if let Ok(api_key) = std::env::var("OG_COMPUTE_API_KEY") {
    http_req = http_req.bearer_auth(api_key);
  }

  let resp = http_req.send().await.context("0G Compute embedding request failed")?;
  if !resp.status().is_success() {
    let status = resp.status();
    let body = resp.text().await.unwrap_or_default();
    anyhow::bail!("0G Compute error {}: {}", status, body);
  }

  let body: Value = resp.json().await.context("Failed to parse 0G Compute response")?;
  let content = body["choices"][0]["message"]["content"]
    .as_str()
    .unwrap_or("")
    .trim()
    .to_string();

  // Parse JSON array of concept strings from LLM response
  let start = content.find('[').unwrap_or(0);
  let end = content.rfind(']').map(|i| i + 1).unwrap_or(content.len());
  let json_slice = &content[start..end];

  let concepts: Vec<String> = serde_json::from_str(json_slice)
    .unwrap_or_else(|_| vec![content.clone()]);

  // Hash concepts deterministically into a 1536-dim vector
  Ok(embed_hash_demo(&concepts.join(" "), 1536))
}

/// OpenAI API embedding (fallback when 0G Compute is unavailable)
async fn embed_openai(client: &Client, text: &str) -> Result<Vec<f64>> {
  let api_key = std::env::var("OPENAI_API_KEY")
    .context("OPENAI_API_KEY not set — required for USE_OPENAI_EMBEDDING=true")?;

  let req = EmbedRequest {
    input: text.chars().take(8000).collect(),
    model: "text-embedding-3-small".to_string(),
  };

  let resp = client
    .post("https://api.openai.com/v1/embeddings")
    .bearer_auth(api_key)
    .json(&req)
    .send()
    .await
    .context("OpenAI API request failed")?;

  if !resp.status().is_success() {
    let status = resp.status();
    let body = resp.text().await.unwrap_or_default();
    anyhow::bail!("OpenAI API error {}: {}", status, body);
  }

  let embed_resp: EmbedResponse = resp.json().await?;
  Ok(embed_resp.data.into_iter().next().map(|d| d.embedding).unwrap_or_default())
}

/// Hash-based demo embedding (deterministic, no API)
fn embed_hash_demo(text: &str, dims: usize) -> Vec<f64> {
  use std::collections::hash_map::DefaultHasher;
  use std::hash::{Hash, Hasher};

  (0..dims)
    .map(|i| {
      let mut hasher = DefaultHasher::new();
      text.hash(&mut hasher);
      i.hash(&mut hasher);
      let hash_val = hasher.finish();
      (hash_val as f64 / u64::MAX as f64) * 2.0 - 1.0
    })
    .collect()
}

// ─── RAG context builder ──────────────────────────────────────────────────────

/// Build RAG context from top-K matches (like full_rag_demo.rs)
pub fn build_rag_context(top: &[(f64, &LoadedExploit)]) -> String {
  let mut ctx = String::new();
  for (i, (score, e)) in top.iter().enumerate() {
    let m = &e.data.metadata;
    let score_rounded = (score * 1000.0).round() / 1000.0;
    let is_real = m.source != "DeFiVulnLabs";

    let header = format!(
      "--- Reference {}: {} ({}) [similarity: {}] [source: {}] ---",
      i + 1,
      m.exploit_name,
      m.date,
      score_rounded,
      m.source,
    );

    let (tx_line, lost_line, type_line) = if is_real {
      (
        format!("Attack Tx: {}", if m.attack_tx.is_empty() { "unknown" } else { &m.attack_tx }),
        format!("Total Lost: {}", m.total_lost),
        String::new(),
      )
    } else {
      (
        "Attack Tx: N/A (educational pattern)".to_string(),
        "Total Lost: N/A".to_string(),
        format!("Vulnerability Type: {}\n", m.vuln_type),
      )
    };

    let code = m.code_snippet.chars().take(1500).collect::<String>();

    ctx.push_str(&format!(
      "\n{}\nChain: {}\n{}\n{}\n{}Vulnerable Contract: {}\nCode Snippet:\n{}\n",
      header,
      m.chain,
      lost_line,
      tx_line,
      type_line,
      if m.vulnerable_contract.is_empty() { "unknown" } else { &m.vulnerable_contract },
      code,
    ));
  }
  ctx
}

// ─── Remote RAG context builder ──────────────────────────────────────────────

/// Build RAG context from remote storage API results (no local storage needed)
pub fn build_rag_context_remote(top: &[og_storage::RemoteExploitResult]) -> String {
  let mut ctx = String::new();
  for (i, e) in top.iter().enumerate() {
    let score_rounded = (e.score * 1000.0).round() / 1000.0;
    let is_real = e.source != "DeFiVulnLabs";

    let header = format!(
      "--- Reference {}: {} ({}) [similarity: {}] [source: {}] ---",
      i + 1,
      e.exploit_name,
      e.date,
      score_rounded,
      e.source,
    );

    let (tx_line, lost_line, type_line) = if is_real {
      (
        format!("Attack Tx: {}", if e.attack_tx.is_empty() { "unknown" } else { &e.attack_tx }),
        format!("Total Lost: {}", e.total_lost),
        String::new(),
      )
    } else {
      (
        "Attack Tx: N/A (educational pattern)".to_string(),
        "Total Lost: N/A".to_string(),
        format!("Vulnerability Type: {}\n", e.vuln_type),
      )
    };

    let code = e.code_snippet.chars().take(1500).collect::<String>();

    ctx.push_str(&format!(
      "\n{}\nChain: {}\n{}\n{}\n{}Code Snippet:\n{}\n",
      header,
      e.chain,
      lost_line,
      tx_line,
      type_line,
      code,
    ));
  }
  ctx
}

// ─── Analysis workflow ────────────────────────────────────────────────────────

/// Analyze contract using pre-loaded exploits
pub async fn analyze(
  http: &Client,
  storage: &OgStorageClient,
  compute: &OgComputeClient,
  contract: &str,
) -> Result<String> {
  println!("[RaxcAnalyzer]   Embedding contract code...");
  let query_vec = embed(http, contract).await?;

  println!("[RaxcAnalyzer]   Querying 722-exploit RAG database...");
  let top_matches = storage.query(&query_vec, TOP_K);

  let top_score = top_matches.first().map(|(s, _)| *s).unwrap_or(0.0);
  println!("[RaxcAnalyzer]   Top similarity: {:.3}", top_score);

  if top_score < SIM_THRESHOLD {
    println!("[!] Similarity {:.3} below threshold {} — skipping 0G Compute, contract appears safe.", top_score, SIM_THRESHOLD);
    return Ok(format!(
      "✅ NO EXPLOITABLE VULNERABILITY FOUND\nTop similarity score ({:.3}) is below minimum threshold ({}).",
      top_score, SIM_THRESHOLD
    ));
  }

  println!("[RaxcAnalyzer]   Building RAG context...");
  let context = build_rag_context(&top_matches);

  println!("[0G Compute]     Sending for analysis...");
  let prompt = format!(
    r#"You are a smart contract security expert specializing in DeFi vulnerabilities.

Analyze the following Solidity contract for potential vulnerabilities.
Use the reference cases below as context — retrieved from DeFiHackLabs (real protocol attacks) and DeFiVulnLabs (educational vulnerability patterns).

## Similar Reference Cases (DeFiHackLabs real exploits + DeFiVulnLabs educational patterns):
{context}

## Contract to Analyze:
{contract}

## Critical instructions before answering:
1. The exploit cases show HOW past vulnerabilities worked. Your job is to determine if THIS contract has the same UNMITIGATED flaw — not just a similar structure.
2. Actively check for these mitigations. If any are correctly implemented, they PREVENT exploitation:
   - ReentrancyGuard modifier or Checks-Effects-Interactions (state update before external call)
   - TWAP / time-weighted average price oracle (resistant to single-block manipulation)
   - onlyOwner / role-based access control on sensitive functions
   - Solidity 0.8+ built-in overflow protection or SafeMath
3. Structural similarity to an exploit is NOT sufficient. The contract must have the same exploitable flaw WITH NO mitigation present.
4. Include a CONFIDENCE score (0-100) reflecting how certain you are a real exploitable vulnerability exists with no mitigation.
5. For EXPLOIT_TX in your report: only cite the exact Attack Tx URLs present in the reference cases above. If a reference shows "N/A" or no real tx, write N/A. Do NOT fabricate or invent transaction hashes.

## Provide a structured security report with the following sections:

**Vulnerability Found:** Yes / No
**Risk Level:** Critical / High / Medium / Low / None
**Vulnerability Type:** (e.g. Reentrancy, Flash Loan, Price Manipulation, Access Control, etc.)
**Confidence:** (0-100 — certainty that a real exploitable vulnerability exists with no mitigation present)
**Similar Exploit Reference:** (which exploit case above is most relevant and why)
**Explanation:** (describe the exact vulnerability and how an attacker could exploit it step-by-step)
**Recommendation:**
IMPORTANT: Provide AT LEAST 3-4 detailed cases (A, B, C, D, ...). Each case must be a complete, standalone solution.
Separate each distinct issue or improvement into its own labeled case (A, B, C, D, ...). For each case:
- State the problem in one sentence.
- Show ONLY the one affected function rewritten in full — do NOT include contract declaration, constructor, imports, structs, or any other functions.
- Every line of the function must be written out completely — the words "existing code", "existing logic", "..." and any placeholder comments are FORBIDDEN.
- Add an inline comment on every line you changed explaining what was fixed and why.
- If a vulnerability was found: each case must directly correspond to one finding named in the Explanation section (Case A: fix reentrancy, Case B: fix oracle manipulation, Case C: add access control, Case D: add input validation).
- If no vulnerability was found: each case must apply a concrete proactive improvement (e.g. Case A: add ReentrancyGuard, Case B: implement TWAP oracle, Case C: add onlyOwner modifier, Case D: add SafeMath).
- You MUST write ALL cases completely. Do NOT summarize, skip, or abbreviate any case. Do NOT end with a generic note like "apply similar changes elsewhere" — write each case in full.
EXAMPLE: If you find a reentrancy bug, provide: Case A (Checks-Effects-Interactions), Case B (ReentrancyGuard modifier), Case C (Oracle TWAP improvement), Case D (Access control for sensitive functions).
"#,
    context = context,
    contract = contract,
  );

  compute.infer(&prompt).await
}

/// Analyze contract using remote api_0g_storage server (no local loading needed).
/// Identical analysis to `analyze()` but queries port 3001 instead of loading locally.
pub async fn analyze_remote(
  http: &Client,
  storage: &og_storage::RemoteOgStorageClient,
  compute: &OgComputeClient,
  contract: &str,
) -> Result<String> {
  println!("[RaxcAnalyzer]   Embedding contract code...");
  let query_vec = embed(http, contract).await?;

  println!("[RaxcAnalyzer]   Querying 722-exploit RAG database...");
  let top_matches = storage.query(&query_vec, TOP_K).await?;

  let top_score = top_matches.first().map(|e| e.score).unwrap_or(0.0);
  println!("[RaxcAnalyzer]   Top similarity: {:.3}", top_score);

  if top_score < SIM_THRESHOLD {
    println!("[!] Similarity {:.3} below threshold {} — skipping 0G Compute, contract appears safe.", top_score, SIM_THRESHOLD);
    return Ok(format!(
      "✅ NO EXPLOITABLE VULNERABILITY FOUND\nTop similarity score ({:.3}) is below minimum threshold ({}).",
      top_score, SIM_THRESHOLD
    ));
  }

  println!("[RaxcAnalyzer]   Building RAG context...");
  let context = build_rag_context_remote(&top_matches);

  println!("[0G Compute]     Sending for analysis...");
  let prompt = format!(
    r#"You are a smart contract security expert specializing in DeFi vulnerabilities.

Analyze the following Solidity contract for potential vulnerabilities.
Use the reference cases below as context — retrieved from DeFiHackLabs (real protocol attacks) and DeFiVulnLabs (educational vulnerability patterns).

## Similar Reference Cases (DeFiHackLabs real exploits + DeFiVulnLabs educational patterns):
{context}

## Contract to Analyze:
{contract}

## Critical instructions before answering:
1. The exploit cases show HOW past vulnerabilities worked. Your job is to determine if THIS contract has the same UNMITIGATED flaw — not just a similar structure.
2. Actively check for these mitigations. If any are correctly implemented, they PREVENT exploitation:
   - ReentrancyGuard modifier or Checks-Effects-Interactions (state update before external call)
   - TWAP / time-weighted average price oracle (resistant to single-block manipulation)
   - onlyOwner / role-based access control on sensitive functions
   - Solidity 0.8+ built-in overflow protection or SafeMath
3. Structural similarity to an exploit is NOT sufficient. The contract must have the same exploitable flaw WITH NO mitigation present.
4. Include a CONFIDENCE score (0-100) reflecting how certain you are a real exploitable vulnerability exists with no mitigation.
5. For EXPLOIT_TX in your report: only cite the exact Attack Tx URLs present in the reference cases above. If a reference shows "N/A" or no real tx, write N/A. Do NOT fabricate or invent transaction hashes.

## Provide a structured security report with the following sections:

**Vulnerability Found:** Yes / No
**Risk Level:** Critical / High / Medium / Low / None
**Vulnerability Type:** (e.g. Reentrancy, Flash Loan, Price Manipulation, Access Control, etc.)
**Confidence:** (0-100 — certainty that a real exploitable vulnerability exists with no mitigation present)
**Similar Exploit Reference:** (which exploit case above is most relevant and why)
**Explanation:** (describe the exact vulnerability and how an attacker could exploit it step-by-step)
**Recommendation:**
IMPORTANT: Provide AT LEAST 3-4 detailed cases (A, B, C, D, ...). Each case must be a complete, standalone solution.
Separate each distinct issue or improvement into its own labeled case (A, B, C, D, ...). For each case:
- State the problem in one sentence.
- Show ONLY the one affected function rewritten in full — do NOT include contract declaration, constructor, imports, structs, or any other functions.
- Every line of the function must be written out completely — the words "existing code", "existing logic", "..." and any placeholder comments are FORBIDDEN.
- Add an inline comment on every line you changed explaining what was fixed and why.
- If a vulnerability was found: each case must directly correspond to one finding named in the Explanation section (Case A: fix reentrancy, Case B: fix oracle manipulation, Case C: add access control, Case D: add input validation).
- If no vulnerability was found: each case must apply a concrete proactive improvement (e.g. Case A: add ReentrancyGuard, Case B: implement TWAP oracle, Case C: add onlyOwner modifier, Case D: add SafeMath).
- You MUST write ALL cases completely. Do NOT summarize, skip, or abbreviate any case. Do NOT end with a generic note like "apply similar changes elsewhere" — write each case in full.
EXAMPLE: If you find a reentrancy bug, provide: Case A (Checks-Effects-Interactions), Case B (ReentrancyGuard modifier), Case C (Oracle TWAP improvement), Case D (Access control for sensitive functions).
"#,
    context = context,
    contract = contract,
  );

  compute.infer(&prompt).await
}

/// Build markdown report (matching full_rag_demo.rs format)
pub fn build_markdown(
  analysis: &str,
  contract_name: &str,
  contract_code: &str,
  top_matches: &[(f64, &LoadedExploit)],
  analysis_time_ms: u128,
  total_loaded: usize,
) -> (String, String) {
  let timestamp = Local::now().format("%Y%m%d_%H%M%S");
  let filename = format!("RAXC_{}_{}.md", contract_name, timestamp);

  let model = std::env::var("OG_COMPUTE_MODEL")
    .unwrap_or_else(|_| "qwen/qwen-2.5-7b-instruct".to_string());

  let matches_table = top_matches
    .iter()
    .enumerate()
    .map(|(i, (score, e))| {
      // Show root hash from manifest.json (more accurate than generic source label)
      let root_display = if e.root_hash.len() > 16 {
        format!("0x{}...{}", &e.root_hash[2..10], &e.root_hash[e.root_hash.len()-6..])
      } else {
        e.root_hash.clone()
      };
      format!(
        "{}. **{}** (similarity: {:.4}) — {}",
        i + 1,
        e.data.metadata.exploit_name,
        score,
        root_display
      )
    })
    .collect::<Vec<_>>()
    .join("\n");

  let content = format!(
    r#"# Smart Contract Security Report

**Generated:** {}  
**Model:** {}  
**Analysis Time:** {} ms

---

## Contract Analyzed

```solidity
{}
```

---

## Top-{} Similar Exploits (from local cache)

{}

---

## 🧠 Agent Analysis

{}

---

## Metadata

- **Total in Cache:** {} exploits
- **Similarity Computation:** {}ms
- **Storage:** 0G Storage (cached locally)
- **Compute:** 0G Compute — {}
- **Embedding:** OpenAI text-embedding-3-small (1536 dims)

---

*Report generated by RAXC — Cached RAG-powered smart contract auditing*
"#,
    Local::now().format("%Y-%m-%d %H:%M:%S"),
    model,
    analysis_time_ms,
    contract_code.trim(),
    TOP_K,
    matches_table,
    analysis,
    total_loaded,
    analysis_time_ms,
    model,
  );

  (filename, content)
}

#[allow(dead_code)]
/// Extract reasoning from the analysis (Explanation section)
fn extract_reasoning(analysis: &str) -> String {
  // Look for "Explanation:" section in the analysis
  if let Some(expl_idx) = analysis.find("**Explanation:**") {
    let after_expl = &analysis[expl_idx..];
    
    // Find the end of the explanation (next ** marker or Recommendation section)
    let end_markers = ["**Recommendation:**", "\n**", "---"];
    let mut end_idx = after_expl.len();
    
    for marker in &end_markers {
      if let Some(idx) = after_expl[14..].find(marker) {
        end_idx = end_idx.min(idx + 14);
      }
    }
    
    let explanation = &after_expl[14..end_idx];
    let trimmed = explanation.trim();
    
    if !trimmed.is_empty() {
      return format!(
        "The agent identified vulnerabilities through the following reasoning:\n\n{}",
        trimmed
      );
    }
  }
  
  // Fallback if no Explanation section found
  "The agent analyzed the contract against known exploit patterns from DeFiHackLabs and DeFiVulnLabs. The similarity scores and vulnerability patterns informed the security assessment.".to_string()
}
