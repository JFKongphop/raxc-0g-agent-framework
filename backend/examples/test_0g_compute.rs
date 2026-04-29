/*!
Test 0G Compute connectivity.

This example tests the actual 0G Compute API to verify connectivity
and proper response handling.

Usage:
  cargo run --example test_0g_compute
*/

use anyhow::Result;
use raxc::{build_og_compute, load_env};

#[tokio::main]
async fn main() -> Result<()> {
  println!("=== Testing 0G Compute API ===\n");

  // Load environment variables
  load_env();

  // Build 0G Compute client
  let compute = build_og_compute()?;
  println!("✓ 0G Compute client initialized");
  
  // Test simple prompt
  println!("\nSending test prompt...");
  let prompt = "What is a reentrancy attack in Solidity? Answer in one sentence.";
  
  match compute.infer(prompt).await {
    Ok(response) => {
      println!("\n✓ Success! Response received:\n");
      println!("---");
      println!("{}", response);
      println!("---");
      println!("\n=== Test Complete ===");
      Ok(())
    }
    Err(e) => {
      eprintln!("\n✗ Error: {}", e);
      eprintln!("\nTroubleshooting:");
      eprintln!("1. Check OG_COMPUTE_ENDPOINT in .env");
      eprintln!("2. Verify OG_COMPUTE_API_KEY is correct");
      eprintln!("3. Ensure model name is valid: {}", std::env::var("OG_COMPUTE_MODEL").unwrap_or_default());
      eprintln!("4. Test endpoint with: ./test_0g_compute.sh");
      Err(e)
    }
  }
}
