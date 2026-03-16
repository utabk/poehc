use alloy::sol;

sol! {
    #[sol(rpc)]
    interface IEngagementVerifier {
        function getVerificationScore(uint256 slotId) external view returns (uint256 scoreBps);
        function getProofCount(uint256 slotId) external view returns (uint256);
        function slotFinalized(uint256 slotId) external view returns (bool);

        event ProofSubmitted(uint256 indexed slotId, bytes32 proofHash, uint16 passed, uint16 total);
        event SlotFinalized(uint256 indexed slotId, address indexed owner, uint256 timeEarned, uint256 scoreBps);
    }

    #[sol(rpc)]
    interface IValidatorRegistry {
        function reportValidation(uint256 slotId, bool approved) external;
        function claimRewards() external;
        function pendingRewards(address validator) external view returns (uint256);
    }
}
