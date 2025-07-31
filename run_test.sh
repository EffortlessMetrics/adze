#!/bin/bash
cd /home/steven/code/rust-sitter
cargo test -p example arithmetic::test_whitespace 2>&1 | tail -100