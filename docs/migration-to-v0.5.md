# Migration Guide: adze v0.4 to v0.5

This guide covers the major architectural changes in adze v0.5 and how to update your code to work with the new APIs.

## Major Changes

### 1. Grammar Rules Storage Change

The most significant change is how grammar rules are stored internally.

**Before (v0.4):**
```rust
pub struct Grammar {
    pub rules: HashMap<RuleId, Rule>,
    // ...
}
```

**After (v0.5):**
```rust
pub struct Grammar {
    pub rules: BTreeMap<SymbolId, Vec<Rule>>,
    // ...
}
```

This change groups all rules for a given non-terminal symbol together, which improves:
- GLR parser performance 
- Cache locality when accessing rules
- Logical organization of the grammar

### 2. Rule Construction Pattern

When adding rules to a grammar, the API pattern has changed:

**Before (v0.4):**
```rust
grammar.rules.insert(
    rule_id,
    Rule {
        lhs: symbol_id,
        rhs: vec![Symbol::Terminal(token_id)],
        // ...
    }
);
```

**After (v0.5):**
```rust
grammar.rules
    .entry(symbol_id)
    .or_insert_with(Vec::new)
    .push(Rule {
        lhs: symbol_id,
        rhs: vec![Symbol::Terminal(token_id)],
        // ...
    });
```

### 3. Rule Access Patterns

Accessing rules for a specific symbol has changed:

**Before (v0.4):**
```rust
// Get all rules (inefficient for large grammars)
for (rule_id, rule) in &grammar.rules {
    if rule.lhs == target_symbol {
        // Process rule
    }
}
```

**After (v0.5):**
```rust
// Direct access to rules for a symbol (efficient)
if let Some(rules) = grammar.rules.get(&target_symbol) {
    for rule in rules {
        // Process rule
    }
}
```

### 4. Rule Iteration

The `all_rules()` method provides a flattened iterator:

```rust
// Iterate over all rules in the grammar
for rule in grammar.all_rules() {
    // Process rule
}
```

### 5. GLR Parser Enhancements

The v0.5 release includes a completely rewritten GLR parser with:

- **Improved Conflict Resolution**: New `RuntimeConflictResolver` trait for custom conflict resolution strategies
- **Vec Wrapper Support**: Built-in resolver for common repetition patterns
- **Better Error Recovery**: Enhanced error recovery with scope tracking
- **Performance Improvements**: Optimized fork/merge operations and better memory usage

Example of using the new conflict resolver:

```rust
use adze_glr_core::VecWrapperResolver;

let resolver = VecWrapperResolver::new(&grammar, &first_follow_sets);
let parser = Parser::with_resolver(Box::new(resolver));
```

### 6. Pure Rust Implementation

v0.5 introduces a pure Rust parser implementation alongside the C-based Tree-sitter backend:

```toml
# Use pure Rust implementation (WASM-compatible)
adze = { version = "0.5", features = ["pure-rust"] }

# Use standard C-based Tree-sitter (default)
adze = { version = "0.5" }
```

## Migration Steps

1. **Update Dependencies**: Change your `Cargo.toml` to use v0.5:
   ```toml
   adze = "0.5.0"
   ```

2. **Update Grammar Construction**: Replace all `HashMap` insertions with the new `BTreeMap` pattern using `entry().or_insert_with(Vec::new).push()`.

3. **Update Rule Access**: Replace rule iteration patterns with direct symbol lookups where possible.

4. **Test Thoroughly**: The parse behavior should remain the same, but the internal representation has changed significantly.

5. **Consider GLR Features**: If you have grammars with conflicts, consider implementing a custom `RuntimeConflictResolver` for better parse results.

## Common Issues and Solutions

### Issue: "cannot find method `insert` for `BTreeMap<SymbolId, Vec<Rule>>`"
**Solution**: Use the `entry().or_insert_with(Vec::new).push()` pattern instead of `insert()`.

### Issue: "cannot iterate over rules with RuleId keys"
**Solution**: Either use `grammar.all_rules()` for a flat iteration or iterate over `&grammar.rules` to get `(SymbolId, &Vec<Rule>)` pairs.

### Issue: Performance regression after migration
**Solution**: Ensure you're using direct symbol lookups (`grammar.rules.get(&symbol_id)`) instead of iterating over all rules when looking for rules for a specific symbol.

## New Features to Explore

- **GLR Parsing**: Handles ambiguous grammars with multiple parse trees
- **Conflict Resolution**: Implement custom strategies for handling parse conflicts
- **Pure Rust Mode**: Deploy to WASM and other targets without C dependencies
- **Enhanced Error Recovery**: Better handling of syntax errors with scope-aware recovery

For more details on these features, see the main documentation.