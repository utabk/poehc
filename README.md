# PoEHC — Proof of Exclusive Human Commitment

Right now, there is no way to cryptographically prove that a person dedicated their time exclusively to one task. Freelancers can secretly work for two clients at once. Remote employees can appear "at work" while doing something else. Every existing solution is either self-reported (easily faked) or surveillance-based (invasive).

PoEHC fixes this. It's a protocol that proves exclusive human commitment without cameras, screen recording, or any form of surveillance. Think of it like Bitcoin's double-spend prevention, but for human time instead of money.

## How it works

1. You register a time commitment on-chain (like reserving a time slot)
2. A local agent on your device measures behavioral patterns (keystroke rhythm, mouse movement) and issues random challenges
3. If you try to commit to two things at once, the challenges collide and your response patterns become statistically detectable
4. When your session is verified, you earn TIME tokens

The key insight: your brain has a hard limit called the Psychological Refractory Period. You physically cannot process two decision-requiring tasks simultaneously. We exploit this as a cryptographic anchor.

## What's in this repo

**contracts/** — Five Solidity smart contracts built with Foundry:
- TIMEToken: ERC-20 token minted by verified commitment
- CommitmentRegistry: Non-overlapping time slots (the "double-spend prevention")
- EngagementVerifier: Accepts proofs and mints TIME
- ValidatorRegistry: Validator staking and committee assignment
- CommitmentMarketplace: Connects clients with workers

**agent/core/** — Rust library that powers the engagement agent:
- Keystroke dynamics and mouse entropy measurement
- VRF-scheduled challenge system
- Proof computation and on-chain submission via Alloy

**validator/** — Rust node that watches the chain, validates proof plausibility, and submits reports

**sdk/** — TypeScript and Rust SDKs for integrating with the protocol

**research/** — Python simulation of the Cognitive Bandwidth Bound theorem (1000 sessions, 100% detection rate, 0% false positives)

**docs/** — Whitepaper

## Quick start

```bash
# Build and test smart contracts
cd contracts
forge build && forge test

# Build and test Rust components
cargo build --release
cargo test

# Run the CBB simulation
python research/cbb_model.py

# Run a CLI engagement session
cargo run --example cli_session

# Deploy to local Anvil
anvil &
cd contracts && forge script script/Deploy.s.sol --broadcast --rpc-url http://127.0.0.1:8545
```

## Links

- Whitepaper: https://paragraph.com/@poehc/poehc-proof-of-exclusive-human-commitment

## License

MIT
