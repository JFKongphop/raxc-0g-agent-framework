---
name: raxc-security-audit
description: Autonomous smart contract security audit using 722 real DeFi exploits from 0G Storage. Detects reentrancy, flash loan attacks, access control vulnerabilities and more using RAXC multi-agent cognition engine on 0G Galileo.
metadata: {"openclaw": {"emoji": "🦀", "requires": {"bins": ["cargo"]}, "homepage": "https://github.com/raxc/raxc-0g-agent-framework"}}
---

# RAXC Security Audit Skill

When the user asks to **audit**, **analyze**, **check**, or **scan** a Solidity smart contract for vulnerabilities — use this skill.

## What RAXC Does

RAXC is an autonomous security cognition engine. It runs a 13-phase multi-agent pipeline:

1. **MemoryTool** — loads past audit history from 0G Storage via ERC-7857 on-chain index
2. **PatternDetector** — static analysis for reentrancy, access control, flash loan patterns
3. **RaxcAnalyzer** — RAG search across 722 real DeFi exploits (cosine similarity)
4. **0G Compute** — LLM reasoning with exploit context injected
5. **Consensus Engine** — multi-agent vote → deterministic verdict
6. **AttackSimulator** — 8-step deterministic attack execution path
7. **GraphConstructor** — attack graph (nodes + edges)
8. **ConsistencyVerifier** — cross-checks simulation vs graph vs tools
9. **FinalDecision** — single authority verdict
10. **Attestation** — replay ID + execution trace hash
11. **ReflectionTool** — 0G Compute self-critique, removes hallucinations
12. **0G Storage** — uploads audit summary, gets merkle root hash
13. **ERC-7857** — updates agent intelligence on-chain (0G Galileo chain 16602)

## How to Invoke

```bash
bash {baseDir}/run.sh
```

## What It Returns

- Vulnerability type (Reentrancy / FlashLoan / AccessControl / None)
- Risk level (CRITICAL / HIGH / MEDIUM / LOW / NONE)
- Confidence score (0–100%)
- Attack simulation: 8-step execution path, success probability
- Attack graph: nodes + edges + root node
- Consistency score across all verification checks
- Replay ID — deterministic reproduction identifier
- Execution trace hash — verifiable attestation
- 0G Storage root hash — merkle root of audit data
- ERC-7857 TX hash — on-chain cognition state update

## Example Output

```
[OpenClaw]       Matched skill → raxc-security-audit
[OpenClaw]       Building execution graph...
[Planner]        Selecting tools: PatternDetector, MemoryTool, RaxcAnalyzer, ReflectionTool
[Planner]        Dispatching to RAXC cognition engine...

[RAXC]           Starting autonomous security analysis...
[MemoryTool]     Loaded 3 past audits from 0G Storage
[RaxcAnalyzer]   Querying 722-exploit RAG database...
[0G Compute]     Sending for analysis...
[RAXC]           Running consensus engine...
[ReflectionTool] 0G Compute self-critique...
[0G Storage]     Uploading audit summary...
[ERC-7857]       Intelligence updated on-chain (chain 16602)
```

## Environment Required

Copy `.env.example` to `.env` and fill in:
- `PRIVATE_KEY` — 0G Galileo wallet private key
- `RAXC_AGENT_NFT_ADDRESS` — ERC-7857 contract address
- `OG_RPC_URL` — 0G EVM RPC endpoint
- `OG_INDEXER_RPC` — 0G Storage indexer

## Architecture

```
OpenClaw Orchestrator
    ↓ matches raxc-security-audit skill
    ↓ calls run.sh
RAXC Rust Cognition Engine
    ↓ RAG: 722 real DeFi exploits on 0G Storage
    ↓ LLM: 0G Compute (qwen-2.5-7b-instruct)
    ↓ Memory: ERC-7857 on-chain index → 0G Storage
ERC-7857 Persistent Agent Identity (0G Galileo)
```
