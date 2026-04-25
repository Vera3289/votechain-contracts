#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::{Address as _, Ledger}, Address, Env, String};
use crate::test_helpers::{setup_env, create_test_proposal, mint_and_vote};

// ── local helpers for tests that need a custom Env/client shape ───────────────

/// Register a fresh token contract, mint `supply` to `admin`, return its address.
fn setup_token(env: &Env, admin: &Address) -> Address {
    let id = env.register(votechain_token::TokenContract, ());
    let t = votechain_token::TokenContractClient::new(env, &id);
    t.initialize(admin, &1_000_000);
    id
}

fn new_client(env: &Env) -> GovernanceContractClient<'static> {
    GovernanceContractClient::new(env, &env.register(GovernanceContract, ()))
}

/// Create a passed proposal (voted Yes, finalised) for access-control tests.
fn setup_passed_proposal(env: &Env, client: &GovernanceContractClient, admin: &Address) -> u64 {
    let voter = Address::generate(env);
    let token_id = setup_token(env, &voter);
    client.initialize(admin, &token_id);
    let id = client.create_proposal(
        &voter,
        &String::from_str(env, "Prop"),
        &String::from_str(env, "desc"),
        &100,
        &3600,
    );
    client.cast_vote(&voter, &id, &Vote::Yes);
    env.ledger().with_mut(|l| l.timestamp += 3601);
    client.finalise(&id);
    id
}

/// Create an active proposal for access-control tests.
fn setup_active_proposal(env: &Env, client: &GovernanceContractClient, admin: &Address) -> u64 {
    let proposer = Address::generate(env);
    let token_id = setup_token(env, admin);
    client.initialize(admin, &token_id);
    client.create_proposal(
        &proposer,
        &String::from_str(env, "Prop"),
        &String::from_str(env, "desc"),
        &100,
        &3600,
    )
}

// ── basic lifecycle ───────────────────────────────────────────────────────────

#[test]
fn test_create_proposal() {
    let t = setup_env();
    let proposer = Address::generate(&t.env);
    let id = create_test_proposal(&t, &proposer);
    assert_eq!(id, 1);
    assert_eq!(t.client.proposal_count(), 1);
    assert_eq!(t.client.get_proposal(&id).status, ProposalStatus::Active);
}

#[test]
fn test_cast_vote_and_finalise_passed() {
    let t = setup_env();
    let voter = Address::generate(&t.env);
    let id = create_test_proposal(&t, &voter);

    mint_and_vote(&t, &voter, id, Vote::Yes, 1_000_000);
    assert!(t.client.has_voted(&id, &voter));
    assert_eq!(t.client.get_proposal(&id).votes_yes, 1_000_000);

    t.env.ledger().with_mut(|l| l.timestamp += 3601);
    t.client.finalise(&id);
    assert_eq!(t.client.get_proposal(&id).status, ProposalStatus::Passed);
}

#[test]
fn test_finalise_rejected_below_quorum() {
    let t = setup_env();
    let voter = Address::generate(&t.env);
    let id = t.client.create_proposal(
        &voter,
        &String::from_str(&t.env, "B"),
        &String::from_str(&t.env, "desc"),
        &9_999_999,
        &3600,
    );
    mint_and_vote(&t, &voter, id, Vote::Yes, 1_000_000);
    t.env.ledger().with_mut(|l| l.timestamp += 3601);
    t.client.finalise(&id);
    assert_eq!(t.client.get_proposal(&id).status, ProposalStatus::Rejected);
}

#[test]
fn test_finalise_rejected_no_wins() {
    let t = setup_env();
    let voter = Address::generate(&t.env);
    let id = create_test_proposal(&t, &voter);
    mint_and_vote(&t, &voter, id, Vote::No, 1_000_000);
    t.env.ledger().with_mut(|l| l.timestamp += 3601);
    t.client.finalise(&id);
    assert_eq!(t.client.get_proposal(&id).status, ProposalStatus::Rejected);
}

#[test]
fn test_execute_passed_proposal() {
    let t = setup_env();
    let voter = Address::generate(&t.env);
    let id = create_test_proposal(&t, &voter);
    mint_and_vote(&t, &voter, id, Vote::Yes, 1_000_000);
    t.env.ledger().with_mut(|l| l.timestamp += 3601);
    t.client.finalise(&id);
    t.client.execute(&t.admin, &id);
    assert_eq!(t.client.get_proposal(&id).status, ProposalStatus::Executed);
}

#[test]
fn test_cancel_proposal() {
    let t = setup_env();
    let proposer = Address::generate(&t.env);
    let id = create_test_proposal(&t, &proposer);
    t.client.cancel(&t.admin, &id);
    assert_eq!(t.client.get_proposal(&id).status, ProposalStatus::Cancelled);
}

// ── TEST-009: concurrent proposal scenario tests ──────────────────────────────

#[test]
fn test_concurrent_proposals_independent_votes() {
    let t = setup_env();
    let voter = Address::generate(&t.env);
    let id1 = create_test_proposal(&t, &voter);
    let id2 = create_test_proposal(&t, &voter);
    let id3 = create_test_proposal(&t, &voter);

    mint_and_vote(&t, &voter, id1, Vote::Yes, 1_000_000);
    assert!(t.client.has_voted(&id1, &voter));
    assert!(!t.client.has_voted(&id2, &voter));
    assert!(!t.client.has_voted(&id3, &voter));
}

#[test]
fn test_concurrent_votes_do_not_bleed() {
    let t = setup_env();
    let voter = Address::generate(&t.env);
    let id1 = create_test_proposal(&t, &voter);
    let id2 = create_test_proposal(&t, &voter);

    mint_and_vote(&t, &voter, id1, Vote::Yes, 1_000_000);

    assert_eq!(t.client.get_proposal(&id1).votes_yes, 1_000_000);
    let p2 = t.client.get_proposal(&id2);
    assert_eq!(p2.votes_yes, 0);
    assert_eq!(p2.votes_no, 0);
    assert_eq!(p2.votes_abstain, 0);
}

#[test]
fn test_finalise_one_does_not_affect_others() {
    let t = setup_env();
    let voter = Address::generate(&t.env);
    let id1 = create_test_proposal(&t, &voter);
    let id2 = t.client.create_proposal(
        &voter,
        &String::from_str(&t.env, "P2"),
        &String::from_str(&t.env, "d"),
        &1,
        &7200,
    );

    mint_and_vote(&t, &voter, id1, Vote::Yes, 1_000_000);
    t.env.ledger().with_mut(|l| l.timestamp += 3601);
    t.client.finalise(&id1);

    assert_ne!(t.client.get_proposal(&id1).status, ProposalStatus::Active);
    assert_eq!(t.client.get_proposal(&id2).status, ProposalStatus::Active);
}

#[test]
fn test_proposal_ids_are_unique() {
    let t = setup_env();
    let proposer = Address::generate(&t.env);
    let id1 = create_test_proposal(&t, &proposer);
    let id2 = create_test_proposal(&t, &proposer);
    let id3 = create_test_proposal(&t, &proposer);
    assert!(id1 != id2 && id2 != id3 && id1 != id3);
    assert_eq!(t.client.proposal_count(), 3);
}

#[test]
fn test_proposals_at_different_lifecycle_stages() {
    let t = setup_env();
    let voter = Address::generate(&t.env);

    let active_id    = t.client.create_proposal(&voter, &String::from_str(&t.env, "Active"),   &String::from_str(&t.env, "d"), &1,         &7200);
    let passed_id    = create_test_proposal(&t, &voter);
    let rejected_id  = t.client.create_proposal(&voter, &String::from_str(&t.env, "Rejected"), &String::from_str(&t.env, "d"), &9_999_999, &3600);
    let cancelled_id = create_test_proposal(&t, &voter);

    mint_and_vote(&t, &voter, passed_id, Vote::Yes, 1_000_000);
    t.client.cancel(&t.admin, &cancelled_id);

    t.env.ledger().with_mut(|l| l.timestamp += 3601);
    t.client.finalise(&passed_id);
    t.client.finalise(&rejected_id);

    assert_eq!(t.client.get_proposal(&active_id).status,    ProposalStatus::Active);
    assert_eq!(t.client.get_proposal(&passed_id).status,    ProposalStatus::Passed);
    assert_eq!(t.client.get_proposal(&rejected_id).status,  ProposalStatus::Rejected);
    assert_eq!(t.client.get_proposal(&cancelled_id).status, ProposalStatus::Cancelled);
}

// ── end TEST-009 ──────────────────────────────────────────────────────────────

#[test]
#[should_panic]
fn test_cannot_vote_twice() {
    let t = setup_env();
    let voter = Address::generate(&t.env);
    let id = create_test_proposal(&t, &voter);
    mint_and_vote(&t, &voter, id, Vote::Yes, 1_000_000);
    t.client.cast_vote(&voter, &id, &Vote::No); // should panic
}

// ── TEST-013: access control negative tests ───────────────────────────────────

#[test]
#[should_panic(expected = "not admin")]
fn test_execute_non_admin_reverts() {
    let env = Env::default();
    env.mock_all_auths();
    let client = new_client(&env);
    let admin = Address::generate(&env);
    let non_admin = Address::generate(&env);
    let id = setup_passed_proposal(&env, &client, &admin);
    client.execute(&non_admin, &id);
}

#[test]
#[should_panic(expected = "not admin")]
fn test_execute_zero_address_reverts() {
    let env = Env::default();
    env.mock_all_auths();
    let client = new_client(&env);
    let admin = Address::generate(&env);
    let id = setup_passed_proposal(&env, &client, &admin);
    let zero = Address::from_str(&env, "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAWHF");
    client.execute(&zero, &id);
}

#[test]
#[should_panic(expected = "not admin")]
fn test_cancel_non_admin_reverts() {
    let env = Env::default();
    env.mock_all_auths();
    let client = new_client(&env);
    let admin = Address::generate(&env);
    let non_admin = Address::generate(&env);
    let id = setup_active_proposal(&env, &client, &admin);
    client.cancel(&non_admin, &id);
}

#[test]
#[should_panic(expected = "not admin")]
fn test_cancel_zero_address_reverts() {
    let env = Env::default();
    env.mock_all_auths();
    let client = new_client(&env);
    let admin = Address::generate(&env);
    let id = setup_active_proposal(&env, &client, &admin);
    let zero = Address::from_str(&env, "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAWHF");
    client.cancel(&zero, &id);
}

// ── end TEST-013 ──────────────────────────────────────────────────────────────

// ── SC-027: update_quorum tests ───────────────────────────────────────────────

#[test]
fn test_update_quorum_success() {
    let t = setup_env();
    let proposer = Address::generate(&t.env);
    let id = create_test_proposal(&t, &proposer);
    t.client.update_quorum(&t.admin, &id, &500);
    assert_eq!(t.client.get_proposal(&id).quorum, 500);
}

#[test]
#[should_panic(expected = "not admin")]
fn test_update_quorum_non_admin_reverts() {
    let t = setup_env();
    let non_admin = Address::generate(&t.env);
    let id = create_test_proposal(&t, &t.admin.clone());
    t.client.update_quorum(&non_admin, &id, &500);
}

#[test]
#[should_panic]
fn test_update_quorum_zero_reverts() {
    let t = setup_env();
    let id = create_test_proposal(&t, &t.admin.clone());
    t.client.update_quorum(&t.admin, &id, &0);
}

#[test]
#[should_panic]
fn test_update_quorum_inactive_proposal_reverts() {
    let t = setup_env();
    let id = create_test_proposal(&t, &t.admin.clone());
    t.client.cancel(&t.admin, &id);
    t.client.update_quorum(&t.admin, &id, &500);
}

// ── end SC-027 ────────────────────────────────────────────────────────────────

// ── storage persistence tests ─────────────────────────────────────────────────

#[test]
fn test_proposal_data_persists_unchanged() {
    let t = setup_env();
    let proposer = Address::generate(&t.env);
    let id = t.client.create_proposal(
        &proposer,
        &String::from_str(&t.env, "Persist title"),
        &String::from_str(&t.env, "Persist desc"),
        &250,
        &1800,
    );
    let p = t.client.get_proposal(&id);
    assert_eq!(p.id, id);
    assert_eq!(p.title, String::from_str(&t.env, "Persist title"));
    assert_eq!(p.description, String::from_str(&t.env, "Persist desc"));
    assert_eq!(p.quorum, 250);
    assert_eq!(p.status, ProposalStatus::Active);
    assert_eq!(p.proposer, proposer);
}

#[test]
fn test_vote_records_persist_across_multiple_voters() {
    let t = setup_env();
    let voter1 = Address::generate(&t.env);
    let voter2 = Address::generate(&t.env);
    let voter3 = Address::generate(&t.env);
    let id = create_test_proposal(&t, &voter1);

    mint_and_vote(&t, &voter1, id, Vote::Yes,     300_000);
    mint_and_vote(&t, &voter2, id, Vote::No,      300_000);
    mint_and_vote(&t, &voter3, id, Vote::Abstain, 300_000);

    assert!(t.client.has_voted(&id, &voter1));
    assert!(t.client.has_voted(&id, &voter2));
    assert!(t.client.has_voted(&id, &voter3));
    let p = t.client.get_proposal(&id);
    assert!(p.votes_yes > 0);
    assert!(p.votes_no > 0);
    assert!(p.votes_abstain > 0);
}

#[test]
fn test_admin_persists_after_initialization() {
    let t = setup_env();
    let id = create_test_proposal(&t, &t.admin.clone());
    t.client.cancel(&t.admin, &id);
    assert_eq!(t.client.get_proposal(&id).status, ProposalStatus::Cancelled);
}

#[test]
fn test_no_data_lost_between_calls() {
    let t = setup_env();
    let voter = Address::generate(&t.env);
    let id1 = create_test_proposal(&t, &voter);
    let id2 = t.client.create_proposal(
        &voter,
        &String::from_str(&t.env, "P2"),
        &String::from_str(&t.env, "d2"),
        &200,
        &7200,
    );

    mint_and_vote(&t, &voter, id1, Vote::Yes, 1_000_000);

    let p2 = t.client.get_proposal(&id2);
    assert_eq!(p2.title, String::from_str(&t.env, "P2"));
    assert_eq!(p2.quorum, 200);
    assert_eq!(p2.votes_yes, 0);
    assert_eq!(p2.status, ProposalStatus::Active);
    assert!(!t.client.has_voted(&id2, &voter));
}

// ── end storage persistence tests ─────────────────────────────────────────────

// ── Issue #28: comprehensive voting scenario tests ────────────────────────────

#[test]
fn test_vote_yes_recorded_correctly() {
    let t = setup_env();
    let voter = Address::generate(&t.env);
    let id = create_test_proposal(&t, &voter);
    mint_and_vote(&t, &voter, id, Vote::Yes, 500_000);
    let p = t.client.get_proposal(&id);
    assert_eq!(p.votes_yes, 500_000);
    assert_eq!(p.votes_no, 0);
    assert_eq!(p.votes_abstain, 0);
}

#[test]
fn test_vote_no_recorded_correctly() {
    let t = setup_env();
    let voter = Address::generate(&t.env);
    let id = create_test_proposal(&t, &voter);
    mint_and_vote(&t, &voter, id, Vote::No, 750_000);
    let p = t.client.get_proposal(&id);
    assert_eq!(p.votes_yes, 0);
    assert_eq!(p.votes_no, 750_000);
    assert_eq!(p.votes_abstain, 0);
}

#[test]
fn test_vote_abstain_recorded_correctly() {
    let t = setup_env();
    let voter = Address::generate(&t.env);
    let id = create_test_proposal(&t, &voter);
    mint_and_vote(&t, &voter, id, Vote::Abstain, 250_000);
    let p = t.client.get_proposal(&id);
    assert_eq!(p.votes_yes, 0);
    assert_eq!(p.votes_no, 0);
    assert_eq!(p.votes_abstain, 250_000);
}

#[test]
fn test_vote_weight_matches_token_balance() {
    let t = setup_env();
    let voter = Address::generate(&t.env);
    let id = create_test_proposal(&t, &voter);
    let balance = 1_234_567;
    mint_and_vote(&t, &voter, id, Vote::Yes, balance);
    let p = t.client.get_proposal(&id);
    assert_eq!(p.votes_yes, balance);
}

#[test]
#[should_panic(expected = "already voted")]
fn test_double_vote_same_choice_reverts() {
    let t = setup_env();
    let voter = Address::generate(&t.env);
    let id = create_test_proposal(&t, &voter);
    mint_and_vote(&t, &voter, id, Vote::Yes, 1_000_000);
    t.client.cast_vote(&voter, &id, &Vote::Yes);
}

#[test]
#[should_panic(expected = "already voted")]
fn test_double_vote_different_choice_reverts() {
    let t = setup_env();
    let voter = Address::generate(&t.env);
    let id = create_test_proposal(&t, &voter);
    mint_and_vote(&t, &voter, id, Vote::Yes, 1_000_000);
    t.client.cast_vote(&voter, &id, &Vote::No);
}

#[test]
#[should_panic(expected = "not active")]
fn test_vote_on_passed_proposal_reverts() {
    let t = setup_env();
    let voter = Address::generate(&t.env);
    let id = create_test_proposal(&t, &voter);
    mint_and_vote(&t, &voter, id, Vote::Yes, 1_000_000);
    t.env.ledger().with_mut(|l| l.timestamp += 3601);
    t.client.finalise(&id);
    let voter2 = Address::generate(&t.env);
    mint_and_vote(&t, &voter2, id, Vote::Yes, 500_000);
}

#[test]
#[should_panic(expected = "not active")]
fn test_vote_on_rejected_proposal_reverts() {
    let t = setup_env();
    let voter = Address::generate(&t.env);
    let id = create_test_proposal(&t, &voter);
    mint_and_vote(&t, &voter, id, Vote::No, 1_000_000);
    t.env.ledger().with_mut(|l| l.timestamp += 3601);
    t.client.finalise(&id);
    let voter2 = Address::generate(&t.env);
    mint_and_vote(&t, &voter2, id, Vote::Yes, 500_000);
}

#[test]
#[should_panic(expected = "not active")]
fn test_vote_on_cancelled_proposal_reverts() {
    let t = setup_env();
    let voter = Address::generate(&t.env);
    let id = create_test_proposal(&t, &voter);
    t.client.cancel(&t.admin, &id);
    mint_and_vote(&t, &voter, id, Vote::Yes, 1_000_000);
}

#[test]
#[should_panic(expected = "not active")]
fn test_vote_on_executed_proposal_reverts() {
    let t = setup_env();
    let voter = Address::generate(&t.env);
    let id = create_test_proposal(&t, &voter);
    mint_and_vote(&t, &voter, id, Vote::Yes, 1_000_000);
    t.env.ledger().with_mut(|l| l.timestamp += 3601);
    t.client.finalise(&id);
    t.client.execute(&t.admin, &id);
    let voter2 = Address::generate(&t.env);
    mint_and_vote(&t, &voter2, id, Vote::Yes, 500_000);
}

#[test]
#[should_panic(expected = "voting period ended")]
fn test_vote_after_end_time_reverts() {
    let t = setup_env();
    let voter = Address::generate(&t.env);
    let id = create_test_proposal(&t, &voter);
    t.env.ledger().with_mut(|l| l.timestamp += 3601);
    mint_and_vote(&t, &voter, id, Vote::Yes, 1_000_000);
}

#[test]
fn test_vote_at_exact_end_time_reverts() {
    let t = setup_env();
    let voter = Address::generate(&t.env);
    let now = t.env.ledger().timestamp();
    let id = t.client.create_proposal(
        &voter,
        &String::from_str(&t.env, "Test"),
        &String::from_str(&t.env, "desc"),
        &1,
        &3600,
    );
    t.env.ledger().with_mut(|l| l.timestamp = now + 3600);
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        mint_and_vote(&t, &voter, id, Vote::Yes, 1_000_000);
    }));
    assert!(result.is_err());
}

#[test]
#[should_panic(expected = "no voting power")]
fn test_vote_with_zero_balance_reverts() {
    let t = setup_env();
    let voter = Address::generate(&t.env);
    let id = create_test_proposal(&t, &voter);
    t.client.cast_vote(&voter, &id, &Vote::Yes);
}

#[test]
fn test_vote_tallies_accumulate_correctly() {
    let t = setup_env();
    let voter1 = Address::generate(&t.env);
    let voter2 = Address::generate(&t.env);
    let voter3 = Address::generate(&t.env);
    let id = create_test_proposal(&t, &voter1);
    
    mint_and_vote(&t, &voter1, id, Vote::Yes, 100_000);
    mint_and_vote(&t, &voter2, id, Vote::Yes, 200_000);
    mint_and_vote(&t, &voter3, id, Vote::No, 150_000);
    
    let p = t.client.get_proposal(&id);
    assert_eq!(p.votes_yes, 300_000);
    assert_eq!(p.votes_no, 150_000);
    assert_eq!(p.votes_abstain, 0);
}

#[test]
fn test_vote_tallies_all_three_types() {
    let t = setup_env();
    let v1 = Address::generate(&t.env);
    let v2 = Address::generate(&t.env);
    let v3 = Address::generate(&t.env);
    let v4 = Address::generate(&t.env);
    let v5 = Address::generate(&t.env);
    let id = create_test_proposal(&t, &v1);
    
    mint_and_vote(&t, &v1, id, Vote::Yes, 100_000);
    mint_and_vote(&t, &v2, id, Vote::Yes, 200_000);
    mint_and_vote(&t, &v3, id, Vote::No, 150_000);
    mint_and_vote(&t, &v4, id, Vote::No, 50_000);
    mint_and_vote(&t, &v5, id, Vote::Abstain, 75_000);
    
    let p = t.client.get_proposal(&id);
    assert_eq!(p.votes_yes, 300_000);
    assert_eq!(p.votes_no, 200_000);
    assert_eq!(p.votes_abstain, 75_000);
}

// ── end Issue #28 ─────────────────────────────────────────────────────────────

// ── SEC-009: re-initialization guard tests ────────────────────────────────────

/// Re-init by the original admin must revert with AlreadyInitialized.
#[test]
#[should_panic]
fn test_reinit_by_original_admin_reverts() {
    let t = setup_env();
    t.client.initialize(&t.admin, &t.token_id);
}

/// Re-init by a new address must revert with AlreadyInitialized.
#[test]
#[should_panic]
fn test_reinit_by_new_address_reverts() {
    let t = setup_env();
    let attacker = Address::generate(&t.env);
    let new_token = Address::generate(&t.env);
    t.client.initialize(&attacker, &new_token);
}

/// Re-init by the zero address must revert with AlreadyInitialized.
#[test]
#[should_panic]
fn test_reinit_by_zero_address_reverts() {
    let t = setup_env();
    let zero = Address::from_str(&t.env, "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAWHF");
    t.client.initialize(&zero, &t.token_id);
}

// ── end SEC-009 ───────────────────────────────────────────────────────────────

// ── Issue #77: comprehensive event verification tests ─────────────────────────

use soroban_sdk::{testutils::Events, IntoVal};

#[test]
fn test_event_proposal_created_topics_and_data() {
    let t = setup_env();
    let proposer = Address::generate(&t.env);
    let id = create_test_proposal(&t, &proposer);
    
    let events = t.env.events().all();
    let created_event = events.iter().find(|(_, topics, _)| {
        topics == &(symbol_short!("created"), id).into_val(&t.env)
    });
    
    assert!(created_event.is_some());
    let (_, topics, data) = created_event.unwrap();
    assert_eq!(topics, (symbol_short!("created"), id).into_val(&t.env));
    assert_eq!(data, proposer.into_val(&t.env));
}

#[test]
fn test_event_vote_cast_topics_and_data() {
    let t = setup_env();
    let voter = Address::generate(&t.env);
    let id = create_test_proposal(&t, &voter);
    let weight = 1_000_000_i128;
    
    mint_and_vote(&t, &voter, id, Vote::Yes, weight);
    
    let events = t.env.events().all();
    let vote_event = events.iter().find(|(_, topics, _)| {
        topics == &(symbol_short!("vote"), id).into_val(&t.env)
    });
    
    assert!(vote_event.is_some());
    let (_, topics, data) = vote_event.unwrap();
    assert_eq!(topics, (symbol_short!("vote"), id).into_val(&t.env));
    assert_eq!(data, (voter.clone(), Vote::Yes, weight).into_val(&t.env));
}

#[test]
fn test_event_proposal_finalised_passed() {
    let t = setup_env();
    let voter = Address::generate(&t.env);
    let id = create_test_proposal(&t, &voter);
    mint_and_vote(&t, &voter, id, Vote::Yes, 1_000_000);
    
    t.env.ledger().with_mut(|l| l.timestamp += 3601);
    t.client.finalise(&id);
    
    let events = t.env.events().all();
    let final_event = events.iter().find(|(_, topics, data)| {
        topics == &(symbol_short!("final"), id).into_val(&t.env)
            && data == &ProposalStatus::Passed.into_val(&t.env)
    });
    
    assert!(final_event.is_some());
}

#[test]
fn test_event_proposal_finalised_rejected() {
    let t = setup_env();
    let voter = Address::generate(&t.env);
    let id = create_test_proposal(&t, &voter);
    mint_and_vote(&t, &voter, id, Vote::No, 1_000_000);
    
    t.env.ledger().with_mut(|l| l.timestamp += 3601);
    t.client.finalise(&id);
    
    let events = t.env.events().all();
    let final_event = events.iter().find(|(_, topics, data)| {
        topics == &(symbol_short!("final"), id).into_val(&t.env)
            && data == &ProposalStatus::Rejected.into_val(&t.env)
    });
    
    assert!(final_event.is_some());
}

#[test]
fn test_event_proposal_executed() {
    let t = setup_env();
    let voter = Address::generate(&t.env);
    let id = create_test_proposal(&t, &voter);
    mint_and_vote(&t, &voter, id, Vote::Yes, 1_000_000);
    
    t.env.ledger().with_mut(|l| l.timestamp += 3601);
    t.client.finalise(&id);
    t.client.execute(&t.admin, &id);
    
    let events = t.env.events().all();
    let exec_event = events.iter().find(|(_, topics, data)| {
        topics == &(symbol_short!("final"), id).into_val(&t.env)
            && data == &ProposalStatus::Executed.into_val(&t.env)
    });
    
    assert!(exec_event.is_some());
}

#[test]
fn test_event_proposal_cancelled() {
    let t = setup_env();
    let proposer = Address::generate(&t.env);
    let id = create_test_proposal(&t, &proposer);
    
    t.client.cancel(&t.admin, &id);
    
    let events = t.env.events().all();
    let cancel_event = events.iter().find(|(_, topics, data)| {
        topics == &(symbol_short!("final"), id).into_val(&t.env)
            && data == &ProposalStatus::Cancelled.into_val(&t.env)
    });
    
    assert!(cancel_event.is_some());
}

#[test]
fn test_event_quorum_updated() {
    let t = setup_env();
    let proposer = Address::generate(&t.env);
    let id = create_test_proposal(&t, &proposer);
    let new_quorum = 500_i128;
    
    t.client.update_quorum(&t.admin, &id, &new_quorum);
    
    let events = t.env.events().all();
    let quorum_event = events.iter().find(|(_, topics, data)| {
        topics == &(symbol_short!("qupdate"), id).into_val(&t.env)
            && data == &new_quorum.into_val(&t.env)
    });
    
    assert!(quorum_event.is_some());
}

#[test]
fn test_event_vote_yes_data_fields() {
    let t = setup_env();
    let voter = Address::generate(&t.env);
    let id = create_test_proposal(&t, &voter);
    let weight = 750_000_i128;
    
    mint_and_vote(&t, &voter, id, Vote::Yes, weight);
    
    let events = t.env.events().all();
    let vote_event = events.iter().find(|(_, topics, _)| {
        topics == &(symbol_short!("vote"), id).into_val(&t.env)
    });
    
    assert!(vote_event.is_some());
    let (_, _, data) = vote_event.unwrap();
    assert_eq!(data, (voter.clone(), Vote::Yes, weight).into_val(&t.env));
}

#[test]
fn test_event_vote_no_data_fields() {
    let t = setup_env();
    let voter = Address::generate(&t.env);
    let id = create_test_proposal(&t, &voter);
    let weight = 250_000_i128;
    
    mint_and_vote(&t, &voter, id, Vote::No, weight);
    
    let events = t.env.events().all();
    let vote_event = events.iter().find(|(_, topics, _)| {
        topics == &(symbol_short!("vote"), id).into_val(&t.env)
    });
    
    assert!(vote_event.is_some());
    let (_, _, data) = vote_event.unwrap();
    assert_eq!(data, (voter.clone(), Vote::No, weight).into_val(&t.env));
}

#[test]
fn test_event_vote_abstain_data_fields() {
    let t = setup_env();
    let voter = Address::generate(&t.env);
    let id = create_test_proposal(&t, &voter);
    let weight = 500_000_i128;
    
    mint_and_vote(&t, &voter, id, Vote::Abstain, weight);
    
    let events = t.env.events().all();
    let vote_event = events.iter().find(|(_, topics, _)| {
        topics == &(symbol_short!("vote"), id).into_val(&t.env)
    });
    
    assert!(vote_event.is_some());
    let (_, _, data) = vote_event.unwrap();
    assert_eq!(data, (voter.clone(), Vote::Abstain, weight).into_val(&t.env));
}

#[test]
fn test_no_unexpected_events_on_create() {
    let t = setup_env();
    let proposer = Address::generate(&t.env);
    
    let events_before = t.env.events().all().len();
    let id = create_test_proposal(&t, &proposer);
    let events_after = t.env.events().all();
    
    let new_events: Vec<_> = events_after.iter().skip(events_before).collect();
    assert_eq!(new_events.len(), 1);
    
    let (_, topics, _) = new_events[0];
    assert_eq!(topics, (symbol_short!("created"), id).into_val(&t.env));
}

#[test]
fn test_no_unexpected_events_on_vote() {
    let t = setup_env();
    let voter = Address::generate(&t.env);
    let id = create_test_proposal(&t, &voter);
    
    let events_before = t.env.events().all().len();
    mint_and_vote(&t, &voter, id, Vote::Yes, 1_000_000);
    let events_after = t.env.events().all();
    
    let new_events: Vec<_> = events_after.iter().skip(events_before).collect();
    // Should have exactly 1 vote event (mint events are from token contract)
    let vote_events: Vec<_> = new_events.iter().filter(|(_, topics, _)| {
        topics == &(symbol_short!("vote"), id).into_val(&t.env)
    }).collect();
    assert_eq!(vote_events.len(), 1);
}

#[test]
fn test_multiple_votes_emit_separate_events() {
    let t = setup_env();
    let v1 = Address::generate(&t.env);
    let v2 = Address::generate(&t.env);
    let v3 = Address::generate(&t.env);
    let id = create_test_proposal(&t, &v1);
    
    mint_and_vote(&t, &v1, id, Vote::Yes, 100_000);
    mint_and_vote(&t, &v2, id, Vote::No, 200_000);
    mint_and_vote(&t, &v3, id, Vote::Abstain, 300_000);
    
    let events = t.env.events().all();
    let vote_events: Vec<_> = events.iter().filter(|(_, topics, _)| {
        topics == &(symbol_short!("vote"), id).into_val(&t.env)
    }).collect();
    
    assert_eq!(vote_events.len(), 3);
}

// ── end Issue #77 ─────────────────────────────────────────────────────────────
