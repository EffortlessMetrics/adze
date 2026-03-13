# ADR 014: Parse Table Compression Strategy

**Status**: Accepted
**Date**: 2025-03-13
**Authors**: adze maintainers
**Related**: ADR-006 (Tree-sitter Compatibility Layer), ADR-011 (Parse Table Binary Format)

## Context

Parse tables for real-world grammars can contain thousands of states and symbols, resulting in sparse matrices that waste memory if stored naively. A Python grammar, for example, may have 3000+ states and 500+ symbols, creating a theoretical table size of 1.5 million entries per table (ACTION and GOTO).

Tree-sitter employs a specific compression scheme with magic constants that must be replicated for ABI compatibility. The compression module in [`tablegen/src/compress.rs`](../../tablegen/src/compress.rs) implements this scheme, while [`tablegen/src/abi.rs`](../../tablegen/src/abi.rs) defines the ABI-compatible structures.

### Key Constraints

1. **ABI Compatibility**: Generated tables must be readable by Tree-sitter's C runtime
2. **Memory Efficiency**: Tables should be as small as possible for fast loading
3. **Lookup Performance**: Compression must not significantly degrade parse speed
4. **Lossless Compression**: All parse actions must be preserved exactly

## Decision

We adopt **Tree-sitter's compression scheme** with the following components:

### 1. Small vs Large Table Threshold

```rust
const small_table_threshold: usize = 32768; // Tree-sitter's magic constant
```

Tables smaller than 32KB use direct encoding; larger tables use compressed encoding.

### 2. Action Encoding

For small tables, actions are encoded directly as `u16`:
- Shift: `state_id` (0x0000-0x7FFF)
- Reduce: `0x8000 | rule_id` (0x8000-0xBFFF)
- Accept: `0xC000`
- Error: `0x0000` (explicit error action)

```rust
pub fn encode_action_small(&self, action: &Action) -> Result<u16> {
    match action {
        Action::Shift(state) => Ok(state.0),
        Action::Reduce(rule) => Ok(0x8000 | rule.0 as u16),
        // ...
    }
}
```

### 3. Row Offset Compression

Sparse rows are compressed using:
- **Row offsets**: Index into the compressed data array
- **Default actions**: Most common action per row stored separately
- **Symbol-indexed lookup**: Symbols map to positions within compressed rows

```rust
pub struct CompressedActionTable {
    pub data: Vec<CompressedActionEntry>,
    pub row_offsets: Vec<u16>,
    pub default_actions: Vec<Action>,
}
```

### 4. Goto Table Compression

Goto tables use run-length encoding for consecutive identical entries:

```rust
pub enum CompressedGotoEntry {
    Single(u16),
    RunLength { state: u16, count: u16 },
}
```

### 5. ABI-Compatible Structures

The [`TSLanguage`](../../tablegen/src/abi.rs:50) structure matches Tree-sitter's ABI v15 exactly:

```rust
#[repr(C)]
pub struct TSLanguage {
    pub version: u32,
    pub symbol_count: u32,
    pub state_count: u32,
    pub parse_table: *const u16,
    pub small_parse_table: *const u16,
    pub parse_actions: *const TSParseAction,
    // ... additional fields for ABI 15
}
```

Compile-time assertions verify size compatibility:
```rust
const _: () = {
    assert!(mem::size_of::<TSSymbol>() == 2);
    assert!(mem::size_of::<TSParseAction>() == 6);
};
```

## Consequences

### Positive

- **ABI Compatibility**: Generated parsers work with Tree-sitter's C runtime
- **Memory Efficiency**: Compression reduces table sizes by 60-80% for typical grammars
- **Fast Loading**: Compressed tables load quickly without decompression
- **Deterministic**: Same grammar always produces identical compressed output

### Negative

- **Complexity**: Compression logic is intricate and hard to modify
- **Magic Constants**: Thresholds must match Tree-sitter exactly
- **Debugging Difficulty**: Compressed tables are harder to inspect
- **Maintenance Burden**: Changes to Tree-sitter's format require updates here

### Neutral

- Compression is lossless; all parse information is preserved
- Lookup requires indirection through row offsets
- Future versions may adopt different compression if Tree-sitter's ABI changes

## Related

- Related ADRs: [ADR-006](006-tree-sitter-compatibility-layer.md), [ADR-011](011-parse-table-binary-format.md)
- Evidence: [`tablegen/src/compress.rs`](../../tablegen/src/compress.rs), [`tablegen/src/abi.rs`](../../tablegen/src/abi.rs)
- Tree-sitter Documentation: https://tree-sitter.github.io/tree-sitter/creating-parsers
