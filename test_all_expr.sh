#!/bin/bash
cd /home/steven/code/rust-sitter/example
cargo run --example test_arithmetic_pure --features pure-rust 2>&1 | grep -E "Parsing:|Success|Failed" | head -20