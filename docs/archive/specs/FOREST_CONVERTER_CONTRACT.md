# Forest-to-Tree Conversion Contract Specification

**Component**: ForestConverter (Phase 3.2, Component 2)
**Version**: 1.0.0
**Date**: 2025-01-19
**Dependencies**: GLREngine ParseForest output, Tree API
**Status**: Specification Complete

---

## Executive Summary

The ForestConverter transforms a ParseForest (potentially containing multiple parse trees due to ambiguity) into a single Tree structure compatible with the Tree-sitter API. This component implements disambiguation strategies to select one parse tree when multiple valid parses exist.

**Key Responsibilities**:
1. Convert ParseForest nodes to Tree nodes
2. Apply disambiguation strategies for ambiguous parses
3. Filter invisible nodes (e.g., parentheses, whitespace)
4. Validate tree structure (no cycles, valid ranges)
5. Preserve all node metadata (symbols, positions, visibility)

---

## Data Structure Contracts

### ForestNode (from GLREngine)

```rust
/// Forest node representation (already defined in glr_engine.rs)
#[derive(Debug, Clone)]
pub enum ForestNode {
    /// Terminal node (token from input)
    Terminal {
        symbol: SymbolId,
        range: Range<usize>,
    },
    /// Nonterminal node (reduction result)
    Nonterminal {
        symbol: SymbolId,
        children: Vec<ForestNodeId>,
        rule_id: RuleId,
    },
    /// Packed node (multiple derivations - ambiguity point)
    Packed {
        alternatives: Vec<ForestNodeId>,
    },
}
```

### ParseForest (from GLREngine)

```rust
/// Parse forest accumulator (already defined in glr_engine.rs)
pub struct ParseForest {
    /// All forest nodes
    pub nodes: Vec<ForestNode>,
    /// Root node IDs (successful parses)
    pub roots: Vec<ForestNodeId>,
}
```

### DisambiguationStrategy

```rust
/// Disambiguation strategies for ambiguous parses
///
/// Contract: Determines which alternative to select when forest has
/// multiple valid parse trees (Packed nodes or multiple roots)
///
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisambiguationStrategy {
    /// Prefer shift over reduce (Tree-sitter default)
    ///
    /// Rationale: Shift actions delay decisions, creating right-associative trees.
    /// Example: "1 + 2 + 3" → (1 + (2 + 3)) [right-associative]
    PreferShift,

    /// Prefer reduce over shift
    ///
    /// Rationale: Reduce actions commit early, creating left-associative trees.
    /// Example: "1 + 2 + 3" → ((1 + 2) + 3) [left-associative]
    PreferReduce,

    /// Use precedence from grammar
    ///
    /// Rationale: Respect explicit precedence/associativity declarations.
    /// Example: "1 + 2 * 3" → (1 + (2 * 3)) [* has higher precedence]
    ///
    /// Note: Requires precedence metadata in nodes (Phase 3.3)
    Precedence,

    /// Take first alternative (fast but arbitrary)
    ///
    /// Rationale: No disambiguation logic, just pick first valid parse.
    /// Useful for unambiguous grammars or debugging.
    First,

    /// Reject ambiguity (return error)
    ///
    /// Rationale: Force user to resolve ambiguity in grammar.
    /// Useful for grammar development/testing.
    RejectAmbiguity,
}
```

### ForestConverter

```rust
/// Converts ParseForest to single Tree
///
/// Contract:
/// - Selects one parse tree from potentially multiple valid parses
/// - Applies disambiguation strategy consistently
/// - Preserves all node metadata (symbols, ranges, visibility)
/// - Validates tree structure (no cycles, valid ranges)
///
pub struct ForestConverter {
    /// Disambiguation strategy to use
    strategy: DisambiguationStrategy,
}
```

---

## API Contract

### ForestConverter::new()

```rust
/// Create converter with strategy
///
/// # Contract
///
/// ## Preconditions
/// - Strategy is valid (any enum variant)
///
/// ## Postconditions
/// - Converter ready to convert forests
/// - Strategy stored for all conversions
///
/// ## Example
///
/// ```rust
/// let converter = ForestConverter::new(DisambiguationStrategy::PreferShift);
/// ```
///
pub fn new(strategy: DisambiguationStrategy) -> Self;
```

### ForestConverter::to_tree()

```rust
/// Convert ParseForest to Tree
///
/// # Contract
///
/// ## Preconditions
/// - `forest.roots` is non-empty (at least one valid parse)
/// - Forest nodes form valid tree structure (no cycles)
/// - All ForestNodeIds reference valid nodes (< forest.nodes.len())
/// - `input` matches the original input used for parsing
///
/// ## Postconditions
/// - Tree has single root node
/// - All visible nodes included in tree
/// - Node ranges are consistent:
///   - Parent contains all children: parent.start <= child.start < child.end <= parent.end
///   - Root node covers entire input: root.range() == 0..input.len()
/// - Tree structure matches grammar rules
/// - Symbol visibility preserved (invisible nodes filtered out)
///
/// ## Invariants
/// - Root node covers entire input: root.range() == 0..input.len()
/// - For all nodes: node.start() <= child.start() < child.end() <= node.end()
/// - For all nodes: node.child_count() >= 0
/// - Symbol metadata respected (visibility, terminal/nonterminal)
///
/// ## Errors
/// - `ConversionError::NoRoots`: Forest has no root nodes
/// - `ConversionError::AmbiguousForest`: Multiple roots (if strategy = RejectAmbiguity)
/// - `ConversionError::InvalidForest`: Forest structure is malformed (cycles, invalid refs)
/// - `ConversionError::InvalidNodeId`: ForestNodeId out of bounds
///
/// ## Performance
/// - Time: O(n) where n = forest node count (single DFS traversal)
/// - Space: O(d) where d = tree depth (stack for recursion)
///
/// ## Algorithm
///
/// Phase 1: Select Root
///   If forest.roots.is_empty() → Error(NoRoots)
///   If forest.roots.len() == 1 → Use that root
///   Else → Apply disambiguation strategy
///
/// Phase 2: Build Tree (DFS)
///   Match forest_node:
///     Terminal → Create leaf node
///     Nonterminal → Recurse on children, filter invisible
///     Packed → Apply disambiguation, recurse on selected alternative
///
/// ## Example
///
/// ```rust
/// let converter = ForestConverter::new(DisambiguationStrategy::PreferShift);
/// let tree = converter.to_tree(&forest, input)?;
/// assert_eq!(tree.root_node().kind(), "expr");
/// ```
///
pub fn to_tree(
    &self,
    forest: &ParseForest,
    input: &[u8],
) -> Result<Tree, ConversionError>;
```

### ForestConverter::detect_ambiguity()

```rust
/// Detect ambiguity in forest (diagnostic utility)
///
/// # Contract
///
/// ## Preconditions
/// - `forest` is valid
///
/// ## Returns
/// - `None`: Forest is unambiguous (single parse tree)
/// - `Some(count)`: Forest has `count` alternative parses
///
/// ## Algorithm
///
/// Count alternatives by:
/// 1. Count roots: forest.roots.len()
/// 2. Count Packed nodes in tree
/// 3. Return max(roots, packed_alternatives)
///
/// ## Performance
/// - Time: O(n) - traverse entire forest
/// - Space: O(1) - constant memory
///
/// ## Example
///
/// ```rust
/// if let Some(count) = converter.detect_ambiguity(&forest) {
///     eprintln!("Warning: {} alternative parses", count);
/// }
/// ```
///
pub fn detect_ambiguity(&self, forest: &ParseForest) -> Option<usize>;
```

---

## Disambiguation Algorithm Specifications

### PreferShift Strategy

**Algorithm**:
```
Given: Packed node with alternatives [alt1, alt2, ...]

Step 1: Identify shift vs reduce alternatives
  For each alternative:
    If came from shift action → shift_alts.push(alt)
    Else → reduce_alts.push(alt)

Step 2: Select
  If shift_alts.is_empty() → return reduce_alts[0]
  Else → return shift_alts[0]
```

**Rationale**: Shift delays decision-making, creating right-associative trees.

**Example**:
```
Input: "1 + 2 + 3"
Grammar: expr → expr + expr (ambiguous)

Parses:
  A: ((1 + 2) + 3)  ← Reduce early (left-associative)
  B: (1 + (2 + 3))  ← Shift first (right-associative)

PreferShift selects B
```

### PreferReduce Strategy

**Algorithm**:
```
Given: Packed node with alternatives [alt1, alt2, ...]

Step 1: Identify shift vs reduce alternatives
  (same as PreferShift)

Step 2: Select
  If reduce_alts.is_empty() → return shift_alts[0]
  Else → return reduce_alts[0]
```

**Rationale**: Reduce commits early, creating left-associative trees.

### First Strategy

**Algorithm**:
```
Given: Packed node with alternatives [alt1, alt2, ...]

Return alternatives[0]
```

**Rationale**: Simple, fast, deterministic. No analysis needed.

### RejectAmbiguity Strategy

**Algorithm**:
```
Given: Packed node with alternatives

If alternatives.len() > 1:
  Return Err(ConversionError::AmbiguousForest {
    count: alternatives.len()
  })
```

**Rationale**: Fails fast on ambiguity, forcing explicit grammar disambiguation.

---

## Error Handling Contract

### ConversionError Type

```rust
/// Forest conversion errors
#[derive(Debug, thiserror::Error)]
pub enum ConversionError {
    #[error("Forest has no root nodes")]
    NoRoots,

    #[error("Ambiguous forest: {count} valid parses")]
    AmbiguousForest { count: usize },

    #[error("Invalid forest structure: {reason}")]
    InvalidForest { reason: String },

    #[error("Invalid node reference: {node_id}")]
    InvalidNodeId { node_id: usize },

    #[error("Cycle detected in forest at node {node_id}")]
    CycleDetected { node_id: usize },
}
```

### Error Conditions

| Condition | Error | When |
|-----------|-------|------|
| No roots | NoRoots | forest.roots.is_empty() |
| Multiple roots + RejectAmbiguity | AmbiguousForest | roots.len() > 1 |
| Packed node + RejectAmbiguity | AmbiguousForest | Packed alternatives > 1 |
| Invalid node ID | InvalidNodeId | node_id >= forest.nodes.len() |
| Cycle in tree | CycleDetected | DFS visits same node twice |
| Invalid range | InvalidForest | child.end > parent.end |

---

## Testing Contract

### Unit Test Requirements

```rust
#[cfg(test)]
mod forest_converter_tests {
    // Basic functionality
    #[test] fn test_unambiguous_forest();        // Single root → single tree
    #[test] fn test_terminal_node_conversion();  // Leaf nodes
    #[test] fn test_nonterminal_node_conversion(); // Internal nodes

    // Disambiguation strategies
    #[test] fn test_prefer_shift_strategy();     // Right-associative
    #[test] fn test_prefer_reduce_strategy();    // Left-associative
    #[test] fn test_first_strategy();            // Arbitrary selection
    #[test] fn test_reject_ambiguity();          // Error on ambiguity

    // Edge cases
    #[test] fn test_empty_forest();              // No roots → error
    #[test] fn test_detect_ambiguity();          // Ambiguity detection
    #[test] fn test_invalid_node_id();           // Out of bounds
}
```

### Integration Test Requirements

```rust
#[cfg(test)]
mod forest_integration {
    #[test] fn test_end_to_end_arithmetic();     // "1 + 2 * 3"
    #[test] fn test_ambiguous_expression();      // "1 + 2 + 3" with disambiguation
    #[test] fn test_nested_expressions();        // "((1 + 2) * 3)"
}
```

---

## Performance Contract

### Time Complexity

| Operation | Best Case | Average Case | Worst Case |
|-----------|-----------|--------------|------------|
| `to_tree()` | O(n) | O(n) | O(n) |
| `detect_ambiguity()` | O(n) | O(n) | O(n) |

Where n = forest.nodes.len()

### Space Complexity

| Operation | Space |
|-----------|-------|
| `to_tree()` | O(d) recursion stack |
| `detect_ambiguity()` | O(1) |

Where d = tree depth (typically log n)

### Performance Targets

- Convert 1000-node forest: < 1ms
- Convert 10000-node forest: < 10ms
- Memory overhead: ≤ 2x forest size

---

## Validation Rules

### Tree Structure Validation

```rust
/// Validate tree structure after conversion
///
/// Invariants checked:
/// 1. Root covers entire input
/// 2. All child ranges within parent range
/// 3. No overlapping siblings
/// 4. Symbol visibility respected
///
fn validate_tree(tree: &Tree, input: &[u8]) -> Result<(), ConversionError> {
    // Rule 1: Root covers input
    if tree.root_node().byte_range() != (0..input.len()) {
        return Err(InvalidForest("Root doesn't cover input"));
    }

    // Rule 2-4: Recursive validation
    validate_node_recursive(tree.root_node())
}
```

---

## Implementation Notes

### Phase 3.2 MVP Scope

**In Scope**:
- ✅ Basic conversion (Terminal and Nonterminal nodes)
- ✅ PreferShift and First strategies
- ✅ Error handling (NoRoots, AmbiguousForest)
- ✅ Basic validation (no cycles, valid ranges)

**Out of Scope** (defer to Phase 3.3):
- ⚠️ Precedence strategy (requires precedence metadata)
- ⚠️ External scanner node handling
- ⚠️ Advanced optimizations (SPPF, lazy evaluation)
- ⚠️ Incremental forest conversion

### Integration Points

**With GLREngine**:
- Input: `ParseForest` from `GLREngine::parse()`
- ForestNode types must match GLREngine output

**With Tree API**:
- Output: `Tree` compatible with existing API
- Node construction via `Tree::new_stub()` or builder

**With Parser**:
- Called from `Parser::parse_glr()` after GLREngine
- Replaces current `Tree::new_stub()` stub

---

## Security Considerations

### Cycle Detection

**Risk**: Malformed forest could contain cycles, causing infinite recursion.

**Mitigation**:
```rust
fn build_node(&self, node_id: ForestNodeId, visited: &mut HashSet<ForestNodeId>) -> Result<Node> {
    if visited.contains(&node_id) {
        return Err(ConversionError::CycleDetected { node_id: node_id.0 });
    }
    visited.insert(node_id);
    // ... build node
}
```

### Stack Overflow Protection

**Risk**: Deep recursion on pathological grammars.

**Mitigation**:
- Limit recursion depth (e.g., max 1000 levels)
- Use iterative DFS with explicit stack if needed

### Memory Bounds

**Risk**: Large forests could exhaust memory.

**Mitigation**:
- Validate forest size before conversion
- Implement streaming/lazy conversion for very large forests

---

## Future Enhancements (Post Phase 3.2)

1. **Precedence Strategy**: Implement using grammar precedence metadata
2. **SPPF Support**: Efficient ambiguity representation
3. **Lazy Conversion**: Build tree on-demand (cursor-driven)
4. **Parallel Conversion**: Multi-threaded tree building
5. **Custom Strategies**: User-defined disambiguation callbacks

---

## References

### Specifications
- [Phase 3.2 Master Spec](./PHASE_3.2_TOKENIZATION_FOREST_CONVERSION.md)
- [GLR Engine Contract](./GLR_ENGINE_CONTRACT.md)

### Academic Papers
- "SPPF-Style Parsing from Earley Recognisers" - Scott & Johnstone (2016)
- "GLL Parsing" - Scott & Johnstone (2010)
- "Efficient Computation of LALR(1) Look-Ahead Sets" - DeRemer & Pennello (1982)

---

**Document Status**: Complete ✅
**Contract Version**: 1.0.0
**Ready for Implementation**: Yes
