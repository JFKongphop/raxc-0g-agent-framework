'use client';

import { useRef, useState } from 'react';

const STEPS: Array<{ id: string; label: string; sub: string; desc: string; color: string }> = [
  {
    id: 'openclaw',
    label: 'OpenClaw',
    sub: 'Orchestration Layer',
    desc: 'Orchestrates exploit pattern detection pipeline across all agent tools',
    color: '#0055ff',
  },
  {
    id: 'raxc',
    label: 'RAXC',
    sub: 'Autonomous Agent',
    desc: 'Multi-tool agentic execution with deterministic consensus',
    color: '#0099ff',
  },
  {
    id: 'compute',
    label: '0G Compute',
    sub: 'Deterministic Exec',
    desc: 'Sovereign off-chain execution with verifiable output',
    color: '#00c4ff',
  },
  {
    id: 'storage',
    label: '0G Storage',
    sub: 'Exploit & Audit Store',
    desc: 'Stores exploit patterns, audit reports, and cognition replay traces on-chain',
    color: '#00d4ff',
  },
  {
    id: 'erc',
    label: 'ERC-7857',
    sub: 'Identity Update',
    desc: 'On-chain agent identity evolves with each cognition cycle',
    color: '#00ff88',
  },
  {
    id: 'erc8183',
    label: 'ERC-8183',
    sub: 'Audit Task Proof',
    desc: 'Creates and finalizes audit task on-chain with cryptographic proof',
    color: '#a0ff60',
  },
];

export function ArchFlow() {
  const scrollRef = useRef<HTMLDivElement>(null);
  const [dragging, setDragging] = useState(false);
  const startX = useRef(0);
  const scrollLeft = useRef(0);

  const onMouseDown = (e: React.MouseEvent) => {
    setDragging(true);
    startX.current = e.pageX - (scrollRef.current?.offsetLeft ?? 0);
    scrollLeft.current = scrollRef.current?.scrollLeft ?? 0;
  };

  const onMouseMove = (e: React.MouseEvent) => {
    if (!dragging || !scrollRef.current) return;
    e.preventDefault();
    const x = e.pageX - scrollRef.current.offsetLeft;
    scrollRef.current.scrollLeft = scrollLeft.current - (x - startX.current);
  };

  const stopDrag = () => setDragging(false);

  return (
    <section className="section">
      <div className="section-inner">
        <div style={{ marginBottom: 52 }}>
          <div className="section-label">Architecture</div>
          <h2 className="section-title">Audit Intelligence Pipeline</h2>
          <p className="section-desc">
            Every audit flows through a deterministic pipeline 
            <br />
            from exploit pattern matching to permanent cognition storage on 0G.
          </p>
        </div>

        {/* Pipeline */}
        <div
          ref={scrollRef}
          onMouseDown={onMouseDown}
          onMouseMove={onMouseMove}
          onMouseUp={stopDrag}
          onMouseLeave={stopDrag}
          style={{
            display: 'flex',
            alignItems: 'stretch',
            gap: 0,
            overflowX: 'auto',
            paddingBottom: 4,
            cursor: dragging ? 'grabbing' : 'grab',
            userSelect: 'none',
            // hide scrollbar
            scrollbarWidth: 'none',
            msOverflowStyle: 'none',
          } as React.CSSProperties}
        >
          {STEPS.map((step, i) => (
            <div
              key={step.id}
              style={{ display: 'flex', alignItems: 'center', flexShrink: 0 }}
            >
              <div
                className="glass-card"
                style={{
                  padding: '24px 22px',
                  textAlign: 'center',
                  minWidth: 178,
                  position: 'relative',
                  borderColor: `${step.color}20`,
                }}
              >
                {/* Top accent line */}
                <div
                  style={{
                    position: 'absolute',
                    top: 0,
                    left: 0,
                    right: 0,
                    height: 2,
                    background: `linear-gradient(90deg, transparent, ${step.color}, transparent)`,
                    borderRadius: '14px 14px 0 0',
                  }}
                />

                <div
                  style={{
                    fontFamily: 'var(--font-mono)',
                    fontWeight: 700,
                    fontSize: 15,
                    marginBottom: 5,
                    color: step.color,
                    letterSpacing: '0.03em',
                  }}
                >
                  {step.label}
                </div>
                <div
                  style={{
                    fontSize: 11,
                    fontWeight: 600,
                    color: 'var(--text-muted)',
                    marginBottom: 8,
                    textTransform: 'uppercase',
                    letterSpacing: '0.06em',
                  }}
                >
                  {step.sub}
                </div>
                <div
                  style={{
                    fontSize: 11,
                    color: 'var(--text-dim)',
                    lineHeight: 1.5,
                  }}
                >
                  {step.desc}
                </div>

              </div>

              {i < STEPS.length - 1 && (
                <div
                  style={{
                    display: 'flex',
                    alignItems: 'center',
                    padding: '0 2px',
                    color: 'var(--border-strong)',
                    flexShrink: 0,
                  }}
                >
                  <div
                    style={{
                      width: 20,
                      height: 1,
                      background:
                        'linear-gradient(90deg, var(--border-strong), rgba(0,212,255,0.5))',
                    }}
                  />
                  <span
                    style={{
                      fontSize: 14,
                      margin: '0 2px',
                      color: 'var(--cyan)',
                    }}
                  >
                    →
                  </span>
                  <div
                    style={{
                      width: 20,
                      height: 1,
                      background:
                        'linear-gradient(90deg, rgba(0,212,255,0.5), var(--border-strong))',
                    }}
                  />
                </div>
              )}
            </div>
          ))}
        </div>
      </div>
    </section>
  );
}
