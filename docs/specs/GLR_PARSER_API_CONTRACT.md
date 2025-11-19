# GLR Parser API Contract

**Status**: Phase 3.1 - Core GLR Runtime
**Feature Flag**: `pure-rust-glr`
**Dependencies**: `glr-core`, `rust-sitter-ir`

---

## Overview

This document defines the contract for the pure-Rust GLR parser API that bypasses TSLanguage encoding and uses ParseTable directly.

**Design Principle**: Provide a safe, ergonomic API for GLR parsing while maintaining Tree-sitter API compatibility where possible.

---

## API Surface

### Parser::set_glr_table()

**Signature**:
```rust
#[cfg(feature = "pure-rust-glr")]
impl Parser {
    pub fn set_glr_table(
        &mut self,
        table: &'static rust_sitter_glr_core::ParseTable
    ) -> Result<(), ParseError>
}
```

**Contract**:
- **Preconditions**:
  - `table` must be a valid ParseTable from glr-core
  - `table.state_count > 0`
  - `table.action_table.len() == table.state_count`
  - ParseTable invariants must hold (documented in CONFLICT_INSPECTION_API.md)

- **Postconditions**:
  - Parser is configured to use GLR mode
  - Subsequent `parse()` calls will use GLR engine
  - No TSLanguage encoding/decoding occurs
  - Multi-action cells are preserved

- **Error Conditions**:
  - `ParseError::InvalidTable`: If table violates invariants
  - `ParseError::MissingMetadata`: If symbol metadata not provided

**Invariants**:
- Once `set_glr_table()` is called, `language` field may be partially populated
- `set_language()` and `set_glr_table()` are mutually exclusive modes
- Calling `set_language()` after `set_glr_table()` switches back to LR mode

---

### Parser Mode Selection

**Behavior**:
```rust
// Mode 1: LR mode via Language (TSLanguage encoding)
let mut parser = Parser::new();
parser.set_language(language)?;
let tree = parser.parse(input, None)?;  // Uses LR runtime

// Mode 2: GLR mode via ParseTable (pure-Rust, no TSLanguage)
#[cfg(feature = "pure-rust-glr")]
{
    let mut parser = Parser::new();
    parser.set_glr_table(&parse_table)?;
    parser.set_symbol_metadata(metadata);
    let tree = parser.parse(input, None)?;  // Uses GLR runtime
}
```

**Contract**:
- Parser maintains internal state to track which mode is active
- `parse()` method automatically routes to correct implementation
- Mode is determined by which setter was called last

---

### Parser::set_symbol_metadata()

**Signature**:
```rust
#[cfg(feature = "pure-rust-glr")]
impl Parser {
    pub fn set_symbol_metadata(
        &mut self,
        metadata: Vec<SymbolMetadata>
    ) -> Result<(), ParseError>
}
```

**Contract**:
- **Preconditions**:
  - `metadata.len()` must match symbol count in ParseTable
  - Each SymbolMetadata must be valid

- **Postconditions**:
  - Symbol metadata available for tree construction
  - Node visibility correctly determined

**Purpose**: GLR mode needs symbol metadata for tree construction, but doesn't use full Language struct.

---

### Parser::parse() - GLR Mode

**Contract for GLR Mode**:

**Preconditions**:
- `set_glr_table()` was called successfully
- `set_symbol_metadata()` was called (optional but recommended)
- Input is valid UTF-8 or binary data

**Behavior**:
1. Tokenize input using default tokenizer (lexical scanning)
2. Run GLR engine with ParseTable
3. Fork on conflicts (multiple stacks maintained)
4. Merge identical stacks (GSS optimization)
5. Produce parse forest (multiple trees if ambiguous)
6. Disambiguate to select single tree
7. Convert forest to Tree

**Postconditions**:
- Returns `Ok(Tree)` if parsing succeeds
- Tree structure matches grammar productions
- All nodes have correct byte ranges
- `ParseError` if:
  - Syntax error (no valid parse)
  - Timeout exceeded
  - Memory limit exceeded (future)

**Performance Guarantees**:
- For unambiguous grammars: ≤ 2x slower than LR mode
- For ambiguous grammars: Polynomial time (O(n³) worst case)
- Memory: O(n * conflicts) where n is input length

---

## Data Structures

### GLRState (Internal)

```rust
#[cfg(feature = "pure-rust-glr")]
struct GLRState {
    /// Direct reference to ParseTable (no copying)
    parse_table: &'static rust_sitter_glr_core::ParseTable,
    /// Symbol metadata for tree construction
    symbol_metadata: Vec<SymbolMetadata>,
    /// Optional tokenizer (defaults to simple lexical scanner)
    tokenizer: Option<Box<dyn Tokenizer>>,
}
```

**Invariants**:
- `parse_table` lifetime is `'static` (embedded in binary or Arc'd)
- `symbol_metadata.len()` matches symbol count in parse_table
- `tokenizer` is Some if custom tokenization needed

---

### ParserMode (Internal)

```rust
enum ParserMode {
    /// LR mode: uses Language with TSLanguage encoding
    LR(Language),
    /// GLR mode: uses ParseTable directly
    #[cfg(feature = "pure-rust-glr")]
    GLR(GLRState),
    /// Not configured
    Unset,
}
```

**State Transitions**:
```
Unset --set_language()--> LR
Unset --set_glr_table()--> GLR
LR --set_glr_table()--> GLR
GLR --set_language()--> LR
```

---

## Error Handling

### ParseError Variants (New)

```rust
#[cfg(feature = "pure-rust-glr")]
pub enum ParseError {
    /// ParseTable is invalid (violates invariants)
    InvalidTable {
        reason: String,
    },
    /// Symbol metadata missing or mismatched
    MissingMetadata {
        expected: usize,
        actual: usize,
    },
    /// GLR fork limit exceeded
    TooManyForks {
        limit: usize,
    },
    /// Parse forest too large
    ForestTooLarge {
        node_count: usize,
        limit: usize,
    },
    // ... existing variants
}
```

**Error Recovery**:
- `InvalidTable`: Programming error, should not happen in production
- `MissingMetadata`: Configuration error, clear diagnostic
- `TooManyForks`: Runtime limit, can be increased via config
- `ForestTooLarge`: Memory protection, indicates pathological grammar

---

## Configuration

### GLRConfig (Future)

```rust
#[cfg(feature = "pure-rust-glr")]
pub struct GLRConfig {
    /// Maximum number of parallel parser stacks
    pub max_forks: usize,  // Default: 1000
    /// Maximum parse forest nodes
    pub max_forest_nodes: usize,  // Default: 10000
    /// Parsing timeout
    pub timeout: Option<Duration>,  // Default: None
    /// Disambiguation strategy
    pub disambiguation: DisambiguationStrategy,  // Default: PreferShift
}
```

**Contract**:
- Config is validated when set
- Invalid config returns `ParseError::InvalidConfig`
- Config can be changed between parses

---

## Usage Patterns

### Pattern 1: Simple GLR Parsing

```rust
use rust_sitter_runtime::Parser;
use rust_sitter_glr_core::build_lr1_automaton;
use rust_sitter_ir::builder::GrammarBuilder;

// Build grammar
let mut grammar = GrammarBuilder::new("expr")
    .token("NUM", r"\d+")
    .token("+", "+")
    .rule("expr", vec!["expr", "+", "expr"])
    .rule("expr", vec!["NUM"])
    .start("expr")
    .build();

// Generate ParseTable
let first_follow = FirstFollowSets::compute_normalized(&mut grammar)?;
let parse_table = build_lr1_automaton(&grammar, &first_follow)?;

// Parse with GLR
let mut parser = Parser::new();
parser.set_glr_table(&parse_table)?;
let tree = parser.parse(b"1 + 2 + 3", None)?;

assert!(tree.root_node().is_some());
```

### Pattern 2: Embedded ParseTable

```rust
// In generated parser crate:
pub static PARSE_TABLE: rust_sitter_glr_core::ParseTable = /* ... */;
pub static SYMBOL_METADATA: &[SymbolMetadata] = /* ... */;

// User code:
use my_generated_parser::{PARSE_TABLE, SYMBOL_METADATA};

let mut parser = Parser::new();
parser.set_glr_table(&PARSE_TABLE)?;
parser.set_symbol_metadata(SYMBOL_METADATA.to_vec())?;
let tree = parser.parse(input, None)?;
```

### Pattern 3: Hybrid Mode (Feature Detection)

```rust
let mut parser = Parser::new();

#[cfg(feature = "pure-rust-glr")]
{
    parser.set_glr_table(&PARSE_TABLE)?;
    parser.set_symbol_metadata(metadata.to_vec())?;
}

#[cfg(not(feature = "pure-rust-glr"))]
{
    parser.set_language(create_language())?;
}

let tree = parser.parse(input, None)?;
```

---

## Testing Contracts

### Unit Tests

```rust
#[test]
#[cfg(feature = "pure-rust-glr")]
fn test_set_glr_table_accepts_valid_table() {
    let mut parser = Parser::new();
    let table = create_valid_parse_table();

    let result = parser.set_glr_table(&table);
    assert!(result.is_ok());
}

#[test]
#[cfg(feature = "pure-rust-glr")]
fn test_set_glr_table_rejects_invalid_table() {
    let mut parser = Parser::new();
    let table = create_invalid_parse_table();

    let result = parser.set_glr_table(&table);
    assert!(matches!(result, Err(ParseError::InvalidTable { .. })));
}

#[test]
#[cfg(feature = "pure-rust-glr")]
fn test_mode_switching_works() {
    let mut parser = Parser::new();

    // Start in GLR mode
    parser.set_glr_table(&GLR_TABLE).unwrap();
    // Switch to LR mode
    parser.set_language(create_language()).unwrap();
    // Switch back to GLR mode
    parser.set_glr_table(&GLR_TABLE).unwrap();
}
```

### Integration Tests

```rust
#[test]
#[cfg(feature = "pure-rust-glr")]
fn test_glr_parses_ambiguous_grammar() {
    let grammar = build_ambiguous_expr_grammar();
    let table = build_lr1_automaton(&grammar).unwrap();

    let mut parser = Parser::new();
    parser.set_glr_table(&table).unwrap();

    // This would fail in LR mode due to conflict
    let tree = parser.parse(b"1 + 2 + 3", None).unwrap();

    assert!(tree.root_node().is_some());
}

#[test]
#[cfg(feature = "pure-rust-glr")]
fn test_glr_preserves_conflicts() {
    let table = build_ambiguous_table();
    let summary = count_conflicts(&table);
    assert_eq!(summary.shift_reduce, 1);  // Conflicts exist

    let mut parser = Parser::new();
    parser.set_glr_table(&table).unwrap();

    // Should parse without error (GLR handles conflicts)
    let result = parser.parse(b"1 + 2", None);
    assert!(result.is_ok());
}
```

---

## Performance Contracts

### Latency

**Contract**:
- Simple grammars (< 10 states): < 1ms for 1KB input
- Medium grammars (< 100 states): < 10ms for 1KB input
- Complex grammars (< 1000 states): < 100ms for 1KB input

**Measured for unambiguous grammars** (ambiguous grammars depend on fork count).

### Memory

**Contract**:
- Baseline: < 1MB overhead
- Per-fork: < 10KB
- Per-forest-node: < 100 bytes

**Maximum**:
- Default config: < 100MB for typical inputs
- Can be configured via GLRConfig

### Scalability

**Contract**:
- Linear in input size for unambiguous grammars
- Polynomial (O(n³)) worst-case for ambiguous grammars
- Practical grammars: O(n log n) average case

---

## Compatibility Guarantees

### Tree-sitter API Compatibility

**Guaranteed**:
- `Tree` structure identical to Tree-sitter
- `Node` API fully compatible
- Byte ranges accurate
- Parent/child relationships correct

**Not Guaranteed**:
- Identical parse trees for ambiguous grammars (disambiguation may differ)
- Identical error recovery (GLR has different error model)

### Feature Interaction

**Compatible Features**:
- ✅ `incremental`: GLR supports incremental parsing
- ✅ `arenas`: GLR can use arena allocation
- ✅ `queries`: Query system works on GLR trees

**Incompatible Features**:
- ❌ `external-scanners`: Not yet supported in pure-Rust GLR
- ⚠️ Mixing LR and GLR modes in same parse (use one or the other)

---

## Security Considerations

### Memory Safety

**Guarantees**:
- No unsafe code in API surface
- All lifetimes statically checked
- No use-after-free possible
- Bounded recursion (stack overflow protection)

**Limits**:
- Max fork count prevents fork bombs
- Max forest size prevents memory exhaustion
- Timeout prevents infinite loops

### Input Validation

**Contract**:
- All inputs validated before processing
- Invalid ParseTable rejected at `set_glr_table()`
- Invalid metadata rejected at `set_symbol_metadata()`
- Malformed input produces `ParseError`, not panic

---

## Versioning and Stability

### API Stability

**Phase 3.1 (Current)**:
- `set_glr_table()`: Experimental, may change
- Mark with `#[doc(cfg(feature = "pure-rust-glr"))]`
- Semantic versioning: 0.x.y allows breaking changes

**Phase 3.4 (Stabilization)**:
- API reviewed and locked in
- Add deprecation warnings before breaking changes
- Migration guide for API changes

### ParseTable Compatibility

**Contract**:
- ParseTable format follows glr-core version
- Breaking changes to ParseTable = major version bump
- Forward compatibility not guaranteed (older runtime can't parse newer tables)

---

## References

- [PHASE_3_PURE_RUST_GLR_RUNTIME.md](./PHASE_3_PURE_RUST_GLR_RUNTIME.md) - Architecture
- [CONFLICT_INSPECTION_API.md](./CONFLICT_INSPECTION_API.md) - ParseTable invariants
- [runtime2/src/parser.rs](../../runtime2/src/parser.rs) - Implementation
- [glr-core validation test](../../glr-core/tests/diagnose_ambiguous_expr.rs) - ParseTable validation

---

**Status**: Contract Defined - Ready for Implementation
**Next**: Implement `Parser::set_glr_table()` stub and tests

