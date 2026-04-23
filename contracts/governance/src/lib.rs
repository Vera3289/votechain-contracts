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
use types::{ContractError, DataKey, Proposal, ProposalStatus, Vote};

#[contract]
pub struct GovernanceContract;

#[contractimpl]
impl GovernanceContract {
    pub fn initialize(env: Env, admin: Address, voting_token: Address) {
        admin.require_auth();
        set_admin(&env, &admin);
        set_voting_token(&env, &voting_token);
    }

    pub fn create_proposal(
        env: Env,
        proposer: Address,
        title: String,
        description: String,
        quorum: i128,
        duration: u64,
    ) -> Result<u64, ContractError> {
        proposer.require_auth();
        if quorum <= 0 { return Err(ContractError::InvalidQuorum); }
        if duration == 0 { return Err(ContractError::InvalidDuration); }

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
        Ok(id)
    }

    pub fn cast_vote(env: Env, voter: Address, proposal_id: u64, vote: Vote) -> Result<(), ContractError> {
        voter.require_auth();

        let mut proposal = load_proposal(&env, proposal_id)?;
        if proposal.status != ProposalStatus::Active {
            return Err(ContractError::ProposalNotActive);
        }

        let now = env.ledger().timestamp();
        if now > proposal.end_time { return Err(ContractError::VotingPeriodEnded); }
        if has_voted(&env, proposal_id, &voter) { return Err(ContractError::AlreadyVoted); }

        let token_client = token::Client::new(&env, &get_voting_token(&env)?);
        let weight = token_client.balance(&voter);
        if weight <= 0 { return Err(ContractError::NoVotingPower); }

        match vote {
            Vote::Yes     => proposal.votes_yes     = proposal.votes_yes.checked_add(weight).expect("vote tally overflow"),
            Vote::No      => proposal.votes_no      = proposal.votes_no.checked_add(weight).expect("vote tally overflow"),
            Vote::Abstain => proposal.votes_abstain = proposal.votes_abstain.checked_add(weight).expect("vote tally overflow"),
        }

        mark_voted(&env, proposal_id, &voter);
        save_proposal(&env, &proposal);
        events::vote_cast(&env, proposal_id, &voter, &vote, weight);
        Ok(())
    }

    pub fn finalise(env: Env, proposal_id: u64) -> Result<(), ContractError> {
        let mut proposal = load_proposal(&env, proposal_id)?;
        if proposal.status != ProposalStatus::Active {
            return Err(ContractError::ProposalNotActive);
        }
        if env.ledger().timestamp() <= proposal.end_time {
            return Err(ContractError::VotingStillOpen);
        }

        let total = proposal.votes_yes + proposal.votes_no + proposal.votes_abstain;
        proposal.status = if total >= proposal.quorum && proposal.votes_yes > proposal.votes_no {
            ProposalStatus::Passed
        } else {
            ProposalStatus::Rejected
        };

        save_proposal(&env, &proposal);
        events::proposal_finalised(&env, proposal_id, &proposal.status);
        Ok(())
    }

    pub fn execute(env: Env, admin: Address, proposal_id: u64) -> Result<(), ContractError> {
        admin.require_auth();
        if get_admin(&env)? != admin { return Err(ContractError::NotAdmin); }
        let mut proposal = load_proposal(&env, proposal_id)?;
        if proposal.status != ProposalStatus::Passed {
            return Err(ContractError::ProposalNotPassed);
        }
        proposal.status = ProposalStatus::Executed;
        save_proposal(&env, &proposal);
        events::proposal_finalised(&env, proposal_id, &ProposalStatus::Executed);
        Ok(())
    }

    pub fn cancel(env: Env, admin: Address, proposal_id: u64) -> Result<(), ContractError> {
        admin.require_auth();
        if get_admin(&env)? != admin { return Err(ContractError::NotAdmin); }
        let mut proposal = load_proposal(&env, proposal_id)?;
        if proposal.status != ProposalStatus::Active {
            return Err(ContractError::ProposalNotActive);
        }
        proposal.status = ProposalStatus::Cancelled;
        save_proposal(&env, &proposal);
        events::proposal_finalised(&env, proposal_id, &ProposalStatus::Cancelled);
        Ok(())
    }

    pub fn get_proposal(env: Env, proposal_id: u64) -> Result<Proposal, ContractError> {
        load_proposal(&env, proposal_id)
    }

    pub fn proposal_count(env: Env) -> u64 {
        env.storage().instance().get(&DataKey::ProposalCount).unwrap_or(0)
    }

    pub fn has_voted(env: Env, proposal_id: u64, voter: Address) -> bool {
        has_voted(&env, proposal_id, &voter)
    }
}
