/**
 * Token counting utilities for GPT-4o model
 * Uses js-tiktoken (official OpenAI tokenizer) for accurate token estimation
 */

import { encodingForModel } from 'js-tiktoken';

// GPT-4o uses cl100k_base encoding (same as GPT-4)
let encoder: ReturnType<typeof encodingForModel> | null = null;

/**
 * Initialize the tokenizer (lazy loaded)
 */
function getEncoder() {
  if (!encoder) {
    encoder = encodingForModel('gpt-4o');
  }
  return encoder;
}

/**
 * Count tokens in a string using GPT-4o tokenizer
 * @param text - The text to count tokens for
 * @returns Number of tokens
 */
export function countTokens(text: string): number {
  try {
    const enc = getEncoder();
    const tokens = enc.encode(text);
    return tokens.length;
  } catch (error) {
    console.error('Token counting error:', error);
    // Fallback: ~4 chars = 1 token (rough estimate)
    return Math.ceil(text.length / 4);
  }
}

/**
 * Estimate prompt tokens for contract analysis
 * Includes system prompt + user contract + context overhead
 * @param contractCode - The Solidity contract code
 * @returns Estimated prompt tokens
 */
export function estimatePromptTokens(contractCode: string): number {
  // System prompt (approximate based on RAXC's actual prompt)
  const systemPrompt = `You are a smart contract security auditor. Analyze the following Solidity contract for vulnerabilities...`;
  
  // RAG context (approximate: top 3 matches, ~2000 tokens each)
  const ragContextEstimate = 6000;
  
  // User message with contract
  const userMessage = `Analyze this contract:\n\n${contractCode}`;
  
  const systemTokens = countTokens(systemPrompt);
  const userTokens = countTokens(userMessage);
  
  // Total: system + user + RAG context + overhead
  return systemTokens + userTokens + ragContextEstimate + 500;
}

/**
 * Fixed completion tokens for RAXC analysis
 * Based on user requirement: 8000 tokens output
 */
export const COMPLETION_TOKENS = 8000;

/**
 * Estimate total tokens for analysis
 * @param contractCode - The Solidity contract code
 * @returns Object with prompt and completion token estimates
 */
export function estimateAnalysisTokens(contractCode: string) {
  return {
    prompt: estimatePromptTokens(contractCode),
    completion: COMPLETION_TOKENS,
  };
}

/**
 * Calculate estimated cost in USDC
 * Based on GPT-4o pricing + 10% platform fee
 * @param promptTokens - Number of prompt tokens
 * @param completionTokens - Number of completion tokens
 * @returns Estimated cost in USDC (6 decimals)
 */
export function estimateCostUSDC(promptTokens: number, completionTokens: number): string {
  // GPT-4o pricing (per 1M tokens)
  const PROMPT_PRICE = 2.50; // $2.50 per 1M prompt tokens
  const COMPLETION_PRICE = 10.00; // $10.00 per 1M completion tokens
  const PLATFORM_FEE = 0.10; // 10% fee
  
  // Calculate base cost
  const promptCost = (promptTokens / 1_000_000) * PROMPT_PRICE;
  const completionCost = (completionTokens / 1_000_000) * COMPLETION_PRICE;
  const baseCost = promptCost + completionCost;
  
  // Add platform fee
  const totalCost = baseCost * (1 + PLATFORM_FEE);
  
  return totalCost.toFixed(6);
}

/**
 * Format token count with commas for display
 */
export function formatTokenCount(tokens: number): string {
  return tokens.toLocaleString();
}

/**
 * Free encoder resources (call on cleanup)
 * Note: js-tiktoken handles cleanup automatically
 */
export function freeEncoder() {
  // Reset encoder reference (actual cleanup handled by js-tiktoken internally)
  encoder = null;
}
