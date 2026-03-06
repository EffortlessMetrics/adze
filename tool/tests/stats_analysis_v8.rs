//! Comprehensive tests for `BuildStats` analysis and validation in adze-tool.
//!
//! 84 tests across 20 categories covering every aspect of BuildStats:
//! invariants, scaling, determinism, traits, grammar complexity, and more.

use adze_ir::builder::GrammarBuilder;
use adze_ir::{Associativity, Grammar};
use adze_tool::pure_rust_builder::{BuildOptions, BuildResult, BuildStats, build_parser};
use tempfile::TempDir;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn tmp_opts() -> (TempDir, BuildOptions) {
    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().into(),
        emit_artifacts: false,
        compress_tables: false,
    };
    (dir, opts)
}

fn do_build(g: Grammar) -> BuildResult {
    let (_dir, opts) = tmp_opts();
    build_parser(g, opts).expect("build should succeed")
}

fn stats(g: Grammar) -> BuildStats {
    do_build(g).build_stats
}

/// Minimal grammar: source_file -> NUMBER
fn minimal(name: &str) -> Grammar {
    GrammarBuilder::new(name)
        .token("NUMBER", r"\d+")
        .rule("source_file", vec!["NUMBER"])
        .start("source_file")
        .build()
}

/// Two-token grammar: source_file -> NUMBER | IDENT
fn two_tok(name: &str) -> Grammar {
    GrammarBuilder::new(name)
        .token("NUMBER", r"\d+")
        .token("IDENT", r"[a-z]+")
        .rule("source_file", vec!["NUMBER"])
        .rule("source_file", vec!["IDENT"])
        .start("source_file")
        .build()
}

/// Three-token grammar: source_file -> NUMBER | IDENT | STRING
fn three_tok(name: &str) -> Grammar {
    GrammarBuilder::new(name)
        .token("NUMBER", r"\d+")
        .token("IDENT", r"[a-z]+")
        .token("STRING", r#""[^"]*""#)
        .rule("source_file", vec!["NUMBER"])
        .rule("source_file", vec!["IDENT"])
        .rule("source_file", vec!["STRING"])
        .start("source_file")
        .build()
}

/// Arithmetic: expr -> NUMBER | expr PLUS expr (left-assoc)
fn arith_left(name: &str) -> Grammar {
    GrammarBuilder::new(name)
        .token("NUMBER", r"\d+")
        .token("PLUS", "+")
        .rule("source_file", vec!["NUMBER"])
        .rule_with_precedence(
            "source_file",
            vec!["source_file", "PLUS", "source_file"],
            1,
            Associativity::Left,
        )
        .start("source_file")
        .build()
}

/// Arithmetic: expr -> NUMBER | expr PLUS expr (right-assoc)
fn arith_right(name: &str) -> Grammar {
    GrammarBuilder::new(name)
        .token("NUMBER", r"\d+")
        .token("PLUS", "+")
        .rule("source_file", vec!["NUMBER"])
        .rule_with_precedence(
            "source_file",
            vec!["source_file", "PLUS", "source_file"],
            1,
            Associativity::Right,
        )
        .start("source_file")
        .build()
}

/// Multi-operator: expr -> NUMBER | expr PLUS expr | expr STAR expr
fn multi_op(name: &str) -> Grammar {
    GrammarBuilder::new(name)
        .token("NUMBER", r"\d+")
        .token("PLUS", "+")
        .token("STAR", "*")
        .rule("source_file", vec!["NUMBER"])
        .rule_with_precedence(
            "source_file",
            vec!["source_file", "PLUS", "source_file"],
            1,
            Associativity::Left,
        )
        .rule_with_precedence(
            "source_file",
            vec!["source_file", "STAR", "source_file"],
            2,
            Associativity::Left,
        )
        .start("source_file")
        .build()
}

/// Three-operator: +, *, -
fn three_op(name: &str) -> Grammar {
    GrammarBuilder::new(name)
        .token("NUMBER", r"\d+")
        .token("PLUS", "+")
        .token("STAR", "*")
        .token("MINUS", "-")
        .rule("source_file", vec!["NUMBER"])
        .rule_with_precedence(
            "source_file",
            vec!["source_file", "PLUS", "source_file"],
            1,
            Associativity::Left,
        )
        .rule_with_precedence(
            "source_file",
            vec!["source_file", "STAR", "source_file"],
            2,
            Associativity::Left,
        )
        .rule_with_precedence(
            "source_file",
            vec!["source_file", "MINUS", "source_file"],
            1,
            Associativity::Left,
        )
        .start("source_file")
        .build()
}

/// N-alternative grammar: source_file -> TOK_0 | TOK_1 | ... | TOK_{n-1}
fn n_alt(name: &str, n: usize) -> Grammar {
    let mut b = GrammarBuilder::new(name);
    for i in 0..n {
        let tok_name = format!("TOK_{i}");
        let pattern = format!("t{i}");
        b = b.token(
            Box::leak(tok_name.clone().into_boxed_str()),
            Box::leak(pattern.into_boxed_str()),
        );
        b = b.rule("source_file", vec![Box::leak(tok_name.into_boxed_str())]);
    }
    b.start("source_file").build()
}

/// Sequence grammar: source_file -> TOK_0 TOK_1 ... TOK_{n-1}
fn seq(name: &str, n: usize) -> Grammar {
    let mut b = GrammarBuilder::new(name);
    let mut rhs_names: Vec<&'static str> = Vec::new();
    for i in 0..n {
        let tok_name = format!("TOK_{i}");
        let pattern = format!("s{i}");
        b = b.token(
            Box::leak(tok_name.clone().into_boxed_str()),
            Box::leak(pattern.into_boxed_str()),
        );
        rhs_names.push(Box::leak(tok_name.into_boxed_str()));
    }
    b = b.rule("source_file", rhs_names);
    b.start("source_file").build()
}

/// Grammar with an intermediate non-terminal (two-level)
fn two_level(name: &str) -> Grammar {
    GrammarBuilder::new(name)
        .token("NUMBER", r"\d+")
        .rule("atom", vec!["NUMBER"])
        .rule("source_file", vec!["atom"])
        .start("source_file")
        .build()
}

/// Grammar with extras (whitespace)
fn with_extras(name: &str) -> Grammar {
    GrammarBuilder::new(name)
        .token("NUMBER", r"\d+")
        .token("WHITESPACE", r"[ \t]+")
        .extra("WHITESPACE")
        .rule("source_file", vec!["NUMBER"])
        .start("source_file")
        .build()
}

/// Grammar with inline rule
fn with_inline(name: &str) -> Grammar {
    GrammarBuilder::new(name)
        .token("NUMBER", r"\d+")
        .rule("helper", vec!["NUMBER"])
        .rule("source_file", vec!["helper"])
        .inline("helper")
        .start("source_file")
        .build()
}

/// Grammar with precedence declaration
fn with_prec_decl(name: &str) -> Grammar {
    GrammarBuilder::new(name)
        .token("NUMBER", r"\d+")
        .token("PLUS", "+")
        .token("STAR", "*")
        .precedence(1, Associativity::Left, vec!["PLUS"])
        .precedence(2, Associativity::Left, vec!["STAR"])
        .rule("source_file", vec!["NUMBER"])
        .rule_with_precedence(
            "source_file",
            vec!["source_file", "PLUS", "source_file"],
            1,
            Associativity::Left,
        )
        .rule_with_precedence(
            "source_file",
            vec!["source_file", "STAR", "source_file"],
            2,
            Associativity::Left,
        )
        .start("source_file")
        .build()
}

/// Ambiguous grammar: source_file -> NUMBER | source_file source_file (no precedence)
fn ambiguous(name: &str) -> Grammar {
    GrammarBuilder::new(name)
        .token("NUMBER", r"\d+")
        .rule("source_file", vec!["NUMBER"])
        .rule("source_file", vec!["source_file", "source_file"])
        .start("source_file")
        .build()
}

// =========================================================================
// 1. state_count > 0 for any grammar (4 tests)
// =========================================================================

#[test]
fn sa_v8_state_count_positive_minimal() {
    let s = stats(minimal("sa_v8_sc_pos_min"));
    assert!(s.state_count > 0, "state_count must be positive");
}

#[test]
fn sa_v8_state_count_positive_two_tok() {
    let s = stats(two_tok("sa_v8_sc_pos_2t"));
    assert!(s.state_count > 0);
}

#[test]
fn sa_v8_state_count_positive_arith() {
    let s = stats(arith_left("sa_v8_sc_pos_ar"));
    assert!(s.state_count > 0);
}

#[test]
fn sa_v8_state_count_positive_multi_op() {
    let s = stats(multi_op("sa_v8_sc_pos_mo"));
    assert!(s.state_count > 0);
}

// =========================================================================
// 2. symbol_count > 0 for any grammar (4 tests)
// =========================================================================

#[test]
fn sa_v8_symbol_count_positive_minimal() {
    let s = stats(minimal("sa_v8_sym_pos_min"));
    assert!(s.symbol_count > 0, "symbol_count must be positive");
}

#[test]
fn sa_v8_symbol_count_positive_two_tok() {
    let s = stats(two_tok("sa_v8_sym_pos_2t"));
    assert!(s.symbol_count > 0);
}

#[test]
fn sa_v8_symbol_count_positive_arith() {
    let s = stats(arith_left("sa_v8_sym_pos_ar"));
    assert!(s.symbol_count > 0);
}

#[test]
fn sa_v8_symbol_count_positive_three_op() {
    let s = stats(three_op("sa_v8_sym_pos_3o"));
    assert!(s.symbol_count > 0);
}

// =========================================================================
// 3. conflict_cells >= 0 for any grammar (always true for usize; verify bounded) (4 tests)
// =========================================================================

#[test]
fn sa_v8_conflict_bounded_minimal() {
    let s = stats(minimal("sa_v8_cb_min"));
    assert!(s.conflict_cells <= s.state_count * s.symbol_count);
}

#[test]
fn sa_v8_conflict_bounded_arith() {
    let s = stats(arith_left("sa_v8_cb_ar"));
    assert!(s.conflict_cells <= s.state_count * s.symbol_count);
}

#[test]
fn sa_v8_conflict_bounded_multi_op() {
    let s = stats(multi_op("sa_v8_cb_mo"));
    assert!(s.conflict_cells <= s.state_count * s.symbol_count);
}

#[test]
fn sa_v8_conflict_bounded_n_alt() {
    let s = stats(n_alt("sa_v8_cb_na", 8));
    assert!(s.conflict_cells <= s.state_count * s.symbol_count);
}

// =========================================================================
// 4. Simple grammar → conflict_cells == 0 (4 tests)
// =========================================================================

#[test]
fn sa_v8_no_conflicts_minimal() {
    let s = stats(minimal("sa_v8_nc_min"));
    assert_eq!(
        s.conflict_cells, 0,
        "minimal grammar should be conflict-free"
    );
}

#[test]
fn sa_v8_no_conflicts_two_tok() {
    let s = stats(two_tok("sa_v8_nc_2t"));
    assert_eq!(
        s.conflict_cells, 0,
        "two-token alternatives should be conflict-free"
    );
}

#[test]
fn sa_v8_no_conflicts_three_tok() {
    let s = stats(three_tok("sa_v8_nc_3t"));
    assert_eq!(s.conflict_cells, 0);
}

#[test]
fn sa_v8_no_conflicts_sequence() {
    let s = stats(seq("sa_v8_nc_seq", 3));
    assert_eq!(s.conflict_cells, 0);
}

// =========================================================================
// 5. state_count >= 2 (start state + accept) (4 tests)
// =========================================================================

#[test]
fn sa_v8_at_least_two_states_minimal() {
    let s = stats(minimal("sa_v8_a2s_min"));
    assert!(
        s.state_count >= 2,
        "need at least start + accept, got {}",
        s.state_count
    );
}

#[test]
fn sa_v8_at_least_two_states_two_tok() {
    let s = stats(two_tok("sa_v8_a2s_2t"));
    assert!(s.state_count >= 2);
}

#[test]
fn sa_v8_at_least_two_states_arith() {
    let s = stats(arith_left("sa_v8_a2s_ar"));
    assert!(s.state_count >= 2);
}

#[test]
fn sa_v8_at_least_two_states_seq() {
    let s = stats(seq("sa_v8_a2s_seq", 4));
    assert!(s.state_count >= 2);
}

// =========================================================================
// 6. symbol_count >= num_tokens + 1 (EOF) (4 tests)
// =========================================================================

#[test]
fn sa_v8_symbols_cover_tokens_plus_eof_minimal() {
    // 1 token + EOF = at least 2
    let s = stats(minimal("sa_v8_scte_min"));
    assert!(
        s.symbol_count >= 2,
        "1 token + EOF → ≥2 symbols, got {}",
        s.symbol_count
    );
}

#[test]
fn sa_v8_symbols_cover_tokens_plus_eof_two_tok() {
    // 2 tokens + EOF = at least 3
    let s = stats(two_tok("sa_v8_scte_2t"));
    assert!(
        s.symbol_count >= 3,
        "2 tokens + EOF → ≥3 symbols, got {}",
        s.symbol_count
    );
}

#[test]
fn sa_v8_symbols_cover_tokens_plus_eof_three_tok() {
    let s = stats(three_tok("sa_v8_scte_3t"));
    assert!(
        s.symbol_count >= 4,
        "3 tokens + EOF → ≥4 symbols, got {}",
        s.symbol_count
    );
}

#[test]
fn sa_v8_symbols_cover_tokens_plus_eof_n_alt() {
    let s = stats(n_alt("sa_v8_scte_n10", 10));
    assert!(
        s.symbol_count >= 11,
        "10 tokens + EOF → ≥11 symbols, got {}",
        s.symbol_count
    );
}

// =========================================================================
// 7. More tokens → more symbols (4 tests)
// =========================================================================

#[test]
fn sa_v8_more_tokens_more_symbols_1_vs_2() {
    let s1 = stats(minimal("sa_v8_mt_1"));
    let s2 = stats(two_tok("sa_v8_mt_2"));
    assert!(
        s2.symbol_count >= s1.symbol_count,
        "{} < {}",
        s2.symbol_count,
        s1.symbol_count
    );
}

#[test]
fn sa_v8_more_tokens_more_symbols_2_vs_3() {
    let s2 = stats(two_tok("sa_v8_mt_2b"));
    let s3 = stats(three_tok("sa_v8_mt_3"));
    assert!(s3.symbol_count >= s2.symbol_count);
}

#[test]
fn sa_v8_more_tokens_more_symbols_5_vs_10() {
    let s5 = stats(n_alt("sa_v8_mt_5", 5));
    let s10 = stats(n_alt("sa_v8_mt_10", 10));
    assert!(s10.symbol_count > s5.symbol_count);
}

#[test]
fn sa_v8_more_tokens_more_symbols_3_vs_15() {
    let s3 = stats(n_alt("sa_v8_mt_3b", 3));
    let s15 = stats(n_alt("sa_v8_mt_15", 15));
    assert!(s15.symbol_count > s3.symbol_count);
}

// =========================================================================
// 8. More rules → potentially more states (4 tests)
// =========================================================================

#[test]
fn sa_v8_more_rules_more_states_minimal_vs_arith() {
    let s_min = stats(minimal("sa_v8_mr_min"));
    let s_ar = stats(arith_left("sa_v8_mr_ar"));
    assert!(
        s_ar.state_count >= s_min.state_count,
        "arith ({}) should have ≥ minimal ({}) states",
        s_ar.state_count,
        s_min.state_count,
    );
}

#[test]
fn sa_v8_more_rules_more_states_arith_vs_multi_op() {
    let s_ar = stats(arith_left("sa_v8_mr_ar2"));
    let s_mo = stats(multi_op("sa_v8_mr_mo"));
    assert!(s_mo.state_count >= s_ar.state_count);
}

#[test]
fn sa_v8_more_rules_more_states_multi_op_vs_three_op() {
    let s_mo = stats(multi_op("sa_v8_mr_mo2"));
    let s_3o = stats(three_op("sa_v8_mr_3o"));
    assert!(s_3o.state_count >= s_mo.state_count);
}

#[test]
fn sa_v8_more_rules_seq_grows() {
    let s2 = stats(seq("sa_v8_mr_s2", 2));
    let s6 = stats(seq("sa_v8_mr_s6", 6));
    assert!(s6.state_count >= s2.state_count);
}

// =========================================================================
// 9. Grammar with precedence → stats (4 tests)
// =========================================================================

#[test]
fn sa_v8_prec_state_count_positive() {
    let s = stats(with_prec_decl("sa_v8_pd_sc"));
    assert!(s.state_count > 0);
}

#[test]
fn sa_v8_prec_symbol_count_positive() {
    let s = stats(with_prec_decl("sa_v8_pd_sym"));
    assert!(s.symbol_count > 0);
}

#[test]
fn sa_v8_prec_at_least_two_states() {
    let s = stats(with_prec_decl("sa_v8_pd_a2"));
    assert!(s.state_count >= 2);
}

#[test]
fn sa_v8_prec_conflict_bounded() {
    let s = stats(with_prec_decl("sa_v8_pd_cb"));
    assert!(s.conflict_cells <= s.state_count * s.symbol_count);
}

// =========================================================================
// 10. Grammar with left assoc → stats (4 tests)
// =========================================================================

#[test]
fn sa_v8_left_assoc_state_positive() {
    let s = stats(arith_left("sa_v8_la_sc"));
    assert!(s.state_count > 0);
}

#[test]
fn sa_v8_left_assoc_symbol_positive() {
    let s = stats(arith_left("sa_v8_la_sym"));
    assert!(s.symbol_count > 0);
}

#[test]
fn sa_v8_left_assoc_at_least_three_states() {
    // Recursive grammar needs initial, shift, and reduce states
    let s = stats(arith_left("sa_v8_la_a3"));
    assert!(
        s.state_count >= 3,
        "left-assoc recursive grammar needs ≥3 states, got {}",
        s.state_count
    );
}

#[test]
fn sa_v8_left_assoc_bounded_conflicts() {
    let s = stats(arith_left("sa_v8_la_bc"));
    assert!(s.conflict_cells <= s.state_count * s.symbol_count);
}

// =========================================================================
// 11. Grammar with right assoc → stats (4 tests)
// =========================================================================

#[test]
fn sa_v8_right_assoc_state_positive() {
    let s = stats(arith_right("sa_v8_ra_sc"));
    assert!(s.state_count > 0);
}

#[test]
fn sa_v8_right_assoc_symbol_positive() {
    let s = stats(arith_right("sa_v8_ra_sym"));
    assert!(s.symbol_count > 0);
}

#[test]
fn sa_v8_right_assoc_at_least_three_states() {
    let s = stats(arith_right("sa_v8_ra_a3"));
    assert!(s.state_count >= 3);
}

#[test]
fn sa_v8_right_assoc_same_symbol_count_as_left() {
    let s_l = stats(arith_left("sa_v8_ra_cmp_l"));
    let s_r = stats(arith_right("sa_v8_ra_cmp_r"));
    // Same tokens → same symbol count
    assert_eq!(s_l.symbol_count, s_r.symbol_count);
}

// =========================================================================
// 12. Determinism: same grammar → same stats (4 tests)
// =========================================================================

#[test]
fn sa_v8_determinism_minimal() {
    let a = stats(minimal("sa_v8_det_min_a"));
    let b = stats(minimal("sa_v8_det_min_b"));
    assert_eq!(a.state_count, b.state_count);
    assert_eq!(a.symbol_count, b.symbol_count);
    assert_eq!(a.conflict_cells, b.conflict_cells);
}

#[test]
fn sa_v8_determinism_arith() {
    let a = stats(arith_left("sa_v8_det_ar_a"));
    let b = stats(arith_left("sa_v8_det_ar_b"));
    assert_eq!(a.state_count, b.state_count);
    assert_eq!(a.symbol_count, b.symbol_count);
    assert_eq!(a.conflict_cells, b.conflict_cells);
}

#[test]
fn sa_v8_determinism_multi_op() {
    let a = stats(multi_op("sa_v8_det_mo_a"));
    let b = stats(multi_op("sa_v8_det_mo_b"));
    assert_eq!(a.state_count, b.state_count);
    assert_eq!(a.symbol_count, b.symbol_count);
}

#[test]
fn sa_v8_determinism_n_alt() {
    let a = stats(n_alt("sa_v8_det_na_a", 7));
    let b = stats(n_alt("sa_v8_det_na_b", 7));
    assert_eq!(a.state_count, b.state_count);
    assert_eq!(a.symbol_count, b.symbol_count);
}

// =========================================================================
// 13. Different grammars → likely different stats (4 tests)
// =========================================================================

#[test]
fn sa_v8_different_minimal_vs_arith() {
    let a = stats(minimal("sa_v8_diff_min"));
    let b = stats(arith_left("sa_v8_diff_ar"));
    let same = a.state_count == b.state_count
        && a.symbol_count == b.symbol_count
        && a.conflict_cells == b.conflict_cells;
    assert!(!same, "minimal and arith should produce different stats");
}

#[test]
fn sa_v8_different_arith_vs_multi_op() {
    let a = stats(arith_left("sa_v8_diff_ar2"));
    let b = stats(multi_op("sa_v8_diff_mo"));
    // At least symbol_count should differ (different number of tokens)
    assert_ne!(a.symbol_count, b.symbol_count);
}

#[test]
fn sa_v8_different_n_alt_5_vs_10() {
    let a = stats(n_alt("sa_v8_diff_n5", 5));
    let b = stats(n_alt("sa_v8_diff_n10", 10));
    assert_ne!(a.symbol_count, b.symbol_count);
}

#[test]
fn sa_v8_different_minimal_vs_two_level() {
    let a = stats(minimal("sa_v8_diff_min2"));
    let b = stats(two_level("sa_v8_diff_2l"));
    // two_level has an extra non-terminal, so symbol_count should differ
    assert_ne!(a.symbol_count, b.symbol_count);
}

// =========================================================================
// 14. Stats Debug format readable (4 tests)
// =========================================================================

#[test]
fn sa_v8_debug_contains_state_count() {
    let s = stats(minimal("sa_v8_dbg_sc"));
    let dbg = format!("{s:?}");
    assert!(
        dbg.contains("state_count"),
        "Debug output should contain 'state_count': {dbg}"
    );
}

#[test]
fn sa_v8_debug_contains_symbol_count() {
    let s = stats(minimal("sa_v8_dbg_sym"));
    let dbg = format!("{s:?}");
    assert!(
        dbg.contains("symbol_count"),
        "Debug output should contain 'symbol_count': {dbg}"
    );
}

#[test]
fn sa_v8_debug_contains_conflict_cells() {
    let s = stats(minimal("sa_v8_dbg_cc"));
    let dbg = format!("{s:?}");
    assert!(
        dbg.contains("conflict_cells"),
        "Debug output should contain 'conflict_cells': {dbg}"
    );
}

#[test]
fn sa_v8_debug_not_empty() {
    let s = stats(arith_left("sa_v8_dbg_ne"));
    let dbg = format!("{s:?}");
    assert!(!dbg.is_empty());
}

// =========================================================================
// 15. Stats Clone independence (4 tests)
// =========================================================================

#[test]
fn sa_v8_clone_preserves_state_count() {
    let s = stats(minimal("sa_v8_cl_sc"));
    let c = s.clone();
    assert_eq!(s.state_count, c.state_count);
}

#[test]
fn sa_v8_clone_preserves_symbol_count() {
    let s = stats(arith_left("sa_v8_cl_sym"));
    let c = s.clone();
    assert_eq!(s.symbol_count, c.symbol_count);
}

#[test]
fn sa_v8_clone_preserves_conflict_cells() {
    let s = stats(multi_op("sa_v8_cl_cc"));
    let c = s.clone();
    assert_eq!(s.conflict_cells, c.conflict_cells);
}

#[test]
fn sa_v8_clone_all_fields_equal() {
    let s = stats(three_op("sa_v8_cl_all"));
    let c = s.clone();
    assert_eq!(s.state_count, c.state_count);
    assert_eq!(s.symbol_count, c.symbol_count);
    assert_eq!(s.conflict_cells, c.conflict_cells);
}

// =========================================================================
// 16. Arithmetic grammar → expected range of stats (4 tests)
// =========================================================================

#[test]
fn sa_v8_arith_states_in_range() {
    let s = stats(arith_left("sa_v8_ar_range_st"));
    // A simple left-recursive + grammar typically has 3-15 states
    assert!(s.state_count >= 3, "too few states: {}", s.state_count);
    assert!(s.state_count <= 50, "too many states: {}", s.state_count);
}

#[test]
fn sa_v8_arith_symbols_in_range() {
    let s = stats(arith_left("sa_v8_ar_range_sym"));
    // NUMBER, PLUS, source_file, EOF, and internal symbols
    assert!(s.symbol_count >= 3, "too few symbols: {}", s.symbol_count);
    assert!(s.symbol_count <= 30, "too many symbols: {}", s.symbol_count);
}

#[test]
fn sa_v8_multi_op_states_in_range() {
    let s = stats(multi_op("sa_v8_mo_range_st"));
    assert!(s.state_count >= 3);
    assert!(s.state_count <= 60);
}

#[test]
fn sa_v8_multi_op_symbols_more_than_arith() {
    let s_ar = stats(arith_left("sa_v8_mo_cmp_ar"));
    let s_mo = stats(multi_op("sa_v8_mo_cmp_mo"));
    assert!(s_mo.symbol_count > s_ar.symbol_count);
}

// =========================================================================
// 17. Grammar with conflicts → conflict_cells > 0 possible (4 tests)
// =========================================================================

#[test]
fn sa_v8_ambiguous_builds_ok() {
    // Ambiguous grammar should still build (GLR supports it)
    let s = stats(ambiguous("sa_v8_amb_ok"));
    assert!(s.state_count > 0);
}

#[test]
fn sa_v8_ambiguous_has_symbols() {
    let s = stats(ambiguous("sa_v8_amb_sym"));
    assert!(s.symbol_count > 0);
}

#[test]
fn sa_v8_ambiguous_conflicts_bounded() {
    let s = stats(ambiguous("sa_v8_amb_cb"));
    assert!(s.conflict_cells <= s.state_count * s.symbol_count);
}

#[test]
fn sa_v8_ambiguous_at_least_two_states() {
    let s = stats(ambiguous("sa_v8_amb_a2"));
    assert!(s.state_count >= 2);
}

// =========================================================================
// 18. Grammar with inline → stats comparison (4 tests)
// =========================================================================

#[test]
fn sa_v8_inline_builds_ok() {
    let s = stats(with_inline("sa_v8_inl_ok"));
    assert!(s.state_count > 0);
}

#[test]
fn sa_v8_inline_symbol_count_positive() {
    let s = stats(with_inline("sa_v8_inl_sym"));
    assert!(s.symbol_count > 0);
}

#[test]
fn sa_v8_inline_at_least_two_states() {
    let s = stats(with_inline("sa_v8_inl_a2"));
    assert!(s.state_count >= 2);
}

#[test]
fn sa_v8_inline_no_conflicts() {
    let s = stats(with_inline("sa_v8_inl_nc"));
    assert_eq!(s.conflict_cells, 0);
}

// =========================================================================
// 19. Grammar with extras → stats (4 tests)
// =========================================================================

#[test]
fn sa_v8_extras_builds_ok() {
    let s = stats(with_extras("sa_v8_ext_ok"));
    assert!(s.state_count > 0);
}

#[test]
fn sa_v8_extras_symbol_count_positive() {
    let s = stats(with_extras("sa_v8_ext_sym"));
    assert!(s.symbol_count > 0);
}

#[test]
fn sa_v8_extras_at_least_two_states() {
    let s = stats(with_extras("sa_v8_ext_a2"));
    assert!(s.state_count >= 2);
}

#[test]
fn sa_v8_extras_no_conflicts() {
    let s = stats(with_extras("sa_v8_ext_nc"));
    assert_eq!(s.conflict_cells, 0);
}

// =========================================================================
// 20. Stats fields are consistent and cross-validated (4 tests)
// =========================================================================

#[test]
fn sa_v8_consistency_conflict_le_total_cells() {
    let s = stats(multi_op("sa_v8_con_cle"));
    // conflict_cells can never exceed total action table cells
    assert!(s.conflict_cells <= s.state_count * s.symbol_count);
}

#[test]
fn sa_v8_consistency_two_level_has_more_symbols_than_minimal() {
    let s_min = stats(minimal("sa_v8_con_min"));
    let s_2l = stats(two_level("sa_v8_con_2l"));
    // two_level adds an extra non-terminal
    assert!(s_2l.symbol_count > s_min.symbol_count);
}

#[test]
fn sa_v8_consistency_extras_includes_whitespace_symbol() {
    let s_plain = stats(minimal("sa_v8_con_plain"));
    let s_extra = stats(with_extras("sa_v8_con_ext"));
    // extras token adds to the symbol count
    assert!(s_extra.symbol_count > s_plain.symbol_count);
}

#[test]
fn sa_v8_consistency_prec_decl_similar_to_inline_prec() {
    // Using precedence declaration vs inline precedence should yield same structure
    let s_decl = stats(with_prec_decl("sa_v8_con_decl"));
    let s_inline = stats(multi_op("sa_v8_con_inl"));
    // Both have same tokens (NUMBER, PLUS, STAR) → same symbol count
    assert_eq!(s_decl.symbol_count, s_inline.symbol_count);
}

// =========================================================================
// Additional tests to reach 84 total
// =========================================================================

#[test]
fn sa_v8_seq_no_conflicts() {
    let s = stats(seq("sa_v8_seq_nc", 5));
    assert_eq!(
        s.conflict_cells, 0,
        "sequence grammar should be conflict-free"
    );
}

#[test]
fn sa_v8_n_alt_no_conflicts() {
    let s = stats(n_alt("sa_v8_nalt_nc", 8));
    assert_eq!(
        s.conflict_cells, 0,
        "alternatives grammar should be conflict-free"
    );
}

#[test]
fn sa_v8_two_level_no_conflicts() {
    let s = stats(two_level("sa_v8_2l_nc"));
    assert_eq!(s.conflict_cells, 0);
}

#[test]
fn sa_v8_two_level_at_least_two_states() {
    let s = stats(two_level("sa_v8_2l_a2"));
    assert!(s.state_count >= 2);
}
