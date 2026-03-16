// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import "forge-std/Script.sol";
import "../src/TIMEToken.sol";
import "../src/CommitmentRegistry.sol";
import "../src/EngagementVerifier.sol";
import "../src/ValidatorRegistry.sol";
import "../src/CommitmentMarketplace.sol";

/// @notice Deploys all PoEHC contracts and wires permissions.
contract Deploy is Script {
    function run() external {
        uint256 deployerPrivateKey = vm.envOr("PRIVATE_KEY", uint256(0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80));
        address deployer = vm.addr(deployerPrivateKey);

        vm.startBroadcast(deployerPrivateKey);

        // 1. Deploy TIMEToken
        TIMEToken timeToken = new TIMEToken(deployer);
        console.log("TIMEToken deployed at:", address(timeToken));

        // 2. Deploy CommitmentRegistry
        CommitmentRegistry registry = new CommitmentRegistry();
        console.log("CommitmentRegistry deployed at:", address(registry));

        // 3. Deploy EngagementVerifier
        EngagementVerifier verifier = new EngagementVerifier(address(registry), address(timeToken));
        console.log("EngagementVerifier deployed at:", address(verifier));

        // 4. Deploy ValidatorRegistry
        ValidatorRegistry validatorReg = new ValidatorRegistry(address(timeToken));
        console.log("ValidatorRegistry deployed at:", address(validatorReg));

        // 5. Deploy CommitmentMarketplace
        CommitmentMarketplace marketplace = new CommitmentMarketplace(
            address(registry),
            address(verifier),
            address(timeToken),
            deployer // fee recipient = deployer for now
        );
        console.log("CommitmentMarketplace deployed at:", address(marketplace));

        // 6. Wire permissions
        // EngagementVerifier can mint TIME
        timeToken.grantRole(timeToken.MINTER_ROLE(), address(verifier));
        console.log("Granted MINTER_ROLE to EngagementVerifier");

        // ValidatorRegistry can mint TIME (for rewards)
        timeToken.grantRole(timeToken.MINTER_ROLE(), address(validatorReg));
        console.log("Granted MINTER_ROLE to ValidatorRegistry");

        // EngagementVerifier can update slot status
        registry.setAuthorizedVerifier(address(verifier), true);
        console.log("Authorized EngagementVerifier as verifier in CommitmentRegistry");

        vm.stopBroadcast();

        console.log("---");
        console.log("Deployment complete. All contracts deployed and permissions wired.");
    }
}
