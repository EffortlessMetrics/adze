#!/bin/bash
set -e
cd /home/steven/code/rust-sitter

echo "=== Building with debug output ==="
cargo build --features pure-rust 2>&1 | grep -E "State (1|2|3|7)|special EOF|Updated non_error_actions" | head -20

echo -e "\n=== Running arithmetic test ==="
cd example && cargo run --example test_arithmetic_pure --features pure-rust 2>&1 | grep -B2 -A5 "1 - 2"