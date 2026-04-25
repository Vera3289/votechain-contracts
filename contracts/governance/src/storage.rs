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
