'use client';

import { useState } from 'react';
import { ethers } from 'ethers';
import { useWallet } from '@/lib/wallet';

interface DepositModalProps {
  isOpen: boolean;
  onClose: () => void;
  onSuccess: () => void;
}

const VAULT_ADDRESS = process.env.NEXT_PUBLIC_VAULT_ADDRESS || '';
const USDC_ADDRESS = process.env.NEXT_PUBLIC_USDC_ADDRESS || '';

// Vault ABI for deposit
const VAULT_ABI = [
  'function deposit(uint256 assets, address receiver) external returns (uint256 shares)',
  'function balanceOf(address account) external view returns (uint256)',
  'function totalAssets() external view returns (uint256)',
  'function convertToShares(uint256 assets) external view returns (uint256)',
];

const USDC_ABI = [
  'function approve(address spender, uint256 amount) external returns (bool)',
  'function allowance(address owner, address spender) external view returns (uint256)',
  'function balanceOf(address account) external view returns (uint256)',
];

const MIN_DEPOSIT = 1_000_000; // 1 USDC minimum

export function DepositModal({ isOpen, onClose, onSuccess }: DepositModalProps) {
  const { address, signer } = useWallet();
  const [loading, setLoading] = useState(false);
  const [step, setStep] = useState<'input' | 'approve' | 'deposit' | 'success'>('input');
  const [amount, setAmount] = useState('');
  const [usdcBalance, setUsdcBalance] = useState<string>('0');
  const [vaultShares, setVaultShares] = useState<string>('0');

  if (!isOpen) return null;

  const formatUSDC = (amount: bigint) => {
    return (Number(amount) / 1_000_000).toFixed(2);
  };

  const loadBalances = async () => {
    if (!signer || !address) return;
    
    try {
      const usdc = new ethers.Contract(USDC_ADDRESS, USDC_ABI, signer);
      const vault = new ethers.Contract(VAULT_ADDRESS, VAULT_ABI, signer);

      const balance = await usdc.balanceOf(address);
      const shares = await vault.balanceOf(address);

      setUsdcBalance(formatUSDC(balance));
      setVaultShares(formatUSDC(shares));
    } catch (err) {
      console.error('Failed to load balances:', err);
    }
  };

  const handleApprove = async () => {
    if (!signer || !address) return;
    const amountWei = BigInt(Math.floor(parseFloat(amount) * 1_000_000));

    if (amountWei < MIN_DEPOSIT) {
      alert('Minimum deposit is 1 USDC');
      return;
    }

    setLoading(true);
    try {
      const usdc = new ethers.Contract(USDC_ADDRESS, USDC_ABI, signer);

      // Check current allowance
      const currentAllowance = await usdc.allowance(address, VAULT_ADDRESS);

      if (currentAllowance >= amountWei) {
        setStep('deposit');
        return;
      }

      // Approve USDC
      const approveTx = await usdc.approve(VAULT_ADDRESS, amountWei);
      await approveTx.wait();
      setStep('deposit');
    } catch (err: any) {
      alert('Failed to approve USDC: ' + err.message);
      console.error(err);
    } finally {
      setLoading(false);
    }
  };

  const handleDeposit = async () => {
    if (!signer || !address) return;
    const amountWei = BigInt(Math.floor(parseFloat(amount) * 1_000_000));

    setLoading(true);
    try {
      const vault = new ethers.Contract(VAULT_ADDRESS, VAULT_ABI, signer);

      // Deposit to vault
      const depositTx = await vault.deposit(amountWei, address);
      const receipt = await depositTx.wait();

      setStep('success');
      await loadBalances();
      setTimeout(() => {
        onSuccess();
        onClose();
      }, 2000);
    } catch (err: any) {
      alert('Failed to deposit: ' + err.message);
      console.error(err);
    } finally {
      setLoading(false);
    }
  };

  // Load balances when modal opens
  useState(() => {
    if (isOpen) {
      loadBalances();
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
          💳 Deposit USDC Credits
        </h2>

        <div style={{
          background: 'var(--bg-input)',
          padding: 16,
          borderRadius: 'var(--radius-sm)',
          marginBottom: 24,
        }}>
          <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: 8, fontSize: 13 }}>
            <span style={{ color: 'var(--text-muted)' }}>USDC Balance:</span>
            <strong>{usdcBalance} USDC</strong>
          </div>
          <div style={{ display: 'flex', justifyContent: 'space-between', fontSize: 13 }}>
            <span style={{ color: 'var(--text-muted)' }}>Vault Credits:</span>
            <strong style={{ color: 'var(--accent)' }}>{vaultShares} vRAXC</strong>
          </div>
        </div>

        {step === 'input' && (
          <>
            <div style={{ marginBottom: 16 }}>
              <label style={{ display: 'block', fontSize: 13, color: 'var(--text-muted)', marginBottom: 8 }}>
                Amount to Deposit (minimum 1 USDC)
              </label>
              <input
                type="number"
                className="input"
                value={amount}
                onChange={(e) => setAmount(e.target.value)}
                placeholder="10.00"
                min="1"
                step="0.01"
                style={{ fontFamily: 'var(--font-sans)', fontSize: 18 }}
              />
            </div>

            <div style={{
              background: 'rgba(0, 150, 255, 0.1)',
              border: '1px solid rgba(0, 150, 255, 0.3)',
              padding: 12,
              borderRadius: 'var(--radius-sm)',
              marginBottom: 16,
              fontSize: 12,
              color: 'var(--text-muted)',
            }}>
              ℹ️ Credits can be used for multiple analyses. Unused credits can be withdrawn anytime.
            </div>

            <button
              className="btn btn-primary"
              onClick={handleApprove}
              disabled={loading || !amount || parseFloat(amount) < 1}
              style={{ width: '100%' }}
            >
              {loading ? (
                <>
                  <div className="spinner" />
                  Processing...
                </>
              ) : (
                'Continue to Approve'
              )}
            </button>
          </>
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
              `Approve ${amount} USDC`
            )}
          </button>
        )}

        {step === 'deposit' && (
          <button
            className="btn btn-primary"
            onClick={handleDeposit}
            disabled={loading}
            style={{ width: '100%' }}
          >
            {loading ? (
              <>
                <div className="spinner" />
                Depositing...
              </>
            ) : (
              `Deposit ${amount} USDC`
            )}
          </button>
        )}

        {step === 'success' && (
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
                Deposit Successful!
              </div>
              <div style={{ fontSize: 13, color: 'var(--text-muted)', marginTop: 8 }}>
                {amount} USDC deposited to your vault
              </div>
            </div>
          </div>
        )}

        <button
          className="btn btn-secondary"
          onClick={onClose}
          disabled={loading}
          style={{ width: '100%', marginTop: 12 }}
        >
          {step === 'success' ? 'Close' : 'Cancel'}
        </button>

        <div style={{
          marginTop: 20,
          fontSize: 11,
          color: 'var(--text-dim)',
          textAlign: 'center',
        }}>
          ERC4626 Vault on Initia
        </div>
      </div>
    </div>
  );
}
