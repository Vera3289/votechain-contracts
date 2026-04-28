# ADR-003: Snapshot-Based Vote Weight

**Status:** Superseded (previously: Use Live Token Balance Instead of Vote Snapshots)  
**Date:** 2026-04-28

## Context

Token-weighted governance systems must decide when to measure a voter's balance: at proposal creation (snapshot) or at the moment of casting the vote (live balance). The original decision (ADR-003 v1) chose live balance for simplicity. SC-020 revisits this decision to prevent flash-loan and transfer-then-vote manipulation.

## Decision

Capture the voter's token balance at the time `cast_vote` is called and persist it as an immutable snapshot keyed by `(proposal_id, voter)`. The stored snapshot is used as the vote weight — the live balance is never re-queried after the snapshot is recorded.

This is a "lazy snapshot" strategy: balances are only recorded for addresses that actually vote, keeping storage overhead proportional to participation rather than total token holders.

## Consequences

- Vote weight cannot be manipulated after a vote is cast (e.g., by transferring tokens to another address and voting again from that address)
- Storage cost is O(voters) per proposal — efficient and bounded
- A voter who acquires tokens after proposal creation but before voting will have their balance at vote time recorded — this is acceptable and expected
- Implementation requires one additional persistent storage write per vote (`VoterSnapshot` key)
- If a voter's balance is zero at vote time, the vote is rejected with `NoVotingPower`
