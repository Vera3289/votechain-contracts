use soroban_sdk::{contracttype, Address, String};

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub enum ProposalStatus {
    Active,
    Passed,
    Rejected,
    Executed,
    Cancelled,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub enum Vote {
    Yes,
    No,
    Abstain,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct Proposal {
    pub id: u64,
    pub proposer: Address,
    pub title: String,
    pub description: String,
    pub votes_yes: i128,
    pub votes_no: i128,
    pub votes_abstain: i128,
    pub quorum: i128,       // minimum total votes required to pass
    pub start_time: u64,
    pub end_time: u64,
    pub status: ProposalStatus,
}

#[contracttype]
pub enum DataKey {
    Proposal(u64),
    ProposalCount,
    HasVoted(u64, Address),  // (proposal_id, voter)
    Admin,
    VotingToken,
}
