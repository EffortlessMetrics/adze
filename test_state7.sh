#!/bin/bash
cd /home/steven/code/rust-sitter/example
cargo build --features pure-rust 2>&1 > build_state7.log
grep -B5 -A5 "State 7" build_state7.log | head -50