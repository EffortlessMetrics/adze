// Tests for compressed table generation

use rust_sitter_glr_core::{FirstFollowSets, build_lr1_automaton};
use rust_sitter_ir::*;
use rust_sitter_tablegen::{TableCompressor, abi_builder::AbiLanguageBuilder, helpers};

#[test]
fn test_compressed_table_generation() {
    // Create a simple grammar: S -> 'a' | 'b'
    let mut grammar = Grammar::new("simple".to_string());

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

    // Add rules
    let s_id = SymbolId(3);

    let rule1 = Rule {
        lhs: s_id,
        rhs: vec![Symbol::Terminal(SymbolId(1))], // S -> 'a'
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    };

    let rule2 = Rule {
        lhs: s_id,
        rhs: vec![Symbol::Terminal(SymbolId(2))], // S -> 'b'
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(1),
    };

    grammar.add_rule(rule1);
    grammar.add_rule(rule2);

    // Build parse table
    let first_follow = FirstFollowSets::compute(&grammar).unwrap();
    let parse_table = build_lr1_automaton(&grammar, &first_follow).unwrap();

    // Compress the tables
    let compressor = TableCompressor::new();
    // Collect token indices properly from grammar and parse table
    let token_indices = helpers::collect_token_indices(&grammar, &parse_table);
    let compressed = compressor
        .compress(&parse_table, &token_indices, false)
        .unwrap();

    // Verify compression
    assert!(!compressed.action_table.data.is_empty());
    assert!(!compressed.goto_table.data.is_empty());

    // Generate ABI-compatible language
    let mut builder = AbiLanguageBuilder::new(&grammar, &parse_table);
    builder = builder.with_compressed_tables(&compressed);

    let code = builder.generate();
    let code_str = code.to_string();

    // Verify generated code
    assert!(code_str.contains("TSLanguage"));
    assert!(code_str.contains("PARSE_TABLE"));
    assert!(code_str.contains("tree_sitter_simple"));
}

#[test]
fn test_table_compression_algorithms() {
    use rust_sitter_glr_core::Action;
    use rust_sitter_ir::{RuleId, StateId};
    use rust_sitter_tablegen::compress::*;

    let compressor = TableCompressor::new();

    // Test action encoding
    let shift = Action::Shift(StateId(42));
    let encoded_shift = compressor.encode_action_small(&shift).unwrap();
    assert_eq!(encoded_shift, 42);

    let reduce = Action::Reduce(RuleId(17));
    let encoded_reduce = compressor.encode_action_small(&reduce).unwrap();
    // Tree-sitter uses 1-based production IDs for reduce actions
    assert_eq!(encoded_reduce, 0x8000 | (17 + 1));

    let accept = Action::Accept;
    let encoded_accept = compressor.encode_action_small(&accept).unwrap();
    assert_eq!(encoded_accept, 0xFFFF);

    let error = Action::Error;
    let encoded_error = compressor.encode_action_small(&error).unwrap();
    assert_eq!(encoded_error, 0xFFFE);
}

#[test]
fn test_goto_table_compression() {
    use rust_sitter_ir::StateId;
    use rust_sitter_tablegen::compress::*;

    let compressor = TableCompressor::new();

    // Test goto table compression with run-length encoding
    let goto_table = vec![
        vec![StateId(0), StateId(0), StateId(0), StateId(1)], // Run of 3 StateId(0)s
        vec![
            StateId(2),
            StateId(2),
            StateId(3),
            StateId(3),
            StateId(3),
            StateId(3),
        ], // Runs of 2 and 4
    ];

    let compressed = compressor.compress_goto_table_small(&goto_table).unwrap();

    // Verify row offsets
    assert_eq!(compressed.row_offsets.len(), 3); // 2 rows + sentinel

    // Verify compression happened (run-length encoding for runs > 2)
    let mut expanded_count = 0;
    for entry in &compressed.data {
        match entry {
            CompressedGotoEntry::Single(_) => expanded_count += 1,
            CompressedGotoEntry::RunLength { count, .. } => expanded_count += *count as usize,
        }
    }
    assert_eq!(expanded_count, 10); // Total original entries
}

#[test]
fn test_deterministic_table_generation() {
    // Create identical grammars and verify they produce identical compressed tables
    let grammar1 = create_test_grammar("test1");
    let grammar2 = create_test_grammar("test2");

    let first_follow1 = FirstFollowSets::compute(&grammar1).unwrap();
    let parse_table1 = build_lr1_automaton(&grammar1, &first_follow1).unwrap();

    let first_follow2 = FirstFollowSets::compute(&grammar2).unwrap();
    let parse_table2 = build_lr1_automaton(&grammar2, &first_follow2).unwrap();

    let compressor = TableCompressor::new();
    // Collect token indices properly from grammar and parse table
    let token_indices1 = helpers::collect_token_indices(&grammar1, &parse_table1);
    let token_indices2 = helpers::collect_token_indices(&grammar2, &parse_table2);
    let compressed1 = compressor
        .compress(&parse_table1, &token_indices1, false)
        .unwrap();
    let compressed2 = compressor
        .compress(&parse_table2, &token_indices2, false)
        .unwrap();

    // Verify deterministic compression
    assert_eq!(
        compressed1.action_table.data.len(),
        compressed2.action_table.data.len()
    );
    assert_eq!(
        compressed1.goto_table.data.len(),
        compressed2.goto_table.data.len()
    );
}

fn create_test_grammar(name: &str) -> Grammar {
    let mut grammar = Grammar::new(name.to_string());

    // Add token
    let token = Token {
        name: "x".to_string(),
        pattern: TokenPattern::String("x".to_string()),
        fragile: false,
    };
    grammar.tokens.insert(SymbolId(1), token);

    // Add rule: S -> 'x'
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
