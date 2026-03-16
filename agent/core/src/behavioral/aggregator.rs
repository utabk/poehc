use crate::behavioral::keystroke::KeystrokeDynamics;
use crate::behavioral::mouse::MouseDynamics;
use crate::types::BehavioralSnapshot;
use chrono::Utc;

pub fn snapshot(keystroke: &KeystrokeDynamics, mouse: &MouseDynamics) -> BehavioralSnapshot {
    BehavioralSnapshot {
        timestamp: Utc::now(),
        keystroke_entropy: keystroke.compute_entropy(),
        mouse_entropy: mouse.compute_entropy(),
        typing_rhythm_hash: keystroke.compute_rhythm_hash(),
        key_event_count: keystroke.event_count() as u32,
        mouse_event_count: mouse.event_count() as u32,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snapshot_from_empty() {
        let kd = KeystrokeDynamics::new();
        let md = MouseDynamics::new();

        let snap = snapshot(&kd, &md);
        assert_eq!(snap.keystroke_entropy, 0.0);
        assert_eq!(snap.mouse_entropy, 0.0);
        assert_eq!(snap.key_event_count, 0);
        assert_eq!(snap.mouse_event_count, 0);
    }
}
