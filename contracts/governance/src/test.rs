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

// ── Issue #28: token contract error handling tests ───────────────────────────

/// Mock token contract that always fails balance queries
#[cfg(test)]
mod failing_token {
    use soroban_sdk::{contract, contractimpl, Address, Env};

    #[contract]
    pub struct FailingTokenContract;

    #[contractimpl]
    impl FailingTokenContract {
        pub fn balance(_env: Env, _owner: Address) -> i128 {
            panic!("Token contract error");
        }
    }
}

#[test]
#[should_panic(expected = "token contract")]
fn test_token_contract_failure_reverts_with_error() {
    let env = Env::default();
    env.mock_all_auths();
    
    // Register governance contract
    let gov_id = env.register(GovernanceContract, ());
    let client = GovernanceContractClient::new(&env, &gov_id);
    
    let admin = Address::generate(&env);
    
    // Register a failing token contract
    let failing_token_id = env.register(failing_token::FailingTokenContract, ());
    
    // Initialize governance with the failing token
    client.initialize(&admin, &failing_token_id);
    
    // Create a proposal
    let proposer = Address::generate(&env);
    let id = client.create_proposal(
        &proposer,
        &String::from_str(&env, "Test"),
        &String::from_str(&env, "desc"),
        &100,
        &3600,
    );
    
    // Attempt to vote - should revert with TokenContractError
    let voter = Address::generate(&env);
    client.cast_vote(&voter, &id, &Vote::Yes);
}

#[test]
fn test_token_contract_failure_no_partial_state_written() {
    let env = Env::default();
    env.mock_all_auths();
    
    let gov_id = env.register(GovernanceContract, ());
    let client = GovernanceContractClient::new(&env, &gov_id);
    
    let admin = Address::generate(&env);
    let failing_token_id = env.register(failing_token::FailingTokenContract, ());
    
    client.initialize(&admin, &failing_token_id);
    
    let proposer = Address::generate(&env);
    let id = client.create_proposal(
        &proposer,
        &String::from_str(&env, "Test"),
        &String::from_str(&env, "desc"),
        &100,
        &3600,
    );
    
    let voter = Address::generate(&env);
    
    // Verify voter hasn't voted yet
    assert!(!client.has_voted(&id, &voter));
    
    // Attempt to vote with failing token contract
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        client.cast_vote(&voter, &id, &Vote::Yes);
    }));
    
    // Should have failed
    assert!(result.is_err());
    
    // Verify no partial state was written
    assert!(!client.has_voted(&id, &voter));
    
    let proposal = client.get_proposal(&id);
    assert_eq!(proposal.votes_yes, 0);
    assert_eq!(proposal.votes_no, 0);
    assert_eq!(proposal.votes_abstain, 0);
}

#[test]
fn test_token_contract_success_after_previous_failure() {
    let env = Env::default();
    env.mock_all_auths();
    
    let gov_id = env.register(GovernanceContract, ());
    let client = GovernanceContractClient::new(&env, &gov_id);
    
    let admin = Address::generate(&env);
    
    // First, use a failing token
    let failing_token_id = env.register(failing_token::FailingTokenContract, ());
    client.initialize(&admin, &failing_token_id);
    
    let proposer = Address::generate(&env);
    let id = client.create_proposal(
        &proposer,
        &String::from_str(&env, "Test"),
        &String::from_str(&env, "desc"),
        &100,
        &3600,
    );
    
    let voter = Address::generate(&env);
    
    // Attempt to vote with failing token - should fail
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        client.cast_vote(&voter, &id, &Vote::Yes);
    }));
    assert!(result.is_err());
    
    // Now create a new governance instance with a working token
    let gov_id2 = env.register(GovernanceContract, ());
    let client2 = GovernanceContractClient::new(&env, &gov_id2);
    
    let tok_id = env.register(votechain_token::TokenContract, ());
    let tok = votechain_token::TokenContractClient::new(&env, &tok_id);
    tok.initialize(&admin, &1_000_000);
    
    client2.initialize(&admin, &tok_id);
    
    let id2 = client2.create_proposal(
        &proposer,
        &String::from_str(&env, "Test2"),
        &String::from_str(&env, "desc"),
        &100,
        &3600,
    );
    
    // Mint tokens and vote - should succeed
    tok.mint(&admin, &voter, &500_000);
    client2.cast_vote(&voter, &id2, &Vote::Yes);
    
    // Verify vote was recorded
    assert!(client2.has_voted(&id2, &voter));
    assert_eq!(client2.get_proposal(&id2).votes_yes, 500_000);
}

#[test]
fn test_token_contract_error_propagated_with_context() {
    let env = Env::default();
    env.mock_all_auths();
    
    let gov_id = env.register(GovernanceContract, ());
    let client = GovernanceContractClient::new(&env, &gov_id);
    
    let admin = Address::generate(&env);
    let failing_token_id = env.register(failing_token::FailingTokenContract, ());
    
    client.initialize(&admin, &failing_token_id);
    
    let proposer = Address::generate(&env);
    let id = client.create_proposal(
        &proposer,
        &String::from_str(&env, "Test"),
        &String::from_str(&env, "desc"),
        &100,
        &3600,
    );
    
    let voter = Address::generate(&env);
    
    // The error should be propagated (panic in this test environment)
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        client.cast_vote(&voter, &id, &Vote::Yes);
    }));
    
    assert!(result.is_err());
    
    // Verify the error message contains context about token contract
    if let Err(e) = result {
        let msg = format!("{:?}", e);
        assert!(msg.to_lowercase().contains("token") || msg.to_lowercase().contains("contract"));
    }
}

#[test]
fn test_multiple_voters_one_token_failure_doesnt_affect_others() {
    let t = setup_env();
    let voter1 = Address::generate(&t.env);
    let voter2 = Address::generate(&t.env);
    let id = create_test_proposal(&t, &voter1);
    
    // Voter 1 votes successfully
    mint_and_vote(&t, &voter1, id, Vote::Yes, 100_000);
    assert!(t.client.has_voted(&id, &voter1));
    assert_eq!(t.client.get_proposal(&id).votes_yes, 100_000);
    
    // Even if voter2 would fail (simulated by not minting tokens),
    // voter1's vote should remain intact
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        t.client.cast_vote(&voter2, &id, &Vote::Yes);
    }));
    
    // Voter2 should fail (no voting power)
    assert!(result.is_err());
    
    // Voter1's vote should still be there
    assert!(t.client.has_voted(&id, &voter1));
    assert_eq!(t.client.get_proposal(&id).votes_yes, 100_000);
    assert!(!t.client.has_voted(&id, &voter2));
}

// ── end Issue #28 ─────────────────────────────────────────────────────────────
