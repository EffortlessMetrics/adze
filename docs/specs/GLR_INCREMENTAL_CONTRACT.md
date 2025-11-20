# GLR Incremental Parsing Contract

**Version**: 1.0.0
**Date**: 2025-11-20
**Status**: 📋 **PLANNED** (Phase II - Weeks 5-8)
**Predecessor**: [GLR_V1_COMPLETION_CONTRACT.md](./GLR_V1_COMPLETION_CONTRACT.md) ✅ COMPLETE
**Target**: v0.9.0 - Production-ready incremental parsing with forest API
**Strategic Context**: [STRATEGIC_IMPLEMENTATION_PLAN.md Phase II](../plans/STRATEGIC_IMPLEMENTATION_PLAN.md)

---

## Executive Summary

This contract defines the complete specification for **GLR Incremental Parsing v1**, building on the production-ready GLR v1 foundation to deliver **editor-class performance** for typical edit workloads.

**Goal**: Enable rust-sitter to compete directly with Tree-sitter for **interactive** use cases (LSPs, editors, live analysis) by providing efficient incremental reparsing and programmatic access to parse ambiguities.

**Success Criteria**:
- ≤30% of full parse cost for single-line edits
- 100% correctness (incremental = full parse results)
- Forest API enables ambiguity inspection
- Production-ready documentation and tooling

---

## I. Strategic Context

### Market Position Evolution

**Current (Post GLR v1)**:
> "A production-ready GLR parser in Rust with strong infra and contracts; ideal for compilers/tools that can afford full parses, less ideal for interactive/editor workloads."

**After Incremental v1**:
> "A Rust-native alternative to Tree-sitter for teams that want *both* high-end parsing (GLR) and first-class infra (Nix, CI, contracts), now with editor-class performance."

### Competitive Gaps Addressed

| Capability | Tree-sitter | rust-sitter (GLR v1) | rust-sitter (After Incremental) |
|------------|-------------|----------------------|----------------------------------|
| **Incremental Parsing** | ✅ Tight optimization | ❌ Full parse only | ✅ <30% cost for edits |
| **Forest / Ambiguity API** | ❌ Single tree only | ⚠️ Internal only | ✅ Public API |
| **GLR Support** | ❌ No | ✅ Production-ready | ✅ Production-ready |
| **Infra as Code** | ⚠️ Basic | ✅ Nix + contracts | ✅ Nix + contracts |
| **Editor Performance** | ✅ Excellent | ❌ Full parse | ✅ Competitive |

---

## II. Scope Definition

### In Scope for Incremental v1

1. **Incremental API** (AC-I1)
   - `Tree::edit(&mut self, edit: &Edit)` - mark affected regions
   - `Parser::parse_incremental(input, old_tree)` - reuse clean subtrees
   - Edit model matching Tree-sitter semantics
   - Stable node IDs for tracking across edits

2. **Incremental Engine** (AC-I2, AC-I3)
   - Dirty region detection (lowest common ancestor)
   - Local reparse window strategy (N tokens/lines padding)
   - Boundary stitching (anchor points at stable contexts)
   - Fallback to full parse (>X% of file changed)
   - Performance within 30% of full parse for typical edits

3. **Forest API v1** (AC-I4)
   - Feature-gated `ForestHandle` for ambiguity inspection
   - Ambiguity count reporting
   - Basic forest traversal (root alternatives, children, kind)
   - Debug visualization (Graphviz export)

4. **Observability & Metrics** (AC-I5)
   - Parse mode tracking (incremental vs full)
   - Reuse percentage metrics
   - Performance instrumentation
   - CI regression gates (30% threshold)

5. **Documentation**
   - Architecture document (INCREMENTAL_GLR_ARCHITECTURE.md)
   - User guide (incremental parsing section)
   - Migration guide (v0.6 → v0.9)
   - Forest API cookbook (debugging ambiguities)

### Out of Scope for Incremental v1

1. **Perfect GLR Incrementality** - Aim for pragmatic, sound under-approximation
2. **Query System** - Deferred to v0.10 (builds on incremental)
3. **Full SPPF API** - Minimal forest API only; full API is v0.11+
4. **Cross-File Incremental** - Single-file edits only
5. **Advanced Reuse Strategies** - Content-based hashing, persistent data structures deferred

---

## III. Acceptance Criteria

### AC-I1: API Surface

**Requirement**: Stable, documented API for incremental parsing and tree editing.

**Success Criteria**:
1. `Edit` struct matches Tree-sitter semantics
2. `Tree::edit(&mut self, edit: &Edit)` updates byte ranges and marks dirty subtrees
3. `Parser::parse_incremental(input, Option<&Tree>)` reuses clean subtrees when possible
4. Stable node IDs survive edits (within clean regions)
5. API documented with examples in rustdoc

**BDD Scenario**:
```gherkin
Scenario: Edit a tree and reparse incrementally
  Given a parsed tree for "let x = 1 + 2;"
  When I edit byte range 8..9 (change "1" to "10")
  And I call tree.edit(edit)
  And I call parser.parse_incremental(new_input, Some(&tree))
  Then the new tree reflects "let x = 10 + 2;"
  And nodes outside the edit are reused from old tree
  And stable node IDs match for unchanged nodes
```

**Implementation Location**:
- API: `runtime2/src/parser.rs`, `runtime2/src/tree.rs`
- Edit model: `runtime2/src/edit.rs` (new)
- Tests: `runtime2/tests/test_incremental_api.rs`

**Deliverables**:
- [ ] `Edit` struct with byte and position fields
- [ ] `Tree::edit` implementation (byte range updates)
- [ ] `Parser::parse_incremental` signature and routing
- [ ] Stable node ID system (IDs or structural anchors)
- [ ] API documentation with 5+ examples

---

### AC-I2: Correctness

**Requirement**: Incremental parsing must produce identical results to full parsing.

**Success Criteria**:
1. Golden test suite: incremental == full parse for all test inputs
2. Property-based tests: any edit sequence produces correct tree
3. Ambiguity preservation: GLR conflicts handled correctly in incremental mode
4. No data loss: all information from old tree preserved or correctly invalidated
5. Edge case coverage: empty edits, deletions, insertions, replacements

**BDD Scenario**:
```gherkin
Scenario: Incremental parse equals full parse
  Given a grammar and test input corpus (100+ files)
  When I parse each file fully to get baseline tree
  And I apply a random edit to each file
  And I parse incrementally with old tree
  And I parse fully from scratch
  Then incremental tree == full tree (structural equality)
  And all node properties match (kind, byte ranges, text)
  And parent/child relationships are consistent
```

**Implementation Location**:
- Core logic: `glr-core/src/incremental.rs` (new)
- Dirty tracking: `runtime2/src/tree.rs` (extend)
- Tests: `runtime2/tests/test_incremental_correctness.rs`

**Test Strategy**:
```rust
// Golden tests
#[test]
fn incremental_equals_full_parse() {
    for test_case in TEST_CORPUS {
        let full_tree1 = parse_full(test_case.input);

        // Apply edit
        let edited_input = apply_edit(test_case.input, test_case.edit);

        // Incremental parse
        full_tree1.edit(test_case.edit);
        let inc_tree = parse_incremental(edited_input, Some(&full_tree1));

        // Full parse from scratch
        let full_tree2 = parse_full(edited_input);

        assert_trees_equal(inc_tree, full_tree2);
    }
}

// Property-based tests
#[quickcheck]
fn incremental_always_correct(input: String, edits: Vec<Edit>) {
    let mut tree = parse_full(&input);
    let mut current_input = input.clone();

    for edit in edits {
        current_input = apply_edit(&current_input, &edit);
        tree.edit(&edit);
        let inc_tree = parse_incremental(&current_input, Some(&tree));
        let full_tree = parse_full(&current_input);
        assert_trees_equal(inc_tree, full_tree);
        tree = inc_tree;
    }
}
```

**Deliverables**:
- [ ] Golden test suite (100+ test cases)
- [ ] Property-based test suite (quickcheck)
- [ ] Corpus testing (Python, JavaScript, Rust grammars)
- [ ] Edge case tests (empty, boundary, large edits)
- [ ] CI integration (all correctness tests in PR gate)

---

### AC-I3: Performance

**Requirement**: Incremental parsing must be significantly faster than full parsing for typical edits.

**Success Criteria**:
1. Single-line edit: ≤30% of full parse cost
2. Multi-line edit (≤10 lines): ≤50% of full parse cost
3. Large edit (>50% of file): automatic fallback to full parse
4. Reuse percentage: ≥70% for single-line edits
5. No pathological cases without fallback

**BDD Scenario**:
```gherkin
Scenario: Single-line edit is fast
  Given a 1000-line Python file
  When I parse it fully (baseline: T_full)
  And I change one character on line 500
  And I parse incrementally
  Then parse time < 0.3 × T_full
  And reuse percentage > 70%
  And no full parse fallback triggered
```

**Performance Model**:
```
Edit Size          Target         Strategy
─────────────────────────────────────────────────────
1 line            ≤30% of full   Local reparse window
2-10 lines        ≤50% of full   Local reparse window
11-50% of file    ≤80% of full   Large window or full
>50% of file      Full parse     Automatic fallback
```

**Implementation Location**:
- Reparse window: `glr-core/src/incremental.rs`
- Metrics: `runtime2/src/metrics.rs` (new)
- Benchmarks: `benchmarks/benches/incremental.rs` (new)

**Benchmark Suite**:
```rust
// Criterion benchmarks
fn bench_incremental_edits(c: &mut Criterion) {
    let mut group = c.benchmark_group("incremental");

    for (name, grammar, file) in BENCHMARK_FILES {
        // Baseline: full parse
        group.bench_function(format!("{}/full", name), |b| {
            b.iter(|| parse_full(file))
        });

        // Single-line edit
        group.bench_function(format!("{}/edit_1line", name), |b| {
            let tree = parse_full(file);
            let edit = make_single_line_edit(file);
            b.iter(|| {
                let mut t = tree.clone();
                t.edit(&edit);
                parse_incremental(file, Some(&t))
            })
        });

        // Multi-line edit
        group.bench_function(format!("{}/edit_10line", name), |b| {
            let tree = parse_full(file);
            let edit = make_multi_line_edit(file, 10);
            b.iter(|| {
                let mut t = tree.clone();
                t.edit(&edit);
                parse_incremental(file, Some(&t))
            })
        });
    }
}
```

**CI Performance Gates**:
```yaml
# .github/workflows/performance.yml
- name: Incremental performance regression check
  run: |
    cargo bench --bench incremental -- --save-baseline pr
    cargo bench --bench incremental -- --baseline main --save-baseline main

    # Fail if incremental edits regress >5%
    ./scripts/check-perf-regression.sh incremental 5
```

**Deliverables**:
- [ ] Benchmark suite (3 grammars × 3 edit sizes)
- [ ] Performance baseline (INCREMENTAL_PERFORMANCE_BASELINE.md)
- [ ] CI regression gates (5% threshold)
- [ ] Reuse percentage metrics
- [ ] Fallback triggering metrics

---

### AC-I4: Forest API v1

**Requirement**: Programmatic access to parse ambiguities for debugging and analysis.

**Success Criteria**:
1. `ForestHandle` exposes ambiguity count
2. Root alternatives traversal available
3. Forest node children, kind, byte range accessible
4. Feature-gated (`feature = "forest-api"`)
5. Debug visualization (Graphviz export)

**BDD Scenario**:
```gherkin
Scenario: Inspect ambiguous parse
  Given the dangling-else grammar
  When I parse "if a then if b then s1 else s2"
  Then parse result reports 2 ambiguities
  And forest_handle.root_alternatives() returns 2 nodes
  And I can traverse both parse trees independently
  And I can export forest as Graphviz for debugging
```

**API Design**:
```rust
/// Feature-gated forest API for ambiguity inspection
#[cfg(feature = "forest-api")]
pub struct ForestHandle {
    nodes: Arena<ForestNode>,
    root_alternatives: Vec<ForestNodeId>,
}

#[cfg(feature = "forest-api")]
pub struct ParseResult {
    /// The default tree (disambiguated via precedence)
    pub tree: Tree,
    /// Optional forest handle (if ambiguities exist)
    pub forest: Option<ForestHandle>,
    /// Number of ambiguous regions
    pub ambiguities: usize,
}

#[cfg(feature = "forest-api")]
impl ForestHandle {
    /// Get all alternative parse trees at root
    pub fn root_alternatives(&self) -> impl Iterator<Item = ForestNodeId> + '_;

    /// Get children of a forest node (may include alternatives)
    pub fn children(&self, id: ForestNodeId) -> &[ForestNodeId];

    /// Get symbol kind for a node
    pub fn kind(&self, id: ForestNodeId) -> SymbolId;

    /// Get byte range for a node
    pub fn byte_range(&self, id: ForestNodeId) -> Range<usize>;

    /// Export forest as Graphviz DOT format
    pub fn to_graphviz(&self) -> String;

    /// Resolve a specific alternative to a Tree
    pub fn resolve_alternative(&self, root_id: ForestNodeId) -> Tree;
}
```

**Implementation Location**:
- Forest handle: `runtime2/src/forest.rs` (new)
- Graphviz export: `runtime2/src/visualization.rs` (new)
- Tests: `runtime2/tests/test_forest_api.rs`

**Test Coverage**:
```rust
#[test]
#[cfg(feature = "forest-api")]
fn forest_api_basic() {
    let grammar = load_dangling_else_grammar();
    let input = "if a then if b then s1 else s2";

    let result = parse_with_forest(grammar, input);

    assert_eq!(result.ambiguities, 2);
    assert!(result.forest.is_some());

    let forest = result.forest.unwrap();
    let alternatives: Vec<_> = forest.root_alternatives().collect();
    assert_eq!(alternatives.len(), 2);

    // Can resolve each alternative to a distinct tree
    let tree1 = forest.resolve_alternative(alternatives[0]);
    let tree2 = forest.resolve_alternative(alternatives[1]);
    assert_ne!(tree1.root_node().to_sexp(), tree2.root_node().to_sexp());
}

#[test]
#[cfg(feature = "forest-api")]
fn forest_graphviz_export() {
    let grammar = load_arithmetic_grammar();
    let input = "1 + 2 * 3"; // No ambiguity with precedence

    let result = parse_with_forest(grammar, input);
    assert_eq!(result.ambiguities, 0);

    // Forest exists but has single path
    let forest = result.forest.unwrap();
    let dot = forest.to_graphviz();
    assert!(dot.contains("digraph"));
    assert!(dot.contains("->"));
}
```

**Deliverables**:
- [ ] `ForestHandle` API implementation
- [ ] Graphviz export functionality
- [ ] Forest API test suite (10+ tests)
- [ ] Forest API cookbook (docs/guides/FOREST_API_COOKBOOK.md)
- [ ] Example: debugging ambiguity with forest API

---

### AC-I5: Observability & Documentation

**Requirement**: Complete documentation and instrumentation for production use.

**Success Criteria**:
1. Metrics emitted for incremental vs full parse
2. Reuse percentage tracking
3. Fallback triggering logged
4. Architecture document complete
5. User guide section complete

**Metrics Model**:
```rust
pub struct IncrementalMetrics {
    pub parse_mode: ParseMode, // Incremental | Full | Fallback
    pub reuse_percentage: f32,  // 0.0 - 100.0
    pub dirty_nodes: usize,
    pub clean_nodes_reused: usize,
    pub reparse_window_bytes: usize,
    pub parse_time_ms: f32,
}

pub enum ParseMode {
    Full,          // No old tree provided
    Incremental,   // Successfully reused subtrees
    Fallback,      // Edit too large, fell back to full parse
}
```

**Logging Strategy**:
```rust
// Performance logging (behind env var)
if std::env::var("RUST_SITTER_LOG_INCREMENTAL").is_ok() {
    eprintln!("Incremental parse metrics:");
    eprintln!("  Mode: {:?}", metrics.parse_mode);
    eprintln!("  Reuse: {:.1}%", metrics.reuse_percentage);
    eprintln!("  Time: {:.2}ms", metrics.parse_time_ms);
}

// Fallback warning
if matches!(metrics.parse_mode, ParseMode::Fallback) {
    warn!("Incremental parse fell back to full parse (edit size: {} bytes)",
          edit_size);
}
```

**Documentation Deliverables**:

1. **docs/architecture/INCREMENTAL_GLR_ARCHITECTURE.md** (500+ lines)
   - Design overview (dirty regions, reparse window, stitching)
   - Performance model (complexity analysis)
   - Trade-offs (pragmatic vs perfect incrementality)
   - Implementation details (data structures, algorithms)

2. **docs/guides/INCREMENTAL_PARSING_USER_GUIDE.md** (400+ lines)
   - When to use incremental parsing
   - How to use the API (examples)
   - Performance tuning (window size, fallback threshold)
   - Debugging tips (metrics, logging)

3. **docs/guides/FOREST_API_COOKBOOK.md** (300+ lines)
   - What is a parse forest?
   - When to use forest API
   - Debugging ambiguities
   - Alternative tree selection
   - Graphviz visualization

4. **Migration Guide**: Section in existing user guide
   - v0.6 → v0.9 API changes
   - Opt-in to incremental parsing
   - Feature flag considerations

**CI Observability**:
```yaml
# .github/workflows/incremental-metrics.yml
- name: Track incremental parse metrics
  run: |
    RUST_SITTER_LOG_INCREMENTAL=1 cargo test --features incremental > metrics.log
    ./scripts/analyze-incremental-metrics.sh metrics.log
    # Upload to GitHub Pages dashboard
```

**Deliverables**:
- [ ] Metrics infrastructure (`runtime2/src/metrics.rs`)
- [ ] Performance logging (env var gated)
- [ ] Architecture document (INCREMENTAL_GLR_ARCHITECTURE.md)
- [ ] User guide (INCREMENTAL_PARSING_USER_GUIDE.md)
- [ ] Forest API cookbook (FOREST_API_COOKBOOK.md)
- [ ] Migration guide section
- [ ] CI metrics dashboard

---

## IV. Implementation Plan

### Phase I: Foundations (Weeks 5-6)

**Goal**: Establish incremental API and metadata infrastructure

**Tasks**:

1. **Edit Model** (Week 5, Days 1-2)
   - [ ] Define `Edit` struct (byte + position fields)
   - [ ] Implement `Tree::edit` (byte range updates)
   - [ ] Add dirty subtree marking
   - [ ] Unit tests for edit operations

2. **Stable Node IDs** (Week 5, Days 3-4)
   - [ ] Design stable ID system (structural anchors)
   - [ ] Implement ID assignment during tree building
   - [ ] Add ID lookup by (id, byte_offset)
   - [ ] Test ID stability across edits

3. **Incremental Metadata** (Week 5, Day 5)
   - [ ] Add parent links to TreeNode
   - [ ] Add dirty flags to subtrees
   - [ ] Extend arena for metadata storage
   - [ ] Test metadata integrity

4. **API Skeleton** (Week 6, Days 1-2)
   - [ ] `Parser::parse_incremental` signature
   - [ ] Route to full parse initially (no actual incrementality)
   - [ ] Add feature flag `incremental`
   - [ ] Write API documentation

5. **BDD Specs & Golden Tests** (Week 6, Days 3-5)
   - [ ] Write BDD scenarios (10+ scenarios)
   - [ ] Create golden test suite (100+ cases)
   - [ ] Set up incremental == full equality tests
   - [ ] CI integration (tests in PR gate)

**Deliverables**:
- [ ] `runtime2/src/edit.rs` (Edit model)
- [ ] `runtime2/src/tree.rs` (extended with IDs, dirty flags)
- [ ] `runtime2/src/parser.rs` (parse_incremental skeleton)
- [ ] `runtime2/tests/test_incremental_api.rs` (API tests)
- [ ] `runtime2/tests/test_incremental_correctness.rs` (golden tests)
- [ ] BDD scenarios document

**Success Criteria**: AC-I1 skeleton complete, golden tests failing (incremental calls full parse)

---

### Phase II: Incremental Engine (Weeks 7-8)

**Goal**: Implement actual incremental parsing with reuse

**Tasks**:

1. **Dirty Region Detection** (Week 7, Days 1-2)
   - [ ] Implement lowest common ancestor (LCA) finding
   - [ ] Mark subtrees as dirty based on edit range
   - [ ] Test dirty region boundary cases
   - [ ] Measure dirty region sizes

2. **Local Reparse Window** (Week 7, Days 3-5)
   - [ ] Define window expansion strategy (N tokens padding)
   - [ ] Implement reparse window calculation
   - [ ] Make window size configurable
   - [ ] Test window expansion edge cases

3. **Boundary Stitching** (Week 8, Days 1-3)
   - [ ] Find stable anchor points (unambiguous contexts)
   - [ ] Implement subtree stitching logic
   - [ ] Handle parent/child link updates
   - [ ] Test stitching correctness

4. **Fallback Logic** (Week 8, Day 4)
   - [ ] Implement fallback threshold (>X% changed)
   - [ ] Add fallback metrics
   - [ ] Test fallback triggering
   - [ ] Document fallback behavior

5. **Performance Optimization** (Week 8, Day 5)
   - [ ] Add reuse percentage tracking
   - [ ] Optimize hot paths (profiling)
   - [ ] Add performance benchmarks
   - [ ] CI performance gates (30% threshold)

**Deliverables**:
- [ ] `glr-core/src/incremental.rs` (core engine)
- [ ] `runtime2/src/reparse.rs` (window + stitching)
- [ ] `runtime2/src/metrics.rs` (observability)
- [ ] `benchmarks/benches/incremental.rs` (benchmarks)
- [ ] Performance baseline document

**Success Criteria**: AC-I2 and AC-I3 complete, golden tests passing, performance within target

---

### Phase III: Forest API v1 (Weeks 9-10)

**Goal**: Expose parse ambiguities for debugging and analysis

**Tasks**:

1. **Forest Handle Wrapper** (Week 9, Days 1-2)
   - [ ] Design `ForestHandle` API
   - [ ] Wrap internal forest structure
   - [ ] Add feature flag `forest-api`
   - [ ] Basic traversal methods

2. **Ambiguity Introspection** (Week 9, Days 3-4)
   - [ ] Implement `root_alternatives()`
   - [ ] Implement `children()`, `kind()`, `byte_range()`
   - [ ] Add ambiguity counting
   - [ ] Test with dangling-else grammar

3. **Graphviz Export** (Week 9, Day 5)
   - [ ] Implement `to_graphviz()`
   - [ ] Add node labeling
   - [ ] Test with various grammars
   - [ ] Example: visualize ambiguity

4. **Alternative Resolution** (Week 10, Days 1-2)
   - [ ] Implement `resolve_alternative()`
   - [ ] Test converting forest node to Tree
   - [ ] Ensure tree API compatibility
   - [ ] Document use cases

5. **Forest API Documentation** (Week 10, Days 3-5)
   - [ ] Write Forest API cookbook
   - [ ] Create debugging examples
   - [ ] Add architecture section on forests
   - [ ] User guide integration

**Deliverables**:
- [ ] `runtime2/src/forest.rs` (ForestHandle)
- [ ] `runtime2/src/visualization.rs` (Graphviz)
- [ ] `runtime2/tests/test_forest_api.rs` (tests)
- [ ] `docs/guides/FOREST_API_COOKBOOK.md` (guide)
- [ ] Example: debugging ambiguity

**Success Criteria**: AC-I4 complete, forest API usable for debugging

---

### Phase IV: Documentation & Polish (Week 11)

**Goal**: Complete documentation and prepare for release

**Tasks**:

1. **Architecture Document** (Week 11, Days 1-2)
   - [ ] Write INCREMENTAL_GLR_ARCHITECTURE.md
   - [ ] Design overview section
   - [ ] Performance model section
   - [ ] Implementation details section
   - [ ] Trade-offs and future work section

2. **User Guide** (Week 11, Days 3-4)
   - [ ] Write INCREMENTAL_PARSING_USER_GUIDE.md
   - [ ] API usage examples
   - [ ] Performance tuning section
   - [ ] Migration guide integration
   - [ ] External review

3. **Release Preparation** (Week 11, Day 5)
   - [ ] Update CHANGELOG.md
   - [ ] Update ROADMAP.md
   - [ ] Release notes draft
   - [ ] Version bump to v0.9.0-alpha

**Deliverables**:
- [ ] `docs/architecture/INCREMENTAL_GLR_ARCHITECTURE.md`
- [ ] `docs/guides/INCREMENTAL_PARSING_USER_GUIDE.md`
- [ ] `docs/guides/FOREST_API_COOKBOOK.md`
- [ ] Migration guide section
- [ ] Release notes (v0.9.0)

**Success Criteria**: AC-I5 complete, documentation reviewed, ready for alpha release

---

## V. Test Strategy

### Test Pyramid

```
          /\
         /  \
        /E2E \         10 tests  - Full editor workflows
       /------\
      / INTEG \        30 tests  - Incremental + forest integration
     /----------\
    /   UNIT     \     60 tests  - Dirty tracking, window, stitching
   /--------------\
```

### Test Categories

#### 1. Unit Tests (60 tests minimum)

**Edit Operations** (`runtime2/tests/edit.rs`):
- [x] Placeholder (replace with actual tests)
- [ ] Edit updates byte ranges correctly
- [ ] Edit updates positions correctly
- [ ] Edit marks affected subtrees dirty
- [ ] Edit preserves stable node IDs in clean regions
- [ ] Empty edit (no-op)
- [ ] Insertion at start, middle, end
- [ ] Deletion at start, middle, end
- [ ] Replacement (delete + insert)

**Dirty Region Detection** (`glr-core/tests/dirty_regions.rs`):
- [ ] Find LCA for single-node edit
- [ ] Find LCA for multi-node edit
- [ ] Mark all descendants as dirty
- [ ] Don't mark siblings as dirty
- [ ] Handle edits at root
- [ ] Handle edits at leaf

**Reparse Window** (`glr-core/tests/reparse_window.rs`):
- [ ] Calculate window with N token padding
- [ ] Expand window to stable boundaries
- [ ] Handle window at start of file
- [ ] Handle window at end of file
- [ ] Window covers entire edit region
- [ ] Window doesn't exceed file bounds

**Boundary Stitching** (`runtime2/tests/stitching.rs`):
- [ ] Find stable anchor points
- [ ] Stitch new subtree at anchors
- [ ] Update parent/child links
- [ ] Preserve node IDs outside stitched region
- [ ] Handle stitching at root
- [ ] Handle stitching at leaf

**Forest API** (`runtime2/tests/forest_api.rs`):
- [ ] Ambiguity count correct
- [ ] Root alternatives traversal
- [ ] Forest node children
- [ ] Forest node kind, byte range
- [ ] Graphviz export produces valid DOT
- [ ] Resolve alternative to Tree

#### 2. Integration Tests (30 tests minimum)

**Incremental Correctness** (`runtime2/tests/incremental_correctness.rs`):
- [ ] Single-line edit: incremental == full
- [ ] Multi-line edit: incremental == full
- [ ] Large edit: incremental == full (or fallback)
- [ ] Sequence of edits: cumulative correctness
- [ ] Edit + reparse + edit: correctness
- [ ] GLR ambiguity preserved incrementally

**Performance Tests** (`benchmarks/benches/incremental.rs`):
- [ ] Single-line edit <30% of full parse
- [ ] Multi-line edit <50% of full parse
- [ ] Large edit triggers fallback
- [ ] Reuse percentage >70% for small edits
- [ ] No pathological cases without fallback

**Grammar Integration** (`runtime2/tests/incremental_grammars.rs`):
- [ ] Arithmetic grammar incremental
- [ ] Dangling-else grammar incremental
- [ ] Python grammar incremental
- [ ] JavaScript grammar incremental
- [ ] Rust grammar incremental

#### 3. Property-Based Tests (Continuous)

**Quickcheck** (`runtime2/tests/incremental_properties.rs`):
```rust
#[quickcheck]
fn incremental_equals_full(input: String, edits: Vec<Edit>) -> bool {
    // For any input and any sequence of edits,
    // incremental parse == full parse
}

#[quickcheck]
fn stable_ids_survive_clean_regions(input: String, edit: Edit) -> bool {
    // Node IDs in clean regions don't change after edit
}

#[quickcheck]
fn reuse_percentage_bounded(input: String, edit: Edit) -> bool {
    // Reuse percentage is 0-100%
}
```

#### 4. End-to-End Tests (10 tests minimum)

**Editor Workflows** (`runtime2/tests/e2e_incremental.rs`):
- [ ] Type a single character
- [ ] Delete a single character
- [ ] Insert a new line
- [ ] Delete a line
- [ ] Paste a block of text
- [ ] Undo an edit (re-apply old tree)
- [ ] Rapid typing (sequence of small edits)
- [ ] Large refactor (50% of file changes)

---

## VI. Performance Model

### Complexity Analysis

**Full Parse**:
- Time: O(n) for deterministic grammars, O(n³) for highly ambiguous
- Space: O(n) for parse tree

**Incremental Parse**:
- Best case (small edit): O(w) where w = reparse window size (w << n)
- Average case (typical edit): O(w + log n) (w for reparse, log n for finding dirty regions)
- Worst case (large edit): O(n) (fallback to full parse)

### Performance Targets

| Edit Size | Target Time | Target Reuse | Strategy |
|-----------|-------------|--------------|----------|
| 1 line    | ≤30% of full | ≥70% | Local window |
| 2-10 lines | ≤50% of full | ≥50% | Local window |
| 11-50% file | ≤80% of full | ≥20% | Large window or full |
| >50% file | Full parse | 0% | Automatic fallback |

### Benchmark Grammars

Test with 3 representative grammars:

1. **Python** (complex, external scanner)
   - File sizes: 100 LOC, 1K LOC, 10K LOC
   - Edit patterns: single char, single line, multi-line, large refactor

2. **JavaScript/TypeScript** (JSX, ambiguity)
   - File sizes: 100 LOC, 1K LOC, 5K LOC
   - Edit patterns: JSX edits, type annotations, imports

3. **Rust** (macros, generics)
   - File sizes: 100 LOC, 1K LOC, 10K LOC
   - Edit patterns: macro edits, function bodies, trait impls

---

## VII. Risk Management

### High Risks

| Risk | Impact | Likelihood | Mitigation |
|------|--------|------------|------------|
| Correctness bugs in incremental | CRITICAL | MEDIUM | Extensive golden tests, property testing |
| Performance targets not met | HIGH | MEDIUM | Early benchmarking, fallback mechanism |
| Stable ID design complexity | HIGH | MEDIUM | Simple structural anchors initially |
| Forest API leaking internals | MEDIUM | LOW | Feature-gated, minimal API surface |

### Mitigation Strategies

**Correctness**:
- Golden test suite comparing incremental vs full (100+ cases)
- Property-based testing with quickcheck
- Corpus testing with real-world grammars
- Fuzz testing with random edits

**Performance**:
- Early benchmarking (Phase II Week 1)
- Fallback mechanism for large edits
- Configurable reparse window size
- CI regression gates (5% threshold)

**Complexity**:
- Start with simplest viable design
- Defer optimizations (content hashing) to v0.10+
- Clear ADR documenting design decisions
- Incremental approach: MVP first, optimize later

---

## VIII. Success Metrics

### Quantitative

| Metric | Target | Measurement |
|--------|--------|-------------|
| **Correctness** | 100% (incremental == full) | Golden tests, property tests |
| **Performance (1 line)** | ≤30% of full parse | Criterion benchmarks |
| **Performance (10 lines)** | ≤50% of full parse | Criterion benchmarks |
| **Reuse (small edits)** | ≥70% | Metrics tracking |
| **Test Coverage** | 100 tests (60 unit + 30 integration + 10 E2E) | CI reports |
| **Documentation** | 1,200+ lines (3 docs) | Line count |

### Qualitative

| Metric | Success Criteria |
|--------|------------------|
| **API Usability** | External reviewer can use incremental API without confusion |
| **Performance Feel** | Editor-like responsiveness for typical edits |
| **Forest API Utility** | Can debug ambiguities without reading GLR internals |
| **Documentation Clarity** | >4.5/5 from external review |

---

## IX. Definition of Done

Incremental v1 is **DONE** when:

1. ✅ All acceptance criteria (AC-I1 through AC-I5) met
2. ✅ All tests passing (100% pass rate, 100+ tests)
3. ✅ Performance targets met (benchmarked and documented)
4. ✅ Documentation complete and externally reviewed
5. ✅ API stable and feature-gated
6. ✅ CI gates in place (correctness + performance)
7. ✅ Migration guide written
8. ✅ Release notes complete
9. ✅ Tagged as `v0.9.0-alpha` or merged to main

---

## X. Current Status

**Status**: 📋 **PLANNED** (not yet started)
**Predecessor**: GLR v1 ✅ COMPLETE
**Target Start**: After Phase 1B (Policy-as-Code) complete
**Estimated Duration**: 11 weeks (Phases I-IV)

**Blockers**: None (GLR v1 provides all necessary foundation)

**Readiness**:
- [x] GLR v1 production-ready ✅
- [x] Tree API 100% compatible ✅
- [x] Performance baseline established ✅
- [x] Infrastructure in place (Nix, CI, docs) ✅
- [ ] Phase 1B (Policy-as-Code) complete ⏳

---

## XI. References

### Related Contracts and Specifications

- [GLR_V1_COMPLETION_CONTRACT.md](./GLR_V1_COMPLETION_CONTRACT.md) ✅ COMPLETE
- [TREE_API_COMPATIBILITY_CONTRACT.md](./TREE_API_COMPATIBILITY_CONTRACT.md) ✅ COMPLETE
- [NIX_CI_INTEGRATION_CONTRACT.md](./NIX_CI_INTEGRATION_CONTRACT.md) ⏳ 80% COMPLETE

### Architecture Decision Records

- [ADR-0009: Incremental Parsing Architecture](../adr/ADR-0009-INCREMENTAL-PARSING-ARCHITECTURE.md) 📋 TO BE CREATED
- [ADR-0010: Forest API Design](../adr/ADR-0010-FOREST-API-DESIGN.md) 📋 TO BE CREATED

### Strategic Planning

- [STRATEGIC_IMPLEMENTATION_PLAN.md](../plans/STRATEGIC_IMPLEMENTATION_PLAN.md) - Phase II: Incremental GLR
- [ROADMAP.md](../../ROADMAP.md) - v0.9.0 target

### External References

- [Tree-sitter Edit API](https://tree-sitter.github.io/tree-sitter/using-parsers#editing)
- [Incremental Parsing Techniques (Paper)](https://dl.acm.org/doi/10.1145/800193.569983)
- [GLR Parsing (Wikipedia)](https://en.wikipedia.org/wiki/GLR_parser)

---

**Contract Version**: 1.0.0
**Last Updated**: 2025-11-20
**Next Review**: After Phase 1B completion
**Owner**: rust-sitter core team

---

**Signatures** (for contract acceptance):

- [ ] Technical Lead: _______________ Date: ___________
- [ ] Quality Assurance: _______________ Date: ___________
- [ ] Documentation Lead: _______________ Date: ___________
- [ ] Performance Lead: _______________ Date: ___________

---

END OF CONTRACT
