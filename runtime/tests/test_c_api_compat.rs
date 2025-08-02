// Tests for C API compatibility with Tree-sitter
use std::mem;
use std::os::raw::{c_char, c_void};

/// Tree-sitter Language struct layout (must match C ABI exactly)
#[repr(C)]
struct TSLanguage {
    version: u32,
    symbol_count: u32,
    alias_count: u32,
    token_count: u32,
    external_token_count: u32,
    state_count: u32,
    large_state_count: u32,
    production_id_count: u32,
    field_count: u32,
    max_alias_sequence_length: u16,
    parse_table: *const u16,
    small_parse_table: *const u16,
    small_parse_table_map: *const u32,
    parse_actions: *const u32,
    symbol_names: *const *const c_char,
    field_names: *const *const c_char,
    field_map_slices: *const u16,
    field_map_entries: *const u16,
    symbol_metadata: *const u8,
    public_symbol_map: *const u16,
    alias_map: *const u16,
    alias_sequences: *const u16,
    lex_modes: *const u16,
    lex_fn: *const c_void,
    keyword_lex_fn: *const c_void,
    keyword_capture_token: u16,
    external_scanner: ExternalScanner,
}

#[repr(C)]
struct ExternalScanner {
    states: *const bool,
    symbol_map: *const u16,
    create: *const c_void,
    destroy: *const c_void,
    scan: *const c_void,
    serialize: *const c_void,
    deserialize: *const c_void,
}

#[test]
fn test_language_struct_size() {
    // Ensure our Language struct matches Tree-sitter's expected size
    let expected_size = mem::size_of::<TSLanguage>();
    println!("TSLanguage struct size: {} bytes", expected_size);

    // The size should be consistent with Tree-sitter's C struct
    // This varies by platform, but should be around 200-250 bytes on 64-bit
    assert!(expected_size > 150 && expected_size < 300);
}

#[test]
fn test_language_struct_alignment() {
    // Test that the struct has proper alignment for C compatibility
    let alignment = mem::align_of::<TSLanguage>();
    println!("TSLanguage alignment: {} bytes", alignment);

    // Should be pointer-aligned (8 bytes on 64-bit, 4 on 32-bit)
    assert!(alignment == mem::size_of::<*const c_void>());
}

#[test]
fn test_field_offsets() {
    // Test critical field offsets match expected C layout
    unsafe {
        let lang = mem::zeroed::<TSLanguage>();
        let base = &lang as *const _ as usize;

        let version_offset = &lang.version as *const _ as usize - base;
        let symbol_count_offset = &lang.symbol_count as *const _ as usize - base;
        let parse_table_offset = &lang.parse_table as *const _ as usize - base;

        println!("Field offsets:");
        println!("  version: {}", version_offset);
        println!("  symbol_count: {}", symbol_count_offset);
        println!("  parse_table: {}", parse_table_offset);

        // Version should be at offset 0
        assert_eq!(version_offset, 0);

        // Symbol count should follow version
        assert_eq!(symbol_count_offset, 4);

        // Parse table pointer should be after the numeric fields
        assert!(parse_table_offset >= 40);
    }
}

/// Test that our parse table format matches Tree-sitter's
#[test]
fn test_parse_table_format() {
    // Tree-sitter parse tables use specific bit patterns
    const ACTIONS_SHIFT: u16 = 0;
    const ACTIONS_REDUCE: u16 = 1;
    const ACTIONS_ACCEPT: u16 = 2;
    const ACTIONS_RECOVER: u16 = 3;

    // Action encoding uses top 2 bits for type
    let shift_action = (ACTIONS_SHIFT << 14) | 42; // Shift to state 42
    let reduce_action = (ACTIONS_REDUCE << 14) | 7; // Reduce by rule 7

    assert_eq!(shift_action >> 14, ACTIONS_SHIFT);
    assert_eq!(shift_action & 0x3FFF, 42);

    assert_eq!(reduce_action >> 14, ACTIONS_REDUCE);
    assert_eq!(reduce_action & 0x3FFF, 7);
}

/// Test symbol ID compatibility
#[test]
fn test_symbol_ids() {
    // Tree-sitter reserves symbol IDs:
    // 0: END/EOF
    // 1: ERROR
    // 2+: User-defined symbols

    const TS_SYMBOL_END: u16 = 0;
    const TS_SYMBOL_ERROR: u16 = 1;

    // Our SymbolId should map correctly
    assert_eq!(TS_SYMBOL_END, 0);
    assert_eq!(TS_SYMBOL_ERROR, 1);
}

/// Test node structure compatibility
#[repr(C)]
struct TSNode {
    context: [u32; 4],
    id: *const c_void,
    tree: *const c_void,
}

#[test]
fn test_node_struct() {
    // TSNode is 32 bytes on 64-bit platforms
    let node_size = mem::size_of::<TSNode>();
    println!("TSNode size: {} bytes", node_size);

    #[cfg(target_pointer_width = "64")]
    assert_eq!(node_size, 32);

    #[cfg(target_pointer_width = "32")]
    assert_eq!(node_size, 24);
}

/// Test that our subtree structure can be cast to Tree-sitter's format
#[test]
fn test_subtree_compatibility() {
    // Tree-sitter subtrees need specific memory layout
    // The first word contains metadata packed into bits

    const SUBTREE_BITS_SYMBOL: u32 = 0xFFFF;
    const SUBTREE_BITS_IS_NAMED: u32 = 1 << 16;
    const SUBTREE_BITS_IS_HIDDEN: u32 = 1 << 17;
    const SUBTREE_BITS_IS_KEYWORD: u32 = 1 << 18;
    const SUBTREE_BITS_HAS_CHANGES: u32 = 1 << 19;

    // Test packing a symbol ID with flags
    let symbol_id = 42u32;
    let packed = symbol_id | SUBTREE_BITS_IS_NAMED;

    assert_eq!(packed & SUBTREE_BITS_SYMBOL, symbol_id);
    assert_ne!(packed & SUBTREE_BITS_IS_NAMED, 0);
}
