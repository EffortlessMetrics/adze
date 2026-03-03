//! Comprehensive tests for the language generation module.
//!
//! Covers: LanguageGenerator construction, symbol names, field names,
//! symbol metadata, production ID counting, code generation output,
//! ABI constants, node type generation, LanguageBuilder, and validators.

use adze_glr_core::{Action, GotoIndexing, LexMode, ParseTable, SymbolMetadata};
use adze_ir::{
    ExternalToken, FieldId, Grammar, ProductionId, Rule, StateId, Symbol, SymbolId, Token,
    TokenPattern,
};
use adze_tablegen::abi::{
    self, ExternalScanner, TREE_SITTER_LANGUAGE_VERSION,
    TREE_SITTER_MIN_COMPATIBLE_LANGUAGE_VERSION, TSFieldId, TSLanguage as AbiTSLanguage,
    TSLexState, TSParseAction, TSStateId, TSSymbol, create_symbol_metadata,
};
use adze_tablegen::compress::CompressedParseTable;
use adze_tablegen::language_gen::LanguageGenerator;
use adze_tablegen::validation::{LanguageValidator, ValidationError};
use adze_tablegen::{LanguageBuilder, NodeTypesGenerator, StaticLanguageGenerator};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Build a minimal grammar with the given name, tokens, and rules.
fn make_grammar(name: &str, tokens: Vec<(SymbolId, Token)>, rules: Vec<Rule>) -> Grammar {
    let mut g = Grammar::new(name.to_string());
    for (id, tok) in tokens {
        g.tokens.insert(id, tok);
    }
    for rule in rules {
        g.add_rule(rule);
    }
    g
}

/// Convenience token constructor.
fn regex_token(name: &str, pattern: &str) -> Token {
    Token {
        name: name.to_string(),
        pattern: TokenPattern::Regex(pattern.to_string()),
        fragile: false,
    }
}

/// Convenience string-literal token constructor.
fn string_token(name: &str, literal: &str) -> Token {
    Token {
        name: name.to_string(),
        pattern: TokenPattern::String(literal.to_string()),
        fragile: false,
    }
}

/// Build a fully-formed ParseTable compatible with LanguageGenerator.
fn make_parse_table_for_gen(
    grammar: &Grammar,
    state_count: usize,
    actions: Vec<Vec<Vec<Action>>>,
) -> ParseTable {
    let symbol_count = actions.first().map(|r| r.len()).unwrap_or(1);
    let mut symbol_to_index = std::collections::BTreeMap::new();
    for i in 0..symbol_count {
        symbol_to_index.insert(SymbolId(i as u16), i);
    }
    let index_to_symbol: Vec<SymbolId> = (0..symbol_count).map(|i| SymbolId(i as u16)).collect();
    let goto_table = vec![vec![StateId(u16::MAX); symbol_count]; state_count];
    let lex_modes = vec![
        LexMode {
            lex_state: 0,
            external_lex_state: 0,
        };
        state_count
    ];
    let eof_symbol = SymbolId((symbol_count.saturating_sub(1)) as u16);

    ParseTable {
        action_table: actions,
        goto_table,
        rules: vec![],
        state_count,
        symbol_count,
        symbol_to_index,
        index_to_symbol,
        nonterminal_to_index: std::collections::BTreeMap::new(),
        symbol_metadata: vec![],
        token_count: symbol_count.saturating_sub(1),
        external_token_count: grammar.externals.len(),
        eof_symbol,
        start_symbol: SymbolId(0),
        initial_state: StateId(0),
        lex_modes,
        extras: vec![],
        external_scanner_states: vec![],
        dynamic_prec_by_rule: vec![],
        rule_assoc_by_rule: vec![],
        alias_sequences: vec![],
        field_names: vec![],
        field_map: std::collections::BTreeMap::new(),
        grammar: grammar.clone(),
        goto_indexing: GotoIndexing::NonterminalMap,
    }
}

// ---------------------------------------------------------------------------
// 1. LanguageGenerator: symbol names
// ---------------------------------------------------------------------------

#[test]
fn gen_symbol_names_start_with_end() {
    let grammar = make_grammar("t", vec![(SymbolId(1), regex_token("num", r"\d+"))], vec![]);
    let pt = make_parse_table_for_gen(&grammar, 2, vec![vec![vec![]; 3]; 2]);
    let generator = LanguageGenerator::new(&grammar, &pt);
    let output = generator.generate().to_string();
    // The first symbol name should be "end" (EOF sentinel).
    assert!(
        output.contains("\"end\""),
        "generated code must include EOF sentinel name"
    );
}

#[test]
fn gen_symbol_names_include_tokens() {
    let grammar = make_grammar(
        "t",
        vec![
            (SymbolId(1), regex_token("identifier", r"[a-z]+")),
            (SymbolId(2), string_token("plus", "+")),
        ],
        vec![],
    );
    let pt = make_parse_table_for_gen(&grammar, 2, vec![vec![vec![]; 4]; 2]);
    let generator = LanguageGenerator::new(&grammar, &pt);
    let output = generator.generate().to_string();
    assert!(output.contains("identifier"), "should contain token name");
    assert!(output.contains("plus"), "should contain string token name");
}

// ---------------------------------------------------------------------------
// 2. LanguageGenerator: field names
// ---------------------------------------------------------------------------

#[test]
fn gen_field_names_empty_grammar() {
    let grammar = make_grammar("t", vec![], vec![]);
    let pt = make_parse_table_for_gen(&grammar, 1, vec![vec![vec![]; 1]; 1]);
    let generator = LanguageGenerator::new(&grammar, &pt);
    let output = generator.generate().to_string();
    // With no fields the FIELD_NAMES array should be empty.
    assert!(
        output.contains("FIELD_NAMES"),
        "output must reference FIELD_NAMES"
    );
}

#[test]
fn gen_field_names_populated() {
    let mut grammar = make_grammar("t", vec![], vec![]);
    grammar.fields.insert(FieldId(0), "left".to_string());
    grammar.fields.insert(FieldId(1), "right".to_string());
    let pt = make_parse_table_for_gen(&grammar, 1, vec![vec![vec![]; 1]; 1]);
    let generator = LanguageGenerator::new(&grammar, &pt);
    let output = generator.generate().to_string();
    assert!(output.contains("left"), "should list 'left' field");
    assert!(output.contains("right"), "should list 'right' field");
}

// ---------------------------------------------------------------------------
// 3. LanguageGenerator: symbol metadata
// ---------------------------------------------------------------------------

#[test]
fn gen_symbol_metadata_length_matches_symbol_count() {
    let grammar = make_grammar(
        "t",
        vec![(SymbolId(1), regex_token("a", "."))],
        vec![Rule {
            lhs: SymbolId(2),
            rhs: vec![Symbol::Terminal(SymbolId(1))],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(0),
        }],
    );
    let pt = make_parse_table_for_gen(&grammar, 2, vec![vec![vec![]; 3]; 2]);
    let generator = LanguageGenerator::new(&grammar, &pt);
    let metadata = generator.generate_symbol_metadata_public();
    // 1 (EOF) + 1 token + 1 rule = 3
    let expected = 1 + grammar.tokens.len() + grammar.rules.len();
    assert_eq!(metadata.len(), expected);
}

#[test]
fn gen_symbol_metadata_all_visible_named() {
    let grammar = make_grammar("t", vec![(SymbolId(1), regex_token("x", "."))], vec![]);
    let pt = make_parse_table_for_gen(&grammar, 1, vec![vec![vec![]; 2]; 1]);
    let generator = LanguageGenerator::new(&grammar, &pt);
    let metadata = generator.generate_symbol_metadata_public();
    // Current implementation marks all symbols 0b11 (visible|named).
    for &byte in &metadata {
        assert_eq!(byte & 0b11, 0b11, "each symbol should be visible+named");
    }
}

// ---------------------------------------------------------------------------
// 4. LanguageGenerator: production ID counting
// ---------------------------------------------------------------------------

#[test]
fn gen_production_id_count_single_rule() {
    let grammar = make_grammar(
        "t",
        vec![(SymbolId(1), regex_token("n", r"\d"))],
        vec![Rule {
            lhs: SymbolId(2),
            rhs: vec![Symbol::Terminal(SymbolId(1))],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(0),
        }],
    );
    let pt = make_parse_table_for_gen(&grammar, 2, vec![vec![vec![]; 3]; 2]);
    let generator = LanguageGenerator::new(&grammar, &pt);
    // ProductionId(0) → count = 1
    assert_eq!(generator.count_production_ids_public(), 1);
}

#[test]
fn gen_production_id_count_multiple_rules() {
    let grammar = make_grammar(
        "t",
        vec![(SymbolId(1), regex_token("n", r"\d"))],
        vec![
            Rule {
                lhs: SymbolId(2),
                rhs: vec![Symbol::Terminal(SymbolId(1))],
                precedence: None,
                associativity: None,
                fields: vec![],
                production_id: ProductionId(0),
            },
            Rule {
                lhs: SymbolId(2),
                rhs: vec![Symbol::Terminal(SymbolId(1)), Symbol::Terminal(SymbolId(1))],
                precedence: None,
                associativity: None,
                fields: vec![],
                production_id: ProductionId(3),
            },
        ],
    );
    let pt = make_parse_table_for_gen(&grammar, 2, vec![vec![vec![]; 3]; 2]);
    let generator = LanguageGenerator::new(&grammar, &pt);
    // max id = 3 → count = 4
    assert_eq!(generator.count_production_ids_public(), 4);
}

// ---------------------------------------------------------------------------
// 5. LanguageGenerator: generated code structure
// ---------------------------------------------------------------------------

#[test]
fn gen_output_contains_language_fn() {
    let grammar = make_grammar("demo", vec![(SymbolId(1), regex_token("tok", "."))], vec![]);
    let pt = make_parse_table_for_gen(&grammar, 1, vec![vec![vec![]; 2]; 1]);
    let generator = LanguageGenerator::new(&grammar, &pt);
    let code = generator.generate().to_string();
    assert!(code.contains("tree_sitter_demo"), "FFI export expected");
    assert!(code.contains("LANGUAGE"), "static LANGUAGE struct expected");
}

#[test]
fn gen_output_contains_version_constant() {
    let grammar = make_grammar("v", vec![], vec![]);
    let pt = make_parse_table_for_gen(&grammar, 1, vec![vec![vec![]; 1]; 1]);
    let generator = LanguageGenerator::new(&grammar, &pt);
    let code = generator.generate().to_string();
    assert!(
        code.contains("TREE_SITTER_LANGUAGE_VERSION"),
        "version const expected"
    );
}

// ---------------------------------------------------------------------------
// 6. ABI constants & struct sizes
// ---------------------------------------------------------------------------

#[test]
fn abi_version_constants() {
    assert_eq!(TREE_SITTER_LANGUAGE_VERSION, 15);
    // Verify min-compat is not above the current version.
    let min_compat = TREE_SITTER_MIN_COMPATIBLE_LANGUAGE_VERSION;
    let current = TREE_SITTER_LANGUAGE_VERSION;
    assert!(
        min_compat <= current,
        "min compat version {min_compat} must be <= current version {current}",
    );
}

#[test]
fn abi_struct_sizes() {
    assert_eq!(std::mem::size_of::<TSSymbol>(), 2);
    assert_eq!(std::mem::size_of::<TSStateId>(), 2);
    assert_eq!(std::mem::size_of::<TSFieldId>(), 2);
    assert_eq!(std::mem::size_of::<TSParseAction>(), 6);
    assert_eq!(std::mem::size_of::<TSLexState>(), 4);
}

#[test]
fn abi_tslanguage_alignment() {
    assert_eq!(
        std::mem::align_of::<AbiTSLanguage>(),
        std::mem::align_of::<*const u8>(),
        "TSLanguage must be pointer-aligned for FFI"
    );
}

// ---------------------------------------------------------------------------
// 7. create_symbol_metadata flag combinations
// ---------------------------------------------------------------------------

#[test]
fn symbol_metadata_flags_visible_named() {
    let m = create_symbol_metadata(true, true, false, false, false);
    assert_eq!(
        m,
        abi::symbol_metadata::VISIBLE | abi::symbol_metadata::NAMED
    );
}

#[test]
fn symbol_metadata_flags_hidden_auxiliary() {
    let m = create_symbol_metadata(false, false, true, true, false);
    assert_eq!(
        m,
        abi::symbol_metadata::HIDDEN | abi::symbol_metadata::AUXILIARY
    );
}

#[test]
fn symbol_metadata_flags_supertype() {
    let m = create_symbol_metadata(true, true, false, false, true);
    assert_eq!(
        m,
        abi::symbol_metadata::VISIBLE
            | abi::symbol_metadata::NAMED
            | abi::symbol_metadata::SUPERTYPE
    );
}

#[test]
fn symbol_metadata_flags_none() {
    assert_eq!(create_symbol_metadata(false, false, false, false, false), 0);
}

#[test]
fn symbol_metadata_flags_all() {
    let m = create_symbol_metadata(true, true, true, true, true);
    let expected = abi::symbol_metadata::VISIBLE
        | abi::symbol_metadata::NAMED
        | abi::symbol_metadata::HIDDEN
        | abi::symbol_metadata::AUXILIARY
        | abi::symbol_metadata::SUPERTYPE;
    assert_eq!(m, expected);
}

// ---------------------------------------------------------------------------
// 8. ExternalScanner default
// ---------------------------------------------------------------------------

#[test]
fn external_scanner_default_is_null() {
    let es = ExternalScanner::default();
    assert!(es.states.is_null());
    assert!(es.symbol_map.is_null());
    assert!(es.create.is_none());
    assert!(es.destroy.is_none());
    assert!(es.scan.is_none());
    assert!(es.serialize.is_none());
    assert!(es.deserialize.is_none());
}

// ---------------------------------------------------------------------------
// 9. NodeTypesGenerator
// ---------------------------------------------------------------------------

#[test]
fn node_types_valid_json_for_simple_grammar() {
    let mut grammar = Grammar::new("ntest".to_string());
    grammar
        .tokens
        .insert(SymbolId(1), regex_token("number", r"\d+"));
    grammar.add_rule(Rule {
        lhs: SymbolId(2),
        rhs: vec![Symbol::Terminal(SymbolId(1))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });

    let generator = NodeTypesGenerator::new(&grammar);
    let json = generator.generate().expect("generate must succeed");
    let parsed: serde_json::Value = serde_json::from_str(&json).expect("must be valid JSON");
    assert!(parsed.is_array());
}

#[test]
fn node_types_unnamed_for_string_token() {
    let mut grammar = Grammar::new("stest".to_string());
    grammar
        .tokens
        .insert(SymbolId(1), string_token("plus", "+"));

    let generator = NodeTypesGenerator::new(&grammar);
    let json = generator.generate().unwrap();
    let arr: Vec<serde_json::Value> = serde_json::from_str(&json).unwrap();
    // String-pattern tokens become unnamed nodes.
    let plus_entry = arr.iter().find(|v| v["type"] == "+");
    assert!(plus_entry.is_some(), "should have '+' node type");
    assert_eq!(plus_entry.unwrap()["named"], false);
}

// ---------------------------------------------------------------------------
// 10. LanguageBuilder
// ---------------------------------------------------------------------------

fn builder_grammar_and_table() -> (Grammar, ParseTable) {
    let mut grammar = Grammar::new("btest".to_string());
    grammar
        .tokens
        .insert(SymbolId(1), regex_token("id", r"[a-z]+"));
    grammar.add_rule(Rule {
        lhs: SymbolId(0),
        rhs: vec![Symbol::Terminal(SymbolId(1))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });

    let mut symbol_to_index = std::collections::BTreeMap::new();
    symbol_to_index.insert(SymbolId(0), 0);
    symbol_to_index.insert(SymbolId(1), 1);

    let pt = ParseTable {
        action_table: vec![vec![vec![Action::Accept]; 2]; 2],
        goto_table: vec![vec![StateId(0); 2]; 2],
        state_count: 2,
        symbol_count: 2,
        symbol_to_index: symbol_to_index.clone(),
        index_to_symbol: vec![SymbolId(0), SymbolId(1)],
        symbol_metadata: vec![
            SymbolMetadata {
                name: "tok".to_string(),
                is_visible: true,
                is_named: true,
                is_supertype: false,
                is_terminal: true,
                is_extra: false,
                is_fragile: false,
                symbol_id: SymbolId(0),
            };
            2
        ],
        nonterminal_to_index: std::collections::BTreeMap::new(),
        eof_symbol: SymbolId(1),
        start_symbol: SymbolId(0),
        grammar: grammar.clone(),
        initial_state: StateId(0),
        token_count: 1,
        external_token_count: 0,
        lex_modes: vec![
            LexMode {
                lex_state: 0,
                external_lex_state: 0,
            };
            2
        ],
        extras: vec![],
        external_scanner_states: vec![],
        dynamic_prec_by_rule: vec![],
        rule_assoc_by_rule: vec![],
        alias_sequences: vec![],
        field_names: vec![],
        field_map: std::collections::BTreeMap::new(),
        rules: vec![],
        goto_indexing: GotoIndexing::NonterminalMap,
    };

    (grammar, pt)
}

#[test]
fn language_builder_version_is_15() {
    let (g, pt) = builder_grammar_and_table();
    let builder = LanguageBuilder::new(g, pt);
    let lang = builder.generate_language().expect("must succeed");
    assert_eq!(lang.version, 15);
}

#[test]
fn language_builder_state_and_symbol_counts() {
    let (g, pt) = builder_grammar_and_table();
    let builder = LanguageBuilder::new(g, pt);
    let lang = builder.generate_language().unwrap();
    assert_eq!(lang.state_count, 2);
    assert_eq!(lang.symbol_count, 2);
}

#[test]
fn language_builder_external_token_count() {
    let (mut g, pt) = builder_grammar_and_table();
    g.externals.push(ExternalToken {
        name: "comment".to_string(),
        symbol_id: SymbolId(100),
    });
    let builder = LanguageBuilder::new(g, pt);
    let lang = builder.generate_language().unwrap();
    assert_eq!(lang.external_token_count, 1);
}

#[test]
fn language_builder_field_count() {
    let (mut g, pt) = builder_grammar_and_table();
    g.fields.insert(FieldId(0), "value".to_string());
    g.fields.insert(FieldId(1), "operator".to_string());
    let builder = LanguageBuilder::new(g, pt);
    let lang = builder.generate_language().unwrap();
    assert_eq!(lang.field_count, 2);
}

#[test]
fn language_builder_code_gen_mentions_tslanguage() {
    let (g, pt) = builder_grammar_and_table();
    let builder = LanguageBuilder::new(g, pt);
    let code = builder.generate_language_code().to_string();
    assert!(code.contains("TSLanguage"));
}

// ---------------------------------------------------------------------------
// 11. StaticLanguageGenerator
// ---------------------------------------------------------------------------

#[test]
fn static_gen_code_contains_key_arrays() {
    let (g, pt) = builder_grammar_and_table();
    let generator = StaticLanguageGenerator::new(g, pt);
    let code = generator.generate_language_code().to_string();
    assert!(code.contains("SYMBOL_NAMES"), "missing SYMBOL_NAMES");
    assert!(code.contains("SYMBOL_METADATA"), "missing SYMBOL_METADATA");
    assert!(code.contains("FIELD_NAMES"), "missing FIELD_NAMES");
}

#[test]
fn static_gen_node_types_is_valid_json() {
    let (g, pt) = builder_grammar_and_table();
    let generator = StaticLanguageGenerator::new(g, pt);
    let json_str = generator.generate_node_types();
    let parsed: serde_json::Value =
        serde_json::from_str(&json_str).expect("node types must be valid JSON");
    assert!(parsed.is_array());
}

// ---------------------------------------------------------------------------
// 12. LanguageValidator
// ---------------------------------------------------------------------------

#[test]
fn validator_rejects_wrong_version() {
    let lang = adze_tablegen::validation::TSLanguage {
        version: 14,
        symbol_count: 5,
        alias_count: 0,
        token_count: 3,
        external_token_count: 0,
        state_count: 10,
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
        external_scanner_data: adze_tablegen::validation::TSExternalScannerData {
            states: std::ptr::null(),
            symbol_map: std::ptr::null(),
            create: None,
            destroy: None,
            scan: None,
            serialize: None,
            deserialize: None,
        },
        primary_state_ids: std::ptr::null(),
    };
    let tables = CompressedParseTable::new_for_testing(5, 10);
    let validator = LanguageValidator::new(&lang, &tables);
    let result = validator.validate();
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(
        errors
            .iter()
            .any(|e| matches!(e, ValidationError::InvalidVersion { .. }))
    );
}

#[test]
fn validator_reports_symbol_count_mismatch() {
    let lang = adze_tablegen::validation::TSLanguage {
        version: 15,
        symbol_count: 99,
        alias_count: 0,
        token_count: 3,
        external_token_count: 0,
        state_count: 10,
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
        external_scanner_data: adze_tablegen::validation::TSExternalScannerData {
            states: std::ptr::null(),
            symbol_map: std::ptr::null(),
            create: None,
            destroy: None,
            scan: None,
            serialize: None,
            deserialize: None,
        },
        primary_state_ids: std::ptr::null(),
    };
    let tables = CompressedParseTable::new_for_testing(5, 10);
    let validator = LanguageValidator::new(&lang, &tables);
    let errors = validator.validate().unwrap_err();
    assert!(
        errors
            .iter()
            .any(|e| matches!(e, ValidationError::SymbolCountMismatch { .. }))
    );
}

#[test]
fn validator_reports_null_symbol_names() {
    let lang = adze_tablegen::validation::TSLanguage {
        version: 15,
        symbol_count: 5,
        alias_count: 0,
        token_count: 3,
        external_token_count: 0,
        state_count: 10,
        large_state_count: 0,
        production_id_count: 0,
        field_count: 0,
        max_alias_sequence_length: 0,
        parse_table: std::ptr::null(),
        small_parse_table: [0u16; 4].as_ptr(),
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
        external_scanner_data: adze_tablegen::validation::TSExternalScannerData {
            states: std::ptr::null(),
            symbol_map: std::ptr::null(),
            create: None,
            destroy: None,
            scan: None,
            serialize: None,
            deserialize: None,
        },
        primary_state_ids: std::ptr::null(),
    };
    let tables = CompressedParseTable::new_for_testing(5, 10);
    let validator = LanguageValidator::new(&lang, &tables);
    let errors = validator.validate().unwrap_err();
    assert!(
        errors
            .iter()
            .any(|e| matches!(e, ValidationError::NullPointer("symbol_names")))
    );
}

// ---------------------------------------------------------------------------
// 13. CompressedParseTable basics
// ---------------------------------------------------------------------------

#[test]
fn compressed_parse_table_roundtrip_counts() {
    let table = CompressedParseTable::new_for_testing(42, 7);
    assert_eq!(table.symbol_count(), 42);
    assert_eq!(table.state_count(), 7);
}

// ---------------------------------------------------------------------------
// 14. Grammar with external tokens in LanguageGenerator
// ---------------------------------------------------------------------------

#[test]
fn gen_with_external_tokens() {
    let mut grammar = make_grammar(
        "ext",
        vec![(SymbolId(1), regex_token("id", "[a-z]+"))],
        vec![],
    );
    grammar.externals.push(ExternalToken {
        name: "indent".to_string(),
        symbol_id: SymbolId(50),
    });
    let pt = make_parse_table_for_gen(&grammar, 2, vec![vec![vec![]; 3]; 2]);
    let generator = LanguageGenerator::new(&grammar, &pt);
    let output = generator.generate().to_string();
    assert!(
        output.contains("EXTERNAL_TOKEN_COUNT"),
        "should reference external token count"
    );
}

// ---------------------------------------------------------------------------
// 15. Grammar name propagates to FFI export
// ---------------------------------------------------------------------------

#[test]
fn gen_ffi_name_matches_grammar() {
    let grammar = make_grammar(
        "my_lang",
        vec![(SymbolId(1), regex_token("x", "."))],
        vec![],
    );
    let pt = make_parse_table_for_gen(&grammar, 1, vec![vec![vec![]; 2]; 1]);
    let generator = LanguageGenerator::new(&grammar, &pt);
    let code = generator.generate().to_string();
    assert!(
        code.contains("tree_sitter_my_lang"),
        "FFI name must incorporate grammar name"
    );
}

// ---------------------------------------------------------------------------
// 16. Node types exclude internal rules
// ---------------------------------------------------------------------------

#[test]
fn node_types_excludes_internal_rules() {
    let mut grammar = Grammar::new("internal".to_string());
    grammar.tokens.insert(SymbolId(1), regex_token("n", r"\d+"));
    // Internal rule name starts with '_'
    grammar
        .rule_names
        .insert(SymbolId(2), "_internal_helper".to_string());
    grammar.add_rule(Rule {
        lhs: SymbolId(2),
        rhs: vec![Symbol::Terminal(SymbolId(1))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });

    let generator = NodeTypesGenerator::new(&grammar);
    let json = generator.generate().unwrap();
    let arr: Vec<serde_json::Value> = serde_json::from_str(&json).unwrap();
    let has_internal = arr
        .iter()
        .any(|v| v["type"].as_str().unwrap_or("").starts_with('_'));
    assert!(
        !has_internal,
        "internal rules must be excluded from node types"
    );
}
