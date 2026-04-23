#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::{Address as _, Ledger}, Address, Env, String};

fn setup() -> (Env, GovernanceContractClient<'static>) {
    let env = Env::default();
    env.mock_all_auths();
    let id = env.register(GovernanceContract, ());
    let client = GovernanceContractClient::new(&env, &id);
    (env, client)
}

fn setup_token(env: &Env, admin: &Address) -> Address {
    let id = env.register(votechain_token::TokenContract, ());
    let t = votechain_token::TokenContractClient::new(env, &id);
    t.initialize(admin, &1_000_000);
    id
}

fn make_proposal(env: &Env, client: &GovernanceContractClient, proposer: &Address, token_id: &Address) -> u64 {
    let admin = Address::generate(env);
    client.initialize(&admin, token_id);
    client.create_proposal(
        proposer,
        &String::from_str(env, "Upgrade protocol"),
        &String::from_str(env, "Upgrade to v2"),
        &100,   // quorum
        &3600,  // 1 hour
    )
}

#[test]
fn test_create_proposal() {
    let (env, client) = setup();
    let admin = Address::generate(&env);
    let proposer = Address::generate(&env);
    let token_id = setup_token(&env, &admin);

    client.initialize(&admin, &token_id);
    let id = client.create_proposal(
        &proposer,
        &String::from_str(&env, "Add new feature"),
        &String::from_str(&env, "Details here"),
        &50,
        &7200,
    );
    assert_eq!(id, 1);
    assert_eq!(client.proposal_count(), 1);
    assert_eq!(client.get_proposal(&id).status, ProposalStatus::Active);
}

#[test]
fn test_cast_vote_and_finalise_passed() {
    let (env, client) = setup();
    let admin = Address::generate(&env);
    let voter = Address::generate(&env);
    let token_id = setup_token(&env, &voter); // voter holds all tokens

    client.initialize(&admin, &token_id);
    let id = client.create_proposal(
        &voter,
        &String::from_str(&env, "Proposal A"),
        &String::from_str(&env, "desc"),
        &100,
        &3600,
    );

    client.cast_vote(&voter, &id, &Vote::Yes);
    assert!(client.has_voted(&id, &voter));

    let p = client.get_proposal(&id);
    assert_eq!(p.votes_yes, 1_000_000);

    // Advance past end_time
    env.ledger().with_mut(|l| l.timestamp += 3601);
    client.finalise(&id);

    assert_eq!(client.get_proposal(&id).status, ProposalStatus::Passed);
}

#[test]
fn test_finalise_rejected_below_quorum() {
    let (env, client) = setup();
    let admin = Address::generate(&env);
    let voter = Address::generate(&env);
    let token_id = setup_token(&env, &voter);

    client.initialize(&admin, &token_id);
    let id = client.create_proposal(
        &voter,
        &String::from_str(&env, "Proposal B"),
        &String::from_str(&env, "desc"),
        &9_999_999, // quorum higher than total supply
        &3600,
    );

    client.cast_vote(&voter, &id, &Vote::Yes);
    env.ledger().with_mut(|l| l.timestamp += 3601);
    client.finalise(&id);

    assert_eq!(client.get_proposal(&id).status, ProposalStatus::Rejected);
}

#[test]
fn test_finalise_rejected_no_wins() {
    let (env, client) = setup();
    let admin = Address::generate(&env);
    let voter = Address::generate(&env);
    let token_id = setup_token(&env, &voter);

    client.initialize(&admin, &token_id);
    let id = client.create_proposal(
        &voter,
        &String::from_str(&env, "Proposal C"),
        &String::from_str(&env, "desc"),
        &100,
        &3600,
    );

    client.cast_vote(&voter, &id, &Vote::No);
    env.ledger().with_mut(|l| l.timestamp += 3601);
    client.finalise(&id);

    assert_eq!(client.get_proposal(&id).status, ProposalStatus::Rejected);
}

#[test]
fn test_execute_passed_proposal() {
    let (env, client) = setup();
    let admin = Address::generate(&env);
    let voter = Address::generate(&env);
    let token_id = setup_token(&env, &voter);

    client.initialize(&admin, &token_id);
    let id = client.create_proposal(
        &voter,
        &String::from_str(&env, "Proposal D"),
        &String::from_str(&env, "desc"),
        &100,
        &3600,
    );
    client.cast_vote(&voter, &id, &Vote::Yes);
    env.ledger().with_mut(|l| l.timestamp += 3601);
    client.finalise(&id);
    client.execute(&admin, &id);

    assert_eq!(client.get_proposal(&id).status, ProposalStatus::Executed);
}

#[test]
fn test_cancel_proposal() {
    let (env, client) = setup();
    let admin = Address::generate(&env);
    let proposer = Address::generate(&env);
    let token_id = setup_token(&env, &admin);

    client.initialize(&admin, &token_id);
    let id = client.create_proposal(
        &proposer,
        &String::from_str(&env, "Proposal E"),
        &String::from_str(&env, "desc"),
        &100,
        &3600,
    );
    client.cancel(&admin, &id);
    assert_eq!(client.get_proposal(&id).status, ProposalStatus::Cancelled);
}

/// SEC-002: verify checked_add panics on i128::MAX overflow
#[test]
#[should_panic(expected = "vote tally overflow")]
fn test_vote_tally_overflow() {
    let (env, client) = setup();
    let admin = Address::generate(&env);
    let voter_a = Address::generate(&env);
    let voter_b = Address::generate(&env);

    // Give voter_a i128::MAX tokens and voter_b 1 token so the second vote overflows
    let token_id = env.register(votechain_token::TokenContract, ());
    let token = votechain_token::TokenContractClient::new(&env, &token_id);
    token.initialize(&admin, &i128::MAX);
    token.transfer(&admin, &voter_a, &i128::MAX);
    // mint 1 more to voter_b (total supply exceeds i128::MAX — only possible in test env)
    token.mint(&admin, &voter_b, &1);

    client.initialize(&admin, &token_id);
    let id = client.create_proposal(
        &voter_a,
        &String::from_str(&env, "Overflow test"),
        &String::from_str(&env, "desc"),
        &1,
        &3600,
    );
    client.cast_vote(&voter_a, &id, &Vote::Yes); // fills tally to i128::MAX
    client.cast_vote(&voter_b, &id, &Vote::Yes); // should panic: overflow
}

#[test]
#[should_panic(expected = "already voted")]
fn test_cannot_vote_twice() {
    let (env, client) = setup();
    let admin = Address::generate(&env);
    let voter = Address::generate(&env);
    let token_id = setup_token(&env, &voter);

    client.initialize(&admin, &token_id);
    let id = client.create_proposal(
        &voter,
        &String::from_str(&env, "Proposal F"),
        &String::from_str(&env, "desc"),
        &100,
        &3600,
    );
    client.cast_vote(&voter, &id, &Vote::Yes);
    client.cast_vote(&voter, &id, &Vote::No); // should panic
}
