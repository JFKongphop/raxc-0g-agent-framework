'use client';

import { useState } from 'react';
import { ethers } from 'ethers';
import { useWallet } from '@/lib/wallet';
import { formatTokenCount, estimateCostUSDC } from '@/lib/tokenizer';

interface PaymentModalProps {
  isOpen: boolean;
  onClose: () => void;
  onSuccess: (paymentData: { paymentId: string; txHash: string }) => void;
  estimatedTokens: { prompt: number; completion: number };
}

const VAULT_ADDRESS = process.env.NEXT_PUBLIC_VAULT_ADDRESS || '';
const USDC_ADDRESS = process.env.NEXT_PUBLIC_USDC_ADDRESS || '';

// Vault ABI (minimal for payment functions)
const VAULT_ABI = [
  'function payForAnalysis(uint256 promptTokens, uint256 completionTokens) external returns (bytes32 paymentId)',
  'function estimateCost(uint256 promptTokens, uint256 completionTokens) external view returns (uint256)',
  'event PaymentReceived(bytes32 indexed paymentId, address indexed user, uint256 amount, uint256 estimatedPromptTokens, uint256 estimatedCompletionTokens)',
];

const USDC_ABI = [
  'function approve(address spender, uint256 amount) external returns (bool)',
  'function allowance(address owner, address spender) external view returns (uint256)',
  'function balanceOf(address account) external view returns (uint256)',
];

export function PaymentModal({ isOpen, onClose, onSuccess, estimatedTokens }: PaymentModalProps) {
  const { address, signer } = useWallet();
  const [loading, setLoading] = useState(false);
  const [step, setStep] = useState<'estimate' | 'approve' | 'pay' | 'confirm'>('estimate');
  const [cost, setCost] = useState<string>('0');
  const [txHash, setTxHash] = useState<string>('');

  if (!isOpen) return null;

  const formatUSDC = (amount: bigint) => {
    return (Number(amount) / 1_000_000).toFixed(2);
  };

  const handleEstimate = async () => {
    if (!signer) return;
    setLoading(true);
    try {
      const vault = new ethers.Contract(VAULT_ADDRESS, VAULT_ABI, signer);
      const estimatedCost = await vault.estimateCost(
        estimatedTokens.prompt,
        estimatedTokens.completion
      );
      setCost(formatUSDC(estimatedCost));
      setStep('approve');
    } catch (err: any) {
      alert('Failed to estimate cost: ' + err.message);
      console.error(err);
    } finally {
      setLoading(false);
    }
  };

  const handleApprove = async () => {
    if (!signer || !address) return;
    setLoading(true);
    try {
      const usdc = new ethers.Contract(USDC_ADDRESS, USDC_ABI, signer);
      const vault = new ethers.Contract(VAULT_ADDRESS, VAULT_ABI, signer);

      // Check current allowance
      const currentAllowance = await usdc.allowance(address, VAULT_ADDRESS);
      const requiredCost = await vault.estimateCost(
        estimatedTokens.prompt,
        estimatedTokens.completion
      );

      if (currentAllowance >= requiredCost) {
        setStep('pay');
        return;
      }

      // Approve USDC
      const approveTx = await usdc.approve(VAULT_ADDRESS, requiredCost);
      await approveTx.wait();
      setStep('pay');
    } catch (err: any) {
      alert('Failed to approve USDC: ' + err.message);
      console.error(err);
    } finally {
      setLoading(false);
    }
  };

  const handlePay = async () => {
    if (!signer) return;
    setLoading(true);
    try {
      const vault = new ethers.Contract(VAULT_ADDRESS, VAULT_ABI, signer);

      // Pay for analysis
      const payTx = await vault.payForAnalysis(
        estimatedTokens.prompt,
        estimatedTokens.completion
      );
      const receipt = await payTx.wait();
      setTxHash(receipt.hash);

      // Extract paymentId from PaymentReceived event
      const paymentEvent = receipt.logs.find((log: any) => {
        try {
          const parsed = vault.interface.parseLog(log);
          return parsed?.name === 'PaymentReceived';
        } catch {
          return false;
        }
      });

      if (!paymentEvent) {
        throw new Error('Payment event not found in transaction');
      }

      const parsed = vault.interface.parseLog(paymentEvent);
      const paymentId = parsed?.args?.paymentId;

      if (!paymentId) {
        throw new Error('Payment ID not found in event');
      }

      setStep('confirm');
      onSuccess({
        paymentId: paymentId,
        txHash: receipt.hash,
      });
    } catch (err: any) {
      alert('Failed to process payment: ' + err.message);
      console.error(err);
    } finally {
      setLoading(false);
    }
  };

  return (
    <div style={{
      position: 'fixed',
      top: 0,
      left: 0,
      right: 0,
      bottom: 0,
      background: 'rgba(0, 0, 0, 0.75)',
      display: 'flex',
      alignItems: 'center',
      justifyContent: 'center',
      zIndex: 1000,
    }}>
      <div style={{
        background: '#000',
        borderRadius: 'var(--radius-lg)',
        padding: 32,
        maxWidth: 500,
        width: '90%',
        border: '1px solid var(--border)',
      }}>
        <h2 style={{ fontSize: 24, fontWeight: 700, marginBottom: 16 }}>
          💳 Pay for Analysis
        </h2>

        <div style={{
          background: 'var(--bg-input)',
          padding: 16,
          borderRadius: 'var(--radius-sm)',
          marginBottom: 24,
        }}>
          <div style={{ fontSize: 13, color: 'var(--text-muted)', marginBottom: 8 }}>
            Token Estimation (GPT-4o)
          </div>
          <div style={{ fontSize: 14, marginBottom: 4, display: 'flex', justifyContent: 'space-between' }}>
            <span>📝 Prompt tokens:</span>
            <strong>{formatTokenCount(estimatedTokens.prompt)}</strong>
          </div>
          <div style={{ fontSize: 14, marginBottom: 8, display: 'flex', justifyContent: 'space-between' }}>
            <span>💬 Completion tokens:</span>
            <strong>{formatTokenCount(estimatedTokens.completion)}</strong>
          </div>
          <div style={{ borderTop: '1px solid var(--border)', marginTop: 8, paddingTop: 8 }}>
            <div style={{ fontSize: 12, color: 'var(--text-dim)', marginBottom: 4 }}>
              Estimated Cost Breakdown:
            </div>
            <div style={{ fontSize: 11, color: 'var(--text-muted)', marginBottom: 2 }}>
              • Prompt: ${((estimatedTokens.prompt / 1_000_000) * 2.5).toFixed(4)}
            </div>
            <div style={{ fontSize: 11, color: 'var(--text-muted)', marginBottom: 2 }}>
              • Completion: ${((estimatedTokens.completion / 1_000_000) * 10).toFixed(4)}
            </div>
            <div style={{ fontSize: 11, color: 'var(--text-muted)', marginBottom: 4 }}>
              • Platform Fee (10%): ${(parseFloat(estimateCostUSDC(estimatedTokens.prompt, estimatedTokens.completion)) * 0.1).toFixed(4)}
            </div>
          </div>
          {cost !== '0' && (
            <div style={{ fontSize: 18, fontWeight: 700, color: 'var(--accent)', marginTop: 12 }}>
              Total: {cost} USDC
            </div>
          )}
        </div>

        {step === 'estimate' && (
          <button
            className="btn btn-primary"
            onClick={handleEstimate}
            disabled={loading}
            style={{ width: '100%' }}
          >
            {loading ? (
              <>
                <div className="spinner" />
                Estimating...
              </>
            ) : (
              'Estimate Cost'
            )}
          </button>
        )}

        {step === 'approve' && (
          <button
            className="btn btn-primary"
            onClick={handleApprove}
            disabled={loading}
            style={{ width: '100%' }}
          >
            {loading ? (
              <>
                <div className="spinner" />
                Approving USDC...
              </>
            ) : (
              `Approve ${cost} USDC`
            )}
          </button>
        )}

        {step === 'pay' && (
          <button
            className="btn btn-primary"
            onClick={handlePay}
            disabled={loading}
            style={{ width: '100%' }}
          >
            {loading ? (
              <>
                <div className="spinner" />
                Processing Payment...
              </>
            ) : (
              'Pay & Analyze'
            )}
          </button>
        )}

        {step === 'confirm' && (
          <div>
            <div style={{
              background: 'rgba(0, 255, 0, 0.1)',
              border: '1px solid rgba(0, 255, 0, 0.3)',
              padding: 16,
              borderRadius: 'var(--radius-sm)',
              marginBottom: 16,
              textAlign: 'center',
            }}>
              <div style={{ fontSize: 32, marginBottom: 8 }}>✅</div>
              <div style={{ fontSize: 16, fontWeight: 600, color: '#00ff00' }}>
                Payment Confirmed
              </div>
            </div>
            {txHash && (
              <div style={{ fontSize: 12, color: 'var(--text-muted)', textAlign: 'center', marginBottom: 16 }}>
                Transaction: {txHash.slice(0, 10)}...{txHash.slice(-8)}
              </div>
            )}
          </div>
        )}

        <button
          className="btn btn-secondary"
          onClick={onClose}
          disabled={loading}
          style={{ width: '100%', marginTop: 12 }}
        >
          {step === 'confirm' ? 'Close' : 'Cancel'}
        </button>

        <div style={{
          marginTop: 20,
          fontSize: 12,
          color: 'var(--text-dim)',
          textAlign: 'center',
        }}>
          Powered by Initia Native Features (auto-signing)
        </div>
      </div>
    </div>
  );
}
