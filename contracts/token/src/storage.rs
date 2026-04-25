use soroban_sdk::{Env, Address};
use crate::types::{ContractError, TokenDataKey};

/// Returns the token balance of `owner`. Defaults to `0` if never set.
pub fn balance_of(env: &Env, owner: &Address) -> i128 {
    env.storage().persistent().get(&TokenDataKey::Balance(owner.clone())).unwrap_or(0)
}

/// Sets the token balance of `owner` to `amount`.
pub fn set_balance(env: &Env, owner: &Address, amount: i128) {
    env.storage().persistent().set(&TokenDataKey::Balance(owner.clone()), &amount);
}

/// Returns the spending allowance granted by `owner` to `spender`. Defaults to `0`.
pub fn allowance(env: &Env, owner: &Address, spender: &Address) -> i128 {
    env.storage().temporary().get(&TokenDataKey::Allowance(owner.clone(), spender.clone())).unwrap_or(0)
}

/// Sets the spending allowance granted by `owner` to `spender`.
pub fn set_allowance(env: &Env, owner: &Address, spender: &Address, amount: i128) {
    env.storage().temporary().set(&TokenDataKey::Allowance(owner.clone(), spender.clone()), &amount);
}

/// Returns the total token supply. Defaults to `0` before initialisation.
pub fn total_supply(env: &Env) -> i128 {
    env.storage().instance().get(&TokenDataKey::TotalSupply).unwrap_or(0)
}

/// Stores the total token supply.
pub fn set_total_supply(env: &Env, s: i128) {
    env.storage().instance().set(&TokenDataKey::TotalSupply, &s);
}

/// Returns the stored admin address.
///
/// # Errors
/// - [`ContractError::AdminNotSet`] if the contract has not been initialised.
pub fn get_admin(env: &Env) -> Result<Address, ContractError> {
    env.storage()
        .instance()
        .get(&TokenDataKey::Admin)
        .ok_or(ContractError::AdminNotSet)
}

/// Stores the admin address in instance storage.
pub fn set_admin(env: &Env, admin: &Address) {
    env.storage().instance().set(&TokenDataKey::Admin, admin);
}

/// Stores the contract version as a `(major, minor, patch)` tuple.
pub fn set_version(env: &Env, version: (u32, u32, u32)) {
    env.storage().instance().set(&TokenDataKey::Version, &version);
}

/// Returns the stored contract version as a `(major, minor, patch)` tuple.
pub fn get_version(env: &Env) -> (u32, u32, u32) {
    env.storage().instance().get(&TokenDataKey::Version).unwrap_or((0, 0, 0))
}