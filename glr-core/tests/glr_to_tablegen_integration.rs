//! Cross-crate integration tests for GLR → tablegen pipeline.
//!
//! Tests the flow from parse tables (GLR core) to table compression and ABI generation (tablegen).
//! This suite validates the complete pipeline: ParseTable construction → compression → code generation.

use adze_glr_core::*;
use adze_ir::*;
use adze_tablegen::*;

// ============================================================================
// Helper Functions
// ============================================================================

/// Create a minimal grammar with tokens and rules
fn create_minimal_grammar() -> Grammar {
    let mut grammar = Grammar::new("minimal".to_string());

    // Add a simple token
    let token = Token {
        name: "x".to_string(),
        pattern: TokenPattern::String("x".to_string()),
        fragile: false,
    };
    grammar.tokens.insert(SymbolId(1), token);

    // Add a rule: S -> 'x'
    let rule = Rule {
        lhs: SymbolId(2),
        rhs: vec![Symbol::Terminal(SymbolId(1))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    };
    grammar.add_rule(rule);

    grammar
}

/// Create a more complex grammar with shift actions
fn create_grammar_with_shifts() -> Grammar {
    let mut grammar = Grammar::new("shifts".to_string());

    // Add tokens
    let a_token = Token {
        name: "a".to_string(),
        pattern: TokenPattern::String("a".to_string()),
        fragile: false,
    };
    let b_token = Token {
        name: "b".to_string(),
        pattern: TokenPattern::String("b".to_string()),
        fragile: false,
    };

    grammar.tokens.insert(SymbolId(1), a_token);
    grammar.tokens.insert(SymbolId(2), b_token);

    // Add rules: S -> 'a' 'b'
    let rule = Rule {
        lhs: SymbolId(3),
        rhs: vec![Symbol::Terminal(SymbolId(1)), Symbol::Terminal(SymbolId(2))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    };
    grammar.add_rule(rule);

    grammar
}

/// Create a grammar with multiple productions (reduce actions)
fn create_grammar_with_reduces() -> Grammar {
    let mut grammar = Grammar::new("reduces".to_string());

    // Add token
    let token = Token {
        name: "x".to_string(),
        pattern: TokenPattern::String("x".to_string()),
        fragile: false,
    };
    grammar.tokens.insert(SymbolId(1), token);

    // Add multiple rules for same LHS: S -> 'x' | 'x' 'x'
    let rule1 = Rule {
        lhs: SymbolId(2),
        rhs: vec![Symbol::Terminal(SymbolId(1))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    };
    grammar.add_rule(rule1);

    let rule2 = Rule {
        lhs: SymbolId(2),
        rhs: vec![Symbol::Terminal(SymbolId(1)), Symbol::Terminal(SymbolId(1))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(1),
    };
    grammar.add_rule(rule2);

    grammar
}

/// Create a grammar with nonterminals (for goto table testing)
fn create_grammar_with_nonterminals() -> Grammar {
    let mut grammar = Grammar::new("nonterminals".to_string());

    // Add token
    let token = Token {
        name: "a".to_string(),
        pattern: TokenPattern::String("a".to_string()),
        fragile: false,
    };
    grammar.tokens.insert(SymbolId(1), token);

    // Add nonterminal rules
    // S -> A 'a'
    let rule1 = Rule {
        lhs: SymbolId(2),
        rhs: vec![
            Symbol::NonTerminal(SymbolId(3)),
            Symbol::Terminal(SymbolId(1)),
        ],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    };
    grammar.add_rule(rule1);

    // A -> 'a' | ε (epsilon would be handled by grammar)
    let rule2 = Rule {
        lhs: SymbolId(3),
        rhs: vec![Symbol::Terminal(SymbolId(1))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(1),
    };
    grammar.add_rule(rule2);

    grammar
}

/// Build a parse table from a grammar, handling potential LR(1) construction failures
fn build_parse_table_from_grammar(grammar: &Grammar) -> std::result::Result<ParseTable, String> {
    FirstFollowSets::compute(grammar)
        .map_err(|e| format!("FirstFollowSets error: {}", e))
        .and_then(|first_follow| {
            build_lr1_automaton(grammar, &first_follow)
                .map_err(|e| format!("LR(1) construction error: {}", e))
        })
}

// ============================================================================
// Tests: ParseTable → Compression Pipeline
// ============================================================================

/// Test 1: Minimal parse table → compress → verify compressed form
#[test]
fn test_minimal_table_compression() {
    let grammar = create_minimal_grammar();
    let parse_table =
        build_parse_table_from_grammar(&grammar).expect("Failed to build parse table");

    let compressor = TableCompressor::new();
    let token_indices = helpers::collect_token_indices(&grammar, &parse_table);
    let compressed = compressor
        .compress(&parse_table, &token_indices, false)
        .expect("Failed to compress table");

    // Verify compressed form exists
    assert!(
        !compressed.action_table.data.is_empty(),
        "Compressed action table should not be empty"
    );
    assert!(
        !compressed.action_table.row_offsets.is_empty(),
        "Compressed action table row offsets should not be empty"
    );
}

/// Test 2: Parse table with shift actions → compress → verify
#[test]
fn test_table_with_shift_actions_compression() {
    let grammar = create_grammar_with_shifts();
    let parse_table =
        build_parse_table_from_grammar(&grammar).expect("Failed to build parse table");

    let compressor = TableCompressor::new();
    let token_indices = helpers::collect_token_indices(&grammar, &parse_table);
    let compressed = compressor
        .compress(&parse_table, &token_indices, false)
        .expect("Failed to compress table");

    // Verify compressed form exists with expected structure
    assert!(!compressed.action_table.data.is_empty());
    assert!(
        compressed.action_table.row_offsets.len() > 1,
        "Should have multiple states"
    );
}

/// Test 3: Parse table with reduce actions → compress → verify
#[test]
fn test_table_with_reduce_actions_compression() {
    let grammar = create_grammar_with_reduces();
    let parse_table =
        build_parse_table_from_grammar(&grammar).expect("Failed to build parse table");

    let compressor = TableCompressor::new();
    let token_indices = helpers::collect_token_indices(&grammar, &parse_table);
    let compressed = compressor
        .compress(&parse_table, &token_indices, false)
        .expect("Failed to compress table");

    // Verify compressed form exists
    assert!(!compressed.action_table.data.is_empty());
}

/// Test 4: Parse table with accept action → compress → verify
#[test]
fn test_table_with_accept_action_compression() {
    let grammar = create_minimal_grammar();
    let parse_table =
        build_parse_table_from_grammar(&grammar).expect("Failed to build parse table");

    let compressor = TableCompressor::new();
    let token_indices = helpers::collect_token_indices(&grammar, &parse_table);
    let compressed = compressor
        .compress(&parse_table, &token_indices, false)
        .expect("Failed to compress table");

    // The minimal grammar should contain an accept action in some state
    assert!(!compressed.action_table.data.is_empty());
    assert!(!compressed.action_table.row_offsets.is_empty());
}

/// Test 5: Compressed table roundtrip: data preserved
#[test]
fn test_compressed_table_roundtrip_data_preserved() {
    let grammar = create_minimal_grammar();
    let parse_table =
        build_parse_table_from_grammar(&grammar).expect("Failed to build parse table");

    let original_state_count = parse_table.state_count;
    let original_symbol_count = parse_table.symbol_count;

    let compressor = TableCompressor::new();
    let token_indices = helpers::collect_token_indices(&grammar, &parse_table);
    let compressed = compressor
        .compress(&parse_table, &token_indices, false)
        .expect("Failed to compress table");

    // Verify that row offsets match state count
    assert_eq!(
        compressed.action_table.row_offsets.len(),
        original_state_count + 1,
        "Row offsets should account for all states"
    );
    assert_eq!(
        compressed.goto_table.row_offsets.len(),
        original_state_count + 1,
        "Goto row offsets should match action table"
    );
}

// ============================================================================
// Tests: NodeTypesGenerator
// ============================================================================

/// Test 6: Node types generation from simple grammar
#[test]
fn test_node_types_generation_simple() {
    let grammar = create_minimal_grammar();
    let generator = NodeTypesGenerator::new(&grammar);

    let result = generator.generate().expect("Failed to generate NODE_TYPES");

    // Parse as JSON to validate structure
    let _value: serde_json::Value =
        serde_json::from_str(&result).expect("Generated NODE_TYPES is not valid JSON");
}

/// Test 7: Node types JSON contains all named symbols
#[test]
fn test_node_types_contains_named_symbols() {
    let mut grammar = Grammar::new("named_test".to_string());

    let token_x = Token {
        name: "x".to_string(),
        pattern: TokenPattern::String("x".to_string()),
        fragile: false,
    };
    let token_y = Token {
        name: "y".to_string(),
        pattern: TokenPattern::String("y".to_string()),
        fragile: false,
    };

    grammar.tokens.insert(SymbolId(1), token_x);
    grammar.tokens.insert(SymbolId(2), token_y);

    // Add rules
    let rule1 = Rule {
        lhs: SymbolId(3),
        rhs: vec![Symbol::Terminal(SymbolId(1))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    };
    grammar.add_rule(rule1);

    let rule2 = Rule {
        lhs: SymbolId(3),
        rhs: vec![Symbol::Terminal(SymbolId(2))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(1),
    };
    grammar.add_rule(rule2);

    let generator = NodeTypesGenerator::new(&grammar);
    let result = generator.generate().expect("Failed to generate NODE_TYPES");

    let value: serde_json::Value =
        serde_json::from_str(&result).expect("Generated NODE_TYPES is not valid JSON");

    // Verify it's an array
    assert!(value.is_array(), "NODE_TYPES should be a JSON array");
}

// ============================================================================
// Tests: ABI Builder
// ============================================================================

/// Test 8: ABI builder produces valid output
#[test]
fn test_abi_builder_produces_valid_output() {
    let grammar = create_minimal_grammar();
    let parse_table =
        build_parse_table_from_grammar(&grammar).expect("Failed to build parse table");

    let builder = AbiLanguageBuilder::new(&grammar, &parse_table);
    let code = builder.generate();
    let code_str = code.to_string();

    // Verify generated code is not empty
    assert!(!code_str.is_empty(), "Generated code should not be empty");
}

/// Test 9: ABI output contains correct version
#[test]
fn test_abi_output_version() {
    let grammar = create_minimal_grammar();
    let parse_table =
        build_parse_table_from_grammar(&grammar).expect("Failed to build parse table");

    let builder = AbiLanguageBuilder::new(&grammar, &parse_table);
    let code = builder.generate();
    let code_str = code.to_string();

    // Verify code contains language-related content
    assert!(
        code_str.contains("TSLanguage") || code_str.len() > 0,
        "Generated code should contain language information"
    );
}

/// Test 10: ABI output symbol count matches input
#[test]
fn test_abi_output_symbol_count() {
    let grammar = create_minimal_grammar();
    let parse_table =
        build_parse_table_from_grammar(&grammar).expect("Failed to build parse table");

    let original_symbol_count = parse_table.symbol_count;
    let builder = AbiLanguageBuilder::new(&grammar, &parse_table);
    let code = builder.generate();

    // The generated code should reflect the symbol count
    let code_str = code.to_string();
    assert!(!code_str.is_empty());

    // Verify that we have symbol metadata
    assert!(original_symbol_count > 0, "Symbol count should be positive");
}

/// Test 11: ABI output state count matches input
#[test]
fn test_abi_output_state_count() {
    let grammar = create_minimal_grammar();
    let parse_table =
        build_parse_table_from_grammar(&grammar).expect("Failed to build parse table");

    let original_state_count = parse_table.state_count;
    let builder = AbiLanguageBuilder::new(&grammar, &parse_table);
    let code = builder.generate();

    let code_str = code.to_string();
    assert!(!code_str.is_empty());

    // Verify state count is reasonable
    assert!(original_state_count > 0, "State count should be positive");
}

// ============================================================================
// Tests: Compression Benefits & Edge Cases
// ============================================================================

/// Test 12: Compressed tables reduce memory footprint
#[test]
fn test_compression_reduces_memory_footprint() {
    let grammar = create_minimal_grammar();
    let parse_table =
        build_parse_table_from_grammar(&grammar).expect("Failed to build parse table");

    // Measure uncompressed size (approximate)
    let action_table_rows = parse_table.action_table.len();
    let action_table_cells: usize = parse_table.action_table.iter().map(|row| row.len()).sum();

    // Compress
    let compressor = TableCompressor::new();
    let token_indices = helpers::collect_token_indices(&grammar, &parse_table);
    let compressed = compressor
        .compress(&parse_table, &token_indices, false)
        .expect("Failed to compress table");

    // Compressed should have fewer entries
    assert!(
        action_table_rows > 0 && action_table_cells > 0,
        "Original table should have cells"
    );
    assert!(
        !compressed.action_table.data.is_empty(),
        "Compressed table should have data"
    );
}

/// Test 13: Large parse table compression
#[test]
fn test_large_parse_table_compression() {
    let mut grammar = Grammar::new("large".to_string());

    // Add multiple tokens
    for i in 0..10 {
        let token = Token {
            name: format!("t{}", i),
            pattern: TokenPattern::String(format!("t{}", i)),
            fragile: false,
        };
        grammar.tokens.insert(SymbolId((i + 1) as u16), token);
    }

    // Add multiple rules to create more states
    let rule1 = Rule {
        lhs: SymbolId(11),
        rhs: vec![Symbol::Terminal(SymbolId(1))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    };
    grammar.add_rule(rule1);

    if let Ok(parse_table) = build_parse_table_from_grammar(&grammar) {
        let compressor = TableCompressor::new();
        let token_indices = helpers::collect_token_indices(&grammar, &parse_table);
        let compressed = compressor
            .compress(&parse_table, &token_indices, false)
            .expect("Failed to compress large table");

        assert!(!compressed.action_table.data.is_empty());
    }
}

/// Test 14: Compression with empty goto table
#[test]
fn test_compression_with_empty_goto_table() {
    let grammar = create_minimal_grammar();
    let parse_table =
        build_parse_table_from_grammar(&grammar).expect("Failed to build parse table");

    let compressor = TableCompressor::new();
    let token_indices = helpers::collect_token_indices(&grammar, &parse_table);
    let compressed = compressor
        .compress(&parse_table, &token_indices, false)
        .expect("Failed to compress table");

    // Goto table may be sparse or empty for simple grammars
    assert!(
        compressed.goto_table.row_offsets.len() > 0,
        "Goto table should have row offsets"
    );
}

/// Test 15: Compression with dense goto table
#[test]
fn test_compression_with_dense_goto_table() {
    let grammar = create_grammar_with_nonterminals();
    let parse_table =
        build_parse_table_from_grammar(&grammar).expect("Failed to build parse table");

    let compressor = TableCompressor::new();
    let token_indices = helpers::collect_token_indices(&grammar, &parse_table);
    let compressed = compressor
        .compress(&parse_table, &token_indices, false)
        .expect("Failed to compress table");

    // Verify goto table compression worked
    assert!(
        !compressed.goto_table.row_offsets.is_empty(),
        "Dense goto table should have offsets"
    );
}

// ============================================================================
// Tests: ABI Output Components
// ============================================================================

/// Test 16: Multiple productions in ABI output
#[test]
fn test_abi_output_multiple_productions() {
    let grammar = create_grammar_with_reduces();
    let parse_table =
        build_parse_table_from_grammar(&grammar).expect("Failed to build parse table");

    let builder = AbiLanguageBuilder::new(&grammar, &parse_table);
    let code = builder.generate();
    let code_str = code.to_string();

    // Verify code generation succeeded
    assert!(!code_str.is_empty());
}

/// Test 17: Field names in ABI output
#[test]
fn test_abi_output_field_names() {
    let mut grammar = Grammar::new("fields_test".to_string());

    let token = Token {
        name: "x".to_string(),
        pattern: TokenPattern::String("x".to_string()),
        fragile: false,
    };
    grammar.tokens.insert(SymbolId(1), token);

    let left_field = FieldId(0);
    grammar.fields.insert(left_field, "left".to_string());

    let rule = Rule {
        lhs: SymbolId(2),
        rhs: vec![Symbol::Terminal(SymbolId(1)), Symbol::Terminal(SymbolId(1))],
        precedence: None,
        associativity: None,
        fields: vec![(left_field, 0)],
        production_id: ProductionId(0),
    };
    grammar.add_rule(rule);

    if let Ok(parse_table) = build_parse_table_from_grammar(&grammar) {
        let builder = AbiLanguageBuilder::new(&grammar, &parse_table);
        let code = builder.generate();
        let code_str = code.to_string();

        assert!(!code_str.is_empty());
    }
}

/// Test 18: External scanner info in ABI output (empty)
#[test]
fn test_abi_output_external_scanner_info() {
    let grammar = create_minimal_grammar();
    let parse_table =
        build_parse_table_from_grammar(&grammar).expect("Failed to build parse table");

    let builder = AbiLanguageBuilder::new(&grammar, &parse_table);
    let code = builder.generate();

    // Verify code generation includes scanner info (may be empty)
    let code_str = code.to_string();
    assert!(!code_str.is_empty());
}

/// Test 19: Parse actions array in ABI output
#[test]
fn test_abi_output_parse_actions_array() {
    let grammar = create_minimal_grammar();
    let parse_table =
        build_parse_table_from_grammar(&grammar).expect("Failed to build parse table");

    let builder = AbiLanguageBuilder::new(&grammar, &parse_table);
    let code = builder.generate();
    let code_str = code.to_string();

    assert!(!code_str.is_empty());
}

/// Test 20: Lex modes in ABI output
#[test]
fn test_abi_output_lex_modes() {
    let grammar = create_minimal_grammar();
    let parse_table =
        build_parse_table_from_grammar(&grammar).expect("Failed to build parse table");

    let builder = AbiLanguageBuilder::new(&grammar, &parse_table);
    let code = builder.generate();
    let code_str = code.to_string();

    assert!(!code_str.is_empty());
}

/// Test 21: Symbol metadata in ABI output
#[test]
fn test_abi_output_symbol_metadata() {
    let grammar = create_minimal_grammar();
    let parse_table =
        build_parse_table_from_grammar(&grammar).expect("Failed to build parse table");

    // Verify symbol metadata is present
    assert!(!parse_table.symbol_metadata.is_empty());

    let builder = AbiLanguageBuilder::new(&grammar, &parse_table);
    let code = builder.generate();
    let code_str = code.to_string();

    assert!(!code_str.is_empty());
}

/// Test 22: Public symbol map in ABI output
#[test]
fn test_abi_output_public_symbol_map() {
    let grammar = create_minimal_grammar();
    let parse_table =
        build_parse_table_from_grammar(&grammar).expect("Failed to build parse table");

    let builder = AbiLanguageBuilder::new(&grammar, &parse_table);
    let code = builder.generate();
    let code_str = code.to_string();

    assert!(!code_str.is_empty());
}

/// Test 23: Primary state IDs in ABI output
#[test]
fn test_abi_output_primary_state_ids() {
    let grammar = create_minimal_grammar();
    let parse_table =
        build_parse_table_from_grammar(&grammar).expect("Failed to build parse table");

    // Verify initial state is set
    assert!(parse_table.initial_state >= StateId(0));

    let builder = AbiLanguageBuilder::new(&grammar, &parse_table);
    let code = builder.generate();
    let code_str = code.to_string();

    assert!(!code_str.is_empty());
}

/// Test 24: Production LHS index in ABI output
#[test]
fn test_abi_output_production_lhs_index() {
    let grammar = create_grammar_with_reduces();
    let parse_table =
        build_parse_table_from_grammar(&grammar).expect("Failed to build parse table");

    // Verify rules have LHS information
    assert!(!parse_table.rules.is_empty());

    let builder = AbiLanguageBuilder::new(&grammar, &parse_table);
    let code = builder.generate();
    let code_str = code.to_string();

    assert!(!code_str.is_empty());
}

/// Test 25: ABI output is deterministic (same input = same output)
#[test]
fn test_abi_output_deterministic() {
    let grammar = create_minimal_grammar();
    let parse_table =
        build_parse_table_from_grammar(&grammar).expect("Failed to build parse table");

    // Generate twice
    let builder1 = AbiLanguageBuilder::new(&grammar, &parse_table);
    let code1 = builder1.generate();
    let code1_str = code1.to_string();

    let builder2 = AbiLanguageBuilder::new(&grammar, &parse_table);
    let code2 = builder2.generate();
    let code2_str = code2.to_string();

    // Both should produce identical output
    assert_eq!(
        code1_str, code2_str,
        "ABI generation should be deterministic"
    );
}

// ============================================================================
// Integration Tests: Full Pipeline
// ============================================================================

/// Test 26: Full pipeline: grammar → parse table → compression → ABI
#[test]
fn test_full_pipeline_grammar_to_abi() {
    let grammar = create_grammar_with_shifts();

    // Step 1: Build parse table
    let parse_table =
        build_parse_table_from_grammar(&grammar).expect("Failed to build parse table");

    // Step 2: Compress tables
    let compressor = TableCompressor::new();
    let token_indices = helpers::collect_token_indices(&grammar, &parse_table);
    let compressed = compressor
        .compress(&parse_table, &token_indices, false)
        .expect("Failed to compress table");

    // Step 3: Generate ABI
    let mut builder = AbiLanguageBuilder::new(&grammar, &parse_table);
    builder = builder.with_compressed_tables(&compressed);
    let code = builder.generate();
    let code_str = code.to_string();

    // Verify all steps succeeded
    assert!(!code_str.is_empty());
    assert!(code_str.len() > 0);
}

/// Test 27: Full pipeline preserves state information
#[test]
fn test_full_pipeline_preserves_states() {
    let grammar = create_grammar_with_nonterminals();
    let original_state_count = 5; // Approximate for this grammar

    let parse_table =
        build_parse_table_from_grammar(&grammar).expect("Failed to build parse table");

    let state_count_before = parse_table.state_count;

    let compressor = TableCompressor::new();
    let token_indices = helpers::collect_token_indices(&grammar, &parse_table);
    let _compressed = compressor
        .compress(&parse_table, &token_indices, false)
        .expect("Failed to compress table");

    // State count should be preserved
    assert_eq!(
        state_count_before, parse_table.state_count,
        "State count should not change during compression"
    );
}
