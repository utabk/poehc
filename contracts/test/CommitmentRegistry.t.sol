// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import "forge-std/Test.sol";
import "../src/CommitmentRegistry.sol";

contract CommitmentRegistryTest is Test {
    CommitmentRegistry public registry;
    address public admin = address(1);
    address public user1 = address(2);
    address public user2 = address(3);
    address public client1 = address(4);

    function setUp() public {
        vm.prank(admin);
        registry = new CommitmentRegistry();
    }

    function _futureTime(uint64 offset) internal view returns (uint64) {
        return uint64(block.timestamp) + offset;
    }

    function test_register_slot() public {
        uint64 start = _futureTime(1 hours);
        uint64 end = _futureTime(4 hours);

        vm.prank(user1);
        uint256 slotId = registry.registerSlot(start, end, client1, CommitmentRegistry.CommitmentLevel.DEEP_FOCUS);

        assertEq(slotId, 1);

        (
            address owner,
            uint64 s,
            uint64 e,
            address client,
            CommitmentRegistry.CommitmentLevel level,
            ,
            CommitmentRegistry.SlotStatus status
        ) = registry.slots(slotId);

        assertEq(owner, user1);
        assertEq(s, start);
        assertEq(e, end);
        assertEq(client, client1);
        assertTrue(level == CommitmentRegistry.CommitmentLevel.DEEP_FOCUS);
        assertTrue(status == CommitmentRegistry.SlotStatus.PENDING);
    }

    function test_register_revert_end_before_start() public {
        vm.prank(user1);
        vm.expectRevert("CommitmentRegistry: end must be after start");
        registry.registerSlot(_futureTime(4 hours), _futureTime(1 hours), client1, CommitmentRegistry.CommitmentLevel.DEEP_FOCUS);
    }

    function test_register_revert_too_short() public {
        vm.prank(user1);
        vm.expectRevert("CommitmentRegistry: minimum 15 minute slot");
        registry.registerSlot(_futureTime(1 hours), _futureTime(1 hours + 10 minutes), client1, CommitmentRegistry.CommitmentLevel.DEEP_FOCUS);
    }

    function test_register_revert_too_long() public {
        vm.prank(user1);
        vm.expectRevert("CommitmentRegistry: maximum 12 hour slot");
        registry.registerSlot(_futureTime(1 hours), _futureTime(14 hours), client1, CommitmentRegistry.CommitmentLevel.DEEP_FOCUS);
    }

    function test_overlap_deep_focus_rejects() public {
        uint64 start1 = _futureTime(1 hours);
        uint64 end1 = _futureTime(4 hours);
        uint64 start2 = _futureTime(2 hours); // overlaps with first slot
        uint64 end2 = _futureTime(5 hours);

        vm.startPrank(user1);
        registry.registerSlot(start1, end1, client1, CommitmentRegistry.CommitmentLevel.DEEP_FOCUS);

        vm.expectRevert("CommitmentRegistry: overlapping slot exists");
        registry.registerSlot(start2, end2, client1, CommitmentRegistry.CommitmentLevel.DEEP_FOCUS);
        vm.stopPrank();
    }

    function test_overlap_active_rejects_deep() public {
        vm.startPrank(user1);
        registry.registerSlot(_futureTime(1 hours), _futureTime(4 hours), client1, CommitmentRegistry.CommitmentLevel.DEEP_FOCUS);

        vm.expectRevert("CommitmentRegistry: overlapping slot exists");
        registry.registerSlot(_futureTime(2 hours), _futureTime(5 hours), client1, CommitmentRegistry.CommitmentLevel.ACTIVE_ENGAGEMENT);
        vm.stopPrank();
    }

    function test_background_allows_background_overlap() public {
        vm.startPrank(user1);
        uint256 id1 = registry.registerSlot(
            _futureTime(1 hours), _futureTime(4 hours), client1,
            CommitmentRegistry.CommitmentLevel.BACKGROUND
        );
        uint256 id2 = registry.registerSlot(
            _futureTime(2 hours), _futureTime(5 hours), client1,
            CommitmentRegistry.CommitmentLevel.BACKGROUND
        );
        vm.stopPrank();

        assertEq(id1, 1);
        assertEq(id2, 2);
    }

    function test_background_rejects_deep_focus_overlap() public {
        vm.startPrank(user1);
        registry.registerSlot(
            _futureTime(1 hours), _futureTime(4 hours), client1,
            CommitmentRegistry.CommitmentLevel.DEEP_FOCUS
        );

        vm.expectRevert("CommitmentRegistry: overlapping slot exists");
        registry.registerSlot(
            _futureTime(2 hours), _futureTime(5 hours), client1,
            CommitmentRegistry.CommitmentLevel.BACKGROUND
        );
        vm.stopPrank();
    }

    function test_no_overlap_adjacent_slots() public {
        vm.startPrank(user1);
        uint256 id1 = registry.registerSlot(
            _futureTime(1 hours), _futureTime(4 hours), client1,
            CommitmentRegistry.CommitmentLevel.DEEP_FOCUS
        );
        // Adjacent (not overlapping) — end of first = start of second
        uint256 id2 = registry.registerSlot(
            _futureTime(4 hours), _futureTime(7 hours), client1,
            CommitmentRegistry.CommitmentLevel.DEEP_FOCUS
        );
        vm.stopPrank();

        assertEq(id1, 1);
        assertEq(id2, 2);
    }

    function test_cancel_slot() public {
        vm.prank(user1);
        uint256 slotId = registry.registerSlot(
            _futureTime(1 hours), _futureTime(4 hours), client1,
            CommitmentRegistry.CommitmentLevel.DEEP_FOCUS
        );

        vm.prank(user1);
        registry.cancelSlot(slotId);

        (, , , , , , CommitmentRegistry.SlotStatus status) = registry.slots(slotId);
        assertTrue(status == CommitmentRegistry.SlotStatus.CANCELLED);
    }

    function test_cancel_revert_not_owner() public {
        vm.prank(user1);
        uint256 slotId = registry.registerSlot(
            _futureTime(1 hours), _futureTime(4 hours), client1,
            CommitmentRegistry.CommitmentLevel.DEEP_FOCUS
        );

        vm.prank(user2);
        vm.expectRevert("CommitmentRegistry: not owner");
        registry.cancelSlot(slotId);
    }

    function test_cancel_revert_already_started() public {
        uint64 start = _futureTime(1 hours);
        vm.prank(user1);
        uint256 slotId = registry.registerSlot(
            start, _futureTime(4 hours), client1,
            CommitmentRegistry.CommitmentLevel.DEEP_FOCUS
        );

        // Warp past start time
        vm.warp(start + 1);

        vm.prank(user1);
        vm.expectRevert("CommitmentRegistry: already started");
        registry.cancelSlot(slotId);
    }

    function test_cancelled_slot_allows_new_registration() public {
        vm.startPrank(user1);
        uint256 slotId = registry.registerSlot(
            _futureTime(1 hours), _futureTime(4 hours), client1,
            CommitmentRegistry.CommitmentLevel.DEEP_FOCUS
        );
        registry.cancelSlot(slotId);

        // Should be able to register in the same time range now
        uint256 newSlotId = registry.registerSlot(
            _futureTime(1 hours), _futureTime(4 hours), client1,
            CommitmentRegistry.CommitmentLevel.DEEP_FOCUS
        );
        vm.stopPrank();

        assertEq(newSlotId, 2);
    }

    function test_get_active_slots() public {
        vm.startPrank(user1);
        registry.registerSlot(_futureTime(1 hours), _futureTime(4 hours), client1, CommitmentRegistry.CommitmentLevel.DEEP_FOCUS);
        uint256 slot2 = registry.registerSlot(_futureTime(5 hours), _futureTime(8 hours), client1, CommitmentRegistry.CommitmentLevel.DEEP_FOCUS);
        registry.registerSlot(_futureTime(9 hours), _futureTime(12 hours), client1, CommitmentRegistry.CommitmentLevel.DEEP_FOCUS);

        // Cancel the second slot
        registry.cancelSlot(slot2);
        vm.stopPrank();

        uint256[] memory active = registry.getActiveSlots(user1);
        assertEq(active.length, 2);
        assertEq(active[0], 1);
        assertEq(active[1], 3);
    }

    function test_level_multipliers() public view {
        assertEq(registry.getLevelMultiplier(CommitmentRegistry.CommitmentLevel.DEEP_FOCUS), 30000);
        assertEq(registry.getLevelMultiplier(CommitmentRegistry.CommitmentLevel.ACTIVE_ENGAGEMENT), 15000);
        assertEq(registry.getLevelMultiplier(CommitmentRegistry.CommitmentLevel.BACKGROUND), 10000);
    }

    function test_different_users_can_overlap() public {
        // Different users should be able to register overlapping times
        vm.prank(user1);
        registry.registerSlot(_futureTime(1 hours), _futureTime(4 hours), client1, CommitmentRegistry.CommitmentLevel.DEEP_FOCUS);

        vm.prank(user2);
        registry.registerSlot(_futureTime(2 hours), _futureTime(5 hours), client1, CommitmentRegistry.CommitmentLevel.DEEP_FOCUS);
    }
}
