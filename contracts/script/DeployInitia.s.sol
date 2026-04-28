// SPDX-License-Identifier: MIT
pragma solidity 0.8.20;

import {Script} from "forge-std/Script.sol";
import {console} from "forge-std/console.sol";
import {RaxcCreditVault} from "../src/RaxcCreditVault.sol";
import {ERC20} from "openzeppelin/token/ERC20/ERC20.sol";
import {IERC20} from "openzeppelin/token/ERC20/IERC20.sol";

// Mock USDC for testing
contract MockUSDC is ERC20 {
    constructor() ERC20("USD Coin", "USDC") {
        _mint(msg.sender, 1_000_000 * 1e6); // Mint 1M USDC to deployer
    }

    function decimals() public pure override returns (uint8) {
        return 6;
    }

    function mint(address to, uint256 amount) external {
        _mint(to, amount);
    }
}

contract DeployInitia is Script {
    function run() external {
        uint256 deployerPrivateKey = vm.envUint("PRIVATE_KEY");
        address deployer = vm.addr(deployerPrivateKey);

        console.log("==============================================");
        console.log("Deploying to Initia");
        console.log("Deployer:", deployer);
        console.log("==============================================");

        vm.startBroadcast(deployerPrivateKey);

        // 1. Deploy MockUSDC
        MockUSDC usdc = new MockUSDC();
        console.log("\n1. MockUSDC deployed:", address(usdc));
        console.log("   Deployer USDC balance:", usdc.balanceOf(deployer) / 1e6, "USDC");

        // 2. Deploy RaxcCreditVault
        RaxcCreditVault vault = new RaxcCreditVault(
            IERC20(address(usdc)),
            "RAXC Credit Vault",
            "vRAXC"
        );
        console.log("\n2. RaxcCreditVault deployed:", address(vault));

        // 3. Grant OPERATOR_ROLE to deployer (for backend)
        bytes32 operatorRole = vault.OPERATOR_ROLE();
        vault.grantRole(operatorRole, deployer);
        console.log("\n3. OPERATOR_ROLE granted to:", deployer);

        vm.stopBroadcast();

        console.log("\n==============================================");
        console.log("DEPLOYMENT SUMMARY");
        console.log("==============================================");
        console.log("Chain: Initia Anvil");
        console.log("MockUSDC:", address(usdc));
        console.log("RaxcCreditVault:", address(vault));
        console.log("Operator:", deployer);
        console.log("\nSave these addresses for frontend/backend configuration!");
        console.log("==============================================");
    }
}
