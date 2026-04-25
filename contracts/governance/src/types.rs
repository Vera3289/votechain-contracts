use soroban_sdk::{contracterror, contracttype, Address, String};

/// All revert conditions for the governance contract.
#[contracterror]
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u32)]
pub enum ContractError {
    /// 1 – Admin address is not set
    AdminNotSet = 1,
    /// 2 – Caller is not the admin
    NotAdmin = 2,
    /// 3 – Voting token address is not set
    VotingTokenNotSet = 3,
    /// 4 – Quorum must be greater than zero
    InvalidQuorum = 4,
    /// 5 – Duration must be greater than zero
    InvalidDuration = 5,
    /// 6 – Proposal with the given ID does not exist
    ProposalNotFound = 6,
    /// 7 – Proposal is not in Active status
    ProposalNotActive = 7,
    /// 8 – Voting period has already ended
    VotingPeriodEnded = 8,
    /// 9 – Voting period has not ended yet
    VotingStillOpen = 9,
    /// 10 – Voter has already cast a vote on this proposal
    AlreadyVoted = 10,
    /// 11 – Voter has no token balance (no voting power)
    NoVotingPower = 11,
    /// 12 – Proposal has not passed
    ProposalNotPassed = 12,
    /// 13 – Contract has already been initialized
    AlreadyInitialized = 13,
    /// 14 – Proposer's token balance is below the required minimum
    InsufficientBalance = 14,
    /// 15 – Proposer must wait for the cooldown period to elapse
    ProposalCooldown = 15,
}

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
    HasVoted(u64, Address),      // (proposal_id, voter)
    Admin,
    VotingToken,
    MinProposalBalance,          // i128: minimum token balance to create a proposal
    ProposalCooldown,            // u64:  seconds between proposals per address
    LastProposal(Address),       // u64:  timestamp of proposer's last proposal
    Version,
}
