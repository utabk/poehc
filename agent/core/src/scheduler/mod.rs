use crate::types::ChallengeType;
use chrono::{DateTime, Duration, Utc};
use hmac::{Hmac, Mac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

const MIN_INTERVAL_SECS: u64 = 15;

const MAX_INTERVAL_SECS: u64 = 180;

#[derive(Debug)]
pub struct ChallengeScheduler {

    secret_key: [u8; 32],

    challenge_count: u64,

    last_challenge_time: DateTime<Utc>,
}

impl ChallengeScheduler {

    pub fn new() -> Self {
        let mut key = [0u8; 32];
        rand::Rng::fill(&mut rand::thread_rng(), &mut key);
        Self {
            secret_key: key,
            challenge_count: 0,
            last_challenge_time: Utc::now(),
        }
    }

    pub fn with_seed(secret_key: [u8; 32], block_hash: [u8; 32]) -> Self {

        let mut mixed_key = [0u8; 32];
        for i in 0..32 {
            mixed_key[i] = secret_key[i] ^ block_hash[i];
        }
        Self {
            secret_key: mixed_key,
            challenge_count: 0,
            last_challenge_time: Utc::now(),
        }
    }

    pub fn next_challenge_time(&self) -> DateTime<Utc> {
        let interval = self.compute_interval();
        self.last_challenge_time + Duration::seconds(interval as i64)
    }

    pub fn is_challenge_due(&self) -> bool {
        Utc::now() >= self.next_challenge_time()
    }

    pub fn select_challenge_type(&self) -> ChallengeType {
        let hash = self.compute_vrf_output(b"type");
        let value = u16::from_le_bytes([hash[0], hash[1]]) % 100;

        match value {
            0..=69 => ChallengeType::ContextRecall,
            70..=84 => ChallengeType::Continuity,
            85..=94 => ChallengeType::EmbodiedPresence,
            _ => ChallengeType::CognitiveSignature,
        }
    }

    pub fn advance(&mut self) {
        self.challenge_count += 1;
        self.last_challenge_time = Utc::now();
    }

    pub fn challenge_count(&self) -> u64 {
        self.challenge_count
    }

    fn compute_interval(&self) -> u64 {
        let hash = self.compute_vrf_output(b"interval");
        let raw = u64::from_le_bytes([
            hash[0], hash[1], hash[2], hash[3], hash[4], hash[5], hash[6], hash[7],
        ]);
        let range = MAX_INTERVAL_SECS - MIN_INTERVAL_SECS;
        MIN_INTERVAL_SECS + (raw % range)
    }

    fn compute_vrf_output(&self, purpose: &[u8]) -> [u8; 32] {
        let mut mac =
            HmacSha256::new_from_slice(&self.secret_key).expect("HMAC accepts any key length");
        mac.update(&self.challenge_count.to_le_bytes());
        mac.update(purpose);
        let result = mac.finalize();
        let mut output = [0u8; 32];
        output.copy_from_slice(&result.into_bytes());
        output
    }
}

impl Default for ChallengeScheduler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interval_within_bounds() {
        let scheduler = ChallengeScheduler::new();
        for _ in 0..100 {
            let interval = scheduler.compute_interval();
            assert!(interval >= MIN_INTERVAL_SECS);
            assert!(interval < MAX_INTERVAL_SECS);
        }
    }

    #[test]
    fn test_challenge_type_distribution() {
        let key = [42u8; 32];
        let block = [0u8; 32];
        let mut scheduler = ChallengeScheduler::with_seed(key, block);

        let mut counts = [0u32; 4]; 
        let n = 10000;

        for _ in 0..n {
            match scheduler.select_challenge_type() {
                ChallengeType::ContextRecall => counts[0] += 1,
                ChallengeType::Continuity => counts[1] += 1,
                ChallengeType::EmbodiedPresence => counts[2] += 1,
                ChallengeType::CognitiveSignature => counts[3] += 1,
            }
            scheduler.challenge_count += 1;
        }

        let total = n as f64;
        let cr_pct = counts[0] as f64 / total;
        let cont_pct = counts[1] as f64 / total;
        let emb_pct = counts[2] as f64 / total;
        let cog_pct = counts[3] as f64 / total;

        assert!((cr_pct - 0.70).abs() < 0.05, "ContextRecall: {:.2}%", cr_pct * 100.0);
        assert!((cont_pct - 0.15).abs() < 0.05, "Continuity: {:.2}%", cont_pct * 100.0);
        assert!((emb_pct - 0.10).abs() < 0.05, "EmbodiedPresence: {:.2}%", emb_pct * 100.0);
        assert!((cog_pct - 0.05).abs() < 0.05, "CognitiveSignature: {:.2}%", cog_pct * 100.0);
    }

    #[test]
    fn test_deterministic_with_seed() {
        let key = [1u8; 32];
        let block = [2u8; 32];

        let s1 = ChallengeScheduler::with_seed(key, block);
        let s2 = ChallengeScheduler::with_seed(key, block);

        assert_eq!(s1.compute_interval(), s2.compute_interval());
        assert_eq!(s1.select_challenge_type(), s2.select_challenge_type());
    }

    #[test]
    fn test_advance_changes_output() {
        let mut scheduler = ChallengeScheduler::new();
        let interval1 = scheduler.compute_interval();
        let type1 = scheduler.select_challenge_type();

        scheduler.advance();

        let interval2 = scheduler.compute_interval();

        assert_eq!(scheduler.challenge_count(), 1);
        let _ = (interval1, interval2, type1); 
    }
}
