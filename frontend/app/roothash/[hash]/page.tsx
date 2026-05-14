'use client';

import { useEffect, useState, useCallback } from 'react';
import { useRouter, useParams } from 'next/navigation';
import ReactMarkdown from 'react-markdown';
import remarkGfm from 'remark-gfm';
import { OG_STORAGE_GATEWAY } from '@/lib/contracts';

// ── 0G Storage binary parsing (mirrors og_storage.rs download_and_parse) ─────
//
// 0g-cli stores files in a binary stream format:
//   [binary header][stream_id bytes][key string][binary][base64(content)][binary]
// We mirror the Rust regex logic: find the longest base64 block (≥100 chars),
// decode it, and try to parse as UTF-8 text (markdown or JSON).

const B64_RE = /[A-Za-z0-9+/]{100,}={0,2}/g;

function extractPayloadFromBinary(buf: ArrayBuffer): string | null {
  // Convert bytes to a binary-safe Latin-1 string so the regex can run over raw bytes
  const bytes = new Uint8Array(buf);
  let binStr = '';
  for (let i = 0; i < bytes.length; i++) {
    binStr += String.fromCharCode(bytes[i]);
  }

  // Collect all base64 candidates, longest first
  const matches: string[] = [];
  let m: RegExpExecArray | null;
  const re = new RegExp(B64_RE.source, 'g');
  while ((m = re.exec(binStr)) !== null) matches.push(m[0]);
  matches.sort((a, b) => b.length - a.length);

  for (const candidate of matches) {
    try {
      const decoded = atob(candidate);
      // Convert the decoded binary string back to a UTF-8 Uint8Array
      const raw = new Uint8Array(decoded.length);
      for (let i = 0; i < decoded.length; i++) raw[i] = decoded.charCodeAt(i);
      const text = new TextDecoder('utf-8', { fatal: true }).decode(raw);
      if (text.trim().length > 10) return text;
    } catch {
      // not valid UTF-8 — try next candidate
    }
  }
  return null;
}

const GATEWAY_URLS = [
  (h: string) => `${OG_STORAGE_GATEWAY}/file?root=${h}`,
  (h: string) => `${OG_STORAGE_GATEWAY}/download/${h}`,
  (h: string) => `https://rpc-storage-testnet.0g.ai/file?root=${h}`,
];

async function downloadFromOgStorage(rootHash: string): Promise<string | null> {
  for (const buildUrl of GATEWAY_URLS) {
    try {
      const res = await fetch(buildUrl(rootHash), {
        signal: AbortSignal.timeout(10000),
      });
      if (!res.ok) continue;

      // Read as binary buffer — the 0G format is NOT plain text
      const buf = await res.arrayBuffer();

      // 1. Try extracting the base64 payload embedded in the binary stream
      const payload = extractPayloadFromBinary(buf);
      if (payload) {
        // If it's JSON, convert to readable markdown
        if (payload.trimStart().startsWith('{')) {
          try {
            return jsonToMarkdown(rootHash, JSON.parse(payload));
          } catch { /* fall through and show raw */ }
        }
        // If it's already markdown or plain text, use directly
        return payload;
      }

      // 2. Last resort: try decoding the whole buffer as UTF-8 (plain file case)
      try {
        const text = new TextDecoder('utf-8', { fatal: true }).decode(buf);
        if (text.trim().length > 10) return text;
      } catch { /* binary-only, skip */ }
    } catch {
      // network error — try next URL
    }
  }
  return null;
}

// ── Helpers ───────────────────────────────────────────────────────────────────

function CopyButton({ value }: { value: string }) {
  const [copied, setCopied] = useState(false);
  return (
    <button
      onClick={() => {
        navigator.clipboard.writeText(value).catch(() => undefined);
        setCopied(true);
        setTimeout(() => setCopied(false), 1500);
      }}
      style={{
        background: 'none',
        border: '1px solid rgba(0,212,255,0.2)',
        color: copied ? '#00ff88' : 'var(--cyan)',
        borderRadius: 4,
        padding: '3px 8px',
        fontSize: 10,
        fontFamily: 'var(--font-mono)',
        cursor: 'pointer',
        whiteSpace: 'nowrap',
      }}
    >
      {copied ? 'copied' : 'copy'}
    </button>
  );
}

// ── Page ──────────────────────────────────────────────────────────────────────

export default function RootHashPage() {
  const router = useRouter();
  const params = useParams<{ hash: string }>();
  const hash   = params.hash;

  // Read optional tx query param passed from AuditExplorer
  const [txHash, setTxHash] = useState<string | null>(null);
  useEffect(() => {
    const p = new URLSearchParams(window.location.search);
    setTxHash(p.get('tx'));
  }, []);

  const [content,  setContent]  = useState<string>('');
  const [fetching, setFetching] = useState(true);
  const [failed,   setFailed]   = useState(false);

  const load = useCallback(async () => {
    setFetching(true);
    setFailed(false);

    const raw = await downloadFromOgStorage(hash);

    if (raw) {
      // downloadFromOgStorage already handles JSON→markdown conversion
      setContent(raw);
    } else {
      setContent(buildHashInfoMarkdown(hash));
      setFailed(true);
    }

    setFetching(false);
  }, [hash]);

  useEffect(() => { load(); }, [load]);

  return (
    <main style={{ minHeight: '100vh', background: 'var(--bg)', color: 'var(--text)' }}>
      {/* Top bar */}
      <div style={{ borderBottom: '1px solid rgba(0,212,255,0.1)', padding: '16px 32px', display: 'flex', alignItems: 'center', gap: 20 }}>
        <button
          onClick={() => router.push('/#audits')}
          style={{ background: 'none', border: '1px solid rgba(0,212,255,0.2)', color: 'var(--cyan)', borderRadius: 6, padding: '6px 16px', cursor: 'pointer', fontFamily: 'var(--font-mono)', fontSize: 11 }}
        >
          &#8592; Back
        </button>
        <div style={{ fontFamily: 'var(--font-mono)', fontSize: 11, color: 'var(--text-dim)' }}>
          0G Storage · Root Hash
        </div>
      </div>

      <div style={{ maxWidth: 900, margin: '0 auto', padding: '40px 32px 80px' }}>
        {/* Hash header */}
        <div className="glass-card" style={{ padding: '20px 24px', marginBottom: 32 }}>
          <div style={{ fontSize: 10, letterSpacing: '0.12em', textTransform: 'uppercase', color: 'var(--cyan)', fontFamily: 'var(--font-mono)', marginBottom: 10 }}>
            0G Storage Merkle Root
          </div>
          <div style={{ display: 'flex', alignItems: 'center', gap: 10 }}>
            <span className="hash" style={{ fontSize: 12, wordBreak: 'break-all', flex: 1 }}>{hash}</span>
            <CopyButton value={hash} />
            {txHash && (
              <a
                href={`https://chainscan-galileo.0g.ai/tx/${txHash}`}
                target="_blank"
                rel="noopener noreferrer"
                style={{ background: 'none', border: '1px solid rgba(0,212,255,0.2)', color: 'var(--cyan)', borderRadius: 4, padding: '3px 8px', fontSize: 10, fontFamily: 'var(--font-mono)', cursor: 'pointer', whiteSpace: 'nowrap', textDecoration: 'none' }}
              >
                chain
              </a>
            )}
          </div>
          {failed && (
            <div style={{ marginTop: 14, fontFamily: 'var(--font-mono)', fontSize: 11, color: 'var(--yellow)' }}>
              &#9888; Could not reach 0G Storage gateway — showing hash info only.
            </div>
          )}
        </div>

        {/* Content */}
        {fetching ? (
          <div style={{ fontFamily: 'var(--font-mono)', fontSize: 12, color: 'var(--text-dim)' }}>
            Fetching report from 0G Storage&#8230;
          </div>
        ) : (
          <div className="report-content">
            <ReactMarkdown remarkPlugins={[remarkGfm]}>{content}</ReactMarkdown>
          </div>
        )}
      </div>
    </main>
  );
}

// ── Formatters ────────────────────────────────────────────────────────────────

function jsonToMarkdown(hash: string, obj: Record<string, unknown>): string {
  const vuln  = String(obj.vulnerability_type ?? obj.vuln_type ?? 'Unknown');
  const risk  = String(obj.risk_level ?? obj.verdict ?? 'Unknown');
  const conf  = obj.confidence ?? obj.confidence_score ?? '—';
  const lines: string[] = [
    `# Audit Report`,
    '',
    `## Summary`,
    '',
    `| Field | Value |`,
    `|-------|-------|`,
    `| **Vulnerability** | ${vuln} |`,
    `| **Risk Level** | ${risk} |`,
    `| **Confidence** | ${conf}% |`,
    `| **Root Hash** | \`${hash}\` |`,
    '',
  ];
  if (obj.explanation) {
    lines.push('## Analysis', '', String(obj.explanation), '');
  }
  if (obj.code_snippet) {
    lines.push('## Vulnerable Code', '', '```solidity', String(obj.code_snippet), '```', '');
  }
  if (obj.recommendation) {
    lines.push('## Recommendation', '', String(obj.recommendation), '');
  }
  // Dump remaining fields
  const shown = new Set(['vulnerability_type', 'vuln_type', 'risk_level', 'verdict', 'confidence', 'confidence_score', 'explanation', 'code_snippet', 'recommendation']);
  const rest = Object.entries(obj).filter(([k]) => !shown.has(k));
  if (rest.length > 0) {
    lines.push('## Additional Fields', '');
    for (const [k, v] of rest) {
      lines.push(`**${k}**: ${String(v)}  `);
    }
  }
  return lines.join('\n');
}

function buildHashInfoMarkdown(hash: string): string {
  return `# 0G Storage Entry

## Root Hash

\`${hash}\`

## Status

The 0G Storage HTTP gateway did not return content for this root hash.

This can happen when:
- The file is stored on a storage node not accessible via the public gateway
- The testnet indexer has not yet propagated this entry
- The entry requires the \`0g-cli\` tool to download directly

## Download via CLI

\`\`\`bash
0g-cli download --indexer https://indexer-storage-testnet-turbo.0g.ai \\
  --root ${hash} \\
  --file ./report.json
\`\`\`
`;
}
