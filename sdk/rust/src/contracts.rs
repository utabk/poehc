use alloy::sol;

sol! {
    #[sol(rpc)]
    interface ICommitmentRegistry {
        function registerSlot(uint64 start, uint64 end, address client, uint8 level) external returns (uint256 slotId);
        function cancelSlot(uint256 slotId) external;
        function getActiveSlots(address user) external view returns (uint256[] memory);
        function getUserSlots(address user) external view returns (uint256[] memory);
        function checkOverlap(address user, uint64 start, uint64 end, uint8 level) external view returns (bool);
        function slots(uint256 slotId) external view returns (address owner, uint64 start, uint64 end, address client, uint8 level, bytes32 challengeStreamSeed, uint8 status);
        function getLevelMultiplier(uint8 level) external pure returns (uint256);

        event SlotRegistered(uint256 indexed slotId, address indexed owner, uint64 start, uint64 end, uint8 level, address client);
        event SlotCancelled(uint256 indexed slotId);
    }

    #[sol(rpc)]
    interface IEngagementVerifier {
        function submitProof(uint256 slotId, bytes32 proofHash, uint16 challengesPassed, uint16 challengesTotal) external;
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
        function stakedBalance(address account) external view returns (uint256);
        function approve(address spender, uint256 amount) external returns (bool);
        function transfer(address to, uint256 amount) external returns (bool);
        function stake(uint256 amount) external;
        function requestUnstake(uint256 amount) external;
        function completeUnstake() external;
    }

    #[sol(rpc)]
    interface IValidatorRegistry {
        function register() external;
        function deactivate() external;
        function reportValidation(uint256 slotId, bool approved) external;
        function claimRewards() external;
        function getActiveValidatorCount() external view returns (uint256);
        function pendingRewards(address validator) external view returns (uint256);
    }

    #[sol(rpc)]
    interface ICommitmentMarketplace {
        function createContract(uint256 budget, uint8 requiredLevel, uint64 startTime, uint64 endTime) external returns (uint256 contractId);
        function acceptContract(uint256 contractId, uint256 slotId) external;
        function settleContract(uint256 contractId) external;
        function cancelContract(uint256 contractId) external;
        function disputeContract(uint256 contractId) external;
    }
}
