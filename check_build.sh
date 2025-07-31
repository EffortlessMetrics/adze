#!/bin/bash
cargo build --features pure-rust 2>&1 | grep -E "Adding special reduce action for state" | head -10