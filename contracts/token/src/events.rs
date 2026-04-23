use soroban_sdk::{symbol_short, Address, Env};

pub fn minted(env: &Env, to: &Address, amount: i128) {
    env.events().publish((symbol_short!("mint"), to.clone()), amount);
}

pub fn transferred(env: &Env, from: &Address, to: &Address, amount: i128) {
    env.events().publish((symbol_short!("transfer"), from.clone(), to.clone()), amount);
}

pub fn burned(env: &Env, from: &Address, amount: i128) {
    env.events().publish((symbol_short!("burn"), from.clone()), amount);
}
