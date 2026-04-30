# TypeScript Indexer for 0G Storage KV

This is a TypeScript alternative to the Python indexer for populating 0G Storage KV with exploit data.

## Setup

Install dependencies:
```bash
npm install
```

## Configuration

Create a `.env` file or use the parent directory's `.env` with these variables:

```bash
# OpenAI API Key
OPENAI_API_KEY=sk-proj-...

# 0G Storage Configuration
BLOCKCHAIN_RPC=https://evmrpc-testnet.0g.ai
INDEXER_RPC=https://indexer-storage-testnet-standard.0g.ai
PRIVATE_KEY=your_private_key_without_0x_prefix

# CLI path (relative to this directory)
OG_CLI_PATH=../0g-cli
```

## Usage

Index DeFiVulnLabs vulnerability patterns:
```bash
npm run index:cases
```

Index DeFiHackLabs protocol exploits (when available):
```bash
npm run index:protocols
```

## Features

- ✅ Base64 encoding for CLI data (avoids shell escaping issues)
- ✅ Automatic retry logic for 503 errors (3 retries with exponential backoff)
- ✅ OpenAI embedding generation with rate limit handling
- ✅ TypeScript type safety

## How It Works

1. **Read** Solidity files from `datasets-case-exploit/src/test/`
2. **Parse** metadata (exploit name, vulnerability type)
3. **Embed** code using OpenAI `text-embedding-3-small` (1536 dimensions)
4. **Encode** JSON as base64 to avoid CLI parsing issues
5. **Write** to 0G Storage KV using `0g-cli kv-write`

## Troubleshooting

**503 Service Unavailable**: 0G testnet indexer is down. The script will automatically retry 3 times with increasing delays (30s, 60s, 90s).

**Rate limits**: OpenAI rate limits are handled automatically with exponential backoff.

**CLI timeout**: Default timeout is 180 seconds per write. Adjust in `indexer_case_0g.ts` if needed.
