# FRIED Staking

Solana staking program and web interface for **FRIED CHICKEN (FRIED)**.

## Pool configuration

| Pool | Lock | APY |
|---|---:|---:|
| Flexible | None | 30% |
| 7 Day | 7 days | 40% |
| 30 Day | 30 days | 60% |

- FRIED mint: `3LaXmnVQxMMArywUBJjGA5EKbUdJ17PFor3M5oeupump`
- Minimum stake: `100,000 FRIED`
- Reward allocation: `20,000,000 FRIED`
- Program duration: one year

## Repository structure

- `programs/fried-staking` — Anchor smart contract
- `tests` — program integration tests
- `app` — React/Vite wallet interface
- `scripts` — initialization and reward-funding scripts

## Important

This repository contains deployable source code, but it is **not live merely because it is on
GitHub**. Before mainnet use, build and test it, deploy the program, replace the placeholder
program ID everywhere, initialize it with the admin wallet, fund the reward vault, and obtain an
independent smart-contract audit. Never commit a wallet seed phrase or private key.

## Local setup

Requirements: Rust, Solana CLI, Anchor 0.30.1, Node.js 20+, and Yarn.

```bash
yarn install
anchor build
anchor test
```

Copy `.env.example` to `.env`, set the deployed program ID and RPC endpoint, then:

```bash
yarn initialize
yarn fund-rewards
yarn app:dev
```

See [DEPLOYMENT.md](DEPLOYMENT.md) for the mainnet checklist.
