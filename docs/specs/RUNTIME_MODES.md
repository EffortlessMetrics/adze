# Runtime Modes: Tree-sitter LR vs Rust-native GLR

**Date**: 2025-11-20
**Status**: ACTIVE
**Related**: GLR_V1_COMPLETION_CONTRACT.md, PARSETABLE_FILE_FORMAT_SPEC.md
**Priority**: CRITICAL (Architecture clarity)

---

## Executive Summary

Rust-sitter provides **two intentional runtime modes**, not one "real" and one "legacy":

1. **Tree-sitter LR Mode** (`runtime/`) - Full TSLanguage ABI compatibility
2. **Rust-native GLR Mode** (`runtime2/` + `.parsetable`) - True GLR semantics

Both modes share the same IR (`adze_ir::Grammar`) and tree-facing API, but differ in their parse table representation and execution semantics.

**Key Principle**:
> "Tree-sitter compatibility is supported fully in LR mode; GLR semantics are available via the `.parsetable` + runtime2 path for grammars that need them."

---

## Mode 1: Tree-sitter LR Mode (`runtime/`)

### What It Is

The Tree-sitter LR mode provides **100% compatibility** with the Tree-sitter ecosystem by using the `TSLanguage` ABI directly.

**Core Characteristics**:
- **Backend**: `TSLanguage` C ABI via `runtime/src/decoder.rs`
- **Parse Tables**: Exactly one `TSParseAction` per `(state, symbol)` cell
- **Conflict Resolution**: Happens at tablegen time (before runtime)
- **API**: Full Tree-sitter compatibility (incremental, queries, external scanners)

### What We Promise

✅ **Full Tree-sitter Compatibility**:
- If it's a valid Tree-sitter grammar, it works here
- Incremental parsing support
- Query system parity
- External scanner integration
- Performance matches Tree-sitter C as closely as possible

✅ **Production-Ready Features**:
- Battle-tested for unambiguous grammars
- Editor integration ready
- WASM support
- Memory safety via Rust

### What We Don't Promise

⚠️ **GLR Limitations** (by design, matching Tree-sitter):
- No true ambiguity preservation
- No multi-tree forest semantics
- Conflicts resolved at table generation time
- Single canonical parse tree only

**This is not a bug** - it's how Tree-sitter works, and we maintain that behavior exactly.

### When to Use TS-LR Mode

Use Tree-sitter LR mode when you need:

- ✅ Existing Tree-sitter grammar compatibility
- ✅ Editor integration (incremental parsing, syntax highlighting)
- ✅ Query system for code analysis
- ✅ Battle-tested, proven parsing infrastructure
- ✅ Maximum ecosystem compatibility

**Ideal for**: Programming language editors, syntax highlighters, code analysis tools where LR grammars are sufficient.

### File Locations

- **Runtime**: `runtime/src/parser.rs`, `runtime/src/decoder.rs`
- **Table Generation**: Via Tree-sitter's C code generation
- **ABI**: `TSLanguage` struct from Tree-sitter
- **Tests**: `runtime/tests/integration_test.rs`

---

## Mode 2: Rust-native GLR Mode (`runtime2/` + `.parsetable`)

### What It Is

The Rust-native GLR mode provides **true GLR semantics** by using the `glr-core` parse table format directly, bypassing the Tree-sitter ABI limitations.

**Core Characteristics**:
- **Backend**: `glr-core::ParseTable` via `.parsetable` binary format
- **Parse Tables**: Multi-action cells preserved intact (1..N actions per cell)
- **Conflict Preservation**: All conflicts maintained in tables and explored at runtime
- **Execution**: `GLREngine` + `ParseForest` + `ForestConverter`

### What We Promise

✅ **True GLR Semantics**:
- Ambiguous grammars stay ambiguous (dangling-else, ambiguous expressions)
- Multi-action cells preserved through entire pipeline
- Parse forest construction and traversal
- Conflict inspection API for grammar authors

✅ **Rust-Native Benefits**:
- No C FFI overhead for GLR operations
- Memory safety guarantees
- WASM first-class support
- Direct access to internal structures

✅ **Disambiguation Strategies**:
- `First`: Take first parse in forest
- `PreferShift`: Prioritize shift over reduce
- `PreferReduce`: Prioritize reduce over shift
- `Forest`: Expose all parse trees

### What We're Building Toward

⏳ **Incremental Parsing**: GLR-aware subtree reuse (planned v0.7.0)
⏳ **Query System**: Forest-compatible queries (planned vNext)
⏳ **External Scanners**: Rust-native scanner API (partial support)

### When to Use GLR Mode

Use Rust-native GLR mode when you need:

- ✅ **Ambiguous grammar support** (languages with inherent ambiguity)
- ✅ **Language research/tooling** (need to explore multiple parses)
- ✅ **Grammar debugging** (conflict inspection, parse forest visualization)
- ✅ **Pure Rust** (no C dependencies, WASM deployment)
- ✅ **Runtime grammar loading** (`.parsetable` as deployable artifact)

**Ideal for**: Language tooling, grammar research, intentionally ambiguous DSLs, environments requiring pure Rust.

### File Locations

- **Runtime**: `runtime2/src/parser.rs`, `runtime2/src/engine.rs`, `runtime2/src/builder.rs`
- **Table Generation**: `glr-core/src/lib.rs`, `tablegen/src/compress.rs`
- **Format**: `.parsetable` binary files (bincode serialization)
- **Tests**: `runtime2/tests/test_glr_integration.rs`, `runtime2/tests/test_bdd_glr_runtime.rs`

---

## Comparison Matrix

| Aspect                    | TS-LR Mode (`runtime/`)         | Rust-GLR Mode (`runtime2/`)     |
|---------------------------|----------------------------------|----------------------------------|
| **ABI**                   | Tree-sitter C ABI                | Pure Rust                        |
| **Actions per cell**      | Exactly 1                        | 1..N (multi-action)              |
| **Ambiguity handling**    | Resolved at tablegen             | Preserved in tables/forest       |
| **Parse result**          | Single tree                      | Forest → Tree (or multi-tree)    |
| **Incremental parsing**   | ✅ Yes (TS parity target)        | ⏳ Planned (v0.7.0)               |
| **Query system**          | ✅ Yes (TS parity target)        | ⏳ Planned (vNext)                |
| **External scanners**     | ✅ TS-style FFI                  | ⚠️ Partial (Rust-native API)     |
| **Conflict inspection**   | ❌ Not exposed                   | ✅ Full API                       |
| **Forest access**         | ❌ Not available                 | ✅ Available                      |
| **WASM support**          | ✅ Yes                           | ✅ Yes (first-class)              |
| **C dependencies**        | Tree-sitter runtime              | None                             |
| **Table format**          | TSLanguage binary                | `.parsetable` (bincode)          |
| **Performance**           | Optimized LR                     | GLR fork/merge overhead          |
| **Maturity**              | Production-ready                 | Beta (v0.7.0 target)             |

---

## Shared Infrastructure

Both modes share the following components:

### 1. Grammar IR (`adze_ir::Grammar`)

Both modes consume the same intermediate representation:

```rust
pub struct Grammar {
    pub name: String,
    pub rules: HashMap<String, Rule>,
    pub precedences: Vec<PrecedenceEntry>,
    pub conflicts: Vec<ConflictSet>,
    pub externals: Vec<ExternalToken>,
    // ... shared metadata
}
```

**Invariant**: Any grammar expressible in `adze_ir` can be processed by both modes.

### 2. Tree API (`runtime/src/tree.rs`)

Both modes produce the same `Tree` type:

```rust
pub struct Tree {
    root: Option<Node>,
    // ... internal structure
}

impl Tree {
    pub fn root_node(&self) -> Option<Node>;
    pub fn walk(&self) -> TreeCursor;
    pub fn edit(&mut self, edit: &InputEdit);
    // ... shared API
}
```

**Invariant**: User-facing tree operations are identical across modes.

### 3. Error Types

Both modes use compatible error representations:

```rust
pub enum ParseError {
    InvalidInput { byte: usize, expected: Vec<String> },
    InternalError(String),
    // ... shared error types
}
```

---

## Configuration and Selection

### Grammar-Level Configuration

Grammars can specify their runtime requirements:

```rust
// In grammar annotation or config
#[adze::grammar("my_lang")]
#[runtime(mode = "lr")]  // or "glr" or "both"
#[glr_required = false]  // or true
pub mod my_lang { ... }
```

**Behavior**:
- `mode = "lr"`: Only TS-LR generation required
- `mode = "glr"`: Only GLR `.parsetable` generation required
- `mode = "both"`: Both backends must succeed
- `glr_required = true`: Multi-action cells must be preserved (GLR-only)

### Runtime Selection

```rust
// TS-LR mode (explicit)
let mut parser = Parser::new();
parser.set_language(ts_language)?;
let tree = parser.parse(source, None)?;

// GLR mode (explicit)
let parser = Parser::load_glr_table_from_bytes(parsetable_bytes)?;
let tree = parser.parse_utf8(source)?;

// Feature-gated (compile-time)
#[cfg(feature = "glr")]
let parser = Parser::with_glr_engine(table);

#[cfg(not(feature = "glr"))]
let parser = Parser::new(); // Falls back to TS-LR
```

---

## Guarantees and Invariants

### TS-LR Mode Guarantees

1. **ABI Compatibility**: 100% compatible with Tree-sitter C runtime
2. **Single Action**: Every `(state, symbol)` has exactly one action
3. **LR Semantics**: Standard LR parsing, deterministic results
4. **Incremental**: Full incremental parsing support
5. **Queries**: Full Tree-sitter query language support

### GLR Mode Guarantees

1. **Conflict Preservation**: All shift/reduce and reduce/reduce conflicts preserved in tables
2. **Multi-Action Cells**: Tables contain 1..N actions per cell where conflicts exist
3. **Forest Construction**: All valid parse paths explored
4. **Serialization Round-Trip**: Generate → `.parsetable` → deserialize → parse (no data loss)
5. **Disambiguation**: Configurable strategies (First, PreferShift, PreferReduce, Forest)

### Cross-Mode Guarantees

1. **Tree API Compatibility**: Same `Tree` type and methods
2. **IR Compatibility**: Same `Grammar` input format
3. **Error Handling**: Compatible error types and messages
4. **WASM Support**: Both modes work in WASM environments
5. **Memory Safety**: Both modes maintain Rust memory safety

---

## Limitations by Design

### TS-LR Mode Limitations

These are **not bugs**, they are **Tree-sitter ABI constraints**:

❌ **No Multi-Parse Forest**: The ABI has no notion of multi-action cells
❌ **Conflicts Resolved Early**: Disambiguation happens at tablegen, not runtime
❌ **Single Tree Only**: No access to alternative parse trees
❌ **Limited Conflict Inspection**: Cannot query conflict sets at runtime

**Why accept these?**: Because we provide 100% Tree-sitter compatibility. Users who need GLR features can use GLR mode.

### GLR Mode Limitations (Current)

These are **work in progress**, not fundamental limitations:

⏳ **Incremental Parsing**: GLR-aware subtree reuse not yet implemented (v0.7.0 target)
⏳ **Query System**: Forest-compatible queries not yet available (vNext)
⚠️ **External Scanners**: Partial support, Rust-native API evolving
⏳ **Performance**: GLR fork/merge has overhead vs LR (optimization ongoing)

**Roadmap**: These will be addressed in v0.7.0 (incremental) and vNext (queries, optimization).

---

## Testing Strategy

### TS-LR Mode Tests

**Location**: `runtime/tests/`

```rust
#[test]
fn test_ts_lr_incremental_parsing() {
    let mut parser = Parser::new();
    parser.set_language(language()).unwrap();

    let tree = parser.parse("fn main() {}", None).unwrap();
    // ... incremental edits, verify Tree-sitter behavior
}
```

**Coverage**:
- Incremental parsing scenarios
- Query system validation
- External scanner integration
- Tree-sitter ABI parity

### GLR Mode Tests

**Location**: `runtime2/tests/`, `glr-core/tests/`

```rust
#[test]
fn test_glr_ambiguous_parse() {
    let parser = Parser::load_glr_table_from_bytes(table).unwrap();
    let forest = parser.parse_to_forest("if a then if b then s1 else s2").unwrap();

    assert!(forest.roots().len() >= 2); // Multiple parses
}
```

**Coverage**:
- Multi-action cell preservation
- Ambiguous grammar parsing
- Forest construction and traversal
- Conflict inspection API
- BDD scenarios (dangling-else, etc.)

### Cross-Mode Tests

**Location**: `integration/tests/`

```rust
#[test]
fn test_same_tree_api_both_modes() {
    // Parse with TS-LR
    let tree_lr = parse_with_ts_lr(source);

    // Parse with GLR (disambiguated)
    let tree_glr = parse_with_glr(source);

    // Verify same Tree API
    assert_eq!(tree_lr.root_node().kind(), tree_glr.root_node().kind());
}
```

---

## Migration Guidance

### From Tree-sitter to adze TS-LR

**Zero code changes required** for basic parsing:

```rust
// Tree-sitter C
let mut parser = ts::Parser::new();
parser.set_language(tree_sitter_json()).unwrap();
let tree = parser.parse(source, None).unwrap();

// adze TS-LR (identical API)
let mut parser = Parser::new();
parser.set_language(language()).unwrap();
let tree = parser.parse(source, None).unwrap();
```

### From TS-LR to GLR (for ambiguous grammars)

**Minimal code changes**:

```rust
// TS-LR (single parse)
let mut parser = Parser::new();
parser.set_language(language()).unwrap();
let tree = parser.parse(source, None).unwrap();

// GLR (multi-parse capable)
let parser = Parser::load_glr_table_from_bytes(include_bytes!("grammar.parsetable")).unwrap();
let tree = parser.parse_utf8(source.as_bytes()).unwrap(); // Default: first tree

// OR: Access full forest
let forest = parser.parse_to_forest(source).unwrap();
for tree in forest.trees() {
    // Process each parse
}
```

---

## Architecture Decision Record

**ADR-001: Dual Runtime Architecture**

**Context**: We need to support both Tree-sitter compatibility and true GLR semantics.

**Decision**: Maintain two runtime modes with shared IR and tree API:
1. TS-LR mode for Tree-sitter compatibility
2. GLR mode for ambiguous grammar support

**Consequences**:
- ✅ **Pro**: Clean separation of concerns
- ✅ **Pro**: 100% Tree-sitter compatibility without compromise
- ✅ **Pro**: True GLR semantics without ABI limitations
- ⚠️ **Con**: Two code paths to maintain
- ⚠️ **Con**: User must choose mode explicitly

**Alternatives Considered**:
1. **GLR-only**: Would lose Tree-sitter compatibility, breaking existing users
2. **TS-ABI-based GLR**: Impossible due to single-action-per-cell limitation
3. **Unified runtime with flag**: Would complicate both paths, unclear semantics

**Status**: ACCEPTED (2025-11-20)

---

## References

### Internal Documents
- [GLR_V1_COMPLETION_CONTRACT.md](./GLR_V1_COMPLETION_CONTRACT.md) - GLR v1 acceptance criteria
- [PARSETABLE_FILE_FORMAT_SPEC.md](./PARSETABLE_FILE_FORMAT_SPEC.md) - `.parsetable` binary format
- [PARSER_V4_TABLE_LOADING_BLOCKER.md](../plans/PARSER_V4_TABLE_LOADING_BLOCKER.md) - Why we need dual modes
- [GLR_POSITIONING_VS_OTHER_TOOLS.md](./GLR_POSITIONING_VS_OTHER_TOOLS.md) - Competitive positioning

### External References
- [Tree-sitter Documentation](https://tree-sitter.github.io/tree-sitter/)
- [GLR Parsing (Wikipedia)](https://en.wikipedia.org/wiki/GLR_parser)
- [Tree-sitter TSLanguage ABI](https://github.com/tree-sitter/tree-sitter/blob/master/lib/include/tree_sitter/api.h)

---

**Next Steps**:
1. Update GLR_V1_COMPLETION_CONTRACT.md to reference this document
2. Add runtime mode selection examples to QUICK_START.md
3. Create comparison benchmarks (TS-LR vs GLR performance)
4. Document migration paths for common use cases

---

END OF SPECIFICATION
