/*!
Test reading from 0G Storage KV using CLI download method.

This approach uses `0g-cli download` to fetch files, then parses the binary
format to extract KV values. This is the working method on testnet.

Usage:
  cargo run --example test_og_download
*/

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::process::Command;
use base64::{Engine as _, engine::general_purpose};

#[derive(Debug, Serialize, Deserialize)]
struct ExploitData {
    embedding: Vec<f64>,
    metadata: ExploitMetadata,
}

#[derive(Debug, Serialize, Deserialize)]
struct ExploitMetadata {
    exploit_name: String,
    vuln_type: String,
    source: String,
    chain: String,
    code_snippet: String,
}

/// Download file from 0G Storage and extract KV value
fn read_from_og_kv(
    root_hash: &str,
    stream_id: &str,
    key: &str,
) -> Result<ExploitData> {
    println!("📥 Downloading file from 0G Storage...");
    println!("   Root: {}", root_hash);

    // Create temporary file path (not the file itself, let CLI create it)
    let temp_path = format!("/tmp/og_download_{}.bin", 
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis()
    );

    // Download file using 0g-cli
    let cli_path = "../0g-cli";
    let indexer_rpc = "https://indexer-storage-testnet-turbo.0g.ai";
    
    let output = Command::new(cli_path)
        .args(&[
            "download",
            "--indexer", indexer_rpc,
            "--root", root_hash,
            "--file", &temp_path,
        ])
        .output()
        .context("Failed to execute 0g-cli download")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Download failed: {}", stderr);
    }

    println!("✅ Downloaded to {}", temp_path);

    // Read the downloaded file
    let data = fs::read(&temp_path)
        .context("Failed to read downloaded file")?;
    
    // Cleanup temp file
    let _ = fs::remove_file(&temp_path);
    
    println!("📊 File size: {} bytes", data.len());

    // Find stream ID in data
    let stream_id_bytes = stream_id.as_bytes();
    let stream_id_index = find_bytes(&data, stream_id_bytes)
        .ok_or_else(|| anyhow::anyhow!("Stream ID '{}' not found in data", stream_id))?;
    
    println!("✅ Found stream ID at offset {}", stream_id_index);

    // Find key after stream ID
    let key_bytes = key.as_bytes();
    let key_index = find_bytes(&data[stream_id_index..], key_bytes)
        .map(|idx| stream_id_index + idx)
        .ok_or_else(|| anyhow::anyhow!("Key '{}' not found after stream ID", key))?;
    
    println!("✅ Found key at offset {}", key_index);

    // Extract base64 value after key
    let after_key_offset = key_index + key_bytes.len();
    let remaining_data = &data[after_key_offset..];
    
    // Convert to string and find base64 pattern
    let remaining_text = String::from_utf8_lossy(remaining_data);
    
    // Find continuous base64 characters (at least 100 chars)
    let base64_pattern = regex::Regex::new(r"([A-Za-z0-9+/]{100,}={0,2})")
        .context("Failed to compile regex")?;
    
    let base64_value = base64_pattern
        .find(&remaining_text)
        .ok_or_else(|| anyhow::anyhow!("No base64 value found after key"))?
        .as_str();
    
    println!("✅ Found base64 value ({} chars)", base64_value.len());

    // Decode base64
    let decoded_bytes = general_purpose::STANDARD
        .decode(base64_value)
        .context("Failed to decode base64")?;
    
    let decoded_text = String::from_utf8(decoded_bytes)
        .context("Decoded data is not valid UTF-8")?;
    
    // Parse JSON
    let exploit_data: ExploitData = serde_json::from_str(&decoded_text)
        .context("Failed to parse JSON")?;
    
    println!("✅ Successfully decoded KV value");
    println!("   Embedding dimension: {}", exploit_data.embedding.len());
    println!("   Exploit: {}", exploit_data.metadata.exploit_name);

    Ok(exploit_data)
}

/// Find a byte pattern in a slice
fn find_bytes(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== Test Read from 0G KV using CLI Download ===\n");

    let root_hash = "0x788ecdc715fc45a2bac2f4e7ca6064b07ed52595e3c7a47ac83082df3b7cac73";
    let stream_id = "defi_cases";
    let key = "test_exploit_001";

    match read_from_og_kv(root_hash, stream_id, key) {
        Ok(result) => {
            println!("\n--- Retrieved Data ---");
            println!("Embedding: [{} dimensions]", result.embedding.len());
            println!("Metadata: {:#?}", result.metadata);
            println!("\n✅ Read test successful!");
        }
        Err(e) => {
            eprintln!("\n❌ Read test failed: {}", e);
            std::process::exit(1);
        }
    }

    Ok(())
}
