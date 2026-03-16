// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

/// @title CommitmentRegistry
/// @notice Registers non-overlapping time commitment slots — the "double-spend prevention"
///         for human time. Like UTXOs but for attention.
contract CommitmentRegistry {
    enum CommitmentLevel {
        DEEP_FOCUS,           // 3.0x multiplier — exclusive cognitive engagement
        ACTIVE_ENGAGEMENT,    // 1.5x multiplier — primary task with minor interruptions
        BACKGROUND            // 1.0x multiplier — monitoring/on-call (parallel allowed)
    }

    enum SlotStatus {
        PENDING,
        ACTIVE,
        VERIFIED,
        DISPUTED,
        EXPIRED,
        CANCELLED
    }

    struct CommitmentSlot {
        address owner;
        uint64 start;
        uint64 end;
        address client;
        CommitmentLevel level;
        bytes32 challengeStreamSeed;
        SlotStatus status;
    }

    /// @notice Auto-incrementing slot ID
    uint256 public nextSlotId = 1;

    /// @notice All commitment slots by ID
    mapping(uint256 => CommitmentSlot) public slots;

    /// @notice Active slot IDs per user
    mapping(address => uint256[]) private _userSlots;

    /// @notice Authorized verifiers that can update slot status
    mapping(address => bool) public authorizedVerifiers;

    /// @notice Contract deployer
    address public admin;

    event SlotRegistered(
        uint256 indexed slotId,
        address indexed owner,
        uint64 start,
        uint64 end,
        CommitmentLevel level,
        address client
    );
    event SlotCancelled(uint256 indexed slotId);
    event SlotStatusChanged(uint256 indexed slotId, SlotStatus newStatus);

    modifier onlyAdmin() {
        require(msg.sender == admin, "CommitmentRegistry: not admin");
        _;
    }

    constructor() {
        admin = msg.sender;
    }

    /// @notice Authorize an address to update slot statuses (e.g., EngagementVerifier)
    function setAuthorizedVerifier(address verifier, bool authorized) external onlyAdmin {
        authorizedVerifiers[verifier] = authorized;
    }

    /// @notice Register a new commitment slot.
    /// @param start Unix timestamp for slot start
    /// @param end Unix timestamp for slot end
    /// @param client Address of the client/contract being committed to
    /// @param level Commitment level (determines multiplier and overlap rules)
    /// @return slotId The ID of the newly created slot
    function registerSlot(
        uint64 start,
        uint64 end,
        address client,
        CommitmentLevel level
    ) external returns (uint256 slotId) {
        require(end > start, "CommitmentRegistry: end must be after start");
        require(start >= block.timestamp, "CommitmentRegistry: start must be in future");
        require(end - start >= 15 minutes, "CommitmentRegistry: minimum 15 minute slot");
        require(end - start <= 12 hours, "CommitmentRegistry: maximum 12 hour slot");

        // Check for overlapping slots
        require(!_hasOverlap(msg.sender, start, end, level), "CommitmentRegistry: overlapping slot exists");

        slotId = nextSlotId++;

        bytes32 seed = keccak256(abi.encodePacked(blockhash(block.number - 1), slotId, msg.sender));

        slots[slotId] = CommitmentSlot({
            owner: msg.sender,
            start: start,
            end: end,
            client: client,
            level: level,
            challengeStreamSeed: seed,
            status: SlotStatus.PENDING
        });

        _userSlots[msg.sender].push(slotId);

        emit SlotRegistered(slotId, msg.sender, start, end, level, client);
    }

    /// @notice Cancel a pending slot before it starts.
    /// @param slotId The slot to cancel
    function cancelSlot(uint256 slotId) external {
        CommitmentSlot storage slot = slots[slotId];
        require(slot.owner == msg.sender, "CommitmentRegistry: not owner");
        require(slot.status == SlotStatus.PENDING, "CommitmentRegistry: not pending");
        require(block.timestamp < slot.start, "CommitmentRegistry: already started");

        slot.status = SlotStatus.CANCELLED;
        emit SlotCancelled(slotId);
    }

    /// @notice Update slot status (called by EngagementVerifier or admin)
    function updateSlotStatus(uint256 slotId, SlotStatus newStatus) external {
        require(
            authorizedVerifiers[msg.sender] || msg.sender == admin,
            "CommitmentRegistry: not authorized"
        );
        slots[slotId].status = newStatus;
        emit SlotStatusChanged(slotId, newStatus);
    }

    /// @notice Check if a user has any overlapping active slots for a given time range.
    function checkOverlap(
        address user,
        uint64 start,
        uint64 end,
        CommitmentLevel level
    ) external view returns (bool) {
        return _hasOverlap(user, start, end, level);
    }

    /// @notice Get all slot IDs for a user (including cancelled/expired).
    function getUserSlots(address user) external view returns (uint256[] memory) {
        return _userSlots[user];
    }

    /// @notice Get active (non-cancelled, non-expired) slot IDs for a user.
    function getActiveSlots(address user) external view returns (uint256[] memory) {
        uint256[] storage allSlots = _userSlots[user];
        uint256 count = 0;

        // Count active slots first
        for (uint256 i = 0; i < allSlots.length; i++) {
            SlotStatus s = slots[allSlots[i]].status;
            if (s == SlotStatus.PENDING || s == SlotStatus.ACTIVE || s == SlotStatus.VERIFIED) {
                count++;
            }
        }

        // Build result array
        uint256[] memory result = new uint256[](count);
        uint256 idx = 0;
        for (uint256 i = 0; i < allSlots.length; i++) {
            SlotStatus s = slots[allSlots[i]].status;
            if (s == SlotStatus.PENDING || s == SlotStatus.ACTIVE || s == SlotStatus.VERIFIED) {
                result[idx++] = allSlots[i];
            }
        }

        return result;
    }

    /// @notice Get the level multiplier in basis points (3.0x = 30000, 1.5x = 15000, 1.0x = 10000)
    function getLevelMultiplier(CommitmentLevel level) public pure returns (uint256) {
        if (level == CommitmentLevel.DEEP_FOCUS) return 30000;
        if (level == CommitmentLevel.ACTIVE_ENGAGEMENT) return 15000;
        return 10000; // BACKGROUND
    }

    /// @dev Check for time overlaps with existing non-cancelled slots.
    ///      DEEP_FOCUS and ACTIVE_ENGAGEMENT reject ALL overlaps.
    ///      BACKGROUND only rejects overlaps with non-BACKGROUND slots.
    function _hasOverlap(
        address user,
        uint64 start,
        uint64 end,
        CommitmentLevel newLevel
    ) internal view returns (bool) {
        uint256[] storage userSlots = _userSlots[user];

        for (uint256 i = 0; i < userSlots.length; i++) {
            CommitmentSlot storage existing = slots[userSlots[i]];

            // Skip cancelled/expired slots
            if (existing.status == SlotStatus.CANCELLED || existing.status == SlotStatus.EXPIRED) {
                continue;
            }

            // Check time overlap: two intervals [s1, e1] and [s2, e2] overlap iff s1 < e2 AND s2 < e1
            if (start < existing.end && existing.start < end) {
                // Time overlap exists — check level compatibility
                if (newLevel == CommitmentLevel.BACKGROUND && existing.level == CommitmentLevel.BACKGROUND) {
                    // BACKGROUND + BACKGROUND is allowed
                    continue;
                }
                // All other combinations are forbidden
                return true;
            }
        }

        return false;
    }
}
