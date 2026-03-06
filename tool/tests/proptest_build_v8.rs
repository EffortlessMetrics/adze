//! Property-based tests for `build_parser` in adze-tool (v8).
//!
//! 84 tests across 17 categories:
//!   1.  prop_pb_v8_code_nonempty_*    — parser_code not empty (5)
//!   2.  prop_pb_v8_json_nonempty_*    — node_types_json not empty (5)
//!   3.  prop_pb_v8_state_ge2_*        — state_count ≥ 2 (5)
//!   4.  prop_pb_v8_sym_ge2_*          — symbol_count ≥ 2 (5)
//!   5.  prop_pb_v8_deterministic_*    — same input → same output (5)
//!   6.  prop_pb_v8_emit_stable_*      — emit_artifacts doesn't change parser_code (5)
//!   7.  prop_pb_v8_compress_ok_*      — compress_tables flag doesn't crash (5)
//!   8.  prop_pb_v8_stats_nonneg_*     — build stats non-negative (usize) (5)
//!   9.  prop_pb_v8_conflict_ge0_*     — conflict_cells ≥ 0 (always usize) (5)
//!   10. test_pb_v8_minimal_*          — minimal grammar option combos (4)
//!   11. test_pb_v8_arith_*            — arithmetic grammar combos (4)
//!   12. test_pb_v8_stats_clone        — BuildStats Clone (1)
//!   13. test_pb_v8_empty_outdir       — empty out_dir (1)
//!   14. prop_pb_v8_token_pat_*        — arbitrary token patterns (5)
//!   15. prop_pb_v8_opts_*             — arbitrary build options (5)
//!   16. test_pb_v8_shape_*            — grammar shape variants (9)
//!   17. test_pb_v8_extra_*            — extra edge-case combos (10)

use adze_ir::Associativity;
use adze_ir::builder::GrammarBuilder;
use adze_tool::pure_rust_builder::{BuildOptions, BuildResult, build_parser};
use proptest::prelude::*;

// ===========================================================================
// Strategies
// ===========================================================================

fn arb_token_pattern() -> impl Strategy<Value = &'static str> {
    prop_oneof![
        Just(r"\d+"),
        Just(r"[a-z]+"),
        Just(r"\w+"),
        Just(r"[0-9]+"),
        Just(r"[A-Za-z_]+"),
    ]
}

fn arb_build_options() -> impl Strategy<Value = BuildOptions> {
    (any::<bool>(), any::<bool>()).prop_map(|(emit, compress)| BuildOptions {
        out_dir: "/tmp/test".to_string(),
        emit_artifacts: emit,
        compress_tables: compress,
    })
}

// ===========================================================================
// Helpers
// ===========================================================================

fn test_opts() -> BuildOptions {
    BuildOptions {
        out_dir: "/tmp/proptest_build_v8".to_string(),
        emit_artifacts: false,
        compress_tables: true,
    }
}

fn minimal_grammar(name: &str) -> BuildResult {
    build_parser(
        GrammarBuilder::new(name)
            .token("a", "a")
            .rule("s", vec!["a"])
            .start("s")
            .build(),
        test_opts(),
    )
    .expect("minimal_grammar failed")
}

fn two_alt_grammar(name: &str) -> BuildResult {
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
    .expect("two_alt_grammar failed")
}

fn arith_grammar(name: &str) -> BuildResult {
    build_parser(
        GrammarBuilder::new(name)
            .token("num", r"\d+")
            .token("plus", r"\+")
            .token("star", r"\*")
            .rule("expr", vec!["num"])
            .rule_with_precedence("expr", vec!["expr", "plus", "expr"], 1, Associativity::Left)
            .rule_with_precedence("expr", vec!["expr", "star", "expr"], 2, Associativity::Left)
            .start("expr")
            .build(),
        test_opts(),
    )
    .expect("arith_grammar failed")
}

fn chain_grammar(name: &str) -> BuildResult {
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
    .expect("chain_grammar failed")
}

fn concat_grammar(name: &str, n: usize) -> BuildResult {
    let tok_names: Vec<String> = (0..n).map(|i| format!("t{i}")).collect();
    let tok_pats: Vec<String> = (0..n).map(|i| format!("q{i}")).collect();
    let rhs: Vec<&str> = tok_names.iter().map(|s| s.as_str()).collect();
    let mut b = GrammarBuilder::new(name);
    for (tname, tpat) in tok_names.iter().zip(tok_pats.iter()) {
        b = b.token(tname, tpat);
    }
    b = b.rule("s", rhs).start("s");
    build_parser(b.build(), test_opts()).expect("concat_grammar failed")
}

fn n_alt_grammar(name: &str, n: usize) -> BuildResult {
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
    build_parser(b.build(), test_opts()).expect("n_alt_grammar failed")
}

fn prec_grammar(name: &str, prec: i16, assoc: Associativity) -> BuildResult {
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
    .expect("prec_grammar failed")
}

fn build_with_pattern(name: &str, pat: &str) -> BuildResult {
    build_parser(
        GrammarBuilder::new(name)
            .token("tok", pat)
            .rule("s", vec!["tok"])
            .start("s")
            .build(),
        test_opts(),
    )
    .expect("build_with_pattern failed")
}

// ===========================================================================
// 1. parser_code not empty (5 properties)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(8))]

    #[test]
    fn prop_pb_v8_code_nonempty_minimal(_i in 0..1u8) {
        let r = minimal_grammar("pb_v8_cn0");
        prop_assert!(!r.parser_code.is_empty());
    }

    #[test]
    fn prop_pb_v8_code_nonempty_two_alt(_i in 0..1u8) {
        let r = two_alt_grammar("pb_v8_cn1");
        prop_assert!(!r.parser_code.is_empty());
    }

    #[test]
    fn prop_pb_v8_code_nonempty_chain(_i in 0..1u8) {
        let r = chain_grammar("pb_v8_cn2");
        prop_assert!(!r.parser_code.is_empty());
    }

    #[test]
    fn prop_pb_v8_code_nonempty_n_alts(n in 1usize..6) {
        let name = format!("pb_v8_cn3_{n}");
        let r = n_alt_grammar(&name, n);
        prop_assert!(!r.parser_code.is_empty());
    }

    #[test]
    fn prop_pb_v8_code_nonempty_concat(n in 1usize..6) {
        let name = format!("pb_v8_cn4_{n}");
        let r = concat_grammar(&name, n);
        prop_assert!(!r.parser_code.is_empty());
    }
}

// ===========================================================================
// 2. node_types_json not empty (5 properties)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(8))]

    #[test]
    fn prop_pb_v8_json_nonempty_minimal(_i in 0..1u8) {
        let r = minimal_grammar("pb_v8_jn0");
        prop_assert!(!r.node_types_json.is_empty());
    }

    #[test]
    fn prop_pb_v8_json_nonempty_two_alt(_i in 0..1u8) {
        let r = two_alt_grammar("pb_v8_jn1");
        prop_assert!(!r.node_types_json.is_empty());
    }

    #[test]
    fn prop_pb_v8_json_nonempty_chain(_i in 0..1u8) {
        let r = chain_grammar("pb_v8_jn2");
        prop_assert!(!r.node_types_json.is_empty());
    }

    #[test]
    fn prop_pb_v8_json_nonempty_n_alts(n in 1usize..6) {
        let name = format!("pb_v8_jn3_{n}");
        let r = n_alt_grammar(&name, n);
        prop_assert!(!r.node_types_json.is_empty());
    }

    #[test]
    fn prop_pb_v8_json_nonempty_concat(n in 1usize..6) {
        let name = format!("pb_v8_jn4_{n}");
        let r = concat_grammar(&name, n);
        prop_assert!(!r.node_types_json.is_empty());
    }
}

// ===========================================================================
// 3. state_count ≥ 2 (5 properties)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(8))]

    #[test]
    fn prop_pb_v8_state_ge2_minimal(_i in 0..1u8) {
        let r = minimal_grammar("pb_v8_sg0");
        prop_assert!(r.build_stats.state_count >= 2);
    }

    #[test]
    fn prop_pb_v8_state_ge2_two_alt(_i in 0..1u8) {
        let r = two_alt_grammar("pb_v8_sg1");
        prop_assert!(r.build_stats.state_count >= 2);
    }

    #[test]
    fn prop_pb_v8_state_ge2_chain(_i in 0..1u8) {
        let r = chain_grammar("pb_v8_sg2");
        prop_assert!(r.build_stats.state_count >= 2);
    }

    #[test]
    fn prop_pb_v8_state_ge2_n_alts(n in 1usize..6) {
        let name = format!("pb_v8_sg3_{n}");
        let r = n_alt_grammar(&name, n);
        prop_assert!(r.build_stats.state_count >= 2);
    }

    #[test]
    fn prop_pb_v8_state_ge2_prec(prec in -20i16..20i16) {
        let name = format!("pb_v8_sg4_{}", (prec + 100) as u16);
        let r = prec_grammar(&name, prec, Associativity::Left);
        prop_assert!(r.build_stats.state_count >= 2);
    }
}

// ===========================================================================
// 4. symbol_count ≥ 2 (5 properties)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(8))]

    #[test]
    fn prop_pb_v8_sym_ge2_minimal(_i in 0..1u8) {
        let r = minimal_grammar("pb_v8_sy0");
        prop_assert!(r.build_stats.symbol_count >= 2);
    }

    #[test]
    fn prop_pb_v8_sym_ge2_two_alt(_i in 0..1u8) {
        let r = two_alt_grammar("pb_v8_sy1");
        prop_assert!(r.build_stats.symbol_count >= 2);
    }

    #[test]
    fn prop_pb_v8_sym_ge2_chain(_i in 0..1u8) {
        let r = chain_grammar("pb_v8_sy2");
        prop_assert!(r.build_stats.symbol_count >= 2);
    }

    #[test]
    fn prop_pb_v8_sym_ge2_n_alts(n in 1usize..6) {
        let name = format!("pb_v8_sy3_{n}");
        let r = n_alt_grammar(&name, n);
        prop_assert!(r.build_stats.symbol_count >= 2);
    }

    #[test]
    fn prop_pb_v8_sym_ge2_concat(n in 1usize..6) {
        let name = format!("pb_v8_sy4_{n}");
        let r = concat_grammar(&name, n);
        prop_assert!(r.build_stats.symbol_count >= 2);
    }
}

// ===========================================================================
// 5. Determinism: same grammar + options → same output (5 properties)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(8))]

    #[test]
    fn prop_pb_v8_deterministic_code(_i in 0..1u8) {
        let r1 = minimal_grammar("pb_v8_det0");
        let r2 = minimal_grammar("pb_v8_det0");
        prop_assert_eq!(r1.parser_code, r2.parser_code);
    }

    #[test]
    fn prop_pb_v8_deterministic_json(_i in 0..1u8) {
        let r1 = two_alt_grammar("pb_v8_det1");
        let r2 = two_alt_grammar("pb_v8_det1");
        prop_assert_eq!(r1.node_types_json, r2.node_types_json);
    }

    #[test]
    fn prop_pb_v8_deterministic_states(_i in 0..1u8) {
        let r1 = chain_grammar("pb_v8_det2");
        let r2 = chain_grammar("pb_v8_det2");
        prop_assert_eq!(r1.build_stats.state_count, r2.build_stats.state_count);
    }

    #[test]
    fn prop_pb_v8_deterministic_symbols(n in 1usize..6) {
        let name = format!("pb_v8_det3_{n}");
        let r1 = n_alt_grammar(&name, n);
        let r2 = n_alt_grammar(&name, n);
        prop_assert_eq!(r1.build_stats.symbol_count, r2.build_stats.symbol_count);
    }

    #[test]
    fn prop_pb_v8_deterministic_conflicts(n in 1usize..6) {
        let name = format!("pb_v8_det4_{n}");
        let r1 = concat_grammar(&name, n);
        let r2 = concat_grammar(&name, n);
        prop_assert_eq!(r1.build_stats.conflict_cells, r2.build_stats.conflict_cells);
    }
}

// ===========================================================================
// 6. emit_artifacts doesn't change parser_code (5 properties)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(8))]

    #[test]
    fn prop_pb_v8_emit_stable_code(_i in 0..1u8) {
        let opts_on = BuildOptions { emit_artifacts: true, ..test_opts() };
        let opts_off = BuildOptions { emit_artifacts: false, ..test_opts() };
        let g1 = GrammarBuilder::new("pb_v8_es0")
            .token("a", "a").rule("s", vec!["a"]).start("s").build();
        let g2 = GrammarBuilder::new("pb_v8_es0")
            .token("a", "a").rule("s", vec!["a"]).start("s").build();
        let r1 = build_parser(g1, opts_on).unwrap();
        let r2 = build_parser(g2, opts_off).unwrap();
        prop_assert_eq!(r1.parser_code, r2.parser_code);
    }

    #[test]
    fn prop_pb_v8_emit_stable_json(_i in 0..1u8) {
        let opts_on = BuildOptions { emit_artifacts: true, ..test_opts() };
        let opts_off = BuildOptions { emit_artifacts: false, ..test_opts() };
        let g1 = GrammarBuilder::new("pb_v8_es1")
            .token("a", "a").token("b", "b")
            .rule("s", vec!["a"]).rule("s", vec!["b"]).start("s").build();
        let g2 = GrammarBuilder::new("pb_v8_es1")
            .token("a", "a").token("b", "b")
            .rule("s", vec!["a"]).rule("s", vec!["b"]).start("s").build();
        let r1 = build_parser(g1, opts_on).unwrap();
        let r2 = build_parser(g2, opts_off).unwrap();
        prop_assert_eq!(r1.node_types_json, r2.node_types_json);
    }

    #[test]
    fn prop_pb_v8_emit_stable_states(_i in 0..1u8) {
        let opts_on = BuildOptions { emit_artifacts: true, ..test_opts() };
        let opts_off = BuildOptions { emit_artifacts: false, ..test_opts() };
        let g1 = GrammarBuilder::new("pb_v8_es2")
            .token("x", "x").rule("s", vec!["x"]).start("s").build();
        let g2 = GrammarBuilder::new("pb_v8_es2")
            .token("x", "x").rule("s", vec!["x"]).start("s").build();
        let r1 = build_parser(g1, opts_on).unwrap();
        let r2 = build_parser(g2, opts_off).unwrap();
        prop_assert_eq!(r1.build_stats.state_count, r2.build_stats.state_count);
    }

    #[test]
    fn prop_pb_v8_emit_stable_symbols(_i in 0..1u8) {
        let opts_on = BuildOptions { emit_artifacts: true, ..test_opts() };
        let opts_off = BuildOptions { emit_artifacts: false, ..test_opts() };
        let g1 = GrammarBuilder::new("pb_v8_es3")
            .token("x", "x").token("op", "op")
            .rule("expr", vec!["x"])
            .rule_with_precedence("expr", vec!["expr", "op", "expr"], 1, Associativity::Left)
            .start("expr").build();
        let g2 = GrammarBuilder::new("pb_v8_es3")
            .token("x", "x").token("op", "op")
            .rule("expr", vec!["x"])
            .rule_with_precedence("expr", vec!["expr", "op", "expr"], 1, Associativity::Left)
            .start("expr").build();
        let r1 = build_parser(g1, opts_on).unwrap();
        let r2 = build_parser(g2, opts_off).unwrap();
        prop_assert_eq!(r1.build_stats.symbol_count, r2.build_stats.symbol_count);
    }

    #[test]
    fn prop_pb_v8_emit_stable_conflicts(_i in 0..1u8) {
        let opts_on = BuildOptions { emit_artifacts: true, ..test_opts() };
        let opts_off = BuildOptions { emit_artifacts: false, ..test_opts() };
        let g1 = GrammarBuilder::new("pb_v8_es4")
            .token("x", "x").rule("leaf", vec!["x"])
            .rule("mid", vec!["leaf"]).rule("s", vec!["mid"]).start("s").build();
        let g2 = GrammarBuilder::new("pb_v8_es4")
            .token("x", "x").rule("leaf", vec!["x"])
            .rule("mid", vec!["leaf"]).rule("s", vec!["mid"]).start("s").build();
        let r1 = build_parser(g1, opts_on).unwrap();
        let r2 = build_parser(g2, opts_off).unwrap();
        prop_assert_eq!(r1.build_stats.conflict_cells, r2.build_stats.conflict_cells);
    }
}

// ===========================================================================
// 7. compress_tables flag doesn't crash (5 properties)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(8))]

    #[test]
    fn prop_pb_v8_compress_ok_true(_i in 0..1u8) {
        let g = GrammarBuilder::new("pb_v8_co0")
            .token("a", "a").rule("s", vec!["a"]).start("s").build();
        let r = build_parser(g, BuildOptions { compress_tables: true, ..test_opts() });
        prop_assert!(r.is_ok());
    }

    #[test]
    fn prop_pb_v8_compress_ok_false(_i in 0..1u8) {
        let g = GrammarBuilder::new("pb_v8_co1")
            .token("a", "a").rule("s", vec!["a"]).start("s").build();
        let r = build_parser(g, BuildOptions { compress_tables: false, ..test_opts() });
        prop_assert!(r.is_ok());
    }

    #[test]
    fn prop_pb_v8_compress_ok_arb(compress in any::<bool>()) {
        let g = GrammarBuilder::new("pb_v8_co2")
            .token("a", "a").rule("s", vec!["a"]).start("s").build();
        let r = build_parser(g, BuildOptions { compress_tables: compress, ..test_opts() });
        prop_assert!(r.is_ok());
    }

    #[test]
    fn prop_pb_v8_compress_ok_two_alt(compress in any::<bool>()) {
        let g = GrammarBuilder::new("pb_v8_co3")
            .token("a", "a").token("b", "b")
            .rule("s", vec!["a"]).rule("s", vec!["b"]).start("s").build();
        let r = build_parser(g, BuildOptions { compress_tables: compress, ..test_opts() });
        prop_assert!(r.is_ok());
    }

    #[test]
    fn prop_pb_v8_compress_ok_chain(compress in any::<bool>()) {
        let g = GrammarBuilder::new("pb_v8_co4")
            .token("x", "x").rule("leaf", vec!["x"])
            .rule("mid", vec!["leaf"]).rule("s", vec!["mid"]).start("s").build();
        let r = build_parser(g, BuildOptions { compress_tables: compress, ..test_opts() });
        prop_assert!(r.is_ok());
    }
}

// ===========================================================================
// 8. Build stats non-negative (usize guarantees, verify structure) (5)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(8))]

    #[test]
    fn prop_pb_v8_stats_nonneg_minimal(_i in 0..1u8) {
        let r = minimal_grammar("pb_v8_nn0");
        prop_assert!(r.build_stats.state_count > 0);
        prop_assert!(r.build_stats.symbol_count > 0);
    }

    #[test]
    fn prop_pb_v8_stats_nonneg_two_alt(_i in 0..1u8) {
        let r = two_alt_grammar("pb_v8_nn1");
        prop_assert!(r.build_stats.state_count > 0);
        prop_assert!(r.build_stats.symbol_count > 0);
    }

    #[test]
    fn prop_pb_v8_stats_nonneg_chain(_i in 0..1u8) {
        let r = chain_grammar("pb_v8_nn2");
        prop_assert!(r.build_stats.state_count > 0);
        prop_assert!(r.build_stats.symbol_count > 0);
    }

    #[test]
    fn prop_pb_v8_stats_nonneg_prec(prec in -20i16..20i16) {
        let name = format!("pb_v8_nn3_{}", (prec + 100) as u16);
        let r = prec_grammar(&name, prec, Associativity::Left);
        prop_assert!(r.build_stats.state_count > 0);
        prop_assert!(r.build_stats.symbol_count > 0);
    }

    #[test]
    fn prop_pb_v8_stats_nonneg_concat(n in 1usize..6) {
        let name = format!("pb_v8_nn4_{n}");
        let r = concat_grammar(&name, n);
        prop_assert!(r.build_stats.state_count > 0);
        prop_assert!(r.build_stats.symbol_count > 0);
    }
}

// ===========================================================================
// 9. conflict_cells ≥ 0 (always true for usize, verify bounded) (5)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(8))]

    #[test]
    fn prop_pb_v8_conflict_ge0_minimal(_i in 0..1u8) {
        let r = minimal_grammar("pb_v8_cf0");
        let upper = r.build_stats.state_count * r.build_stats.symbol_count;
        prop_assert!(r.build_stats.conflict_cells <= upper);
    }

    #[test]
    fn prop_pb_v8_conflict_ge0_two_alt(_i in 0..1u8) {
        let r = two_alt_grammar("pb_v8_cf1");
        let upper = r.build_stats.state_count * r.build_stats.symbol_count;
        prop_assert!(r.build_stats.conflict_cells <= upper);
    }

    #[test]
    fn prop_pb_v8_conflict_ge0_chain(_i in 0..1u8) {
        let r = chain_grammar("pb_v8_cf2");
        let upper = r.build_stats.state_count * r.build_stats.symbol_count;
        prop_assert!(r.build_stats.conflict_cells <= upper);
    }

    #[test]
    fn prop_pb_v8_conflict_ge0_n_alts(n in 1usize..6) {
        let name = format!("pb_v8_cf3_{n}");
        let r = n_alt_grammar(&name, n);
        let upper = r.build_stats.state_count * r.build_stats.symbol_count;
        prop_assert!(r.build_stats.conflict_cells <= upper);
    }

    #[test]
    fn prop_pb_v8_conflict_ge0_prec(prec in -20i16..20i16) {
        let name = format!("pb_v8_cf4_{}", (prec + 100) as u16);
        let r = prec_grammar(&name, prec, Associativity::Right);
        let upper = r.build_stats.state_count * r.build_stats.symbol_count;
        prop_assert!(r.build_stats.conflict_cells <= upper);
    }
}

// ===========================================================================
// 10. Minimal grammar with various option combos (4 unit tests)
// ===========================================================================

#[test]
fn test_pb_v8_minimal_tt() {
    let g = GrammarBuilder::new("pb_v8_m0")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let opts = BuildOptions {
        out_dir: "/tmp/pb_v8_min".to_string(),
        emit_artifacts: true,
        compress_tables: true,
    };
    let r = build_parser(g, opts).unwrap();
    assert!(!r.parser_code.is_empty());
    assert!(r.build_stats.state_count >= 2);
}

#[test]
fn test_pb_v8_minimal_tf() {
    let g = GrammarBuilder::new("pb_v8_m1")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let opts = BuildOptions {
        out_dir: "/tmp/pb_v8_min".to_string(),
        emit_artifacts: true,
        compress_tables: false,
    };
    let r = build_parser(g, opts).unwrap();
    assert!(!r.parser_code.is_empty());
    assert!(r.build_stats.symbol_count >= 2);
}

#[test]
fn test_pb_v8_minimal_ft() {
    let g = GrammarBuilder::new("pb_v8_m2")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let opts = BuildOptions {
        out_dir: "/tmp/pb_v8_min".to_string(),
        emit_artifacts: false,
        compress_tables: true,
    };
    let r = build_parser(g, opts).unwrap();
    assert!(!r.node_types_json.is_empty());
}

#[test]
fn test_pb_v8_minimal_ff() {
    let g = GrammarBuilder::new("pb_v8_m3")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let opts = BuildOptions {
        out_dir: "/tmp/pb_v8_min".to_string(),
        emit_artifacts: false,
        compress_tables: false,
    };
    let r = build_parser(g, opts).unwrap();
    assert!(!r.parser_code.is_empty());
}

// ===========================================================================
// 11. Arithmetic grammar with all combos (4 unit tests)
// ===========================================================================

#[test]
fn test_pb_v8_arith_tt() {
    let r = arith_grammar("pb_v8_a0");
    assert!(!r.parser_code.is_empty());
    assert!(r.build_stats.state_count >= 2);
    assert!(r.build_stats.symbol_count >= 2);
}

#[test]
fn test_pb_v8_arith_tf() {
    let g = GrammarBuilder::new("pb_v8_a1")
        .token("num", r"\d+")
        .token("plus", r"\+")
        .token("star", r"\*")
        .rule("expr", vec!["num"])
        .rule_with_precedence("expr", vec!["expr", "plus", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "star", "expr"], 2, Associativity::Left)
        .start("expr")
        .build();
    let opts = BuildOptions {
        out_dir: "/tmp/pb_v8_arith".to_string(),
        emit_artifacts: true,
        compress_tables: false,
    };
    let r = build_parser(g, opts).unwrap();
    assert!(!r.parser_code.is_empty());
}

#[test]
fn test_pb_v8_arith_ft() {
    let g = GrammarBuilder::new("pb_v8_a2")
        .token("num", r"\d+")
        .token("plus", r"\+")
        .token("star", r"\*")
        .rule("expr", vec!["num"])
        .rule_with_precedence("expr", vec!["expr", "plus", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "star", "expr"], 2, Associativity::Left)
        .start("expr")
        .build();
    let opts = BuildOptions {
        out_dir: "/tmp/pb_v8_arith".to_string(),
        emit_artifacts: false,
        compress_tables: true,
    };
    let r = build_parser(g, opts).unwrap();
    let parsed: Result<serde_json::Value, _> = serde_json::from_str(&r.node_types_json);
    assert!(parsed.is_ok());
}

#[test]
fn test_pb_v8_arith_ff() {
    let g = GrammarBuilder::new("pb_v8_a3")
        .token("num", r"\d+")
        .token("plus", r"\+")
        .token("star", r"\*")
        .rule("expr", vec!["num"])
        .rule_with_precedence("expr", vec!["expr", "plus", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "star", "expr"], 2, Associativity::Left)
        .start("expr")
        .build();
    let opts = BuildOptions {
        out_dir: "/tmp/pb_v8_arith".to_string(),
        emit_artifacts: false,
        compress_tables: false,
    };
    let r = build_parser(g, opts).unwrap();
    assert!(r.build_stats.state_count >= 2);
}

// ===========================================================================
// 12. BuildStats Clone works (1 unit test)
// ===========================================================================

#[test]
fn test_pb_v8_stats_clone() {
    let r = minimal_grammar("pb_v8_sc0");
    let cloned = r.build_stats.clone();
    assert_eq!(cloned.state_count, r.build_stats.state_count);
    assert_eq!(cloned.symbol_count, r.build_stats.symbol_count);
    assert_eq!(cloned.conflict_cells, r.build_stats.conflict_cells);
}

// ===========================================================================
// 13. BuildOptions with empty out_dir (1 unit test)
// ===========================================================================

#[test]
fn test_pb_v8_empty_outdir() {
    let g = GrammarBuilder::new("pb_v8_eo0")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let opts = BuildOptions {
        out_dir: String::new(),
        emit_artifacts: false,
        compress_tables: true,
    };
    let r = build_parser(g, opts).unwrap();
    assert!(!r.parser_code.is_empty());
}

// ===========================================================================
// 14. Arbitrary token patterns (5 properties)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(8))]

    #[test]
    fn prop_pb_v8_token_pat_builds(pat in arb_token_pattern()) {
        let r = build_with_pattern("pb_v8_tp0", pat);
        prop_assert!(!r.parser_code.is_empty());
    }

    #[test]
    fn prop_pb_v8_token_pat_json(pat in arb_token_pattern()) {
        let r = build_with_pattern("pb_v8_tp1", pat);
        prop_assert!(!r.node_types_json.is_empty());
    }

    #[test]
    fn prop_pb_v8_token_pat_states(pat in arb_token_pattern()) {
        let r = build_with_pattern("pb_v8_tp2", pat);
        prop_assert!(r.build_stats.state_count >= 2);
    }

    #[test]
    fn prop_pb_v8_token_pat_symbols(pat in arb_token_pattern()) {
        let r = build_with_pattern("pb_v8_tp3", pat);
        prop_assert!(r.build_stats.symbol_count >= 2);
    }

    #[test]
    fn prop_pb_v8_token_pat_deterministic(pat in arb_token_pattern()) {
        let r1 = build_with_pattern("pb_v8_tp4", pat);
        let r2 = build_with_pattern("pb_v8_tp4", pat);
        prop_assert_eq!(r1.parser_code, r2.parser_code);
    }
}

// ===========================================================================
// 15. Arbitrary build options (5 properties)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(8))]

    #[test]
    fn prop_pb_v8_opts_builds(opts in arb_build_options()) {
        let g = GrammarBuilder::new("pb_v8_ob0")
            .token("a", "a").rule("s", vec!["a"]).start("s").build();
        let r = build_parser(g, opts);
        prop_assert!(r.is_ok());
    }

    #[test]
    fn prop_pb_v8_opts_code(opts in arb_build_options()) {
        let g = GrammarBuilder::new("pb_v8_ob1")
            .token("a", "a").rule("s", vec!["a"]).start("s").build();
        let r = build_parser(g, opts).unwrap();
        prop_assert!(!r.parser_code.is_empty());
    }

    #[test]
    fn prop_pb_v8_opts_json(opts in arb_build_options()) {
        let g = GrammarBuilder::new("pb_v8_ob2")
            .token("a", "a").token("b", "b")
            .rule("s", vec!["a"]).rule("s", vec!["b"]).start("s").build();
        let r = build_parser(g, opts).unwrap();
        prop_assert!(!r.node_types_json.is_empty());
    }

    #[test]
    fn prop_pb_v8_opts_states(opts in arb_build_options()) {
        let g = GrammarBuilder::new("pb_v8_ob3")
            .token("x", "x").rule("leaf", vec!["x"])
            .rule("mid", vec!["leaf"]).rule("s", vec!["mid"]).start("s").build();
        let r = build_parser(g, opts).unwrap();
        prop_assert!(r.build_stats.state_count >= 2);
    }

    #[test]
    fn prop_pb_v8_opts_symbols(opts in arb_build_options()) {
        let g = GrammarBuilder::new("pb_v8_ob4")
            .token("x", "x").token("op", "op")
            .rule("expr", vec!["x"])
            .rule_with_precedence("expr", vec!["expr", "op", "expr"], 1, Associativity::Left)
            .start("expr").build();
        let r = build_parser(g, opts).unwrap();
        prop_assert!(r.build_stats.symbol_count >= 2);
    }
}

// ===========================================================================
// 16. Grammar shape variants (9 unit tests)
// ===========================================================================

#[test]
fn test_pb_v8_shape_single_token() {
    let r = minimal_grammar("pb_v8_sh0");
    assert!(!r.parser_code.is_empty());
    assert!(r.build_stats.state_count >= 2);
}

#[test]
fn test_pb_v8_shape_two_alternatives() {
    let r = two_alt_grammar("pb_v8_sh1");
    assert!(r.build_stats.symbol_count >= 2);
}

#[test]
fn test_pb_v8_shape_three_deep_chain() {
    let r = chain_grammar("pb_v8_sh2");
    assert!(r.build_stats.state_count >= 2);
    assert!(!r.node_types_json.is_empty());
}

#[test]
fn test_pb_v8_shape_prec_left() {
    let r = prec_grammar("pb_v8_sh3", 1, Associativity::Left);
    assert!(!r.parser_code.is_empty());
}

#[test]
fn test_pb_v8_shape_prec_right() {
    let r = prec_grammar("pb_v8_sh4", 1, Associativity::Right);
    assert!(r.build_stats.state_count >= 2);
}

#[test]
fn test_pb_v8_shape_prec_none() {
    let r = prec_grammar("pb_v8_sh5", 1, Associativity::None);
    assert!(r.build_stats.state_count >= 2);
}

#[test]
fn test_pb_v8_shape_five_alts() {
    let r = n_alt_grammar("pb_v8_sh6", 5);
    assert!(r.build_stats.symbol_count >= 5);
}

#[test]
fn test_pb_v8_shape_five_concat() {
    let r = concat_grammar("pb_v8_sh7", 5);
    assert!(r.build_stats.state_count >= 2);
}

#[test]
fn test_pb_v8_shape_multi_nonterminal() {
    let g = GrammarBuilder::new("pb_v8_sh8")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("item", vec!["a"])
        .rule("item", vec!["b"])
        .rule("item", vec!["c"])
        .rule("s", vec!["item"])
        .start("s")
        .build();
    let r = build_parser(g, test_opts()).unwrap();
    assert!(r.build_stats.symbol_count >= 3);
    assert!(!r.parser_code.is_empty());
}

// ===========================================================================
// 17. Extra edge-case combos (10 unit tests)
// ===========================================================================

#[test]
fn test_pb_v8_extra_single_char_token() {
    let g = GrammarBuilder::new("pb_v8_x0")
        .token("x", "x")
        .rule("s", vec!["x"])
        .start("s")
        .build();
    let r = build_parser(g, test_opts()).unwrap();
    assert!(r.build_stats.state_count >= 2);
}

#[test]
fn test_pb_v8_extra_seven_alts() {
    let r = n_alt_grammar("pb_v8_x1", 7);
    assert!(r.build_stats.symbol_count >= 7);
    assert!(!r.parser_code.is_empty());
}

#[test]
fn test_pb_v8_extra_seven_concat() {
    let r = concat_grammar("pb_v8_x2", 7);
    assert!(r.build_stats.state_count >= 2);
}

#[test]
fn test_pb_v8_extra_prec_zero() {
    let r = prec_grammar("pb_v8_x3", 0, Associativity::Left);
    assert!(r.build_stats.state_count >= 2);
}

#[test]
fn test_pb_v8_extra_prec_negative() {
    let r = prec_grammar("pb_v8_x4", -50, Associativity::Right);
    assert!(!r.parser_code.is_empty());
}

#[test]
fn test_pb_v8_extra_prec_high_positive() {
    let r = prec_grammar("pb_v8_x5", 49, Associativity::None);
    assert!(!r.node_types_json.is_empty());
}

#[test]
fn test_pb_v8_extra_valid_json() {
    let r = arith_grammar("pb_v8_x6");
    let parsed: Result<serde_json::Value, _> = serde_json::from_str(&r.node_types_json);
    assert!(parsed.is_ok());
}

#[test]
fn test_pb_v8_extra_outdir_no_effect() {
    let g1 = GrammarBuilder::new("pb_v8_x7")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let g2 = GrammarBuilder::new("pb_v8_x7")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let opts1 = BuildOptions {
        out_dir: "/tmp/pb_v8_dir_a".to_string(),
        emit_artifacts: false,
        compress_tables: true,
    };
    let opts2 = BuildOptions {
        out_dir: "/tmp/pb_v8_dir_b".to_string(),
        emit_artifacts: false,
        compress_tables: true,
    };
    let r1 = build_parser(g1, opts1).unwrap();
    let r2 = build_parser(g2, opts2).unwrap();
    assert_eq!(r1.build_stats.state_count, r2.build_stats.state_count);
    assert_eq!(r1.build_stats.symbol_count, r2.build_stats.symbol_count);
}

#[test]
fn test_pb_v8_extra_scaling_alts() {
    let small = n_alt_grammar("pb_v8_x8a", 2);
    let large = n_alt_grammar("pb_v8_x8b", 4);
    assert!(large.parser_code.len() >= small.parser_code.len());
}

#[test]
fn test_pb_v8_extra_scaling_concat() {
    let small = concat_grammar("pb_v8_x9a", 2);
    let large = concat_grammar("pb_v8_x9b", 4);
    assert!(large.parser_code.len() >= small.parser_code.len());
}
