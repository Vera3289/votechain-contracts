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

// ── TEST-009: Concurrent proposal scenario tests ─────────────────────────────

/// Multiple active proposals can coexist and receive independent votes.
#[test]
fn test_concurrent_proposals_independent_votes() {
    let (env, client) = setup();
    let admin = Address::generate(&env);
    let voter = Address::generate(&env);
    let token_id = setup_token(&env, &voter);

    client.initialize(&admin, &token_id);
    let id1 = client.create_proposal(&voter, &String::from_str(&env, "P1"), &String::from_str(&env, "d"), &1, &3600);
    let id2 = client.create_proposal(&voter, &String::from_str(&env, "P2"), &String::from_str(&env, "d"), &1, &3600);
    let id3 = client.create_proposal(&voter, &String::from_str(&env, "P3"), &String::from_str(&env, "d"), &1, &3600);

    assert_eq!(client.get_proposal(&id1).status, ProposalStatus::Active);
    assert_eq!(client.get_proposal(&id2).status, ProposalStatus::Active);
    assert_eq!(client.get_proposal(&id3).status, ProposalStatus::Active);

    client.cast_vote(&voter, &id1, &Vote::Yes);
    // voter has not voted on id2 or id3
    assert!(client.has_voted(&id1, &voter));
    assert!(!client.has_voted(&id2, &voter));
    assert!(!client.has_voted(&id3, &voter));
}

/// Votes on one proposal do not affect tallies of another.
#[test]
fn test_concurrent_votes_do_not_bleed() {
    let (env, client) = setup();
    let admin = Address::generate(&env);
    let voter = Address::generate(&env);
    let token_id = setup_token(&env, &voter);

    client.initialize(&admin, &token_id);
    let id1 = client.create_proposal(&voter, &String::from_str(&env, "P1"), &String::from_str(&env, "d"), &1, &3600);
    let id2 = client.create_proposal(&voter, &String::from_str(&env, "P2"), &String::from_str(&env, "d"), &1, &3600);

    client.cast_vote(&voter, &id1, &Vote::Yes);

    let p1 = client.get_proposal(&id1);
    let p2 = client.get_proposal(&id2);
    assert_eq!(p1.votes_yes, 1_000_000);
    assert_eq!(p2.votes_yes, 0);
    assert_eq!(p2.votes_no, 0);
    assert_eq!(p2.votes_abstain, 0);
}

/// Finalising one proposal does not change the status of others.
#[test]
fn test_finalise_one_does_not_affect_others() {
    let (env, client) = setup();
    let admin = Address::generate(&env);
    let voter = Address::generate(&env);
    let token_id = setup_token(&env, &voter);

    client.initialize(&admin, &token_id);
    let id1 = client.create_proposal(&voter, &String::from_str(&env, "P1"), &String::from_str(&env, "d"), &1, &3600);
    let id2 = client.create_proposal(&voter, &String::from_str(&env, "P2"), &String::from_str(&env, "d"), &1, &7200);

    client.cast_vote(&voter, &id1, &Vote::Yes);
    env.ledger().with_mut(|l| l.timestamp += 3601);
    client.finalise(&id1);

    assert_ne!(client.get_proposal(&id1).status, ProposalStatus::Active);
    assert_eq!(client.get_proposal(&id2).status, ProposalStatus::Active);
}

/// Proposal IDs are unique and monotonically increasing.
#[test]
fn test_proposal_ids_are_unique() {
    let (env, client) = setup();
    let admin = Address::generate(&env);
    let proposer = Address::generate(&env);
    let token_id = setup_token(&env, &admin);

    client.initialize(&admin, &token_id);
    let id1 = client.create_proposal(&proposer, &String::from_str(&env, "P1"), &String::from_str(&env, "d"), &1, &3600);
    let id2 = client.create_proposal(&proposer, &String::from_str(&env, "P2"), &String::from_str(&env, "d"), &1, &3600);
    let id3 = client.create_proposal(&proposer, &String::from_str(&env, "P3"), &String::from_str(&env, "d"), &1, &3600);

    assert!(id1 != id2 && id2 != id3 && id1 != id3);
    assert_eq!(client.proposal_count(), 3);
}

/// Proposals at different lifecycle stages coexist correctly.
#[test]
fn test_proposals_at_different_lifecycle_stages() {
    let (env, client) = setup();
    let admin = Address::generate(&env);
    let voter = Address::generate(&env);
    let token_id = setup_token(&env, &voter);

    client.initialize(&admin, &token_id);
    let active_id   = client.create_proposal(&voter, &String::from_str(&env, "Active"),   &String::from_str(&env, "d"), &1,         &7200);
    let passed_id   = client.create_proposal(&voter, &String::from_str(&env, "Passed"),   &String::from_str(&env, "d"), &1,         &3600);
    let rejected_id = client.create_proposal(&voter, &String::from_str(&env, "Rejected"), &String::from_str(&env, "d"), &9_999_999, &3600);
    let cancelled_id = client.create_proposal(&voter, &String::from_str(&env, "Cancel"),  &String::from_str(&env, "d"), &1,         &3600);

    client.cast_vote(&voter, &passed_id, &Vote::Yes);
    client.cast_vote(&voter, &rejected_id, &Vote::Yes);
    client.cancel(&admin, &cancelled_id);

    env.ledger().with_mut(|l| l.timestamp += 3601);
    client.finalise(&passed_id);
    client.finalise(&rejected_id);

    assert_eq!(client.get_proposal(&active_id).status,    ProposalStatus::Active);
    assert_eq!(client.get_proposal(&passed_id).status,    ProposalStatus::Passed);
    assert_eq!(client.get_proposal(&rejected_id).status,  ProposalStatus::Rejected);
    assert_eq!(client.get_proposal(&cancelled_id).status, ProposalStatus::Cancelled);
}

// ── end TEST-009 ─────────────────────────────────────────────────────────────

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
