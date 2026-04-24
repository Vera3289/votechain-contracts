# ADR-001: Use Stellar Soroban as the Smart Contract Platform

**Status:** Accepted  
**Date:** 2024-01-01

## Context

VoteChain requires a smart contract platform to host on-chain governance logic. Candidates considered were Ethereum (Solidity), Solana (Rust/Anchor), and Stellar Soroban (Rust). Key requirements were low transaction fees, deterministic execution, Rust-native development, and a growing ecosystem suited to DAOs and community governance.

## Decision

Use Stellar Soroban as the smart contract platform.

Soroban offers:
- Sub-cent transaction fees on Stellar, making frequent voting economically viable
- Rust as the first-class contract language, enabling strong type safety and tooling
- WASM-based execution with deterministic, auditable bytecode
- Native asset integration with Stellar's existing token standard (SEP-41)

## Consequences

- All contracts are written in Rust and compiled to WASM
- Developers need familiarity with the Soroban SDK and Stellar's account model
- The project is tied to Stellar's roadmap and Soroban's maturity trajectory
- Ethereum tooling (Hardhat, Foundry, etc.) is not applicable
