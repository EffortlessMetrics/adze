//! Property-based tests for the adze-tool build pipeline (v8).
//!
//! 85 tests across 16 categories:
//!   1.  prop_v8_build_ok_*        — any valid grammar → build succeeds (6)
//!   2.  prop_v8_parser_code_*     — parser_code non-empty (5)
//!   3.  prop_v8_node_types_*      — node_types_json non-empty (5)
//!   4.  prop_v8_state_count_*     — state_count > 0 (5)
//!   5.  prop_v8_symbol_count_*    — symbol_count > 0 (5)
//!   6.  prop_v8_conflict_*        — conflict_cells bounded (5)
//!   7.  prop_v8_det_*             — determinism: same input → same output (5)
//!   8.  prop_v8_diff_*            — different grammar → different code (5)
//!   9.  prop_v8_compress_*        — compress_tables flag doesn't crash (5)
//!   10. prop_v8_emit_*            — emit_artifacts flag doesn't crash (5)
//!   11. prop_v8_json_*            — node_types_json is valid JSON (5)
//!   12. prop_v8_symge_*           — symbol_count >= token count (5)
//!   13. prop_v8_scaling_*         — parser_code length scales with grammar (5)
//!   14. test_v8_pattern_*         — specific grammar patterns (7)
//!   15. test_v8_edge_*            — edge cases (6)
//!   16. test_v8_combo_*           — option combinations (6)

use adze_ir::Associativity;
use adze_ir::builder::GrammarBuilder;
use adze_tool::pure_rust_builder::{BuildOptions, BuildResult, build_parser};
use proptest::prelude::*;

// ===========================================================================
// Helpers
// ===========================================================================

fn test_opts() -> BuildOptions {
    BuildOptions {
        out_dir: "/tmp/proptest_pipeline_v8".to_string(),
        emit_artifacts: false,
        compress_tables: true,
    }
}

/// Build an n-alternative grammar: s -> t0 | t1 | … | t(n-1).
fn build_n_alts(name: &str, n: usize) -> BuildResult {
    let tok_names: Vec<String> = (0..n).map(|i| format!("t{i}")).collect();
    let tok_pats: Vec<String> = (0..n).map(|i| format!("p{i}")).collect();
    let mut b = GrammarBuilder::new(name);
    for (tname, tpat) in tok_names.iter().zip(tok_pats.iter()) {
        b = b.token(tname, tpat);
    }
    for tname in &tok_names {
        b = b.rule("s", vec![tname.as_str()]);
    }
    b = b.start("s");
    build_parser(b.build(), test_opts()).expect("build_n_alts failed")
}

/// Build a simple single-token grammar.
fn build_single(name: &str) -> BuildResult {
    build_parser(
        GrammarBuilder::new(name)
            .token("a", "a")
            .rule("s", vec!["a"])
            .start("s")
            .build(),
        test_opts(),
    )
    .expect("build_single failed")
}

/// Build a two-token alternative grammar.
fn build_two_alt(name: &str) -> BuildResult {
    build_parser(
        GrammarBuilder::new(name)
            .token("a", "a")
            .token("b", "b")
            .rule("s", vec!["a"])
            .rule("s", vec!["b"])
            .start("s")
            .build(),
        test_opts(),
    )
    .expect("build_two_alt failed")
}

/// Build a grammar with a precedence rule.
fn build_prec(name: &str, prec: i16, assoc: Associativity) -> BuildResult {
    build_parser(
        GrammarBuilder::new(name)
            .token("x", "x")
            .token("op", "op")
            .rule("expr", vec!["x"])
            .rule_with_precedence("expr", vec!["expr", "op", "expr"], prec, assoc)
            .start("expr")
            .build(),
        test_opts(),
    )
    .expect("build_prec failed")
}

/// Build a concatenation grammar with `n` tokens.
fn build_concat(name: &str, n: usize) -> BuildResult {
    let tok_names: Vec<String> = (0..n).map(|i| format!("t{i}")).collect();
    let tok_pats: Vec<String> = (0..n).map(|i| format!("q{i}")).collect();
    let rhs: Vec<&str> = tok_names.iter().map(|t| t.as_str()).collect();
    let mut b = GrammarBuilder::new(name);
    for (tname, tpat) in tok_names.iter().zip(tok_pats.iter()) {
        b = b.token(tname, tpat);
    }
    b = b.rule("s", rhs).start("s");
    build_parser(b.build(), test_opts()).expect("build_concat failed")
}

/// Build a chain grammar: s -> mid -> leaf -> token.
fn build_chain(name: &str) -> BuildResult {
    build_parser(
        GrammarBuilder::new(name)
            .token("x", "x")
            .rule("leaf", vec!["x"])
            .rule("mid", vec!["leaf"])
            .rule("s", vec!["mid"])
            .start("s")
            .build(),
        test_opts(),
    )
    .expect("build_chain failed")
}

// ===========================================================================
// Strategies
// ===========================================================================

fn arb_token_count() -> impl Strategy<Value = usize> {
    1usize..8
}

fn arb_rule_count() -> impl Strategy<Value = usize> {
    1usize..5
}

fn arb_bool() -> impl Strategy<Value = bool> {
    any::<bool>()
}

fn grammar_name_strategy() -> impl Strategy<Value = String> {
    "[a-z]{3,8}".prop_filter("avoid reserved words", |s| {
        !matches!(
            s.as_str(),
            "gen"
                | "do"
                | "self"
                | "type"
                | "fn"
                | "use"
                | "mod"
                | "pub"
                | "let"
                | "mut"
                | "ref"
                | "for"
                | "if"
                | "else"
                | "loop"
                | "while"
                | "match"
                | "impl"
                | "enum"
                | "struct"
                | "trait"
                | "where"
                | "async"
                | "await"
                | "dyn"
                | "move"
                | "return"
                | "break"
                | "continue"
                | "const"
                | "static"
                | "extern"
                | "crate"
                | "super"
                | "as"
                | "in"
                | "box"
                | "macro"
                | "try"
                | "yield"
                | "abstract"
                | "become"
                | "final"
                | "override"
                | "priv"
                | "typeof"
                | "unsized"
                | "virtual"
        )
    })
}

// ===========================================================================
// 1. Any valid grammar → build succeeds (5 properties)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(8))]

    #[test]
    fn prop_v8_build_ok_single(name in grammar_name_strategy()) {
        let r = build_parser(
            GrammarBuilder::new(&name)
                .token("a", "a")
                .rule("s", vec!["a"])
                .start("s")
                .build(),
            test_opts(),
        );
        prop_assert!(r.is_ok());
    }

    #[test]
    fn prop_v8_build_ok_two_alt(name in grammar_name_strategy()) {
        let r = build_parser(
            GrammarBuilder::new(&name)
                .token("a", "a")
                .token("b", "b")
                .rule("s", vec!["a"])
                .rule("s", vec!["b"])
                .start("s")
                .build(),
            test_opts(),
        );
        prop_assert!(r.is_ok());
    }

    #[test]
    fn prop_v8_build_ok_n_alts(n in arb_token_count()) {
        let name = format!("pp_v8_{}", n + 10);
        let tok_names: Vec<String> = (0..n).map(|i| format!("t{i}")).collect();
        let tok_pats: Vec<String> = (0..n).map(|i| format!("p{i}")).collect();
        let mut b = GrammarBuilder::new(&name);
        for (tname, tpat) in tok_names.iter().zip(tok_pats.iter()) {
            b = b.token(tname, tpat);
        }
        for tname in &tok_names {
            b = b.rule("s", vec![tname.as_str()]);
        }
        b = b.start("s");
        let r = build_parser(b.build(), test_opts());
        prop_assert!(r.is_ok());
    }

    #[test]
    fn prop_v8_build_ok_multi_rule(nr in arb_rule_count()) {
        let name = format!("pp_v8_{}", nr + 15);
        let tok_names: Vec<String> = (0..nr).map(|i| format!("t{i}")).collect();
        let tok_pats: Vec<String> = (0..nr).map(|i| format!("r{i}")).collect();
        let mut b = GrammarBuilder::new(&name);
        for (tname, tpat) in tok_names.iter().zip(tok_pats.iter()) {
            b = b.token(tname, tpat);
        }
        for tname in &tok_names {
            b = b.rule("s", vec![tname.as_str()]);
        }
        b = b.start("s");
        let r = build_parser(b.build(), test_opts());
        prop_assert!(r.is_ok());
    }

    #[test]
    fn prop_v8_build_ok_concat(n in arb_token_count()) {
        let name = format!("pp_v8_{}", n + 20);
        let tok_names: Vec<String> = (0..n).map(|i| format!("t{i}")).collect();
        let tok_pats: Vec<String> = (0..n).map(|i| format!("q{i}")).collect();
        let rhs: Vec<&str> = tok_names.iter().map(|t| t.as_str()).collect();
        let mut b = GrammarBuilder::new(&name);
        for (tname, tpat) in tok_names.iter().zip(tok_pats.iter()) {
            b = b.token(tname, tpat);
        }
        b = b.rule("s", rhs).start("s");
        let r = build_parser(b.build(), test_opts());
        prop_assert!(r.is_ok());
    }

    #[test]
    fn prop_v8_build_ok_prec(prec in -50i16..50i16) {
        let name = format!("pp_v8_{}", (prec + 100) as u16);
        let r = build_parser(
            GrammarBuilder::new(&name)
                .token("x", "x")
                .token("op", "op")
                .rule("expr", vec!["x"])
                .rule_with_precedence("expr", vec!["expr", "op", "expr"], prec, Associativity::Left)
                .start("expr")
                .build(),
            test_opts(),
        );
        prop_assert!(r.is_ok());
    }
}

// ===========================================================================
// 2. parser_code non-empty (5 properties)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(8))]

    #[test]
    fn prop_v8_parser_code_single(name in grammar_name_strategy()) {
        let r = build_single(&name);
        prop_assert!(!r.parser_code.is_empty());
    }

    #[test]
    fn prop_v8_parser_code_two_alt(name in grammar_name_strategy()) {
        let r = build_two_alt(&name);
        prop_assert!(!r.parser_code.is_empty());
    }

    #[test]
    fn prop_v8_parser_code_n_alts(n in arb_token_count()) {
        let name = format!("pp_v8_{}", n + 200);
        let r = build_n_alts(&name, n);
        prop_assert!(!r.parser_code.is_empty());
    }

    #[test]
    fn prop_v8_parser_code_concat(n in arb_token_count()) {
        let name = format!("pp_v8_{}", n + 210);
        let r = build_concat(&name, n);
        prop_assert!(!r.parser_code.is_empty());
    }

    #[test]
    fn prop_v8_parser_code_chain(name in grammar_name_strategy()) {
        let r = build_chain(&name);
        prop_assert!(!r.parser_code.is_empty());
    }
}

// ===========================================================================
// 3. node_types_json non-empty (5 properties)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(8))]

    #[test]
    fn prop_v8_node_types_single(name in grammar_name_strategy()) {
        let r = build_single(&name);
        prop_assert!(!r.node_types_json.is_empty());
    }

    #[test]
    fn prop_v8_node_types_two_alt(name in grammar_name_strategy()) {
        let r = build_two_alt(&name);
        prop_assert!(!r.node_types_json.is_empty());
    }

    #[test]
    fn prop_v8_node_types_n_alts(n in arb_token_count()) {
        let name = format!("pp_v8_{}", n + 300);
        let r = build_n_alts(&name, n);
        prop_assert!(!r.node_types_json.is_empty());
    }

    #[test]
    fn prop_v8_node_types_concat(n in arb_token_count()) {
        let name = format!("pp_v8_{}", n + 310);
        let r = build_concat(&name, n);
        prop_assert!(!r.node_types_json.is_empty());
    }

    #[test]
    fn prop_v8_node_types_chain(name in grammar_name_strategy()) {
        let r = build_chain(&name);
        prop_assert!(!r.node_types_json.is_empty());
    }
}

// ===========================================================================
// 4. state_count > 0 (5 properties)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(8))]

    #[test]
    fn prop_v8_state_count_single(name in grammar_name_strategy()) {
        let r = build_single(&name);
        prop_assert!(r.build_stats.state_count > 0);
    }

    #[test]
    fn prop_v8_state_count_two_alt(name in grammar_name_strategy()) {
        let r = build_two_alt(&name);
        prop_assert!(r.build_stats.state_count > 0);
    }

    #[test]
    fn prop_v8_state_count_n_alts(n in arb_token_count()) {
        let name = format!("pp_v8_{}", n + 400);
        let r = build_n_alts(&name, n);
        prop_assert!(r.build_stats.state_count > 0);
    }

    #[test]
    fn prop_v8_state_count_concat(n in arb_token_count()) {
        let name = format!("pp_v8_{}", n + 410);
        let r = build_concat(&name, n);
        prop_assert!(r.build_stats.state_count > 0);
    }

    #[test]
    fn prop_v8_state_count_prec(prec in -50i16..50i16) {
        let name = format!("pp_v8_{}", (prec + 500) as u16);
        let r = build_prec(&name, prec, Associativity::Left);
        prop_assert!(r.build_stats.state_count > 0);
    }
}

// ===========================================================================
// 5. symbol_count > 0 (5 properties)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(8))]

    #[test]
    fn prop_v8_symbol_count_single(name in grammar_name_strategy()) {
        let r = build_single(&name);
        prop_assert!(r.build_stats.symbol_count > 0);
    }

    #[test]
    fn prop_v8_symbol_count_two_alt(name in grammar_name_strategy()) {
        let r = build_two_alt(&name);
        prop_assert!(r.build_stats.symbol_count > 0);
    }

    #[test]
    fn prop_v8_symbol_count_n_alts(n in arb_token_count()) {
        let name = format!("pp_v8_{}", n + 600);
        let r = build_n_alts(&name, n);
        prop_assert!(r.build_stats.symbol_count > 0);
    }

    #[test]
    fn prop_v8_symbol_count_concat(n in arb_token_count()) {
        let name = format!("pp_v8_{}", n + 610);
        let r = build_concat(&name, n);
        prop_assert!(r.build_stats.symbol_count > 0);
    }

    #[test]
    fn prop_v8_symbol_count_chain(name in grammar_name_strategy()) {
        let r = build_chain(&name);
        prop_assert!(r.build_stats.symbol_count > 0);
    }
}

// ===========================================================================
// 6. conflict_cells bounded by state_count * symbol_count (5 properties)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(8))]

    #[test]
    fn prop_v8_conflict_bounded_single(name in grammar_name_strategy()) {
        let r = build_single(&name);
        let upper = r.build_stats.state_count * r.build_stats.symbol_count;
        prop_assert!(r.build_stats.conflict_cells <= upper);
    }

    #[test]
    fn prop_v8_conflict_bounded_two_alt(name in grammar_name_strategy()) {
        let r = build_two_alt(&name);
        let upper = r.build_stats.state_count * r.build_stats.symbol_count;
        prop_assert!(r.build_stats.conflict_cells <= upper);
    }

    #[test]
    fn prop_v8_conflict_bounded_n_alts(n in arb_token_count()) {
        let name = format!("pp_v8_{}", n + 700);
        let r = build_n_alts(&name, n);
        let upper = r.build_stats.state_count * r.build_stats.symbol_count;
        prop_assert!(r.build_stats.conflict_cells <= upper);
    }

    #[test]
    fn prop_v8_conflict_bounded_concat(n in arb_token_count()) {
        let name = format!("pp_v8_{}", n + 710);
        let r = build_concat(&name, n);
        let upper = r.build_stats.state_count * r.build_stats.symbol_count;
        prop_assert!(r.build_stats.conflict_cells <= upper);
    }

    #[test]
    fn prop_v8_conflict_bounded_prec(prec in -50i16..50i16) {
        let name = format!("pp_v8_{}", (prec + 800) as u16);
        let r = build_prec(&name, prec, Associativity::Left);
        let upper = r.build_stats.state_count * r.build_stats.symbol_count;
        prop_assert!(r.build_stats.conflict_cells <= upper);
    }
}

// ===========================================================================
// 7. Determinism: same grammar → same stats and code (5 properties)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(8))]

    #[test]
    fn prop_v8_det_single(name in grammar_name_strategy()) {
        let r1 = build_single(&name);
        let r2 = build_single(&name);
        prop_assert_eq!(r1.build_stats.state_count, r2.build_stats.state_count);
        prop_assert_eq!(r1.build_stats.symbol_count, r2.build_stats.symbol_count);
    }

    #[test]
    fn prop_v8_det_two_alt(name in grammar_name_strategy()) {
        let r1 = build_two_alt(&name);
        let r2 = build_two_alt(&name);
        prop_assert_eq!(r1.parser_code, r2.parser_code);
    }

    #[test]
    fn prop_v8_det_n_alts(n in arb_token_count()) {
        let name = format!("pp_v8_{}", n + 900);
        let r1 = build_n_alts(&name, n);
        let r2 = build_n_alts(&name, n);
        prop_assert_eq!(r1.build_stats.state_count, r2.build_stats.state_count);
        prop_assert_eq!(r1.build_stats.symbol_count, r2.build_stats.symbol_count);
        prop_assert_eq!(r1.build_stats.conflict_cells, r2.build_stats.conflict_cells);
    }

    #[test]
    fn prop_v8_det_concat(n in arb_token_count()) {
        let name = format!("pp_v8_{}", n + 910);
        let r1 = build_concat(&name, n);
        let r2 = build_concat(&name, n);
        prop_assert_eq!(r1.parser_code, r2.parser_code);
    }

    #[test]
    fn prop_v8_det_prec(prec in -50i16..50i16) {
        let name = format!("pp_v8_{}", (prec + 1000) as u16);
        let r1 = build_prec(&name, prec, Associativity::Left);
        let r2 = build_prec(&name, prec, Associativity::Left);
        prop_assert_eq!(r1.build_stats.state_count, r2.build_stats.state_count);
    }
}

// ===========================================================================
// 8. Different grammar → different parser code (5 properties)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(8))]

    #[test]
    fn prop_v8_diff_one_vs_two(name in grammar_name_strategy()) {
        let r1 = build_single(&name);
        let r2 = build_two_alt(&name);
        prop_assert_ne!(r1.parser_code, r2.parser_code);
    }

    #[test]
    fn prop_v8_diff_alt_vs_concat(_i in 0..1u8) {
        let r1 = build_n_alts("pp_v8_d0", 3);
        let r2 = build_concat("pp_v8_d1", 3);
        prop_assert_ne!(r1.parser_code, r2.parser_code);
    }

    #[test]
    fn prop_v8_diff_single_vs_chain(name in grammar_name_strategy()) {
        let r1 = build_single(&name);
        let r2 = build_chain(&name);
        prop_assert_ne!(r1.parser_code, r2.parser_code);
    }

    #[test]
    fn prop_v8_diff_sizes(n in 1usize..4) {
        let r1 = build_n_alts(&format!("pp_v8_{}", n + 1100), n);
        let r2 = build_n_alts(&format!("pp_v8_{}", n + 1110), n + 1);
        prop_assert_ne!(r1.parser_code, r2.parser_code);
    }

    #[test]
    fn prop_v8_diff_concat_sizes(n in 1usize..4) {
        let r1 = build_concat(&format!("pp_v8_{}", n + 1200), n);
        let r2 = build_concat(&format!("pp_v8_{}", n + 1210), n + 1);
        prop_assert_ne!(r1.parser_code, r2.parser_code);
    }
}

// ===========================================================================
// 9. compress_tables flag doesn't crash (5 properties)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(8))]

    #[test]
    fn prop_v8_compress_true_single(name in grammar_name_strategy()) {
        let g = GrammarBuilder::new(&name)
            .token("a", "a")
            .rule("s", vec!["a"])
            .start("s")
            .build();
        let r = build_parser(g, BuildOptions { compress_tables: true, ..test_opts() });
        prop_assert!(r.is_ok());
    }

    #[test]
    fn prop_v8_compress_false_single(name in grammar_name_strategy()) {
        let g = GrammarBuilder::new(&name)
            .token("a", "a")
            .rule("s", vec!["a"])
            .start("s")
            .build();
        let r = build_parser(g, BuildOptions { compress_tables: false, ..test_opts() });
        prop_assert!(r.is_ok());
    }

    #[test]
    fn prop_v8_compress_n_alts(n in arb_token_count()) {
        let name = format!("pp_v8_{}", n + 1300);
        let tok_names: Vec<String> = (0..n).map(|i| format!("t{i}")).collect();
        let tok_pats: Vec<String> = (0..n).map(|i| format!("p{i}")).collect();
        let mut b = GrammarBuilder::new(&name);
        for (tname, tpat) in tok_names.iter().zip(tok_pats.iter()) {
            b = b.token(tname, tpat);
        }
        for tname in &tok_names {
            b = b.rule("s", vec![tname.as_str()]);
        }
        b = b.start("s");
        let r = build_parser(b.build(), BuildOptions { compress_tables: true, ..test_opts() });
        prop_assert!(r.is_ok());
    }

    #[test]
    fn prop_v8_compress_concat(n in arb_token_count()) {
        let name = format!("pp_v8_{}", n + 1310);
        let tok_names: Vec<String> = (0..n).map(|i| format!("t{i}")).collect();
        let tok_pats: Vec<String> = (0..n).map(|i| format!("q{i}")).collect();
        let rhs: Vec<&str> = tok_names.iter().map(|t| t.as_str()).collect();
        let mut b = GrammarBuilder::new(&name);
        for (tname, tpat) in tok_names.iter().zip(tok_pats.iter()) {
            b = b.token(tname, tpat);
        }
        b = b.rule("s", rhs).start("s");
        let r = build_parser(b.build(), BuildOptions { compress_tables: false, ..test_opts() });
        prop_assert!(r.is_ok());
    }

    #[test]
    fn prop_v8_compress_arb(compress in arb_bool()) {
        let g = GrammarBuilder::new("pp_v8_arbc")
            .token("a", "a")
            .rule("s", vec!["a"])
            .start("s")
            .build();
        let r = build_parser(g, BuildOptions { compress_tables: compress, ..test_opts() });
        prop_assert!(r.is_ok());
    }
}

// ===========================================================================
// 10. emit_artifacts flag doesn't crash (5 properties)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(8))]

    #[test]
    fn prop_v8_emit_true_single(name in grammar_name_strategy()) {
        let g = GrammarBuilder::new(&name)
            .token("a", "a")
            .rule("s", vec!["a"])
            .start("s")
            .build();
        let r = build_parser(g, BuildOptions { emit_artifacts: true, ..test_opts() });
        prop_assert!(r.is_ok());
    }

    #[test]
    fn prop_v8_emit_false_single(name in grammar_name_strategy()) {
        let g = GrammarBuilder::new(&name)
            .token("a", "a")
            .rule("s", vec!["a"])
            .start("s")
            .build();
        let r = build_parser(g, BuildOptions { emit_artifacts: false, ..test_opts() });
        prop_assert!(r.is_ok());
    }

    #[test]
    fn prop_v8_emit_n_alts(n in arb_token_count()) {
        let name = format!("pp_v8_{}", n + 1400);
        let tok_names: Vec<String> = (0..n).map(|i| format!("t{i}")).collect();
        let tok_pats: Vec<String> = (0..n).map(|i| format!("p{i}")).collect();
        let mut b = GrammarBuilder::new(&name);
        for (tname, tpat) in tok_names.iter().zip(tok_pats.iter()) {
            b = b.token(tname, tpat);
        }
        for tname in &tok_names {
            b = b.rule("s", vec![tname.as_str()]);
        }
        b = b.start("s");
        let r = build_parser(b.build(), BuildOptions { emit_artifacts: true, ..test_opts() });
        prop_assert!(r.is_ok());
    }

    #[test]
    fn prop_v8_emit_arb(emit in arb_bool()) {
        let g = GrammarBuilder::new("pp_v8_arbe")
            .token("a", "a")
            .rule("s", vec!["a"])
            .start("s")
            .build();
        let r = build_parser(g, BuildOptions { emit_artifacts: emit, ..test_opts() });
        prop_assert!(r.is_ok());
    }

    #[test]
    fn prop_v8_emit_both_flags(emit in arb_bool(), compress in arb_bool()) {
        let g = GrammarBuilder::new("pp_v8_arbec")
            .token("a", "a")
            .rule("s", vec!["a"])
            .start("s")
            .build();
        let opts = BuildOptions {
            out_dir: "/tmp/proptest_pipeline_v8_emit".to_string(),
            emit_artifacts: emit,
            compress_tables: compress,
        };
        let r = build_parser(g, opts);
        prop_assert!(r.is_ok());
    }
}

// ===========================================================================
// 11. node_types_json is valid JSON (5 properties)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(8))]

    #[test]
    fn prop_v8_json_single(name in grammar_name_strategy()) {
        let r = build_single(&name);
        let parsed: Result<serde_json::Value, _> = serde_json::from_str(&r.node_types_json);
        prop_assert!(parsed.is_ok(), "not valid JSON: {:?}", parsed.err());
    }

    #[test]
    fn prop_v8_json_two_alt(name in grammar_name_strategy()) {
        let r = build_two_alt(&name);
        let parsed: Result<serde_json::Value, _> = serde_json::from_str(&r.node_types_json);
        prop_assert!(parsed.is_ok());
    }

    #[test]
    fn prop_v8_json_n_alts(n in arb_token_count()) {
        let name = format!("pp_v8_{}", n + 1500);
        let r = build_n_alts(&name, n);
        let parsed: Result<serde_json::Value, _> = serde_json::from_str(&r.node_types_json);
        prop_assert!(parsed.is_ok());
    }

    #[test]
    fn prop_v8_json_concat(n in arb_token_count()) {
        let name = format!("pp_v8_{}", n + 1510);
        let r = build_concat(&name, n);
        let parsed: Result<serde_json::Value, _> = serde_json::from_str(&r.node_types_json);
        prop_assert!(parsed.is_ok());
    }

    #[test]
    fn prop_v8_json_prec(prec in -50i16..50i16) {
        let name = format!("pp_v8_{}", (prec + 1600) as u16);
        let r = build_prec(&name, prec, Associativity::Left);
        let parsed: Result<serde_json::Value, _> = serde_json::from_str(&r.node_types_json);
        prop_assert!(parsed.is_ok());
    }
}

// ===========================================================================
// 12. symbol_count >= token count (5 properties)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(8))]

    #[test]
    fn prop_v8_symge_single(_i in 0..1u8) {
        let r = build_single("pp_v8_sg0");
        // 1 token defined → symbol_count should be at least 1
        prop_assert!(r.build_stats.symbol_count >= 1);
    }

    #[test]
    fn prop_v8_symge_two_alt(_i in 0..1u8) {
        let r = build_two_alt("pp_v8_sg1");
        // 2 tokens defined
        prop_assert!(r.build_stats.symbol_count >= 2);
    }

    #[test]
    fn prop_v8_symge_n_alts(n in arb_token_count()) {
        let name = format!("pp_v8_{}", n + 1700);
        let r = build_n_alts(&name, n);
        prop_assert!(
            r.build_stats.symbol_count >= n,
            "symbol_count {} < token count {}",
            r.build_stats.symbol_count,
            n,
        );
    }

    #[test]
    fn prop_v8_symge_concat(n in arb_token_count()) {
        let name = format!("pp_v8_{}", n + 1710);
        let r = build_concat(&name, n);
        prop_assert!(
            r.build_stats.symbol_count >= n,
            "symbol_count {} < token count {}",
            r.build_stats.symbol_count,
            n,
        );
    }

    #[test]
    fn prop_v8_symge_prec(_i in 0..1u8) {
        let r = build_prec("pp_v8_sg2", 1, Associativity::Left);
        // 2 tokens: x, op
        prop_assert!(r.build_stats.symbol_count >= 2);
    }
}

// ===========================================================================
// 13. parser_code length scales with grammar (5 properties)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(8))]

    #[test]
    fn prop_v8_scaling_alts(n in 1usize..4) {
        let small = build_n_alts(&format!("pp_v8_{}", n + 1800), n);
        let large = build_n_alts(&format!("pp_v8_{}", n + 1810), n + 1);
        prop_assert!(
            large.parser_code.len() >= small.parser_code.len(),
            "code len small={} (n={}) > large={} (n={})",
            small.parser_code.len(), n, large.parser_code.len(), n + 1,
        );
    }

    #[test]
    fn prop_v8_scaling_concat(n in 1usize..4) {
        let small = build_concat(&format!("pp_v8_{}", n + 1900), n);
        let large = build_concat(&format!("pp_v8_{}", n + 1910), n + 1);
        prop_assert!(
            large.parser_code.len() >= small.parser_code.len(),
            "code len small={} (n={}) > large={} (n={})",
            small.parser_code.len(), n, large.parser_code.len(), n + 1,
        );
    }

    #[test]
    fn prop_v8_scaling_symbol_monotonic(n in 1usize..4) {
        let small = build_n_alts(&format!("pp_v8_{}", n + 2000), n);
        let large = build_n_alts(&format!("pp_v8_{}", n + 2010), n + 1);
        prop_assert!(
            large.build_stats.symbol_count >= small.build_stats.symbol_count,
            "symbol_count small={} > large={}",
            small.build_stats.symbol_count, large.build_stats.symbol_count,
        );
    }

    #[test]
    fn prop_v8_scaling_out_dir_no_effect(name in grammar_name_strategy()) {
        let g1 = GrammarBuilder::new(&name)
            .token("a", "a")
            .rule("s", vec!["a"])
            .start("s")
            .build();
        let g2 = GrammarBuilder::new(&name)
            .token("a", "a")
            .rule("s", vec!["a"])
            .start("s")
            .build();
        let opts1 = BuildOptions {
            out_dir: "/tmp/proptest_v8_dir_a".to_string(),
            emit_artifacts: false,
            compress_tables: true,
        };
        let opts2 = BuildOptions {
            out_dir: "/tmp/proptest_v8_dir_b".to_string(),
            emit_artifacts: false,
            compress_tables: true,
        };
        let r1 = build_parser(g1, opts1).unwrap();
        let r2 = build_parser(g2, opts2).unwrap();
        prop_assert_eq!(r1.build_stats.state_count, r2.build_stats.state_count);
        prop_assert_eq!(r1.build_stats.symbol_count, r2.build_stats.symbol_count);
    }

    #[test]
    fn prop_v8_scaling_state_monotonic(n in 1usize..4) {
        let small = build_concat(&format!("pp_v8_{}", n + 2100), n);
        let large = build_concat(&format!("pp_v8_{}", n + 2110), n + 1);
        prop_assert!(
            large.build_stats.state_count >= small.build_stats.state_count,
            "state_count small={} > large={}",
            small.build_stats.state_count, large.build_stats.state_count,
        );
    }
}

// ===========================================================================
// 14. Specific grammar patterns (7 unit tests)
// ===========================================================================

#[test]
fn test_v8_pattern_single_token() {
    let r = build_single("pp_v8_pat0");
    assert!(!r.parser_code.is_empty());
    assert!(r.build_stats.state_count > 0);
}

#[test]
fn test_v8_pattern_two_alternatives() {
    let r = build_two_alt("pp_v8_pat1");
    assert!(r.build_stats.symbol_count >= 2);
}

#[test]
fn test_v8_pattern_chain_three_deep() {
    let r = build_chain("pp_v8_pat2");
    assert!(r.build_stats.state_count > 0);
    assert!(!r.node_types_json.is_empty());
}

#[test]
fn test_v8_pattern_prec_left() {
    let r = build_prec("pp_v8_pat3", 1, Associativity::Left);
    assert!(r.build_stats.state_count > 0);
    assert!(!r.parser_code.is_empty());
}

#[test]
fn test_v8_pattern_prec_right() {
    let r = build_prec("pp_v8_pat4", 1, Associativity::Right);
    assert!(r.build_stats.state_count > 0);
}

#[test]
fn test_v8_pattern_prec_none() {
    let r = build_prec("pp_v8_pat5", 1, Associativity::None);
    assert!(r.build_stats.state_count > 0);
}

#[test]
fn test_v8_pattern_multi_rule_nonterminal() {
    let g = GrammarBuilder::new("pp_v8_pat6")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("item", vec!["a"])
        .rule("item", vec!["b"])
        .rule("item", vec!["c"])
        .rule("s", vec!["item"])
        .start("s")
        .build();
    let r = build_parser(g, test_opts()).expect("multi-rule build failed");
    assert!(r.build_stats.symbol_count >= 3);
    assert!(!r.parser_code.is_empty());
}

// ===========================================================================
// 15. Edge cases (6 unit tests)
// ===========================================================================

#[test]
fn test_v8_edge_single_char_token() {
    let g = GrammarBuilder::new("pp_v8_e0")
        .token("x", "x")
        .rule("s", vec!["x"])
        .start("s")
        .build();
    let r = build_parser(g, test_opts()).unwrap();
    assert!(r.build_stats.state_count > 0);
}

#[test]
fn test_v8_edge_seven_alts() {
    let r = build_n_alts("pp_v8_e1", 7);
    assert!(r.build_stats.symbol_count >= 7);
    assert!(!r.parser_code.is_empty());
}

#[test]
fn test_v8_edge_seven_concat() {
    let r = build_concat("pp_v8_e2", 7);
    assert!(r.build_stats.state_count > 0);
}

#[test]
fn test_v8_edge_prec_zero() {
    let r = build_prec("pp_v8_e3", 0, Associativity::Left);
    assert!(r.build_stats.state_count > 0);
}

#[test]
fn test_v8_edge_prec_negative() {
    let r = build_prec("pp_v8_e4", -50, Associativity::Right);
    assert!(!r.parser_code.is_empty());
}

#[test]
fn test_v8_edge_prec_high_positive() {
    let r = build_prec("pp_v8_e5", 49, Associativity::None);
    assert!(!r.node_types_json.is_empty());
}

// ===========================================================================
// 16. Option combinations (6 unit tests)
// ===========================================================================

#[test]
fn test_v8_combo_compress_true_emit_true() {
    let g = GrammarBuilder::new("pp_v8_c0")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let opts = BuildOptions {
        out_dir: "/tmp/proptest_v8_combo".to_string(),
        emit_artifacts: true,
        compress_tables: true,
    };
    let r = build_parser(g, opts).unwrap();
    assert!(!r.parser_code.is_empty());
}

#[test]
fn test_v8_combo_compress_true_emit_false() {
    let g = GrammarBuilder::new("pp_v8_c1")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let opts = BuildOptions {
        out_dir: "/tmp/proptest_v8_combo".to_string(),
        emit_artifacts: false,
        compress_tables: true,
    };
    let r = build_parser(g, opts).unwrap();
    assert!(!r.parser_code.is_empty());
}

#[test]
fn test_v8_combo_compress_false_emit_true() {
    let g = GrammarBuilder::new("pp_v8_c2")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let opts = BuildOptions {
        out_dir: "/tmp/proptest_v8_combo".to_string(),
        emit_artifacts: true,
        compress_tables: false,
    };
    let r = build_parser(g, opts).unwrap();
    assert!(!r.parser_code.is_empty());
}

#[test]
fn test_v8_combo_compress_false_emit_false() {
    let g = GrammarBuilder::new("pp_v8_c3")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let opts = BuildOptions {
        out_dir: "/tmp/proptest_v8_combo".to_string(),
        emit_artifacts: false,
        compress_tables: false,
    };
    let r = build_parser(g, opts).unwrap();
    assert!(!r.parser_code.is_empty());
}

#[test]
fn test_v8_combo_two_alt_all_opts() {
    let g = GrammarBuilder::new("pp_v8_c4")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a"])
        .rule("s", vec!["b"])
        .start("s")
        .build();
    let opts = BuildOptions {
        out_dir: "/tmp/proptest_v8_combo".to_string(),
        emit_artifacts: true,
        compress_tables: true,
    };
    let r = build_parser(g, opts).unwrap();
    assert!(r.build_stats.symbol_count >= 2);
    let parsed: Result<serde_json::Value, _> = serde_json::from_str(&r.node_types_json);
    assert!(parsed.is_ok());
}

#[test]
fn test_v8_combo_chain_all_opts() {
    let g = GrammarBuilder::new("pp_v8_c5")
        .token("x", "x")
        .rule("leaf", vec!["x"])
        .rule("mid", vec!["leaf"])
        .rule("s", vec!["mid"])
        .start("s")
        .build();
    let opts = BuildOptions {
        out_dir: "/tmp/proptest_v8_combo".to_string(),
        emit_artifacts: true,
        compress_tables: false,
    };
    let r = build_parser(g, opts).unwrap();
    assert!(r.build_stats.state_count > 0);
    assert!(!r.node_types_json.is_empty());
}
