# Table Generation Validation Contract

**Status**: SPECIFICATION
**Date**: 2025-11-19
**Phase**: 2 - GLR Conflict Preservation Validation
**Related**: [CONFLICT_INSPECTION_API.md](./CONFLICT_INSPECTION_API.md), [AMBIGUOUS_GRAMMAR_TEST_SUITE.md](./AMBIGUOUS_GRAMMAR_TEST_SUITE.md)

---

## Overview

This specification defines the contract for validating that parse table generation correctly preserves GLR conflicts. It bridges the gap between grammar IR and conflict detection by specifying how to generate ParseTables from test grammars and validate their conflict properties.

### Purpose

- **Integrate**: Connect grammar IR → table generation → conflict inspection
- **Validate**: Ensure ambiguous grammars generate expected conflicts
- **Test**: Automated validation of GLR conflict preservation
- **Document**: Clear contract for table generation testing

### Scope

- **In Scope**: ParseTable generation from Grammar IR
- **In Scope**: Conflict validation against specifications
- **In Scope**: Integration test framework for ambiguous grammars
- **Out of Scope**: Grammar IR generation from Rust types (macro/tool responsibility)
- **Out of Scope**: Runtime parsing behavior (Phase 3+)

---

## Architecture

### Current Table Generation Pipeline

```
Grammar IR (adze-ir::Grammar)
    ↓
FirstFollowSets::compute_normalized()
    ↓
build_lr1_automaton(grammar, first_follow)
    ↓
ParseTable (with multi-action cells for conflicts)
    ↓
count_conflicts(table) ← NEW VALIDATION STEP
    ↓
ConflictSummary (validation results)
```

### Integration Points

**Module**: `glr-core/src/lib.rs`
- `pub fn build_lr1_automaton(grammar: &Grammar, first_follow: &FirstFollowSets) -> Result<ParseTable, GLRError>`

**Module**: `glr-core/src/conflict_inspection.rs`
- `pub fn count_conflicts(table: &ParseTable) -> ConflictSummary`

**Test Location**: `glr-core/tests/table_generation_validation.rs` (new)

---

## Contract Definition

### 1. Input Contract

#### 1.1 Grammar IR Requirements

Test grammars MUST be provided as `adze_ir::Grammar` instances with:

```rust
pub struct Grammar {
    pub name: String,
    pub rules: IndexMap<SymbolId, Vec<Rule>>,
    pub tokens: IndexMap<SymbolId, Token>,
    pub rule_names: IndexMap<SymbolId, String>,
    // ... other fields
}
```

**Validation**:
- Grammar MUST have a valid start symbol
- All symbol references MUST be resolvable
- No circular dependencies in rules

---

### 2. Output Contract

#### 2.1 Generated ParseTable

`build_lr1_automaton` MUST produce a ParseTable with:

```rust
pub struct ParseTable {
    pub action_table: Vec<Vec<ActionCell>>,  // ActionCell = Vec<Action>
    pub symbol_metadata: Vec<SymbolMetadata>,
    pub state_count: usize,
    // ... other fields
}
```

**Requirements**:
- `action_table[state][symbol]` contains Vec<Action> (multi-action cells)
- Conflicts are preserved as multiple actions in the same cell
- State count matches the number of LR(1) item sets
- Symbol metadata is populated for all symbols

**Invariants** (see [CONFLICT_INSPECTION_API.md](./CONFLICT_INSPECTION_API.md#parsetable-invariants-contract)):
1. `state_count == action_table.len()` (validated via debug assertions)
2. All symbol indices are valid in `index_to_symbol` mapping
3. Empty cells represent error states (not conflicts)
4. Multi-action cells (len > 1) represent GLR conflicts

---

#### 2.2 Conflict Validation Results

`count_conflicts` MUST produce a ConflictSummary with:

```rust
pub struct ConflictSummary {
    pub shift_reduce: usize,
    pub reduce_reduce: usize,
    pub states_with_conflicts: Vec<StateId>,
    pub conflict_details: Vec<ConflictDetail>,
}
```

**Validation**:
- Conflict counts MUST match test specifications
- Conflict details MUST identify correct states and symbols
- ConflictType classification MUST be accurate

---

### 3. Test Contract

#### 3.1 Test Helper: Grammar Builder

```rust
/// Create a Grammar IR from a minimal specification for testing
pub fn build_test_grammar(
    name: &str,
    rules: Vec<(&str, Vec<&str>)>,  // (lhs, rhs symbols)
    terminals: Vec<&str>,
) -> Grammar {
    let mut grammar = Grammar::new(name.to_string());

    // Register terminals
    for (idx, term) in terminals.iter().enumerate() {
        let symbol_id = SymbolId(idx as u16);
        grammar.tokens.insert(symbol_id, Token {
            name: term.to_string(),
            pattern: TokenPattern::String(term.to_string()),
        });
        grammar.rule_names.insert(symbol_id, term.to_string());
    }

    // Register non-terminals and rules
    let nt_offset = terminals.len() as u16;
    for (rule_idx, (lhs, rhs)) in rules.iter().enumerate() {
        let lhs_id = SymbolId(nt_offset + rule_idx as u16);
        grammar.rule_names.insert(lhs_id, lhs.to_string());

        // Convert RHS symbol names to IDs
        let rhs_symbols: Vec<Symbol> = rhs.iter()
            .map(|sym_name| {
                // Look up in terminals or non-terminals
                grammar.rule_names.iter()
                    .find(|(_, name)| name.as_str() == *sym_name)
                    .map(|(id, _)| {
                        if (*id.0 as usize) < terminals.len() {
                            Symbol::Terminal(*id)
                        } else {
                            Symbol::NonTerminal(*id)
                        }
                    })
                    .unwrap_or(Symbol::Epsilon)
            })
            .collect();

        let rule = Rule {
            lhs: lhs_id,
            rhs: rhs_symbols,
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(rule_idx as u16),
        };

        grammar.add_rule(rule);
    }

    grammar
}
```

---

#### 3.2 Test Helper: Table Generation + Validation

```rust
/// Generate parse table and validate conflict properties
pub fn generate_and_validate_table(
    grammar: &mut Grammar,
    expected_sr: usize,
    expected_rr: usize,
) -> Result<(ParseTable, ConflictSummary), GLRError> {
    // Step 1: Compute FIRST/FOLLOW sets
    let first_follow = FirstFollowSets::compute_normalized(grammar)?;

    // Step 2: Build LR(1) automaton
    let table = build_lr1_automaton(grammar, &first_follow)?;

    // Step 3: Inspect conflicts
    let summary = count_conflicts(&table);

    // Step 4: Validate against expectations
    assert_eq!(
        summary.shift_reduce, expected_sr,
        "Expected {} S/R conflicts, found {}",
        expected_sr, summary.shift_reduce
    );

    assert_eq!(
        summary.reduce_reduce, expected_rr,
        "Expected {} R/R conflicts, found {}",
        expected_rr, summary.reduce_reduce
    );

    Ok((table, summary))
}
```

---

### 4. Integration Test Contracts

#### 4.1 Test: Dangling Else (TG-001)

**Grammar Definition** (from specification):
```
Statement → if Expr then Statement        (rule 1)
Statement → if Expr then Statement else Statement  (rule 2)
Statement → other                         (rule 3)
Expr → id                                 (rule 4)

Terminals: if, then, else, other, id
```

**Expected Conflicts**:
- 1 shift/reduce conflict on "else" token
- State after "if Expr then Statement" with lookahead "else":
  - Shift: to handle "else Statement" continuation
  - Reduce: to complete inner if-then

**Test Implementation**:
```rust
#[test]
fn test_dangling_else_table_generation() {
    let mut grammar = build_test_grammar(
        "dangling_else",
        vec![
            ("Statement", vec!["if", "Expr", "then", "Statement"]),
            ("Statement", vec!["if", "Expr", "then", "Statement", "else", "Statement"]),
            ("Statement", vec!["other"]),
            ("Expr", vec!["id"]),
        ],
        vec!["if", "then", "else", "other", "id"],
    );

    let (table, summary) = generate_and_validate_table(&mut grammar, 1, 0)
        .expect("Table generation failed");

    // Additional validation: find the "else" conflict
    let else_symbol = grammar.find_symbol_by_name("else")
        .expect("'else' symbol should exist");

    let else_conflicts = find_conflicts_for_symbol(&table, else_symbol);
    assert_eq!(else_conflicts.len(), 1, "Should have exactly one conflict on 'else'");

    let conflict = &else_conflicts[0];
    assert_eq!(conflict.conflict_type, ConflictType::ShiftReduce);
    assert_eq!(conflict.actions.len(), 2);

    // Verify one Shift and one Reduce
    let has_shift = conflict.actions.iter().any(|a| matches!(a, Action::Shift(_)));
    let has_reduce = conflict.actions.iter().any(|a| matches!(a, Action::Reduce(_)));
    assert!(has_shift && has_reduce, "Should have both Shift and Reduce actions");
}
```

---

#### 4.2 Test: Precedence-Free Expression (TG-002)

**Grammar Definition**:
```
Expr → Expr Op Expr    (rule 1)
Expr → Number          (rule 2)
Op → +                 (rule 3)
Op → *                 (rule 4)

Terminals: +, *, Number
```

**Expected Conflicts**:
- >= 2 shift/reduce conflicts (one per operator type minimum)
- State after "Expr Op Expr" with lookahead operator:
  - Shift: continue parsing (right-associative)
  - Reduce: complete binary expression (left-associative)

**Test Implementation**:
```rust
#[test]
fn test_precedence_free_expr_table_generation() {
    let mut grammar = build_test_grammar(
        "precedence_free",
        vec![
            ("Expr", vec!["Expr", "Op", "Expr"]),
            ("Expr", vec!["Number"]),
            ("Op", vec!["+"]),
            ("Op", vec!["*"]),
        ],
        vec!["+", "*", "Number"],
    );

    let (table, summary) = generate_and_validate_table(&mut grammar, 2, 0)
        .expect("Table generation failed");

    // Should have at least 2 S/R conflicts (one per operator)
    assert!(
        summary.shift_reduce >= 2,
        "Expected at least 2 S/R conflicts, got {}",
        summary.shift_reduce
    );

    // All conflicts should be shift/reduce (no reduce/reduce)
    assert_eq!(summary.reduce_reduce, 0);

    // Verify conflicts occur on operator symbols
    for conflict in &summary.conflict_details {
        assert_eq!(conflict.conflict_type, ConflictType::ShiftReduce);
        // Conflict should be on + or * symbols
        let sym_name = &conflict.symbol_name;
        assert!(
            sym_name.contains('+') || sym_name.contains('*') || sym_name.contains("symbol"),
            "Conflict should be on operator symbol, got: {}",
            sym_name
        );
    }
}
```

---

## Implementation Checklist

### Phase 2.2.1: Test Infrastructure ✅
- [x] Conflict inspection API implemented
- [x] Integration test structure created
- [ ] Grammar builder helper function
- [ ] Table generation + validation helper

### Phase 2.2.2: Grammar Builder
- [ ] Implement `build_test_grammar()` helper
- [ ] Add symbol name resolution
- [ ] Add rule construction
- [ ] Unit test grammar builder

### Phase 2.2.3: Table Generation Tests
- [ ] Implement TG-001 (dangling_else) integration test
- [ ] Implement TG-002 (precedence_free) integration test
- [ ] Validate conflict counts match specifications
- [ ] Validate conflict locations and types

### Phase 2.2.4: Real Grammar Integration
- [ ] Load existing dangling_else.rs grammar IR (if available)
- [ ] Load existing ambiguous_expr.rs grammar IR (if available)
- [ ] Validate against expected conflicts
- [ ] Enable `#[ignore]` tests in example crate

---

## Success Criteria

**Acceptance Criteria**:
1. `build_test_grammar()` creates valid Grammar IR
2. `generate_and_validate_table()` produces ParseTable with conflicts
3. TG-001 test generates exactly 1 S/R conflict
4. TG-002 test generates >= 2 S/R conflicts
5. All conflict details are accurate (state, symbol, type)
6. Integration tests pass in CI

**Validation**:
```bash
cargo test -p adze-glr-core --test table_generation_validation
# All tests should pass
```

---

## Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Grammar builder creates invalid IR | Medium | High | Extensive validation in tests |
| Table generation fails on test grammars | Medium | High | Use minimal grammars first |
| Conflict counts don't match specs | Low | Medium | Verify with manual LR(1) construction |
| Symbol name resolution issues | Medium | Low | Explicit symbol ID mapping |

---

## Timeline

- **Specification**: 1 hour (this document) ✅
- **Grammar builder**: 1-2 hours
- **Integration tests**: 2-3 hours
- **Validation**: 1 hour

**Total**: 5-7 hours

---

## References

- [CONFLICT_INSPECTION_API.md](./CONFLICT_INSPECTION_API.md) - Conflict detection API
- [AMBIGUOUS_GRAMMAR_TEST_SUITE.md](./AMBIGUOUS_GRAMMAR_TEST_SUITE.md) - Test grammar specs
- [glr-core/src/lib.rs](../../glr-core/src/lib.rs) - Table generation implementation
- [PRODUCTION_READINESS_ROADMAP.md](../PRODUCTION_READINESS_ROADMAP.md) - Overall roadmap

---

**Status**: Ready for Implementation
**Next**: Implement grammar builder helper and integration tests

