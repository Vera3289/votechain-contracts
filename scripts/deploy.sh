#!/usr/bin/env bash
# Deploy governance and token contracts to the selected NETWORK.
# Usage: NETWORK=testnet ./scripts/deploy.sh
# Contract IDs are written to .env.<NETWORK> after a successful deploy.
set -euo pipefail

NETWORK="${NETWORK:-local}"
CONFIG="config/${NETWORK}.toml"
ENV_FILE=".env.${NETWORK}"

if [[ ! -f "$CONFIG" ]]; then
  echo "Error: config file '$CONFIG' not found. Valid values: local, testnet, mainnet" >&2
  exit 1
fi

rpc_url=$(grep 'rpc_url' "$CONFIG" | sed 's/.*= *"\(.*\)"/\1/')
passphrase=$(grep 'network_passphrase' "$CONFIG" | sed 's/.*= *"\(.*\)"/\1/')

echo "Deploying to: $NETWORK  (RPC: $rpc_url)"

stellar contract build

_deploy() {
  local wasm="$1"
  stellar contract deploy \
    --wasm "$wasm" \
    --rpc-url "$rpc_url" \
    --network-passphrase "$passphrase"
}

TOKEN_ID=$(_deploy target/wasm32-unknown-unknown/release/votechain_token.wasm)
GOVERNANCE_ID=$(_deploy target/wasm32-unknown-unknown/release/votechain_governance.wasm)

# Write (or overwrite) the env file — idempotent
cat > "$ENV_FILE" <<EOF
NETWORK=${NETWORK}
TOKEN_CONTRACT_ID=${TOKEN_ID}
GOVERNANCE_CONTRACT_ID=${GOVERNANCE_ID}
EOF

echo "Contract IDs saved to $ENV_FILE"
echo "  TOKEN_CONTRACT_ID=${TOKEN_ID}"
echo "  GOVERNANCE_CONTRACT_ID=${GOVERNANCE_ID}"
