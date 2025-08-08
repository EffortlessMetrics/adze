# rust-sitter Migration Guide: GLR Architecture Update

This guide helps you migrate existing rust-sitter code to the new GLR-enabled architecture.

## Overview

The rust-sitter codebase has undergone a major architectural upgrade to support GLR (Generalized LR) parsing, enabling support for ambiguous grammars and more complex language features.

### Key Changes

1. **Grammar rules storage**: Changed from single rule per ID to multiple rules per symbol
2. **Enhanced data structures**: Added new required fields to core structs
3. **Removed obsolete fields**: Cleaned up deprecated rule attributes
4. **Import path updates**: Module reorganization for better API clarity

## Migration Steps

### 1. Update Grammar Rule Insertion

**Old Pattern:**
```rust
let mut grammar = Grammar::new();
grammar.rules.insert(rule_id, Rule {
    lhs: symbol_id,
    rhs: vec![...],
    // ...
});
```

**New Pattern:**
```rust
let mut grammar = Grammar::new();
grammar.rules.entry(symbol_id).or_insert_with(Vec::new).push(Rule {
    lhs: symbol_id,
    rhs: vec![...],
    // ...
});
```

### 2. Update Grammar Struct Initialization

**Old:**
```rust
let grammar = Grammar {
    start_symbol: SymbolId(0),
    rules: BTreeMap::new(),
    // Missing fields will cause compilation errors
};
```

**New:**
```rust
let grammar = Grammar {
    start_symbol: SymbolId(0),
    rules: BTreeMap::new(),
    extras: vec![],              // New required field
    symbol_registry: HashMap::new(), // New required field
};
```

### 3. Update ParseTable Struct

**Old:**
```rust
let parse_table = ParseTable {
    states: vec![],
    symbols: vec![],
    state_count: 0,
    symbol_count: 0,
    // Missing symbol_to_index field
};
```

**New:**
```rust
let parse_table = ParseTable {
    states: vec![],
    symbols: vec![],
    state_count: 0,
    symbol_count: 0,
    symbol_to_index: BTreeMap::new(), // New required field
};
```

### 4. Update TSParseAction Struct

**Old:**
```rust
let action = TSParseAction {
    type_: 1,
    state_or_production: 0,
    // Missing dynamic_precedence field
};
```

**New:**
```rust
let action = TSParseAction {
    type_: 1,
    state_or_production: 0,
    dynamic_precedence: 0, // New required field
};
```

### 5. Remove Obsolete Rule Fields

**Old:**
```rust
let rule = Rule {
    lhs: symbol_id,
    rhs: vec![...],
    fields: IndexMap::new(), // Remove
    inline: false,           // Remove
    fragile: false,          // Remove
    visible: true,           // Remove
    // ...
};
```

**New:**
```rust
let rule = Rule {
    lhs: symbol_id,
    rhs: vec![...],
    fields: vec![], // Changed from IndexMap to Vec
    // Removed: inline, fragile, visible
    // ...
};
```

### 6. Update Imports

**Old:**
```rust
use rust_sitter::parser_v2::{StateId, Parser};
use indexmap::IndexMap;
```

**New:**
```rust
use rust_sitter::parser::{StateId, Parser}; // parser_v2 → parser
use rust_sitter_ir::{SymbolId, RuleId, ProductionId};
// Remove IndexMap imports if only used for fields
```

### 7. Update Rule Iteration Patterns

**Old:**
```rust
// Assuming single rule per ID
if let Some(rule) = grammar.rules.get(&rule_id) {
    // process rule
}
```

**New:**
```rust
// Multiple rules per symbol
if let Some(rules) = grammar.rules.get(&symbol_id) {
    for rule in rules {
        // process each rule
    }
}
```

### 8. Update GLR Parser Rule Lookups

**Old:**
```rust
// Finding rule by production ID
let rule = grammar.rules.values()
    .find(|r| r.production_id == production_id);
```

**New:**
```rust
// Iterate through all rules across all symbols
let rule = grammar.rules.values()
    .flat_map(|rules| rules.iter())
    .find(|r| r.production_id == production_id);
```

## Error Recovery API Changes

The error recovery API has been simplified:

**Removed methods:**
- `enable_error_nodes()`
- `set_max_recovery_attempts()`
- `add_deletable_token()` (use sync tokens instead)

**Updated usage:**
```rust
let error_recovery = ErrorRecoveryConfigBuilder::new()
    .add_sync_token(semicolon_token_id)
    .add_insertable_token(closing_paren_id)
    .enable_scope_recovery(true)
    .build();
```

## Testing Changes

### Snapshot Testing

No changes required for `insta` snapshot tests. Run `cargo insta review` after migration to update any changed outputs.

### Mock Data

Update test mocks to use `BTreeMap` instead of `HashMap` where appropriate:

```rust
// Old
symbol_to_index: HashMap::new(),

// New  
symbol_to_index: BTreeMap::new(),
```

## Build Configuration

No changes required to `build.rs` files. The `rust_sitter_tool::build_parsers()` function remains unchanged.

## Common Compilation Errors and Fixes

### Error: "no method named `insert` found for type `BTreeMap<SymbolId, Vec<Rule>>`"
**Fix:** Use `.entry(symbol_id).or_insert_with(Vec::new).push(rule)` pattern

### Error: "missing field `extras` in initializer of `Grammar`"
**Fix:** Add `extras: vec![]` to Grammar initialization

### Error: "missing field `symbol_to_index` in initializer of `ParseTable`"
**Fix:** Add `symbol_to_index: BTreeMap::new()` to ParseTable initialization

### Error: "missing field `dynamic_precedence` in initializer of `TSParseAction`"
**Fix:** Add `dynamic_precedence: 0` to TSParseAction initialization

### Error: "no field `inline` on type `Rule`"
**Fix:** Remove the `inline`, `fragile`, and `visible` fields from Rule structs

## Benefits of Migration

After migration, your rust-sitter project will support:

- **GLR parsing**: Handle ambiguous grammars with multiple parse paths
- **Dynamic precedence**: Runtime conflict resolution
- **Multiple rules per symbol**: More flexible grammar definitions
- **Enhanced error recovery**: Better parse error handling
- **Future compatibility**: Aligned with rust-sitter's roadmap

## Getting Help

If you encounter issues not covered in this guide:

1. Check the example grammars in `/example/src/` for reference implementations
2. Review the test files for usage patterns
3. Open an issue at https://github.com/your-org/rust-sitter/issues

## Version Compatibility

This migration is required for rust-sitter version 2.0.0 and later. Projects using earlier versions can continue to use the old API but will not receive new features or GLR support.