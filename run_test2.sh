#!/bin/bash
cd /home/steven/code/rust-sitter
RUST_BACKTRACE=1 cargo test -p rust-sitter --test arithmetic test_whitespace 2>&1 | tail -200