#!/usr/bin/env bash
set -euo pipefail

# Script to generate TypeScript bindings for the Stellar Tickets Ticketing contract
# using stellar contract bindings typescript

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
CONTRACT_DIR="$PROJECT_ROOT/contracts/ticketing"
BINDINGS_DIR="$PROJECT_ROOT/bindings/typescript"

echo "==> Generating TypeScript bindings for stellar-tickets-ticketing contract"
echo "Project root: $PROJECT_ROOT"
echo "Contract dir: $CONTRACT_DIR"
echo "Bindings output: $BINDINGS_DIR"

# Check if stellar CLI is available
if ! command -v stellar >/dev/null 2>&1; then
  echo "Error: 'stellar' CLI is not installed or not available in PATH." >&2
  echo "Install it via: cargo install --locked stellar-cli --version ^27" >&2
  exit 1
fi

# Check if wasm exists
WASM_PATH="$CONTRACT_DIR/target/wasm32v1-none/release/stellar_tickets_ticketing.wasm"
if [ ! -f "$WASM_PATH" ]; then
  WASM_PATH="$CONTRACT_DIR/target/wasm32-unknown-unknown/release/stellar_tickets_ticketing.wasm"
fi

if [ ! -f "$WASM_PATH" ]; then
  echo "Error: Contract WASM not found at $WASM_PATH" >&2
  echo "Run 'make build' in $CONTRACT_DIR first" >&2
  exit 1
fi

echo "==> Found WASM at: $WASM_PATH"

# Create output directory
mkdir -p "$BINDINGS_DIR"

# Generate TypeScript bindings
echo "==> Running stellar contract bindings typescript..."
cd "$CONTRACT_DIR"
stellar contract bindings typescript \
  --wasm "$WASM_PATH" \
  --output-dir "$BINDINGS_DIR" \
  --overwrite

# Verify output
if [ -d "$BINDINGS_DIR" ] && [ -f "$BINDINGS_DIR/package.json" ]; then
  echo "==> TypeScript bindings generated successfully!"
  echo "Output directory: $BINDINGS_DIR"
  echo ""
  echo "Contents:"
  ls -la "$BINDINGS_DIR"
  echo ""
  echo "To use in your backend project:"
  echo "  1. Copy the bindings directory to your project"
  echo "  2. Run: npm install (or yarn/pnpm install)"
  echo "  3. Import: import { Client } from './bindings/typescript'"
else
  echo "Error: Failed to generate bindings" >&2
  exit 1
fi