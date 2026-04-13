#![no_std]

mod events;
mod storage;
mod types;

#[cfg(test)]
mod test;

use soroban_sdk::{contract, contractimpl, token, Address, Env, String};
use storage::{
    get_admin, get_voting_token, has_voted, load_proposal, mark_voted,
    next_id, save_proposal, set_admin, set_voting_token,
};
use types::{DataKey, Proposal, ProposalStatus, Vote};

#[contract]
pub struct GovernanceContract;

#[contractimpl]
impl GovernanceContract {
    pub fn initialize(env: Env, admin: Address, voting_token: Address) {
        admin.require_auth();
        set_admin(&env, &admin);
        set_voting_token(&env, &voting_token);
    }

    /// Create a proposal. `duration` is seconds the vote stays open.
    pub fn create_proposal(
        env: Env,
        proposer: Address,
        title: String,
        description: String,
        quorum: i128,
        duration: u64,
    ) -> u64 {
        proposer.require_auth();
        assert!(quorum > 0, "quorum must be positive");
        assert!(duration > 0, "duration must be positive");

        let now = env.ledger().timestamp();
        let id = next_id(&env);
        let proposal = Proposal {
            id,
            proposer: proposer.clone(),
            title,
            description,
            votes_yes: 0,
            votes_no: 0,
            votes_abstain: 0,
            quorum,
            start_time: now,
            end_time: now + duration,
            status: ProposalStatus::Active,
        };
        save_proposal(&env, &proposal);
        events::proposal_created(&env, id, &proposer);
        id
    }

    /// Cast a vote. Weight = voter's token balance at time of vote.
    pub fn cast_vote(env: Env, voter: Address, proposal_id: u64, vote: Vote) {
        voter.require_auth();

        let mut proposal = load_proposal(&env, proposal_id).expect("proposal not found");
        assert_eq!(proposal.status, ProposalStatus::Active, "proposal not active");

        let now = env.ledger().timestamp();
        assert!(now <= proposal.end_time, "voting period has ended");
        assert!(!has_voted(&env, proposal_id, &voter), "already voted");

        // Token-weighted: weight = voter's balance
        let token_client = token::Client::new(&env, &get_voting_token(&env));
        let weight = token_client.balance(&voter);
        assert!(weight > 0, "no voting power");

        match vote {
            Vote::Yes     => proposal.votes_yes     += weight,
            Vote::No      => proposal.votes_no      += weight,
            Vote::Abstain => proposal.votes_abstain += weight,
        }

        mark_voted(&env, proposal_id, &voter);
        save_proposal(&env, &proposal);
        events::vote_cast(&env, proposal_id, &voter, &vote, weight);
    }

    /// Finalise a proposal after its voting period ends.
    pub fn finalise(env: Env, proposal_id: u64) {
        let mut proposal = load_proposal(&env, proposal_id).expect("proposal not found");
        assert_eq!(proposal.status, ProposalStatus::Active, "already finalised");
        assert!(env.ledger().timestamp() > proposal.end_time, "voting still open");

        let total = proposal.votes_yes + proposal.votes_no + proposal.votes_abstain;
        proposal.status = if total >= proposal.quorum && proposal.votes_yes > proposal.votes_no {
            ProposalStatus::Passed
        } else {
            ProposalStatus::Rejected
        };

        save_proposal(&env, &proposal);
        events::proposal_finalised(&env, proposal_id, &proposal.status);
    }

    /// Admin marks a passed proposal as executed.
    pub fn execute(env: Env, admin: Address, proposal_id: u64) {
        admin.require_auth();
        assert_eq!(get_admin(&env), admin, "not admin");
        let mut proposal = load_proposal(&env, proposal_id).expect("proposal not found");
        assert_eq!(proposal.status, ProposalStatus::Passed, "proposal not passed");
        proposal.status = ProposalStatus::Executed;
        save_proposal(&env, &proposal);
        events::proposal_finalised(&env, proposal_id, &ProposalStatus::Executed);
    }

    /// Admin cancels an active proposal.
    pub fn cancel(env: Env, admin: Address, proposal_id: u64) {
        admin.require_auth();
        assert_eq!(get_admin(&env), admin, "not admin");
        let mut proposal = load_proposal(&env, proposal_id).expect("proposal not found");
        assert_eq!(proposal.status, ProposalStatus::Active, "proposal not active");
        proposal.status = ProposalStatus::Cancelled;
        save_proposal(&env, &proposal);
        events::proposal_finalised(&env, proposal_id, &ProposalStatus::Cancelled);
    }

    pub fn get_proposal(env: Env, proposal_id: u64) -> Proposal {
        load_proposal(&env, proposal_id).expect("proposal not found")
    }

    pub fn proposal_count(env: Env) -> u64 {
        env.storage().instance().get(&DataKey::ProposalCount).unwrap_or(0)
    }

    pub fn has_voted(env: Env, proposal_id: u64, voter: Address) -> bool {
        has_voted(&env, proposal_id, &voter)
    }
}
