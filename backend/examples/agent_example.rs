/*!
Example: Using the RAXC Agent for vulnerability analysis.

This demonstrates the high-level Agent API that abstracts away
the complexity of 0G Storage and 0G Compute integration.
*/

use anyhow::Result;
use raxc::{build_og_compute, build_og_storage, load_env, Agent};

#[tokio::main]
async fn main() -> Result<()> {
  // Load environment variables
  load_env();

  // Initialize 0G clients
  let storage = build_og_storage()?;
  let compute = build_og_compute()?;

  // Create agent
  let agent = Agent::new(storage, compute);

  // Example contract to analyze
  let contract = r#"
    pragma solidity ^0.8.0;
    
    contract Example {
      mapping(address => uint256) public balances;
      
      function withdraw() external {
        uint256 amount = balances[msg.sender];
        (bool success, ) = msg.sender.call{value: amount}("");
        require(success);
        balances[msg.sender] = 0; // VULNERABILITY: state update after external call
      }
    }
  "#;

  println!("[*] Running RAXC Agent analysis...\n");

  // Run analysis
  let report = agent.run(contract).await?;

  println!("=== SECURITY REPORT ===");
  println!("{}", report);
  println!("=======================");

  Ok(())
}
