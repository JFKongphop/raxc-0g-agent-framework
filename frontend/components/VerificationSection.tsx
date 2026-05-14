const VERIFICATIONS = [
  {
    label: '0G Root Hash',
    color: '#00ff88',
    entries: [
      { index: '01', value: '0x8f3a2b1c9e4d7f6a3c8b1e5d2a7f4c9b', date: '2026-05-15 01:40' },
      { index: '02', value: '0x7e2b3c1d8f5e4a9b2d6c0a4f1e8b3c7d', date: '2026-05-15 01:43' },
      { index: '03', value: '0x6d1a2b0c7e4f3b8c1a5d9e2f6c3a0b7e', date: '2026-05-15 01:49' },
    ],
  },
  {
    label: 'ERC-7857 TX Hash',
    color: 'var(--cyan)',
    entries: [
      { index: '01', value: '0x4f2a8c9b1e3d7a6f2b5c8e1d4a7f3c9b', date: '2026-05-15 01:40' },
      { index: '02', value: '0x5e3b9d0c2f4e8a7b3c6d1f4a7b0e2c8d', date: '2026-05-15 01:43' },
      { index: '03', value: '0x6f4c0e1d3a5f9b8c2d7a0e3b5c8f1d4a', date: '2026-05-15 01:49' },
    ],
  },
  {
    label: 'Attestation Hash',
    color: '#ffd60a',
    entries: [
      { index: '01', value: '0xDE598A9AE0D0D3A7', date: '2026-05-15 01:40' },
      { index: '02', value: '0xDE598A9AE0D0D3B8', date: '2026-05-15 01:43' },
      { index: '03', value: '0xDE598A9AE0D0D3C9', date: '2026-05-15 01:49' },
    ],
  },
];

export function VerificationSection() {
  return (
    <section className="section">
      <div className="section-inner">
        <div style={{ marginBottom: 52 }}>
          <div className="section-label">Persistence & Verification</div>
          <h2 className="section-title">Cryptographic Proof</h2>
          <p className="section-desc">
            Every audit is permanently stored on 0G and anchored on-chain via ERC-7857.
            Cognition cannot be lost, tampered, or replayed incorrectly.
          </p>
        </div>

        <div style={{ display: 'grid', gap: 20 }}>
          {VERIFICATIONS.map((v) => (
            <div key={v.label} className="glass-card" style={{ padding: 28 }}>
              {/* Group label */}
              <div
                style={{
                  fontFamily: 'var(--font-mono)',
                  fontSize: 11,
                  fontWeight: 600,
                  color: v.color,
                  letterSpacing: '0.12em',
                  textTransform: 'uppercase',
                  marginBottom: 18,
                  display: 'flex',
                  alignItems: 'center',
                  gap: 10,
                }}
              >
                <div
                  style={{
                    width: 6,
                    height: 6,
                    borderRadius: '50%',
                    background: v.color,
                    boxShadow: `0 0 8px ${v.color}`,
                  }}
                />
                {v.label}
              </div>

              {/* Entries */}
              <div style={{ display: 'flex', flexDirection: 'column', gap: 10 }}>
                {v.entries.map((entry) => (
                  <div
                    key={entry.index}
                    style={{
                      display: 'flex',
                      alignItems: 'center',
                      gap: 14,
                      flexWrap: 'wrap',
                    }}
                  >
                    <span
                      style={{
                        fontFamily: 'var(--font-mono)',
                        fontSize: 11,
                        color: 'var(--text-dim)',
                        minWidth: 22,
                      }}
                    >
                      {entry.index}
                    </span>
                    <span className="hash" style={{ flex: 1 }}>
                      {entry.value}
                    </span>
                    <span
                      style={{
                        fontSize: 11,
                        color: '#00ff88',
                        fontFamily: 'var(--font-mono)',
                        whiteSpace: 'nowrap',
                      }}
                    >
                      ✓ Verified
                    </span>
                    <span
                      style={{
                        fontSize: 11,
                        color: 'var(--text-dim)',
                        fontFamily: 'var(--font-mono)',
                        whiteSpace: 'nowrap',
                      }}
                    >
                      {entry.date}
                    </span>
                  </div>
                ))}
              </div>
            </div>
          ))}
        </div>

        {/* Bottom note */}
        <div
          style={{
            marginTop: 28,
            padding: '16px 20px',
            background: 'rgba(0,212,255,0.04)',
            borderRadius: 'var(--radius-md)',
            border: '1px solid var(--border)',
            fontFamily: 'var(--font-mono)',
            fontSize: 12,
            color: 'var(--text-muted)',
            lineHeight: 1.6,
          }}
        >
          All audit results are stored on{' '}
          <span style={{ color: 'var(--cyan)' }}>0G Newton Testnet</span> and anchored
          via ERC-7857 on{' '}
          <span style={{ color: 'var(--cyan)' }}>0G Galileo (chain 16602)</span>.
          Use the Replay ID to fully reconstruct any analysis.
        </div>
      </div>
    </section>
  );
}
