// RAXC Frontend Integration Example — Initia Hackathon
// This shows the complete payment flow with Initia Native Features

import { ethers } from 'ethers';

// Contract ABIs (simplified)
const RAXC_VAULT_ABI = [
  "function payForAnalysis(uint256 promptTokens, uint256 completionTokens) external returns (bytes32)",
  "function estimateCost(uint256 promptTokens, uint256 completionTokens) external pure returns (uint256)",
  "function verifyPayment(bytes32 paymentId) external view returns (bool, address, uint256)",
  "function getCreditBalance(address user) external view returns (uint256)",
  "event PaymentReceived(bytes32 indexed paymentId, address indexed user, uint256 amount, uint256 estimatedPromptTokens, uint256 estimatedCompletionTokens)"
];

const ERC20_ABI = [
  "function approve(address spender, uint256 amount) external returns (bool)",
  "function balanceOf(address account) external view returns (uint256)"
];

// Configuration
const INITIA_RPC_URL = "https://jsonrpc-evm-1.anvil.asia-southeast.initia.xyz";
const RAXC_VAULT_ADDRESS = "0x..."; // Deploy address on Initia
const USDC_ADDRESS = "0x..."; // USDC on Initia
const API_ENDPOINT = "https://api.raxc.io/analyze";
const FIXED_OUTPUT_TOKENS = 8000; // As per your requirement

/**
 * Estimate token count from contract code
 * Simple heuristic: ~4 characters = 1 token
 * For production, use @anthropic-ai/tokenizer or OpenAI's tiktoken
 */
function estimateTokens(contractCode: string): number {
  return Math.ceil(contractCode.length / 4);
}

/**
 * Main analysis flow
 */
async function analyzeContract(contractCode: string) {
  // 1. Connect wallet (Initia)
  const provider = new ethers.BrowserProvider(window.ethereum);
  const signer = await provider.getSigner();
  const userAddress = await signer.getAddress();

  // 2. Estimate input tokens from contract
  const inputTokens = estimateTokens(contractCode);
  const outputTokens = FIXED_OUTPUT_TOKENS;

  console.log(`Estimated tokens: ${inputTokens} input + ${outputTokens} output`);

  // 3. Get cost estimate from contract
  const vault = new ethers.Contract(RAXC_VAULT_ADDRESS, RAXC_VAULT_ABI, signer);
  const estimatedCost = await vault.estimateCost(inputTokens, outputTokens);
  
  const costInUSDC = Number(estimatedCost) / 1e6;
  console.log(`Analysis cost: ${costInUSDC} USDC`);

  // Ask user confirmation
  if (!confirm(`Analyze contract for ${costInUSDC} USDC?`)) {
    return;
  }

  // 4. Approve USDC (if needed)
  const usdc = new ethers.Contract(USDC_ADDRESS, ERC20_ABI, signer);
  const currentAllowance = await usdc.allowance(userAddress, RAXC_VAULT_ADDRESS);
  
  if (currentAllowance < estimatedCost) {
    console.log("Approving USDC...");
    const approveTx = await usdc.approve(RAXC_VAULT_ADDRESS, estimatedCost);
    await approveTx.wait();
  }

  // 5. Pay for analysis
  console.log("Processing payment...");
  const paymentTx = await vault.payForAnalysis(inputTokens, outputTokens);
  const receipt = await paymentTx.wait();

  // Extract paymentId from event
  const event = receipt.logs.find(
    (log: any) => log.topics[0] === ethers.id("PaymentReceived(bytes32,address,uint256,uint256,uint256)")
  );
  const paymentId = event.topics[1];
  
  console.log(`Payment successful! ID: ${paymentId}`);
  console.log(`Transaction: ${receipt.hash}`);

  // 6. Submit to API with payment proof
  console.log("Submitting to analysis API...");
  const response = await fetch(API_ENDPOINT, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify({
      contract: contractCode,
      paymentId: paymentId,
      txHash: receipt.hash,
      user: userAddress,
      chain: 'initia'
    })
  });

  if (!response.ok) {
    throw new Error(`API error: ${response.statusText}`);
  }

  const result = await response.json();
  
  // 7. Display results
  console.log("Analysis complete!");
  return result;
}

/**
 * React Component Example
 */
export function ContractAnalyzer() {
  const [contractCode, setContractCode] = React.useState('');
  const [loading, setLoading] = React.useState(false);
  const [result, setResult] = React.useState(null);

  const handleAnalyze = async () => {
    try {
      setLoading(true);
      const analysisResult = await analyzeContract(contractCode);
      setResult(analysisResult);
    } catch (error) {
      console.error('Analysis failed:', error);
      alert(`Error: ${error.message}`);
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="analyzer">
      <h2>RAXC Smart Contract Security Scanner</h2>
      
      <textarea
        value={contractCode}
        onChange={(e) => setContractCode(e.target.value)}
        placeholder="Paste your Solidity contract here..."
        rows={20}
        cols={80}
      />
      
      <button 
        onClick={handleAnalyze} 
        disabled={loading || !contractCode}
      >
        {loading ? 'Analyzing...' : 'Analyze Contract'}
      </button>

      {result && (
        <div className="results">
          <h3>Analysis Results</h3>
          <div className={`risk-badge risk-${result.riskLevel.toLowerCase()}`}>
            Risk Level: {result.riskLevel}
          </div>
          
          <div className="vulnerabilities">
            <h4>Vulnerabilities Found: {result.vulnerabilities.length}</h4>
            {result.vulnerabilities.map((vuln, i) => (
              <div key={i} className="vulnerability">
                <h5>{vuln.type} — {vuln.severity}</h5>
                <p><strong>Location:</strong> {vuln.location}</p>
                <p><strong>Description:</strong> {vuln.description}</p>
                <p><strong>Exploit Reference:</strong> {vuln.exploitReference}</p>
                <p><strong>Fix:</strong> {vuln.recommendation}</p>
              </div>
            ))}
          </div>

          <div className="cost-breakdown">
            <h4>Cost Breakdown</h4>
            <p>Prompt Tokens: {result.cost.promptTokens}</p>
            <p>Completion Tokens: {result.cost.completionTokens}</p>
            <p>Total Charged: {result.cost.totalCharged} USDC</p>
          </div>
        </div>
      )}
    </div>
  );
}

/**
 * Backend API Handler (Node.js/Express)
 */
export async function handleAnalysisRequest(req, res) {
  const { contract, paymentId, txHash, user, chain } = req.body;

  // 1. Verify payment on-chain
  const provider = new ethers.JsonRpcProvider(INITIA_RPC_URL);
  const vault = new ethers.Contract(RAXC_VAULT_ADDRESS, RAXC_VAULT_ABI, provider);
  
  const [isValid, paymentUser, amount] = await vault.verifyPayment(paymentId);
  
  if (!isValid) {
    return res.status(400).json({ error: 'Invalid or already used payment' });
  }
  
  if (paymentUser.toLowerCase() !== user.toLowerCase()) {
    return res.status(403).json({ error: 'Payment user mismatch' });
  }

  // 2. Mark payment as used (using operator private key)
  const operatorSigner = new ethers.Wallet(OPERATOR_PRIVATE_KEY, provider);
  const vaultAsOperator = vault.connect(operatorSigner);
  
  const markTx = await vaultAsOperator.markPaymentUsed(paymentId);
  await markTx.wait();

  // 3. Run GPT-4o analysis
  const openai = new OpenAI({ apiKey: process.env.OPENAI_API_KEY });
  
  // Retrieve similar exploits from Qdrant
  const similarExploits = await retrieveExploits(contract);
  
  const completion = await openai.chat.completions.create({
    model: "gpt-4o",
    messages: [
      {
        role: "system",
        content: `You are a smart contract security expert. Analyze the following contract for vulnerabilities, referencing these real exploits: ${JSON.stringify(similarExploits)}`
      },
      {
        role: "user",
        content: contract
      }
    ],
    max_tokens: 8000 // Fixed output as per requirement
  });

  // 4. Parse vulnerabilities and return
  const analysis = parseAnalysisResponse(completion.choices[0].message.content);
  
  return res.json({
    paymentId,
    status: 'completed',
    vulnerabilities: analysis.vulnerabilities,
    riskLevel: analysis.riskLevel,
    cost: {
      promptTokens: completion.usage.prompt_tokens,
      completionTokens: completion.usage.completion_tokens,
      totalCharged: (amount / 1e6).toString()
    }
  });
}

/**
 * Auto-signing Integration (Initia Native Feature)
 */
async function setupAutosigning() {
  // Enable auto-approval for RAXC payments up to 5 USDC
  if (window.initia && window.initia.autosigning) {
    await window.initia.autosigning.enable({
      contract: RAXC_VAULT_ADDRESS,
      functions: ['payForAnalysis'],
      maxAmount: ethers.parseUnits('5', 6), // 5 USDC
      duration: 7 * 24 * 60 * 60 // 7 days
    });
    
    console.log("Auto-signing enabled for RAXC payments");
  }
}

/**
 * Interwoven Bridge Integration (cross-chain payment)
 */
async function payFromEthereum(contractCode: string) {
  const inputTokens = estimateTokens(contractCode);
  const outputTokens = FIXED_OUTPUT_TOKENS;
  
  // Bridge USDC from Ethereum → Pay on Initia in one transaction
  const bridge = new ethers.Contract(
    INTERWOVEN_BRIDGE_ADDRESS, 
    INTERWOVEN_ABI, 
    ethereumSigner
  );
  
  const tx = await bridge.bridgeAndExecute({
    sourceChain: 'ethereum',
    targetChain: 'initia',
    token: ETH_USDC_ADDRESS,
    targetContract: RAXC_VAULT_ADDRESS,
    targetFunction: 'payForAnalysis',
    params: ethers.AbiCoder.defaultAbiCoder().encode(
      ['uint256', 'uint256'],
      [inputTokens, outputTokens]
    )
  });
  
  await tx.wait();
  console.log("Cross-chain payment successful!");
}
