// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import "forge-std/Test.sol";
import "../src/TIMEToken.sol";

contract TIMETokenTest is Test {
    TIMEToken public token;
    address public admin = address(1);
    address public minter = address(2);
    address public slasher = address(3);
    address public user1 = address(4);
    address public user2 = address(5);

    function setUp() public {
        vm.prank(admin);
        token = new TIMEToken(admin);

        vm.startPrank(admin);
        token.grantRole(token.MINTER_ROLE(), minter);
        token.grantRole(token.SLASHER_ROLE(), slasher);
        vm.stopPrank();
    }

    function test_name_and_symbol() public view {
        assertEq(token.name(), "TIME");
        assertEq(token.symbol(), "TIME");
    }

    function test_mint_with_minter_role() public {
        vm.prank(minter);
        token.mint(user1, 100e18);
        assertEq(token.balanceOf(user1), 100e18);
    }

    function test_mint_revert_without_role() public {
        vm.prank(user1);
        vm.expectRevert();
        token.mint(user1, 100e18);
    }

    function test_mint_revert_exceeds_max_supply() public {
        // Mint up to max supply first
        vm.startPrank(minter);
        token.mint(user1, token.MAX_SUPPLY());

        // Next mint should revert
        vm.expectRevert("TIMEToken: max supply exceeded");
        token.mint(user1, 1);
        vm.stopPrank();
    }

    function test_stake() public {
        vm.prank(minter);
        token.mint(user1, 1000e18);

        vm.prank(user1);
        token.stake(500e18);

        assertEq(token.stakedBalance(user1), 500e18);
        assertEq(token.balanceOf(user1), 500e18);
    }

    function test_stake_revert_insufficient_balance() public {
        vm.prank(user1);
        vm.expectRevert("TIMEToken: insufficient balance");
        token.stake(100e18);
    }

    function test_stake_revert_zero() public {
        vm.prank(user1);
        vm.expectRevert("TIMEToken: cannot stake zero");
        token.stake(0);
    }

    function test_request_unstake() public {
        vm.prank(minter);
        token.mint(user1, 1000e18);

        vm.prank(user1);
        token.stake(500e18);

        vm.prank(user1);
        token.requestUnstake(200e18);

        assertEq(token.unstakePendingAmount(user1), 200e18);
        assertGt(token.unstakeRequestedAt(user1), 0);
    }

    function test_complete_unstake_after_cooldown() public {
        vm.prank(minter);
        token.mint(user1, 1000e18);

        vm.prank(user1);
        token.stake(500e18);

        vm.prank(user1);
        token.requestUnstake(200e18);

        // Warp past cooldown
        vm.warp(block.timestamp + 7 days + 1);

        vm.prank(user1);
        token.completeUnstake();

        assertEq(token.stakedBalance(user1), 300e18);
        assertEq(token.balanceOf(user1), 700e18);
        assertEq(token.unstakeRequestedAt(user1), 0);
        assertEq(token.unstakePendingAmount(user1), 0);
    }

    function test_complete_unstake_revert_before_cooldown() public {
        vm.prank(minter);
        token.mint(user1, 1000e18);

        vm.prank(user1);
        token.stake(500e18);

        vm.prank(user1);
        token.requestUnstake(200e18);

        vm.prank(user1);
        vm.expectRevert("TIMEToken: cooldown not elapsed");
        token.completeUnstake();
    }

    function test_slash() public {
        vm.prank(minter);
        token.mint(user1, 1000e18);

        vm.prank(user1);
        token.stake(500e18);

        uint256 totalBefore = token.totalSupply();

        vm.prank(slasher);
        token.slash(user1, 100e18);

        assertEq(token.stakedBalance(user1), 400e18);
        assertEq(token.totalSupply(), totalBefore - 100e18);
    }

    function test_slash_revert_without_role() public {
        vm.prank(minter);
        token.mint(user1, 1000e18);

        vm.prank(user1);
        token.stake(500e18);

        vm.prank(user1);
        vm.expectRevert();
        token.slash(user1, 100e18);
    }

    function test_burn() public {
        vm.prank(minter);
        token.mint(user1, 1000e18);

        vm.prank(user1);
        token.burn(300e18);

        assertEq(token.balanceOf(user1), 700e18);
    }
}
