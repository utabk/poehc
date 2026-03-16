use alloy::primitives::{Address, U256};
use alloy::providers::builder as provider_builder;
use alloy::providers::Provider;
use alloy::signers::local::PrivateKeySigner;
use alloy::network::{Ethereum, EthereumWallet};
use alloy::sol;
use chrono::Utc;
use poehc_agent_core::chain::client::{ChainClient, ContractAddresses};
use poehc_agent_core::crypto;
use poehc_agent_core::types::*;
use std::str::FromStr;

sol! {
    #[sol(rpc, bytecode = "")]
    contract DeployHelper {}
}

const ANVIL_KEY: &str = "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";
const RPC_URL: &str = "http://127.0.0.1:8545";

async fn deploy_contracts() -> Result<ContractAddresses, String> {

    let home = std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .unwrap_or_default();
    let forge_path = format!("{}/.foundry/bin/forge", home);
    let forge_cmd = if std::path::Path::new(&forge_path).exists() {
        forge_path
    } else {
        "forge".to_string()
    };

    let project_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent() 
        .and_then(|p| p.parent()) 
        .expect("Could not find project root");

    let contracts_dir = project_root.join("contracts");

    let output = tokio::process::Command::new(&forge_cmd)
        .args([
            "script",
            "script/Deploy.s.sol",
            "--broadcast",
            "--rpc-url",
            RPC_URL,
        ])
        .current_dir(&contracts_dir)
        .output()
        .await
        .map_err(|e| format!("Failed to run forge script: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    if !output.status.success() {
        return Err(format!("Deploy failed:\nstdout: {}\nstderr: {}", stdout, stderr));
    }

    let combined = format!("{}\n{}", stdout, stderr);

    let find_address = |prefix: &str| -> Result<Address, String> {
        combined
            .lines()
            .find(|line| line.contains(prefix))
            .and_then(|line| {
                line.split_whitespace()
                    .last()
                    .and_then(|addr| Address::from_str(addr).ok())
            })
            .ok_or_else(|| format!("Could not find {} address in deploy output:\n{}", prefix, combined))
    };

    Ok(ContractAddresses {
        registry: find_address("CommitmentRegistry deployed at:")?,
        verifier: find_address("EngagementVerifier deployed at:")?,
        time_token: find_address("TIMEToken deployed at:")?,
    })
}

#[tokio::test]
#[ignore] 
async fn test_full_e2e_flow() {
    println!("=== PoEHC End-to-End Integration Test ===\n");

    println!("[1/7] Deploying contracts to Anvil...");
    let addresses = deploy_contracts().await.expect("Deploy should succeed");
    println!("  Registry:  {:?}", addresses.registry);
    println!("  Verifier:  {:?}", addresses.verifier);
    println!("  TIMEToken: {:?}", addresses.time_token);

    println!("\n[2/7] Creating chain client...");
    let client = ChainClient::new(RPC_URL, ANVIL_KEY, addresses.clone())
        .expect("Client creation should succeed");
    println!("  Signer address: {:?}", client.address());

    println!("\n[3/7] Checking initial TIME balance...");
    let initial_balance = client
        .get_time_balance(client.address())
        .await
        .expect("Balance read should succeed");
    println!("  Initial TIME balance: {}", initial_balance);
    assert_eq!(initial_balance, U256::ZERO);

    println!("\n[4/7] Registering commitment slot...");

    let signer = PrivateKeySigner::from_str(ANVIL_KEY).unwrap();
    let wallet = EthereumWallet::from(signer);
    let provider = provider_builder::<Ethereum>()
        .wallet(wallet)
        .connect_http(RPC_URL.parse().unwrap());

    let block_ts: U256 = provider
        .raw_request("eth_getBlockByNumber".into(), ("latest", false))
        .await
        .map(|block: serde_json::Value| {
            let ts_hex = block["timestamp"].as_str().unwrap_or("0x0");
            U256::from_str_radix(ts_hex.trim_start_matches("0x"), 16).unwrap_or(U256::ZERO)
        })
        .unwrap();

    let current_ts: u64 = block_ts.to::<u64>();
    let start = current_ts + 3600; 
    let end = start + 10800;       

    let registered = client
        .register_slot(start, end, client.address(), 0) 
        .await
        .expect("Slot registration should succeed");

    println!("  Slot ID: {}", registered.slot_id);
    println!("  Tx hash: {:?}", registered.tx_hash);
    assert!(registered.slot_id > U256::ZERO);

    println!("\n[5/7] Warping time and submitting proofs...");

    let warp_to = start + 1800; 
    let _: serde_json::Value = provider
        .raw_request("evm_setNextBlockTimestamp".into(), [U256::from(warp_to)])
        .await
        .unwrap();
    let _: serde_json::Value = provider.raw_request("evm_mine".into(), ()).await.unwrap();

    let now = Utc::now();
    let session = EngagementSession {
        slot_id: 1,
        start: now,
        end: now + chrono::Duration::hours(3),
        level: CommitmentLevel::DeepFocus,
        challenge_results: vec![
            ChallengeResult {
                challenge_id: 0,
                challenge_type: ChallengeType::ContextRecall,
                issued_at: now,
                responded_at: Some(now + chrono::Duration::milliseconds(1500)),
                correct: true,
                response_time_ms: Some(1500),
            },
            ChallengeResult {
                challenge_id: 1,
                challenge_type: ChallengeType::ContextRecall,
                issued_at: now,
                responded_at: Some(now + chrono::Duration::milliseconds(1200)),
                correct: true,
                response_time_ms: Some(1200),
            },
            ChallengeResult {
                challenge_id: 2,
                challenge_type: ChallengeType::Continuity,
                issued_at: now,
                responded_at: Some(now + chrono::Duration::milliseconds(2000)),
                correct: false,
                response_time_ms: Some(2000),
            },
        ],
        behavioral_snapshots: vec![BehavioralSnapshot {
            timestamp: now,
            keystroke_entropy: 3.2,
            mouse_entropy: 2.8,
            typing_rhythm_hash: [42u8; 32],
            key_event_count: 500,
            mouse_event_count: 1200,
        }],
    };

    let proof_hash = crypto::compute_proof_hash(&session);

    for i in 0..3 {
        let (passed, total) = match i {
            0 => (9u16, 10u16),
            1 => (10, 10),
            2 => (8, 10),
            _ => unreachable!(),
        };

        let result = client
            .submit_proof(registered.slot_id, proof_hash, passed, total)
            .await
            .expect("Proof submission should succeed");

        println!("  Proof {} submitted: {:?}", i + 1, result.tx_hash);
    }

    let proof_count = client
        .get_proof_count(registered.slot_id)
        .await
        .expect("Proof count read should succeed");
    println!("  Total proofs: {}", proof_count);
    assert_eq!(proof_count, U256::from(3));

    let score = client
        .get_verification_score(registered.slot_id)
        .await
        .expect("Score read should succeed");
    println!("  Verification score: {} bps ({}%)", score, score / U256::from(100));

    println!("\n[6/7] Warping to slot end and finalizing...");

    let warp_to_end = end + 1;
    let _: serde_json::Value = provider
        .raw_request("evm_setNextBlockTimestamp".into(), [U256::from(warp_to_end)])
        .await
        .unwrap();
    let _: serde_json::Value = provider.raw_request("evm_mine".into(), ()).await.unwrap();

    let finalization = client
        .finalize_slot(registered.slot_id)
        .await
        .expect("Finalization should succeed");

    println!("  Finalization tx: {:?}", finalization.tx_hash);
    println!("  TIME earned: {} wei", finalization.time_earned);

    let is_finalized = client
        .is_slot_finalized(registered.slot_id)
        .await
        .expect("Finalized check should succeed");
    assert!(is_finalized, "Slot should be finalized");

    println!("\n[7/7] Checking final TIME balance...");
    let final_balance = client
        .get_time_balance(client.address())
        .await
        .expect("Balance read should succeed");

    let time_in_ether = final_balance / U256::from(10).pow(U256::from(18));
    println!("  Final TIME balance: {} wei ({} TIME)", final_balance, time_in_ether);
    assert!(final_balance > U256::ZERO, "Should have earned TIME tokens");

    let expected = U256::from(81) * U256::from(10).pow(U256::from(17));
    assert_eq!(final_balance, expected, "Should earn exactly 8.1 TIME");

    println!("\n=== Test PASSED ===");
    println!("Successfully completed the full PoEHC flow:");
    println!("  1. Deployed 5 contracts to Anvil");
    println!("  2. Registered a 3-hour Deep Focus commitment slot");
    println!("  3. Submitted 3 engagement proofs (90% score)");
    println!("  4. Finalized slot and minted 8.1 TIME tokens");
    println!("  5. Verified correct token balance");
}
