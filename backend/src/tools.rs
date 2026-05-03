/*!
RAXC Analysis Tools — Multi-tool orchestration for smart contract vulnerability detection.

These tools are plugged into the agent framework for comprehensive analysis:
- GasAnalyzerTool: Identifies gas optimization opportunities
- PatternDetectorTool: Detects common vulnerability patterns using regex/static analysis
*/

use anyhow::Result;
use async_trait::async_trait;

use crate::agent::{Tool, ToolSignal};

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
