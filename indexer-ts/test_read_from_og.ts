import * as fs from 'fs';
import * as path from 'path';
import { exec } from 'child_process';
import { promisify } from 'util';
import * as dotenv from 'dotenv';

dotenv.config();

const execAsync = promisify(exec);

const INDEXER_RPC = process.env.INDEXER_RPC || 'https://indexer-storage-testnet-turbo.0g.ai';
const CLI_PATH = '../0g-cli';

interface ExploitData {
  embedding: number[];
  metadata: {
    exploit_name: string;
    vuln_type: string;
    source: string;
    chain: string;
    code_snippet: string;
  };
}

/**
 * Download file from 0G Storage using CLI and extract KV value
 */
async function readFromOgKv(
  rootHash: string,
  streamId: string,
  key: string
): Promise<ExploitData | null> {
  const tempFile = path.join('/tmp', `og_download_${Date.now()}.bin`);
  
  try {
    console.log(`📥 Downloading file from 0G Storage...`);
    console.log(`   Root: ${rootHash}`);
    
    // Download file using CLI
    const downloadCmd = `${CLI_PATH} download --indexer ${INDEXER_RPC} --root ${rootHash} --file ${tempFile}`;
    const { stdout, stderr } = await execAsync(downloadCmd);
    
    if (stderr && stderr.includes('ERROR')) {
      throw new Error(`Download failed: ${stderr}`);
    }
    
    console.log(`✅ Downloaded to ${tempFile}`);
    
    // Read and parse the file
    const data = fs.readFileSync(tempFile);
    console.log(`📊 File size: ${data.length} bytes`);
    
    // Find stream ID
    const streamIdBytes = Buffer.from(streamId, 'utf8');
    const streamIdIndex = data.indexOf(streamIdBytes);
    
    if (streamIdIndex === -1) {
      throw new Error(`Stream ID "${streamId}" not found in downloaded data`);
    }
    
    console.log(`✅ Found stream ID at offset ${streamIdIndex}`);
    
    // Find key after stream ID
    const keyBytes = Buffer.from(key, 'utf8');
    const keyIndex = data.indexOf(keyBytes, streamIdIndex);
    
    if (keyIndex === -1) {
      throw new Error(`Key "${key}" not found after stream ID`);
    }
    
    console.log(`✅ Found key at offset ${keyIndex}`);
    
    // Extract value (base64-encoded JSON after key)
    const afterKeyOffset = keyIndex + keyBytes.length;
    const remainingData = data.slice(afterKeyOffset);
    const remainingText = remainingData.toString('utf8');
    
    // Find base64 pattern
    const base64Match = remainingText.match(/([A-Za-z0-9+/]{100,}={0,2})/);
    
    if (!base64Match) {
      throw new Error('No base64 value found after key');
    }
    
    console.log(`✅ Found base64 value (${base64Match[1].length} chars)`);
    
    // Decode base64 and parse JSON
    const decodedBytes = Buffer.from(base64Match[1], 'base64');
    const decodedText = decodedBytes.toString('utf8');
    const parsedData = JSON.parse(decodedText) as ExploitData;
    
    console.log(`✅ Successfully decoded KV value`);
    console.log(`   Embedding dimension: ${parsedData.embedding.length}`);
    console.log(`   Exploit: ${parsedData.metadata.exploit_name}`);
    
    // Cleanup temp file
    fs.unlinkSync(tempFile);
    
    return parsedData;
    
  } catch (error: any) {
    console.error('❌ Error reading from 0G KV:', error.message);
    
    // Cleanup temp file if exists
    if (fs.existsSync(tempFile)) {
      fs.unlinkSync(tempFile);
    }
    
    return null;
  }
}

/**
 * Test the read functionality
 */
async function testRead() {
  console.log('=== Test Read from 0G KV ===\n');
  
  const rootHash = '0x788ecdc715fc45a2bac2f4e7ca6064b07ed52595e3c7a47ac83082df3b7cac73';
  const streamId = 'defi_cases';
  const key = 'test_exploit_001';
  
  const result = await readFromOgKv(rootHash, streamId, key);
  
  if (result) {
    console.log('\n--- Retrieved Data ---');
    console.log(JSON.stringify({
      embedding: `[${result.embedding.length} dimensions]`,
      metadata: result.metadata
    }, null, 2));
    
    console.log('\n✅ Read test successful!');
  } else {
    console.log('\n❌ Read test failed');
    process.exit(1);
  }
}

// Run test
testRead();
