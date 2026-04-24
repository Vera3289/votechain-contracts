# SEC-009 — Re-initialization Guard

**Component:** `contracts/governance/src/lib.rs` → `initialize`  
**Found:** 2026-04-24  
**Severity:** Critical  
**Status:** ✅ Fixed

---

## 1. Vulnerability

The original `initialize` function contained no guard against being called more than once:

```rust
// VULNERABLE — before fix
pub fn initialize(env: Env, admin: Address, voting_token: Address) {
    admin.require_auth();
    set_admin(&env, &admin);
    set_voting_token(&env, &voting_token);
}
```

Any caller who could satisfy `require_auth` (trivially bypassed in tests via `mock_all_auths`, and exploitable on-chain by the original admin or any address that can forge auth) could overwrite the stored `Admin` and `VotingToken` values after deployment. This would allow:

- Replacing the admin with an attacker-controlled address, granting full governance control.
- Replacing the voting token with a malicious contract that returns arbitrary balances, enabling vote manipulation.

---

## 2. Fix

Added an `is_initialized` check in `storage.rs` that reads the presence of the `Admin` key, and a guard at the top of `initialize` that returns `AlreadyInitialized` (error code 13) if the key already exists:

```rust
// storage.rs
pub fn is_initialized(env: &Env) -> bool {
    env.storage().instance().has(&DataKey::Admin)
}

// lib.rs — after fix
pub fn initialize(env: Env, admin: Address, voting_token: Address) -> Result<(), ContractError> {
    if is_initialized(&env) { return Err(ContractError::AlreadyInitialized); }
    admin.require_auth();
    set_admin(&env, &admin);
    set_voting_token(&env, &voting_token);
    Ok(())
}
```

The guard fires before `require_auth`, so no authentication is needed to trigger the revert — the call fails unconditionally regardless of caller identity.

---

## 3. Test Coverage

Three tests added in `contracts/governance/src/test.rs` under the `SEC-009` block:

| Test | Caller | Expected outcome |
|---|---|---|
| `test_reinit_by_original_admin_reverts` | Original admin | Panics (AlreadyInitialized) |
| `test_reinit_by_new_address_reverts` | Arbitrary new address | Panics (AlreadyInitialized) |
| `test_reinit_by_zero_address_reverts` | Zero address | Panics (AlreadyInitialized) |

---

## 4. Impact Assessment

| Property | Before fix | After fix |
|---|---|---|
| Admin overwrite possible | ✅ Yes | ❌ No |
| Token overwrite possible | ✅ Yes | ❌ No |
| Auth required to exploit | ✅ Yes (original admin) | N/A — blocked unconditionally |
| Error code | — | 13 (`AlreadyInitialized`) |
