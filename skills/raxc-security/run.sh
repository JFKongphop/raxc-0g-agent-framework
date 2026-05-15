#!/usr/bin/env bash
# RAXC Security Audit — OpenClaw skill invocation script
# Called by OpenClaw when user asks to audit a Solidity contract.
#
# Usage (direct): bash skills/raxc-security/run.sh
# Usage (raxclaw): ./dist/raxclaw run
# Usage (OpenClaw): automatic via SKILL.md
#
# Zero-config: all values are baked in for dev/demo.
# Clone the repo and run — no .env setup required.

set -e

# ── Resolve repo root ─────────────────────────────────────────────────────────
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
BACKEND_DIR="$REPO_ROOT/backend"

# ── Baked-in dev configuration (override by setting vars before running) ──────
# 0G Galileo Testnet
export OG_RPC_URL="${OG_RPC_URL:-https://evmrpc-testnet.0g.ai}"
export RPC_URL="${RPC_URL:-https://evmrpc-testnet.0g.ai}"
export BLOCKCHAIN_RPC="${BLOCKCHAIN_RPC:-https://evmrpc-testnet.0g.ai}"
export OG_INDEXER_RPC="${OG_INDEXER_RPC:-https://indexer-storage-testnet-turbo.0g.ai}"
export INDEXER_RPC="${INDEXER_RPC:-https://indexer-storage-testnet-standard.0g.ai}"

# 0G Storage
export OG_STORAGE_ENDPOINT="${OG_STORAGE_ENDPOINT:-https://storage-testnet.0g.ai/api}"
export OG_STORAGE_STREAM_ID="${OG_STORAGE_STREAM_ID:-defi_cases}"

# 0G Compute
export OG_COMPUTE_ENDPOINT="${OG_COMPUTE_ENDPOINT:-https://compute-network-6.integratenetwork.work/v1/proxy/chat/completions}"
export OG_COMPUTE_MODEL="${OG_COMPUTE_MODEL:-qwen/qwen-2.5-7b-instruct}"
export OG_COMPUTE_API_KEY="${OG_COMPUTE_API_KEY:-app-sk-eyJhZGRyZXNzIjoiMHgyMDRhNzNlODMwM0YzZDA5QjEyMDYyZEVkQUE3NEIxQ0RBNkUxNjdkIiwicHJvdmlkZXIiOiIweGE0OGYwMTI4NzIzMzUwOUZENjk0YTIyQmY4NDAyMjUwNjJFNjc4MzYiLCJ0aW1lc3RhbXAiOjE3NzczOTI3MzYxMjIsImV4cGlyZXNBdCI6MCwibm9uY2UiOiI1MzM1ODk3NzdjYTdmNGY2ZTkxMjRlMGE1MDNlYWE5MiIsImdlbmVyYXRpb24iOjAsInRva2VuSWQiOjB9fDB4OTVhNTc3MDVhODAzOTZjMzkwYTQyMjM1NGU0ZDMxYTUwYWVjZDdlYzBhNTVmYzE0MmU0ZDBiNjliNTM0MTViODQ3YjA4ZmYwYTYzZmVmZTk3NTFkODlmYmZlY2MyZWUyZmZhMjRiNTk4ODJlNGYzYmNmNzRiMGUwY2E3N2MwMDgxYg==}"

# Deployed contracts (ERC-8183 + ERC-7857)
export RAXC_AUDIT_TASK_8183_ADDRESS="${RAXC_AUDIT_TASK_8183_ADDRESS:-0x6FFc92b063Fc470Dd2D4Cbd0f64E75eD96AE7a8c}"
export RAXC_AGENT_NFT_ADDRESS="${RAXC_AGENT_NFT_ADDRESS:-0xe3c7863AD3176E88E9C75a580fC15a2976D5fF53}"
export RAXC_AGENT_TOKEN_ID="${RAXC_AGENT_TOKEN_ID:-0}"
export VAULT_ADDRESS="${VAULT_ADDRESS:-0x7Ad0e4B636C63CdfF4e73895855E0a3Fe087C16c}"

# Dev wallet (testnet only — no real funds)
export PRIVATE_KEY="${PRIVATE_KEY:-0x5368e0ef6bb84d4143b17f35a021eb7fb9c077c611b7fb8a6c58336ee831810e}"
export OPERATOR_PRIVATE_KEY="${OPERATOR_PRIVATE_KEY:-0x5368e0ef6bb84d4143b17f35a021eb7fb9c077c611b7fb8a6c58336ee831810e}"
export OG_CLI_PATH="${OG_CLI_PATH:-./0g-cli}"

# OpenAI (embeddings)
export OPENAI_API_KEY="${OPENAI_API_KEY:-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}"
export USE_OPENAI_EMBEDDING="${USE_OPENAI_EMBEDDING:-true}"

# ── Load .env if present (allows overriding any value above) ──────────────────
ENV_FILE="$REPO_ROOT/.env"
if [ -f "$ENV_FILE" ]; then
  set -a
  # shellcheck source=/dev/null
  source "$ENV_FILE"
  set +a
fi

# ── OpenClaw orchestration preamble ──────────────────────────────────────────
echo ""
echo "[OpenClaw]       Received request: smart contract security audit"
echo "[OpenClaw]       Matched skill    → raxc-security-audit"
echo "[OpenClaw]       Building execution graph..."
echo "[Planner]        Analyzing contract scope..."
echo "[Planner]        Selecting tools: PatternDetector, MemoryTool, RaxcAnalyzer, ReflectionTool"
echo "[Planner]        Execution order: Memory → RAG → LLM → Consensus → Simulate → Reflect → Persist"
echo "[Planner]        Dispatching to RAXC cognition engine..."
echo ""

# ── Run RAXC cognition engine ─────────────────────────────────────────────────
cd "$BACKEND_DIR"

# Prefer the pre-compiled release binary (built by `pnpm build:rust`).
# Falls back to `cargo run` if not yet compiled (slower — compiles on demand).
PREBUILT="$BACKEND_DIR/target/release/examples/agent_example_remote"

if [ -f "$PREBUILT" ]; then
  echo "[RAXC]           Using prebuilt binary: $PREBUILT"
  exec "$PREBUILT" 2>&1
else
  echo "[RAXC]           Prebuilt binary not found — running via cargo (first run is slow)"
  echo "[RAXC]           Tip: run 'pnpm build:rust' once to pre-compile for instant startup"
  exec cargo run --release --example agent_example_remote 2>&1
fi
