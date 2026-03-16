use crate::behavioral::keystroke::KeystrokeProfile;
use crate::types::{Challenge, ChallengeType, KeyEvent};
use chrono::Utc;
use rand::seq::SliceRandom;

const CHALLENGE_WORDS: &[&str] = &[
    "elephant",
    "mountain",
    "keyboard",
    "protocol",
    "beautiful",
    "challenge",
    "tomorrow",
    "spectrum",
    "discover",
    "original",
    "patience",
    "triangle",
    "universe",
    "valuable",
    "wonderful",
];

pub fn generate(challenge_id: u64) -> Challenge {
    let mut rng = rand::thread_rng();
    let word = CHALLENGE_WORDS.choose(&mut rng).unwrap_or(&"keyboard");

    Challenge {
        id: challenge_id,
        challenge_type: ChallengeType::CognitiveSignature,
        prompt: format!("Type the following word: {}", word),
        expected_answer: Some(word.to_string()),
        issued_at: Utc::now(),
        response_window_ms: 15_000, 
    }
}

pub fn evaluate_typing(
    expected_word: &str,
    typed_word: &str,
    keystrokes: &[KeyEvent],
    baseline: Option<&KeystrokeProfile>,
) -> f64 {

    let word_correct = typed_word.trim().to_lowercase() == expected_word.to_lowercase();
    let word_score = if word_correct { 1.0 } else { 0.0 };

    let dynamics_score = if let Some(baseline) = baseline {
        evaluate_keystroke_match(keystrokes, baseline)
    } else {
        0.5 
    };

    0.5 * word_score + 0.5 * dynamics_score
}

fn evaluate_keystroke_match(keystrokes: &[KeyEvent], baseline: &KeystrokeProfile) -> f64 {
    let press_events: Vec<&KeyEvent> = keystrokes.iter().filter(|e| e.is_press).collect();

    if press_events.len() < 3 {
        return 0.0;
    }

    let mut intervals = Vec::new();
    for i in 1..press_events.len() {
        let dt = (press_events[i].timestamp - press_events[i - 1].timestamp).num_milliseconds() as f64;
        if dt > 0.0 && dt < 2000.0 {
            intervals.push(dt);
        }
    }

    if intervals.is_empty() || baseline.mean_interval == 0.0 {
        return 0.0;
    }

    let mean = intervals.iter().sum::<f64>() / intervals.len() as f64;
    let diff_ratio = (mean - baseline.mean_interval).abs() / baseline.mean_interval;

    (1.0 - diff_ratio).max(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_challenge() {
        let challenge = generate(42);
        assert_eq!(challenge.challenge_type, ChallengeType::CognitiveSignature);
        assert!(challenge.expected_answer.is_some());
        assert!(challenge.prompt.starts_with("Type the following word:"));
    }

    #[test]
    fn test_evaluate_correct_no_baseline() {
        let score = evaluate_typing("keyboard", "keyboard", &[], None);

        assert!((score - 0.75).abs() < 0.01);
    }

    #[test]
    fn test_evaluate_wrong_word() {
        let score = evaluate_typing("keyboard", "keybord", &[], None);

        assert!((score - 0.25).abs() < 0.01);
    }
}
