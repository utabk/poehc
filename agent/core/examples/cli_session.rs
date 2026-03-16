use chrono::Utc;
use poehc_agent_core::behavioral::aggregator;
use poehc_agent_core::behavioral::keystroke::KeystrokeDynamics;
use poehc_agent_core::behavioral::mouse::MouseDynamics;
use poehc_agent_core::challenges::{cognitive_signature, context_recall, continuity};
use poehc_agent_core::challenges::context_recall::WorkContext;
use poehc_agent_core::challenges::continuity::SessionTracker;
use poehc_agent_core::cognitive::CognitiveProfile;
use poehc_agent_core::crypto;
use poehc_agent_core::scheduler::ChallengeScheduler;
use poehc_agent_core::types::*;
use std::io::{self, BufRead, Write};
use std::time::Duration;

fn main() {
    println!("╔══════════════════════════════════════════════════════════╗");
    println!("║       PoEHC Engagement Agent — CLI Test Harness         ║");
    println!("║  Proof of Exclusive Human Commitment Protocol v0.1      ║");
    println!("╚══════════════════════════════════════════════════════════╝");
    println!();

    let duration_minutes = std::env::args()
        .skip_while(|a| a != "--duration-minutes")
        .nth(1)
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(3);

    println!("Session duration: {} minutes", duration_minutes);
    println!("Commitment level: Deep Focus (3.0x multiplier)");
    println!();
    println!("The agent will issue challenges at random intervals.");
    println!("Type your response and press Enter when prompted.");
    println!("Between challenges, type normally — your keystroke patterns are being measured.");
    println!();
    println!("Starting session in 3 seconds...");
    std::thread::sleep(Duration::from_secs(3));

    let session_start = Utc::now();
    let session_end = session_start + chrono::Duration::minutes(duration_minutes as i64);

    let mut scheduler = ChallengeScheduler::new();
    let mut keystroke = KeystrokeDynamics::new();
    let mut mouse = MouseDynamics::new();
    let mut work_context = WorkContext::new();
    let mut session_tracker = SessionTracker::new();
    let mut challenge_results: Vec<ChallengeResult> = Vec::new();
    let mut behavioral_snapshots: Vec<BehavioralSnapshot> = Vec::new();
    let mut next_snapshot_time = Utc::now() + chrono::Duration::seconds(30);

    work_context.recent_windows.push("Terminal".to_string());
    work_context.recent_windows.push("CLI Session".to_string());
    session_tracker.set_first_activity("CLI engagement session");

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Session ACTIVE. Type anything between challenges to generate behavioral data.");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    let stdin = io::stdin();

    loop {
        let now = Utc::now();

        if now >= session_end {
            break;
        }

        if now >= next_snapshot_time {
            let snap = aggregator::snapshot(&keystroke, &mouse);
            println!(
                "  [snapshot] keystroke_entropy={:.2} mouse_entropy={:.2} keys={} mouse={}",
                snap.keystroke_entropy, snap.mouse_entropy, snap.key_event_count, snap.mouse_event_count
            );
            behavioral_snapshots.push(snap);
            next_snapshot_time = now + chrono::Duration::seconds(30);
        }

        if scheduler.is_challenge_due() {
            let challenge_type = scheduler.select_challenge_type();
            let challenge_id = scheduler.challenge_count();

            let challenge = match challenge_type {
                ChallengeType::ContextRecall => {
                    context_recall::generate(challenge_id, &work_context)
                }
                ChallengeType::Continuity => {
                    continuity::generate(challenge_id, &session_tracker)
                }
                ChallengeType::CognitiveSignature => {
                    cognitive_signature::generate(challenge_id)
                }
                ChallengeType::EmbodiedPresence => {

                    Challenge {
                        id: challenge_id,
                        challenge_type: ChallengeType::EmbodiedPresence,
                        prompt: "Type 'here' to confirm your presence.".to_string(),
                        expected_answer: Some("here".to_string()),
                        issued_at: Utc::now(),
                        response_window_ms: 10_000,
                    }
                }
            };

            println!();
            println!("┌─── CHALLENGE #{} ({:?}) ───", challenge_id, challenge_type);
            println!("│ {}", challenge.prompt);
            print!("│ > ");
            io::stdout().flush().unwrap();

            let issued_at = Utc::now();

            let mut response = String::new();
            stdin.lock().read_line(&mut response).unwrap();
            let responded_at = Utc::now();

            let response_time = (responded_at - issued_at).num_milliseconds() as u64;

            for (i, ch) in response.chars().enumerate() {
                keystroke.record_event(KeyEvent {
                    timestamp: issued_at + chrono::Duration::milliseconds(i as i64 * 80),
                    key_code: ch as u32,
                    is_press: true,
                });
                keystroke.record_event(KeyEvent {
                    timestamp: issued_at + chrono::Duration::milliseconds(i as i64 * 80 + 60),
                    key_code: ch as u32,
                    is_press: false,
                });
            }

            let correct = match challenge_type {
                ChallengeType::ContextRecall => {
                    context_recall::evaluate(&challenge, &response)
                }
                ChallengeType::Continuity => {
                    continuity::evaluate(&challenge, &response)
                }
                ChallengeType::CognitiveSignature => {
                    response.trim().to_lowercase()
                        == challenge.expected_answer.as_deref().unwrap_or("").to_lowercase()
                }
                ChallengeType::EmbodiedPresence => {
                    response.trim().to_lowercase() == "here"
                }
            };

            let result_str = if correct { "CORRECT" } else { "INCORRECT" };
            println!(
                "│ {} ({}ms response time)",
                result_str, response_time
            );
            println!("└───────────────────────────────────");

            challenge_results.push(ChallengeResult {
                challenge_id,
                challenge_type,
                issued_at,
                responded_at: Some(responded_at),
                correct,
                response_time_ms: Some(response_time),
            });

            scheduler.advance();
        } else {

            let time_left = (session_end - now).num_seconds();
            let next_challenge_in = (scheduler.next_challenge_time() - now).num_seconds().max(0);
            print!(
                "\r  [{}s left | next challenge in ~{}s] Type anything: ",
                time_left, next_challenge_in
            );
            io::stdout().flush().unwrap();

            let mut input = String::new();
            stdin.lock().read_line(&mut input).unwrap();

            let now = Utc::now();
            for (i, ch) in input.chars().enumerate() {
                keystroke.record_event(KeyEvent {
                    timestamp: now + chrono::Duration::milliseconds(i as i64 * 100),
                    key_code: ch as u32,
                    is_press: true,
                });
            }

            let mouse_now = Utc::now();
            for i in 0..5 {
                mouse.record_event(MouseEvent {
                    timestamp: mouse_now + chrono::Duration::milliseconds(i * 50),
                    x: (i as f64 * 10.0) + (input.len() as f64),
                    y: (i as f64 * 5.0),
                    event_type: MouseEventType::Move,
                });
            }
        }
    }

    println!();
    println!("╔══════════════════════════════════════════════════════════╗");
    println!("║                   SESSION COMPLETE                       ║");
    println!("╚══════════════════════════════════════════════════════════╝");
    println!();

    let final_snap = aggregator::snapshot(&keystroke, &mouse);
    behavioral_snapshots.push(final_snap);

    let session = EngagementSession {
        slot_id: 1,
        start: session_start,
        end: session_end,
        level: CommitmentLevel::DeepFocus,
        challenge_results: challenge_results.clone(),
        behavioral_snapshots: behavioral_snapshots.clone(),
    };

    let score = crypto::compute_verification_score(&session);
    let proof_hash = crypto::compute_proof_hash(&session);
    let proof_hex = crypto::hex(&proof_hash);

    let keystroke_profile = keystroke.build_profile();
    let mut cognitive_profile = CognitiveProfile::new();
    cognitive_profile.update(&session, Some(&keystroke_profile));

    let total = challenge_results.len();
    let passed = challenge_results.iter().filter(|r| r.correct).count();
    let avg_response_time: f64 = challenge_results
        .iter()
        .filter_map(|r| r.response_time_ms.map(|t| t as f64))
        .sum::<f64>()
        / total.max(1) as f64;

    let duration_hours = (session_end - session_start).num_seconds() as f64 / 3600.0;
    let time_earned = duration_hours * CommitmentLevel::DeepFocus.multiplier() * score.composite;

    println!("── Challenge Results ──");
    println!("  Challenges issued:  {}", total);
    println!("  Challenges passed:  {} ({:.0}%)", passed, (passed as f64 / total.max(1) as f64) * 100.0);
    println!("  Avg response time:  {:.0}ms", avg_response_time);
    println!();
    println!("── Behavioral Metrics ──");
    println!("  Keystroke entropy:  {:.3}", keystroke.compute_entropy());
    println!("  Key events:         {}", keystroke.event_count());
    println!("  Mouse events:       {}", mouse.event_count());
    println!("  Snapshots taken:    {}", behavioral_snapshots.len());
    println!();
    println!("── Verification Score ──");
    println!("  Challenge score:    {:.3} (weight: 70%)", score.challenge_score);
    println!("  Behavioral score:   {:.3} (weight: 30%)", score.behavioral_score);
    println!("  Composite score:    {:.3}", score.composite);
    println!();
    println!("── TIME Token Calculation ──");
    println!("  Duration:           {:.2} hours", duration_hours);
    println!("  Level multiplier:   {:.1}x (Deep Focus)", CommitmentLevel::DeepFocus.multiplier());
    println!("  Verification score: {:.3}", score.composite);
    println!("  TIME earned:        {:.4} TIME", time_earned);
    println!();
    println!("── Proof Hash (for on-chain submission) ──");
    println!("  0x{}", proof_hex);
    println!();
    println!("── Keystroke Profile ──");
    println!("  Mean interval:      {:.1}ms", keystroke_profile.mean_interval);
    println!("  Std deviation:      {:.1}ms", keystroke_profile.std_interval);
    println!("  Mean hold time:     {:.1}ms", keystroke_profile.mean_hold);
    println!("  Sample count:       {}", keystroke_profile.sample_count);
    println!();
    println!("Session data would be submitted to CommitmentRegistry slot #1");
    println!("on Ethereum L2 via the EngagementVerifier contract.");
}
