use std::collections::{hash_map::Entry, HashMap};
use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use chrono::Local;
use qdrant_client::qdrant::value::Kind;
use qdrant_client::qdrant::{Query, QueryPointsBuilder};
use qdrant_client::Qdrant;
use regex::Regex;
use reqwest::Client;
use serde::Deserialize;
use serde_json::{json, Value};

// ─── Constants ────────────────────────────────────────────────────────────────

pub const COLL_PROTOCOLS: &str = "defi_protocols";
pub const COLL_CASES: &str = "defi_cases";
pub const EMBED_MODEL: &str = "text-embedding-3-small";
pub const TOP_K: usize = 5;
pub const CODE_TRUNCATE: usize = 6000;
pub const SIM_THRESHOLD: f64 = 0.60;
pub const CONF_THRESHOLD: i64 = 60;

// ─── Data types ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct ExploitMatch {
  pub score: f64,
  pub payload: HashMap<String, Value>,
}

#[derive(Debug)]
pub struct FunctionMatch {
  pub function: String,
  pub matches: Vec<ExploitMatch>,
}

pub struct ReportFields {
  pub vuln_found: String,
  pub risk_level: String,
  pub vuln_type: String,
  pub confidence: String,
}

// ── OpenAI response types (private) ──────────────────────────────────────────

#[derive(Deserialize)]
struct EmbeddingData {
  embedding: Vec<f64>,
}

#[derive(Deserialize)]
struct EmbeddingResponse {
  data: Vec<EmbeddingData>,
}

#[derive(Deserialize)]
struct ChatMessage {
  content: String,
}

#[derive(Deserialize)]
struct ChatChoice {
  message: ChatMessage,
}

#[derive(Deserialize)]
struct ChatResponse {
  choices: Vec<ChatChoice>,
}

// ─── Environment / client setup ───────────────────────────────────────────────

/// Load `.env` from the project root (where `Cargo.toml` lives).
pub fn load_env() {
  // 1. Try current working directory first (works in Docker when --env-file
  //    mounts a .env, or for local `docker run` with a volume).
  dotenv::dotenv().ok();
  // 2. Fall back to the source-tree root (works during `cargo run` locally).
  let root = Path::new(env!("CARGO_MANIFEST_DIR"));
  dotenv::from_path(root.join(".env")).ok();
}

/// Build a `Qdrant` client from `QDRANT_URL` + `QDRANT_API_KEY` env vars.
/// Automatically converts the REST port 6333 → gRPC port 6334.
pub fn build_qdrant() -> Result<Qdrant> {
  let url = std::env::var("QDRANT_URL").context("QDRANT_URL not set in .env")?;
  let key = std::env::var("QDRANT_API_KEY").context("QDRANT_API_KEY not set in .env")?;
  let url = url.replace(":6333", ":6334");
  Qdrant::from_url(&url)
    .api_key(key)
    .build()
    .context("Failed to build Qdrant client")
}

// ─── Qdrant payload helper ────────────────────────────────────────────────────

pub fn qdrant_val_to_json(v: &qdrant_client::qdrant::Value) -> Value {
  match v.kind.as_ref() {
    Some(Kind::StringValue(s)) => Value::String(s.clone()),
    Some(Kind::IntegerValue(i)) => Value::Number((*i).into()),
    Some(Kind::DoubleValue(d)) => serde_json::Number::from_f64(*d)
      .map(Value::Number)
      .unwrap_or(Value::Null),
    Some(Kind::BoolValue(b)) => Value::Bool(*b),
    _ => Value::Null,
  }
}

// ─── RAG helpers ──────────────────────────────────────────────────────────────

pub async fn embed(client: &Client, api_key: &str, text: &str) -> Result<Vec<f64>> {
  let truncated = truncate_utf8(text, CODE_TRUNCATE);
  let resp: EmbeddingResponse = client
    .post("https://api.openai.com/v1/embeddings")
    .bearer_auth(api_key)
    .json(&json!({ "input": truncated, "model": EMBED_MODEL }))
    .send()
    .await
    .context("OpenAI embeddings request failed")?
    .json()
    .await
    .context("Failed to parse OpenAI embeddings response")?;
  Ok(
    resp
      .data
      .into_iter()
      .next()
      .map(|d| d.embedding)
      .unwrap_or_default(),
  )
}

fn truncate_utf8(s: &str, max_bytes: usize) -> &str {
  if s.len() <= max_bytes {
    return s;
  }
  let mut end = max_bytes;
  while !s.is_char_boundary(end) {
    end -= 1;
  }
  &s[..end]
}

pub async fn query_collection(
  qdrant: &Qdrant,
  vector: &[f64],
  collection: &str,
  top_k: usize,
) -> Vec<ExploitMatch> {
  let vec_f32: Vec<f32> = vector.iter().map(|&x| x as f32).collect();
  let result = qdrant
    .query(
      QueryPointsBuilder::new(collection)
        .query(Query::new_nearest(vec_f32))
        .limit((top_k * 2) as u64)
        .with_payload(true),
    )
    .await;

  match result {
    Ok(response) => response
      .result
      .into_iter()
      .map(|p| ExploitMatch {
        score: p.score as f64,
        payload: p
          .payload
          .into_iter()
          .map(|(k, v)| (k, qdrant_val_to_json(&v)))
          .collect(),
      })
      .collect(),
    Err(_) => vec![],
  }
}

pub async fn retrieve(
  http: &Client,
  qdrant: &Qdrant,
  api_key: &str,
  contract_code: &str,
  top_k: usize,
) -> Result<Vec<ExploitMatch>> {
  let vector = embed(http, api_key, contract_code).await?;

  let mut raw: Vec<ExploitMatch> = Vec::new();
  raw.extend(query_collection(qdrant, &vector, COLL_PROTOCOLS, top_k).await);
  raw.extend(query_collection(qdrant, &vector, COLL_CASES, top_k).await);

  let mut seen: HashMap<String, ExploitMatch> = HashMap::new();
  for r in raw {
    let name = payload_str(&r.payload, "exploit_name").to_string();
    match seen.entry(name) {
      Entry::Vacant(e) => {
        e.insert(r);
      }
      Entry::Occupied(mut e) => {
        if r.score > e.get().score {
          e.insert(r);
        }
      }
    }
  }

  let mut results: Vec<ExploitMatch> = seen.into_values().collect();
  results.sort_by(|a, b| {
    b.score
      .partial_cmp(&a.score)
      .unwrap_or(std::cmp::Ordering::Equal)
  });
  results.truncate(top_k);
  Ok(results)
}

/// Extract a string value from a payload map, returning "unknown" if missing.
pub fn payload_str<'a>(payload: &'a HashMap<String, Value>, key: &str) -> &'a str {
  payload
    .get(key)
    .and_then(|v| v.as_str())
    .unwrap_or("unknown")
}

pub fn build_context(results: &[ExploitMatch]) -> String {
  let mut context = String::new();
  for (i, r) in results.iter().enumerate() {
    let p = &r.payload;
    let score = (r.score * 1000.0).round() / 1000.0;
    let source = payload_str(p, "source");
    let is_real = source != "DeFiVulnLabs";

    let header = format!(
      "--- Reference {}: {} ({}) [similarity: {}] [source: {}] ---",
      i + 1,
      payload_str(p, "exploit_name"),
      payload_str(p, "date"),
      score,
      source
    );

    let (tx_line, lost_line, type_line) = if is_real {
      (
        format!("Attack Tx: {}", payload_str(p, "attack_tx")),
        format!("Total Lost: {}", payload_str(p, "total_lost")),
        String::new(),
      )
    } else {
      (
        "Attack Tx: N/A (educational pattern — no on-chain incident)".to_string(),
        "Total Lost: N/A".to_string(),
        format!("Vulnerability Type: {}\n", payload_str(p, "vuln_type")),
      )
    };

    let code = p
      .get("code")
      .and_then(|v| v.as_str())
      .unwrap_or("")
      .chars()
      .take(1500)
      .collect::<String>();

    context.push_str(&format!(
      "\n{}\nChain: {}\n{}\n{}\n{}Vulnerable Contract: {}\nCode Snippet:\n{}\n",
      header,
      payload_str(p, "chain"),
      lost_line,
      tx_line,
      type_line,
      payload_str(p, "vulnerable_contract"),
      code
    ));
  }
  context
}

// ─── Function-level exploit matching ─────────────────────────────────────────

pub fn extract_functions(code: &str) -> HashMap<String, String> {
  let mut functions = HashMap::new();
  let re = Regex::new(r"\bfunction\s+(\w+)\s*\(").unwrap();
  let bytes = code.as_bytes();

  for cap in re.captures_iter(code) {
    let name = cap[1].to_string();
    let match_start = cap.get(0).unwrap().start();
    let match_end = cap.get(0).unwrap().end();

    let brace_rel = match code[match_end..].find('{') {
      Some(pos) => pos,
      None => continue,
    };
    let brace_pos = match_end + brace_rel;

    let mut depth: i32 = 0;
    let mut end = code.len();
    for i in brace_pos..bytes.len() {
      match bytes[i] {
        b'{' => depth += 1,
        b'}' => {
          depth -= 1;
          if depth == 0 {
            end = i + 1;
            break;
          }
        }
        _ => {}
      }
    }
    functions.insert(name, code[match_start..end].to_string());
  }
  functions
}

pub async fn match_functions(
  http: &Client,
  qdrant: &Qdrant,
  api_key: &str,
  contract_code: &str,
  top_k: usize,
) -> Result<Vec<FunctionMatch>> {
  let functions = extract_functions(contract_code);
  let mut results: Vec<FunctionMatch> = Vec::new();

  for (func_name, func_body) in &functions {
    let vector = embed(http, api_key, func_body).await?;

    let mut raw: Vec<ExploitMatch> = Vec::new();
    raw.extend(query_collection(qdrant, &vector, COLL_PROTOCOLS, top_k).await);
    raw.extend(query_collection(qdrant, &vector, COLL_CASES, top_k).await);

    let mut seen: HashMap<String, ExploitMatch> = HashMap::new();
    for r in raw {
      let name = payload_str(&r.payload, "exploit_name").to_string();
      match seen.entry(name) {
        Entry::Vacant(e) => {
          e.insert(r);
        }
        Entry::Occupied(mut e) => {
          if r.score > e.get().score {
            e.insert(r);
          }
        }
      }
    }

    let mut top: Vec<ExploitMatch> = seen.into_values().collect();
    top.sort_by(|a, b| {
      b.score
        .partial_cmp(&a.score)
        .unwrap_or(std::cmp::Ordering::Equal)
    });
    top.truncate(top_k);
    results.push(FunctionMatch {
      function: func_name.clone(),
      matches: top,
    });
  }
  Ok(results)
}

// ─── Analysis ─────────────────────────────────────────────────────────────────

pub async fn analyze(
  http: &Client,
  qdrant: &Qdrant,
  api_key: &str,
  contract_code: &str,
) -> Result<(String, Vec<ExploitMatch>)> {
  println!(
    "[*] Retrieving top {} results from defi_protocols + defi_cases...",
    TOP_K
  );
  let results = retrieve(http, qdrant, api_key, contract_code, TOP_K).await?;

  let top_sim = results.first().map(|r| r.score).unwrap_or(0.0);
  println!("[*] Top similarity score: {:.3}", top_sim);

  if top_sim < SIM_THRESHOLD {
    println!(
      "[!] Similarity {:.3} below threshold {} — skipping GPT-4o, contract appears safe.",
      top_sim, SIM_THRESHOLD
    );
    let report = format!(
      "**Vulnerability Found:** No\n\
             **Risk Level:** None\n\
             **Vulnerability Type:** N/A\n\
             **Similar Exploit Reference:** NONE\n\
             **Explanation:** Top similarity score ({:.3}) is below the minimum threshold ({}). \
             No sufficiently similar exploit pattern found in the database.",
      top_sim, SIM_THRESHOLD
    );
    return Ok((report, results));
  }

  println!("[*] Building prompt and calling GPT-4o...\n");
  let context = build_context(&results);

  let prompt = format!(
    r#"You are a smart contract security expert specializing in DeFi vulnerabilities.

Analyze the following Solidity contract for potential vulnerabilities.
Use the reference cases below as context — retrieved from DeFiHackLabs (real protocol attacks) and DeFiVulnLabs (educational vulnerability patterns).

## Similar Reference Cases (DeFiHackLabs real exploits + DeFiVulnLabs educational patterns):
{context}

## Contract to Analyze:
{contract_code}

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
Separate each distinct issue or improvement into its own labeled case (A, B, C, ...). For each case:
- State the problem in one sentence.
- Show ONLY the one affected function rewritten in full — do NOT include contract declaration, constructor, imports, structs, or any other functions.
- Every line of the function must be written out completely — the words "existing code", "existing logic", "..." and any placeholder comments are FORBIDDEN.
- Add an inline comment on every line you changed explaining what was fixed and why.
- If a vulnerability was found: each case must directly correspond to one finding named in the Explanation section.
- If no vulnerability was found: each case must apply a concrete proactive improvement (e.g. access control, input validation, oracle integration, checks-effects-interactions) to one specific sensitive function.
- You MUST write ALL cases completely. Do NOT summarize, skip, or abbreviate any case. Do NOT end with a generic note like "apply similar changes elsewhere" — write each case in full.
"#,
    context = context,
    contract_code = contract_code
  );

  let chat_resp: ChatResponse = http
    .post("https://api.openai.com/v1/chat/completions")
    .bearer_auth(api_key)
    .json(&json!({
        "model": "gpt-4o",
        "messages": [{ "role": "user", "content": prompt }],
        "max_tokens": 8000
    }))
    .send()
    .await
    .context("OpenAI chat request failed")?
    .json()
    .await
    .context("Failed to parse OpenAI chat response")?;

  let mut report = chat_resp
    .choices
    .into_iter()
    .next()
    .map(|c| c.message.content)
    .unwrap_or_default();

  let conf_re = Regex::new(r"(?i)\*\*confidence[^0-9\n]*?(\d+)").unwrap();
  let conf: i64 = conf_re
    .captures(&report)
    .and_then(|c| c.get(1))
    .and_then(|m| m.as_str().parse().ok())
    .unwrap_or(50);

  if conf < CONF_THRESHOLD {
    println!(
      "[!] Model confidence {} below threshold {} — overriding to No vulnerability.",
      conf, CONF_THRESHOLD
    );
    report = report.replace(
      "**Vulnerability Found:** Yes",
      "**Vulnerability Found:** No (low confidence override)",
    );
  }

  Ok((report, results))
}

// ─── Report formatting ────────────────────────────────────────────────────────

pub fn sim_badge(score: f64) -> &'static str {
  if score >= 0.65 {
    "🔴 HIGH"
  } else if score >= 0.55 {
    "🟡 MED"
  } else {
    "🟢 LOW"
  }
}

pub fn parse_report_fields(report: &str) -> ReportFields {
  let mut fields = ReportFields {
    vuln_found: "Unknown".to_string(),
    risk_level: "Unknown".to_string(),
    vuln_type: "N/A".to_string(),
    confidence: "?".to_string(),
  };
  for line in report.lines() {
    let lower = line.trim().to_lowercase();
    if lower.starts_with("**vulnerability found:**") {
      fields.vuln_found = after_colon(line);
    } else if lower.starts_with("**risk level:**") {
      fields.risk_level = after_colon(line);
    } else if lower.starts_with("**vulnerability type:**") {
      fields.vuln_type = after_colon(line);
    } else if lower.starts_with("**confidence:**") {
      let raw = after_colon(line);
      if let Ok(n) = raw.split_whitespace().next().unwrap_or("").parse::<i64>() {
        fields.confidence = n.to_string();
      }
    }
  }
  fields
}

fn after_colon(line: &str) -> String {
  line
    .splitn(2, ':')
    .nth(1)
    .unwrap_or("")
    .trim()
    .trim_matches('*')
    .trim()
    .to_string()
}

fn verdict_banner(fields: &ReportFields) -> Vec<String> {
  let vuln = fields.vuln_found.to_lowercase();
  let risk = fields.risk_level.to_lowercase();
  let (icon, bar) = if vuln.contains("yes") {
    if risk.contains("critical") {
      ("🚨", "CRITICAL VULNERABILITY FOUND")
    } else if risk.contains("high") {
      ("🔴", "HIGH RISK VULNERABILITY FOUND")
    } else if risk.contains("medium") {
      ("🟠", "MEDIUM RISK VULNERABILITY FOUND")
    } else {
      ("🟡", "LOW RISK VULNERABILITY FOUND")
    }
  } else {
    ("✅", "NO EXPLOITABLE VULNERABILITY FOUND")
  };
  vec![
    format!("> ## {} {}", icon, bar),
    format!(
      "> **Risk Level:** {}  |  **Type:** {}  |  **Confidence:** {}/100",
      fields.risk_level, fields.vuln_type, fields.confidence
    ),
    String::new(),
  ]
}

/// Build the markdown report as a String without writing to disk.
/// Returns `(filename, content)` where filename is `RAXC_{name}_{timestamp}.md`.
pub fn build_markdown(
  report: &str,
  results: &[ExploitMatch],
  contract_name: &str,
  func_matches: Option<&[FunctionMatch]>,
) -> (String, String) {
  let timestamp = Local::now().format("%Y%m%d_%H%M%S").to_string();
  let filename = format!("RAXC_{}_{}.md", contract_name, timestamp);

  let fields = parse_report_fields(report);
  let mut lines: Vec<String> = Vec::new();

  lines.push("# RAXC Security Report".to_string());
  lines.push(format!(
    "> **Generated:** {}  ",
    Local::now().format("%Y-%m-%d %H:%M:%S")
  ));
  lines.push(String::new());
  lines.push("---".to_string());
  lines.push(String::new());

  lines.extend(verdict_banner(&fields));
  lines.push("---".to_string());
  lines.push(String::new());

  lines.push("## Top Similar Exploit References".to_string());
  lines.push(String::new());
  lines.push(String::new());
  lines.push("| # | Exploit | Date | Chain | Total Lost | Similarity |".to_string());
  lines.push("|---|---------|------|-------|------------|------------|".to_string());
  for (i, r) in results.iter().enumerate() {
    let p = &r.payload;
    let score = (r.score * 1000.0).round() / 1000.0;
    let tx = payload_str(p, "attack_tx");
    let name = payload_str(p, "exploit_name");
    let name_cell = if tx.starts_with("http") {
      format!("[{}]({})", name, tx)
    } else {
      name.to_string()
    };
    lines.push(format!(
      "| {} | **{}** | {} | {} | {} | {} {} |",
      i + 1,
      name_cell,
      payload_str(p, "date"),
      payload_str(p, "chain"),
      payload_str(p, "total_lost"),
      score,
      sim_badge(score)
    ));
  }
  lines.push(String::new());
  lines.push("---".to_string());
  lines.push(String::new());

  if let Some(fm_list) = func_matches {
    lines.push("## Function-Level Exploit Matching".to_string());
    lines.push(String::new());
    lines.push(
      "*Each contract function embedded and matched independently against the exploit database.*"
        .to_string(),
    );
    lines.push(String::new());
    lines.push("| Similarity | Meaning |".to_string());
    lines.push("|-----------|---------|".to_string());
    lines.push("| 🔴 ≥ 0.65 | High — strong structural match to known exploit |".to_string());
    lines.push("| 🟡 0.55–0.65 | Medium — partial overlap with exploit pattern |".to_string());
    lines.push("| 🟢 < 0.55 | Low — weak or incidental similarity |".to_string());
    lines.push(String::new());
    for fm in fm_list {
      let top_score = fm.matches.first().map(|r| r.score).unwrap_or(0.0);
      lines.push(format!("### `{}` {}", fm.function, sim_badge(top_score)));
      lines.push(String::new());
      lines.push("| # | Exploit | Date | Chain | Total Lost | Similarity |".to_string());
      lines.push("|---|---------|------|-------|------------|------------|".to_string());
      for (j, r) in fm.matches.iter().enumerate() {
        let p = &r.payload;
        let score = (r.score * 1000.0).round() / 1000.0;
        let tx = payload_str(p, "attack_tx");
        let name = payload_str(p, "exploit_name");
        let name_cell = if tx.starts_with("http") {
          format!("[{}]({})", name, tx)
        } else {
          name.to_string()
        };
        lines.push(format!(
          "| {} | **{}** | {} | {} | {} | {} {} |",
          j + 1,
          name_cell,
          payload_str(p, "date"),
          payload_str(p, "chain"),
          payload_str(p, "total_lost"),
          score,
          sim_badge(score)
        ));
      }
      lines.push(String::new());
    }
    lines.push("---".to_string());
    lines.push(String::new());
  }

  lines.push("## Analysis & Recommendation".to_string());
  lines.push(String::new());
  lines.push(report.to_string());
  lines.push(String::new());
  lines.push("---".to_string());
  lines.push(String::new());
  lines.push("> *Powered by RAXC — RAG-based smart contract vulnerability scanner*  ".to_string());
  lines.push(
    "> *Embeddings: OpenAI text-embedding-3-small · LLM: GPT-4o · Vector DB: Qdrant*".to_string(),
  );

  (filename, lines.join("\n"))
}

/// Build the markdown report and write it to the `reports/` directory.
/// Returns the file path as a String.
pub fn save_markdown(
  report: &str,
  results: &[ExploitMatch],
  contract_name: &str,
  func_matches: Option<&[FunctionMatch]>,
) -> Result<String> {
  let (filename, content) = build_markdown(report, results, contract_name, func_matches);
  let out_dir = Path::new("reports");
  fs::create_dir_all(out_dir)?;
  let filepath = out_dir.join(&filename);
  fs::write(&filepath, content)?;
  Ok(filepath.to_string_lossy().to_string())
}
