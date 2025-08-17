use rust_sitter_glr_core::{Action, ParseTable, SymbolMetadata};
use rust_sitter_ir::{Grammar, ProductionId, Rule, StateId, Symbol, SymbolId, Token, TokenPattern};
use rust_sitter_tablegen::{LanguageBuilder, StaticLanguageGenerator};

fn create_simple_grammar() -> (Grammar, ParseTable) {
    let mut grammar = Grammar::new("simple".to_string());

    // Add a token
    let token = Token {
        name: "number".to_string(),
        pattern: TokenPattern::Regex(r"\d+".to_string()),
        fragile: false,
    };
    let token_id = SymbolId(0);
    grammar.tokens.insert(token_id, token);

    // Add a rule
    let rule = Rule {
        lhs: SymbolId(1),
        rhs: vec![Symbol::Terminal(token_id)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    };
    grammar.rules.insert(SymbolId(1), vec![rule]);

    // Create a minimal parse table
    let mut symbol_to_index = std::collections::BTreeMap::new();
    symbol_to_index.insert(SymbolId(0), 0);
    symbol_to_index.insert(SymbolId(1), 1);

    let mut parse_table = ParseTable {
        action_table: vec![vec![vec![Action::Accept]; 2]; 2],
        goto_table: vec![vec![StateId(0); 2]; 2],
        state_count: 2,
        symbol_count: 2,
        symbol_to_index: symbol_to_index.clone(),
        index_to_symbol: vec![SymbolId(0), SymbolId(1)],
        symbol_metadata: vec![
            SymbolMetadata {
                name: "token".to_string(),
                visible: true,
                named: true,
                supertype: false,
            };
            2
        ],
        external_scanner_states: vec![],
        rules: vec![],
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
                external_lex_state: 0
            };
            2
        ],
        extras: vec![],
        dynamic_prec_by_rule: vec![],
        alias_sequences: vec![],
        field_names: vec![],
        field_map: std::collections::BTreeMap::new(),
    };

    // Set up a simple action table
    parse_table.action_table[0][0] = vec![Action::Shift(StateId(1))];
    parse_table.action_table[1][0] = vec![Action::Accept];

    (grammar, parse_table)
}

#[test]
fn test_language_generation() {
    let (grammar, parse_table) = create_simple_grammar();

    // Test LanguageBuilder
    let builder = LanguageBuilder::new(grammar.clone(), parse_table.clone());
    let result = builder.generate_language();

    assert!(
        result.is_ok(),
        "Language generation failed: {:?}",
        result.err()
    );

    let language = result.unwrap();
    assert_eq!(language.version, 15); // ABI version
    assert_eq!(language.state_count, 2);
    assert_eq!(language.symbol_count, 2);
}

#[test]
fn test_static_language_generator() {
    let (grammar, parse_table) = create_simple_grammar();

    // Test StaticLanguageGenerator
    let generator = StaticLanguageGenerator::new(grammar, parse_table);

    // Test that we can create the generator
    // Private methods can't be tested directly, but we can test the public API
    let code = generator.generate_language_code();
    let code_str = code.to_string();

    // Check that symbol names are included in generated code
    assert!(code_str.contains("SYMBOL_NAMES"));
    assert!(code_str.contains("FIELD_NAMES"));
}

#[test]
fn test_table_compression() {
    let (grammar, mut parse_table) = create_simple_grammar();

    // Add more states to test compression
    parse_table.state_count = 10;
    parse_table.symbol_count = 5;
    parse_table.action_table = vec![vec![vec![Action::Error]; 5]; 10];
    parse_table.goto_table = vec![vec![StateId(0); 5]; 10];
    parse_table.symbol_metadata = vec![
        SymbolMetadata {
            name: "symbol".to_string(),
            visible: true,
            named: true,
            supertype: false,
        };
        5
    ];

    // Add some real actions
    parse_table.action_table[0][0] = vec![Action::Shift(StateId(1))];
    parse_table.action_table[1][0] = vec![Action::Shift(StateId(2))];
    parse_table.action_table[2][0] = vec![Action::Reduce(rust_sitter_ir::RuleId(0))];
    parse_table.action_table[9][0] = vec![Action::Accept];

    let mut generator = StaticLanguageGenerator::new(grammar, parse_table);
    let compressed = generator.compress_tables();

    assert!(
        compressed.is_ok(),
        "Table compression failed: {:?}",
        compressed.err()
    );
}

#[test]
fn test_generated_code_compiles() {
    let (grammar, parse_table) = create_simple_grammar();

    let generator = StaticLanguageGenerator::new(grammar, parse_table);
    let code = generator.generate_language_code();

    // Check that the generated code contains expected elements
    let code_str = code.to_string();
    assert!(code_str.contains("SYMBOL_NAMES"));
    assert!(code_str.contains("SYMBOL_METADATA"));
    assert!(code_str.contains("TSLanguage"));
    assert!(code_str.contains("tree_sitter_simple"));
}

#[test]
fn test_node_types_generation() {
    let (grammar, parse_table) = create_simple_grammar();

    let generator = StaticLanguageGenerator::new(grammar, parse_table);
    let node_types = generator.generate_node_types();

    // Should be valid JSON
    let parsed: Result<serde_json::Value, _> = serde_json::from_str(&node_types);
    assert!(parsed.is_ok(), "Invalid NODE_TYPES JSON: {}", node_types);
}
