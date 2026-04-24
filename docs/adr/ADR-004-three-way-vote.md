# ADR-004: Three-Way Vote — Yes / No / Abstain

**Status:** Accepted  
**Date:** 2024-01-01

## Context

Binary Yes/No voting forces participants to either support or oppose a proposal, or stay silent. Many governance participants want to signal presence and quorum contribution without taking a directional stance — particularly when they lack sufficient context or have a conflict of interest. The question is whether to support a binary or three-way vote model.

## Decision

Support three vote options: **Yes**, **No**, and **Abstain**.

Abstain votes count toward the quorum threshold but not toward the Yes/No majority calculation. The pass condition is:

```
total_votes >= quorum  AND  votes_yes > votes_no
```

This matches the model used by major governance frameworks (OpenZeppelin Governor, Compound) and gives token holders a meaningful way to participate without distorting the directional outcome.

## Consequences

- The `VoteChoice` enum has three variants: `Yes`, `No`, `Abstain`
- Quorum is reached by total participation, not just Yes+No votes
- A proposal can pass with a small Yes majority if many holders abstain, which is intentional
- UI implementations must clearly communicate what Abstain means to voters
