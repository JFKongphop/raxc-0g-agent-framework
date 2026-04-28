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
use std::sync::{Arc, Mutex};

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
use raxc::{analyze, build_markdown, build_qdrant, load_env, match_functions, parse_report_fields};
use reqwest::Client;
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

struct AppState {
  http: Client,
  qdrant: qdrant_client::Qdrant,
  api_key: String,
  provider: Arc<Provider<Http>>,
  vault_contract: RaxcVault<Provider<Http>>,
  operator_wallet: LocalWallet,
  /// In-memory report store: filename → markdown content (no disk writes)
  reports: Mutex<HashMap<String, String>>,
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
  vulnerability_found: String,
  risk_level: String,
  vulnerability_type: String,
  confidence: String,
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

async fn handle_analyze(
  State(state): State<Arc<AppState>>,
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
      anyhow::anyhow!(
        "Transaction recipient mismatch: not sent to vault contract"
      )
      .into(),
    );
  }

  println!(
    "[*] Transaction verified: {} from {} (status: success)",
    tx_hash,
    user_address
  );

  // 3. Verify payment on-chain
  let (is_valid, payment_user, amount) = state
    .vault_contract
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
  let signer = SignerMiddleware::new(state.provider.as_ref().clone(), state.operator_wallet.clone());
  let contract_with_signer = state.vault_contract.connect(Arc::new(signer));
  
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

  // 4. Run analysis (now that payment is verified and marked)
  let (report, results) =
    analyze(&state.http, &state.qdrant, &state.api_key, &req.contract).await?;

  let func_matches =
    match_functions(&state.http, &state.qdrant, &state.api_key, &req.contract, 3).await?;

  let (filename, content) = build_markdown(&report, &results, &req.name, Some(&func_matches));

  // Store in memory — no disk write
  state
    .reports
    .lock()
    .unwrap()
    .insert(filename.clone(), content);

  let fields = parse_report_fields(&report);

  Ok(Json(AnalyzeResponse {
    download_url: format!("/reports/{}", filename),
    vulnerability_found: fields.vuln_found,
    risk_level: fields.risk_level,
    vulnerability_type: fields.vuln_type,
    confidence: fields.confidence,
  }))
}

async fn download_report(
  State(state): State<Arc<AppState>>,
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
    .unwrap()
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

  let api_key = std::env::var("OPENAI_API_KEY").context("OPENAI_API_KEY not set")?;
  let rpc_url = std::env::var("RPC_URL").context("RPC_URL not set (e.g., Initia RPC endpoint)")?;
  let vault_address =
    std::env::var("VAULT_ADDRESS").context("VAULT_ADDRESS not set (deployed contract)")?;
  let operator_key =
    std::env::var("OPERATOR_PRIVATE_KEY").context("OPERATOR_PRIVATE_KEY not set")?;

  let http = Client::new();
  let qdrant = build_qdrant()?;

  // Initialize blockchain provider
  let provider = Arc::new(
    Provider::<Http>::try_from(rpc_url).context("Failed to connect to RPC endpoint")?
  );
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
  let vault_contract = RaxcVault::new(vault_address, provider.clone());

  println!("[*] Vault contract: {}", vault_address);

  let state = Arc::new(AppState {
    http,
    qdrant,
    api_key,
    provider: provider.clone(),
    vault_contract,
    operator_wallet,
    reports: Mutex::new(HashMap::new()),
  });

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
