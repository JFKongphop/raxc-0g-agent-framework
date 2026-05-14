#!/usr/bin/env tsx
/**
 * raxclaw — RAXC × OpenClaw branded CLI
 *
 * Usage:
 *   npx tsx raxclaw.ts analyze [contract.sol]
 *   npx tsx raxclaw.ts agent --message "audit DeFiVault.sol"
 *
 * Internally spawns OpenClaw with the raxc-security-audit skill registered,
 * which routes the request through skills/raxc-security/run.sh → RAXC Rust engine.
 */

import { execFileSync } from "child_process";
import * as path from "path";
import * as fs from "fs";

const REPO_ROOT = path.resolve(__dirname);
const OPENCLAW_BIN = path.join(REPO_ROOT, "node_modules", ".bin", "openclaw");

// ── RAXC banner ──────────────────────────────────────────────────────────────
function banner() {
  console.log("");
  console.log("██████╗   █████╗ ██╗  ██╗ ██████╗██╗      █████╗ ██╗    ██╗");
  console.log("██╔══██╗ ██╔══██╗╚██╗██╔╝██╔════╝██║     ██╔══██╗██║    ██║");
  console.log("██████╔╝ ███████║ ╚███╔╝ ██║     ██║     ███████║██║ █╗ ██║");
  console.log("██╔══██╗ ██╔══██║ ██╔██╗ ██║     ██║     ██╔══██║██║███╗██║");
  console.log("██║  ██║ ██║  ██║██╔╝ ██╗╚██████╗███████╗██║  ██║╚███╔███╔╝");
  console.log("╚═╝  ╚═╝ ╚═╝  ╚═╝╚═╝  ╚═╝ ╚═════╝╚══════╝╚═╝  ╚═╝ ╚══╝╚══╝ ");
  console.log("");
  console.log("RAXC × OpenClaw    —    Autonomous Security Cognition on 0G");
  console.log("════════════════════════════════════════════════════════════");
  console.log("");
}

// ── Validate OpenClaw installed ───────────────────────────────────────────────
function requireOpenclaw() {
  if (!fs.existsSync(OPENCLAW_BIN)) {
    console.error("[raxclaw] OpenClaw not installed. Run: npm install");
    process.exit(1);
  }
}

// ── CLI routing ───────────────────────────────────────────────────────────────
const [, , command, ...args] = process.argv;

banner();

switch (command) {
  case "analyze": {
    const contract = args[0] ?? "contract.sol";
    requireOpenclaw();
    console.log(`[raxclaw]        Routing to OpenClaw → skill: raxc-security-audit`);
    console.log(`[raxclaw]        Contract target: ${contract}`);
    console.log("");
    execFileSync(
      OPENCLAW_BIN,
      ["agent", "--message", `audit the Solidity smart contract: ${contract}`],
      { stdio: "inherit", cwd: REPO_ROOT }
    );
    break;
  }

  case "agent": {
    // Pass-through to openclaw agent with any flags
    requireOpenclaw();
    console.log(`[raxclaw]        Delegating to OpenClaw agent...`);
    console.log("");
    execFileSync(OPENCLAW_BIN, ["agent", ...args], {
      stdio: "inherit",
      cwd: REPO_ROOT,
    });
    break;
  }

  case "run": {
    // Direct skill execution — bypass OpenClaw, call run.sh directly (CI mode)
    const runScript = path.join(REPO_ROOT, "skills", "raxc-security", "run.sh");
    if (!fs.existsSync(runScript)) {
      console.error("[raxclaw] skills/raxc-security/run.sh not found");
      process.exit(1);
    }
    console.log(`[raxclaw]        Direct skill execution (CI mode)`);
    console.log("");
    execFileSync("bash", [runScript], { stdio: "inherit", cwd: REPO_ROOT });
    break;
  }

  default: {
    console.log("Usage:");
    console.log("  npx tsx raxclaw.ts analyze [contract.sol]   — full OpenClaw orchestration");
    console.log("  npx tsx raxclaw.ts run                      — direct skill execution (CI)");
    console.log("  npx tsx raxclaw.ts agent --message <msg>    — raw OpenClaw agent passthrough");
    console.log("");
    console.log("OpenClaw skill registered: raxc-security-audit");
    console.log("Config: openclaw.json");
    process.exit(0);
  }
}
