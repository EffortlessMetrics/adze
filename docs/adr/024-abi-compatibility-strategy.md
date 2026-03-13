# ADR-024: ABI Compatibility Strategy

## Status

Accepted

## Context

Adze generates parse tables that must be usable by Tree-sitter's runtime and compatible with the broader Tree-sitter ecosystem. This requires exact binary-level compatibility with Tree-sitter's Application Binary Interface (ABI).

The Tree-sitter ABI defines:
- Memory layout of language structures
- Size and alignment of primitive types
- Version constants for compatibility checking
- Function signatures for external scanners

Without ABI compatibility:
- Generated grammars would not load in Tree-sitter-powered editors
- Parse tables would be unusable by the Tree-sitter C runtime
- Users would lose access to the existing Tree-sitter ecosystem

### Alternatives Considered

1. **Pure Rust Format**: Define our own binary format without Tree-sitter constraints
2. **API-Only Compatibility**: Provide conversion layers at runtime
3. **Full ABI Compatibility**: Match Tree-sitter's binary layout exactly
4. **Version-Specific ABIs**: Support multiple Tree-sitter ABI versions

## Decision

We implement **full ABI compatibility** with Tree-sitter ABI v15 through exact struct layout matching in [`tablegen/src/abi.rs`](../../tablegen/src/abi.rs).

### Version Constants

```rust
/// Tree-sitter ABI version 15
pub const TREE_SITTER_LANGUAGE_VERSION: u32 = 15;

/// Minimum compatible ABI version
pub const TREE_SITTER_MIN_COMPATIBLE_LANGUAGE_VERSION: u32 = 13;
```

These constants ensure:
- Generated languages report correct version to Tree-sitter runtime
- Backward compatibility with parsers expecting ABI v13+

### FFI-Safe Type Definitions

All types use `#[repr(C)]` or `#[repr(C, packed)]` for predictable memory layout:

```rust
/// Tree-sitter symbol type - must match C definition exactly
#[repr(C)]
pub struct TSSymbol(pub u16);

/// Tree-sitter state ID type
#[repr(C)]
pub struct TSStateId(pub u16);

/// Tree-sitter field ID type
#[repr(C)]
pub struct TSFieldId(pub u16);

/// Parse action type for ABI 15
#[repr(C, packed)]
pub struct TSParseAction {
    pub action_type: u8,
    pub extra: u8,  // Use u8 instead of bool for consistent size
    pub child_count: u8,
    pub dynamic_precedence: i8,
    pub symbol: TSSymbol,
}
```

### Compile-Time Size Assertions

Struct sizes are verified at compile time to catch ABI drift:

```rust
const _: () = {
    use std::mem;

    // These sizes must match tree-sitter's expectations
    assert!(mem::size_of::<TSSymbol>() == 2);
    assert!(mem::size_of::<TSStateId>() == 2);
    assert!(mem::size_of::<TSFieldId>() == 2);
    assert!(mem::size_of::<TSParseAction>() == 6);
    assert!(mem::size_of::<TSLexState>() == 4);

    // Language struct must be pointer-sized aligned
    assert!(mem::align_of::<TSLanguage>() == mem::align_of::<*const u8>());
};
```

### TSLanguage Structure

The main language structure matches Tree-sitter's layout exactly:

```rust
#[repr(C)]
pub struct TSLanguage {
    pub version: u32,
    pub symbol_count: u32,
    pub alias_count: u32,
    pub token_count: u32,
    pub external_token_count: u32,
    pub state_count: u32,
    pub large_state_count: u32,
    pub production_id_count: u32,
    pub field_count: u32,
    pub max_alias_sequence_length: u16,
    pub production_id_map: *const u16,
    pub parse_table: *const u16,
    // ... additional fields matching Tree-sitter layout
}
```

### Symbol Metadata Flags

Symbol metadata uses bit flags compatible with Tree-sitter:

```rust
pub mod symbol_metadata {
    pub const VISIBLE: u8 = 0x01;
    pub const NAMED: u8 = 0x02;
    pub const HIDDEN: u8 = 0x04;
    pub const AUXILIARY: u8 = 0x08;
    pub const SUPERTYPE: u8 = 0x10;
}
```

## Consequences

### Positive

- **Ecosystem Compatibility**: Generated grammars work with Neovim, Emacs, Atom, and other Tree-sitter consumers
- **Runtime Interoperability**: Can use Tree-sitter's C runtime for parsing
- **Validation**: Binary compatibility enables direct comparison testing against Tree-sitter
- **Editor Integration**: No changes needed to existing Tree-sitter editor plugins

### Negative

- **Layout Constraints**: Must maintain exact struct layouts, limiting refactoring flexibility
- **Version Lock**: Tied to Tree-sitter ABI v15; upgrades require coordinated changes
- **Packed Structs**: `#[repr(C, packed)]` can cause performance issues on some platforms
- **Pointer Usage**: Must use raw pointers for FFI, requiring unsafe code

### Neutral

- **Testing Requirements**: Must verify struct sizes match on all target platforms
- **Documentation Burden**: Need to document ABI constraints for contributors
- **C Interop**: Some patterns are driven by C conventions rather than Rust idioms

## Implementation Notes

### Size Verification Tests

Runtime tests verify compile-time assertions:

```rust
#[test]
fn test_struct_sizes() {
    assert_eq!(mem::size_of::<TSSymbol>(), 2);
    assert_eq!(mem::size_of::<TSStateId>(), 2);
    assert_eq!(mem::size_of::<TSFieldId>(), 2);
    assert_eq!(mem::size_of::<TSParseAction>(), 6);
    assert_eq!(mem::size_of::<TSLexState>(), 4);
}
```

### External Scanner ABI

External scanners require function pointer fields matching Tree-sitter's expectations:

```rust
#[repr(C)]
pub struct ExternalScanner {
    pub states: *const bool,
    pub symbol_map: *const TSSymbol,
    pub create: Option<unsafe extern "C" fn() -> *mut c_void>,
    pub destroy: Option<unsafe extern "C" fn(*mut c_void)>,
    pub scan: Option<unsafe extern "C" fn(*mut c_void, *mut c_void, *const bool) -> bool>,
    pub serialize: Option<unsafe extern "C" fn(*mut c_void, *mut u8) -> u32>,
    pub deserialize: Option<unsafe extern "C" fn(*mut c_void, *const u8, u32)>,
}
```

## Related

- Related ADRs: [ADR-006: Tree-sitter Compatibility Layer](006-tree-sitter-compatibility-layer.md)
- Implementation: [tablegen/src/abi.rs](../../tablegen/src/abi.rs)
- Tests: [tablegen/src/abi.rs - tests module](../../tablegen/src/abi.rs:164)
- Reference: [Tree-sitter Documentation](https://tree-sitter.github.io/tree-sitter/)
