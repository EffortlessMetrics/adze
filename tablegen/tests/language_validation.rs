use rust_sitter_glr_core::{Action, LexMode, ParseRule, ParseTable};
use rust_sitter_ir::{FieldId, Grammar, StateId, SymbolId, Token, TokenPattern};
use rust_sitter_tablegen::validation::TSLanguage;
use rust_sitter_tablegen::{
    CompressedParseTable, LanguageBuilder, LanguageValidator, ValidationError,
};

// Helper function to create a default ParseTable for testing
fn create_test_parse_table(grammar: Grammar) -> ParseTable {
    ParseTable {
        action_table: vec![],
        goto_table: vec![],
        symbol_metadata: vec![],
        state_count: 0,
        symbol_count: 0,
        symbol_to_index: std::collections::BTreeMap::new(),
        index_to_symbol: vec![],
        external_scanner_states: vec![],
        rules: vec![],
        nonterminal_to_index: std::collections::BTreeMap::new(),
        eof_symbol: SymbolId(0),
        start_symbol: SymbolId(1),
        grammar,
        initial_state: StateId(0),
        token_count: 0,
        external_token_count: 0,
        lex_modes: vec![],
        extras: vec![],
        dynamic_prec_by_rule: vec![],
        alias_sequences: vec![],
        field_names: vec![],
        field_map: std::collections::BTreeMap::new(),
    }
}

#[test]
fn test_language_generation_and_validation() {
    // Create a simple grammar
    let mut grammar = Grammar::new("test".to_string());

    // Add tokens
    let token = Token {
        name: "NUMBER".to_string(),
        pattern: TokenPattern::Regex(r"\d+".to_string()),
        fragile: false,
    };
    grammar.tokens.insert(SymbolId(0), token);

    // Add fields
    grammar.fields.insert(FieldId(0), "value".to_string());

    // Create parse table
    let mut parse_table = create_test_parse_table(grammar.clone());
    parse_table.action_table = vec![
        vec![vec![Action::Shift(StateId(1))]],
        vec![vec![Action::Accept]],
    ];
    parse_table.goto_table = vec![vec![StateId(0)], vec![StateId(1)]];
    parse_table.state_count = 2;
    parse_table.symbol_count = 2;

    // Create compressed table before moving parse_table
    let compressed = CompressedParseTable::from_parse_table(&parse_table);

    // Generate Language
    let generator = LanguageBuilder::new(grammar, parse_table);
    let result = generator.generate_language();

    assert!(
        result.is_ok(),
        "Language generation failed: {:?}",
        result.err()
    );

    let language = result.unwrap();

    // Verify basic properties
    assert_eq!(language.version, 15);
    assert_eq!(language.state_count, 2);
    assert_eq!(language.symbol_count, 2);
    assert_eq!(language.field_count, 1);

    // Validate the generated language
    let validator = LanguageValidator::new(&language, &compressed);
    let validation_result = validator.validate();
    if let Err(errors) = &validation_result {
        eprintln!("Validation errors: {:?}", errors);
    }
    assert!(validation_result.is_ok());
}

#[test]
fn test_language_validation_catches_version_error() {
    // Create a language with wrong version
    let mut language = create_test_language();
    language.version = 14; // Wrong version

    let compressed = CompressedParseTable::new_for_testing(10, 20);
    let validator = LanguageValidator::new(&language, &compressed);

    let result = validator.validate();
    assert!(result.is_err());

    let errors = result.unwrap_err();
    assert!(errors.iter().any(|e| matches!(
        e,
        ValidationError::InvalidVersion {
            expected: 15,
            actual: 14
        }
    )));
}

#[test]
fn test_language_validation_catches_symbol_count_mismatch() {
    // Create a language with mismatched symbol count
    let mut language = create_test_language();
    language.symbol_count = 15;

    let compressed = CompressedParseTable::new_for_testing(10, 20); // symbol_count = 10
    let validator = LanguageValidator::new(&language, &compressed);

    let result = validator.validate();
    assert!(result.is_err());

    let errors = result.unwrap_err();
    assert!(
        errors
            .iter()
            .any(|e| matches!(e, ValidationError::SymbolCountMismatch { .. }))
    );
}

#[test]
fn test_language_validation_catches_state_count_mismatch() {
    // Create a language with mismatched state count
    let mut language = create_test_language();
    language.state_count = 25;
    language.symbol_count = 10; // Match compressed table

    let compressed = CompressedParseTable::new_for_testing(10, 20); // state_count = 20
    let validator = LanguageValidator::new(&language, &compressed);

    let result = validator.validate();
    assert!(result.is_err());

    let errors = result.unwrap_err();
    assert!(
        errors
            .iter()
            .any(|e| matches!(e, ValidationError::StateCountMismatch { .. }))
    );
}

#[test]
fn test_language_validation_field_names_ordering() {
    // Test that field names must be in lexicographic order
    let mut grammar = Grammar::new("test".to_string());

    // Add fields in correct order
    grammar.fields.insert(FieldId(0), "alpha".to_string());
    grammar.fields.insert(FieldId(1), "beta".to_string());
    grammar.fields.insert(FieldId(2), "gamma".to_string());

    let mut parse_table = create_test_parse_table(grammar.clone());
    parse_table.state_count = 1;
    parse_table.symbol_count = 1;

    let generator = LanguageBuilder::new(grammar, parse_table);
    let result = generator.generate_language();

    // Should succeed with properly ordered fields
    assert!(result.is_ok());
}

#[test]
fn test_symbol_metadata_validation() {
    // Create a grammar with various symbol types
    let mut grammar = Grammar::new("test".to_string());

    // Add visible named token
    grammar.tokens.insert(
        SymbolId(1),
        Token {
            name: "identifier".to_string(),
            pattern: TokenPattern::Regex(r"[a-z]+".to_string()),
            fragile: false,
        },
    );

    // Add hidden token (starts with _)
    grammar.tokens.insert(
        SymbolId(2),
        Token {
            name: "_whitespace".to_string(),
            pattern: TokenPattern::Regex(r"\s+".to_string()),
            fragile: false,
        },
    );

    // Add anonymous token (string literal)
    grammar.tokens.insert(
        SymbolId(3),
        Token {
            name: "+".to_string(),
            pattern: TokenPattern::String("+".to_string()),
            fragile: false,
        },
    );

    let mut parse_table = create_test_parse_table(grammar.clone());
    parse_table.action_table = vec![vec![vec![Action::Accept]]];
    parse_table.goto_table = vec![vec![StateId(0)]];
    parse_table.state_count = 1;
    parse_table.symbol_count = 4; // EOF + 3 tokens

    let generator = LanguageBuilder::new(grammar, parse_table);
    let result = generator.generate_language();

    assert!(result.is_ok());

    let language = result.unwrap();

    // Verify symbol metadata is correct
    unsafe {
        let metadata =
            std::slice::from_raw_parts(language.symbol_metadata, language.symbol_count as usize);

        // First symbol (EOF) should be invisible and unnamed
        assert!(!metadata[0].visible);
        assert!(!metadata[0].named);

        // "identifier" should be visible and named
        assert!(metadata[1].visible);
        assert!(metadata[1].named);

        // "_whitespace" should be invisible and unnamed
        assert!(!metadata[2].visible);
        assert!(!metadata[2].named);

        // "+" should be visible but unnamed (anonymous)
        assert!(metadata[3].visible);
        assert!(!metadata[3].named);
    }
}

#[test]
fn test_empty_grammar_validation() {
    // Test that an empty grammar still produces a valid Language
    let grammar = Grammar::new("empty".to_string());
    let parse_table = create_test_parse_table(grammar.clone());

    let generator = LanguageBuilder::new(grammar, parse_table);
    let result = generator.generate_language();

    assert!(result.is_ok());

    let language = result.unwrap();
    assert_eq!(language.version, 15);
    assert_eq!(language.state_count, 0);
    assert_eq!(language.symbol_count, 0);
    assert_eq!(language.field_count, 0);
}

// Helper function to create a test Language struct
fn create_test_language() -> TSLanguage {
    TSLanguage {
        version: 15,
        symbol_count: 10,
        alias_count: 0,
        token_count: 5,
        external_token_count: 0,
        state_count: 20,
        large_state_count: 0,
        production_id_count: 0,
        field_count: 0,
        max_alias_sequence_length: 0,
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
        external_scanner_data: rust_sitter_tablegen::validation::TSExternalScannerData {
            states: std::ptr::null(),
            symbol_map: std::ptr::null(),
            create: None,
            destroy: None,
            scan: None,
            serialize: None,
            deserialize: None,
        },
        primary_state_ids: std::ptr::null(),
    }
}
