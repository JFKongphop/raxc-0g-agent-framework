# RAXC Frontend

Next.js frontend for the RAXC smart contract security scanner with Initia blockchain payment integration.

## Features

- 🔍 **Contract Analysis** — Submit Solidity contracts for vulnerability detection
- 💳 **Pay-per-Analysis** — USDC payment via Initia blockchain
- 🔗 **Initia Native Features** — Auto-signing and interwoven-bridge support
- 📊 **Real-time Results** — Download markdown vulnerability reports
- 🎯 **Accurate Token Counting** — Uses `js-tiktoken` (OpenAI's official tokenizer) for precise GPT-4o token estimation
- 💰 **Live Cost Estimation** — See exact USDC cost as you type your contract

---

## Setup

### 1. Install Dependencies

```bash
npm install
```

### 2. Configure Environment

Copy `.env.example` to `.env.local`:

```bash
cp .env.example .env.local
```

Update with your values:

```bash
# Backend API
NEXT_PUBLIC_API_URL=http://localhost:8080

# Initia Network
NEXT_PUBLIC_RPC_URL=https://jsonrpc-evm-1.anvil.asia-southeast.initia.xyz
NEXT_PUBLIC_CHAIN_ID=your_chain_id_here

# Contract Addresses (from deployment)
NEXT_PUBLIC_VAULT_ADDRESS=0x...
NEXT_PUBLIC_USDC_ADDRESS=0x...
```

### 3. Get Chain ID

```bash
cast chain-id --rpc-url https://jsonrpc-evm-1.anvil.asia-southeast.initia.xyz
```

---

## Development

```bash
npm run dev
```

Open [http://localhost:3000](http://localhost:3000)

---

## Payment Flow

### User Experience

1. **Paste Contract** → User pastes Solidity code
2. **Click Analyze** → Prompts wallet connection (if not connected)
3. **Payment Modal** → Shows:
   - Estimated tokens (prompt + 8000 completion)
   - USDC cost (GPT-4o pricing + 10% fee)
   - Step-by-step: Estimate → Approve → Pay
4. **Transaction** → Uses `payForAnalysis()` on vault contract
5. **Analysis** → Backend verifies payment and runs GPT-4o
6. **Report** → Download markdown vulnerability report

### Token Estimation

- **Tokenizer:** `js-tiktoken` (OpenAI's official tokenizer for GPT-4o)
- **Prompt tokens:** 
  - User contract code (accurately counted with tiktoken)
  - System prompt (~200 tokens)
  - RAG context from exploit database (~6000 tokens)
  - Overhead (~500 tokens)
- **Completion tokens:** `8000` (fixed output as per requirement)
- **Cost calculation:** 
  - Prompt: $2.50 per 1M tokens
  - Completion: $10.00 per 1M tokens
  - Platform fee: 10%
  - **Live preview shown as you type!**

### Why js-tiktoken?

Unlike simple character-based estimates (e.g., `length / 4`), `js-tiktoken` uses OpenAI's actual tokenization algorithm:
- ✅ Accurate for all languages and special characters
- ✅ Handles Unicode, emojis, code properly
- ✅ Same tokenizer used by GPT-4o API
- ✅ Prevents over/undercharging users

---

## Initia Integration

Using `@initia/interwovenkit-react` for:
- **Auto-signing:** Gasless transactions for better UX
- **Interwoven-bridge:** Cross-chain USDC deposits from Ethereum
- **Wallet connection:** Unified Initia wallet interface

---

## Production Build

```bash
npm run build
npm start
```

---

## Links

- [Smart Contract Docs](../contracts/VAULT_README.md)
- [Integration Guide](../contracts/INTEGRATION_GUIDE.md)
- [Backend API Docs](../backend/API_README.md)
- [Deployment Guide](../DEPLOYMENT_INITIA.md)
