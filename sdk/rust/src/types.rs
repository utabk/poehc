use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum CommitmentLevel {
    DeepFocus = 0,
    ActiveEngagement = 1,
    Background = 2,
}

impl CommitmentLevel {
    pub fn multiplier(&self) -> f64 {
        match self {
            Self::DeepFocus => 3.0,
            Self::ActiveEngagement => 1.5,
            Self::Background => 1.0,
        }
    }

    pub fn multiplier_bps(&self) -> u64 {
        match self {
            Self::DeepFocus => 30000,
            Self::ActiveEngagement => 15000,
            Self::Background => 10000,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum SlotStatus {
    Pending = 0,
    Active = 1,
    Verified = 2,
    Disputed = 3,
    Expired = 4,
    Cancelled = 5,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractAddresses {
    pub time_token: alloy::primitives::Address,
    pub commitment_registry: alloy::primitives::Address,
    pub engagement_verifier: alloy::primitives::Address,
    pub validator_registry: alloy::primitives::Address,
    pub marketplace: alloy::primitives::Address,
}

pub fn estimate_time_earned(duration_hours: f64, level: CommitmentLevel, score: f64) -> f64 {
    duration_hours * level.multiplier() * score
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_multipliers() {
        assert!((CommitmentLevel::DeepFocus.multiplier() - 3.0).abs() < f64::EPSILON);
        assert!((CommitmentLevel::ActiveEngagement.multiplier() - 1.5).abs() < f64::EPSILON);
        assert!((CommitmentLevel::Background.multiplier() - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_estimate() {
        let earned = estimate_time_earned(3.0, CommitmentLevel::DeepFocus, 0.9);
        assert!((earned - 8.1).abs() < 0.001);
    }
}
