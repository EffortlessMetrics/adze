use rust_sitter_glr_core::{Action, GotoIndexing, LexMode, ParseTable, SymbolMetadata};
use rust_sitter_ir::{ExternalToken, Grammar, StateId, SymbolId};
use rust_sitter_tablegen::StaticLanguageGenerator;
use rust_sitter_tablegen::external_scanner::ExternalScannerGenerator;

#[test]
fn test_language_generation_with_external_scanner() {
    // Create a grammar with external tokens (like heredoc strings)
    let mut grammar = Grammar::new("shell".to_string());

    // Add external tokens
    grammar.externals.push(ExternalToken {
        name: "HEREDOC_BODY".to_string(),
        symbol_id: SymbolId(100),
    });

    grammar.externals.push(ExternalToken {
        name: "TEMPLATE_STRING".to_string(),
        symbol_id: SymbolId(101),
    });

    // Create a simple parse table
    let mut symbol_to_index = std::collections::BTreeMap::new();
    for i in 0..102 {
        symbol_to_index.insert(SymbolId(i), i as usize);
    }
    let index_to_symbol: Vec<SymbolId> = (0..102).map(|i| SymbolId(i)).collect();

    let mut parse_table = ParseTable {
        state_count: 5,
        symbol_count: 102, // Must include external token IDs
        symbol_to_index: symbol_to_index.clone(),
        index_to_symbol,
        action_table: vec![vec![vec![Action::Error]; 102]; 5],
        goto_table: vec![vec![StateId(0); 102]; 5],
        symbol_metadata: vec![],
        external_scanner_states: vec![],
        rules: vec![],
        nonterminal_to_index: std::collections::BTreeMap::new(),
        eof_symbol: SymbolId(0),
        start_symbol: SymbolId(1),
        grammar: grammar.clone(),
        initial_state: StateId(0),
        token_count: 50,
        external_token_count: 2,
        lex_modes: vec![
            LexMode {
                lex_state: 0,
                external_lex_state: 0
            };
            5
        ],
        extras: vec![],
        dynamic_prec_by_rule: vec![],
        rule_assoc_by_rule: vec![],
        alias_sequences: vec![],
        field_names: vec![],
        field_map: std::collections::BTreeMap::new(),
        goto_indexing: GotoIndexing::NonterminalMap,
    };

    // Add symbol metadata for all symbols including externals
    parse_table.symbol_metadata = vec![
        SymbolMetadata {
            name: "EOF".to_string(),
            visible: false,
            named: false,
            supertype: false,
        };
        102
    ];

    // Mark external tokens as visible and named
    parse_table.symbol_metadata[100] = SymbolMetadata {
        name: "HEREDOC_BODY".to_string(),
        visible: true,
        named: true,
        supertype: false,
    };

    parse_table.symbol_metadata[101] = SymbolMetadata {
        name: "TEMPLATE_STRING".to_string(),
        visible: true,
        named: true,
        supertype: false,
    };

    // Create the language generator
    let mut generator = StaticLanguageGenerator::new(grammar.clone(), parse_table);

    // Compress tables
    generator.compress_tables().unwrap();

    // Generate the language code
    let code = generator.generate_language_code();
    let code_str = code.to_string();

    // Print the generated code for debugging
    println!("Generated code:\n{}", code_str);

    // Verify external scanner data is generated
    assert!(code_str.contains("EXTERNAL_TOKEN_COUNT"));
    assert!(code_str.contains("EXTERNAL_SCANNER_DATA"));
    assert!(code_str.contains("EXTERNAL_SCANNER_STATES"));
    assert!(code_str.contains("EXTERNAL_SCANNER_SYMBOL_MAP"));

    // Test external scanner generator directly
    let scanner_gen = ExternalScannerGenerator::new(grammar);
    assert_eq!(scanner_gen.external_token_count(), 2);
    assert!(scanner_gen.has_external_tokens());

    let symbol_map = scanner_gen.generate_symbol_map();
    assert_eq!(symbol_map, vec![100, 101]);
}

#[test]
fn test_node_types_include_external_tokens() {
    let mut grammar = Grammar::new("test".to_string());

    // Add external token
    grammar.externals.push(ExternalToken {
        name: "COMMENT".to_string(),
        symbol_id: SymbolId(50),
    });

    // Add hidden external token (should not appear in NODE_TYPES)
    grammar.externals.push(ExternalToken {
        name: "_WHITESPACE".to_string(),
        symbol_id: SymbolId(51),
    });

    let mut symbol_to_index = std::collections::BTreeMap::new();
    for i in 0..52 {
        symbol_to_index.insert(SymbolId(i), i as usize);
    }
    let index_to_symbol: Vec<SymbolId> = (0..52).map(|i| SymbolId(i)).collect();

    let parse_table = ParseTable {
        state_count: 1,
        symbol_to_index: symbol_to_index.clone(),
        index_to_symbol,
        symbol_count: 52,
        action_table: vec![vec![vec![Action::Error]; 52]; 1],
        goto_table: vec![vec![StateId(0); 52]; 1],
        rules: vec![],
        nonterminal_to_index: std::collections::BTreeMap::new(),
        eof_symbol: SymbolId(0),
        start_symbol: SymbolId(1),
        grammar: grammar.clone(),
        initial_state: StateId(0),
        token_count: 49,
        external_token_count: 2,
        lex_modes: vec![
            LexMode {
                lex_state: 0,
                external_lex_state: 0
            };
            1
        ],
        extras: vec![],
        dynamic_prec_by_rule: vec![],
        rule_assoc_by_rule: vec![],
        alias_sequences: vec![],
        field_names: vec![],
        field_map: std::collections::BTreeMap::new(),
        symbol_metadata: vec![
            SymbolMetadata {
                name: "EOF".to_string(),
                visible: false,
                named: false,
                supertype: false,
            };
            52
        ],
        external_scanner_states: vec![],
        goto_indexing: GotoIndexing::NonterminalMap,
    };
    let generator = StaticLanguageGenerator::new(grammar, parse_table);

    let node_types_json = generator.generate_node_types();

    // Verify COMMENT appears but _WHITESPACE doesn't
    assert!(node_types_json.contains("\"type\": \"COMMENT\""));
    assert!(node_types_json.contains("\"named\": true"));
    assert!(!node_types_json.contains("_WHITESPACE"));
}

#[test]
fn test_external_scanner_state_computation() {
    // Test that external scanner states are correctly computed
    // In a real implementation, this would be computed from the parse table
    let mut grammar = Grammar::new("markdown".to_string());

    grammar.externals.push(ExternalToken {
        name: "CODE_BLOCK".to_string(),
        symbol_id: SymbolId(200),
    });

    grammar.externals.push(ExternalToken {
        name: "HTML_BLOCK".to_string(),
        symbol_id: SymbolId(201),
    });

    let scanner_gen = ExternalScannerGenerator::new(grammar);

    // Test state bitmap generation for 3 states
    let state_bitmap = scanner_gen.generate_state_bitmap(3);

    assert_eq!(state_bitmap.len(), 3);
    assert_eq!(state_bitmap[0].len(), 2);

    // Currently all tokens are valid in all states
    // TODO: Compute actual validity from parse table
    for state in &state_bitmap {
        assert!(state.iter().all(|&valid| valid));
    }
}
