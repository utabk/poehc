# PoEHC — Proof of Exclusive Human Commitment

Bitcoin solved the double-spend problem for money. PoEHC solves the double-spend problem for human time.

PoEHC is a cryptographic protocol that proves a human exclusively dedicated their attention and cognitive effort to a specific task during a specific period — without surveillance.

## Repository Structure

- `contracts/` — Solidity smart contracts (Foundry)
- `agent/core/` — Engagement agent core library (Rust)
- `validator/` — Validator node (Rust)
- `sdk/` — Developer SDKs (TypeScript + Rust)
- `docs/` — Whitepaper
- `research/` — CBB theorem simulation

## Building

### Smart Contracts
```bash
cd contracts
forge build
forge test
```

### Agent Core + Validator
```bash
cargo build --release
cargo test
```

## License

MIT
