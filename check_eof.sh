#!/bin/bash
cd /home/steven/code/rust-sitter/example
echo "Building..."
cargo build --features pure-rust 2>build_err.txt
echo "Checking build output for EOF handling..."
grep -i "EOF\|State.*special" build_err.txt | head -20
echo "Running test..."
cargo run --example test_arithmetic_pure --features pure-rust 2>&1 | grep -A50 "Parsing: '1 - 2'" | grep -E "State 7|EOF|Error" | head -20