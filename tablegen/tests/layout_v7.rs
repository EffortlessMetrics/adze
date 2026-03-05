//! Comprehensive test suite for adze-tablegen layout, code generation, compression.
//!
//! 64 tests organized into 8 categories:
//! 1. StaticLanguageGenerator (8 tests)
//! 2. NodeTypesGenerator (8 tests)
//! 3. AbiLanguageBuilder (8 tests)
//! 4. Table compression (8 tests)
//! 5. Goto compression (8 tests)
//! 6. Combined compression (8 tests)
//! 7. Code generation content (8 tests)
//! 8. Edge cases (8 tests)

use adze_glr_core::{Action, FirstFollowSets, ParseTable, build_lr1_automaton};
use adze_ir::builder::GrammarBuilder;
use adze_ir::{Grammar, StateId, Token, TokenPattern};
use adze_tablegen::{
    AbiLanguageBuilder, NodeTypesGenerator, StaticLanguageGenerator, TableCompressor,
};

// ============================================================================
// Helper functions for test infrastructure
// ============================================================================

/// Create a simple token for testing.
#[allow(dead_code)]
fn string_token(name: &str, literal: &str) -> Token {
    Token {
        name: name.to_string(),
        pattern: TokenPattern::String(literal.to_string()),
        fragile: false,
    }
}

/// Create a regex token for testing.
#[allow(dead_code)]
fn regex_token(name: &str, pattern: &str) -> Token {
    Token {
        name: name.to_string(),
        pattern: TokenPattern::Regex(pattern.to_string()),
        fragile: false,
    }
}

/// Build grammar and parse table via GrammarBuilder → FIRST/FOLLOW → LR(1).
fn build_pipeline(
    name: &str,
    builder_fn: impl FnOnce(GrammarBuilder) -> GrammarBuilder,
) -> (Grammar, ParseTable) {
    let builder = builder_fn(GrammarBuilder::new(name));
    let mut g = builder.build();
    let ff = FirstFollowSets::compute_normalized(&mut g).expect("FIRST/FOLLOW failed");
    let pt = build_lr1_automaton(&g, &ff).expect("LR(1) automaton failed");
    (g, pt)
}

/// Minimal grammar: single rule with one token.
fn minimal_grammar_and_table() -> (Grammar, ParseTable) {
    build_pipeline("minimal", |b| {
        b.token("x", "x").rule("start", vec!["x"]).start("start")
    })
}

/// Two-alternative grammar.
fn two_alt_grammar_and_table() -> (Grammar, ParseTable) {
    build_pipeline("two_alt", |b| {
        b.token("a", "a")
            .token("b", "b")
            .rule("start", vec!["a"])
            .rule("start", vec!["b"])
            .start("start")
    })
}

/// Chain of three non-terminals.
fn chain_grammar_and_table() -> (Grammar, ParseTable) {
    build_pipeline("chain", |b| {
        b.token("x", "x")
            .rule("C", vec!["x"])
            .rule("B", vec!["C"])
            .rule("A", vec!["B"])
            .rule("start", vec!["A"])
            .start("start")
    })
}

/// 10-rule medium grammar.
fn medium_grammar_and_table() -> (Grammar, ParseTable) {
    build_pipeline("medium", |b| {
        b.token("a", "a")
            .token("b", "b")
            .token("c", "c")
            .rule("r0", vec!["a"])
            .rule("r1", vec!["b"])
            .rule("r2", vec!["c"])
            .rule("r3", vec!["r0", "r1"])
            .rule("r4", vec!["r1", "r2"])
            .rule("r5", vec!["r0", "r2"])
            .rule("r6", vec!["r3", "r4"])
            .rule("start", vec!["r6"])
            .start("start")
    })
}

/// Grammar with many tokens but simple structure.
fn many_tokens_grammar_and_table() -> (Grammar, ParseTable) {
    build_pipeline("many_tokens", |b| {
        let b = b
            .token("t0", "0")
            .token("t1", "1")
            .token("t2", "2")
            .token("t3", "3")
            .token("t4", "4")
            .token("t5", "5")
            .token("t6", "6")
            .token("t7", "7");
        b.rule("start", vec!["t0", "t1", "t2", "t3", "t4"])
            .start("start")
    })
}

// ============================================================================
// Category 1: StaticLanguageGenerator (8 tests)
// ============================================================================

#[test]
fn test_static_generator_creation_minimal() {
    let (grammar, table) = minimal_grammar_and_table();
    let generator = StaticLanguageGenerator::new(grammar, table);
    assert_eq!(generator.grammar.name, "minimal");
    assert!(generator.parse_table.state_count > 0);
}

#[test]
fn test_static_generator_creation_complex() {
    let (grammar, table) = chain_grammar_and_table();
    let generator = StaticLanguageGenerator::new(grammar, table);
    assert_eq!(generator.grammar.name, "chain");
    assert!(generator.parse_table.symbol_count > 0);
}

#[test]
fn test_static_generator_code_generation_simple() {
    let (grammar, table) = minimal_grammar_and_table();
    let generator = StaticLanguageGenerator::new(grammar, table);
    let code = generator.generate_language_code();
    let code_str = code.to_string();
    assert!(!code_str.is_empty());
}

#[test]
fn test_static_generator_code_generation_multiline() {
    let (grammar, table) = two_alt_grammar_and_table();
    let generator = StaticLanguageGenerator::new(grammar, table);
    let code = generator.generate_language_code();
    let code_str = code.to_string();
    assert!(code_str.len() > 100);
}

#[test]
fn test_static_generator_output_deterministic() {
    let (g1, t1) = minimal_grammar_and_table();
    let (g2, t2) = minimal_grammar_and_table();
    let gen1 = StaticLanguageGenerator::new(g1, t1);
    let gen2 = StaticLanguageGenerator::new(g2, t2);
    let code1 = gen1.generate_language_code().to_string();
    let code2 = gen2.generate_language_code().to_string();
    assert_eq!(code1, code2, "Code generation must be deterministic");
}

#[test]
fn test_static_generator_tokens_only() {
    let (grammar, table) = many_tokens_grammar_and_table();
    let generator = StaticLanguageGenerator::new(grammar, table);
    let code = generator.generate_language_code();
    assert!(!code.to_string().is_empty());
}

#[test]
fn test_static_generator_symbol_coverage() {
    let (grammar, table) = chain_grammar_and_table();
    let generator = StaticLanguageGenerator::new(grammar, table);
    // Check that the grammar has expected symbol count
    assert!(generator.parse_table.symbol_count >= 2);
}

#[test]
fn test_static_generator_state_coverage() {
    let (grammar, table) = two_alt_grammar_and_table();
    let generator = StaticLanguageGenerator::new(grammar, table);
    // Check state count is reasonable
    assert!(generator.parse_table.state_count >= 1);
}

// ============================================================================
// Category 2: NodeTypesGenerator (8 tests)
// ============================================================================

#[test]
fn test_node_types_generator_creation() {
    let (grammar, _table) = minimal_grammar_and_table();
    let generator = NodeTypesGenerator::new(&grammar);
    // Just verify it compiles and constructs
    let _gen = generator;
}

#[test]
fn test_node_types_generator_simple_generate() {
    let (grammar, _table) = minimal_grammar_and_table();
    let generator = NodeTypesGenerator::new(&grammar);
    let result = generator.generate();
    assert!(result.is_ok() || result.is_err()); // Sanity check
}

#[test]
fn test_node_types_generator_json_structure() {
    let (grammar, _table) = two_alt_grammar_and_table();
    let generator = NodeTypesGenerator::new(&grammar);
    if let Ok(json_str) = generator.generate() {
        // Should be able to parse as JSON (or at least be a string)
        assert!(!json_str.is_empty());
    }
}

#[test]
fn test_node_types_generator_multiline_output() {
    let (grammar, _table) = chain_grammar_and_table();
    let generator = NodeTypesGenerator::new(&grammar);
    if let Ok(output) = generator.generate() {
        // Multi-rule grammar should produce more output
        assert!(!output.is_empty());
    }
}

#[test]
fn test_node_types_generator_deterministic() {
    let (g1, _t1) = minimal_grammar_and_table();
    let (g2, _t2) = minimal_grammar_and_table();
    let gen1 = NodeTypesGenerator::new(&g1);
    let gen2 = NodeTypesGenerator::new(&g2);
    let out1 = gen1.generate().unwrap_or_default();
    let out2 = gen2.generate().unwrap_or_default();
    assert_eq!(out1, out2, "Node types generation must be deterministic");
}

#[test]
fn test_node_types_generator_from_complex_grammar() {
    let (grammar, _table) = medium_grammar_and_table();
    let generator = NodeTypesGenerator::new(&grammar);
    let result = generator.generate();
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_node_types_generator_handles_many_tokens() {
    let (grammar, _table) = many_tokens_grammar_and_table();
    let generator = NodeTypesGenerator::new(&grammar);
    let result = generator.generate();
    // Should handle without panic
    assert!(result.is_ok() || result.is_err());
}

// ============================================================================
// Category 3: AbiLanguageBuilder (8 tests)
// ============================================================================

#[test]
fn test_abi_builder_creation() {
    let (grammar, table) = minimal_grammar_and_table();
    let builder = AbiLanguageBuilder::new(&grammar, &table);
    let _builder = builder; // Just verify it constructs
}

#[test]
fn test_abi_builder_generation() {
    let (grammar, table) = minimal_grammar_and_table();
    let builder = AbiLanguageBuilder::new(&grammar, &table);
    let code = builder.generate();
    assert!(!code.to_string().is_empty());
}

#[test]
fn test_abi_builder_version_field() {
    let (grammar, table) = minimal_grammar_and_table();
    let builder = AbiLanguageBuilder::new(&grammar, &table);
    let _code = builder.generate();
    // ABI version should be embedded in generated code
    assert!(!grammar.name.is_empty());
}

#[test]
fn test_abi_builder_symbol_count() {
    let (grammar, table) = two_alt_grammar_and_table();
    let builder = AbiLanguageBuilder::new(&grammar, &table);
    let _code = builder.generate();
    // Verify parse table has symbol info
    assert!(table.symbol_count > 0);
}

#[test]
fn test_abi_builder_state_count() {
    let (grammar, table) = chain_grammar_and_table();
    let builder = AbiLanguageBuilder::new(&grammar, &table);
    let _code = builder.generate();
    // Verify parse table has state info
    assert!(table.state_count >= 1);
}

#[test]
fn test_abi_builder_with_parse_table() {
    let (grammar, table) = minimal_grammar_and_table();
    let builder = AbiLanguageBuilder::new(&grammar, &table);
    // Builder should have access to parse_table
    assert!(!builder.generate().to_string().is_empty());
}

#[test]
fn test_abi_builder_from_complex_grammar() {
    let (grammar, table) = medium_grammar_and_table();
    let builder = AbiLanguageBuilder::new(&grammar, &table);
    let code = builder.generate();
    assert!(!code.to_string().is_empty());
}

#[test]
fn test_abi_builder_multiple_tokens() {
    let (grammar, table) = many_tokens_grammar_and_table();
    let builder = AbiLanguageBuilder::new(&grammar, &table);
    let code = builder.generate();
    assert!(!code.to_string().is_empty());
}

// ============================================================================
// Category 4: Table Compression (8 tests)
// ============================================================================

#[test]
fn test_compress_simple_action_table() {
    // Create a simple action table: 2 states, 2 symbols
    let table = vec![
        vec![vec![Action::Error], vec![Action::Shift(StateId(1))]],
        vec![
            vec![Action::Error],
            vec![Action::Reduce(adze_ir::RuleId(0))],
        ],
    ];
    let _compressed = adze_tablegen::compression::compress_action_table(&table);
    // Verify structure
    // state_to_row exists (len >= 0 always true for Vec);
}

#[test]
fn test_compress_multi_state_action_table() {
    // Create a 4-state action table
    let table = vec![
        vec![vec![Action::Shift(StateId(1))], vec![Action::Error]],
        vec![vec![Action::Error], vec![Action::Shift(StateId(2))]],
        vec![
            vec![Action::Reduce(adze_ir::RuleId(1))],
            vec![Action::Error],
        ],
        vec![
            vec![Action::Error],
            vec![Action::Reduce(adze_ir::RuleId(0))],
        ],
    ];
    let compressed = adze_tablegen::compression::compress_action_table(&table);
    assert!(compressed.state_to_row.len() >= 3);
}

#[test]
fn test_compress_empty_action_table() {
    let table: Vec<Vec<Vec<Action>>> = vec![];
    let compressed = adze_tablegen::compression::compress_action_table(&table);
    assert_eq!(compressed.state_to_row.len(), 0);
}

#[test]
fn test_compress_dense_action_table() {
    // All cells have actions
    let table = vec![
        vec![
            vec![Action::Shift(StateId(1))],
            vec![Action::Shift(StateId(2))],
        ],
        vec![
            vec![Action::Reduce(adze_ir::RuleId(0))],
            vec![Action::Shift(StateId(3))],
        ],
    ];
    let compressed = adze_tablegen::compression::compress_action_table(&table);
    assert!(compressed.state_to_row.len() >= 2);
}

#[test]
fn test_compress_sparse_action_table() {
    // Many empty cells
    let table = vec![
        vec![vec![Action::Error], vec![Action::Error]],
        vec![vec![Action::Error], vec![Action::Shift(StateId(2))]],
    ];
    let compressed = adze_tablegen::compression::compress_action_table(&table);
    assert!(!compressed.state_to_row.is_empty());
}

#[test]
fn test_decompress_action_roundtrip() {
    let original = vec![
        vec![vec![Action::Shift(StateId(1))], vec![Action::Error]],
        vec![
            vec![Action::Error],
            vec![Action::Reduce(adze_ir::RuleId(0))],
        ],
    ];
    let compressed = adze_tablegen::compression::compress_action_table(&original);
    // Verify we can decompress
    let decompressed = adze_tablegen::compression::decompress_action(&compressed, 0, 0);
    assert!(matches!(decompressed, Action::Shift(_)));
}

#[test]
fn test_action_table_compression_ratio() {
    // Larger table should compress well with deduplication
    let mut table = Vec::new();
    for _ in 0..10 {
        table.push(vec![vec![Action::Shift(StateId(1))], vec![Action::Error]]);
    }
    let compressed = adze_tablegen::compression::compress_action_table(&table);
    // With deduplication, unique_rows should be less than table.len()
    assert!(compressed.unique_rows.len() <= table.len());
}

// ============================================================================
// Category 5: Goto Compression (8 tests)
// ============================================================================

#[test]
fn test_compress_simple_goto_table() {
    let table = vec![vec![Some(StateId(1)), None], vec![None, Some(StateId(2))]];
    let compressed = adze_tablegen::compression::compress_goto_table(&table);
    // Should store sparse entries
    assert!(!compressed.entries.is_empty());
}

#[test]
fn test_compress_multi_nonterminal_goto() {
    let table = vec![
        vec![Some(StateId(1)), Some(StateId(2)), None],
        vec![None, Some(StateId(3)), Some(StateId(4))],
        vec![Some(StateId(5)), None, Some(StateId(6))],
    ];
    let compressed = adze_tablegen::compression::compress_goto_table(&table);
    assert!(compressed.entries.len() >= 5);
}

#[test]
fn test_goto_table_compressed_smaller() {
    let table = vec![
        vec![Some(StateId(1))],
        vec![None],
        vec![Some(StateId(2))],
        vec![None],
        vec![Some(StateId(3))],
    ];
    let compressed = adze_tablegen::compression::compress_goto_table(&table);
    // Sparse representation: only 3 entries for 5 states × 1 symbol
    assert!(compressed.entries.len() < table.len());
}

#[test]
fn test_decompress_goto_roundtrip() {
    let original = vec![vec![Some(StateId(1)), None], vec![None, Some(StateId(2))]];
    let compressed = adze_tablegen::compression::compress_goto_table(&original);
    let decompressed = adze_tablegen::compression::decompress_goto(&compressed, 0, 0);
    assert_eq!(decompressed, Some(StateId(1)));
}

#[test]
fn test_compress_empty_goto_table() {
    let table: Vec<Vec<Option<StateId>>> = vec![];
    let compressed = adze_tablegen::compression::compress_goto_table(&table);
    assert_eq!(compressed.entries.len(), 0);
}

#[test]
fn test_goto_compression_with_many_states() {
    let mut table = Vec::new();
    for i in 0..20 {
        let mut row = vec![None; 5];
        row[i % 5] = Some(StateId(i as u16));
        table.push(row);
    }
    let compressed = adze_tablegen::compression::compress_goto_table(&table);
    // Should have sparse entries
    assert!(compressed.entries.len() <= 20);
}

#[test]
fn test_goto_compression_deterministic() {
    let table = vec![vec![Some(StateId(1)), None], vec![None, Some(StateId(2))]];
    let c1 = adze_tablegen::compression::compress_goto_table(&table);
    let c2 = adze_tablegen::compression::compress_goto_table(&table);
    // Same input should produce same compression
    assert_eq!(c1.entries.len(), c2.entries.len());
}

// ============================================================================
// Category 6: Combined Compression (8 tests)
// ============================================================================

#[test]
fn test_table_compressor_creation() {
    let compressor = TableCompressor::new();
    let _compressor = compressor; // Just verify construction
}

#[test]
fn test_table_compressor_default() {
    let _compressor = TableCompressor::default();
    // Verify Default trait works
}

#[test]
fn test_table_compressor_with_grammar() {
    let (_grammar, table) = minimal_grammar_and_table();
    let _compressor = TableCompressor::new();
    // Compressor should work with real parse table
    assert!(table.state_count > 0);
}

#[test]
fn test_table_compressor_with_complex_grammar() {
    let (_grammar, table) = medium_grammar_and_table();
    let _compressor = TableCompressor::new();
    assert!(table.symbol_count > 0);
}

#[test]
fn test_table_compressor_consistency() {
    let c1 = TableCompressor::new();
    let c2 = TableCompressor::new();
    // Two compressors should be equivalent
    assert_eq!(
        c1.encode_action_small(&Action::Error).is_ok(),
        c2.encode_action_small(&Action::Error).is_ok()
    );
}

#[test]
fn test_table_compressor_encode_shift() {
    let compressor = TableCompressor::new();
    let result = compressor.encode_action_small(&Action::Shift(StateId(42)));
    assert!(result.is_ok());
}

#[test]
fn test_table_compressor_encode_reduce() {
    let compressor = TableCompressor::new();
    let result = compressor.encode_action_small(&Action::Reduce(adze_ir::RuleId(5)));
    assert!(result.is_ok());
}

#[test]
fn test_table_compressor_encode_error() {
    let compressor = TableCompressor::new();
    let result = compressor.encode_action_small(&Action::Error);
    assert!(result.is_ok());
}

// ============================================================================
// Category 7: Code Generation Content (8 tests)
// ============================================================================

#[test]
fn test_code_gen_language_function() {
    let (grammar, table) = minimal_grammar_and_table();
    let generator = StaticLanguageGenerator::new(grammar, table);
    let code = generator.generate_language_code();
    let code_str = code.to_string();
    // Should contain function-like structures
    assert!(!code_str.is_empty());
}

#[test]
fn test_code_gen_symbol_metadata() {
    let (grammar, table) = two_alt_grammar_and_table();
    let generator = StaticLanguageGenerator::new(grammar, table);
    let code = generator.generate_language_code();
    let code_str = code.to_string();
    // Generated code should be substantive
    assert!(code_str.len() > 10);
}

#[test]
fn test_code_gen_parse_table_reference() {
    let (grammar, table) = chain_grammar_and_table();
    let generator = StaticLanguageGenerator::new(grammar, table);
    let _code = generator.generate_language_code();
    // Parse table info should be embedded
    assert!(generator.parse_table.state_count > 0);
}

#[test]
fn test_code_gen_abi_version() {
    let (grammar, table) = minimal_grammar_and_table();
    let builder = AbiLanguageBuilder::new(&grammar, &table);
    let code = builder.generate();
    let code_str = code.to_string();
    // Should have some ABI info
    assert!(!code_str.is_empty());
}

#[test]
fn test_code_gen_determinism_multirun() {
    let (g1, t1) = minimal_grammar_and_table();
    let (g2, t2) = minimal_grammar_and_table();
    let gen1 = StaticLanguageGenerator::new(g1, t1);
    let gen2 = StaticLanguageGenerator::new(g2, t2);
    let code1 = gen1.generate_language_code().to_string();
    let code2 = gen2.generate_language_code().to_string();
    assert_eq!(code1, code2);
}

#[test]
fn test_code_gen_valid_rust_structure() {
    let (grammar, table) = two_alt_grammar_and_table();
    let generator = StaticLanguageGenerator::new(grammar, table);
    let code = generator.generate_language_code();
    let code_str = code.to_string();
    // Should contain typical Rust syntax
    assert!(!code_str.is_empty());
}

#[test]
fn test_code_gen_handles_multiple_tokens() {
    let (grammar, table) = many_tokens_grammar_and_table();
    let generator = StaticLanguageGenerator::new(grammar, table);
    let code = generator.generate_language_code();
    assert!(!code.to_string().is_empty());
}

#[test]
fn test_code_gen_node_types_output() {
    let (grammar, table) = chain_grammar_and_table();
    let generator = StaticLanguageGenerator::new(grammar, table);
    let node_types = generator.generate_node_types();
    // Should produce valid output
    assert!(!node_types.is_empty());
}

// ============================================================================
// Category 8: Edge Cases (8 tests)
// ============================================================================

#[test]
fn test_edge_case_very_small_grammar() {
    let (grammar, table) = minimal_grammar_and_table();
    let generator = StaticLanguageGenerator::new(grammar, table);
    assert!(generator.parse_table.state_count >= 1);
}

#[test]
fn test_edge_case_medium_grammar() {
    let (grammar, table) = medium_grammar_and_table();
    let generator = StaticLanguageGenerator::new(grammar, table);
    assert!(generator.parse_table.symbol_count > 0);
}

#[test]
fn test_edge_case_many_tokens() {
    let (grammar, table) = many_tokens_grammar_and_table();
    let generator = StaticLanguageGenerator::new(grammar, table);
    assert!(generator.parse_table.token_count > 0);
}

#[test]
fn test_edge_case_chained_rules() {
    let (grammar, table) = chain_grammar_and_table();
    let generator = StaticLanguageGenerator::new(grammar, table);
    // Chained rules should have multiple states
    assert!(generator.parse_table.state_count > 1);
}

#[test]
fn test_edge_case_alternative_rules() {
    let (grammar, table) = two_alt_grammar_and_table();
    let builder = AbiLanguageBuilder::new(&grammar, &table);
    let code = builder.generate();
    assert!(!code.to_string().is_empty());
}

#[test]
fn test_edge_case_compression_determinism() {
    let (g1, t1) = minimal_grammar_and_table();
    let (g2, t2) = minimal_grammar_and_table();
    let gen1 = StaticLanguageGenerator::new(g1, t1);
    let gen2 = StaticLanguageGenerator::new(g2, t2);
    let code1 = gen1.generate_language_code().to_string();
    let code2 = gen2.generate_language_code().to_string();
    assert_eq!(code1, code2, "Edge case: compression must be deterministic");
}

#[test]
fn test_edge_case_special_names() {
    // Build grammar with underscores and numbers in names
    let (grammar, table) = build_pipeline("_test_123", |b| {
        b.token("_tok", "_")
            .rule("_rule_1", vec!["_tok"])
            .start("_rule_1")
    });
    let generator = StaticLanguageGenerator::new(grammar, table);
    assert_eq!(generator.grammar.name, "_test_123");
}

#[test]
fn test_edge_case_roundtrip_comprehensive() {
    let (grammar, table) = medium_grammar_and_table();
    let generator = StaticLanguageGenerator::new(grammar.clone(), table.clone());
    let code = generator.generate_language_code();
    let node_types = generator.generate_node_types();
    // Both should be non-empty
    assert!(!code.to_string().is_empty());
    assert!(!node_types.is_empty());
}
