# ADR-009: Symbol Registry Unification

## Status

Accepted

## Context

During pure-Rust GLR implementation, a critical bug emerged: **complete parser failure on all inputs**. The root cause was identified as "Symbol ID Assignment Chaos" - three separate systems assigning symbol IDs independently without coordination.

### The Problem

```
Grammar Extraction → Symbol IDs based on discovery order
GLR Table Generation → Creates new symbol-to-index mapping  
Lexer Generation → Uses yet another mapping
Parse Table Compression → Stores indices instead of IDs
```

**Symptoms**:
- Lexer emits symbol ID 2 for "number"
- Parser expects symbol ID 3 for "number"
- Parse failures on all inputs - no valid parse possible

**Impact**: Complete parser failure - no input could be parsed correctly.

### Related Issues

The symbol ID chaos also caused:
1. **Parse table generation instability**: State count varied between builds (9 → 14 → 16 states)
2. **State 0 with no transitions**: Incorrect start symbol selection due to symbol mapping issues
3. **Extras token mishandling**: Whitespace tokens not properly marked as hidden

### Alternatives Considered

1. **Coordinate at build time**: Pass symbol maps between phases
   - Pros: No central registry needed
   - Cons: Error-prone, hard to verify consistency

2. **Use string names everywhere**: No numeric IDs until final output
   - Pros: No coordination needed
   - Cons: Performance overhead, memory usage

3. **Unified symbol registry**: Single source of truth for all symbol IDs
   - Pros: Guaranteed consistency, deterministic builds
   - Cons: Centralized component, requires careful design

## Decision

We implemented a **unified symbol registry** as a single source of truth for symbol ID assignment across all pipeline stages.

### SymbolRegistry Design

```rust
// In ir/src/grammar.rs
pub struct SymbolRegistry {
    // Deterministic symbol ordering
    symbols: IndexMap<String, SymbolId>,
    // Reverse lookup
    ids: HashMap<SymbolId, String>,
    // Metadata tracking
    metadata: HashMap<SymbolId, SymbolMetadata>,
}

impl Grammar {
    pub fn build_registry(&mut self) -> SymbolRegistry {
        let mut registry = SymbolRegistry::new();
        
        // 1. EOF is always 0
        registry.register("end", SymbolId(0));
        
        // 2. Sort tokens deterministically
        let mut tokens: Vec<_> = self.tokens.keys().collect();
        tokens.sort_by_key(|k| {
            // Underscored tokens last
            (k.starts_with('_'), k.as_str())
        });
        
        // 3. Assign sequential IDs
        for token_name in tokens {
            registry.register(token_name, next_id());
        }
        
        // 4. Non-terminals follow tokens
        let mut rules: Vec<_> = self.rules.keys().collect();
        rules.sort();
        
        for rule_name in rules {
            registry.register(rule_name, next_id());
        }
        
        registry
    }
}
```

### Deterministic Ordering Rules

1. **EOF is always SymbolId(0)**: Reserved for end-of-input
2. **Terminals before non-terminals**: All tokens come first
3. **Alphabetic sorting within categories**: Stable cross-build IDs
4. **Underscored tokens last**: Internal tokens sorted to end

### Pipeline Integration

All stages use the registry:
- **Grammar extraction**: Registers symbols as discovered
- **GLR table generation**: Uses registry for all symbol lookups
- **Lexer generation**: Gets token IDs from registry
- **Parse table compression**: Validates IDs match registry

### Consistency Validation

```rust
fn validate_symbol_consistency(
    grammar: &Grammar,
    lexer: &GeneratedLexer,
    parse_table: &ParseTable,
    language: &TSLanguage
) -> Result<()> {
    let registry = grammar.get_registry();
    
    // 1. Verify lexer uses correct IDs
    for (token_name, token_id) in &lexer.token_map {
        let expected = registry.get_id(token_name)?;
        assert_eq!(*token_id, expected);
    }
    
    // 2. Verify parse table uses correct IDs
    for state in &parse_table.states {
        for (symbol, _) in &state.transitions {
            let name = registry.get_name(symbol)?;
            assert!(registry.is_valid(symbol));
        }
    }
    
    Ok(())
}
```

## Consequences

### Positive

- **Deterministic builds**: Same grammar always produces same symbol IDs
- **Debuggable**: Symbol names can be looked up from IDs
- **Consistency guaranteed**: Single source of truth prevents mismatches
- **Simpler pipeline**: All stages reference the same registry
- **Validation support**: Can verify consistency at any point
- **Fixed critical bug**: Parser now works correctly on all inputs

### Negative

- **Centralized component**: Registry is a dependency for all stages
- **Memory overhead**: Additional maps for bidirectional lookup
- **Build order dependency**: Registry must be built before other phases
- **Migration effort**: Existing code needed updates to use registry

### Neutral

- **IndexMap usage**: Preserves insertion order for determinism
- **Immutable after construction**: Registry is built once then read-only
- **Thread-safe read access**: Multiple stages can read concurrently

## Related

- Related ADRs: [ADR-001](001-pure-rust-glr-implementation.md)
- Reference: [docs/archive/implementation/PURE_RUST_IMPLEMENTATION_ROADMAP.md](../archive/implementation/PURE_RUST_IMPLEMENTATION_ROADMAP.md) - "Symbol ID Assignment Chaos" section
- Reference: [ir/src/grammar.rs](../../ir/src/grammar.rs) - SymbolRegistry implementation
