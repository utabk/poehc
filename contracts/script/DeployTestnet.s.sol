// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import "forge-std/Script.sol";
import "../src/TIMEToken.sol";
import "../src/CommitmentRegistry.sol";
import "../src/EngagementVerifier.sol";
import "../src/ValidatorRegistry.sol";
import "../src/CommitmentMarketplace.sol";

/// @notice Deploys all PoEHC contracts to a testnet (Base Sepolia, Arbitrum Sepolia, etc.)
/// Usage:
///   export PRIVATE_KEY=0x...
///   forge script script/DeployTestnet.s.sol --broadcast --rpc-url $RPC_URL --verify
contract DeployTestnet is Script {
    function run() external {
        uint256 deployerPrivateKey = vm.envUint("PRIVATE_KEY");
        address deployer = vm.addr(deployerPrivateKey);

        console.log("Deployer:", deployer);
        console.log("Chain ID:", block.chainid);

        vm.startBroadcast(deployerPrivateKey);

        // 1. Deploy TIMEToken
        TIMEToken timeToken = new TIMEToken(deployer);
        console.log("TIMEToken:", address(timeToken));

        // 2. Deploy CommitmentRegistry
        CommitmentRegistry registry = new CommitmentRegistry();
        console.log("CommitmentRegistry:", address(registry));

        // 3. Deploy EngagementVerifier
        EngagementVerifier verifier = new EngagementVerifier(address(registry), address(timeToken));
        console.log("EngagementVerifier:", address(verifier));

        // 4. Deploy ValidatorRegistry
        ValidatorRegistry validatorReg = new ValidatorRegistry(address(timeToken));
        console.log("ValidatorRegistry:", address(validatorReg));

        // 5. Deploy Marketplace
        CommitmentMarketplace marketplace = new CommitmentMarketplace(
            address(registry), address(verifier), address(timeToken), deployer
        );
        console.log("CommitmentMarketplace:", address(marketplace));

        // 6. Wire permissions
        timeToken.grantRole(timeToken.MINTER_ROLE(), address(verifier));
        timeToken.grantRole(timeToken.MINTER_ROLE(), address(validatorReg));
        registry.setAuthorizedVerifier(address(verifier), true);

        vm.stopBroadcast();

        // Output deployment summary for config files
        console.log("");
        console.log("=== DEPLOYMENT COMPLETE ===");
        console.log("Network:", block.chainid);
        console.log("");
        console.log("--- Copy to web/src/config/wagmi.ts ---");
        console.log("timeToken:", address(timeToken));
        console.log("commitmentRegistry:", address(registry));
        console.log("engagementVerifier:", address(verifier));
        console.log("validatorRegistry:", address(validatorReg));
        console.log("marketplace:", address(marketplace));
    }
}
