# VoteChain Contracts

[![CI](https://github.com/Vera3289/votechain-contracts/actions/workflows/ci.yml/badge.svg)](https://github.com/Vera3289/votechain-contracts/actions/workflows/ci.yml)
[![License: Apache 2.0](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](LICENSE)

Soroban smart contracts for **VoteChain** — decentralized on-chain governance and voting on the Stellar blockchain.

VoteChain enables DAOs, protocols, and communities to create proposals, cast token-weighted votes, enforce quorum, and execute decisions — all transparently on-chain with an immutable audit trail.

---

## Features

- **Proposals** — create governance proposals with title, description, quorum, and voting duration
- **Token-weighted voting** — vote weight equals the voter's governance token balance
- **Yes / No / Abstain** — three-way vote with quorum and majority enforcement
- **Double-vote prevention** — each address can vote exactly once per proposal
- **Lifecycle management** — Active → Passed/Rejected → Executed, or Cancelled by admin
- **On-chain events** — every action emits a verifiable event for off-chain indexers

---

## Project Structure

```
.
├── contracts
│   ├── governance          # Proposal creation, voting, finalisation, execution
│   │   ├── src
│   │   │   ├── lib.rs
│   │   │   ├── storage.rs
│   │   │   ├── events.rs
│   │   │   ├── types.rs
│   │   │   └── test.rs
│   │   └── Cargo.toml
│   └── token               # Governance token contract
│       ├── src
│       │   ├── lib.rs
│       │   ├── storage.rs
│       │   ├── types.rs
│       │   └── test.rs
│       └── Cargo.toml
├── Cargo.toml
├── Makefile
├── CONTRIBUTING.md
├── SECURITY.md
└── README.md
```

---

## Quick Start

```bash
git clone https://github.com/Vera3289/votechain-contracts.git
cd votechain-contracts
rustup target add wasm32-unknown-unknown
make test
make build
```

---

## Governance Contract Reference

| Function | Caller | Description |
|---|---|---|
| `initialize(admin, voting_token)` | Admin | Set admin and governance token |
| `create_proposal(proposer, title, description, quorum, duration)` | Anyone | Create a new proposal |
| `cast_vote(voter, proposal_id, vote)` | Token holder | Cast Yes/No/Abstain vote |
| `finalise(proposal_id)` | Anyone | Finalise after voting period ends |
| `execute(admin, proposal_id)` | Admin | Mark a passed proposal as executed |
| `cancel(admin, proposal_id)` | Admin | Cancel an active proposal |
| `get_proposal(proposal_id)` | Anyone | Read proposal state |
| `has_voted(proposal_id, voter)` | Anyone | Check if address has voted |
| `proposal_count()` | Anyone | Total proposals created |

### Proposal Lifecycle

```
Active → Passed   → Executed
       → Rejected
       → Cancelled
```

### Pass Conditions

```
total_votes >= quorum  AND  votes_yes > votes_no
```

---

## Technology Stack

| Layer | Technology |
|---|---|
| Blockchain | Stellar (Soroban) |
| Language | Rust |
| SDK | Soroban SDK v22.0.0 |
| CI/CD | GitHub Actions |

---

## Environment Configuration

Config files live in `config/` — one per environment:

| File | Environment |
|---|---|
| `config/local.toml` | Local Stellar node (default) |
| `config/testnet.toml` | Stellar Testnet |
| `config/mainnet.toml` | Stellar Mainnet (no real values committed) |

Each file contains the RPC URL, network passphrase, and deployed contract addresses.

**Switching environments** — set the `NETWORK` variable before running deploy scripts:

```bash
# Local (default)
./scripts/deploy.sh

# Testnet
NETWORK=testnet ./scripts/deploy.sh

# Mainnet — fill in contract addresses in config/mainnet.toml first
NETWORK=mainnet ./scripts/deploy.sh
```

> **Security**: `config/mainnet.toml` is committed with placeholder values only.  
> Never commit real contract addresses or private keys.

---

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md).

## Security

See [SECURITY.md](SECURITY.md).

## License

[Apache 2.0](LICENSE)

---

Built with ❤️ on Stellar
