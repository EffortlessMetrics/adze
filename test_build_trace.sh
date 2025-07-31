#!/bin/bash
set -e
cd /home/steven/code/rust-sitter

echo "=== Building tablegen with trace ==="
RUST_LOG=debug cargo build -p rust-sitter-tablegen --features pure-rust 2>&1 | grep -E "State (1|2|3|7)|special EOF|Updated non_error_actions" | head -30

echo -e "\n=== Building example ==="
cd example
RUST_LOG=debug cargo build --features pure-rust 2>&1 | grep -E "State (1|2|3|7)|special EOF|Updated non_error_actions" | head -30