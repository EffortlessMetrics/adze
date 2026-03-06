//! Property-based tests for the adze-tool build pipeline.
//!
//! Tests exercise `build_parser` via `GrammarBuilder` to verify pipeline
//! properties: valid output, positive stats, determinism, monotonicity,
//! error handling, grammar variations, and edge cases.

use adze_ir::Associativity;
use adze_ir::builder::GrammarBuilder;
use adze_tool::pure_rust_builder::{BuildOptions, BuildResult, build_parser};
use proptest::prelude::*;

// ===========================================================================
// Helpers
// ===========================================================================

fn test_opts() -> BuildOptions {
    BuildOptions {
        out_dir: "/tmp/proptest_build_v2".to_string(),
        emit_artifacts: false,
        compress_tables: true,
    }
}

fn build_ok(name: &str, tokens: &[(&str, &str)], rules: &[(&str, Vec<&str>)]) -> BuildResult {
    let mut b = GrammarBuilder::new(name);
    for &(tname, tpat) in tokens {
        b = b.token(tname, tpat);
    }
    for (lhs, rhs) in rules {
        b = b.rule(lhs, rhs.clone());
    }
    if let Some((lhs, _)) = rules.last() {
        b = b.start(lhs);
    }
    build_parser(b.build(), test_opts()).expect("build_ok: build should succeed")
}

/// Build a grammar with `n` alternative rules: s -> t0 | t1 | … | t(n-1).
fn build_n_alts(name: &str, n: usize) -> BuildResult {
    let tok_names: Vec<String> = (0..n).map(|i| format!("t{i}")).collect();
    let tok_pats: Vec<String> = (0..n).map(|i| format!("t{i}")).collect();
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
    build_parser(b.build(), test_opts()).expect("build_n_alts: build should succeed")
}

// ===========================================================================
// Strategies
// ===========================================================================

fn grammar_name_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("alpha".to_string()),
        Just("beta".to_string()),
        Just("gamma".to_string()),
        Just("delta".to_string()),
        Just("epsilon".to_string()),
        Just("zeta".to_string()),
        Just("eta".to_string()),
        Just("theta".to_string()),
    ]
}

// ===========================================================================
// 1. Build produces valid table (5 proptest tests)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(8))]

    #[test]
    fn valid_table_parser_code_nonempty(name in grammar_name_strategy()) {
        let r = build_ok(&name, &[("a", "a")], &[("s", vec!["a"])]);
        prop_assert!(!r.parser_code.is_empty());
    }

    #[test]
    fn valid_table_node_types_nonempty(name in grammar_name_strategy()) {
        let r = build_ok(&name, &[("a", "a")], &[("s", vec!["a"])]);
        prop_assert!(!r.node_types_json.is_empty());
    }

    #[test]
    fn valid_table_node_types_is_json(name in grammar_name_strategy()) {
        let r = build_ok(&name, &[("a", "a")], &[("s", vec!["a"])]);
        let parsed: Result<serde_json::Value, _> = serde_json::from_str(&r.node_types_json);
        prop_assert!(parsed.is_ok(), "node_types_json must be valid JSON");
    }

    #[test]
    fn valid_table_grammar_name_preserved(name in grammar_name_strategy()) {
        let r = build_ok(&name, &[("a", "a")], &[("s", vec!["a"])]);
        prop_assert_eq!(r.grammar_name, name);
    }

    #[test]
    fn valid_table_parser_path_contains_name(name in grammar_name_strategy()) {
        let r = build_ok(&name, &[("a", "a")], &[("s", vec!["a"])]);
        prop_assert!(
            r.parser_path.contains(&name),
            "parser_path '{}' should contain grammar name '{}'",
            r.parser_path, name
        );
    }
}

// ===========================================================================
// 2. Stats are positive (5 proptest tests)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(8))]

    #[test]
    fn stats_state_count_positive(name in grammar_name_strategy()) {
        let r = build_ok(&name, &[("a", "a")], &[("s", vec!["a"])]);
        prop_assert!(r.build_stats.state_count > 0);
    }

    #[test]
    fn stats_symbol_count_positive(name in grammar_name_strategy()) {
        let r = build_ok(&name, &[("a", "a")], &[("s", vec!["a"])]);
        prop_assert!(r.build_stats.symbol_count > 0);
    }

    #[test]
    fn stats_two_tokens_more_symbols(name in grammar_name_strategy()) {
        let r = build_ok(
            &name,
            &[("a", "a"), ("b", "b")],
            &[("s", vec!["a"]), ("s", vec!["b"])],
        );
        // At least 2 terminal symbols + EOF + start nonterminal
        prop_assert!(r.build_stats.symbol_count >= 3);
    }

    #[test]
    fn stats_conflict_cells_non_negative(name in grammar_name_strategy()) {
        let r = build_ok(&name, &[("a", "a")], &[("s", vec!["a"])]);
        // conflict_cells is usize, always >= 0; just verify it doesn't panic
        let _ = r.build_stats.conflict_cells;
    }

    #[test]
    fn stats_debug_repr_nonempty(name in grammar_name_strategy()) {
        let r = build_ok(&name, &[("a", "a")], &[("s", vec!["a"])]);
        let debug = format!("{:?}", r.build_stats);
        prop_assert!(!debug.is_empty());
    }
}

// ===========================================================================
// 3. Build is deterministic (5 proptest tests)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(6))]

    #[test]
    fn deterministic_parser_code(name in grammar_name_strategy()) {
        let r1 = build_ok(&name, &[("a", "a")], &[("s", vec!["a"])]);
        let r2 = build_ok(&name, &[("a", "a")], &[("s", vec!["a"])]);
        prop_assert_eq!(r1.parser_code, r2.parser_code);
    }

    #[test]
    fn deterministic_node_types(name in grammar_name_strategy()) {
        let r1 = build_ok(&name, &[("a", "a")], &[("s", vec!["a"])]);
        let r2 = build_ok(&name, &[("a", "a")], &[("s", vec!["a"])]);
        prop_assert_eq!(r1.node_types_json, r2.node_types_json);
    }

    #[test]
    fn deterministic_state_count(name in grammar_name_strategy()) {
        let r1 = build_ok(&name, &[("a", "a")], &[("s", vec!["a"])]);
        let r2 = build_ok(&name, &[("a", "a")], &[("s", vec!["a"])]);
        prop_assert_eq!(r1.build_stats.state_count, r2.build_stats.state_count);
    }

    #[test]
    fn deterministic_symbol_count(name in grammar_name_strategy()) {
        let r1 = build_ok(&name, &[("a", "a")], &[("s", vec!["a"])]);
        let r2 = build_ok(&name, &[("a", "a")], &[("s", vec!["a"])]);
        prop_assert_eq!(r1.build_stats.symbol_count, r2.build_stats.symbol_count);
    }

    #[test]
    fn deterministic_two_tokens(name in grammar_name_strategy()) {
        let toks = [("a", "a"), ("b", "b")];
        let rules = [("s", vec!["a"]), ("s", vec!["b"])];
        let r1 = build_ok(&name, &toks, &rules);
        let r2 = build_ok(&name, &toks, &rules);
        prop_assert_eq!(r1.parser_code, r2.parser_code);
    }
}

// ===========================================================================
// 4. More rules → more symbols (5 proptest tests)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(6))]

    #[test]
    fn monotonic_alts_state_nondecreasing(n in 1..=4usize) {
        let r_small = build_n_alts("mono_s", n);
        let r_large = build_n_alts("mono_l", n + 1);
        prop_assert!(
            r_large.build_stats.state_count >= r_small.build_stats.state_count,
            "Adding alternatives should not decrease state count: {} vs {}",
            r_small.build_stats.state_count,
            r_large.build_stats.state_count,
        );
    }

    #[test]
    fn monotonic_alts_symbol_nondecreasing(n in 1..=4usize) {
        let r_small = build_n_alts("mono_s2", n);
        let r_large = build_n_alts("mono_l2", n + 1);
        prop_assert!(
            r_large.build_stats.symbol_count >= r_small.build_stats.symbol_count,
            "Adding alternatives should not decrease symbol count: {} vs {}",
            r_small.build_stats.symbol_count,
            r_large.build_stats.symbol_count,
        );
    }

    #[test]
    fn monotonic_code_length_nondecreasing(n in 1..=4usize) {
        let r_small = build_n_alts("mono_c", n);
        let r_large = build_n_alts("mono_d", n + 1);
        prop_assert!(
            r_large.parser_code.len() >= r_small.parser_code.len(),
            "More alternatives should produce equal or longer code: {} vs {}",
            r_small.parser_code.len(),
            r_large.parser_code.len(),
        );
    }

    #[test]
    fn monotonic_node_types_grows(n in 1..=4usize) {
        let r_small = build_n_alts("mono_n", n);
        let r_large = build_n_alts("mono_m", n + 1);
        prop_assert!(
            r_large.node_types_json.len() >= r_small.node_types_json.len(),
            "More alternatives should produce equal or longer node types: {} vs {}",
            r_small.node_types_json.len(),
            r_large.node_types_json.len(),
        );
    }

    #[test]
    fn monotonic_single_vs_multi(n in 2..=5usize) {
        let r_one = build_n_alts("one", 1);
        let r_many = build_n_alts("many", n);
        prop_assert!(
            r_many.build_stats.symbol_count >= r_one.build_stats.symbol_count,
            "Multi-alt grammar should have >= symbols than single-alt"
        );
    }
}

// ===========================================================================
// 5. Regular build tests (10 tests)
// ===========================================================================

#[test]
fn regular_single_token_grammar() {
    let r = build_ok("single", &[("a", "a")], &[("s", vec!["a"])]);
    assert_eq!(r.grammar_name, "single");
    assert!(r.build_stats.state_count > 0);
}

#[test]
fn regular_two_token_grammar() {
    let r = build_ok(
        "two",
        &[("a", "a"), ("b", "b")],
        &[("s", vec!["a"]), ("s", vec!["b"])],
    );
    assert!(r.build_stats.symbol_count >= 3);
}

#[test]
fn regular_concat_rule() {
    let r = build_ok(
        "concat",
        &[("a", "a"), ("b", "b")],
        &[("s", vec!["a", "b"])],
    );
    assert!(!r.parser_code.is_empty());
}

#[test]
fn regular_chain_rule() {
    let g = GrammarBuilder::new("chain")
        .token("x", "x")
        .rule("inner", vec!["x"])
        .rule("s", vec!["inner"])
        .start("s")
        .build();
    let r = build_parser(g, test_opts()).unwrap();
    assert!(r.build_stats.state_count > 0);
}

#[test]
fn regular_with_left_precedence() {
    let g = GrammarBuilder::new("leftprec")
        .token("a", "a")
        .token("plus", "plus")
        .rule("s", vec!["a"])
        .rule_with_precedence("s", vec!["s", "plus", "s"], 1, Associativity::Left)
        .start("s")
        .build();
    let r = build_parser(g, test_opts()).unwrap();
    assert!(r.build_stats.state_count > 0);
}

#[test]
fn regular_with_right_precedence() {
    let g = GrammarBuilder::new("rightprec")
        .token("a", "a")
        .token("eq", "eq")
        .rule("s", vec!["a"])
        .rule_with_precedence("s", vec!["s", "eq", "s"], 1, Associativity::Right)
        .start("s")
        .build();
    let r = build_parser(g, test_opts()).unwrap();
    assert!(r.build_stats.state_count > 0);
}

#[test]
fn regular_no_compression() {
    let opts = BuildOptions {
        out_dir: "/tmp/proptest_nocomp".to_string(),
        emit_artifacts: false,
        compress_tables: false,
    };
    let g = GrammarBuilder::new("nocomp")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let r = build_parser(g, opts).unwrap();
    assert!(!r.parser_code.is_empty());
}

#[test]
fn regular_emit_artifacts_false() {
    let r = build_ok("noartifact", &[("a", "a")], &[("s", vec!["a"])]);
    assert!(!r.parser_code.is_empty());
}

#[test]
fn regular_multi_rhs_three_tokens() {
    let r = build_ok(
        "triple",
        &[("a", "a"), ("b", "b"), ("c", "c")],
        &[("s", vec!["a", "b", "c"])],
    );
    assert!(r.build_stats.state_count > 0);
}

#[test]
fn regular_grammar_name_in_code() {
    let r = build_ok("marker", &[("a", "a")], &[("s", vec!["a"])]);
    assert!(
        r.parser_code.contains("marker"),
        "Generated code should reference grammar name"
    );
}

// ===========================================================================
// 6. Error handling tests (5 tests)
// ===========================================================================

#[test]
fn error_empty_grammar_fails() {
    let g = GrammarBuilder::new("empty").build();
    let result = build_parser(g, test_opts());
    // An empty grammar with no rules/tokens should fail
    assert!(result.is_err(), "Empty grammar should fail to build");
}

#[test]
fn error_no_start_symbol_still_builds() {
    // GrammarBuilder auto-selects a start symbol, so the build succeeds.
    // Verify the result is usable even without an explicit .start() call.
    let g = GrammarBuilder::new("nostart")
        .token("a", "a")
        .rule("orphan", vec!["a"])
        .build();
    let result = build_parser(g, test_opts());
    assert!(result.is_ok(), "Builder auto-selects a start symbol");
}

#[test]
fn error_result_contains_message() {
    let g = GrammarBuilder::new("empty_err").build();
    let result = build_parser(g, test_opts());
    if let Err(e) = result {
        let msg = format!("{e}");
        assert!(!msg.is_empty(), "Error message should not be empty");
    }
}

#[test]
fn error_options_clone_independence() {
    let opts1 = BuildOptions {
        out_dir: "/tmp/err_clone".to_string(),
        emit_artifacts: false,
        compress_tables: true,
    };
    let opts2 = opts1.clone();
    assert_eq!(opts1.out_dir, opts2.out_dir);
    assert_eq!(opts1.compress_tables, opts2.compress_tables);
}

#[test]
fn error_build_stats_debug() {
    let r = build_ok("dbg", &[("a", "a")], &[("s", vec!["a"])]);
    let debug_str = format!("{:?}", r.build_stats);
    assert!(debug_str.contains("state_count"));
    assert!(debug_str.contains("symbol_count"));
    assert!(debug_str.contains("conflict_cells"));
}

// ===========================================================================
// 7. Grammar variations (5 tests)
// ===========================================================================

#[test]
fn variation_five_alternatives() {
    let r = build_n_alts("five", 5);
    assert!(r.build_stats.symbol_count >= 5);
}

#[test]
fn variation_single_alternative() {
    let r = build_n_alts("solo", 1);
    assert!(r.build_stats.state_count > 0);
}

#[test]
fn variation_with_extras() {
    let g = GrammarBuilder::new("extras")
        .token("a", "a")
        .token("ws", "ws")
        .rule("s", vec!["a"])
        .extra("ws")
        .start("s")
        .build();
    let r = build_parser(g, test_opts()).unwrap();
    assert!(r.build_stats.symbol_count > 0);
}

#[test]
fn variation_nested_nonterminals() {
    let g = GrammarBuilder::new("nested")
        .token("x", "x")
        .rule("leaf", vec!["x"])
        .rule("mid", vec!["leaf"])
        .rule("top", vec!["mid"])
        .start("top")
        .build();
    let r = build_parser(g, test_opts()).unwrap();
    assert!(r.build_stats.state_count > 0);
}

#[test]
fn variation_two_level_precedence() {
    let g = GrammarBuilder::new("twoprec")
        .token("n", "n")
        .token("plus", "plus")
        .token("star", "star")
        .rule("e", vec!["n"])
        .rule_with_precedence("e", vec!["e", "plus", "e"], 1, Associativity::Left)
        .rule_with_precedence("e", vec!["e", "star", "e"], 2, Associativity::Left)
        .start("e")
        .build();
    let r = build_parser(g, test_opts()).unwrap();
    assert!(r.build_stats.state_count > 0);
}

// ===========================================================================
// 8. Edge cases (10 tests)
// ===========================================================================

#[test]
fn edge_single_char_grammar_name() {
    let r = build_ok("q", &[("a", "a")], &[("s", vec!["a"])]);
    assert_eq!(r.grammar_name, "q");
}

#[test]
fn edge_long_grammar_name() {
    let name = "a_very_long_grammar_name_for_testing";
    let r = build_ok(name, &[("a", "a")], &[("s", vec!["a"])]);
    assert_eq!(r.grammar_name, name);
}

#[test]
fn edge_same_grammar_twice() {
    let r1 = build_ok("dup", &[("a", "a")], &[("s", vec!["a"])]);
    let r2 = build_ok("dup", &[("a", "a")], &[("s", vec!["a"])]);
    assert_eq!(r1.parser_code, r2.parser_code);
}

#[test]
fn edge_build_options_default() {
    let opts = BuildOptions::default();
    assert!(!opts.emit_artifacts);
    assert!(opts.compress_tables);
}

#[test]
fn edge_compressed_vs_uncompressed_both_succeed() {
    let g1 = GrammarBuilder::new("comp")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let g2 = GrammarBuilder::new("uncomp")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let r1 = build_parser(
        g1,
        BuildOptions {
            compress_tables: true,
            ..test_opts()
        },
    );
    let r2 = build_parser(
        g2,
        BuildOptions {
            compress_tables: false,
            ..test_opts()
        },
    );
    assert!(r1.is_ok());
    assert!(r2.is_ok());
}

#[test]
fn edge_six_alternatives() {
    let r = build_n_alts("six", 6);
    assert!(r.build_stats.state_count > 0);
    assert!(r.build_stats.symbol_count >= 6);
}

#[test]
fn edge_node_types_json_is_array() {
    let r = build_ok("arr", &[("a", "a")], &[("s", vec!["a"])]);
    let v: serde_json::Value = serde_json::from_str(&r.node_types_json).unwrap();
    assert!(v.is_array(), "NODE_TYPES.json should be a JSON array");
}

#[test]
fn edge_parser_code_is_nonempty_rust() {
    let r = build_ok("commented", &[("a", "a")], &[("s", vec!["a"])]);
    // Generated code should be non-trivial Rust
    assert!(
        !r.parser_code.is_empty(),
        "Generated parser code should be non-empty"
    );
}

#[test]
fn edge_build_result_debug() {
    let r = build_ok("debugme", &[("a", "a")], &[("s", vec!["a"])]);
    let debug_str = format!("{r:?}");
    assert!(debug_str.contains("debugme"));
}

#[test]
fn edge_identical_token_names_different_grammars() {
    let r1 = build_ok("first", &[("a", "a")], &[("s", vec!["a"])]);
    let r2 = build_ok("second", &[("a", "a")], &[("s", vec!["a"])]);
    assert_ne!(r1.grammar_name, r2.grammar_name);
    // Both should succeed independently
    assert!(r1.build_stats.state_count > 0);
    assert!(r2.build_stats.state_count > 0);
}
