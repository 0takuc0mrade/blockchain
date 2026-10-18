#!/usr/bin/env bash
# Deploys the ticketing contract's wasm to the given network.
#
# Usage: scripts/deploy.sh <identity> <network> [--dry-run] [--yes]
#
# Safety (issue #228):
#   --dry-run  prints the unsigned deployment transaction instead of
#              submitting it; useful on any network, submits nothing.
#   mainnet    requires an explicit typed confirmation unless --yes is
#              passed for scripted deployments.
set -euo pipefail
cd "$(dirname "$0")/.."

# 1. Validate required arguments
if [ -z "${1:-}" ]; then
  echo "Error: Identity is required as the first argument." >&2
  echo "Usage: scripts/deploy.sh <identity> <network> [--dry-run] [--yes]" >&2
  echo "Example: scripts/deploy.sh default testnet" >&2
  exit 1
fi

if [ -z "${2:-}" ]; then
  echo "Error: Network is required as the second argument." >&2
  echo "Usage: scripts/deploy.sh <identity> <network> [--dry-run] [--yes]" >&2
  echo "Supported networks: testnet, futurenet, mainnet" >&2
  exit 1
fi

IDENTITY="$1"
NETWORK="$2"
shift 2

# 2. Validate network
case "$NETWORK" in
  testnet|futurenet|mainnet)
    ;;
  *)
    echo "Error: Invalid network '$NETWORK'. Supported networks are: testnet, futurenet, mainnet." >&2
    exit 1
    ;;
esac

# 3. Parse and validate options
DRY_RUN=false
ASSUME_YES=false
while [ "$#" -gt 0 ]; do
  case "$1" in
    --dry-run) DRY_RUN=true ;;
    --yes) ASSUME_YES=true ;;
    *)
      echo "Error: Unknown option '$1'." >&2
      echo "Usage: scripts/deploy.sh <identity> <network> [--dry-run] [--yes]" >&2
      exit 1
      ;;
  esac
  shift
done

# 4. Validate Stellar CLI availability
if ! command -v stellar >/dev/null 2>&1; then
  echo "Error: 'stellar' CLI is not installed or not available in PATH." >&2
  echo "Install it via: cargo install --locked stellar-cli --version ^27" >&2
  exit 1
fi

# 5. Validate identity in Stellar keys if named identity
if [[ ! "$IDENTITY" =~ ^S[A-Z0-9]{55}$ ]] && [[ ! "$IDENTITY" =~ ^G[A-Z0-9]{55}$ ]]; then
  if ! stellar keys address "$IDENTITY" >/dev/null 2>&1 && ! stellar keys show "$IDENTITY" >/dev/null 2>&1; then
    echo "Error: Identity '$IDENTITY' not found in Stellar keys." >&2
    echo "Generate or import it using: stellar keys generate $IDENTITY --network $NETWORK" >&2
    exit 1
  fi
fi

# 6. Validate WASM binary existence
WASM="target/wasm32v1-none/release/stellar_tickets_ticketing.optimized.wasm"
if [ ! -f "$WASM" ]; then
  WASM="target/wasm32v1-none/release/stellar_tickets_ticketing.wasm"
fi
if [ ! -f "$WASM" ]; then
  WASM="target/wasm32-unknown-unknown/release/stellar_tickets_ticketing.wasm"
fi

if [ ! -f "$WASM" ]; then
  echo "Error: Compiled contract WASM binary not found." >&2
  echo "Expected at: target/wasm32v1-none/release/stellar_tickets_ticketing.optimized.wasm or target/wasm32v1-none/release/stellar_tickets_ticketing.wasm" >&2
  echo "Build the contract first with: scripts/build.sh or stellar contract build" >&2
  exit 1
fi

echo "Deploying $WASM to $NETWORK using identity '$IDENTITY'..."

if [ "$DRY_RUN" = true ]; then
  echo "Dry run: printing the unsigned deployment transaction (nothing is submitted)." >&2
  stellar contract deploy \
    --wasm "$WASM" \
    --source "$IDENTITY" \
    --network "$NETWORK" \
    --build-only
  exit 0
fi

if [ "$NETWORK" = mainnet ] && [ "$ASSUME_YES" = false ]; then
  printf 'You are about to deploy to MAINNET. This cannot be undone.\nType "mainnet" to continue: '
  read -r ANSWER || ANSWER=""
  if [ "$ANSWER" != "mainnet" ]; then
    echo "Aborted: mainnet deployment requires typing the network name to confirm." >&2
    exit 1
  fi
fi

stellar contract deploy \
  --wasm "$WASM" \
  --source "$IDENTITY" \
  --network "$NETWORK"
