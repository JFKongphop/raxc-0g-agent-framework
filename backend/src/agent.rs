/*!
Agent abstraction for RAXC vulnerability analysis.

This module provides a high-level Agent interface that encapsulates
the complete analysis workflow using 0G Storage (memory) and 0G Compute (reasoning).
*/

use anyhow::Result;
use reqwest::Client;

use crate::{analyze, match_functions, OgComputeClient, OgStorageClient};

/// Agent with 0G-powered memory and compute capabilities
pub struct Agent {
  http: Client,
  memory: OgStorageClient,
  compute: OgComputeClient,
}

impl Agent {
  /// Create a new Agent with 0G Storage and 0G Compute clients
  pub fn new(memory: OgStorageClient, compute: OgComputeClient) -> Self {
    Self {
      http: Client::new(),
      memory,
      compute,
    }
  }

  /// Run complete vulnerability analysis on a contract
  ///
  /// Workflow:
  /// 1. Load exploits from 0G Storage
  /// 2. Run similarity matching (in Rust)
  /// 3. Send reasoning prompt to 0G Compute
  /// 4. Return security report
  pub async fn run(&self, contract: &str) -> Result<String> {
    // Run full analysis
    let (report, _results) = analyze(&self.http, &self.memory, &self.compute, contract).await?;
    Ok(report)
  }

  /// Run analysis with function-level matching
  pub async fn run_with_functions(&self, contract: &str) -> Result<(String, Vec<crate::FunctionMatch>)> {
    // Run full analysis
    let (report, _results) = analyze(&self.http, &self.memory, &self.compute, contract).await?;
    
    // Run function-level matching
    let func_matches = match_functions(&self.http, &self.memory, contract, 3).await?;
    
    Ok((report, func_matches))
  }
}
