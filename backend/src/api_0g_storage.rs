/*!
0G Storage API Server — Pre-loads ALL exploits once, serves fast queries via HTTP.

Problem solved:
  - agent_example.rs / OpenClaw skill would download 777 exploits every run (2-3 min)
  - This server loads them ONCE at startup, keeps in RAM forever
  - All clients call POST /query with an embedding → instant response

Endpoints:
  GET  /health         → { "status": "ok", "loaded": 777 }
  POST /query          → { "embedding": [...], "top_k": 5 }
                       → { "results": [{ "score": 0.91, "exploit": {...} }] }

Run:
  cargo run --bin api_0g_storage

Then call from anywhere:
  curl http://localhost:3001/health
  curl -X POST http://localhost:3001/query \
    -H 'Content-Type: application/json' \
    -d '{"embedding": [...512 floats...], "top_k": 5}'
*/

use std::sync::Arc;

use axum::{
  extract::State,
  http::StatusCode,
  response::IntoResponse,
  routing::{get, post},
  Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tower_http::cors::{Any, CorsLayer};

use raxc::{build_og_storage, load_env};

// ─── Shared state ─────────────────────────────────────────────────────────────

#[derive(Clone)]
struct StorageState {
  storage: Arc<raxc::OgStorageClient>,
}

// ─── Request / Response types ─────────────────────────────────────────────────

#[derive(Deserialize)]
struct QueryRequest {
  embedding: Vec<f64>,
  #[serde(default = "default_top_k")]
  top_k: usize,
}

fn default_top_k() -> usize {
  5
}

#[derive(Serialize)]
struct ExploitResult {
  score: f64,
  exploit_name: String,
  vuln_type: String,
  chain: String,
  date: String,
  total_lost: String,
  source: String,
  code_snippet: String,
  attack_tx: String,
  embedding_dim: usize,
}

#[derive(Serialize)]
struct QueryResponse {
  results: Vec<ExploitResult>,
  total_searched: usize,
  query_time_ms: u64,
}

// ─── Handlers ─────────────────────────────────────────────────────────────────

async fn health(State(state): State<StorageState>) -> impl IntoResponse {
  let loaded = state.storage.total_loaded();
  Json(json!({
    "status": "ok",
    "loaded": loaded,
    "message": format!("{} exploits ready for querying", loaded)
  }))
}

async fn query(
  State(state): State<StorageState>,
  Json(req): Json<QueryRequest>,
) -> Result<Json<QueryResponse>, (StatusCode, Json<serde_json::Value>)> {
  if req.embedding.is_empty() {
    return Err((
      StatusCode::BAD_REQUEST,
      Json(json!({ "error": "embedding cannot be empty" })),
    ));
  }

  let top_k = req.top_k.min(20); // cap at 20

  let start = std::time::Instant::now();

  // Fast in-memory cosine similarity — no network calls
  let matches = state.storage.query(&req.embedding, top_k);

  let elapsed_ms = start.elapsed().as_millis() as u64;
  let total_searched = state.storage.total_loaded();

  let results = matches
    .into_iter()
    .map(|(score, exploit)| ExploitResult {
      score,
      exploit_name: exploit.data.metadata.exploit_name.clone(),
      vuln_type: exploit.data.metadata.vuln_type.clone(),
      chain: exploit.data.metadata.chain.clone(),
      date: exploit.data.metadata.date.clone(),
      total_lost: exploit.data.metadata.total_lost.clone(),
      source: exploit.data.metadata.source.clone(),
      code_snippet: exploit.data.metadata.code_snippet.clone(),
      attack_tx: exploit.data.metadata.attack_tx.clone(),
      embedding_dim: exploit.data.embedding.len(),
    })
    .collect();

  Ok(Json(QueryResponse {
    results,
    total_searched,
    query_time_ms: elapsed_ms,
  }))
}

// ─── Entry point ──────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  load_env();

  let port = std::env::var("STORAGE_PORT")
    .unwrap_or_else(|_| "8080".to_string());

  println!("╔══════════════════════════════════════════════════════╗");
  println!("║        RAXC 0G Storage API Server                   ║");
  println!("║  Loads 777 exploits ONCE — serves queries instantly  ║");
  println!("╚══════════════════════════════════════════════════════╝\n");

  println!("[*] Loading 0G Storage exploits into memory...");
  println!("[*] This takes 2-3 min once, then all queries are instant.\n");

  let storage = build_og_storage().await?;
  let loaded = storage.total_loaded();

  println!("\n[✓] Loaded {} exploits into RAM", loaded);
  println!("[✓] Storage API ready — queries will respond in <10ms\n");

  let state = StorageState {
    storage: Arc::new(storage),
  };

  let cors = CorsLayer::new()
    .allow_origin(Any)
    .allow_methods(Any)
    .allow_headers(Any);

  let app = Router::new()
    .route("/health", get(health))
    .route("/query", post(query))
    .layer(cors)
    .with_state(state);

  let addr = format!("0.0.0.0:{}", port);
  println!("[*] Listening on http://{}", addr);
  println!("[*] Endpoints:");
  println!("    GET  /health  → storage status");
  println!("    POST /query   → {{ \"embedding\": [...], \"top_k\": 5 }}\n");

  let listener = tokio::net::TcpListener::bind(&addr).await?;
  axum::serve(listener, app).await?;

  Ok(())
}
