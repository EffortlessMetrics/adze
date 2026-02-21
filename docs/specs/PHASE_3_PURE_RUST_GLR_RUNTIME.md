# Phase 3: Pure-Rust GLR Runtime Architecture

**Status**: Planning
**Dependencies**: Phase 2 Complete ✅
**Objective**: Implement GLR parsing runtime that preserves multi-action cells
**Timeline**: 2-3 weeks (estimated)

---

## Executive Summary

Phase 2 conclusively validated that:
1. ✅ glr-core generates conflicts correctly (1 S/R conflict in ambiguous_expr test)
2. ✅ Multi-action cells are created in ParseTable before encoding
3. ❌ TSLanguage ABI cannot represent multi-action cells (architectural limitation)
4. ✅ Conflict inspection API works correctly

**Phase 3 Goal**: Implement a pure-Rust GLR runtime path that **bypasses TSLanguage encoding** and uses ParseTable directly, enabling full GLR conflict preservation.

---

## Architectural Decision

### Current Architecture (LR Mode)

```
Grammar IR
    ↓
glr-core::build_lr1_automaton()
    ↓
ParseTable (multi-action cells) ← Conflicts exist here!
    ↓
tablegen::compress() [uses choose_action()]
    ↓
TSLanguage (single action per cell) ← Conflicts eliminated!
    ↓
runtime::decoder::decode_parse_table()
    ↓
ParseTable (single-action cells) ← Conflicts lost!
    ↓
Runtime parsing (LR mode only)
```

### Proposed Architecture (GLR Mode)

```
Grammar IR
    ↓
glr-core::build_lr1_automaton()
    ↓
ParseTable (multi-action cells) ← Conflicts preserved!
    ↓
[BYPASS TSLanguage encoding]
    ↓
runtime2::GLRParser::new(parse_table)
    ↓
GLR Runtime (pure-Rust)
    ↓
ParseForest (multiple parse trees)
```

**Key Insight**: The GLR runtime should consume ParseTable directly, skipping TSLanguage entirely in pure-Rust mode.

---

## Design Requirements

### 1. Feature Flag Strategy

```toml
[features]
default = ["tree-sitter-c2rust"]  # Existing LR runtime
glr-core = ["dep:adze-glr-core"]  # Enable GLR parsing
pure-rust-glr = ["glr-core"]  # Full pure-Rust GLR stack (no TSLanguage)
```

**Behavior**:
- `default`: LR parsing via TSLanguage (current behavior)
- `glr-core`: Enable GLR parsing capabilities
- `pure-rust-glr`: GLR parsing with ParseTable direct access (Phase 3 goal)

### 2. Runtime API Compatibility

**Goal**: Maintain Tree-sitter API compatibility while enabling GLR mode.

```rust
// runtime2/src/parser.rs
pub struct Parser {
    language: Option<&'static TSLanguage>,  // LR mode
    #[cfg(feature = "pure-rust-glr")]
    glr_table: Option<ParseTable>,  // GLR mode (bypasses TSLanguage)
}

impl Parser {
    /// LR mode: Use TSLanguage (current API)
    pub fn set_language(&mut self, lang: &'static TSLanguage) {
        self.language = Some(lang);
    }

    /// GLR mode: Use ParseTable directly (new API)
    #[cfg(feature = "pure-rust-glr")]
    pub fn set_glr_table(&mut self, table: ParseTable) {
        self.glr_table = Some(table);
    }

    /// Parse with automatic mode selection
    pub fn parse(&mut self, input: &[u8]) -> Result<Tree, ParseError> {
        #[cfg(feature = "pure-rust-glr")]
        if let Some(table) = &self.glr_table {
            return self.parse_glr(input, table);
        }

        // Fall back to LR mode
        self.parse_lr(input)
    }
}
```

### 3. GLR Parsing Pipeline

```rust
// runtime2/src/glr_parser.rs
impl Parser {
    fn parse_glr(&mut self, input: &[u8], table: &ParseTable) -> Result<Tree, ParseError> {
        // Step 1: Initialize GLR engine with ParseTable
        let mut glr_engine = GLREngine::new(table);

        // Step 2: Tokenize input
        let tokens = self.tokenize(input, table)?;

        // Step 3: GLR parsing (handles conflicts via forking)
        let forest = glr_engine.parse(&tokens)?;

        // Step 4: Disambiguation (select single parse tree)
        let tree = forest.select_best_parse()?;

        Ok(tree)
    }
}
```

### 4. ParseTable Serialization (Future)

For build-time generation and runtime loading:

```rust
// tablegen/src/lib.rs
#[cfg(feature = "pure-rust-glr")]
pub fn serialize_parse_table(table: &ParseTable) -> Vec<u8> {
    // Serialize ParseTable as MessagePack or bincode
    // Preserves multi-action cells
}

// runtime2/src/parser.rs
#[cfg(feature = "pure-rust-glr")]
impl Parser {
    pub fn load_glr_table(bytes: &[u8]) -> Result<ParseTable, Error> {
        // Deserialize ParseTable
    }
}
```

---

## Implementation Phases

### Phase 3.1: Core GLR Runtime (1 week)

**Objective**: Get basic GLR parsing working with direct ParseTable access.

**Tasks**:
1. ✅ Validate glr-core generates conflicts (DONE - diagnostic test)
2. Add `pure-rust-glr` feature flag to runtime2
3. Implement `Parser::set_glr_table()`
4. Implement basic GLR parsing engine:
   - Fork on conflicts
   - Maintain multiple stacks (GSS)
   - Merge identical stacks
5. Create minimal test: parse "1 + 2 + 3" with ambiguous grammar

**Success Criteria**:
- [ ] Parse ambiguous grammar without panicking
- [ ] Generate multiple parse trees for ambiguous input
- [ ] Tests pass with `--features pure-rust-glr`

---

### Phase 3.2: Disambiguation and Tree Selection (4-5 days)

**Objective**: Select a single parse tree from the forest.

**Tasks**:
1. Implement parse forest representation
2. Add disambiguation strategies:
   - Prefer shift over reduce (Tree-sitter default)
   - Precedence-aware selection
   - Longest match
3. Integrate with runtime2 Tree API
4. Add forest → Tree conversion

**Success Criteria**:
- [ ] Ambiguous input produces single Tree
- [ ] Disambiguation respects precedence declarations
- [ ] Tree structure matches expected AST

---

### Phase 3.3: Integration Testing (3-4 days)

**Objective**: Validate end-to-end GLR pipeline.

**Tasks**:
1. Update example grammars to use `pure-rust-glr` feature
2. Run ambiguous_expr and dangling_else tests with GLR runtime
3. Compare GLR output with LR output (for unambiguous grammars)
4. Performance benchmarking
5. Memory profiling (GSS can grow large)

**Success Criteria**:
- [ ] All example grammars parse correctly
- [ ] GLR runtime ≤ 2x slower than LR runtime (for unambiguous grammars)
- [ ] Memory usage reasonable (< 100MB for typical inputs)

---

### Phase 3.4: Documentation and Stabilization (2-3 days)

**Objective**: Lock in the API and create comprehensive docs.

**Tasks**:
1. Update CLAUDE.md with GLR runtime usage
2. Create architecture decision record (ADR)
3. Document feature flag combinations
4. Add examples to docs/examples/
5. Create migration guide (LR → GLR)

**Success Criteria**:
- [ ] API documented with examples
- [ ] ADR explains TSLanguage vs ParseTable decision
- [ ] Users can enable GLR mode by adding feature flag

---

## API Contracts

### ParseTable Direct Access

```rust
/// Contract: ParseTable from glr-core is immutable and thread-safe
///
/// Requirements:
/// - ParseTable must be Send + Sync (for multi-threaded parsing)
/// - action_table: Vec<Vec<Vec<Action>>> preserved exactly
/// - No TSLanguage encoding/decoding involved
///
/// Guarantees:
/// - Multi-action cells preserved
/// - Conflicts detected and handled via forking
/// - No data loss through serialization
#[cfg(feature = "pure-rust-glr")]
pub struct GLRRuntime {
    table: &'static ParseTable,  // Or Arc<ParseTable> for dynamic loading
}
```

### Conflict Handling Contract

```rust
/// Contract: GLR engine handles all conflicts via forking
///
/// Behavior:
/// - On Shift/Reduce conflict: Fork into 2 branches
/// - On Reduce/Reduce conflict: Fork into N branches (one per reduce)
/// - Identical stacks merged (GSS optimization)
///
/// Invariants:
/// - Every conflict creates ≥ 2 branches
/// - All branches are explored
/// - Final forest contains all valid parse trees
pub fn handle_conflict(cell: &[Action]) -> Vec<ParserBranch> {
    cell.iter().map(|action| ParserBranch::new(action.clone())).collect()
}
```

---

## Testing Strategy

### Unit Tests

```rust
#[cfg(all(test, feature = "pure-rust-glr"))]
mod tests {
    #[test]
    fn test_glr_parses_ambiguous_grammar() {
        let grammar = GrammarBuilder::new("test")
            .token("NUM", r"\d+")
            .token("+", "+")
            .rule("expr", vec!["expr", "+", "expr"])  // NO precedence
            .rule("expr", vec!["NUM"])
            .build();

        let table = build_lr1_automaton(&grammar).unwrap();
        let mut parser = Parser::new();
        parser.set_glr_table(table);

        let tree = parser.parse(b"1 + 2 + 3").unwrap();
        assert!(tree.root_node().is_some());
    }

    #[test]
    fn test_glr_generates_multiple_trees() {
        // ... parse ambiguous input and verify forest has > 1 tree
    }

    #[test]
    fn test_glr_selects_correct_tree() {
        // ... verify disambiguation picks the right tree
    }
}
```

### Integration Tests

```rust
// runtime2/tests/glr_integration.rs
#[test]
#[cfg(feature = "pure-rust-glr")]
fn test_ambiguous_expr_end_to_end() {
    // Load example grammar
    // Build ParseTable
    // Parse multiple inputs
    // Validate all parses succeed
}

#[test]
#[cfg(feature = "pure-rust-glr")]
fn test_glr_vs_lr_parity() {
    // For unambiguous grammars, GLR should produce same tree as LR
}
```

### Diagnostic Tests

```rust
#[test]
#[cfg(feature = "pure-rust-glr")]
fn test_conflict_count_matches_glr_core() {
    // Ensure runtime sees same conflicts as glr-core generated
    let table = build_lr1_automaton(&grammar).unwrap();
    let runtime_conflicts = count_runtime_forks(&table, input);
    let static_conflicts = count_conflicts(&table);

    assert_eq!(runtime_conflicts, static_conflicts.shift_reduce);
}
```

---

## Performance Considerations

### Expected Performance

**Unambiguous Grammars**:
- GLR runtime should be ≤ 2x slower than LR (due to fork overhead)
- Most forks will merge quickly (identical stacks)

**Ambiguous Grammars**:
- Parse forests can be exponential in worst case
- Practical grammars: ≤ 10 trees for typical inputs
- Need timeout mechanism for pathological cases

### Optimizations

1. **Stack Merging**: Merge identical GSS nodes aggressively
2. **Lazy Forest Construction**: Only materialize trees when needed
3. **Early Pruning**: Discard low-priority branches during disambiguation
4. **Memoization**: Cache subtree results (packrat-style)

### Memory Limits

```rust
pub struct GLRConfig {
    pub max_forks: usize,       // Default: 1000
    pub max_forest_nodes: usize, // Default: 10000
    pub timeout_ms: u64,        // Default: 5000
}
```

---

## Migration Guide

### From LR Runtime to GLR Runtime

**Before** (LR mode):
```rust
let lang = unsafe { &adze_example::ambiguous_expr::LANGUAGE };
let mut parser = Parser::new();
parser.set_language(lang);
let tree = parser.parse(b"1 + 2 + 3").unwrap();
```

**After** (GLR mode):
```rust
use adze_glr_core::build_lr1_automaton;

let grammar = /* load grammar */;
let table = build_lr1_automaton(&grammar).unwrap();

let mut parser = Parser::new();
parser.set_glr_table(table);  // No TSLanguage!
let tree = parser.parse(b"1 + 2 + 3").unwrap();
```

**Hybrid** (both modes supported):
```rust
#[cfg(feature = "pure-rust-glr")]
let mut parser = create_glr_parser();

#[cfg(not(feature = "pure-rust-glr"))]
let mut parser = create_lr_parser();
```

---

## Risk Mitigation

### Risks

1. **Performance**: GLR can be slow for deeply ambiguous grammars
   - **Mitigation**: Add configurable limits, timeout mechanism

2. **Memory**: Parse forests can grow exponentially
   - **Mitigation**: Streaming disambiguation, early pruning

3. **API Stability**: New API might need changes
   - **Mitigation**: Mark as experimental feature in Phase 3

4. **Testing Coverage**: Hard to test all conflict scenarios
   - **Mitigation**: Comprehensive test suite, fuzzing

---

## Success Metrics

### Phase 3 Complete When:

- [ ] GLR runtime parses ambiguous grammars without errors
- [ ] Conflict handling verified (fork on S/R conflicts)
- [ ] Forest → Tree conversion working
- [ ] Disambiguation produces single Tree
- [ ] Integration tests passing with example grammars
- [ ] Documentation complete
- [ ] Performance acceptable (≤ 2x LR runtime for unambiguous grammars)
- [ ] API contracts documented and stable

---

## Dependencies

### Internal Dependencies
- ✅ glr-core (conflict generation validated)
- ✅ adze-ir (Grammar representation)
- ⏳ runtime2 (needs GLR engine integration)
- ⏳ tablegen (may need ParseTable serialization)

### External Dependencies
- None (pure-Rust implementation)

---

## Follow-Up Work (Phase 4+)

### TSLanguage Extension for C Compatibility

For users who need C-compatible GLR:

```c
struct TSLanguageGLR {
    TSLanguage base;
    uint16_t *conflict_table;  // Conflict action entries
    uint32_t conflict_count;
    // ...
};
```

This is **future work** - Phase 3 focuses on pure-Rust path first.

### Advanced GLR Features

- Incremental parsing with GLR
- Error recovery in GLR mode
- Parallel parsing (exploit fork/join parallelism)
- GLR-specific optimizations (SPPF representation)

---

## References

- [PHASE_2_FINDINGS.md](../status/PHASE_2_FINDINGS.md) - Root cause analysis
- [CONFLICT_INSPECTION_API.md](./CONFLICT_INSPECTION_API.md) - Conflict detection
- [glr-core validation test](../../glr-core/tests/diagnose_ambiguous_expr.rs) - Proof of conflict generation
- runtime2 GLR integration (PR #14) - Existing GLR runtime foundation

---

**Status**: Phase 3 Specification Complete - Ready for Implementation
**Next**: Begin Phase 3.1 - Core GLR Runtime Implementation
**Timeline**: 2-3 weeks to full GLR runtime completion

