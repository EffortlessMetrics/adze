#[cfg(test)]
mod tests {
    use rust_sitter::pure_parser::{ExternalScanner, TSLanguage};
    use rust_sitter::unified_parser::Parser;

    // Minimal symbol tables required for language initialization
    const SYMBOL_0: &[u8] = b"ERROR\0";
    const SYMBOL_1: &[u8] = b"token1\0";
    const SYMBOL_2: &[u8] = b"token2\0";
    const SYMBOL_3: &[u8] = b"token3\0";
    const SYMBOL_4: &[u8] = b"token4\0";
    const SYMBOL_NAMES: [*const u8; 5] = [
        SYMBOL_0.as_ptr(),
        SYMBOL_1.as_ptr(),
        SYMBOL_2.as_ptr(),
        SYMBOL_3.as_ptr(),
        SYMBOL_4.as_ptr(),
    ];
    const SYMBOL_METADATA: [u8; 5] = [0; 5];

    // Mock language for testing
    static TEST_LANGUAGE: TSLanguage = TSLanguage {
        version: 14,
        symbol_count: 5,
        alias_count: 0,
        token_count: 5,
        external_token_count: 0,
        state_count: 20,
        large_state_count: 0,
        production_id_count: 0,
        field_count: 0,
        max_alias_sequence_length: 0,
        production_id_map: std::ptr::null(),
        parse_table: std::ptr::null(),
        small_parse_table: std::ptr::null(),
        small_parse_table_map: std::ptr::null(),
        parse_actions: std::ptr::null(),
        symbol_names: SYMBOL_NAMES.as_ptr(),
        field_names: std::ptr::null(),
        field_map_slices: std::ptr::null(),
        field_map_entries: std::ptr::null(),
        symbol_metadata: SYMBOL_METADATA.as_ptr(),
        public_symbol_map: std::ptr::null(),
        alias_map: std::ptr::null(),
        alias_sequences: std::ptr::null(),
        lex_modes: std::ptr::null(),
        lex_fn: None,
        keyword_lex_fn: None,
        keyword_capture_token: 0,
        external_scanner: ExternalScanner {
            states: std::ptr::null(),
            symbol_map: std::ptr::null(),
            create: None,
            destroy: None,
            scan: None,
            serialize: None,
            deserialize: None,
        },
        primary_state_ids: std::ptr::null(),
        eof_symbol: 0,
        rules: std::ptr::null(),
        rule_count: 0,
        production_count: 0,
        production_lhs_index: std::ptr::null(),
    };

    #[test]
    fn test_parser_creation() {
        let mut parser = Parser::new();
        assert!(parser.set_language(&TEST_LANGUAGE).is_ok());
    }

    #[test]
    fn test_parser_reset() {
        let mut parser = Parser::new();
        parser.set_language(&TEST_LANGUAGE).unwrap();
        parser.reset();
    }

    #[test]
    fn test_parser_with_timeout() {
        let mut parser = Parser::new();
        parser.set_language(&TEST_LANGUAGE).unwrap();
        parser.set_timeout_micros(100_000);
    }
}
