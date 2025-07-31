#!/bin/bash
cd /home/steven/code/rust-sitter
cargo build --features pure-rust 2>&1 | grep -E "State (2|3|7)|special EOF|Updated non_error_actions" | head -20