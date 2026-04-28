// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import {Script} from "forge-std/Script.sol";
import {console} from "forge-std/console.sol";
import {MockUSDC} from "../src/MockUSDC.sol";

contract TransferUSDC is Script {
    function run() external {
        uint256 deployerPrivateKey = vm.envUint("PRIVATE_KEY");
        address recipient = vm.envAddress("RECIPIENT_ADDRESS");
        address usdcAddress = vm.envAddress("USDC_ADDRESS");

        vm.startBroadcast(deployerPrivateKey);

        MockUSDC usdc = MockUSDC(usdcAddress);
        
        // Transfer 100 USDC (100 * 10^6 since USDC has 6 decimals)
        uint256 amount = 100 * 10**6;
        
        console.log("Transferring USDC from deployer to recipient...");
        console.log("From:", msg.sender);
        console.log("To:", recipient);
        console.log("Amount:", amount / 10**6, "USDC");
        
        usdc.transfer(recipient, amount);
        
        uint256 balance = usdc.balanceOf(recipient);
        console.log("New balance:", balance / 10**6, "USDC");

        vm.stopBroadcast();
    }
}
