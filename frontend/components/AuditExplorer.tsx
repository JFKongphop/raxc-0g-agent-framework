'use client';

import { useEffect, useState } from 'react';
import { useRouter } from 'next/navigation';
import { fetchAuditTasks, OnChainAudit } from '@/lib/contracts';

function shortHash(h: string, len = 12): string {
  if (!h || h.length <= len + 2) return h;
  return h.slice(0, len + 2) + '\u2026';
}

function formatDate(d: Date): string {
  return d.toLocaleString(undefined, { year: 'numeric', month: '2-digit', day: '2-digit', hour: '2-digit', minute: '2-digit', second: '2-digit', hour12: false });
}

export function AuditExplorer() {
  const router = useRouter();
  const [audits, setAudits] = useState<OnChainAudit[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError]   = useState(false);

  useEffect(() => {
    fetchAuditTasks()
      .then(setAudits)
      .catch(() => setError(true))
      .finally(() => setLoading(false));
  }, []);

  return (
    <section className="section" id="audits">
      <div className="section-inner">
        <div style={{ marginBottom: 52 }}>
          <div className="section-label">Audit Explorer</div>
          <h2 className="section-title">Cognition History</h2>
          <p className="section-desc">
            Live audit records from the ERC-8183 contract on 0G Galileo. 
            <br />
            Click any row to view the full security report stored on 0G Storage.
          </p>
        </div>

        {loading && (
          <div style={{ fontFamily: 'var(--font-mono)', fontSize: 12, color: 'var(--text-dim)', padding: '40px 0' }}>
            Fetching audit tasks from 0G Galileo…
          </div>
        )}

        {error && !loading && (
          <div style={{ fontFamily: 'var(--font-mono)', fontSize: 12, color: 'var(--yellow)', padding: '40px 0' }}>
            ⚠ RPC error — audit data unavailable.
          </div>
        )}

        {!loading && !error && audits.length === 0 && (
          <div style={{ fontFamily: 'var(--font-mono)', fontSize: 12, color: 'var(--text-dim)', padding: '40px 0' }}>
            No finalized audit tasks on-chain yet — or RPC is temporarily unreachable.
          </div>
        )}

        <div style={{ display: 'grid', gap: 12 }}>
          {audits.map((audit) => (
            (
              <div
                key={audit.taskId}
                className="glass-card"
                style={{ padding: '0', cursor: 'pointer', overflow: 'hidden', display: 'flex' }}
                onClick={() => router.push(`/roothash/${audit.rootHash}${audit.txHash ? `?tx=${audit.txHash}` : ''}`)}
              >
                {/* Left accent bar */}
                <div style={{ width: 3, flexShrink: 0, background: 'var(--cyan)', opacity: 0.7 }} />

                <div style={{ flex: 1, padding: '20px 24px', display: 'flex', alignItems: 'center', gap: 20, flexWrap: 'wrap' }}>
                  <div style={{ flex: 1, minWidth: 200 }}>
                    <div style={{ fontFamily: 'var(--font-mono)', fontSize: 13, fontWeight: 700, letterSpacing: '0.04em', color: 'var(--cyan)', marginBottom: 6 }}>
                      {audit.contractName ?? shortHash(audit.replayId ?? audit.taskId, 16)}
                    </div>
                    <div style={{ display: 'flex', alignItems: 'center', gap: 12 }}>
                      <span style={{ fontFamily: 'var(--font-mono)', fontSize: 11, color: 'var(--text-muted)' }}>
                        {formatDate(audit.completedAt)}
                      </span>
                      {audit.verdict && (
                        <>
                          <span style={{ color: 'rgba(255,255,255,0.15)', fontSize: 10 }}>|</span>
                          <span style={{ fontFamily: 'var(--font-mono)', fontSize: 11, color: 'rgba(255,255,255,0.45)', textTransform: 'uppercase', letterSpacing: '0.08em' }}>
                            {audit.verdict.replace(/_/g, ' ')}
                          </span>
                        </>
                      )}
                    </div>
                  </div>
                  <div style={{ display: 'flex', alignItems: 'center', gap: 14 }}>
                    <span className="hash" style={{ fontSize: 11 }}>
                      {shortHash(audit.rootHash)}
                    </span>
                    <span style={{ color: 'var(--cyan)', fontSize: 16, opacity: 0.8 }}>→</span>
                  </div>
                </div>
              </div>
            )
          ))}
        </div>
      </div>
    </section>
  );
}
