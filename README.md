# RAXCLAW — Autonomous Security Cognition on 0G

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![0G Testnet](https://img.shields.io/badge/0G-Galileo%20Testnet-cyan)](https://evmrpc-testnet.0g.ai)
[![ERC-8183](https://img.shields.io/badge/ERC--8183-Audit%20Task-green)](https://chainscan-galileo.0g.ai/address/0x6FFc92b063Fc470Dd2D4Cbd0f64E75eD96AE7a8c)
[![ERC-7857](https://img.shields.io/badge/ERC--7857-Agent%20NFT-purple)](https://chainscan-galileo.0g.ai/address/0xe3c7863AD3176E88E9C75a580fC15a2976D5fF53)

> *"Don't just ask an AI if your contract is safe — ask an AI that has seen 722 real hacks."*

🌐 **Frontend:** [raxc-0g-agent-framework.vercel.app](https://raxc-0g-agent-framework.vercel.app)  
�️ **Remote Storage:** [raxc-0g-agent-framework-j43hng.fly.dev](https://raxc-0g-agent-framework-j43hng.fly.dev)

---

## What is RAXCLAW?

RAXCLAW is an **autonomous smart contract security agent** that detects vulnerabilities by combining:

- **722 real-world DeFi exploits** ($4.1B+ in total losses) indexed in a vector database build 0G Storage
- **Multi-tool agentic reasoning** — reentrancy, flash loans, price manipulation, access control
- **0G Storage** — decentralized, persistent exploit memory
- **0G Compute** — decentralized LLM inference (qwen-2.5-7b-instruct)
- **On-chain proof** — every audit result is stored as an ERC-8183 task + ERC-7857 NFT update

The CLI (`raxclaw`) is the primary product. The frontend is a verification and replay interface.

---

## Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                        raxclaw CLI (Ink/React)                      │
│   run │ analyze │ list │ show │ agent │ health                      │
└─────────────────────────────────┬───────────────────────────────────┘
                                  │ spawns
                                  ▼
┌─────────────────────────────────────────────────────────────────────┐
│              skills/raxc-security/run.sh                            │
│  (all env baked in — zero-config for users)                         │
└─────────────────────────────────┬───────────────────────────────────┘
                                  │ exec
                                  ▼
┌─────────────────────────────────────────────────────────────────────┐
│           backend/examples/agent_example_remote.rs                  │
│                  RAXC Cognition Engine (Rust)                       │
│                                                                     │
│  1. load_env()              Load baked config                       │
│  2. RemoteOgStorageClient   Query 722 exploits via HTTP (<10ms)     │
│  3. build_og_compute()      0G Compute LLM endpoint                 │
│  4. AgentCore::new()        Assemble multi-tool agent               │
│     ├─ RaxcAnalyzerRemote   RAG semantic similarity                 │
│     ├─ PatternDetectorTool  CEI / reentrancy patterns               │
│     ├─ GasAnalyzerTool      Gas griefing vectors                    │
│     ├─ FlashLoanTool        Flash loan attack paths                 │
│     ├─ AccessControlTool    Owner / role checks                     │
│     ├─ ReflectionTool       Self-review loop (max 2 iter)           │
│     └─ MemoryTool           Persistent cognition memory             │
│  5. LLM tool selection      Picks relevant tools per contract       │
│  6. Parallel execution      All selected tools run concurrently     │
│  7. Aggregation             Dedup + confidence breakdown            │
│  8. Reflection              Validates + improves findings           │
│  9. create_audit_task()     ERC-8183: mint on-chain audit task      │
│  10. finalize_audit_task()  ERC-8183: write verdict + root hash     │
│  11. update_agent_nft()     ERC-7857: push memory pointer to NFT    │
│  12. og_storage upload      Store full report on 0G Storage         │
└─────────────────────────────────┬───────────────────────────────────┘
                                  │
          ┌───────────────────────┴──────────────────────┐
          ▼                                              ▼
┌───────────────────────────────┐   ┌─────────────────────────────────┐
│  0G Galileo                   │   │  0G Storage                     │
│  Testnet                      │   │  (722 exploits + reports)       │
│                               │   │                                 │
│  ERC-8183                     │   │  RemoteOgStorage server         │
│  RaxcAuditTask                │   │  (fly.dev — port 3001)          │
│                               │   │                                 │
│  ERC-7857                     │   │  Indexer RPC                    │
│  RaxcAgentNFT                 │   │  turbo.0g.ai                    │
└───────────────────────────────┘   └─────────────────────────────────┘
```

---

## On-Chain Contracts (0G Galileo Testnet — Chain ID 16602)

| Contract | Standard | Address |
|----------|----------|---------|
| **RaxcAuditTask8183** | ERC-8183 | [`0x6FFc92b063Fc470Dd2D4Cbd0f64E75eD96AE7a8c`](https://chainscan-galileo.0g.ai/address/0x6FFc92b063Fc470Dd2D4Cbd0f64E75eD96AE7a8c) |
| **RaxcAgentNFT** | ERC-7857 | [`0xe3c7863AD3176E88E9C75a580fC15a2976D5fF53`](https://chainscan-galileo.0g.ai/address/0xe3c7863AD3176E88E9C75a580fC15a2976D5fF53) |

**ERC-8183** — Audit task lifecycle: `createTask → finalizeTask(verdict, rootHash, replayId)`  
**ERC-7857** — Intelligent agent NFT: on-chain memory pointer updated after every audit

---

## 0G Integration — Technical Deep Dive

RAXCLAW uses three distinct 0G primitives: **0G Storage KV** (exploit database + audit reports), **0G Compute** (decentralized LLM inference), and **0G Galileo EVM** (on-chain proof). Here is exactly how each one is wired in.

---

### 1. 0G Storage — Exploit Database (Indexing Phase)

`indexer-ts/indexer_protocol_0g.ts` runs once to populate the exploit database:

```
datasets-protocol-exploit/src/   ← 722 Solidity exploit files
         │
         │  indexer_protocol_0g.ts
         │  1. Read each .sol file
         │  2. Generate 1536-dim embedding via OpenAI text-embedding-3-small
         │  3. Serialize as JSON → Base64-encode
         │  4. Upload to 0G Storage KV via @0gfoundation/0g-ts-sdk
         │     └─ Indexer + Batcher → getFlowContract(0G Galileo RPC)
         │     └─ Stream ID: "defi_protocols"
         │     └─ Each entry: { root_hash, stream_id, key }
         │  5. Save root_hash + stream_id + key → manifest.json
         │
         ▼
  0G Storage KV (indexer-storage-testnet-turbo.0g.ai)
```

**Key SDK call:**
```ts
import { Indexer, Batcher, getFlowContract } from '@0gfoundation/0g-ts-sdk';

const indexer = new Indexer(INDEXER_RPC);
const flow = getFlowContract(BLOCKCHAIN_RPC, signer);
// upload: Batcher.appendFile() → indexer.upload()
```

**Proof of real 0G Storage** — every exploit file has a permanent root hash written to [`backend/manifest.json`](https://github.com/JFKongphop/raxc-0g-agent-framework/blob/main/backend/manifest.json). Sample entries:

| Exploit | Category | 0G Root Hash |
|---------|----------|-------------|
| `Reentrancy` | defi_cases | `0x3f3a2145...779f789` |
| `Flashloan-flaw` | defi_cases | `0x95aa40ce...b268da6` |
| `Price_manipulation` | defi_cases | `0xb570cc94...df8dd8` |
| `Overflow` | defi_cases | `0x11aaf4ee...692ba` |
| `ERC777-reentrancy` | defi_cases | `0x969e9402...e9f58` |
| `Parity_first_hack` | defi_protocols | `0xcb4b0f21...abb93f` |
| `dodo_flashloan` | defi_protocols | `0xb31ff05c...e107` |
| `SpankChain` | defi_protocols | `0xb98e53f4...b51a` |

Verify any entry by downloading it directly from 0G Storage:
```bash
./backend/0g-cli download \
  --indexer https://indexer-storage-testnet-turbo.0g.ai \
  --root 0x3f3a21452d595f571c16caa908f68a612156d9ef2b2e0262599d0483c779f789 \
  --file /tmp/reentrancy.bin
```

Full manifest (722 entries): [`backend/manifest.json`](https://github.com/JFKongphop/raxc-0g-agent-framework/blob/main/backend/manifest.json)

---

### 2. 0G Storage — Remote Query Server

Downloading 722 exploits every run takes 2–3 minutes. Instead, `api_0g_storage` (Axum HTTP server) pre-loads all exploits once at startup and serves vector queries over HTTP:

```
Startup (once):
  build_og_storage()
    └─ reads manifest.json  →  list of { root_hash, stream_id, key }
    └─ for each entry (10 concurrent):
         0g-cli download --indexer <INDEXER_RPC> --root <root_hash> --file /tmp/*.bin
         parse binary: find stream_id → find key → extract base64 → decode JSON
         → LoadedExploit { embedding: Vec<f64>, metadata, code_snippet }
    └─ all 722 exploits loaded into Vec<LoadedExploit> in RAM

Runtime (per query, <10ms):
  POST /query { "embedding": [1536 floats], "top_k": 5 }
    └─ cosine_similarity(query_vec, exploit_vec) for all 722
    └─ return top-K sorted by score
```

The server is deployed to fly.dev: `https://raxc-0g-agent-framework-j43hng.fly.dev`

**Rust client** (`RemoteOgStorageClient`):
```rust
let remote = RemoteOgStorageClient::new("https://raxc-0g-agent-framework-j43hng.fly.dev");
let loaded = remote.health().await?;   // GET /health → { "loaded": 722 }
// inside RaxcAnalyzerRemote tool:
// remote.query(embedding, top_k).await?  →  Vec<SimilarExploit>
```

---

### 3. 0G Compute — Decentralized LLM Inference

`og_compute.rs` wraps the 0G Compute network behind an OpenAI-compatible chat completions interface:

```rust
// Build client
let compute = OgComputeClient::with_api_key(
    endpoint,          // https://compute-network-6.integratenetwork.work
    "qwen-2.5-7b-instruct".to_string(),
    api_key,
);

// Call (used for tool selection, report generation, reflection)
let result = compute.infer(prompt).await?;
// POST /chat/completions  { model, messages, max_tokens: 8192 }
// Authorization: Bearer <OG_COMPUTE_API_KEY>
```

**Used in 3 places:**
| Where | What it does |
|-------|-------------|
| `AgentCore::analyze()` | Tool selection — picks which tools to run for this contract |
| `RaxcAnalyzerRemote` | RAG report generation — explains the most similar exploit |
| `ReflectionTool` | Self-critique — validates all findings, fills reasoning gaps |

---

### 4. 0G Galileo EVM — On-Chain Proof (ERC-8183 + ERC-7857)

`erc8183.rs` writes to the deployed contracts on 0G Galileo (chain ID 16602) using raw ABI encoding via `ethers-rs`:

```rust
// Step 1 — register the job (before analysis)
let task_id = create_audit_task("DeFiVault").await?;
// → sends createAuditTask(string) tx to RaxcAuditTask8183
// → reads AuditTaskCreated event → returns taskId

// Step 2 — commit the proof (after analysis + NFT update)
finalize_audit_task(task_id, verdict, root_hash, replay_id).await?;
// → sends finalizeAuditTask(uint256, string, bytes32, string) tx
// → root_hash = keccak256 of the 0G Storage upload root
```

```rust
// ERC-7857 — update agent NFT memory pointer
update_agent_nft(nft_address, token_id, memory_root_hash).await?;
// → sends updateMemory(uint256, bytes32) tx to RaxcAgentNFT
```

**No ethers-rs ABI macro needed** — calls are hand-encoded:
```rust
let selector = &keccak256(b"createAuditTask(string)")[..4];
let params = abi::encode(&[Token::String(contract_name.to_string())]);
let calldata = [selector, &params].concat();
```

---

### 5. 0G Storage — Audit Report Upload

After finalizing the ERC-8183 task, the full markdown report is uploaded to 0G Storage:

```
agent_example_remote.rs
  └─ analysis complete
  └─ og_storage.upload_report(markdown, filename)
       └─ 0g-cli upload --indexer <INDEXER_RPC> --file <report.md>
       └─ returns root_hash (bytes32)
  └─ root_hash → finalize_audit_task(..., root_hash, ...)
       └─ stored on-chain in ERC-8183 task record
       └─ frontend reads root_hash → reconstructs 0G Storage download link
```

---

### Full 0G Data Flow (One Audit)

```
[User: ./dist/raxclaw run]
        │
        ▼
run.sh → agent_example_remote (Rust)
        │
        ├─ GET  fly.dev/health                ← 0G Storage: verify 722 exploits loaded
        ├─ POST 0G Galileo: createAuditTask   ← ERC-8183: register job on-chain
        ├─ POST 0G Compute: tool selection    ← LLM picks tools for this contract
        ├─ POST fly.dev/query (embedding)     ← 0G Storage: top-5 similar exploits
        ├─ POST 0G Compute: RAG explanation   ← LLM explains exploit match
        ├─ POST 0G Compute: reflection        ← LLM self-critique of findings
        ├─ 0g-cli upload report.md            ← 0G Storage: save full audit report
        ├─ POST 0G Galileo: updateMemory      ← ERC-7857: NFT memory pointer updated
        └─ POST 0G Galileo: finalizeAuditTask ← ERC-8183: verdict + root_hash on-chain
```

---

## Repo Structure

```
raxc-0g-agent-framework/
├── raxclaw.tsx                       # CLI entry — Ink/React UI (TypeScript)
├── build.cjs                         # esbuild → dist/raxclaw.mjs + dist/raxclaw
├── dist/
│   ├── raxclaw                       # Executable shell wrapper (no tsx needed)
│   └── raxclaw.mjs                   # Self-contained ESM bundle (1.7MB)
│
├── skills/raxc-security/
│   └── run.sh                        # Zero-config runner (all env baked in)
│
├── backend/                          # Rust cognition engine
│   ├── src/
│   │   ├── lib.rs                    # Core: AgentCore, tools, OG infra
│   │   ├── agent.rs                  # Multi-tool agent + ERC-8183/7857 calls
│   │   ├── erc8183.rs                # ERC-8183 on-chain task management
│   │   ├── og_storage.rs             # 0G Storage client (exploit DB)
│   │   ├── og_compute.rs             # 0G Compute LLM client
│   │   ├── tools.rs                  # Security analysis tools
│   │   └── api.rs                    # REST API server (Axum)
│   └── examples/
│       └── agent_example_remote.rs   # Main entrypoint (spawned by run.sh)
│
├── frontend/                         # Next.js 14 verification UI
│   ├── app/page.tsx
│   └── components/
│       ├── ArchFlow.tsx              # 6-step pipeline visualization
│       ├── AuditExplorer.tsx         # Browse on-chain audit tasks
│       ├── MemorySection.tsx         # Live cognition count from chain
│       └── DownloadSection.tsx
│
├── contracts/                        # Foundry smart contracts
│   └── src/
│       ├── RaxcAuditTask8183.sol
│       └── RaxcAgentNFT.sol
│
├── datasets-protocol-exploit/        # 722 real-world exploit dataset
├── indexer-ts/                       # 0G Storage indexer (TypeScript)
└── .env.local                        # Override template (all vars, empty values)
```

---

## Quick Start

### Prerequisites

- **Node.js 18+**
- **Rust + cargo** — [rustup.rs](https://rustup.rs)
- **pnpm** — `npm install -g pnpm`

### Install & Run

```bash
# 1. Clone
git clone https://github.com/JFKongphop/raxc-0g-agent-framework
cd raxc-0g-agent-framework

# 2. Build — one-time (JS CLI + Rust cognition engine)
pnpm install && pnpm build:all

# 3. Run — no .env setup required, everything is baked in
./dist/raxclaw run
```

> All API keys, contract addresses, and RPC endpoints are baked into `skills/raxc-security/run.sh` for zero-config dev/demo use. To use your own keys, create a `.env` file (see [`.env.local`](.env.local) for the full variable list).

---

## CLI Commands

```bash
./dist/raxclaw run                          # Full audit — default demo contract
./dist/raxclaw run --file MyContract.sol    # Audit a specific Solidity file
./dist/raxclaw list                         # List all saved audit reports
./dist/raxclaw show <name|index>            # Print a report to the terminal
./dist/raxclaw analyze MyContract.sol       # Analyze via OpenClaw orchestration
./dist/raxclaw agent --message "..."        # Pass-through to OpenClaw agent CLI
./dist/raxclaw health                       # Check remote agent server status
```

---

## Build Scripts

| Command | What it does |
|---------|-------------|
| `pnpm build` | JS CLI only → `dist/raxclaw.mjs` + `dist/raxclaw` |
| `pnpm build:rust` | Rust binary → `backend/target/release/examples/agent_example_remote` |
| `pnpm build:all` | Both — recommended before first run |
| `pnpm dev` | Run CLI via `tsx` (no build needed, requires node_modules) |

After `pnpm build:rust`, `./dist/raxclaw run` uses the prebuilt binary — no `cargo run` compile delay on each invocation.

---

## Agent Intelligence — Factors & Scoring

### 7 Security Tools (Registered at Runtime)

| # | Tool | Trust Weight | What It Detects |
|---|------|-------------|-----------------|
| 1 | `RaxcAnalyzerRemote` | **1.0** (highest) | RAG: semantic match against 777 real exploits via 0G Storage |
| 2 | `PatternDetectorTool` | **0.8** | CEI violations, reentrancy, unchecked `.call` return values |
| 3 | `FlashLoanTool` | **0.7** | Flash loan vectors, spot price oracles (`getReserves()`), callback exploits |
| 4 | `AccessControlTool` | **0.7** | Missing `onlyOwner`, unguarded `initialize()`, unprotected admin functions |
| 5 | `ReflectionTool` | **0.7** | LLM self-critique — validates all findings, fills reasoning gaps |
| 6 | `MemoryTool` | **0.7** | Loads historical cognition from `/tmp/raxc_memory/` via 0G Storage |
| 7 | `GasAnalyzerTool` | **0.2** (penalized) | `array.length` in loops, unbounded loops, DoS gas griefing |

> Trust weights are applied via `ToolTrustWeighting` — higher trust = higher contribution to final confidence.

---

### 13-Field Agent Output (`AgentOutput`)

Every analysis produces a structured 13-field result:

| Field | Type | Description |
|-------|------|-------------|
| `vulnerability_found` | `bool` | Whether any vulnerability was detected |
| `risk_level` | `String` | `Critical` / `High` / `Medium` / `Low` / `None` |
| `vulnerability_type` | `String` | Primary type: Reentrancy, Flash Loan, Access Control, etc. |
| `confidence` | `f64` | 0–100% — weighted aggregate across all tools |
| `markdown` | `String` | Full audit report (saved to `backend/reports/`) |
| `reasoning` | `String` | LLM explanation of the primary finding |
| `similar_exploits` | `Vec<String>` | Top-K most similar real exploits from the 777 dataset |
| `filename` | `String` | Report filename with timestamp and confidence score |
| `tool_selection` | `ToolSelection` | Which tools were selected + LLM reasoning |
| `confidence_breakdown` | `ConfidenceBreakdown` | Per-tool scores + agreement bonus |
| `memory_influence` | `MemoryInfluence` | Past patterns + decisions influenced by memory |
| `agent_decision` | `AgentDecision` | Primary signal, supporting evidence, ignored signals |
| `reflection_iterations` | `u8` | Number of self-review loops performed (0–2) |

---

### 4-Factor Risk Scoring Formula (`RiskScoringEngine`)

```
RiskScore = (SeverityWeight × 0.35)
          + (ConfidenceScore × 0.25)
          + (ToolAgreement  × 0.20)
          + (ExploitSimilarity × 0.20)
```

**Severity weights:**

| Severity | Weight |
|----------|--------|
| Critical | 1.00 |
| High | 0.75 |
| Medium | 0.50 |
| Low | 0.25 |

**Agreement bonus:** +0.05 if `tool_agreement ≥ 1.0` AND `severity = High` AND `confidence ≥ 85%`

**Final classification:**

| RiskScore | Classification |
|-----------|---------------|
| ≥ 0.75 | `CRITICAL RISK` |
| ≥ 0.60 | `HIGH RISK` |
| ≥ 0.40 | `MEDIUM RISK` |
| < 0.40 | `LOW RISK` |

---

### Exploitability Estimator

A separate `ExploitabilityEstimator` scores real-world attack feasibility (0.0–1.0):

| Factor | Score |
|--------|-------|
| External call before state update / callback present | +0.40 |
| ETH transfer (`.call{value}`, `.send`, `.transfer`) | +0.20 |
| Recursive entry possible (reentrancy pattern) | +0.20 |
| Historical exploit similarity (from RAG) | +0.00–0.20 |

---

### Deterministic Severity Locks (`SeverityLock`)

Vulnerability type → severity is deterministic (not LLM-decided):

| Vulnerability | Locked Severity |
|---------------|----------------|
| Reentrancy | `High` |
| Access Control / Authorization | `Critical` |
| Flash Loan | `High` |
| Price Oracle | `High` |
| Overflow / Underflow | `Medium-High` |
| Front-running | `Medium` |
| DoS / Gas Griefing | `Medium` |
| Timestamp dependence | `Low-Medium` |

---

### Attack Simulation Engine

Beyond detection, the agent builds a full **deterministic attack simulation** for each finding:

- **`ExploitGraph`** — directed graph of the attack flow (nodes + edges)
- **`AttackSimulation`** — step-by-step execution path with state transitions
- **`AttackerPersona`** — classifies attacker as `MEVBot`, `ProtocolHacker`, or `ContractExploiter`
- **`AttackerCapabilities`** — required skill level and resources
- **`ExploitVerdict`** — feasibility status + success probability
- **`DeterministicReplay`** — reproducible replay ID + seed for audit trail
- **`ConfidenceEngine`** — single source of truth for final confidence (no LLM override)
- **`StateProof`** — before/after state transitions during simulated exploit
- **`SeverityProof`** — justification chain for severity classification

---

### Signal Normalizer (Production Hardening)

Before aggregation, all tool outputs pass through `SignalNormalizer`:

- Drops signals with empty vulnerability field
- Drops signals with confidence < 5%
- Drops signals with empty evidence
- Locks confidence to 2 decimal places
- Strips markdown, emojis, non-ASCII from evidence
- Truncates evidence to max 5 lines / 400 characters

---

## Agent Pipeline

| Step | Component | What Happens |
|------|-----------|-------------|
| 1 | **Contract Input** | `RAXC_CONTRACT_FILE`, `RAXC_CONTRACT_CODE`, or built-in demo |
| 2 | **MemoryTool** | Load persistent cognition history from 0G Storage |
| 3 | **RaxcAnalyzerRemote** | RAG: semantic similarity search across 722 exploits |
| 4 | **LLM Tool Selection** | 0G Compute picks which tools to run per contract |
| 5 | **Parallel Execution** | Pattern detection, gas, flash loan, access control — concurrent |
| 6 | **Aggregation** | Deduplicate findings, calculate per-tool confidence scores |
| 7 | **ReflectionTool** | LLM validates its output, fills gaps (max 2 iterations) |
| 8 | **ERC-8183** | `createAuditTask` → analysis → `finalizeAuditTask(verdict, rootHash)` |
| 9 | **ERC-7857** | Agent NFT memory pointer updated with new cognition trace |
| 10 | **0G Storage** | Full markdown report uploaded, root hash stored on-chain |

---

## Security Tools

| Tool | Detects |
|------|---------|
| `RaxcAnalyzerRemote` | Semantic similarity to 722 real exploits (RAG) |
| `PatternDetectorTool` | CEI violations, reentrancy, unchecked external calls |
| `FlashLoanTool` | Flash loan attack vectors, price oracle manipulation |
| `GasAnalyzerTool` | Gas griefing, unbounded loops, DoS patterns |
| `AccessControlTool` | Missing owner checks, unprotected admin functions |
| `ReflectionTool` | Self-review: validates and improves all findings |
| `MemoryTool` | Historical analyses from persistent 0G cognition memory |

---

## Exploit Dataset

| Source | Count | Coverage |
|--------|-------|----------|
| DeFiHackLabs | 626 | Real on-chain incidents with tx hashes and root cause |
| DeFiVulnLabs | 151 | Vulnerability pattern library |
| **Total** | **722** | **$4.1B+ in documented losses** |

Indexed with OpenAI `text-embedding-3-small` (1536-dim vectors) and served via the remote storage server on fly.dev.

---

## Environment Variables

All variables have dev defaults baked into `run.sh`. Override any of them via `.env` at the repo root:

```bash
# Copy .env.local → .env and fill in only what you want to override
cp .env.local .env
```

| Variable | Default | Purpose |
|----------|---------|---------|
| `OG_RPC_URL` | `https://evmrpc-testnet.0g.ai` | 0G Galileo RPC |
| `OG_COMPUTE_ENDPOINT` | baked | 0G Compute LLM API |
| `OG_COMPUTE_API_KEY` | baked | 0G Compute auth token |
| `OPENAI_API_KEY` | baked | Embedding generation only |
| `PRIVATE_KEY` | baked (testnet) | Wallet for ERC-8183/7857 writes |
| `RAXC_AUDIT_TASK_8183_ADDRESS` | baked | ERC-8183 contract |
| `RAXC_AGENT_NFT_ADDRESS` | baked | ERC-7857 contract |
| `RAXC_CONTRACT_FILE` | — | Path to `.sol` file to audit |
| `RAXC_CONTRACT_CODE` | — | Inline Solidity source to audit |

---

## Frontend

Next.js 14 frontend — read-only verification interface, reads directly from 0G Galileo via ethers.js:

- **ArchFlow** — 6-step pipeline visualization: OpenClaw → RAXC → 0G Compute → 0G Storage → ERC-7857 → ERC-8183
- **AuditExplorer** — All on-chain audit tasks from ERC-8183, with root hash links to 0G Storage reports
- **MemorySection** — Live on-chain cognition entry count + full task list
- **DownloadSection** — Install instructions

---

## Deployed Services

| Service | URL |
|---------|-----|
| Frontend | [raxc-0g-agent-framework.vercel.app](https://raxc-0g-agent-framework.vercel.app) |
| Remote Storage | [raxc-0g-agent-framework-j43hng.fly.dev](https://raxc-0g-agent-framework-j43hng.fly.dev) |
| 0G RPC | `https://evmrpc-testnet.0g.ai` |
| 0G Indexer | `https://indexer-storage-testnet-turbo.0g.ai` |
| 0G Compute | `https://compute-network-6.integratenetwork.work` |

---

## Credits

- **0G Foundation** — Storage, Compute, and Galileo Testnet infrastructure
- **DeFiHackLabs** — Real-world DeFi exploit collection
- **DeFiVulnLabs** — Vulnerability pattern library
- **OpenClaw** — Agent CLI orchestration layer

---

## License

MIT — see [LICENSE](backend/LICENSE)
