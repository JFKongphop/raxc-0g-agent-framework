'use client';

import ReactMarkdown from 'react-markdown';
import remarkGfm from 'remark-gfm';
import { useState, useEffect, useCallback } from 'react';

const API_URL = process.env.NEXT_PUBLIC_API_URL ?? '';
const ZG_EXPLORER = 'https://storagescan-newton.0g.ai/tx';

interface ResultSectionProps {
  result: any;
}

function HashRow({ label, value, explorerUrl }: { label: string; value: string; explorerUrl?: string }) {
  const [copied, setCopied] = useState(false);
  const copy = useCallback(() => {
    navigator.clipboard.writeText(value);
    setCopied(true);
    setTimeout(() => setCopied(false), 1500);
  }, [value]);

  const short = value.length > 20 ? `${value.slice(0, 10)}...${value.slice(-8)}` : value;

  return (
    <div style={{ display: 'flex', alignItems: 'center', gap: 8, padding: '8px 0', borderBottom: '1px solid var(--border)' }}>
      <span style={{ fontSize: 12, color: 'var(--text-muted)', width: 140, flexShrink: 0 }}>{label}</span>
      <code style={{ fontSize: 12, color: 'var(--text)', flex: 1, wordBreak: 'break-all' }}>{short}</code>
      <button
        onClick={copy}
        title="Copy"
        style={{ background: 'none', border: 'none', cursor: 'pointer', color: 'var(--text-muted)', fontSize: 14, padding: '2px 6px' }}
      >
        {copied ? '✓' : '⧉'}
      </button>
      {explorerUrl && (
        <a href={explorerUrl} target="_blank" rel="noreferrer"
          style={{ fontSize: 12, color: 'var(--purple)', textDecoration: 'none', whiteSpace: 'nowrap' }}>
          ↗ View
        </a>
      )}
    </div>
  );
}

export function ResultSection({ result }: ResultSectionProps) {
  const [reportContent, setReportContent] = useState<string | null>(null);

  useEffect(() => {
    if (result?.download_url) {
      fetch(`${API_URL}${result.download_url}`)
        .then(res => res.text())
        .then(setReportContent)
        .catch(console.error);
    }
  }, [result?.download_url]);

  if (result.error) {
    return (
      <div className="card" style={{ background: 'rgba(255,69,58,0.1)', borderColor: 'var(--red)' }}>
        <div style={{ fontSize: 16, fontWeight: 600, color: 'var(--red)', marginBottom: 8 }}>❌ Error</div>
        <div style={{ fontSize: 14, color: 'var(--text-muted)' }}>{result.error}</div>
      </div>
    );
  }

  const getRiskBadge = (risk: string) => {
    const lower = risk.toLowerCase();
    if (lower.includes('critical')) return 'badge-critical';
    if (lower.includes('high')) return 'badge-high';
    if (lower.includes('medium')) return 'badge-medium';
    if (lower.includes('low')) return 'badge-low';
    return 'badge-none';
  };

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: 16 }}>

      {/* ── Summary Card ── */}
      <div className="card">
        <div className="card-header">📊 Analysis Results</div>
        <div className="result-grid">
          <div className="stat-card">
            <div className="stat-label">Vulnerability</div>
            <div className="stat-value">{result.vulnerability_found ? 'Found' : 'None'}</div>
          </div>
          <div className="stat-card">
            <div className="stat-label">Risk Level</div>
            <div className="stat-value">
              <span className={`badge ${getRiskBadge(result.risk_level || '')}`}>{result.risk_level || 'None'}</span>
            </div>
          </div>
          <div className="stat-card">
            <div className="stat-label">Confidence</div>
            <div className="stat-value">{result.confidence ?? '?'}%</div>
          </div>
          <div className="stat-card">
            <div className="stat-label">Type</div>
            <div className="stat-value" style={{ fontSize: 14 }}>{result.vulnerability_type || 'N/A'}</div>
          </div>
        </div>

        {result.download_url && (
          <a
            href={`${API_URL}${result.download_url}`}
            download
            className="btn btn-secondary"
            style={{ width: '100%', justifyContent: 'center', marginTop: 12 }}
          >
            📄 Download Full Report ({result.report_filename || 'report.md'})
          </a>
        )}
      </div>

      {/* ── On-Chain Verification Card ── */}
      {(result.storage_root_hash || result.report_root_hash || result.replay_id || result.trace_hash) && (
        <div className="card">
          <div className="card-header">🔗 On-Chain Verification (0G Storage)</div>
          <div style={{ marginTop: 8 }}>
            {result.storage_root_hash && (
              <HashRow
                label="0G JSON Root Hash"
                value={result.storage_root_hash}
                explorerUrl={`${ZG_EXPLORER}/${result.storage_root_hash}`}
              />
            )}
            {result.report_root_hash && (
              <HashRow
                label="0G Report Root Hash"
                value={result.report_root_hash}
                explorerUrl={`${ZG_EXPLORER}/${result.report_root_hash}`}
              />
            )}
            {result.replay_id && (
              <HashRow label="Attestation Replay ID" value={result.replay_id} />
            )}
            {result.trace_hash && (
              <HashRow label="Execution Trace Hash" value={result.trace_hash} />
            )}
            {result.report_filename && (
              <HashRow label="Report Filename" value={result.report_filename} />
            )}
          </div>
          <div style={{ marginTop: 12, fontSize: 12, color: 'var(--text-muted)' }}>
            This audit result is permanently stored on 0G Storage and anchored via ERC-7857 on 0G Galileo (chain 16602).
          </div>
        </div>
      )}

      {/* ── Download RAXC CLI Binary ── */}
      <div className="card">
        <div className="card-header">⬇️ Download RAXC CLI</div>
        <div style={{ fontSize: 13, color: 'var(--text-muted)', marginBottom: 16 }}>
          Run audits locally — same Step 9.9 pipeline, 722 exploit RAG, full attestation. No setup required.
        </div>
        <div style={{ display: 'flex', gap: 10, flexWrap: 'wrap' }}>
          <a
            href={`${API_URL}/downloads/raxclaw-darwin-arm64`}
            download="raxclaw"
            className="btn btn-secondary"
            style={{ flex: 1, minWidth: 160, justifyContent: 'center' }}
          >
            🍎 macOS (Apple Silicon)
          </a>
          <a
            href={`${API_URL}/downloads/raxclaw-darwin-x64`}
            download="raxclaw"
            className="btn btn-secondary"
            style={{ flex: 1, minWidth: 160, justifyContent: 'center' }}
          >
            🍎 macOS (Intel)
          </a>
          <a
            href={`${API_URL}/downloads/raxclaw-linux-x64`}
            download="raxclaw"
            className="btn btn-secondary"
            style={{ flex: 1, minWidth: 160, justifyContent: 'center' }}
          >
            🐧 Linux (x64)
          </a>
        </div>
        <div style={{ marginTop: 10, fontSize: 12, color: 'var(--text-muted)' }}>
          After download: <code style={{ background: 'var(--surface)', padding: '2px 6px', borderRadius: 4 }}>chmod +x raxclaw && ./raxclaw run "pragma solidity..."</code>
        </div>
      </div>

      {/* ── Full Report Card ── */}
      {reportContent && (
        <div className="card">
          <div className="card-header">📋 Full Security Report</div>
          <div className="markdown-body">
            <ReactMarkdown remarkPlugins={[remarkGfm]}>{reportContent}</ReactMarkdown>
          </div>
        </div>
      )}

    </div>
  );
}
