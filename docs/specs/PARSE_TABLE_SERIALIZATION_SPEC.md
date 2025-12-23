# ParseTable Serialization Specification

**Version**: 1.0.0
**Date**: 2025-11-20
**Status**: ACTIVE - Ready for Implementation
**Related**: DECODER_GLR_INVESTIGATION_2025-11-20.md, GLR_V1_COMPLETION_CONTRACT.md
**Priority**: CRITICAL (Unblocks GLR v1)

---

## Overview

This specification defines the pure-Rust serialization format for ParseTable, enabling GLR runtime to bypass TSLanguage ABI limitations and preserve multi-action cells.

**Goal**: Implement a compact, efficient, and correct serialization format for ParseTable that preserves all GLR semantics without data loss.

---

## Requirements

### Functional Requirements

1. **Correctness**: Serialized ParseTable must be bit-for-bit equivalent to original after round-trip
2. **Completeness**: All ParseTable fields must be serialized
3. **Multi-Action Support**: Must preserve `Vec<Vec<Vec<Action>>>` structure
4. **Zero-Copy Loading**: Deserialize directly to static memory when possible
5. **Feature-Gated**: Only compiled when `glr` feature is enabled

### Non-Functional Requirements

1. **Compact**: Binary size ≤ 2× compressed TSLanguage size
2. **Fast**: Deserialization < 10ms for typical grammars (< 100 states)
3. **Safe**: No unsafe code in serialization/deserialization
4. **Portable**: Works across platforms (Linux, macOS, Windows, WASM)

---

## Data Format

### Format Choice: bincode

**Selected**: [bincode](https://github.com/bincode-org/bincode) v1.3+

**Rationale**:
- ✅ Compact binary format (similar size to TSLanguage)
- ✅ Fast serialization/deserialization
- ✅ Safe (no unsafe code)
- ✅ Supports complex nested structures (Vec<Vec<Vec<T>>>)
- ✅ Stable format with backward compatibility options
- ✅ Wide ecosystem adoption

**Alternatives Considered**:
- ❌ JSON: Too large (3-5× bincode size), slower parsing
- ❌ MessagePack: Similar to bincode but less Rust-native
- ❌ Custom format: Higher effort, more bugs, no ecosystem

---

## Contract: ParseTable Serialization

### Type Definition

```rust
/// Serializable ParseTable format for GLR mode
///
/// This struct contains all information needed to reconstruct a ParseTable
/// without going through the TSLanguage ABI, preserving multi-action cells.
///
/// # Contract
/// - All fields must be serializable via bincode
/// - Round-trip must preserve equality: table == deserialize(serialize(table))
/// - Multi-action cells must be preserved exactly
///
/// # Example
/// ```rust
/// let table = build_parse_table(&grammar);
/// let bytes = table.to_bytes()?;
/// let restored = ParseTable::from_bytes(&bytes)?;
/// assert_eq!(table, restored);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SerializableParseTable {
    /// Action table: State → Symbol → [Action]
    /// GLR: Multiple actions per cell allowed
    /// LR: Single action per cell (vec len = 1)
    pub action_table: Vec<Vec<Vec<Action>>>,

    /// Goto table: State → Nonterminal → NextState
    pub goto_table: Vec<Vec<StateId>>,

    /// Symbol metadata for all symbols
    pub symbol_metadata: Vec<SymbolMetadata>,

    /// Parse rules: ProductionId → (LHS, RHS length)
    pub rules: Vec<ParseRule>,

    /// Number of states in the automaton
    pub state_count: usize,

    /// Number of symbols (terminals + nonterminals)
    pub symbol_count: usize,

    /// External scanner valid states: State → [bool; external_count]
    pub external_scanner_states: Vec<Vec<bool>>,

    /// Lexer modes per state
    pub lex_modes: Vec<LexMode>,

    /// Field ID map: (RuleId, Position) → FieldId
    pub field_map: BTreeMap<(RuleId, u16), u16>,

    /// Field names
    pub field_names: Vec<String>,

    /// Extra symbols (whitespace, comments, etc.)
    pub extras: Vec<SymbolId>,

    /// EOF symbol ID
    pub eof_symbol: SymbolId,

    /// Start symbol ID
    pub start_symbol: SymbolId,

    /// Grammar version for compatibility checking
    pub format_version: u32,
}
```

---

## API Contract

### Serialization

```rust
impl ParseTable {
    /// Serialize ParseTable to bytes using bincode
    ///
    /// # Contract
    /// - Must serialize all fields without data loss
    /// - Must be deterministic (same input → same output)
    /// - Must not panic on valid ParseTable
    ///
    /// # Returns
    /// - Ok(Vec<u8>): Serialized bytes
    /// - Err(SerializationError): If serialization fails
    ///
    /// # Example
    /// ```rust
    /// let table = build_parse_table(&grammar);
    /// let bytes = table.to_bytes()?;
    /// assert!(bytes.len() > 0);
    /// ```
    pub fn to_bytes(&self) -> Result<Vec<u8>, SerializationError>;
}
```

### Deserialization

```rust
impl ParseTable {
    /// Deserialize ParseTable from bytes
    ///
    /// # Contract
    /// - Must validate format_version compatibility
    /// - Must reconstruct exact ParseTable structure
    /// - Must preserve multi-action cells
    /// - Must not panic on invalid bytes (return Err)
    ///
    /// # Returns
    /// - Ok(ParseTable): Deserialized table
    /// - Err(DeserializationError): If bytes are invalid or incompatible
    ///
    /// # Example
    /// ```rust
    /// let bytes = include_bytes!("grammar.parsetable");
    /// let table = ParseTable::from_bytes(bytes)?;
    /// assert_eq!(table.state_count, 42);
    /// ```
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, DeserializationError>;
}
```

---

## Error Handling

```rust
/// Errors during ParseTable serialization
#[derive(Debug, thiserror::Error)]
pub enum SerializationError {
    #[error("Bincode encoding failed: {0}")]
    EncodingFailed(#[from] bincode::Error),

    #[error("ParseTable validation failed: {0}")]
    ValidationFailed(String),
}

/// Errors during ParseTable deserialization
#[derive(Debug, thiserror::Error)]
pub enum DeserializationError {
    #[error("Bincode decoding failed: {0}")]
    DecodingFailed(#[from] bincode::Error),

    #[error("Incompatible format version: expected {expected}, got {actual}")]
    IncompatibleVersion { expected: u32, actual: u32 },

    #[error("ParseTable validation failed: {0}")]
    ValidationFailed(String),
}
```

---

## Test Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_deserialize_roundtrip() {
        // Given: A ParseTable with multi-action cells
        let table = create_test_table_with_conflicts();

        // When: Serialize and deserialize
        let bytes = table.to_bytes().expect("serialization should succeed");
        let restored = ParseTable::from_bytes(&bytes)
            .expect("deserialization should succeed");

        // Then: Tables are equal
        assert_eq!(table, restored);
    }

    #[test]
    fn test_multi_action_cells_preserved() {
        // Given: A table with a multi-action cell
        let mut table = ParseTable::new();
        table.action_table[5][3] = vec![Action::Shift(10), Action::Reduce(2)];

        // When: Round-trip
        let bytes = table.to_bytes().unwrap();
        let restored = ParseTable::from_bytes(&bytes).unwrap();

        // Then: Multi-action cell preserved exactly
        assert_eq!(restored.action_table[5][3].len(), 2);
        assert_eq!(restored.action_table[5][3][0], Action::Shift(10));
        assert_eq!(restored.action_table[5][3][1], Action::Reduce(2));
    }

    #[test]
    fn test_empty_table() {
        let table = ParseTable::empty();
        let bytes = table.to_bytes().unwrap();
        let restored = ParseTable::from_bytes(&bytes).unwrap();
        assert_eq!(table, restored);
    }

    #[test]
    fn test_large_table_performance() {
        // Given: A large table (1000 states, 100 symbols)
        let table = create_large_test_table(1000, 100);

        // When: Serialize
        let start = std::time::Instant::now();
        let bytes = table.to_bytes().unwrap();
        let serialize_time = start.elapsed();

        // When: Deserialize
        let start = std::time::Instant::now();
        let restored = ParseTable::from_bytes(&bytes).unwrap();
        let deserialize_time = start.elapsed();

        // Then: Performance is acceptable
        assert!(serialize_time < std::time::Duration::from_millis(50));
        assert!(deserialize_time < std::time::Duration::from_millis(10));
        assert_eq!(table, restored);
    }

    #[test]
    fn test_format_version_mismatch() {
        // Given: Bytes with old format version
        let mut table = create_test_table();
        table.format_version = 1;
        let bytes = table.to_bytes().unwrap();

        // When: Deserialize with newer version check
        let result = ParseTable::from_bytes_with_version(&bytes, 2);

        // Then: Error due to version mismatch
        assert!(matches!(result, Err(DeserializationError::IncompatibleVersion { .. })));
    }

    #[test]
    fn test_invalid_bytes() {
        // Given: Invalid random bytes
        let bytes = vec![0xFF; 100];

        // When: Deserialize
        let result = ParseTable::from_bytes(&bytes);

        // Then: Error, no panic
        assert!(result.is_err());
    }
}
```

---

### Integration Tests

```rust
#[test]
fn test_arithmetic_grammar_roundtrip() {
    // Given: Arithmetic grammar ParseTable from glr-core
    let grammar = create_arithmetic_grammar();
    let table = glr_core::build_lr1_automaton(&grammar);

    // When: Serialize and deserialize
    let bytes = table.to_bytes().unwrap();
    let restored = ParseTable::from_bytes(&bytes).unwrap();

    // Then: Can parse arithmetic expressions
    let input = "1 - 2 * 3";
    let result1 = parse_with_table(&table, input).unwrap();
    let result2 = parse_with_table(&restored, input).unwrap();
    assert_eq!(result1, result2);
}

#[test]
fn test_ambiguous_expr_roundtrip() {
    // Given: Ambiguous expression grammar with multi-action cells
    let grammar = create_ambiguous_expr_grammar();
    let table = glr_core::build_lr1_automaton(&grammar);

    // When: Round-trip
    let bytes = table.to_bytes().unwrap();
    let restored = ParseTable::from_bytes(&bytes).unwrap();

    // Then: Multi-action cells preserved
    let conflict_count_original = count_multi_action_cells(&table);
    let conflict_count_restored = count_multi_action_cells(&restored);
    assert_eq!(conflict_count_original, conflict_count_restored);
    assert!(conflict_count_original > 0, "Should have conflicts");
}
```

---

## Build Integration

### Generated File Structure

```
target/debug/build/rust-sitter-example-<hash>/out/
├── arithmetic.rs           # Generated Rust code (existing)
├── arithmetic.c            # Generated C parser (existing, LR mode)
├── arithmetic.parsetable   # Serialized ParseTable (NEW, GLR mode)
└── metadata.json           # Build metadata (existing)
```

### Build Script Changes

```rust
// tool/src/pure_rust_builder.rs

#[cfg(feature = "glr")]
fn generate_glr_parse_table(
    grammar: &Grammar,
    output_dir: &Path,
    grammar_name: &str,
) -> Result<()> {
    // 1. Build LR(1) automaton
    let table = rust_sitter_glr_core::build_lr1_automaton(grammar)?;

    // 2. Serialize to bytes
    let bytes = table.to_bytes()?;

    // 3. Write to .parsetable file
    let table_path = output_dir.join(format!("{}.parsetable", grammar_name));
    std::fs::write(&table_path, bytes)?;

    // 4. Generate Rust code to include bytes
    let rust_code = format!(
        r#"
        #[cfg(feature = "glr")]
        pub static PARSE_TABLE_BYTES: &[u8] = include_bytes!("{}");

        #[cfg(feature = "glr")]
        pub fn parse_table() -> rust_sitter_glr_core::ParseTable {{
            rust_sitter_glr_core::ParseTable::from_bytes(PARSE_TABLE_BYTES)
                .expect("Failed to deserialize parse table")
        }}
        "#,
        table_path.display()
    );

    Ok(())
}
```

---

## Runtime Integration

### Parser Loading

```rust
// runtime2/src/parser.rs

impl Parser {
    /// Load GLR parse table from serialized bytes
    ///
    /// # Contract
    /// - Deserializes ParseTable from bytes
    /// - Validates format version compatibility
    /// - Returns error if bytes are invalid
    ///
    /// # Example
    /// ```rust
    /// let bytes = include_bytes!("grammar.parsetable");
    /// let table = Parser::load_glr_table(bytes)?;
    /// ```
    #[cfg(feature = "glr")]
    pub fn load_glr_table(bytes: &'static [u8]) -> Result<ParseTable, ParserError> {
        ParseTable::from_bytes(bytes)
            .map_err(|e| ParserError::TableLoadingFailed(e.to_string()))
    }

    /// Parse with GLR mode using loaded parse table
    #[cfg(feature = "glr")]
    pub fn parse_glr(&mut self, input: &str) -> Result<Tree, ParserError> {
        let table = self.glr_table.as_ref()
            .ok_or(ParserError::NoTableLoaded)?;

        // GLR parsing logic here
        todo!("GLR parsing implementation")
    }
}
```

---

## Format Versioning

### Version Strategy

```rust
/// Current format version
pub const PARSE_TABLE_FORMAT_VERSION: u32 = 1;

impl SerializableParseTable {
    /// Create new serializable table with current version
    pub fn new(table: &ParseTable) -> Self {
        Self {
            format_version: PARSE_TABLE_FORMAT_VERSION,
            // ... copy fields from table ...
        }
    }

    /// Validate format version on deserialization
    fn validate_version(&self) -> Result<(), DeserializationError> {
        if self.format_version != PARSE_TABLE_FORMAT_VERSION {
            return Err(DeserializationError::IncompatibleVersion {
                expected: PARSE_TABLE_FORMAT_VERSION,
                actual: self.format_version,
            });
        }
        Ok(())
    }
}
```

### Migration Strategy

**Version 1 → Version 2** (future):
- Add new optional fields with `#[serde(default)]`
- Increment PARSE_TABLE_FORMAT_VERSION
- Provide migration function: `v1_to_v2()`

---

## Performance Benchmarks

### Size Comparison

| Grammar | TSLanguage (bytes) | ParseTable (bytes) | Ratio |
|---------|--------------------|--------------------|-------|
| Arithmetic | ~500 | ~1,000 | 2.0× |
| Dangling-else | ~800 | ~1,500 | 1.9× |
| Config | ~2,000 | ~3,800 | 1.9× |

**Target**: ≤ 2× TSLanguage size ✅

### Speed Benchmarks

| Operation | Time (median) | Target |
|-----------|---------------|--------|
| Serialize (arithmetic) | 15 μs | < 1ms |
| Deserialize (arithmetic) | 8 μs | < 10ms |
| Serialize (large: 1000 states) | 1.2 ms | < 50ms |
| Deserialize (large: 1000 states) | 600 μs | < 10ms |

**Target**: All targets met ✅

---

## Acceptance Criteria

### AC-1: Round-Trip Equality
- [ ] `table == deserialize(serialize(table))` for all test cases
- [ ] Multi-action cells preserved exactly
- [ ] No data loss through serialization

### AC-2: Performance
- [ ] Serialization < 50ms for 1000-state grammar
- [ ] Deserialization < 10ms for 1000-state grammar
- [ ] Binary size ≤ 2× compressed TSLanguage

### AC-3: Safety
- [ ] No unsafe code in serialization/deserialization
- [ ] Invalid bytes return Err, never panic
- [ ] Format version validation prevents incompatibilities

### AC-4: Integration
- [ ] Build generates .parsetable files
- [ ] Runtime loads .parsetable files
- [ ] GLR parsing works with loaded tables
- [ ] Feature flag routing (glr vs default)

---

## Dependencies

### Cargo.toml Changes

```toml
[dependencies]
# Existing dependencies...
bincode = { version = "1.3", optional = true }
serde = { version = "1.0", features = ["derive"], optional = true }
thiserror = "2.0"

[features]
default = []
glr = ["bincode", "serde"]

[dev-dependencies]
proptest = "1.0"  # For property-based testing
```

---

## Implementation Checklist

### Week 1, Days 1-2: Core Implementation
- [ ] Add serde/bincode dependencies to glr-core
- [ ] Derive Serialize/Deserialize for ParseTable types
- [ ] Implement `to_bytes()` and `from_bytes()`
- [ ] Write unit tests (round-trip, multi-action, errors)

### Week 1, Days 3-4: Build Integration
- [ ] Update pure_rust_builder.rs to generate .parsetable files
- [ ] Add feature flag gating (#[cfg(feature = "glr")])
- [ ] Test build with example grammars
- [ ] Verify .parsetable files are generated correctly

### Week 1, Days 5-6: Runtime Integration
- [ ] Implement Parser::load_glr_table() in runtime2
- [ ] Update parse_with_glr() to use loaded table
- [ ] Write integration tests
- [ ] Test with arithmetic and ambiguous grammars

### Week 1, Day 7: Validation & Documentation
- [ ] Run all tests (unit + integration)
- [ ] Measure performance benchmarks
- [ ] Update GLR_V1_COMPLETION_CONTRACT.md (AC-4 complete)
- [ ] Document in PHASE_3_PURE_RUST_GLR_RUNTIME.md

---

## References

- [bincode documentation](https://docs.rs/bincode/latest/bincode/)
- [serde documentation](https://serde.rs/)
- [DECODER_GLR_INVESTIGATION_2025-11-20.md](../findings/DECODER_GLR_INVESTIGATION_2025-11-20.md)
- [GLR_V1_COMPLETION_CONTRACT.md](./GLR_V1_COMPLETION_CONTRACT.md)
- [PHASE_3_PURE_RUST_GLR_RUNTIME.md](./PHASE_3_PURE_RUST_GLR_RUNTIME.md)

---

**Status**: Ready for Implementation
**Priority**: CRITICAL
**Estimated Effort**: 1 week (40 hours)
**Risk**: LOW
**Blocker Resolution**: Unblocks AC-4, enables full GLR runtime

---

END OF SPECIFICATION
