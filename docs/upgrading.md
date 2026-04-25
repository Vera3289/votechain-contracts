# Contract Upgrade Guide

Soroban supports in-place contract upgrades via `stellar contract invoke … --fn upgrade_wasm`.
The contract address and all on-chain storage are preserved; only the executable WASM is replaced.

---

## Prerequisites

- Stellar CLI installed (`stellar --version`)
- Admin key with authority over the deployed contract
- New WASM built and tested locally (`make build && make test`)
- Contract address and network config for the target environment

---

## Step-by-Step Upgrade

### 1. Build the new WASM

```bash
make build
# Outputs:
#   target/wasm32-unknown-unknown/release/votechain_governance.wasm
#   target/wasm32-unknown-unknown/release/votechain_token.wasm
```

### 2. Record the current WASM hash (for rollback)

```bash
stellar contract info \
  --contract-id <CONTRACT_ID> \
  --rpc-url <RPC_URL> \
  --network-passphrase "<PASSPHRASE>"
```

Save the `wasm_hash` value — you will need it if you need to roll back.

### 3. Upload the new WASM to the network

```bash
stellar contract upload \
  --wasm target/wasm32-unknown-unknown/release/votechain_governance.wasm \
  --rpc-url <RPC_URL> \
  --network-passphrase "<PASSPHRASE>" \
  --source <ADMIN_KEY>
```

The command prints the new `WASM_HASH`. Record it.

### 4. Invoke the upgrade

```bash
stellar contract invoke \
  --id <CONTRACT_ID> \
  --rpc-url <RPC_URL> \
  --network-passphrase "<PASSPHRASE>" \
  --source <ADMIN_KEY> \
  -- upgrade_wasm \
  --new_wasm_hash <WASM_HASH>
```

> The `upgrade_wasm` host function is provided by the Soroban runtime and does not require
> a custom entry point in the contract source.

### 5. Verify the upgrade

```bash
# Confirm the on-chain WASM hash matches the new build
stellar contract info \
  --contract-id <CONTRACT_ID> \
  --rpc-url <RPC_URL> \
  --network-passphrase "<PASSPHRASE>"

# Confirm the version stored in contract state
stellar contract invoke \
  --id <CONTRACT_ID> \
  --rpc-url <RPC_URL> \
  --network-passphrase "<PASSPHRASE>" \
  -- get_version
```

### 6. Repeat for the token contract (if applicable)

Run steps 1–5 again using `votechain_token.wasm` and the token contract address.

---

## Using the Deploy Script

For testnet and mainnet, the deploy script reads config from `config/<NETWORK>.toml`:

```bash
# Testnet
NETWORK=testnet ./scripts/deploy.sh

# Mainnet — ensure config/mainnet.toml has the correct contract addresses
NETWORK=mainnet ./scripts/deploy.sh
```

The script builds and deploys fresh contract instances. Use the manual steps above
when upgrading an existing deployment in-place.

---

## Rollback Procedure

Soroban upgrades are reversible as long as the previous WASM hash is known.

### 1. Retrieve the previous WASM hash

If you recorded it in step 2 above, use that value directly.
Otherwise, check your deployment log, CI artefacts, or the release tag:

```bash
git show v<PREVIOUS_VERSION>:contracts/governance/Cargo.toml
# Then rebuild that version to obtain its hash:
git checkout v<PREVIOUS_VERSION>
make build
stellar contract upload \
  --wasm target/wasm32-unknown-unknown/release/votechain_governance.wasm \
  --rpc-url <RPC_URL> \
  --network-passphrase "<PASSPHRASE>" \
  --source <ADMIN_KEY>
```

### 2. Re-invoke upgrade with the old hash

```bash
stellar contract invoke \
  --id <CONTRACT_ID> \
  --rpc-url <RPC_URL> \
  --network-passphrase "<PASSPHRASE>" \
  --source <ADMIN_KEY> \
  -- upgrade_wasm \
  --new_wasm_hash <PREVIOUS_WASM_HASH>
```

### 3. Verify rollback

```bash
stellar contract invoke \
  --id <CONTRACT_ID> \
  --rpc-url <RPC_URL> \
  --network-passphrase "<PASSPHRASE>" \
  -- get_version
# Should return the previous (major, minor, patch) tuple
```

> **Storage compatibility**: rolling back the WASM does not revert on-chain storage.
> If the new version wrote data in a format the old version cannot read, the rollback
> may leave the contract in an inconsistent state. Always test storage migrations
> on testnet before upgrading mainnet.

---

## Storage Migration

When a new version changes the shape of stored data (e.g. adds a field to `Proposal`):

1. Add a migration function to the new contract (e.g. `migrate_v1_to_v2`).
2. Call it once immediately after the WASM upgrade, before any other transactions.
3. Remove the migration function in the subsequent release.

If no storage schema changes are made, no migration step is needed.

---

## Version Compatibility Matrix

The contract exposes its version via `get_version()` which returns a `(major, minor, patch)` tuple.
The version is set during `initialize` and should be updated in the source before each release.

| Contract Version | Soroban SDK | Storage Schema | Notes |
|-----------------|-------------|----------------|-------|
| 0.1.0 | 22.0.0 | v1 | Initial release. Governance + Token contracts. |
| 0.1.1 | 22.0.0 | v1 | Patch: bug fixes only. No migration required. |

### Compatibility rules

- **Same major version** — storage schema is stable; upgrades and rollbacks are safe without migration.
- **Minor version bump** — additive changes only (new functions, new optional fields). Rollback is safe; old clients may not see new fields.
- **Major version bump** — breaking changes to storage schema or function signatures. A migration function is required before rollback is safe.

---

## Checklist

Before upgrading a production contract:

- [ ] New WASM passes all tests (`make test`)
- [ ] Previous WASM hash recorded
- [ ] Upgrade tested on testnet with the same steps
- [ ] Storage migration function called if schema changed
- [ ] `get_version()` returns the expected new version after upgrade
- [ ] Off-chain indexers / clients updated if function signatures changed
