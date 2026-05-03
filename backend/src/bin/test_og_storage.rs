/*!
Test NEW OgStorageClient - Exact copy from test_og_storage.rs

Run: cargo run --example test_og_storage_new
*/

use anyhow::Result;
use raxc::og_storage::OgStorageClient;

#[tokio::main]
async fn main() -> Result<()> {
  println!("🧪 Testing NEW OgStorageClient (copied from test_og_storage.rs)\n");

  let indexer_rpc = std::env::var("OG_INDEXER_RPC")
    .unwrap_or_else(|_| "https://indexer-storage-testnet-turbo.0g.ai".to_string());
  let stream_id = "defi_cases".to_string();
  let cli_path = "./0g-cli".to_string();
  let manifest_path = "./manifest.json".to_string();

  let start = std::time::Instant::now();
  
  let client = OgStorageClient::new(
    indexer_rpc,
    stream_id,
    cli_path,
    manifest_path,
  ).await?;

  let elapsed = start.elapsed();

  println!("\n{}", "=".repeat(60));
  println!("✅ NEW OgStorageClient created successfully!");
  println!("   Total exploits loaded: {}", client.total_loaded());
  println!("   Time taken: {:.2}s", elapsed.as_secs_f64());
  println!("{}", "=".repeat(60));

  Ok(())
}
