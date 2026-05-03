'use client';

import { useState } from 'react';
import { useWallet } from '@/lib/wallet';
import { useVaultBalance } from '@/lib/useVaultBalance';
import { AnalyzeSection } from '@/components/AnalyzeSection';
import { ResultSection } from '@/components/ResultSection';
import { PaymentModal } from '@/components/PaymentModal';
import { DepositModal } from '@/components/DepositModal';
import { MintModal } from '@/components/MintModal';
import { estimateAnalysisTokens } from '@/lib/tokenizer';

export default function Home() {
  const [result, setResult] = useState<any>(null);
  const [loading, setLoading] = useState(false);
  const [showPayment, setShowPayment] = useState(false);
  const [showDeposit, setShowDeposit] = useState(false);
  const [showMint, setShowMint] = useState(false);
  const [pendingContract, setPendingContract] = useState<{ contract: string; name: string } | null>(null);
  const [activeTab, setActiveTab] = useState<'input' | 'results'>('input');
  const { address, connect } = useWallet();
  const { vaultShares, usdcValue, refresh: refreshBalance } = useVaultBalance();

  const handleAnalyze = async (contract: string, name: string) => {
    if (!address) {
      alert('Please connect your wallet first!');
      await connect();
      return;
    }

    // Store contract for later and show payment modal
    setPendingContract({ contract, name });
    setShowPayment(true);
  };

  const handlePaymentComplete = async (paymentData: { paymentId: string; txHash: string }) => {
    if (!pendingContract || !address) return;

    setShowPayment(false);
    setLoading(true);
    setResult(null);
    
    // Switch to results tab to show loading state
    setActiveTab('results');

    try {
      const response = await fetch(`${process.env.NEXT_PUBLIC_API_URL}/analyze`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          contract: pendingContract.contract,
          name: pendingContract.name,
          payment_id: paymentData.paymentId,
          tx_hash: paymentData.txHash,
          user: address,
        }),
      });

      if (!response.ok) {
        const error = await response.json();
        throw new Error(error.error || 'Analysis failed');
      }

      const data = await response.json();
      setResult(data);
      setPendingContract(null);
      
      // Keep results tab active on success (already set)
    } catch (err: any) {
      setResult({ error: err.message });
      // Keep results tab active to show error
    } finally {
      setLoading(false);
    }
  };

  return (
    <div style={{ paddingTop: 60 }}>
      <div style={{ maxWidth: 1200, margin: '0 auto', padding: '0 24px' }}>
        <div style={{ textAlign: 'center', marginBottom: 48 }}>
          <h1 style={{ fontSize: 48, fontWeight: 700, marginBottom: 12, letterSpacing: '-0.02em' }}>
            RA<span style={{ color: 'var(--purple)' }}>X</span>C Security Agent
          </h1>
          <p style={{ fontSize: 18, color: 'var(--text-muted)', maxWidth: 600, margin: '0 auto' }}>
            Agentic vulnerability detection using real exploit patterns {process.env.NEXT_PUBLIC_API_URL}
          </p>
        </div>
        
        <div style={{ marginTop: 24, display: 'flex', gap: 12, justifyContent: 'center', alignItems: 'center', flexWrap: 'wrap' }}>
          {!address ? (
            <button
              className="btn btn-primary"
              onClick={connect}
              disabled={loading}
              style={{ fontSize: 16, padding: '12px 24px' }}
            >
              🔗 Connect Wallet
            </button>
          ) : (
            <>
              <div style={{
                padding: '8px 16px',
                background: 'var(--bg-input)',
                borderRadius: 'var(--radius-sm)',
                fontSize: 14,
                color: 'var(--accent)',
                border: '1px solid var(--border)',
              }}>
                🔗 {address.slice(0, 6)}...{address.slice(-4)}
              </div>
              
              {/* Credit Balance Display */}
              {parseFloat(usdcValue) > 0 && (
                <div style={{
                  padding: '8px 16px',
                  background: 'linear-gradient(135deg, rgba(0, 200, 100, 0.1), rgba(0, 150, 255, 0.1))',
                  borderRadius: 'var(--radius-sm)',
                  fontSize: 14,
                  border: '1px solid rgba(0, 200, 100, 0.3)',
                  display: 'flex',
                  alignItems: 'center',
                  gap: 8,
                }}>
                  <span style={{ opacity: 0.8 }}>💰 Credits:</span>
                  <strong style={{ color: 'var(--accent)' }}>{usdcValue} USDC</strong>
                  <span style={{ opacity: 0.6, fontSize: 12 }}>({vaultShares} vRAXC)</span>
                </div>
              )}
              
              <button
                className="btn btn-secondary"
                onClick={() => setShowDeposit(true)}
                style={{ fontSize: 14, padding: '8px 16px' }}
              >
                💳 Deposit Credits
              </button>
              
              <button
                className="btn btn-secondary"
                onClick={() => setShowMint(true)}
                style={{ fontSize: 14, padding: '8px 16px', borderColor: 'rgba(255, 200, 0, 0.5)' }}
              >
                🪙 Mint USDC
              </button>
            </>
          )}
        </div>
      </div>

      {/* Tab Navigation - Full Width */}
      {(result || loading) && (
        <div 
          className="slide-down"
          style={{
            position: 'sticky',
            top: 0,
            left: 0,
            right: 0,
            width: '100vw',
            marginLeft: 'calc(-50vw + 50%)',
            borderBottom: '1px solid var(--border)',
            marginBottom: 0,
            background: 'rgba(17, 17, 17, 0.8)',
            zIndex: 50,
            backdropFilter: 'blur(12px)',
            WebkitBackdropFilter: 'blur(12px)',
          }}>
          <div style={{
            maxWidth: 1200,
            margin: '0 auto',
            padding: '0 24px',
            display: 'flex',
            gap: 8,
          }}>
            <button
              onClick={() => setActiveTab('input')}
              style={{
                flex: 1,
                background: activeTab === 'input' ? 'var(--bg-surface)' : 'transparent',
                border: 'none',
                borderBottom: activeTab === 'input' ? '2px solid var(--accent)' : '2px solid transparent',
                color: activeTab === 'input' ? 'var(--text)' : 'var(--text-muted)',
                padding: '12px 24px',
                fontSize: 14,
                fontWeight: 600,
                cursor: 'pointer',
                transition: 'all 0.2s',
                fontFamily: 'var(--font-sans)',
                display: 'flex',
                justifyContent: 'center',
                alignItems: 'center',
              }}
            >
              📝 Contract Input
            </button>
            <button
              onClick={() => setActiveTab('results')}
              style={{
                flex: 1,
                background: activeTab === 'results' ? 'var(--bg-surface)' : 'transparent',
                border: 'none',
                borderBottom: activeTab === 'results' ? '2px solid var(--accent)' : '2px solid transparent',
                color: activeTab === 'results' ? 'var(--text)' : 'var(--text-muted)',
                padding: '12px 24px',
                fontSize: 14,
                fontWeight: 600,
                cursor: 'pointer',
                transition: 'all 0.2s',
                fontFamily: 'var(--font-sans)',
                display: 'flex',
                justifyContent: 'center',
                alignItems: 'center',
                gap: 8,
              }}
            >
              📊 Analysis Results
              {loading ? (
                <div className="spinner" style={{ width: 14, height: 14 }} />
              ) : result && !result.error && (
                <span style={{
                  background: result.risk_level?.toLowerCase().includes('high') ? 'var(--red)' : 
                             result.risk_level?.toLowerCase().includes('medium') ? 'var(--yellow)' : 
                             'var(--green)',
                  color: result.risk_level?.toLowerCase().includes('medium') ? '#000' : '#fff',
                  padding: '2px 8px',
                  borderRadius: '100px',
                  fontSize: 11,
                  fontWeight: 700,
                }}>
                  {result.risk_level || 'Done'}
                </span>
              )}
            </button>
          </div>
        </div>
      )}

      {/* Content Area */}
      <div style={{ maxWidth: 1200, margin: '0 auto', padding: '48px 24px 80px' }}>
        <div style={{ maxWidth: (result || loading) && activeTab === 'results' ? '1400px' : '900px', margin: '0 auto' }} className="fade-in" key={activeTab}>
        {activeTab === 'input' ? (
          <AnalyzeSection onAnalyze={handleAnalyze} loading={loading} hasResults={!!(result || loading)} />
        ) : loading && !result ? (
          <div className="card" style={{ textAlign: 'center', padding: 60 }}>
            <div className="spinner" style={{ margin: '0 auto 20px', width: 40, height: 40 }} />
            <div style={{ fontSize: 16, fontWeight: 600, marginBottom: 8 }}>Analyzing Contract...</div>
            <div style={{ fontSize: 14, color: 'var(--text-muted)' }}>
              This may take 30-60 seconds. Searching exploit database and running qwen/qwen-2.5-7b-instruct (7B parameter instruction-tuned LLM) analysis.
            </div>
          </div>
        ) : result ? (
          <ResultSection result={result} />
        ) : null}
        </div>
      </div>

      {showPayment && pendingContract && (
        <PaymentModal
          isOpen={showPayment}
          onClose={() => setShowPayment(false)}
          onSuccess={handlePaymentComplete}
          estimatedTokens={estimateAnalysisTokens(pendingContract.contract)}
        />
      )}

      {showDeposit && (
        <DepositModal
          isOpen={showDeposit}
          onClose={() => setShowDeposit(false)}
          onSuccess={() => {
            setShowDeposit(false);
            refreshBalance(); // Refresh balance after deposit
          }}
        />
      )}

      {showMint && (
        <MintModal
          isOpen={showMint}
          onClose={() => setShowMint(false)}
          onSuccess={() => {
            setShowMint(false);
            refreshBalance(); // Refresh balance after minting
          }}
        />
      )}
    </div>
  );
}
