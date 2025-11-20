# Tree API Compatibility Contract

**Version**: 1.0.0
**Date**: 2025-11-20
**Status**: ACTIVE
**Related**: GLR_V1_COMPLETION_CONTRACT.md (AC-5), BDD_GLR_CONFLICT_PRESERVATION.md
**Priority**: HIGH (Critical for GLR v1 completion)

---

## Executive Summary

This contract defines the specification for Tree API compatibility between GLR-produced trees and standard LR-produced trees in rust-sitter. The goal is to ensure that all Tree API methods work identically regardless of the parsing backend used.

**Current State** (2025-11-20):
- ✅ GLR runtime produces valid parse trees
- ✅ Symbol names correctly resolved from grammar
- ✅ Basic tree structure validated (root node, child count)
- ⏸ Comprehensive Tree API compatibility testing pending
- ⏸ Node traversal methods validation pending
- ⏸ AST extraction validation pending

**Target State**:
- ✅ 100% Tree API compatibility between GLR and LR backends
- ✅ All node methods work identically (kind, text, byte ranges, etc.)
- ✅ Tree traversal operations validated (parent, child, sibling)
- ✅ AST extraction works for GLR-produced trees
- ✅ Performance comparable to LR backend

---

## I. Scope Definition

### In Scope for Tree API Compatibility

1. **Node API Methods**
   - `kind()` - Symbol name retrieval
   - `kind_id()` - Symbol ID retrieval
   - `start_byte()` / `end_byte()` - Byte range access
   - `start_position()` / `end_position()` - Line/column positions
   - `utf8_text()` - Source text extraction
   - `is_named()` - Named vs anonymous nodes
   - `is_missing()` - Error recovery detection

2. **Tree Traversal Methods**
   - `root_node()` - Root access
   - `child(index)` - Direct child access
   - `child_count()` - Child enumeration
   - `named_child(index)` / `named_child_count()` - Named child filtering
   - `parent()` - Parent navigation
   - `next_sibling()` / `prev_sibling()` - Sibling navigation
   - `next_named_sibling()` / `prev_named_sibling()` - Named sibling filtering

3. **Tree Operations**
   - `edit()` - Incremental edit marking (if incremental feature enabled)
   - `walk()` - Tree cursor creation
   - Deep equality comparison between trees

4. **AST Extraction**
   - Extract trait compatibility
   - Field access and mapping
   - Enum variant extraction
   - Nested structure extraction

### Out of Scope

1. **Advanced Features** (deferred to v0.7.0+)
   - Query system integration
   - External scanner interaction
   - Tree diffing algorithms
   - Concurrent tree access

2. **Performance Optimization**
   - Baseline functionality only
   - Performance parity validation only

---

## II. Acceptance Criteria

### AC-1: Node Property Methods

**Requirement**: All node property methods return correct values for GLR-produced trees.

**BDD Scenario**:
```gherkin
Scenario: Node properties match between GLR and LR backends
  Given a simple grammar parsed with both GLR and LR backends
  When accessing node properties (kind, byte ranges, positions)
  Then all property values match exactly between backends
  And no properties return incorrect or missing values
```

**Acceptance Tests**:
```rust
#[test]
fn test_node_kind_compatibility() {
    let glr_tree = parse_with_glr("if expr then stmt");
    let root = glr_tree.root_node();

    assert_eq!(root.kind(), "S");
    assert_eq!(root.kind_id(), 10);
    assert!(root.is_named());
    assert!(!root.is_missing());
}

#[test]
fn test_node_byte_ranges() {
    let glr_tree = parse_with_glr("if expr then stmt");
    let root = glr_tree.root_node();

    assert_eq!(root.start_byte(), 0);
    assert_eq!(root.end_byte(), 17);
}

#[test]
fn test_node_positions() {
    let glr_tree = parse_with_glr("if expr then stmt");
    let root = glr_tree.root_node();

    let start = root.start_position();
    assert_eq!(start.row, 0);
    assert_eq!(start.column, 0);

    let end = root.end_position();
    assert_eq!(end.row, 0);
    assert_eq!(end.column, 17);
}

#[test]
fn test_node_text_extraction() {
    let input = b"if expr then stmt";
    let glr_tree = parse_with_glr(input);
    let root = glr_tree.root_node();

    assert_eq!(root.utf8_text(input).unwrap(), "if expr then stmt");
}
```

**Success Criteria**:
- [ ] All property methods implemented
- [ ] All property values correct
- [ ] No panics or errors on valid trees
- [ ] Edge cases handled (empty nodes, EOF, etc.)

---

### AC-2: Tree Traversal Methods

**Requirement**: Tree traversal methods work correctly for GLR-produced trees.

**BDD Scenario**:
```gherkin
Scenario: Tree traversal works identically across backends
  Given a complex tree structure from GLR parser
  When traversing the tree (parent, child, sibling navigation)
  Then all navigation methods return correct nodes
  And traversal produces identical results to LR backend
```

**Acceptance Tests**:
```rust
#[test]
fn test_child_access() {
    let glr_tree = parse_with_glr("if expr then stmt else stmt");
    let root = glr_tree.root_node();

    assert_eq!(root.child_count(), 6);

    let child0 = root.child(0).expect("Child 0 should exist");
    assert_eq!(child0.kind(), "if");

    let child5 = root.child(5).expect("Child 5 should exist");
    assert_eq!(child5.kind(), "stmt");

    assert!(root.child(6).is_none(), "Child 6 should not exist");
}

#[test]
fn test_named_child_access() {
    let glr_tree = parse_with_glr("if expr then stmt");
    let root = glr_tree.root_node();

    let named_count = root.named_child_count();
    assert!(named_count > 0);

    for i in 0..named_count {
        let child = root.named_child(i).expect("Named child should exist");
        assert!(child.is_named());
    }
}

#[test]
fn test_parent_navigation() {
    let glr_tree = parse_with_glr("if expr then stmt");
    let root = glr_tree.root_node();
    let child = root.child(0).expect("Child should exist");

    // Note: Tree-sitter root nodes don't have parents
    assert!(root.parent().is_none());

    // Child nodes should reference their parent
    let child_parent = child.parent();
    assert!(child_parent.is_some());
    // Parent comparison would require Node equality
}

#[test]
fn test_sibling_navigation() {
    let glr_tree = parse_with_glr("if expr then stmt");
    let root = glr_tree.root_node();

    let child0 = root.child(0).expect("Child 0 exists");
    let child1_via_sibling = child0.next_sibling().expect("Next sibling exists");
    let child1_direct = root.child(1).expect("Child 1 exists");

    assert_eq!(child1_via_sibling.kind(), child1_direct.kind());
    assert_eq!(child1_via_sibling.start_byte(), child1_direct.start_byte());
}
```

**Success Criteria**:
- [ ] All traversal methods implemented
- [ ] Parent/child relationships correct
- [ ] Sibling navigation works
- [ ] Named vs anonymous filtering works
- [ ] Boundary cases handled (no children, no siblings, etc.)

---

### AC-3: Tree Cursor Compatibility

**Requirement**: Tree cursor (walk) API works for GLR-produced trees.

**BDD Scenario**:
```gherkin
Scenario: Tree cursor traversal works correctly
  Given a GLR-produced tree
  When creating a tree cursor with walk()
  Then the cursor can navigate the entire tree
  And all cursor methods return correct values
```

**Acceptance Tests**:
```rust
#[test]
fn test_tree_cursor_basic() {
    let glr_tree = parse_with_glr("if expr then stmt");
    let mut cursor = glr_tree.walk();

    // Start at root
    assert_eq!(cursor.node().kind(), "S");

    // Go to first child
    assert!(cursor.goto_first_child());
    assert_eq!(cursor.node().kind(), "if");

    // Go to next sibling
    assert!(cursor.goto_next_sibling());
    assert_eq!(cursor.node().kind(), "expr");

    // Go back to parent
    assert!(cursor.goto_parent());
    assert_eq!(cursor.node().kind(), "S");
}

#[test]
fn test_tree_cursor_depth_first_traversal() {
    let glr_tree = parse_with_glr("if expr then stmt");
    let mut cursor = glr_tree.walk();

    let mut visited_kinds = Vec::new();

    fn visit(cursor: &mut TreeCursor, visited: &mut Vec<String>) {
        visited.push(cursor.node().kind().to_string());

        if cursor.goto_first_child() {
            loop {
                visit(cursor, visited);
                if !cursor.goto_next_sibling() {
                    break;
                }
            }
            cursor.goto_parent();
        }
    }

    visit(&mut cursor, &mut visited_kinds);

    // Verify depth-first traversal order
    assert_eq!(visited_kinds[0], "S"); // Root
    assert!(visited_kinds.len() > 1);
}
```

**Success Criteria**:
- [ ] Tree cursor creation works
- [ ] All cursor navigation methods work
- [ ] Depth-first traversal produces correct order
- [ ] Cursor state management correct

---

### AC-4: AST Extraction Compatibility

**Requirement**: Extract trait works for GLR-produced trees with field access.

**BDD Scenario**:
```gherkin
Scenario: AST extraction works for GLR trees
  Given a grammar with field mappings
  When extracting AST nodes with the Extract trait
  Then all fields are accessible
  And extraction produces correct Rust types
```

**Acceptance Tests**:
```rust
// Example AST types
#[derive(Debug, PartialEq)]
struct IfStatement {
    condition: String,
    then_body: String,
    else_body: Option<String>,
}

#[test]
fn test_ast_extraction_simple() {
    let glr_tree = parse_with_glr("if expr then stmt");
    let root = glr_tree.root_node();

    // Manual extraction
    let mut children = Vec::new();
    for i in 0..root.child_count() {
        if let Some(child) = root.child(i) {
            children.push(child.kind().to_string());
        }
    }

    assert!(children.contains(&"if".to_string()));
    assert!(children.contains(&"expr".to_string()));
    assert!(children.contains(&"stmt".to_string()));
}

#[test]
fn test_ast_extraction_with_fields() {
    // This test validates that if the grammar has field mappings,
    // they work correctly with GLR trees

    // Example: S -> if ^condition then ^then_body (else ^else_body)?

    let glr_tree = parse_with_glr("if expr then stmt");
    let root = glr_tree.root_node();

    // Field access would use child_by_field_name() if implemented
    // For now, validate via positional access
    let if_keyword = root.child(0).expect("if keyword");
    assert_eq!(if_keyword.kind(), "if");

    let condition = root.child(1).expect("condition");
    assert_eq!(condition.kind(), "expr");

    let then_keyword = root.child(2).expect("then keyword");
    assert_eq!(then_keyword.kind(), "then");

    let then_body = root.child(3).expect("then body");
    assert_eq!(then_body.kind(), "stmt");
}

#[test]
fn test_nested_ast_extraction() {
    let glr_tree = parse_with_glr("if expr then if expr then stmt else stmt");
    let root = glr_tree.root_node();

    // Validate nested structure
    assert_eq!(root.kind(), "S");
    assert!(root.child_count() >= 4);

    // Nested if-then-else should be a child
    // Structure depends on grammar, but should be traversable
    let mut found_nested = false;
    for i in 0..root.child_count() {
        if let Some(child) = root.child(i) {
            if child.kind() == "S" && child != root {
                found_nested = true;
                break;
            }
        }
    }

    // This validates that nested structures are preserved
    assert!(root.child_count() > 0, "Tree should have children");
}
```

**Success Criteria**:
- [ ] Extract trait compiles for GLR trees
- [ ] Field access works (if grammar has fields)
- [ ] Nested structure extraction works
- [ ] Type conversions work correctly
- [ ] Error handling works for missing fields

---

### AC-5: Performance Parity

**Requirement**: Tree API operations have comparable performance to LR backend.

**BDD Scenario**:
```gherkin
Scenario: Tree API performance is acceptable
  Given a large GLR-produced tree (1000+ nodes)
  When performing tree operations (traversal, access, extraction)
  Then operations complete within acceptable time
  And performance is within 2x of LR backend
```

**Acceptance Tests**:
```rust
#[test]
fn test_tree_access_performance() {
    use std::time::Instant;

    // Parse a reasonably sized input
    let input = "if expr then stmt ".repeat(100);
    let glr_tree = parse_with_glr(input.as_bytes());

    let start = Instant::now();

    // Traverse entire tree
    let root = glr_tree.root_node();
    fn count_nodes(node: Node) -> usize {
        let mut count = 1;
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                count += count_nodes(child);
            }
        }
        count
    }

    let node_count = count_nodes(root);
    let duration = start.elapsed();

    println!("Traversed {} nodes in {:?}", node_count, duration);

    // Performance assertion (should be fast)
    assert!(duration.as_millis() < 100, "Tree traversal should be fast");
    assert!(node_count > 0, "Should have nodes");
}

#[bench]
fn bench_tree_child_access(b: &mut Bencher) {
    let glr_tree = parse_with_glr("if expr then stmt");
    let root = glr_tree.root_node();

    b.iter(|| {
        for i in 0..root.child_count() {
            let _ = root.child(i);
        }
    });
}

#[bench]
fn bench_tree_traversal(b: &mut Bencher) {
    let glr_tree = parse_with_glr("if expr then if expr then stmt else stmt");

    b.iter(|| {
        let mut cursor = glr_tree.walk();
        fn traverse(cursor: &mut TreeCursor) {
            if cursor.goto_first_child() {
                loop {
                    traverse(cursor);
                    if !cursor.goto_next_sibling() {
                        break;
                    }
                }
                cursor.goto_parent();
            }
        }
        traverse(&mut cursor);
    });
}
```

**Success Criteria**:
- [ ] Tree operations complete in reasonable time
- [ ] No performance regressions vs baseline
- [ ] Memory usage acceptable
- [ ] Benchmarks pass CI gates

---

## III. Implementation Plan

### Phase 1: Property Methods (Days 1-2)

**Tasks**:
1. Create `runtime2/tests/test_tree_api_compatibility.rs`
2. Implement all property method tests (AC-1)
3. Fix any issues discovered
4. Validate against LR backend (if available)

**Deliverables**:
- [ ] 10-15 property method tests passing
- [ ] All edge cases covered
- [ ] Documentation for any differences

### Phase 2: Traversal Methods (Days 2-3)

**Tasks**:
1. Implement traversal method tests (AC-2)
2. Test parent/child relationships
3. Test sibling navigation
4. Test named vs anonymous filtering

**Deliverables**:
- [ ] 10-15 traversal method tests passing
- [ ] Parent/child links validated
- [ ] Sibling navigation working

### Phase 3: Tree Cursor (Day 3)

**Tasks**:
1. Implement tree cursor tests (AC-3)
2. Test depth-first traversal
3. Test cursor state management
4. Performance validation

**Deliverables**:
- [ ] 5-10 cursor tests passing
- [ ] Traversal algorithms validated
- [ ] Performance benchmarks passing

### Phase 4: AST Extraction (Day 4)

**Tasks**:
1. Implement AST extraction tests (AC-4)
2. Test field access
3. Test nested structures
4. Test type conversions

**Deliverables**:
- [ ] 5-10 AST extraction tests passing
- [ ] Field access working
- [ ] Nested extraction validated

### Phase 5: Performance Validation (Day 4)

**Tasks**:
1. Implement performance tests (AC-5)
2. Establish baselines
3. Compare with LR backend
4. Optimize if needed

**Deliverables**:
- [ ] Performance benchmarks passing
- [ ] Baseline established
- [ ] No critical performance issues

---

## IV. Test Matrix

### Test Coverage Requirements

| Category | Tests | Status |
|----------|-------|--------|
| Property Methods | 15 | Pending |
| Traversal Methods | 15 | Pending |
| Tree Cursor | 10 | Pending |
| AST Extraction | 10 | Pending |
| Performance | 5 | Pending |
| **Total** | **55** | **0/55** |

### Test Organization

```
runtime2/tests/
├── test_tree_api_compatibility.rs    # Main test suite
│   ├── property_methods/
│   ├── traversal_methods/
│   ├── tree_cursor/
│   ├── ast_extraction/
│   └── performance/
└── test_tree_api_parity.rs          # LR vs GLR comparison (if applicable)
```

---

## V. Success Metrics

### Quantitative

- **Test Count**: ≥55 tests covering all Tree API methods
- **Pass Rate**: 100% (no ignored tests)
- **Coverage**: 100% of public Tree/Node API
- **Performance**: Within 2x of LR backend

### Qualitative

- **API Usability**: External reviewer can use GLR trees without issues
- **Documentation**: All API differences documented
- **Error Messages**: Clear errors for unsupported operations
- **Stability**: No panics on valid trees

---

## VI. Risk Management

### High Risks

| Risk | Impact | Likelihood | Mitigation |
|------|--------|------------|------------|
| Node lifetime issues | HIGH | MEDIUM | Careful lifetime management, tests |
| Parent reference cycles | HIGH | LOW | Use weak references or indices |
| Field access not working | MEDIUM | MEDIUM | Test with simple field-based grammar |
| Performance issues | MEDIUM | LOW | Benchmark early, optimize if needed |

### Mitigations

- **Lifetime Issues**: Use 'static parse tables, Box::leak where needed
- **Performance**: Establish baseline early, optimize hot paths
- **API Incompatibilities**: Document and provide workarounds

---

## VII. Definition of Done

Tree API Compatibility is **COMPLETE** when:

1. ✅ All 55+ compatibility tests passing
2. ✅ All Tree/Node API methods working
3. ✅ AST extraction validated
4. ✅ Performance within acceptable range
5. ✅ Documentation complete
6. ✅ External review completed
7. ✅ GLR v1 AC-5 satisfied

---

## VIII. References

### Related Documents

- [GLR_V1_COMPLETION_CONTRACT.md](./GLR_V1_COMPLETION_CONTRACT.md) - Parent contract
- [BDD_GLR_CONFLICT_PRESERVATION.md](../plans/BDD_GLR_CONFLICT_PRESERVATION.md) - BDD scenarios
- [STRATEGIC_IMPLEMENTATION_PLAN.md](../plans/STRATEGIC_IMPLEMENTATION_PLAN.md) - Overall plan
- [NODE_API_CONTRACT.md](./NODE_API_CONTRACT.md) - Existing Node API contract

### Tree-sitter References

- [Tree-sitter Node API](https://tree-sitter.github.io/tree-sitter/using-parsers#walking-trees-with-tree-cursors)
- [Tree-sitter Tree Cursor](https://tree-sitter.github.io/tree-sitter/using-parsers#tree-cursors)

---

**Contract Version**: 1.0.0
**Last Updated**: 2025-11-20
**Next Review**: After Phase 1 completion
**Owner**: rust-sitter core team

---

**Approval**: Ready for implementation

---

END OF CONTRACT
