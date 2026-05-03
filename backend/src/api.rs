/*!
RAXC API server — HTTP interface to the smart contract vulnerability scanner.

Usage:
  cargo run --bin api

Endpoints:
  POST /analyze          { "contract": "...solidity code...", "payment_id": "0x...", "tx_hash": "0x...", "user": "0x..." }
                         → { "download_url": "/reports/RAXC_...md", "vulnerability_found": "...", ... }
  GET  /reports/{file}   download the generated markdown report
  GET  /health           liveness check
*/

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

use anyhow::Context;
use axum::{
  body::Body,
  extract::{Path, State},
  http::{header, StatusCode},
  response::{IntoResponse, Response},
  routing::{get, post},
  Json, Router,
};
use ethers::{
  prelude::*,
  providers::{Http, Provider},
};
use raxc::{build_og_compute, build_og_storage, load_env, AgentCore, RaxcAnalyzer, GasAnalyzerTool, PatternDetectorTool};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tower_http::cors::{Any, CorsLayer};

// ─── Smart Contract ABI ───────────────────────────────────────────────────────

abigen!(
  RaxcVault,
  r#"[
    function verifyPayment(bytes32 paymentId) external view returns (bool isValid, address user, uint256 amount)
    function markPaymentUsed(bytes32 paymentId) external
  ]"#,
);

// ─── Shared state ─────────────────────────────────────────────────────────────

#[derive(Clone)]
struct AppState {
  agent_core: Arc<Mutex<AgentCore>>,
  provider: Arc<Provider<Http>>,
  vault_contract: Arc<RaxcVault<Provider<Http>>>,
  operator_wallet: Arc<LocalWallet>,
  /// In-memory report store: filename → markdown content (no disk writes)
  reports: Arc<Mutex<HashMap<String, String>>>,
}

// ─── Request / response types ─────────────────────────────────────────────────

#[derive(Deserialize)]
struct AnalyzeRequest {
  contract: String,
  #[serde(default = "default_name")]
  name: String,
  /// Payment ID from payForAnalysis() transaction
  payment_id: String,
  /// Transaction hash for verification
  tx_hash: String,
  /// User address who made the payment
  user: String,
}

fn default_name() -> String {
  "contract".to_string()
}

#[derive(Serialize)]
struct AnalyzeResponse {
  download_url: String,
  vulnerability_found: bool,
  risk_level: String,
  vulnerability_type: String,
  confidence: f64,
  risk_score: f64,
  exploitability: f64,
  attack_probability: f64,
  consistency_score: f64,
  graph_nodes: usize,
  graph_edges: usize,
  replay_id: String,
  trace_hash: String,
}

// ─── Error type ───────────────────────────────────────────────────────────────

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

#[axum::debug_handler]
async fn handle_analyze(
  State(state): State<AppState>,
  Json(req): Json<AnalyzeRequest>,
) -> Result<Json<AnalyzeResponse>, AppError> {
  // 1. Parse payment ID, tx hash, and user address
  let payment_id: [u8; 32] = hex::decode(req.payment_id.trim_start_matches("0x"))
    .context("Invalid payment_id hex")?
    .try_into()
    .map_err(|_| anyhow::anyhow!("payment_id must be 32 bytes"))?;

  let tx_hash: H256 = req.tx_hash.parse().context("Invalid tx_hash")?;
  let user_address: Address = req.user.parse().context("Invalid user address")?;

  // 2. Verify transaction exists and was successful
  let tx_receipt = state
    .vault_contract
    .client()
    .get_transaction_receipt(tx_hash)
    .await
    .context("Failed to fetch transaction receipt")?
    .ok_or_else(|| anyhow::anyhow!("Transaction not found: {}", tx_hash))?;

  // Check transaction was successful
  if tx_receipt.status != Some(U64::from(1)) {
    return Err(anyhow::anyhow!("Transaction failed or pending").into());
  }

  // Check transaction was sent by the claimed user
  if tx_receipt.from != user_address {
    return Err(
      anyhow::anyhow!(
        "Transaction sender mismatch: expected {}, got {}",
        user_address,
        tx_receipt.from
      )
      .into(),
    );
  }

  // Check transaction was sent to the vault contract
  if tx_receipt.to != Some(state.vault_contract.address()) {
    return Err(
      anyhow::anyhow!("Transaction recipient mismatch: not sent to vault contract").into(),
    );
  }

  println!(
    "[*] Transaction verified: {} from {} (status: success)",
    tx_hash, user_address
  );

  // 3. Verify payment on-chain
  let (is_valid, payment_user, amount) = state
    .vault_contract
    .as_ref()
    .verify_payment(payment_id)
    .call()
    .await
    .context("Failed to verify payment on-chain")?;

  if !is_valid {
    return Err(anyhow::anyhow!("Payment is invalid or already used").into());
  }

  if payment_user != user_address {
    return Err(
      anyhow::anyhow!(
        "Payment user mismatch: expected {}, got {}",
        user_address,
        payment_user
      )
      .into(),
    );
  }

  println!(
    "[*] Payment verified: {} USDC from {}",
    amount.as_u128() as f64 / 1e6,
    user_address
  );

  // 3. Mark payment as used (before analysis to prevent replay attacks)
  let signer = SignerMiddleware::new(
    state.provider.as_ref().clone(),
    state.operator_wallet.as_ref().clone(),
  );
  let contract_with_signer = state.vault_contract.as_ref().clone().connect(Arc::new(signer));

  // Call markPaymentUsed using the method() pattern - all in one chain
  contract_with_signer
    .method::<_, H256>("markPaymentUsed", payment_id)
    .context("Failed to build markPaymentUsed call")?
    .send()
    .await
    .context("Failed to send markPaymentUsed transaction")?
    .await
    .context("markPaymentUsed transaction failed")?;

  println!("[*] Payment marked as used: {}", req.payment_id);

  // 4. Run Step 9.9 analysis (now that payment is verified and marked)
  println!("[*] Running RAXC Hybrid Authority Agent (Tools→Agent→LLM)...");
  let result = {
    let core = state.agent_core.lock().await;
    core.analyze(&req.contract, &req.name).await?
  };

  println!("[✓] Analysis complete: {}", result.filename);

  // Store in memory — no disk write
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

  // Build response with Step 9.9 comprehensive data
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

async fn download_report(
  State(state): State<AppState>,
  Path(filename): Path<String>,
) -> Result<Response, AppError> {
  // Strip directory components to prevent path traversal.
  let safe = std::path::Path::new(&filename)
    .file_name()
    .and_then(|n| n.to_str())
    .ok_or_else(|| anyhow::anyhow!("Invalid filename"))?
    .to_owned();

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

async fn health() -> impl IntoResponse {
  Json(json!({ "status": "ok" }))
}

// ─── Entry point ──────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  load_env();

  let rpc_url = std::env::var("RPC_URL").context("RPC_URL not set (e.g., Initia RPC endpoint)")?;
  let vault_address =
    std::env::var("VAULT_ADDRESS").context("VAULT_ADDRESS not set (deployed contract)")?;
  let operator_key =
    std::env::var("OPERATOR_PRIVATE_KEY").context("OPERATOR_PRIVATE_KEY not set")?;

  let storage = build_og_storage().await?;
  let compute = build_og_compute()?;

  // Initialize blockchain provider
  let provider =
    Arc::new(Provider::<Http>::try_from(rpc_url).context("Failed to connect to RPC endpoint")?);
  let chain_id = provider.get_chainid().await?;
  println!("[*] Connected to chain ID: {}", chain_id);

  // Initialize operator wallet
  let operator_wallet: LocalWallet = operator_key
    .parse::<LocalWallet>()
    .context("Invalid OPERATOR_PRIVATE_KEY")?
    .with_chain_id(chain_id.as_u64());

  println!("[*] Operator address: {}", operator_wallet.address());

  // Initialize vault contract
  let vault_address: Address = vault_address
    .parse()
    .context("Invalid VAULT_ADDRESS format")?;
  let vault_contract = Arc::new(RaxcVault::new(vault_address, provider.clone()));

  println!("[*] Vault contract: {}", vault_address);

  // Initialize Step 9.9 AgentCore with tools
  println!("[*] Initializing Step 9.9 AgentCore with comprehensive analysis...");
  let mut agent_core = AgentCore::new(storage, compute);
  
  // Register tools
  agent_core.tools.register(Box::new(RaxcAnalyzer::new(agent_core.memory.storage.clone(), agent_core.compute.clone())));
  agent_core.tools.register(Box::new(GasAnalyzerTool));
  agent_core.tools.register(Box::new(PatternDetectorTool));
  
  println!("[✓] AgentCore initialized with {} tools", agent_core.tools.tool_count());

  let state = AppState {
    agent_core: Arc::new(Mutex::new(agent_core)),
    provider: provider.clone(),
    vault_contract,
    operator_wallet: Arc::new(operator_wallet),
    reports: Arc::new(Mutex::new(HashMap::new())),
  };

  let cors = CorsLayer::new()
    .allow_origin(Any)
    .allow_methods(Any)
    .allow_headers(Any);

  let app = Router::new()
    .route("/analyze", post(handle_analyze))
    .route("/reports/*filename", get(download_report))
    .route("/health", get(health))
    .layer(cors)
    .with_state(state);

  let addr = "0.0.0.0:8080";
  println!("[*] RAXC API server → http://{}", addr);
  println!("[*]   POST /analyze          body: {{\"contract\":\"...\",\"payment_id\":\"0x...\",\"tx_hash\":\"0x...\",\"user\":\"0x...\"}}");
  println!("[*]   GET  /reports/{{file}}   download the markdown report");
  println!("[*]   GET  /health           liveness check");

  let listener = tokio::net::TcpListener::bind(addr).await?;
  axum::serve(listener, app).await?;

  Ok(())
}
