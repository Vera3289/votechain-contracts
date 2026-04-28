//! Storage accessors for the governance contract.
//!
//! # Namespacing strategy
//!
//! All storage entries are keyed by variants of [`DataKey`].  Soroban
//! serialises the enum variant discriminant into the XDR key before any
//! payload, so every variant occupies a completely isolated key space.
//! Adding a new data type requires only a new enum variant — there is no
//! risk of collision with existing keys.
//!
//! Storage tiers in use:
//! - **Instance** – singleton config values (`Admin`, `VotingToken`,
//!   `ProposalCount`, `MinProposalBalance`, `ProposalCooldown`, `Version`).
//!   Shares the contract instance TTL; cheap to access.
//! - **Persistent** – proposal data and per-voter records (`Proposal`,
//!   `HasVoted`, `VoteRecord`, `VoterSnapshot`, `LastProposal`).
//!   Survives ledger expiry; must be bumped explicitly for long-lived entries.
//!
//! ## Key-space isolation between related variants
//!
//! Several variants share the same payload shape `(u64, Address)` but are
//! distinct enum variants (`HasVoted`, `VoteRecord`, `VoterSnapshot`).
//! Because the discriminant is part of the serialised key, these can never
//! collide even when called with identical arguments.

use soroban_sdk::{Env, Address};
use crate::types::{ContractError, DataKey, Proposal, VoteRecord};

/// Persists a proposal to contract storage, keyed by its ID.
pub fn save_proposal(env: &Env, p: &Proposal) {
    env.storage().persistent().set(&DataKey::Proposal(p.id), p);
}

/// Loads a proposal from storage by ID.
///
/// # Errors
/// - [`ContractError::ProposalNotFound`] if no proposal exists for `id`.
pub fn load_proposal(env: &Env, id: u64) -> Result<Proposal, ContractError> {
    env.storage()
        .persistent()
        .get(&DataKey::Proposal(id))
        .ok_or(ContractError::ProposalNotFound)
}

/// Increments the proposal counter and returns the new ID.
pub fn next_id(env: &Env) -> u64 {
    let n: u64 = env.storage().instance().get(&DataKey::ProposalCount).unwrap_or(0) + 1;
    env.storage().instance().set(&DataKey::ProposalCount, &n);
    n
}

/// Stores the admin address in instance storage.
pub fn set_admin(env: &Env, admin: &Address) {
    env.storage().instance().set(&DataKey::Admin, admin);
}

/// Returns `true` if the contract has been initialised (admin key exists).
pub fn is_initialized(env: &Env) -> bool {
    env.storage().instance().has(&DataKey::Admin)
}

/// Returns the stored admin address.
///
/// # Errors
/// - [`ContractError::AdminNotSet`] if the contract has not been initialised.
pub fn get_admin(env: &Env) -> Result<Address, ContractError> {
    env.storage()
        .instance()
        .get(&DataKey::Admin)
        .ok_or(ContractError::AdminNotSet)
}

/// Stores the governance token address in instance storage.
pub fn set_voting_token(env: &Env, token: &Address) {
    env.storage().instance().set(&DataKey::VotingToken, token);
}

/// Returns the stored governance token address.
///
/// # Errors
/// - [`ContractError::VotingTokenNotSet`] if the contract has not been initialised.
pub fn get_voting_token(env: &Env) -> Result<Address, ContractError> {
    env.storage()
        .instance()
        .get(&DataKey::VotingToken)
        .ok_or(ContractError::VotingTokenNotSet)
}

/// Records that `voter` has voted on `proposal_id`.
pub fn mark_voted(env: &Env, proposal_id: u64, voter: &Address) {
    env.storage().persistent().set(&DataKey::HasVoted(proposal_id, voter.clone()), &true);
}

/// Returns `true` if `voter` has already voted on `proposal_id`.
pub fn has_voted(env: &Env, proposal_id: u64, voter: &Address) -> bool {
    env.storage()
        .persistent()
        .get(&DataKey::HasVoted(proposal_id, voter.clone()))
        .unwrap_or(false)
}

/// Stores the vote record for `voter` on `proposal_id`.
pub fn save_vote_record(env: &Env, proposal_id: u64, voter: &Address, record: &VoteRecord) {
    env.storage().persistent().set(&DataKey::VoteRecord(proposal_id, voter.clone()), record);
}

/// Returns the vote record for `voter` on `proposal_id`, or `None` if not voted.
pub fn get_vote_record(env: &Env, proposal_id: u64, voter: &Address) -> Option<VoteRecord> {
    env.storage().persistent().get(&DataKey::VoteRecord(proposal_id, voter.clone()))
}

pub fn set_min_proposal_balance(env: &Env, v: i128) {
    env.storage().instance().set(&DataKey::MinProposalBalance, &v);
}

pub fn get_min_proposal_balance(env: &Env) -> i128 {
    env.storage().instance().get(&DataKey::MinProposalBalance).unwrap_or(0)
}

pub fn set_proposal_cooldown(env: &Env, v: u64) {
    env.storage().instance().set(&DataKey::ProposalCooldown, &v);
}

pub fn get_proposal_cooldown(env: &Env) -> u64 {
    env.storage().instance().get(&DataKey::ProposalCooldown).unwrap_or(0)
}

pub fn set_last_proposal(env: &Env, proposer: &Address, ts: u64) {
    env.storage().persistent().set(&DataKey::LastProposal(proposer.clone()), &ts);
}

pub fn get_last_proposal(env: &Env, proposer: &Address) -> u64 {
    env.storage().persistent().get(&DataKey::LastProposal(proposer.clone())).unwrap_or(0)
}

/// Records the voter's token balance snapshot for a given proposal.
/// Called once per voter per proposal at the time of casting their vote.
pub fn save_voter_snapshot(env: &Env, proposal_id: u64, voter: &Address, weight: i128) {
    env.storage()
        .persistent()
        .set(&DataKey::VoterSnapshot(proposal_id, voter.clone()), &weight);
}

/// Returns the stored vote-weight snapshot for a voter on a proposal.
/// Returns `None` if no snapshot has been recorded yet.
pub fn get_voter_snapshot(env: &Env, proposal_id: u64, voter: &Address) -> Option<i128> {
    env.storage()
        .persistent()
        .get(&DataKey::VoterSnapshot(proposal_id, voter.clone()))
}
/// Stores the contract version as a `(major, minor, patch)` tuple.
pub fn set_version(env: &Env, version: (u32, u32, u32)) {
    env.storage().instance().set(&DataKey::Version, &version);
}

/// Returns the stored contract version as a `(major, minor, patch)` tuple.
pub fn get_version(env: &Env) -> (u32, u32, u32) {
    env.storage().instance().get(&DataKey::Version).unwrap_or((0, 0, 0))
}
