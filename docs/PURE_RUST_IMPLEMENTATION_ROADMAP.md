# Pure-Rust Parser Implementation Roadmap

## Executive Summary

The pure-Rust Tree-sitter implementation has made substantial progress but faces critical issues with symbol ID consistency, parse table generation, and extras handling. This document provides a technical analysis and phased resolution plan.

## 🔴 Critical Issues Analysis

### 1. Symbol ID Assignment Chaos

**Root Cause**: Three separate systems assign symbol IDs independently without coordination.

```
Grammar Extraction → Symbol IDs based on discovery order
GLR Table Generation → Creates new symbol-to-index mapping  
Lexer Generation → Uses yet another mapping
Parse Table Compression → Stores indices instead of IDs
```

**Symptoms**:
- Lexer emits symbol ID 2 for "number"
- Parser expects symbol ID 3 for "number"
- Parse failures on all inputs

**Impact**: Complete parser failure - no input can be parsed

### 2. Parse Table Generation Instability

**Issues**:
- State 0 sometimes has NO valid transitions
- State count varies between builds (9 → 14 → 16 states)
- Nondeterministic automaton generation

**Root Cause**: 
- Incorrect start symbol selection
- Inconsistent symbol ordering affects state generation
- Missing validation of generated states

### 3. Extras Token Mishandling

**Current State**:
```rust
extras: vec![SymbolId(5)]  // Points to non-terminal "Whitespace"
```

**Required**:
```rust
// Terminal token "_3" (whitespace) should be marked HIDDEN
symbol_metadata[2] = 0x04  // Currently 0x00
```

**Impact**: Parser doesn't skip whitespace, causing parse failures

## 🛠️ Technical Solution Architecture

### Phase 1: Unified Symbol Registry (Critical)

Create a single source of truth for symbol ID assignment:

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

### Phase 2: Parse Table Generation Fix

Fix the GLR automaton to use consistent symbols:

```rust
// In glr-core/src/builder.rs
impl LR1AutomatonBuilder {
    pub fn build_with_registry(
        &mut self, 
        grammar: &Grammar,
        registry: &SymbolRegistry
    ) -> ParseTable {
        // Use registry for ALL symbol lookups
        self.symbol_to_index = registry.to_index_map();
        
        // Fix start symbol detection
        let start = self.find_start_symbol(grammar)
            .expect("No valid start symbol");
        
        // Build with validation
        let table = self.build_canonical_collection(start);
        
        // Validate every state has transitions
        for (state_id, state) in &table.states {
            assert!(!state.transitions.is_empty() || 
                   state.is_accept_state,
                   "State {} has no valid transitions", state_id);
        }
        
        table
    }
}
```

### Phase 3: Recursive Extras Resolution

Properly identify all terminal tokens that should be hidden:

```rust
// In tablegen/src/abi_builder.rs
fn resolve_extra_tokens(&self, grammar: &Grammar) -> HashSet<SymbolId> {
    let mut terminals = HashSet::new();
    let mut visited = HashSet::new();
    let mut stack: Vec<SymbolId> = grammar.extras.clone();
    
    while let Some(symbol_id) = stack.pop() {
        if !visited.insert(symbol_id) {
            continue;
        }
        
        // Check if it's a terminal
        if grammar.tokens.contains_key(&symbol_id) {
            terminals.insert(symbol_id);
        }
        // If non-terminal, explore its productions
        else if let Some(rules) = grammar.rules.get(&symbol_id) {
            for rule in rules {
                for symbol in &rule.rhs {
                    match symbol {
                        Symbol::Terminal(tid) => {
                            terminals.insert(*tid);
                        }
                        Symbol::NonTerminal(ntid) => {
                            stack.push(*ntid);
                        }
                    }
                }
            }
        }
    }
    
    terminals
}
```

### Phase 4: Symbol Consistency Validation

Add comprehensive validation:

```rust
// In tool/src/lib.rs
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
            assert!(registry.contains_id(*symbol));
        }
    }
    
    // 3. Verify metadata consistency
    assert_eq!(
        language.symbol_count as usize,
        registry.len()
    );
    
    Ok(())
}
```

## 🎯 Implementation Strategy

### Immediate Actions (Week 1)
1. Create SymbolRegistry in ir crate
2. Update Grammar to maintain registry
3. Fix parse table compression (already started)

### Short Term (Week 2)
1. Update all components to use registry
2. Fix GLR start symbol detection
3. Implement extras resolution

### Medium Term (Week 3-4)
1. Add comprehensive validation
2. Create regression test suite
3. Document architecture

## 🧪 Test Cases for Validation

```rust
// Test 1: Symbol Consistency
#[test]
fn test_symbol_ids_consistent() {
    let grammar = build_test_grammar();
    let registry = grammar.build_registry();
    
    // Build multiple times - should be deterministic
    for _ in 0..10 {
        let new_registry = grammar.build_registry();
        assert_eq!(registry, new_registry);
    }
}

// Test 2: Extras Handling
#[test]
fn test_whitespace_is_hidden() {
    let language = generate_arithmetic_parser();
    let ws_symbol = find_symbol(&language, "_3");
    
    assert_eq!(
        language.symbol_metadata[ws_symbol],
        HIDDEN_FLAG
    );
}

// Test 3: Parse Table Validity
#[test]
fn test_all_states_have_transitions() {
    let table = generate_parse_table();
    
    for (id, state) in table.states() {
        assert!(
            !state.transitions.is_empty() || 
            state.has_reduce_action(),
            "State {} is invalid", id
        );
    }
}
```

## 📊 Success Metrics

1. **Deterministic Builds**: Same input → same symbol IDs across builds
2. **Parse Success**: Simple arithmetic expressions parse correctly
3. **Whitespace Handling**: Parser automatically skips extras
4. **State Validity**: No empty states in parse table
5. **Performance**: Parse time within 10% of C implementation

## 🚦 Risk Mitigation

1. **Backward Compatibility**: Keep existing APIs, add new internally
2. **Incremental Migration**: Fix one component at a time
3. **Extensive Testing**: Each phase includes regression tests
4. **Fallback Plan**: Can revert to C backend if needed

## 📅 Timeline

- **Week 1**: Symbol Registry + Basic Integration
- **Week 2**: Parse Table Fixes + Extras Handling  
- **Week 3**: Validation + Testing
- **Week 4**: Documentation + Performance Tuning

This roadmap provides a systematic approach to achieving a fully functional pure-Rust Tree-sitter implementation.