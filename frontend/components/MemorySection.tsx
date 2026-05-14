'use client';

import { useEffect, useState } from 'react';
import { fetchAuditTasks, OnChainAudit } from '@/lib/contracts';

function verdictColor(v: string) {
  if (v.toUpperCase().includes('HIGH'))   return 'var(--orange)';
  if (v.toUpperCase().includes('MEDIUM')) return 'var(--yellow)';
  return '#00ff88';
}

function shortVerdict(v: string) {
  const u = v.toUpperCase();
  if (u.includes('CRITICAL')) return 'CRITICAL';
  if (u.includes('HIGH'))     return 'HIGH';
  if (u.includes('MEDIUM'))   return 'MEDIUM';
  return 'LOW';
}

function formatDate(d: Date) {
  return d.toLocaleString(undefined, {
    year: 'numeric', month: '2-digit', day: '2-digit',
    hour: '2-digit', minute: '2-digit', hour12: false,
  });
}

export function MemorySection() {
  const [audits, setAudits] = useState<OnChainAudit[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    fetchAuditTasks().then(setAudits).finally(() => setLoading(false));
  }, []);

  // audits contains every on-chain entry (no slice cap)
  const total = audits.length;

  return (
    <section className="section">
      <div className="section-inner">
        <div className="grid-2" style={{ alignItems: 'start' }}>

          {/* Left */}
          <div>
            <div className="section-label">Memory Evolution</div>
            <h2 className="section-title">Growing Cognition</h2>
            <p className="section-desc" style={{ marginBottom: 36 }}>
              Each audit accumulates in long-context memory. 
              <br />
              The agent learns from
              every execution building persistent intelligence over time that
              improves future analyses.
            </p>

            <div className="glass-card" style={{ padding: '28px 26px', display: 'inline-block', minWidth: 280 }}>
              {loading ? (
                <div style={{ fontFamily: 'var(--font-mono)', fontSize: 13, color: 'var(--text-dim)' }}>Loading…</div>
              ) : (
                <>
                  <div style={{ fontFamily: 'var(--font-mono)', fontSize: 52, fontWeight: 700, color: 'var(--cyan)', lineHeight: 1, marginBottom: 6, letterSpacing: '-0.03em' }}>
                    {total}
                  </div>
                  <div style={{ fontSize: 14, color: 'var(--text-muted)', marginBottom: 16 }}>
                    on-chain cognition entries
                  </div>
                  <div style={{ height: 4, background: 'rgba(0,212,255,0.1)', borderRadius: 2, overflow: 'hidden', marginBottom: 8 }}>
                    <div style={{ height: '100%', width: total > 0 ? '100%' : '0%', background: 'linear-gradient(90deg, var(--blue), var(--cyan))', borderRadius: 2, boxShadow: '0 0 8px rgba(0,212,255,0.5)', transition: 'width 0.6s ease' }} />
                  </div>
                  <div style={{ fontFamily: 'var(--font-mono)', fontSize: 11, color: 'var(--text-dim)' }}>
                    {total} verified · stored on 0G Storage
                  </div>
                </>
              )}
            </div>

            {/* Contract addresses */}
            <div style={{ marginTop: 20, display: 'flex', flexDirection: 'column', gap: 8 }}>
              {[
                { label: 'ERC-8183', address: '0x6ffc92b063fc470dd2d4cbd0f64e75ed96ae7a8c', color: '#a0ff60' },
                { label: 'ERC-7857', address: '0xe3c7863ad3176e88e9c75a580fc15a2976d5ff53', color: '#00ff88' },
              ].map(({ label, address, color }) => (
                <div key={label} style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
                  <span style={{ fontFamily: 'var(--font-mono)', fontSize: 10, fontWeight: 700, color, minWidth: 56 }}>{label}</span>
                  <a
                    href={`https://chainscan-galileo.0g.ai/address/${address}`}
                    target="_blank"
                    rel="noopener noreferrer"
                    style={{ fontFamily: 'var(--font-mono)', fontSize: 10, color: 'var(--text-dim)', textDecoration: 'none', letterSpacing: '0.02em' }}
                  >
                    {address}
                  </a>
                </div>
              ))}
            </div>
          </div>

          {/* Right: live list */}
          <div style={{ display: 'flex', flexDirection: 'column', gap: 8, maxHeight: 500, overflowY: 'auto' }}>
            {loading && (
              <div style={{ fontFamily: 'var(--font-mono)', fontSize: 12, color: 'var(--text-dim)', padding: '20px 0' }}>Fetching from chain…</div>
            )}
            {!loading && audits.length === 0 && (
              <div style={{ fontFamily: 'var(--font-mono)', fontSize: 12, color: 'var(--text-dim)', padding: '20px 0' }}>No entries yet.</div>
            )}
            {audits.map((audit, i) => (
              <div key={audit.taskId} className="glass-card" style={{ padding: '12px 18px', borderColor: 'var(--border-strong)' }}>
                <div style={{ display: 'flex', alignItems: 'center', gap: 12 }}>
                  <span style={{ fontFamily: 'var(--font-mono)', fontSize: 10, color: 'var(--text-dim)', minWidth: 36 }}>
                    #{String(i + 1).padStart(3, '0')}
                  </span>
                  <span style={{ fontWeight: 600, fontSize: 13, flex: 1, fontFamily: 'var(--font-mono)', color: 'var(--cyan)' }}>
                    {audit.contractName ?? audit.replayId ?? `Task #${audit.taskId}`}
                  </span>
                  <span style={{ fontFamily: 'var(--font-mono)', fontSize: 12, fontWeight: 700, color: verdictColor(audit.verdict) }}>
                    {shortVerdict(audit.verdict)}
                  </span>
                  <span style={{ fontSize: 10, color: 'var(--text-dim)', fontFamily: 'var(--font-mono)', whiteSpace: 'nowrap' }}>
                    {formatDate(audit.completedAt)}
                  </span>
                </div>
              </div>
            ))}
          </div>

        </div>
      </div>
    </section>
  );
}


