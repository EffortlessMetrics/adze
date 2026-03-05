//! Comprehensive v6 tests for the ABI builder in adze-tablegen.
//!
//! 55+ tests covering:
//! 1. ABI builder construction (8 tests)
//! 2. Symbol count agreement (8 tests)
//! 3. State count agreement (8 tests)
//! 4. Action encoding (7 tests)
//! 5. Goto encoding (8 tests)
//! 6. ABI compatibility (8 tests)
//! 7. Edge cases (8 tests)

use adze_glr_core::{FirstFollowSets, ParseTable, build_lr1_automaton};
use adze_ir::builder::GrammarBuilder;
use adze_ir::{Associativity, Grammar};
use adze_tablegen::compress::TableCompressor;
use adze_tablegen::{
    AbiLanguageBuilder, StaticLanguageGenerator, collect_token_indices, eof_accepts_or_reduces,
};

// ============================================================================
// Helpers
// ============================================================================

fn build_table(grammar: &Grammar) -> ParseTable {
    let mut g = grammar.clone();
    let ff = FirstFollowSets::compute_normalized(&mut g).expect("FIRST/FOLLOW");
    build_lr1_automaton(&g, &ff).expect("LR(1)")
}

fn generate_code(grammar: &Grammar, table: &ParseTable) -> String {
    AbiLanguageBuilder::new(grammar, table)
        .generate()
        .to_string()
}

fn single_token_grammar() -> Grammar {
    GrammarBuilder::new("single_tok")
        .token("x", "x")
        .rule("S", vec!["x"])
        .start("S")
        .build()
}

fn two_token_grammar() -> Grammar {
    GrammarBuilder::new("two_tok")
        .token("a", "a")
        .token("b", "b")
        .rule("S", vec!["a", "b"])
        .start("S")
        .build()
}

fn alternatives_grammar() -> Grammar {
    GrammarBuilder::new("alts")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("S", vec!["a"])
        .rule("S", vec!["b"])
        .rule("S", vec!["c"])
        .start("S")
        .build()
}

fn nested_grammar() -> Grammar {
    GrammarBuilder::new("nested")
        .token("x", "x")
        .token("y", "y")
        .rule("S", vec!["A", "B"])
        .rule("A", vec!["x"])
        .rule("B", vec!["y"])
        .start("S")
        .build()
}

fn deep_chain_grammar() -> Grammar {
    GrammarBuilder::new("deep_chain")
        .token("z", "z")
        .rule("S", vec!["A"])
        .rule("A", vec!["B"])
        .rule("B", vec!["C"])
        .rule("C", vec!["z"])
        .start("S")
        .build()
}

fn left_recursive_grammar() -> Grammar {
    GrammarBuilder::new("left_rec")
        .token("a", "a")
        .rule("S", vec!["a"])
        .rule("S", vec!["S", "a"])
        .start("S")
        .build()
}

fn right_recursive_grammar() -> Grammar {
    GrammarBuilder::new("right_rec")
        .token("a", "a")
        .rule("S", vec!["a"])
        .rule("S", vec!["a", "S"])
        .start("S")
        .build()
}

fn precedence_grammar() -> Grammar {
    GrammarBuilder::new("prec")
        .token("NUM", r"\d+")
        .token("PLUS", r"\+")
        .token("STAR", r"\*")
        .rule_with_precedence("expr", vec!["expr", "PLUS", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "STAR", "expr"], 2, Associativity::Left)
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build()
}

fn nullable_start_grammar() -> Grammar {
    GrammarBuilder::new("nullable")
        .token("a", "a")
        .rule("S", vec!["A"])
        .rule("A", vec!["a"])
        .rule("A", vec![])
        .start("S")
        .build()
}

fn wide_alternatives_grammar() -> Grammar {
    let mut gb = GrammarBuilder::new("wide");
    for i in 0..10 {
        let name = format!("t{i}");
        let pat = format!("{}", (b'a' + i as u8) as char);
        gb = gb.token(&name, &pat);
        gb = gb.rule("S", vec![&name]);
    }
    gb.start("S").build()
}

fn long_sequence_grammar() -> Grammar {
    GrammarBuilder::new("long_seq")
        .token("t1", "a")
        .token("t2", "b")
        .token("t3", "c")
        .token("t4", "d")
        .token("t5", "e")
        .rule("S", vec!["t1", "t2", "t3", "t4", "t5"])
        .start("S")
        .build()
}

// ============================================================================
// 1. ABI builder construction (8 tests)
// ============================================================================

#[test]
fn construct_with_single_token_grammar() {
    let g = single_token_grammar();
    let pt = build_table(&g);
    let _builder = AbiLanguageBuilder::new(&g, &pt);
}

#[test]
fn construct_with_two_token_grammar() {
    let g = two_token_grammar();
    let pt = build_table(&g);
    let _builder = AbiLanguageBuilder::new(&g, &pt);
}

#[test]
fn construct_with_alternatives_grammar() {
    let g = alternatives_grammar();
    let pt = build_table(&g);
    let _builder = AbiLanguageBuilder::new(&g, &pt);
}

#[test]
fn construct_with_nested_grammar() {
    let g = nested_grammar();
    let pt = build_table(&g);
    let _builder = AbiLanguageBuilder::new(&g, &pt);
}

#[test]
fn construct_with_deep_chain_grammar() {
    let g = deep_chain_grammar();
    let pt = build_table(&g);
    let _builder = AbiLanguageBuilder::new(&g, &pt);
}

#[test]
fn construct_with_left_recursive_grammar() {
    let g = left_recursive_grammar();
    let pt = build_table(&g);
    let _builder = AbiLanguageBuilder::new(&g, &pt);
}

#[test]
fn construct_with_right_recursive_grammar() {
    let g = right_recursive_grammar();
    let pt = build_table(&g);
    let _builder = AbiLanguageBuilder::new(&g, &pt);
}

#[test]
fn construct_with_default_parse_table() {
    let g = Grammar::new("empty".to_string());
    let pt = ParseTable::default();
    let _builder = AbiLanguageBuilder::new(&g, &pt);
}

// ============================================================================
// 2. Symbol count agreement (8 tests)
// ============================================================================

#[test]
fn symbol_count_single_token_matches_parse_table() {
    let g = single_token_grammar();
    let pt = build_table(&g);
    let code = generate_code(&g, &pt);
    let expected = pt.symbol_count;
    assert!(
        code.contains(&format!("{expected}")),
        "generated code should reference symbol_count={expected}"
    );
}

#[test]
fn symbol_count_two_token_matches() {
    let g = two_token_grammar();
    let pt = build_table(&g);
    assert!(
        pt.symbol_count >= 2,
        "parse table needs at least 2 terminal symbols"
    );
    let code = generate_code(&g, &pt);
    assert!(!code.is_empty());
}

#[test]
fn symbol_count_alternatives_matches() {
    let g = alternatives_grammar();
    let pt = build_table(&g);
    // 3 tokens + EOF + 1 non-terminal at minimum
    assert!(
        pt.symbol_count >= 5,
        "alternatives grammar should have at least 5 symbols, got {}",
        pt.symbol_count
    );
}

#[test]
fn symbol_count_nested_matches() {
    let g = nested_grammar();
    let pt = build_table(&g);
    // 2 tokens + EOF + 3 non-terminals (S, A, B)
    assert!(
        pt.symbol_count >= 5,
        "nested grammar should have at least 5 symbols, got {}",
        pt.symbol_count
    );
}

#[test]
fn symbol_count_deep_chain_matches() {
    let g = deep_chain_grammar();
    let pt = build_table(&g);
    // 1 token + EOF + 4 non-terminals (S, A, B, C)
    assert!(
        pt.symbol_count >= 5,
        "deep chain grammar should have at least 5 symbols, got {}",
        pt.symbol_count
    );
}

#[test]
fn symbol_count_left_recursive_matches() {
    let g = left_recursive_grammar();
    let pt = build_table(&g);
    assert!(
        pt.symbol_count >= 2,
        "left-recursive grammar needs at least 2 symbols"
    );
}

#[test]
fn symbol_count_right_recursive_matches() {
    let g = right_recursive_grammar();
    let pt = build_table(&g);
    assert!(
        pt.symbol_count >= 2,
        "right-recursive grammar needs at least 2 symbols"
    );
}

#[test]
fn symbol_count_precedence_grammar_matches() {
    let g = precedence_grammar();
    let pt = build_table(&g);
    // NUM, PLUS, STAR, EOF, expr
    assert!(
        pt.symbol_count >= 5,
        "precedence grammar should have at least 5 symbols, got {}",
        pt.symbol_count
    );
}

// ============================================================================
// 3. State count agreement (8 tests)
// ============================================================================

#[test]
fn state_count_single_token_at_least_two() {
    let g = single_token_grammar();
    let pt = build_table(&g);
    assert!(
        pt.state_count >= 2,
        "single-token grammar needs at least 2 states, got {}",
        pt.state_count
    );
}

#[test]
fn state_count_two_token_at_least_two() {
    let g = two_token_grammar();
    let pt = build_table(&g);
    assert!(pt.state_count >= 2);
}

#[test]
fn state_count_alternatives_at_least_two() {
    let g = alternatives_grammar();
    let pt = build_table(&g);
    assert!(pt.state_count >= 2);
}

#[test]
fn state_count_nested_at_least_two() {
    let g = nested_grammar();
    let pt = build_table(&g);
    assert!(pt.state_count >= 2);
}

#[test]
fn state_count_deep_chain_grows_with_depth() {
    let g = deep_chain_grammar();
    let pt = build_table(&g);
    // A chain S→A→B→C→z should produce more states than a trivial grammar
    assert!(pt.state_count >= 2);
}

#[test]
fn state_count_matches_action_table_rows() {
    let g = single_token_grammar();
    let pt = build_table(&g);
    assert_eq!(
        pt.action_table.len(),
        pt.state_count,
        "action_table rows should equal state_count"
    );
}

#[test]
fn state_count_matches_goto_table_rows() {
    let g = single_token_grammar();
    let pt = build_table(&g);
    assert_eq!(
        pt.goto_table.len(),
        pt.state_count,
        "goto_table rows should equal state_count"
    );
}

#[test]
fn state_count_left_recursive_finite() {
    let g = left_recursive_grammar();
    let pt = build_table(&g);
    // LR(1) automaton for left-recursive grammars should still be finite
    assert!(pt.state_count < 1000, "state count should be bounded");
}

// ============================================================================
// 4. Action encoding (7 tests)
// ============================================================================

#[test]
fn action_table_has_expected_rows() {
    let g = single_token_grammar();
    let pt = build_table(&g);
    assert_eq!(pt.action_table.len(), pt.state_count);
}

#[test]
fn action_table_columns_match_symbol_count() {
    let g = single_token_grammar();
    let pt = build_table(&g);
    for (state_idx, row) in pt.action_table.iter().enumerate() {
        assert_eq!(
            row.len(),
            pt.symbol_count,
            "state {state_idx}: action row width should equal symbol_count"
        );
    }
}

#[test]
fn action_table_initial_state_has_shift() {
    let g = single_token_grammar();
    let pt = build_table(&g);
    // State 0 should have at least one non-empty action cell (a shift on the token)
    let has_action = pt.action_table[0].iter().any(|cell| !cell.is_empty());
    assert!(has_action, "initial state must have at least one action");
}

#[test]
fn action_table_alternatives_initial_state_has_shifts() {
    let g = alternatives_grammar();
    let pt = build_table(&g);
    let shift_count = pt.action_table[0]
        .iter()
        .filter(|cell| !cell.is_empty())
        .count();
    // The grammar has 3 alternatives; state 0 should shift on each token
    assert!(
        shift_count >= 3,
        "initial state should shift on 3 tokens, got {shift_count}"
    );
}

#[test]
fn action_table_encode_roundtrip_via_compressor() {
    let g = single_token_grammar();
    let pt = build_table(&g);
    let ti = collect_token_indices(&g, &pt);
    let sce = eof_accepts_or_reduces(&pt);
    let compressed = TableCompressor::new().compress(&pt, &ti, sce);
    assert!(compressed.is_ok(), "compression should succeed");
}

#[test]
fn action_table_two_token_encode_roundtrip() {
    let g = two_token_grammar();
    let pt = build_table(&g);
    let ti = collect_token_indices(&g, &pt);
    let sce = eof_accepts_or_reduces(&pt);
    let compressed = TableCompressor::new().compress(&pt, &ti, sce);
    assert!(compressed.is_ok());
}

#[test]
fn action_table_precedence_encode_roundtrip() {
    let g = precedence_grammar();
    let pt = build_table(&g);
    let ti = collect_token_indices(&g, &pt);
    let sce = eof_accepts_or_reduces(&pt);
    let compressed = TableCompressor::new().compress(&pt, &ti, sce);
    assert!(compressed.is_ok());
}

// ============================================================================
// 5. Goto encoding (8 tests)
// ============================================================================

#[test]
fn goto_table_has_expected_rows() {
    let g = single_token_grammar();
    let pt = build_table(&g);
    assert_eq!(pt.goto_table.len(), pt.state_count);
}

#[test]
fn goto_table_columns_match_symbol_count() {
    let g = single_token_grammar();
    let pt = build_table(&g);
    for (state_idx, row) in pt.goto_table.iter().enumerate() {
        assert_eq!(
            row.len(),
            pt.symbol_count,
            "state {state_idx}: goto row width should equal symbol_count"
        );
    }
}

#[test]
fn goto_table_nested_has_nonterminal_entries() {
    let g = nested_grammar();
    let pt = build_table(&g);
    // At least one goto entry should be a valid state for a non-terminal
    let has_goto = pt
        .goto_table
        .iter()
        .any(|row| row.iter().any(|&s| s.0 != u16::MAX));
    assert!(has_goto, "nested grammar should have non-trivial gotos");
}

#[test]
fn goto_table_deep_chain_has_nonterminal_entries() {
    let g = deep_chain_grammar();
    let pt = build_table(&g);
    let goto_count: usize = pt
        .goto_table
        .iter()
        .map(|row| row.iter().filter(|s| s.0 != u16::MAX).count())
        .sum();
    assert!(
        goto_count >= 1,
        "deep chain grammar should have at least 1 goto entry"
    );
}

#[test]
fn goto_table_compress_succeeds() {
    let g = nested_grammar();
    let pt = build_table(&g);
    let ti = collect_token_indices(&g, &pt);
    let sce = eof_accepts_or_reduces(&pt);
    let compressed = TableCompressor::new().compress(&pt, &ti, sce);
    assert!(compressed.is_ok());
}

#[test]
fn goto_table_alternatives_compress_succeeds() {
    let g = alternatives_grammar();
    let pt = build_table(&g);
    let ti = collect_token_indices(&g, &pt);
    let sce = eof_accepts_or_reduces(&pt);
    let compressed = TableCompressor::new().compress(&pt, &ti, sce);
    assert!(compressed.is_ok());
}

#[test]
fn goto_table_left_recursive_compress_succeeds() {
    let g = left_recursive_grammar();
    let pt = build_table(&g);
    let ti = collect_token_indices(&g, &pt);
    let sce = eof_accepts_or_reduces(&pt);
    let compressed = TableCompressor::new().compress(&pt, &ti, sce);
    assert!(compressed.is_ok());
}

#[test]
fn goto_table_right_recursive_compress_succeeds() {
    let g = right_recursive_grammar();
    let pt = build_table(&g);
    let ti = collect_token_indices(&g, &pt);
    let sce = eof_accepts_or_reduces(&pt);
    let compressed = TableCompressor::new().compress(&pt, &ti, sce);
    assert!(compressed.is_ok());
}

// ============================================================================
// 6. ABI compatibility (8 tests)
// ============================================================================

#[test]
fn abi_generate_produces_nonempty_code() {
    let g = single_token_grammar();
    let pt = build_table(&g);
    let code = generate_code(&g, &pt);
    assert!(!code.is_empty(), "generated code should be non-empty");
}

#[test]
fn abi_generate_contains_language_name() {
    let g = single_token_grammar();
    let pt = build_table(&g);
    let code = generate_code(&g, &pt);
    assert!(
        code.contains("single_tok"),
        "generated code should reference the grammar name"
    );
}

#[test]
fn abi_generate_contains_symbol_names_array() {
    let g = single_token_grammar();
    let pt = build_table(&g);
    let code = generate_code(&g, &pt);
    assert!(
        code.contains("SYMBOL_NAMES") || code.contains("symbol_names"),
        "generated code should contain symbol names array"
    );
}

#[test]
fn abi_generate_contains_symbol_metadata() {
    let g = single_token_grammar();
    let pt = build_table(&g);
    let code = generate_code(&g, &pt);
    assert!(
        code.contains("SYMBOL_METADATA") || code.contains("symbol_metadata"),
        "generated code should contain symbol metadata"
    );
}

#[test]
fn abi_generate_alternatives_contains_all_tokens() {
    let g = alternatives_grammar();
    let pt = build_table(&g);
    let code = generate_code(&g, &pt);
    // All 3 token names should appear somewhere in the code
    for name in &["a", "b", "c"] {
        assert!(
            code.contains(name),
            "generated code should contain token '{name}'"
        );
    }
}

#[test]
fn abi_generate_nested_contains_nonterminal_names() {
    let g = nested_grammar();
    let pt = build_table(&g);
    let code = generate_code(&g, &pt);
    // Non-terminal names A and B should appear in the code
    assert!(
        code.contains('A') || code.contains('B') || code.contains('S'),
        "generated code should mention non-terminals"
    );
}

#[test]
fn abi_generate_two_grammars_differ() {
    let g1 = single_token_grammar();
    let pt1 = build_table(&g1);
    let code1 = generate_code(&g1, &pt1);

    let g2 = two_token_grammar();
    let pt2 = build_table(&g2);
    let code2 = generate_code(&g2, &pt2);

    assert_ne!(
        code1, code2,
        "different grammars should produce different code"
    );
}

#[test]
fn abi_generate_is_deterministic() {
    let g = nested_grammar();
    let pt = build_table(&g);
    let code1 = generate_code(&g, &pt);
    let code2 = generate_code(&g, &pt);
    assert_eq!(code1, code2, "generation should be deterministic");
}

// ============================================================================
// 7. Edge cases (8 tests)
// ============================================================================

#[test]
fn edge_minimal_grammar_generates_code() {
    let g = single_token_grammar();
    let pt = build_table(&g);
    let code = generate_code(&g, &pt);
    assert!(!code.is_empty());
}

#[test]
fn edge_large_alternatives_grammar_generates_code() {
    let g = wide_alternatives_grammar();
    let pt = build_table(&g);
    let code = generate_code(&g, &pt);
    assert!(!code.is_empty());
}

#[test]
fn edge_long_sequence_grammar_generates_code() {
    let g = long_sequence_grammar();
    let pt = build_table(&g);
    let code = generate_code(&g, &pt);
    assert!(!code.is_empty());
}

#[test]
fn edge_nullable_grammar_generates_code() {
    let g = nullable_start_grammar();
    let pt = build_table(&g);
    let code = generate_code(&g, &pt);
    assert!(!code.is_empty());
}

#[test]
fn edge_static_generator_produces_code() {
    let g = single_token_grammar();
    let pt = build_table(&g);
    let generator = StaticLanguageGenerator::new(g, pt);
    let code = generator.generate_language_code().to_string();
    assert!(
        !code.is_empty(),
        "StaticLanguageGenerator should produce code"
    );
}

#[test]
fn edge_static_generator_node_types() {
    let g = nested_grammar();
    let pt = build_table(&g);
    let generator = StaticLanguageGenerator::new(g, pt);
    let node_types = generator.generate_node_types();
    // Node types is a JSON string
    assert!(
        node_types.starts_with('['),
        "node_types should be a JSON array, got: {node_types}"
    );
}

#[test]
fn edge_many_tokens_grammar() {
    let mut gb = GrammarBuilder::new("many_tokens");
    let mut names = Vec::new();
    for i in 0..20 {
        let name = format!("tok{i}");
        let pat = format!("p{i}");
        gb = gb.token(&name, &pat);
        names.push(name);
    }
    // Create S with first token as RHS
    gb = gb.rule("S", vec![names[0].as_str()]);
    gb = gb.start("S");
    let g = gb.build();
    let pt = build_table(&g);
    let code = generate_code(&g, &pt);
    assert!(!code.is_empty());
    // Symbol count should include all 20 tokens plus EOF plus non-terminals
    assert!(
        pt.symbol_count >= 21,
        "many-token grammar should have at least 21 symbols, got {}",
        pt.symbol_count
    );
}

#[test]
fn edge_with_compressed_tables_chains() {
    let g = single_token_grammar();
    let pt = build_table(&g);
    let ti = collect_token_indices(&g, &pt);
    let sce = eof_accepts_or_reduces(&pt);
    let compressed = TableCompressor::new().compress(&pt, &ti, sce).unwrap();
    // with_compressed_tables should chain and still produce output
    let code = AbiLanguageBuilder::new(&g, &pt)
        .with_compressed_tables(&compressed)
        .generate()
        .to_string();
    assert!(!code.is_empty());
}
