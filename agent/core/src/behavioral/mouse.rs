use crate::types::{MouseEvent, MouseEventType};
use std::collections::VecDeque;

const MAX_BUFFER_SIZE: usize = 5000;
const VELOCITY_BINS: usize = 40;
const MAX_VELOCITY: f64 = 5000.0; 

#[derive(Debug)]
pub struct MouseDynamics {
    events: VecDeque<MouseEvent>,
    velocities: Vec<f64>,
    direction_changes: u32,
}

impl MouseDynamics {
    pub fn new() -> Self {
        Self {
            events: VecDeque::with_capacity(MAX_BUFFER_SIZE),
            velocities: Vec::new(),
            direction_changes: 0,
        }
    }

    pub fn record_event(&mut self, event: MouseEvent) {
        if event.event_type == MouseEventType::Move {
            if let Some(prev) = self.events.back() {
                if prev.event_type == MouseEventType::Move {
                    let dt = (event.timestamp - prev.timestamp).num_milliseconds() as f64 / 1000.0;
                    if dt > 0.0 {
                        let dx = event.x - prev.x;
                        let dy = event.y - prev.y;
                        let distance = (dx * dx + dy * dy).sqrt();
                        let velocity = distance / dt;

                        if velocity < MAX_VELOCITY {
                            self.velocities.push(velocity);
                        }

                        if self.events.len() >= 2 {
                            let prev2 = &self.events[self.events.len() - 2];
                            if prev2.event_type == MouseEventType::Move {
                                let prev_dx = prev.x - prev2.x;
                                let prev_dy = prev.y - prev2.y;

                                if prev_dx * dx + prev_dy * dy < 0.0 {
                                    self.direction_changes += 1;
                                }
                            }
                        }
                    }
                }
            }
        }

        self.events.push_back(event);
        if self.events.len() > MAX_BUFFER_SIZE {
            self.events.pop_front();
        }
    }

    pub fn compute_entropy(&self) -> f64 {
        if self.velocities.len() < 20 {
            return 0.0;
        }

        let mut bins = vec![0u32; VELOCITY_BINS];
        let bin_width = MAX_VELOCITY / VELOCITY_BINS as f64;

        for &v in &self.velocities {
            let bin = ((v / bin_width) as usize).min(VELOCITY_BINS - 1);
            bins[bin] += 1;
        }

        let total = self.velocities.len() as f64;
        let mut entropy = 0.0;

        for &count in &bins {
            if count > 0 {
                let p = count as f64 / total;
                entropy -= p * p.log2();
            }
        }

        entropy
    }

    pub fn detect_automation(&self) -> bool {
        if self.velocities.len() < 50 {
            return false; 
        }

        let entropy = self.compute_entropy();

        if entropy < 0.5 {
            return true;
        }

        if self.direction_changes == 0 && self.events.len() > 100 {
            return true;
        }

        false
    }

    pub fn event_count(&self) -> usize {
        self.events.len()
    }

    pub fn direction_change_count(&self) -> u32 {
        self.direction_changes
    }

    pub fn clear(&mut self) {
        self.events.clear();
        self.velocities.clear();
        self.direction_changes = 0;
    }
}

impl Default for MouseDynamics {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_empty_entropy_is_zero() {
        let md = MouseDynamics::new();
        assert_eq!(md.compute_entropy(), 0.0);
    }

    #[test]
    fn test_linear_movement_low_entropy() {
        let mut md = MouseDynamics::new();
        let base = Utc::now();

        for i in 0..200 {
            md.record_event(MouseEvent {
                timestamp: base + chrono::Duration::milliseconds(i * 10),
                x: i as f64 * 5.0,
                y: 100.0,
                event_type: MouseEventType::Move,
            });
        }

        let entropy = md.compute_entropy();
        assert!(entropy < 2.0, "Linear movement should have low entropy, got {}", entropy);
    }

    #[test]
    fn test_natural_movement_higher_entropy() {
        let mut md = MouseDynamics::new();
        let base = Utc::now();

        for i in 0..200 {
            let jitter_x = if i % 4 == 0 { 20.0 } else if i % 4 == 1 { -10.0 } else if i % 4 == 2 { 5.0 } else { -30.0 };
            let jitter_y = if i % 3 == 0 { 15.0 } else if i % 3 == 1 { -25.0 } else { 8.0 };
            let dt = if i % 2 == 0 { 10 } else { 25 };

            md.record_event(MouseEvent {
                timestamp: base + chrono::Duration::milliseconds(i * dt),
                x: i as f64 * 3.0 + jitter_x,
                y: i as f64 * 2.0 + jitter_y,
                event_type: MouseEventType::Move,
            });
        }

        let entropy = md.compute_entropy();
        assert!(entropy > 0.5, "Natural movement should have some entropy, got {}", entropy);
        assert!(md.direction_change_count() > 0, "Natural movement should have direction changes");
    }

    #[test]
    fn test_detect_automation_constant_velocity() {
        let mut md = MouseDynamics::new();
        let base = Utc::now();

        for i in 0..200 {
            md.record_event(MouseEvent {
                timestamp: base + chrono::Duration::milliseconds(i * 10),
                x: i as f64 * 10.0,
                y: i as f64 * 10.0,
                event_type: MouseEventType::Move,
            });
        }

        assert!(md.detect_automation(), "Constant velocity linear movement should be detected as automation");
    }
}
