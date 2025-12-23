# GLR Core Parity Testing Status

## ✅ Completed
1. **Built tree-sitter-json library** - Successfully compiled with ts-bridge
2. **Extracted parse tables** - Generated `/tmp/json-grammar.json` with:
   - 25 symbols
   - 32 states
   - Proper action/goto tables
3. **Created parity test infrastructure** - `test_json_parity.rs` loads and uses extracted tables

## 🔴 Issue Found: GLR Driver Crash

The GLR driver crashes when parsing JSON with extracted Tree-sitter tables.

### Problem
```
thread 'test_json_simple_object' panicked at glr-core/src/driver.rs:160:45:
index out of bounds: the len is 0 but the index is 0
```

The issue occurs at the start of parsing - `state.stacks` becomes empty after the first token.

### Root Cause Analysis
The GLR driver expects the action table to use the exact same format as our internal GLR table generator, but Tree-sitter's extracted tables have a different structure:
- Tree-sitter uses compact, sparse action tables
- Our GLR driver expects dense action tables indexed directly by symbol ID
- The symbol-to-index mapping is incomplete

### Next Steps to Fix
1. **Update ParseTable construction** to properly map Tree-sitter's sparse format to our dense format
2. **Add proper terminal/nonterminal distinction** - Tree-sitter separates these, our driver needs correct indexing
3. **Debug action table population** - Ensure all valid actions from Tree-sitter are properly loaded

## Summary
The GLR engine itself is solid (all unit tests pass), but we need to fix the bridge between Tree-sitter's table format and our GLR driver's expectations. This is a data transformation issue, not a fundamental GLR algorithm problem.