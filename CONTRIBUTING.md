# Contributing to VoteChain

Thank you for contributing! VoteChain is part of the Stellar open-source ecosystem.

## Getting Started

```bash
git clone https://github.com/Vera3289/votechain-contracts.git
cd votechain-contracts
rustup target add wasm32-unknown-unknown
make test
```

## Standards

- Pass `cargo clippy -- -D warnings` and `cargo fmt --check`
- Every new function needs a test in `test.rs`
- Emit events for all state-changing operations
- No floating-point — all vote weights use `i128`
- `#![no_std]` in all contract crates

## Commit Messages

Follow [Conventional Commits](https://www.conventionalcommits.org/):
```
feat: add delegation support to governance contract
fix: prevent double-vote across proposal lifecycle
test: add quorum boundary test cases
```

## PR Checklist
- [ ] `make test` passes
- [ ] `make lint` passes
- [ ] `make fmt-check` passes
- [ ] Events emitted for state changes
- [ ] README updated if behaviour changed

## License

Apache 2.0
