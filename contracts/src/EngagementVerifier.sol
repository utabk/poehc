// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import "./CommitmentRegistry.sol";
import "./TIMEToken.sol";

/// @title EngagementVerifier
/// @notice Accepts proof submissions from the engagement agent and mints TIME tokens
///         when commitment slots are finalized with sufficient verification scores.
contract EngagementVerifier {
    struct ProofSubmission {
        bytes32 proofHash;
        uint64 timestamp;
        uint16 challengesPassed;
        uint16 challengesTotal;
    }

    CommitmentRegistry public immutable registry;
    TIMEToken public immutable timeToken;

    /// @notice Proof submissions per slot
    mapping(uint256 => ProofSubmission[]) public slotProofs;

    /// @notice Whether a slot has been finalized (TIME minted)
    mapping(uint256 => bool) public slotFinalized;

    /// @notice Minimum verification score (0-10000 basis points) to earn any TIME
    uint256 public constant MIN_SCORE_BPS = 3000; // 30%

    /// @notice Minimum number of proofs required to finalize
    uint256 public constant MIN_PROOFS = 3;

    event ProofSubmitted(uint256 indexed slotId, bytes32 proofHash, uint16 passed, uint16 total);
    event SlotFinalized(uint256 indexed slotId, address indexed owner, uint256 timeEarned, uint256 scoreBps);
    event DisputeRaised(uint256 indexed slotId, uint256 proofIndex, address challenger);

    constructor(address _registry, address _timeToken) {
        registry = CommitmentRegistry(_registry);
        timeToken = TIMEToken(_timeToken);
    }

    /// @notice Submit a proof of engagement for a commitment slot.
    /// @param slotId The commitment slot ID
    /// @param proofHash Hash of the ZK proof (or behavioral proof hash for MVP)
    /// @param challengesPassed Number of challenges correctly answered
    /// @param challengesTotal Total challenges issued
    function submitProof(
        uint256 slotId,
        bytes32 proofHash,
        uint16 challengesPassed,
        uint16 challengesTotal
    ) external {
        (
            address owner,
            uint64 start,
            uint64 end,
            ,
            ,
            ,
            CommitmentRegistry.SlotStatus status
        ) = registry.slots(slotId);

        require(msg.sender == owner, "EngagementVerifier: not slot owner");
        require(
            status == CommitmentRegistry.SlotStatus.PENDING ||
            status == CommitmentRegistry.SlotStatus.ACTIVE,
            "EngagementVerifier: slot not active"
        );
        require(block.timestamp >= start, "EngagementVerifier: slot not started");
        require(block.timestamp <= end + 1 hours, "EngagementVerifier: submission window closed");
        require(challengesPassed <= challengesTotal, "EngagementVerifier: invalid challenge count");
        require(challengesTotal > 0, "EngagementVerifier: no challenges");
        require(!slotFinalized[slotId], "EngagementVerifier: already finalized");

        slotProofs[slotId].push(ProofSubmission({
            proofHash: proofHash,
            timestamp: uint64(block.timestamp),
            challengesPassed: challengesPassed,
            challengesTotal: challengesTotal
        }));

        // Activate the slot if it was pending
        if (status == CommitmentRegistry.SlotStatus.PENDING) {
            registry.updateSlotStatus(slotId, CommitmentRegistry.SlotStatus.ACTIVE);
        }

        emit ProofSubmitted(slotId, proofHash, challengesPassed, challengesTotal);
    }

    /// @notice Get the aggregate verification score for a slot (in basis points, 0-10000).
    /// @param slotId The commitment slot ID
    /// @return scoreBps Weighted average score in basis points
    function getVerificationScore(uint256 slotId) public view returns (uint256 scoreBps) {
        ProofSubmission[] storage proofs = slotProofs[slotId];
        if (proofs.length == 0) return 0;

        uint256 totalPassed;
        uint256 totalChallenges;

        for (uint256 i = 0; i < proofs.length; i++) {
            totalPassed += proofs[i].challengesPassed;
            totalChallenges += proofs[i].challengesTotal;
        }

        if (totalChallenges == 0) return 0;
        scoreBps = (totalPassed * 10000) / totalChallenges;
    }

    /// @notice Get the number of proof submissions for a slot.
    function getProofCount(uint256 slotId) external view returns (uint256) {
        return slotProofs[slotId].length;
    }

    /// @notice Finalize a completed slot and mint TIME tokens.
    ///         Can be called by the slot owner after the slot end time.
    /// @param slotId The commitment slot ID
    function finalizeSlot(uint256 slotId) external {
        (
            address owner,
            uint64 start,
            uint64 end,
            ,
            CommitmentRegistry.CommitmentLevel level,
            ,
            CommitmentRegistry.SlotStatus status
        ) = registry.slots(slotId);

        require(
            status == CommitmentRegistry.SlotStatus.ACTIVE ||
            status == CommitmentRegistry.SlotStatus.PENDING,
            "EngagementVerifier: slot not active"
        );
        require(block.timestamp >= end, "EngagementVerifier: slot not ended");
        require(!slotFinalized[slotId], "EngagementVerifier: already finalized");
        require(slotProofs[slotId].length >= MIN_PROOFS, "EngagementVerifier: insufficient proofs");

        uint256 scoreBps = getVerificationScore(slotId);
        require(scoreBps >= MIN_SCORE_BPS, "EngagementVerifier: score too low");

        slotFinalized[slotId] = true;

        // Calculate TIME to mint:
        // amount = duration_hours * level_multiplier * score
        // Using fixed-point: (duration_seconds * multiplier_bps * score_bps) / (3600 * 10000 * 10000)
        uint256 durationSeconds = uint256(end - start);
        uint256 multiplierBps = registry.getLevelMultiplier(level);

        // TIME has 18 decimals
        // amount = (durationSeconds * multiplierBps * scoreBps * 1e18) / (3600 * 10000 * 10000)
        uint256 amount = (durationSeconds * multiplierBps * scoreBps * 1e18) / (3600 * 10000 * 10000);

        // Update slot status
        registry.updateSlotStatus(slotId, CommitmentRegistry.SlotStatus.VERIFIED);

        // Mint TIME
        timeToken.mint(owner, amount);

        emit SlotFinalized(slotId, owner, amount, scoreBps);
    }

    /// @notice Raise a dispute against a specific proof submission.
    ///         Placeholder for future dispute resolution mechanism.
    function challengeProof(uint256 slotId, uint256 proofIndex, bytes calldata /* evidence */) external {
        require(proofIndex < slotProofs[slotId].length, "EngagementVerifier: invalid proof index");
        require(!slotFinalized[slotId], "EngagementVerifier: already finalized");

        emit DisputeRaised(slotId, proofIndex, msg.sender);
        // Full dispute resolution logic deferred to future version
    }
}
