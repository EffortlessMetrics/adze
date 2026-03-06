//! Property-based tests for the adze-tool build pipeline (v6).
//!
//! 64 proptest properties across 13 categories:
//!   1.  prop_v6_ntok_*           — grammar with N tokens builds (5)
//!   2.  prop_v6_parser_code_*    — parser_code non-empty (5)
//!   3.  prop_v6_node_types_*     — node_types_json non-empty (5)
//!   4.  prop_v6_state_count_*    — state_count > 0 (5)
//!   5.  prop_v6_symbol_count_*   — symbol_count > 0 (5)
//!   6.  prop_v6_json_valid_*     — node_types_json is valid JSON (5)
//!   7.  prop_v6_det_state_*      — determinism: same state_count (5)
//!   8.  prop_v6_det_symbol_*     — determinism: same symbol_count (5)
//!   9.  prop_v6_compress_*       — compress_tables doesn't crash (5)
//!   10. prop_v6_emit_*           — emit_artifacts doesn't crash (5)
//!   11. prop_v6_prec_*           — precedence values build (5)
//!   12. prop_v6_assoc_*          — various Associativity builds (5)
//!   13. prop_v6_scaling_*        — code length, out_dir, conflict (4)

use adze_ir::Associativity;
use adze_ir::builder::GrammarBuilder;
use adze_tool::pure_rust_builder::{BuildOptions, BuildResult, build_parser};
use proptest::prelude::*;

// ===========================================================================
// Helpers
// ===========================================================================

fn test_opts() -> BuildOptions {
    BuildOptions {
        out_dir: "/tmp/proptest_pipeline_v6".to_string(),
        emit_artifacts: false,
        compress_tables: true,
    }
}

/// Build an n-alternative grammar: s -> tok0 | tok1 | … | tok(n-1).
fn build_n_alts(name: &str, n: usize) -> BuildResult {
    let tok_names: Vec<String> = (0..n).map(|i| format!("t{i}")).collect();
    let tok_pats: Vec<String> = (0..n).map(|i| format!("p{i}")).collect();
    let pairs: Vec<(&str, &str)> = tok_names
        .iter()
        .zip(tok_pats.iter())
        .map(|(a, b)| (a.as_str(), b.as_str()))
        .collect();
    let rules: Vec<(&str, Vec<&str>)> = tok_names.iter().map(|t| ("s", vec![t.as_str()])).collect();
    let mut b = GrammarBuilder::new(name);
    for &(tname, tpat) in &pairs {
        b = b.token(tname, tpat);
    }
    for (lhs, rhs) in &rules {
        b = b.rule(lhs, rhs.clone());
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
    let pairs: Vec<(&str, &str)> = tok_names
        .iter()
        .zip(tok_pats.iter())
        .map(|(a, b)| (a.as_str(), b.as_str()))
        .collect();
    let rhs: Vec<&str> = tok_names.iter().map(|t| t.as_str()).collect();
    let mut b = GrammarBuilder::new(name);
    for &(tname, tpat) in &pairs {
        b = b.token(tname, tpat);
    }
    b = b.rule("s", rhs).start("s");
    build_parser(b.build(), test_opts()).expect("build_concat failed")
}

// ===========================================================================
// Strategies
// ===========================================================================

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
// 1. Grammar with N tokens (1..6) → build succeeds (5 properties)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(8))]

    #[test]
    fn prop_v6_ntok_single(_i in 0..1u8) {
        let r = build_n_alts("pp_v6_0", 1);
        prop_assert!(!r.parser_code.is_empty());
    }

    #[test]
    fn prop_v6_ntok_two(_i in 0..1u8) {
        let r = build_n_alts("pp_v6_1", 2);
        prop_assert!(r.build_stats.state_count > 0);
    }

    #[test]
    fn prop_v6_ntok_three(_i in 0..1u8) {
        let r = build_n_alts("pp_v6_2", 3);
        prop_assert!(r.build_stats.symbol_count > 0);
    }

    #[test]
    fn prop_v6_ntok_range(n in 1..6usize) {
        let name = format!("pp_v6_{}", n + 10);
        let r = build_n_alts(&name, n);
        prop_assert!(!r.parser_code.is_empty());
    }

    #[test]
    fn prop_v6_ntok_concat_range(n in 1..6usize) {
        let name = format!("pp_v6_{}", n + 20);
        let r = build_concat(&name, n);
        prop_assert!(r.build_stats.state_count > 0);
    }
}

// ===========================================================================
// 2. parser_code non-empty for any valid grammar (5 properties)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(8))]

    #[test]
    fn prop_v6_parser_code_single(name in grammar_name_strategy()) {
        let r = build_single(&name);
        prop_assert!(!r.parser_code.is_empty());
    }

    #[test]
    fn prop_v6_parser_code_two_alt(name in grammar_name_strategy()) {
        let r = build_two_alt(&name);
        prop_assert!(!r.parser_code.is_empty());
    }

    #[test]
    fn prop_v6_parser_code_n_alts(n in 1..6usize) {
        let name = format!("pp_v6_{}", n + 30);
        let r = build_n_alts(&name, n);
        prop_assert!(!r.parser_code.is_empty());
    }

    #[test]
    fn prop_v6_parser_code_concat(n in 1..6usize) {
        let name = format!("pp_v6_{}", n + 40);
        let r = build_concat(&name, n);
        prop_assert!(!r.parser_code.is_empty());
    }

    #[test]
    fn prop_v6_parser_code_prec(prec in -50i16..50i16) {
        let name = format!("pp_v6_{}", (prec + 100) as u16);
        let r = build_prec(&name, prec, Associativity::Left);
        prop_assert!(!r.parser_code.is_empty());
    }
}

// ===========================================================================
// 3. node_types_json non-empty for any valid grammar (5 properties)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(8))]

    #[test]
    fn prop_v6_node_types_single(name in grammar_name_strategy()) {
        let r = build_single(&name);
        prop_assert!(!r.node_types_json.is_empty());
    }

    #[test]
    fn prop_v6_node_types_two_alt(name in grammar_name_strategy()) {
        let r = build_two_alt(&name);
        prop_assert!(!r.node_types_json.is_empty());
    }

    #[test]
    fn prop_v6_node_types_n_alts(n in 1..6usize) {
        let name = format!("pp_v6_{}", n + 50);
        let r = build_n_alts(&name, n);
        prop_assert!(!r.node_types_json.is_empty());
    }

    #[test]
    fn prop_v6_node_types_concat(n in 1..6usize) {
        let name = format!("pp_v6_{}", n + 60);
        let r = build_concat(&name, n);
        prop_assert!(!r.node_types_json.is_empty());
    }

    #[test]
    fn prop_v6_node_types_chain(name in grammar_name_strategy()) {
        let g = GrammarBuilder::new(&name)
            .token("x", "x")
            .rule("leaf", vec!["x"])
            .rule("s", vec!["leaf"])
            .start("s")
            .build();
        let r = build_parser(g, test_opts()).unwrap();
        prop_assert!(!r.node_types_json.is_empty());
    }
}

// ===========================================================================
// 4. state_count > 0 for any valid grammar (5 properties)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(8))]

    #[test]
    fn prop_v6_state_count_single(name in grammar_name_strategy()) {
        let r = build_single(&name);
        prop_assert!(r.build_stats.state_count > 0);
    }

    #[test]
    fn prop_v6_state_count_two_alt(name in grammar_name_strategy()) {
        let r = build_two_alt(&name);
        prop_assert!(r.build_stats.state_count > 0);
    }

    #[test]
    fn prop_v6_state_count_n_alts(n in 1..6usize) {
        let name = format!("pp_v6_{}", n + 70);
        let r = build_n_alts(&name, n);
        prop_assert!(r.build_stats.state_count > 0);
    }

    #[test]
    fn prop_v6_state_count_concat(n in 1..6usize) {
        let name = format!("pp_v6_{}", n + 80);
        let r = build_concat(&name, n);
        prop_assert!(r.build_stats.state_count > 0);
    }

    #[test]
    fn prop_v6_state_count_prec(prec in -50i16..50i16) {
        let name = format!("pp_v6_{}", (prec + 200) as u16);
        let r = build_prec(&name, prec, Associativity::Left);
        prop_assert!(r.build_stats.state_count > 0);
    }
}

// ===========================================================================
// 5. symbol_count > 0 for any valid grammar (5 properties)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(8))]

    #[test]
    fn prop_v6_symbol_count_single(name in grammar_name_strategy()) {
        let r = build_single(&name);
        prop_assert!(r.build_stats.symbol_count > 0);
    }

    #[test]
    fn prop_v6_symbol_count_two_alt(name in grammar_name_strategy()) {
        let r = build_two_alt(&name);
        prop_assert!(r.build_stats.symbol_count > 0);
    }

    #[test]
    fn prop_v6_symbol_count_n_alts(n in 1..6usize) {
        let name = format!("pp_v6_{}", n + 90);
        let r = build_n_alts(&name, n);
        prop_assert!(r.build_stats.symbol_count > 0);
    }

    #[test]
    fn prop_v6_symbol_count_concat(n in 1..6usize) {
        let name = format!("pp_v6_{}", n + 100);
        let r = build_concat(&name, n);
        prop_assert!(r.build_stats.symbol_count > 0);
    }

    #[test]
    fn prop_v6_symbol_count_chain(name in grammar_name_strategy()) {
        let g = GrammarBuilder::new(&name)
            .token("x", "x")
            .rule("leaf", vec!["x"])
            .rule("mid", vec!["leaf"])
            .rule("s", vec!["mid"])
            .start("s")
            .build();
        let r = build_parser(g, test_opts()).unwrap();
        prop_assert!(r.build_stats.symbol_count > 0);
    }
}

// ===========================================================================
// 6. node_types_json is valid JSON for any grammar (5 properties)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(8))]

    #[test]
    fn prop_v6_json_valid_single(name in grammar_name_strategy()) {
        let r = build_single(&name);
        let parsed: Result<serde_json::Value, _> = serde_json::from_str(&r.node_types_json);
        prop_assert!(parsed.is_ok(), "node_types_json not valid JSON: {:?}", parsed.err());
    }

    #[test]
    fn prop_v6_json_valid_two_alt(name in grammar_name_strategy()) {
        let r = build_two_alt(&name);
        let parsed: Result<serde_json::Value, _> = serde_json::from_str(&r.node_types_json);
        prop_assert!(parsed.is_ok());
    }

    #[test]
    fn prop_v6_json_valid_n_alts(n in 1..6usize) {
        let name = format!("pp_v6_{}", n + 110);
        let r = build_n_alts(&name, n);
        let parsed: Result<serde_json::Value, _> = serde_json::from_str(&r.node_types_json);
        prop_assert!(parsed.is_ok());
    }

    #[test]
    fn prop_v6_json_valid_concat(n in 1..6usize) {
        let name = format!("pp_v6_{}", n + 120);
        let r = build_concat(&name, n);
        let parsed: Result<serde_json::Value, _> = serde_json::from_str(&r.node_types_json);
        prop_assert!(parsed.is_ok());
    }

    #[test]
    fn prop_v6_json_valid_prec(prec in -50i16..50i16) {
        let name = format!("pp_v6_{}", (prec + 300) as u16);
        let r = build_prec(&name, prec, Associativity::Left);
        let parsed: Result<serde_json::Value, _> = serde_json::from_str(&r.node_types_json);
        prop_assert!(parsed.is_ok());
    }
}

// ===========================================================================
// 7. Determinism: same grammar spec → same state_count (5 properties)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(8))]

    #[test]
    fn prop_v6_det_state_single(name in grammar_name_strategy()) {
        let r1 = build_single(&name);
        let r2 = build_single(&name);
        prop_assert_eq!(r1.build_stats.state_count, r2.build_stats.state_count);
    }

    #[test]
    fn prop_v6_det_state_two_alt(name in grammar_name_strategy()) {
        let r1 = build_two_alt(&name);
        let r2 = build_two_alt(&name);
        prop_assert_eq!(r1.build_stats.state_count, r2.build_stats.state_count);
    }

    #[test]
    fn prop_v6_det_state_n_alts(n in 1..6usize) {
        let name = format!("pp_v6_{}", n + 130);
        let r1 = build_n_alts(&name, n);
        let r2 = build_n_alts(&name, n);
        prop_assert_eq!(r1.build_stats.state_count, r2.build_stats.state_count);
    }

    #[test]
    fn prop_v6_det_state_concat(n in 1..6usize) {
        let name = format!("pp_v6_{}", n + 140);
        let r1 = build_concat(&name, n);
        let r2 = build_concat(&name, n);
        prop_assert_eq!(r1.build_stats.state_count, r2.build_stats.state_count);
    }

    #[test]
    fn prop_v6_det_state_prec(prec in -50i16..50i16) {
        let name = format!("pp_v6_{}", (prec + 400) as u16);
        let r1 = build_prec(&name, prec, Associativity::Left);
        let r2 = build_prec(&name, prec, Associativity::Left);
        prop_assert_eq!(r1.build_stats.state_count, r2.build_stats.state_count);
    }
}

// ===========================================================================
// 8. Determinism: same grammar spec → same symbol_count (5 properties)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(8))]

    #[test]
    fn prop_v6_det_symbol_single(name in grammar_name_strategy()) {
        let r1 = build_single(&name);
        let r2 = build_single(&name);
        prop_assert_eq!(r1.build_stats.symbol_count, r2.build_stats.symbol_count);
    }

    #[test]
    fn prop_v6_det_symbol_two_alt(name in grammar_name_strategy()) {
        let r1 = build_two_alt(&name);
        let r2 = build_two_alt(&name);
        prop_assert_eq!(r1.build_stats.symbol_count, r2.build_stats.symbol_count);
    }

    #[test]
    fn prop_v6_det_symbol_n_alts(n in 1..6usize) {
        let name = format!("pp_v6_{}", n + 150);
        let r1 = build_n_alts(&name, n);
        let r2 = build_n_alts(&name, n);
        prop_assert_eq!(r1.build_stats.symbol_count, r2.build_stats.symbol_count);
    }

    #[test]
    fn prop_v6_det_symbol_concat(n in 1..6usize) {
        let name = format!("pp_v6_{}", n + 160);
        let r1 = build_concat(&name, n);
        let r2 = build_concat(&name, n);
        prop_assert_eq!(r1.build_stats.symbol_count, r2.build_stats.symbol_count);
    }

    #[test]
    fn prop_v6_det_symbol_prec(prec in -50i16..50i16) {
        let name = format!("pp_v6_{}", (prec + 500) as u16);
        let r1 = build_prec(&name, prec, Associativity::Left);
        let r2 = build_prec(&name, prec, Associativity::Left);
        prop_assert_eq!(r1.build_stats.symbol_count, r2.build_stats.symbol_count);
    }
}

// ===========================================================================
// 9. compress_tables doesn't crash for any grammar (5 properties)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(8))]

    #[test]
    fn prop_v6_compress_single(name in grammar_name_strategy()) {
        let g = GrammarBuilder::new(&name)
            .token("a", "a")
            .rule("s", vec!["a"])
            .start("s")
            .build();
        let r = build_parser(g, BuildOptions { compress_tables: true, ..test_opts() });
        prop_assert!(r.is_ok());
    }

    #[test]
    fn prop_v6_compress_false_single(name in grammar_name_strategy()) {
        let g = GrammarBuilder::new(&name)
            .token("a", "a")
            .rule("s", vec!["a"])
            .start("s")
            .build();
        let r = build_parser(g, BuildOptions { compress_tables: false, ..test_opts() });
        prop_assert!(r.is_ok());
    }

    #[test]
    fn prop_v6_compress_two_alt(name in grammar_name_strategy()) {
        let g = GrammarBuilder::new(&name)
            .token("a", "a")
            .token("b", "b")
            .rule("s", vec!["a"])
            .rule("s", vec!["b"])
            .start("s")
            .build();
        let r = build_parser(g, BuildOptions { compress_tables: true, ..test_opts() });
        prop_assert!(r.is_ok());
    }

    #[test]
    fn prop_v6_compress_n_alts(n in 1..6usize) {
        let name = format!("pp_v6_{}", n + 170);
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
    fn prop_v6_compress_prec(prec in -50i16..50i16) {
        let name = format!("pp_v6_{}", (prec + 600) as u16);
        let g = GrammarBuilder::new(&name)
            .token("x", "x")
            .token("op", "op")
            .rule("expr", vec!["x"])
            .rule_with_precedence("expr", vec!["expr", "op", "expr"], prec, Associativity::Left)
            .start("expr")
            .build();
        let r = build_parser(g, BuildOptions { compress_tables: true, ..test_opts() });
        prop_assert!(r.is_ok());
    }
}

// ===========================================================================
// 10. emit_artifacts doesn't crash for any grammar (5 properties)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(8))]

    #[test]
    fn prop_v6_emit_true_single(name in grammar_name_strategy()) {
        let g = GrammarBuilder::new(&name)
            .token("a", "a")
            .rule("s", vec!["a"])
            .start("s")
            .build();
        let r = build_parser(g, BuildOptions { emit_artifacts: true, ..test_opts() });
        prop_assert!(r.is_ok());
    }

    #[test]
    fn prop_v6_emit_false_single(name in grammar_name_strategy()) {
        let g = GrammarBuilder::new(&name)
            .token("a", "a")
            .rule("s", vec!["a"])
            .start("s")
            .build();
        let r = build_parser(g, BuildOptions { emit_artifacts: false, ..test_opts() });
        prop_assert!(r.is_ok());
    }

    #[test]
    fn prop_v6_emit_true_two_alt(name in grammar_name_strategy()) {
        let g = GrammarBuilder::new(&name)
            .token("a", "a")
            .token("b", "b")
            .rule("s", vec!["a"])
            .rule("s", vec!["b"])
            .start("s")
            .build();
        let r = build_parser(g, BuildOptions { emit_artifacts: true, ..test_opts() });
        prop_assert!(r.is_ok());
    }

    #[test]
    fn prop_v6_emit_n_alts(n in 1..6usize) {
        let name = format!("pp_v6_{}", n + 180);
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
    fn prop_v6_emit_both_flags(name in grammar_name_strategy()) {
        let g = GrammarBuilder::new(&name)
            .token("a", "a")
            .rule("s", vec!["a"])
            .start("s")
            .build();
        let opts = BuildOptions {
            out_dir: "/tmp/proptest_pipeline_v6_emit".to_string(),
            emit_artifacts: true,
            compress_tables: true,
        };
        let r = build_parser(g, opts);
        prop_assert!(r.is_ok());
    }
}

// ===========================================================================
// 11. Grammar with precedence values (-50..50) → builds (5 properties)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(8))]

    #[test]
    fn prop_v6_prec_left(prec in -50i16..50i16) {
        let name = format!("pp_v6_{}", (prec + 700) as u16);
        let r = build_prec(&name, prec, Associativity::Left);
        prop_assert!(r.build_stats.state_count > 0);
    }

    #[test]
    fn prop_v6_prec_right(prec in -50i16..50i16) {
        let name = format!("pp_v6_{}", (prec + 800) as u16);
        let r = build_prec(&name, prec, Associativity::Right);
        prop_assert!(r.build_stats.state_count > 0);
    }

    #[test]
    fn prop_v6_prec_none(prec in -50i16..50i16) {
        let name = format!("pp_v6_{}", (prec + 900) as u16);
        let r = build_prec(&name, prec, Associativity::None);
        prop_assert!(r.build_stats.state_count > 0);
    }

    #[test]
    fn prop_v6_prec_zero(_i in 0..1u8) {
        let r = build_prec("pp_v6_3000", 0, Associativity::Left);
        prop_assert!(!r.parser_code.is_empty());
    }

    #[test]
    fn prop_v6_prec_extremes(extreme in prop::sample::select(vec![-50i16, -1, 0, 1, 49])) {
        let name = format!("pp_v6_{}", (extreme + 1000) as u16);
        let r = build_prec(&name, extreme, Associativity::Left);
        prop_assert!(r.build_stats.symbol_count > 0);
    }
}

// ===========================================================================
// 12. Grammar with various Associativity → builds (5 properties)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(8))]

    #[test]
    fn prop_v6_assoc_left_builds(_i in 0..1u8) {
        let r = build_prec("pp_v6_4000", 1, Associativity::Left);
        prop_assert!(r.build_stats.state_count > 0);
    }

    #[test]
    fn prop_v6_assoc_right_builds(_i in 0..1u8) {
        let r = build_prec("pp_v6_4001", 1, Associativity::Right);
        prop_assert!(r.build_stats.state_count > 0);
    }

    #[test]
    fn prop_v6_assoc_none_builds(_i in 0..1u8) {
        let r = build_prec("pp_v6_4002", 1, Associativity::None);
        prop_assert!(r.build_stats.state_count > 0);
    }

    #[test]
    fn prop_v6_assoc_all_produce_code(
        assoc_idx in 0u8..3u8,
    ) {
        let assoc = match assoc_idx {
            0 => Associativity::Left,
            1 => Associativity::Right,
            _ => Associativity::None,
        };
        let name = format!("pp_v6_{}", 4010 + assoc_idx as u16);
        let r = build_prec(&name, 5, assoc);
        prop_assert!(!r.parser_code.is_empty());
    }

    #[test]
    fn prop_v6_assoc_all_valid_json(
        assoc_idx in 0u8..3u8,
    ) {
        let assoc = match assoc_idx {
            0 => Associativity::Left,
            1 => Associativity::Right,
            _ => Associativity::None,
        };
        let name = format!("pp_v6_{}", 4020 + assoc_idx as u16);
        let r = build_prec(&name, 5, assoc);
        let parsed: Result<serde_json::Value, _> = serde_json::from_str(&r.node_types_json);
        prop_assert!(parsed.is_ok());
    }
}

// ===========================================================================
// 13. Scaling, out_dir, conflict_cells (4 properties)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(8))]

    #[test]
    fn prop_v6_scaling_code_length(n in 1..4usize) {
        let small = build_n_alts(&format!("pp_v6_{}", n + 5000), n);
        let large = build_n_alts(&format!("pp_v6_{}", n + 5010), n + 1);
        prop_assert!(
            large.parser_code.len() >= small.parser_code.len(),
            "code len {} (n={}) should be <= {} (n={})",
            small.parser_code.len(), n,
            large.parser_code.len(), n + 1,
        );
    }

    #[test]
    fn prop_v6_scaling_out_dir_no_effect_on_stats(name in grammar_name_strategy()) {
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
            out_dir: "/tmp/proptest_v6_dir_a".to_string(),
            emit_artifacts: false,
            compress_tables: true,
        };
        let opts2 = BuildOptions {
            out_dir: "/tmp/proptest_v6_dir_b".to_string(),
            emit_artifacts: false,
            compress_tables: true,
        };
        let r1 = build_parser(g1, opts1).unwrap();
        let r2 = build_parser(g2, opts2).unwrap();
        prop_assert_eq!(r1.build_stats.state_count, r2.build_stats.state_count);
        prop_assert_eq!(r1.build_stats.symbol_count, r2.build_stats.symbol_count);
        prop_assert_eq!(r1.build_stats.conflict_cells, r2.build_stats.conflict_cells);
    }

    #[test]
    fn prop_v6_scaling_conflict_bounded(n in 1..6usize) {
        let name = format!("pp_v6_{}", n + 6000);
        let r = build_n_alts(&name, n);
        let upper = r.build_stats.state_count * r.build_stats.symbol_count;
        prop_assert!(
            r.build_stats.conflict_cells <= upper,
            "conflicts {} > upper bound {}",
            r.build_stats.conflict_cells, upper,
        );
    }

    #[test]
    fn prop_v6_scaling_symbol_monotonic(n in 1..4usize) {
        let small = build_n_alts(&format!("pp_v6_{}", n + 7000), n);
        let large = build_n_alts(&format!("pp_v6_{}", n + 7010), n + 1);
        prop_assert!(
            large.build_stats.symbol_count >= small.build_stats.symbol_count,
            "symbol_count {} (n={}) should be <= {} (n={})",
            small.build_stats.symbol_count, n,
            large.build_stats.symbol_count, n + 1,
        );
    }
}
