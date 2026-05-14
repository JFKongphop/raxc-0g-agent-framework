import { Nav } from '@/components/Nav';
import { HeroSection } from '@/components/HeroSection';
import { ArchFlow } from '@/components/ArchFlow';
import { StatsSection } from '@/components/StatsSection';
import { AuditExplorer } from '@/components/AuditExplorer';
import { TerminalShowcase } from '@/components/TerminalShowcase';
import { MemorySection } from '@/components/MemorySection';
import { DownloadSection } from '@/components/DownloadSection';

export default function Home() {
  return (
    <>
      <Nav />
      <HeroSection />
      <div className="divider" />
      <ArchFlow />
      <div className="divider" />
      <StatsSection />
      <div className="divider" />
      <AuditExplorer />
      <div className="divider" />
      <TerminalShowcase />
      <div className="divider" />
      <MemorySection />
      <div className="divider" />
      <DownloadSection />
      <footer
        style={{
          textAlign: 'center',
          padding: '40px 24px',
          color: 'var(--text-dim)',
          fontSize: 13,
          fontFamily: 'var(--font-mono)',
          borderTop: '1px solid var(--border)',
          position: 'relative',
          zIndex: 1,
        }}
      >
        <span style={{ color: 'var(--cyan)' }}>RAXCLAW</span> — Autonomous Security Cognition Infrastructure
        {' · '}Built on{' '}
        <span style={{ color: 'var(--cyan)' }}>0G Network</span>
      </footer>
    </>
  );
}
