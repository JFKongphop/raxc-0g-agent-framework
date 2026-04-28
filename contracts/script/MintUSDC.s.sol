// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import {Script} from "forge-std/Script.sol";
import {console} from "forge-std/console.sol";
import {MockUSDC} from "../src/MockUSDC.sol";

contract MintUSDC is Script {
    function run() external {
        uint256 deployerPrivateKey = vm.envUint("PRIVATE_KEY");
        address usdcAddress = 0x8A2cCc6764063904745BBA687AF11190f7a50461; // From frontend .env
        address userAddress = 0x874604c87A1FEF538Ce21192aac0Db131F5F24ae; // Your wallet

        vm.startBroadcast(deployerPrivateKey);

        MockUSDC usdc = MockUSDC(usdcAddress);
        
        // Check current balance
        uint256 currentBalance = usdc.balanceOf(userAddress);
        console.log("Current USDC balance:", currentBalance / 10**6, "USDC");
        
        // Mint 1000 USDC if balance is low
        if (currentBalance < 10 * 10**6) {
            uint256 mintAmount = 1000 * 10**6;
            console.log("Minting", mintAmount / 10**6, "USDC to", userAddress);
            usdc.mint(userAddress, mintAmount);
            
            uint256 newBalance = usdc.balanceOf(userAddress);
            console.log("New balance:", newBalance / 10**6, "USDC");
        } else {
            console.log("Balance is sufficient!");
        }

        vm.stopBroadcast();
    }
}
