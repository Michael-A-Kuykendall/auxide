#!/usr/bin/env bash
# Integration gate for the auxide stack.
#
# Runs build + test + clippy (warning-clean, including tests/examples via
# --all-targets) across all four crates, which live as sibling directories.
# Exits non-zero on the first crate that fails, so a single command proves
# the whole stack builds, lints, and tests together.
#
# CI / Linux (or git-bash / WSL on Windows). On native Windows use
# verify_all.ps1.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
CRATES="auxide auxide-dsp auxide-io auxide-midi"

for c in $CRATES; do
  DIR="$ROOT/$c"
  echo "=== $c ==="
  (
    cd "$DIR"
    cargo build
    cargo test
    cargo clippy --all-targets -- -D warnings
  ) || exit 1
done

echo "ALL CRATES GREEN"
