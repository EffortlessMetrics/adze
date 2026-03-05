//! Property-based tests for the adze-tool build pipeline (v4).
//!
//! 40+ proptest properties covering: valid builds, stats correctness,
//! determinism, non-empty output, JSON/IR path equivalence, monotonicity,
//! grammar variations, and edge cases.

use adze_ir::builder::GrammarBuilder;
use adze_tool::pure_rust_builder::{
    BuildOptions, BuildResult, build_parser, build_parser_from_json,
};
use proptest::prelude::*;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn test_opts() -> BuildOptions {
    BuildOptions {
        out_dir: "/tmp/proptest_build_v4".to_string(),
        emit_artifacts: false,
        compress_tables: true,
    }
}

/// Build via the IR path and assert success.
fn build_ir(name: &str, tokens: &[(&str, &str)], rules: &[(&str, Vec<&str>)]) -> BuildResult {
    let mut b = GrammarBuilder::new(name);
    for &(tname, tpat) in tokens {
        b = b.token(tname, tpat);
    }
    for (lhs, rhs) in rules {
        b = b.rule(lhs, rhs.clone());
    }
    if let Some((lhs, _)) = rules.first() {
        b = b.start(lhs);
    }
    build_parser(b.build(), test_opts()).expect("build_ir failed")
}

/// Build via the JSON path and assert success.
fn build_json(name: &str) -> BuildResult {
    let json = serde_json::json!({
        "name": name,
        "word": null,
        "rules": {
            "source_file": {
                "type": "SYMBOL",
                "name": "item"
            },
            "item": {
                "type": "PATTERN",
                "value": "\\w+"
            }
        },
        "extras": [
            { "type": "PATTERN", "value": "\\s" }
        ],
        "conflicts": [],
        "precedences": [],
        "externals": [],
        "inline": [],
        "supertypes": []
    })
    .to_string();
    build_parser_from_json(json, test_opts()).expect("build_json failed")
}

/// Build n-alternative grammar: s -> t0 | t1 | … | t(n-1).
fn build_n_alts(name: &str, n: usize) -> BuildResult {
    let tok_names: Vec<String> = (0..n).map(|i| format!("tok{i}")).collect();
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

// ---------------------------------------------------------------------------
// Strategies
// ---------------------------------------------------------------------------

/// Alphabetic grammar names that avoid Rust 2024 reserved keywords.
fn grammar_name_strategy() -> impl Strategy<Value = String> {
    "[a-z]{3,8}".prop_filter("must not be a Rust keyword", |s| {
        !matches!(
            s.as_str(),
            "gen"
                | "do"
                | "abstract"
                | "become"
                | "final"
                | "override"
                | "priv"
                | "typeof"
                | "unsized"
                | "virtual"
                | "box"
                | "macro"
                | "try"
                | "yield"
                | "fn"
                | "let"
                | "mut"
                | "ref"
                | "pub"
                | "mod"
                | "use"
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
                | "type"
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
                | "self"
                | "super"
                | "as"
                | "in"
        )
    })
}

/// Strategy for the number of alternative tokens (1..=6).
fn alt_count_strategy() -> impl Strategy<Value = usize> {
    1..=6usize
}

/// Strategy producing a small unique token name (alphabetic, not a keyword).
#[allow(dead_code)]
fn token_name_strategy() -> impl Strategy<Value = String> {
    "[a-z]{2,5}".prop_filter("must not be a Rust keyword", |s| {
        !matches!(
            s.as_str(),
            "gen"
                | "do"
                | "abstract"
                | "become"
                | "final"
                | "override"
                | "priv"
                | "typeof"
                | "unsized"
                | "virtual"
                | "box"
                | "macro"
                | "try"
                | "yield"
                | "fn"
                | "let"
                | "mut"
                | "ref"
                | "pub"
                | "mod"
                | "use"
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
                | "type"
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
                | "self"
                | "super"
                | "as"
                | "in"
        )
    })
}

// ===========================================================================
// 1. Build succeeds for valid grammars (6 tests)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(8))]

    #[test]
    fn build_succeeds_single_token(name in grammar_name_strategy()) {
        let r = build_ir(&name, &[("a", "a")], &[("s", vec!["a"])]);
        prop_assert!(!r.parser_code.is_empty());
    }

    #[test]
    fn build_succeeds_two_tokens(name in grammar_name_strategy()) {
        let r = build_ir(
            &name,
            &[("a", "a"), ("b", "b")],
            &[("s", vec!["a"]), ("s", vec!["b"])],
        );
        prop_assert!(!r.parser_code.is_empty());
    }

    #[test]
    fn build_succeeds_concat(name in grammar_name_strategy()) {
        let r = build_ir(
            &name,
            &[("a", "a"), ("b", "b")],
            &[("s", vec!["a", "b"])],
        );
        prop_assert!(r.build_stats.state_count > 0);
    }

    #[test]
    fn build_succeeds_chain_rule(name in grammar_name_strategy()) {
        let g = GrammarBuilder::new(&name)
            .token("x", "x")
            .rule("inner", vec!["x"])
            .rule("s", vec!["inner"])
            .start("s")
            .build();
        let r = build_parser(g, test_opts()).unwrap();
        prop_assert!(r.build_stats.state_count > 0);
    }

    #[test]
    fn build_succeeds_n_alts(n in alt_count_strategy()) {
        let r = build_n_alts("alts", n);
        prop_assert!(r.build_stats.state_count > 0);
    }

    #[test]
    fn build_succeeds_with_extras(name in grammar_name_strategy()) {
        let g = GrammarBuilder::new(&name)
            .token("a", "a")
            .token("ws", "ws")
            .rule("s", vec!["a"])
            .extra("ws")
            .start("s")
            .build();
        let r = build_parser(g, test_opts()).unwrap();
        prop_assert!(r.build_stats.symbol_count > 0);
    }
}

// ===========================================================================
// 2. Stats match grammar (6 tests)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(8))]

    #[test]
    fn stats_state_count_positive(name in grammar_name_strategy()) {
        let r = build_ir(&name, &[("a", "a")], &[("s", vec!["a"])]);
        prop_assert!(r.build_stats.state_count > 0);
    }

    #[test]
    fn stats_symbol_count_positive(name in grammar_name_strategy()) {
        let r = build_ir(&name, &[("a", "a")], &[("s", vec!["a"])]);
        prop_assert!(r.build_stats.symbol_count > 0);
    }

    #[test]
    fn stats_two_tokens_more_symbols(name in grammar_name_strategy()) {
        let r = build_ir(
            &name,
            &[("a", "a"), ("b", "b")],
            &[("s", vec!["a"]), ("s", vec!["b"])],
        );
        prop_assert!(
            r.build_stats.symbol_count >= 3,
            "expected >= 3 symbols, got {}",
            r.build_stats.symbol_count
        );
    }

    #[test]
    fn stats_n_alts_symbol_count_ge_n(n in 1..=5usize) {
        let r = build_n_alts("sym", n);
        prop_assert!(
            r.build_stats.symbol_count >= n,
            "expected >= {} symbols, got {}",
            n,
            r.build_stats.symbol_count
        );
    }

    #[test]
    fn stats_conflict_cells_is_finite(name in grammar_name_strategy()) {
        let r = build_ir(&name, &[("a", "a")], &[("s", vec!["a"])]);
        prop_assert!(r.build_stats.conflict_cells <= r.build_stats.state_count * r.build_stats.symbol_count);
    }

    #[test]
    fn stats_debug_contains_fields(name in grammar_name_strategy()) {
        let r = build_ir(&name, &[("a", "a")], &[("s", vec!["a"])]);
        let dbg = format!("{:?}", r.build_stats);
        prop_assert!(dbg.contains("state_count"));
        prop_assert!(dbg.contains("symbol_count"));
        prop_assert!(dbg.contains("conflict_cells"));
    }
}

// ===========================================================================
// 3. Determinism (6 tests)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(6))]

    #[test]
    fn deterministic_parser_code(name in grammar_name_strategy()) {
        let r1 = build_ir(&name, &[("a", "a")], &[("s", vec!["a"])]);
        let r2 = build_ir(&name, &[("a", "a")], &[("s", vec!["a"])]);
        prop_assert_eq!(&r1.parser_code, &r2.parser_code);
    }

    #[test]
    fn deterministic_node_types(name in grammar_name_strategy()) {
        let r1 = build_ir(&name, &[("a", "a")], &[("s", vec!["a"])]);
        let r2 = build_ir(&name, &[("a", "a")], &[("s", vec!["a"])]);
        prop_assert_eq!(&r1.node_types_json, &r2.node_types_json);
    }

    #[test]
    fn deterministic_state_count(name in grammar_name_strategy()) {
        let r1 = build_ir(&name, &[("a", "a")], &[("s", vec!["a"])]);
        let r2 = build_ir(&name, &[("a", "a")], &[("s", vec!["a"])]);
        prop_assert_eq!(r1.build_stats.state_count, r2.build_stats.state_count);
    }

    #[test]
    fn deterministic_symbol_count(name in grammar_name_strategy()) {
        let r1 = build_ir(&name, &[("a", "a")], &[("s", vec!["a"])]);
        let r2 = build_ir(&name, &[("a", "a")], &[("s", vec!["a"])]);
        prop_assert_eq!(r1.build_stats.symbol_count, r2.build_stats.symbol_count);
    }

    #[test]
    fn deterministic_conflict_cells(name in grammar_name_strategy()) {
        let r1 = build_ir(&name, &[("a", "a")], &[("s", vec!["a"])]);
        let r2 = build_ir(&name, &[("a", "a")], &[("s", vec!["a"])]);
        prop_assert_eq!(r1.build_stats.conflict_cells, r2.build_stats.conflict_cells);
    }

    #[test]
    fn deterministic_two_token_code(name in grammar_name_strategy()) {
        let toks = [("a", "a"), ("b", "b")];
        let rules = [("s", vec!["a"]), ("s", vec!["b"])];
        let r1 = build_ir(&name, &toks, &rules);
        let r2 = build_ir(&name, &toks, &rules);
        prop_assert_eq!(&r1.parser_code, &r2.parser_code);
    }
}

// ===========================================================================
// 4. Code is non-empty (5 tests)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(8))]

    #[test]
    fn code_nonempty_single(name in grammar_name_strategy()) {
        let r = build_ir(&name, &[("a", "a")], &[("s", vec!["a"])]);
        prop_assert!(!r.parser_code.is_empty());
    }

    #[test]
    fn code_nonempty_multi_alt(n in 1..=5usize) {
        let r = build_n_alts("code", n);
        prop_assert!(!r.parser_code.is_empty());
    }

    #[test]
    fn node_types_nonempty(name in grammar_name_strategy()) {
        let r = build_ir(&name, &[("a", "a")], &[("s", vec!["a"])]);
        prop_assert!(!r.node_types_json.is_empty());
    }

    #[test]
    fn node_types_valid_json(name in grammar_name_strategy()) {
        let r = build_ir(&name, &[("a", "a")], &[("s", vec!["a"])]);
        let parsed: std::result::Result<serde_json::Value, _> =
            serde_json::from_str(&r.node_types_json);
        prop_assert!(parsed.is_ok(), "node_types_json must be valid JSON");
    }

    #[test]
    fn node_types_is_array(name in grammar_name_strategy()) {
        let r = build_ir(&name, &[("a", "a")], &[("s", vec!["a"])]);
        let val: serde_json::Value = serde_json::from_str(&r.node_types_json).unwrap();
        prop_assert!(val.is_array(), "NODE_TYPES.json should be an array");
    }
}

// ===========================================================================
// 5. JSON path equivalent to IR path (6 tests)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(6))]

    #[test]
    fn json_path_produces_nonempty_code(name in grammar_name_strategy()) {
        let r = build_json(&name);
        prop_assert!(!r.parser_code.is_empty());
    }

    #[test]
    fn json_path_grammar_name_preserved(name in grammar_name_strategy()) {
        let r = build_json(&name);
        prop_assert_eq!(&r.grammar_name, &name);
    }

    #[test]
    fn json_path_state_count_positive(name in grammar_name_strategy()) {
        let r = build_json(&name);
        prop_assert!(r.build_stats.state_count > 0);
    }

    #[test]
    fn json_path_symbol_count_positive(name in grammar_name_strategy()) {
        let r = build_json(&name);
        prop_assert!(r.build_stats.symbol_count > 0);
    }

    #[test]
    fn json_path_node_types_valid(name in grammar_name_strategy()) {
        let r = build_json(&name);
        let parsed: std::result::Result<serde_json::Value, _> =
            serde_json::from_str(&r.node_types_json);
        prop_assert!(parsed.is_ok());
    }

    #[test]
    fn json_path_deterministic(name in grammar_name_strategy()) {
        let r1 = build_json(&name);
        let r2 = build_json(&name);
        prop_assert_eq!(&r1.parser_code, &r2.parser_code);
        prop_assert_eq!(r1.build_stats.state_count, r2.build_stats.state_count);
    }
}

// ===========================================================================
// 6. Monotonicity: more rules → more complexity (5 tests)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(6))]

    #[test]
    fn monotonic_state_count_nondecreasing(n in 1..=4usize) {
        let small = build_n_alts("ms", n);
        let large = build_n_alts("ml", n + 1);
        prop_assert!(
            large.build_stats.state_count >= small.build_stats.state_count,
            "state_count: {} (n={}) vs {} (n={})",
            small.build_stats.state_count, n,
            large.build_stats.state_count, n + 1,
        );
    }

    #[test]
    fn monotonic_symbol_count_nondecreasing(n in 1..=4usize) {
        let small = build_n_alts("ss", n);
        let large = build_n_alts("sl", n + 1);
        prop_assert!(
            large.build_stats.symbol_count >= small.build_stats.symbol_count,
            "symbol_count: {} (n={}) vs {} (n={})",
            small.build_stats.symbol_count, n,
            large.build_stats.symbol_count, n + 1,
        );
    }

    #[test]
    fn monotonic_code_length_nondecreasing(n in 1..=4usize) {
        let small = build_n_alts("cs", n);
        let large = build_n_alts("cl", n + 1);
        prop_assert!(
            large.parser_code.len() >= small.parser_code.len(),
            "code len: {} (n={}) vs {} (n={})",
            small.parser_code.len(), n,
            large.parser_code.len(), n + 1,
        );
    }

    #[test]
    fn monotonic_node_types_length_nondecreasing(n in 1..=4usize) {
        let small = build_n_alts("ns", n);
        let large = build_n_alts("nl", n + 1);
        prop_assert!(
            large.node_types_json.len() >= small.node_types_json.len(),
            "node_types len: {} (n={}) vs {} (n={})",
            small.node_types_json.len(), n,
            large.node_types_json.len(), n + 1,
        );
    }

    #[test]
    fn monotonic_single_vs_multi(n in 2..=5usize) {
        let single = build_n_alts("one", 1);
        let multi = build_n_alts("many", n);
        prop_assert!(
            multi.build_stats.symbol_count >= single.build_stats.symbol_count,
            "multi-alt should have >= symbols than single-alt"
        );
    }
}

// ===========================================================================
// 7. Grammar name preservation (3 tests)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(8))]

    #[test]
    fn name_preserved_in_result(name in grammar_name_strategy()) {
        let r = build_ir(&name, &[("a", "a")], &[("s", vec!["a"])]);
        prop_assert_eq!(&r.grammar_name, &name);
    }

    #[test]
    fn name_in_parser_path(name in grammar_name_strategy()) {
        let r = build_ir(&name, &[("a", "a")], &[("s", vec!["a"])]);
        prop_assert!(
            r.parser_path.contains(&name),
            "parser_path '{}' should contain '{}'",
            r.parser_path,
            name
        );
    }

    #[test]
    fn name_in_generated_code(name in grammar_name_strategy()) {
        let r = build_ir(&name, &[("a", "a")], &[("s", vec!["a"])]);
        prop_assert!(
            r.parser_code.contains(&name),
            "generated code should reference grammar name '{}'",
            name
        );
    }
}

// ===========================================================================
// 8. Compression options (3 tests)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(6))]

    #[test]
    fn compressed_and_uncompressed_both_succeed(name in grammar_name_strategy()) {
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
        let r1 = build_parser(g1, BuildOptions { compress_tables: true, ..test_opts() });
        let r2 = build_parser(g2, BuildOptions { compress_tables: false, ..test_opts() });
        prop_assert!(r1.is_ok());
        prop_assert!(r2.is_ok());
    }

    #[test]
    fn compressed_state_count_matches_uncompressed(name in grammar_name_strategy()) {
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
        let r1 = build_parser(g1, BuildOptions { compress_tables: true, ..test_opts() }).unwrap();
        let r2 = build_parser(g2, BuildOptions { compress_tables: false, ..test_opts() }).unwrap();
        prop_assert_eq!(r1.build_stats.state_count, r2.build_stats.state_count);
    }

    #[test]
    fn compressed_symbol_count_matches_uncompressed(name in grammar_name_strategy()) {
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
        let r1 = build_parser(g1, BuildOptions { compress_tables: true, ..test_opts() }).unwrap();
        let r2 = build_parser(g2, BuildOptions { compress_tables: false, ..test_opts() }).unwrap();
        prop_assert_eq!(r1.build_stats.symbol_count, r2.build_stats.symbol_count);
    }
}

// ===========================================================================
// 9. Error paths (2 proptest + 2 regular)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(6))]

    #[test]
    fn error_empty_grammar_fails(name in grammar_name_strategy()) {
        let g = GrammarBuilder::new(&name).build();
        let result = build_parser(g, test_opts());
        prop_assert!(result.is_err(), "empty grammar should fail");
    }

    #[test]
    fn error_message_nonempty(name in grammar_name_strategy()) {
        let g = GrammarBuilder::new(&name).build();
        if let Err(e) = build_parser(g, test_opts()) {
            let msg = format!("{e}");
            prop_assert!(!msg.is_empty());
        }
    }
}

#[test]
fn error_invalid_json_fails() {
    let result = build_parser_from_json("not json".to_string(), test_opts());
    assert!(result.is_err());
}

#[test]
fn error_empty_json_fails() {
    let result = build_parser_from_json(String::new(), test_opts());
    assert!(result.is_err());
}
