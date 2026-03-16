mod consensus;
mod contracts;

use alloy::providers::Provider;
use clap::Parser;
use tracing::{info, warn, error};

#[derive(Parser, Debug)]
#[command(name = "poehc-validator", about = "PoEHC Validator Node")]
struct Args {

    #[arg(long, default_value = "http://127.0.0.1:8545")]
    rpc_url: String,

    #[arg(long, env = "VALIDATOR_PRIVATE_KEY")]
    private_key: String,

    #[arg(long, env = "REGISTRY_ADDRESS")]
    registry: String,

    #[arg(long, env = "VERIFIER_ADDRESS")]
    verifier: String,

    #[arg(long, env = "VALIDATOR_REGISTRY_ADDRESS")]
    validator_registry: String,

    #[arg(long, env = "TIME_TOKEN_ADDRESS")]
    time_token: String,

    #[arg(long, default_value = "5")]
    poll_interval: u64,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let args = Args::parse();

    info!("╔══════════════════════════════════════════════╗");
    info!("║       PoEHC Validator Node v0.1              ║");
    info!("╚══════════════════════════════════════════════╝");
    info!("RPC URL: {}", args.rpc_url);
    info!("Poll interval: {}s", args.poll_interval);

    let registry_addr: alloy::primitives::Address = args.registry.parse()
        .map_err(|e| format!("Invalid registry address: {}", e))?;
    let verifier_addr: alloy::primitives::Address = args.verifier.parse()
        .map_err(|e| format!("Invalid verifier address: {}", e))?;
    let validator_registry_addr: alloy::primitives::Address = args.validator_registry.parse()
        .map_err(|e| format!("Invalid validator registry address: {}", e))?;

    let signer: alloy::signers::local::PrivateKeySigner = args.private_key.parse()
        .map_err(|e| format!("Invalid private key: {}", e))?;
    let wallet = alloy::network::EthereumWallet::from(signer.clone());
    let provider = alloy::providers::ProviderBuilder::new()
        .wallet(wallet)
        .connect_http(args.rpc_url.parse()?);

    info!("Validator address: {:?}", signer.address());

    let verifier = contracts::IEngagementVerifier::new(verifier_addr, &provider);
    let validator_reg = contracts::IValidatorRegistry::new(validator_registry_addr, &provider);

    let mut last_block: u64 = provider.get_block_number().await?;
    info!("Starting from block {}", last_block);

    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(args.poll_interval)).await;

        let current_block = match provider.get_block_number().await {
            Ok(b) => b,
            Err(e) => {
                warn!("Failed to get block number: {}", e);
                continue;
            }
        };

        if current_block <= last_block {
            continue;
        }

        info!("Processing blocks {} to {}", last_block + 1, current_block);

        let filter = verifier
            .ProofSubmitted_filter()
            .from_block(last_block + 1)
            .to_block(current_block);

        match filter.query().await {
            Ok(events) => {
                for (event, log) in &events {
                    info!(
                        "ProofSubmitted: slot={}, passed={}, total={} (block {})",
                        event.slotId,
                        event.passed,
                        event.total,
                        log.block_number.unwrap_or(0)
                    );

                    let validation = consensus::validate_proof(
                        event.passed,
                        event.total,
                    );

                    match validation {
                        consensus::ValidationResult::Approved(reason) => {
                            info!("  -> APPROVED: {}", reason);

                            match validator_reg
                                .reportValidation(event.slotId, true)
                                .send()
                                .await
                            {
                                Ok(tx) => {
                                    match tx.get_receipt().await {
                                        Ok(receipt) => {
                                            info!("  -> Report submitted: {:?}", receipt.transaction_hash);
                                        }
                                        Err(e) => warn!("  -> Report receipt failed: {}", e),
                                    }
                                }
                                Err(e) => warn!("  -> Report submission failed: {}", e),
                            }
                        }
                        consensus::ValidationResult::Rejected(reason) => {
                            warn!("  -> REJECTED: {}", reason);

                            match validator_reg
                                .reportValidation(event.slotId, false)
                                .send()
                                .await
                            {
                                Ok(tx) => {
                                    match tx.get_receipt().await {
                                        Ok(receipt) => {
                                            info!("  -> Rejection report submitted: {:?}", receipt.transaction_hash);
                                        }
                                        Err(e) => warn!("  -> Report receipt failed: {}", e),
                                    }
                                }
                                Err(e) => warn!("  -> Report submission failed: {}", e),
                            }
                        }
                        consensus::ValidationResult::Suspicious(reason) => {
                            warn!("  -> SUSPICIOUS: {} (flagged for review)", reason);
                        }
                    }
                }
            }
            Err(e) => {
                error!("Failed to query events: {}", e);
            }
        }

        let finalized_filter = verifier
            .SlotFinalized_filter()
            .from_block(last_block + 1)
            .to_block(current_block);

        match finalized_filter.query().await {
            Ok(events) => {
                for (event, _log) in &events {
                    let time_ether = event.timeEarned / alloy::primitives::U256::from(10).pow(alloy::primitives::U256::from(18));
                    info!(
                        "SlotFinalized: slot={}, owner={:?}, earned={} TIME, score={} bps",
                        event.slotId, event.owner, time_ether, event.scoreBps
                    );
                }
            }
            Err(e) => {
                error!("Failed to query finalized events: {}", e);
            }
        }

        last_block = current_block;
    }
}
