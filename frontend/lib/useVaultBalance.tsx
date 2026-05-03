'use client';

import { useState, useEffect } from 'react';
import { ethers } from 'ethers';
import { useWallet } from './wallet';

const VAULT_ADDRESS = process.env.NEXT_PUBLIC_VAULT_ADDRESS || '';

const VAULT_ABI = [
  'function balanceOf(address account) external view returns (uint256)',
  'function convertToAssets(uint256 shares) external view returns (uint256)',
  'function totalAssets() external view returns (uint256)',
];

export function useVaultBalance() {
  const { address, signer } = useWallet();
  const [vaultShares, setVaultShares] = useState<string>('0');
  const [usdcValue, setUsdcValue] = useState<string>('0');
  const [loading, setLoading] = useState(false);

  const formatUSDC = (amount: bigint) => {
    return (Number(amount) / 1_000_000).toFixed(2);
  };

  const fetchBalance = async () => {
    if (!signer || !address) {
      setVaultShares('0');
      setUsdcValue('0');
      return;
    }

    setLoading(true);
    try {
      const vault = new ethers.Contract(VAULT_ADDRESS, VAULT_ABI, signer);

      const shares = await vault.balanceOf(address);
      setVaultShares(formatUSDC(shares));

      // Convert shares to USDC value
      if (shares > 0) {
        const assets = await vault.convertToAssets(shares);
        setUsdcValue(formatUSDC(assets));
      } else {
        setUsdcValue('0');
      }
    } catch (err) {
      console.error('Failed to fetch vault balance:', err);
      setVaultShares('0');
      setUsdcValue('0');
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchBalance();
    
    // Refresh balance every 10 seconds
    const interval = setInterval(fetchBalance, 10000);
    return () => clearInterval(interval);
  }, [address, signer]);

  return {
    vaultShares,
    usdcValue,
    loading,
    refresh: fetchBalance,
  };
}
