# Phase 3.3 Component 2: GLR/LR Parity Testing

**Version**: 1.0
**Status**: Draft → Active
**Phase**: 3.3.2
**Dependencies**: Component 1 (Core Parsing Working)
**Timeline**: 1 day (7 hours)
**Date**: 2025-11-19

---

## Purpose

Validate that the GLR runtime produces **semantically identical** output to the LR runtime for unambiguous grammars. This ensures correctness and builds confidence that GLR can be used as a drop-in replacement for deterministic parsing.

## Problem Statement

**Current State**:
- GLR parsing engine operational (Finding 2 resolved)
- Arithmetic grammar parsing successfully (8/10 tests)
- No validation that GLR output matches expected LR behavior

**Risk**:
- Subtle differences in tree structure could break typed AST extraction
- Symbol ID mismatches could cause downstream failures
- Range calculations might differ between implementations

**Need**:
Systematic testing to prove **GLR ≡ LR** for unambiguous grammars.

---

## Scope

### In Scope

1. **Tree Structure Comparison**
   - Symbol IDs match
   - Byte ranges identical
   - Child counts equal
   - Tree depth same

2. **Grammars to Test**
   - Arithmetic (precedence + associativity)
   - Future: Repetitions, Optionals (if runtime implementations exist)

3. **Test Inputs**
   - Simple cases (single tokens, binary ops)
   - Complex cases (deeply nested, mixed operators)
   - Edge cases (empty, maximal)

### Out of Scope

- Performance comparison (Component 3)
- Memory comparison (Component 4)
- Ambiguous grammars (GLR-only, no LR equivalent)
- Error handling differences

---

## Design

### Testing Strategy

**Approach**: Parse same inputs with both runtimes, compare Trees structurally.

```
Input → GLR Runtime → Tree_GLR
     ↓
     → LR Runtime  → Tree_LR
     ↓
     Compare(Tree_GLR, Tree_LR) → PASS/FAIL
```

### Tree Comparison Algorithm

```rust
fn trees_equal(glr: &Tree, lr: &Tree) -> bool {
    nodes_equal(&glr.root_node(), &lr.root_node())
}

fn nodes_equal(glr_node: &Node, lr_node: &Node) -> bool {
    // 1. Symbol IDs match
    if glr_node.kind_id() != lr_node.kind_id() {
        return false;
    }

    // 2. Byte ranges match
    if glr_node.byte_range() != lr_node.byte_range() {
        return false;
    }

    // 3. Child counts match
    if glr_node.child_count() != lr_node.child_count() {
        return false;
    }

    // 4. All children match recursively
    for i in 0..glr_node.child_count() {
        let glr_child = glr_node.child(i).unwrap();
        let lr_child = lr_node.child(i).unwrap();
        if !nodes_equal(&glr_child, &lr_child) {
            return false;
        }
    }

    true
}
```

### Test File Structure

```
runtime2/tests/glr_lr_parity_test.rs
├── mod arithmetic_parity
│   ├── test_simple_number
│   ├── test_binary_ops
│   ├── test_precedence
│   ├── test_associativity
│   └── test_complex_expressions
├── mod repetitions_parity (future)
└── mod optionals_parity (future)
```

---

## Contract

### Preconditions

- GLR runtime functional (Component 1 complete)
- LR runtime exists and works (baseline)
- Grammar is unambiguous (no GLR-specific conflicts)
- Same grammar definition used for both runtimes

### Postconditions

**Success**: For all test inputs, `trees_equal(glr_tree, lr_tree) == true`

**Failure**: Any structural difference detected → investigate and fix

### Invariants

- Parsing same input always produces same tree (determinism)
- Tree structure independent of runtime choice (for unambiguous grammars)
- No memory safety violations during comparison

---

## Test Cases

### Arithmetic Grammar

#### TC-1: Simple Number

**Input**: `"42"`

**Expected**:
```
Tree {
  root: Node { symbol: NUMBER(1), range: 0..2, children: [] }
}
```

**Test**:
```rust
#[test]
fn test_arithmetic_parity_simple_number() {
    let input = "42";
    let glr_tree = parse_glr(input).expect("GLR parse failed");
    let lr_tree = parse_lr(input).expect("LR parse failed");

    assert!(trees_equal(&glr_tree, &lr_tree),
        "GLR and LR trees differ for input '{}'", input);
}
```

#### TC-2: Binary Subtraction

**Input**: `"1-2"`

**Expected**:
```
Tree {
  root: Node {
    symbol: expr(4),
    range: 0..3,
    children: [
      Node { symbol: NUMBER(1), range: 0..1 },
      Node { symbol: MINUS(2), range: 1..2 },
      Node { symbol: NUMBER(1), range: 2..3 },
    ]
  }
}
```

**Test**: Same pattern as TC-1

#### TC-3: Precedence

**Input**: `"1-2*3"`

**Expected**: Multiplication binds tighter, tree structure reflects "1-(2*3)"

**Test**: Verify child structure matches precedence

#### TC-4: Left Associativity

**Input**: `"1-2-3"`

**Expected**: Left-to-right grouping "(1-2)-3"

**Test**: Verify left subtree is binary op, right subtree is leaf

#### TC-5: Complex Expression

**Input**: `"1-2*3-4"`

**Expected**: Correct precedence and associativity

**Test**: Full tree structural comparison

### Edge Cases

#### TC-6: Empty Input

**Input**: `""`

**Expected**: Both fail with same error OR both produce empty tree

**Test**: Compare error messages or tree structures

#### TC-7: Whitespace Variations

**Input**: `"1 - 2"`, `"1-2"`, `"1  -  2"`

**Expected**: Same tree structure (ranges might differ if whitespace included)

**Test**: Compare after normalizing whitespace handling

#### TC-8: Large Expression

**Input**: `"1-2-3-4-5-6-7-8-9-10"` (deeply left-nested)

**Expected**: Deep left-associative tree, both runtimes agree

**Test**: Verify tree depth and structure

---

## Implementation Plan

### Phase 1: Test Harness (2 hours)

**File**: `runtime2/tests/glr_lr_parity_test.rs`

**Tasks**:
1. Create `trees_equal()` comparison function
2. Create helper `parse_glr()` (uses GLREngine path)
3. Create helper `parse_lr()` (uses standard LR path OR stubbed if not available)
4. Add debug output for tree structure diffs

**Acceptance**:
- Harness compiles
- Can parse with both runtimes
- trees_equal() detects differences

### Phase 2: Arithmetic Tests (3 hours)

**File**: Same as Phase 1

**Tasks**:
1. Implement TC-1 through TC-8
2. Run tests, validate all pass
3. If failures: investigate, document findings, fix if GLR bug

**Acceptance**:
- All 8 test cases implemented
- At least 6/8 passing (80% threshold)
- Any failures documented with root cause

### Phase 3: Documentation (1 hour)

**File**: `docs/status/PHASE_3.3_FINDINGS.md`

**Tasks**:
1. Add "Finding 4: GLR/LR Parity Results"
2. Document test results (pass/fail counts)
3. Document any differences found
4. Recommend fixes or acceptable variances

**Acceptance**:
- Finding 4 added to PHASE_3.3_FINDINGS.md
- Results table included
- Next steps clear

### Phase 4: Future Grammar Support (1 hour, optional)

**Files**: `runtime2/tests/glr_lr_parity_test.rs` (extend)

**Tasks**:
1. Add repetitions_parity module (if grammar available)
2. Add optionals_parity module (if grammar available)

**Acceptance**:
- Additional grammars tested if available
- Or: documented as future work if not available

---

## Success Criteria

### Minimum Viable (MVP)

- ✅ Test harness functional
- ✅ 6/8 arithmetic tests passing (75%)
- ✅ Failures documented with root cause
- ✅ trees_equal() detects differences correctly

### Full Success

- ✅ 8/8 arithmetic tests passing (100%)
- ✅ No structural differences found
- ✅ Documentation complete
- ✅ Findings added to PHASE_3.3_FINDINGS.md

### Stretch Goals

- ✅ Repetitions and Optionals tested (if grammars available)
- ✅ Performance comparison (preview of Component 3)
- ✅ Automated CI integration

---

## Risk Analysis

### High Risk

**Risk**: LR runtime not available for comparison
**Mitigation**: Use existing `runtime/` crate if compatible, or stub with expected trees
**Contingency**: Document GLR behavior as baseline, validate manually

### Medium Risk

**Risk**: Subtle tree differences (whitespace, EOF nodes)
**Mitigation**: Normalize comparison (ignore whitespace nodes, EOF handling)
**Contingency**: Document differences as acceptable if semantically equivalent

### Low Risk

**Risk**: Test harness bugs
**Mitigation**: Simple comparison logic, well-tested
**Contingency**: Add debug output, manual inspection

---

## Deliverables

1. **Test File**: `runtime2/tests/glr_lr_parity_test.rs`
   - trees_equal() function
   - 8 arithmetic parity tests
   - Optional: additional grammar tests

2. **Documentation**: `docs/status/PHASE_3.3_FINDINGS.md`
   - Finding 4: GLR/LR Parity Results
   - Test results table
   - Root cause analysis for any failures

3. **Status Update**: `docs/status/PHASE_3.3_STATUS.md`
   - Component 2 marked complete
   - Metrics updated
   - Next component identified

---

## Validation

### Unit Tests

Test the tree comparison logic itself:

```rust
#[test]
fn test_trees_equal_identical() {
    let tree1 = create_simple_tree();
    let tree2 = create_simple_tree();
    assert!(trees_equal(&tree1, &tree2));
}

#[test]
fn test_trees_equal_different_symbols() {
    let tree1 = create_tree_with_symbol(1);
    let tree2 = create_tree_with_symbol(2);
    assert!(!trees_equal(&tree1, &tree2));
}

#[test]
fn test_trees_equal_different_ranges() {
    let tree1 = create_tree_with_range(0..5);
    let tree2 = create_tree_with_range(0..6);
    assert!(!trees_equal(&tree1, &tree2));
}
```

### Integration Tests

8 test cases defined in Test Cases section above.

---

## Open Questions

1. **Q**: What if LR runtime doesn't exist for runtime2?
   **A**: Use expected tree structures as baseline, validate GLR manually

2. **Q**: Should we compare source text extraction?
   **A**: Phase 2: No. Phase 3: Yes (if needed for Extract trait)

3. **Q**: What about error messages?
   **A**: Out of scope for Component 2. Validate in Component 5 (E2E)

4. **Q**: Performance overhead of comparison?
   **A**: Negligible for test suite. Component 3 will measure parsing overhead.

---

## Timeline

| Phase | Duration | Deliverable |
|-------|----------|-------------|
| Phase 1: Test Harness | 2 hours | glr_lr_parity_test.rs with helpers |
| Phase 2: Arithmetic Tests | 3 hours | 8 test cases implemented + run |
| Phase 3: Documentation | 1 hour | Finding 4 in PHASE_3.3_FINDINGS.md |
| Phase 4: Future Grammars (optional) | 1 hour | Additional test modules |
| **Total** | **7 hours** | **Complete parity validation** |

---

## Dependencies

**Requires**:
- Component 1 complete (GLR parsing working)
- Node API functional (child access)
- Arithmetic grammar available

**Blocks**:
- Component 3 (Performance) - uses parity results
- Component 5 (E2E) - builds on parity confidence

**Optional**:
- LR runtime availability (can work around if missing)

---

## Approval

**Status**: Draft
**Reviewer**: TBD
**Approval Date**: TBD

---

**Next**: Implement test harness in `runtime2/tests/glr_lr_parity_test.rs`
**Success**: 8/8 tests passing, no structural differences found
