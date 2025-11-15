#[cfg(test)]
mod tests {
    use rust_sitter::pure_parser::{ExternalScanner, TSLanguage};
    use rust_sitter::unified_parser::Parser;

    // Mock language for testing
    static TEST_LANGUAGE: TSLanguage = TSLanguage {
        version: 14,
        symbol_count: 10,
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
        symbol_names: std::ptr::null(),
        field_names: std::ptr::null(),
        field_map_slices: std::ptr::null(),
        field_map_entries: std::ptr::null(),
        symbol_metadata: std::ptr::null(),
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
    #[ignore = "Unified parser test needs valid parse table in TEST_LANGUAGE"]
    fn test_parser_reset() {
        let mut parser = Parser::new();
        parser.set_language(&TEST_LANGUAGE).unwrap();

        // Parse some input
        let input = "test input";
        let _result = parser.parse(input, None);

        // Reset should clear state
        parser.reset();

        // Should be able to parse again
        let _result2 = parser.parse(input, None);
    }

    #[test]
    #[ignore = "Unified parser test needs valid parse table in TEST_LANGUAGE"]
    fn test_parser_with_timeout() {
        let mut parser = Parser::new();
        parser.set_language(&TEST_LANGUAGE).unwrap();

        // Set a timeout
        parser.set_timeout_micros(100_000);

        let input = "test";
        let _result = parser.parse(input, None);
    }
}
