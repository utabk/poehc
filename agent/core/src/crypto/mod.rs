use crate::types::{BehavioralSnapshot, ChallengeResult, EngagementSession, VerificationScore};
use sha2::{Digest, Sha256};

pub fn hash_behavioral_snapshot(snapshot: &BehavioralSnapshot) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(snapshot.timestamp.timestamp().to_le_bytes());
    hasher.update(snapshot.keystroke_entropy.to_le_bytes());
    hasher.update(snapshot.mouse_entropy.to_le_bytes());
    hasher.update(&snapshot.typing_rhythm_hash);
    hasher.update(snapshot.key_event_count.to_le_bytes());
    hasher.update(snapshot.mouse_event_count.to_le_bytes());
    hasher.finalize().into()
}

pub fn hash_challenge_result(result: &ChallengeResult) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(result.challenge_id.to_le_bytes());
    hasher.update(result.issued_at.timestamp().to_le_bytes());
    hasher.update([result.correct as u8]);
    if let Some(rt) = result.response_time_ms {
        hasher.update(rt.to_le_bytes());
    }
    hasher.finalize().into()
}

pub fn compute_proof_hash(session: &EngagementSession) -> [u8; 32] {
    let mut hasher = Sha256::new();

    hasher.update(session.slot_id.to_le_bytes());
    hasher.update(session.start.timestamp().to_le_bytes());
    hasher.update(session.end.timestamp().to_le_bytes());

    for result in &session.challenge_results {
        let h = hash_challenge_result(result);
        hasher.update(h);
    }

    for snapshot in &session.behavioral_snapshots {
        let h = hash_behavioral_snapshot(snapshot);
        hasher.update(h);
    }

    hasher.finalize().into()
}

pub fn compute_verification_score(session: &EngagementSession) -> VerificationScore {

    let total_challenges = session.challenge_results.len() as f64;
    let passed_challenges = session
        .challenge_results
        .iter()
        .filter(|r| r.correct)
        .count() as f64;

    let challenge_score = if total_challenges > 0.0 {
        passed_challenges / total_challenges
    } else {
        0.0
    };

    let behavioral_score = if session.behavioral_snapshots.is_empty() {
        0.0
    } else {
        let avg_keystroke_entropy: f64 = session
            .behavioral_snapshots
            .iter()
            .map(|s| s.keystroke_entropy)
            .sum::<f64>()
            / session.behavioral_snapshots.len() as f64;

        let avg_mouse_entropy: f64 = session
            .behavioral_snapshots
            .iter()
            .map(|s| s.mouse_entropy)
            .sum::<f64>()
            / session.behavioral_snapshots.len() as f64;

        let keystroke_normalized = (avg_keystroke_entropy / 3.0).min(1.0);
        let mouse_normalized = (avg_mouse_entropy / 3.0).min(1.0);

        0.5 * keystroke_normalized + 0.5 * mouse_normalized
    };

    VerificationScore::compute(challenge_score, behavioral_score)
}

pub fn hex(hash: &[u8; 32]) -> String {
    hash.iter().map(|b| format!("{:02x}", b)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::*;
    use chrono::Utc;

    fn make_session(passed: usize, total: usize) -> EngagementSession {
        let now = Utc::now();
        let mut results = Vec::new();

        for i in 0..total {
            results.push(ChallengeResult {
                challenge_id: i as u64,
                challenge_type: ChallengeType::ContextRecall,
                issued_at: now,
                responded_at: Some(now),
                correct: i < passed,
                response_time_ms: Some(1500),
            });
        }

        EngagementSession {
            slot_id: 1,
            start: now,
            end: now + chrono::Duration::hours(3),
            level: CommitmentLevel::DeepFocus,
            challenge_results: results,
            behavioral_snapshots: vec![BehavioralSnapshot {
                timestamp: now,
                keystroke_entropy: 3.5,
                mouse_entropy: 2.8,
                typing_rhythm_hash: [0u8; 32],
                key_event_count: 500,
                mouse_event_count: 1000,
            }],
        }
    }

    #[test]
    fn test_proof_hash_deterministic() {
        let session = make_session(9, 10);
        let hash1 = compute_proof_hash(&session);
        let hash2 = compute_proof_hash(&session);
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_proof_hash_changes_with_results() {
        let session1 = make_session(9, 10);
        let session2 = make_session(8, 10);
        let hash1 = compute_proof_hash(&session1);
        let hash2 = compute_proof_hash(&session2);
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_verification_score_perfect() {
        let session = make_session(10, 10);
        let score = compute_verification_score(&session);
        assert!((score.challenge_score - 1.0).abs() < f64::EPSILON);
        assert!(score.composite > 0.8);
    }

    #[test]
    fn test_verification_score_partial() {
        let session = make_session(7, 10);
        let score = compute_verification_score(&session);
        assert!((score.challenge_score - 0.7).abs() < f64::EPSILON);
    }

    #[test]
    fn test_hex_formatting() {
        let hash = [0xab, 0xcd, 0xef, 0x01, 0x23, 0x45, 0x67, 0x89,
                     0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                     0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                     0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xff];
        let s = hex(&hash);
        assert!(s.starts_with("abcdef0123456789"));
        assert!(s.ends_with("ff"));
        assert_eq!(s.len(), 64);
    }
}
