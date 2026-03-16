use crate::chain::contracts::{ICommitmentRegistry, IEngagementVerifier, ITIMEToken};
use alloy::network::EthereumWallet;
use alloy::primitives::{Address, FixedBytes, U256};
use alloy::providers::{Provider, ProviderBuilder};
use alloy::signers::local::PrivateKeySigner;
use std::str::FromStr;

#[derive(Debug, Clone)]
pub struct ContractAddresses {
    pub registry: Address,
    pub verifier: Address,
    pub time_token: Address,
}

#[derive(Debug, Clone)]
pub struct ChainClient {
    pub rpc_url: String,
    pub addresses: ContractAddresses,
    pub signer: PrivateKeySigner,
}

#[derive(Debug)]
pub struct RegisteredSlot {
    pub slot_id: U256,
    pub tx_hash: FixedBytes<32>,
}

#[derive(Debug)]
pub struct ProofSubmissionResult {
    pub tx_hash: FixedBytes<32>,
}

#[derive(Debug)]
pub struct FinalizationResult {
    pub tx_hash: FixedBytes<32>,
    pub time_earned: U256,
}

impl ChainClient {

    pub fn new(rpc_url: &str, private_key: &str, addresses: ContractAddresses) -> Result<Self, String> {
        let signer = PrivateKeySigner::from_str(private_key)
            .map_err(|e| format!("Invalid private key: {}", e))?;

        Ok(Self {
            rpc_url: rpc_url.to_string(),
            addresses,
            signer,
        })
    }

    pub fn local_anvil(addresses: ContractAddresses) -> Self {
        let signer = PrivateKeySigner::from_str(
            "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80",
        )
        .expect("Valid Anvil default key");

        Self {
            rpc_url: "http://127.0.0.1:8545".to_string(),
            addresses,
            signer,
        }
    }

    async fn build_provider(
        &self,
    ) -> Result<
        impl Provider + Clone,
        String,
    > {
        let wallet = EthereumWallet::from(self.signer.clone());
        let provider = ProviderBuilder::new()
            .wallet(wallet)
            .connect_http(self.rpc_url.parse().map_err(|e| format!("Bad URL: {}", e))?);

        Ok(provider)
    }

    pub async fn register_slot(
        &self,
        start: u64,
        end: u64,
        client_addr: Address,
        level: u8, 
    ) -> Result<RegisteredSlot, String> {
        let provider = self.build_provider().await?;
        let registry = ICommitmentRegistry::new(self.addresses.registry, &provider);

        let level_enum = match level {
            0 => ICommitmentRegistry::CommitmentLevel::DEEP_FOCUS,
            1 => ICommitmentRegistry::CommitmentLevel::ACTIVE_ENGAGEMENT,
            2 => ICommitmentRegistry::CommitmentLevel::BACKGROUND,
            _ => return Err("Invalid commitment level".to_string()),
        };

        let tx = registry
            .registerSlot(start, end, client_addr, level_enum)
            .send()
            .await
            .map_err(|e| format!("Failed to send registerSlot tx: {}", e))?;

        let receipt = tx
            .get_receipt()
            .await
            .map_err(|e| format!("Failed to get receipt: {}", e))?;

        let slot_id = receipt
            .inner
            .logs()
            .iter()
            .find_map(|log| {
                log.log_decode::<ICommitmentRegistry::SlotRegistered>()
                    .ok()
                    .map(|decoded| decoded.inner.data.slotId)
            })
            .unwrap_or(U256::ZERO);

        Ok(RegisteredSlot {
            slot_id,
            tx_hash: receipt.transaction_hash,
        })
    }

    pub async fn submit_proof(
        &self,
        slot_id: U256,
        proof_hash: [u8; 32],
        challenges_passed: u16,
        challenges_total: u16,
    ) -> Result<ProofSubmissionResult, String> {
        let provider = self.build_provider().await?;
        let verifier = IEngagementVerifier::new(self.addresses.verifier, &provider);

        let tx = verifier
            .submitProof(
                slot_id,
                FixedBytes::from(proof_hash),
                challenges_passed,
                challenges_total,
            )
            .send()
            .await
            .map_err(|e| format!("Failed to send submitProof tx: {}", e))?;

        let receipt = tx
            .get_receipt()
            .await
            .map_err(|e| format!("Failed to get receipt: {}", e))?;

        Ok(ProofSubmissionResult {
            tx_hash: receipt.transaction_hash,
        })
    }

    pub async fn finalize_slot(&self, slot_id: U256) -> Result<FinalizationResult, String> {
        let provider = self.build_provider().await?;
        let verifier = IEngagementVerifier::new(self.addresses.verifier, &provider);

        let tx = verifier
            .finalizeSlot(slot_id)
            .send()
            .await
            .map_err(|e| format!("Failed to send finalizeSlot tx: {}", e))?;

        let receipt = tx
            .get_receipt()
            .await
            .map_err(|e| format!("Failed to get receipt: {}", e))?;

        let time_earned = receipt
            .inner
            .logs()
            .iter()
            .find_map(|log| {
                log.log_decode::<IEngagementVerifier::SlotFinalized>()
                    .ok()
                    .map(|decoded| decoded.inner.data.timeEarned)
            })
            .unwrap_or(U256::ZERO);

        Ok(FinalizationResult {
            tx_hash: receipt.transaction_hash,
            time_earned,
        })
    }

    pub async fn get_time_balance(&self, account: Address) -> Result<U256, String> {
        let provider = self.build_provider().await?;
        let token = ITIMEToken::new(self.addresses.time_token, &provider);

        let balance = token
            .balanceOf(account)
            .call()
            .await
            .map_err(|e| format!("Failed to read balance: {}", e))?;

        Ok(balance)
    }

    pub async fn get_verification_score(&self, slot_id: U256) -> Result<U256, String> {
        let provider = self.build_provider().await?;
        let verifier = IEngagementVerifier::new(self.addresses.verifier, &provider);

        let score = verifier
            .getVerificationScore(slot_id)
            .call()
            .await
            .map_err(|e| format!("Failed to read score: {}", e))?;

        Ok(score)
    }

    pub async fn get_proof_count(&self, slot_id: U256) -> Result<U256, String> {
        let provider = self.build_provider().await?;
        let verifier = IEngagementVerifier::new(self.addresses.verifier, &provider);

        let count = verifier
            .getProofCount(slot_id)
            .call()
            .await
            .map_err(|e| format!("Failed to read proof count: {}", e))?;

        Ok(count)
    }

    pub async fn is_slot_finalized(&self, slot_id: U256) -> Result<bool, String> {
        let provider = self.build_provider().await?;
        let verifier = IEngagementVerifier::new(self.addresses.verifier, &provider);

        let finalized = verifier
            .slotFinalized(slot_id)
            .call()
            .await
            .map_err(|e| format!("Failed to read finalized: {}", e))?;

        Ok(finalized)
    }

    pub fn address(&self) -> Address {
        self.signer.address()
    }
}
