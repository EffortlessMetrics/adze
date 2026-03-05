//! Property-based tests for the adze-tool build pipeline (v4).
//!
//! 45+ proptest properties covering `build_parser` and `build_parser_from_json`
//! across nine categories: valid grammars, stats, parser code, node types,
//! determinism, size correlation, invalid inputs, grammar names, and edge cases.

use adze_ir::builder::GrammarBuilder;
use adze_tool::pure_rust_builder::{
    BuildOptions, BuildResult, build_parser, build_parser_from_json,
};
use proptest::prelude::*;

// ===========================================================================
// Helpers
// ===========================================================================

fn test_opts() -> BuildOptions {
    BuildOptions {
        out_dir: "/tmp/proptest_pipeline_v4".to_string(),
        emit_artifacts: false,
        compress_tables: true,
    }
}

/// Build a grammar from tokens and rules via GrammarBuilder.
fn build_ok(name: &str, tokens: &[(&str, &str)], rules: &[(&str, Vec<&str>)]) -> BuildResult {
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
    build_parser(b.build(), test_opts()).expect("build_ok failed")
}

/// Build from a JSON grammar string, returning the result.
fn build_json_ok(json: &str) -> BuildResult {
    build_parser_from_json(json.to_string(), test_opts()).expect("build_json_ok failed")
}

/// Produce a minimal JSON grammar with a STRING rule.
fn minimal_json(name: &str, value: &str) -> String {
    format!(
        r#"{{"name":"{}","rules":{{"start":{{"type":"STRING","value":"{}"}}}}}}"#,
        name, value
    )
}

/// Produce a JSON grammar with a PATTERN rule.
fn pattern_json(name: &str, pattern: &str) -> String {
    format!(
        r#"{{"name":"{}","rules":{{"start":{{"type":"PATTERN","value":"{}"}}}}}}"#,
        name, pattern
    )
}

/// Produce a JSON grammar with a SEQ of two STRING rules.
fn seq_json(name: &str, a: &str, b: &str) -> String {
    format!(
        r#"{{"name":"{}","rules":{{"start":{{"type":"SEQ","members":[{{"type":"STRING","value":"{}"}},{{"type":"STRING","value":"{}"}}]}}}}}}"#,
        name, a, b
    )
}

/// Produce a JSON grammar with a CHOICE of N STRING members.
fn choice_json(name: &str, members: &[&str]) -> String {
    let member_strs: Vec<String> = members
        .iter()
        .map(|m| format!(r#"{{"type":"STRING","value":"{}"}}"#, m))
        .collect();
    format!(
        r#"{{"name":"{}","rules":{{"start":{{"type":"CHOICE","members":[{}]}}}}}}"#,
        name,
        member_strs.join(",")
    )
}

/// Build n-alternative grammar via GrammarBuilder.
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
    build_parser(b.build(), test_opts()).expect("build_n_alts failed")
}

// ===========================================================================
// Strategies
// ===========================================================================

fn grammar_name_strategy() -> impl Strategy<Value = String> {
    "[a-z]{2,8}".prop_filter("avoid reserved words", |s| {
        !matches!(
            s.as_str(),
            "gen" | "do" | "self" | "type" | "fn" | "use" | "mod" | "pub"
        )
    })
}

fn token_name_strategy() -> impl Strategy<Value = String> {
    "[a-z]{2,8}".prop_filter("avoid reserved words", |s| {
        !matches!(
            s.as_str(),
            "gen" | "do" | "self" | "type" | "fn" | "use" | "mod" | "pub"
        )
    })
}

fn string_value_strategy() -> impl Strategy<Value = String> {
    "[a-z]{1,6}"
}

// ===========================================================================
// 1. Valid JSON grammars always produce BuildResult (5 properties)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(8))]

    #[test]
    fn valid_json_string_rule_produces_result(
        name in grammar_name_strategy(),
        val in string_value_strategy(),
    ) {
        let json = minimal_json(&name, &val);
        let r = build_parser_from_json(json, test_opts());
        prop_assert!(r.is_ok(), "STRING rule should always build: {:?}", r.err());
    }

    #[test]
    fn valid_json_pattern_rule_produces_result(name in grammar_name_strategy()) {
        let json = pattern_json(&name, "[a-z]+");
        let r = build_parser_from_json(json, test_opts());
        prop_assert!(r.is_ok(), "PATTERN rule should always build: {:?}", r.err());
    }

    #[test]
    fn valid_json_seq_produces_result(
        name in grammar_name_strategy(),
        a in string_value_strategy(),
        b in string_value_strategy(),
    ) {
        let json = seq_json(&name, &a, &b);
        let r = build_parser_from_json(json, test_opts());
        prop_assert!(r.is_ok(), "SEQ rule should always build: {:?}", r.err());
    }

    #[test]
    fn valid_json_choice_produces_result(
        name in grammar_name_strategy(),
        a in string_value_strategy(),
        b in string_value_strategy(),
    ) {
        let json = choice_json(&name, &[&a, &b]);
        let r = build_parser_from_json(json, test_opts());
        prop_assert!(r.is_ok(), "CHOICE rule should always build: {:?}", r.err());
    }

    #[test]
    fn valid_builder_single_token_produces_result(
        name in grammar_name_strategy(),
        tok in token_name_strategy(),
    ) {
        let r = build_parser(
            GrammarBuilder::new(&name).token(&tok, &tok).rule("s", vec![&*tok]).start("s").build(),
            test_opts(),
        );
        prop_assert!(r.is_ok(), "Single token grammar should build: {:?}", r.err());
    }
}

// ===========================================================================
// 2. Build stats are non-negative (5 properties)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(8))]

    #[test]
    fn stats_state_count_positive(name in grammar_name_strategy()) {
        let r = build_json_ok(&minimal_json(&name, "x"));
        prop_assert!(r.build_stats.state_count > 0);
    }

    #[test]
    fn stats_symbol_count_positive(name in grammar_name_strategy()) {
        let r = build_json_ok(&minimal_json(&name, "x"));
        prop_assert!(r.build_stats.symbol_count > 0);
    }

    #[test]
    fn stats_conflict_cells_non_negative(name in grammar_name_strategy()) {
        let r = build_json_ok(&minimal_json(&name, "x"));
        // usize is always >= 0; verify access doesn't panic
        let _ = r.build_stats.conflict_cells;
    }

    #[test]
    fn stats_all_fields_debug_non_empty(name in grammar_name_strategy()) {
        let r = build_json_ok(&minimal_json(&name, "y"));
        let debug = format!("{:?}", r.build_stats);
        prop_assert!(!debug.is_empty());
    }

    #[test]
    fn stats_builder_route_positive(
        name in grammar_name_strategy(),
        tok in token_name_strategy(),
    ) {
        let r = build_ok(&name, &[(&tok, &tok)], &[("s", vec![&*tok])]);
        prop_assert!(r.build_stats.state_count > 0);
        prop_assert!(r.build_stats.symbol_count > 0);
    }
}

// ===========================================================================
// 3. Parser code is non-empty for valid grammars (5 properties)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(8))]

    #[test]
    fn parser_code_nonempty_string(
        name in grammar_name_strategy(),
        val in string_value_strategy(),
    ) {
        let r = build_json_ok(&minimal_json(&name, &val));
        prop_assert!(!r.parser_code.is_empty());
    }

    #[test]
    fn parser_code_nonempty_pattern(name in grammar_name_strategy()) {
        let r = build_json_ok(&pattern_json(&name, "[0-9]+"));
        prop_assert!(!r.parser_code.is_empty());
    }

    #[test]
    fn parser_code_nonempty_seq(
        name in grammar_name_strategy(),
        a in string_value_strategy(),
        b in string_value_strategy(),
    ) {
        let r = build_json_ok(&seq_json(&name, &a, &b));
        prop_assert!(!r.parser_code.is_empty());
    }

    #[test]
    fn parser_code_nonempty_choice(
        name in grammar_name_strategy(),
        a in string_value_strategy(),
        b in string_value_strategy(),
    ) {
        let r = build_json_ok(&choice_json(&name, &[&a, &b]));
        prop_assert!(!r.parser_code.is_empty());
    }

    #[test]
    fn parser_code_nonempty_builder(
        name in grammar_name_strategy(),
        tok in token_name_strategy(),
    ) {
        let r = build_ok(&name, &[(&tok, &tok)], &[("s", vec![&*tok])]);
        prop_assert!(!r.parser_code.is_empty());
    }
}

// ===========================================================================
// 4. Node types is valid JSON (5 properties)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(8))]

    #[test]
    fn node_types_valid_json_string(
        name in grammar_name_strategy(),
        val in string_value_strategy(),
    ) {
        let r = build_json_ok(&minimal_json(&name, &val));
        let parsed: Result<serde_json::Value, _> = serde_json::from_str(&r.node_types_json);
        prop_assert!(parsed.is_ok(), "node_types_json must be valid JSON");
    }

    #[test]
    fn node_types_valid_json_pattern(name in grammar_name_strategy()) {
        let r = build_json_ok(&pattern_json(&name, "[a-z]+"));
        let parsed: Result<serde_json::Value, _> = serde_json::from_str(&r.node_types_json);
        prop_assert!(parsed.is_ok());
    }

    #[test]
    fn node_types_valid_json_seq(
        name in grammar_name_strategy(),
        a in string_value_strategy(),
        b in string_value_strategy(),
    ) {
        let r = build_json_ok(&seq_json(&name, &a, &b));
        let parsed: Result<serde_json::Value, _> = serde_json::from_str(&r.node_types_json);
        prop_assert!(parsed.is_ok());
    }

    #[test]
    fn node_types_is_json_array(
        name in grammar_name_strategy(),
        val in string_value_strategy(),
    ) {
        let r = build_json_ok(&minimal_json(&name, &val));
        let parsed: serde_json::Value = serde_json::from_str(&r.node_types_json).unwrap();
        prop_assert!(parsed.is_array(), "node_types_json should be a JSON array");
    }

    #[test]
    fn node_types_nonempty_builder(
        name in grammar_name_strategy(),
        tok in token_name_strategy(),
    ) {
        let r = build_ok(&name, &[(&tok, &tok)], &[("s", vec![&*tok])]);
        prop_assert!(!r.node_types_json.is_empty());
        let parsed: Result<serde_json::Value, _> = serde_json::from_str(&r.node_types_json);
        prop_assert!(parsed.is_ok());
    }
}

// ===========================================================================
// 5. Build is deterministic (5 properties)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(6))]

    #[test]
    fn deterministic_parser_code_json(
        name in grammar_name_strategy(),
        val in string_value_strategy(),
    ) {
        let json = minimal_json(&name, &val);
        let r1 = build_json_ok(&json);
        let r2 = build_json_ok(&json);
        prop_assert_eq!(r1.parser_code, r2.parser_code);
    }

    #[test]
    fn deterministic_node_types_json(
        name in grammar_name_strategy(),
        val in string_value_strategy(),
    ) {
        let json = minimal_json(&name, &val);
        let r1 = build_json_ok(&json);
        let r2 = build_json_ok(&json);
        prop_assert_eq!(r1.node_types_json, r2.node_types_json);
    }

    #[test]
    fn deterministic_state_count_json(
        name in grammar_name_strategy(),
        val in string_value_strategy(),
    ) {
        let json = minimal_json(&name, &val);
        let r1 = build_json_ok(&json);
        let r2 = build_json_ok(&json);
        prop_assert_eq!(r1.build_stats.state_count, r2.build_stats.state_count);
    }

    #[test]
    fn deterministic_symbol_count_json(
        name in grammar_name_strategy(),
        val in string_value_strategy(),
    ) {
        let json = minimal_json(&name, &val);
        let r1 = build_json_ok(&json);
        let r2 = build_json_ok(&json);
        prop_assert_eq!(r1.build_stats.symbol_count, r2.build_stats.symbol_count);
    }

    #[test]
    fn deterministic_builder_route(
        name in grammar_name_strategy(),
        tok in token_name_strategy(),
    ) {
        let r1 = build_ok(&name, &[(&tok, &tok)], &[("s", vec![&*tok])]);
        let r2 = build_ok(&name, &[(&tok, &tok)], &[("s", vec![&*tok])]);
        prop_assert_eq!(r1.parser_code, r2.parser_code);
        prop_assert_eq!(r1.node_types_json, r2.node_types_json);
    }
}

// ===========================================================================
// 6. Stats correlate with grammar size (5 properties)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(6))]

    #[test]
    fn more_alts_nondecreasing_state_count(n in 1..=4usize) {
        let r_small = build_n_alts("corr_s", n);
        let r_large = build_n_alts("corr_l", n + 1);
        prop_assert!(
            r_large.build_stats.state_count >= r_small.build_stats.state_count,
            "state_count should not decrease: {} vs {}",
            r_small.build_stats.state_count,
            r_large.build_stats.state_count,
        );
    }

    #[test]
    fn more_alts_nondecreasing_symbol_count(n in 1..=4usize) {
        let r_small = build_n_alts("sym_s", n);
        let r_large = build_n_alts("sym_l", n + 1);
        prop_assert!(
            r_large.build_stats.symbol_count >= r_small.build_stats.symbol_count,
            "symbol_count should not decrease: {} vs {}",
            r_small.build_stats.symbol_count,
            r_large.build_stats.symbol_count,
        );
    }

    #[test]
    fn more_alts_nondecreasing_code_length(n in 1..=4usize) {
        let r_small = build_n_alts("code_s", n);
        let r_large = build_n_alts("code_l", n + 1);
        prop_assert!(
            r_large.parser_code.len() >= r_small.parser_code.len(),
            "code length should not decrease: {} vs {}",
            r_small.parser_code.len(),
            r_large.parser_code.len(),
        );
    }

    #[test]
    fn more_alts_nondecreasing_node_types_length(n in 1..=4usize) {
        let r_small = build_n_alts("nt_s", n);
        let r_large = build_n_alts("nt_l", n + 1);
        prop_assert!(
            r_large.node_types_json.len() >= r_small.node_types_json.len(),
            "node_types length should not decrease: {} vs {}",
            r_small.node_types_json.len(),
            r_large.node_types_json.len(),
        );
    }

    #[test]
    fn multi_alt_has_more_symbols_than_single(n in 2..=5usize) {
        let r_one = build_n_alts("single", 1);
        let r_many = build_n_alts("multi", n);
        prop_assert!(
            r_many.build_stats.symbol_count >= r_one.build_stats.symbol_count,
            "multi-alt should have >= symbols: {} vs {}",
            r_many.build_stats.symbol_count,
            r_one.build_stats.symbol_count,
        );
    }
}

// ===========================================================================
// 7. Invalid inputs produce errors (5 properties)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))]

    #[test]
    fn invalid_empty_string_errors(_dummy in 0..1u8) {
        let r = build_parser_from_json(String::new(), test_opts());
        prop_assert!(r.is_err());
    }

    #[test]
    fn invalid_bare_number_errors(n in 0..1000i32) {
        let r = build_parser_from_json(n.to_string(), test_opts());
        prop_assert!(r.is_err());
    }

    #[test]
    fn invalid_no_rules_errors(name in grammar_name_strategy()) {
        let json = format!(r#"{{"name":"{}"}}"#, name);
        let r = build_parser_from_json(json, test_opts());
        prop_assert!(r.is_err());
    }

    #[test]
    fn invalid_garbage_errors(s in "[^{}\"]{1,20}") {
        let r = build_parser_from_json(s, test_opts());
        prop_assert!(r.is_err());
    }

    #[test]
    fn invalid_null_json_errors(_dummy in 0..1u8) {
        let r = build_parser_from_json("null".to_string(), test_opts());
        prop_assert!(r.is_err());
    }
}

// ===========================================================================
// 8. Grammar name survives pipeline (5 properties)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(8))]

    #[test]
    fn name_preserved_json_string(
        name in grammar_name_strategy(),
        val in string_value_strategy(),
    ) {
        let r = build_json_ok(&minimal_json(&name, &val));
        prop_assert_eq!(r.grammar_name, name);
    }

    #[test]
    fn name_preserved_json_pattern(name in grammar_name_strategy()) {
        let r = build_json_ok(&pattern_json(&name, "[a-z]+"));
        prop_assert_eq!(r.grammar_name, name);
    }

    #[test]
    fn name_preserved_json_seq(
        name in grammar_name_strategy(),
        a in string_value_strategy(),
        b in string_value_strategy(),
    ) {
        let r = build_json_ok(&seq_json(&name, &a, &b));
        prop_assert_eq!(r.grammar_name, name);
    }

    #[test]
    fn name_preserved_builder(
        name in grammar_name_strategy(),
        tok in token_name_strategy(),
    ) {
        let r = build_ok(&name, &[(&tok, &tok)], &[("s", vec![&*tok])]);
        prop_assert_eq!(r.grammar_name, name);
    }

    #[test]
    fn name_in_parser_path(
        name in grammar_name_strategy(),
        val in string_value_strategy(),
    ) {
        let r = build_json_ok(&minimal_json(&name, &val));
        prop_assert!(
            r.parser_path.contains(&name),
            "parser_path '{}' should contain name '{}'",
            r.parser_path, name
        );
    }
}

// ===========================================================================
// 9. Edge cases (6 properties)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(6))]

    #[test]
    fn edge_single_char_value(
        name in grammar_name_strategy(),
        ch in "[a-z]",
    ) {
        let r = build_json_ok(&minimal_json(&name, &ch));
        prop_assert!(!r.parser_code.is_empty());
        prop_assert_eq!(r.grammar_name, name);
    }

    #[test]
    fn edge_choice_three_members(
        name in grammar_name_strategy(),
        a in string_value_strategy(),
        b in string_value_strategy(),
        c in string_value_strategy(),
    ) {
        let json = choice_json(&name, &[&a, &b, &c]);
        let r = build_parser_from_json(json, test_opts());
        prop_assert!(r.is_ok());
    }

    #[test]
    fn edge_build_result_debug_nonempty(
        name in grammar_name_strategy(),
        val in string_value_strategy(),
    ) {
        let r = build_json_ok(&minimal_json(&name, &val));
        let debug = format!("{:?}", r);
        prop_assert!(!debug.is_empty());
    }

    #[test]
    fn edge_compressed_vs_uncompressed_both_succeed(
        name in grammar_name_strategy(),
        val in string_value_strategy(),
    ) {
        let json = minimal_json(&name, &val);
        let opts_compressed = BuildOptions {
            out_dir: "/tmp/proptest_v4_comp".to_string(),
            emit_artifacts: false,
            compress_tables: true,
        };
        let opts_uncompressed = BuildOptions {
            out_dir: "/tmp/proptest_v4_uncomp".to_string(),
            emit_artifacts: false,
            compress_tables: false,
        };
        let r1 = build_parser_from_json(json.clone(), opts_compressed);
        let r2 = build_parser_from_json(json, opts_uncompressed);
        prop_assert!(r1.is_ok());
        prop_assert!(r2.is_ok());
    }

    #[test]
    fn edge_grammar_name_preserved_across_opts(
        name in grammar_name_strategy(),
        val in string_value_strategy(),
        compress in proptest::bool::ANY,
    ) {
        let json = minimal_json(&name, &val);
        let opts = BuildOptions {
            out_dir: "/tmp/proptest_v4_opts".to_string(),
            emit_artifacts: false,
            compress_tables: compress,
        };
        let r = build_parser_from_json(json, opts).unwrap();
        prop_assert_eq!(r.grammar_name, name);
    }

    #[test]
    fn edge_repeated_builds_consistent(
        name in grammar_name_strategy(),
        val in string_value_strategy(),
    ) {
        let json = minimal_json(&name, &val);
        let r1 = build_json_ok(&json);
        let r2 = build_json_ok(&json);
        let r3 = build_json_ok(&json);
        prop_assert_eq!(&r1.parser_code, &r2.parser_code);
        prop_assert_eq!(&r2.parser_code, &r3.parser_code);
        prop_assert_eq!(r1.build_stats.state_count, r3.build_stats.state_count);
    }
}
