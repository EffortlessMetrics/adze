//! Comprehensive roundtrip tests for table compression in adze-tablegen (v9).
//!
//! Covers: minimal and complex grammars, determinism, state/symbol preservation,
//! static code generation, precedence, inline rules, extras, externals, conflicts,
//! alternatives, many-token/rule grammars, deep nesting, recursion, and
//! `CompressedParseTable::from_parse_table`.

use adze_glr_core::{FirstFollowSets, build_lr1_automaton};
use adze_ir::builder::GrammarBuilder;
use adze_ir::{Associativity, ConflictDeclaration, ConflictResolution, Grammar};
use adze_tablegen::compress::{CompressedParseTable, CompressedTables, TableCompressor};
use adze_tablegen::{StaticLanguageGenerator, collect_token_indices, eof_accepts_or_reduces};

// ── Helpers ─────────────────────────────────────────────────────────────────

/// Full pipeline: Grammar → FIRST/FOLLOW → LR(1) → CompressedTables.
fn compress_pipeline(grammar: &mut Grammar) -> (adze_glr_core::ParseTable, CompressedTables) {
    let ff = FirstFollowSets::compute_normalized(grammar).expect("FIRST/FOLLOW computation failed");
    let table = build_lr1_automaton(grammar, &ff).expect("LR(1) automaton construction failed");
    let token_indices = collect_token_indices(grammar, &table);
    let start_empty = eof_accepts_or_reduces(&table);
    let compressor = TableCompressor::new();
    let compressed = compressor
        .compress(&table, &token_indices, start_empty)
        .expect("Table compression failed");
    (table, compressed)
}

/// Compress and also return grammar+table for generator tests (consumes grammar).
fn make_pipeline(grammar: Grammar) -> (Grammar, adze_glr_core::ParseTable) {
    let ff = FirstFollowSets::compute(&grammar).expect("FIRST/FOLLOW failed");
    let pt = build_lr1_automaton(&grammar, &ff).expect("LR(1) automaton failed");
    (grammar, pt)
}

/// Build a minimal grammar: start → a.
fn minimal_grammar(name: &str) -> Grammar {
    GrammarBuilder::new(name)
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build()
}

/// Build an arithmetic grammar with precedence.
fn arithmetic_grammar(name: &str) -> Grammar {
    GrammarBuilder::new(name)
        .token("num", r"\d+")
        .token("plus", r"\+")
        .token("star", r"\*")
        .rule("expr", vec!["num"])
        .rule_with_precedence("expr", vec!["expr", "plus", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "star", "expr"], 2, Associativity::Left)
        .start("expr")
        .build()
}

/// Build an alternatives grammar: start → t0 | t1 | … | tN-1.
fn alternatives_grammar(name: &str, n: usize) -> Grammar {
    let mut b = GrammarBuilder::new(name);
    for i in 0..n {
        let tok: &str = Box::leak(format!("t{i}").into_boxed_str());
        b = b.token(tok, tok).rule("start", vec![tok]);
    }
    b.start("start").build()
}

/// Build a many-token grammar with `n` tokens each in its own rule.
fn many_token_grammar(name: &str, n: usize) -> Grammar {
    let mut b = GrammarBuilder::new(name);
    for i in 0..n {
        let tok: &str = Box::leak(format!("tok{i}").into_boxed_str());
        b = b.token(tok, tok).rule("start", vec![tok]);
    }
    b.start("start").build()
}

/// Build a many-rule grammar: many non-terminals chaining to a single token.
fn many_rule_grammar(name: &str, n: usize) -> Grammar {
    let mut b = GrammarBuilder::new(name).token("x", "x");
    let names: Vec<String> = (0..n).map(|i| format!("r{i}")).collect();
    let first: &str = Box::leak(names[0].clone().into_boxed_str());
    b = b.rule(first, vec!["x"]);
    for i in 1..n {
        let lhs: &str = Box::leak(names[i].clone().into_boxed_str());
        let rhs: &str = Box::leak(names[i - 1].clone().into_boxed_str());
        b = b.rule(lhs, vec![rhs]);
    }
    let last: &str = Box::leak(names[n - 1].clone().into_boxed_str());
    b = b.rule("start", vec![last]);
    b.start("start").build()
}

/// Build a deep-nesting chain grammar of given depth.
fn deep_nesting_grammar(name: &str, depth: usize) -> Grammar {
    many_rule_grammar(name, depth)
}

/// Build a left-recursive grammar: list → list a | a.
fn left_recursive_grammar(name: &str) -> Grammar {
    GrammarBuilder::new(name)
        .token("a", "a")
        .rule("list", vec!["list", "a"])
        .rule("list", vec!["a"])
        .rule("start", vec!["list"])
        .start("start")
        .build()
}

/// Build a right-recursive grammar: list → a list | a.
fn right_recursive_grammar(name: &str) -> Grammar {
    GrammarBuilder::new(name)
        .token("a", "a")
        .rule("list", vec!["a", "list"])
        .rule("list", vec!["a"])
        .rule("start", vec!["list"])
        .start("start")
        .build()
}

// ═══════════════════════════════════════════════════════════════════════════
// 1. Compress minimal grammar → non-empty result
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn cr_v9_minimal_compress_action_non_empty() {
    let mut g = minimal_grammar("cr_v9_min1");
    let (_pt, compressed) = compress_pipeline(&mut g);
    assert!(!compressed.action_table.data.is_empty());
}

#[test]
fn cr_v9_minimal_compress_goto_non_empty() {
    let mut g = minimal_grammar("cr_v9_min2");
    let (_pt, compressed) = compress_pipeline(&mut g);
    assert!(!compressed.goto_table.data.is_empty());
}

#[test]
fn cr_v9_minimal_compress_action_has_row_offsets() {
    let mut g = minimal_grammar("cr_v9_min3");
    let (_pt, compressed) = compress_pipeline(&mut g);
    assert!(!compressed.action_table.row_offsets.is_empty());
}

#[test]
fn cr_v9_minimal_compress_goto_has_row_offsets() {
    let mut g = minimal_grammar("cr_v9_min4");
    let (_pt, compressed) = compress_pipeline(&mut g);
    assert!(!compressed.goto_table.row_offsets.is_empty());
}

// ═══════════════════════════════════════════════════════════════════════════
// 2. Compress arithmetic grammar → non-empty result
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn cr_v9_arithmetic_compress_action_non_empty() {
    let mut g = arithmetic_grammar("cr_v9_arith1");
    let (_pt, compressed) = compress_pipeline(&mut g);
    assert!(!compressed.action_table.data.is_empty());
}

#[test]
fn cr_v9_arithmetic_compress_goto_non_empty() {
    let mut g = arithmetic_grammar("cr_v9_arith2");
    let (_pt, compressed) = compress_pipeline(&mut g);
    assert!(!compressed.goto_table.data.is_empty());
}

#[test]
fn cr_v9_arithmetic_has_more_states_than_minimal() {
    let mut g_min = minimal_grammar("cr_v9_arith3a");
    let (pt_min, _) = compress_pipeline(&mut g_min);
    let mut g_arith = arithmetic_grammar("cr_v9_arith3b");
    let (pt_arith, _) = compress_pipeline(&mut g_arith);
    assert!(
        pt_arith.state_count >= pt_min.state_count,
        "arithmetic grammar should have at least as many states as minimal"
    );
}

#[test]
fn cr_v9_arithmetic_compress_more_entries_than_minimal() {
    let mut g_min = minimal_grammar("cr_v9_arith4a");
    let (_, c_min) = compress_pipeline(&mut g_min);
    let mut g_arith = arithmetic_grammar("cr_v9_arith4b");
    let (_, c_arith) = compress_pipeline(&mut g_arith);
    assert!(
        c_arith.action_table.data.len() >= c_min.action_table.data.len(),
        "arithmetic should have at least as many action entries"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// 3. Compress then generate static → non-empty code
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn cr_v9_static_gen_minimal_non_empty() {
    let (g, pt) = make_pipeline(minimal_grammar("cr_v9_sg1"));
    let slg = StaticLanguageGenerator::new(g, pt);
    let code = slg.generate_language_code().to_string();
    assert!(!code.is_empty());
}

#[test]
fn cr_v9_static_gen_arithmetic_non_empty() {
    let (g, pt) = make_pipeline(arithmetic_grammar("cr_v9_sg2"));
    let slg = StaticLanguageGenerator::new(g, pt);
    let code = slg.generate_language_code().to_string();
    assert!(!code.is_empty());
}

#[test]
fn cr_v9_static_gen_alternatives_non_empty() {
    let (g, pt) = make_pipeline(alternatives_grammar("cr_v9_sg3", 4));
    let slg = StaticLanguageGenerator::new(g, pt);
    let code = slg.generate_language_code().to_string();
    assert!(!code.is_empty());
}

#[test]
fn cr_v9_static_gen_after_compress_tables() {
    let (g, pt) = make_pipeline(minimal_grammar("cr_v9_sg4"));
    let mut slg = StaticLanguageGenerator::new(g, pt);
    slg.compress_tables().expect("compress_tables failed");
    assert!(slg.compressed_tables.is_some());
    let code = slg.generate_language_code().to_string();
    assert!(!code.is_empty());
}

// ═══════════════════════════════════════════════════════════════════════════
// 4. Two compress calls → deterministic (same result)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn cr_v9_determinism_minimal_action_data() {
    let mut g1 = minimal_grammar("cr_v9_det1a");
    let (_, c1) = compress_pipeline(&mut g1);
    let mut g2 = minimal_grammar("cr_v9_det1b");
    let (_, c2) = compress_pipeline(&mut g2);
    assert_eq!(c1.action_table.data.len(), c2.action_table.data.len());
}

#[test]
fn cr_v9_determinism_minimal_goto_data() {
    let mut g1 = minimal_grammar("cr_v9_det2a");
    let (_, c1) = compress_pipeline(&mut g1);
    let mut g2 = minimal_grammar("cr_v9_det2b");
    let (_, c2) = compress_pipeline(&mut g2);
    assert_eq!(c1.goto_table.data.len(), c2.goto_table.data.len());
}

#[test]
fn cr_v9_determinism_arithmetic_action_offsets() {
    let mut g1 = arithmetic_grammar("cr_v9_det3a");
    let (_, c1) = compress_pipeline(&mut g1);
    let mut g2 = arithmetic_grammar("cr_v9_det3b");
    let (_, c2) = compress_pipeline(&mut g2);
    assert_eq!(c1.action_table.row_offsets, c2.action_table.row_offsets);
}

#[test]
fn cr_v9_determinism_arithmetic_goto_offsets() {
    let mut g1 = arithmetic_grammar("cr_v9_det4a");
    let (_, c1) = compress_pipeline(&mut g1);
    let mut g2 = arithmetic_grammar("cr_v9_det4b");
    let (_, c2) = compress_pipeline(&mut g2);
    assert_eq!(c1.goto_table.row_offsets, c2.goto_table.row_offsets);
}

#[test]
fn cr_v9_determinism_threshold_unchanged() {
    let mut g1 = minimal_grammar("cr_v9_det5a");
    let (_, c1) = compress_pipeline(&mut g1);
    let mut g2 = minimal_grammar("cr_v9_det5b");
    let (_, c2) = compress_pipeline(&mut g2);
    assert_eq!(c1.small_table_threshold, c2.small_table_threshold);
}

// ═══════════════════════════════════════════════════════════════════════════
// 5. Different grammars → different compressed output
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn cr_v9_different_grammars_different_action_len() {
    let mut g_min = minimal_grammar("cr_v9_diff1a");
    let (_, c_min) = compress_pipeline(&mut g_min);
    let mut g_alt = alternatives_grammar("cr_v9_diff1b", 5);
    let (_, c_alt) = compress_pipeline(&mut g_alt);
    assert_ne!(c_min.action_table.data.len(), c_alt.action_table.data.len());
}

#[test]
fn cr_v9_different_grammars_different_goto_len() {
    let mut g_min = minimal_grammar("cr_v9_diff2a");
    let (_, c_min) = compress_pipeline(&mut g_min);
    let mut g_chain = many_rule_grammar("cr_v9_diff2b", 5);
    let (_, c_chain) = compress_pipeline(&mut g_chain);
    assert_ne!(c_min.goto_table.data.len(), c_chain.goto_table.data.len());
}

#[test]
fn cr_v9_different_grammars_different_row_offsets() {
    let mut g_min = minimal_grammar("cr_v9_diff3a");
    let (_, c_min) = compress_pipeline(&mut g_min);
    let mut g_arith = arithmetic_grammar("cr_v9_diff3b");
    let (_, c_arith) = compress_pipeline(&mut g_arith);
    assert_ne!(
        c_min.action_table.row_offsets.len(),
        c_arith.action_table.row_offsets.len()
    );
}

#[test]
fn cr_v9_different_grammars_different_state_count() {
    let mut g_min = minimal_grammar("cr_v9_diff4a");
    let (pt_min, _) = compress_pipeline(&mut g_min);
    let mut g_many = many_token_grammar("cr_v9_diff4b", 8);
    let (pt_many, _) = compress_pipeline(&mut g_many);
    assert_ne!(pt_min.state_count, pt_many.state_count);
}

// ═══════════════════════════════════════════════════════════════════════════
// 6. Compress preserves all states
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn cr_v9_preserves_states_minimal() {
    let mut g = minimal_grammar("cr_v9_ps1");
    let (pt, compressed) = compress_pipeline(&mut g);
    assert_eq!(
        compressed.action_table.row_offsets.len(),
        pt.state_count + 1
    );
}

#[test]
fn cr_v9_preserves_states_arithmetic() {
    let mut g = arithmetic_grammar("cr_v9_ps2");
    let (pt, compressed) = compress_pipeline(&mut g);
    assert_eq!(
        compressed.action_table.row_offsets.len(),
        pt.state_count + 1
    );
}

#[test]
fn cr_v9_preserves_states_alternatives() {
    let mut g = alternatives_grammar("cr_v9_ps3", 4);
    let (pt, compressed) = compress_pipeline(&mut g);
    assert_eq!(
        compressed.action_table.row_offsets.len(),
        pt.state_count + 1
    );
}

#[test]
fn cr_v9_preserves_states_chain() {
    let mut g = many_rule_grammar("cr_v9_ps4", 6);
    let (pt, compressed) = compress_pipeline(&mut g);
    assert_eq!(
        compressed.action_table.row_offsets.len(),
        pt.state_count + 1
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// 7. Compress preserves all symbols
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn cr_v9_preserves_symbols_via_cpt_minimal() {
    let mut g = minimal_grammar("cr_v9_sym1");
    let (pt, _) = compress_pipeline(&mut g);
    let cpt = CompressedParseTable::from_parse_table(&pt);
    assert_eq!(cpt.symbol_count(), pt.symbol_count);
}

#[test]
fn cr_v9_preserves_symbols_via_cpt_arithmetic() {
    let mut g = arithmetic_grammar("cr_v9_sym2");
    let (pt, _) = compress_pipeline(&mut g);
    let cpt = CompressedParseTable::from_parse_table(&pt);
    assert_eq!(cpt.symbol_count(), pt.symbol_count);
}

#[test]
fn cr_v9_preserves_state_count_via_cpt() {
    let mut g = alternatives_grammar("cr_v9_sym3", 3);
    let (pt, _) = compress_pipeline(&mut g);
    let cpt = CompressedParseTable::from_parse_table(&pt);
    assert_eq!(cpt.state_count(), pt.state_count);
}

#[test]
fn cr_v9_preserves_symbols_many_tokens() {
    let mut g = many_token_grammar("cr_v9_sym4", 10);
    let (pt, _) = compress_pipeline(&mut g);
    let cpt = CompressedParseTable::from_parse_table(&pt);
    assert_eq!(cpt.symbol_count(), pt.symbol_count);
}

// ═══════════════════════════════════════════════════════════════════════════
// 8. Static code contains grammar name
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn cr_v9_static_code_contains_name_minimal() {
    let (g, pt) = make_pipeline(minimal_grammar("cr_v9_name1"));
    let code = StaticLanguageGenerator::new(g, pt)
        .generate_language_code()
        .to_string();
    assert!(
        code.contains("cr_v9_name1"),
        "generated code should contain the grammar name"
    );
}

#[test]
fn cr_v9_static_code_contains_name_arithmetic() {
    let (g, pt) = make_pipeline(arithmetic_grammar("cr_v9_name2"));
    let code = StaticLanguageGenerator::new(g, pt)
        .generate_language_code()
        .to_string();
    assert!(
        code.contains("cr_v9_name2"),
        "generated code should contain the grammar name"
    );
}

#[test]
fn cr_v9_static_code_contains_name_alternatives() {
    let (g, pt) = make_pipeline(alternatives_grammar("cr_v9_name3", 3));
    let code = StaticLanguageGenerator::new(g, pt)
        .generate_language_code()
        .to_string();
    assert!(
        code.contains("cr_v9_name3"),
        "generated code should contain the grammar name"
    );
}

#[test]
fn cr_v9_static_code_contains_name_chain() {
    let (g, pt) = make_pipeline(many_rule_grammar("cr_v9_name4", 3));
    let code = StaticLanguageGenerator::new(g, pt)
        .generate_language_code()
        .to_string();
    assert!(
        code.contains("cr_v9_name4"),
        "generated code should contain the grammar name"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// 9. Static code is valid Rust (contains "fn" or "const" or "static")
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn cr_v9_static_code_has_rust_keywords_minimal() {
    let (g, pt) = make_pipeline(minimal_grammar("cr_v9_rk1"));
    let code = StaticLanguageGenerator::new(g, pt)
        .generate_language_code()
        .to_string();
    let has_keyword = code.contains("fn ") || code.contains("const ") || code.contains("static ");
    assert!(has_keyword, "generated code should contain Rust keywords");
}

#[test]
fn cr_v9_static_code_has_rust_keywords_arithmetic() {
    let (g, pt) = make_pipeline(arithmetic_grammar("cr_v9_rk2"));
    let code = StaticLanguageGenerator::new(g, pt)
        .generate_language_code()
        .to_string();
    let has_keyword = code.contains("fn ") || code.contains("const ") || code.contains("static ");
    assert!(has_keyword, "generated code should contain Rust keywords");
}

#[test]
fn cr_v9_static_code_has_rust_keywords_alternatives() {
    let (g, pt) = make_pipeline(alternatives_grammar("cr_v9_rk3", 5));
    let code = StaticLanguageGenerator::new(g, pt)
        .generate_language_code()
        .to_string();
    let has_keyword = code.contains("fn ") || code.contains("const ") || code.contains("static ");
    assert!(has_keyword, "generated code should contain Rust keywords");
}

#[test]
fn cr_v9_static_code_has_rust_keywords_recursive() {
    let (g, pt) = make_pipeline(left_recursive_grammar("cr_v9_rk4"));
    let code = StaticLanguageGenerator::new(g, pt)
        .generate_language_code()
        .to_string();
    let has_keyword = code.contains("fn ") || code.contains("const ") || code.contains("static ");
    assert!(has_keyword, "generated code should contain Rust keywords");
}

// ═══════════════════════════════════════════════════════════════════════════
// 10. Compress handles grammar with precedence
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn cr_v9_precedence_compresses_successfully() {
    let mut g = arithmetic_grammar("cr_v9_prec1");
    let (_pt, compressed) = compress_pipeline(&mut g);
    assert!(!compressed.action_table.data.is_empty());
}

#[test]
fn cr_v9_precedence_three_levels() {
    let mut g = GrammarBuilder::new("cr_v9_prec2")
        .token("num", r"\d+")
        .token("plus", r"\+")
        .token("star", r"\*")
        .token("caret", r"\^")
        .rule("expr", vec!["num"])
        .rule_with_precedence("expr", vec!["expr", "plus", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "star", "expr"], 2, Associativity::Left)
        .rule_with_precedence(
            "expr",
            vec!["expr", "caret", "expr"],
            3,
            Associativity::Right,
        )
        .start("expr")
        .build();
    let (_pt, compressed) = compress_pipeline(&mut g);
    assert!(!compressed.action_table.data.is_empty());
    assert!(!compressed.goto_table.data.is_empty());
}

#[test]
fn cr_v9_precedence_right_associative() {
    let mut g = GrammarBuilder::new("cr_v9_prec3")
        .token("x", "x")
        .token("op", "=")
        .rule("expr", vec!["x"])
        .rule_with_precedence("expr", vec!["expr", "op", "expr"], 1, Associativity::Right)
        .start("expr")
        .build();
    let (_pt, compressed) = compress_pipeline(&mut g);
    assert!(!compressed.action_table.data.is_empty());
}

#[test]
fn cr_v9_precedence_with_builder_decl() {
    let mut g = GrammarBuilder::new("cr_v9_prec4")
        .token("num", r"\d+")
        .token("plus", r"\+")
        .token("star", r"\*")
        .precedence(1, Associativity::Left, vec!["plus"])
        .precedence(2, Associativity::Left, vec!["star"])
        .rule("expr", vec!["num"])
        .rule("expr", vec!["expr", "plus", "expr"])
        .rule("expr", vec!["expr", "star", "expr"])
        .start("expr")
        .build();
    let (_pt, compressed) = compress_pipeline(&mut g);
    assert!(!compressed.action_table.data.is_empty());
}

// ═══════════════════════════════════════════════════════════════════════════
// 11. Compress handles grammar with inline rules
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn cr_v9_inline_rule_compresses() {
    let mut g = GrammarBuilder::new("cr_v9_inl1")
        .token("a", "a")
        .token("b", "b")
        .rule("helper", vec!["a"])
        .rule("start", vec!["helper", "b"])
        .inline("helper")
        .start("start")
        .build();
    let (_pt, compressed) = compress_pipeline(&mut g);
    assert!(!compressed.action_table.data.is_empty());
}

#[test]
fn cr_v9_inline_rule_action_offsets_valid() {
    let mut g = GrammarBuilder::new("cr_v9_inl2")
        .token("x", "x")
        .token("y", "y")
        .rule("inner", vec!["x"])
        .rule("start", vec!["inner", "y"])
        .inline("inner")
        .start("start")
        .build();
    let (pt, compressed) = compress_pipeline(&mut g);
    assert_eq!(
        compressed.action_table.row_offsets.len(),
        pt.state_count + 1
    );
}

#[test]
fn cr_v9_multiple_inline_rules() {
    let mut g = GrammarBuilder::new("cr_v9_inl3")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("h1", vec!["a"])
        .rule("h2", vec!["b"])
        .rule("start", vec!["h1", "h2", "c"])
        .inline("h1")
        .inline("h2")
        .start("start")
        .build();
    let (_pt, compressed) = compress_pipeline(&mut g);
    assert!(!compressed.action_table.data.is_empty());
}

// ═══════════════════════════════════════════════════════════════════════════
// 12. Compress handles grammar with extras
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn cr_v9_extras_compresses() {
    let mut g = GrammarBuilder::new("cr_v9_ext1")
        .token("a", "a")
        .token("ws", r"\s+")
        .rule("start", vec!["a"])
        .extra("ws")
        .start("start")
        .build();
    let (_pt, compressed) = compress_pipeline(&mut g);
    assert!(!compressed.action_table.data.is_empty());
}

#[test]
fn cr_v9_extras_preserves_states() {
    let mut g = GrammarBuilder::new("cr_v9_ext2")
        .token("a", "a")
        .token("b", "b")
        .token("ws", r"\s+")
        .rule("start", vec!["a", "b"])
        .extra("ws")
        .start("start")
        .build();
    let (pt, compressed) = compress_pipeline(&mut g);
    assert_eq!(
        compressed.action_table.row_offsets.len(),
        pt.state_count + 1
    );
}

#[test]
fn cr_v9_multiple_extras() {
    let mut g = GrammarBuilder::new("cr_v9_ext3")
        .token("a", "a")
        .token("ws", r"\s+")
        .token("nl", r"\n")
        .rule("start", vec!["a"])
        .extra("ws")
        .extra("nl")
        .start("start")
        .build();
    let (_pt, compressed) = compress_pipeline(&mut g);
    assert!(!compressed.action_table.data.is_empty());
}

// ═══════════════════════════════════════════════════════════════════════════
// 13. Compress handles grammar with externals
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn cr_v9_externals_compresses() {
    let mut g = GrammarBuilder::new("cr_v9_extn1")
        .token("a", "a")
        .external("ext_tok")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let (_pt, compressed) = compress_pipeline(&mut g);
    assert!(!compressed.action_table.data.is_empty());
}

#[test]
fn cr_v9_externals_preserves_state_count() {
    let mut g = GrammarBuilder::new("cr_v9_extn2")
        .token("a", "a")
        .token("b", "b")
        .external("scanner_tok")
        .rule("start", vec!["a", "b"])
        .start("start")
        .build();
    let (pt, compressed) = compress_pipeline(&mut g);
    assert_eq!(
        compressed.action_table.row_offsets.len(),
        pt.state_count + 1
    );
}

#[test]
fn cr_v9_multiple_externals() {
    let mut g = GrammarBuilder::new("cr_v9_extn3")
        .token("a", "a")
        .external("ext1")
        .external("ext2")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let (_pt, compressed) = compress_pipeline(&mut g);
    assert!(!compressed.action_table.data.is_empty());
}

// ═══════════════════════════════════════════════════════════════════════════
// 14. Compress handles grammar with conflicts
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn cr_v9_conflicts_compresses() {
    let mut g = GrammarBuilder::new("cr_v9_conf1")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["item"])
        .rule("item", vec!["a"])
        .rule("item", vec!["b"])
        .start("start")
        .build();
    // Add conflict declaration post-build
    let syms: Vec<_> = g.find_symbol_by_name("item").into_iter().collect();
    if !syms.is_empty() {
        g.conflicts.push(ConflictDeclaration {
            symbols: syms,
            resolution: ConflictResolution::GLR,
        });
    }
    let (_pt, compressed) = compress_pipeline(&mut g);
    assert!(!compressed.action_table.data.is_empty());
}

#[test]
fn cr_v9_conflicts_deterministic() {
    let build = || {
        let mut g = GrammarBuilder::new("cr_v9_conf2")
            .token("a", "a")
            .rule("start", vec!["a"])
            .start("start")
            .build();
        let syms: Vec<_> = g.find_symbol_by_name("start").into_iter().collect();
        if !syms.is_empty() {
            g.conflicts.push(ConflictDeclaration {
                symbols: syms,
                resolution: ConflictResolution::GLR,
            });
        }
        let mut g2 = g;
        compress_pipeline(&mut g2)
    };
    let (_, c1) = build();
    let (_, c2) = build();
    assert_eq!(c1.action_table.data.len(), c2.action_table.data.len());
}

#[test]
fn cr_v9_conflicts_preserves_states() {
    let mut g = GrammarBuilder::new("cr_v9_conf3")
        .token("x", "x")
        .token("y", "y")
        .rule("start", vec!["x"])
        .rule("start", vec!["y"])
        .start("start")
        .build();
    let syms: Vec<_> = g.find_symbol_by_name("start").into_iter().collect();
    if !syms.is_empty() {
        g.conflicts.push(ConflictDeclaration {
            symbols: syms,
            resolution: ConflictResolution::GLR,
        });
    }
    let (pt, compressed) = compress_pipeline(&mut g);
    assert_eq!(
        compressed.action_table.row_offsets.len(),
        pt.state_count + 1
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// 15. Compress handles grammar with alternatives
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn cr_v9_alternatives_three() {
    let mut g = alternatives_grammar("cr_v9_alt1", 3);
    let (_pt, compressed) = compress_pipeline(&mut g);
    assert!(!compressed.action_table.data.is_empty());
}

#[test]
fn cr_v9_alternatives_six() {
    let mut g = alternatives_grammar("cr_v9_alt2", 6);
    let (_pt, compressed) = compress_pipeline(&mut g);
    assert!(!compressed.action_table.data.is_empty());
}

#[test]
fn cr_v9_alternatives_ten() {
    let mut g = alternatives_grammar("cr_v9_alt3", 10);
    let (_pt, compressed) = compress_pipeline(&mut g);
    assert!(!compressed.action_table.data.is_empty());
}

#[test]
fn cr_v9_alternatives_scaling() {
    let mut g3 = alternatives_grammar("cr_v9_alt4a", 3);
    let (_, c3) = compress_pipeline(&mut g3);
    let mut g8 = alternatives_grammar("cr_v9_alt4b", 8);
    let (_, c8) = compress_pipeline(&mut g8);
    assert!(
        c8.action_table.data.len() > c3.action_table.data.len(),
        "more alternatives should produce more action entries"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// 16. Compress handles many-token grammar
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn cr_v9_many_tokens_five() {
    let mut g = many_token_grammar("cr_v9_mt1", 5);
    let (_pt, compressed) = compress_pipeline(&mut g);
    assert!(!compressed.action_table.data.is_empty());
}

#[test]
fn cr_v9_many_tokens_fifteen() {
    let mut g = many_token_grammar("cr_v9_mt2", 15);
    let (_pt, compressed) = compress_pipeline(&mut g);
    assert!(!compressed.action_table.data.is_empty());
}

#[test]
fn cr_v9_many_tokens_twenty() {
    let mut g = many_token_grammar("cr_v9_mt3", 20);
    let (pt, compressed) = compress_pipeline(&mut g);
    assert!(
        pt.symbol_count > 10,
        "many-token grammar should have many symbols"
    );
    assert!(!compressed.action_table.data.is_empty());
}

#[test]
fn cr_v9_many_tokens_preserves_states() {
    let mut g = many_token_grammar("cr_v9_mt4", 12);
    let (pt, compressed) = compress_pipeline(&mut g);
    assert_eq!(
        compressed.action_table.row_offsets.len(),
        pt.state_count + 1
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// 17. Compress handles many-rule grammar
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn cr_v9_many_rules_five() {
    let mut g = many_rule_grammar("cr_v9_mr1", 5);
    let (_pt, compressed) = compress_pipeline(&mut g);
    assert!(!compressed.action_table.data.is_empty());
}

#[test]
fn cr_v9_many_rules_ten() {
    let mut g = many_rule_grammar("cr_v9_mr2", 10);
    let (_pt, compressed) = compress_pipeline(&mut g);
    assert!(!compressed.goto_table.data.is_empty());
}

#[test]
fn cr_v9_many_rules_fifteen() {
    let mut g = many_rule_grammar("cr_v9_mr3", 15);
    let (pt, compressed) = compress_pipeline(&mut g);
    assert_eq!(
        compressed.action_table.row_offsets.len(),
        pt.state_count + 1
    );
}

#[test]
fn cr_v9_many_rules_scaling_goto() {
    let mut g5 = many_rule_grammar("cr_v9_mr4a", 5);
    let (_, c5) = compress_pipeline(&mut g5);
    let mut g12 = many_rule_grammar("cr_v9_mr4b", 12);
    let (_, c12) = compress_pipeline(&mut g12);
    assert!(
        c12.goto_table.data.len() > c5.goto_table.data.len(),
        "more rules should produce more goto entries"
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// 18. Compress handles deep nesting
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn cr_v9_deep_nesting_three() {
    let mut g = deep_nesting_grammar("cr_v9_dn1", 3);
    let (_pt, compressed) = compress_pipeline(&mut g);
    assert!(!compressed.action_table.data.is_empty());
}

#[test]
fn cr_v9_deep_nesting_eight() {
    let mut g = deep_nesting_grammar("cr_v9_dn2", 8);
    let (_pt, compressed) = compress_pipeline(&mut g);
    assert!(!compressed.goto_table.data.is_empty());
}

#[test]
fn cr_v9_deep_nesting_twelve() {
    let mut g = deep_nesting_grammar("cr_v9_dn3", 12);
    let (pt, compressed) = compress_pipeline(&mut g);
    assert_eq!(
        compressed.action_table.row_offsets.len(),
        pt.state_count + 1
    );
}

#[test]
fn cr_v9_deep_nesting_preserves_symbols() {
    let mut g = deep_nesting_grammar("cr_v9_dn4", 6);
    let (pt, _) = compress_pipeline(&mut g);
    let cpt = CompressedParseTable::from_parse_table(&pt);
    assert_eq!(cpt.symbol_count(), pt.symbol_count);
    assert_eq!(cpt.state_count(), pt.state_count);
}

// ═══════════════════════════════════════════════════════════════════════════
// 19. Compress handles recursive grammar
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn cr_v9_left_recursive_compresses() {
    let mut g = left_recursive_grammar("cr_v9_rec1");
    let (_pt, compressed) = compress_pipeline(&mut g);
    assert!(!compressed.action_table.data.is_empty());
}

#[test]
fn cr_v9_right_recursive_compresses() {
    let mut g = right_recursive_grammar("cr_v9_rec2");
    let (_pt, compressed) = compress_pipeline(&mut g);
    assert!(!compressed.action_table.data.is_empty());
}

#[test]
fn cr_v9_left_recursive_preserves_states() {
    let mut g = left_recursive_grammar("cr_v9_rec3");
    let (pt, compressed) = compress_pipeline(&mut g);
    assert_eq!(
        compressed.action_table.row_offsets.len(),
        pt.state_count + 1
    );
}

#[test]
fn cr_v9_right_recursive_preserves_symbols() {
    let mut g = right_recursive_grammar("cr_v9_rec4");
    let (pt, _) = compress_pipeline(&mut g);
    let cpt = CompressedParseTable::from_parse_table(&pt);
    assert_eq!(cpt.symbol_count(), pt.symbol_count);
}

#[test]
fn cr_v9_left_recursive_deterministic() {
    let mut g1 = left_recursive_grammar("cr_v9_rec5a");
    let (_, c1) = compress_pipeline(&mut g1);
    let mut g2 = left_recursive_grammar("cr_v9_rec5b");
    let (_, c2) = compress_pipeline(&mut g2);
    assert_eq!(c1.action_table.data.len(), c2.action_table.data.len());
    assert_eq!(c1.goto_table.data.len(), c2.goto_table.data.len());
}

#[test]
fn cr_v9_recursive_static_gen() {
    let (g, pt) = make_pipeline(left_recursive_grammar("cr_v9_rec6"));
    let code = StaticLanguageGenerator::new(g, pt)
        .generate_language_code()
        .to_string();
    assert!(!code.is_empty());
}

// ═══════════════════════════════════════════════════════════════════════════
// 20. Compress from_parse_table → valid compressor
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn cr_v9_from_parse_table_minimal() {
    let mut g = minimal_grammar("cr_v9_fpt1");
    let (pt, _) = compress_pipeline(&mut g);
    let cpt = CompressedParseTable::from_parse_table(&pt);
    assert!(cpt.symbol_count() > 0);
    assert!(cpt.state_count() > 0);
}

#[test]
fn cr_v9_from_parse_table_arithmetic() {
    let mut g = arithmetic_grammar("cr_v9_fpt2");
    let (pt, _) = compress_pipeline(&mut g);
    let cpt = CompressedParseTable::from_parse_table(&pt);
    assert!(cpt.symbol_count() > 0);
    assert!(cpt.state_count() > 0);
}

#[test]
fn cr_v9_from_parse_table_matches_state_count() {
    let mut g = alternatives_grammar("cr_v9_fpt3", 4);
    let (pt, _) = compress_pipeline(&mut g);
    let cpt = CompressedParseTable::from_parse_table(&pt);
    assert_eq!(cpt.state_count(), pt.state_count);
}

#[test]
fn cr_v9_from_parse_table_matches_symbol_count() {
    let mut g = many_token_grammar("cr_v9_fpt4", 7);
    let (pt, _) = compress_pipeline(&mut g);
    let cpt = CompressedParseTable::from_parse_table(&pt);
    assert_eq!(cpt.symbol_count(), pt.symbol_count);
}

// ═══════════════════════════════════════════════════════════════════════════
// Additional coverage: validate, node types, compress_tables, edge cases
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn cr_v9_validate_minimal() {
    let mut g = minimal_grammar("cr_v9_val1");
    let (pt, compressed) = compress_pipeline(&mut g);
    compressed.validate(&pt).expect("validation should pass");
}

#[test]
fn cr_v9_validate_arithmetic() {
    let mut g = arithmetic_grammar("cr_v9_val2");
    let (pt, compressed) = compress_pipeline(&mut g);
    compressed.validate(&pt).expect("validation should pass");
}

#[test]
fn cr_v9_validate_many_tokens() {
    let mut g = many_token_grammar("cr_v9_val3", 10);
    let (pt, compressed) = compress_pipeline(&mut g);
    compressed.validate(&pt).expect("validation should pass");
}

#[test]
fn cr_v9_node_types_valid_json() {
    let (g, pt) = make_pipeline(minimal_grammar("cr_v9_nt1"));
    let slg = StaticLanguageGenerator::new(g, pt);
    let json_str = slg.generate_node_types();
    let parsed: serde_json::Value =
        serde_json::from_str(&json_str).expect("node types must be valid JSON");
    assert!(parsed.is_array());
}

#[test]
fn cr_v9_node_types_arithmetic_valid_json() {
    let (g, pt) = make_pipeline(arithmetic_grammar("cr_v9_nt2"));
    let slg = StaticLanguageGenerator::new(g, pt);
    let json_str = slg.generate_node_types();
    let parsed: serde_json::Value =
        serde_json::from_str(&json_str).expect("node types must be valid JSON");
    assert!(parsed.is_array());
}

#[test]
fn cr_v9_compress_tables_on_generator() {
    let (g, pt) = make_pipeline(alternatives_grammar("cr_v9_ct1", 4));
    let mut slg = StaticLanguageGenerator::new(g, pt);
    slg.compress_tables()
        .expect("compress_tables should succeed");
    assert!(slg.compressed_tables.is_some());
}

#[test]
fn cr_v9_compress_tables_then_code() {
    let (g, pt) = make_pipeline(many_rule_grammar("cr_v9_ct2", 4));
    let mut slg = StaticLanguageGenerator::new(g, pt);
    slg.compress_tables()
        .expect("compress_tables should succeed");
    let code = slg.generate_language_code().to_string();
    assert!(!code.is_empty());
}

#[test]
fn cr_v9_set_start_can_be_empty() {
    let (g, pt) = make_pipeline(minimal_grammar("cr_v9_sce1"));
    let mut slg = StaticLanguageGenerator::new(g, pt);
    slg.set_start_can_be_empty(true);
    assert!(slg.start_can_be_empty);
    slg.compress_tables()
        .expect("compress_tables should succeed");
    assert!(slg.compressed_tables.is_some());
}

#[test]
fn cr_v9_compressor_default() {
    let c1 = TableCompressor::new();
    let c2 = TableCompressor::default();
    // Both should produce the same results on identical input
    let mut g1 = minimal_grammar("cr_v9_def1a");
    let ff1 = FirstFollowSets::compute_normalized(&mut g1).unwrap();
    let pt1 = build_lr1_automaton(&g1, &ff1).unwrap();
    let ti1 = collect_token_indices(&g1, &pt1);
    let se1 = eof_accepts_or_reduces(&pt1);
    let r1 = c1.compress(&pt1, &ti1, se1).unwrap();
    let r2 = c2.compress(&pt1, &ti1, se1).unwrap();
    assert_eq!(r1.action_table.data.len(), r2.action_table.data.len());
    assert_eq!(r1.goto_table.data.len(), r2.goto_table.data.len());
}

#[test]
fn cr_v9_row_offsets_monotonic_action() {
    let mut g = arithmetic_grammar("cr_v9_mono1");
    let (_pt, compressed) = compress_pipeline(&mut g);
    for pair in compressed.action_table.row_offsets.windows(2) {
        assert!(
            pair[0] <= pair[1],
            "action row offsets must be monotonically non-decreasing"
        );
    }
}

#[test]
fn cr_v9_row_offsets_monotonic_goto() {
    let mut g = arithmetic_grammar("cr_v9_mono2");
    let (_pt, compressed) = compress_pipeline(&mut g);
    for pair in compressed.goto_table.row_offsets.windows(2) {
        assert!(
            pair[0] <= pair[1],
            "goto row offsets must be monotonically non-decreasing"
        );
    }
}

#[test]
fn cr_v9_action_offsets_within_data_bounds() {
    let mut g = many_token_grammar("cr_v9_bnd1", 8);
    let (_pt, compressed) = compress_pipeline(&mut g);
    let data_len = compressed.action_table.data.len();
    for &offset in &compressed.action_table.row_offsets {
        assert!(
            usize::from(offset) <= data_len,
            "action row offset {offset} exceeds data length {data_len}"
        );
    }
}

#[test]
fn cr_v9_goto_offsets_within_data_bounds() {
    let mut g = many_rule_grammar("cr_v9_bnd2", 8);
    let (_pt, compressed) = compress_pipeline(&mut g);
    let data_len = compressed.goto_table.data.len();
    for &offset in &compressed.goto_table.row_offsets {
        assert!(
            usize::from(offset) <= data_len,
            "goto row offset {offset} exceeds data length {data_len}"
        );
    }
}

#[test]
fn cr_v9_extras_and_inline_combined() {
    let mut g = GrammarBuilder::new("cr_v9_combo1")
        .token("a", "a")
        .token("b", "b")
        .token("ws", r"\s+")
        .rule("inner", vec!["a"])
        .rule("start", vec!["inner", "b"])
        .inline("inner")
        .extra("ws")
        .start("start")
        .build();
    let (_pt, compressed) = compress_pipeline(&mut g);
    assert!(!compressed.action_table.data.is_empty());
}

#[test]
fn cr_v9_externals_and_extras_combined() {
    let mut g = GrammarBuilder::new("cr_v9_combo2")
        .token("a", "a")
        .token("ws", r"\s+")
        .external("ext_tok")
        .rule("start", vec!["a"])
        .extra("ws")
        .start("start")
        .build();
    let (_pt, compressed) = compress_pipeline(&mut g);
    assert!(!compressed.action_table.data.is_empty());
}

#[test]
fn cr_v9_two_token_sequence_compresses() {
    let mut g = GrammarBuilder::new("cr_v9_seq1")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a", "b"])
        .start("start")
        .build();
    let (pt, compressed) = compress_pipeline(&mut g);
    assert!(!compressed.action_table.data.is_empty());
    assert_eq!(
        compressed.action_table.row_offsets.len(),
        pt.state_count + 1
    );
}

#[test]
fn cr_v9_three_token_sequence_compresses() {
    let mut g = GrammarBuilder::new("cr_v9_seq2")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("start", vec!["a", "b", "c"])
        .start("start")
        .build();
    let (_pt, compressed) = compress_pipeline(&mut g);
    assert!(!compressed.action_table.data.is_empty());
}

#[test]
fn cr_v9_static_gen_deep_chain() {
    let (g, pt) = make_pipeline(deep_nesting_grammar("cr_v9_sgd1", 5));
    let code = StaticLanguageGenerator::new(g, pt)
        .generate_language_code()
        .to_string();
    assert!(!code.is_empty());
    assert!(
        code.contains("cr_v9_sgd1"),
        "generated code should reference grammar name"
    );
}

#[test]
fn cr_v9_static_gen_many_tokens() {
    let (g, pt) = make_pipeline(many_token_grammar("cr_v9_sgm1", 6));
    let code = StaticLanguageGenerator::new(g, pt)
        .generate_language_code()
        .to_string();
    assert!(!code.is_empty());
}

#[test]
fn cr_v9_cpt_from_recursive() {
    let mut g = left_recursive_grammar("cr_v9_cptl1");
    let (pt, _) = compress_pipeline(&mut g);
    let cpt = CompressedParseTable::from_parse_table(&pt);
    assert_eq!(cpt.state_count(), pt.state_count);
    assert_eq!(cpt.symbol_count(), pt.symbol_count);
}

#[test]
fn cr_v9_cpt_from_deep_nesting() {
    let mut g = deep_nesting_grammar("cr_v9_cptd1", 10);
    let (pt, _) = compress_pipeline(&mut g);
    let cpt = CompressedParseTable::from_parse_table(&pt);
    assert_eq!(cpt.state_count(), pt.state_count);
    assert_eq!(cpt.symbol_count(), pt.symbol_count);
}

#[test]
fn cr_v9_small_table_threshold_positive() {
    let mut g = minimal_grammar("cr_v9_thr1");
    let (_, compressed) = compress_pipeline(&mut g);
    assert!(
        compressed.small_table_threshold > 0,
        "threshold should be positive"
    );
}

#[test]
fn cr_v9_action_data_entries_non_negative_symbols() {
    let mut g = arithmetic_grammar("cr_v9_nns1");
    let (_, compressed) = compress_pipeline(&mut g);
    for entry in &compressed.action_table.data {
        // symbol is u16, always >= 0; just verify entries exist
        let _ = entry.symbol;
    }
    assert!(!compressed.action_table.data.is_empty());
}

#[test]
fn cr_v9_supertype_compresses() {
    let mut g = GrammarBuilder::new("cr_v9_sup1")
        .token("a", "a")
        .token("b", "b")
        .rule("item", vec!["a"])
        .rule("item", vec!["b"])
        .rule("start", vec!["item"])
        .supertype("item")
        .start("start")
        .build();
    let (_pt, compressed) = compress_pipeline(&mut g);
    assert!(!compressed.action_table.data.is_empty());
}
