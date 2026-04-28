# Contributing to VoteChain

Thank you for contributing! VoteChain is an open-source governance protocol built on Stellar Soroban.

---

## Table of Contents

- [Getting Started](#getting-started)
- [Branching Strategy](#branching-strategy)
- [Commit Messages](#commit-messages)
- [Development Workflow](#development-workflow)
- [Pull Request Process](#pull-request-process)
- [Code Review Expectations](#code-review-expectations)
- [Reporting Bugs](#reporting-bugs)
- [License](#license)

---

## Getting Started

**Prerequisites:** Rust stable toolchain, `wasm32-unknown-unknown` target, and (optionally) Docker.

```bash
git clone https://github.com/Vera3289/votechain-contracts.git
cd votechain-contracts
rustup target add wasm32-unknown-unknown
make test
```

For a fully reproducible environment without a local Rust installation, use Docker:

```bash
docker compose run --rm dev make test
```

---

## Branching Strategy

All work happens in short-lived topic branches that target `main`.

| Prefix | Purpose | Example |
| ------ | ------- | ------- |
| `feature/` | New functionality | `feature/delegation-support` |
| `fix/` | Bug fixes | `fix/double-vote-edge-case` |
| `docs/` | Documentation only | `docs/update-lifecycle-diagram` |
| `test/` | New or improved tests | `test/quorum-boundary-cases` |
| `chore/` | Maintenance, tooling, CI | `chore/bump-soroban-sdk` |
| `security/` | Security fixes | `security/reinit-guard` |
| `refactor/` | Code restructuring without behaviour change | `refactor/storage-helpers` |

**Rules:**

- Branch from the latest `main`.
- Keep branches focused — one logical change per branch.
- Delete the branch after it is merged.
- Do not commit directly to `main`; all changes must go through a pull request.

---

## Commit Messages

Follow [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/):

```text
<type>(<optional scope>): <short summary in lower case>

<optional body — explain the why, not the what>
```

**Types:**

| Type | When to use |
| ---- | ----------- |
| `feat` | New feature or contract function |
| `fix` | Bug fix |
| `docs` | Documentation changes only |
| `test` | Adding or updating tests |
| `refactor` | Code change that neither fixes a bug nor adds a feature |
| `chore` | Build scripts, CI, dependencies, tooling |
| `security` | Security-related fixes or hardening |
| `perf` | Performance improvement |

**Examples:**

```text
feat: add delegation support to governance contract

fix(cast_vote): prevent double-vote across proposal lifecycle

test: add quorum boundary edge cases to prop_tests

chore: upgrade soroban-sdk to 22.1.0

docs(lifecycle): clarify abstain vote quorum behaviour
```

**Rules:**

- Summary line ≤ 72 characters, lower case, no trailing period.
- Use the imperative mood: "add", "fix", "remove" — not "added" or "fixes".
- Reference the relevant issue in the body when applicable: `Closes #52`.

---

## Development Workflow

### Running tests

```bash
# Run the full test suite (unit + property-based)
make test

# Run tests for a single crate
cargo test -p votechain-governance
cargo test -p votechain-token

# Run a specific test by name
cargo test test_cast_vote_and_finalise_passed

# Show println!/dbg! output from passing tests
cargo test -- --nocapture
```

### Formatting and linting

```bash
# Auto-format all source files (run before every commit)
make fmt

# Check formatting without modifying files (same check CI runs)
make fmt-check

# Run Clippy and fail on any warning (same check CI runs)
make lint
```

### Building WASM contracts

```bash
# Compile both contracts to optimised WASM
make build

# Alternatively, use the Stellar CLI directly
stellar contract build
```

Built WASM files are written to `target/wasm32-unknown-unknown/release/`.

### Contract standards

Every contribution to the contract crates must follow these invariants or the CI will fail:

- `#![no_std]` — all contract crates are `no_std`.
- No floating-point arithmetic — all vote weights and balances use `i128`.
- Every state-changing function must emit the corresponding on-chain event.
- Every new public function requires at least one test in `test.rs`.
- `cargo fmt --check` and `cargo clippy -- -D warnings` must pass cleanly.
- `cargo audit` must report zero advisories.

---

## Pull Request Process

1. **Open a draft PR early** if you want feedback on the approach before the implementation is complete.
2. **Fill in the PR template** — describe the change, link the issue, and check every box in the checklist.
3. **Keep PRs small and focused.** A PR that fixes one thing is easier to review and faster to merge than one that fixes five.
4. **Resolve all CI failures before requesting review.** Do not ask reviewers to look at a red build.
5. **Respond to review comments** within a reasonable time. If a thread is resolved by a code change, mark it resolved.
6. Squash or clean up noisy "fixup" commits before the final merge.

### PR checklist

- [ ] `make fmt` run locally
- [ ] `make test` passes
- [ ] `make lint` passes
- [ ] Events emitted for every state-changing operation
- [ ] New public functions have tests
- [ ] `README.md` updated if observable behaviour changed
- [ ] `CHANGELOG.md` `[Unreleased]` section updated for user-visible changes

---

## Code Review Expectations

**For authors:**

- A PR description should make it easy for reviewers to understand *why* the change is needed, not just *what* changed.
- Annotate non-obvious design choices with inline comments or PR comments so reviewers don't have to reverse-engineer your reasoning.
- Be receptive to feedback — a requested change is a conversation, not a rejection.

**For reviewers:**

- Every PR targeting `main` requires at least **one approving review** from a maintainer before merge.
- Check that:
  - The logic is correct and the new/changed code is tested.
  - All state-changing functions emit the appropriate event.
  - No `f32`/`f64` arithmetic is introduced.
  - The `no_std` constraint is preserved.
  - Error variants are descriptive and match the existing `ContractError` style.
  - The PR checklist has been completed.
- Distinguish between blocking concerns (must fix) and suggestions (nice to have) when leaving comments.
- Approve once all blocking concerns are addressed; do not block a merge on optional style preferences.

---

## Reporting Bugs

**Security vulnerabilities** — do **not** open a public issue. Follow the responsible disclosure process in [SECURITY.md](SECURITY.md).

**Regular bugs** — open a GitHub Issue using the [bug report template](.github/ISSUE_TEMPLATE/bug_report.yml). Include:

- A short, clear title describing the unexpected behaviour.
- The function or contract that exhibits the bug.
- Steps to reproduce (minimal Rust test case preferred).
- Expected behaviour vs. actual behaviour.
- Soroban SDK version and Rust toolchain version (`rustc --version`).

Pull requests that fix bugs are welcome alongside or instead of an issue.

---

## License

By contributing you agree that your contributions will be licensed under the [Apache 2.0 License](LICENSE).
