use crate::types::{Challenge, ChallengeType};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionTracker {

    pub switches: Vec<DateTime<Utc>>,

    pub idle_periods: Vec<(DateTime<Utc>, DateTime<Utc>)>,

    pub first_activity: Option<String>,

    pub total_key_events: u64,
}

impl SessionTracker {
    pub fn new() -> Self {
        Self {
            switches: Vec::new(),
            idle_periods: Vec::new(),
            first_activity: None,
            total_key_events: 0,
        }
    }

    pub fn record_switch(&mut self) {
        self.switches.push(Utc::now());
    }

    pub fn record_idle_start(&mut self) -> DateTime<Utc> {
        let now = Utc::now();
        self.idle_periods.push((now, now)); 
        now
    }

    pub fn record_idle_end(&mut self) {
        if let Some(last) = self.idle_periods.last_mut() {
            last.1 = Utc::now();
        }
    }

    pub fn set_first_activity(&mut self, description: &str) {
        if self.first_activity.is_none() {
            self.first_activity = Some(description.to_string());
        }
    }

    pub fn recent_switch_count(&self, minutes: i64) -> u32 {
        let cutoff = Utc::now() - chrono::Duration::minutes(minutes);
        self.switches.iter().filter(|t| **t > cutoff).count() as u32
    }
}

impl Default for SessionTracker {
    fn default() -> Self {
        Self::new()
    }
}

pub fn generate(challenge_id: u64, tracker: &SessionTracker) -> Challenge {
    let recent_switches = tracker.recent_switch_count(5);

    let prompt = "How many times did you switch windows/tabs in the last 5 minutes? (approximate number)".to_string();
    let expected = recent_switches.to_string();

    Challenge {
        id: challenge_id,
        challenge_type: ChallengeType::Continuity,
        prompt,
        expected_answer: Some(expected),
        issued_at: Utc::now(),
        response_window_ms: 15_000, 
    }
}

pub fn evaluate(challenge: &Challenge, response: &str) -> bool {
    let expected: i32 = challenge
        .expected_answer
        .as_ref()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);

    let actual: i32 = response.trim().parse().unwrap_or(-100);

    (actual - expected).abs() <= 2
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_evaluate_exact() {
        let challenge = Challenge {
            id: 1,
            challenge_type: ChallengeType::Continuity,
            prompt: "test".to_string(),
            expected_answer: Some("5".to_string()),
            issued_at: Utc::now(),
            response_window_ms: 15_000,
        };

        assert!(evaluate(&challenge, "5"));
        assert!(evaluate(&challenge, "3")); 
        assert!(evaluate(&challenge, "7")); 
        assert!(!evaluate(&challenge, "10")); 
    }

    #[test]
    fn test_session_tracker_switches() {
        let mut tracker = SessionTracker::new();
        tracker.record_switch();
        tracker.record_switch();
        tracker.record_switch();

        assert_eq!(tracker.recent_switch_count(5), 3);
    }
}
