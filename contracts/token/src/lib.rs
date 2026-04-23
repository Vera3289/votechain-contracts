#![no_std]

mod events;
mod storage;
mod types;

#[cfg(test)]
mod test;

use soroban_sdk::{contract, contractimpl, Address, Env};
use storage::*;
use types::ContractError;

#[contract]
pub struct TokenContract;

#[contractimpl]
impl TokenContract {
    pub fn initialize(env: Env, admin: Address, initial_supply: i128) {
        admin.require_auth();
        set_admin(&env, &admin);
        set_balance(&env, &admin, initial_supply);
        set_total_supply(&env, initial_supply);
    }

    pub fn total_supply(env: Env) -> i128 { total_supply(&env) }

    pub fn balance(env: Env, owner: Address) -> i128 { balance_of(&env, &owner) }

    pub fn transfer(env: Env, from: Address, to: Address, amount: i128) -> Result<(), ContractError> {
        from.require_auth();
        if amount <= 0 { return Err(ContractError::InvalidAmount); }
        let b = balance_of(&env, &from);
        if b < amount { return Err(ContractError::InsufficientBalance); }
        set_balance(&env, &from, b - amount);
        set_balance(&env, &to, balance_of(&env, &to) + amount);
        Ok(())
    }

    pub fn approve(env: Env, owner: Address, spender: Address, amount: i128) {
        owner.require_auth();
        set_allowance(&env, &owner, &spender, amount);
    }

    pub fn transfer_from(env: Env, spender: Address, from: Address, to: Address, amount: i128) -> Result<(), ContractError> {
        spender.require_auth();
        let allowed = allowance(&env, &from, &spender);
        if allowed < amount { return Err(ContractError::AllowanceExceeded); }
        let b = balance_of(&env, &from);
        if b < amount { return Err(ContractError::InsufficientBalance); }
        set_allowance(&env, &from, &spender, allowed - amount);
        set_balance(&env, &from, b - amount);
        set_balance(&env, &to, balance_of(&env, &to) + amount);
        Ok(())
    }

    pub fn mint(env: Env, admin: Address, to: Address, amount: i128) -> Result<(), ContractError> {
        admin.require_auth();
        if get_admin(&env)? != admin { return Err(ContractError::NotAdmin); }
        if amount <= 0 { return Err(ContractError::InvalidAmount); }
        set_balance(&env, &to, balance_of(&env, &to) + amount);
        set_total_supply(&env, total_supply(&env) + amount);
        Ok(())
    }

    pub fn burn(env: Env, admin: Address, from: Address, amount: i128) -> Result<(), ContractError> {
        admin.require_auth();
        if get_admin(&env)? != admin { return Err(ContractError::NotAdmin); }
        let b = balance_of(&env, &from);
        if b < amount { return Err(ContractError::InsufficientBalance); }
        set_balance(&env, &from, b - amount);
        set_total_supply(&env, total_supply(&env) - amount);
        Ok(())
    }
}
