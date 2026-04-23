# Security Policy

## Supported Versions

| Version | Supported |
|--------:|:---------:|
| `main`  | ✅ |

If you are using a fork or a pinned commit, please still report vulnerabilities, but fixes may land on `main` first.

## Reporting a Vulnerability (Responsible Disclosure)

Please **do not** open a public GitHub Issue for security reports.

### Preferred contact
- Email: **zanabal.nowshad@icloud.com**

### Alternative contact (GitHub)
- If GitHub Private Vulnerability Reporting / Security Advisories are enabled for this repo, you may use that channel instead of email.

### What to include
- A clear description of the issue and impact
- Steps to reproduce (PoC if possible)
- Affected component(s) (contract name, function, file path)
- Any proposed mitigation or patch (optional)

### Response SLA
- **Acknowledgement:** within **48 hours**
- **Status update:** within **7 days** (or sooner if critical)

We aim to coordinate a fix and disclosure timeline with the reporter.

## Scope

### In scope
- `contracts/governance/**`
- `contracts/token/**`
- Build/test tooling that could impact contract correctness (e.g., scripts, CI)

### Out of scope
- Third-party dependencies and upstream toolchains (please report to the upstream project as well)
- Social engineering, phishing, or physical attacks
- Denial-of-service via unrealistic network-level assumptions outside the contract’s threat model

## Bug Bounty

This project **does not currently run a paid bug bounty program**.

If that changes, we will update this document with program rules, payout ranges, and a link to the bounty platform.

## Security Design Notes (High-level)

- `cast_vote` requires `require_auth()` — votes cannot be forged
- Double-vote prevention via persistent `HasVoted(proposal_id, voter)` key
- Vote weight = token balance at time of vote — no snapshot manipulation
- Only admin can execute or cancel proposals
- Quorum enforced at finalisation — proposals cannot pass silently with low turnout
- All amounts use `i128` — no floating-point arithmetic
