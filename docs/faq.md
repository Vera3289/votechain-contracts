# Frequently Asked Questions

## General

**Q: What is VoteChain?**  
A: VoteChain is a decentralized on-chain governance protocol built with Soroban smart contracts on the Stellar blockchain. It lets DAOs, protocols, and communities create proposals, cast token-weighted votes, enforce quorum, and execute decisions — all transparently and immutably on-chain.

**Q: What blockchain does VoteChain run on?**  
A: VoteChain runs on Stellar using the Soroban smart contract platform (SDK v22.0.0).

**Q: What is Soroban and why does VoteChain use it?**  
A: Soroban is Stellar's native smart contract environment. It provides the execution layer for complex logic like weighted voting, automated tallying, and state archival — capabilities not available with traditional Stellar operations.

**Q: Is VoteChain permissionless?**  
A: Proposal creation is open to any address. Voting requires holding governance tokens. Admin-only actions (execute, cancel, initialize) are restricted to the configured admin address.

---

## Proposal Creation

**Q: Who can create a proposal?**  
A: Any address can call `create_proposal`. There is no minimum token balance required to submit a proposal.

**Q: What information is required to create a proposal?**  
A: A proposal requires a title, description, quorum threshold (minimum total votes needed to pass), and a voting duration (measured in ledgers).

**Q: How long can a voting period last?**  
A: The duration is set by the proposer at creation time as a ledger count. There is no protocol-enforced maximum — it is up to the proposer and community convention.

**Q: Can a proposal be cancelled after creation?**  
A: Yes. The admin can cancel any Active proposal by calling `cancel(admin, proposal_id)`. Cancelled proposals cannot be reactivated.

---

## Voting Mechanics

**Q: What vote options are available?**  
A: Voters can choose one of three options: Yes, No, or Abstain.

**Q: How is vote weight determined?**  
A: Vote weight equals the voter's live governance token balance at the time they cast their vote. There are no snapshots — the balance is read directly from the token contract when `cast_vote` is called.

**Q: Can I change my vote after submitting?**  
A: No. Each address can vote exactly once per proposal. Attempting to vote again returns error `102 AlreadyVoted`.

**Q: Do Abstain votes count toward quorum?**  
A: Yes. Abstain votes contribute to `total_votes` and therefore count toward meeting the quorum threshold, but they do not count as Yes or No for the majority check.

---

## Token Requirements

**Q: Which token is used for voting?**  
A: The governance token is set during contract initialization via `initialize(admin, voting_token)`. It must be a Soroban-compatible token contract deployed on Stellar.

**Q: What balance do I need to vote?**  
A: Any non-zero balance of the governance token allows you to vote. Your vote weight is proportional to your balance — a larger balance carries more weight.

**Q: What happens if I transfer tokens after voting?**  
A: Your vote is already recorded and cannot be changed. Token transfers after voting have no effect on the current proposal, but will affect your weight in future proposals.

---

## Quorum & Finalisation

**Q: How does quorum work?**  
A: Each proposal has a `quorum` value set at creation. For a proposal to pass, two conditions must both be true:
```
total_votes >= quorum  AND  votes_yes > votes_no
```
If quorum is not reached, the proposal is Rejected regardless of the Yes/No split.

**Q: Who finalises a proposal?**  
A: Anyone can call `finalise(proposal_id)` after the voting period ends. The contract evaluates the pass conditions and transitions the proposal to Passed or Rejected.

**Q: What happens after a proposal passes?**  
A: A Passed proposal must be explicitly executed by the admin via `execute(admin, proposal_id)`, which transitions it to the Executed state. Execution is a manual step — VoteChain does not automatically trigger cross-contract calls.

**Q: Can I query the current state of a proposal?**  
A: Yes. Call `get_proposal(proposal_id)` to read the full proposal state, or `has_voted(proposal_id, voter)` to check whether a specific address has already voted.

---

## Errors

**Q: What do the contract error codes mean?**

| Code | Name | Cause |
|------|------|-------|
| 101 | Unauthorized | Caller is not the admin or required signer |
| 102 | AlreadyVoted | Address has already voted on this proposal |
| 103 | ProposalExpired | Voting period has ended |
| 104 | InsufficientStake | Voter holds no governance tokens |
| 105 | InvalidStatus | Proposal is not in the required state for the operation |
