# Specification: End-to-End Ambiguous Grammar GLR Validation

## Version

1.0.0 (2025-11-19)

## Status

Specification - Implementation Pending

## Purpose

Define the end-to-end validation contract for proving that enum variant inlining (ADR-0003) combined with GLR conflict preservation enables true ambiguous grammar parsing with the GLR runtime.

## Overview

This specification validates the complete pipeline:
1. Enum-based grammar extraction with variant inlining
2. GLR table generation with conflict preservation
3. Runtime GLR parsing with fork/merge behavior
4. AST extraction from ambiguous parse forests

## Terminology

- **Ambiguous Grammar**: Grammar with inherent shift/reduce or reduce/reduce conflicts
- **GLR Conflict**: Multi-action cell in parse table (multiple valid actions)
- **Parse Forest**: Multiple parse trees representing different derivations
- **Disambiguation**: Runtime selection from multiple valid parses

## Prerequisites

### ADR-0003 (Enum Variant Inlining)
- ✅ Implemented and tested
- ✅ Enum variants without precedence are inlined
- ✅ Direct productions generated (no intermediate symbols)
- See: `docs/adr/0003-enum-variant-inlining-for-glr.md`

### GLR Conflict Preservation Fix
- ✅ Implemented in `glr-core/src/lib.rs:2019-2077`
- ✅ Shift/reduce conflicts preserve both actions with priority
- ✅ No conflict elimination during table generation
- See: `docs/plans/PARSER_V4_TABLE_LOADING_BLOCKER.md#resolution`

### Test Grammar: ambiguous_expr.rs
```rust
#[rust_sitter::language]
enum Expr {
    Binary(Box<Expr>, String, Box<Expr>),  // NO precedence → creates conflict
    Number(i32),
}
```

Expected grammar structure (after inlining):
```
Expr → Expr OP Expr  (direct production)
Expr → NUMBER        (direct production)
```

## Test Scenarios

### Scenario 1: Conflict Generation Validation

**Given**: ambiguous_expr.rs grammar with inlined variants
**When**: Building LR(1) parse table
**Then**: Parse table contains multi-action cells (GLR conflicts)

**Contract Assertions:**
```rust
// Load generated parse table
let lang = grammar::language();
let parse_table = decoder::decode_parse_table(lang);

// Assertion 1: Multi-action cells exist
let conflict_count = count_multi_action_cells(&parse_table);
assert!(conflict_count > 0,
    "Contract violation: Ambiguous grammar must generate GLR conflicts");

// Assertion 2: Conflicts are in expected states
// When parsing "Expr OP Expr" with lookahead "OP":
//   - SHIFT: Continue parsing (Expr OP Expr) OP ...
//   - REDUCE: Complete binary expression
let has_binary_conflict = parse_table.action_table.iter().any(|state| {
    state.iter().any(|cell| cell.len() > 1 && contains_shift_reduce(cell))
});
assert!(has_binary_conflict,
    "Contract violation: Expected shift/reduce conflict for binary expression");
```

**Success Criteria:**
- ✅ At least 1 multi-action cell exists
- ✅ Conflicts match expected pattern (shift + reduce for binary operators)
- ✅ Both actions preserved (not eliminated)

---

### Scenario 2: GLR Parsing Behavior

**Given**: Ambiguous input "1 + 2 + 3"
**When**: Parsing with GLR runtime
**Then**: Parser creates fork points and maintains multiple derivations

**Contract Assertions:**
```rust
// Parse ambiguous input
let input = "1 + 2 + 3";
let result = grammar::parse(input);

// Assertion 1: Parse succeeds (no error)
assert!(result.is_ok(),
    "Contract violation: GLR should handle ambiguous input without error");

// Assertion 2: Result is deterministic (one AST selected)
let expr = result.unwrap();
assert!(matches!(expr, Expr::Binary(..)),
    "Contract violation: Should produce valid binary expression");

// Assertion 3: AST structure is valid (either left or right associative)
// Left-associative:  (1 + 2) + 3
// Right-associative: 1 + (2 + 3)
verify_valid_parse_tree(&expr);
```

**Success Criteria:**
- ✅ Parse succeeds without error
- ✅ Produces valid AST (single deterministic result)
- ✅ AST matches one of the valid derivations

---

### Scenario 3: Backward Compatibility

**Given**: Arithmetic grammar with precedence
**When**: Parsing with GLR runtime
**Then**: Behavior identical to non-GLR mode (precedence respected)

**Contract Assertions:**
```rust
// Test precedence grammar
let input = "1 - 2 * 3";
let result = arithmetic::grammar::parse(input).unwrap();

// Assertion: Multiplication binds tighter (due to prec_left(2) vs prec_left(1))
match result {
    Expression::Sub(left, _, right) => {
        assert_eq!(*left, Expression::Number(1));
        assert!(matches!(*right, Expression::Mul(_, _, _)));
    }
    _ => panic!("Expected Sub at top level"),
}
```

**Success Criteria:**
- ✅ Precedence-based grammars work correctly
- ✅ No regression from pre-GLR behavior
- ✅ Deterministic parse selection based on precedence

---

## Implementation Contract

### Test File Location

`runtime/tests/test_e2e_ambiguous_grammar_glr.rs`

### Test Structure

```rust
#[test]
fn test_ambiguous_grammar_conflict_generation() {
    // Load ambiguous_expr grammar parse table
    // Assert multi-action cells exist
    // Assert conflicts match expected pattern
}

#[test]
fn test_ambiguous_grammar_glr_parsing() {
    // Parse ambiguous input "1 + 2 + 3"
    // Assert no error
    // Assert valid AST produced
}

#[test]
fn test_glr_backward_compatibility() {
    // Parse arithmetic with precedence
    // Assert correct associativity
    // Assert no regression
}

#[test]
fn test_ambiguous_vs_arithmetic_comparison() {
    // Load both grammars
    // Ambiguous: should have conflicts
    // Arithmetic: should have zero conflicts
    // Demonstrate difference
}
```

### Dependencies

```toml
[dev-dependencies]
rust-sitter-example = { path = "../example", features = ["glr"] }
```

---

## Error Conditions

### Error 1: No Conflicts Generated

**Symptom**: `conflict_count == 0`

**Diagnosis**:
1. Check enum variant inlining is applied
2. Verify no precedence attributes on variants
3. Inspect generated grammar JSON

**Remediation**: Review ADR-0003 implementation

---

### Error 2: Parse Failure on Ambiguous Input

**Symptom**: `grammar::parse(input).is_err()`

**Diagnosis**:
1. Check GLR runtime is selected (feature flag)
2. Verify decoder loads parse table correctly
3. Check parser_v4 handles multi-action cells

**Remediation**: Review GLR conflict preservation fix

---

### Error 3: Wrong AST Structure

**Symptom**: Invalid or unexpected parse tree

**Diagnosis**:
1. Check disambiguation strategy
2. Verify action priority ordering
3. Inspect parse tree conversion

**Remediation**: Review priority semantics in conflict resolution

---

## Acceptance Criteria

- [ ] Test file created: `runtime/tests/test_e2e_ambiguous_grammar_glr.rs`
- [ ] Scenario 1 test passes: Conflict generation validated
- [ ] Scenario 2 test passes: GLR parsing produces valid AST
- [ ] Scenario 3 test passes: Backward compatibility maintained
- [ ] Comparison test passes: Ambiguous vs arithmetic differentiated
- [ ] All tests pass with `cargo test -p rust-sitter --features glr`
- [ ] Documentation updated with findings

---

## Success Metrics

**When all tests pass:**

✅ Proves: Enum variant inlining enables ambiguous grammar authoring
✅ Proves: GLR runtime handles multi-action cells correctly
✅ Proves: Complete pipeline works end-to-end
✅ Proves: Backward compatibility preserved

**Impact**: Production-ready GLR support for rust-sitter

---

## References

- **ADR**: `docs/adr/0003-enum-variant-inlining-for-glr.md`
- **Blocker Analysis**: `docs/plans/PARSER_V4_TABLE_LOADING_BLOCKER.md`
- **Contract**: `docs/specs/GRAMMAR_EXTRACTION_CONTRACT.md`
- **Test Grammar**: `example/src/ambiguous_expr.rs`
- **GLR Core**: `glr-core/src/lib.rs:2019-2077`

---

## Changelog

- **1.0.0** (2025-11-19): Initial specification
