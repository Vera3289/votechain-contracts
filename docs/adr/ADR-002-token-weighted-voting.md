# ADR-002: Token-Weighted Voting Model

**Status:** Accepted  
**Date:** 2024-01-01

## Context

A governance system must define how much influence each participant has. Common models include one-address-one-vote, reputation-based voting, and token-weighted voting. VoteChain targets DAOs and protocols where governance tokens represent economic stake and accountability.

## Decision

Use token-weighted voting: a voter's weight equals their governance token balance at the time of casting the vote.

This aligns voting power with economic stake, which is the standard model for on-chain DAO governance (e.g., Compound, Uniswap). It is simple to implement, auditable, and familiar to the target audience.

## Consequences

- Voters with larger token holdings have proportionally more influence
- Token distribution directly affects governance outcomes — fair initial distribution is important
- Whale dominance is a known risk; quorum thresholds partially mitigate this
- Vote weight is read live from the token contract at cast time (see ADR-003)
