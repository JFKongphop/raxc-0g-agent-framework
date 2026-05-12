/*!
RAXC Analysis Tools — Multi-tool orchestration for smart contract vulnerability detection.

These tools are plugged into the agent framework for comprehensive analysis:
- GasAnalyzerTool: Identifies gas optimization opportunities
- PatternDetectorTool: Detects common vulnerability patterns using regex/static analysis
- FlashLoanTool: Detects flash loan and price oracle attack surfaces
- AccessControlTool: Deep access control and privilege escalation checks
- ReflectionTool: LLM self-critique via 0G Compute — removes hallucinated fixes
- MemoryTool: Loads past audit sessions from 0G Storage for persistent memory
*/

use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;

use crate::agent::{Tool, ToolSignal};
use crate::og_compute::OgComputeClient;
use crate::og_storage::OgStorageClient;

// ─── Gas Analyzer Tool ────────────────────────────────────────────────────────

/// Static analyzer for gas optimization opportunities
pub struct GasAnalyzerTool;

impl GasAnalyzerTool {
  pub fn new() -> Self {
    Self
  }
}

#[async_trait]
impl Tool for GasAnalyzerTool {
  async fn execute(&self, contract: &str) -> Result<ToolSignal> {
    let mut findings = Vec::new();

    // Check for common gas inefficiencies
    if contract.contains("for (") && contract.contains(".length") {
      findings.push("⛽ Gas: Cache array length in loops to save gas");
    }

    if contract.contains("uint8") || contract.contains("uint16") {
      findings.push("⛽ Gas: Consider using uint256 for storage (cheaper in EVM)");
    }

    if contract.contains("public ") && contract.contains("returns") {
      findings.push("⛽ Gas: Consider using 'external' instead of 'public' for external-only functions");
    }

    if contract.contains("string memory") || contract.contains("bytes memory") {
      findings.push("⛽ Gas: Dynamic types in memory can be expensive - consider calldata for read-only params");
    }

    if contract.contains("storage") && contract.contains("memory") {
      findings.push("⛽ Gas: Minimize storage reads - cache storage variables in memory when accessed multiple times");
    }

    let evidence = if findings.is_empty() {
      "**Gas Analysis:** No major gas optimization opportunities detected.".to_string()
    } else {
      format!(
        "**Gas Analysis:**\n\nFound {} potential gas optimizations:\n\n{}",
        findings.len(),
        findings.iter().map(|f| format!("- {}", f)).collect::<Vec<_>>().join("\n")
      )
    };

    // Gas issues are not security vulnerabilities
    Ok(ToolSignal {
      id: "GasAnalyzerTool#1".to_string(),
      tool_name: "GasAnalyzerTool".to_string(),
      vulnerability: None,
      severity: None,
      confidence: 0.60,  // Lower confidence since gas != security
      evidence,
    })
  }

  fn name(&self) -> &str {
    "GasAnalyzerTool"
  }
}

// ─── Pattern Detector Tool ────────────────────────────────────────────────────

/// Pattern-based static analyzer for common vulnerabilities
pub struct PatternDetectorTool;

impl PatternDetectorTool {
  pub fn new() -> Self {
    Self
  }
}

#[async_trait]
impl Tool for PatternDetectorTool {
  async fn execute(&self, contract: &str) -> Result<ToolSignal> {
    let mut patterns = Vec::new();
    let mut vulnerability_type = None;
    let mut severity = None;

    // Reentrancy patterns
    if contract.contains(".call{value:") || contract.contains(".call(") {
      if let Some(idx) = contract.find(".call") {
        let before = &contract[..idx];
        let after = &contract[idx..];
        
        // Check if state update happens after the call
        if after.contains("=") && !before.contains("nonReentrant") {
          patterns.push("🚨 Pattern: External call detected - check for reentrancy (CEI pattern required)");
          vulnerability_type = Some("Reentrancy".to_string());
          severity = Some("High".to_string());
        }
      }
    }

    // Unchecked return value
    if contract.contains(".transfer(") || contract.contains(".send(") {
      patterns.push("⚠️  Pattern: Using transfer/send - consider using call with return value check");
      if vulnerability_type.is_none() {
        vulnerability_type = Some("Unchecked Return Value".to_string());
        severity = Some("Medium".to_string());
      }
    }

    // Delegatecall usage
    if contract.contains("delegatecall") {
      patterns.push("🚨 Pattern: delegatecall detected - ensure destination is trusted (storage collision risk)");
      if vulnerability_type.is_none() {
        vulnerability_type = Some("Delegatecall".to_string());
        severity = Some("Critical".to_string());
      }
    }

    // tx.origin usage
    if contract.contains("tx.origin") {
      patterns.push("🚨 Pattern: tx.origin detected - vulnerable to phishing attacks (use msg.sender)");
      if vulnerability_type.is_none() {
        vulnerability_type = Some("Access Control".to_string());
        severity = Some("High".to_string());
      }
    }

    // Timestamp dependence
    if contract.contains("block.timestamp") || contract.contains("now") {
      patterns.push("⚠️  Pattern: Timestamp usage detected - can be manipulated by miners (15-second window)");
      if vulnerability_type.is_none() {
        vulnerability_type = Some("Timestamp Dependence".to_string());
        severity = Some("Medium".to_string());
      }
    }

    // Unprotected selfdestruct
    if contract.contains("selfdestruct") && !contract.contains("onlyOwner") {
      patterns.push("🚨 Pattern: selfdestruct without access control - critical vulnerability");
      vulnerability_type = Some("Access Control".to_string());
      severity = Some("Critical".to_string());
    }

    // Integer overflow (if old Solidity)
    if contract.contains("pragma solidity") {
      if let Some(version_line) = contract.lines().find(|l| l.contains("pragma solidity")) {
        if version_line.contains("^0.7") || version_line.contains("^0.6") || version_line.contains("^0.5") {
          if !contract.contains("SafeMath") && (contract.contains("+=") || contract.contains("-=") || contract.contains("*=")) {
            patterns.push("⚠️  Pattern: Arithmetic operations in Solidity <0.8 without SafeMath - overflow risk");
            if vulnerability_type.is_none() {
              vulnerability_type = Some("Integer Overflow".to_string());
              severity = Some("High".to_string());
            }
          }
        }
      }
    }

    let evidence = if patterns.is_empty() {
      "**Pattern Analysis:** No common vulnerability patterns detected.".to_string()
    } else {
      format!(
        "**Pattern Analysis:**\n\nDetected {} vulnerability patterns:\n\n{}",
        patterns.len(),
        patterns.iter().map(|p| format!("- {}", p)).collect::<Vec<_>>().join("\n")
      )
    };

    let confidence = if vulnerability_type.is_some() {
      0.70  // Pattern matching has decent confidence
    } else {
      0.50  // No vulnerability detected
    };

    Ok(ToolSignal {
      id: "PatternDetectorTool#1".to_string(),
      tool_name: "PatternDetectorTool".to_string(),
      vulnerability: vulnerability_type,
      severity,
      confidence,
      evidence,
    })
  }

  fn name(&self) -> &str {
    "PatternDetectorTool"
  }
}

// ─── Flash Loan Tool ──────────────────────────────────────────────────────────

/// Detects flash loan attack surfaces and price oracle manipulation risks
pub struct FlashLoanTool;

impl FlashLoanTool {
  pub fn new() -> Self {
    Self
  }
}

#[async_trait]
impl Tool for FlashLoanTool {
  async fn execute(&self, contract: &str) -> Result<ToolSignal> {
    let mut findings = Vec::new();
    let mut vulnerability_type = None;
    let mut severity = None;

    // Flash loan callback patterns
    if contract.contains("flashLoan") || contract.contains("flash_loan") || contract.contains("executeOperation") {
      findings.push("🚨 FlashLoan: Flash loan callback detected — verify state is not manipulable within single tx");
      vulnerability_type = Some("Flash Loan".to_string());
      severity = Some("Critical".to_string());
    }

    // Price oracle relying on spot balanceOf (manipulable in single tx)
    if (contract.contains("balanceOf") || contract.contains("getReserves")) && contract.contains("price") {
      findings.push("🚨 FlashLoan: Spot price oracle detected — use TWAP to prevent single-block manipulation");
      if vulnerability_type.is_none() {
        vulnerability_type = Some("Price Oracle Manipulation".to_string());
        severity = Some("Critical".to_string());
      }
    }

    // AMM price calculation without TWAP
    if contract.contains("getAmountsOut") || contract.contains("getAmountOut") {
      findings.push("⚠️  FlashLoan: AMM price query detected — vulnerable to sandwich and flash loan price manipulation");
      if vulnerability_type.is_none() {
        vulnerability_type = Some("Price Oracle Manipulation".to_string());
        severity = Some("High".to_string());
      }
    }

    // Borrow + operation in same function
    if contract.contains("borrow") && (contract.contains("swap") || contract.contains("liquidate")) {
      findings.push("⚠️  FlashLoan: Borrow + swap/liquidate in same call — verify flash loan atomicity guard");
      if vulnerability_type.is_none() {
        vulnerability_type = Some("Flash Loan".to_string());
        severity = Some("High".to_string());
      }
    }

    let evidence = if findings.is_empty() {
      "**Flash Loan Analysis:** No flash loan or price oracle attack surface detected.".to_string()
    } else {
      format!(
        "**Flash Loan Analysis:**\n\nFound {} flash loan / oracle risk(s):\n\n{}",
        findings.len(),
        findings.iter().map(|f| format!("- {}", f)).collect::<Vec<_>>().join("\n")
      )
    };

    let confidence = if vulnerability_type.is_some() { 0.82 } else { 0.55 };

    Ok(ToolSignal {
      id: "FlashLoanTool#1".to_string(),
      tool_name: "FlashLoanTool".to_string(),
      vulnerability: vulnerability_type,
      severity,
      confidence,
      evidence,
    })
  }

  fn name(&self) -> &str {
    "FlashLoanTool"
  }
}

// ─── Access Control Tool ──────────────────────────────────────────────────────

/// Deep access control and privilege escalation checks
pub struct AccessControlTool;

impl AccessControlTool {
  pub fn new() -> Self {
    Self
  }
}

#[async_trait]
impl Tool for AccessControlTool {
  async fn execute(&self, contract: &str) -> Result<ToolSignal> {
    let mut findings = Vec::new();
    let mut vulnerability_type = None;
    let mut severity = None;

    // Unprotected critical functions
    let critical_fn_patterns = ["withdraw", "transferOwnership", "upgradeTo", "mint", "burn", "setOwner", "initialize"];
    for fname in &critical_fn_patterns {
      if contract.contains(&format!("function {}", fname)) {
        let fn_idx = contract.find(&format!("function {}", fname)).unwrap_or(0);
        let fn_body = &contract[fn_idx..std::cmp::min(fn_idx + 300, contract.len())];
        if !fn_body.contains("onlyOwner") && !fn_body.contains("require(msg.sender") && !fn_body.contains("onlyRole") {
          findings.push(format!("🚨 AccessControl: `{}()` has no owner/role guard — callable by anyone", fname));
          vulnerability_type = Some("Access Control".to_string());
          severity = Some("Critical".to_string());
        }
      }
    }

    // Ownership renounced without timelock
    if contract.contains("renounceOwnership") && !contract.contains("timelock") && !contract.contains("TimeLock") {
      findings.push("⚠️  AccessControl: renounceOwnership without timelock — irreversible ownership loss".to_string());
      if vulnerability_type.is_none() {
        vulnerability_type = Some("Access Control".to_string());
        severity = Some("High".to_string());
      }
    }

    // Initializer without initializer guard
    if contract.contains("function initialize") && !contract.contains("initializer") && !contract.contains("_initialized") {
      findings.push("🚨 AccessControl: initialize() without initializer guard — can be called multiple times".to_string());
      vulnerability_type = Some("Access Control".to_string());
      severity = Some("Critical".to_string());
    }

    // Public setter with no access check
    if contract.contains("function set") {
      let has_any_guard = contract.contains("onlyOwner") || contract.contains("require(msg.sender") || contract.contains("onlyRole");
      if !has_any_guard {
        findings.push("⚠️  AccessControl: setter functions detected with no access control pattern found in contract".to_string());
        if vulnerability_type.is_none() {
          vulnerability_type = Some("Access Control".to_string());
          severity = Some("Medium".to_string());
        }
      }
    }

    let evidence = if findings.is_empty() {
      "**Access Control Analysis:** No access control vulnerabilities detected.".to_string()
    } else {
      format!(
        "**Access Control Analysis:**\n\nFound {} access control issue(s):\n\n{}",
        findings.len(),
        findings.iter().map(|f| format!("- {}", f)).collect::<Vec<_>>().join("\n")
      )
    };

    let confidence = if vulnerability_type.is_some() { 0.85 } else { 0.60 };

    Ok(ToolSignal {
      id: "AccessControlTool#1".to_string(),
      tool_name: "AccessControlTool".to_string(),
      vulnerability: vulnerability_type,
      severity,
      confidence,
      evidence,
    })
  }

  fn name(&self) -> &str {
    "AccessControlTool"
  }
}

// ─── Reflection Tool ──────────────────────────────────────────────────────────

/// LLM self-critique via 0G Compute — removes hallucinated fixes, verifies confidence
pub struct ReflectionTool {
  compute: Arc<OgComputeClient>,
}

impl ReflectionTool {
  pub fn new(compute: Arc<OgComputeClient>) -> Self {
    Self { compute }
  }
}

#[async_trait]
impl Tool for ReflectionTool {
  /// input = initial analysis text (vulnerability type + evidence from other tools)
  async fn execute(&self, input: &str) -> Result<ToolSignal> {
    let prompt = format!(
      "You are a senior smart contract security auditor performing self-critique.\n\
       Review this vulnerability analysis and:\n\
       1. Remove any hallucinated or unsupported fix recommendations\n\
       2. Verify the vulnerability is real based on the evidence\n\
       3. Adjust confidence: increase if evidence is strong, decrease if speculative\n\
       4. Output ONLY: VERDICT: <CONFIRMED|REDUCED|REJECTED> | CONFIDENCE: <0-100> | NOTE: <one sentence>\n\n\
       Analysis to review:\n{}", input
    );

    let response = self.compute.infer_with_max_tokens(&prompt, Some(256)).await?;

    // Parse structured response
    let verdict = if response.contains("CONFIRMED") {
      "CONFIRMED"
    } else if response.contains("REJECTED") {
      "REJECTED"
    } else {
      "REDUCED"
    };

    // Extract confidence from response
    let refined_confidence = if let Some(idx) = response.find("CONFIDENCE:") {
      let after = &response[idx + 11..];
      let num_str: String = after.chars().take_while(|c| c.is_ascii_digit()).collect();
      num_str.parse::<f64>().unwrap_or(70.0) / 100.0
    } else {
      0.70
    };

    let evidence = format!(
      "**Reflection (0G Compute self-critique):**\n\nVerdict: {}\n\n{}",
      verdict, response.trim()
    );

    Ok(ToolSignal {
      id: "ReflectionTool#1".to_string(),
      tool_name: "ReflectionTool".to_string(),
      vulnerability: None, // Reflection refines, doesn't add new vuln
      severity: None,
      confidence: refined_confidence,
      evidence,
    })
  }

  fn name(&self) -> &str {
    "ReflectionTool"
  }
}

// ─── Memory Tool ──────────────────────────────────────────────────────────────

/// Loads past audit sessions from 0G Storage for persistent memory context
pub struct MemoryTool {
  storage: Arc<OgStorageClient>,
}

impl MemoryTool {
  pub fn new(storage: Arc<OgStorageClient>) -> Self {
    Self { storage }
  }
}

#[async_trait]
impl Tool for MemoryTool {
  /// input = contract source code — used to find similar past audits
  async fn execute(&self, contract: &str) -> Result<ToolSignal> {
    let past_analyses = self.storage.search_analyses(contract).await.unwrap_or_default();

    let evidence = if past_analyses.is_empty() {
      "**Memory (0G Storage):** No past audit sessions found — this is a first-time analysis.".to_string()
    } else {
      format!(
        "**Memory (0G Storage):** Retrieved {} past audit session(s):\n\n{}",
        past_analyses.len(),
        past_analyses
          .iter()
          .enumerate()
          .map(|(i, s)| format!("- [Session {}] {}", i + 1, s))
          .collect::<Vec<_>>()
          .join("\n")
      )
    };

    let confidence = if past_analyses.is_empty() { 0.50 } else { 0.75 };

    Ok(ToolSignal {
      id: "MemoryTool#1".to_string(),
      tool_name: "MemoryTool".to_string(),
      vulnerability: None,
      severity: None,
      confidence,
      evidence,
    })
  }

  fn name(&self) -> &str {
    "MemoryTool"
  }
}
