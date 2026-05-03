/*!
RAXC API Server — Step 9.9 Deterministic Exploit Execution + Verification Framework

This API provides HTTP interface to the Step 9.9 RAXC framework with:
  ✓ ToolRegistry (pluggable tools)
  ✓ Multi-Agent Reasoning
  ✓ ConsensusEngine
  ✓ MemoryLayer (0G Storage)
  ✓ Intelligence Layer
  ✓ Attack Simulation Engine
  ✓ GraphConstructionEngine [NEW Step 9.9]
  ✓ ConsistencyEngine [NEW Step 9.9]
  ✓ FinalDecisionEngine [NEW Step 9.9]
  ✓ AttestationEngine [NEW Step 9.9]
  ✓ LLM Layer (0G Compute)
  ✓ ReportEngine

Usage:
  cargo run --bin api_step9_9

Endpoints:
  POST /analyze          { "contract": "...solidity code...", "name": "ContractName" }
                         → { "download_url": "/reports/RAXC_...md", "vulnerability_found": true, ... }
  GET  /reports/{file}   Download the generated markdown report
  GET  /health           Health check
*/

use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Context;
use axum::{
  body::Body,
  extract::{Path, State},
  http::{header, StatusCode},
  response::{IntoResponse, Response},
  routing::{get, post},
  Json, Router,
};
use raxc::{
  build_og_compute, build_og_storage, load_env, 
  AgentCore, RaxcAnalyzer, GasAnalyzerTool, PatternDetectorTool
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::sync::Mutex;
use tower_http::cors::{Any, CorsLayer};

// ─── Shared State ─────────────────────────────────────────────────────────────

#[derive(Clone)]
struct AppState {
  /// Step 9.9 AgentCore with full verification framework
  agent_core: Arc<Mutex<AgentCore>>,
  /// In-memory report store: filename → markdown content
  reports: Arc<Mutex<HashMap<String, String>>>,
}

// ─── Request / Response Types ─────────────────────────────────────────────────

#[derive(Deserialize)]
struct AnalyzeRequest {
  /// Solidity contract source code
  contract: String,
  /// Contract name (optional, defaults to "contract")
  #[serde(default = "default_name")]
  name: String,
}

fn default_name() -> String {
  "contract".to_string()
}

#[derive(Serialize)]
struct AnalyzeResponse {
  /// Download URL for markdown report
  download_url: String,
  /// Whether vulnerability was found
  vulnerability_found: bool,
  /// Risk level classification
  risk_level: String,
  /// Primary vulnerability type
  vulnerability_type: String,
  /// Final confidence score (0-100)
  confidence: f64,
  /// Risk score (0-100)
  risk_score: f64,
  /// Exploitability score (0-100)
  exploitability: f64,
  /// Attack probability (0-100)
  attack_probability: f64,
  /// Consistency score from verification (0-100)
  consistency_score: f64,
  /// Graph nodes count
  graph_nodes: usize,
  /// Graph edges count
  graph_edges: usize,
  /// Attestation replay ID
  replay_id: String,
  /// Attestation trace hash
  trace_hash: String,
}

// ─── Error Type ───────────────────────────────────────────────────────────────

struct AppError(anyhow::Error);

impl IntoResponse for AppError {
  fn into_response(self) -> Response {
    (
      StatusCode::INTERNAL_SERVER_ERROR,
      Json(json!({ "error": self.0.to_string() })),
    )
      .into_response()
  }
}

impl<E> From<E> for AppError
where
  E: Into<anyhow::Error>,
{
  fn from(e: E) -> Self {
    AppError(e.into())
  }
}

// ─── Handlers ─────────────────────────────────────────────────────────────────

/// POST /analyze — Run Step 9.9 analysis on contract
#[axum::debug_handler]
async fn handle_analyze(
  State(state): State<AppState>,
  Json(req): Json<AnalyzeRequest>,
) -> Result<Json<AnalyzeResponse>, AppError> {
  println!("\n[*] Received analyze request for: {}", req.name);
  println!("[*] Contract length: {} bytes", req.contract.len());

  // Run Step 9.9 analysis (with full verification pipeline)
  let result = {
    let core = state.agent_core.lock().await;
    core.analyze(&req.contract, &req.name).await?
  };

  println!("[✓] Analysis complete: {}", result.filename);

  // Store markdown report in memory
  state
    .reports
    .lock()
    .await
    .insert(result.filename.clone(), result.markdown.clone());

  // Extract primary vulnerability type
  let vulnerability_type = result
    .decision
    .primary_vulnerability
    .clone()
    .unwrap_or_else(|| "None".to_string());

  // Build response with Step 9.9 data
  Ok(Json(AnalyzeResponse {
    download_url: format!("/reports/{}", result.filename),
    vulnerability_found: result.decision.vulnerability_found,
    risk_level: result.decision.risk_level.clone(),
    vulnerability_type,
    confidence: (result.final_decision.final_confidence * 100.0).round(),
    risk_score: (result.final_decision.final_risk_score * 100.0).round(),
    exploitability: (result.intelligence_report.exploitability_score * 100.0).round(),
    attack_probability: (result.final_decision.final_attack_probability * 100.0).round(),
    consistency_score: (result.consistency_check.consistency_score * 100.0).round(),
    graph_nodes: result.attack_graph.nodes.len(),
    graph_edges: result.attack_graph.edges.len(),
    replay_id: result.attestation.replay_id.clone(),
    trace_hash: result.attestation.execution_trace_hash.clone(),
  }))
}

/// GET /reports/{filename} — Download markdown report
async fn download_report(
  State(state): State<AppState>,
  Path(filename): Path<String>,
) -> Result<Response, AppError> {
  // Strip directory components to prevent path traversal
  let safe = std::path::Path::new(&filename)
    .file_name()
    .and_then(|n| n.to_str())
    .ok_or_else(|| anyhow::anyhow!("Invalid filename"))?
    .to_owned();

  // Get report from memory
  let content = state
    .reports
    .lock()
    .await
    .get(&safe)
    .cloned()
    .ok_or_else(|| anyhow::anyhow!("Report not found: {}", safe))?;

  let disposition = format!("attachment; filename=\"{}\"", safe);
  Ok(
    Response::builder()
      .header(header::CONTENT_TYPE, "text/markdown; charset=utf-8")
      .header(header::CONTENT_DISPOSITION, disposition)
      .body(Body::from(content))
      .unwrap(),
  )
}

/// GET /health — Health check
async fn health() -> impl IntoResponse {
  Json(json!({ 
    "status": "ok",
    "framework": "RAXC Step 9.9 - Deterministic Exploit Execution + Verification",
    "features": [
      "ToolRegistry",
      "Multi-Agent Reasoning",
      "ConsensusEngine",
      "MemoryLayer (0G Storage)",
      "Intelligence Layer",
      "Attack Simulation Engine",
      "GraphConstructionEngine",
      "ConsistencyEngine",
      "FinalDecisionEngine",
      "AttestationEngine",
      "LLM Layer (0G Compute)",
      "ReportEngine"
    ]
  }))
}

// ─── Entry Point ──────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  // Load environment variables
  load_env();

  // Enable OpenAI embeddings
  std::env::set_var("USE_OPENAI_EMBEDDING", "true");

  println!("╔══════════════════════════════════════════════════════════════════════════╗");
  println!("║    RAXC API Server - Step 9.9 Framework                                  ║");
  println!("║    Deterministic Exploit Execution + Verification                        ║");
  println!("╚══════════════════════════════════════════════════════════════════════════╝\n");

  // Initialize 0G clients
  println!("[*] Initializing 0G Storage client...");
  let storage = build_og_storage().await?;
  
  println!("[*] Initializing 0G Compute client...");
  let compute = build_og_compute()?;

  // Create AgentCore with Step 9.9 framework
  println!("[*] Creating AgentCore with Step 9.9 verification framework...");
  let mut core = AgentCore::new(storage.clone(), compute.clone());

  // Register tools to ToolRegistry
  println!("[*] Registering tools to ToolRegistry...");
  core.tools.register(Box::new(RaxcAnalyzer::new(storage, compute)));
  core.tools.register(Box::new(GasAnalyzerTool::new()));
  core.tools.register(Box::new(PatternDetectorTool::new()));
  println!("[✓] Registered {} tools", core.tools.tool_count());

  // Initialize app state
  let state = AppState {
    agent_core: Arc::new(Mutex::new(core)),
    reports: Arc::new(Mutex::new(HashMap::new())),
  };

  // Build router
  let app = Router::new()
    .route("/analyze", post(handle_analyze))
    .route("/reports/:filename", get(download_report))
    .route("/health", get(health))
    .layer(
      CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any),
    )
    .with_state(state);

  // Start server
  let addr = "0.0.0.0:3000";
  println!("\n[✓] Server starting on http://{}", addr);
  println!("\n╔══════════════════════════════════════════════════════════════════════════╗");
  println!("║  ENDPOINTS:                                                              ║");
  println!("║    POST /analyze       — Analyze smart contract                         ║");
  println!("║    GET  /reports/:file — Download markdown report                       ║");
  println!("║    GET  /health        — Health check                                   ║");
  println!("╚══════════════════════════════════════════════════════════════════════════╝\n");

  let listener = tokio::net::TcpListener::bind(addr)
    .await
    .context("Failed to bind to address")?;

  axum::serve(listener, app)
    .await
    .context("Server failed")?;

  Ok(())
}
