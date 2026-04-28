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
    /// 14 – Vote tally arithmetic overflow
    VoteTallyOverflow = 14,
    /// 15 – Proposer has insufficient token balance to create a proposal
    InsufficientBalance = 15,
    /// 16 – Proposer must wait for the cooldown period to expire
    ProposalCooldown = 16,
    /// 17 – Proposal title exceeds maximum byte length
    TitleTooLong = 17,
    /// 18 – Proposal description exceeds maximum byte length
    DescriptionTooLong = 18,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub enum ProposalState {
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
    pub state: ProposalState,
}

/// Storage key enum for the governance contract.
///
/// Every storage entry is keyed by a variant of this enum.  Because Soroban
/// serialises the variant discriminant as part of the XDR key, each variant
/// occupies a completely separate key space — two variants with the same
/// payload can never collide.
///
/// ## Key-space map
///
/// | Variant                          | Storage tier | Description                                        |
/// |----------------------------------|--------------|---------------------------------------------------|
/// | `Proposal(u64)`                  | Persistent   | Full proposal struct, keyed by proposal ID         |
/// | `ProposalCount`                  | Instance     | Monotonic counter used to assign proposal IDs      |
/// | `HasVoted(u64, Address)`         | Persistent   | Boolean flag: has this voter voted on this proposal|
/// | `VoteRecord(u64, Address)`       | Persistent   | Detailed vote record (type + weight) per voter     |
/// | `VoterSnapshot(u64, Address)`    | Persistent   | Token-balance snapshot captured at vote time       |
/// | `LastProposal(Address)`          | Persistent   | Timestamp of a proposer's most recent proposal     |
/// | `Admin`                          | Instance     | Contract administrator address                     |
/// | `VotingToken`                    | Instance     | Governance token contract address                  |
/// | `MinProposalBalance`             | Instance     | Minimum token balance required to create a proposal|
/// | `ProposalCooldown`               | Instance     | Seconds a proposer must wait between proposals     |
/// | `Version`                        | Instance     | Semver tuple `(major, minor, patch)`               |
#[contracttype]
pub enum DataKey {
    /// Full [`Proposal`] struct, keyed by proposal ID (persistent storage).
    /// Key space: one entry per unique `u64` proposal ID.
    Proposal(u64),

    /// Monotonic counter used to derive the next proposal ID (instance storage).
    /// Key space: singleton — only one `ProposalCount` entry exists.
    ProposalCount,

    /// Boolean flag recording whether `voter` has voted on `proposal_id` (persistent storage).
    /// Key space: one entry per `(proposal_id, voter)` pair.
    /// Kept separate from `VoteRecord` so existence checks are cheap.
    HasVoted(u64, Address),

    /// Detailed vote record (vote type + weight) for `voter` on `proposal_id` (persistent storage).
    /// Key space: one entry per `(proposal_id, voter)` pair.
    VoteRecord(u64, Address),

    /// Contract administrator address (instance storage).
    /// Key space: singleton — only one `Admin` entry exists.
    Admin,

    /// Address of the governance token contract (instance storage).
    /// Key space: singleton — only one `VotingToken` entry exists.
    VotingToken,

    /// Minimum token balance a proposer must hold to create a proposal (instance storage).
    /// Key space: singleton — only one `MinProposalBalance` entry exists.
    MinProposalBalance,

    /// Minimum seconds a proposer must wait between consecutive proposals (instance storage).
    /// Key space: singleton — only one `ProposalCooldown` entry exists.
    ProposalCooldown,

    /// Timestamp (Unix seconds) of `proposer`'s most recent proposal (persistent storage).
    /// Key space: one entry per unique proposer address.
    LastProposal(Address),

    /// Contract version stored as a `(major, minor, patch)` semver tuple (instance storage).
    /// Key space: singleton — only one `Version` entry exists.
    Version,

    /// Token-balance snapshot for `voter` on `proposal_id`, captured at vote time (persistent storage).
    /// Key space: one entry per `(proposal_id, voter)` pair.
    /// Kept separate from `VoteRecord` to allow independent querying of vote weight.
    VoterSnapshot(u64, Address),
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct VoteRecord {
    pub vote_type: Vote,
    pub weight: i128,
}
