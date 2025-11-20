# ADR-0009: Incremental Parsing Architecture

**Status**: Proposed
**Date**: 2025-11-20
**Context**: Incremental GLR parsing design for editor-class performance
**Related**: [GLR_INCREMENTAL_CONTRACT.md](../specs/GLR_INCREMENTAL_CONTRACT.md), ADR-0007 (Runtime2 GLR Integration)

---

## Context

### Problem Statement

rust-sitter GLR v1 is production-ready but performs **full parses** for every edit. This is unsuitable for interactive workloads (LSPs, editors) where:

1. Users expect <100ms response times for single-character edits
2. Full parses on 10K LOC files take 100ms-1s
3. 99% of edits affect <1% of the file
4. Tree-sitter C achieves sub-10ms incremental edits

**Current State**:
- Full parse every time: O(n) even for 1-byte edit
- No subtree reuse across edits
- Competitive only for batch processing

**Target State**:
- Incremental parse: O(w) where w << n (reparse window)
- Subtree reuse: ≥70% for typical edits
- ≤30% of full parse cost for single-line edits

---

## Decision

We will implement **pragmatic incremental GLR parsing** using:

1. **Local Reparse Window** strategy (not perfect incrementality)
2. **Dirty Region Tracking** at subtree granularity
3. **Boundary Stitching** at stable grammar contexts
4. **Automatic Fallback** for large edits
5. **Feature-Gated Forest API** for ambiguity inspection

This is a **sound under-approximation**: we may reparse more than strictly necessary, but we never produce incorrect results.

---

## Design Overview

### Core Strategy: Local Reparse Window

**Philosophy**: Reparse a **bounded region** around the edit, reuse everything else.

```
Original tree:
[--clean--][----dirty----][--clean--]
            ^             ^
         edit start    edit end

Reparse window:
[--clean--][-PAD-][dirty][-PAD-][--clean--]
            <-w->         <-w->
          (N tokens)    (N tokens)

Strategy:
1. Mark dirty region (LCA of affected nodes)
2. Expand by N tokens on both sides
3. Find stable anchor points at window boundaries
4. Reparse window with GLR engine
5. Stitch new subtree between anchors
6. Reuse clean subtrees
```

**Why This Works**:
- Most edits are localized (function body, single statement)
- Grammar contexts are often unambiguous at function/statement boundaries
- Padding ensures we capture context for disambiguation
- Fallback handles pathological cases

---

## Architecture Components

### 1. Edit Model

```rust
/// Tree-sitter compatible edit representation
pub struct Edit {
    /// Start byte offset in old text
    pub start_byte: u32,
    /// End byte offset in old text (before edit)
    pub old_end_byte: u32,
    /// End byte offset in new text (after edit)
    pub new_end_byte: u32,
    /// Start position (line, column) in old text
    pub start_position: Point,
    /// Old end position
    pub old_end_position: Point,
    /// New end position
    pub new_end_position: Point,
}

impl Tree {
    /// Update byte ranges and mark affected subtrees as dirty
    pub fn edit(&mut self, edit: &Edit) {
        // 1. Adjust byte offsets for all nodes after edit
        // 2. Find LCA of affected nodes
        // 3. Mark LCA and descendants as dirty
        // 4. Invalidate cached properties (text, position)
    }
}
```

**Design Decision**: Match Tree-sitter API exactly for drop-in compatibility.

**Alternative Considered**: Custom edit model with more metadata
- **Rejected**: Interoperability more valuable than expressiveness

---

### 2. Stable Node IDs

**Challenge**: How do we identify "same node" across edits?

**Solution**: Structural anchors (not persistent IDs)

```rust
/// Structural anchor for identifying nodes across edits
#[derive(Debug, Clone, PartialEq)]
pub struct NodeAnchor {
    /// Production rule (e.g. "function_definition")
    symbol: SymbolId,
    /// Byte offset in original input
    byte_offset: u32,
    /// Path from root (e.g. [0, 2, 1] = root.child(0).child(2).child(1))
    path: Vec<usize>,
}

impl Tree {
    /// Create anchor for a node (survives edits in clean regions)
    pub fn anchor(&self, node: Node) -> NodeAnchor;

    /// Resolve anchor to node in (possibly edited) tree
    pub fn resolve_anchor(&self, anchor: &NodeAnchor) -> Option<Node>;
}
```

**Properties**:
- Anchors in clean regions resolve to same logical node
- Anchors in dirty regions return `None` (must reparse)
- No persistent ID storage required

**Alternative Considered**: UUID-based persistent IDs
- **Rejected**: Complex to maintain, storage overhead, no clear benefit

---

### 3. Dirty Region Detection

**Algorithm**: Find Lowest Common Ancestor (LCA) of affected byte range

```rust
impl Tree {
    fn find_dirty_region(&self, edit: &Edit) -> DirtyRegion {
        // 1. Find all nodes whose byte range intersects edit
        let mut affected = Vec::new();
        self.visit_depth_first(|node| {
            if node.byte_range().intersects(edit.byte_range()) {
                affected.push(node);
            }
        });

        // 2. Find LCA of affected nodes
        let lca = self.lowest_common_ancestor(&affected);

        // 3. Mark LCA and all descendants as dirty
        DirtyRegion {
            root: lca,
            byte_range: lca.byte_range(),
            affected_nodes: affected.len(),
        }
    }
}
```

**Complexity**: O(log n) for tree traversal, O(k) for k affected nodes

---

### 4. Reparse Window Strategy

**Configuration**:
```rust
pub struct ReparseConfig {
    /// Number of tokens to pad on each side
    pub padding_tokens: usize,  // Default: 5
    /// Maximum window size as % of file
    pub max_window_percent: f32,  // Default: 20%
    /// Fallback threshold as % of file
    pub fallback_threshold: f32,  // Default: 50%
}
```

**Algorithm**:
```rust
fn calculate_reparse_window(
    tree: &Tree,
    dirty_region: &DirtyRegion,
    config: &ReparseConfig,
) -> ReparseWindow {
    // 1. Start with dirty region
    let mut window = dirty_region.byte_range();

    // 2. Expand by N tokens on each side
    window.start = find_token_boundary(
        tree,
        window.start,
        -config.padding_tokens as isize,
    );
    window.end = find_token_boundary(
        tree,
        window.end,
        config.padding_tokens as isize,
    );

    // 3. Clamp to file bounds
    window.start = window.start.max(0);
    window.end = window.end.min(tree.byte_len());

    // 4. Check if window exceeds threshold
    let window_percent = (window.end - window.start) as f32
                        / tree.byte_len() as f32 * 100.0;

    if window_percent > config.fallback_threshold {
        return ReparseWindow::Fallback; // Too large, use full parse
    }

    ReparseWindow::Local {
        byte_range: window,
        anchor_start: find_stable_anchor(tree, window.start),
        anchor_end: find_stable_anchor(tree, window.end),
    }
}
```

**Trade-offs**:
- Larger padding → more reuse, higher correctness margin
- Smaller padding → faster reparse, risk missing context
- Empirical tuning needed per grammar

---

### 5. Boundary Stitching

**Challenge**: How do we connect the newly parsed subtree to the old tree?

**Solution**: Find stable anchor points where grammar context is unambiguous

```rust
/// Find a stable anchor point near byte offset
fn find_stable_anchor(tree: &Tree, byte_offset: u32) -> Anchor {
    // Strategy: Walk up tree until we find a node at a "safe" boundary

    let mut node = tree.node_at_byte(byte_offset);

    while let Some(parent) = node.parent() {
        if is_stable_boundary(parent) {
            return Anchor {
                node: parent,
                child_index: node.index_in_parent(),
            };
        }
        node = parent;
    }

    // Fallback: root
    Anchor::root()
}

/// Check if a node represents a stable grammar boundary
fn is_stable_boundary(node: Node) -> bool {
    // Heuristics (grammar-specific):
    matches!(node.kind(),
        | "function_definition"
        | "class_definition"
        | "statement"
        | "block"
        | "module"
        // ... other stable contexts
    )
}
```

**Stitching Algorithm**:
```rust
fn stitch_subtree(
    old_tree: &Tree,
    new_subtree: Tree,
    anchor_start: &Anchor,
    anchor_end: &Anchor,
) -> Tree {
    // 1. Remove old subtree between anchors
    let mut result = old_tree.clone();
    result.remove_range(anchor_start.byte_offset()..anchor_end.byte_offset());

    // 2. Insert new subtree
    result.insert_subtree(anchor_start.byte_offset(), new_subtree);

    // 3. Update parent/child links
    result.fix_parent_links(anchor_start, anchor_end);

    // 4. Recompute cached properties (byte ranges, positions)
    result.recompute_ranges();

    result
}
```

**Correctness Guarantee**: If anchors are at stable grammar contexts (unambiguous), stitching preserves parse correctness.

---

### 6. Fallback Mechanism

**Triggers**:
1. Edit affects >50% of file → full parse
2. Window expansion exceeds 20% of file → full parse
3. Failed to find stable anchors → full parse
4. Stitching validation fails → full parse

**Implementation**:
```rust
pub enum ParseResult {
    Incremental {
        tree: Tree,
        metrics: IncrementalMetrics,
    },
    Fallback {
        tree: Tree,
        reason: FallbackReason,
    },
}

pub enum FallbackReason {
    EditTooLarge { percent: f32 },
    WindowTooLarge { percent: f32 },
    NoStableAnchors,
    ValidationFailed,
}
```

**Logging**:
```rust
if let ParseResult::Fallback { reason, .. } = result {
    warn!("Incremental parse fell back to full parse: {:?}", reason);
}
```

---

## Forest API Design

### Motivation

GLR produces a **parse forest** (SPPF - Shared Packed Parse Forest) when ambiguities exist. Today, this is internal-only. Users need visibility for:

1. **Debugging**: "Why did GLR choose this tree?"
2. **Analysis**: "What are all valid interpretations?"
3. **Tools**: Linters that check all ambiguities

### Design: Minimal Feature-Gated API

```rust
#[cfg(feature = "forest-api")]
pub struct ForestHandle {
    /// Internal forest representation (opaque)
    forest: Arc<Forest>,
}

#[cfg(feature = "forest-api")]
impl ForestHandle {
    /// Number of ambiguous regions
    pub fn ambiguity_count(&self) -> usize;

    /// Get all alternative root nodes
    pub fn root_alternatives(&self) -> impl Iterator<Item = ForestNodeId> + '_;

    /// Get children of a forest node
    pub fn children(&self, id: ForestNodeId) -> &[ForestNodeId];

    /// Get symbol kind for a node
    pub fn kind(&self, id: ForestNodeId) -> SymbolId;

    /// Get byte range for a node
    pub fn byte_range(&self, id: ForestNodeId) -> Range<usize>;

    /// Export forest as Graphviz for visualization
    pub fn to_graphviz(&self) -> String;

    /// Resolve a specific alternative to a Tree
    pub fn resolve_alternative(&self, root_id: ForestNodeId) -> Tree;
}

#[cfg(feature = "forest-api")]
pub struct ParseResult {
    /// Default tree (disambiguated)
    pub tree: Tree,
    /// Optional forest handle (if ambiguities exist)
    pub forest: Option<ForestHandle>,
    /// Ambiguity count
    pub ambiguities: usize,
}
```

**Why Feature-Gated**:
- Forest API adds complexity (Arc, internal exposure)
- Most users don't need it (default tree is sufficient)
- Allows us to stabilize core API first, iterate on forest separately

**Why Minimal**:
- Full SPPF API is complex (many node types, sharing, packing)
- Minimal API covers 90% of use cases (inspect, visualize, resolve)
- Can expand later without breaking changes

---

## Performance Model

### Complexity Analysis

**Full Parse**:
- Time: O(n) for deterministic, O(n³) for ambiguous
- Space: O(n)

**Incremental Parse**:
- Best: O(w + log n) where w = window size, log n = LCA finding
- Average: O(w + k log n) where k = dirty nodes
- Worst: O(n) (fallback)

**Expected**:
- w << n (window = 0.1% - 5% of file)
- k << n (dirty nodes = 1 - 50)
- Speedup: 3x - 10x for typical edits

### Performance Targets (from Contract)

| Edit Size | Target Time | Target Reuse | Strategy |
|-----------|-------------|--------------|----------|
| 1 line    | ≤30% of full | ≥70% | Local window |
| 2-10 lines | ≤50% of full | ≥50% | Local window |
| >50% file | Full parse | 0% | Automatic fallback |

---

## Trade-offs and Alternatives

### Alternative 1: Perfect GLR Incrementality

**Approach**: Track every GLR state, fork, and merge across edits

**Pros**:
- Theoretically optimal reuse
- Minimal reparse

**Cons**:
- Extremely complex implementation
- No clear algorithmic path (research problem)
- Risk of subtle correctness bugs
- Diminishing returns (local window already fast enough)

**Decision**: Rejected. Pragmatic local window is "good enough" and much simpler.

---

### Alternative 2: Content-Based Hashing

**Approach**: Hash subtree contents, reuse if hash matches

**Pros**:
- Can detect "edit then undo" scenarios
- Reuse even if byte offsets changed

**Cons**:
- Hash computation overhead
- Storage overhead (hash per node)
- Benefit limited (most edits don't "undo")
- Complexity for marginal gain

**Decision**: Deferred to v0.10+. Start simple, add if profiling shows need.

---

### Alternative 3: Persistent Data Structures

**Approach**: Use persistent trees (e.g. rope-based) for structural sharing

**Pros**:
- Automatic subtree sharing
- Efficient cloning

**Cons**:
- Major refactor of Tree representation
- Performance overhead for non-incremental path
- Complexity throughout codebase

**Decision**: Rejected. Too invasive for incremental benefit.

---

## Implementation Phases

### Phase I: Foundations (2 weeks)
- Edit model, Tree::edit skeleton
- Stable node anchors
- Dirty region detection
- Golden tests (incremental == full)

### Phase II: Engine (2 weeks)
- Reparse window calculation
- Boundary stitching
- Fallback logic
- Performance benchmarks

### Phase III: Forest API (2 weeks)
- ForestHandle wrapper
- Traversal methods
- Graphviz export
- Documentation

### Phase IV: Polish (1 week)
- Architecture docs
- User guide
- Performance tuning
- Release prep

**Total**: 11 weeks (including documentation)

---

## Success Criteria

Incremental v1 is successful when:

1. **Correctness**: 100% golden test pass rate (incremental == full)
2. **Performance**: <30% of full parse for single-line edits
3. **Reuse**: ≥70% for typical edits
4. **Fallback**: No pathological cases without automatic fallback
5. **Documentation**: Complete architecture + user guide + API docs

---

## Risk Mitigation

### Correctness Risks

**Risk**: Incremental produces different tree than full parse
**Mitigation**:
- Extensive golden test suite (100+ cases)
- Property-based testing (quickcheck)
- Corpus testing (real-world grammars)
- Fallback on validation failure

### Performance Risks

**Risk**: Targets not met (still >30% for small edits)
**Mitigation**:
- Early benchmarking (week 1 of Phase II)
- Profiling-guided optimization
- Fallback ensures no worse than full parse
- Configurable window size for tuning

### Complexity Risks

**Risk**: Implementation too complex, hard to maintain
**Mitigation**:
- Start with simplest viable design
- Clear separation of concerns (dirty tracking, window, stitching)
- Comprehensive documentation (ADR, architecture, comments)
- Incremental approach (MVP first, optimize later)

---

## Future Work (Out of Scope)

**v0.10+**:
- Content-based hashing for undo scenarios
- Cross-file incremental (imports, dependencies)
- Persistent data structures for structural sharing
- Advanced reuse strategies (token-based, semantic)

**Research**:
- True incremental GLR (state tracking across edits)
- Minimal reparse bounds (theoretical optimal)

---

## Metrics and Monitoring

### Development Metrics

Track during implementation:

```rust
pub struct IncrementalMetrics {
    pub parse_mode: ParseMode,      // Incremental | Full | Fallback
    pub reuse_percentage: f32,       // 0-100%
    pub dirty_nodes: usize,
    pub clean_nodes_reused: usize,
    pub reparse_window_bytes: usize,
    pub parse_time_ms: f32,
    pub fallback_reason: Option<FallbackReason>,
}
```

### CI Metrics

Monitor in CI:

1. **Correctness**: Golden test pass rate (must be 100%)
2. **Performance**: Benchmark comparison vs baseline
3. **Regressions**: Alert if >5% slowdown
4. **Coverage**: Test coverage % (target >80%)

---

## References

### Internal

- [GLR_INCREMENTAL_CONTRACT.md](../specs/GLR_INCREMENTAL_CONTRACT.md) - Full contract
- [ADR-0007: Runtime2 GLR Integration](./ADR-0007-RUNTIME2-GLR-INTEGRATION.md)
- [GLR_V1_COMPLETION_CONTRACT.md](../specs/GLR_V1_COMPLETION_CONTRACT.md)

### External

- [Tree-sitter Edit API](https://tree-sitter.github.io/tree-sitter/using-parsers#editing)
- [Incremental Parsing Techniques](https://dl.acm.org/doi/10.1145/800193.569983)
- [Efficient Incremental Parsing (Ghezzi & Mandrioli)](https://doi.org/10.1145/357162.357166)

---

## Decision Record

**Decision**: Adopt pragmatic local reparse window strategy for incremental GLR

**Rationale**:
1. Simple enough to implement correctly (3-4 weeks core work)
2. Fast enough for editor-class performance (<30% of full parse)
3. Safe (automatic fallback prevents pathological cases)
4. Extensible (can add optimizations later without API changes)

**Alternatives Considered**:
- Perfect GLR incrementality (too complex, research problem)
- Content-based hashing (deferred, marginal benefit)
- Persistent data structures (too invasive, not worth refactor)

**Approval Status**: Proposed (pending team review)

---

**ADR Version**: 1.0.0
**Author**: rust-sitter core team
**Review Date**: TBD (after Phase 1B)
**Status**: Proposed → (Accepted | Rejected | Superseded)

---

END OF ADR
