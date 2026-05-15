import { JsonRpcProvider, Contract, AbiCoder, id as keccak256id } from 'ethers';

// ── 0G Mainnet ───────────────────────────────────────────────────────────────
export const RPC_URL = 'https://evmrpc.0g.ai';
export const CHAIN_ID = 16661;

// ── Deployed contract addresses ───────────────────────────────────────────────
export const ADDRESSES = {
  /** ERC-8183 audit task lifecycle */
  auditTask8183: '0xa018a255881e0525831df7bcdf9a03d1b06e1790',
  /** ERC-7857 intelligent agent NFT */
  agentNFT: '0xf335a9b58f2aa6a2f884d2da4e308f7378a4cf7e',
} as const;

// ── Event topic hashes (keccak256 of canonical signature) ─────────────────────
// AuditTaskCompleted(uint256,string,bytes32,string,uint256)
const TOPIC_AUDIT_COMPLETED = keccak256id(
  'AuditTaskCompleted(uint256,string,bytes32,string,uint256)'
);
// Updated(uint256,(string,bytes32)[],(string,bytes32)[])
const TOPIC_AGENT_UPDATED = keccak256id(
  'Updated(uint256,(string,bytes32)[],(string,bytes32)[])'
);

export interface ChainStats {
  /** Number of finalized audit tasks on-chain (ERC-8183) */
  auditsCompleted: number;
  /** Same as auditsCompleted — each audit produces one replay trace */
  replayTraces: number;
  /** Root hashes stored on 0G — one per completed audit */
  rootHashesStored: number;
  /** ERC-7857 intelligence pointer updates */
  erc7857Updates: number;
  /** Whether the RPC was reachable */
  online: boolean;
}

/**
 * Read live stats from the deployed contracts on 0G Galileo.
 * Returns null counts (offline) if the RPC is unreachable.
 */
export async function fetchChainStats(): Promise<ChainStats> {
  try {
    const provider = new JsonRpcProvider(RPC_URL);

    const [auditLogs, updatedLogs] = await Promise.all([
      // Count AuditTaskCompleted events on RaxcAuditTask8183
      provider.getLogs({
        address: ADDRESSES.auditTask8183,
        topics: [TOPIC_AUDIT_COMPLETED],
        fromBlock: 0,
        toBlock: 'latest',
      }),
      // Count Updated events on RaxcAgentNFT
      provider.getLogs({
        address: ADDRESSES.agentNFT,
        topics: [TOPIC_AGENT_UPDATED],
        fromBlock: 0,
        toBlock: 'latest',
      }),
    ]);

    const completed = auditLogs.length;

    return {
      auditsCompleted: completed,
      replayTraces: completed,
      rootHashesStored: completed,
      erc7857Updates: updatedLogs.length,
      online: true,
    };
  } catch {
    return {
      auditsCompleted: 0,
      replayTraces: 0,
      rootHashesStored: 0,
      erc7857Updates: 0,
      online: false,
    };
  }
}

// ── Audit task types & fetchers ───────────────────────────────────────────────

export interface OnChainAudit {
  taskId: string;
  rootHash: string;   // topics[2] — bytes32 indexed
  verdict: string;    // event data[0]
  replayId: string;   // event data[1]
  completedAt: Date;  // event data[2]
  txHash?: string;     // transaction hash of the AuditTaskCompleted event
  // Detail-only fields (populated by fetchAuditTask via getTask)
  contractName?: string;
  confidence?: number;
  traceHash?: string;
  requester?: string;
}

// ABI for single-task detail fetch only
const AUDIT_TASK_ABI = [
  'function getTask(uint256) view returns (address, string, uint8, string, uint256, bytes32, string, bytes32, uint256, uint256)',
];

function verdictToSeverity(verdict: string): 'critical' | 'high' | 'medium' | 'low' {
  const v = verdict.toUpperCase();
  if (v.includes('CRITICAL')) return 'critical';
  if (v.includes('HIGH'))     return 'high';
  if (v.includes('MEDIUM'))   return 'medium';
  return 'low';
}

/**
 * Fetch all finalized audit tasks — event-only, no extra RPC calls.
 * Decodes AuditTaskCompleted logs:
 *   topics[1] = taskId (indexed uint256)
 *   topics[2] = rootHash (indexed bytes32)
 *   data      = ABI(verdict: string, replayId: string, timestamp: uint256)
 */
export async function fetchAuditTasks(): Promise<OnChainAudit[]> {
  try {
    const provider = new JsonRpcProvider(RPC_URL);
    const coder = AbiCoder.defaultAbiCoder();

    const logs = await provider.getLogs({
      address: ADDRESSES.auditTask8183,
      topics: [TOPIC_AUDIT_COMPLETED],
      fromBlock: 0,
      toBlock: 'latest',
    });

    if (logs.length === 0) return [];

    const tasks: OnChainAudit[] = logs.map((log) => {
      const taskId   = BigInt(log.topics[1]).toString();
      const rootHash = log.topics[2];
      const [verdict, replayId, timestamp] = coder.decode(
        ['string', 'string', 'uint256'],
        log.data
      );
      return {
        taskId,
        rootHash,
        verdict:     String(verdict),
        replayId:    String(replayId),
        completedAt: new Date(Number(timestamp) * 1000),
        txHash:      log.transactionHash,
      };
    });

    return tasks.sort((a, b) => b.completedAt.getTime() - a.completedAt.getTime());
  } catch {
    return [];
  }
}

/**
 * Fetch a single audit task by taskId.
 */
export async function fetchAuditTask(taskId: string): Promise<OnChainAudit | null> {
  try {
    const provider = new JsonRpcProvider(RPC_URL);
    const contract = new Contract(ADDRESSES.auditTask8183, AUDIT_TASK_ABI, provider);
    const t = await contract.getTask(taskId);

    if (Number(t[2]) !== 1) return null; // not Completed

    return {
      taskId,
      contractName: t[1] as string,
      verdict: t[3] as string,
      confidence: Number(t[4]) / 100,
      rootHash: t[5] as string,
      replayId: t[6] as string,
      traceHash: t[7] as string,
      requester: t[0] as string,
      completedAt: new Date(Number(t[9]) * 1000),
    };
  } catch {
    return null;
  }
}

export { verdictToSeverity };

// 0G Storage HTTP gateway — used to try fetching the full markdown report
export const OG_STORAGE_GATEWAY = 'https://indexer-storage-turbo.0g.ai';

// ── ERC-7857 root hash fetcher (mirrors og_storage.rs load_from_chain_full) ──

export interface RootHashEntry {
  rootHash: string; // 0x + 64 hex — 0G Storage merkle root
  dataKey: string;  // human label stored alongside the hash
  tokenId: string;
}

const ZERO_HASH      = '0x' + '0'.repeat(64);
const BOOTSTRAP_HASH = '0x' + '0'.repeat(63) + '1';

/**
 * Fetch 0G Storage root hashes from the ERC-7857 Updated events.
 * Mirrors the Rust backend og_storage.rs load_from_chain_full() logic:
 *   Updated(uint256 indexed tokenId, (string,bytes32)[] oldDatas, (string,bytes32)[] newDatas)
 *   → collect newDatas[].dataHash (bytes32), skip zeros and bootstrap placeholder.
 */
export async function fetchERC7857RootHashes(): Promise<RootHashEntry[]> {
  try {
    const provider = new JsonRpcProvider(RPC_URL);
    const coder = AbiCoder.defaultAbiCoder();

    const logs = await provider.getLogs({
      address: ADDRESSES.agentNFT,
      topics: [TOPIC_AGENT_UPDATED],
      fromBlock: 0,
      toBlock: 'latest',
    });

    const seen = new Set<string>();
    const entries: RootHashEntry[] = [];

    for (const log of logs) {
      const tokenId = BigInt(log.topics[1]).toString();
      // data = ABI((string,bytes32)[] oldDatas, (string,bytes32)[] newDatas)
      const [, newDatas] = coder.decode(
        ['(string,bytes32)[]', '(string,bytes32)[]'],
        log.data
      );
      for (const item of newDatas as Array<[string, string]>) {
        const dataKey  = item[0] as string;
        const rootHash = item[1] as string; // bytes32 → 0x-prefixed hex
        if (rootHash === ZERO_HASH || rootHash === BOOTSTRAP_HASH) continue;
        if (!seen.has(rootHash)) {
          seen.add(rootHash);
          entries.push({ rootHash, dataKey, tokenId });
        }
      }
    }

    return entries;
  } catch {
    return [];
  }
}
