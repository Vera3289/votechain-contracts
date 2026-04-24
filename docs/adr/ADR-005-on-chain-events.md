# ADR-005: Emit On-Chain Events for All State Transitions

**Status:** Accepted  
**Date:** 2024-01-01

## Context

Governance activity needs to be observable by off-chain systems — dashboards, indexers, notification services, and audit tools. On Soroban, contracts can emit events that are recorded in the transaction metadata and accessible via the Horizon API and RPC. The alternative is to rely solely on on-chain storage queries, which requires polling and is less efficient.

## Decision

Emit a Soroban event for every state-changing action: proposal creation, vote cast, proposal finalised, proposal executed, and proposal cancelled.

Each event includes the relevant identifiers and outcome so that an off-chain indexer can reconstruct full governance history without reading contract storage directly.

## Consequences

- Every mutating function calls `env.events().publish(...)` before returning
- Off-chain indexers can stream events via the Stellar RPC `getEvents` endpoint
- Transaction costs increase marginally due to event data in the ledger footprint
- Event schema changes are breaking for indexers — event topics and data must be treated as a public API
