#!/bin/bash
cd /home/steven/code/rust-sitter
cargo build --features pure-rust 2>&1 | grep -E "State 2|Adding special" | head -30