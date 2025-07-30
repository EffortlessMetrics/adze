#!/bin/bash

# Build the example crate with RUST_SITTER_EMIT_ARTIFACTS to generate parser
cd /home/steven/code/rust-sitter
RUST_SITTER_EMIT_ARTIFACTS=true cargo build -p rust-sitter-example 2>&1

# Check the generated metadata
echo "=== Checking arithmetic parser metadata ==="
find target/debug/build -name "parser_arithmetic.rs" -type f | head -1 | xargs grep -A 1 "SYMBOL_METADATA"

echo ""
echo "=== Symbol names ==="
find target/debug/build -name "parser_arithmetic.rs" -type f | head -1 | xargs grep "SYMBOL_NAME_" | head -10

echo ""
echo "=== Checking metadata values ==="
# Extract metadata array and decode values
find target/debug/build -name "parser_arithmetic.rs" -type f | head -1 | xargs grep "SYMBOL_METADATA.*=" | sed 's/.*= & \[//' | sed 's/\] ;.*//' | tr ',' '\n' | while read -r byte; do
    # Remove 'u8' suffix and whitespace
    value=$(echo "$byte" | sed 's/u8//' | tr -d ' ')
    if [ ! -z "$value" ]; then
        # Convert to hex and check if hidden flag (0x4) is set
        hex_value=$(printf "0x%02x" "$value")
        if [ $(($value & 0x04)) -ne 0 ]; then
            echo "Byte: $hex_value (HIDDEN)"
        else
            echo "Byte: $hex_value"
        fi
    fi
done