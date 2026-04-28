// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "forge-std/Test.sol";
import "../src/RaxcCreditVault.sol";
import "openzeppelin/token/ERC20/ERC20.sol";

// Mock USDC token for testing
contract MockUSDC is ERC20 {
  constructor() ERC20("Mock USDC", "USDC") {
    _mint(msg.sender, 1_000_000 * 10 ** 6); // 1M USDC (6 decimals)
  }

  function decimals() public pure override returns (uint8) {
    return 6;
  }

  function mint(address to, uint256 amount) external {
    _mint(to, amount);
  }
}

contract RaxcCreditVaultTest is Test {
  RaxcCreditVault public vault;
  MockUSDC public usdc;

  address public admin = address(1);
  address public operator = address(2);
  address public user1 = address(3);
  address public user2 = address(4);

  uint256 constant USDC_DECIMALS = 6;
  uint256 constant USDC_UNIT = 10 ** USDC_DECIMALS;

  function setUp() public {
    // Deploy mock USDC
    usdc = new MockUSDC();

    // Deploy vault
    vm.prank(admin);
    vault = new RaxcCreditVault(IERC20(address(usdc)), "RAXC Credit Vault", "vRAXC");

    // Grant operator role
    vm.prank(admin);
    vault.addOperator(operator);

    // Fund users with USDC
    usdc.mint(user1, 10_000 * USDC_UNIT); // 10,000 USDC
    usdc.mint(user2, 5000 * USDC_UNIT); // 5,000 USDC
  }

  function testDeposit() public {
    uint256 depositAmount = 1000 * USDC_UNIT; // 1,000 USDC

    vm.startPrank(user1);
    usdc.approve(address(vault), depositAmount);
    vault.deposit(depositAmount, user1);
    vm.stopPrank();

    assertEq(vault.getCreditBalance(user1), depositAmount);
  }

  function testCalculateCost() public view {
    uint256 promptTokens = 1000;
    uint256 completionTokens = 500;

    (uint256 actualCost, uint256 platformFee, uint256 totalCost) = vault.calculateCost(promptTokens, completionTokens);

    // Expected: (1000 * 2.5/1M) + (500 * 10/1M) = 0.0025 + 0.005 = 0.0075 USDC
    // With 6 decimals: 7500 (0.0075 USDC)
    uint256 expectedActualCost = 7500;

    // Platform fee: 10% of actual cost = 750
    uint256 expectedPlatformFee = 750;

    // Total: 7500 + 750 = 8250
    uint256 expectedTotalCost = 8250;

    assertEq(actualCost, expectedActualCost, "Actual cost mismatch");
    assertEq(platformFee, expectedPlatformFee, "Platform fee mismatch");
    assertEq(totalCost, expectedTotalCost, "Total cost mismatch");
  }

  function testDeductCost() public {
    // User deposits 100 USDC
    uint256 depositAmount = 100 * USDC_UNIT;
    vm.startPrank(user1);
    usdc.approve(address(vault), depositAmount);
    vault.deposit(depositAmount, user1);
    vm.stopPrank();

    // Operator deducts cost for analysis
    uint256 promptTokens = 100_000; // 100k tokens
    uint256 completionTokens = 50_000; // 50k tokens

    uint256 balanceBefore = vault.getCreditBalance(user1);

    vm.prank(operator);
    vault.deductCost(user1, promptTokens, completionTokens);

    uint256 balanceAfter = vault.getCreditBalance(user1);

    (,, uint256 expectedDeduction) = vault.calculateCost(promptTokens, completionTokens);

    // Check that balance decreased
    assertLt(balanceAfter, balanceBefore, "Balance should decrease");
    
    // Check the deduction is approximately correct (allow for rounding)
    assertApproxEqAbs(
      balanceBefore - balanceAfter,
      expectedDeduction,
      100, // Allow small rounding difference due to share conversion
      "Balance deduction mismatch"
    );
  }

  function testDeductCostInsufficientBalance() public {
    // User deposits minimum 1 USDC
    uint256 depositAmount = 1 * USDC_UNIT; // 1 USDC
    vm.startPrank(user1);
    usdc.approve(address(vault), depositAmount);
    vault.deposit(depositAmount, user1);
    vm.stopPrank();

    // Try to deduct cost for large analysis that exceeds balance (should fail)
    uint256 promptTokens = 1_000_000; // 1M tokens
    uint256 completionTokens = 500_000; // 500k tokens

    vm.prank(operator);
    vm.expectRevert("Insufficient credit balance");
    vault.deductCost(user1, promptTokens, completionTokens);
  }

  function testOnlyOperatorCanDeduct() public {
    uint256 depositAmount = 100 * USDC_UNIT;
    vm.startPrank(user1);
    usdc.approve(address(vault), depositAmount);
    vault.deposit(depositAmount, user1);
    vm.stopPrank();

    // Non-operator tries to deduct (should fail)
    vm.prank(user2);
    vm.expectRevert();
    vault.deductCost(user1, 1000, 500);
  }

  function testWithdraw() public {
    uint256 depositAmount = 1000 * USDC_UNIT;

    vm.startPrank(user1);
    usdc.approve(address(vault), depositAmount);
    vault.deposit(depositAmount, user1);

    uint256 withdrawAmount = 500 * USDC_UNIT;
    uint256 usdcBefore = usdc.balanceOf(user1);
    vault.withdraw(withdrawAmount, user1, user1);
    uint256 usdcAfter = usdc.balanceOf(user1);
    vm.stopPrank();

    assertEq(usdcAfter - usdcBefore, withdrawAmount);
    assertEq(vault.getCreditBalance(user1), depositAmount - withdrawAmount);
  }

  function testFeeCollection() public {
    // User deposits
    uint256 depositAmount = 100 * USDC_UNIT;
    vm.startPrank(user1);
    usdc.approve(address(vault), depositAmount);
    vault.deposit(depositAmount, user1);
    vm.stopPrank();

    // Deduct cost (which includes platform fee)
    uint256 promptTokens = 100_000;
    uint256 completionTokens = 50_000;

    (, uint256 expectedFee,) = vault.calculateCost(promptTokens, completionTokens);

    vm.prank(operator);
    vault.deductCost(user1, promptTokens, completionTokens);

    assertApproxEqAbs(vault.getAvailableFees(), expectedFee, 10, "Fee collection mismatch");
  }

  function testWithdrawFees() public {
    // Setup: user deposits and incurs costs
    uint256 depositAmount = 100 * USDC_UNIT;
    vm.startPrank(user1);
    usdc.approve(address(vault), depositAmount);
    vault.deposit(depositAmount, user1);
    vm.stopPrank();

    vm.prank(operator);
    vault.deductCost(user1, 100_000, 50_000);

    uint256 availableFees = vault.getAvailableFees();
    address feeRecipient = address(5);

    // Admin withdraws fees
    vm.prank(admin);
    vault.withdrawFees(feeRecipient, availableFees);

    assertEq(usdc.balanceOf(feeRecipient), availableFees);
    assertEq(vault.getAvailableFees(), 0);
  }

  function testOnlyAdminCanWithdrawFees() public {
    vm.prank(user1);
    vm.expectRevert();
    vault.withdrawFees(user1, 100);
  }

  function testEstimateCost() public view {
    uint256 promptTokens = 50_000;
    uint256 completionTokens = 25_000;

    uint256 estimatedCost = vault.estimateCost(promptTokens, completionTokens);
    (,, uint256 calculatedCost) = vault.calculateCost(promptTokens, completionTokens);

    assertEq(estimatedCost, calculatedCost);
  }

  function testMultipleUsersIndependentBalances() public {
    // User1 deposits 1000 USDC
    uint256 deposit1 = 1000 * USDC_UNIT;
    vm.startPrank(user1);
    usdc.approve(address(vault), deposit1);
    vault.deposit(deposit1, user1);
    vm.stopPrank();

    // User2 deposits 500 USDC
    uint256 deposit2 = 500 * USDC_UNIT;
    vm.startPrank(user2);
    usdc.approve(address(vault), deposit2);
    vault.deposit(deposit2, user2);
    vm.stopPrank();

    uint256 user2BalanceBefore = vault.getCreditBalance(user2);

    // Deduct from user1
    vm.prank(operator);
    vault.deductCost(user1, 100_000, 50_000);

    // User2's balance should remain approximately the same
    // (may increase slightly due to ERC4626 mechanics where burned shares increase value per share)
    uint256 user2BalanceAfter = vault.getCreditBalance(user2);
    assertGe(user2BalanceAfter, user2BalanceBefore, "User2 balance should not decrease");
    assertApproxEqAbs(user2BalanceAfter, deposit2, deposit2 / 1000, "User2 balance should be approximately unchanged");

    // User1's balance should be reduced
    assertLt(vault.getCreditBalance(user1), deposit1);
  }

  function testAddRemoveOperator() public {
    address newOperator = address(6);

    // Add operator
    vm.prank(admin);
    vault.addOperator(newOperator);

    assertTrue(vault.hasRole(vault.OPERATOR_ROLE(), newOperator));

    // Remove operator
    vm.prank(admin);
    vault.removeOperator(newOperator);

    assertFalse(vault.hasRole(vault.OPERATOR_ROLE(), newOperator));
  }

  function testPayForAnalysis() public {
    uint256 estimatedPromptTokens = 50000; // 50k tokens
    uint256 estimatedCompletionTokens = 8000; // 8k tokens (fixed)

    // Calculate expected cost
    (,, uint256 expectedCost) = vault.calculateCost(estimatedPromptTokens, estimatedCompletionTokens);

    // User approves and pays for analysis
    vm.startPrank(user1);
    usdc.approve(address(vault), expectedCost);
    bytes32 paymentId = vault.payForAnalysis(estimatedPromptTokens, estimatedCompletionTokens);
    vm.stopPrank();

    // Verify payment exists and is valid
    (bool isValid, address user, uint256 amount) = vault.verifyPayment(paymentId);
    assertTrue(isValid, "Payment should be valid");
    assertEq(user, user1, "User mismatch");
    assertEq(amount, expectedCost, "Amount mismatch");

    // Get payment details
    RaxcCreditVault.Payment memory payment = vault.getPayment(paymentId);
    assertEq(payment.user, user1);
    assertEq(payment.amount, expectedCost);
    assertEq(payment.estimatedPromptTokens, estimatedPromptTokens);
    assertEq(payment.estimatedCompletionTokens, estimatedCompletionTokens);
    assertFalse(payment.used);
  }

  function testMarkPaymentUsed() public {
    uint256 estimatedPromptTokens = 50000;
    uint256 estimatedCompletionTokens = 8000;

    vm.startPrank(user1);
    (,, uint256 cost) = vault.calculateCost(estimatedPromptTokens, estimatedCompletionTokens);
    usdc.approve(address(vault), cost);
    bytes32 paymentId = vault.payForAnalysis(estimatedPromptTokens, estimatedCompletionTokens);
    vm.stopPrank();

    // Operator marks payment as used
    vm.prank(operator);
    vault.markPaymentUsed(paymentId);

    // Verify payment is now marked as used
    (bool isValid, , ) = vault.verifyPayment(paymentId);
    assertFalse(isValid, "Payment should be invalid after use");

    RaxcCreditVault.Payment memory payment = vault.getPayment(paymentId);
    assertTrue(payment.used, "Payment should be marked as used");
  }

  function testPayForAnalysisBelowMinimum() public {
    // Try to pay with zero tokens (should fail with zero amount)
    uint256 zeroTokens = 0;

    vm.startPrank(user1);
    vm.expectRevert("Payment amount must be greater than zero");
    vault.payForAnalysis(zeroTokens, zeroTokens);
    vm.stopPrank();
  }

  function testOnlyOperatorCanMarkPaymentUsed() public {
    vm.startPrank(user1);
    (,, uint256 cost) = vault.calculateCost(50000, 8000);
    usdc.approve(address(vault), cost);
    bytes32 paymentId = vault.payForAnalysis(50000, 8000);
    vm.stopPrank();

    // Non-operator tries to mark as used
    vm.prank(user2);
    vm.expectRevert();
    vault.markPaymentUsed(paymentId);
  }

  function testMinimumDepositEnforced() public {
    uint256 tooSmall = (5 * USDC_UNIT) / 10; // 0.5 USDC (below 1 USDC minimum)

    vm.startPrank(user1);
    usdc.approve(address(vault), tooSmall);
    vm.expectRevert("Deposit below minimum (1 USDC)");
    vault.deposit(tooSmall, user1);
    vm.stopPrank();
  }

  function testMinimumDepositAllowed() public {
    uint256 exactMinimum = 1 * USDC_UNIT; // Exactly 1 USDC

    vm.startPrank(user1);
    usdc.approve(address(vault), exactMinimum);
    vault.deposit(exactMinimum, user1);
    vm.stopPrank();

    assertEq(vault.getCreditBalance(user1), exactMinimum);
  }
}

