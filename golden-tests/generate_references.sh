#!/bin/bash
set -euo pipefail

# This script generates reference parse trees using the official Tree-sitter parsers
# Prerequisites: tree-sitter CLI and grammars must be installed

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TARGET_LANGUAGE="${1:-}"

# Function to generate S-expression and hash for a file
generate_reference() {
    local lang=$1
    local filename=$2
    local input_path="$SCRIPT_DIR/$lang/fixtures/$filename"
    local base_name="${filename%.*}"
    local expected_dir="$SCRIPT_DIR/$lang/expected"
    local sexp_path="$expected_dir/${base_name}.sexp"
    local hash_path="$expected_dir/${base_name}.sha256"

    mkdir -p "$expected_dir"
    
    echo "Generating reference for $lang/$filename..."
    
    # Generate S-expression using tree-sitter CLI
    tree-sitter parse "$input_path" --quiet > "$sexp_path" 2>/dev/null || {
        echo "Error: Failed to parse $input_path with tree-sitter"
        echo "Make sure tree-sitter CLI is installed and $lang grammar is available"
        exit 1
    }
    
    # Generate SHA256 hash of the S-expression
    sha256sum "$sexp_path" | cut -d' ' -f1 > "$hash_path"
    
    echo "  Generated: $sexp_path"
    echo "  Hash: $(cat "$hash_path")"
}

# Check if tree-sitter is available
if ! command -v tree-sitter &> /dev/null; then
    echo "Error: tree-sitter CLI not found. Please install it first:"
    echo "  npm install -g tree-sitter-cli"
    exit 1
fi

# Python test files
if [ -z "$TARGET_LANGUAGE" ] || [ "$TARGET_LANGUAGE" = "python" ]; then
    if [ -d "$SCRIPT_DIR/python/fixtures" ] && [ "$(ls -A "$SCRIPT_DIR/python/fixtures")" ]; then
        echo "=== Generating Python references ==="
        for file in "$SCRIPT_DIR/python/fixtures"/*.py; do
            if [ -f "$file" ]; then
                generate_reference "python" "$(basename "$file")"
            fi
        done
    else
        echo "No Python fixtures found in $SCRIPT_DIR/python/fixtures/"
    fi
fi

echo

# JavaScript test files
if [ -z "$TARGET_LANGUAGE" ] || [ "$TARGET_LANGUAGE" = "javascript" ]; then
    if [ -d "$SCRIPT_DIR/javascript/fixtures" ] && [ "$(ls -A "$SCRIPT_DIR/javascript/fixtures")" ]; then
        echo "=== Generating JavaScript references ==="
        for file in "$SCRIPT_DIR/javascript/fixtures"/*.js; do
            if [ -f "$file" ]; then
                generate_reference "javascript" "$(basename "$file")"
            fi
        done
    else
        echo "No JavaScript fixtures found in $SCRIPT_DIR/javascript/fixtures/"
    fi
fi


echo
echo "Reference generation complete!"
echo "You can now run the golden-master tests with: cargo test golden_master"
