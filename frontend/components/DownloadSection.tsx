const PLATFORMS = [
  {
    os: 'macOS',
    sub: 'Apple Silicon · Intel x86_64',
    cmd: 'brew install raxclaw',
    alt: 'cargo install raxclaw',
    icon: '🍎',
  },
  {
    os: 'Linux',
    sub: 'x86_64 · ARM64 · MUSL',
    cmd: 'cargo install raxclaw',
    alt: 'curl -fsSL https://raxclaw.sh | sh',
    icon: '🐧',
  },
  {
    os: 'Windows',
    sub: 'x86_64 MSVC',
    cmd: 'cargo install raxclaw',
    alt: 'winget install raxclaw',
    icon: '⊞',
  },
];

export function DownloadSection() {
  return (
    <section className="section" id="download">
      <div className="section-inner">
        <div style={{ textAlign: 'center', marginBottom: 60 }}>
          <div className="section-label">Download</div>
          <h2 className="section-title">Deploy the Runtime</h2>
          <p
            className="section-desc"
            style={{ margin: '0 auto', textAlign: 'center' }}
          >
            RAXCLAW runs locally from the terminal. The frontend is only a replay
            and verification interface — the runtime is the primary product.
          </p>
        </div>

        {/* Platform cards */}
        <div
          style={{
            display: 'grid',
            gridTemplateColumns: 'repeat(auto-fit, minmax(290px, 1fr))',
            gap: 20,
            marginBottom: 48,
          }}
        >
          {PLATFORMS.map((p) => (
            <div
              key={p.os}
              className="glass-card"
              style={{ padding: '30px 26px', position: 'relative', overflow: 'hidden' }}
            >
              {/* Top cyan accent */}
              <div
                aria-hidden
                style={{
                  position: 'absolute',
                  top: 0,
                  left: 0,
                  right: 0,
                  height: 2,
                  background:
                    'linear-gradient(90deg, transparent, var(--cyan), transparent)',
                  borderRadius: '14px 14px 0 0',
                }}
              />

              <div
                style={{
                  display: 'flex',
                  alignItems: 'center',
                  gap: 10,
                  marginBottom: 18,
                }}
              >
                <span style={{ fontSize: 22 }}>{p.icon}</span>
                <div>
                  <div style={{ fontWeight: 700, fontSize: 15 }}>{p.os}</div>
                  <div
                    style={{
                      fontSize: 11,
                      color: 'var(--text-dim)',
                      fontFamily: 'var(--font-mono)',
                      marginTop: 2,
                    }}
                  >
                    {p.sub}
                  </div>
                </div>
              </div>

              <div
                style={{
                  fontFamily: 'var(--font-mono)',
                  fontSize: 13,
                  color: 'var(--cyan)',
                  background: 'rgba(0,212,255,0.06)',
                  padding: '10px 14px',
                  borderRadius: 'var(--radius-sm)',
                  border: '1px solid var(--border)',
                  marginBottom: 8,
                  userSelect: 'all',
                }}
              >
                {p.cmd}
              </div>

              <div
                style={{
                  fontFamily: 'var(--font-mono)',
                  fontSize: 11,
                  color: 'var(--text-dim)',
                  padding: '0 2px',
                }}
              >
                or: {p.alt}
              </div>
            </div>
          ))}
        </div>

        {/* Usage snippet */}
        <div
          style={{
            background: '#010a16',
            border: '1px solid var(--border-strong)',
            borderRadius: 'var(--radius-lg)',
            padding: '24px 28px',
            marginBottom: 40,
            fontFamily: 'var(--font-mono)',
            fontSize: 13,
          }}
        >
          <div
            style={{
              fontSize: 10,
              color: 'var(--text-dim)',
              textTransform: 'uppercase',
              letterSpacing: '0.1em',
              marginBottom: 16,
            }}
          >
            Quick Start
          </div>
          {[
            { prompt: '$ ', code: 'raxclaw analyze --contract MyContract.sol', color: 'var(--cyan)' },
            { prompt: '$ ', code: 'raxclaw analyze --contract MyContract.sol --store 0g', color: 'var(--cyan)' },
            { prompt: '$ ', code: 'raxclaw replay --id 0x55EA9AC0EA590488', color: 'var(--cyan)' },
            { prompt: '$ ', code: 'raxclaw history', color: 'var(--cyan)' },
          ].map(({ prompt, code, color }, i) => (
            <div key={i} style={{ marginBottom: 8, color: 'var(--text-muted)' }}>
              <span style={{ color: 'var(--text-dim)' }}>{prompt}</span>
              <span style={{ color }}>{code}</span>
            </div>
          ))}
        </div>

        {/* CTA buttons */}
        <div
          style={{
            display: 'flex',
            gap: 12,
            justifyContent: 'center',
            flexWrap: 'wrap',
          }}
        >
          <a
            href="https://github.com/JFKongphop/raxc-0g-agent-framework"
            className="btn btn-primary"
            style={{ fontSize: 15, padding: '13px 30px' }}
            target="_blank"
            rel="noopener noreferrer"
          >
            View on GitHub ↗
          </a>
          <a
            href="https://github.com/JFKongphop/raxc-0g-agent-framework/blob/main/README.md"
            className="btn btn-secondary"
            style={{ fontSize: 15, padding: '13px 30px' }}
            target="_blank"
            rel="noopener noreferrer"
          >
            Documentation
          </a>
        </div>
      </div>
    </section>
  );
}
