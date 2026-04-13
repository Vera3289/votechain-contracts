# Security Policy

## Reporting a Vulnerability

**Do not open a public issue.** Email `security@votechain.example`.

## Security Design Notes

- `cast_vote` requires `require_auth()` — votes cannot be forged
- Double-vote prevention via persistent `HasVoted(proposal_id, voter)` key
- Vote weight = token balance at time of vote — no snapshot manipulation
- Only admin can execute or cancel proposals
- Quorum enforced at finalisation — proposals cannot pass silently with low turnout
- All amounts use `i128` — no floating-point arithmetic
