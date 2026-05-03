# RAXC — AI-Powered DeFi Smart Contract Vulnerability Scanner

> **R**etrieval **A**ugmented e**X**ploit **C**hecker  
> Grant Application One-Pager | May 2026

**RAXC** stands for:

| Letter | Word | Meaning |
|--------|------|---------|
| **R** | Retrieval | Finds the most semantically similar real-world exploits from a vector database |
| **A** | Augmented | Augments LLM analysis with grounded, evidence-based exploit context |
| **X** | eXploit | Focused specifically on DeFi exploit patterns — not generic code review |
| **C** | Checker | Fast, automated pre-deployment security check — not a full audit replacement |

> *"Don't just ask an AI if your contract is safe — ask an AI that has seen 626 real hacks."*

---

## 🚀 RAXC Security Agent

Autonomous vulnerability detection using real exploit patterns.

**Built on 0G Storage + 0G Compute**

→ 777 real exploits  
→ Agentic reasoning + attack simulation  
→ Detect vulnerabilities before deployment  

[Run Demo](#-quick-start-60-seconds) • [Architecture](#️-architecture-0g-aligned) • [Dataset](#-the-problem)

---

## 🧩 Hook

**RAXC**  
*Autonomous vulnerability detection using real exploit patterns*

Powered by **0G Storage** + **0G Compute** + **777 Real Exploits** from DeFiHackLabs

---

## ⚠️ The Problem

**Smart contracts are deployed faster than they are audited.**  
**Most tools miss real-world exploit patterns used in live attacks.**

DeFi protocols have lost over **$4.1 billion** to smart contract exploits — and the same vulnerability patterns keep repeating year after year.

> These numbers are drawn directly from **474 real on-chain exploits** in the RAXC dataset (DeFiHackLabs). An additional 104 exploits with losses in ETH/BNB/tokens are not included, meaning the true total is significantly higher.

| Year | Confirmed USD Lost | Trend |
|------|--------------------|-------|
| 2017 | $30,000,000 | Early days |
| 2018 | $140,000,155 | — |
| 2020 | $20,000,000 | — |
| 2021 | $124,365,000 | ↑ DeFi summer |
| 2022 | $205,809,017 | ↑ Bridge attacks |
| 2023 | $443,980,241 | ↑↑ |
| 2024 | $1,386,601,430 | ↑↑↑ |
| 2025 | $1,777,671,071 | ↑↑↑↑ Worst year ever |
| 2026 | $7,655,193 | (Jan–Apr only) |
| **Total** | **$4,136,086,808** | |

**Average loss per exploit: $11.2 million USD**

The losses are accelerating every year. The same vulnerability types — reentrancy, price manipulation, flash loans, access control — appear across hundreds of incidents. These are **preventable** if caught before deployment.

**The root cause:** Developers and auditors lack fast, evidence-based tools to catch vulnerabilities before deployment. Traditional static analysis tools generate too many false positives and miss novel attack patterns. LLMs alone hallucinate and lack grounding in real exploit data.

---

## 🧠 Solution

**RAXC is an autonomous security agent that detects smart contract vulnerabilities using real exploit memory and agentic reasoning.**

It builds a continuously evolving exploit memory and simulates attacker behavior to uncover vulnerabilities before deployment.

**Built on 0G Storage for exploit memory and 0G Compute for autonomous reasoning.** RAXC leverages decentralized infrastructure to run security analysis at scale.

### Why RAXC is Different

Unlike traditional tools that rely on static rules or generic LLMs that hallucinate, RAXC:

**🧬 Autonomous Agent Architecture**
- **Multi-phase reasoning:** tool selection → execution → aggregation → reflection
- **Self-improving:** reflection loop (max 2 iterations) validates findings and fills gaps
- **Explainable decisions:** every report includes primary signals, supporting evidence, and ignored patterns
- **Confidence scoring:** tool-by-tool breakdown shows exactly why the agent is confident

**🗄️ Continuously Evolving Exploit Memory**
- **777 real exploits** indexed from DeFiHackLabs (626) + DeFiVulnLabs (151)
- **$4.1B+ in losses** analyzed across 474 on-chain exploits
- **Semantic retrieval** using 1536-dimensional embeddings
- **Persistent memory on 0G Storage:** learns from past analyses and adapts over time
- **Vector similarity search:** finds exploits semantically similar to the contract under analysis

**🎯 Simulates Attacker Behavior**
- **Pattern detection:** CEI violations, reentrancy, flash loans, price manipulation
- **Attack path simulation:** tests potential exploit vectors before returning findings
- **RAG-based context:** every analysis grounded in real-world exploit patterns
- **Novel vulnerability detection:** LLM reasoning identifies attacks not in static rule sets

**⚡ Powered by 0G Decentralized Infrastructure**

**0G Storage** — Exploit memory layer
- 777 exploit files with contract code, attack transactions, root cause analysis
- Persistent memory system tracks historical analyses and pattern adaptations
- Immutable, censorship-resistant knowledge base
  
**0G Compute** — Autonomous reasoning layer
- **All analysis and reasoning:** tool selection, reflection, aggregation, vulnerability detection
- **Tool selection:** decides which security tools to run based on contract patterns
- **Reflection:** reviews its own output to validate findings and improve quality
- **Aggregation:** deduplicates findings and calculates confidence scores
- **Model:** qwen/qwen-2.5-7b-instruct (7B parameter instruction-tuned LLM)
  
**OpenAI** — Semantic similarity embeddings only
- Generates 1536-dimensional vectors for exploit pattern matching
- **Note:** OpenAI is only used for embeddings, not LLM reasoning

**🔧 Extensible & Production-Ready**
- **Modular tool system:** easily add custom security analyzers (gas, formal verification, fuzzing)
- **Parallel execution:** multiple tools run concurrently for speed
- **Structured output:** 13-field `AgentOutput` with risk level, confidence, reasoning, similar exploits
- **API-first design:** deploy as a service for CI/CD integration or on-demand analysis

---

## 🧬 How It Works

**1. Retrieval → Fetch real exploit patterns**  
- Semantic search across 777 exploits from DeFiHackLabs
- Find similar vulnerabilities using OpenAI embeddings (1536-dim vectors)
- Stored on 0G Storage for decentralized, persistent memory

**2. Reasoning → AI analyzes contract logic**  
- LLM-based tool selection (RaxcAnalyzer, PatternDetector, GasAnalyzer)
- Intelligent aggregation with deduplication
- Confidence breakdown showing tool contributions

**3. Simulation → Agent tests attack paths**  
- Reflection loop (max 2 iterations) for self-improvement
- Validates findings against known exploit patterns
- Identifies CEI violations, reentrancy, flash loan attacks

**4. Output → Vulnerability report**  
- **Risk level:** Critical, High, Medium, Low, None
- **Confidence score:** 0-100%
- **Similar exploits** + reasoning
- **Markdown report** with actionable recommendations

---

## ⚙️ Architecture (0G Aligned)

**RAG Exploit Database + Agent Framework**

```
┌─────────────────────────────────────────────────────────────┐
│                    RAXC Intelligent Agent                    │
├─────────────────────────────────────────────────────────────┤
│                                                               │
│  1. Load Adaptive Memory (0G Storage)                       │
│     ↓                                                         │
│  2. LLM Tool Selection ──→ ToolSelection                    │
│     ↓                       (selected_tools, reasoning)      │
│  3. Execute Selected Tools (parallel)                        │
│     ↓                                                         │
│  4. Intelligent Aggregation ──→ Structured findings          │
│     ↓                           (deduplication, attribution) │
│  5. Reflection Loop (max 2)                                  │
│     ↓                                                         │
│  6. Confidence Breakdown ──→ ConfidenceBreakdown            │
│     ↓                        (tool contributions + bonus)    │
│  7. Extract Decision ──→ AgentDecision                      │
│     ↓                    (primary signal, evidence, ignored) │
│  8. Return AgentOutput (13 fields)                           │
│                                                               │
└─────────────────────────────────────────────────────────────┘
```

**Key Components:**

- **On-chain contract analysis:** Ethers-rs for blockchain interaction
- **Off-chain reasoning engine:** 0G Compute for LLM inference (qwen-2.5-7b-instruct)
- **Modular security skills:** extensible tool system (RAG, pattern detection, gas analysis)
- **Persistent memory:** 0G Storage for decentralized RAG database (777 exploits)

---

## 🚀 Impact

**From static scanning → autonomous security intelligence**  
**Prevent exploits before deployment.**

### Measurable Impact

| Metric | Value |
|--------|-------|
| **Exploits Loaded** | 777 (DeFiHackLabs + DeFiVulnLabs) |
| **Analysis Time** | 4-13s per contract |
| **Confidence Scores** | 70-95% (tool-based breakdown) |
| **LLM Efficiency** | 1500-3000 tokens per analysis |
| **Detection Rate** | Catches reentrancy, flash loans, price manipulation |

### Real-World Use Cases

✅ **Pre-deployment checks:** fast security scan before mainnet launch  
✅ **Audit preparation:** identify vulnerabilities before formal audit  
✅ **Continuous monitoring:** integrate into CI/CD pipelines  
✅ **Educational tool:** learn from real exploits with context  
✅ **DAO governance:** verify proposals before execution

---

## 🚀 Quick Start (60 Seconds)

### Prerequisites
```bash
# Set environment variables
export OPENAI_API_KEY="sk-your-key-here"  # Only for embeddings/similarity
export USE_OPENAI_EMBEDDING="true"        # Enable semantic search
export OG_STORAGE_RPC="https://rpc-storage-testnet.0g.ai"
export OG_COMPUTE_ENDPOINT="https://api.compute.testnet.openlayer.network"  # All LLM reasoning

# Create memory directory
mkdir -p /tmp/raxc_memory
```

> **Important:** OpenAI is only used for generating embeddings to find similar exploits. All vulnerability analysis, reasoning, and decision-making is performed by 0G Compute (qwen-2.5-7b). The testnet does not yet support embedding models on 0G Compute.

### Run Example
```bash
cd backend
cargo run --example agent_example_openai
```

### Expected Output
```
[*] Running RAXC Intelligent Agent Framework...
[✓] Loaded 777 exploits from DeFiHackLabs

[*] Using LLM for intelligent tool selection...
[✓] Selected 2 tools: RaxcAnalyzer, PatternDetectorTool
[i] Selection reasoning: Contract has external calls, prioritizing reentrancy checks

[*] Phase 1: Executing 2 selected tools in parallel...
[*] Phase 2: Intelligent aggregation with deduplication...
[*] Phase 3: Reflection loop (max 2 iterations)...
[✓] Reflection: Analysis is complete and confident
[*] Confidence breakdown: 82.50% (from 2 tools)

╔════════════════════════════════════════════════════════════════════╗
║                  ADVANCED AGENT OUTPUT                             ║
╚════════════════════════════════════════════════════════════════════╝
Vulnerability Found:  true
Risk Level:          High
Vulnerability Type:  Reentrancy
Confidence:          82%

[TOOL SELECTION]
Selected Tools: RaxcAnalyzer, PatternDetectorTool
Reasoning: Contract has external calls, prioritizing reentrancy checks

[CONFIDENCE BREAKDOWN]
  • RaxcAnalyzer: 85.0%
  • PatternDetectorTool: 80.0%
  • Agreement Bonus: +5.0%
  • Final Confidence: 82.5%

[AGENT DECISION]
Primary Signal: RaxcAnalyzer detected vulnerability via RAG semantic similarity
Supporting Evidence:
  • PatternDetectorTool confirmed CEI pattern violation
```

---

## 📦 Tech Stack

- **Language:** Rust 2021 edition
- **Async Runtime:** Tokio
- **HTTP:** Reqwest + Axum
- **Blockchain:** Ethers-rs
- **Embeddings:** OpenAI text-embedding-3-small (1536 dims) — *Similarity search only*
- **LLM Inference:** 0G Compute (qwen/qwen-2.5-7b-instruct) — *All analysis + reasoning*
- **Storage:** 0G Storage (decentralized RAG memory)

> **Note:** 0G Compute handles all vulnerability analysis, reasoning, tool selection, and reflection. OpenAI is only used for embeddings.

---

## Future Plans

**Public Exploit Database API**

We plan to separate the 0G Storage exploit database into a publicly accessible service that anyone can use via HTTP requests.

### Planned Features

🌐 **HTTP API Access**
- Public endpoint for querying the 777-exploit database
- RESTful API for semantic similarity search
- No setup required — just send HTTP requests

📊 **Query Capabilities**
- Search by vulnerability type (reentrancy, flash loan, price manipulation, etc.)
- Semantic similarity search (find exploits similar to your contract)
- Filter by year, loss amount, attack vector, or protocol
- Get exploit details: contract code, attack transaction, root cause analysis

🔓 **Open Access**
- Free public access to the exploit knowledge base
- No authentication required for read-only queries
- Community contributions welcome (submit new exploits)

🚀 **Use Cases**
- Security researchers analyzing patterns
- Developers checking if their code matches known exploits
- Auditors searching for similar vulnerabilities
- Educational institutions teaching smart contract security
- Other security tools integrating RAXC's exploit database

**Example Future API Usage:**
```bash
# Search for reentrancy exploits
curl https://api.raxc.0g.ai/exploits?type=reentrancy

# Find exploits similar to a contract
curl -X POST https://api.raxc.0g.ai/search/similar \
  -H "Content-Type: application/json" \
  -d '{"contract_code": "contract VulnerableVault { ... }"}'

# Get specific exploit details
curl https://api.raxc.0g.ai/exploits/2024-001
```

This will enable the entire Web3 security ecosystem to benefit from RAXC's curated, decentralized exploit memory — **powered by 0G Storage.**

---

## Credits

**Built on the 0G Foundation:**
- **0G Storage:** Decentralized data availability for RAG memory
- **0G Compute:** Decentralized LLM inference for intelligent agent

**Exploit datasets from:**
- **DeFiHackLabs:** Real-world DeFi exploit collection (721 exploits)
- **DeFiVulnLabs:** Vulnerability pattern library (56 patterns)

---

## 📄 License

MIT License — see LICENSE file

---

**🚀 The RAXC Intelligent Agent Framework is production-ready for advanced smart contract security analysis!**

Get started: `cargo run --example agent_example_openai`
