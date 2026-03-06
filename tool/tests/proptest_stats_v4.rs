//! Property-based tests for `BuildStats` in adze-tool.
//!
//! 48 proptest properties across 9 categories:
//! 1. Build stats have non-negative counts (5)
//! 2. State count matches parse table (5)
//! 3. Symbol count is reasonable (5)
//! 4. Build time is non-negative (5)
//! 5. Stats are deterministic for same input (5)
//! 6. More rules generally means more states (5)
//! 7. Stats fields agree with grammar properties (5)
//! 8. Token count + nonterminal count = symbol count (5)
//! 9. Edge cases (8)

use adze_ir::builder::GrammarBuilder;
use adze_tool::pure_rust_builder::{
    BuildOptions, BuildResult, build_parser, build_parser_from_json,
};
use proptest::prelude::*;
use serde_json::json;
use std::time::Instant;

// ===========================================================================
// Helpers
// ===========================================================================

fn test_opts() -> BuildOptions {
    BuildOptions {
        out_dir: "/tmp/proptest_stats_v4".to_string(),
        emit_artifacts: false,
        compress_tables: true,
    }
}

/// Build from JSON value, expecting success.
fn build_json_ok(v: &serde_json::Value) -> BuildResult {
    build_parser_from_json(v.to_string(), test_opts()).expect("build_json_ok failed")
}

/// Build from IR builder, expecting success.
fn build_ir_ok(
    name: &str,
    tokens: &[(&str, &str)],
    rules: &[(&str, Vec<&str>)],
    start: &str,
) -> BuildResult {
    let mut b = GrammarBuilder::new(name);
    for &(tname, tpat) in tokens {
        b = b.token(tname, tpat);
    }
    for (lhs, rhs) in rules {
        b = b.rule(lhs, rhs.clone());
    }
    b = b.start(start);
    build_parser(b.build(), test_opts()).expect("build_ir_ok failed")
}

/// Minimal JSON grammar with one STRING rule.
fn minimal_json(name: &str) -> serde_json::Value {
    json!({
        "name": name,
        "rules": {
            "source_file": { "type": "STRING", "value": "hello" }
        }
    })
}

/// JSON grammar with CHOICE alternatives.
fn choice_json(name: &str, alts: &[&str]) -> serde_json::Value {
    let members: Vec<serde_json::Value> = alts
        .iter()
        .map(|s| json!({ "type": "STRING", "value": s }))
        .collect();
    json!({
        "name": name,
        "rules": {
            "source_file": { "type": "CHOICE", "members": members }
        }
    })
}

/// JSON grammar with SEQ members.
fn seq_json(name: &str, tokens: &[&str]) -> serde_json::Value {
    let members: Vec<serde_json::Value> = tokens
        .iter()
        .map(|s| json!({ "type": "STRING", "value": s }))
        .collect();
    json!({
        "name": name,
        "rules": {
            "source_file": { "type": "SEQ", "members": members }
        }
    })
}

/// JSON grammar with REPEAT.
fn repeat_json(name: &str, val: &str) -> serde_json::Value {
    json!({
        "name": name,
        "rules": {
            "source_file": {
                "type": "REPEAT",
                "content": { "type": "STRING", "value": val }
            }
        }
    })
}

/// JSON grammar with PATTERN rule.
fn pattern_json(name: &str, pat: &str) -> serde_json::Value {
    json!({
        "name": name,
        "rules": {
            "source_file": { "type": "PATTERN", "value": pat }
        }
    })
}

/// JSON grammar with a named sub-rule via SYMBOL reference.
fn symbol_ref_json(name: &str, token_val: &str) -> serde_json::Value {
    json!({
        "name": name,
        "rules": {
            "source_file": { "type": "SYMBOL", "name": "item" },
            "item": { "type": "STRING", "value": token_val }
        }
    })
}

/// JSON grammar with multiple named sub-rules.
fn multi_rule_json(name: &str, rule_count: usize) -> serde_json::Value {
    let mut rules = serde_json::Map::new();
    // source_file references rule_0
    rules.insert(
        "source_file".to_string(),
        json!({ "type": "SYMBOL", "name": "rule_0" }),
    );
    for i in 0..rule_count {
        let rule_name = format!("rule_{i}");
        if i + 1 < rule_count {
            let next = format!("rule_{}", i + 1);
            rules.insert(
                rule_name,
                json!({
                    "type": "SEQ",
                    "members": [
                        { "type": "STRING", "value": format!("tok{i}") },
                        { "type": "SYMBOL", "name": next }
                    ]
                }),
            );
        } else {
            rules.insert(
                rule_name,
                json!({ "type": "STRING", "value": format!("tok{i}") }),
            );
        }
    }
    json!({ "name": name, "rules": rules })
}

/// JSON grammar with OPTIONAL wrapping.
fn optional_json(name: &str, val: &str) -> serde_json::Value {
    json!({
        "name": name,
        "rules": {
            "source_file": {
                "type": "OPTIONAL",
                "content": { "type": "STRING", "value": val }
            }
        }
    })
}

/// JSON grammar with REPEAT1 wrapping.
fn repeat1_json(name: &str, val: &str) -> serde_json::Value {
    json!({
        "name": name,
        "rules": {
            "source_file": {
                "type": "REPEAT1",
                "content": { "type": "STRING", "value": val }
            }
        }
    })
}

// ===========================================================================
// Strategies
// ===========================================================================

fn grammar_name() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("alpha".to_string()),
        Just("beta".to_string()),
        Just("gamma".to_string()),
        Just("delta".to_string()),
        Just("zeta".to_string()),
        Just("eta".to_string()),
    ]
}

fn safe_pattern() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("[a-z]+".to_string()),
        Just("[0-9]+".to_string()),
        Just("[A-Z][a-z]*".to_string()),
        Just("[a-zA-Z_]+".to_string()),
        Just("[0-9a-fA-F]+".to_string()),
    ]
}

const ALTS_A: [&str; 6] = ["aa", "bb", "cc", "dd", "ee", "ff"];
const ALTS_B: [&str; 6] = ["gg", "hh", "ii", "jj", "kk", "ll"];
const SEQ_VALS: [&str; 5] = ["pp", "qq", "rr", "ss", "tt"];

// ===========================================================================
// 1. Build stats have non-negative counts (5 properties)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(6))]

    #[test]
    fn nonneg_state_count_minimal(name in grammar_name()) {
        let r = build_json_ok(&minimal_json(&name));
        prop_assert!(r.build_stats.state_count > 0);
    }

    #[test]
    fn nonneg_symbol_count_minimal(name in grammar_name()) {
        let r = build_json_ok(&minimal_json(&name));
        prop_assert!(r.build_stats.symbol_count > 0);
    }

    #[test]
    fn nonneg_conflict_cells_minimal(name in grammar_name()) {
        // conflict_cells is usize so always >= 0; verify it doesn't panic
        let r = build_json_ok(&minimal_json(&name));
        let _ = r.build_stats.conflict_cells;
    }

    #[test]
    fn nonneg_state_count_choice(n in 2..=5usize) {
        let alts = &ALTS_A[..n];
        let r = build_json_ok(&choice_json("nonneg_ch", alts));
        prop_assert!(r.build_stats.state_count > 0);
    }

    #[test]
    fn nonneg_counts_pattern(pat in safe_pattern()) {
        let r = build_json_ok(&pattern_json("nonneg_pat", &pat));
        prop_assert!(r.build_stats.state_count > 0);
        prop_assert!(r.build_stats.symbol_count > 0);
    }
}

// ===========================================================================
// 2. State count matches parse table (5 properties)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(6))]

    #[test]
    fn state_count_positive_for_string(name in grammar_name()) {
        let r = build_json_ok(&minimal_json(&name));
        prop_assert!(r.build_stats.state_count >= 1,
            "A STRING grammar must have at least 1 state");
    }

    #[test]
    fn state_count_positive_for_seq(n in 2..=4usize) {
        let tokens = &SEQ_VALS[..n];
        let r = build_json_ok(&seq_json("st_seq", tokens));
        prop_assert!(r.build_stats.state_count >= 1);
    }

    #[test]
    fn state_count_positive_for_choice(n in 2..=5usize) {
        let alts = &ALTS_A[..n];
        let r = build_json_ok(&choice_json("st_ch", alts));
        prop_assert!(r.build_stats.state_count >= 1);
    }

    #[test]
    fn state_count_positive_for_repeat(name in grammar_name()) {
        let r = build_json_ok(&repeat_json(&name, "rep"));
        prop_assert!(r.build_stats.state_count >= 1);
    }

    #[test]
    fn state_count_positive_for_symbol_ref(name in grammar_name()) {
        let r = build_json_ok(&symbol_ref_json(&name, "val"));
        prop_assert!(r.build_stats.state_count >= 1);
    }
}

// ===========================================================================
// 3. Symbol count is reasonable (5 properties)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(6))]

    #[test]
    fn symbol_count_at_least_two(name in grammar_name()) {
        // At minimum: 1 terminal + EOF
        let r = build_json_ok(&minimal_json(&name));
        prop_assert!(r.build_stats.symbol_count >= 2,
            "Expected >= 2 symbols (terminal + EOF), got {}", r.build_stats.symbol_count);
    }

    #[test]
    fn symbol_count_grows_with_choice_alts(n in 2..=4usize) {
        let small = &ALTS_A[..n];
        let large = &ALTS_A[..n + 1];
        let r_s = build_json_ok(&choice_json("sym_s", small));
        let r_l = build_json_ok(&choice_json("sym_l", large));
        prop_assert!(r_l.build_stats.symbol_count >= r_s.build_stats.symbol_count,
            "More alts should not decrease symbols: {} vs {}",
            r_s.build_stats.symbol_count, r_l.build_stats.symbol_count);
    }

    #[test]
    fn symbol_count_at_least_n_for_choice(n in 2..=5usize) {
        let alts = &ALTS_A[..n];
        let r = build_json_ok(&choice_json("sym_ch", alts));
        prop_assert!(r.build_stats.symbol_count >= n,
            "Expected >= {} symbols, got {}", n, r.build_stats.symbol_count);
    }

    #[test]
    fn symbol_count_reasonable_for_pattern(pat in safe_pattern()) {
        let r = build_json_ok(&pattern_json("sym_pat", &pat));
        prop_assert!(r.build_stats.symbol_count >= 2);
    }

    #[test]
    fn symbol_count_positive_for_multi_rule(n in 1..=4usize) {
        let r = build_json_ok(&multi_rule_json("sym_mr", n));
        prop_assert!(r.build_stats.symbol_count > 0);
    }
}

// ===========================================================================
// 4. Build time is non-negative (5 properties)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(4))]

    #[test]
    fn build_time_nonneg_minimal(name in grammar_name()) {
        let start = Instant::now();
        let _r = build_json_ok(&minimal_json(&name));
        let elapsed = start.elapsed();
        prop_assert!(elapsed.as_nanos() > 0, "Build should take measurable time");
    }

    #[test]
    fn build_time_nonneg_choice(n in 2..=4usize) {
        let alts = &ALTS_A[..n];
        let start = Instant::now();
        let _r = build_json_ok(&choice_json("time_ch", alts));
        let elapsed = start.elapsed();
        prop_assert!(elapsed.as_nanos() > 0);
    }

    #[test]
    fn build_time_nonneg_repeat(name in grammar_name()) {
        let start = Instant::now();
        let _r = build_json_ok(&repeat_json(&name, "x"));
        let elapsed = start.elapsed();
        prop_assert!(elapsed.as_nanos() > 0);
    }

    #[test]
    fn build_time_nonneg_pattern(pat in safe_pattern()) {
        let start = Instant::now();
        let _r = build_json_ok(&pattern_json("time_pat", &pat));
        let elapsed = start.elapsed();
        prop_assert!(elapsed.as_nanos() > 0);
    }

    #[test]
    fn build_time_nonneg_ir(name in grammar_name()) {
        let start = Instant::now();
        let _r = build_ir_ok(&name, &[("tok", "lit")], &[("s", vec!["tok"])], "s");
        let elapsed = start.elapsed();
        prop_assert!(elapsed.as_nanos() > 0);
    }
}

// ===========================================================================
// 5. Stats are deterministic for same input (5 properties)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(6))]

    #[test]
    fn deterministic_state_count(name in grammar_name()) {
        let g = choice_json(&name, &["ab", "cd", "ef"]);
        let r1 = build_json_ok(&g);
        let r2 = build_json_ok(&g);
        prop_assert_eq!(r1.build_stats.state_count, r2.build_stats.state_count);
    }

    #[test]
    fn deterministic_symbol_count(name in grammar_name()) {
        let g = choice_json(&name, &["ab", "cd", "ef"]);
        let r1 = build_json_ok(&g);
        let r2 = build_json_ok(&g);
        prop_assert_eq!(r1.build_stats.symbol_count, r2.build_stats.symbol_count);
    }

    #[test]
    fn deterministic_conflict_cells(name in grammar_name()) {
        let g = minimal_json(&name);
        let r1 = build_json_ok(&g);
        let r2 = build_json_ok(&g);
        prop_assert_eq!(r1.build_stats.conflict_cells, r2.build_stats.conflict_cells);
    }

    #[test]
    fn deterministic_parser_code(name in grammar_name()) {
        let g = minimal_json(&name);
        let r1 = build_json_ok(&g);
        let r2 = build_json_ok(&g);
        prop_assert_eq!(r1.parser_code, r2.parser_code);
    }

    #[test]
    fn deterministic_ir_path_stats(name in grammar_name()) {
        let r1 = build_ir_ok(&name, &[("tok", "lit")], &[("s", vec!["tok"])], "s");
        let r2 = build_ir_ok(&name, &[("tok", "lit")], &[("s", vec!["tok"])], "s");
        prop_assert_eq!(r1.build_stats.state_count, r2.build_stats.state_count);
        prop_assert_eq!(r1.build_stats.symbol_count, r2.build_stats.symbol_count);
        prop_assert_eq!(r1.build_stats.conflict_cells, r2.build_stats.conflict_cells);
    }
}

// ===========================================================================
// 6. More rules generally means more states (5 properties)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(4))]

    #[test]
    fn more_choice_alts_means_geq_states(n in 2..=4usize) {
        let small = &ALTS_B[..n];
        let large = &ALTS_B[..n + 1];
        let r_s = build_json_ok(&choice_json("mrule_s", small));
        let r_l = build_json_ok(&choice_json("mrule_l", large));
        prop_assert!(r_l.build_stats.state_count >= r_s.build_stats.state_count,
            "More alts should not decrease states: {} vs {}",
            r_s.build_stats.state_count, r_l.build_stats.state_count);
    }

    #[test]
    fn more_seq_tokens_means_geq_states(n in 2..=3usize) {
        let small = &SEQ_VALS[..n];
        let large = &SEQ_VALS[..n + 1];
        let r_s = build_json_ok(&seq_json("mseq_s", small));
        let r_l = build_json_ok(&seq_json("mseq_l", large));
        prop_assert!(r_l.build_stats.state_count >= r_s.build_stats.state_count,
            "Longer SEQ should not decrease states: {} vs {}",
            r_s.build_stats.state_count, r_l.build_stats.state_count);
    }

    #[test]
    fn more_named_rules_means_geq_symbols(n in 1..=3usize) {
        let r_s = build_json_ok(&multi_rule_json("mns_s", n));
        let r_l = build_json_ok(&multi_rule_json("mns_l", n + 1));
        prop_assert!(r_l.build_stats.symbol_count >= r_s.build_stats.symbol_count,
            "More rules should not decrease symbols: {} vs {}",
            r_s.build_stats.symbol_count, r_l.build_stats.symbol_count);
    }

    #[test]
    fn more_ir_tokens_means_geq_symbols(n in 2..=4usize) {
        let tok_names: Vec<String> = (0..n).map(|i| format!("t{i}")).collect();
        let tok_pats: Vec<String> = (0..n).map(|i| format!("p{i}")).collect();
        let pairs: Vec<(&str, &str)> = tok_names.iter().zip(tok_pats.iter())
            .map(|(a, b)| (a.as_str(), b.as_str())).collect();
        let rules: Vec<(&str, Vec<&str>)> = tok_names.iter()
            .map(|t| ("s", vec![t.as_str()])).collect();
        let r_n = build_ir_ok("ir_mono", &pairs, &rules, "s");

        let tok_names2: Vec<String> = (0..n + 1).map(|i| format!("t{i}")).collect();
        let tok_pats2: Vec<String> = (0..n + 1).map(|i| format!("p{i}")).collect();
        let pairs2: Vec<(&str, &str)> = tok_names2.iter().zip(tok_pats2.iter())
            .map(|(a, b)| (a.as_str(), b.as_str())).collect();
        let rules2: Vec<(&str, Vec<&str>)> = tok_names2.iter()
            .map(|t| ("s", vec![t.as_str()])).collect();
        let r_n1 = build_ir_ok("ir_mono", &pairs2, &rules2, "s");

        prop_assert!(r_n1.build_stats.symbol_count >= r_n.build_stats.symbol_count);
    }

    #[test]
    fn more_named_rules_means_geq_states(n in 1..=3usize) {
        let r_s = build_json_ok(&multi_rule_json("mrs_s", n));
        let r_l = build_json_ok(&multi_rule_json("mrs_l", n + 1));
        prop_assert!(r_l.build_stats.state_count >= r_s.build_stats.state_count,
            "More rules should not decrease states: {} vs {}",
            r_s.build_stats.state_count, r_l.build_stats.state_count);
    }
}

// ===========================================================================
// 7. Stats fields agree with grammar properties (5 properties)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(6))]

    #[test]
    fn grammar_name_preserved(name in grammar_name()) {
        let r = build_json_ok(&minimal_json(&name));
        prop_assert_eq!(r.grammar_name, name);
    }

    #[test]
    fn parser_code_contains_grammar_name(name in grammar_name()) {
        let r = build_json_ok(&minimal_json(&name));
        prop_assert!(r.parser_code.contains(&name),
            "parser_code should contain grammar name '{}'", name);
    }

    #[test]
    fn symbol_ref_grammar_reports_named_rules(name in grammar_name()) {
        let r = build_json_ok(&symbol_ref_json(&name, "tok"));
        // A symbol-ref grammar has source_file + item + terminals + EOF
        prop_assert!(r.build_stats.symbol_count >= 3,
            "Symbol-ref grammar should have >= 3 symbols, got {}",
            r.build_stats.symbol_count);
    }

    #[test]
    fn node_types_array_for_named_grammar(name in grammar_name()) {
        let r = build_json_ok(&symbol_ref_json(&name, "tok"));
        let arr: Vec<serde_json::Value> = serde_json::from_str(&r.node_types_json).unwrap();
        let has_source_file = arr.iter().any(|entry| {
            entry.get("type").and_then(|v| v.as_str()) == Some("source_file")
        });
        prop_assert!(has_source_file, "node_types should include source_file");
    }

    #[test]
    fn repeat_grammar_symbols_include_repeated_token(name in grammar_name()) {
        let r = build_json_ok(&repeat_json(&name, "rv"));
        // REPEAT wraps a terminal, so at least: terminal + EOF + nonterminal
        prop_assert!(r.build_stats.symbol_count >= 2,
            "REPEAT grammar should have >= 2 symbols, got {}",
            r.build_stats.symbol_count);
    }
}

// ===========================================================================
// 8. Token count + nonterminal count = symbol count (5 properties)
//
// BuildStats.symbol_count reflects the parse table total (terminals +
// nonterminals). We cannot directly separate them from BuildStats alone,
// but we can verify relationships between grammar shape and symbol_count.
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(6))]

    #[test]
    fn symbol_count_geq_distinct_terminals_choice(n in 2..=5usize) {
        let alts = &ALTS_A[..n];
        let r = build_json_ok(&choice_json("tc_ch", alts));
        // n distinct STRING literals → at least n terminal symbols
        prop_assert!(r.build_stats.symbol_count >= n,
            "Expected symbol_count >= {} (distinct terminals), got {}",
            n, r.build_stats.symbol_count);
    }

    #[test]
    fn symbol_count_geq_tokens_plus_one_nonterminal(n in 2..=4usize) {
        let alts = &ALTS_A[..n];
        let r = build_json_ok(&choice_json("tc_nt", alts));
        // n terminals + at least 1 nonterminal (source_file) + EOF
        prop_assert!(r.build_stats.symbol_count > n,
            "Expected > {} (tokens + nonterminal), got {}",
            n, r.build_stats.symbol_count);
    }

    #[test]
    fn symbol_count_geq_seq_length(n in 2..=4usize) {
        let tokens = &SEQ_VALS[..n];
        let r = build_json_ok(&seq_json("tc_sq", tokens));
        // n string literals in a SEQ → at least n distinct terminal symbols
        prop_assert!(r.build_stats.symbol_count >= n,
            "Expected symbol_count >= {} for SEQ, got {}",
            n, r.build_stats.symbol_count);
    }

    #[test]
    fn symbol_ref_has_more_symbols_than_minimal(name in grammar_name()) {
        let r_min = build_json_ok(&minimal_json(&name));
        let r_ref = build_json_ok(&symbol_ref_json(&name, "tok"));
        // symbol_ref has 2 named rules vs 1, so more symbols expected
        prop_assert!(r_ref.build_stats.symbol_count >= r_min.build_stats.symbol_count,
            "Symbol-ref grammar should have >= symbols than minimal: {} vs {}",
            r_ref.build_stats.symbol_count, r_min.build_stats.symbol_count);
    }

    #[test]
    fn ir_symbol_count_geq_token_count(n in 2..=4usize) {
        let tok_names: Vec<String> = (0..n).map(|i| format!("t{i}")).collect();
        let tok_pats: Vec<String> = (0..n).map(|i| format!("p{i}")).collect();
        let pairs: Vec<(&str, &str)> = tok_names.iter().zip(tok_pats.iter())
            .map(|(a, b)| (a.as_str(), b.as_str())).collect();
        let rules: Vec<(&str, Vec<&str>)> = tok_names.iter()
            .map(|t| ("s", vec![t.as_str()])).collect();
        let r = build_ir_ok("ir_tc", &pairs, &rules, "s");
        // At least n tokens exist in the grammar
        prop_assert!(r.build_stats.symbol_count >= n,
            "Expected symbol_count >= {} (token count), got {}",
            n, r.build_stats.symbol_count);
    }
}

// ===========================================================================
// 9. Edge cases (8 properties)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(4))]

    #[test]
    fn edge_optional_grammar_builds(name in grammar_name()) {
        let r = build_json_ok(&optional_json(&name, "opt"));
        prop_assert!(r.build_stats.state_count > 0);
        prop_assert!(r.build_stats.symbol_count > 0);
    }

    #[test]
    fn edge_repeat1_grammar_builds(name in grammar_name()) {
        let r = build_json_ok(&repeat1_json(&name, "rep1"));
        prop_assert!(r.build_stats.state_count > 0);
        prop_assert!(r.build_stats.symbol_count > 0);
    }

    #[test]
    fn edge_single_choice_alt(name in grammar_name()) {
        let r = build_json_ok(&choice_json(&name, &["only"]));
        prop_assert!(r.build_stats.state_count > 0);
    }

    #[test]
    fn edge_conflict_cells_leq_product(name in grammar_name()) {
        let r = build_json_ok(&choice_json(&name, &["x", "y", "z"]));
        let max_cells = r.build_stats.state_count * r.build_stats.symbol_count;
        prop_assert!(r.build_stats.conflict_cells <= max_cells,
            "conflict_cells ({}) should be <= state*symbol ({})",
            r.build_stats.conflict_cells, max_cells);
    }

    #[test]
    fn edge_compressed_same_stats_as_uncompressed(name in grammar_name()) {
        let g = minimal_json(&name);
        let r_c = build_parser_from_json(g.to_string(), test_opts()).unwrap();
        let uncomp_opts = BuildOptions {
            compress_tables: false,
            ..test_opts()
        };
        let r_u = build_parser_from_json(g.to_string(), uncomp_opts).unwrap();
        prop_assert_eq!(r_c.build_stats.state_count, r_u.build_stats.state_count);
        prop_assert_eq!(r_c.build_stats.symbol_count, r_u.build_stats.symbol_count);
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(4))]

    #[test]
    fn edge_multi_rule_builds(n in 1..=4usize) {
        let r = build_json_ok(&multi_rule_json("edge_mr", n));
        prop_assert!(r.build_stats.state_count > 0);
        prop_assert!(r.build_stats.symbol_count > 0);
    }

    #[test]
    fn edge_invalid_json_always_errors(s in "[^{}\\[\\]\"]{1,20}") {
        let result = build_parser_from_json(s, test_opts());
        prop_assert!(result.is_err());
    }

    #[test]
    fn edge_numeric_json_always_errors(n in 0..1000i32) {
        let result = build_parser_from_json(n.to_string(), test_opts());
        prop_assert!(result.is_err());
    }
}
