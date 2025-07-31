#!/bin/bash
cargo build --features pure-rust 2>&1 | grep -A200 "DEBUG: Arithmetic grammar JSON:" | head -100