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

// ── TEST-002: create_proposal unit tests ─────────────────────────────────────

#[test]
fn test_create_proposal_stores_fields_correctly() {
    let (env, client) = setup();
    let admin = Address::generate(&env);
    let proposer = Address::generate(&env);
    let token_id = setup_token(&env, &admin);

    client.initialize(&admin, &token_id);
    let id = client.create_proposal(
        &proposer,
        &String::from_str(&env, "My Title"),
        &String::from_str(&env, "My Description"),
        &200,
        &7200,
    );

    let p = client.get_proposal(&id);
    assert_eq!(p.id, id);
    assert_eq!(p.proposer, proposer);
    assert_eq!(p.quorum, 200);
    assert_eq!(p.status, ProposalStatus::Active);
    assert_eq!(p.votes_yes, 0);
    assert_eq!(p.votes_no, 0);
    assert_eq!(p.votes_abstain, 0);
}

#[test]
fn test_create_proposal_id_increments() {
    let (env, client) = setup();
    let admin = Address::generate(&env);
    let proposer = Address::generate(&env);
    let token_id = setup_token(&env, &admin);

    client.initialize(&admin, &token_id);
    let id1 = client.create_proposal(&proposer, &String::from_str(&env, "P1"), &String::from_str(&env, "d"), &1, &60);
    let id2 = client.create_proposal(&proposer, &String::from_str(&env, "P2"), &String::from_str(&env, "d"), &1, &60);
    let id3 = client.create_proposal(&proposer, &String::from_str(&env, "P3"), &String::from_str(&env, "d"), &1, &60);

    assert_eq!(id1, 1);
    assert_eq!(id2, 2);
    assert_eq!(id3, 3);
    assert_eq!(client.proposal_count(), 3);
}

#[test]
fn test_create_proposal_emits_event() {
    let (env, client) = setup();
    let admin = Address::generate(&env);
    let proposer = Address::generate(&env);
    let token_id = setup_token(&env, &admin);

    client.initialize(&admin, &token_id);
    client.create_proposal(
        &proposer,
        &String::from_str(&env, "Event Test"),
        &String::from_str(&env, "desc"),
        &100,
        &3600,
    );

    // proposal_created publishes ("created", id) topic with proposer as data
    let events = env.events().all();
    assert!(!events.is_empty());
}

#[test]
#[should_panic(expected = "title cannot be empty")]
fn test_create_proposal_empty_title_reverts() {
    let (env, client) = setup();
    let admin = Address::generate(&env);
    let proposer = Address::generate(&env);
    let token_id = setup_token(&env, &admin);
    client.initialize(&admin, &token_id);
    client.create_proposal(&proposer, &String::from_str(&env, ""), &String::from_str(&env, "desc"), &100, &3600);
}

#[test]
#[should_panic(expected = "description cannot be empty")]
fn test_create_proposal_empty_description_reverts() {
    let (env, client) = setup();
    let admin = Address::generate(&env);
    let proposer = Address::generate(&env);
    let token_id = setup_token(&env, &admin);
    client.initialize(&admin, &token_id);
    client.create_proposal(&proposer, &String::from_str(&env, "Title"), &String::from_str(&env, ""), &100, &3600);
}

#[test]
#[should_panic(expected = "title too long")]
fn test_create_proposal_title_exceeds_max_length_reverts() {
    let (env, client) = setup();
    let admin = Address::generate(&env);
    let proposer = Address::generate(&env);
    let token_id = setup_token(&env, &admin);
    client.initialize(&admin, &token_id);
    // 65-character title
    let long_title = String::from_str(&env, "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA");
    client.create_proposal(&proposer, &long_title, &String::from_str(&env, "desc"), &100, &3600);
}

#[test]
#[should_panic(expected = "description too long")]
fn test_create_proposal_description_exceeds_max_length_reverts() {
    let (env, client) = setup();
    let admin = Address::generate(&env);
    let proposer = Address::generate(&env);
    let token_id = setup_token(&env, &admin);
    client.initialize(&admin, &token_id);
    // 257-character description
    let long_desc = String::from_str(&env, "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA");
    client.create_proposal(&proposer, &String::from_str(&env, "Title"), &long_desc, &100, &3600);
}

#[test]
#[should_panic(expected = "quorum must be positive")]
fn test_create_proposal_zero_quorum_reverts() {
    let (env, client) = setup();
    let admin = Address::generate(&env);
    let proposer = Address::generate(&env);
    let token_id = setup_token(&env, &admin);
    client.initialize(&admin, &token_id);
    client.create_proposal(&proposer, &String::from_str(&env, "Title"), &String::from_str(&env, "desc"), &0, &3600);
}

#[test]
#[should_panic(expected = "duration out of bounds")]
fn test_create_proposal_duration_too_short_reverts() {
    let (env, client) = setup();
    let admin = Address::generate(&env);
    let proposer = Address::generate(&env);
    let token_id = setup_token(&env, &admin);
    client.initialize(&admin, &token_id);
    client.create_proposal(&proposer, &String::from_str(&env, "Title"), &String::from_str(&env, "desc"), &100, &59);
}

#[test]
#[should_panic(expected = "duration out of bounds")]
fn test_create_proposal_duration_too_long_reverts() {
    let (env, client) = setup();
    let admin = Address::generate(&env);
    let proposer = Address::generate(&env);
    let token_id = setup_token(&env, &admin);
    client.initialize(&admin, &token_id);
    client.create_proposal(&proposer, &String::from_str(&env, "Title"), &String::from_str(&env, "desc"), &100, &31_536_001);
}
