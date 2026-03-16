// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import "./TIMEToken.sol";

/// @title ValidatorRegistry
/// @notice Manages validator registration, staking, committee assignment,
///         and reward/slashing for the PoEHC validation network.
contract ValidatorRegistry {
    struct Validator {
        uint256 stakedAmount;
        uint256 validationCount;
        uint256 slashCount;
        bool active;
    }

    TIMEToken public immutable timeToken;

    /// @notice Minimum stake required to become a validator
    uint256 public constant MIN_STAKE = 1000e18; // 1000 TIME

    /// @notice Committee size for each validation
    uint256 public constant COMMITTEE_SIZE = 3;

    /// @notice Validator data by address
    mapping(address => Validator) public validators;

    /// @notice List of all active validator addresses
    address[] public activeValidators;

    /// @notice Validation reports: slotId => validator => approved
    mapping(uint256 => mapping(address => bool)) public validationReports;

    /// @notice Whether a validator has reported for a slot
    mapping(uint256 => mapping(address => bool)) public hasReported;

    /// @notice Accumulated rewards per validator (in TIME wei)
    mapping(address => uint256) public pendingRewards;

    /// @notice Contract admin
    address public admin;

    event ValidatorRegistered(address indexed validator, uint256 stakedAmount);
    event ValidatorDeactivated(address indexed validator);
    event CommitteeAssigned(uint256 indexed slotId, address[] committee);
    event ValidationReported(uint256 indexed slotId, address indexed validator, bool approved);
    event RewardsClaimed(address indexed validator, uint256 amount);
    event ValidatorSlashed(address indexed validator, uint256 amount);

    modifier onlyAdmin() {
        require(msg.sender == admin, "ValidatorRegistry: not admin");
        _;
    }

    constructor(address _timeToken) {
        timeToken = TIMEToken(_timeToken);
        admin = msg.sender;
    }

    /// @notice Register as a validator by staking TIME tokens.
    ///         Tokens must be staked in TIMEToken contract first.
    function register() external {
        require(!validators[msg.sender].active, "ValidatorRegistry: already active");
        require(
            timeToken.stakedBalance(msg.sender) >= MIN_STAKE,
            "ValidatorRegistry: insufficient stake in TIMEToken"
        );

        validators[msg.sender] = Validator({
            stakedAmount: timeToken.stakedBalance(msg.sender),
            validationCount: 0,
            slashCount: 0,
            active: true
        });

        activeValidators.push(msg.sender);

        emit ValidatorRegistered(msg.sender, timeToken.stakedBalance(msg.sender));
    }

    /// @notice Deactivate as a validator.
    function deactivate() external {
        require(validators[msg.sender].active, "ValidatorRegistry: not active");
        validators[msg.sender].active = false;

        // Remove from active list
        _removeFromActiveList(msg.sender);

        emit ValidatorDeactivated(msg.sender);
    }

    /// @notice Assign a pseudo-random committee for a given slot.
    /// @param slotId The commitment slot to assign validators to
    /// @return committee Array of selected validator addresses
    function assignCommittee(uint256 slotId) external view returns (address[] memory committee) {
        uint256 validatorCount = activeValidators.length;
        require(validatorCount >= COMMITTEE_SIZE, "ValidatorRegistry: insufficient validators");

        committee = new address[](COMMITTEE_SIZE);
        uint256 assigned = 0;

        for (uint256 i = 0; assigned < COMMITTEE_SIZE && i < validatorCount * 3; i++) {
            uint256 seed = uint256(keccak256(abi.encodePacked(slotId, blockhash(block.number - 1), i)));
            uint256 index = seed % validatorCount;
            address candidate = activeValidators[index];

            // Check for duplicates
            bool duplicate = false;
            for (uint256 j = 0; j < assigned; j++) {
                if (committee[j] == candidate) {
                    duplicate = true;
                    break;
                }
            }

            if (!duplicate && validators[candidate].active) {
                committee[assigned] = candidate;
                assigned++;
            }
        }

        require(assigned == COMMITTEE_SIZE, "ValidatorRegistry: could not form committee");
    }

    /// @notice Submit a validation report for a slot.
    /// @param slotId The commitment slot being validated
    /// @param approved Whether the proof is approved
    function reportValidation(uint256 slotId, bool approved) external {
        require(validators[msg.sender].active, "ValidatorRegistry: not active validator");
        require(!hasReported[slotId][msg.sender], "ValidatorRegistry: already reported");

        hasReported[slotId][msg.sender] = true;
        validationReports[slotId][msg.sender] = approved;
        validators[msg.sender].validationCount++;

        emit ValidationReported(slotId, msg.sender, approved);
    }

    /// @notice Distribute rewards to a validator (called by admin/protocol).
    function addReward(address validator, uint256 amount) external onlyAdmin {
        require(validators[validator].active, "ValidatorRegistry: not active");
        pendingRewards[validator] += amount;
    }

    /// @notice Claim accumulated rewards.
    function claimRewards() external {
        uint256 amount = pendingRewards[msg.sender];
        require(amount > 0, "ValidatorRegistry: no rewards");

        pendingRewards[msg.sender] = 0;
        timeToken.mint(msg.sender, amount);

        emit RewardsClaimed(msg.sender, amount);
    }

    /// @notice Get the count of active validators.
    function getActiveValidatorCount() external view returns (uint256) {
        return activeValidators.length;
    }

    /// @dev Remove an address from the active validators list.
    function _removeFromActiveList(address validator) internal {
        for (uint256 i = 0; i < activeValidators.length; i++) {
            if (activeValidators[i] == validator) {
                activeValidators[i] = activeValidators[activeValidators.length - 1];
                activeValidators.pop();
                break;
            }
        }
    }
}
