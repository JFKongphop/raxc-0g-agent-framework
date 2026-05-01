// RAXC Indexer — Index real DeFi protocol exploits into 0G Storage KV
// Source: datasets-protocol-exploit/src/test/ (recursive .sol files)
// Target: 0G Storage KV (stream: defi_protocols)
// Run: npm run index:protocols

 

import { readFile, writeFile } from 'fs/promises';
import { existsSync } from 'fs';
import { glob } from 'glob';
import OpenAI from 'openai';
import dotenv from 'dotenv';
import path from 'path';
import { ethers } from 'ethers';
import { Indexer, Batcher, getFlowContract } from '@0gfoundation/0g-ts-sdk';

dotenv.config({ path: '../.env' });

// 0G Storage KV Configuration
const STREAM_ID = 'defi_protocols';
const MANIFEST_PATH = '../manifest.json';
const BLOCKCHAIN_RPC = process.env.BLOCKCHAIN_RPC || 'https://evmrpc-testnet.0g.ai';
const INDEXER_RPC = process.env.INDEXER_RPC || 'https://indexer-storage-testnet-turbo.0g.ai';
const PRIVATE_KEY = process.env.PRIVATE_KEY || '';

const EMBED_MODEL = 'text-embedding-3-small';
const VECTOR_SIZE = 1536;
const CODE_TRUNCATE = 6000;
const SKIP_FILES = new Set(['Exploit-template.sol', 'Exploit-template_new.sol', 'RPCS_alive_test.sol']);
const CONCURRENCY = 5; // Parallel upload workers

// Initialize clients
const openai = new OpenAI({
  apiKey: process.env.OPENAI_API_KEY,
});

let indexer: Indexer;
let signer: ethers.Wallet;
let provider: ethers.JsonRpcProvider;

// Atomic nonce counter — safe to increment without locks in single-threaded JS
let _nextNonce = -1;

// ---------------------------------------------------------------------------
// Concurrency primitives
// ---------------------------------------------------------------------------

class Semaphore {
  private available: number;
  private queue: Array<() => void> = [];
  constructor(max: number) { this.available = max; }
  acquire(): Promise<void> {
    if (this.available > 0) { this.available--; return Promise.resolve(); }
    return new Promise<void>(resolve => this.queue.push(resolve));
  }
  release(): void {
    const next = this.queue.shift();
    if (next) { next(); } else { this.available++; }
  }
}

const consoleLock = new Semaphore(1);  // Serialize SDK console.log interception
const manifestLock = new Semaphore(1); // Serialize manifest.json writes

interface ExploitMetadata {
  exploit_name: string;
  source: string;
  date: string;
  chain: string;
  total_lost: string;
  attacker: string;
  attack_tx: string;
  vulnerable_contract: string;
  code: string;
}

interface StorageValue {
  embedding: number[];
  metadata: ExploitMetadata;
}

// ---------------------------------------------------------------------------
// Parsing helpers
// ---------------------------------------------------------------------------

function detectChain(content: string): string {
  const c = content.toLowerCase();
  if (c.includes('bscscan')) return 'BSC';
  if (c.includes('arbiscan')) return 'Arbitrum';
  if (c.includes('optimistic.etherscan') || c.includes('optimism')) return 'Optimism';
  if (c.includes('polygonscan')) return 'Polygon';
  if (c.includes('basescan')) return 'Base';
  if (c.includes('snowtrace') || c.includes('avalanche')) return 'Avalanche';
  if (c.includes('etherscan')) return 'ETH';
  return 'unknown';
}

async function parseSolFile(filePath: string): Promise<ExploitMetadata> {
  const content = await readFile(filePath, 'utf-8');
  const pathParts = filePath.split(path.sep);
  const fileName = path.basename(filePath, '.sol');
  
  // Extract metadata from comments
  const lostMatch = content.match(/Total Lost\s*:\s*(.+)/i);
  const attackerMatch = content.match(/Attacker\s*:\s*(.+)/i);
  const vulnMatch = content.match(/Vulnerable Contract\s*:\s*(.+)/i);
  const attackTxMatch = content.match(/Attack Tx\s*:\s*(.+)/i);
  
  // Extract date from directory structure (e.g., datasets-protocol-exploit/src/test/2023-09-01/)
  let date = 'unknown';
  const dateIndex = pathParts.findIndex(part => /^\d{4}-\d{2}-\d{2}$/.test(part));
  if (dateIndex !== -1) {
    date = pathParts[dateIndex];
  }
  
  return {
    exploit_name: fileName.replace(/_exp$/, ''),
    source: 'DeFiHackLabs-Protocol',
    date: date,
    total_lost: lostMatch ? lostMatch[1].trim() : 'unknown',
    attacker: attackerMatch ? attackerMatch[1].trim() : 'unknown',
    vulnerable_contract: vulnMatch ? vulnMatch[1].trim() : 'unknown',
    attack_tx: attackTxMatch ? attackTxMatch[1].trim() : 'unknown',
    chain: detectChain(content),
    code: content.slice(0, CODE_TRUNCATE),
  };
}

// ---------------------------------------------------------------------------
// Embedding generation
// ---------------------------------------------------------------------------

async function embedText(text: string, retries = 6): Promise<number[]> {
  for (let attempt = 0; attempt < retries; attempt++) {
    try {
      const response = await openai.embeddings.create({
        input: text.slice(0, CODE_TRUNCATE),
        model: EMBED_MODEL,
      });
      return response.data[0].embedding;
    } catch (error: any) {
      if (error.status === 429 && attempt < retries - 1) {
        const waitTime = 10 * Math.pow(2, attempt);
        console.log(`  [~] Rate limited. Waiting ${waitTime}s (retry ${attempt + 1}/${retries})...`);
        await new Promise(resolve => setTimeout(resolve, waitTime * 1000));
      } else {
        throw error;
      }
    }
  }
  throw new Error('Max retries exceeded on embedding');
}

// ---------------------------------------------------------------------------
// 0G Storage KV writer using SDK
// ---------------------------------------------------------------------------

async function initializeClients() {
  if (!PRIVATE_KEY) {
    throw new Error('PRIVATE_KEY not set in .env');
  }
  
  // Create provider and signer
  provider = new ethers.JsonRpcProvider(BLOCKCHAIN_RPC);
  signer = new ethers.Wallet(PRIVATE_KEY, provider);
  
  // Create indexer
  indexer = new Indexer(INDEXER_RPC);
  
  console.log(`[*] Connected to indexer: ${INDEXER_RPC}`);
  console.log(`[*] Signer address: ${signer.address}`);
  _nextNonce = await provider.getTransactionCount(signer.address, 'pending');
  console.log(`[*] Starting nonce: ${_nextNonce}`);
}

// ---------------------------------------------------------------------------
// Manifest: saves { stream -> { key -> rootHash } } for RAG reads
// ---------------------------------------------------------------------------

async function saveToManifest(stream: string, key: string, rootHash: string): Promise<void> {
  await manifestLock.acquire();
  try {
    let manifest: Record<string, Record<string, string>> = {};
    if (existsSync(MANIFEST_PATH)) {
      const raw = await readFile(MANIFEST_PATH, 'utf-8');
      manifest = JSON.parse(raw);
    }
    if (!manifest[stream]) manifest[stream] = {};
    manifest[stream][key] = rootHash;
    await writeFile(MANIFEST_PATH, JSON.stringify(manifest, null, 2));
  } finally {
    manifestLock.release();
  }
}

async function writeToOgKv(
  key: string,
  value: StorageValue,
  maxRetries = 3
): Promise<string | null> {
  // Grab nonce atomically — JS single-threaded event loop guarantees no race on ++
  const myNonce = _nextNonce++;

  // Proxy the signer to inject our explicit nonce so parallel workers don't collide
  const noncedSigner = new Proxy(signer, {
    get(target: any, prop: string) {
      if (prop === 'sendTransaction') {
        return (tx: any) => target.sendTransaction({ ...tx, nonce: myNonce });
      }
      return target[prop];
    },
  });

  const simplifiedValue = {
    embedding: value.embedding,
    metadata: {
      exploit_name: value.metadata.exploit_name,
      source: value.metadata.source,
      date: value.metadata.date,
      chain: value.metadata.chain,
      total_lost: value.metadata.total_lost,
      code_snippet: value.metadata.code.slice(0, 500),
    },
  };
  const encodedValue = Buffer.from(JSON.stringify(simplifiedValue), 'utf-8').toString('base64');

  for (let attempt = 0; attempt < maxRetries; attempt++) {
    try {
      const [nodes, selectErr] = await indexer.selectNodes(1);
      if (selectErr !== null) throw new Error(`Failed to select nodes: ${selectErr}`);

      const FLOW_CONTRACT_ADDRESS = '0x22E03a6A89B950F1c82ec5e74F8eCa321a105296';
      const flowContract = getFlowContract(FLOW_CONTRACT_ADDRESS, signer);
      const batcher = new Batcher(1, nodes, flowContract, BLOCKCHAIN_RPC);

      const streamIdBytes = Uint8Array.from(Buffer.from(STREAM_ID, 'utf-8'));
      const keyBytes = Uint8Array.from(Buffer.from(key, 'utf-8'));
      const valueBytes = Uint8Array.from(Buffer.from(encodedValue, 'utf-8'));
      batcher.streamDataBuilder.set(streamIdBytes as any, keyBytes, valueBytes);

      // Hold consoleLock only while SDK prints "Data prepared root=0x..." (~1-2s).
      // Release BEFORE the long storage-sync wait so other workers can capture their hashes.
      await consoleLock.acquire();
      let capturedRootHash = '';
      const _origLog = console.log;
      const _restoreLog = () => { console.log = _origLog; };

      const rootHashPromise = new Promise<string>((resolve, reject) => {
        const timer = setTimeout(() => reject(new Error('root hash capture timeout')), 15000);
        console.log = (...args: any[]) => {
          const msg = args.map(a => typeof a === 'object' ? JSON.stringify(a) : String(a)).join(' ');
          const m = msg.match(/root=(0x[a-fA-F0-9]{64})/);
          if (m) { clearTimeout(timer); resolve(m[1]); }
          _origLog.apply(console, args);
        };
      });

      // Start exec — runs concurrently; the storage sync happens after root hash is logged
      const execPromise = batcher.exec(noncedSigner as any);

      try {
        capturedRootHash = await rootHashPromise;
      } catch (_) {
        // root hash not captured within timeout
      } finally {
        _restoreLog();
        consoleLock.release(); // Release here — storage sync continues without the lock
      }

      const SYNC_TIMEOUT_MS = 8 * 60 * 1000; // 8-minute cap on storage node sync
      const timeoutPromise = new Promise<[any, Error]>((_, reject) =>
        setTimeout(() => reject(new Error(`Storage node sync timeout after 8 minutes`)), SYNC_TIMEOUT_MS)
      );
      const [, execErr] = await Promise.race([execPromise, timeoutPromise]);
      if (execErr !== null) throw new Error(`Batcher exec failed: ${execErr}`);

      return capturedRootHash || null;
    } catch (error: any) {
      const errorMsg = error.message || String(error);

      if (errorMsg.includes('503') || errorMsg.includes('Service Temporarily Unavailable') ||
          errorMsg.includes('ECONNREFUSED') || errorMsg.includes('network') ||
          errorMsg.includes('failed to select nodes') || errorMsg.includes('timeout')) {
        if (attempt < maxRetries - 1) {
          const waitTime = 30 * (attempt + 1);
          console.log(`  [~] Network error, retrying in ${waitTime}s (attempt ${attempt + 2}/${maxRetries})...`);
          await new Promise(resolve => setTimeout(resolve, waitTime * 1000));
          continue;
        }
      }

      console.log(`  [!] SDK error: ${errorMsg.slice(0, 200)}`);
      return null;
    }
  }

  return null;
}

// ---------------------------------------------------------------------------
// Main indexing loop
// ---------------------------------------------------------------------------

async function indexProtocols() {
  // Initialize clients
  await initializeClients();
  
  const pattern = '../datasets-protocol-exploit/src/test/**/*.sol';  // Only src/test, not script/
  const files = (await glob(pattern))
    .filter(f => !SKIP_FILES.has(path.basename(f)))
    .sort();
  
  const total = files.length;
  console.log(`[*] Found ${total} protocol exploit files\n`);
  console.log(`[*] Target: 0G Storage KV stream '${STREAM_ID}'\n`);
  
  // Load existing manifest to enable resume (skip already-indexed files)
  let existingManifest: Record<string, Record<string, string>> = {};
  if (existsSync(MANIFEST_PATH)) {
    const raw = await readFile(MANIFEST_PATH, 'utf-8');
    existingManifest = JSON.parse(raw);
  }
  const alreadyIndexed = existingManifest[STREAM_ID] || {};
  const alreadyCount = Object.keys(alreadyIndexed).length;
  if (alreadyCount > 0) {
    console.log(`[*] Resuming: skipping ${alreadyCount} already-indexed files\n`);
  }

  // In-memory dedup: claimed the moment a worker picks up a key, preventing
  // parallel workers from uploading the same exploitId twice in one run.
  const claimedThisRun = new Set<string>(Object.keys(alreadyIndexed));

  let indexed = 0;
  let failed = 0;
  const sem = new Semaphore(CONCURRENCY);

  console.log(`[*] Running ${CONCURRENCY} parallel workers\n`);

  const tasks = files.map(filePath => async () => {
    await sem.acquire();
    try {
      const metadata = await parseSolFile(filePath);
      const exploitId = metadata.exploit_name;

      if (claimedThisRun.has(exploitId)) {
        process.stdout.write(`  [skip] ${exploitId}\n`);
        indexed++;
        return;
      }
      // Claim this key immediately so no other parallel worker picks it up
      claimedThisRun.add(exploitId);

      process.stdout.write(`  [~] Embedding: ${exploitId} (${metadata.date})... `);
      const embedding = await embedText(metadata.code);

      const storageValue: StorageValue = { embedding, metadata };
      const result = await writeToOgKv(exploitId, storageValue);

      if (result) {
        console.log(`✓ [${metadata.chain}] root=${result}`);
        await saveToManifest(STREAM_ID, exploitId, result);
        indexed++;
      } else {
        console.log(`✗ FAILED: ${exploitId}`);
        failed++;
      }
    } catch (error: any) {
      console.log(`  [!] Error processing ${filePath}: ${error.message}`);
      failed++;
    } finally {
      sem.release();
    }
  });

  await Promise.all(tasks.map(t => t()));

  console.log(`\n[Done] Indexed: ${indexed}  Failed: ${failed}`);
  console.log(`[*] Data stored in 0G Storage KV stream: ${STREAM_ID}`);
}

// Run the indexer
indexProtocols().catch(console.error);
