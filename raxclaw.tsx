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

// Gradient colours for the 8 rows of BANNER (row 0 and 7 are blank padding)
const BANNER_COLORS = [
  'cyan',      // 0 blank
  'cyan',      // 1 top
  '#4dd9ff',   // 2 light blue
  '#818cf8',   // 3 indigo
  '#a78bfa',   // 4 violet
  '#e879f9',   // 5 fuchsia
  '#f472b6',   // 6 pink  ╚═╝ bottom row
  'cyan',      // 7 blank
];

const Banner: FC = () => (
  <Box flexDirection="column" marginBottom={1}>
    {BANNER.map((line, i) => (
      <Text key={i} bold color={BANNER_COLORS[i] ?? 'cyan'}>
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
          <Text key={line.id}>
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
  const dir = path.join(REPO_ROOT, "backend", "reports");
  const reports: ReportMeta[] = [];
  if (!fs.existsSync(dir)) return reports;
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
  return reports.sort((a, b) => a.name.localeCompare(b.name));
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
// Markdown renderer helpers
// ─────────────────────────────────────────────────────────────────────────────

type InlineSeg = { text: string; bold?: boolean; code?: boolean };

function parseInline(raw: string): InlineSeg[] {
  const result: InlineSeg[] = [];
  const re = /\*\*([^*]+)\*\*|`([^`]+)`|([^*`]+)/g;
  let m: RegExpExecArray | null;
  while ((m = re.exec(raw)) !== null) {
    if (m[1] !== undefined) result.push({ text: m[1], bold: true });
    else if (m[2] !== undefined) result.push({ text: m[2], code: true });
    else if (m[3] !== undefined) result.push({ text: m[3] });
  }
  return result.length ? result : [{ text: raw }];
}

const InlineLine: FC<{ raw: string; color?: string }> = ({ raw, color }) => (
  <Text>
    {parseInline(raw).map((seg, i) =>
      seg.bold ? (
        <Text key={i} bold color={color ?? "white"}>{seg.text}</Text>
      ) : seg.code ? (
        <Text key={i} color="yellow">{seg.text}</Text>
      ) : (
        <Text key={i} color={color}>{seg.text}</Text>
      )
    )}
  </Text>
);

// ─────────────────────────────────────────────────────────────────────────────
// Show a single report — Ink-rendered markdown
// ─────────────────────────────────────────────────────────────────────────────
const ShowUI: FC<{ query: string }> = ({ query }) => {
  const { exit } = useApp();
  useEffect(() => { exit(); }, [exit]);

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
  const rawLines = content.split("\n");
  const W = Math.min((process.stdout.columns ?? 100) - 4, 96);
  const divider = "─".repeat(W);

  // ── Pre-process lines into typed blocks (handles code fences correctly) ───
  type Block =
    | { kind: "h1" | "h2" | "h3" | "h4"; text: string }
    | { kind: "rule" | "blank" | "codeend" }
    | { kind: "bullet"; indent: number; text: string }
    | { kind: "numbered"; num: string; text: string }
    | { kind: "blockquote"; text: string }
    | { kind: "tablerow"; cells: string[]; isSep: boolean }
    | { kind: "codestart"; lang: string }
    | { kind: "codeline"; text: string }
    | { kind: "text"; text: string };

  const blocks: Block[] = [];
  let inCode = false;
  for (const line of rawLines) {
    if (line.startsWith("```")) {
      inCode = !inCode;
      if (inCode) blocks.push({ kind: "codestart", lang: line.slice(3).trim() });
      else         blocks.push({ kind: "codeend" });
      continue;
    }
    if (inCode)            { blocks.push({ kind: "codeline", text: line }); continue; }
    if (line.trim() === "") { blocks.push({ kind: "blank" }); continue; }
    if (line.startsWith("# "))    { blocks.push({ kind: "h1", text: line.slice(2) }); continue; }
    if (line.startsWith("## "))   { blocks.push({ kind: "h2", text: line.slice(3) }); continue; }
    if (line.startsWith("### "))  { blocks.push({ kind: "h3", text: line.slice(4) }); continue; }
    if (line.startsWith("#### ")) { blocks.push({ kind: "h4", text: line.slice(5) }); continue; }
    if (/^[-*_]{3,}\s*$/.test(line)) { blocks.push({ kind: "rule" }); continue; }
    const bm = line.match(/^(\s*)[-*] (.*)/);
    if (bm) { blocks.push({ kind: "bullet", indent: bm[1].length, text: bm[2] }); continue; }
    const nm = line.match(/^(\d+)\. (.*)/);
    if (nm) { blocks.push({ kind: "numbered", num: nm[1], text: nm[2] }); continue; }
    if (line.startsWith("> ")) { blocks.push({ kind: "blockquote", text: line.slice(2) }); continue; }
    if (line.startsWith("|")) {
      const stripMdLink = (s: string) => s.replace(/\[([^\]]+)\]\([^)]+\)/g, "$1");
      const cells = line.split("|").slice(1, -1).map(c => stripMdLink(c.trim()));
      blocks.push({ kind: "tablerow", cells, isSep: cells.every(c => /^[-: ]+$/.test(c)) });
      continue;
    }
    blocks.push({ kind: "text", text: line });
  }

  // Pre-compute max column widths per consecutive table group
  const tableColWidths = new Map<number, number[]>();
  {
    let ws: number[] = [];
    let lastIdx = -2;
    for (let idx = 0; idx < blocks.length; idx++) {
      const blk = blocks[idx];
      if (blk.kind === "tablerow") {
        if (idx !== lastIdx + 1) ws = [];
        lastIdx = idx;
        if (!blk.isSep) blk.cells.forEach((cell, ci) => { ws[ci] = Math.max(ws[ci] ?? 0, cell.length); });
        tableColWidths.set(idx, ws);
      }
    }
  }

  return (
    <Box flexDirection="column" paddingX={1}>
      <Banner />
      <Box flexDirection="column" marginBottom={1}>
        <Text bold color="cyanBright">{"📄  "}{found.name}</Text>
        <Text color="gray" dimColor>{found.filePath}</Text>
      </Box>
      <Text color="gray">{divider}</Text>
      <Newline />

      {blocks.map((b, i) => {
        switch (b.kind) {
          case "h1": return (
            <Box key={i} flexDirection="column" marginTop={1}>
              <Text bold color="cyanBright">{"┌─  "}<Text bold color="whiteBright">{b.text}</Text></Text>
              <Text color="cyan">{"└" + "─".repeat(Math.min(b.text.length + 5, W - 1))}</Text>
            </Box>
          );
          case "h2": return (
            <Box key={i} flexDirection="column" marginTop={1}>
              <Text bold color="yellowBright">{"  ▶  "}{b.text}</Text>
              <Text color="yellow" dimColor>{"  " + "─".repeat(Math.min(b.text.length + 5, W - 2))}</Text>
            </Box>
          );
          case "h3": return (
            <Text key={i} bold color="greenBright">{"    ◆  "}{b.text}</Text>
          );
          case "h4": return (
            <Text key={i} bold color="whiteBright">{"      ▸  "}{b.text}</Text>
          );
          case "rule": return (
            <Text key={i} color="gray" dimColor>{divider}</Text>
          );
          case "blank": return <Newline key={i} />;
          case "bullet": return (
            <Text key={i}>
              <Text color="cyan">{"  ".repeat(Math.floor(b.indent / 2)) + "  • "}</Text>
              <InlineLine raw={b.text} />
            </Text>
          );
          case "numbered": return (
            <Text key={i}>
              <Text bold color="cyan">{`  ${b.num.padStart(2)}.  `}</Text>
              <InlineLine raw={b.text} />
            </Text>
          );
          case "blockquote": return (
            <Text key={i}>
              <Text bold color="cyan">{"  │ "}</Text>
              <Text color="white" dimColor>{b.text}</Text>
            </Text>
          );
          case "tablerow": {
            const widths = tableColWidths.get(i) ?? b.cells.map(c => c.length);
            if (b.isSep) return (
              <Text key={i} color="gray" dimColor>
                {"  " + widths.map(w => "─".repeat(Math.max(w, 3))).join("─┼─")}
              </Text>
            );
            const isHeader = blocks[i + 1]?.kind === "tablerow" && (blocks[i + 1] as any).isSep;
            return (
              <Text key={i}>
                <Text color="gray">{"  "}</Text>
                {b.cells.map((cell, ci) => (
                  <Text key={ci}>
                    {ci > 0 ? <Text color="gray">{" │ "}</Text> : null}
                    <Text bold={isHeader} color={isHeader ? "whiteBright" : ci === 0 ? "cyan" : ci === 1 ? "yellowBright" : "white"}>
                      {cell.padEnd(widths[ci] ?? cell.length)}
                    </Text>
                  </Text>
                ))}
              </Text>
            );
          }
          case "codestart": {
            const lang = b.lang || "code";
            const fill = "═".repeat(Math.max(0, W - 7 - lang.length));
            return (
              <Box key={i} marginTop={1}>
                <Text color="gray">{"  ╔═ "}<Text color="yellow">{lang}</Text><Text color="gray">{" " + fill + "╗"}</Text></Text>
              </Box>
            );
          }
          case "codeend": return (
            <Text key={i} color="gray">{"  ╚" + "═".repeat(Math.max(0, W - 4)) + "╝"}</Text>
          );
          case "codeline": {
            const maxText = W - 7;
            const display = b.text.length > maxText ? b.text.slice(0, maxText - 1) + "…" : b.text;
            const pad = " ".repeat(Math.max(0, maxText - display.length));
            return (
              <Text key={i}>
                <Text color="gray">{"  ║  "}</Text>
                <Text color="greenBright">{display}</Text>
                <Text color="gray">{pad + " ║"}</Text>
              </Text>
            );
          }
          case "text": return <Text key={i}><InlineLine raw={b.text} /></Text>;
          default: return null;
        }
      })}

      <Newline />
      <Text color="gray">{divider}</Text>
      <Text color="gray" dimColor>{"  "}{found.name}{"  ·  raxclaw v1.0.0"}</Text>
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
