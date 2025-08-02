// Comprehensive tests for the pure-Rust Tree-sitter implementation
// Tests the tablegen module's functionality

use rust_sitter_glr_core::{Action, ParseTable};
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
    let mut parse_table = ParseTable {
        action_table: vec![],
        goto_table: vec![],
        symbol_metadata: vec![],
        state_count: 0,
        symbol_count: 0,
        symbol_to_index: std::collections::BTreeMap::new(),
    };

    // Add some basic states
    parse_table
        .action_table
        .push(vec![Action::Shift(StateId(1)), Action::Error]);

    parse_table
        .action_table
        .push(vec![Action::Shift(StateId(2)), Action::Error]);

    parse_table
        .action_table
        .push(vec![Action::Shift(StateId(3)), Action::Error]);

    parse_table
        .action_table
        .push(vec![Action::Accept, Action::Reduce(RuleId(0))]);

    parse_table.goto_table = vec![
        vec![StateId(0)],
        vec![StateId(1)],
        vec![StateId(2)],
        vec![StateId(3)],
    ];

    parse_table.state_count = 4;
    parse_table.symbol_count = 4;

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
    let parse_table = create_test_parse_table();
    let compressor = TableCompressor::new();

    let compressed = compressor.compress(&parse_table);
    assert!(compressed.is_ok());

    let compressed = compressed.unwrap();
    assert!(compressed.validate(&parse_table).is_ok());
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
    assert!(code_str.contains("visible"));
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

    let compressed = compressor.compress(&parse_table).unwrap();

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
    assert!(TREE_SITTER_LANGUAGE_VERSION >= TREE_SITTER_MIN_COMPATIBLE_LANGUAGE_VERSION);

    // Verify symbol metadata creation
    let metadata = create_symbol_metadata(true, true, false, false, false);
    assert_eq!(metadata, symbol_metadata::VISIBLE | symbol_metadata::NAMED);
}
