#!/bin/bash
# Test GLR parity with extracted Tree-sitter tables
set -euo pipefail

echo "=== GLR Parity Testing Script ==="
echo ""

# Check if ts-bridge is built
if ! cargo build -p ts-bridge 2>/dev/null; then
    echo "⚠️  ts-bridge failed to build (requires Tree-sitter libraries)"
fi

# Test directories
GRAMMARS_DIR="tools/ts-bridge/tests/grammars"
OUTPUT_DIR="target/glr-parity-test"
mkdir -p "$OUTPUT_DIR"

echo "📋 Looking for test grammars in $GRAMMARS_DIR..."

# List available grammars (once with-grammars feature works)
if [ -d "$GRAMMARS_DIR" ]; then
    for grammar_so in "$GRAMMARS_DIR"/*.so; do
        if [ -f "$grammar_so" ]; then
            grammar_name=$(basename "$grammar_so" .so | sed 's/libtree-sitter-//')
            echo "  Found: $grammar_name"
            
            # Extract tables (when ts-bridge links properly)
            # cargo run -p ts-bridge -- "$grammar_so" "$OUTPUT_DIR/$grammar_name.json" "tree_sitter_$grammar_name"
            
            # Run parity test
            # cargo test -p rust-sitter-glr-core --test parity -- --grammar "$OUTPUT_DIR/$grammar_name.json"
        fi
    done
else
    echo "⚠️  No grammars directory found. To test parity:"
    echo "   1. Build Tree-sitter grammars (e.g., tree-sitter-json)"
    echo "   2. Place .so files in $GRAMMARS_DIR"
    echo "   3. Re-run this script"
fi

echo ""
echo "📊 GLR Trace Testing (for debugging conflicts):"
echo "   cargo test -p rust-sitter-glr-core --features glr-trace -- --nocapture"
echo ""
echo "🔍 To examine specific state/symbol conflicts:"
echo "   1. Enable glr-trace feature in Cargo.toml"
echo "   2. Use glr_trace! macro in driver.rs"
echo "   3. Run tests with --nocapture to see output"