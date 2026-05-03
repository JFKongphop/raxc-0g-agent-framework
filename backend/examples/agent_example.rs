/*!
Example: RAXC Multi-Agent Framework (Step 9.9)

This demonstrates the Step 9.9 Deterministic Exploit Execution + Verification Framework with:
  ✓ ToolRegistry (pluggable tools)
  ✓ Multi-Agent Reasoning (AgentVote)
  ✓ ConsensusEngine (vote aggregation)
  ✓ MemoryLayer (0G Storage integration)
  ✓ Intelligence Layer (risk scoring, exploitability, tool trust)
  ✓ Attack Simulation Engine (execution path, state transitions, attacker modeling)
  ✓ GraphConstructionEngine (deterministic attack graph) [NEW Step 9.9]
  ✓ ConsistencyEngine (verification layer) [NEW Step 9.9]
  ✓ FinalDecisionEngine (single authority) [NEW Step 9.9]
  ✓ AttestationEngine (verifiable proof) [NEW Step 9.9]
  ✓ LLM Layer (0G Compute explanations)
  ✓ ReportEngine (comprehensive markdown report)
  ✓ AgentCore (framework orchestrator)

Architecture (Step 9.9 - Enhanced Verification Pipeline):
  Phase 1: ToolRegistry executes all tools in parallel
  Phase 1.5: SignalNormalizer filters and validates tool signals
  Phase 2: Convert tool signals to agent votes (multi-agent reasoning)
  Phase 3: ConsensusEngine aggregates votes using weighted consensus
  Phase 4: MemoryLayer stores result to 0G Storage
  Phase 4.5: Intelligence Layer calculates risk scores
  Phase 4.75: Attack Simulation Engine generates execution path
  Phase 4.8: GraphConstructionEngine builds deterministic attack graph [NEW]
  Phase 4.85: ConsistencyEngine verifies simulation correctness [NEW]
  Phase 4.9: FinalDecisionEngine makes authoritative decision [NEW]
  Phase 4.95: AttestationEngine generates verifiable proof [NEW]
  Phase 5: LLM Layer generates explanation using 0G Compute
  Phase 6: ReportEngine produces markdown report with all verification data

Run: cargo run --example agent_example
*/

use anyhow::Result;
use raxc::{
  build_og_compute, build_og_storage, load_env, 
  AgentCore, RaxcAnalyzer, GasAnalyzerTool, PatternDetectorTool
};

#[tokio::main]
async fn main() -> Result<()> {
  // Load environment variables
  load_env();

  // Enable OpenAI embeddings
  std::env::set_var("USE_OPENAI_EMBEDDING", "true");

  println!("╔══════════════════════════════════════════════════════════════════════════╗");
  println!("║    RAXC Multi-Agent Framework (Step 9.9)                                ║");
  println!("║    Deterministic Exploit Execution + Verification Framework             ║");
  println!("╚══════════════════════════════════════════════════════════════════════════╝\n");

  // Initialize 0G clients
  let storage = build_og_storage().await?;
  let compute = build_og_compute()?;

  // Create AgentCore (Step 9.9 framework with verification)
  let mut core = AgentCore::new(storage.clone(), compute.clone());

  // Register tools using ToolRegistry
  println!("[*] Registering tools to ToolRegistry...");
  core.tools.register(Box::new(RaxcAnalyzer::new(storage, compute)));
  core.tools.register(Box::new(GasAnalyzerTool::new()));
  core.tools.register(Box::new(PatternDetectorTool::new()));
  println!("[✓] Registered {} tools\n", core.tools.tool_count());

  // VulnerableVault contract to analyze
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

  // Run Step 9.9 multi-agent framework analysis with verification
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
  println!("║         STEP 9.9 DETERMINISTIC EXPLOIT EXECUTION + VERIFICATION          ║");
  println!("║                          FRAMEWORK COMPLETE                              ║");
  println!("╠══════════════════════════════════════════════════════════════════════════╣");
  println!("║  ✓ ToolRegistry (pluggable tool system)                                 ║");
  println!("║  ✓ SignalNormalizer (validation & filtering)                            ║");
  println!("║  ✓ Multi-Agent Reasoning (tool signals → agent votes)                   ║");
  println!("║  ✓ ConsensusEngine (weighted vote aggregation)                          ║");
  println!("║  ✓ MemoryLayer (0G Storage integration)                                 ║");
  println!("║  ✓ Intelligence Layer (risk scoring & exploitability)                   ║");
  println!("║  ✓ Attack Simulation Engine (execution path & state modeling)           ║");
  println!("║  ✓ GraphConstructionEngine (deterministic attack graph) [NEW 9.9]       ║");
  println!("║  ✓ ConsistencyEngine (simulation verification) [NEW 9.9]                ║");
  println!("║  ✓ FinalDecisionEngine (single authority) [NEW 9.9]                     ║");
  println!("║  ✓ AttestationEngine (verifiable proof) [NEW 9.9]                       ║");
  println!("║  ✓ LLM Layer (0G Compute for explanations)                              ║");
  println!("║  ✓ ReportEngine (comprehensive markdown report)                         ║");
  println!("║  ✓ AgentCore (orchestrates full 13-phase pipeline)                      ║");
  println!("╠══════════════════════════════════════════════════════════════════════════╣");
  println!("║  🔐 VERIFIABLE: Deterministic replay with attestation proof             ║");
  println!("║  📊 GRAPH-BASED: Attack flow modeled as execution graph                 ║");
  println!("║  ✅ CONSISTENT: Validation across tools, simulation, and graph          ║");
  println!("║  🎯 DECISIVE: Single source of truth for final verdict                  ║");
  println!("╚══════════════════════════════════════════════════════════════════════════╝");

  Ok(())
}
