#!/usr/bin/env bash
# RAXC Security Audit — OpenClaw skill invocation script
# Called by OpenClaw when user asks to audit a Solidity contract.
#
# Usage (direct): bash skills/raxc-security/run.sh
# Usage (OpenClaw): automatic via SKILL.md

set -e

# ── Resolve repo root (script lives at skills/raxc-security/run.sh) ──────────
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
BACKEND_DIR="$REPO_ROOT/backend"
ENV_FILE="$REPO_ROOT/.env"

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

# ── Validate environment ──────────────────────────────────────────────────────
if [ ! -f "$ENV_FILE" ]; then
  echo "[Error] .env file not found at $ENV_FILE"
  echo "        Copy .env.example to .env and fill in PRIVATE_KEY, RAXC_AGENT_NFT_ADDRESS, OG_RPC_URL"
  exit 1
fi

if [ ! -d "$BACKEND_DIR" ]; then
  echo "[Error] RAXC backend not found at $BACKEND_DIR"
  exit 1
fi

# ── Load environment variables ────────────────────────────────────────────────
set -a
# shellcheck source=/dev/null
source "$ENV_FILE"
set +a

# ── Run RAXC cognition engine ─────────────────────────────────────────────────
cd "$BACKEND_DIR"
exec cargo run --example agent_example_remote 2>&1
