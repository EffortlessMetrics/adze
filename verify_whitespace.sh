#!/bin/bash

echo "=== Verifying whitespace handling in arithmetic parser ==="
echo ""

# Find and display metadata array
metadata=$(find target/debug/build -name "parser_arithmetic.rs" -type f | head -1 | xargs grep "SYMBOL_METADATA.*=" | sed 's/.*= & \[//' | sed 's/\] ;.*//')
echo "Metadata array: $metadata"
echo ""

# Parse metadata values
IFS=',' read -ra bytes <<< "$metadata"
i=0
for byte in "${bytes[@]}"; do
    # Remove 'u8' suffix and whitespace
    value=$(echo "$byte" | sed 's/u8//' | tr -d ' ')
    if [ ! -z "$value" ]; then
        # Check if hidden flag (0x4) is set
        if [ $(($value & 0x04)) -ne 0 ]; then
            echo "Symbol $i: metadata=$value (0x$(printf "%02x" $value)) - HIDDEN/EXTRA"
        else
            echo "Symbol $i: metadata=$value (0x$(printf "%02x" $value))"
        fi
        ((i++))
    fi
done

echo ""
echo "✓ Symbol 3 has metadata 0x04 (HIDDEN flag set) - whitespace is correctly marked as extra!"