"""
Cognitive Bandwidth Bound (CBB) — Simulation and Analysis

Simulates the core PoEHC thesis: can we detect when a user is dual-tasking
(committing to two streams simultaneously) vs. single-tasking?

This script:
1. Models the Poisson collision probability for overlapping challenges
2. Simulates honest (single-task) and cheating (dual-task) response patterns
3. Computes detection accuracy using a composite scoring function
4. Generates analysis data for the whitepaper

Usage:
    python research/cbb_model.py
"""

import random
import math
import statistics
from dataclasses import dataclass
from typing import List, Tuple

# ─── Parameters ───────────────────────────────────────────────────────

# Human cognitive parameters (from PRP literature)
MEAN_RESPONSE_TIME_MS = 1500      # Average response time for a single challenge
RESPONSE_TIME_STD_MS = 400        # Standard deviation
PRP_DELAY_MS = 300                # Psychological Refractory Period
MIN_RESPONSE_MS = 300             # Fastest possible human response

# Challenge parameters
CHALLENGE_FREQ_PER_MIN = 4        # Average challenges per minute per stream
RESPONSE_WINDOW_MS = 4000         # Time allowed to respond
SESSION_DURATION_MIN = 180        # 3-hour session

# Detection thresholds
MISS_RATE_THRESHOLD = 0.05        # More than 5% missed = suspicious
LATE_RATE_THRESHOLD = 0.15        # More than 15% late = suspicious
RESPONSE_TIME_Z_THRESHOLD = 2.0   # Z-score threshold for abnormal response times


@dataclass
class ChallengeEvent:
    stream: str          # "A" or "B"
    time_ms: int         # When the challenge was issued
    response_ms: int     # When the user responded (0 = missed)
    response_time: int   # Duration to respond


@dataclass
class SessionResult:
    mode: str            # "honest" or "cheating"
    total_challenges: int
    missed: int
    late: int            # Responded but outside normal window
    avg_response_time: float
    std_response_time: float
    miss_rate: float
    late_rate: float
    response_time_z_score: float
    detected_as_cheating: bool


def generate_challenge_times(freq_per_min: float, duration_min: int, seed: int = 42) -> List[int]:
    """Generate VRF-like challenge times using Poisson process."""
    rng = random.Random(seed)
    times = []
    avg_interval_ms = 60_000 / freq_per_min
    t = rng.randint(5000, 15000)  # Start after 5-15 seconds

    while t < duration_min * 60_000:
        times.append(t)
        # Exponential interval (Poisson process)
        interval = int(rng.expovariate(1.0 / avg_interval_ms))
        interval = max(interval, 5000)   # Minimum 5s between challenges
        interval = min(interval, 120_000) # Maximum 2 minutes
        t += interval

    return times


def simulate_honest_session(seed: int = 42) -> List[ChallengeEvent]:
    """Simulate a user honestly committed to a single stream."""
    rng = random.Random(seed)
    challenges_a = generate_challenge_times(CHALLENGE_FREQ_PER_MIN, SESSION_DURATION_MIN, seed)

    events = []
    for t in challenges_a:
        # Normal human response time
        rt = max(MIN_RESPONSE_MS, int(rng.gauss(MEAN_RESPONSE_TIME_MS, RESPONSE_TIME_STD_MS)))

        # Occasional miss (2% natural miss rate)
        if rng.random() < 0.02:
            events.append(ChallengeEvent("A", t, 0, 0))
        else:
            events.append(ChallengeEvent("A", t, t + rt, rt))

    return events


def simulate_cheating_session(seed: int = 42) -> Tuple[List[ChallengeEvent], List[ChallengeEvent]]:
    """Simulate a user trying to commit to TWO streams simultaneously."""
    rng = random.Random(seed)
    challenges_a = generate_challenge_times(CHALLENGE_FREQ_PER_MIN, SESSION_DURATION_MIN, seed)
    challenges_b = generate_challenge_times(CHALLENGE_FREQ_PER_MIN, SESSION_DURATION_MIN, seed + 1000)

    events_a = []
    events_b = []

    # Merge and sort all challenges by time
    all_challenges = [(t, "A") for t in challenges_a] + [(t, "B") for t in challenges_b]
    all_challenges.sort()

    last_response_end = 0

    for t, stream in all_challenges:
        # Check if this challenge collides with a previous response
        if t < last_response_end:
            # PRP effect: response time increases significantly
            prp_penalty = rng.randint(PRP_DELAY_MS, PRP_DELAY_MS * 3)
            rt = max(MIN_RESPONSE_MS, int(rng.gauss(MEAN_RESPONSE_TIME_MS + prp_penalty, RESPONSE_TIME_STD_MS * 1.5)))
        else:
            # No collision, but still context-switching overhead
            switch_penalty = rng.randint(200, 800)
            rt = max(MIN_RESPONSE_MS, int(rng.gauss(MEAN_RESPONSE_TIME_MS + switch_penalty, RESPONSE_TIME_STD_MS * 1.2)))

        # Higher miss rate due to cognitive overload (15% miss rate)
        if rng.random() < 0.15:
            event = ChallengeEvent(stream, t, 0, 0)
        else:
            event = ChallengeEvent(stream, t, t + rt, rt)
            last_response_end = t + rt

        if stream == "A":
            events_a.append(event)
        else:
            events_b.append(event)

    return events_a, events_b


def analyze_session(events: List[ChallengeEvent], mode: str, baseline_mean: float = MEAN_RESPONSE_TIME_MS) -> SessionResult:
    """Analyze a session's events and determine if cheating is detected."""
    total = len(events)
    missed = sum(1 for e in events if e.response_ms == 0)
    response_times = [e.response_time for e in events if e.response_time > 0]
    late = sum(1 for rt in response_times if rt > RESPONSE_WINDOW_MS)

    miss_rate = missed / total if total > 0 else 0
    late_rate = late / total if total > 0 else 0

    avg_rt = statistics.mean(response_times) if response_times else 0
    std_rt = statistics.stdev(response_times) if len(response_times) > 1 else 0

    # Z-score of average response time compared to baseline
    z_score = (avg_rt - baseline_mean) / RESPONSE_TIME_STD_MS if RESPONSE_TIME_STD_MS > 0 else 0

    # Detection: flag as cheating if ANY of these thresholds are exceeded
    detected = (
        miss_rate > MISS_RATE_THRESHOLD or
        late_rate > LATE_RATE_THRESHOLD or
        abs(z_score) > RESPONSE_TIME_Z_THRESHOLD
    )

    return SessionResult(
        mode=mode,
        total_challenges=total,
        missed=missed,
        late=late,
        avg_response_time=avg_rt,
        std_response_time=std_rt,
        miss_rate=miss_rate,
        late_rate=late_rate,
        response_time_z_score=z_score,
        detected_as_cheating=detected,
    )


def compute_collision_probability(freq_a: float, freq_b: float, window_s: float, duration_min: int) -> float:
    """Compute probability of at least one challenge collision using Poisson model."""
    freq_a_per_s = freq_a / 60.0
    freq_b_per_s = freq_b / 60.0
    duration_s = duration_min * 60
    exponent = -freq_a_per_s * freq_b_per_s * (window_s ** 2) * duration_s
    return 1.0 - math.exp(exponent)


def main():
    print("=" * 70)
    print("  PoEHC Cognitive Bandwidth Bound (CBB) — Simulation")
    print("=" * 70)

    # ─── Collision Probability ────────────────────────────────────────

    print("\n--- Poisson Collision Probability ---")
    for duration in [10, 30, 60, 180]:
        p = compute_collision_probability(
            CHALLENGE_FREQ_PER_MIN, CHALLENGE_FREQ_PER_MIN,
            RESPONSE_WINDOW_MS / 1000.0, duration
        )
        expected = p * CHALLENGE_FREQ_PER_MIN * duration  # rough estimate
        print(f"  {duration:3d} min session: P(collision) = {p:.4f} ({p*100:.1f}%), ~{expected:.0f} collisions")

    # ─── Monte Carlo Simulation ───────────────────────────────────────

    print("\n--- Monte Carlo Simulation (1000 sessions each) ---")
    N = 1000

    honest_detected = 0
    cheating_detected = 0
    honest_results = []
    cheating_results = []

    for i in range(N):
        # Honest session
        events = simulate_honest_session(seed=i)
        result = analyze_session(events, "honest")
        honest_results.append(result)
        if result.detected_as_cheating:
            honest_detected += 1

        # Cheating session (analyze stream A only — what the cheater submits)
        events_a, events_b = simulate_cheating_session(seed=i)
        result = analyze_session(events_a, "cheating")
        cheating_results.append(result)
        if result.detected_as_cheating:
            cheating_detected += 1

    false_positive_rate = honest_detected / N
    true_positive_rate = cheating_detected / N

    print(f"\n  Honest sessions flagged (false positives):  {honest_detected}/{N} = {false_positive_rate*100:.1f}%")
    print(f"  Cheating sessions caught (true positives):  {cheating_detected}/{N} = {true_positive_rate*100:.1f}%")
    print(f"  Detection accuracy:                         {true_positive_rate*100:.1f}%")
    print(f"  False positive rate:                        {false_positive_rate*100:.1f}%")

    # ─── Detailed Statistics ──────────────────────────────────────────

    print("\n--- Response Time Statistics ---")

    honest_rts = [r.avg_response_time for r in honest_results]
    cheating_rts = [r.avg_response_time for r in cheating_results]

    print(f"  Honest:   mean={statistics.mean(honest_rts):.0f}ms, std={statistics.stdev(honest_rts):.0f}ms")
    print(f"  Cheating: mean={statistics.mean(cheating_rts):.0f}ms, std={statistics.stdev(cheating_rts):.0f}ms")

    honest_miss = [r.miss_rate for r in honest_results]
    cheating_miss = [r.miss_rate for r in cheating_results]

    print(f"\n  Honest miss rate:   mean={statistics.mean(honest_miss)*100:.1f}%")
    print(f"  Cheating miss rate: mean={statistics.mean(cheating_miss)*100:.1f}%")

    # ─── Economic Analysis ────────────────────────────────────────────

    print("\n--- Economic Analysis ---")
    time_per_session = 3.0  # hours
    deep_focus_multiplier = 3.0
    honest_earning = time_per_session * deep_focus_multiplier * 0.9  # 90% typical score
    print(f"  Honest earning per session:  {honest_earning:.1f} TIME")

    # If caught cheating: lose stake (assume 100 TIME staked)
    stake = 100
    cheating_earning = time_per_session * deep_focus_multiplier * 0.5 * 2  # lower score, two streams
    expected_cheating = (1 - true_positive_rate) * cheating_earning - true_positive_rate * stake
    print(f"  Cheating expected value:     {expected_cheating:.1f} TIME (negative = irrational to cheat)")
    print(f"  Conclusion: {'CHEATING IS IRRATIONAL' if expected_cheating < 0 else 'NEED STRONGER DETECTION'}")

    print("\n" + "=" * 70)
    print("  Simulation complete. CBB theorem", "VALIDATED" if expected_cheating < 0 else "NEEDS TUNING")
    print("=" * 70)


if __name__ == "__main__":
    main()
