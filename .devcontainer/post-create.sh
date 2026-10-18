#!/usr/bin/env bash
set -euo pipefail

echo "==> Installing wasm32v1-none target..."
rustup target add wasm32v1-none

echo "==> Installing stellar-cli..."
cargo install --locked stellar-cli --version ^27

echo "==> Verifying installation..."
stellar --version
rustup target list --installed | grep wasm32v1-none

echo "==> Post-create setup complete!"