use crate::types::KeyEvent;
use sha2::{Digest, Sha256};
use std::collections::VecDeque;

const MAX_BUFFER_SIZE: usize = 2000;

const HISTOGRAM_BINS: usize = 50;

const MAX_INTERVAL_MS: f64 = 2000.0;

#[derive(Debug)]
pub struct KeystrokeDynamics {

    events: VecDeque<KeyEvent>,

    intervals: Vec<f64>,

    hold_durations: Vec<f64>,
}

impl KeystrokeDynamics {
    pub fn new() -> Self {
        Self {
            events: VecDeque::with_capacity(MAX_BUFFER_SIZE),
            intervals: Vec::new(),
            hold_durations: Vec::new(),
        }
    }

    pub fn record_event(&mut self, event: KeyEvent) {

        if event.is_press {
            if let Some(prev) = self.events.iter().rev().find(|e| e.is_press) {
                let interval = (event.timestamp - prev.timestamp).num_milliseconds() as f64;
                if interval > 0.0 && interval < MAX_INTERVAL_MS {
                    self.intervals.push(interval);
                }
            }
        }

        if !event.is_press {
            if let Some(press) = self
                .events
                .iter()
                .rev()
                .find(|e| e.is_press && e.key_code == event.key_code)
            {
                let hold = (event.timestamp - press.timestamp).num_milliseconds() as f64;
                if hold > 0.0 && hold < 1000.0 {
                    self.hold_durations.push(hold);
                }
            }
        }

        self.events.push_back(event);
        if self.events.len() > MAX_BUFFER_SIZE {
            self.events.pop_front();
        }
    }

    pub fn compute_entropy(&self) -> f64 {
        if self.intervals.len() < 10 {
            return 0.0;
        }

        let mut bins = vec![0u32; HISTOGRAM_BINS];
        let bin_width = MAX_INTERVAL_MS / HISTOGRAM_BINS as f64;

        for &interval in &self.intervals {
            let bin = ((interval / bin_width) as usize).min(HISTOGRAM_BINS - 1);
            bins[bin] += 1;
        }

        let total = self.intervals.len() as f64;
        let mut entropy = 0.0;

        for &count in &bins {
            if count > 0 {
                let p = count as f64 / total;
                entropy -= p * p.log2();
            }
        }

        entropy
    }

    pub fn compute_rhythm_hash(&self) -> [u8; 32] {
        let mut bins = vec![0u32; HISTOGRAM_BINS];
        let bin_width = MAX_INTERVAL_MS / HISTOGRAM_BINS as f64;

        for &interval in &self.intervals {
            let bin = ((interval / bin_width) as usize).min(HISTOGRAM_BINS - 1);
            bins[bin] += 1;
        }

        let mut hasher = Sha256::new();
        for &count in &bins {
            hasher.update(count.to_le_bytes());
        }
        hasher.finalize().into()
    }

    pub fn match_against_baseline(&self, baseline: &KeystrokeProfile) -> f64 {
        if self.intervals.len() < 20 || baseline.mean_interval == 0.0 {
            return 0.0;
        }

        let current_mean = self.intervals.iter().sum::<f64>() / self.intervals.len() as f64;
        let current_std = std_dev(&self.intervals);

        let mean_diff = (current_mean - baseline.mean_interval).abs() / baseline.mean_interval;
        let mean_score = (1.0 - mean_diff).max(0.0);

        let std_diff = if baseline.std_interval > 0.0 {
            (current_std - baseline.std_interval).abs() / baseline.std_interval
        } else {
            1.0
        };
        let std_score = (1.0 - std_diff).max(0.0);

        let hold_score = if !self.hold_durations.is_empty() && baseline.mean_hold > 0.0 {
            let current_hold_mean =
                self.hold_durations.iter().sum::<f64>() / self.hold_durations.len() as f64;
            let hold_diff = (current_hold_mean - baseline.mean_hold).abs() / baseline.mean_hold;
            (1.0 - hold_diff).max(0.0)
        } else {
            0.5 
        };

        0.4 * mean_score + 0.3 * std_score + 0.3 * hold_score
    }

    pub fn build_profile(&self) -> KeystrokeProfile {
        let mean_interval = if self.intervals.is_empty() {
            0.0
        } else {
            self.intervals.iter().sum::<f64>() / self.intervals.len() as f64
        };

        let std_interval = std_dev(&self.intervals);

        let mean_hold = if self.hold_durations.is_empty() {
            0.0
        } else {
            self.hold_durations.iter().sum::<f64>() / self.hold_durations.len() as f64
        };

        KeystrokeProfile {
            mean_interval,
            std_interval,
            mean_hold,
            sample_count: self.intervals.len() as u32,
        }
    }

    pub fn event_count(&self) -> usize {
        self.events.len()
    }

    pub fn clear(&mut self) {
        self.events.clear();
        self.intervals.clear();
        self.hold_durations.clear();
    }
}

impl Default for KeystrokeDynamics {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct KeystrokeProfile {
    pub mean_interval: f64,
    pub std_interval: f64,
    pub mean_hold: f64,
    pub sample_count: u32,
}

fn std_dev(values: &[f64]) -> f64 {
    if values.len() < 2 {
        return 0.0;
    }
    let mean = values.iter().sum::<f64>() / values.len() as f64;
    let variance = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / (values.len() - 1) as f64;
    variance.sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn make_press(time_offset_ms: i64, key_code: u32) -> KeyEvent {
        KeyEvent {
            timestamp: Utc::now() + chrono::Duration::milliseconds(time_offset_ms),
            key_code,
            is_press: true,
        }
    }

    fn make_release(time_offset_ms: i64, key_code: u32) -> KeyEvent {
        KeyEvent {
            timestamp: Utc::now() + chrono::Duration::milliseconds(time_offset_ms),
            key_code,
            is_press: false,
        }
    }

    #[test]
    fn test_empty_entropy_is_zero() {
        let kd = KeystrokeDynamics::new();
        assert_eq!(kd.compute_entropy(), 0.0);
    }

    #[test]
    fn test_entropy_increases_with_variety() {
        let mut kd = KeystrokeDynamics::new();
        let base = Utc::now();

        for i in 0..100 {
            kd.record_event(KeyEvent {
                timestamp: base + chrono::Duration::milliseconds(i * 100),
                key_code: 65,
                is_press: true,
            });
        }
        let uniform_entropy = kd.compute_entropy();

        let mut kd2 = KeystrokeDynamics::new();
        for i in 0..100 {
            let jitter = if i % 3 == 0 { 50 } else if i % 3 == 1 { 200 } else { 500 };
            kd2.record_event(KeyEvent {
                timestamp: base + chrono::Duration::milliseconds(i * 100 + jitter),
                key_code: 65,
                is_press: true,
            });
        }
        let varied_entropy = kd2.compute_entropy();

        assert!(varied_entropy > uniform_entropy, "Varied typing should have higher entropy");
    }

    #[test]
    fn test_rhythm_hash_deterministic() {
        let mut kd = KeystrokeDynamics::new();
        let base = Utc::now();

        for i in 0..50 {
            kd.record_event(KeyEvent {
                timestamp: base + chrono::Duration::milliseconds(i * 120),
                key_code: 65 + (i as u32 % 26),
                is_press: true,
            });
        }

        let hash1 = kd.compute_rhythm_hash();
        let hash2 = kd.compute_rhythm_hash();
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_build_and_match_profile() {
        let mut kd = KeystrokeDynamics::new();
        let base = Utc::now();

        for i in 0..100 {
            let offset = i * 120; 
            kd.record_event(KeyEvent {
                timestamp: base + chrono::Duration::milliseconds(offset),
                key_code: 65,
                is_press: true,
            });
            kd.record_event(KeyEvent {
                timestamp: base + chrono::Duration::milliseconds(offset + 80), 
                key_code: 65,
                is_press: false,
            });
        }

        let profile = kd.build_profile();
        assert!(profile.mean_interval > 0.0);
        assert!(profile.sample_count > 0);

        let score = kd.match_against_baseline(&profile);
        assert!(score > 0.5, "Self-match should be above 0.5, got {}", score);
    }
}
