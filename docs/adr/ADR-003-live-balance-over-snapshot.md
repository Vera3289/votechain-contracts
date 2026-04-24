# ADR-003: Use Live Token Balance Instead of Vote Snapshots

**Status:** Accepted  
**Date:** 2024-01-01

## Context

Token-weighted governance systems must decide when to measure a voter's balance: at proposal creation (snapshot) or at the moment of casting the vote (live balance). Snapshot voting (e.g., Snapshot.org, ERC-20 `getPastVotes`) prevents balance manipulation during a vote but requires additional infrastructure. Live balance is simpler but exposes a flash-loan or transfer-then-vote attack vector.

## Decision

Use the voter's live token balance at the time `cast_vote` is called.

On Soroban, cross-contract calls are synchronous and atomic within a transaction. A voter cannot transfer tokens and vote in the same transaction without the transfer being visible. Combined with Stellar's lack of flash loans (no uncollateralised borrowing primitive), the practical attack surface is low. Implementing snapshot logic would require a separate checkpoint contract and significantly more complexity.

## Consequences

- Implementation is simple: one cross-contract `balance` call per vote
- No snapshot infrastructure or additional storage is needed
- A voter who transfers tokens before voting will have a lower weight — this is acceptable and expected behaviour
- If flash-loan primitives are introduced to Soroban in the future, this decision should be revisited
