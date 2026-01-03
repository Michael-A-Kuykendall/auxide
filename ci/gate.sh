#!/bin/bash
set -e

echo "Running Auxide gate script..."

# Truthfulness gates (must be zero hits)
echo "Checking for forbidden words in tests/ and .docs/..."
if rg "TODO|placeholder|design goal|verified by code review|Perhaps" tests .docs; then
    echo "ERROR: Forbidden words found in tests/ or .docs/"
    exit 1
fi

# RT panic gates
echo "Checking for unwrap/expect/assert/panic in process_block..."
if sed -n '/fn process_block/,/}/p' src/rt.rs | rg "expect\\(|assert\\(|panic!"; then
    echo "ERROR: Forbidden panic paths in process_block"
    exit 1
fi

# Run tests
echo "Running cargo test..."
cargo test

# Run offline render example
echo "Running offline render example..."
cargo run --example offline_render

echo "All gates passed!"