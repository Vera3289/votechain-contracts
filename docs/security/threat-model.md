# Threat Model

**Project:** VoteChain Contracts  
**Version:** 0.1.1  
**Date:** 2026-04-25  

---

## 1. System Overview

VoteChain is a pair of Soroban smart contracts deployed on the Stellar blockchain:

- **Governance contract** — proposal creation, voting, finalisation, execution, cancellation
- **Token contract** — SEP-41-compatible governance token (balances, transfers, mint, burn)

All state is stored on-chain. There is no off-chain component in scope.

---

## 2. Trust Boundaries

| Actor | Trust Level | Capabilities |
|-------|-------------|--------------|
| Admin | Privileged | Execute/cancel proposals, update quorum, mint/burn tokens |
| Token holder | Unprivileged | Create proposals, cast votes |
| Anyone | Untrusted | Call `finalise`, `get_proposal`, `has_voted`, `proposal_count` |
| Soroban runtime | Trusted | Enforces auth, atomicity, storage isolation |
| Token contract | Trusted (deployed by admin) | Returns balances used as vote weight |

---

## 3. Assets

| Asset | Sensitivity | Location |
|-------|-------------|----------|
| Admin address | High | Instance storage (`DataKey::Admin`) |
| Voting token address | High | Instance storage (`DataKey::VotingToken`) |
| Proposal state (status, tallies) | High | Persistent storage (`DataKey::Proposal(id)`) |
| Vote records | Medium | Persistent storage (`DataKey::HasVoted(id, addr)`) |
| Token balances | High | Token contract persistent storage |

---

## 4. Threat Actors

**T1 — Malicious voter**
Goal: cast more votes than their token balance entitles them to, or vote multiple times.

**T2 — Malicious proposer**
Goal: create proposals that pass without genuine community support (e.g., with a quorum of 1).

**T3 — Malicious admin**
Goal: abuse privileged functions to cancel legitimate proposals, execute rejected ones, or replace the voting token.

**T4 — External attacker (no tokens)**
Goal: manipulate proposal outcomes, drain balances, or disrupt governance without holding tokens.

**T5 — Compromised token contract**
Goal: return inflated balances to favoured voters, enabling vote weight manipulation.

---

## 5. Threat Analysis

### T1 — Malicious voter

| Threat | Mitigation | Residual risk |
|--------|-----------|---------------|
| Vote multiple times on one proposal | `has_voted` guard + `mark_voted` before `save_proposal` | None |
| Vote with zero balance | `weight <= 0` check returns `NoVotingPower` | None |
| Inflate weight via token recycling | Transfer tokens to another address after voting; second address votes with same tokens | **Medium** — see Known Issues KI-001 |
| Overflow vote tally | `checked_add` returns `VoteTallyOverflow` error | None |

### T2 — Malicious proposer

| Threat | Mitigation | Residual risk |
|--------|-----------|---------------|
| Set quorum = 1 to pass with minimal votes | Quorum is caller-supplied; admin can update via `update_quorum` | Low — admin oversight required |
| Create spam proposals | No proposal creation fee or rate limit | Low — no economic damage, only storage cost |

### T3 — Malicious admin

| Threat | Mitigation | Residual risk |
|--------|-----------|---------------|
| Cancel a legitimate proposal | `cancel` is admin-only by design | Accepted — admin is a trusted role |
| Execute a rejected proposal | `execute` requires `ProposalStatus::Passed` | None |
| Replace voting token post-init | `initialize` is guarded by `is_initialized`; no `set_voting_token` function exists | None |
| Replace admin | No `set_admin` function exists after init | None |

### T4 — External attacker

| Threat | Mitigation | Residual risk |
|--------|-----------|---------------|
| Forge a vote for another address | `voter.require_auth()` enforced by Soroban | None |
| Re-initialise contract | `AlreadyInitialized` guard fires before auth check | None |
| Call `finalise` early | `VotingStillOpen` check on `end_time` | None |
| Call `execute` on non-passed proposal | `ProposalNotPassed` check | None |

### T5 — Compromised token contract

| Threat | Mitigation | Residual risk |
|--------|-----------|---------------|
| Return inflated balance for a voter | No mitigation — governance contract trusts the configured token | **High** — admin must deploy a trustworthy token |

---

## 6. Out-of-Scope Threats

- Stellar network-level attacks (eclipse, validator collusion)
- Soroban runtime bugs
- Key compromise of the admin account (operational security)
- Social engineering of token holders

---

## 7. Security Properties

1. **Vote integrity** — each address votes at most once per proposal with weight equal to its live token balance.
2. **Admin confinement** — admin can only cancel active proposals and execute passed ones; cannot alter vote tallies or voter records.
3. **Initialisation safety** — contract can only be initialised once; admin and token addresses are immutable after init.
4. **Arithmetic safety** — all tally arithmetic uses `checked_add`; overflow returns a contract error rather than panicking.
5. **Finalisation correctness** — pass condition (`total >= quorum && yes > no`) is evaluated atomically at finalisation time.
