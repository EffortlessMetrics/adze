#!/bin/bash
# Check for direct goto_indexing enum flips without proper remapping
# This prevents the class of bugs where GOTO tables get corrupted

set -e

# Check for ripgrep availability
command -v rg >/dev/null 2>&1 || {
  echo "ripgrep (rg) not found; skipping GOTO indexing check"
  exit 0
}

echo "Checking for direct goto_indexing assignments without remapping..."

# Check for DirectSymbolId assignment without remap (exclude the implementation in glr-core/src/lib.rs)
violations=$(rg -n 'goto_indexing\s*=\s*GotoIndexing::DirectSymbolId' --type rust | rg -v 'remap_goto_to_direct_symbol_id|glr-core/src/lib.rs' 2>/dev/null || true)
if [ -n "$violations" ]; then
    echo "❌ ERROR: Found direct goto_indexing = DirectSymbolId without using remap_goto_to_direct_symbol_id()"
    echo "Use table.remap_goto_to_direct_symbol_id() instead of directly setting the field"
    echo "$violations"
    exit 1
fi

# Check for NonterminalMap assignment without remap (exclude the implementation in glr-core/src/lib.rs)
violations=$(rg -n 'goto_indexing\s*=\s*GotoIndexing::NonterminalMap' --type rust | rg -v 'remap_goto_to_nonterminal_map|glr-core/src/lib.rs' 2>/dev/null || true)
if [ -n "$violations" ]; then
    echo "❌ ERROR: Found direct goto_indexing = NonterminalMap without using remap_goto_to_nonterminal_map()"
    echo "Use table.remap_goto_to_nonterminal_map() instead of directly setting the field"
    echo "$violations"
    exit 1
fi

# Check for SymbolId(0) usage in non-EOF contexts
echo "Checking for SymbolId(0) usage in grammar definitions..."
if rg -n 'SymbolId\(0\)' runtime/tests --type rust | rg -vq 'eof_symbol|normalize_eof_to_zero|table\.eof_symbol' 2>/dev/null; then
    echo "⚠️  WARNING: Found SymbolId(0) in tests (reserved for EOF)"
    echo "Consider starting symbol IDs at 1 to avoid EOF collision"
    rg -n 'SymbolId\(0\)' runtime/tests --type rust | rg -v 'eof_symbol|normalize_eof_to_zero|table\.eof_symbol' | head -5
fi

echo "✅ All goto_indexing checks passed!"