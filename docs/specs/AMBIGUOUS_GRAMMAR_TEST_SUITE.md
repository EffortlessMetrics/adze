# Ambiguous Grammar Test Suite Specification

**Status**: SPECIFICATION
**Date**: 2025-11-19
**Phase**: 2 - GLR Conflict Preservation Validation
**Related**: [PRODUCTION_READINESS_ROADMAP.md](../PRODUCTION_READINESS_ROADMAP.md)

---

## Overview

This specification defines a comprehensive test suite of intentionally ambiguous grammars that validate GLR conflict preservation, fork/merge behavior, and parse forest generation.

### Purpose

- **Validate**: GLR parser correctly handles ambiguous grammars
- **Prove**: Conflict preservation works end-to-end (grammar → table → runtime)
- **Test**: Parse forest generation and ambiguity resolution
- **Benchmark**: GLR performance characteristics

### Scope

- **In Scope**: Grammar definitions, expected conflicts, validation criteria
- **In Scope**: BDD scenarios for each grammar
- **In Scope**: Conflict inspection and forest validation
- **Out of Scope**: Production grammar optimization
- **Out of Scope**: Disambiguation heuristics (future work)

---

## Test Grammar Catalog

### TG-001: Dangling Else

**Classic Ambiguity**: If-then-else nesting

#### Grammar Definition

```rust
#[adze::grammar("dangling_else")]
mod dangling_else {
    #[adze::language]
    pub enum Stmt {
        If(Box<Expr>, Box<Stmt>),
        IfElse(Box<Expr>, Box<Stmt>, Box<Stmt>),
        Expr(Box<Expr>),
    }

    pub enum Expr {
        Id(#[leaf(pattern = r"[a-z]+")] String),
    }
}
```

#### Expected Conflicts

| State | Symbol | Actions | Type | Precedence |
|-------|--------|---------|------|------------|
| 5 | `else` | [Shift(6), Reduce(If)] | S/R | None |

**Conflict Count**: 1 shift/reduce

#### Ambiguous Input

```
if a if b c else d
```

**Valid Parse Trees**: 2

1. `if a (if b c else d)` - else binds to inner if
2. `if a (if b c) else d` - else binds to outer if

#### Expected GLR Behavior

- **Fork**: Parser creates 2 branches at "else" token
- **Merge**: Both branches complete successfully
- **Forest**: Contains 2 complete parse trees
- **Default**: Nearest-if binding (shift preferred)

#### BDD Scenario

```gherkin
Feature: Dangling Else Ambiguity

  Scenario: Nested if-then-else with ambiguous else binding
    Given a grammar with dangling else ambiguity
    When I parse "if a if b c else d"
    Then the parse should succeed
    And the parse forest should contain 2 trees
    And tree 1 should have structure "If(a, IfElse(b, c, d))"
    And tree 2 should have structure "IfElse(a, If(b, c), d)"
    And the default parse should prefer tree 1 (shift preferred)
```

#### Validation Criteria

- [ ] Grammar generates exactly 1 shift/reduce conflict
- [ ] GLR parser forks on "else" token
- [ ] Both parse trees are structurally valid
- [ ] Error count is 0
- [ ] Parse forest serialization works

---

### TG-002: Precedence-Free Expression Grammar

**Classic Ambiguity**: Arithmetic without precedence annotations

#### Grammar Definition

```rust
#[adze::grammar("precedence_free")]
mod precedence_free {
    #[adze::language]
    pub enum Expr {
        // NO precedence annotations - intentionally ambiguous
        Binary(Box<Expr>, Op, Box<Expr>),
        Number(#[leaf(pattern = r"\d+")] i32),
    }

    pub enum Op {
        Add(#[leaf(text = "+")] ()),
        Mul(#[leaf(text = "*")] ()),
    }
}
```

#### Expected Conflicts

| State | Symbol | Actions | Type | Count |
|-------|--------|---------|------|-------|
| 3 | `+` | [Shift(4), Reduce(Binary)] | S/R | 1 |
| 3 | `*` | [Shift(5), Reduce(Binary)] | S/R | 1 |

**Conflict Count**: 2 shift/reduce conflicts

#### Ambiguous Input

```
1 + 2 * 3
```

**Valid Parse Trees**: 2

1. `(1 + 2) * 3` - left-associative
2. `1 + (2 * 3)` - right-associative / precedence

#### Expected GLR Behavior

- **Fork**: Parser creates branches at each operator
- **Merge**: All valid parse paths complete
- **Forest**: Contains 2 complete trees
- **No Default**: Without precedence, both are equally valid

#### BDD Scenario

```gherkin
Feature: Precedence-Free Expression Ambiguity

  Scenario: Arithmetic without precedence annotations
    Given a grammar without precedence annotations
    When I parse "1 + 2 * 3"
    Then the parse should succeed
    And the parse forest should contain 2 trees
    And tree 1 should evaluate to "((1 + 2) * 3) = 9"
    And tree 2 should evaluate to "(1 + (2 * 3)) = 7"

  Scenario: Longer expression with multiple ambiguities
    When I parse "1 + 2 + 3"
    Then the parse forest should contain 2 trees
    And tree 1 should be "((1 + 2) + 3)"
    And tree 2 should be "(1 + (2 + 3))"
```

#### Validation Criteria

- [ ] Grammar generates 2+ shift/reduce conflicts
- [ ] GLR parser forks on each operator
- [ ] All parse trees evaluate correctly
- [ ] Forest contains multiple valid trees
- [ ] No precedence bias in conflict resolution

---

### TG-003: Operator Associativity Ambiguity

**Classic Ambiguity**: Same-precedence operators without associativity

#### Grammar Definition

```rust
#[adze::grammar("assoc_ambiguous")]
mod assoc_ambiguous {
    #[adze::language]
    pub enum Expr {
        // Same precedence, NO associativity - ambiguous
        #[prec(1)]
        Sub(Box<Expr>, #[leaf(text = "-")] (), Box<Expr>),

        #[prec(1)]
        Div(Box<Expr>, #[leaf(text = "/")] (), Box<Expr>),

        Number(#[leaf(pattern = r"\d+")] i32),
    }
}
```

#### Expected Conflicts

| State | Symbol | Actions | Type | Note |
|-------|--------|---------|------|------|
| 4 | `-` | [Shift(5), Reduce(Sub)] | S/R | Same prec |
| 4 | `/` | [Shift(6), Reduce(Div)] | S/R | Same prec |

**Conflict Count**: 2 shift/reduce conflicts

#### Ambiguous Input

```
10 - 5 - 2
```

**Valid Parse Trees**: 2

1. `(10 - 5) - 2 = 3` - left-associative
2. `10 - (5 - 2) = 7` - right-associative

#### Expected GLR Behavior

- **Fork**: Parser creates branches at second operator
- **Merge**: Both associativity interpretations complete
- **Forest**: Contains 2 trees with different semantics
- **Critical**: Results are mathematically different

#### BDD Scenario

```gherkin
Feature: Operator Associativity Ambiguity

  Scenario: Subtraction without associativity annotation
    Given a grammar with same-precedence operators
    And no associativity annotations
    When I parse "10 - 5 - 2"
    Then the parse should succeed
    And the parse forest should contain 2 trees
    And tree 1 should evaluate to "((10 - 5) - 2) = 3"
    And tree 2 should evaluate to "(10 - (5 - 2)) = 7"
    And both trees should have error_count = 0
```

#### Validation Criteria

- [ ] Grammar generates conflicts for same-precedence operators
- [ ] GLR explores both associativity interpretations
- [ ] Evaluation yields different results
- [ ] Parse forest correctly represents both trees

---

### TG-004: Expression Statement Ambiguity

**Real-World Ambiguity**: C/Java-style expression statements

#### Grammar Definition

```rust
#[adze::grammar("expr_stmt")]
mod expr_stmt {
    #[adze::language]
    pub struct Program {
        pub stmts: Vec<Stmt>,
    }

    pub enum Stmt {
        Expr(Box<Expr>, #[leaf(text = ";")] ()),
        Block(#[leaf(text = "{")] (), Vec<Stmt>, #[leaf(text = "}")] ()),
    }

    pub enum Expr {
        Call(Box<Expr>, #[leaf(text = "(")] (), #[leaf(text = ")")] ()),
        Id(#[leaf(pattern = r"[a-z]+")] String),
    }
}
```

#### Expected Conflicts

| State | Symbol | Actions | Type | Context |
|-------|--------|---------|------|---------|
| 7 | `{` | [Shift(8), Reduce(Expr)] | S/R | Block vs expr |

**Conflict Count**: 1 shift/reduce

#### Ambiguous Input

```
{ foo(); }
```

**Valid Parse Trees**: 2

1. `Block([Expr(foo())])` - block containing expr statement
2. `Expr(Block([...]))` - expression statement of block expression (Rust-style)

#### Validation Criteria

- [ ] Grammar generates expected conflict
- [ ] Both interpretations are syntactically valid
- [ ] GLR preserves both parse trees

---

### TG-005: Reduce-Reduce Conflict

**Advanced Ambiguity**: Multiple reduction paths

#### Grammar Definition

```rust
#[adze::grammar("reduce_reduce")]
mod reduce_reduce {
    #[adze::language]
    pub enum Decl {
        TypeDecl(Type),
        VarDecl(Var),
    }

    pub struct Type {
        pub name: #[leaf(pattern = r"[A-Z][a-z]*")] String,
    }

    pub struct Var {
        pub name: #[leaf(pattern = r"[A-Z][a-z]*")] String,
    }
}
```

#### Expected Conflicts

| State | Symbol | Actions | Type | Note |
|-------|--------|---------|------|------|
| 2 | EOF | [Reduce(TypeDecl), Reduce(VarDecl)] | R/R | Same token pattern |

**Conflict Count**: 1 reduce/reduce

#### Ambiguous Input

```
Foo
```

**Valid Parse Trees**: 2

1. `TypeDecl("Foo")`
2. `VarDecl("Foo")`

#### Validation Criteria

- [ ] Grammar generates reduce/reduce conflict
- [ ] GLR creates fork for both reductions
- [ ] Both trees are structurally valid
- [ ] Parser handles R/R correctly (not just S/R)

---

## Test Infrastructure

### Conflict Inspection API

```rust
/// Inspect parse table for GLR conflicts
pub fn count_conflicts(table: &ParseTable) -> ConflictSummary {
    ConflictSummary {
        shift_reduce: usize,
        reduce_reduce: usize,
        states_with_conflicts: Vec<StateId>,
        conflict_details: Vec<ConflictDetail>,
    }
}

pub struct ConflictDetail {
    pub state: StateId,
    pub symbol: SymbolId,
    pub conflict_type: ConflictType,
    pub actions: Vec<Action>,
}
```

### Parse Forest API

```rust
/// Access GLR parse forest
pub trait ParseForest {
    fn tree_count(&self) -> usize;
    fn get_tree(&self, index: usize) -> Option<&ParseTree>;
    fn all_trees(&self) -> Vec<&ParseTree>;
    fn default_tree(&self) -> &ParseTree;
}
```

### Validation Helpers

```rust
/// Assert grammar generates expected conflicts
#[track_caller]
pub fn assert_conflict_count(
    grammar: &Grammar,
    expected_sr: usize,
    expected_rr: usize,
) {
    let summary = count_conflicts(&grammar.parse_table);
    assert_eq!(summary.shift_reduce, expected_sr,
        "Expected {} shift/reduce conflicts, found {}",
        expected_sr, summary.shift_reduce);
    assert_eq!(summary.reduce_reduce, expected_rr,
        "Expected {} reduce/reduce conflicts, found {}",
        expected_rr, summary.reduce_reduce);
}

/// Assert parse forest contains expected number of trees
#[track_caller]
pub fn assert_forest_size(
    forest: &impl ParseForest,
    expected: usize,
) {
    assert_eq!(forest.tree_count(), expected,
        "Expected {} parse trees, found {}",
        expected, forest.tree_count());
}
```

---

## BDD Test Structure

### Feature File Location

`tests/features/glr_ambiguous_grammars.feature`

### Scenario Template

```gherkin
Feature: GLR Ambiguous Grammar Handling

  Background:
    Given a GLR-enabled parser
    And conflict preservation is enabled

  Scenario Outline: <Grammar> handles <Input>
    Given grammar "<Grammar>"
    When I parse "<Input>"
    Then the parse should <Result>
    And the parse forest should contain <TreeCount> trees
    And conflict count should be <ConflictCount>
    And error count should be <ErrorCount>

  Examples:
    | Grammar | Input | Result | TreeCount | ConflictCount | ErrorCount |
    | dangling_else | if a if b c else d | succeed | 2 | 1 | 0 |
    | precedence_free | 1 + 2 * 3 | succeed | 2 | 2 | 0 |
    | assoc_ambiguous | 10 - 5 - 2 | succeed | 2 | 2 | 0 |
```

---

## Implementation Checklist

### Phase 2.1: Grammar Creation

- [ ] Create `example/src/dangling_else.rs`
- [ ] Create `example/src/precedence_free.rs`
- [ ] Create `example/src/assoc_ambiguous.rs`
- [ ] Create `example/src/expr_stmt.rs`
- [ ] Create `example/src/reduce_reduce.rs`

### Phase 2.2: Conflict Inspection

- [ ] Implement `count_conflicts()` in `glr-core`
- [ ] Add conflict inspection to `ParseTable`
- [ ] Create `ConflictSummary` type
- [ ] Write unit tests for conflict detection

### Phase 2.3: Parse Forest Support

- [ ] Define `ParseForest` trait
- [ ] Implement forest in GLR runtime
- [ ] Add forest serialization
- [ ] Write forest inspection tests

### Phase 2.4: Validation Tests

- [ ] Write conflict count tests for each grammar
- [ ] Write parse forest tests
- [ ] Implement BDD scenarios
- [ ] Add snapshot tests for parse trees

### Phase 2.5: Documentation

- [ ] Document each test grammar
- [ ] Update GLR status in STATUS_NOW.md
- [ ] Create GLR validation report
- [ ] Add examples to tutorial

---

## Success Criteria

### Per-Grammar Validation

For each test grammar:

1. ✅ Grammar compiles without errors
2. ✅ Conflict count matches specification
3. ✅ GLR parser forks at expected points
4. ✅ Parse forest contains expected tree count
5. ✅ All trees are structurally valid
6. ✅ Error count is 0 (or documented)
7. ✅ BDD scenarios pass

### Overall Validation

- [ ] All 5 test grammars pass validation
- [ ] Conflict inspection API works
- [ ] Parse forest API works
- [ ] BDD test suite is comprehensive
- [ ] Documentation is complete
- [ ] CI validates GLR behavior

---

## Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Decoder doesn't handle conflicts | Medium | High | Audit decoder in parallel |
| Parse forest too complex | Low | Medium | Start with simple tree count |
| Performance issues with large forests | Medium | Low | Add performance tests |
| Snapshot tests too brittle | Medium | Low | Use structural assertions |

---

## Timeline

- **Specification**: 1 hour (this document)
- **Grammar Creation**: 2-3 hours
- **Conflict Inspection**: 2-3 hours
- **Parse Forest Support**: 3-4 hours
- **Validation Tests**: 2-3 hours
- **Documentation**: 1-2 hours

**Total**: 11-16 hours (1-2 weeks part-time)

---

## References

- [GLR Runtime Wiring Plan](../plans/GLR_RUNTIME_WIRING_PLAN.md)
- [Production Readiness Roadmap](../PRODUCTION_READINESS_ROADMAP.md)
- [Tree-sitter Ambiguous Grammars](https://tree-sitter.github.io/tree-sitter/creating-parsers#structuring-rules-well)
- [Bison GLR](https://www.gnu.org/software/bison/manual/html_node/GLR-Parsers.html)

---

**Status**: Ready for Implementation
**Next**: Create TG-001 (Dangling Else) grammar
