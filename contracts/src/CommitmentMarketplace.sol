// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import "./CommitmentRegistry.sol";
import "./EngagementVerifier.sol";
import "./TIMEToken.sol";

/// @title CommitmentMarketplace
/// @notice Connects clients who need verified human commitment with workers.
///         Automates slot registration and payment settlement based on verification scores.
contract CommitmentMarketplace {
    enum ContractStatus {
        OPEN,       // Client posted, waiting for worker
        ACCEPTED,   // Worker accepted, slot registered
        SETTLED,    // Work verified and paid
        DISPUTED,   // Under dispute
        CANCELLED   // Cancelled before acceptance
    }

    struct WorkContract {
        address client;
        address worker;
        uint256 budget;           // TIME tokens locked as payment
        CommitmentRegistry.CommitmentLevel requiredLevel;
        uint64 startTime;
        uint64 endTime;
        uint256 slotId;           // CommitmentRegistry slot ID (set on accept)
        ContractStatus status;
    }

    CommitmentRegistry public immutable registry;
    EngagementVerifier public immutable verifier;
    TIMEToken public immutable timeToken;

    uint256 public nextContractId = 1;
    mapping(uint256 => WorkContract) public contracts;

    /// @notice Protocol fee in basis points (1% = 100 bps)
    uint256 public constant PROTOCOL_FEE_BPS = 100; // 1%

    /// @notice Protocol fee recipient
    address public feeRecipient;

    event ContractCreated(uint256 indexed contractId, address indexed client, uint256 budget);
    event ContractAccepted(uint256 indexed contractId, address indexed worker, uint256 slotId);
    event ContractSettled(uint256 indexed contractId, uint256 workerPayment, uint256 protocolFee);
    event ContractCancelled(uint256 indexed contractId);
    event ContractDisputed(uint256 indexed contractId, address disputant);

    constructor(address _registry, address _verifier, address _timeToken, address _feeRecipient) {
        registry = CommitmentRegistry(_registry);
        verifier = EngagementVerifier(_verifier);
        timeToken = TIMEToken(_timeToken);
        feeRecipient = _feeRecipient;
    }

    /// @notice Client creates a work contract by locking TIME tokens.
    /// @param budget Amount of TIME to lock as payment
    /// @param requiredLevel Required commitment level
    /// @param startTime When work should begin
    /// @param endTime When work should end
    function createContract(
        uint256 budget,
        CommitmentRegistry.CommitmentLevel requiredLevel,
        uint64 startTime,
        uint64 endTime
    ) external returns (uint256 contractId) {
        require(budget > 0, "Marketplace: zero budget");
        require(endTime > startTime, "Marketplace: invalid time range");
        require(startTime > block.timestamp, "Marketplace: start must be future");

        // Transfer budget from client to this contract
        require(
            timeToken.transferFrom(msg.sender, address(this), budget),
            "Marketplace: transfer failed"
        );

        contractId = nextContractId++;
        contracts[contractId] = WorkContract({
            client: msg.sender,
            worker: address(0),
            budget: budget,
            requiredLevel: requiredLevel,
            startTime: startTime,
            endTime: endTime,
            slotId: 0,
            status: ContractStatus.OPEN
        });

        emit ContractCreated(contractId, msg.sender, budget);
    }

    /// @notice Worker accepts a contract by providing a pre-registered commitment slot.
    ///         The worker must call CommitmentRegistry.registerSlot() first, then pass
    ///         the slotId here. This ensures the worker (not the marketplace) owns the slot.
    /// @param contractId The contract to accept
    /// @param slotId The worker's pre-registered commitment slot ID
    function acceptContract(uint256 contractId, uint256 slotId) external {
        WorkContract storage wc = contracts[contractId];
        require(wc.status == ContractStatus.OPEN, "Marketplace: not open");
        require(block.timestamp < wc.startTime, "Marketplace: start time passed");

        // Verify the slot matches contract requirements
        (
            address slotOwner,
            uint64 slotStart,
            uint64 slotEnd,
            ,
            CommitmentRegistry.CommitmentLevel slotLevel,
            ,
            CommitmentRegistry.SlotStatus slotStatus
        ) = registry.slots(slotId);

        require(slotOwner == msg.sender, "Marketplace: slot not owned by worker");
        require(slotStart == wc.startTime, "Marketplace: slot start mismatch");
        require(slotEnd == wc.endTime, "Marketplace: slot end mismatch");
        require(slotLevel == wc.requiredLevel, "Marketplace: slot level mismatch");
        require(slotStatus == CommitmentRegistry.SlotStatus.PENDING, "Marketplace: slot not pending");

        wc.worker = msg.sender;
        wc.status = ContractStatus.ACCEPTED;
        wc.slotId = slotId;

        emit ContractAccepted(contractId, msg.sender, slotId);
    }

    /// @notice Settle a contract after the slot has been verified.
    ///         Pays the worker proportional to their verification score.
    /// @param contractId The contract to settle
    function settleContract(uint256 contractId) external {
        WorkContract storage wc = contracts[contractId];
        require(wc.status == ContractStatus.ACCEPTED, "Marketplace: not accepted");
        require(block.timestamp >= wc.endTime, "Marketplace: not ended");
        require(verifier.slotFinalized(wc.slotId), "Marketplace: slot not finalized");

        wc.status = ContractStatus.SETTLED;

        uint256 scoreBps = verifier.getVerificationScore(wc.slotId);

        // Payment proportional to score
        uint256 grossPayment = (wc.budget * scoreBps) / 10000;
        uint256 protocolFee = (grossPayment * PROTOCOL_FEE_BPS) / 10000;
        uint256 workerPayment = grossPayment - protocolFee;
        uint256 refund = wc.budget - grossPayment;

        // Pay worker
        if (workerPayment > 0) {
            timeToken.transfer(wc.worker, workerPayment);
        }

        // Pay protocol fee
        if (protocolFee > 0) {
            timeToken.transfer(feeRecipient, protocolFee);
        }

        // Refund remaining to client
        if (refund > 0) {
            timeToken.transfer(wc.client, refund);
        }

        emit ContractSettled(contractId, workerPayment, protocolFee);
    }

    /// @notice Cancel an open contract (before anyone accepts).
    function cancelContract(uint256 contractId) external {
        WorkContract storage wc = contracts[contractId];
        require(msg.sender == wc.client, "Marketplace: not client");
        require(wc.status == ContractStatus.OPEN, "Marketplace: not open");

        wc.status = ContractStatus.CANCELLED;

        // Return budget to client
        timeToken.transfer(wc.client, wc.budget);

        emit ContractCancelled(contractId);
    }

    /// @notice Raise a dispute on an accepted contract.
    function disputeContract(uint256 contractId) external {
        WorkContract storage wc = contracts[contractId];
        require(
            msg.sender == wc.client || msg.sender == wc.worker,
            "Marketplace: not party to contract"
        );
        require(wc.status == ContractStatus.ACCEPTED, "Marketplace: not accepted");

        wc.status = ContractStatus.DISPUTED;

        emit ContractDisputed(contractId, msg.sender);
        // Full dispute resolution deferred to future version
    }
}
