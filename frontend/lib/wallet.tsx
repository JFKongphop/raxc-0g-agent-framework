'use client';

import { createContext, useContext, useState, useEffect, ReactNode } from 'react';
import { ethers } from 'ethers';

interface WalletContextType {
  address: string | null;
  signer: ethers.Signer | null;
  provider: ethers.BrowserProvider | null;
  connect: () => Promise<void>;
  disconnect: () => void;
  isConnecting: boolean;
}

const WalletContext = createContext<WalletContextType>({
  address: null,
  signer: null,
  provider: null,
  connect: async () => {},
  disconnect: () => {},
  isConnecting: false,
});

export function useWallet() {
  return useContext(WalletContext);
}

export function WalletProvider({ children }: { children: ReactNode }) {
  const [address, setAddress] = useState<string | null>(null);
  const [signer, setSigner] = useState<ethers.Signer | null>(null);
  const [provider, setProvider] = useState<ethers.BrowserProvider | null>(null);
  const [isConnecting, setIsConnecting] = useState(false);

  const connect = async () => {
    if (typeof window === 'undefined' || !window.ethereum) {
      alert('Please install MetaMask or another Web3 wallet!');
      return;
    }

    setIsConnecting(true);
    try {
      const browserProvider = new ethers.BrowserProvider(window.ethereum);
      await browserProvider.send('eth_requestAccounts', []);
      
      // 0G Galileo Testnet configuration
      const targetChainId = '0x40E2'; // 16602 in hex
      const currentChainId = await window.ethereum.request({ method: 'eth_chainId' });
      
      if (currentChainId !== targetChainId) {
        try {
          // Try to switch to 0G network
          await window.ethereum.request({
            method: 'wallet_switchEthereumChain',
            params: [{ chainId: targetChainId }],
          });
        } catch (switchError: any) {
          // This error code indicates that the chain has not been added to MetaMask
          if (switchError.code === 4902) {
            try {
              await window.ethereum.request({
                method: 'wallet_addEthereumChain',
                params: [
                  {
                    chainId: targetChainId,
                    chainName: '0G-Galileo-Testnet',
                    nativeCurrency: {
                      name: '0G',
                      symbol: '0G',
                      decimals: 18,
                    },
                    rpcUrls: ['https://evmrpc-testnet.0g.ai'],
                    blockExplorerUrls: ['https://explorer.0g.ai/testnet'],
                  },
                ],
              });
            } catch (addError: any) {
              console.error('Failed to add 0G network:', addError);
              // Don't block connection - user can switch manually
              console.warn('Please manually add 0G-Galileo-Testnet to your wallet');
              alert('Could not add 0G network automatically. Please add it manually:\n\n' +
                'Network: 0G-Galileo-Testnet\n' +
                'Chain ID: 16602\n' +
                'RPC: https://evmrpc-testnet.0g.ai\n' +
                'Symbol: 0G\n' +
                'Explorer: https://explorer.0g.ai/testnet');
            }
          } else if (switchError.code === 4001) {
            // User rejected the request
            console.log('User rejected network switch');
          } else {
            console.error('Failed to switch to 0G network:', switchError);
            console.warn('Please manually switch to 0G-Galileo-Testnet');
          }
        }
      }
      
      const signer = await browserProvider.getSigner();
      const address = await signer.getAddress();

      setProvider(browserProvider);
      setSigner(signer);
      setAddress(address);

      // Store connection state
      localStorage.setItem('walletConnected', 'true');
    } catch (error: any) {
      console.error('Failed to connect wallet:', error);
      alert('Failed to connect wallet: ' + error.message);
    } finally {
      setIsConnecting(false);
    }
  };

  const disconnect = () => {
    setAddress(null);
    setSigner(null);
    setProvider(null);
    localStorage.removeItem('walletConnected');
  };

  // Auto-connect on mount if previously connected
  useEffect(() => {
    const wasConnected = localStorage.getItem('walletConnected');
    if (wasConnected === 'true' && window.ethereum) {
      window.ethereum.request({ method: 'eth_accounts' }).then((accounts: string[]) => {
        if (accounts.length > 0) {
          connect();
        }
      });
    }
  }, []);

  // Listen for account changes
  useEffect(() => {
    if (!window.ethereum) return;

    const handleAccountsChanged = (accounts: string[]) => {
      if (accounts.length === 0) {
        disconnect();
      } else {
        connect();
      }
    };

    const handleChainChanged = () => {
      window.location.reload();
    };

    window.ethereum.on('accountsChanged', handleAccountsChanged);
    window.ethereum.on('chainChanged', handleChainChanged);

    return () => {
      window.ethereum?.removeListener('accountsChanged', handleAccountsChanged);
      window.ethereum?.removeListener('chainChanged', handleChainChanged);
    };
  }, []);

  return (
    <WalletContext.Provider value={{ address, signer, provider, connect, disconnect, isConnecting }}>
      {children}
    </WalletContext.Provider>
  );
}

// Extend Window interface for TypeScript
declare global {
  interface Window {
    ethereum?: any;
  }
}
