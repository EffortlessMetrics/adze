//! Property-based tests for adze-tool JSON grammar building and IR builder paths.
//!
//! 40+ proptest properties covering:
//! 1. JSON output structure properties (8)
//! 2. Roundtrip consistency — IR vs JSON paths (6)
//! 3. Build stats correctness (6)
//! 4. Determinism (6)
//! 5. Monotonicity / scaling (6)
//! 6. Grammar name handling (4)
//! 7. Node types JSON properties (5)
//! 8. Edge cases and error paths (4)

use adze_ir::builder::GrammarBuilder;
use adze_tool::pure_rust_builder::{
    BuildOptions, BuildResult, build_parser, build_parser_from_json,
};
use proptest::prelude::*;
use serde_json::json;

// ===========================================================================
// Helpers
// ===========================================================================

fn test_opts() -> BuildOptions {
    BuildOptions {
        out_dir: "/tmp/proptest_json_v4".to_string(),
        emit_artifacts: false,
        compress_tables: true,
    }
}

fn uncompressed_opts() -> BuildOptions {
    BuildOptions {
        compress_tables: false,
        ..test_opts()
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

/// JSON grammar with PATTERN rule.
fn pattern_json(name: &str, pat: &str) -> serde_json::Value {
    json!({
        "name": name,
        "rules": {
            "source_file": { "type": "PATTERN", "value": pat }
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

#[allow(dead_code)]
fn token_val() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("foo".to_string()),
        Just("bar".to_string()),
        Just("baz".to_string()),
        Just("qux".to_string()),
        Just("nop".to_string()),
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
// 1. JSON output structure properties (8 tests)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(8))]

    #[test]
    fn json_output_node_types_is_valid_json(name in grammar_name()) {
        let r = build_json_ok(&minimal_json(&name));
        let parsed: serde_json::Value = serde_json::from_str(&r.node_types_json)
            .expect("node_types_json must be valid JSON");
        prop_assert!(parsed.is_array(), "node_types_json should be an array");
    }

    #[test]
    fn json_output_parser_code_nonempty(name in grammar_name()) {
        let r = build_json_ok(&minimal_json(&name));
        prop_assert!(!r.parser_code.is_empty());
    }

    #[test]
    fn json_output_parser_path_nonempty(name in grammar_name()) {
        let r = build_json_ok(&minimal_json(&name));
        prop_assert!(!r.parser_path.is_empty());
    }

    #[test]
    fn json_output_grammar_name_matches(name in grammar_name()) {
        let r = build_json_ok(&minimal_json(&name));
        prop_assert_eq!(r.grammar_name, name);
    }

    #[test]
    fn json_output_node_types_contains_source_file(name in grammar_name()) {
        let r = build_json_ok(&symbol_ref_json(&name, "tok"));
        let arr: Vec<serde_json::Value> = serde_json::from_str(&r.node_types_json).unwrap();
        let has_source_file = arr.iter().any(|entry| {
            entry.get("type").and_then(|v| v.as_str()) == Some("source_file")
        });
        prop_assert!(has_source_file, "node_types should include source_file");
    }

    #[test]
    fn json_output_choice_builds_ok(name in grammar_name()) {
        let r = build_json_ok(&choice_json(&name, &["xx", "yy", "zz"]));
        prop_assert!(r.build_stats.state_count > 0);
    }

    #[test]
    fn json_output_seq_builds_ok(name in grammar_name()) {
        let r = build_json_ok(&seq_json(&name, &["mm", "nn"]));
        prop_assert!(!r.parser_code.is_empty());
    }

    #[test]
    fn json_output_repeat_builds_ok(name in grammar_name()) {
        let r = build_json_ok(&repeat_json(&name, "item"));
        prop_assert!(r.build_stats.symbol_count > 0);
    }
}

// ===========================================================================
// 2. Roundtrip consistency — IR vs JSON paths (6 tests)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(6))]

    #[test]
    fn roundtrip_grammar_name_ir_vs_json(name in grammar_name()) {
        let ir_result = build_ir_ok(&name, &[("tok", "hello")], &[("s", vec!["tok"])], "s");
        let json_result = build_json_ok(&minimal_json(&name));
        prop_assert_eq!(ir_result.grammar_name, name.clone());
        prop_assert_eq!(json_result.grammar_name, name);
    }

    #[test]
    fn roundtrip_both_paths_produce_states(name in grammar_name()) {
        let ir_result = build_ir_ok(&name, &[("tok", "hello")], &[("s", vec!["tok"])], "s");
        let json_result = build_json_ok(&minimal_json(&name));
        prop_assert!(ir_result.build_stats.state_count > 0);
        prop_assert!(json_result.build_stats.state_count > 0);
    }

    #[test]
    fn roundtrip_both_paths_produce_symbols(name in grammar_name()) {
        let ir_result = build_ir_ok(&name, &[("tok", "hello")], &[("s", vec!["tok"])], "s");
        let json_result = build_json_ok(&minimal_json(&name));
        prop_assert!(ir_result.build_stats.symbol_count > 0);
        prop_assert!(json_result.build_stats.symbol_count > 0);
    }

    #[test]
    fn roundtrip_both_paths_produce_valid_node_types(name in grammar_name()) {
        let ir_result = build_ir_ok(&name, &[("tok", "hello")], &[("s", vec!["tok"])], "s");
        let json_result = build_json_ok(&minimal_json(&name));
        let ir_nt: serde_json::Value = serde_json::from_str(&ir_result.node_types_json).unwrap();
        let json_nt: serde_json::Value = serde_json::from_str(&json_result.node_types_json).unwrap();
        prop_assert!(ir_nt.is_array());
        prop_assert!(json_nt.is_array());
    }

    #[test]
    fn roundtrip_ir_json_parser_code_nonempty(name in grammar_name()) {
        let ir_result = build_ir_ok(&name, &[("tok", "hello")], &[("s", vec!["tok"])], "s");
        let json_result = build_json_ok(&minimal_json(&name));
        prop_assert!(!ir_result.parser_code.is_empty());
        prop_assert!(!json_result.parser_code.is_empty());
    }

    #[test]
    fn roundtrip_compressed_vs_uncompressed_same_stats(name in grammar_name()) {
        let g1 = minimal_json(&name);
        let g2 = minimal_json(&name);
        let r1 = build_parser_from_json(g1.to_string(), test_opts()).unwrap();
        let r2 = build_parser_from_json(g2.to_string(), uncompressed_opts()).unwrap();
        prop_assert_eq!(r1.build_stats.state_count, r2.build_stats.state_count);
        prop_assert_eq!(r1.build_stats.symbol_count, r2.build_stats.symbol_count);
    }
}

// ===========================================================================
// 3. Build stats correctness (6 tests)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(8))]

    #[test]
    fn stats_state_count_positive_minimal(name in grammar_name()) {
        let r = build_json_ok(&minimal_json(&name));
        prop_assert!(r.build_stats.state_count > 0);
    }

    #[test]
    fn stats_symbol_count_positive_minimal(name in grammar_name()) {
        let r = build_json_ok(&minimal_json(&name));
        prop_assert!(r.build_stats.symbol_count > 0);
    }

    #[test]
    fn stats_conflict_cells_non_negative(name in grammar_name()) {
        let r = build_json_ok(&minimal_json(&name));
        // conflict_cells is usize, always >= 0; verify it doesn't panic
        let _ = r.build_stats.conflict_cells;
    }

    #[test]
    fn stats_choice_has_at_least_n_symbols(n in 2..=5usize) {
        let alts = &ALTS_A[..n];
        let r = build_json_ok(&choice_json("stat_ch", alts));
        // At least N terminal symbols + EOF + source_file nonterminal
        prop_assert!(
            r.build_stats.symbol_count >= n,
            "Expected >= {} symbols, got {}",
            n,
            r.build_stats.symbol_count,
        );
    }

    #[test]
    fn stats_seq_has_positive_states(n in 2..=4usize) {
        let tokens = &SEQ_VALS[..n];
        let r = build_json_ok(&seq_json("stat_seq", tokens));
        prop_assert!(r.build_stats.state_count > 0);
    }

    #[test]
    fn stats_pattern_has_positive_counts(pat in safe_pattern()) {
        let r = build_json_ok(&pattern_json("stat_pat", &pat));
        prop_assert!(r.build_stats.state_count > 0);
        prop_assert!(r.build_stats.symbol_count > 0);
    }
}

// ===========================================================================
// 4. Determinism (6 tests)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(6))]

    #[test]
    fn deterministic_parser_code_json(name in grammar_name()) {
        let g = minimal_json(&name);
        let r1 = build_json_ok(&g);
        let r2 = build_json_ok(&g);
        prop_assert_eq!(r1.parser_code, r2.parser_code);
    }

    #[test]
    fn deterministic_node_types_json(name in grammar_name()) {
        let g = minimal_json(&name);
        let r1 = build_json_ok(&g);
        let r2 = build_json_ok(&g);
        prop_assert_eq!(r1.node_types_json, r2.node_types_json);
    }

    #[test]
    fn deterministic_state_count_json(name in grammar_name()) {
        let g = choice_json(&name, &["ab", "cd", "ef"]);
        let r1 = build_json_ok(&g);
        let r2 = build_json_ok(&g);
        prop_assert_eq!(r1.build_stats.state_count, r2.build_stats.state_count);
    }

    #[test]
    fn deterministic_symbol_count_json(name in grammar_name()) {
        let g = choice_json(&name, &["ab", "cd", "ef"]);
        let r1 = build_json_ok(&g);
        let r2 = build_json_ok(&g);
        prop_assert_eq!(r1.build_stats.symbol_count, r2.build_stats.symbol_count);
    }

    #[test]
    fn deterministic_conflict_cells_json(name in grammar_name()) {
        let g = minimal_json(&name);
        let r1 = build_json_ok(&g);
        let r2 = build_json_ok(&g);
        prop_assert_eq!(r1.build_stats.conflict_cells, r2.build_stats.conflict_cells);
    }

    #[test]
    fn deterministic_ir_path(name in grammar_name()) {
        let r1 = build_ir_ok(&name, &[("tok", "lit")], &[("s", vec!["tok"])], "s");
        let r2 = build_ir_ok(&name, &[("tok", "lit")], &[("s", vec!["tok"])], "s");
        prop_assert_eq!(r1.parser_code, r2.parser_code);
        prop_assert_eq!(r1.build_stats.state_count, r2.build_stats.state_count);
    }
}

// ===========================================================================
// 5. Monotonicity / scaling (6 tests)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(6))]

    #[test]
    fn monotonic_choice_symbol_count(n in 2..=4usize) {
        let small = &ALTS_A[..n];
        let large = &ALTS_A[..n + 1];
        let r_s = build_json_ok(&choice_json("mc_s", small));
        let r_l = build_json_ok(&choice_json("mc_l", large));
        prop_assert!(
            r_l.build_stats.symbol_count >= r_s.build_stats.symbol_count,
            "More alts should not decrease symbols: {} vs {}",
            r_s.build_stats.symbol_count,
            r_l.build_stats.symbol_count,
        );
    }

    #[test]
    fn monotonic_choice_state_count(n in 2..=4usize) {
        let small = &ALTS_B[..n];
        let large = &ALTS_B[..n + 1];
        let r_s = build_json_ok(&choice_json("ms_s", small));
        let r_l = build_json_ok(&choice_json("ms_l", large));
        prop_assert!(
            r_l.build_stats.state_count >= r_s.build_stats.state_count,
            "More alts should not decrease states: {} vs {}",
            r_s.build_stats.state_count,
            r_l.build_stats.state_count,
        );
    }

    #[test]
    fn monotonic_seq_code_length(n in 2..=4usize) {
        let small = &SEQ_VALS[..n];
        let large = &SEQ_VALS[..n + 1];
        let r_s = build_json_ok(&seq_json("sq_s", small));
        let r_l = build_json_ok(&seq_json("sq_l", large));
        prop_assert!(
            r_l.parser_code.len() >= r_s.parser_code.len(),
            "Longer SEQ should produce equal or longer parser code",
        );
    }

    #[test]
    fn monotonic_choice_node_types_length(n in 2..=4usize) {
        let small = &ALTS_A[..n];
        let large = &ALTS_A[..n + 1];
        let r_s = build_json_ok(&choice_json("nt_s", small));
        let r_l = build_json_ok(&choice_json("nt_l", large));
        prop_assert!(
            r_l.node_types_json.len() >= r_s.node_types_json.len(),
            "More alts should produce equal or longer node_types",
        );
    }

    #[test]
    fn monotonic_ir_symbol_count(n in 2..=4usize) {
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
    fn monotonic_ir_parser_code_grows(n in 2..=4usize) {
        let tok_names: Vec<String> = (0..n).map(|i| format!("t{i}")).collect();
        let tok_pats: Vec<String> = (0..n).map(|i| format!("p{i}")).collect();
        let pairs: Vec<(&str, &str)> = tok_names.iter().zip(tok_pats.iter())
            .map(|(a, b)| (a.as_str(), b.as_str())).collect();
        let rules: Vec<(&str, Vec<&str>)> = tok_names.iter()
            .map(|t| ("s", vec![t.as_str()])).collect();
        let r_n = build_ir_ok("ir_grow", &pairs, &rules, "s");

        let tok_names2: Vec<String> = (0..n + 1).map(|i| format!("t{i}")).collect();
        let tok_pats2: Vec<String> = (0..n + 1).map(|i| format!("p{i}")).collect();
        let pairs2: Vec<(&str, &str)> = tok_names2.iter().zip(tok_pats2.iter())
            .map(|(a, b)| (a.as_str(), b.as_str())).collect();
        let rules2: Vec<(&str, Vec<&str>)> = tok_names2.iter()
            .map(|t| ("s", vec![t.as_str()])).collect();
        let r_n1 = build_ir_ok("ir_grow", &pairs2, &rules2, "s");

        prop_assert!(r_n1.parser_code.len() >= r_n.parser_code.len());
    }
}

// ===========================================================================
// 6. Grammar name handling (4 tests)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(8))]

    #[test]
    fn name_preserved_in_parser_code(name in grammar_name()) {
        let r = build_json_ok(&minimal_json(&name));
        prop_assert!(
            r.parser_code.contains(&name),
            "parser_code should contain grammar name '{}'",
            name,
        );
    }

    #[test]
    fn name_preserved_pattern_grammar(name in grammar_name()) {
        let r = build_json_ok(&pattern_json(&name, "[a-z]+"));
        prop_assert_eq!(r.grammar_name, name);
    }

    #[test]
    fn name_preserved_symbol_ref_grammar(name in grammar_name()) {
        let r = build_json_ok(&symbol_ref_json(&name, "val"));
        prop_assert_eq!(r.grammar_name, name);
    }

    #[test]
    fn name_preserved_ir_path(name in grammar_name()) {
        let r = build_ir_ok(&name, &[("tok", "lit")], &[("s", vec!["tok"])], "s");
        prop_assert_eq!(r.grammar_name, name);
    }
}

// ===========================================================================
// 7. Node types JSON properties (5 tests)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(8))]

    #[test]
    fn node_types_is_json_array(name in grammar_name()) {
        let r = build_json_ok(&minimal_json(&name));
        let v: serde_json::Value = serde_json::from_str(&r.node_types_json).unwrap();
        prop_assert!(v.is_array());
    }

    #[test]
    fn node_types_entries_have_type_field(name in grammar_name()) {
        let r = build_json_ok(&symbol_ref_json(&name, "tok"));
        let arr: Vec<serde_json::Value> = serde_json::from_str(&r.node_types_json).unwrap();
        for entry in &arr {
            prop_assert!(
                entry.get("type").is_some(),
                "Each node_types entry should have a 'type' field",
            );
        }
    }

    #[test]
    fn node_types_type_fields_are_strings(name in grammar_name()) {
        let r = build_json_ok(&symbol_ref_json(&name, "tok"));
        let arr: Vec<serde_json::Value> = serde_json::from_str(&r.node_types_json).unwrap();
        for entry in &arr {
            if let Some(t) = entry.get("type") {
                prop_assert!(t.is_string(), "'type' field should be a string");
            }
        }
    }

    #[test]
    fn node_types_nonempty_for_symbol_grammar(name in grammar_name()) {
        let r = build_json_ok(&symbol_ref_json(&name, "tok"));
        let arr: Vec<serde_json::Value> = serde_json::from_str(&r.node_types_json).unwrap();
        prop_assert!(!arr.is_empty(), "node_types should not be empty for symbol grammar");
    }

    #[test]
    fn node_types_deterministic(name in grammar_name()) {
        let g = symbol_ref_json(&name, "tok");
        let r1 = build_json_ok(&g);
        let r2 = build_json_ok(&g);
        prop_assert_eq!(r1.node_types_json, r2.node_types_json);
    }
}

// ===========================================================================
// 8. Edge cases and error paths (4 proptest + 5 regular = 9)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(6))]

    #[test]
    fn error_invalid_json_string(s in "[^{}\\[\\]\"]{1,20}") {
        let r = build_parser_from_json(s, test_opts());
        prop_assert!(r.is_err());
    }

    #[test]
    fn error_json_number_input(n in 0..1000i32) {
        let r = build_parser_from_json(n.to_string(), test_opts());
        prop_assert!(r.is_err());
    }

    #[test]
    fn error_empty_ir_grammar_fails(name in grammar_name()) {
        let g = GrammarBuilder::new(&name).build();
        let result = build_parser(g, test_opts());
        prop_assert!(result.is_err(), "empty grammar should fail");
    }

    #[test]
    fn error_message_is_nonempty_on_failure(name in grammar_name()) {
        let g = GrammarBuilder::new(&name).build();
        if let Err(e) = build_parser(g, test_opts()) {
            let msg = format!("{e}");
            prop_assert!(!msg.is_empty());
        }
    }
}

#[test]
fn error_empty_string_input() {
    assert!(build_parser_from_json(String::new(), test_opts()).is_err());
}

#[test]
fn error_json_null_input() {
    assert!(build_parser_from_json("null".to_string(), test_opts()).is_err());
}

#[test]
fn error_json_bool_input() {
    assert!(build_parser_from_json("true".to_string(), test_opts()).is_err());
}

#[test]
fn error_json_array_input() {
    assert!(build_parser_from_json("[]".to_string(), test_opts()).is_err());
}

#[test]
fn error_missing_rules_field() {
    let g = json!({"name": "orphan"});
    assert!(build_parser_from_json(g.to_string(), test_opts()).is_err());
}
