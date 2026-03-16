use crate::contracts::*;
use crate::types::{CommitmentLevel, ContractAddresses};
use alloy::network::EthereumWallet;
use alloy::primitives::{Address, FixedBytes, U256};
use alloy::providers::{Provider, ProviderBuilder};
use alloy::signers::local::PrivateKeySigner;
use std::str::FromStr;

#[derive(thiserror::Error, Debug)]
pub enum SdkError {
    #[error("RPC error: {0}")]
    Rpc(String),
    #[error("Transaction failed: {0}")]
    Transaction(String),
    #[error("Invalid input: {0}")]
    InvalidInput(String),
}

pub struct PoEHCClient {
    addresses: ContractAddresses,
    rpc_url: String,
    signer: PrivateKeySigner,
}

impl PoEHCClient {
    pub fn new(
        rpc_url: &str,
        private_key: &str,
        addresses: ContractAddresses,
    ) -> Result<Self, SdkError> {
        let signer = PrivateKeySigner::from_str(private_key)
            .map_err(|e| SdkError::InvalidInput(format!("Invalid key: {}", e)))?;
        Ok(Self {
            addresses,
            rpc_url: rpc_url.to_string(),
            signer,
        })
    }

    fn provider(&self) -> Result<impl Provider + Clone, SdkError> {
        let wallet = EthereumWallet::from(self.signer.clone());
        let provider = ProviderBuilder::new()
            .wallet(wallet)
            .connect_http(
                self.rpc_url
                    .parse()
                    .map_err(|e| SdkError::Rpc(format!("Bad URL: {}", e)))?,
            );
        Ok(provider)
    }

    pub fn address(&self) -> Address {
        self.signer.address()
    }

    pub async fn get_balance(&self, account: Address) -> Result<U256, SdkError> {
        let p = self.provider()?;
        let token = ITIMEToken::new(self.addresses.time_token, &p);
        token
            .balanceOf(account)
            .call()
            .await
            .map_err(|e| SdkError::Rpc(e.to_string()))
    }

    pub async fn get_staked_balance(&self, account: Address) -> Result<U256, SdkError> {
        let p = self.provider()?;
        let token = ITIMEToken::new(self.addresses.time_token, &p);
        token
            .stakedBalance(account)
            .call()
            .await
            .map_err(|e| SdkError::Rpc(e.to_string()))
    }

    pub async fn register_slot(
        &self,
        start: u64,
        end: u64,
        client: Address,
        level: CommitmentLevel,
    ) -> Result<FixedBytes<32>, SdkError> {
        let p = self.provider()?;
        let registry = ICommitmentRegistry::new(self.addresses.commitment_registry, &p);
        let tx = registry
            .registerSlot(start, end, client, level as u8)
            .send()
            .await
            .map_err(|e| SdkError::Transaction(e.to_string()))?;
        let receipt = tx
            .get_receipt()
            .await
            .map_err(|e| SdkError::Transaction(e.to_string()))?;
        Ok(receipt.transaction_hash)
    }

    pub async fn submit_proof(
        &self,
        slot_id: U256,
        proof_hash: [u8; 32],
        passed: u16,
        total: u16,
    ) -> Result<FixedBytes<32>, SdkError> {
        let p = self.provider()?;
        let verifier = IEngagementVerifier::new(self.addresses.engagement_verifier, &p);
        let tx = verifier
            .submitProof(slot_id, FixedBytes::from(proof_hash), passed, total)
            .send()
            .await
            .map_err(|e| SdkError::Transaction(e.to_string()))?;
        let receipt = tx
            .get_receipt()
            .await
            .map_err(|e| SdkError::Transaction(e.to_string()))?;
        Ok(receipt.transaction_hash)
    }

    pub async fn finalize_slot(&self, slot_id: U256) -> Result<FixedBytes<32>, SdkError> {
        let p = self.provider()?;
        let verifier = IEngagementVerifier::new(self.addresses.engagement_verifier, &p);
        let tx = verifier
            .finalizeSlot(slot_id)
            .send()
            .await
            .map_err(|e| SdkError::Transaction(e.to_string()))?;
        let receipt = tx
            .get_receipt()
            .await
            .map_err(|e| SdkError::Transaction(e.to_string()))?;
        Ok(receipt.transaction_hash)
    }

    pub async fn get_verification_score(&self, slot_id: U256) -> Result<U256, SdkError> {
        let p = self.provider()?;
        let verifier = IEngagementVerifier::new(self.addresses.engagement_verifier, &p);
        verifier
            .getVerificationScore(slot_id)
            .call()
            .await
            .map_err(|e| SdkError::Rpc(e.to_string()))
    }

    pub async fn get_active_slots(&self, user: Address) -> Result<Vec<U256>, SdkError> {
        let p = self.provider()?;
        let registry = ICommitmentRegistry::new(self.addresses.commitment_registry, &p);
        let slots = registry
            .getActiveSlots(user)
            .call()
            .await
            .map_err(|e| SdkError::Rpc(e.to_string()))?;
        Ok(slots.to_vec())
    }

    pub async fn is_slot_finalized(&self, slot_id: U256) -> Result<bool, SdkError> {
        let p = self.provider()?;
        let verifier = IEngagementVerifier::new(self.addresses.engagement_verifier, &p);
        verifier
            .slotFinalized(slot_id)
            .call()
            .await
            .map_err(|e| SdkError::Rpc(e.to_string()))
    }
}
