#![no_std]

mod events;
mod storage;
mod types;

#[cfg(test)]
mod test;
#[cfg(test)]
pub mod test_helpers;

use soroban_sdk::{contract, contractimpl, token, Address, Env, String};
use storage::{
    get_admin, get_voting_token, has_voted, is_initialized, load_proposal, mark_voted,
    next_id, save_proposal, set_admin, set_voting_token,
};
use types::{ContractError, DataKey, Proposal, ProposalStatus, Vote};

#[contract]
pub struct GovernanceContract;

#[contractimpl]
impl GovernanceContract {
    /// Initialises the governance contract with an admin and a voting token.
    ///
    /// Must be called exactly once before any other function.
    ///
    /// # Parameters
    /// - `env` – Soroban execution environment.
    /// - `admin` – Address that will have admin privileges (execute, cancel, update quorum).
    /// - `voting_token` – Address of the SEP-41 token used to determine vote weight.
    ///
    /// # Errors
    /// - [`ContractError::AlreadyInitialized`] if the contract has already been initialised.
    pub fn initialize(env: Env, admin: Address, voting_token: Address) -> Result<(), ContractError> {
        if is_initialized(&env) { return Err(ContractError::AlreadyInitialized); }
        admin.require_auth();
        set_admin(&env, &admin);
        set_voting_token(&env, &voting_token);
        Ok(())
    }

    /// Creates a new governance proposal.
    ///
    /// The proposal starts in [`ProposalStatus::Active`] and accepts votes until
    /// `start_time + duration` (ledger timestamp).
    ///
    /// # Parameters
    /// - `env` – Soroban execution environment.
    /// - `proposer` – Address creating the proposal; must authorise the call.
    /// - `title` – Short human-readable title.
    /// - `description` – Full proposal description.
    /// - `quorum` – Minimum total weighted votes required for the proposal to pass.
    /// - `duration` – Voting window in seconds (added to the current ledger timestamp).
    ///
    /// # Returns
    /// The numeric ID assigned to the new proposal.
    ///
    /// # Errors
    /// - [`ContractError::InvalidQuorum`] if `quorum` is zero or negative.
    /// - [`ContractError::InvalidDuration`] if `duration` is zero.
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

    /// Casts a vote on an active proposal.
    ///
    /// Vote weight equals the voter's current governance token balance.
    /// Each address may vote at most once per proposal.
    ///
    /// # Parameters
    /// - `env` – Soroban execution environment.
    /// - `voter` – Address casting the vote; must authorise the call.
    /// - `proposal_id` – ID of the proposal to vote on.
    /// - `vote` – [`Vote::Yes`], [`Vote::No`], or [`Vote::Abstain`].
    ///
    /// # Errors
    /// - [`ContractError::ProposalNotFound`] if `proposal_id` does not exist.
    /// - [`ContractError::ProposalNotActive`] if the proposal is not in `Active` status.
    /// - [`ContractError::VotingPeriodEnded`] if the voting window has closed.
    /// - [`ContractError::AlreadyVoted`] if the voter has already voted on this proposal.
    /// - [`ContractError::NoVotingPower`] if the voter's token balance is zero.
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

    /// Finalises a proposal after its voting period has ended.
    ///
    /// Sets the status to [`ProposalStatus::Passed`] when
    /// `total_votes >= quorum && votes_yes > votes_no`, otherwise
    /// [`ProposalStatus::Rejected`]. Can be called by anyone.
    ///
    /// # Parameters
    /// - `env` – Soroban execution environment.
    /// - `proposal_id` – ID of the proposal to finalise.
    ///
    /// # Errors
    /// - [`ContractError::ProposalNotFound`] if `proposal_id` does not exist.
    /// - [`ContractError::ProposalNotActive`] if the proposal is not in `Active` status.
    /// - [`ContractError::VotingStillOpen`] if the voting window has not yet closed.
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

    /// Marks a passed proposal as executed.
    ///
    /// Only the admin may call this function. Execution is a bookkeeping step;
    /// on-chain side-effects must be handled by the calling application.
    ///
    /// # Parameters
    /// - `env` – Soroban execution environment.
    /// - `admin` – Admin address; must authorise the call.
    /// - `proposal_id` – ID of the proposal to execute.
    ///
    /// # Errors
    /// - [`ContractError::NotAdmin`] if `admin` does not match the stored admin.
    /// - [`ContractError::ProposalNotFound`] if `proposal_id` does not exist.
    /// - [`ContractError::ProposalNotPassed`] if the proposal has not passed.
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

    /// Cancels an active proposal.
    ///
    /// Only the admin may cancel a proposal. Once cancelled the proposal
    /// cannot be re-opened.
    ///
    /// # Parameters
    /// - `env` – Soroban execution environment.
    /// - `admin` – Admin address; must authorise the call.
    /// - `proposal_id` – ID of the proposal to cancel.
    ///
    /// # Errors
    /// - [`ContractError::NotAdmin`] if `admin` does not match the stored admin.
    /// - [`ContractError::ProposalNotFound`] if `proposal_id` does not exist.
    /// - [`ContractError::ProposalNotActive`] if the proposal is not in `Active` status.
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

    /// Updates the quorum threshold of an active proposal.
    ///
    /// Only the admin may change the quorum. The proposal must still be active.
    ///
    /// # Parameters
    /// - `env` – Soroban execution environment.
    /// - `admin` – Admin address; must authorise the call.
    /// - `proposal_id` – ID of the proposal to update.
    /// - `new_quorum` – New minimum total weighted votes required to pass.
    ///
    /// # Errors
    /// - [`ContractError::NotAdmin`] if `admin` does not match the stored admin.
    /// - [`ContractError::InvalidQuorum`] if `new_quorum` is zero or negative.
    /// - [`ContractError::ProposalNotFound`] if `proposal_id` does not exist.
    /// - [`ContractError::ProposalNotActive`] if the proposal is not in `Active` status.
    pub fn update_quorum(env: Env, admin: Address, proposal_id: u64, new_quorum: i128) -> Result<(), ContractError> {
        admin.require_auth();
        if get_admin(&env)? != admin { return Err(ContractError::NotAdmin); }
        if new_quorum <= 0 { return Err(ContractError::InvalidQuorum); }
        let mut proposal = load_proposal(&env, proposal_id)?;
        if proposal.status != ProposalStatus::Active {
            return Err(ContractError::ProposalNotActive);
        }
        proposal.quorum = new_quorum;
        save_proposal(&env, &proposal);
        events::quorum_updated(&env, proposal_id, new_quorum);
        Ok(())
    }

    /// Returns the full state of a proposal.
    ///
    /// # Parameters
    /// - `env` – Soroban execution environment.
    /// - `proposal_id` – ID of the proposal to retrieve.
    ///
    /// # Returns
    /// A [`Proposal`] struct with all fields populated.
    ///
    /// # Errors
    /// - [`ContractError::ProposalNotFound`] if `proposal_id` does not exist.
    pub fn get_proposal(env: Env, proposal_id: u64) -> Result<Proposal, ContractError> {
        load_proposal(&env, proposal_id)
    }

    /// Returns the total number of proposals ever created.
    ///
    /// # Parameters
    /// - `env` – Soroban execution environment.
    ///
    /// # Returns
    /// Proposal count as `u64`. Returns `0` before any proposals are created.
    pub fn proposal_count(env: Env) -> u64 {
        env.storage().instance().get(&DataKey::ProposalCount).unwrap_or(0)
    }

    /// Returns whether an address has already voted on a given proposal.
    ///
    /// # Parameters
    /// - `env` – Soroban execution environment.
    /// - `proposal_id` – ID of the proposal to check.
    /// - `voter` – Address to check.
    ///
    /// # Returns
    /// `true` if the address has cast a vote, `false` otherwise.
    pub fn has_voted(env: Env, proposal_id: u64, voter: Address) -> bool {
        has_voted(&env, proposal_id, &voter)
    }
}
