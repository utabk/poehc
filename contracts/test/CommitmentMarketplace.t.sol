// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import "forge-std/Test.sol";
import "../src/TIMEToken.sol";
import "../src/CommitmentRegistry.sol";
import "../src/EngagementVerifier.sol";
import "../src/CommitmentMarketplace.sol";

contract CommitmentMarketplaceTest is Test {
    TIMEToken public token;
    CommitmentRegistry public registry;
    EngagementVerifier public verifier;
    CommitmentMarketplace public marketplace;

    address public admin = address(1);
    address public client1 = address(2);
    address public worker1 = address(3);
    address public feeRecipient = address(4);

    function setUp() public {
        vm.startPrank(admin);

        token = new TIMEToken(admin);
        registry = new CommitmentRegistry();
        verifier = new EngagementVerifier(address(registry), address(token));
        marketplace = new CommitmentMarketplace(
            address(registry), address(verifier), address(token), feeRecipient
        );

        // Setup roles
        token.grantRole(token.MINTER_ROLE(), admin);
        token.grantRole(token.MINTER_ROLE(), address(verifier));
        registry.setAuthorizedVerifier(address(verifier), true);

        // Fund client
        token.mint(client1, 10000e18);

        vm.stopPrank();

        // Client approves marketplace to spend tokens
        vm.prank(client1);
        token.approve(address(marketplace), type(uint256).max);
    }

    function _futureTime(uint64 offset) internal view returns (uint64) {
        return uint64(block.timestamp) + offset;
    }

    /// @dev Helper: worker registers slot and accepts contract
    function _workerAccept(
        uint256 contractId,
        uint64 start,
        uint64 end,
        CommitmentRegistry.CommitmentLevel level
    ) internal returns (uint256 slotId) {
        vm.startPrank(worker1);
        slotId = registry.registerSlot(start, end, client1, level);
        marketplace.acceptContract(contractId, slotId);
        vm.stopPrank();
    }

    function test_create_contract() public {
        vm.prank(client1);
        uint256 contractId = marketplace.createContract(
            100e18,
            CommitmentRegistry.CommitmentLevel.DEEP_FOCUS,
            _futureTime(1 hours),
            _futureTime(4 hours)
        );

        assertEq(contractId, 1);

        (
            address client,
            address worker,
            uint256 budget,
            ,
            ,
            ,
            ,
            CommitmentMarketplace.ContractStatus status
        ) = marketplace.contracts(contractId);

        assertEq(client, client1);
        assertEq(worker, address(0));
        assertEq(budget, 100e18);
        assertTrue(status == CommitmentMarketplace.ContractStatus.OPEN);
        assertEq(token.balanceOf(address(marketplace)), 100e18);
    }

    function test_accept_contract() public {
        uint64 start = _futureTime(1 hours);
        uint64 end = _futureTime(4 hours);

        vm.prank(client1);
        uint256 contractId = marketplace.createContract(
            100e18,
            CommitmentRegistry.CommitmentLevel.DEEP_FOCUS,
            start,
            end
        );

        uint256 slotId = _workerAccept(contractId, start, end, CommitmentRegistry.CommitmentLevel.DEEP_FOCUS);

        (
            ,
            address worker,
            ,
            ,
            ,
            ,
            uint256 storedSlotId,
            CommitmentMarketplace.ContractStatus status
        ) = marketplace.contracts(contractId);

        assertEq(worker, worker1);
        assertTrue(status == CommitmentMarketplace.ContractStatus.ACCEPTED);
        assertEq(storedSlotId, slotId);
    }

    function test_cancel_contract() public {
        uint256 balanceBefore = token.balanceOf(client1);

        vm.prank(client1);
        uint256 contractId = marketplace.createContract(
            100e18,
            CommitmentRegistry.CommitmentLevel.DEEP_FOCUS,
            _futureTime(1 hours),
            _futureTime(4 hours)
        );

        vm.prank(client1);
        marketplace.cancelContract(contractId);

        assertEq(token.balanceOf(client1), balanceBefore);
    }

    function test_cancel_revert_not_client() public {
        vm.prank(client1);
        uint256 contractId = marketplace.createContract(
            100e18,
            CommitmentRegistry.CommitmentLevel.DEEP_FOCUS,
            _futureTime(1 hours),
            _futureTime(4 hours)
        );

        vm.prank(worker1);
        vm.expectRevert("Marketplace: not client");
        marketplace.cancelContract(contractId);
    }

    function test_full_flow_create_accept_settle() public {
        uint64 start = _futureTime(1 hours);
        uint64 end = _futureTime(4 hours);

        // Client creates contract
        vm.prank(client1);
        uint256 contractId = marketplace.createContract(
            100e18,
            CommitmentRegistry.CommitmentLevel.DEEP_FOCUS,
            start,
            end
        );

        // Worker registers slot and accepts contract
        uint256 slotId = _workerAccept(contractId, start, end, CommitmentRegistry.CommitmentLevel.DEEP_FOCUS);

        // Warp to during slot and submit proofs
        vm.warp(start + 30 minutes);

        vm.startPrank(worker1);
        verifier.submitProof(slotId, keccak256("p1"), 9, 10);
        verifier.submitProof(slotId, keccak256("p2"), 10, 10);
        verifier.submitProof(slotId, keccak256("p3"), 9, 10);
        vm.stopPrank();

        // Warp past end, finalize
        vm.warp(end + 1);
        verifier.finalizeSlot(slotId);

        // Score: 28/30 = 9333 bps
        uint256 score = verifier.getVerificationScore(slotId);
        assertEq(score, 9333);

        // Settle
        marketplace.settleContract(contractId);

        // Verify fee recipient got protocol fee
        uint256 grossPayment = (100e18 * score) / 10000;
        uint256 protocolFee = (grossPayment * 100) / 10000;
        assertEq(token.balanceOf(feeRecipient), protocolFee);

        // Marketplace should be empty
        assertEq(token.balanceOf(address(marketplace)), 0);
    }

    function test_dispute_contract() public {
        uint64 start = _futureTime(1 hours);
        uint64 end = _futureTime(4 hours);

        vm.prank(client1);
        uint256 contractId = marketplace.createContract(
            100e18,
            CommitmentRegistry.CommitmentLevel.DEEP_FOCUS,
            start,
            end
        );

        _workerAccept(contractId, start, end, CommitmentRegistry.CommitmentLevel.DEEP_FOCUS);

        vm.prank(client1);
        marketplace.disputeContract(contractId);

        (, , , , , , , CommitmentMarketplace.ContractStatus status) = marketplace.contracts(contractId);
        assertTrue(status == CommitmentMarketplace.ContractStatus.DISPUTED);
    }
}
