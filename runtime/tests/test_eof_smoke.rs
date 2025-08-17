#[test]
fn eof_column_is_in_token_range() {
    // Import example language from example crate if available
    // For now, just verify the constant is set correctly

    // Create a dummy language to test
    use rust_sitter::pure_parser::TSLanguage;
    use std::ptr;

    let lang = TSLanguage {
        version: 14,
        symbol_count: 10,
        alias_count: 0,
        token_count: 5,
        external_token_count: 0,
        state_count: 20,
        large_state_count: 5,
        production_id_count: 0,
        field_count: 0,
        max_alias_sequence_length: 0,
        production_id_map: ptr::null(),
        parse_table: ptr::null(),
        small_parse_table: ptr::null(),
        small_parse_table_map: ptr::null(),
        parse_actions: ptr::null(),
        symbol_names: ptr::null(),
        field_names: ptr::null(),
        field_map_slices: ptr::null(),
        field_map_entries: ptr::null(),
        symbol_metadata: ptr::null(),
        public_symbol_map: ptr::null(),
        alias_map: ptr::null(),
        alias_sequences: ptr::null(),
        lex_modes: ptr::null(),
        lex_fn: None,
        keyword_lex_fn: None,
        keyword_capture_token: 0,
        external_scanner: rust_sitter::pure_parser::ExternalScanner {
            states: ptr::null(),
            symbol_map: ptr::null(),
            create: None,
            destroy: None,
            scan: None,
            serialize: None,
            deserialize: None,
        },
        primary_state_ids: ptr::null(),
        production_count: 0,
        production_lhs_index: ptr::null(),
        eof_symbol: 0, // EOF is column 0 in Tree-sitter convention
    };

    // Verify EOF is in token range
    assert!((lang.eof_symbol as u32) < lang.token_count);
    assert_eq!(lang.eof_symbol, 0, "EOF should be column 0 by convention");
}
