// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import "@openzeppelin/contracts/token/ERC20/ERC20.sol";
import "@openzeppelin/contracts/token/ERC20/extensions/ERC20Burnable.sol";
import "@openzeppelin/contracts/access/AccessControl.sol";

/// @title TIME Token
/// @notice ERC-20 token representing verified human commitment.
///         Minted by the EngagementVerifier when commitment slots are finalized.
///         Can be staked for validator eligibility and slashed for dishonest validation.
contract TIMEToken is ERC20, ERC20Burnable, AccessControl {
    bytes32 public constant MINTER_ROLE = keccak256("MINTER_ROLE");
    bytes32 public constant SLASHER_ROLE = keccak256("SLASHER_ROLE");

    /// @notice Maximum total supply: 10 billion TIME (18 decimals)
    uint256 public constant MAX_SUPPLY = 10_000_000_000 * 1e18;

    /// @notice Minimum cooldown before unstaking (7 days)
    uint256 public constant UNSTAKE_COOLDOWN = 7 days;

    /// @notice Staked balance per address
    mapping(address => uint256) public stakedBalance;

    /// @notice Timestamp when unstake was requested (0 = no pending unstake)
    mapping(address => uint256) public unstakeRequestedAt;

    /// @notice Amount pending unstake
    mapping(address => uint256) public unstakePendingAmount;

    event Staked(address indexed user, uint256 amount);
    event UnstakeRequested(address indexed user, uint256 amount, uint256 availableAt);
    event Unstaked(address indexed user, uint256 amount);
    event Slashed(address indexed user, uint256 amount);

    constructor(address admin) ERC20("TIME", "TIME") {
        _grantRole(DEFAULT_ADMIN_ROLE, admin);
    }

    /// @notice Mint TIME tokens. Only callable by addresses with MINTER_ROLE.
    /// @param to Recipient address
    /// @param amount Amount to mint (18 decimals)
    function mint(address to, uint256 amount) external onlyRole(MINTER_ROLE) {
        require(totalSupply() + amount <= MAX_SUPPLY, "TIMEToken: max supply exceeded");
        _mint(to, amount);
    }

    /// @notice Stake TIME tokens for validator eligibility.
    /// @param amount Amount to stake
    function stake(uint256 amount) external {
        require(amount > 0, "TIMEToken: cannot stake zero");
        require(balanceOf(msg.sender) >= amount, "TIMEToken: insufficient balance");

        _transfer(msg.sender, address(this), amount);
        stakedBalance[msg.sender] += amount;

        emit Staked(msg.sender, amount);
    }

    /// @notice Request to unstake. Begins cooldown period.
    /// @param amount Amount to unstake
    function requestUnstake(uint256 amount) external {
        require(amount > 0, "TIMEToken: cannot unstake zero");
        require(stakedBalance[msg.sender] >= amount, "TIMEToken: insufficient staked balance");
        require(unstakeRequestedAt[msg.sender] == 0, "TIMEToken: unstake already pending");

        unstakeRequestedAt[msg.sender] = block.timestamp;
        unstakePendingAmount[msg.sender] = amount;

        emit UnstakeRequested(msg.sender, amount, block.timestamp + UNSTAKE_COOLDOWN);
    }

    /// @notice Complete unstake after cooldown period.
    function completeUnstake() external {
        require(unstakeRequestedAt[msg.sender] != 0, "TIMEToken: no pending unstake");
        require(
            block.timestamp >= unstakeRequestedAt[msg.sender] + UNSTAKE_COOLDOWN,
            "TIMEToken: cooldown not elapsed"
        );

        uint256 amount = unstakePendingAmount[msg.sender];
        stakedBalance[msg.sender] -= amount;
        unstakeRequestedAt[msg.sender] = 0;
        unstakePendingAmount[msg.sender] = 0;

        _transfer(address(this), msg.sender, amount);

        emit Unstaked(msg.sender, amount);
    }

    /// @notice Slash a validator's staked tokens. Only callable by SLASHER_ROLE.
    /// @param validator Address to slash
    /// @param amount Amount to slash (burned)
    function slash(address validator, uint256 amount) external onlyRole(SLASHER_ROLE) {
        require(stakedBalance[validator] >= amount, "TIMEToken: insufficient staked balance");

        stakedBalance[validator] -= amount;
        _burn(address(this), amount);

        emit Slashed(validator, amount);
    }
}
