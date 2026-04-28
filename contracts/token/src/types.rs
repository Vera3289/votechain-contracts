use soroban_sdk::{contracterror, contracttype, Address};

/// All revert conditions for the token contract.
#[contracterror]
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u32)]
pub enum ContractError {
    /// 1 – Admin address is not set
    AdminNotSet = 1,
    /// 2 – Caller is not the admin
    NotAdmin = 2,
    /// 3 – Transfer/mint/burn amount must be positive
    InvalidAmount = 3,
    /// 4 – Sender has insufficient balance
    InsufficientBalance = 4,
    /// 5 – Spender allowance is insufficient
    AllowanceExceeded = 5,
}

/// Storage key enum for the token contract.
///
/// Every storage entry is keyed by a variant of this enum.  Because Soroban
/// serialises the variant discriminant as part of the XDR key, each variant
/// occupies a completely separate key space — two variants with the same
/// payload can never collide.
///
/// ## Key-space map
///
/// | Variant                        | Storage tier | Description                              |
/// |-------------------------------|--------------|------------------------------------------|
/// | `Balance(Address)`            | Persistent   | Per-address token balance                |
/// | `Allowance(Address, Address)` | Temporary    | Spender allowance granted by owner       |
/// | `TotalSupply`                 | Instance     | Aggregate token supply                   |
/// | `Admin`                       | Instance     | Contract administrator address           |
/// | `Version`                     | Instance     | Semver tuple `(major, minor, patch)`     |
#[contracttype]
pub enum TokenDataKey {
    /// Per-address token balance (persistent storage).
    /// Key space: one entry per unique holder address.
    Balance(Address),

    /// Spender allowance granted by `owner` to `spender` (temporary storage).
    /// Key space: one entry per `(owner, spender)` pair; expires with the ledger.
    Allowance(Address, Address),

    /// Aggregate token supply across all holders (instance storage).
    /// Key space: singleton — only one `TotalSupply` entry exists.
    TotalSupply,

    /// Contract administrator address (instance storage).
    /// Key space: singleton — only one `Admin` entry exists.
    Admin,

    /// Contract version stored as a `(major, minor, patch)` semver tuple (instance storage).
    /// Key space: singleton — only one `Version` entry exists.
    Version,
}
