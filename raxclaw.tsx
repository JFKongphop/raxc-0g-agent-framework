/**
 * raxclaw — RAXC × OpenClaw  |  Autonomous Security Cognition on 0G
 *
 * Usage:
 *   raxclaw [command] [flags]
 *
 * Built with Ink (React for CLIs) + esbuild compiled binary.
 */

import { FC, useState, useEffect, useRef } from "react";
import { render, Box, Text, Static, useApp, Newline } from "ink";
import Spinner from "ink-spinner";
import { spawn } from "child_process";
import * as path from "path";
import * as fs from "fs";
import { fileURLToPath } from "url";

// ESM-compatible __dirname
const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

// Resolve repo root: dist/raxclaw → __dirname = dist/ → go up one level
const REPO_ROOT =
  path.basename(__dirname) === "dist"
    ? path.resolve(__dirname, "..")
    : path.resolve(__dirname);

const OPENCLAW_BIN = path.join(REPO_ROOT, "node_modules", ".bin", "openclaw");
const RUN_SH = path.join(REPO_ROOT, "skills", "raxc-security", "run.sh");

// ─────────────────────────────────────────────────────────────────────────────
// Banner
// ─────────────────────────────────────────────────────────────────────────────
const BANNER = [
  "                                                           ",
  "██████╗   █████╗ ██╗  ██╗ ██████╗██╗      █████╗ ██╗    ██╗",
  "██╔══██╗ ██╔══██╗╚██╗██╔╝██╔════╝██║     ██╔══██╗██║    ██║",
  "██████╔╝ ███████║ ╚███╔╝ ██║     ██║     ███████║██║ █╗ ██║",
  "██╔══██╗ ██╔══██║ ██╔██╗ ██║     ██║     ██╔══██║██║███╗██║",
  "██║  ██║ ██║  ██║██╔╝ ██╗╚██████╗███████╗██║  ██║╚███╔███╔╝",
  "╚═╝  ╚═╝ ╚═╝  ╚═╝╚═╝  ╚═╝ ╚═════╝╚══════╝╚═╝  ╚═╝ ╚══╝╚══╝",
  "                                                           ",
];

const Banner: FC = () => (
  <Box flexDirection="column" marginBottom={1}>
    {BANNER.map((line, i) => (
      <Text key={i} bold color="cyan">
        {line}
      </Text>
    ))}
    <Text color="gray">
      {"  Autonomous Security Cognition on 0G   "}
      <Text color="yellow" bold>
        v1.0.0
      </Text>
    </Text>
  </Box>
);

// ─────────────────────────────────────────────────────────────────────────────
// Help Screen
// ─────────────────────────────────────────────────────────────────────────────
const HelpUI: FC = () => {
  const { exit } = useApp();
  useEffect(() => {
    exit();
  }, [exit]);

  return (
    <Box flexDirection="column" paddingX={1}>
      <Banner />

      <Box marginBottom={1}>
        <Text>
          <Text bold>Usage:{"  "}</Text>
          <Text color="cyan" bold>
            raxclaw
          </Text>
          <Text dimColor> [command] [flags]{"\n"}</Text>
          <Text bold>{"        "}</Text>
          <Text color="cyan" bold>
            raxclaw
          </Text>
          <Text dimColor> [command] --help</Text>
        </Text>
      </Box>

      <Box
        flexDirection="column"
        borderStyle="round"
        borderColor="cyan"
        paddingX={2}
        paddingY={0}
        marginBottom={1}
      >
        <Text bold color="white">
          {" "}
          Available Commands:
        </Text>
        <Newline />
        <Text>
          {"  "}
          <Text color="green" bold>
            {"run     "}
          </Text>
          {"  Run RAXC security audit (direct skill, CI mode)"}
        </Text>
        <Text>
          {"  "}
          <Text color="green" bold>
            {"analyze "}
          </Text>
          {"  Analyze a contract via OpenClaw orchestration"}
        </Text>
        <Text>
          {"  "}
          <Text color="green" bold>
            {"list    "}
          </Text>
          {"  List all saved audit reports"}
        </Text>
        <Text>
          {"  "}
          <Text color="green" bold>
            {"show    "}
          </Text>
          {"  Show a report in the terminal  "}
          <Text dimColor>{"(raxclaw show <name|index>)"}</Text>
        </Text>
        <Text>
          {"  "}
          <Text color="green" bold>
            {"agent   "}
          </Text>
          {"  Pass-through to OpenClaw agent CLI"}
        </Text>
        <Newline />
        <Text bold color="white">
          {" "}
          Flags:
        </Text>
        <Newline />
        <Text>
          {"  "}
          <Text color="yellow">{"-h, --help         "}</Text>
          {"  help for raxclaw"}
        </Text>
        <Text>
          {"  "}
          <Text color="yellow">{"-V, --version      "}</Text>
          {"  show version"}
        </Text>
        <Text>
          {"  "}
          <Text color="yellow">{"--contract [file]  "}</Text>
          {"  Solidity contract path (for analyze)"}
        </Text>
        <Text>
          {"  "}
          <Text color="yellow">{"--message [msg]    "}</Text>
          {"  Natural language prompt (for agent)"}
        </Text>
        <Newline />
      </Box>

      <Text dimColor>
        {'Use "raxclaw [command] --help" for more information about a command.'}
      </Text>
      <Newline />
    </Box>
  );
};

// ─────────────────────────────────────────────────────────────────────────────
// Run  (streams audit output with live spinner)
// ─────────────────────────────────────────────────────────────────────────────
interface OutputLine {
  id: number;
  text: string;
}

const RunUI: FC<{ contractCode?: string; contractFile?: string }> = ({ contractCode, contractFile }) => {
  const { exit } = useApp();
  const [lines, setLines] = useState<OutputLine[]>([]);
  const [phase, setPhase] = useState("Initializing...");
  const [done, setDone] = useState(false);
  const [code, setCode] = useState(0);
  const lineId = useRef(0);

  useEffect(() => {
    const extraEnv: Record<string, string> = {};
    if (contractFile) extraEnv.RAXC_CONTRACT_FILE = path.resolve(contractFile);
    if (contractCode) extraEnv.RAXC_CONTRACT_CODE = contractCode;
    const proc = spawn("bash", [RUN_SH], {
      cwd: REPO_ROOT,
      env: { ...process.env, ...extraEnv },
    });

    const onData = (chunk: Buffer) => {
      const text = chunk.toString();
      const newLines: OutputLine[] = text
        .split("\n")
        .filter((l) => l.trim().length > 0)
        .map((l) => ({ id: lineId.current++, text: l }));

      if (newLines.length > 0) setLines((prev) => [...prev, ...newLines]);

      // Track active RAXC module label from output e.g. [RAXC], [MemoryTool], [0G Storage]
      const m = text.match(/\[(RAXC|MemoryTool|RaxcAnalyzer|0G Storage|0G Compute|ReflectionTool|ERC-7857|Consensus|Planner|OpenClaw)[^\]]*\]/);
      if (m) setPhase(m[0]);
    };

    proc.stdout.on("data", onData);
    proc.stderr.on("data", onData);

    proc.on("close", (exitCode) => {
      setCode(exitCode ?? 0);
      setDone(true);
      setTimeout(
        () => exit(exitCode ? new Error(`exit ${exitCode}`) : undefined),
        300
      );
    });

    return () => {
      proc.kill("SIGTERM");
    };
  }, [exit]);

  return (
    <Box flexDirection="column">
      <Banner />
      <Box marginBottom={1}>
        <Text bold color="cyan">
          ▶  RAXC Security Audit
        </Text>
      </Box>

      <Static items={lines}>
        {(line) => (
          <Text key={line.id} dimColor>
            {line.text}
          </Text>
        )}
      </Static>

      <Box marginTop={1} paddingX={1}>
        {done ? (
          <Text bold color={code === 0 ? "green" : "red"}>
            {code === 0 ? "✔  Audit complete" : `✘  Audit failed (exit ${code})`}
          </Text>
        ) : (
          <Text color="green">
            <Spinner type="dots" />
            <Text dimColor>{"  "}{phase}</Text>
          </Text>
        )}
      </Box>
    </Box>
  );
};

// ─────────────────────────────────────────────────────────────────────────────
// Analyze  (OpenClaw orchestration, stdio:inherit)
// ─────────────────────────────────────────────────────────────────────────────
const AnalyzeUI: FC<{ contract: string }> = ({ contract }) => {
  const { exit } = useApp();
  const [done, setDone] = useState(false);
  const [code, setCode] = useState(0);

  useEffect(() => {
    if (!fs.existsSync(OPENCLAW_BIN)) {
      console.error("\n  openclaw not found — run `pnpm install`\n");
      exit(new Error("missing openclaw"));
      return;
    }

    const args = [
      "run",
      "--skill",
      "raxc-security-audit",
      "--message",
      `Analyze ${contract}`,
    ];

    const proc = spawn(OPENCLAW_BIN, args, {
      cwd: REPO_ROOT,
      env: { ...process.env },
      stdio: "inherit",
    });

    proc.on("close", (exitCode) => {
      setCode(exitCode ?? 0);
      setDone(true);
      setTimeout(
        () => exit(exitCode ? new Error(`exit ${exitCode}`) : undefined),
        300
      );
    });

    return () => {
      proc.kill("SIGTERM");
    };
  }, [contract, exit]);

  return (
    <Box flexDirection="column">
      <Banner />
      <Box marginBottom={1}>
        <Text bold color="cyan">
          ▶  OpenClaw Analysis
        </Text>
        <Text dimColor>{"  "}{contract}</Text>
      </Box>
      {!done && (
        <Text color="green">
          <Spinner type="dots" />
          <Text dimColor>{"  Orchestrating OpenClaw..."}</Text>
        </Text>
      )}
      {done && (
        <Text bold color={code === 0 ? "green" : "red"}>
          {code === 0
            ? "✔  Analysis complete"
            : `✘  Analysis failed (exit ${code})`}
        </Text>
      )}
    </Box>
  );
};

// ─────────────────────────────────────────────────────────────────────────────
// List audit reports
// ─────────────────────────────────────────────────────────────────────────────
interface ReportMeta {
  name: string;
  filePath: string;
  contract: string;
  vuln: string;
  date: string;
  confidence: string;
}

function findReports(): ReportMeta[] {
  const dirs = [
    path.join(REPO_ROOT, "backend"),
    path.join(REPO_ROOT, "backend", "reports"),
  ];
  const reports: ReportMeta[] = [];
  for (const dir of dirs) {
    if (!fs.existsSync(dir)) continue;
    const files = fs.readdirSync(dir).filter(
      (f) => f.startsWith("RAXC_") && f.endsWith(".md")
    );
    for (const file of files) {
      const m = file.match(/^RAXC_(.+?)_(.+?)_(\d{8})_(\d{6})_(\d+)pct\.md$/);
      reports.push({
        name: file,
        filePath: path.join(dir, file),
        contract: m ? m[1] : "Unknown",
        vuln: m ? m[2] : "Unknown",
        date: m
          ? `${m[3].slice(0, 4)}-${m[3].slice(4, 6)}-${m[3].slice(6, 8)} ${m[4].slice(0, 2)}:${m[4].slice(2, 4)}`
          : "",
        confidence: m ? `${m[5]}%` : "",
      });
    }
  }
  return reports.sort((a, b) => b.name.localeCompare(a.name));
}

const ListUI: FC = () => {
  const { exit } = useApp();
  useEffect(() => { exit(); }, [exit]);

  const reports = findReports();

  return (
    <Box flexDirection="column" paddingX={1}>
      <Banner />
      <Text bold color="cyan">📋  Audit Reports</Text>
      <Newline />
      {reports.length === 0 ? (
        <Text dimColor>  No reports found. Run `raxclaw run` to generate one.</Text>
      ) : (
        reports.map((r, i) => (
          <Box key={i} flexDirection="column" marginBottom={0}>
            <Text>
              <Text color="yellow" bold>{`  ${String(i + 1).padStart(2)}.  `}</Text>
              <Text color="white" bold>{r.name}</Text>
            </Text>
            <Text dimColor>{`        ${r.contract}  │  ${r.vuln}  │  conf: ${r.confidence}  │  ${r.date}`}</Text>
          </Box>
        ))
      )}
      <Newline />
      {reports.length > 0 && (
        <Text dimColor>{`  ${reports.length} report(s) found — use \`raxclaw show <name>\` to view`}</Text>
      )}
      <Newline />
    </Box>
  );
};

// ─────────────────────────────────────────────────────────────────────────────
// Show a single report
// ─────────────────────────────────────────────────────────────────────────────
const ShowUI: FC<{ query: string }> = ({ query }) => {
  const { exit } = useApp();
  useEffect(() => { exit(); }, [exit]);

  // Match by exact name, partial name, or index ("1", "2", …)
  const reports = findReports();
  let found: ReportMeta | undefined;

  const idx = parseInt(query, 10);
  if (!isNaN(idx) && idx >= 1 && idx <= reports.length) {
    found = reports[idx - 1];
  } else {
    found =
      reports.find((r) => r.name === query || r.name === query + ".md") ??
      reports.find((r) => r.name.toLowerCase().includes(query.toLowerCase()));
  }

  if (!found) {
    return (
      <Box flexDirection="column" paddingX={1}>
        <Banner />
        <Text color="red">✘  Report not found: {query}</Text>
        <Text dimColor>  Use `raxclaw list` to see available reports.</Text>
        <Newline />
      </Box>
    );
  }

  const content = fs.readFileSync(found.filePath, "utf-8");
  const lines = content.split("\n");

  return (
    <Box flexDirection="column" paddingX={1}>
      <Banner />
      <Text bold color="cyan">📄  {found.name}</Text>
      <Newline />
      {lines.map((line, i) => {
        if (line.startsWith("# "))
          return <Text key={i} bold color="cyan">{line}</Text>;
        if (line.startsWith("## "))
          return <Text key={i} bold color="yellow">{line}</Text>;
        if (line.startsWith("### "))
          return <Text key={i} bold color="green">{line}</Text>;
        if (line.startsWith("```") || line.startsWith("|"))
          return <Text key={i} color="gray">{line}</Text>;
        if (line.startsWith("- ") || line.startsWith("* "))
          return <Text key={i}><Text color="cyan">{"  • "}</Text><Text>{line.slice(2)}</Text></Text>;
        if (line.trim() === "")
          return <Newline key={i} />;
        return <Text key={i}>{line}</Text>;
      })}
      <Newline />
    </Box>
  );
};

// ─────────────────────────────────────────────────────────────────────────────
// Agent passthrough
// ─────────────────────────────────────────────────────────────────────────────
const AgentUI: FC<{ args: string[] }> = ({ args }) => {
  const { exit } = useApp();

  useEffect(() => {
    if (!fs.existsSync(OPENCLAW_BIN)) {
      console.error("\n  openclaw not found — run `pnpm install`\n");
      exit(new Error("missing openclaw"));
      return;
    }

    const proc = spawn(OPENCLAW_BIN, args, {
      cwd: REPO_ROOT,
      env: { ...process.env },
      stdio: "inherit",
    });

    proc.on("close", (exitCode) => {
      exit(exitCode ? new Error(`exit ${exitCode}`) : undefined);
    });

    return () => {
      proc.kill("SIGTERM");
    };
  }, [args, exit]);

  return (
    <Box flexDirection="column">
      <Banner />
      <Text color="cyan" bold>
        ▶  OpenClaw Agent
      </Text>
    </Box>
  );
};

// ─────────────────────────────────────────────────────────────────────────────
// Entry point  — simple manual arg routing
// ─────────────────────────────────────────────────────────────────────────────
const argv = process.argv.slice(2);
const cmd = argv[0];

function getFlag(flag: string): string | undefined {
  const i = argv.indexOf(flag);
  return i >= 0 ? argv[i + 1] : undefined;
}

switch (cmd) {
  case "run": {
    const contractFile = getFlag("--file");
    const contractCode = !contractFile
      ? argv.slice(1).find((a) => !a.startsWith("-"))
      : undefined;
    render(<RunUI contractCode={contractCode} contractFile={contractFile} />);
    break;
  }

  case "analyze": {
    const positional = argv.slice(1).find((a) => !a.startsWith("-")) ?? "RaxcCreditVault.sol";
    const contract = getFlag("--contract") ?? positional;
    render(<AnalyzeUI contract={contract} />);
    break;
  }

  case "list":
  case "reports":
    render(<ListUI />);
    break;

  case "show": {
    const query = argv.slice(1).find((a) => !a.startsWith("-")) ?? "";
    if (!query) {
      console.error("  Usage: raxclaw show <filename|index>\n  Run `raxclaw list` to see available reports.");
      process.exit(1);
    }
    render(<ShowUI query={query} />);
    break;
  }

  case "agent":
    render(<AgentUI args={argv.slice(1)} />);
    break;

  case "--version":
  case "-V":
    console.log("raxclaw v1.0.0");
    break;

  case "help":
  case "--help":
  case "-h":
  default:
    render(<HelpUI />);
}
