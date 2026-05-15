const PLATFORMS = [
  {
    os: 'Prerequisites',
    sub: 'Node.js 18+ · Rust · pnpm',
    cmd: 'git clone https://github.com/JFKongphop/raxc-0g-agent-framework',
    alt: 'cd raxc-0g-agent-framework',
    icon: '📦',
  },
  {
    os: 'Build',
    sub: 'JS CLI + Rust binary · one-time',
    cmd: 'pnpm install && pnpm build:all',
    alt: 'builds dist/raxclaw + prebuilt Rust binary',
    icon: '⚙️',
  },
  {
    os: 'Run',
    sub: 'no .env setup needed',
    cmd: './dist/raxclaw run',
    alt: './dist/raxclaw run --file MyContract.sol',
    icon: '🚀',
  },
];

export function DownloadSection() {
  return (
    <section className="section" id="download">
      <div className="section-inner">
        <div style={{ textAlign: 'center', marginBottom: 60 }}>
          <div className="section-label">Download</div>
          <h2 className="section-title">Runtime Installation</h2>
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
            { prompt: '$ ', code: 'git clone https://github.com/JFKongphop/raxc-0g-agent-framework', color: 'var(--text-muted)' },
            { prompt: '$ ', code: 'cd raxc-0g-agent-framework && pnpm install && pnpm build:all', color: 'var(--text-muted)' },
            { prompt: '$ ', code: './dist/raxclaw run', color: 'var(--cyan)' },
            { prompt: '$ ', code: './dist/raxclaw run --file MyContract.sol', color: 'var(--cyan)' },
            { prompt: '$ ', code: './dist/raxclaw list', color: 'var(--cyan)' },
            { prompt: '$ ', code: './dist/raxclaw show <report>', color: 'var(--cyan)' },
          ].map(({ prompt, code, color }, i) => (
            <div key={i} style={{ marginBottom: 8, color: 'var(--text-muted)' }}>
              <span style={{ color: 'var(--text-dim)' }}>{prompt}</span>
              <span style={{ color }}>{code}</span>
            </div>
          ))}
        </div>

        {/* Inline contract demo block */}
        <div
          style={{
            background: '#010a16',
            border: '1px solid var(--border-strong)',
            borderRadius: 'var(--radius-lg)',
            overflow: 'hidden',
            marginBottom: 40,
          }}
        >
          {/* Header bar */}
          <div
            style={{
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'space-between',
              padding: '10px 18px',
              borderBottom: '1px solid var(--border)',
              background: 'rgba(0,212,255,0.04)',
            }}
          >
            <div style={{ display: 'flex', alignItems: 'center', gap: 10 }}>
              {/* Traffic-light dots */}
              {['#ff5f57','#febc2e','#28c840'].map((c) => (
                <div key={c} style={{ width: 10, height: 10, borderRadius: '50%', background: c }} />
              ))}
              <span
                style={{
                  fontFamily: 'var(--font-mono)',
                  fontSize: 11,
                  color: 'var(--text-dim)',
                  marginLeft: 6,
                }}
              >
                DemoVault.sol — inline audit
              </span>
            </div>
            <span
              style={{
                fontFamily: 'var(--font-mono)',
                fontSize: 10,
                color: 'var(--cyan)',
                background: 'rgba(0,212,255,0.08)',
                padding: '2px 8px',
                borderRadius: 4,
                border: '1px solid rgba(0,212,255,0.2)',
              }}
            >
              ⚠ reentrancy · flash loan · access control
            </span>
          </div>

          {/* Command line */}
          <div
            style={{
              padding: '12px 18px',
              borderBottom: '1px solid rgba(255,255,255,0.04)',
              fontFamily: 'var(--font-mono)',
              fontSize: 12,
              color: 'var(--cyan)',
              background: 'rgba(0,212,255,0.03)',
            }}
          >
            <span style={{ color: 'var(--text-dim)', marginRight: 6 }}>$</span>
            ./dist/raxclaw run <span style={{ color: '#7dd3fc' }}>&quot;&lt;contract code&gt;&quot;</span>
          </div>

          {/* Contract source */}
          <div style={{ padding: '18px 22px', overflowX: 'auto' }}>
            <pre
              style={{
                margin: 0,
                fontFamily: 'var(--font-mono)',
                fontSize: 12,
                lineHeight: 1.7,
                color: '#94a3b8',
              }}
            >
{[
  { t: 'comment',  v: '// SPDX-License-Identifier: MIT' },
  { t: 'kw',       v: 'pragma solidity ', s: 'plain', rest: '^0.8.0;' },
  { t: 'blank' },
  { t: 'kw',       v: 'contract ', s: 'name', rest: 'DemoVault {' },
  { t: 'field',    v: '    address public ', rest: 'owner;' },
  { t: 'field',    v: '    mapping(address => uint256) public ', rest: 'balances;' },
  { t: 'field',    v: '    uint256 public ', rest: 'totalDeposits;' },
  { t: 'field',    v: '    uint256 public ', rest: 'flashLoanFee = 10;' },
  { t: 'blank' },
  { t: 'fn',       v: '    constructor', rest: '() { owner = msg.sender; }' },
  { t: 'blank' },
  { t: 'warn',     v: '    // ⚠ CEI violation — state updated AFTER external call' },
  { t: 'fn',       v: '    function ', s: 'fn', rest: 'withdraw(uint256 amount) external {' },
  { t: 'plain',    v: '        require(balances[msg.sender] >= amount, "insufficient balance");' },
  { t: 'plain',    v: '        (bool ok, ) = msg.sender.call{value: amount}("");' },
  { t: 'plain',    v: '        require(ok, "transfer failed");' },
  { t: 'plain',    v: '        balances[msg.sender] -= amount;   ', rest: '// ← runs AFTER call' },
  { t: 'plain',    v: '    }' },
  { t: 'blank' },
  { t: 'warn',     v: '    // ⚠ flash loan — spot balance as oracle reference' },
  { t: 'fn',       v: '    function ', s: 'fn', rest: 'flashLoan(uint256 amount) external {' },
  { t: 'plain',    v: '        uint256 balanceBefore = address(this).balance;' },
  { t: 'plain',    v: '        (bool ok, ) = msg.sender.call{value: amount}(...);' },
  { t: 'plain',    v: '    }' },
  { t: 'blank' },
  { t: 'warn',     v: '    // ⚠ no access control — anyone can call setFee' },
  { t: 'fn',       v: '    function ', s: 'fn', rest: 'setFee(uint256 newFee) external {' },
  { t: 'plain',    v: '        flashLoanFee = newFee;' },
  { t: 'plain',    v: '    }' },
  { t: 'plain',    v: '}' },
].map((line, i) => {
  if (line.t === 'blank') return <span key={i}>{'\n'}</span>;
  if (line.t === 'comment') return <span key={i} style={{ color: '#4a5568' }}>{line.v}{'\n'}</span>;
  if (line.t === 'warn')    return <span key={i} style={{ color: '#f59e0b' }}>{line.v}{'\n'}</span>;
  if (line.t === 'kw')      return <span key={i}><span style={{ color: '#7c3aed' }}>{line.v}</span><span style={{ color: 'var(--cyan)' }}>{line.s === 'name' ? '' : ''}</span>{line.rest}{'\n'}</span>;
  if (line.t === 'fn')      return <span key={i}><span style={{ color: '#7c3aed' }}>{line.v}</span><span style={{ color: '#38bdf8' }}>{line.rest}</span>{'\n'}</span>;
  if (line.t === 'field')   return <span key={i}><span style={{ color: '#475569' }}>{line.v}</span><span style={{ color: '#64748b' }}>{line.rest}</span>{'\n'}</span>;
  return <span key={i} style={{ color: '#64748b' }}>{line.v}{line.rest ?? ''}{'\n'}</span>;
})}
            </pre>
          </div>
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
