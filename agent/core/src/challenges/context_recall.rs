use crate::types::{Challenge, ChallengeType};
use chrono::Utc;
use rand::Rng;

#[derive(Debug, Clone)]
pub struct WorkContext {

    pub recent_windows: Vec<String>,

    pub recent_switch_count: u32,

    pub last_activity_description: String,
}

impl WorkContext {
    pub fn new() -> Self {
        Self {
            recent_windows: Vec::new(),
            recent_switch_count: 0,
            last_activity_description: String::new(),
        }
    }
}

impl Default for WorkContext {
    fn default() -> Self {
        Self::new()
    }
}

pub fn generate(challenge_id: u64, context: &WorkContext) -> Challenge {
    let mut rng = rand::thread_rng();

    let (prompt, expected) = if !context.recent_windows.is_empty() {
        let idx = rng.gen_range(0..context.recent_windows.len());
        let window = &context.recent_windows[idx];
        (
            format!("Which of these applications have you used recently? Type the name: {}", window),
            Some(window.clone()),
        )
    } else {

        (
            "Are you currently working? Type 'yes' to confirm.".to_string(),
            Some("yes".to_string()),
        )
    };

    Challenge {
        id: challenge_id,
        challenge_type: ChallengeType::ContextRecall,
        prompt,
        expected_answer: expected,
        issued_at: Utc::now(),
        response_window_ms: 10_000, 
    }
}

pub fn evaluate(challenge: &Challenge, response: &str) -> bool {
    match &challenge.expected_answer {
        Some(expected) => response.trim().to_lowercase() == expected.trim().to_lowercase(),
        None => !response.trim().is_empty(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_with_context() {
        let mut ctx = WorkContext::new();
        ctx.recent_windows.push("VS Code".to_string());
        ctx.recent_windows.push("Chrome".to_string());

        let challenge = generate(1, &ctx);
        assert_eq!(challenge.challenge_type, ChallengeType::ContextRecall);
        assert!(challenge.expected_answer.is_some());
    }

    #[test]
    fn test_generate_fallback() {
        let ctx = WorkContext::new();
        let challenge = generate(1, &ctx);
        assert_eq!(challenge.expected_answer, Some("yes".to_string()));
    }

    #[test]
    fn test_evaluate_correct() {
        let challenge = Challenge {
            id: 1,
            challenge_type: ChallengeType::ContextRecall,
            prompt: "Test".to_string(),
            expected_answer: Some("VS Code".to_string()),
            issued_at: Utc::now(),
            response_window_ms: 10_000,
        };
        assert!(evaluate(&challenge, "vs code"));
        assert!(evaluate(&challenge, "  VS Code  "));
        assert!(!evaluate(&challenge, "notepad"));
    }
}
