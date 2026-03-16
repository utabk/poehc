use alloy::sol;

sol! {
    #[sol(rpc)]
    interface ICommitmentRegistry {
        enum CommitmentLevel {
            DEEP_FOCUS,
            ACTIVE_ENGAGEMENT,
            BACKGROUND
        }

        enum SlotStatus {
            PENDING,
            ACTIVE,
            VERIFIED,
            DISPUTED,
            EXPIRED,
            CANCELLED
        }

        function registerSlot(
            uint64 start,
            uint64 end,
            address client,
            CommitmentLevel level
        ) external returns (uint256 slotId);

        function cancelSlot(uint256 slotId) external;

        function getActiveSlots(address user) external view returns (uint256[] memory);

        function checkOverlap(
            address user,
            uint64 start,
            uint64 end,
            CommitmentLevel level
        ) external view returns (bool);

        function slots(uint256 slotId) external view returns (
            address owner,
            uint64 start,
            uint64 end,
            address client,
            CommitmentLevel level,
            bytes32 challengeStreamSeed,
            SlotStatus status
        );

        function getLevelMultiplier(CommitmentLevel level) external pure returns (uint256);

        event SlotRegistered(
            uint256 indexed slotId,
            address indexed owner,
            uint64 start,
            uint64 end,
            CommitmentLevel level,
            address client
        );
    }

    #[sol(rpc)]
    interface IEngagementVerifier {
        function submitProof(
            uint256 slotId,
            bytes32 proofHash,
            uint16 challengesPassed,
            uint16 challengesTotal
        ) external;

        function getVerificationScore(uint256 slotId) external view returns (uint256 scoreBps);

        function getProofCount(uint256 slotId) external view returns (uint256);

        function finalizeSlot(uint256 slotId) external;

        function slotFinalized(uint256 slotId) external view returns (bool);

        event ProofSubmitted(uint256 indexed slotId, bytes32 proofHash, uint16 passed, uint16 total);
        event SlotFinalized(uint256 indexed slotId, address indexed owner, uint256 timeEarned, uint256 scoreBps);
    }

    #[sol(rpc)]
    interface ITIMEToken {
        function balanceOf(address account) external view returns (uint256);
        function totalSupply() external view returns (uint256);
        function mint(address to, uint256 amount) external;
        function stake(uint256 amount) external;
        function stakedBalance(address account) external view returns (uint256);
        function approve(address spender, uint256 amount) external returns (bool);
        function transfer(address to, uint256 amount) external returns (bool);
    }
}
