// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import {Script, console} from "forge-std/Script.sol";
import {RaxcAuditTask8183} from "../src/RaxcAuditTask8183.sol";

contract DeployRaxcAuditTask8183 is Script {
  function run() external {
    vm.startBroadcast();

    RaxcAuditTask8183 auditTask = new RaxcAuditTask8183();

    console.log("RaxcAuditTask8183 deployed at:", address(auditTask));

    vm.stopBroadcast();
  }
}
