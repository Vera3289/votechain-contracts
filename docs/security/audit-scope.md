# Audit Scope

**Project:** VoteChain Contracts  
**Version:** 0.1.1  
**Audit type:** External security audit  
**Date prepared:** 2026-04-25  

---

## In-Scope Files

### Governance contract

| File | Description |
|------|-------------|
| `contracts/governance/src/lib.rs` | All public entry points: `initialize`, `create_proposal`, `cast_vote`, `finalise`, `execute`, `cancel`, `update_quorum`, `get_proposal`, `has_voted`, `proposal_count`, `get_version` |
| `contracts/governance/src/storage.rs` | Storage read/write helpers for proposals, admin, token, vote records, version |
| `contracts/governance/src/types.rs` | `ContractError`, `ProposalStatus`, `Vote`, `Proposal`, `DataKey` |
| `contracts/governance/src/events.rs` | On-chain event emission |

### Token contract

| File | Description |
|------|-------------|
| `contracts/token/src/lib.rs` | `initialize`, `balance`, `transfer`, `approve`, `transfer_from`, `mint`, `burn`, `total_supply`, `get_version` |
| `contracts/token/src/storage.rs` | Balance, allowance, total supply, admin, version storage helpers |
| `contracts/token/src/types.rs` | `ContractError`, `TokenDataKey` |

### Build and CI

| File | Description |
|------|-------------|
| `Cargo.toml` | Workspace dependency pinning |
| `contracts/governance/Cargo.toml` | Governance crate dependencies |
| `contracts/token/Cargo.toml` | Token crate dependencies |
| `.github/workflows/ci.yml` | Build, test, and lint pipeline |
| `.github/workflows/audit.yml` | Automated `cargo audit` dependency scan |

---

## Out-of-Scope

- `contracts/governance/src/test.rs` and `test_helpers.rs` — test code only
- `contracts/governance/src/prop_tests.rs` — property tests only
- `contracts/token/src/test.rs` — test code only
- `scripts/` — deployment scripts (shell, not contract logic)
- `docs/` — documentation only
- `config/` — environment configuration
- Third-party crates (`soroban-sdk`, `stellar-xdr`, etc.)

---

## Key Areas of Focus

1. **Access control** — verify `require_auth` is called on all state-mutating functions with the correct signer, and that admin-only functions correctly reject non-admin callers.

2. **Vote integrity** — verify the double-vote guard (`has_voted` / `mark_voted`) cannot be bypassed, and that vote weight is correctly read from the token contract.

3. **Arithmetic safety** — verify `checked_add` in `cast_vote` correctly handles overflow and that no other arithmetic can overflow or underflow.

4. **Proposal lifecycle** — verify that state transitions are exhaustive and that terminal states (Passed, Rejected, Executed, Cancelled) cannot be re-entered.

5. **Initialisation guard** — verify `is_initialized` prevents re-initialisation and that the guard fires before any auth check.

6. **Storage key collisions** — verify `DataKey` and `TokenDataKey` variants cannot collide across proposals or voters.

7. **Cross-contract trust** — verify the governance contract's reliance on the token contract's `balance` return value and assess the impact of a malicious or buggy token contract.

8. **Event correctness** — verify emitted events accurately reflect state transitions and cannot be spoofed.

---

## Commit Reference

Auditors should check out the tagged release:

```bash
git checkout v0.1.1
```

All findings should reference file paths and line numbers relative to this tag.

---

## Environment

| Component | Version |
|-----------|---------|
| Rust | 1.85+ (stable) |
| Soroban SDK | 22.0.0 |
| Target | `wasm32-unknown-unknown` |
| Stellar Protocol | 22 |

---

## Contacts

| Role | Contact |
|------|---------|
| Lead maintainer | security@votechain.dev |
| Security disclosures | See [SECURITY.md](../../SECURITY.md) |
