'use client';

import { useState, useMemo } from 'react';
import { estimateAnalysisTokens, formatTokenCount, estimateCostUSDC } from '@/lib/tokenizer';

interface AnalyzeSectionProps {
  onAnalyze: (contract: string, name: string) => Promise<void>;
  loading: boolean;
  hasResults?: boolean;
}

export function AnalyzeSection({ onAnalyze, loading, hasResults }: AnalyzeSectionProps) {
  const [contract, setContract] = useState('');
  const [name, setName] = useState('');

  // Calculate tokens in real-time as user types
  const tokenEstimate = useMemo(() => {
    if (!contract.trim()) return null;
    return estimateAnalysisTokens(contract);
  }, [contract]);

  // Calculate estimated cost
  const estimatedCost = useMemo(() => {
    if (!tokenEstimate) return null;
    return estimateCostUSDC(tokenEstimate.prompt, tokenEstimate.completion);
  }, [tokenEstimate]);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!contract.trim()) return;
    await onAnalyze(contract, name || 'contract');
  };

  const sampleContract = `// https://tornado.cash
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

import {MerkleTreeWithHistory} from "./MerkleTreeWithHistory.sol";
import {ReentrancyGuard} from "./ReentrancyGuard.sol";
import {MockToken} from "./MockToken.sol";
import {Groth16Verifier} from "./Groth16Verifier.sol";

interface IVerifier {
  function verifyProof(
    uint256[2] calldata _pA,
    uint256[2][2] calldata _pB,
    uint256[2] calldata _pC,
    uint256[26] calldata _pubSignals
  ) external view returns (bool);
}

contract Zringotts is MerkleTreeWithHistory, ReentrancyGuard {
  IVerifier public immutable verifier;

  MockToken public weth;
  MockToken public usdc;

  struct State {
    int256 weth_deposit_amount;
    int256 weth_borrow_amount;
    int256 usdc_deposit_amount;
    int256 usdc_borrow_amount;
  }

  State public state;

  struct Liquidated {
    uint256 liq_price;
    uint256 timestamp;
  }

  uint256 public constant LIQUIDATED_ARRAY_NUMBER = 10;
  Liquidated[] public liquidated_array;

  mapping(bytes32 => bool) public nullifierHashes;
  mapping(bytes32 => bool) public commitments;

  event Deposit(bytes32 nullifierHash, uint256 timestamp);
  event Borrow(address to, bytes32 nullifierHash, uint256 timestamp);
  event Repay(bytes32 nullifierHash, uint256 timestamp);
  event Withdraw(address to, bytes32 nullifierHash, uint256 timestamp);
  event Claim(address to, bytes32 nullifierHash, uint256 timestamp);
  event CommitmentAdded(bytes32 indexed commitment, uint32 indexed leafIndex);

  constructor(
    IVerifier _verifier,
    uint32 _merkleTreeHeight,
    MockToken _weth,
    MockToken _usdc
  ) MerkleTreeWithHistory(_merkleTreeHeight) {
    verifier = _verifier;
    for (uint256 i = 0; i < LIQUIDATED_ARRAY_NUMBER; i++) {
      liquidated_array.push(Liquidated({liq_price: i + 1, timestamp: 0}));
    }
    weth = _weth;
    usdc = _usdc;
  }

  modifier isWethOrUsdc(MockToken _token) {
    require(address(_token) == address(weth) || address(_token) == address(usdc), "Token must be weth or usdc");
    _;
  }

  function deposit(
    bytes32 _new_note_hash,
    bytes32,
    uint256 _new_timestamp,
    bytes32 _root,
    bytes32 _old_nullifier,
    uint256[2] calldata _pA,
    uint256[2][2] calldata _pB,
    uint256[2] calldata _pC,
    uint256 _lend_amt,
    MockToken _lend_token
  ) external payable nonReentrant isWethOrUsdc(_lend_token) {
    require(_new_timestamp > block.timestamp - 5 minutes, "Invalid timestamp");
    require(_new_timestamp <= block.timestamp, "Invalid timestamp");
    require(_lend_token.transferFrom(msg.sender, address(this), _lend_amt), "Token lend failed");
  }

  function borrow(
    bytes32 _new_note_hash,
    bytes32,
    uint256 _new_timestamp,
    bytes32 _root,
    bytes32 _old_nullifier,
    uint256[2] calldata _pA,
    uint256[2][2] calldata _pB,
    uint256[2] calldata _pC,
    uint256 _borrow_amt,
    MockToken _borrow_token,
    address _to
  ) external payable nonReentrant isWethOrUsdc(_borrow_token) {
    require(_new_timestamp > block.timestamp - 5 minutes, "Invalid timestamp");
    require(_new_timestamp <= block.timestamp, "Invalid timestamp");
    _borrow_token.transfer(_to, _borrow_amt);
  }
}`;

  const loadSample = () => {
    setContract(sampleContract);
    setName('Zringotts_fullContract');
  };

  return (
    <div className="card">
      <div className="card-header">📝 Contract Input</div>

      <form onSubmit={handleSubmit}>
        <div style={{ marginBottom: 16 }}>
          <label style={{ display: 'block', fontSize: 13, color: 'var(--text-muted)', marginBottom: 8 }}>
            Contract Name (optional)
          </label>
          <input
            type="text"
            className="input"
            value={name}
            onChange={(e) => setName(e.target.value)}
            placeholder="MyContract"
            disabled={loading}
            style={{ fontFamily: 'var(--font-sans)' }}
          />
        </div>

        <div style={{ marginBottom: 16 }}>
          <label style={{ display: 'block', fontSize: 13, color: 'var(--text-muted)', marginBottom: 8 }}>
            Solidity Contract Code
          </label>
          <textarea
            className="textarea"
            value={contract}
            onChange={(e) => setContract(e.target.value)}
            placeholder="// Paste your Solidity contract here..."
            disabled={loading}
            style={{ minHeight: '400px' }}
          />
          {tokenEstimate && (
            <div style={{
              marginTop: 8,
              padding: 10,
              background: 'var(--bg-elevated)',
              border: '1px solid var(--border)',
              borderRadius: 'var(--radius-sm)',
              fontSize: 11,
            }}>
              <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: 4 }}>
                <span style={{ color: 'var(--text-muted)' }}>📊 Tokens</span>
                <span style={{ fontWeight: 600, fontSize: 13 }}>
                  {formatTokenCount(tokenEstimate.prompt + tokenEstimate.completion)}
                </span>
              </div>
              <div style={{ color: 'var(--text-dim)', fontSize: 10, marginBottom: 6 }}>
                Prompt: {formatTokenCount(tokenEstimate.prompt)} · Completion: {formatTokenCount(tokenEstimate.completion)}
              </div>
              {estimatedCost && (
                <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', paddingTop: 8, borderTop: '1px solid var(--border)' }}>
                  <span style={{ color: 'var(--text-muted)' }}>💰 Est. cost</span>
                  <span style={{ fontWeight: 700, color: 'var(--accent)', fontSize: 13 }}>
                    ~{estimatedCost} USDC
                  </span>
                </div>
              )}
            </div>
          )}
        </div>

        <div style={{ display: 'flex', gap: 12 }}>
          <button
            type="submit"
            className="btn btn-primary"
            disabled={loading || !contract.trim()}
            style={{ flex: 1 }}
          >
            {loading ? (
              <>
                <div className="spinner" />
                Analyzing...
              </>
            ) : (
              '🔍 Analyze Contract'
            )}
          </button>

          <button
            type="button"
            className="btn btn-secondary"
            onClick={loadSample}
            disabled={loading}
          >
            Load Sample
          </button>
        </div>
      </form>

      <div style={{ marginTop: 20, padding: 16, background: 'var(--bg-input)', borderRadius: 'var(--radius-sm)', fontSize: 12, color: 'var(--text-dim)' }}>
        <strong style={{ color: 'var(--text-muted)' }}>How it works:</strong><br />
        1. <strong>Connect wallet</strong> (MetaMask or Web3 wallet)<br />
        2. <strong>Deposit credits</strong> OR <strong>pay per analysis</strong><br />
        3. Paste your Solidity contract<br />
        4. Click "Analyze" → Payment modal opens<br />
        5. Approve USDC → Pay → Get analysis report<br />
        <div style={{ marginTop: 12, padding: 8, background: 'rgba(0, 150, 255, 0.1)', borderRadius: 4, fontSize: 11 }}>
          💡 <strong>Tip:</strong> Deposit credits once for multiple analyses, or pay per analysis (1 USDC minimum)
        </div>
      </div>
    </div>
  );
}
