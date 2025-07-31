#!/bin/bash
cargo build --features pure-rust 2>&1 | grep -E "DEBUG:|Production|symbol|state=|reduces to" | head -200