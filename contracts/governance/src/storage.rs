use soroban_sdk::{Env, Address};
use crate::types::{ContractError, DataKey, Proposal};

pub fn save_proposal(env: &Env, p: &Proposal) {
    env.storage().persistent().set(&DataKey::Proposal(p.id), p);
}

pub fn load_proposal(env: &Env, id: u64) -> Result<Proposal, ContractError> {
    env.storage()
        .persistent()
        .get(&DataKey::Proposal(id))
        .ok_or(ContractError::ProposalNotFound)
}

pub fn next_id(env: &Env) -> u64 {
    let n: u64 = env.storage().instance().get(&DataKey::ProposalCount).unwrap_or(0) + 1;
    env.storage().instance().set(&DataKey::ProposalCount, &n);
    n
}

pub fn set_admin(env: &Env, admin: &Address) {
    env.storage().instance().set(&DataKey::Admin, admin);
}

pub fn is_initialized(env: &Env) -> bool {
    env.storage().instance().has(&DataKey::Admin)
}

pub fn get_admin(env: &Env) -> Result<Address, ContractError> {
    env.storage()
        .instance()
        .get(&DataKey::Admin)
        .ok_or(ContractError::AdminNotSet)
}

pub fn set_voting_token(env: &Env, token: &Address) {
    env.storage().instance().set(&DataKey::VotingToken, token);
}

pub fn get_voting_token(env: &Env) -> Result<Address, ContractError> {
    env.storage()
        .instance()
        .get(&DataKey::VotingToken)
        .ok_or(ContractError::VotingTokenNotSet)
}

pub fn mark_voted(env: &Env, proposal_id: u64, voter: &Address) {
    env.storage().persistent().set(&DataKey::HasVoted(proposal_id, voter.clone()), &true);
}

pub fn has_voted(env: &Env, proposal_id: u64, voter: &Address) -> bool {
    env.storage()
        .persistent()
        .get(&DataKey::HasVoted(proposal_id, voter.clone()))
        .unwrap_or(false)
}

/// Stores the contract version as a `(major, minor, patch)` tuple.
pub fn set_version(env: &Env, version: (u32, u32, u32)) {
    env.storage().instance().set(&DataKey::Version, &version);
}

/// Returns the stored contract version as a `(major, minor, patch)` tuple.
pub fn get_version(env: &Env) -> (u32, u32, u32) {
    env.storage().instance().get(&DataKey::Version).unwrap_or((0, 0, 0))
}