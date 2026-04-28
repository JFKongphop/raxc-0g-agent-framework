// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "openzeppelin/token/ERC20/extensions/ERC4626.sol";
import "openzeppelin/access/AccessControl.sol";
import "openzeppelin/token/ERC20/IERC20.sol";
import "openzeppelin/token/ERC20/utils/SafeERC20.sol";

/**
 * @title RaxcCreditVault
 * @notice ERC4626 vault for RAXC platform credit system
 * @dev Users deposit USDC to receive vault shares representing their credit balance.
 *      Backend operator deducts analysis costs from user balances based on actual AI token usage.
 *      Fee model: user_charge = actual_cost × 1.10 (10% platform fee)
 */
contract RaxcCreditVault is ERC4626, AccessControl {
  using SafeERC20 for IERC20;

  // Role for backend operator to deduct costs
  bytes32 public constant OPERATOR_ROLE = keccak256("OPERATOR_ROLE");

  // Role for admin functions
  bytes32 public constant ADMIN_ROLE = keccak256("ADMIN_ROLE");

  // Platform fee percentage (10% = 1000 basis points)
  uint256 public constant PLATFORM_FEE_BPS = 1000;
  uint256 public constant BPS_DENOMINATOR = 10_000;

  // OpenAI pricing constants (in USDC units with 6 decimals per million tokens)
  // GPT-4o: $2.50/1M prompt tokens, $10.00/1M completion tokens
  // USDC has 6 decimals, so 1 USDC = 1e6 units
  uint256 public constant PROMPT_TOKEN_COST_PER_MILLION = 2_500_000; // 2.5 USDC per million tokens
  uint256 public constant COMPLETION_TOKEN_COST_PER_MILLION = 10_000_000; // 10 USDC per million tokens
  uint256 public constant ONE_MILLION = 1_000_000;

  // Minimum deposit requirement (1 USDC with 6 decimals)
  uint256 public constant MIN_DEPOSIT = 1_000_000; // 1 USDC

  // Track total fees collected for platform
  uint256 public totalFeesCollected;

  // Track total costs deducted (to adjust totalAssets calculation)
  uint256 public totalCostsDeducted;

  // Payment tracking for pay-per-analysis model
  struct Payment {
    address user;
    uint256 amount;
    uint256 estimatedPromptTokens;
    uint256 estimatedCompletionTokens;
    bool used;
    uint256 timestamp;
  }
  
  mapping(bytes32 => Payment) public payments; // paymentId => Payment

  // Events
  event CostDeducted(
    address indexed user,
    uint256 promptTokens,
    uint256 completionTokens,
    uint256 actualCost,
    uint256 platformFee,
    uint256 totalCharged
  );

  event PaymentReceived(
    bytes32 indexed paymentId,
    address indexed user,
    uint256 amount,
    uint256 estimatedPromptTokens,
    uint256 estimatedCompletionTokens
  );

  event PaymentUsed(bytes32 indexed paymentId, address indexed user);

  event FeesWithdrawn(address indexed recipient, uint256 amount);
  event OperatorAdded(address indexed operator);
  event OperatorRemoved(address indexed operator);

  /**
   * @notice Constructor
   * @param _asset The underlying asset (USDC token address)
   * @param _name The name of the vault share token
   * @param _symbol The symbol of the vault share token
   */
  constructor(IERC20 _asset, string memory _name, string memory _symbol) ERC4626(_asset) ERC20(_name, _symbol) {
    _grantRole(DEFAULT_ADMIN_ROLE, msg.sender);
    _grantRole(ADMIN_ROLE, msg.sender);
  }

  /**
   * @notice Add an operator (backend service) that can deduct costs
   * @param operator Address to grant operator role
   */
  function addOperator(address operator) external onlyRole(ADMIN_ROLE) {
    grantRole(OPERATOR_ROLE, operator);
    emit OperatorAdded(operator);
  }

  /**
   * @notice Remove an operator
   * @param operator Address to revoke operator role from
   */
  function removeOperator(address operator) external onlyRole(ADMIN_ROLE) {
    revokeRole(OPERATOR_ROLE, operator);
    emit OperatorRemoved(operator);
  }

  /**
   * @notice Override totalAssets to account for deducted costs
   * @dev Deducted costs remain in the contract but shouldn't be counted toward share value
   * @return Total assets available to share holders (excluding deducted costs)
   */
  function totalAssets() public view virtual override returns (uint256) {
    uint256 balance = IERC20(asset()).balanceOf(address(this));
    // Subtract the costs that have been deducted (these are earmarked for platform use)
    return balance > totalCostsDeducted ? balance - totalCostsDeducted : 0;
  }

  /**
   * @notice Calculate cost based on OpenAI token usage
   * @param promptTokens Number of prompt tokens used
   * @param completionTokens Number of completion tokens used
   * @return actualCost The raw cost based on OpenAI pricing
   * @return platformFee The 10% platform fee
   * @return totalCost The total amount to charge user (actualCost + platformFee)
   */
  function calculateCost(
    uint256 promptTokens,
    uint256 completionTokens
  )
    public
    pure
    returns (uint256 actualCost, uint256 platformFee, uint256 totalCost)
  {
    // Calculate actual cost: (prompt_tokens × $2.50/1M) + (completion_tokens × $10.00/1M)
    uint256 promptCost = (promptTokens * PROMPT_TOKEN_COST_PER_MILLION) / ONE_MILLION;
    uint256 completionCost = (completionTokens * COMPLETION_TOKEN_COST_PER_MILLION) / ONE_MILLION;
    actualCost = promptCost + completionCost;

    // Calculate 10% platform fee
    platformFee = (actualCost * PLATFORM_FEE_BPS) / BPS_DENOMINATOR;

    // Total charge = actual cost + platform fee (110% of actual cost)
    totalCost = actualCost + platformFee;
  }

  /**
   * @notice Deduct analysis cost from user's vault balance
   * @dev Only callable by operator role (backend service)
   * @param user Address of the user to deduct from
   * @param promptTokens Number of prompt tokens used in the analysis
   * @param completionTokens Number of completion tokens used in the analysis
   */
  function deductCost(address user, uint256 promptTokens, uint256 completionTokens) external onlyRole(OPERATOR_ROLE) {
    // Calculate costs
    (uint256 actualCost, uint256 platformFee, uint256 totalCost) = calculateCost(promptTokens, completionTokens);

    // Check user has sufficient balance
    uint256 userAssets = convertToAssets(balanceOf(user));
    require(userAssets >= totalCost, "Insufficient credit balance");

    // Calculate shares needed for the total cost
    uint256 sharesToBurn = convertToShares(totalCost);
    require(balanceOf(user) >= sharesToBurn, "Insufficient shares");

    // Burn the user's shares - this reduces their balance
    _burn(user, sharesToBurn);

    // Track the total costs deducted (affects totalAssets calculation)
    totalCostsDeducted += totalCost;

    // Platform fee portion is tracked separately for admin withdrawal
    totalFeesCollected += platformFee;

    emit CostDeducted(user, promptTokens, completionTokens, actualCost, platformFee, totalCost);
  }

  /**
   * @notice Get user's credit balance in underlying asset (USDC)
   * @param user Address to check balance for
   * @return balance The amount of USDC credit the user has
   */
  function getCreditBalance(address user) public view returns (uint256) {
    uint256 shares = balanceOf(user);
    return convertToAssets(shares);
  }

  /**
   * @notice Withdraw accumulated platform fees
   * @dev Only callable by admin
   * @param recipient Address to receive the fees
   * @param amount Amount of fees to withdraw
   */
  function withdrawFees(address recipient, uint256 amount) external onlyRole(ADMIN_ROLE) {
    require(amount <= totalFeesCollected, "Amount exceeds collected fees");

    totalFeesCollected -= amount;
    totalCostsDeducted -= amount; // Reduce deducted costs since assets are leaving
    IERC20(asset()).safeTransfer(recipient, amount);

    emit FeesWithdrawn(recipient, amount);
  }

  /**
   * @notice Get the total amount of fees available for withdrawal
   * @return The total fees collected and available
   */
  function getAvailableFees() external view returns (uint256) {
    return totalFeesCollected;
  }

  /**
   * @notice Estimate the cost for a given token usage
   * @param promptTokens Estimated prompt tokens
   * @param completionTokens Estimated completion tokens
   * @return totalCost The total cost user would be charged
   */
  function estimateCost(uint256 promptTokens, uint256 completionTokens) external pure returns (uint256 totalCost) {
    (,, totalCost) = calculateCost(promptTokens, completionTokens);
  }

  /**
   * @notice Pay for a single analysis (pay-per-use model)
   * @dev User sends USDC payment before analysis. Frontend calculates cost based on estimated tokens.
   * @param estimatedPromptTokens Estimated prompt tokens (from contract size)
   * @param estimatedCompletionTokens Estimated completion tokens (fixed at 8000)
   * @return paymentId Unique payment ID for tracking
   */
  function payForAnalysis(
    uint256 estimatedPromptTokens,
    uint256 estimatedCompletionTokens
  ) external returns (bytes32 paymentId) {
    // Calculate required payment
    (,, uint256 totalCost) = calculateCost(estimatedPromptTokens, estimatedCompletionTokens);
    require(totalCost > 0, "Payment amount must be greater than zero");

    // Generate unique payment ID
    paymentId = keccak256(abi.encodePacked(msg.sender, block.timestamp, estimatedPromptTokens));
    require(payments[paymentId].user == address(0), "Payment ID collision");

    // Transfer USDC from user
    IERC20(asset()).safeTransferFrom(msg.sender, address(this), totalCost);

    // Store payment info
    payments[paymentId] = Payment({
      user: msg.sender,
      amount: totalCost,
      estimatedPromptTokens: estimatedPromptTokens,
      estimatedCompletionTokens: estimatedCompletionTokens,
      used: false,
      timestamp: block.timestamp
    });

    // Track as deducted cost (removed from vault's totalAssets)
    totalCostsDeducted += totalCost;

    // Track platform fee
    (, uint256 platformFee, ) = calculateCost(estimatedPromptTokens, estimatedCompletionTokens);
    totalFeesCollected += platformFee;

    emit PaymentReceived(paymentId, msg.sender, totalCost, estimatedPromptTokens, estimatedCompletionTokens);
  }

  /**
   * @notice Verify if a payment is valid and unused (for API validation)
   * @param paymentId The payment ID to verify
   * @return isValid True if payment exists and hasn't been used
   * @return user Address of the user who made the payment
   * @return amount Amount paid
   */
  function verifyPayment(bytes32 paymentId) external view returns (bool isValid, address user, uint256 amount) {
    Payment memory payment = payments[paymentId];
    isValid = payment.user != address(0) && !payment.used;
    user = payment.user;
    amount = payment.amount;
  }

  /**
   * @notice Mark payment as used after analysis is complete (operator only)
   * @param paymentId The payment ID to mark as used
   */
  function markPaymentUsed(bytes32 paymentId) external onlyRole(OPERATOR_ROLE) {
    require(payments[paymentId].user != address(0), "Payment does not exist");
    require(!payments[paymentId].used, "Payment already used");
    
    payments[paymentId].used = true;
    emit PaymentUsed(paymentId, payments[paymentId].user);
  }

  /**
   * @notice Get payment details
   * @param paymentId The payment ID to query
   * @return payment The payment struct
   */
  function getPayment(bytes32 paymentId) external view returns (Payment memory) {
    return payments[paymentId];
  }

  /**
   * @dev Override deposit to enforce minimum deposit requirement
   */
  function deposit(uint256 assets, address receiver) public virtual override returns (uint256) {
    require(assets >= MIN_DEPOSIT, "Deposit below minimum (1 USDC)");
    return super.deposit(assets, receiver);
  }

  /**
   * @dev Override mint to enforce minimum deposit requirement
   */
  function mint(uint256 shares, address receiver) public virtual override returns (uint256) {
    uint256 assets = previewMint(shares);
    require(assets >= MIN_DEPOSIT, "Deposit below minimum (1 USDC)");
    return super.mint(shares, receiver);
  }

  /**
   * @dev Override to add any custom withdrawal logic if needed
   */
  function withdraw(uint256 assets, address receiver, address owner) public virtual override returns (uint256) {
    return super.withdraw(assets, receiver, owner);
  }
}
