# RAXC — AI-Powered Smart Contract Vulnerability Scanner

[![Crates.io](https://img.shields.io/crates/v/raxc.svg)](https://crates.io/crates/raxc)
[![Documentation](https://docs.rs/raxc/badge.svg)](https://docs.rs/raxc)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

**RAXC** (Retrieval Augmented eXploit Checker) is an autonomous security agent that detects smart contract vulnerabilities by combining real exploit pattern retrieval with agentic reasoning.

Powered by **0G Storage** and **0G Compute**, RAXC analyzes contracts against 777 real-world exploits worth over $4.1 billion in losses.

## Features

- 🧠 **Autonomous Agent Architecture** — LLM-based tool selection, reflection, and reasoning
- 🗄️ **777 Real Exploits** — RAG-based semantic search across DeFiHackLabs dataset
- ⚡ **0G Infrastructure** — Decentralized storage and compute for scalable analysis
- 🔍 **Explainable Results** — Confidence scores with tool-by-tool breakdown
- 🔧 **Extensible** — Add custom security tools with the `Tool` trait
- 📊 **Structured Output** — 13-field `AgentOutput` with risk level, reasoning, and reports

## Quick Start

```rust
use raxc::{Agent, build_og_storage, build_og_compute};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize 0G infrastructure
    let storage = build_og_storage()?;
    let compute = build_og_compute()?;
    
    // Create agent
    let mut agent = Agent::new(storage, compute);
    
    // Analyze contract
    let contract = r#"
        contract VulnerableVault {
            mapping(address => uint) public balances;
            
            function withdraw(uint amount) public {
                require(balances[msg.sender] >= amount);
                (bool success, ) = msg.sender.call{value: amount}("");
                require(success);
                balances[msg.sender] -= amount;
            }
        }
    "#;
    
    let result = agent.analyze(contract, "VulnerableVault").await?;
    
    println!("Vulnerability Found: {}", result.vulnerability_found);
    println!("Risk Level: {}", result.risk_level);
    println!("Confidence: {}%", result.confidence);
    
    Ok(())
}
```

## Environment Setup

```bash
export OPENAI_API_KEY="sk-your-key"           # For embeddings only
export USE_OPENAI_EMBEDDING="true"
export OG_STORAGE_RPC="https://rpc-storage-testnet.0g.ai"
export OG_COMPUTE_ENDPOINT="https://api.compute.testnet.openlayer.network"
```

## Live Demo

- 🌐 **Web Interface:** [https://raxclaw.vercel.app](https://raxclaw.vercel.app)
- 🔌 **API:** [https://raxc-0g-agent-framework.fly.dev](https://raxc-0g-agent-framework.fly.dev)

## Architecture

```
┌─────────────────────────────────────────────┐
│         RAXC Intelligent Agent              │
├─────────────────────────────────────────────┤
│  1. Load Exploit Memory (0G Storage)        │
│  2. LLM Tool Selection (0G Compute)         │
│  3. Execute Selected Tools (parallel)       │
│  4. Intelligent Aggregation                 │
│  5. Reflection Loop (max 2 iterations)      │
│  6. Confidence Breakdown                    │
│  7. Return Structured Output                │
└─────────────────────────────────────────────┘
```

## Adding Custom Tools

```rust
use raxc::Tool;
use async_trait::async_trait;

pub struct MySecurityTool;

#[async_trait]
impl Tool for MySecurityTool {
    fn name(&self) -> &str {
        "MySecurityTool"
    }
    
    async fn execute(&self, input: &str) -> anyhow::Result<String> {
        // Your analysis logic
        Ok("Analysis result".to_string())
    }
}

// Register with agent
agent.add_tool(Box::new(MySecurityTool));
```

## Documentation

For complete documentation, see:
- [Main Repository](https://github.com/kongphop3212/raxc-0g-agent-framework)
- [Technical Docs](https://github.com/kongphop3212/raxc-0g-agent-framework#readme)
- [API Reference](https://docs.rs/raxc)

## License

MIT License — see LICENSE file for details

## Credits

Built on:
- **0G Storage** — Decentralized data availability
- **0G Compute** — Decentralized LLM inference
- **DeFiHackLabs** — Real-world exploit dataset (721 exploits)
- **DeFiVulnLabs** — Vulnerability patterns (56 patterns)
