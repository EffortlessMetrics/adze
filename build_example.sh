#!/bin/bash
cd /home/steven/code/rust-sitter/example
cargo build --example test_arithmetic_pure --features pure-rust > build.log 2>&1
tail -5 build.log