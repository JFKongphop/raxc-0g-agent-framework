'use client';

import { useState } from 'react';
import { ethers } from 'ethers';
import { useWallet } from '@/lib/wallet';

interface MintModalProps {
  isOpen: boolean;
  onClose: () => void;
  onSuccess: () => void;
}

const USDC_ADDRESS = process.env.NEXT_PUBLIC_USDC_ADDRESS || '';

// MockUSDC ABI for minting
const USDC_ABI = [
  'function mint(address to, uint256 amount) external',
  'function balanceOf(address account) external view returns (uint256)',
];

export function MintModal({ isOpen, onClose, onSuccess }: MintModalProps) {
  const { address, signer } = useWallet();
  const [loading, setLoading] = useState(false);
  const [amount, setAmount] = useState('1000');
  const [usdcBalance, setUsdcBalance] = useState<string>('0');

  if (!isOpen) return null;

  const formatUSDC = (amount: bigint) => {
    return (Number(amount) / 1_000_000).toFixed(2);
  };

  const loadBalance = async () => {
    if (!signer || !address) return;
    
    try {
      const usdc = new ethers.Contract(USDC_ADDRESS, USDC_ABI, signer);
      const balance = await usdc.balanceOf(address);
      setUsdcBalance(formatUSDC(balance));
    } catch (err) {
      console.error('Failed to load balance:', err);
    }
  };

  const handleMint = async () => {
    if (!signer || !address) {
      alert('Please connect your wallet first');
      return;
    }

    if (!USDC_ADDRESS || USDC_ADDRESS === '') {
      alert('USDC contract address not configured. Please set NEXT_PUBLIC_USDC_ADDRESS in environment variables.');
      console.error('NEXT_PUBLIC_USDC_ADDRESS:', process.env.NEXT_PUBLIC_USDC_ADDRESS);
      return;
    }

    const amountWei = BigInt(Math.floor(parseFloat(amount) * 1_000_000));

    if (amountWei <= 0) {
      alert('Amount must be greater than 0');
      return;
    }

    setLoading(true);
    try {
      const usdc = new ethers.Contract(USDC_ADDRESS, USDC_ABI, signer);

      // Mint USDC tokens
      const mintTx = await usdc.mint(address, amountWei);
      await mintTx.wait();

      await loadBalance();
      alert(`✅ Successfully minted ${amount} USDC!`);
      onSuccess();
      onClose();
    } catch (err: any) {
      alert('Failed to mint USDC: ' + err.message);
      console.error(err);
    } finally {
      setLoading(false);
    }
  };

  // Load balance when modal opens
  useState(() => {
    if (isOpen) {
      loadBalance();
    }
  });

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
          🪙 Mint Test USDC
        </h2>

        <div style={{
          background: 'var(--bg-input)',
          padding: 16,
          borderRadius: 'var(--radius-sm)',
          marginBottom: 24,
        }}>
          <div style={{ display: 'flex', justifyContent: 'space-between', fontSize: 13 }}>
            <span style={{ color: 'var(--text-muted)' }}>Current Balance:</span>
            <strong>{usdcBalance} USDC</strong>
          </div>
        </div>

        <div style={{ marginBottom: 16 }}>
          <label style={{ display: 'block', fontSize: 13, color: 'var(--text-muted)', marginBottom: 8 }}>
            Amount to Mint
          </label>
          <input
            type="number"
            className="input"
            value={amount}
            onChange={(e) => setAmount(e.target.value)}
            placeholder="1000.00"
            min="1"
            step="1"
            style={{ fontFamily: 'var(--font-sans)', fontSize: 18 }}
          />
        </div>

        <div style={{
          background: 'rgba(255, 200, 0, 0.1)',
          border: '1px solid rgba(255, 200, 0, 0.3)',
          padding: 12,
          borderRadius: 'var(--radius-sm)',
          marginBottom: 16,
          fontSize: 12,
          color: 'var(--text-muted)',
        }}>
          ⚠️ This is a test function for development only. Works with MockUSDC on Initia testnet.
        </div>

        <div style={{ display: 'flex', gap: 12 }}>
          <button
            onClick={onClose}
            disabled={loading}
            style={{
              flex: 1,
              padding: '12px 24px',
              fontSize: 14,
              fontWeight: 600,
              border: '1px solid var(--border)',
              borderRadius: 'var(--radius-sm)',
              background: 'transparent',
              color: 'var(--text)',
              cursor: loading ? 'not-allowed' : 'pointer',
              opacity: loading ? 0.5 : 1,
            }}
          >
            Cancel
          </button>

          <button
            onClick={handleMint}
            disabled={loading || !amount || parseFloat(amount) <= 0}
            style={{
              flex: 1,
              padding: '12px 24px',
              fontSize: 14,
              fontWeight: 600,
              border: 'none',
              borderRadius: 'var(--radius-sm)',
              background: loading ? 'var(--text-muted)' : 'var(--accent)',
              color: '#fff',
              cursor: loading || !amount ? 'not-allowed' : 'pointer',
              opacity: loading || !amount ? 0.5 : 1,
            }}
          >
            {loading ? 'Minting...' : `Mint ${amount} USDC`}
          </button>
        </div>
      </div>
    </div>
  );
}
