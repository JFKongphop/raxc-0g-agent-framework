'use client';

import { useEffect, useState } from 'react';
import { fetchChainStats, ChainStats } from '@/lib/contracts';

export function HeroSection() {
  const [chain, setChain] = useState<ChainStats | null>(null);

  useEffect(() => {
    fetchChainStats().then(setChain).catch(() => {});
  }, []);

  const audits  = chain?.online ? chain.auditsCompleted : 3;
  const erc7857 = chain?.online ? chain.erc7857Updates  : 0;

  return (
    <section
      className="section"
      style={{
        paddingTop: 160,
        paddingBottom: 130,
        textAlign: 'center',
        position: 'relative',
        overflow: 'hidden',
      }}
    >
      {/* Ambient glow behind hero text */}
      <div
        aria-hidden
        style={{
          position: 'absolute',
          top: '38%',
          left: '50%',
          transform: 'translate(-50%, -50%)',
          width: 900,
          height: 500,
          background:
            'radial-gradient(ellipse, rgba(0,212,255,0.07) 0%, rgba(0,85,255,0.05) 40%, transparent 70%)',
          pointerEvents: 'none',
          zIndex: 0,
        }}
      />

      <div className="section-inner" style={{ position: 'relative', zIndex: 1 }}>
        {/* Pill label */}
        <div style={{ display: 'inline-block', marginBottom: 28 }}>
          <span
            style={{
              fontFamily: 'var(--font-mono)',
              fontSize: 11,
              letterSpacing: '0.16em',
              color: 'var(--cyan)',
              textTransform: 'uppercase',
              border: '1px solid rgba(0,212,255,0.3)',
              borderRadius: 100,
              padding: '5px 18px',
              background: 'rgba(0,212,255,0.06)',
              display: 'inline-block',
            }}
          >
            Autonomous Security Cognition — On 0G Network
          </span>
        </div>

        {/* Logotype */}
        <h1
          style={{
            fontSize: 'clamp(64px, 10vw, 112px)',
            fontWeight: 900,
            lineHeight: 0.92,
            letterSpacing: '-0.045em',
            marginBottom: 32,
          }}
        >
          RAX
          <span style={{ color: 'var(--cyan)', textShadow: '0 0 40px rgba(0,212,255,0.5)' }}>
            CLAW
          </span>
        </h1>

        {/* Tagline */}
        <p
          style={{
            fontSize: 'clamp(17px, 2.2vw, 22px)',
            color: 'var(--text-muted)',
            maxWidth: 560,
            margin: '0 auto 14px',
            lineHeight: 1.55,
          }}
        >
          Persistent autonomous security cognition.
          <br />
          Replayable intelligence that evolves with every audit.
        </p>

        {/* Stack line */}
        <p
          style={{
            fontFamily: 'var(--font-mono)',
            fontSize: 12,
            color: 'white',
            letterSpacing: '0.06em',
            marginBottom: 48,
          }}
        >
          OpenClaw · RAXC · 0G Compute · 0G Storage · ERC-7857 · ERC-8183
        </p>

        {/* CTAs */}
        <div
          style={{
            display: 'flex',
            gap: 12,
            justifyContent: 'center',
            flexWrap: 'wrap',
          }}
        >
          <a
            href="#download"
            className="btn btn-primary"
            style={{ fontSize: 15, padding: '13px 30px' }}
          >
            Download Runtime ↓
          </a>
          <a
            href="#audits"
            className="btn btn-secondary"
            style={{ fontSize: 15, padding: '13px 30px' }}
          >
            View Audit Explorer
          </a>
          <a
            href="https://github.com/JFKongphop/raxc-0g-agent-framework"
            className="btn btn-ghost"
            style={{ fontSize: 15, padding: '13px 30px' }}
            target="_blank"
            rel="noopener noreferrer"
          >
            GitHub ↗
          </a>
        </div>

        {/* Bottom status indicator */}
        <div
          style={{
            marginTop: 64,
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            gap: 20,
            flexWrap: 'wrap',
          }}
        >
          {[
            { dot: '#00ff88', label: `${audits} audit${audits !== 1 ? 's' : ''} persisted` },
            { dot: 'var(--cyan)', label: '14 memories loaded' },
            { dot: '#ffd60a', label: erc7857 > 0 ? `ERC-7857 · ${erc7857} update${erc7857 !== 1 ? 's' : ''}` : 'ERC-7857 live' },
          ].map(({ dot, label }) => (
            <div
              key={label}
              style={{ display: 'flex', alignItems: 'center', gap: 8 }}
            >
              <div
                style={{
                  width: 6,
                  height: 6,
                  borderRadius: '50%',
                  background: dot,
                  boxShadow: `0 0 8px ${dot}`,
                }}
              />
              <span
                style={{
                  fontFamily: 'var(--font-mono)',
                  fontSize: 12,
                  color: 'var(--text-muted)',
                }}
              >
                {label}
              </span>
            </div>
          ))}
        </div>
      </div>
    </section>
  );
}
