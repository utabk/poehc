use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CommitmentLevel {

    DeepFocus,

    ActiveEngagement,

    Background,
}

impl CommitmentLevel {

    pub fn multiplier(&self) -> f64 {
        match self {
            Self::DeepFocus => 3.0,
            Self::ActiveEngagement => 1.5,
            Self::Background => 1.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChallengeType {

    ContextRecall,

    Continuity,

    EmbodiedPresence,

    CognitiveSignature,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Challenge {
    pub id: u64,
    pub challenge_type: ChallengeType,
    pub prompt: String,
    pub expected_answer: Option<String>,
    pub issued_at: DateTime<Utc>,

    pub response_window_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChallengeResult {
    pub challenge_id: u64,
    pub challenge_type: ChallengeType,
    pub issued_at: DateTime<Utc>,
    pub responded_at: Option<DateTime<Utc>>,
    pub correct: bool,

    pub response_time_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehavioralSnapshot {
    pub timestamp: DateTime<Utc>,
    pub keystroke_entropy: f64,
    pub mouse_entropy: f64,
    pub typing_rhythm_hash: [u8; 32],

    pub key_event_count: u32,

    pub mouse_event_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngagementSession {
    pub slot_id: u64,
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub level: CommitmentLevel,
    pub challenge_results: Vec<ChallengeResult>,
    pub behavioral_snapshots: Vec<BehavioralSnapshot>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationScore {

    pub challenge_score: f64,

    pub behavioral_score: f64,

    pub composite: f64,
}

impl VerificationScore {
    pub fn compute(challenge_score: f64, behavioral_score: f64) -> Self {
        let challenge_score = challenge_score.clamp(0.0, 1.0);
        let behavioral_score = behavioral_score.clamp(0.0, 1.0);
        let composite = 0.7 * challenge_score + 0.3 * behavioral_score;
        Self {
            challenge_score,
            behavioral_score,
            composite,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyEvent {
    pub timestamp: DateTime<Utc>,
    pub key_code: u32,
    pub is_press: bool, 
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MouseEvent {
    pub timestamp: DateTime<Utc>,
    pub x: f64,
    pub y: f64,
    pub event_type: MouseEventType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MouseEventType {
    Move,
    ButtonPress,
    ButtonRelease,
    Scroll,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_commitment_level_multiplier() {
        assert!((CommitmentLevel::DeepFocus.multiplier() - 3.0).abs() < f64::EPSILON);
        assert!((CommitmentLevel::ActiveEngagement.multiplier() - 1.5).abs() < f64::EPSILON);
        assert!((CommitmentLevel::Background.multiplier() - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_verification_score_compute() {
        let score = VerificationScore::compute(0.9, 0.8);
        let expected = 0.7 * 0.9 + 0.3 * 0.8;
        assert!((score.composite - expected).abs() < f64::EPSILON);
    }

    #[test]
    fn test_verification_score_clamps() {
        let score = VerificationScore::compute(1.5, -0.3);
        assert!((score.challenge_score - 1.0).abs() < f64::EPSILON);
        assert!((score.behavioral_score - 0.0).abs() < f64::EPSILON);
    }
}
