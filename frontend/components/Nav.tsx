export function Nav() {
  return (
    <nav
      style={{
        position: 'fixed',
        top: 0,
        left: 0,
        right: 0,
        zIndex: 100,
        padding: '0 32px',
        height: 60,
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'space-between',
        background: 'rgba(2, 12, 27, 0.88)',
        borderBottom: '1px solid var(--border)',
        backdropFilter: 'blur(20px)',
        WebkitBackdropFilter: 'blur(20px)',
      }}
    >
      <div style={{ display: 'flex', alignItems: 'center', gap: 12 }}>
        <div
          style={{
            width: 8,
            height: 8,
            borderRadius: '50%',
            background: 'var(--cyan)',
            animation: 'pulseGlow 2.5s infinite',
            flexShrink: 0,
          }}
        />
        <span
          style={{
            fontFamily: 'var(--font-mono)',
            fontWeight: 700,
            fontSize: 15,
            letterSpacing: '0.06em',
          }}
        >
          RAXCLAW
        </span>
        <span className="badge badge-info" style={{ padding: '2px 8px', fontSize: 10 }}>
          v0.9.1
        </span>
      </div>

      <div style={{ display: 'flex', gap: 8, alignItems: 'center' }}>
        <a href="#audits" className="btn btn-ghost" style={{ fontSize: 13, padding: '7px 16px' }}>
          Audit Explorer
        </a>
        <a href="#download" className="btn btn-secondary" style={{ fontSize: 13, padding: '7px 16px' }}>
          Download
        </a>
        <a
          href="https://github.com/JFKongphop/raxc-0g-agent-framework"
          className="btn btn-primary"
          style={{ fontSize: 13, padding: '7px 16px' }}
          target="_blank"
          rel="noopener noreferrer"
        >
          GitHub ↗
        </a>
      </div>
    </nav>
  );
}
