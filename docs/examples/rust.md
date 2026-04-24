# Rust Examples — Governance Contract

These examples use the Soroban SDK test environment (`soroban_sdk::testutils`) and can be run against testnet by substituting `Env::default()` with a live RPC client.

All examples assume the governance and token contracts are already deployed and their IDs are known.

---

## Setup

```toml
# Cargo.toml
[dev-dependencies]
soroban-sdk = { version = "22.0.0", features = ["testutils"] }
```

```rust
use soroban_sdk::{testutils::Address as _, Address, Env, String};

// Re-export the generated client (produced by contractimport! or soroban-cli bindings)
mod governance {
    soroban_sdk::contractimport!(
        file = "../../target/wasm32-unknown-unknown/release/governance.wasm"
    );
}
mod token {
    soroban_sdk::contractimport!(
        file = "../../target/wasm32-unknown-unknown/release/token.wasm"
    );
}
```

---

## initialize

```rust
#[test]
fn example_initialize() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let token_id = env.register_contract_wasm(None, token::WASM);
    let gov_id = env.register_contract_wasm(None, governance::WASM);

    let gov = governance::Client::new(&env, &gov_id);
    gov.initialize(&admin, &token_id);
}
```

---

## create_proposal

```rust
#[test]
fn example_create_proposal() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let proposer = Address::generate(&env);
    let token_id = env.register_contract_wasm(None, token::WASM);
    let gov_id = env.register_contract_wasm(None, governance::WASM);

    let gov = governance::Client::new(&env, &gov_id);
    gov.initialize(&admin, &token_id);

    let proposal_id = gov.create_proposal(
        &proposer,
        &String::from_str(&env, "Increase treasury allocation"),
        &String::from_str(&env, "Allocate 10% more to dev fund"),
        &1000_i128,   // quorum: minimum total token-weighted votes
        &604800_u64,  // duration: ~7 days in seconds (Soroban uses Unix timestamps)
    );

    assert_eq!(proposal_id, 1);
}
```

---

## cast_vote

```rust
#[test]
fn example_cast_vote() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let voter = Address::generate(&env);
    let token_id = env.register_contract_wasm(None, token::WASM);
    let gov_id = env.register_contract_wasm(None, governance::WASM);

    let token = token::Client::new(&env, &token_id);
    let gov = governance::Client::new(&env, &gov_id);

    token.initialize(&admin, &7, &String::from_str(&env, "GOV"), &String::from_str(&env, "GOV"));
    token.mint(&voter, &500_i128); // give voter 500 tokens
    gov.initialize(&admin, &token_id);
    let proposal_id = gov.create_proposal(
        &admin,
        &String::from_str(&env, "Test proposal"),
        &String::from_str(&env, "Description"),
        &100_i128,
        &604800_u64,
    );

    // Vote::Yes = 0, Vote::No = 1, Vote::Abstain = 2
    gov.cast_vote(&voter, &proposal_id, &governance::Vote::Yes);
}
```

---

## finalise

```rust
#[test]
fn example_finalise() {
    let env = Env::default();
    env.mock_all_auths();

    // ... (setup as above, cast votes) ...

    // Advance ledger timestamp past end_time
    env.ledger().with_mut(|l| l.timestamp += 604801);

    gov.finalise(&proposal_id);

    let proposal = gov.get_proposal(&proposal_id);
    // status is now Passed or Rejected
    assert_ne!(proposal.status, governance::ProposalStatus::Active);
}
```

---

## execute

```rust
#[test]
fn example_execute() {
    let env = Env::default();
    env.mock_all_auths();

    // ... (setup, vote, finalise — proposal must be Passed) ...

    gov.execute(&admin, &proposal_id);

    let proposal = gov.get_proposal(&proposal_id);
    assert_eq!(proposal.status, governance::ProposalStatus::Executed);
}
```

---

## cancel

```rust
#[test]
fn example_cancel() {
    let env = Env::default();
    env.mock_all_auths();

    // ... (setup, proposal must be Active) ...

    gov.cancel(&admin, &proposal_id);

    let proposal = gov.get_proposal(&proposal_id);
    assert_eq!(proposal.status, governance::ProposalStatus::Cancelled);
}
```

---

## Running against testnet

Build the contracts and run tests:

```bash
make build
cargo test -- --nocapture
```

To invoke against a live testnet deployment, use `soroban-cli`:

```bash
soroban contract invoke \
  --network testnet \
  --source <SECRET_KEY> \
  --id <GOVERNANCE_CONTRACT_ID> \
  -- cast_vote \
  --voter <VOTER_ADDRESS> \
  --proposal_id 1 \
  --vote '{"Yes": null}'
```
