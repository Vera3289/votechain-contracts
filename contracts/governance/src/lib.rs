#![no_std]

mod events;
mod storage;
mod types;

#[cfg(test)]
mod test;
#[cfg(test)]
pub mod test_helpers;
#[cfg(test)]
mod prop_tests;

use soroban_sdk::{contract, contractimpl, token, Address, Env, String};
use storage::{
    get_admin, get_last_proposal, get_min_proposal_balance, get_proposal_cooldown,
    get_version, get_voting_token, has_voted, is_initialized, load_proposal, mark_voted,
    next_id, save_proposal, set_admin, set_last_proposal, set_min_proposal_balance,
    set_proposal_cooldown, set_version, set_voting_token,
};
use types::{ContractError, DataKey, Proposal, ProposalState, Vote};

const MAX_TITLE_LEN: u32 = 256;
const MAX_DESC_LEN: u32 = 4096;

#[contract]
pub struct GovernanceContract;

#[contractimpl]
impl GovernanceContract {
    /// Initialises the governance contract with an admin and a voting token.
    ///
    /// Must be called exactly once before any other function.
    ///
    /// # Errors
    /// - [`ContractError::AlreadyInitialized`] if the contract has already been initialised.
    pub fn initialize(
        env: Env,
        admin: Address,
        voting_token: Address,
        min_proposal_balance: i128,
        proposal_cooldown: u64,
    ) -> Result<(), ContractError> {
        if is_initialized(&env) {
            return Err(ContractError::AlreadyInitialized);
        }
        admin.require_auth();
        set_admin(&env, &admin);
        set_voting_token(&env, &voting_token);
        if min_proposal_balance > 0 {
            set_min_proposal_balance(&env, min_proposal_balance);
        }
        if proposal_cooldown > 0 {
            set_proposal_cooldown(&env, proposal_cooldown);
        }
        set_version(&env, (1, 0, 0));
        Ok(())
    }

    /// Creates a new governance proposal.
    ///
    /// # Returns
    /// The numeric ID assigned to the new proposal.
    ///
    /// # Errors
    /// - [`ContractError::InvalidQuorum`] if `quorum` is zero or negative.
    /// - [`ContractError::InvalidDuration`] if `duration` is zero.
    /// - [`ContractError::InsufficientBalance`] if proposer balance is below minimum.
    /// - [`ContractError::ProposalCooldown`] if proposer is within cooldown period.
    pub fn create_proposal(
        env: Env,
        proposer: Address,
        title: String,
        description: String,
        quorum: i128,
        duration: u64,
    ) -> Result<u64, ContractError> {
        proposer.require_auth();
        if quorum <= 0 {
            return Err(ContractError::InvalidQuorum);
        }
        if duration == 0 {
            return Err(ContractError::InvalidDuration);
        }
        if title.len() > MAX_TITLE_LEN {
            return Err(ContractError::TitleTooLong);
        }
        if description.len() > MAX_DESC_LEN {
            return Err(ContractError::DescriptionTooLong);
        }

        let token_client = token::Client::new(&env, &get_voting_token(&env)?);

        let min_balance = get_min_proposal_balance(&env);
        if min_balance > 0 {
            let balance = token_client.balance(&proposer);
            if balance < min_balance {
                return Err(ContractError::InsufficientBalance);
            }
        }

        let cooldown = get_proposal_cooldown(&env);
        if cooldown > 0 {
            let now = env.ledger().timestamp();
            let last = get_last_proposal(&env, &proposer);
            if last > 0 && now < last + cooldown {
                return Err(ContractError::ProposalCooldown);
            }
        }

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
            status: ProposalState::Active,
        };
        save_proposal(&env, &proposal);
        set_last_proposal(&env, &proposer, now);
        events::proposal_created(&env, id, &proposer);
        Ok(id)
    }

    /// Casts a vote on an active proposal.
    ///
    /// # Errors
    /// - [`ContractError::ProposalNotFound`] if `proposal_id` does not exist.
    /// - [`ContractError::ProposalNotActive`] if the proposal is not in `Active` status.
    /// - [`ContractError::VotingPeriodEnded`] if the voting window has closed.
    /// - [`ContractError::AlreadyVoted`] if the voter has already voted on this proposal.
    /// - [`ContractError::NoVotingPower`] if the voter's token balance is zero.
    /// - [`ContractError::VoteTallyOverflow`] if adding the vote weight would overflow `i128`.
    pub fn cast_vote(
        env: Env,
        voter: Address,
        proposal_id: u64,
        vote: Vote,
    ) -> Result<(), ContractError> {
        voter.require_auth();

        let proposal = load_proposal(&env, proposal_id)?;
        if proposal.status != ProposalState::Active {
            return Err(ContractError::ProposalNotActive);
        }

        let now = env.ledger().timestamp();
        if now > proposal.end_time {
            return Err(ContractError::VotingPeriodEnded);
        }
        if has_voted(&env, proposal_id, &voter) {
            return Err(ContractError::AlreadyVoted);
        }

        let token_client = token::Client::new(&env, &get_voting_token(&env)?);
        let weight = token_client.balance(&voter);
        if weight <= 0 {
            return Err(ContractError::NoVotingPower);
        }

        let mut proposal = proposal;
        match vote {
            Vote::Yes => {
                proposal.votes_yes = proposal
                    .votes_yes
                    .checked_add(weight)
                    .ok_or(ContractError::VoteTallyOverflow)?
            }
            Vote::No => {
                proposal.votes_no = proposal
                    .votes_no
                    .checked_add(weight)
                    .ok_or(ContractError::VoteTallyOverflow)?
            }
            Vote::Abstain => {
                proposal.votes_abstain = proposal
                    .votes_abstain
                    .checked_add(weight)
                    .ok_or(ContractError::VoteTallyOverflow)?
            }
        }

        mark_voted(&env, proposal_id, &voter);
        save_proposal(&env, &proposal);
        events::vote_cast(&env, proposal_id, &voter, &vote, weight);
        Ok(())
    }

    /// Finalises a proposal after its voting period has ended.
    ///
    /// # Errors
    /// - [`ContractError::ProposalNotFound`] if `proposal_id` does not exist.
    /// - [`ContractError::ProposalNotActive`] if the proposal is not in `Active` status.
    /// - [`ContractError::VotingStillOpen`] if the voting window has not yet closed.
    pub fn finalise(env: Env, proposal_id: u64) -> Result<(), ContractError> {
        let mut proposal = load_proposal(&env, proposal_id)?;
        if proposal.status != ProposalState::Active {
            return Err(ContractError::ProposalNotActive);
        }
        if env.ledger().timestamp() <= proposal.end_time {
            return Err(ContractError::VotingStillOpen);
        }

        let total = proposal.votes_yes + proposal.votes_no + proposal.votes_abstain;
        proposal.status =
            if total >= proposal.quorum && proposal.votes_yes > proposal.votes_no {
                ProposalState::Passed
            } else {
                ProposalState::Rejected
            };

        save_proposal(&env, &proposal);
        events::proposal_finalised(&env, proposal_id, &proposal.status);
        Ok(())
    }

    /// Marks a passed proposal as executed. Only the admin may call this.
    ///
    /// # Errors
    /// - [`ContractError::NotAdmin`] if `admin` does not match the stored admin.
    /// - [`ContractError::ProposalNotFound`] if `proposal_id` does not exist.
    /// - [`ContractError::ProposalNotPassed`] if the proposal has not passed.
    pub fn execute(env: Env, admin: Address, proposal_id: u64) -> Result<(), ContractError> {
        admin.require_auth();
        if get_admin(&env)? != admin {
            return Err(ContractError::NotAdmin);
        }
        let mut proposal = load_proposal(&env, proposal_id)?;
        if proposal.status != ProposalState::Passed {
            return Err(ContractError::ProposalNotPassed);
        }
        proposal.status = ProposalState::Executed;
        save_proposal(&env, &proposal);
        events::proposal_finalised(&env, proposal_id, &ProposalState::Executed);
        Ok(())
    }

    /// Cancels an active proposal. Only the admin may cancel.
    ///
    /// # Errors
    /// - [`ContractError::NotAdmin`] if `admin` does not match the stored admin.
    /// - [`ContractError::ProposalNotFound`] if `proposal_id` does not exist.
    /// - [`ContractError::ProposalNotActive`] if the proposal is not in `Active` status.
    pub fn cancel(env: Env, admin: Address, proposal_id: u64) -> Result<(), ContractError> {
        admin.require_auth();
        if get_admin(&env)? != admin {
            return Err(ContractError::NotAdmin);
        }
        let mut proposal = load_proposal(&env, proposal_id)?;
        if proposal.status != ProposalState::Active {
            return Err(ContractError::ProposalNotActive);
        }
        proposal.status = ProposalState::Cancelled;
        save_proposal(&env, &proposal);
        events::proposal_finalised(&env, proposal_id, &ProposalState::Cancelled);
        Ok(())
    }

    /// Updates the quorum threshold of an active proposal. Only the admin may call this.
    ///
    /// # Errors
    /// - [`ContractError::NotAdmin`] if `admin` does not match the stored admin.
    /// - [`ContractError::InvalidQuorum`] if `new_quorum` is zero or negative.
    /// - [`ContractError::ProposalNotFound`] if `proposal_id` does not exist.
    /// - [`ContractError::ProposalNotActive`] if the proposal is not in `Active` status.
    pub fn update_quorum(
        env: Env,
        admin: Address,
        proposal_id: u64,
        new_quorum: i128,
    ) -> Result<(), ContractError> {
        admin.require_auth();
        if get_admin(&env)? != admin {
            return Err(ContractError::NotAdmin);
        }
        if new_quorum <= 0 {
            return Err(ContractError::InvalidQuorum);
        }
        let mut proposal = load_proposal(&env, proposal_id)?;
        if proposal.status != ProposalState::Active {
            return Err(ContractError::ProposalNotActive);
        }
        proposal.quorum = new_quorum;
        save_proposal(&env, &proposal);
        events::quorum_updated(&env, proposal_id, new_quorum);
        Ok(())
    }

    /// Returns the full state of a proposal.
    ///
    /// # Errors
    /// - [`ContractError::ProposalNotFound`] if `proposal_id` does not exist.
    pub fn get_proposal(env: Env, proposal_id: u64) -> Result<Proposal, ContractError> {
        load_proposal(&env, proposal_id)
    }

    /// Returns the total number of proposals ever created.
    pub fn proposal_count(env: Env) -> u64 {
        env.storage()
            .instance()
            .get(&DataKey::ProposalCount)
            .unwrap_or(0)
    }

    /// Returns whether an address has already voted on a given proposal.
    ///
    /// # Returns
    /// `true` if the address has cast a vote, `false` otherwise.
    ///
    /// # Errors
    /// - [`ContractError::ProposalNotFound`] if `proposal_id` does not exist.
    pub fn has_voted(
        env: Env,
        proposal_id: u64,
        voter: Address,
    ) -> Result<bool, ContractError> {
        load_proposal(&env, proposal_id)?;
        Ok(has_voted(&env, proposal_id, &voter))
    }

    /// Returns the contract version as a `(major, minor, patch)` semver tuple.
    pub fn get_version(env: Env) -> (u32, u32, u32) {
        get_version(&env)
    }
}
