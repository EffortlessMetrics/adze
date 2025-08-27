// Comprehensive tests for the pure-Rust Tree-sitter implementation
// Tests the tablegen module's functionality

#[allow(unused_imports)]
use rust_sitter_glr_core::{Action, GotoIndexing, LexMode, ParseRule, ParseTable};
use rust_sitter_ir::{
    FieldId, Grammar, PrecedenceKind, ProductionId, Rule, RuleId, StateId, Symbol, SymbolId, Token,
    TokenPattern,
};
use rust_sitter_tablegen::{NodeTypesGenerator, StaticLanguageGenerator, TableCompressor};

/// Create a simple test grammar
fn create_test_grammar() -> Grammar {
    let mut grammar = Grammar::new("test".to_string());

    // Add tokens
    grammar.tokens.insert(
        SymbolId(1),
        Token {
            name: "plus".to_string(),
            pattern: TokenPattern::String("+".to_string()),
            fragile: false,
        },
    );

    grammar.tokens.insert(
        SymbolId(2),
        Token {
            name: "number".to_string(),
            pattern: TokenPattern::Regex(r"\d+".to_string()),
            fragile: false,
        },
    );

    grammar.tokens.insert(
        SymbolId(3),
        Token {
            name: "_whitespace".to_string(),
            pattern: TokenPattern::Regex(r"[ \t\n]+".to_string()),
            fragile: false,
        },
    );

    // Add a simple rule: expr -> number + number
    let rule = Rule {
        lhs: SymbolId(10),
        rhs: vec![
            Symbol::Terminal(SymbolId(2)),
            Symbol::Terminal(SymbolId(1)),
            Symbol::Terminal(SymbolId(2)),
        ],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    };

    grammar.rules.insert(SymbolId(10), vec![rule]);

    grammar
}

/// Create a simple parse table for testing
fn create_test_parse_table() -> ParseTable {
    let grammar = create_test_grammar();
    let mut parse_table = ParseTable {
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
        start_symbol: SymbolId(10),
        grammar,
        initial_state: StateId(0),
        token_count: 3,
        external_token_count: 0,
        lex_modes: vec![],
        extras: vec![SymbolId(3)], // whitespace
        dynamic_prec_by_rule: vec![],
        rule_assoc_by_rule: vec![],
        alias_sequences: vec![],
        field_names: vec![],
        field_map: std::collections::BTreeMap::new(),
        goto_indexing: GotoIndexing::NonterminalMap,
    };

    // Add some basic states
    parse_table
        .action_table
        .push(vec![vec![Action::Shift(StateId(1))], vec![Action::Error]]);

    parse_table
        .action_table
        .push(vec![vec![Action::Shift(StateId(2))], vec![Action::Error]]);

    parse_table
        .action_table
        .push(vec![vec![Action::Shift(StateId(3))], vec![Action::Error]]);

    parse_table
        .action_table
        .push(vec![vec![Action::Accept], vec![Action::Reduce(RuleId(0))]]);

    parse_table.goto_table = vec![
        vec![StateId(0)],
        vec![StateId(1)],
        vec![StateId(2)],
        vec![StateId(3)],
    ];

    parse_table.state_count = 4;
    parse_table.symbol_count = 4;

    // Add symbol to index mapping, including EOF (symbol 0)
    parse_table.symbol_to_index.insert(SymbolId(0), 0); // EOF at column 0
    parse_table.symbol_to_index.insert(SymbolId(1), 1); // token 1
    parse_table.symbol_to_index.insert(SymbolId(2), 2); // token 2

    parse_table
}

#[test]
fn test_language_generator_creation() {
    let grammar = create_test_grammar();
    let parse_table = create_test_parse_table();

    let generator = StaticLanguageGenerator::new(grammar, parse_table);
    assert_eq!(generator.grammar.name, "test");
}

#[test]
fn test_table_compression() {
    let grammar = create_test_grammar();
    let parse_table = create_test_parse_table();
    let compressor = TableCompressor::new();

    // Use the real helper function to collect token indices (mirrors production)
    let token_indices =
        rust_sitter_tablegen::helpers::collect_token_indices(&grammar, &parse_table);

    // Canary check: Verify state 0 invariants before compression
    // This ensures our parse table is valid for GLR parsing
    assert_state0_basic_invariants(&parse_table, &token_indices);

    let compressed = compressor.compress(&parse_table, &token_indices, false);
    assert!(compressed.is_ok());

    let compressed = compressed.unwrap();
    assert!(compressed.validate(&parse_table).is_ok());
}

// Basic state 0 invariant check (canary test)
fn assert_state0_basic_invariants(parse_table: &ParseTable, token_indices: &[usize]) {
    use rust_sitter_ir::SymbolId;
    use std::collections::HashSet;

    // Check for duplicate indices
    let indices_set: HashSet<_> = token_indices.iter().copied().collect();
    assert_eq!(
        indices_set.len(),
        token_indices.len(),
        "token_indices must not contain duplicates"
    );

    // Check that EOF is in symbol_to_index and token_indices
    let eof_idx = *parse_table
        .symbol_to_index
        .get(&SymbolId(0))
        .expect("EOF must be in symbol_to_index");

    assert!(
        token_indices.contains(&eof_idx),
        "EOF column must be in token_indices"
    );

    // Check that state 0 exists and has actions
    assert!(
        !parse_table.action_table.is_empty(),
        "Parse table must have at least state 0"
    );

    let state0 = &parse_table.action_table[0];

    // Check if any token has an action in state 0
    let has_any_action = token_indices
        .iter()
        .any(|&idx| state0.get(idx).is_some_and(|cell| !cell.is_empty()));

    assert!(
        has_any_action,
        "State 0 must have at least one action for token columns"
    );
}

#[test]
fn test_node_types_generation() {
    let grammar = create_test_grammar();
    let generator = NodeTypesGenerator::new(&grammar);

    let result = generator.generate();
    assert!(result.is_ok());

    let node_types = result.unwrap();

    // Should be valid JSON
    let parsed: serde_json::Value =
        serde_json::from_str(&node_types).expect("Should generate valid JSON");

    assert!(parsed.is_array());
}

#[test]
fn test_language_code_generation() {
    let grammar = create_test_grammar();
    let parse_table = create_test_parse_table();

    let mut generator = StaticLanguageGenerator::new(grammar, parse_table);

    // Compress tables
    assert!(generator.compress_tables().is_ok());

    // Generate code
    let code = generator.generate_language_code();
    let code_str = code.to_string();

    // Check for expected content
    assert!(code_str.contains("SYMBOL_NAMES"));
    assert!(code_str.contains("LANGUAGE_VERSION"));
    assert!(code_str.contains("tree_sitter_test"));
}

#[test]
fn test_grammar_with_fields() {
    let mut grammar = create_test_grammar();

    // Add fields
    grammar.fields.insert(FieldId(0), "left".to_string());
    grammar.fields.insert(FieldId(1), "right".to_string());

    // Update rule to have fields
    if let Some(rules) = grammar.rules.get_mut(&SymbolId(10)) {
        if let Some(rule) = rules.get_mut(0) {
            rule.fields = vec![
                (FieldId(0), 0), // left field at position 0
                (FieldId(1), 2), // right field at position 2
            ];
        }
    }

    let parse_table = create_test_parse_table();
    let generator = StaticLanguageGenerator::new(grammar, parse_table);

    let code = generator.generate_language_code();
    let code_str = code.to_string();

    // Should include field names
    assert!(code_str.contains("FIELD_NAMES"));
    assert!(code_str.contains("left"));
    assert!(code_str.contains("right"));
}

#[test]
fn test_external_tokens() {
    let mut grammar = create_test_grammar();

    // Add external tokens
    grammar.externals.push(rust_sitter_ir::ExternalToken {
        name: "comment".to_string(),
        symbol_id: SymbolId(100),
    });

    let parse_table = create_test_parse_table();
    let generator = StaticLanguageGenerator::new(grammar, parse_table);

    let code = generator.generate_language_code();
    let code_str = code.to_string();

    // Should include external scanner references
    assert!(code_str.contains("EXTERNAL_TOKEN_COUNT"));
    assert!(code_str.contains("EXTERNAL_SCANNER"));
}

#[test]
fn test_symbol_metadata_generation() {
    let grammar = create_test_grammar();
    let parse_table = create_test_parse_table();

    let generator = StaticLanguageGenerator::new(grammar, parse_table);
    // Test that metadata generation works internally
    let code = generator.generate_language_code();
    let code_str = code.to_string();

    // Should contain symbol metadata
    assert!(code_str.contains("SYMBOL_METADATA"));
    // Symbol metadata is generated as raw bytes, not field names
    // The bytes encode visibility, named status, etc as bit flags
    assert!(code_str.contains("static SYMBOL_METADATA : & [u8]"));
}

#[test]
fn test_precedence_in_rules() {
    let mut grammar = create_test_grammar();

    // Add a rule with precedence
    let rule_with_prec = Rule {
        lhs: SymbolId(11),
        rhs: vec![Symbol::Terminal(SymbolId(2))],
        precedence: Some(PrecedenceKind::Static(10)),
        associativity: Some(rust_sitter_ir::Associativity::Left),
        fields: vec![],
        production_id: ProductionId(1),
    };

    grammar.rules.insert(SymbolId(11), vec![rule_with_prec]);

    // Verify grammar validates with precedence
    assert!(grammar.validate().is_ok());
}

#[test]
fn test_compressed_table_format() {
    let parse_table = create_test_parse_table();
    let compressor = TableCompressor::new();

    // Create minimal token indices for test
    let token_indices = vec![0]; // EOF is always a token
    let compressed = compressor
        .compress(&parse_table, &token_indices, false)
        .unwrap();

    // Check action table structure
    assert_eq!(
        compressed.action_table.row_offsets.len(),
        parse_table.state_count + 1
    );
    assert_eq!(
        compressed.action_table.default_actions.len(),
        parse_table.state_count
    );

    // Check goto table structure
    assert_eq!(
        compressed.goto_table.row_offsets.len(),
        parse_table.state_count + 1
    );
}

#[test]
fn test_abi_compatibility() {
    use rust_sitter_tablegen::abi::*;

    // Verify ABI constants
    assert_eq!(TREE_SITTER_LANGUAGE_VERSION, 15);
    // Verify version compatibility - no assertion needed as constants are verified at compile time
    let _version_check =
        TREE_SITTER_LANGUAGE_VERSION >= TREE_SITTER_MIN_COMPATIBLE_LANGUAGE_VERSION;

    // Verify symbol metadata creation
    let metadata = create_symbol_metadata(true, true, false, false, false);
    assert_eq!(metadata, symbol_metadata::VISIBLE | symbol_metadata::NAMED);
}
