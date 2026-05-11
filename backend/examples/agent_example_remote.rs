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
use raxc::{
  build_og_compute, load_env,
  AgentCore, RaxcAnalyzerRemote, GasAnalyzerTool, PatternDetectorTool,
  RemoteOgStorageClient,
};

#[tokio::main]
async fn main() -> Result<()> {
  // Load environment variables
  load_env();

  // Enable OpenAI embeddings
  std::env::set_var("USE_OPENAI_EMBEDDING", "true");

  println!("╔══════════════════════════════════════════════════════════════════════════╗");
  println!("║    RAXC Multi-Agent Framework (Step 9.9) — Remote Storage Mode          ║");
  println!("║    Deterministic Exploit Execution + Verification Framework             ║");
  println!("╚══════════════════════════════════════════════════════════════════════════╝\n");

  // ─── Connect to remote storage API (no local download) ───────────────────────
  println!("[*] Connecting to api_0g_storage server (http://localhost:3001)...");
  let remote_storage = RemoteOgStorageClient::local();

  let loaded = remote_storage.health().await
    .map_err(|e| anyhow::anyhow!(
      "api_0g_storage server not reachable: {}\n\
      → Start it first: cargo run --bin api_0g_storage", e
    ))?;

  println!("[✓] Storage server online — {} exploits loaded\n", loaded);

  // ─── Initialize 0G Compute client ────────────────────────────────────────────
  let compute = build_og_compute()?;

  // ─── Create AgentCore (remote mode — no local storage download) ──────────────
  let mut core = AgentCore::new_remote(compute.clone());

  // ─── Register tools ──────────────────────────────────────────────────────────
  println!("[*] Registering tools to ToolRegistry...");
  core.tools.register(Box::new(RaxcAnalyzerRemote::new(remote_storage, compute)));
  core.tools.register(Box::new(GasAnalyzerTool::new()));
  core.tools.register(Box::new(PatternDetectorTool::new()));
  println!("[✓] Registered {} tools\n", core.tools.tool_count());

  // ─── VulnerableVault contract ─────────────────────────────────────────────────
  let contract = r#"
pragma solidity ^0.7.0;

contract VulnerableVault {
    mapping(address => uint256) public balances;

    function deposit() external payable {
        balances[msg.sender] += msg.value;
    }

    function withdraw() external {
        uint256 amount = balances[msg.sender];
        require(amount > 0, "Nothing to withdraw");
        // VULNERABILITY: external call before state update — reentrancy risk
        (bool ok, ) = msg.sender.call{value: amount}("");
        require(ok, "Transfer failed");
        balances[msg.sender] = 0;  // state updated AFTER the call
    }

    function getPrice() external view returns (uint256) {
        // single-block spot price — manipulable via flash loan
        return address(this).balance;
    }
}
  "#;

  // ─── Run analysis ─────────────────────────────────────────────────────────────
  println!("\n[*] Starting Step 9.9 analysis with full verification pipeline...\n");
  let result = core.analyze(contract, "VulnerableVault").await?;

  // Save markdown report
  std::fs::write(&result.filename, &result.markdown)?;
  println!("\n✅ Report saved to: {}\n", result.filename);

  println!("\n╔══════════════════════════════════════════════════════════════════════════╗");
  println!("║                  STEP 9.9 FRAMEWORK ANALYSIS RESULT                      ║");
  println!("╚══════════════════════════════════════════════════════════════════════════╝\n");

  println!("📊 BASIC DECISION:");
  println!("  Vulnerability Found:  {}", result.decision.vulnerability_found);
  println!("  Risk Level:          {}", result.decision.risk_level);
  if let Some(vuln) = &result.decision.primary_vulnerability {
    println!("  Vulnerability Type:  {}", vuln);
  }
  println!("  Confidence:          {:.1}%", result.decision.confidence * 100.0);
  println!("  Tool Signals:        {}", result.signals.len());

  println!("\n📈 INTELLIGENCE REPORT:");
  println!("  Risk Score:          {:.2}%", result.intelligence_report.risk_score * 100.0);
  println!("  Exploitability:      {:.2}%", result.intelligence_report.exploitability_score * 100.0);
  println!("  Attack Likelihood:   {:.2}%", result.intelligence_report.attack_likelihood * 100.0);
  println!("  Classification:      {}", result.intelligence_report.final_classification);

  println!("\n🧪 ATTACK SIMULATION:");
  println!("  Execution Path:      {} steps", result.attack_simulation.execution_path.len());
  println!("  State Transitions:   {} tracked", result.attack_simulation.state_transitions.len());
  println!("  Attacker Type:       {}", result.attack_simulation.attacker_model.attacker_type);
  println!("  Exploit Status:      {}", result.attack_simulation.exploit_verdict.status);
  println!("  Success Probability: {:.1}%", result.attack_simulation.exploit_verdict.success_probability * 100.0);
  println!("  Replay ID:           {}", result.attack_simulation.replay_info.replay_id);

  println!("\n📊 GRAPH CONSTRUCTION [NEW Step 9.9]:");
  println!("  Graph Nodes:         {}", result.attack_graph.nodes.len());
  println!("  Graph Edges:         {}", result.attack_graph.edges.len());
  println!("  Root Node:           {}", result.attack_graph.root_node);

  println!("\n✅ CONSISTENCY VERIFICATION [NEW Step 9.9]:");
  println!("  Simulation Valid:    {}", if result.consistency_check.simulation_valid { "✅ PASS" } else { "❌ FAIL" });
  println!("  Graph Consistent:    {}", if result.consistency_check.graph_consistent { "✅ PASS" } else { "❌ FAIL" });
  println!("  State Correct:       {}", if result.consistency_check.state_correct { "✅ PASS" } else { "❌ FAIL" });
  println!("  Tool Conflict:       {}", if result.consistency_check.tool_conflict { "⚠️  YES" } else { "✅ NO" });
  println!("  Consistency Score:   {:.2}%", result.consistency_check.consistency_score * 100.0);

  println!("\n🎯 FINAL DECISION [NEW Step 9.9 - SINGLE AUTHORITY]:");
  println!("  Final Verdict:       {}", result.final_decision.final_verdict);
  println!("  Final Confidence:    {:.2}%", result.final_decision.final_confidence * 100.0);
  println!("  Final Attack Prob:   {:.2}%", result.final_decision.final_attack_probability * 100.0);
  println!("  Final Risk Score:    {:.2}%", result.final_decision.final_risk_score * 100.0);

  println!("\n🔐 ATTESTATION [NEW Step 9.9 - VERIFIABLE PROOF]:");
  println!("  Replay ID:           {}", result.attestation.replay_id);
  println!("  Seed:                {}", result.attestation.seed);
  println!("  Trace Hash:          {}", result.attestation.execution_trace_hash);
  println!("  Timestamp:           {}", result.attestation.timestamp);
  println!("  Verdict:             {}", result.attestation.final_verdict);

  println!("\n[🧠 LLM EXPLANATION]");
  println!("{}", result.explanation);

  println!("\n╔══════════════════════════════════════════════════════════════════════════╗");
  println!("║         STEP 9.9 — REMOTE STORAGE MODE COMPLETE                          ║");
  println!("╠══════════════════════════════════════════════════════════════════════════╣");
  println!("║  ✓ No local 0G Storage download — instant startup                       ║");
  println!("║  ✓ Queries api_0g_storage server (<10ms per lookup)                     ║");
  println!("║  ✓ Same Step 9.9 pipeline: 13 phases, full attestation                  ║");
  println!("║  ✓ 777 real DeFi exploits as vector DB (loaded once by server)          ║");
  println!("╚══════════════════════════════════════════════════════════════════════════╝");

  Ok(())
}
