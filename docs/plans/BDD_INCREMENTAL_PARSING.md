# BDD Specifications: Incremental Parsing

**Version**: 1.0.0
**Date**: 2025-11-20
**Status**: 📋 **PLANNED** (Phase II - Weeks 5-8)
**Contract**: [GLR_INCREMENTAL_CONTRACT.md](../specs/GLR_INCREMENTAL_CONTRACT.md)
**Architecture**: [ADR-0009: Incremental Parsing Architecture](../adr/ADR-0009-INCREMENTAL-PARSING-ARCHITECTURE.md)

---

## Overview

This document defines **Behavior-Driven Development (BDD) scenarios** for incremental parsing, following the Gherkin syntax for clear, testable specifications.

**Purpose**: Provide executable specifications that guide implementation and serve as acceptance tests.

**Methodology**:
- Given-When-Then format (Gherkin)
- Scenarios map to acceptance criteria (AC-I1 through AC-I5)
- Each scenario becomes a test in the test suite
- Contract-first: scenarios written before implementation

---

## Scenario Categories

1. **API Surface** (AC-I1) - 8 scenarios
2. **Correctness** (AC-I2) - 10 scenarios
3. **Performance** (AC-I3) - 6 scenarios
4. **Forest API** (AC-I4) - 5 scenarios
5. **Observability** (AC-I5) - 3 scenarios

**Total**: 32 BDD scenarios

---

## Phase I: API Surface (AC-I1)

### Scenario 1.1: Basic Edit and Incremental Reparse

```gherkin
Feature: Incremental API Surface
  As a developer
  I want to edit a tree and reparse incrementally
  So that I can efficiently update parse trees after small changes

Scenario: Edit a tree and reparse incrementally
  Given a parsed tree for input "let x = 1 + 2;"
  And the tree has nodes [LetStmt, Identifier("x"), BinExpr, Number(1), Plus, Number(2)]
  When I create an edit changing byte range 8..9 from "1" to "10"
  And I call tree.edit(edit)
  And I call parser.parse_incremental(new_input, Some(&tree))
  Then the new tree reflects "let x = 10 + 2;"
  And the tree structure is [LetStmt, Identifier("x"), BinExpr, Number(10), Plus, Number(2)]
  And nodes outside the edit are reused from old tree
```

**Test Location**: `runtime2/tests/test_incremental_api.rs::test_basic_edit_and_reparse`

---

### Scenario 1.2: Edit Updates Byte Ranges

```gherkin
Scenario: Edit updates byte ranges correctly
  Given a parsed tree for input "hello world"
  And the tree has nodes [Module, Identifier("hello"), Identifier("world")]
  When I create an edit inserting " beautiful" at byte 5
  And I call tree.edit(edit)
  Then node "hello" byte range is still 0..5
  And node "world" byte range is updated from 6..11 to 15..20
  And all ancestor nodes have updated byte ranges
```

**Test Location**: `runtime2/tests/test_incremental_api.rs::test_edit_updates_byte_ranges`

---

### Scenario 1.3: Edit Marks Dirty Subtrees

```gherkin
Scenario: Edit marks affected subtrees as dirty
  Given a parsed tree for input "fn foo() { x + y }"
  And the tree structure is [FnDef, Name("foo"), Block, BinExpr]
  When I edit the function body (byte range 11..17)
  And I call tree.edit(edit)
  Then the Block node is marked dirty
  And the BinExpr node is marked dirty
  And the FnDef node is marked dirty (ancestor)
  And nodes before byte 11 are NOT marked dirty
```

**Test Location**: `runtime2/tests/test_incremental_api.rs::test_edit_marks_dirty_subtrees`

---

### Scenario 1.4: Stable Node IDs Survive Clean Regions

```gherkin
Scenario: Stable node IDs survive in clean regions
  Given a parsed tree for input "fn foo() { x + y }\nfn bar() { a * b }"
  And I create anchors for all nodes
  When I edit the first function body (byte range 11..17)
  And I reparse incrementally
  Then anchors for "foo" function resolve to same node
  And anchors for "bar" function resolve to same node (clean region)
  And anchors for nodes in edited region return None
```

**Test Location**: `runtime2/tests/test_incremental_api.rs::test_stable_node_ids`

---

### Scenario 1.5: Empty Edit is a No-Op

```gherkin
Scenario: Empty edit has no effect
  Given a parsed tree for input "let x = 1;"
  When I create an edit with start_byte == old_end_byte == new_end_byte
  And I call tree.edit(edit)
  Then the tree is unchanged
  And no nodes are marked dirty
  And byte ranges are unchanged
```

**Test Location**: `runtime2/tests/test_incremental_api.rs::test_empty_edit_noop`

---

### Scenario 1.6: Insertion Edit

```gherkin
Scenario: Insertion edit shifts subsequent nodes
  Given a parsed tree for input "let x = 1;"
  When I insert " + 2" at byte 9 (before semicolon)
  And I call tree.edit(edit)
  And I reparse incrementally
  Then the new tree reflects "let x = 1 + 2;"
  And the semicolon byte range shifts from 9..10 to 14..15
  And the let statement is reparsed correctly
```

**Test Location**: `runtime2/tests/test_incremental_api.rs::test_insertion_edit`

---

### Scenario 1.7: Deletion Edit

```gherkin
Scenario: Deletion edit shrinks byte ranges
  Given a parsed tree for input "let x = 1 + 2;"
  When I delete " + 2" (byte range 9..13)
  And I call tree.edit(edit)
  And I reparse incrementally
  Then the new tree reflects "let x = 1;"
  And the semicolon byte range shifts from 13..14 to 9..10
  And the let statement is reparsed correctly
```

**Test Location**: `runtime2/tests/test_incremental_api.rs::test_deletion_edit`

---

### Scenario 1.8: Replacement Edit

```gherkin
Scenario: Replacement edit combines delete and insert
  Given a parsed tree for input "let x = 1;"
  When I replace "1" with "42" (byte range 8..9 → "42")
  And I call tree.edit(edit)
  And I reparse incrementally
  Then the new tree reflects "let x = 42;"
  And the semicolon byte range shifts from 9..10 to 10..11
  And the number node kind is still Number
```

**Test Location**: `runtime2/tests/test_incremental_api.rs::test_replacement_edit`

---

## Phase II: Correctness (AC-I2)

### Scenario 2.1: Incremental Equals Full Parse (Golden Test)

```gherkin
Feature: Incremental Correctness
  As a parser user
  I want incremental parsing to produce identical results to full parsing
  So that I can trust the incremental path

Scenario: Incremental parse equals full parse
  Given a grammar and test input corpus (100+ files)
  For each test file:
    When I parse the file fully to get baseline tree
    And I apply a random edit to the file
    And I parse incrementally with old tree
    And I parse fully from scratch
    Then incremental tree structure == full tree structure
    And all node kinds match
    And all byte ranges match
    And all text content matches
```

**Test Location**: `runtime2/tests/test_incremental_correctness.rs::test_golden_corpus`

---

### Scenario 2.2: Property-Based Correctness

```gherkin
Scenario: Arbitrary edit sequences produce correct trees
  Given a random input string
  And a random sequence of edits
  For each edit in sequence:
    When I apply edit to input
    And I parse incrementally
    Then incremental tree == full parse tree for current input
```

**Test Location**: `runtime2/tests/test_incremental_correctness.rs::test_quickcheck_arbitrary_edits`

**Implementation**:
```rust
#[quickcheck]
fn incremental_always_correct(input: String, edits: Vec<Edit>) -> bool {
    let mut tree = parse_full(&input);
    let mut current_input = input;

    for edit in edits {
        current_input = apply_edit(&current_input, &edit);
        tree.edit(&edit);
        let inc_tree = parse_incremental(&current_input, Some(&tree));
        let full_tree = parse_full(&current_input);

        if !trees_equal(&inc_tree, &full_tree) {
            return false;
        }

        tree = inc_tree;
    }
    true
}
```

---

### Scenario 2.3: Ambiguity Preservation

```gherkin
Scenario: GLR ambiguities handled correctly in incremental mode
  Given the dangling-else grammar
  And input "if a then if b then s1 else s2"
  And a full parse producing 2 ambiguous trees
  When I edit "s1" to "statement1"
  And I parse incrementally
  Then the parse still reports 2 ambiguities
  And both alternative trees reflect the edit
  And default tree selection is consistent with full parse
```

**Test Location**: `runtime2/tests/test_incremental_correctness.rs::test_ambiguity_preservation`

---

### Scenario 2.4: Edge Case - Edit at Start of File

```gherkin
Scenario: Edit at start of file
  Given input "let x = 1;"
  When I insert "// comment\n" at byte 0
  And I parse incrementally
  Then the new tree starts with Comment node
  And the LetStmt node byte range shifts correctly
  And tree structure matches full parse
```

**Test Location**: `runtime2/tests/test_incremental_correctness.rs::test_edit_at_start`

---

### Scenario 2.5: Edge Case - Edit at End of File

```gherkin
Scenario: Edit at end of file
  Given input "let x = 1;"
  When I append "\nlet y = 2;" at end
  And I parse incrementally
  Then the tree has two LetStmt nodes
  And first LetStmt is unchanged (clean region)
  And second LetStmt is parsed correctly
  And tree structure matches full parse
```

**Test Location**: `runtime2/tests/test_incremental_correctness.rs::test_edit_at_end`

---

### Scenario 2.6: Edge Case - Multi-Line Edit

```gherkin
Scenario: Multi-line edit spanning functions
  Given input:
    """
    fn foo() { x + y }
    fn bar() { a * b }
    fn baz() { i - j }
    """
  When I delete lines 2-3 (entire bar function)
  And I parse incrementally
  Then the tree has two FnDef nodes (foo, baz)
  And foo function is unchanged (clean region)
  And baz function byte ranges are correct
  And tree structure matches full parse
```

**Test Location**: `runtime2/tests/test_incremental_correctness.rs::test_multiline_edit`

---

### Scenario 2.7: Parent-Child Relationships Preserved

```gherkin
Scenario: Parent-child relationships remain consistent
  Given a parsed tree for input "fn foo() { x + y }"
  When I edit the function body
  And I parse incrementally
  Then all nodes have correct parent links
  And parent.child(i) == child for all i
  And child.parent() == parent for all children
  And tree.root_node().parent() is None
```

**Test Location**: `runtime2/tests/test_incremental_correctness.rs::test_parent_child_consistency`

---

### Scenario 2.8: No Data Loss

```gherkin
Scenario: All information preserved or correctly invalidated
  Given a parsed tree with cached properties (text, positions)
  When I apply an edit
  And I parse incrementally
  Then all properties in clean regions are preserved
  And all properties in dirty regions are correctly recomputed
  And no stale cached data remains
```

**Test Location**: `runtime2/tests/test_incremental_correctness.rs::test_no_data_loss`

---

### Scenario 2.9: Undo Scenario

```gherkin
Scenario: Edit followed by undo (reverse edit)
  Given a parsed tree for input "let x = 1;"
  When I change "1" to "42"
  And I parse incrementally to get tree2
  And I change "42" back to "1" (undo)
  And I parse incrementally to get tree3
  Then tree3 structure matches original tree
  And all byte ranges match original
```

**Test Location**: `runtime2/tests/test_incremental_correctness.rs::test_undo_scenario`

---

### Scenario 2.10: Large File Correctness

```gherkin
Scenario: Incremental correct for large files
  Given a 10,000 line Python file
  When I parse it fully (baseline)
  And I edit line 5,000 (middle of file)
  And I parse incrementally
  Then incremental tree == full parse tree
  And the edit completes in reasonable time (<1s)
```

**Test Location**: `runtime2/tests/test_incremental_correctness.rs::test_large_file_correctness`

---

## Phase III: Performance (AC-I3)

### Scenario 3.1: Single-Line Edit Performance

```gherkin
Feature: Incremental Performance
  As a parser user
  I want single-line edits to be much faster than full parses
  So that interactive tools have sub-100ms latency

Scenario: Single-line edit is fast
  Given a 1000-line Python file
  And a full parse taking T_full milliseconds (baseline)
  When I change one character on line 500
  And I parse incrementally
  Then parse time < 0.3 × T_full
  And reuse percentage > 70%
  And no full parse fallback triggered
  And metrics report ParseMode::Incremental
```

**Test Location**: `benchmarks/benches/incremental.rs::bench_single_line_edit`

---

### Scenario 3.2: Multi-Line Edit Performance

```gherkin
Scenario: Multi-line edit is reasonably fast
  Given a 1000-line Python file
  And a full parse taking T_full milliseconds (baseline)
  When I edit 10 consecutive lines in the middle
  And I parse incrementally
  Then parse time < 0.5 × T_full
  And reuse percentage > 50%
  And no full parse fallback triggered
```

**Test Location**: `benchmarks/benches/incremental.rs::bench_multiline_edit`

---

### Scenario 3.3: Large Edit Fallback

```gherkin
Scenario: Large edit triggers automatic fallback
  Given a 1000-line Python file
  When I replace 600 lines (60% of file)
  And I parse incrementally
  Then fallback is triggered automatically
  And parse mode is ParseMode::Fallback
  And fallback reason is EditTooLarge { percent: 60.0 }
  And warning is logged
  And parse time is comparable to full parse
```

**Test Location**: `runtime2/tests/test_incremental_performance.rs::test_large_edit_fallback`

---

### Scenario 3.4: Reuse Percentage Tracking

```gherkin
Scenario: Reuse percentage is tracked accurately
  Given a parsed tree for input with 100 nodes
  When I edit affecting 10 nodes (10% of tree)
  And I parse incrementally
  Then metrics report reuse_percentage ≈ 90%
  And clean_nodes_reused ≈ 90
  And dirty_nodes ≈ 10
```

**Test Location**: `runtime2/tests/test_incremental_performance.rs::test_reuse_percentage`

---

### Scenario 3.5: No Pathological Cases

```gherkin
Scenario: Worst-case edits have bounded cost
  Given various pathological edit patterns:
    | Pattern | Description |
    | Prepend 1KB | Insert at start |
    | Append 1KB | Insert at end |
    | Middle insert 1KB | Insert in middle |
    | Delete first half | Large deletion |
    | Replace all | Full replacement |
  For each pattern:
    When I apply the edit
    And I parse incrementally
    Then either reparse is reasonably fast (<2x full parse)
    Or fallback is triggered (parse mode == Fallback)
    And no unbounded hang or crash
```

**Test Location**: `runtime2/tests/test_incremental_performance.rs::test_pathological_cases`

---

### Scenario 3.6: Performance Regression Detection

```gherkin
Scenario: Performance regressions are caught in CI
  Given baseline performance measurements
  When a PR is opened with changes to incremental parsing
  And CI benchmarks run
  Then performance is compared to baseline
  And if any benchmark regresses >5%
  Then CI fails with clear error message
  And regression report is posted to PR
```

**Test Location**: `.github/workflows/performance.yml` (CI configuration)

---

## Phase IV: Forest API (AC-I4)

### Scenario 4.1: Inspect Ambiguous Parse

```gherkin
Feature: Forest API
  As a parser user
  I want to inspect ambiguities in parse results
  So that I can debug or analyze all valid interpretations

Scenario: Inspect ambiguous parse
  Given the dangling-else grammar
  When I parse "if a then if b then s1 else s2"
  Then parse result reports ambiguities == 2
  And forest_handle is Some
  And forest_handle.root_alternatives() returns 2 nodes
  And I can traverse both parse trees independently
  And both trees have valid structure
```

**Test Location**: `runtime2/tests/test_forest_api.rs::test_inspect_ambiguity`

---

### Scenario 4.2: Unambiguous Parse Has Trivial Forest

```gherkin
Scenario: Unambiguous parse has single-path forest
  Given the arithmetic grammar with precedence
  When I parse "1 + 2 * 3"
  Then parse result reports ambiguities == 0
  And forest_handle is Some (forest always available)
  And forest_handle.root_alternatives() returns 1 node
  And forest structure matches default tree
```

**Test Location**: `runtime2/tests/test_forest_api.rs::test_unambiguous_forest`

---

### Scenario 4.3: Forest Node Traversal

```gherkin
Scenario: Traverse forest nodes
  Given a forest from ambiguous parse
  And a forest node ID
  When I call forest.children(id)
  Then I get a slice of child node IDs
  And I can call forest.kind(child_id) for each child
  And I can call forest.byte_range(child_id) for each child
  And all child byte ranges are within parent byte range
```

**Test Location**: `runtime2/tests/test_forest_api.rs::test_forest_traversal`

---

### Scenario 4.4: Graphviz Export

```gherkin
Scenario: Export forest as Graphviz for visualization
  Given a forest from dangling-else parse
  When I call forest.to_graphviz()
  Then I get a valid DOT format string
  And the string contains "digraph"
  And the string contains nodes with labels
  And the string contains edges (->)
  And I can render it with Graphviz tools
```

**Test Location**: `runtime2/tests/test_forest_api.rs::test_graphviz_export`

---

### Scenario 4.5: Resolve Alternative to Tree

```gherkin
Scenario: Resolve a specific alternative to a Tree
  Given a forest with 2 root alternatives
  When I call forest.resolve_alternative(alt1_id)
  Then I get a Tree object
  And the tree has full Tree API support
  And tree.root_node().to_sexp() shows first interpretation
  When I call forest.resolve_alternative(alt2_id)
  Then I get a different Tree object
  And tree.root_node().to_sexp() shows second interpretation
```

**Test Location**: `runtime2/tests/test_forest_api.rs::test_resolve_alternative`

---

## Phase V: Observability (AC-I5)

### Scenario 5.1: Metrics Tracking

```gherkin
Feature: Observability
  As a parser user
  I want visibility into parse behavior
  So that I can debug performance and understand system behavior

Scenario: Incremental metrics are tracked
  Given RUST_SITTER_LOG_INCREMENTAL=1 environment variable
  When I parse incrementally
  Then metrics are emitted to stderr
  And metrics include:
    | Field | Type |
    | parse_mode | Incremental \| Full \| Fallback |
    | reuse_percentage | f32 (0-100) |
    | dirty_nodes | usize |
    | clean_nodes_reused | usize |
    | parse_time_ms | f32 |
```

**Test Location**: `runtime2/tests/test_incremental_observability.rs::test_metrics_tracking`

---

### Scenario 5.2: Fallback Logging

```gherkin
Scenario: Fallback is logged with reason
  Given a parser configured with logging
  When an edit triggers fallback
  Then a warning is logged
  And the warning includes fallback reason
  And the warning includes edit size percentage
  Example: "Incremental parse fell back to full parse: EditTooLarge { percent: 55.0 }"
```

**Test Location**: `runtime2/tests/test_incremental_observability.rs::test_fallback_logging`

---

### Scenario 5.3: CI Metrics Dashboard

```gherkin
Scenario: Incremental metrics tracked in CI
  Given a PR with changes
  When CI runs incremental tests
  Then metrics are collected for each test
  And metrics are aggregated into a report
  And report shows:
    - Average reuse percentage
    - Average parse time ratio (incremental / full)
    - Fallback frequency
  And report is uploaded to CI artifacts
  And trend over time is visible on dashboard
```

**Test Location**: `.github/workflows/incremental-metrics.yml` (CI configuration)

---

## Test Implementation Guide

### Mapping Scenarios to Test Code

Each scenario maps to one or more test functions:

```rust
// Scenario 1.1: Basic Edit and Incremental Reparse
#[test]
fn test_basic_edit_and_reparse() {
    // Given
    let grammar = load_test_grammar();
    let input = "let x = 1 + 2;";
    let tree = parse_full(&grammar, input);
    assert_tree_structure(&tree, &["LetStmt", "Identifier", "BinExpr", ...]);

    // When
    let edit = Edit {
        start_byte: 8,
        old_end_byte: 9,
        new_end_byte: 10,
        // ... positions ...
    };
    let new_input = "let x = 10 + 2;";
    let mut tree = tree;
    tree.edit(&edit);
    let inc_tree = parse_incremental(&grammar, new_input, Some(&tree));

    // Then
    assert_eq!(inc_tree.root_node().utf8_text(new_input.as_bytes()), new_input);
    assert_tree_structure(&inc_tree, &["LetStmt", "Identifier", "BinExpr", ...]);
    assert!(node_was_reused(&inc_tree, &tree, "Identifier")); // "x" reused
}
```

### Property-Based Testing

Use `quickcheck` for property-based scenarios:

```rust
#[quickcheck]
fn prop_incremental_equals_full(input: String, edit: Edit) -> bool {
    let tree = parse_full(&input);
    let new_input = apply_edit(&input, &edit);

    let mut tree_copy = tree.clone();
    tree_copy.edit(&edit);
    let inc_tree = parse_incremental(&new_input, Some(&tree_copy));

    let full_tree = parse_full(&new_input);

    trees_equal(&inc_tree, &full_tree)
}
```

### Benchmark Testing

Use `criterion` for performance scenarios:

```rust
fn bench_single_line_edit(c: &mut Criterion) {
    let mut group = c.benchmark_group("incremental");

    let grammar = load_python_grammar();
    let input = load_1000_line_file();

    // Baseline: full parse
    group.bench_function("full_parse", |b| {
        b.iter(|| parse_full(&grammar, &input))
    });

    // Incremental: single-line edit
    let tree = parse_full(&grammar, &input);
    let edit = make_single_line_edit(&input, 500); // Edit line 500

    group.bench_function("incremental_1line", |b| {
        b.iter(|| {
            let mut t = tree.clone();
            t.edit(&edit);
            parse_incremental(&grammar, &input, Some(&t))
        })
    });
}
```

---

## BDD Coverage Matrix

| Acceptance Criterion | Scenario Count | Coverage |
|----------------------|----------------|----------|
| AC-I1: API Surface | 8 | API usage, edit types, node IDs |
| AC-I2: Correctness | 10 | Golden tests, properties, edge cases |
| AC-I3: Performance | 6 | Speed targets, reuse, fallback |
| AC-I4: Forest API | 5 | Traversal, visualization, alternatives |
| AC-I5: Observability | 3 | Metrics, logging, CI |
| **Total** | **32** | **Complete contract coverage** |

---

## BDD Workflow

### 1. Scenario Definition (Week 1)

Write scenarios **before** implementation:
- Define given/when/then for each AC
- Review with team
- Ensure testability

### 2. Test Skeleton (Week 1)

Create failing test stubs:
```rust
#[test]
#[ignore = "Not yet implemented"]
fn test_basic_edit_and_reparse() {
    unimplemented!("Scenario 1.1")
}
```

### 3. Implementation (Weeks 2-4)

Implement incrementally:
- Red: Write test (fails)
- Green: Implement (passes)
- Refactor: Clean up

### 4. Validation (Week 5)

All scenarios pass:
- Remove `#[ignore]` attributes
- Verify 100% pass rate
- Document any deviations

---

## Success Criteria

BDD specifications are complete when:

1. ✅ All 32 scenarios defined and reviewed
2. ✅ Each scenario maps to test function(s)
3. ✅ All tests passing (100% pass rate)
4. ✅ Coverage verified (all ACs covered)
5. ✅ Edge cases identified and tested
6. ✅ Property tests validate invariants
7. ✅ Performance benchmarks meet targets

---

## References

- [GLR_INCREMENTAL_CONTRACT.md](../specs/GLR_INCREMENTAL_CONTRACT.md) - Full contract
- [ADR-0009: Incremental Parsing Architecture](../adr/ADR-0009-INCREMENTAL-PARSING-ARCHITECTURE.md) - Design
- [BDD_GLR_CONFLICT_PRESERVATION.md](./BDD_GLR_CONFLICT_PRESERVATION.md) - GLR v1 BDD examples

---

**Document Version**: 1.0.0
**Last Updated**: 2025-11-20
**Maintained By**: rust-sitter core team

---

END OF BDD SPECIFICATIONS
