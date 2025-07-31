#!/bin/bash
cargo test arithmetic::tests::test_pure_rust_parser --features pure-rust -- --nocapture 2>&1 | grep -A100 "DEBUG: Arithmetic" | head -150