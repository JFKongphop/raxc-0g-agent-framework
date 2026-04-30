/**
 * RAXC Indexer — Index DeFiVulnLabs vulnerability patterns into 0G Storage KV
 * Source: datasets-case-exploit/src/test/*.sol
 * Target: 0G Storage KV (stream: defi_cases)
 * Run: npm run index:cases
 */

import { readFile } from 'fs/promises';
import { glob } from 'glob';
import OpenAI from 'openai';
import dotenv from 'dotenv';
import path from 'path';
import { ethers } from 'ethers';
import { Indexer, Batcher, getFlowContract } from '@0gfoundation/0g-ts-sdk';

dotenv.config({ path: '../.env' });

// 0G Storage KV Configuration
const STREAM_ID = 'defi_cases';
const BLOCKCHAIN_RPC = process.env.BLOCKCHAIN_RPC || 'https://evmrpc-testnet.0g.ai';
const INDEXER_RPC = process.env.INDEXER_RPC || 'https://indexer-storage-testnet-turbo.0g.ai';
const PRIVATE_KEY = process.env.PRIVATE_KEY || '';

const EMBED_MODEL = 'text-embedding-3-small';
const VECTOR_SIZE = 1536;
const CODE_TRUNCATE = 6000;
const SKIP_FILES = new Set(['interface.sol']);

// Initialize clients
const openai = new OpenAI({
  apiKey: process.env.OPENAI_API_KEY,
});

let indexer: Indexer;
let signer: ethers.Wallet;
let provider: ethers.JsonRpcProvider;

interface ExploitMetadata {
  exploit_name: string;
  vuln_type: string;
  source: string;
  chain: string;
  date: string;
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
// Solidity file parser
// ---------------------------------------------------------------------------

async function parseSolFile(filePath: string): Promise<ExploitMetadata> {
  const content = await readFile(filePath, 'utf-8');
  const fileName = path.basename(filePath, '.sol');
  
  // Extract vulnerability type from filename or comments
  let vulnType = '';
  const vulnMatch = fileName.match(/VulnLabs[_-](.+)/i);
  if (vulnMatch) {
    vulnType = vulnMatch[1].replace(/[-_]/g, ' ');
  }
  
  // Try to find vulnerability description in comments
  const commentMatch = content.match(/\/\/.*vuln.*:(.+)/i);
  if (commentMatch) {
    vulnType = commentMatch[1].trim();
  }
  
  return {
    exploit_name: fileName,
    vuln_type: vulnType,
    source: 'DeFiVulnLabs',
    chain: 'educational',
    date: '2024-01-01',
    total_lost: 'N/A',
    attacker: 'N/A',
    attack_tx: 'NONE',
    vulnerable_contract: path.basename(filePath),
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
}

async function writeToOgKv(
  key: string,
  value: StorageValue,
  maxRetries = 3
): Promise<boolean> {
  // Simplify the value
  const simplifiedValue = {
    embedding: value.embedding,
    metadata: {
      exploit_name: value.metadata.exploit_name,
      vuln_type: value.metadata.vuln_type,
      source: value.metadata.source,
      chain: value.metadata.chain,
      code_snippet: value.metadata.code.slice(0, 500),
    },
  };
  
  const jsonValue = JSON.stringify(simplifiedValue);
  
  // Base64 encode to match our Rust backend expectations
  const encodedValue = Buffer.from(jsonValue, 'utf-8').toString('base64');
  
  // Retry logic for network issues
  for (let attempt = 0; attempt < maxRetries; attempt++) {
    try {
      // Select nodes for KV write
      const [nodes, selectErr] = await indexer.selectNodes(1);
      if (selectErr !== null) {
        throw new Error(`Failed to select nodes: ${selectErr}`);
      }
      
      // Get flow contract (0G Storage official address)
      const FLOW_CONTRACT_ADDRESS = '0x22E03a6A89B950F1c82ec5e74F8eCa321a105296';
      const flowContract = getFlowContract(FLOW_CONTRACT_ADDRESS, signer);
      
      // Create batcher for KV write
      const batcher = new Batcher(1, nodes, flowContract, BLOCKCHAIN_RPC);
      
      // Convert key and value to Uint8Array
      const streamIdBytes = Uint8Array.from(Buffer.from(STREAM_ID, 'utf-8'));
      const keyBytes = Uint8Array.from(Buffer.from(key, 'utf-8'));
      const valueBytes = Uint8Array.from(Buffer.from(encodedValue, 'utf-8'));
      
      // Set key-value pair
      batcher.streamDataBuilder.set(streamIdBytes as any, keyBytes, valueBytes);
      
      // Execute batch write
      const [tx, execErr] = await batcher.exec(signer as any);
      if (execErr !== null) {
        throw new Error(`Batcher exec failed: ${execErr}`);
      }
      
      return true;
    } catch (error: any) {
      const errorMsg = error.message || String(error);
      
      // Check if it's a 503 error or network issue
      if (errorMsg.includes('503') || errorMsg.includes('Service Temporarily Unavailable') || 
          errorMsg.includes('ECONNREFUSED') || errorMsg.includes('network') ||
          errorMsg.includes('failed to select nodes')) {
        if (attempt < maxRetries - 1) {
          const waitTime = 30 * (attempt + 1);
          console.log(`  [~] Network error, retrying in ${waitTime}s (attempt ${attempt + 2}/${maxRetries})...`);
          await new Promise(resolve => setTimeout(resolve, waitTime * 1000));
          continue;
        }
      }
      
      console.log(`  [!] SDK error: ${errorMsg.slice(0, 200)}`);
      return false;
    }
  }
  
  return false;
}

// ---------------------------------------------------------------------------
// Main indexing loop
// ---------------------------------------------------------------------------

async function indexVulnLabs() {
  // Initialize clients
  await initializeClients();
  
  const srcDir = '../datasets-case-exploit/src/test';
  const pattern = path.join(srcDir, '*.sol');
  const files = (await glob(pattern))
    .filter(f => !SKIP_FILES.has(path.basename(f)))
    .sort();
  
  const total = files.length;
  console.log(`[*] Found ${total} vulnerability pattern files in ${srcDir}\n`);
  console.log(`[*] Target: 0G Storage KV stream '${STREAM_ID}'\n`);
  
  let indexed = 0;
  let failed = 0;
  
  for (const filePath of files) {
    try {
      // Parse metadata
      const metadata = await parseSolFile(filePath);
      const exploitId = metadata.exploit_name;
      
      // Generate embedding
      process.stdout.write(`  [~] Embedding: ${exploitId}... `);
      const embedding = await embedText(metadata.code);
      
      // Prepare storage payload
      const storageValue: StorageValue = {
        embedding,
        metadata,
      };
      
      // Write to 0G Storage KV
      if (await writeToOgKv(exploitId, storageValue)) {
        console.log(`✓ [${metadata.vuln_type}]`);
        indexed++;
      } else {
        console.log(`✗ FAILED`);
        failed++;
      }
      
      // Rate limit protection
      await new Promise(resolve => setTimeout(resolve, 1500));
    } catch (error: any) {
      console.log(`  [!] Error processing ${filePath}: ${error.message}`);
      failed++;
    }
  }
  
  console.log(`\n[Done] Indexed: ${indexed}  Failed: ${failed}`);
  console.log(`[*] Data stored in 0G Storage KV stream: ${STREAM_ID}`);
}

// Run the indexer
indexVulnLabs().catch(console.error);
