// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import "forge-std/Test.sol";
import "../src/TIMEToken.sol";
import "../src/ValidatorRegistry.sol";

contract ValidatorRegistryTest is Test {
    TIMEToken public token;
    ValidatorRegistry public validatorReg;

    address public admin = address(1);
    address public val1 = address(10);
    address public val2 = address(11);
    address public val3 = address(12);
    address public val4 = address(13);

    function setUp() public {
        vm.startPrank(admin);
        token = new TIMEToken(admin);
        validatorReg = new ValidatorRegistry(address(token));

        // Grant minter role to admin for setup and to validatorReg for rewards
        token.grantRole(token.MINTER_ROLE(), admin);
        token.grantRole(token.MINTER_ROLE(), address(validatorReg));

        // Mint and stake for validators
        _setupValidator(val1, 2000e18);
        _setupValidator(val2, 1500e18);
        _setupValidator(val3, 1000e18);
        _setupValidator(val4, 3000e18);

        vm.stopPrank();
    }

    function _setupValidator(address val, uint256 amount) internal {
        token.mint(val, amount);
        vm.stopPrank();
        vm.prank(val);
        token.stake(amount);
        vm.startPrank(admin);
    }

    function test_register_validator() public {
        vm.prank(val1);
        validatorReg.register();

        (uint256 staked, , , bool active) = validatorReg.validators(val1);
        assertTrue(active);
        assertEq(staked, 2000e18);
        assertEq(validatorReg.getActiveValidatorCount(), 1);
    }

    function test_register_revert_insufficient_stake() public {
        address lowStaker = address(99);
        vm.prank(admin);
        token.mint(lowStaker, 500e18);
        vm.prank(lowStaker);
        token.stake(500e18);

        vm.prank(lowStaker);
        vm.expectRevert("ValidatorRegistry: insufficient stake in TIMEToken");
        validatorReg.register();
    }

    function test_register_revert_already_active() public {
        vm.prank(val1);
        validatorReg.register();

        vm.prank(val1);
        vm.expectRevert("ValidatorRegistry: already active");
        validatorReg.register();
    }

    function test_deactivate() public {
        vm.prank(val1);
        validatorReg.register();
        assertEq(validatorReg.getActiveValidatorCount(), 1);

        vm.prank(val1);
        validatorReg.deactivate();

        (, , , bool active) = validatorReg.validators(val1);
        assertFalse(active);
        assertEq(validatorReg.getActiveValidatorCount(), 0);
    }

    function test_assign_committee() public {
        // Register enough validators
        vm.prank(val1);
        validatorReg.register();
        vm.prank(val2);
        validatorReg.register();
        vm.prank(val3);
        validatorReg.register();

        address[] memory committee = validatorReg.assignCommittee(42);
        assertEq(committee.length, 3);

        // All should be unique
        assertTrue(committee[0] != committee[1]);
        assertTrue(committee[1] != committee[2]);
        assertTrue(committee[0] != committee[2]);
    }

    function test_assign_committee_revert_insufficient() public {
        vm.prank(val1);
        validatorReg.register();
        // Only 1 validator, need 3

        vm.expectRevert("ValidatorRegistry: insufficient validators");
        validatorReg.assignCommittee(42);
    }

    function test_report_validation() public {
        vm.prank(val1);
        validatorReg.register();

        vm.prank(val1);
        validatorReg.reportValidation(1, true);

        assertTrue(validatorReg.hasReported(1, val1));
        assertTrue(validatorReg.validationReports(1, val1));

        (, uint256 validationCount, , ) = validatorReg.validators(val1);
        assertEq(validationCount, 1);
    }

    function test_report_revert_not_validator() public {
        vm.prank(address(99));
        vm.expectRevert("ValidatorRegistry: not active validator");
        validatorReg.reportValidation(1, true);
    }

    function test_report_revert_double_report() public {
        vm.prank(val1);
        validatorReg.register();

        vm.prank(val1);
        validatorReg.reportValidation(1, true);

        vm.prank(val1);
        vm.expectRevert("ValidatorRegistry: already reported");
        validatorReg.reportValidation(1, false);
    }

    function test_claim_rewards() public {
        vm.prank(val1);
        validatorReg.register();

        vm.prank(admin);
        validatorReg.addReward(val1, 50e18);

        uint256 balanceBefore = token.balanceOf(val1);

        vm.prank(val1);
        validatorReg.claimRewards();

        assertEq(token.balanceOf(val1), balanceBefore + 50e18);
        assertEq(validatorReg.pendingRewards(val1), 0);
    }

    function test_claim_revert_no_rewards() public {
        vm.prank(val1);
        validatorReg.register();

        vm.prank(val1);
        vm.expectRevert("ValidatorRegistry: no rewards");
        validatorReg.claimRewards();
    }
}
