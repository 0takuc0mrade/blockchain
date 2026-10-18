#!/usr/bin/env bash
set -euo pipefail

echo "==> Checking build environment..."
which cargo
which stellar
rustup show

echo "==> Dev container ready!"