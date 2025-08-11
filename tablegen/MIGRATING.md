# Migration Guide for rust-sitter-tablegen

## Breaking Changes in Table Compression API

The `TableCompressor::compress()` method signature has changed to properly handle nullable start symbols and GLR multi-action cells.

### Before (Old API)
```rust
let compressor = TableCompressor::new();
let compressed = compressor.compress(&parse_table)?;
```

### After (New API)
```rust
use rust_sitter_tablegen::helpers::collect_token_indices;
use rust_sitter_ir::SymbolId;
use rust_sitter_glr_core::Action;

// Step 1: Collect token indices (includes EOF)
let token_indices = collect_token_indices(&grammar, &parse_table);

// Step 2: Determine if start symbol can be empty
// Check EOF cell in state 0 for Accept/Reduce actions
let eof_idx = *parse_table.symbol_to_index.get(&SymbolId(0)).unwrap();
let state0 = &parse_table.action_table[0];
let start_can_be_empty = state0[eof_idx].iter().any(|a| 
    matches!(a, Action::Accept | Action::Reduce(_))
);

// Step 3: Call compress with new parameters
let compressor = TableCompressor::new();
let compressed = compressor.compress(&parse_table, &token_indices, start_can_be_empty)?;
```

### Why This Change?

This change fixes the "State 0 bug" that prevented parsing files beginning with certain tokens in grammars with nullable start symbols (like Python's module rule that allows empty files). The GLR parser now correctly maintains multiple actions in state 0.

### Key Points

1. **`token_indices`**: A sorted list of column indices for all tokens, including EOF. Use the `collect_token_indices()` helper to generate this.

2. **`start_can_be_empty`**: Whether the start symbol can derive the empty string. This is typically determined by checking if the EOF cell in state 0 has Accept or Reduce actions.

3. **Helper Function**: The `collect_token_indices()` helper automatically includes EOF and handles all the column index mapping for you.

### Simplified Migration

If you don't need fine control over the parameters, you can use this pattern:

```rust
use rust_sitter_tablegen::TableGenerator;

let mut gen = TableGenerator::new(grammar, parse_table);
gen.compress_tables()?;  // Handles all parameters internally
```