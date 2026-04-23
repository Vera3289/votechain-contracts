#!/usr/bin/env bash
# Deploy contracts using config for the selected NETWORK.
# Usage: NETWORK=testnet ./scripts/deploy.sh
set -euo pipefail

NETWORK="${NETWORK:-local}"
CONFIG="config/${NETWORK}.toml"

if [[ ! -f "$CONFIG" ]]; then
  echo "Error: config file '$CONFIG' not found. Valid values: local, testnet, mainnet" >&2
  exit 1
fi

# Parse TOML values with grep/sed (no extra deps required)
rpc_url=$(grep 'rpc_url' "$CONFIG" | sed 's/.*= *"\(.*\)"/\1/')
passphrase=$(grep 'network_passphrase' "$CONFIG" | sed 's/.*= *"\(.*\)"/\1/')

echo "Deploying to: $NETWORK"
echo "RPC: $rpc_url"

stellar contract build

stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/votechain_token.wasm \
  --rpc-url "$rpc_url" \
  --network-passphrase "$passphrase"

stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/votechain_governance.wasm \
  --rpc-url "$rpc_url" \
  --network-passphrase "$passphrase"
