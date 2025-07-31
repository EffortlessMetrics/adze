#!/bin/bash
set -e
cd /home/steven/code/rust-sitter

echo "=== Building with pure-rust feature ==="
cargo build --features pure-rust

echo -e "\n=== Running arithmetic tests ==="
RUST_BACKTRACE=1 cargo test -p rust-sitter --features pure-rust -- arithmetic --nocapture