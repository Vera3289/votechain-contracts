use soroban_sdk::{Env, Address, symbol_short};
use crate::types::{ProposalState, Vote};

/// Emits a `created` event when a new proposal is created.
///
/// # Parameters
/// - `env` – Soroban execution environment.
/// - `id` – ID of the newly created proposal.
/// - `proposer` – Address that created the proposal.
pub fn proposal_created(env: &Env, id: u64, proposer: &Address) {
    env.events().publish((symbol_short!("created"), id), proposer.clone());
}

/// Emits a `vote` event when a vote is cast.
///
/// # Parameters
/// - `env` – Soroban execution environment.
/// - `id` – ID of the proposal being voted on.
/// - `voter` – Address casting the vote.
/// - `vote` – The vote choice ([`Vote::Yes`], [`Vote::No`], or [`Vote::Abstain`]).
/// - `weight` – Token balance used as vote weight.
pub fn vote_cast(env: &Env, id: u64, voter: &Address, vote: &Vote, weight: i128) {
    env.events().publish((symbol_short!("vote"), id), (voter.clone(), vote.clone(), weight));
}

/// Emits a `final` event when a proposal is finalised, executed, or cancelled.
///
/// # Parameters
/// - `env` – Soroban execution environment.
/// - `id` – ID of the proposal.
/// - `status` – The new [`ProposalState`] after the state transition.
pub fn proposal_finalised(env: &Env, id: u64, status: &ProposalState) {
    env.events().publish((symbol_short!("final"), id), status.clone());
}

/// Emits a `cancelled` event when a proposal is cancelled by admin.
pub fn proposal_cancelled(env: &Env, id: u64) {
    env.events().publish((symbol_short!("cancelled"), id), ());
}

/// Emits a `qupdate` event when a proposal's quorum is updated.
///
/// # Parameters
/// - `env` – Soroban execution environment.
/// - `id` – ID of the proposal whose quorum was changed.
/// - `new_quorum` – The updated quorum value.
pub fn quorum_updated(env: &Env, id: u64, new_quorum: i128) {
    env.events().publish((symbol_short!("qupdate"), id), new_quorum);
}
