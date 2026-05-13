// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import {Script, console} from "forge-std/Script.sol";
import {RaxcAgentNFT, IntelligentData} from "../src/RaxcAgentNFT.sol";

contract DeployRaxcAgentNFT is Script {
  function run() external {
    vm.startBroadcast();

    RaxcAgentNFT nft = new RaxcAgentNFT();

    // Mint token 0 to deployer with a placeholder initial entry
    IntelligentData[] memory initial = new IntelligentData[](1);
    initial[0] = IntelligentData({
      dataDescription: "RAXC Agent Genesis",
      dataHash: bytes32(uint256(1))
    });
    nft.mint(initial, msg.sender, msg.sender);

    console.log("RaxcAgentNFT deployed at:", address(nft));
    console.log("Token 0 minted to:", msg.sender);

    vm.stopBroadcast();
  }
}
