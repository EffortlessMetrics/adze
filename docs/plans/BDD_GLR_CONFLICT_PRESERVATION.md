# BDD Specification: GLR Conflict Preservation

**Date**: 2025-11-19
**Status**: ACTIVE
**Related**: PARSER_V4_TABLE_LOADING_BLOCKER.md, GLR_CONFLICT_PRESERVATION_FIX.md
**Priority**: HIGH (Validates GLR runtime integration)

---

## 🎯 Overview

This document defines behavior-driven development (BDD) scenarios for validating GLR conflict preservation in parse table generation and runtime execution.

**Goal**: Ensure that glr-core preserves shift/reduce conflicts with proper precedence ordering, enabling true GLR parsing behavior.

---

## 📋 Feature: GLR Conflict Detection and Preservation

### Background
GLR parsers must detect conflicts during parse table generation and preserve both conflicting actions (ordered by precedence) instead of eliminating one. This enables the parser to explore multiple derivation paths at runtime.

---

## Scenario 1: Detect Shift/Reduce Conflicts in Ambiguous Grammars

**Given** a grammar with inherent shift/reduce ambiguity
**When** the LR(1) automaton is constructed
**Then** shift/reduce conflicts are detected in the parse table
**And** the conflicts are reported with state and symbol information

**Example: Dangling Else Problem**
```
Grammar:
  Statement → if Expr then Statement
  Statement → if Expr then Statement else Statement
  Statement → other

Input: "if a then if b then s1 else s2"

Conflict: In state after "if b then s1", on lookahead "else":
  - Shift: Continue with outer if (attach else to outer)
  - Reduce: Complete inner if (attach else to inner)
```

**Acceptance Criteria**:
- [ ] Conflict detected in correct state
- [ ] Both shift and reduce actions identified
- [ ] Conflict type correctly classified as ShiftReduce
- [ ] Symbol ID correctly captured

---

## Scenario 2: Preserve Conflicts with Precedence Ordering (PreferShift)

**Given** a shift/reduce conflict with precedence favoring shift
**When** `resolve_shift_reduce_conflict()` is called
**Then** both actions are preserved in order `[shift, reduce]`
**And** the first action (shift) has higher runtime priority

**Example:**
```rust
// Input conflict: [Shift(5), Reduce(3)]
// Precedence: shift_prec > reduce_prec

// Expected output:
conflict.actions == vec![Shift(5), Reduce(3)]
```

**Acceptance Criteria**:
- [ ] Both actions present in resulting conflict
- [ ] Shift action appears first in vector
- [ ] Reduce action appears second in vector
- [ ] No Fork action created

---

## Scenario 3: Preserve Conflicts with Precedence Ordering (PreferReduce)

**Given** a shift/reduce conflict with precedence favoring reduce
**When** `resolve_shift_reduce_conflict()` is called
**Then** both actions are preserved in order `[reduce, shift]`
**And** the first action (reduce) has higher runtime priority

**Example:**
```rust
// Input conflict: [Shift(5), Reduce(3)]
// Precedence: reduce_prec > shift_prec

// Expected output:
conflict.actions == vec![Reduce(3), Shift(5)]
```

**Acceptance Criteria**:
- [ ] Both actions present in resulting conflict
- [ ] Reduce action appears first in vector
- [ ] Shift action appears second in vector
- [ ] No Fork action created

---

## Scenario 4: Use Fork for No Precedence Information

**Given** a shift/reduce conflict with no precedence defined
**When** `resolve_shift_reduce_conflict()` is called
**Then** a Fork action is created with both actions
**And** both paths are explored at runtime with equal priority

**Example:**
```rust
// Input conflict: [Shift(5), Reduce(3)]
// Precedence: None

// Expected output:
conflict.actions == vec![Fork(vec![Shift(5), Reduce(3)])]
```

**Acceptance Criteria**:
- [ ] Single Fork action created
- [ ] Fork contains both shift and reduce
- [ ] No precedence ordering applied

---

## Scenario 5: Use Fork for Non-Associative Conflicts

**Given** a shift/reduce conflict with non-associative precedence
**When** `resolve_shift_reduce_conflict()` is called
**Then** a Fork action is created signaling an error
**And** the parser can report ambiguity at runtime

**Example:**
```rust
// Input conflict: [Shift(5), Reduce(3)]
// Precedence: Error (non-associative)

// Expected output:
conflict.actions == vec![Fork(vec![Shift(5), Reduce(3)])]
```

**Acceptance Criteria**:
- [ ] Single Fork action created
- [ ] Fork marked as error condition
- [ ] Runtime can detect and report ambiguity

---

## Scenario 6: Multi-Action Cells in Generated Parse Tables

**Given** a grammar with preserved conflicts
**When** the parse table is generated via tablegen
**Then** multi-action cells are created in the action table
**And** cells contain all preserved actions in correct order

**Example:**
```
State 7, Symbol "else":
  Actions: [Shift(8), Reduce(RuleId(2))]
```

**Acceptance Criteria**:
- [ ] Action table contains cells with >1 action
- [ ] Decoder correctly loads multi-action cells
- [ ] Action order matches conflict resolution order
- [ ] Diagnostic test detects multi-action cells

---

## Scenario 7: GLR Runtime Explores Both Paths

**Given** a parse table with multi-action cells
**When** the GLR runtime encounters a conflict during parsing
**Then** the parser forks into multiple derivation paths
**And** all valid parse trees are produced

**Example:**
```
Input: "if a then if b then s1 else s2"

Expected: Parser forks at "else", produces 2 parse trees:
  1. if a then (if b then s1 else s2)   [shift interpretation]
  2. if a then (if b then s1) else s2   [reduce interpretation]
```

**Acceptance Criteria**:
- [ ] Parser creates fork when encountering multi-action cell
- [ ] Both derivation paths explored
- [ ] Valid parse trees produced for all interpretations
- [ ] No parse errors for ambiguous input

---

## Scenario 8: Precedence Ordering Affects Tree Selection

**Given** a parse table with precedence-ordered conflicts
**When** multiple parse trees are produced
**Then** the tree from the first action (higher priority) is preferred
**And** alternative trees are available but deprioritized

**Example:**
```
Input: "if a then if b then s1 else s2"
Precedence: Shift > Reduce (prefer shift)

Primary tree: if a then (if b then s1 else s2)
Alternative:  if a then (if b then s1) else s2
```

**Acceptance Criteria**:
- [ ] Primary tree matches first action in conflict
- [ ] Alternative trees available on request
- [ ] API allows accessing all parse trees
- [ ] Default behavior returns highest-priority tree

---

## 🧪 Test Implementation Strategy

### Phase 1: Unit Tests (glr-core)
**Location**: `glr-core/tests/test_conflict_preservation.rs`

```rust
#[test]
fn test_detect_shift_reduce_conflict() {
    // Create dangling-else grammar
    // Build LR(1) automaton
    // Assert conflicts detected
}

#[test]
fn test_preserve_prefer_shift() {
    // Create conflict with shift precedence
    // Call resolve_shift_reduce_conflict
    // Assert actions == [shift, reduce]
}

#[test]
fn test_preserve_prefer_reduce() {
    // Create conflict with reduce precedence
    // Call resolve_shift_reduce_conflict
    // Assert actions == [reduce, shift]
}
```

### Phase 2: Integration Tests (tablegen)
**Location**: `tablegen/tests/test_multi_action_cells.rs`

```rust
#[test]
fn test_generate_multi_action_cells() {
    // Generate parse table for dangling-else grammar
    // Inspect SMALL_PARSE_TABLE
    // Assert multi-action cells present
}
```

### Phase 3: End-to-End Tests (runtime)
**Location**: `runtime/tests/test_glr_fork_merge.rs`

```rust
#[test]
fn test_glr_parses_ambiguous_input() {
    // Parse "if a then if b then s1 else s2"
    // Assert multiple parse trees produced
    // Verify both interpretations present
}
```

### Phase 4: BDD Scenario Tests
**Location**: `runtime/tests/bdd_glr_scenarios.rs`

```rust
mod scenario_1_detect_conflicts {
    #[test]
    fn given_ambiguous_grammar_when_automaton_built_then_conflicts_detected() {
        // Full BDD-style test
    }
}
```

---

## 📊 Test Grammar: Dangling Else

**Grammar Definition** (in rust-sitter syntax):
```rust
#[rust_sitter::grammar("dangling_else")]
pub mod dangling_else {
    #[rust_sitter::language]
    pub enum Statement {
        // If-then (no else)
        IfThen(
            #[rust_sitter::leaf(text = "if")] (),
            Box<Expr>,
            #[rust_sitter::leaf(text = "then")] (),
            Box<Statement>,
        ),

        // If-then-else (with else)
        IfThenElse(
            #[rust_sitter::leaf(text = "if")] (),
            Box<Expr>,
            #[rust_sitter::leaf(text = "then")] (),
            Box<Statement>,
            #[rust_sitter::leaf(text = "else")] (),
            Box<Statement>,
        ),

        // Simple statement
        Other(#[rust_sitter::leaf(text = "other")] ()),
    }

    #[rust_sitter::language]
    pub enum Expr {
        Var(#[rust_sitter::leaf(pattern = r"[a-z]+")] String),
    }
}
```

**Expected Conflict**:
```
State X, Symbol "else":
  Shift(Y)    # Continue outer if-then, attach else to outer
  Reduce(Z)   # Complete inner if-then, attach else to inner
```

---

## ✅ Success Criteria

The GLR conflict preservation feature is complete when:

1. **Detection**: All scenarios 1 pass (conflicts detected)
2. **Preservation**: Scenarios 2-5 pass (actions preserved correctly)
3. **Table Generation**: Scenario 6 passes (multi-action cells generated)
4. **Runtime**: Scenarios 7-8 pass (GLR fork/merge works)
5. **Documentation**: All test results documented
6. **CI Integration**: BDD tests run in CI pipeline

---

## 📅 Implementation Timeline

- [x] **Phase 0**: BDD specification created (this document)
- [ ] **Phase 1**: Dangling-else grammar implemented
- [ ] **Phase 2**: Unit tests for conflict preservation
- [ ] **Phase 3**: Integration tests for table generation
- [ ] **Phase 4**: End-to-end GLR runtime tests
- [ ] **Phase 5**: CI integration and documentation

**Estimated Effort**: 6-8 hours total

---

## 📚 References

- [GLR Parsing (Scott & Johnstone)](https://en.wikipedia.org/wiki/GLR_parser)
- [Dangling Else Problem](https://en.wikipedia.org/wiki/Dangling_else)
- [Tree-sitter GLR Implementation](https://tree-sitter.github.io/tree-sitter/)
- [Cucumber BDD Framework](https://cucumber.io/docs/gherkin/reference/)

---

**Next Action**: Implement dangling-else grammar and Phase 1 unit tests.
