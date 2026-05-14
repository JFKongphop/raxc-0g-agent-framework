'use client';

// Place your terminal recording at: frontend/public/terminal.mov
const VIDEO_SRC = '/terminal.mov';

export function TerminalShowcase() {
  return (
    <section className="section">
      <div className="section-inner">
        <div className="grid-2">
          {/* Left: Copy */}
          <div>
            <div className="section-label">Terminal Runtime</div>
            <h2 className="section-title">
              Autonomous
              <br />
              Execution
            </h2>
            <p className="section-desc">
              RAXCLAW runs entirely from the command line
            </p>
            <p className="section-desc">
              Every audit is deterministic and cryptographically verifiable
            </p>
            <p className="section-desc" style={{ marginBottom: 36 }}>
              The frontend only replays what the terminal already proved
            </p>
            <div style={{ display: 'flex', gap: 12, flexWrap: 'wrap' }}>
              <a
                href="#download"
                className="btn btn-primary"
                style={{ fontSize: 14 }}
              >
                Download Runtime ↓
              </a>
              <a
                href="https://github.com/JFKongphop/raxc-0g-agent-framework"
                className="btn btn-secondary"
                style={{ fontSize: 14 }}
                target="_blank"
                rel="noopener noreferrer"
              >
                View Source
              </a>
            </div>
          </div>

          {/* Right: Terminal window */}
          <div>
            <div
              style={{
                background: '#010a16',
                border: '1px solid var(--border-strong)',
                borderRadius: 12,
                overflow: 'hidden',
                boxShadow:
                  '0 0 50px rgba(0,212,255,0.10), 0 24px 60px rgba(0,0,0,0.5)',
              }}
            >
              {/* Chrome bar */}
              <div
                style={{
                  padding: '10px 16px',
                  background: 'rgba(0,212,255,0.04)',
                  borderBottom: '1px solid var(--border)',
                  display: 'flex',
                  alignItems: 'center',
                  gap: 8,
                }}
              >
                <div style={{ display: 'flex', gap: 6 }}>
                  <div style={{ width: 10, height: 10, borderRadius: '50%', background: '#ff4466' }} />
                  <div style={{ width: 10, height: 10, borderRadius: '50%', background: '#ffd60a' }} />
                  <div style={{ width: 10, height: 10, borderRadius: '50%', background: '#00ff88' }} />
                </div>
                <span
                  style={{
                    marginLeft: 8,
                    color: 'var(--text-dim)',
                    fontSize: 11,
                    fontFamily: 'var(--font-mono)',
                  }}
                >
                  raxclaw — zsh
                </span>
              </div>

              {/* Video */}
              <div style={{ position: 'relative', lineHeight: 0 }}>
                <video
                  src={VIDEO_SRC}
                  autoPlay
                  loop
                  muted
                  playsInline
                  style={{
                    width: '100%',
                    display: 'block',
                    background: '#010a16',
                  }}
                />
                {/* Scanline overlay */}
                <div
                  aria-hidden
                  style={{
                    position: 'absolute',
                    inset: 0,
                    background:
                      'repeating-linear-gradient(0deg, transparent, transparent 2px, rgba(0,0,0,0.06) 2px, rgba(0,0,0,0.06) 4px)',
                    pointerEvents: 'none',
                  }}
                />
              </div>
            </div>
          </div>
        </div>
      </div>
    </section>
  );
}

