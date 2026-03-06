//! ABI builder v4 tests for adze-tablegen.
//!
//! 57 tests covering:
//! 1. ABI builder construction — builds from grammar + table (8 tests)
//! 2. ABI output format — valid C-compatible struct layout (8 tests)
//! 3. Symbol table entries — all symbols represented (8 tests)
//! 4. Parse table encoding — action/goto tables in output (9 tests)
//! 5. Determinism — same input → same ABI output (8 tests)
//! 6. Grammar variations — simple, expression, recursive (8 tests)
//! 7. Edge cases — minimal grammar, large grammar, many states (8 tests)

use adze_glr_core::{FirstFollowSets, ParseTable, build_lr1_automaton};
use adze_ir::builder::GrammarBuilder;
use adze_ir::{Associativity, Grammar};
use adze_tablegen::compress::TableCompressor;
use adze_tablegen::{
    AbiLanguageBuilder, NodeTypesGenerator, StaticLanguageGenerator, collect_token_indices,
    eof_accepts_or_reduces,
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

// --- Grammars ---

fn minimal_grammar() -> Grammar {
    GrammarBuilder::new("minimal")
        .token("a", "a")
        .rule("S", vec!["a"])
        .start("S")
        .build()
}

fn pair_grammar() -> Grammar {
    GrammarBuilder::new("pair")
        .token("x", "x")
        .token("y", "y")
        .rule("S", vec!["x", "y"])
        .start("S")
        .build()
}

fn triple_alt_grammar() -> Grammar {
    GrammarBuilder::new("triple_alt")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("S", vec!["a"])
        .rule("S", vec!["b"])
        .rule("S", vec!["c"])
        .start("S")
        .build()
}

fn nested_rules_grammar() -> Grammar {
    GrammarBuilder::new("nested_rules")
        .token("m", "m")
        .token("n", "n")
        .rule("S", vec!["P", "Q"])
        .rule("P", vec!["m"])
        .rule("Q", vec!["n"])
        .start("S")
        .build()
}

fn left_rec_grammar() -> Grammar {
    GrammarBuilder::new("left_rec")
        .token("a", "a")
        .rule("L", vec!["a"])
        .rule("L", vec!["L", "a"])
        .start("L")
        .build()
}

fn right_rec_grammar() -> Grammar {
    GrammarBuilder::new("right_rec")
        .token("a", "a")
        .rule("R", vec!["a"])
        .rule("R", vec!["a", "R"])
        .start("R")
        .build()
}

fn chain_grammar() -> Grammar {
    GrammarBuilder::new("chain")
        .token("z", "z")
        .rule("S", vec!["A"])
        .rule("A", vec!["B"])
        .rule("B", vec!["C"])
        .rule("C", vec!["z"])
        .start("S")
        .build()
}

fn expression_grammar() -> Grammar {
    GrammarBuilder::new("expr")
        .token("NUM", r"\d+")
        .token("PLUS", r"\+")
        .token("STAR", r"\*")
        .rule_with_precedence("expr", vec!["expr", "PLUS", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "STAR", "expr"], 2, Associativity::Left)
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build()
}

fn nullable_grammar() -> Grammar {
    GrammarBuilder::new("nullable")
        .token("a", "a")
        .rule("S", vec!["A"])
        .rule("A", vec!["a"])
        .rule("A", vec![])
        .start("S")
        .build()
}

fn long_sequence_grammar() -> Grammar {
    GrammarBuilder::new("long_seq")
        .token("t1", "a")
        .token("t2", "b")
        .token("t3", "c")
        .token("t4", "d")
        .token("t5", "e")
        .token("t6", "f")
        .rule("S", vec!["t1", "t2", "t3", "t4", "t5", "t6"])
        .start("S")
        .build()
}

fn wide_grammar() -> Grammar {
    let mut gb = GrammarBuilder::new("wide");
    for i in 0..12 {
        let name = format!("w{i}");
        let pat = format!("{}", (b'a' + i as u8) as char);
        gb = gb.token(&name, &pat);
        gb = gb.rule("S", vec![&name]);
    }
    gb.start("S").build()
}

fn many_tokens_grammar() -> Grammar {
    let mut gb = GrammarBuilder::new("many_tok");
    let mut names = Vec::new();
    for i in 0..25 {
        let name = format!("tok{i}");
        let pat = format!("p{i}");
        gb = gb.token(&name, &pat);
        names.push(name);
    }
    gb = gb.rule("S", vec![names[0].as_str()]);
    gb = gb.start("S");
    gb.build()
}

fn diamond_grammar() -> Grammar {
    GrammarBuilder::new("diamond")
        .token("d", "d")
        .rule("S", vec!["A"])
        .rule("S", vec!["B"])
        .rule("A", vec!["C"])
        .rule("B", vec!["C"])
        .rule("C", vec!["d"])
        .start("S")
        .build()
}

fn multi_rule_grammar() -> Grammar {
    GrammarBuilder::new("multi_rule")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("S", vec!["X", "Y"])
        .rule("X", vec!["a"])
        .rule("X", vec!["b"])
        .rule("Y", vec!["c"])
        .start("S")
        .build()
}

// ============================================================================
// 1. ABI builder construction (8 tests)
// ============================================================================

#[test]
fn construction_minimal_grammar() {
    let g = minimal_grammar();
    let pt = build_table(&g);
    let _builder = AbiLanguageBuilder::new(&g, &pt);
}

#[test]
fn construction_pair_grammar() {
    let g = pair_grammar();
    let pt = build_table(&g);
    let _builder = AbiLanguageBuilder::new(&g, &pt);
}

#[test]
fn construction_triple_alt_grammar() {
    let g = triple_alt_grammar();
    let pt = build_table(&g);
    let _builder = AbiLanguageBuilder::new(&g, &pt);
}

#[test]
fn construction_nested_rules_grammar() {
    let g = nested_rules_grammar();
    let pt = build_table(&g);
    let _builder = AbiLanguageBuilder::new(&g, &pt);
}

#[test]
fn construction_chain_grammar() {
    let g = chain_grammar();
    let pt = build_table(&g);
    let _builder = AbiLanguageBuilder::new(&g, &pt);
}

#[test]
fn construction_expression_grammar() {
    let g = expression_grammar();
    let pt = build_table(&g);
    let _builder = AbiLanguageBuilder::new(&g, &pt);
}

#[test]
fn construction_default_parse_table() {
    let g = Grammar::new("empty".to_string());
    let pt = ParseTable::default();
    let _builder = AbiLanguageBuilder::new(&g, &pt);
}

#[test]
fn construction_with_compressed_tables() {
    let g = minimal_grammar();
    let pt = build_table(&g);
    let ti = collect_token_indices(&g, &pt);
    let sce = eof_accepts_or_reduces(&pt);
    let compressed = TableCompressor::new().compress(&pt, &ti, sce).unwrap();
    let _builder = AbiLanguageBuilder::new(&g, &pt).with_compressed_tables(&compressed);
}

// ============================================================================
// 2. ABI output format — valid C-compatible struct layout (8 tests)
// ============================================================================

#[test]
fn output_nonempty_for_minimal_grammar() {
    let g = minimal_grammar();
    let pt = build_table(&g);
    let code = generate_code(&g, &pt);
    assert!(!code.is_empty(), "generated code must not be empty");
}

#[test]
fn output_contains_language_struct() {
    let g = minimal_grammar();
    let pt = build_table(&g);
    let code = generate_code(&g, &pt);
    assert!(
        code.contains("LANGUAGE") || code.contains("TSLanguage"),
        "output must contain LANGUAGE struct"
    );
}

#[test]
fn output_contains_symbol_metadata() {
    let g = minimal_grammar();
    let pt = build_table(&g);
    let code = generate_code(&g, &pt);
    assert!(
        code.contains("SYMBOL_METADATA") || code.contains("symbol_metadata"),
        "output must contain symbol metadata array"
    );
}

#[test]
fn output_contains_parse_table_data() {
    let g = minimal_grammar();
    let pt = build_table(&g);
    let code = generate_code(&g, &pt);
    assert!(
        code.contains("PARSE_TABLE") || code.contains("SMALL_PARSE_TABLE"),
        "output must contain parse table data"
    );
}

#[test]
fn output_contains_parse_actions() {
    let g = minimal_grammar();
    let pt = build_table(&g);
    let code = generate_code(&g, &pt);
    assert!(
        code.contains("PARSE_ACTIONS") || code.contains("parse_actions"),
        "output must contain parse actions array"
    );
}

#[test]
fn output_contains_lex_modes() {
    let g = minimal_grammar();
    let pt = build_table(&g);
    let code = generate_code(&g, &pt);
    assert!(
        code.contains("LEX_MODES") || code.contains("lex_modes"),
        "output must contain lex modes array"
    );
}

#[test]
fn output_contains_field_map() {
    let g = minimal_grammar();
    let pt = build_table(&g);
    let code = generate_code(&g, &pt);
    assert!(
        code.contains("FIELD_MAP_SLICES") || code.contains("field_map_slices"),
        "output must contain field map slices"
    );
}

#[test]
fn output_contains_version_constant() {
    let g = minimal_grammar();
    let pt = build_table(&g);
    let code = generate_code(&g, &pt);
    assert!(
        code.contains("TREE_SITTER_LANGUAGE_VERSION") || code.contains("version"),
        "output must reference ABI version"
    );
}

// ============================================================================
// 3. Symbol table entries — all symbols represented (8 tests)
// ============================================================================

#[test]
fn symbols_minimal_grammar_has_token_name() {
    let g = minimal_grammar();
    let pt = build_table(&g);
    let code = generate_code(&g, &pt);
    assert!(code.contains('a'), "output must mention token 'a'");
}

#[test]
fn symbols_pair_grammar_has_both_tokens() {
    let g = pair_grammar();
    let pt = build_table(&g);
    let code = generate_code(&g, &pt);
    assert!(code.contains('x'), "output must mention token 'x'");
    assert!(code.contains('y'), "output must mention token 'y'");
}

#[test]
fn symbols_triple_alt_all_three_present() {
    let g = triple_alt_grammar();
    let pt = build_table(&g);
    let code = generate_code(&g, &pt);
    for tok in &["a", "b", "c"] {
        assert!(code.contains(tok), "output must mention token '{tok}'");
    }
}

#[test]
fn symbols_nested_has_nonterminal_names() {
    let g = nested_rules_grammar();
    let pt = build_table(&g);
    let code = generate_code(&g, &pt);
    assert!(
        code.contains('S') || code.contains('P') || code.contains('Q'),
        "output must mention at least one non-terminal"
    );
}

#[test]
fn symbols_chain_has_intermediate_names() {
    let g = chain_grammar();
    let pt = build_table(&g);
    let code = generate_code(&g, &pt);
    assert!(
        code.contains('A') || code.contains('B') || code.contains('C'),
        "output must mention intermediate non-terminals"
    );
}

#[test]
fn symbols_expression_has_operator_names() {
    let g = expression_grammar();
    let pt = build_table(&g);
    let code = generate_code(&g, &pt);
    // The expression grammar must reference "expr" in output
    assert!(
        code.contains("expr"),
        "output must mention the expr non-terminal"
    );
}

#[test]
fn symbols_count_matches_parse_table() {
    let g = pair_grammar();
    let pt = build_table(&g);
    let code = generate_code(&g, &pt);
    let expected = pt.symbol_count;
    assert!(
        code.contains(&format!("{expected}")),
        "output must reference symbol_count = {expected}"
    );
}

#[test]
fn symbols_public_symbol_map_present() {
    let g = minimal_grammar();
    let pt = build_table(&g);
    let code = generate_code(&g, &pt);
    assert!(
        code.contains("PUBLIC_SYMBOL_MAP") || code.contains("public_symbol_map"),
        "output must contain public symbol map"
    );
}

// ============================================================================
// 4. Parse table encoding — action/goto tables in output (9 tests)
// ============================================================================

#[test]
fn encoding_action_table_row_count_matches_states() {
    let g = minimal_grammar();
    let pt = build_table(&g);
    assert_eq!(
        pt.action_table.len(),
        pt.state_count,
        "action table rows must equal state count"
    );
}

#[test]
fn encoding_action_table_column_width_matches_symbols() {
    let g = pair_grammar();
    let pt = build_table(&g);
    for (idx, row) in pt.action_table.iter().enumerate() {
        assert_eq!(
            row.len(),
            pt.symbol_count,
            "state {idx}: action column count must equal symbol count"
        );
    }
}

#[test]
fn encoding_goto_table_row_count_matches_states() {
    let g = minimal_grammar();
    let pt = build_table(&g);
    assert_eq!(
        pt.goto_table.len(),
        pt.state_count,
        "goto table rows must equal state count"
    );
}

#[test]
fn encoding_goto_table_column_width_matches_symbols() {
    let g = pair_grammar();
    let pt = build_table(&g);
    for (idx, row) in pt.goto_table.iter().enumerate() {
        assert_eq!(
            row.len(),
            pt.symbol_count,
            "state {idx}: goto column count must equal symbol count"
        );
    }
}

#[test]
fn encoding_initial_state_has_actions() {
    let g = minimal_grammar();
    let pt = build_table(&g);
    let has_action = pt.action_table[0].iter().any(|cell| !cell.is_empty());
    assert!(has_action, "initial state must have at least one action");
}

#[test]
fn encoding_triple_alt_initial_state_has_three_shifts() {
    let g = triple_alt_grammar();
    let pt = build_table(&g);
    let shift_count = pt.action_table[0]
        .iter()
        .filter(|cell| !cell.is_empty())
        .count();
    assert!(
        shift_count >= 3,
        "initial state should shift on 3 tokens, got {shift_count}"
    );
}

#[test]
fn encoding_compression_roundtrip_minimal() {
    let g = minimal_grammar();
    let pt = build_table(&g);
    let ti = collect_token_indices(&g, &pt);
    let sce = eof_accepts_or_reduces(&pt);
    let result = TableCompressor::new().compress(&pt, &ti, sce);
    assert!(
        result.is_ok(),
        "compression must succeed for minimal grammar"
    );
}

#[test]
fn encoding_compression_roundtrip_pair() {
    let g = pair_grammar();
    let pt = build_table(&g);
    let ti = collect_token_indices(&g, &pt);
    let sce = eof_accepts_or_reduces(&pt);
    let result = TableCompressor::new().compress(&pt, &ti, sce);
    assert!(result.is_ok(), "compression must succeed for pair grammar");
}

#[test]
fn encoding_compression_roundtrip_expression() {
    let g = expression_grammar();
    let pt = build_table(&g);
    let ti = collect_token_indices(&g, &pt);
    let sce = eof_accepts_or_reduces(&pt);
    let result = TableCompressor::new().compress(&pt, &ti, sce);
    assert!(
        result.is_ok(),
        "compression must succeed for expression grammar"
    );
}

// ============================================================================
// 5. Determinism — same input → same ABI output (8 tests)
// ============================================================================

#[test]
fn determinism_minimal_grammar_same_output() {
    let g = minimal_grammar();
    let pt = build_table(&g);
    let code1 = generate_code(&g, &pt);
    let code2 = generate_code(&g, &pt);
    assert_eq!(code1, code2, "generation must be deterministic");
}

#[test]
fn determinism_pair_grammar_same_output() {
    let g = pair_grammar();
    let pt = build_table(&g);
    let code1 = generate_code(&g, &pt);
    let code2 = generate_code(&g, &pt);
    assert_eq!(code1, code2);
}

#[test]
fn determinism_triple_alt_same_output() {
    let g = triple_alt_grammar();
    let pt = build_table(&g);
    let code1 = generate_code(&g, &pt);
    let code2 = generate_code(&g, &pt);
    assert_eq!(code1, code2);
}

#[test]
fn determinism_nested_same_output() {
    let g = nested_rules_grammar();
    let pt = build_table(&g);
    let code1 = generate_code(&g, &pt);
    let code2 = generate_code(&g, &pt);
    assert_eq!(code1, code2);
}

#[test]
fn determinism_expression_same_output() {
    let g = expression_grammar();
    let pt = build_table(&g);
    let code1 = generate_code(&g, &pt);
    let code2 = generate_code(&g, &pt);
    assert_eq!(code1, code2);
}

#[test]
fn determinism_chain_same_output() {
    let g = chain_grammar();
    let pt = build_table(&g);
    let code1 = generate_code(&g, &pt);
    let code2 = generate_code(&g, &pt);
    assert_eq!(code1, code2);
}

#[test]
fn determinism_different_grammars_differ() {
    let g1 = minimal_grammar();
    let pt1 = build_table(&g1);
    let code1 = generate_code(&g1, &pt1);

    let g2 = pair_grammar();
    let pt2 = build_table(&g2);
    let code2 = generate_code(&g2, &pt2);

    assert_ne!(
        code1, code2,
        "different grammars must produce different code"
    );
}

#[test]
fn determinism_rebuild_same_grammar_same_output() {
    let code1 = {
        let g = minimal_grammar();
        let pt = build_table(&g);
        generate_code(&g, &pt)
    };
    let code2 = {
        let g = minimal_grammar();
        let pt = build_table(&g);
        generate_code(&g, &pt)
    };
    assert_eq!(code1, code2, "rebuilding same grammar must yield same code");
}

// ============================================================================
// 6. Grammar variations — simple, expression, recursive (8 tests)
// ============================================================================

#[test]
fn variation_left_recursive_builds() {
    let g = left_rec_grammar();
    let pt = build_table(&g);
    let code = generate_code(&g, &pt);
    assert!(!code.is_empty());
}

#[test]
fn variation_right_recursive_builds() {
    let g = right_rec_grammar();
    let pt = build_table(&g);
    let code = generate_code(&g, &pt);
    assert!(!code.is_empty());
}

#[test]
fn variation_left_rec_state_count_bounded() {
    let g = left_rec_grammar();
    let pt = build_table(&g);
    assert!(
        pt.state_count < 1000,
        "left-recursive grammar must have bounded state count"
    );
}

#[test]
fn variation_right_rec_state_count_bounded() {
    let g = right_rec_grammar();
    let pt = build_table(&g);
    assert!(
        pt.state_count < 1000,
        "right-recursive grammar must have bounded state count"
    );
}

#[test]
fn variation_expression_grammar_has_precedence_tokens() {
    let g = expression_grammar();
    let pt = build_table(&g);
    // Expression grammar should have at least NUM, PLUS, STAR, EOF, expr
    assert!(
        pt.symbol_count >= 5,
        "expression grammar should have at least 5 symbols, got {}",
        pt.symbol_count
    );
}

#[test]
fn variation_diamond_grammar_builds() {
    let g = diamond_grammar();
    let pt = build_table(&g);
    let code = generate_code(&g, &pt);
    assert!(!code.is_empty());
}

#[test]
fn variation_multi_rule_grammar_builds() {
    let g = multi_rule_grammar();
    let pt = build_table(&g);
    let code = generate_code(&g, &pt);
    assert!(!code.is_empty());
}

#[test]
fn variation_nullable_grammar_builds() {
    let g = nullable_grammar();
    let pt = build_table(&g);
    let code = generate_code(&g, &pt);
    assert!(!code.is_empty());
}

// ============================================================================
// 7. Edge cases — minimal grammar, large grammar, many states (8 tests)
// ============================================================================

#[test]
fn edge_wide_grammar_generates_code() {
    let g = wide_grammar();
    let pt = build_table(&g);
    let code = generate_code(&g, &pt);
    assert!(!code.is_empty());
}

#[test]
fn edge_long_sequence_generates_code() {
    let g = long_sequence_grammar();
    let pt = build_table(&g);
    let code = generate_code(&g, &pt);
    assert!(!code.is_empty());
}

#[test]
fn edge_many_tokens_grammar_generates_code() {
    let g = many_tokens_grammar();
    let pt = build_table(&g);
    let code = generate_code(&g, &pt);
    assert!(!code.is_empty());
    assert!(
        pt.symbol_count >= 26,
        "many-token grammar should have at least 26 symbols, got {}",
        pt.symbol_count
    );
}

#[test]
fn edge_static_generator_produces_code() {
    let g = minimal_grammar();
    let pt = build_table(&g);
    let generator = StaticLanguageGenerator::new(g, pt);
    let code = generator.generate_language_code().to_string();
    assert!(
        !code.is_empty(),
        "StaticLanguageGenerator must produce code"
    );
}

#[test]
fn edge_static_generator_node_types_json() {
    let g = nested_rules_grammar();
    let pt = build_table(&g);
    let generator = StaticLanguageGenerator::new(g, pt);
    let node_types = generator.generate_node_types();
    assert!(
        node_types.starts_with('['),
        "node_types must be a JSON array, got: {node_types}"
    );
}

#[test]
fn edge_node_types_generator_produces_json() {
    let g = nested_rules_grammar();
    let generator = NodeTypesGenerator::new(&g);
    let json = generator.generate().expect("generate must succeed");
    assert!(
        json.starts_with('['),
        "NodeTypesGenerator output must be a JSON array"
    );
}

#[test]
fn edge_compressed_tables_chain_generates_code() {
    let g = minimal_grammar();
    let pt = build_table(&g);
    let ti = collect_token_indices(&g, &pt);
    let sce = eof_accepts_or_reduces(&pt);
    let compressed = TableCompressor::new().compress(&pt, &ti, sce).unwrap();
    let code = AbiLanguageBuilder::new(&g, &pt)
        .with_compressed_tables(&compressed)
        .generate()
        .to_string();
    assert!(!code.is_empty());
}

#[test]
fn edge_goto_table_has_valid_nonterminal_entries() {
    let g = nested_rules_grammar();
    let pt = build_table(&g);
    let has_goto = pt
        .goto_table
        .iter()
        .any(|row| row.iter().any(|s| s.0 != u16::MAX));
    assert!(
        has_goto,
        "nested grammar must have at least one valid goto entry"
    );
}
