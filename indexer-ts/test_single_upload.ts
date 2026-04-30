/**
 * Simple test script to upload a single KV entry to 0G Storage
 * Run: npx tsx test_single_upload.ts
 */

import dotenv from 'dotenv';
import { ethers } from 'ethers';
import { Indexer, Batcher, getFlowContract } from '@0gfoundation/0g-ts-sdk';

dotenv.config({ path: '../.env' });

// 0G Storage KV Configuration
const STREAM_ID = 'defi_cases';
const BLOCKCHAIN_RPC = process.env.BLOCKCHAIN_RPC || 'https://evmrpc-testnet.0g.ai';
const INDEXER_RPC = process.env.INDEXER_RPC || 'https://indexer-storage-testnet-turbo.0g.ai';
const PRIVATE_KEY = process.env.PRIVATE_KEY || '';

async function testSingleUpload() {
  console.log('=== 0G Storage KV Single Upload Test (TESTNET) ===\n');
  
  // Initialize clients
  console.log('[1/5] Creating provider and signer...');
  const provider = new ethers.JsonRpcProvider(BLOCKCHAIN_RPC);
  const signer = new ethers.Wallet(PRIVATE_KEY, provider);
  console.log(`      Signer address: ${signer.address}`);
  
  // Check balance
  const balance = await provider.getBalance(signer.address);
  console.log(`      Balance: ${ethers.formatEther(balance)} A0GI\n`);
  
  if (balance === 0n) {
    console.log('⚠️  WARNING: Wallet has zero balance!');
    console.log('   You may need testnet A0GI tokens from a faucet.');
    console.log('   Continuing anyway - testnet might not require gas...\n');
  }
  
  console.log('[2/5] Connecting to indexer...');
  const indexer = new Indexer(INDEXER_RPC);
  console.log(`      Indexer: ${INDEXER_RPC}\n`);
  
  // Create test data
  console.log('[3/5] Creating test data...');
  const testKey = 'test_exploit_001';
  const testValue = {
    embedding: Array(1536).fill(0).map(() => Math.random()),
    metadata: {
      exploit_name: 'Test Reentrancy',
      vuln_type: 'Reentrancy Attack',
      source: 'Test',
      chain: 'testnet',
      code_snippet: 'contract Vulnerable { ... }',
    },
  };
  
  const jsonValue = JSON.stringify(testValue);
  const encodedValue = Buffer.from(jsonValue, 'utf-8').toString('base64');
  console.log(`      Key: ${testKey}`);
  console.log(`      Value size: ${jsonValue.length} bytes (${encodedValue.length} bytes base64)\n`);
  
  // Select nodes
  console.log('[4/5] Selecting storage nodes...');
  const [nodes, selectErr] = await indexer.selectNodes(1);
  if (selectErr !== null) {
    console.error(`      ✗ Failed to select nodes: ${selectErr}`);
    return;
  }
  console.log(`      ✓ Selected ${nodes.length} node(s)`);
  console.log(`      Node info:`, JSON.stringify(nodes[0], null, 2), '\n');
  
  // Upload to 0G KV
  console.log('[5/5] Uploading to 0G Storage KV...');
  try {
    // 0G Storage Flow contract (official address)
    const FLOW_CONTRACT_ADDRESS = '0x22E03a6A89B950F1c82ec5e74F8eCa321a105296';
    console.log(`      Using flow contract: ${FLOW_CONTRACT_ADDRESS}`);
    const flowContract = getFlowContract(FLOW_CONTRACT_ADDRESS, signer);
    
    // Create batcher
    const batcher = new Batcher(1, nodes, flowContract, BLOCKCHAIN_RPC);
    
    // Convert to bytes
    const streamIdBytes = Uint8Array.from(Buffer.from(STREAM_ID, 'utf-8'));
    const keyBytes = Uint8Array.from(Buffer.from(testKey, 'utf-8'));
    const valueBytes = Uint8Array.from(Buffer.from(encodedValue, 'utf-8'));
    
    // Set key-value pair
    batcher.streamDataBuilder.set(streamIdBytes as any, keyBytes, valueBytes);
    
    // Execute
    console.log('      Executing transaction...');
    const [tx, execErr] = await batcher.exec(signer as any);
    
    if (execErr !== null) {
      console.error(`      ✗ Batcher exec failed: ${execErr}`);
      return;
    }
    
    console.log(`      ✓ Transaction successful!`);
    console.log(`      Stream: ${STREAM_ID}`);
    console.log(`      Key: ${testKey}`);
    console.log('\n✅ Upload completed successfully!');
    
  } catch (error: any) {
    console.error(`\n❌ Upload failed: ${error.message}`);
    if (error.message.includes('503')) {
      console.log('\n⚠️  The 0G testnet indexer is currently unavailable (503 error).');
      console.log('   Please try again later or check https://docs.0g.ai for status.');
    }
  }
}

// Run the test
testSingleUpload().catch(console.error);
