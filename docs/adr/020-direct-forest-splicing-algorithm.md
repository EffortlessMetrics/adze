# ADR-020: Direct Forest Splicing Algorithm

## Status

Accepted

## Context

Incremental parsing is essential for IDE features like syntax highlighting, error detection, and code intelligence. When a user types a single character, re-parsing the entire file is wasteful and creates noticeable latency in large files.

Traditional incremental parsers (including Tree-sitter) restore parser state from previous parses using Graph-Structured Stack (GSS) restoration:

1. Store parse state (GSS positions) from previous parse
2. Find the edit location in the old parse
3. Restore parser state at that position
4. Re-parse from that point forward

For GLR parsers, this traditional approach has significant challenges:

### Problems with GSS Restoration

1. **High Overhead**: Restoring GSS state requires 3-4× memory and computation overhead compared to forward parsing
2. **Complex Bookkeeping**: GLR maintains multiple parallel parse stacks (forks); managing fork states across edits is error-prone
3. **Poor Cache Locality**: State restoration thrashes CPU caches due to non-linear memory access patterns
4. **GLR-Specific Complexity**: Multiple valid parse paths must all be restored consistently

### Alternatives Considered

1. **No Incremental Support**: Always re-parse from scratch — simple but too slow for IDE use cases
2. **Tree-sitter Style GSS Restoration**: Port Tree-sitter's approach directly — high overhead for GLR
3. **Tree Reuse Only**: Reuse unchanged subtrees without state restoration — limited reuse potential
4. **Direct Forest Splicing**: Parse only the changed region and splice forests — novel approach

## Decision

We implemented **Direct Forest Splicing**, a novel algorithm that bypasses GSS state restoration entirely by operating directly on parse forests.

### Algorithm Overview

```
┌─────────────────────────────────────────────────────────────┐
│                    Old Parse Forest                          │
│   [Prefix unchanged] [Edited region] [Suffix unchanged]     │
└─────────────────────────────────────────────────────────────┘
                              │
          ┌───────────────────┼───────────────────┐
          ▼                   ▼                   ▼
┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐
│ Extract Prefix  │  │  Parse Middle   │  │ Extract Suffix  │
│    Nodes        │  │   (changed)     │  │    Nodes        │
└─────────────────┘  └─────────────────┘  └─────────────────┘
          │                   │                   │
          └───────────────────┼───────────────────┘
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                    Spliced Forest                            │
│         [Old prefix] + [New middle] + [Old suffix]          │
└─────────────────────────────────────────────────────────────┘
```

### Phase 1: Chunk Identification

```rust
pub struct ChunkIdentifier {
    // Identifies unchanged prefix/suffix at token granularity
    // Handles byte offset adjustments for suffix after edits
    // Returns (prefix_len, suffix_len) in token counts
}
```

The chunking phase operates on tokenized input, finding maximal unchanged regions before and after the edit location. This is O(edit size) because we only need to examine tokens near edit boundaries.

### Phase 2: Forest Node Extraction

```rust
fn extract_reusable_nodes(
    old_forest: &ForestNode,
    target_range: Range<usize>,  // Token range
) -> Vec<ExtractedNode>
```

Recursively traverses the old parse forest to find all maximal subtrees fully contained within unchanged regions:

- Extracts multiple smaller nodes rather than one large node for maximum reuse
- Adjusts byte ranges for suffix nodes based on edit delta
- Preserves all alternatives in ambiguous nodes (critical for GLR correctness)

### Phase 3: Middle-Only Parsing

Parse ONLY the edited middle segment using the standard GLR parser. This is where the O(edit size) guarantee comes from — the parse time depends only on the size of the edited region, not the file size.

### Phase 4: Forest Splicing

```rust
fn splice_forests(
    prefix: Vec<Arc<ForestNode>>,
    middle: Option<Arc<ForestNode>>,
    suffix: Vec<Arc<ForestNode>>,
) -> Arc<ForestNode>
```

Combines extracted prefix/suffix nodes with newly parsed middle:

- Creates synthetic root with all children
- Calculates correct byte/token ranges
- Optimizes single-child cases to avoid unnecessary tree depth
- Preserves ambiguity alternatives throughout

### Performance Results

From real-world testing:

| Test Case | Full Parse | Incremental | Speedup | Reused Nodes |
|-----------|------------|-------------|---------|--------------|
| 1,000 tokens, single edit | 3.5ms | 215μs | **16.34×** | 999 |
| 500 tokens, middle edit | 1.8ms | 180μs | **10×** | 498 |
| 100 tokens, end edit | 350μs | 45μs | **7.8×** | 99 |

The algorithm achieves **O(edit size)** performance, meaning parsing time scales with the size of the edit, not the size of the file.

## Consequences

### Positive

- **O(edit size) Performance**: Parsing time scales with edit size, not file size — critical for IDE responsiveness
- **No State Restoration Overhead**: Eliminates 3-4× memory/compute overhead of GSS restoration
- **GLR Compatible**: Preserves all parse alternatives in ambiguous regions — essential for languages like C++, Rust, Python
- **Cache Friendly**: Linear traversal of old forest has excellent locality compared to GSS graph traversal
- **Simplicity**: Algorithm is easier to understand and maintain than GSS restoration
- **High Reuse Rate**: 999/1000 subtrees reused on typical edits

### Negative

- **Conservative Reuse**: May re-parse more than necessary near edit boundaries due to token-level granularity
- **Memory Overhead**: Must retain previous parse forest between edits (already required for tree queries)
- **Fallback Required**: Some complex edit patterns may still require full re-parse
- **Token Granularity**: Works at token level, not byte level — may re-parse more than byte-level approaches

### Neutral

- **Forest Representation**: Requires SPPF (Shared Packed Parse Forest) for GLR — already part of architecture
- **Edit Tracking**: Caller must provide edit ranges — standard for incremental parsing APIs
- **Implementation Maturity**: Novel algorithm, less battle-tested than Tree-sitter's approach

## Implementation Details

### Core Components

1. **IncrementalGLRParser** (`glr_incremental.rs`)
   - Manages previous forest for reuse
   - Implements chunk identification logic
   - Coordinates forest extraction and splicing

2. **ChunkIdentifier**
   - Token-level diff algorithm
   - Finds maximal unchanged prefix/suffix
   - Handles byte offset adjustments

3. **Forest Extraction** (`extract_reusable_nodes`)
   - Recursive tree traversal
   - Collects all nodes in target ranges
   - Adjusts suffix node byte ranges

4. **Forest Splicing** (`splice_forests`)
   - Combines prefix + middle + suffix
   - Creates synthetic root node
   - Preserves ambiguity alternatives

### Design Principles

1. **Correctness First**: Never sacrifice parse correctness for performance
2. **Ambiguity Preservation**: All valid interpretations must be maintained
3. **Minimal Reparsing**: Reparse the smallest possible region
4. **Fork Independence**: Each fork's decisions are tracked independently

## Related

- Related ADRs: [ADR-005: Incremental Parsing Architecture](005-incremental-parsing-architecture.md)
- Implementation: [docs/archive/implementation/GLR_INCREMENTAL_DESIGN.md](../archive/implementation/GLR_INCREMENTAL_DESIGN.md)
- Tests: [grammars/python/tests/incremental_glr_test.rs](../../grammars/python/tests/incremental_glr_test.rs)
- API: [runtime2/src/engine.rs](../../runtime2/src/engine.rs) — `parse_incremental` function
