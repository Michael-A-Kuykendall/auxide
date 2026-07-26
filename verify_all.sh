#!/usr/bin/env bash
# Integration gate for the auxide stack.
#
# The repo is currently a SINGLE crate (`auxide`, Cargo.toml has no [workspace]).
# This script builds, tests, and lints that single crate (including tests and
# examples via --all-targets) so one command proves the whole stack is clean.
#
# CI / Linux (or git-bash / WSL on Windows). On native Windows use verify_all.ps1.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

echo "=== auxide (single crate) ==="
cargo build --all-targets
cargo test --all-targets
cargo clippy --all-targets -- -D warnings

echo "ALL CRATES GREEN"
