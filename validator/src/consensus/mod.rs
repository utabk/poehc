#[derive(Debug)]
pub enum ValidationResult {

    Approved(String),

    Rejected(String),

    Suspicious(String),
}

pub fn validate_proof(
    challenges_passed: u16,
    challenges_total: u16,
) -> ValidationResult {

    if challenges_total == 0 {
        return ValidationResult::Rejected("Zero challenges submitted".to_string());
    }

    let pass_rate = challenges_passed as f64 / challenges_total as f64;

    if challenges_passed == 0 {
        return ValidationResult::Rejected(format!(
            "0% pass rate ({}/{})",
            challenges_passed, challenges_total
        ));
    }

    if pass_rate == 1.0 && challenges_total > 20 {
        return ValidationResult::Suspicious(format!(
            "Perfect 100% score with {} challenges — statistically unlikely",
            challenges_total
        ));
    }

    if pass_rate < 0.3 {
        return ValidationResult::Rejected(format!(
            "Pass rate {:.0}% below 30% minimum ({}/{})",
            pass_rate * 100.0,
            challenges_passed,
            challenges_total
        ));
    }

    if challenges_total > 500 {
        return ValidationResult::Suspicious(format!(
            "Unusually high challenge count: {}",
            challenges_total
        ));
    }

    ValidationResult::Approved(format!(
        "Pass rate {:.0}% ({}/{})",
        pass_rate * 100.0,
        challenges_passed,
        challenges_total
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normal_proof_approved() {
        match validate_proof(9, 10) {
            ValidationResult::Approved(_) => {}
            other => panic!("Expected Approved, got {:?}", other),
        }
    }

    #[test]
    fn test_zero_challenges_rejected() {
        match validate_proof(0, 0) {
            ValidationResult::Rejected(_) => {}
            other => panic!("Expected Rejected, got {:?}", other),
        }
    }

    #[test]
    fn test_zero_passed_rejected() {
        match validate_proof(0, 10) {
            ValidationResult::Rejected(_) => {}
            other => panic!("Expected Rejected, got {:?}", other),
        }
    }

    #[test]
    fn test_low_pass_rate_rejected() {
        match validate_proof(2, 10) {
            ValidationResult::Rejected(_) => {}
            other => panic!("Expected Rejected, got {:?}", other),
        }
    }

    #[test]
    fn test_perfect_large_batch_suspicious() {
        match validate_proof(25, 25) {
            ValidationResult::Suspicious(_) => {}
            other => panic!("Expected Suspicious, got {:?}", other),
        }
    }

    #[test]
    fn test_perfect_small_batch_ok() {
        match validate_proof(10, 10) {
            ValidationResult::Approved(_) => {}
            other => panic!("Expected Approved, got {:?}", other),
        }
    }

    #[test]
    fn test_borderline_pass_rate() {

        match validate_proof(3, 10) {
            ValidationResult::Approved(_) => {}
            other => panic!("Expected Approved, got {:?}", other),
        }
    }
}
