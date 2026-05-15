//! ERC-8183 autonomous audit task lifecycle for RAXCLAW.
//!
//! Two functions cover the full task lifecycle:
//!   1. `create_audit_task`   — called before analysis, registers job on-chain
//!   2. `finalize_audit_task` — called after pipeline + ERC-7857 update, commits proof
//!
//! Env vars required:
//!   RAXC_AUDIT_TASK_8183_ADDRESS — deployed RaxcAuditTask8183 address (0x...)
//!   PRIVATE_KEY                  — agent wallet private key (0x...)
//!   OG_RPC_URL                   — 0G Mainnet RPC (default: https://evmrpc.0g.ai)

use std::sync::Arc;
use ethers::{
  abi::{self, Token},
  middleware::SignerMiddleware,
  providers::{Http, Middleware, Provider},
  signers::{LocalWallet, Signer},
  types::{Address, BlockId, BlockNumber, Bytes, H256, TransactionRequest, U256},
};
use crate::AnalysisResult;

// ─── Utility ──────────────────────────────────────────────────────────────────

/// Decode a 0x-prefixed hex string into a fixed 32-byte array (zero-left-padded).
pub fn hex_to_bytes32(s: &str) -> anyhow::Result<[u8; 32]> {
  let hex = s.trim_start_matches("0x");
  let padded = format!("{:0>64}", hex);
  let bytes = hex::decode(&padded[..64])
    .map_err(|e| anyhow::anyhow!("hex decode failed for '{}': {}", s, e))?;
  let mut arr = [0u8; 32];
  arr.copy_from_slice(&bytes);
  Ok(arr)
}

fn build_client() -> anyhow::Result<(Arc<SignerMiddleware<Provider<Http>, LocalWallet>>, Address)> {
  let contract_addr_str = std::env::var("RAXC_AUDIT_TASK_8183_ADDRESS")
    .map_err(|_| anyhow::anyhow!("RAXC_AUDIT_TASK_8183_ADDRESS not set"))?;
  let private_key = std::env::var("PRIVATE_KEY")
    .map_err(|_| anyhow::anyhow!("PRIVATE_KEY not set"))?;
  let rpc_url = std::env::var("OG_RPC_URL")
    .unwrap_or_else(|_| "https://evmrpc.0g.ai".to_string());

  let provider = Provider::<Http>::try_from(rpc_url.as_str())?;
  let contract_addr: Address = contract_addr_str.parse()?;

  let wallet: LocalWallet = private_key.trim_start_matches("0x")
    .parse::<LocalWallet>()?
    .with_chain_id(16661u64);
  let client = Arc::new(SignerMiddleware::new(provider, wallet));

  Ok((client, contract_addr))
}

// ─── Public API ───────────────────────────────────────────────────────────────

/// Register a new audit task on-chain (ERC-8183 `createAuditTask`).
///
/// Returns the `taskId` emitted by the `AuditTaskCreated` event.
/// Call this **before** `AgentCore::analyze()`.
pub async fn create_audit_task(contract_name: &str) -> anyhow::Result<u64> {
  let (client, contract_addr) = build_client()?;

  println!("\n\x1b[35m[ERC-8183]       Creating audit task on 0G Mainnet...\x1b[0m");
  println!("\x1b[2m    Contract:    {}\x1b[0m", contract_name);
  println!("\x1b[2m    Task Addr:   {}\x1b[0m", contract_addr);

  // createAuditTask(string)
  let selector = &ethers::utils::keccak256(b"createAuditTask(string)")[..4];
  let params = abi::encode(&[Token::String(contract_name.to_string())]);
  let mut calldata = selector.to_vec();
  calldata.extend_from_slice(&params);

  let tx = TransactionRequest::new()
    .to(contract_addr)
    .data(Bytes::from(calldata))
    .gas_price(3_000_000_000u64)
    .chain_id(16661u64);

  let pending = client.send_transaction(tx, None).await
    .map_err(|e| anyhow::anyhow!("createAuditTask tx failed: {}", e))?;

  let receipt = pending.await?
    .ok_or_else(|| anyhow::anyhow!("no receipt for createAuditTask"))?;

  // Extract taskId from AuditTaskCreated(uint256 indexed taskId, ...)
  // topic[0] = event sig hash, topic[1] = taskId (indexed uint256)
  let event_sig = H256::from(ethers::utils::keccak256(
    b"AuditTaskCreated(uint256,address,string,uint256)"
  ));
  let task_id = receipt.logs
    .iter()
    .find(|log| log.topics.first() == Some(&event_sig))
    .and_then(|log| log.topics.get(1))
    .map(|t| U256::from(t.as_bytes()).as_u64())
    .ok_or_else(|| anyhow::anyhow!("AuditTaskCreated event not found in receipt"))?;

  println!("\x1b[35m[ERC-8183]       Task #{} created (TX: https://chainscan.0g.ai/tx/0x{:x})\x1b[0m", task_id, receipt.transaction_hash);

  Ok(task_id)
}

/// Commit audit proof on-chain (ERC-8183 `finalizeAuditTask`).
///
/// Call this **after** `AgentCore::analyze()` and ERC-7857 update.
/// Attaches verdict, confidence, 0G root hash, replay ID, and trace hash to the task.
pub async fn finalize_audit_task(
  task_id: u64,
  result: &AnalysisResult,
  contract_name: &str,
) -> anyhow::Result<String> {
  let (client, contract_addr) = build_client()?;

  let verdict = result.final_decision.final_verdict.clone();
  let confidence_bps = (result.decision.confidence * 10_000.0) as u64; // 77.50% → 7750

  // Prefer report_root_hash (markdown), fall back to storage_root_hash (JSON)
  let root_hash_str = if !result.report_root_hash.is_empty() {
    result.report_root_hash.clone()
  } else if !result.storage_root_hash.is_empty() {
    result.storage_root_hash.clone()
  } else {
    "0x0000000000000000000000000000000000000000000000000000000000000000".to_string()
  };
  let root_hash = hex_to_bytes32(&root_hash_str)?;

  let replay_id = result.attestation.replay_id.clone();
  let trace_hash_str = &result.attestation.execution_trace_hash;
  let trace_hash_full = if trace_hash_str.starts_with("0x") {
    trace_hash_str.clone()
  } else {
    format!("0x{}", trace_hash_str)
  };
  let trace_hash = hex_to_bytes32(&trace_hash_full)?;

  println!("\n\x1b[35m[ERC-8183]       Finalizing audit task #{} on 0G Mainnet...\x1b[0m", task_id);
  println!("\x1b[2m    Contract:    {}\x1b[0m", contract_name);
  println!("\x1b[2m    Verdict:     {}\x1b[0m", verdict);
  println!("\x1b[2m    Confidence:  {:.2}%\x1b[0m", result.decision.confidence * 100.0);
  println!("\x1b[2m    Root Hash:   {}\x1b[0m", root_hash_str);
  println!("\x1b[2m    Replay ID:   {}\x1b[0m", replay_id);

  // finalizeAuditTask(uint256,string,uint256,bytes32,string,bytes32)
  let selector = &ethers::utils::keccak256(
    b"finalizeAuditTask(uint256,string,uint256,bytes32,string,bytes32)"
  )[..4];
  let params = abi::encode(&[
    Token::Uint(U256::from(task_id)),
    Token::String(verdict),
    Token::Uint(U256::from(confidence_bps)),
    Token::FixedBytes(root_hash.to_vec()),
    Token::String(replay_id),
    Token::FixedBytes(trace_hash.to_vec()),
  ]);
  let mut calldata = selector.to_vec();
  calldata.extend_from_slice(&params);

  // Fetch pending nonce explicitly to avoid race with ERC-7857 tx sent just before us
  let from_addr = client.signer().address();
  let nonce = client
    .inner()
    .get_transaction_count(from_addr, Some(BlockId::Number(BlockNumber::Pending)))
    .await?;

  let tx = TransactionRequest::new()
    .to(contract_addr)
    .data(Bytes::from(calldata))
    .nonce(nonce)
    .gas_price(3_000_000_000u64)
    .chain_id(16661u64);

  let pending = client.send_transaction(tx, None).await
    .map_err(|e| anyhow::anyhow!("finalizeAuditTask tx failed: {}", e))?;

  println!("\x1b[35m[ERC-8183]       Audit task #{} finalized on-chain (chain 16661)\x1b[0m", task_id);
  let tx_hash = format!("0x{:x}", pending.tx_hash());
  println!("    TX:  \x1b[92m{}\x1b[0m", tx_hash);
  println!("    URL: \x1b[94mhttps://chainscan.0g.ai/tx/{}\x1b[0m", tx_hash);
  println!("\x1b[2m    Task #{} is now COMPLETED and verifiable on-chain\x1b[0m", task_id);
  println!("\n\x1b[2m    View Report on Frontend:\x1b[0m");
  println!("    \x1b[94mhttps://raxclaw.vercel.app/roothash/{}?tx={}\x1b[0m", root_hash_str, tx_hash);

  Ok(tx_hash)
}
