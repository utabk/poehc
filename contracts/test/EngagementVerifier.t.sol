// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import "forge-std/Test.sol";
import "../src/TIMEToken.sol";
import "../src/CommitmentRegistry.sol";
import "../src/EngagementVerifier.sol";

contract EngagementVerifierTest is Test {
    TIMEToken public token;
    CommitmentRegistry public registry;
    EngagementVerifier public verifier;

    address public admin = address(1);
    address public user1 = address(2);
    address public client1 = address(3);

    function setUp() public {
        vm.startPrank(admin);

        token = new TIMEToken(admin);
        registry = new CommitmentRegistry();
        verifier = new EngagementVerifier(address(registry), address(token));

        // Grant MINTER_ROLE to verifier
        token.grantRole(token.MINTER_ROLE(), address(verifier));

        // Authorize verifier to update slot status
        registry.setAuthorizedVerifier(address(verifier), true);

        vm.stopPrank();
    }

    function _registerSlot(
        address user,
        uint64 start,
        uint64 end,
        CommitmentRegistry.CommitmentLevel level
    ) internal returns (uint256) {
        vm.prank(user);
        return registry.registerSlot(start, end, client1, level);
    }

    function _futureTime(uint64 offset) internal view returns (uint64) {
        return uint64(block.timestamp) + offset;
    }

    function test_submit_proof() public {
        uint64 start = _futureTime(1 hours);
        uint64 end = _futureTime(4 hours);
        uint256 slotId = _registerSlot(user1, start, end, CommitmentRegistry.CommitmentLevel.DEEP_FOCUS);

        // Warp to during slot
        vm.warp(start + 30 minutes);

        vm.prank(user1);
        verifier.submitProof(slotId, keccak256("proof1"), 9, 10);

        assertEq(verifier.getProofCount(slotId), 1);
    }

    function test_submit_proof_revert_not_owner() public {
        uint64 start = _futureTime(1 hours);
        uint64 end = _futureTime(4 hours);
        uint256 slotId = _registerSlot(user1, start, end, CommitmentRegistry.CommitmentLevel.DEEP_FOCUS);

        vm.warp(start + 30 minutes);

        vm.prank(admin); // wrong user
        vm.expectRevert("EngagementVerifier: not slot owner");
        verifier.submitProof(slotId, keccak256("proof1"), 9, 10);
    }

    function test_submit_proof_revert_before_start() public {
        uint64 start = _futureTime(1 hours);
        uint64 end = _futureTime(4 hours);
        uint256 slotId = _registerSlot(user1, start, end, CommitmentRegistry.CommitmentLevel.DEEP_FOCUS);

        // Don't warp — still before start
        vm.prank(user1);
        vm.expectRevert("EngagementVerifier: slot not started");
        verifier.submitProof(slotId, keccak256("proof1"), 9, 10);
    }

    function test_verification_score() public {
        uint64 start = _futureTime(1 hours);
        uint64 end = _futureTime(4 hours);
        uint256 slotId = _registerSlot(user1, start, end, CommitmentRegistry.CommitmentLevel.DEEP_FOCUS);

        vm.warp(start + 30 minutes);

        vm.startPrank(user1);
        verifier.submitProof(slotId, keccak256("p1"), 8, 10);  // 80%
        verifier.submitProof(slotId, keccak256("p2"), 10, 10); // 100%
        verifier.submitProof(slotId, keccak256("p3"), 9, 10);  // 90%
        vm.stopPrank();

        // Aggregate: 27/30 = 9000 bps = 90%
        uint256 score = verifier.getVerificationScore(slotId);
        assertEq(score, 9000);
    }

    function test_finalize_deep_focus_perfect_score() public {
        // 3 hours of Deep Focus with perfect score should earn 9 TIME
        uint64 start = _futureTime(1 hours);
        uint64 end = _futureTime(4 hours); // 3 hour slot
        uint256 slotId = _registerSlot(user1, start, end, CommitmentRegistry.CommitmentLevel.DEEP_FOCUS);

        vm.warp(start + 30 minutes);

        vm.startPrank(user1);
        verifier.submitProof(slotId, keccak256("p1"), 10, 10);
        verifier.submitProof(slotId, keccak256("p2"), 10, 10);
        verifier.submitProof(slotId, keccak256("p3"), 10, 10);
        vm.stopPrank();

        // Warp past slot end
        vm.warp(end + 1);

        verifier.finalizeSlot(slotId);

        // 3 hours * 3.0x * 1.0 = 9 TIME
        // durationSeconds = 3 * 3600 = 10800
        // multiplier = 30000 bps
        // score = 10000 bps
        // amount = (10800 * 30000 * 10000 * 1e18) / (3600 * 10000 * 10000)
        //        = (10800 * 30000 * 1e18) / (3600 * 10000)
        //        = 9 * 1e18
        assertEq(token.balanceOf(user1), 9e18);
    }

    function test_finalize_active_engagement() public {
        // 2 hours of Active Engagement with 80% score
        uint64 start = _futureTime(1 hours);
        uint64 end = _futureTime(3 hours); // 2 hour slot
        uint256 slotId = _registerSlot(user1, start, end, CommitmentRegistry.CommitmentLevel.ACTIVE_ENGAGEMENT);

        vm.warp(start + 30 minutes);

        vm.startPrank(user1);
        verifier.submitProof(slotId, keccak256("p1"), 8, 10);
        verifier.submitProof(slotId, keccak256("p2"), 8, 10);
        verifier.submitProof(slotId, keccak256("p3"), 8, 10);
        vm.stopPrank();

        vm.warp(end + 1);
        verifier.finalizeSlot(slotId);

        // 2 hours * 1.5x * 0.8 = 2.4 TIME
        // durationSeconds = 7200
        // multiplier = 15000
        // score = 8000
        // amount = (7200 * 15000 * 8000 * 1e18) / (3600 * 10000 * 10000)
        //        = (7200 * 15000 * 8000 * 1e18) / (360000000000)
        //        = 2.4e18
        assertEq(token.balanceOf(user1), 24e17);
    }

    function test_finalize_revert_before_end() public {
        uint64 start = _futureTime(1 hours);
        uint64 end = _futureTime(4 hours);
        uint256 slotId = _registerSlot(user1, start, end, CommitmentRegistry.CommitmentLevel.DEEP_FOCUS);

        vm.warp(start + 30 minutes);

        vm.startPrank(user1);
        verifier.submitProof(slotId, keccak256("p1"), 10, 10);
        verifier.submitProof(slotId, keccak256("p2"), 10, 10);
        verifier.submitProof(slotId, keccak256("p3"), 10, 10);
        vm.stopPrank();

        // Still during slot
        vm.expectRevert("EngagementVerifier: slot not ended");
        verifier.finalizeSlot(slotId);
    }

    function test_finalize_revert_insufficient_proofs() public {
        uint64 start = _futureTime(1 hours);
        uint64 end = _futureTime(4 hours);
        uint256 slotId = _registerSlot(user1, start, end, CommitmentRegistry.CommitmentLevel.DEEP_FOCUS);

        vm.warp(start + 30 minutes);

        vm.prank(user1);
        verifier.submitProof(slotId, keccak256("p1"), 10, 10);
        // Only 1 proof, need MIN_PROOFS (3)

        vm.warp(end + 1);

        vm.expectRevert("EngagementVerifier: insufficient proofs");
        verifier.finalizeSlot(slotId);
    }

    function test_finalize_revert_score_too_low() public {
        uint64 start = _futureTime(1 hours);
        uint64 end = _futureTime(4 hours);
        uint256 slotId = _registerSlot(user1, start, end, CommitmentRegistry.CommitmentLevel.DEEP_FOCUS);

        vm.warp(start + 30 minutes);

        vm.startPrank(user1);
        verifier.submitProof(slotId, keccak256("p1"), 1, 10); // 10%
        verifier.submitProof(slotId, keccak256("p2"), 2, 10); // 20%
        verifier.submitProof(slotId, keccak256("p3"), 3, 10); // 30%
        vm.stopPrank();
        // Aggregate: 6/30 = 2000 bps = 20%, below MIN_SCORE_BPS (3000)

        vm.warp(end + 1);

        vm.expectRevert("EngagementVerifier: score too low");
        verifier.finalizeSlot(slotId);
    }

    function test_finalize_revert_double_finalize() public {
        uint64 start = _futureTime(1 hours);
        uint64 end = _futureTime(4 hours);
        uint256 slotId = _registerSlot(user1, start, end, CommitmentRegistry.CommitmentLevel.DEEP_FOCUS);

        vm.warp(start + 30 minutes);

        vm.startPrank(user1);
        verifier.submitProof(slotId, keccak256("p1"), 10, 10);
        verifier.submitProof(slotId, keccak256("p2"), 10, 10);
        verifier.submitProof(slotId, keccak256("p3"), 10, 10);
        vm.stopPrank();

        vm.warp(end + 1);

        verifier.finalizeSlot(slotId);

        vm.expectRevert("EngagementVerifier: slot not active");
        verifier.finalizeSlot(slotId);
    }

    function test_slot_status_updated_on_finalize() public {
        uint64 start = _futureTime(1 hours);
        uint64 end = _futureTime(4 hours);
        uint256 slotId = _registerSlot(user1, start, end, CommitmentRegistry.CommitmentLevel.DEEP_FOCUS);

        vm.warp(start + 30 minutes);

        vm.startPrank(user1);
        verifier.submitProof(slotId, keccak256("p1"), 10, 10);
        verifier.submitProof(slotId, keccak256("p2"), 10, 10);
        verifier.submitProof(slotId, keccak256("p3"), 10, 10);
        vm.stopPrank();

        vm.warp(end + 1);
        verifier.finalizeSlot(slotId);

        (, , , , , , CommitmentRegistry.SlotStatus status) = registry.slots(slotId);
        assertTrue(status == CommitmentRegistry.SlotStatus.VERIFIED);
    }
}
