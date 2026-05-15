/*!
Example: RAXC Multi-Agent Framework — Remote Storage Mode

Identical to agent_example.rs but uses api_0g_storage server (port 3001)
instead of downloading 777 exploits locally (~2-3 min cold start).

Prerequisites:
  1. Start the storage server first (one terminal):
       cargo run --bin api_0g_storage
     Wait for: "[✓] Listening on http://0.0.0.0:3001"

  2. Then run this example (instant start):
       cargo run --example agent_example_remote

Architecture change vs agent_example.rs:
  BEFORE: build_og_storage().await?  →  downloads 777 exploits (~2-3 min)
  AFTER:  RemoteOgStorageClient::local()  →  HTTP to port 3001 (<10ms per query)
*/

use anyhow::Result;
use std::sync::Arc;
use raxc::{
  build_og_compute, load_env,
  AgentCore, RaxcAnalyzerRemote, GasAnalyzerTool, PatternDetectorTool,
  FlashLoanTool, AccessControlTool, ReflectionTool, MemoryTool,
  RemoteOgStorageClient, OgStorageClient,
  create_audit_task, finalize_audit_task,
};
use ethers::{
  abi::{self, Token},
  middleware::SignerMiddleware,
  providers::{Http, Middleware, Provider},
  signers::{LocalWallet, Signer},
  types::{Address, Bytes, Filter, H256, TransactionRequest, U256},
};

#[tokio::main]
async fn main() -> Result<()> {
  // Load environment variables
  load_env();

  // Demo: use OpenAI embeddings (matches the vector space of the indexed 722 exploits).
  // Production: re-index exploits with 0G Compute vectors, then switch to embed_0g_compute().
  std::env::set_var("USE_OPENAI_EMBEDDING", "true");

  println!("\x1b[1;96m╔══════════════════════════════════════════════════════════════════════════╗\x1b[0m");
  println!("\x1b[1;96m║\x1b[0m  \x1b[1;96mRAXC Autonomous Exploit Intelligence Core — Sovereign Execution Mode\x1b[0m    \x1b[1;96m║\x1b[0m");
  println!("\x1b[1;96m║\x1b[0m         \x1b[2mDeterministic Exploit Execution + Verification Framework\x1b[0m         \x1b[1;96m║\x1b[0m");
  println!("\x1b[1;96m╚══════════════════════════════════════════════════════════════════════════╝\x1b[0m\n");

  // ─── Connect to remote storage API (fly.dev deployed server) ──────────────
  let server_url = "https://raxc-0g-agent-framework-j43hng.fly.dev";
  println!("\x1b[33m[*] Connecting to api_0g_storage server ({})...\x1b[0m", server_url);
  let remote_storage = RemoteOgStorageClient::new(server_url);

  let loaded = remote_storage.health().await
    .map_err(|e| anyhow::anyhow!(
      "api_0g_storage server not reachable: {}\n\
      → URL: {}", e, server_url
    ))?;

  println!("\x1b[92m[✓] Storage server online — {} exploits loaded\x1b[0m\n", loaded);

  // ─── Initialize 0G Compute client ────────────────────────────────────────────
  let compute = Arc::new(build_og_compute()?);

  // ─── Create AgentCore (remote mode — no local storage download) ──────────────
  let mut core = AgentCore::new_remote((*compute).clone());

  // ─── Register tools ──────────────────────────────────────────────────────────
  println!("\x1b[33m[*] Registering tools to ToolRegistry...\x1b[0m");
  core.tools.register(Box::new(RaxcAnalyzerRemote::new(remote_storage, (*compute).clone())));
  core.tools.register(Box::new(GasAnalyzerTool::new()));
  core.tools.register(Box::new(PatternDetectorTool::new()));
  core.tools.register(Box::new(FlashLoanTool::new()));
  core.tools.register(Box::new(AccessControlTool::new()));
  core.tools.register(Box::new(ReflectionTool::new(compute.clone())));
  // MemoryTool: reads past audit results from 0G Storage cache (/tmp/raxc_memory/)
  // Uses OgStorageClient::new_empty() — no exploit DB needed, only search_analyses()
  let memory_storage = Arc::new(OgStorageClient::new_empty());
  core.tools.register(Box::new(MemoryTool::new(memory_storage)));
  println!("\x1b[92m[✓] Registered {} tools\x1b[0m\n", core.tools.tool_count());

  // ─── DeFiVault — triggers all 6 tools ────────────────────────────────────────
  // ✅ PatternDetectorTool  : reentrancy (.call before state update)
  // ✅ FlashLoanTool        : getReserves() spot price oracle + flashLoan callback
  // ✅ AccessControlTool    : withdraw() and initialize() missing onlyOwner
  // ✅ GasAnalyzerTool      : array.length in loop, string memory param
  // ✅ RaxcAnalyzerRemote   : RAG match against 722 real exploits
  // ✅ ReflectionTool       : 0G Compute self-critique of consensus result
  let default_contract = r#"
pragma solidity ^0.7.0;

contract DeFiVault {
    mapping(address => uint256) public balances;
    address[] public depositors;
    address public owner;
    bool private initialized;

    // ❌ AccessControl: no initializer guard, callable multiple times
    function initialize(address _owner) external {
        owner = _owner;
    }

    function deposit() external payable {
        balances[msg.sender] += msg.value;
        depositors.push(msg.sender);
    }

    // ❌ Reentrancy: external call before state update
    // ❌ AccessControl: no onlyOwner guard on withdraw
    function withdraw() external {
        uint256 amount = balances[msg.sender];
        require(amount > 0, "Nothing to withdraw");
        (bool ok, ) = msg.sender.call{value: amount}("");
        require(ok, "Transfer failed");
        balances[msg.sender] = 0;
    }

    // ❌ FlashLoan: spot price oracle via getReserves — manipulable in one tx
    function getPrice() external view returns (uint256) {
        (uint112 reserve0, uint112 reserve1,) = IUniswapPair(address(this)).getReserves();
        return uint256(reserve0) * 1e18 / uint256(reserve1);
    }

    // ❌ FlashLoan: flash loan callback with no reentrancy guard
    function executeOperation(uint256 amount) external {
        uint256 price = this.getPrice();
        balances[msg.sender] += price * amount;
    }

    // ❌ Gas: array.length in loop, string memory param
    function distributeRewards(string memory label) external {
        for (uint i = 0; i < depositors.length; i++) {
            balances[depositors[i]] += 100;
        }
    }
}

interface IUniswapPair {
    function getReserves() external view returns (uint112, uint112, uint32);
}
  "#;

  // ─── Load contract (inline code, --file path, or built-in DeFiVault demo) ─────────────
  let (contract_code, contract_name) = if let Ok(code) = std::env::var("RAXC_CONTRACT_CODE") {
    // Extract name from "contract FooBar {" pattern
    let name = code.split_whitespace()
      .skip_while(|w| *w != "contract")
      .nth(1)
      .map(|s| s.trim_matches(|c: char| !c.is_alphanumeric() && c != '_').to_string())
      .filter(|s| !s.is_empty())
      .unwrap_or_else(|| "Contract".to_string());
    println!("\x1b[33m[*]\x1b[0m Analyzing inline contract: \x1b[97m{}\x1b[0m", name);
    (code, name)
  } else if let Ok(file_path) = std::env::var("RAXC_CONTRACT_FILE") {
    println!("\x1b[33m[*]\x1b[0m Loading contract from: \x1b[97m{}\x1b[0m", file_path);
    let code = std::fs::read_to_string(&file_path)
      .map_err(|e| anyhow::anyhow!("Cannot read '{}': {}", file_path, e))?;
    let name = std::path::Path::new(&file_path)
      .file_stem()
      .and_then(|s| s.to_str())
      .unwrap_or("Contract")
      .to_string();
    (code, name)
  } else {
    println!("\x1b[2m    (no --file given — using built-in DeFiVault demo contract)\x1b[0m");
    (default_contract.to_string(), "DeFiVault".to_string())
  };

  // ─── ERC-8183: Create audit task on-chain (before analysis) ────────────────────
  let task_id_8183: Option<u64> = match create_audit_task(&contract_name).await {
    Ok(id) => {
      println!("\n\x1b[35m[ERC-8183]       Audit task created on 0G Galileo\x1b[0m");
      println!("\x1b[2m    Task ID:     #{}\x1b[0m", id);
      Some(id)
    }
    Err(e) => {
      println!("\n\x1b[2m[ERC-8183] Task creation skipped: {}\x1b[0m", e);
      println!("\x1b[2m    → Set RAXC_AUDIT_TASK_8183_ADDRESS and PRIVATE_KEY to enable\x1b[0m");
      None
    }
  };

  // ─── Run analysis ─────────────────────────────────────────────────────────────
  println!("\n\x1b[33m[*]\x1b[0m Initiating autonomous exploit analysis — 13-phase verification pipeline...\n");
  let result = core.analyze(&contract_code, &contract_name).await?;

  // Save markdown report
  let reports_dir = std::path::Path::new("reports");
  std::fs::create_dir_all(reports_dir)?;
  let report_path = reports_dir.join(&result.filename);
  std::fs::write(&report_path, &result.markdown)?;
  println!("\n\x1b[92m✅ Report saved to: {}\x1b[0m\n", report_path.display());

  println!("\n\x1b[36m╔══════════════════════════════════════════════════════════════════════════╗\x1b[0m");
  println!("\x1b[36m║                  AUTONOMOUS EXPLOIT INTELLIGENCE RESULT                  ║\x1b[0m");
  println!("\x1b[36m╚══════════════════════════════════════════════════════════════════════════╝\x1b[0m\n");

  println!("\x1b[1;96m📊 BASIC DECISION:\x1b[0m");
  println!("  Vulnerability Found:  {}", result.decision.vulnerability_found);
  println!("  Risk Level:          {}", result.decision.risk_level);
  if let Some(vuln) = &result.decision.primary_vulnerability {
    println!("  Vulnerability Type:  {}", vuln);
  }
  println!("  Confidence:          {:.1}%", result.decision.confidence * 100.0);
  println!("  Tool Signals:        {}", result.signals.len());

  println!("\n\x1b[1;96m📈 INTELLIGENCE REPORT:\x1b[0m");
  println!("  Risk Score:          {:.2}%", result.intelligence_report.risk_score * 100.0);
  println!("  Exploitability:      {:.2}%", result.intelligence_report.exploitability_score * 100.0);
  println!("  Attack Likelihood:   {:.2}%", result.intelligence_report.attack_likelihood * 100.0);
  println!("  Classification:      {}", result.intelligence_report.final_classification);

  println!("\n\x1b[1;96m🧪 ATTACK SIMULATION:\x1b[0m");
  println!("  Execution Path:      {} steps", result.attack_simulation.execution_path.len());
  println!("  State Transitions:   {} tracked", result.attack_simulation.state_transitions.len());
  println!("  Attacker Type:       {}", result.attack_simulation.attacker_model.attacker_type);
  println!("  Exploit Status:      {}", result.attack_simulation.exploit_verdict.status);
  println!("  Success Probability: {:.1}%", result.attack_simulation.exploit_verdict.success_probability * 100.0);
  println!("  Replay ID:           {}", result.attack_simulation.replay_info.replay_id);

  println!("\n\x1b[1;96m📊 GRAPH CONSTRUCTION — ATTACK MAP ENGINE:\x1b[0m");
  println!("  Graph Nodes:         {}", result.attack_graph.nodes.len());
  println!("  Graph Edges:         {}", result.attack_graph.edges.len());
  println!("  Root Node:           {}", result.attack_graph.root_node);

  println!("\n\x1b[1;96m✅ CONSISTENCY VERIFICATION — GATEKEEPER:\x1b[0m");
  println!("  Simulation Valid:    {}", if result.consistency_check.simulation_valid { "✅ PASS" } else { "❌ FAIL" });
  println!("  Graph Consistent:    {}", if result.consistency_check.graph_consistent { "✅ PASS" } else { "❌ FAIL" });
  println!("  State Correct:       {}", if result.consistency_check.state_correct { "✅ PASS" } else { "❌ FAIL" });
  println!("  Tool Conflict:       {}", if result.consistency_check.tool_conflict { "⚠️  YES" } else { "✅ NO" });
  println!("  Consistency Score:   {:.2}%", result.consistency_check.consistency_score * 100.0);

  println!("\n\x1b[1;96m🎯 FINAL DECISION — SOLE AUTHORITY:\x1b[0m");
  println!("  Final Verdict:       {}", result.final_decision.final_verdict);
  println!("  Final Confidence:    {:.2}%", result.final_decision.final_confidence * 100.0);
  println!("  Final Attack Prob:   {:.2}%", result.final_decision.final_attack_probability * 100.0);
  println!("  Final Risk Score:    {:.2}%", result.final_decision.final_risk_score * 100.0);

  println!("\n\x1b[1;96m🔐 ATTESTATION — CRYPTOGRAPHIC PROOF:\x1b[0m");
  println!("  Replay ID:           {}", result.attestation.replay_id);
  println!("  Seed:                {}", result.attestation.seed);
  println!("  Trace Hash:          {}", result.attestation.execution_trace_hash);
  println!("  Timestamp:           {}", result.attestation.timestamp);
  println!("  Verdict:             {}", result.attestation.final_verdict);

  println!("\n\x1b[1;35m[🧠 LLM EXPLANATION]\x1b[0m");
  println!("\x1b[97m{}\x1b[0m", result.explanation);

  println!("\n\x1b[36m╔════════════════════════════════════════════════════════════════════════════╗\x1b[0m");
  println!("\x1b[36m║         AUTONOMOUS ENGINE — REMOTE STORAGE MODE COMPLETE                   ║\x1b[0m");
  println!("\x1b[36m╠════════════════════════════════════════════════════════════════════════════╣\x1b[0m");
  println!("\x1b[36m║\x1b[0m  \x1b[92m✓\x1b[0m No local 0G Storage download — instant startup                          \x1b[36m║\x1b[0m");
  println!("\x1b[36m║\x1b[0m  \x1b[92m✓\x1b[0m Queries api_0g_storage server (<10ms per lookup)                        \x1b[36m║\x1b[0m");
  println!("\x1b[36m║\x1b[0m  \x1b[92m✓\x1b[0m Same 13-phase autonomous pipeline, full attestation                     \x1b[36m║\x1b[0m");
  println!("\x1b[36m║\x1b[0m  \x1b[92m✓\x1b[0m 777 real DeFi exploits as vector DB (loaded once by server)             \x1b[36m║\x1b[0m");
  println!("\x1b[36m╚════════════════════════════════════════════════════════════════════════════╝\x1b[0m");

  // ─── ERC-7857: Record audit result on-chain (0G Galileo) ─────────────────────
  let erc7857_tx = match update_agent_nft(&result, &contract_name).await {
    Ok(tx) => Some(tx),
    Err(e) => {
      println!("\n\x1b[31m[!] ERC-7857 update skipped: {}\x1b[0m", e);
      println!("\x1b[2m    → Set RAXC_AGENT_NFT_ADDRESS, RAXC_AGENT_TOKEN_ID, PRIVATE_KEY to enable\x1b[0m");
      None
    }
  };

  // ─── ERC-8183: Finalize audit task with proof ─────────────────────────────────
  let erc8183_tx = if let Some(task_id) = task_id_8183 {
    match finalize_audit_task(task_id, &result, &contract_name).await {
      Ok(tx) => Some(tx),
      Err(e) => {
        println!("\n\x1b[31m[!] ERC-8183 finalize skipped: {}\x1b[0m", e);
        None
      }
    }
  } else { None };

  // ─── Append on-chain proof section to saved report ────────────────────────────
  const EXPLORER: &str = "https://chainscan.0g.ai/tx/";
  let fmt_tx = |tx: Option<&str>| -> String {
    match tx {
      Some(hash) => format!("[{}]({}{})", hash, EXPLORER, hash.trim_start_matches("0x")),
      None => "—".to_string(),
    }
  };
  let chain_proof = format!(
    "\n\n---\n\n## 🔗 On-Chain Proof (0G Mainnet)\n\n\
| Field | Value |\n\
|-------|-------|\n\
| 0G Storage — JSON Summary | `{}` |\n\
| 0G Storage — Full Report  | `{}` |\n\
| Attestation Replay ID     | `{}` |\n\
| Execution Trace Hash      | `{}` |\n\
| ERC-7857 Intelligence TX  | {} |\n\
| ERC-8183 Finalize TX      | {} |\n\
| Chain                     | [0G Mainnet (Chain 16661)](https://chainscan.0g.ai) |\n",
    if result.storage_root_hash.is_empty() { "—".to_string() } else { result.storage_root_hash.clone() },
    if result.report_root_hash.is_empty() { "—".to_string() } else { result.report_root_hash.clone() },
    result.attestation.replay_id,
    result.attestation.execution_trace_hash,
    fmt_tx(erc7857_tx.as_deref()),
    fmt_tx(erc8183_tx.as_deref()),
  );
  if let Err(e) = std::fs::OpenOptions::new().append(true).open(&report_path)
    .and_then(|mut f| { use std::io::Write; f.write_all(chain_proof.as_bytes()) }) {
    println!("[!] Could not append chain proof to report: {}", e);
  }

  Ok(())
}

/// Call update(tokenId, IntelligentData[]) on the deployed RaxcAgentNFT contract.
///
/// Pure ethers-rs implementation — no Foundry/cast required.
///   function update(uint256 tokenId, IntelligentData[] calldata newDatas)
///   IntelligentData = (string dataDescription, bytes32 dataHash)
///
/// Env vars required:
///   RAXC_AGENT_NFT_ADDRESS  — deployed contract address (0x...)
///   RAXC_AGENT_TOKEN_ID     — token ID (default: 0)
///   PRIVATE_KEY             — agent wallet private key (0x...)
///   OG_RPC_URL              — 0G Mainnet RPC (default: https://evmrpc.0g.ai)
async fn update_agent_nft(result: &raxc::AnalysisResult, contract_name: &str) -> anyhow::Result<String> {
  let contract_addr = std::env::var("RAXC_AGENT_NFT_ADDRESS")
    .map_err(|_| anyhow::anyhow!("RAXC_AGENT_NFT_ADDRESS not set"))?;
  let token_id: u64 = std::env::var("RAXC_AGENT_TOKEN_ID")
    .unwrap_or_else(|_| "0".to_string())
    .parse()?;
  let private_key = std::env::var("PRIVATE_KEY")
    .map_err(|_| anyhow::anyhow!("PRIVATE_KEY not set"))?;
  let rpc_url = std::env::var("OG_RPC_URL")
    .unwrap_or_else(|_| "https://evmrpc.0g.ai".to_string());

  // Query on-chain Updated events to get current audit number
  let provider = Provider::<Http>::try_from(rpc_url.as_str())?;
  let contract_addr: Address = contract_addr.parse()?;
  let event_sig = H256::from(ethers::utils::keccak256(
    b"Updated(uint256,(string,bytes32)[],(string,bytes32)[])"
  ));
  let filter = Filter::new()
    .address(contract_addr)
    .topic0(event_sig)
    .from_block(0u64)
    .to_block(ethers::types::BlockNumber::Latest);
  let past_count = provider.get_logs(&filter).await.unwrap_or_default().len();
  let audit_number = past_count + 1;

  // Build description
  let description = format!(
    "RAXC Audit #{}: {} | {} | {} | {:.0}% confidence | {}",
    audit_number,
    contract_name,
    result.decision.primary_vulnerability.as_deref().unwrap_or("No vuln"),
    result.decision.risk_level,
    result.decision.confidence * 100.0,
    result.attestation.replay_id,
  );

  // Resolve root hash → bytes32
  let root_hash_str = if !result.storage_root_hash.is_empty() {
    result.storage_root_hash.clone()
  } else {
    let h = &result.attestation.execution_trace_hash;
    if h.starts_with("0x") { h.clone() } else { format!("0x{}", h) }
  };
  let hash_hex = root_hash_str.trim_start_matches("0x");
  let hash_hex_padded = format!("{:0>64}", hash_hex);
  let hash_bytes = hex::decode(&hash_hex_padded[..64])?;
  let mut data_hash = [0u8; 32];
  data_hash.copy_from_slice(&hash_bytes);

  println!("\n\x1b[35m[ERC-7857]       Updating agent intelligence on 0G Mainnet...\x1b[0m");
  println!("\x1b[2m    Contract:    {}\x1b[0m", contract_addr);
  println!("\x1b[2m    Agent NFT:   Token #{} (Update #{} on this NFT)\x1b[0m", token_id, audit_number);
  println!("\x1b[2m    Description: {}\x1b[0m", description);
  println!("\x1b[2m    Data Hash:   0x{}\x1b[0m", hash_hex_padded);

  // ── ABI-encode update(uint256, (string,bytes32)[]) ────────────────────────
  // Selector: keccak256("update(uint256,(string,bytes32)[])")[0..4]
  let selector = {
    use ethers::utils::keccak256;
    let sig = b"update(uint256,(string,bytes32)[])";
    keccak256(sig)[..4].to_vec()
  };

  // Encode params: (uint256 tokenId, tuple[] newDatas)
  // tuple = (string dataDescription, bytes32 dataHash)
  let inner_tuple = Token::Tuple(vec![
    Token::String(description.clone()),
    Token::FixedBytes(data_hash.to_vec()),
  ]);
  let params = abi::encode(&[
    Token::Uint(U256::from(token_id)),
    Token::Array(vec![inner_tuple]),
  ]);

  let mut calldata = selector;
  calldata.extend_from_slice(&params);

  // ── Build signer + provider ───────────────────────────────────────────────
  let wallet: LocalWallet = private_key.trim_start_matches("0x")
    .parse::<ethers::signers::LocalWallet>()?
    .with_chain_id(16661u64);
  let client = Arc::new(SignerMiddleware::new(provider, wallet.clone()));

  let tx = TransactionRequest::new()
    .to(contract_addr)
    .data(Bytes::from(calldata))
    .gas_price(3_000_000_000u64)  // 3 gwei — legacy tx for 0G Mainnet
    .chain_id(16661u64);

  let pending = client.send_transaction(tx, None).await
    .map_err(|e| anyhow::anyhow!("send_transaction failed: {}", e))?;

  println!("\x1b[35m[ERC-7857]       Intelligence updated on-chain (chain 16661)\x1b[0m");
  let tx_hash = format!("0x{:x}", pending.tx_hash());
  println!("    TX:  \x1b[92m{}\x1b[0m", tx_hash);
  println!("    URL: \x1b[94mhttps://chainscan.0g.ai/tx/{}\x1b[0m", tx_hash);
  println!("\x1b[2m    Audit trace committed to 0G Mainnet\x1b[0m");

  Ok(tx_hash)
}

// ERC-8183 functions live in raxc::erc8183 (src/erc8183.rs)
// imported above via: use raxc::{create_audit_task, finalize_audit_task}
