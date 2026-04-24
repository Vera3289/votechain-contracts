# Security Policy

## Supported Versions

| Version | Supported |
|--------:|:---------:|
| `main`  | ✅ |

Fixes are applied to `main` first. If you are running a fork or a pinned commit, please still report — we will coordinate disclosure with you.

---

## Responsible Disclosure

Please **do not** open a public GitHub Issue for security vulnerabilities. Public disclosure before a fix is available puts all users at risk.

Follow this process instead:

1. **Report privately** using one of the contact methods below.
2. We will **acknowledge your report within 48 hours**.
3. We will investigate and provide a status update within **7 days**.
4. We will work with you to agree on a coordinated disclosure date (typically after a fix is released).
5. You will be credited in the release notes unless you prefer to remain anonymous.

---

## Contact

| Method | Details |
|--------|---------|
| Email | **security@votechain.dev** (monitored by maintainers) |
| GitHub Private Advisory | Use [GitHub Security Advisories](https://github.com/Vera3289/votechain-contracts/security/advisories/new) to report confidentially without email |

When reporting, please include:

- A clear description of the vulnerability and its potential impact
- Steps to reproduce or a proof-of-concept (PoC)
- Affected component(s) — contract name, function, file path
- Any suggested mitigation or patch (optional but appreciated)

---

## Response SLA

| Milestone | Target |
|-----------|--------|
| Acknowledgement | **≤ 48 hours** |
| Initial assessment & severity triage | **≤ 7 days** |
| Fix or mitigation for critical issues | **≤ 14 days** |
| Coordinated public disclosure | Agreed with reporter, typically after fix is released |

---

## Scope

### In scope

- `contracts/governance/**` — proposal creation, voting, finalisation, execution, cancellation
- `contracts/token/**` — governance token minting, balances, transfers
- Build and CI tooling that could affect contract correctness (`scripts/`, `.github/workflows/`)

### Out of scope

- Third-party dependencies and upstream toolchains (Rust, Soroban SDK, Stellar Core) — please report those to the respective upstream projects
- Social engineering, phishing, or physical attacks
- Denial-of-service attacks that rely on unrealistic network-level assumptions outside the Soroban execution model
- Issues in forks or unofficial deployments not maintained by this repository

---

## Bug Bounty

This project **does not currently operate a paid bug bounty program**.

We recognise and publicly credit all valid security reports in release notes. If a bounty program is introduced in the future, this document will be updated with program rules, payout ranges, and a link to the bounty platform.

---

## Security Design Notes

Key security properties of the contracts:

- `cast_vote` calls `require_auth()` — votes cannot be forged by a third party
- Double-vote prevention via a persistent `HasVoted(proposal_id, voter)` storage key
- Vote weight equals the voter's token balance at the time of the vote — no snapshot manipulation
- Only the designated admin address can execute or cancel proposals
- Quorum is enforced at finalisation — proposals cannot pass silently with low turnout
- All token amounts use `i128` — no floating-point arithmetic or rounding errors
