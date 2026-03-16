use crate::behavioral::keystroke::KeystrokeProfile;
use crate::types::{ChallengeType, EngagementSession};
use std::collections::HashMap;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CognitiveProfile {

    pub keystroke_baseline: Option<KeystrokeProfile>,

    pub avg_response_times: HashMap<String, f64>,

    pub mouse_entropy_baseline: f64,

    pub session_count: u32,
}

impl CognitiveProfile {
    pub fn new() -> Self {
        Self {
            keystroke_baseline: None,
            avg_response_times: HashMap::new(),
            mouse_entropy_baseline: 0.0,
            session_count: 0,
        }
    }

    pub fn update(&mut self, session: &EngagementSession, keystroke_profile: Option<&KeystrokeProfile>) {
        self.session_count += 1;
        let alpha = if self.session_count <= 5 {
            1.0 / self.session_count as f64 
        } else {
            0.1 
        };

        if let Some(new_profile) = keystroke_profile {
            self.keystroke_baseline = Some(match &self.keystroke_baseline {
                None => new_profile.clone(),
                Some(existing) => KeystrokeProfile {
                    mean_interval: ema(existing.mean_interval, new_profile.mean_interval, alpha),
                    std_interval: ema(existing.std_interval, new_profile.std_interval, alpha),
                    mean_hold: ema(existing.mean_hold, new_profile.mean_hold, alpha),
                    sample_count: existing.sample_count + new_profile.sample_count,
                },
            });
        }

        let mut type_times: HashMap<String, Vec<f64>> = HashMap::new();
        for result in &session.challenge_results {
            if let Some(rt) = result.response_time_ms {
                let key = challenge_type_key(result.challenge_type);
                type_times.entry(key).or_default().push(rt as f64);
            }
        }

        for (ctype, times) in type_times {
            let new_avg = times.iter().sum::<f64>() / times.len() as f64;
            let current = self.avg_response_times.get(&ctype).copied().unwrap_or(new_avg);
            self.avg_response_times
                .insert(ctype, ema(current, new_avg, alpha));
        }

        if !session.behavioral_snapshots.is_empty() {
            let avg_entropy: f64 = session
                .behavioral_snapshots
                .iter()
                .map(|s| s.mouse_entropy)
                .sum::<f64>()
                / session.behavioral_snapshots.len() as f64;
            self.mouse_entropy_baseline = ema(self.mouse_entropy_baseline, avg_entropy, alpha);
        }
    }

    pub fn match_score(&self, session: &EngagementSession) -> f64 {
        if self.session_count == 0 {
            return 0.5; 
        }

        let mut scores = Vec::new();

        for result in &session.challenge_results {
            if let Some(rt) = result.response_time_ms {
                let key = challenge_type_key(result.challenge_type);
                if let Some(&baseline_rt) = self.avg_response_times.get(&key) {
                    if baseline_rt > 0.0 {
                        let diff_ratio = ((rt as f64) - baseline_rt).abs() / baseline_rt;
                        scores.push((1.0 - diff_ratio).max(0.0));
                    }
                }
            }
        }

        if !session.behavioral_snapshots.is_empty() && self.mouse_entropy_baseline > 0.0 {
            let avg_entropy: f64 = session
                .behavioral_snapshots
                .iter()
                .map(|s| s.mouse_entropy)
                .sum::<f64>()
                / session.behavioral_snapshots.len() as f64;

            let diff_ratio =
                (avg_entropy - self.mouse_entropy_baseline).abs() / self.mouse_entropy_baseline;
            scores.push((1.0 - diff_ratio).max(0.0));
        }

        if scores.is_empty() {
            0.5
        } else {
            scores.iter().sum::<f64>() / scores.len() as f64
        }
    }
}

impl Default for CognitiveProfile {
    fn default() -> Self {
        Self::new()
    }
}

fn ema(old: f64, new: f64, alpha: f64) -> f64 {
    (1.0 - alpha) * old + alpha * new
}

fn challenge_type_key(ct: ChallengeType) -> String {
    match ct {
        ChallengeType::ContextRecall => "context_recall".to_string(),
        ChallengeType::Continuity => "continuity".to_string(),
        ChallengeType::EmbodiedPresence => "embodied_presence".to_string(),
        ChallengeType::CognitiveSignature => "cognitive_signature".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::*;
    use chrono::Utc;

    fn make_session_with_times(response_times: &[u64]) -> EngagementSession {
        let now = Utc::now();
        let results: Vec<ChallengeResult> = response_times
            .iter()
            .enumerate()
            .map(|(i, &rt)| ChallengeResult {
                challenge_id: i as u64,
                challenge_type: ChallengeType::ContextRecall,
                issued_at: now,
                responded_at: Some(now + chrono::Duration::milliseconds(rt as i64)),
                correct: true,
                response_time_ms: Some(rt),
            })
            .collect();

        EngagementSession {
            slot_id: 1,
            start: now,
            end: now + chrono::Duration::hours(1),
            level: CommitmentLevel::DeepFocus,
            challenge_results: results,
            behavioral_snapshots: vec![BehavioralSnapshot {
                timestamp: now,
                keystroke_entropy: 3.0,
                mouse_entropy: 2.5,
                typing_rhythm_hash: [0u8; 32],
                key_event_count: 100,
                mouse_event_count: 200,
            }],
        }
    }

    #[test]
    fn test_new_profile() {
        let profile = CognitiveProfile::new();
        assert_eq!(profile.session_count, 0);
        assert!(profile.keystroke_baseline.is_none());
    }

    #[test]
    fn test_update_profile() {
        let mut profile = CognitiveProfile::new();
        let session = make_session_with_times(&[1500, 1200, 1800, 1400]);

        profile.update(&session, None);

        assert_eq!(profile.session_count, 1);
        assert!(profile.avg_response_times.contains_key("context_recall"));
        assert!(profile.mouse_entropy_baseline > 0.0);
    }

    #[test]
    fn test_match_score_same_pattern() {
        let mut profile = CognitiveProfile::new();

        for _ in 0..5 {
            let session = make_session_with_times(&[1500, 1400, 1600, 1450]);
            profile.update(&session, None);
        }

        let test_session = make_session_with_times(&[1500, 1400, 1550, 1500]);
        let score = profile.match_score(&test_session);
        assert!(score > 0.7, "Similar pattern should match well, got {}", score);
    }

    #[test]
    fn test_match_score_different_pattern() {
        let mut profile = CognitiveProfile::new();

        for _ in 0..5 {
            let session = make_session_with_times(&[500, 600, 550, 480]);
            profile.update(&session, None);
        }

        let test_session = make_session_with_times(&[3000, 3500, 4000, 2800]);
        let score = profile.match_score(&test_session);
        assert!(score < 0.5, "Different pattern should score low, got {}", score);
    }
}
