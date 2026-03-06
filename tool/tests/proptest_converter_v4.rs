//! Property-based tests for JSON grammar conversion in adze-tool.
//!
//! 46 proptest properties covering:
//! 1. Valid JSON grammars produce non-error results (5)
//! 2. Grammar name preserved from JSON (5)
//! 3. Build stats are positive for valid inputs (5)
//! 4. Parser code is non-empty (5)
//! 5. Node types is valid JSON (5)
//! 6. Build is deterministic (5)
//! 7. Grammar with more rules produces larger output (5)
//! 8. Invalid inputs produce errors (not panics) (5)
//! 9. Edge cases (6)

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
        out_dir: "/tmp/proptest_converter_v4".to_string(),
        emit_artifacts: false,
        compress_tables: true,
    }
}

/// Build from JSON value, expecting success.
fn build_json_ok(v: &serde_json::Value) -> BuildResult {
    build_parser_from_json(v.to_string(), test_opts()).expect("build_json_ok failed")
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

/// JSON grammar with SEQ of STRING members.
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

/// JSON grammar with a PATTERN rule.
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

/// JSON grammar with extras (whitespace).
fn extras_json(name: &str) -> serde_json::Value {
    json!({
        "name": name,
        "rules": {
            "source_file": { "type": "SYMBOL", "name": "item" },
            "item": { "type": "PATTERN", "value": "\\w+" }
        },
        "extras": [
            { "type": "PATTERN", "value": "\\s" }
        ]
    })
}

/// JSON grammar with REPEAT1.
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

// ===========================================================================
// Strategies
// ===========================================================================

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
                | "true"
                | "false"
        )
    })
}

/// Safe regex patterns that tree-sitter can handle.
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
// 1. Valid JSON grammars produce non-error results (5 properties)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(8))]

    #[test]
    fn valid_minimal_string_grammar_succeeds(name in grammar_name_strategy()) {
        let r = build_parser_from_json(minimal_json(&name).to_string(), test_opts());
        prop_assert!(r.is_ok(), "minimal STRING grammar should succeed");
    }

    #[test]
    fn valid_choice_grammar_succeeds(name in grammar_name_strategy()) {
        let r = build_parser_from_json(choice_json(&name, &["xx", "yy"]).to_string(), test_opts());
        prop_assert!(r.is_ok(), "CHOICE grammar should succeed");
    }

    #[test]
    fn valid_seq_grammar_succeeds(name in grammar_name_strategy()) {
        let r = build_parser_from_json(seq_json(&name, &["aa", "bb"]).to_string(), test_opts());
        prop_assert!(r.is_ok(), "SEQ grammar should succeed");
    }

    #[test]
    fn valid_pattern_grammar_succeeds(pat in safe_pattern()) {
        let r = build_parser_from_json(pattern_json("patlang", &pat).to_string(), test_opts());
        prop_assert!(r.is_ok(), "PATTERN grammar should succeed");
    }

    #[test]
    fn valid_symbol_ref_grammar_succeeds(name in grammar_name_strategy()) {
        let r = build_parser_from_json(
            symbol_ref_json(&name, "tok").to_string(),
            test_opts(),
        );
        prop_assert!(r.is_ok(), "SYMBOL ref grammar should succeed");
    }
}

// ===========================================================================
// 2. Grammar name preserved from JSON (5 properties)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(8))]

    #[test]
    fn name_preserved_minimal(name in grammar_name_strategy()) {
        let r = build_json_ok(&minimal_json(&name));
        prop_assert_eq!(r.grammar_name, name);
    }

    #[test]
    fn name_preserved_choice(name in grammar_name_strategy()) {
        let r = build_json_ok(&choice_json(&name, &["xa", "xb"]));
        prop_assert_eq!(r.grammar_name, name);
    }

    #[test]
    fn name_preserved_pattern(name in grammar_name_strategy()) {
        let r = build_json_ok(&pattern_json(&name, "[a-z]+"));
        prop_assert_eq!(r.grammar_name, name);
    }

    #[test]
    fn name_preserved_symbol_ref(name in grammar_name_strategy()) {
        let r = build_json_ok(&symbol_ref_json(&name, "val"));
        prop_assert_eq!(r.grammar_name, name);
    }

    #[test]
    fn name_embedded_in_parser_code(name in grammar_name_strategy()) {
        let r = build_json_ok(&minimal_json(&name));
        prop_assert!(
            r.parser_code.contains(&name),
            "parser_code should contain grammar name '{name}'",
        );
    }
}

// ===========================================================================
// 3. Build stats are positive for valid inputs (5 properties)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(8))]

    #[test]
    fn stats_state_count_positive(name in grammar_name_strategy()) {
        let r = build_json_ok(&minimal_json(&name));
        prop_assert!(r.build_stats.state_count > 0);
    }

    #[test]
    fn stats_symbol_count_positive(name in grammar_name_strategy()) {
        let r = build_json_ok(&minimal_json(&name));
        prop_assert!(r.build_stats.symbol_count > 0);
    }

    #[test]
    fn stats_positive_for_choice(n in 2..=5usize) {
        let alts = &ALTS_A[..n];
        let r = build_json_ok(&choice_json("chstat", alts));
        prop_assert!(r.build_stats.state_count > 0);
        prop_assert!(r.build_stats.symbol_count > 0);
    }

    #[test]
    fn stats_positive_for_seq(n in 2..=4usize) {
        let tokens = &SEQ_VALS[..n];
        let r = build_json_ok(&seq_json("sqstat", tokens));
        prop_assert!(r.build_stats.state_count > 0);
        prop_assert!(r.build_stats.symbol_count > 0);
    }

    #[test]
    fn stats_choice_has_at_least_n_symbols(n in 2..=5usize) {
        let alts = &ALTS_A[..n];
        let r = build_json_ok(&choice_json("symcnt", alts));
        prop_assert!(
            r.build_stats.symbol_count >= n,
            "Expected >= {n} symbols, got {}",
            r.build_stats.symbol_count,
        );
    }
}

// ===========================================================================
// 4. Parser code is non-empty (5 properties)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(8))]

    #[test]
    fn parser_code_nonempty_minimal(name in grammar_name_strategy()) {
        let r = build_json_ok(&minimal_json(&name));
        prop_assert!(!r.parser_code.is_empty());
    }

    #[test]
    fn parser_code_nonempty_choice(name in grammar_name_strategy()) {
        let r = build_json_ok(&choice_json(&name, &["ab", "cd"]));
        prop_assert!(!r.parser_code.is_empty());
    }

    #[test]
    fn parser_code_nonempty_seq(name in grammar_name_strategy()) {
        let r = build_json_ok(&seq_json(&name, &["ee", "ff"]));
        prop_assert!(!r.parser_code.is_empty());
    }

    #[test]
    fn parser_code_nonempty_repeat(name in grammar_name_strategy()) {
        let r = build_json_ok(&repeat_json(&name, "rval"));
        prop_assert!(!r.parser_code.is_empty());
    }

    #[test]
    fn parser_path_nonempty(name in grammar_name_strategy()) {
        let r = build_json_ok(&minimal_json(&name));
        prop_assert!(!r.parser_path.is_empty());
    }
}

// ===========================================================================
// 5. Node types is valid JSON (5 properties)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(8))]

    #[test]
    fn node_types_is_json_array(name in grammar_name_strategy()) {
        let r = build_json_ok(&minimal_json(&name));
        let v: serde_json::Value = serde_json::from_str(&r.node_types_json).unwrap();
        prop_assert!(v.is_array());
    }

    #[test]
    fn node_types_entries_have_type_field(name in grammar_name_strategy()) {
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
    fn node_types_type_fields_are_strings(name in grammar_name_strategy()) {
        let r = build_json_ok(&symbol_ref_json(&name, "tok"));
        let arr: Vec<serde_json::Value> = serde_json::from_str(&r.node_types_json).unwrap();
        for entry in &arr {
            if let Some(t) = entry.get("type") {
                prop_assert!(t.is_string(), "'type' field should be a string");
            }
        }
    }

    #[test]
    fn node_types_nonempty_for_symbol_grammar(name in grammar_name_strategy()) {
        let r = build_json_ok(&symbol_ref_json(&name, "tok"));
        let arr: Vec<serde_json::Value> = serde_json::from_str(&r.node_types_json).unwrap();
        prop_assert!(!arr.is_empty(), "node_types should not be empty");
    }

    #[test]
    fn node_types_contains_source_file(name in grammar_name_strategy()) {
        let r = build_json_ok(&symbol_ref_json(&name, "tok"));
        let arr: Vec<serde_json::Value> = serde_json::from_str(&r.node_types_json).unwrap();
        let has_source_file = arr.iter().any(|entry| {
            entry.get("type").and_then(|v| v.as_str()) == Some("source_file")
        });
        prop_assert!(has_source_file, "node_types should include source_file");
    }
}

// ===========================================================================
// 6. Build is deterministic (5 properties)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(6))]

    #[test]
    fn deterministic_parser_code(name in grammar_name_strategy()) {
        let g = minimal_json(&name);
        let r1 = build_json_ok(&g);
        let r2 = build_json_ok(&g);
        prop_assert_eq!(r1.parser_code, r2.parser_code);
    }

    #[test]
    fn deterministic_node_types(name in grammar_name_strategy()) {
        let g = minimal_json(&name);
        let r1 = build_json_ok(&g);
        let r2 = build_json_ok(&g);
        prop_assert_eq!(r1.node_types_json, r2.node_types_json);
    }

    #[test]
    fn deterministic_state_count(name in grammar_name_strategy()) {
        let g = choice_json(&name, &["ab", "cd", "ef"]);
        let r1 = build_json_ok(&g);
        let r2 = build_json_ok(&g);
        prop_assert_eq!(r1.build_stats.state_count, r2.build_stats.state_count);
    }

    #[test]
    fn deterministic_symbol_count(name in grammar_name_strategy()) {
        let g = choice_json(&name, &["ab", "cd", "ef"]);
        let r1 = build_json_ok(&g);
        let r2 = build_json_ok(&g);
        prop_assert_eq!(r1.build_stats.symbol_count, r2.build_stats.symbol_count);
    }

    #[test]
    fn deterministic_conflict_cells(name in grammar_name_strategy()) {
        let g = minimal_json(&name);
        let r1 = build_json_ok(&g);
        let r2 = build_json_ok(&g);
        prop_assert_eq!(r1.build_stats.conflict_cells, r2.build_stats.conflict_cells);
    }
}

// ===========================================================================
// 7. Grammar with more rules produces larger output (5 properties)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(6))]

    #[test]
    fn monotonic_choice_symbol_count(n in 2..=4usize) {
        let small = &ALTS_A[..n];
        let large = &ALTS_A[..n + 1];
        let r_s = build_json_ok(&choice_json("mcs", small));
        let r_l = build_json_ok(&choice_json("mcl", large));
        prop_assert!(
            r_l.build_stats.symbol_count >= r_s.build_stats.symbol_count,
            "More alts should not decrease symbol count",
        );
    }

    #[test]
    fn monotonic_choice_state_count(n in 2..=4usize) {
        let small = &ALTS_B[..n];
        let large = &ALTS_B[..n + 1];
        let r_s = build_json_ok(&choice_json("mss", small));
        let r_l = build_json_ok(&choice_json("msl", large));
        prop_assert!(
            r_l.build_stats.state_count >= r_s.build_stats.state_count,
            "More alts should not decrease state count",
        );
    }

    #[test]
    fn monotonic_seq_code_length(n in 2..=4usize) {
        let small = &SEQ_VALS[..n];
        let large = &SEQ_VALS[..n + 1];
        let r_s = build_json_ok(&seq_json("sqs", small));
        let r_l = build_json_ok(&seq_json("sql", large));
        prop_assert!(
            r_l.parser_code.len() >= r_s.parser_code.len(),
            "Longer SEQ should produce equal or longer parser code",
        );
    }

    #[test]
    fn monotonic_choice_node_types_length(n in 2..=4usize) {
        let small = &ALTS_A[..n];
        let large = &ALTS_A[..n + 1];
        let r_s = build_json_ok(&choice_json("nts", small));
        let r_l = build_json_ok(&choice_json("ntl", large));
        prop_assert!(
            r_l.node_types_json.len() >= r_s.node_types_json.len(),
            "More alts should produce equal or longer node_types",
        );
    }

    #[test]
    fn monotonic_ir_symbol_count(n in 2..=4usize) {
        let tok_names: Vec<String> = (0..n).map(|i| format!("t{i}")).collect();
        let tok_pats: Vec<String> = (0..n).map(|i| format!("p{i}")).collect();
        let pairs: Vec<(&str, &str)> = tok_names
            .iter()
            .zip(tok_pats.iter())
            .map(|(a, b)| (a.as_str(), b.as_str()))
            .collect();
        let rules: Vec<(&str, Vec<&str>)> = tok_names
            .iter()
            .map(|t| ("s", vec![t.as_str()]))
            .collect();
        let r_n = build_ir_ok("irmono", &pairs, &rules, "s");

        let tok_names2: Vec<String> = (0..n + 1).map(|i| format!("t{i}")).collect();
        let tok_pats2: Vec<String> = (0..n + 1).map(|i| format!("p{i}")).collect();
        let pairs2: Vec<(&str, &str)> = tok_names2
            .iter()
            .zip(tok_pats2.iter())
            .map(|(a, b)| (a.as_str(), b.as_str()))
            .collect();
        let rules2: Vec<(&str, Vec<&str>)> = tok_names2
            .iter()
            .map(|t| ("s", vec![t.as_str()]))
            .collect();
        let r_n1 = build_ir_ok("irmono", &pairs2, &rules2, "s");

        prop_assert!(r_n1.build_stats.symbol_count >= r_n.build_stats.symbol_count);
    }
}

// ===========================================================================
// 8. Invalid inputs produce errors (not panics) (5 properties)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(8))]

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
    fn error_empty_ir_grammar_fails(name in grammar_name_strategy()) {
        let g = GrammarBuilder::new(&name).build();
        let result = build_parser(g, test_opts());
        prop_assert!(result.is_err(), "empty grammar should fail");
    }

    #[test]
    fn error_message_is_nonempty_on_failure(name in grammar_name_strategy()) {
        let g = GrammarBuilder::new(&name).build();
        if let Err(e) = build_parser(g, test_opts()) {
            let msg = format!("{e}");
            prop_assert!(!msg.is_empty());
        }
    }

    #[test]
    fn error_missing_name_field(s in "[a-z]{2,6}") {
        let g = json!({
            "rules": {
                "source_file": { "type": "STRING", "value": s }
            }
        });
        let r = build_parser_from_json(g.to_string(), test_opts());
        prop_assert!(r.is_err(), "grammar without name should fail");
    }
}

// ===========================================================================
// 9. Edge cases (6 properties)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(6))]

    #[test]
    fn edge_extras_grammar_builds(name in grammar_name_strategy()) {
        let r = build_json_ok(&extras_json(&name));
        prop_assert!(!r.parser_code.is_empty());
        prop_assert!(r.build_stats.state_count > 0);
    }

    #[test]
    fn edge_repeat_grammar_builds(name in grammar_name_strategy()) {
        let r = build_json_ok(&repeat_json(&name, "rv"));
        prop_assert!(r.build_stats.state_count > 0);
    }

    #[test]
    fn edge_repeat1_grammar_builds(name in grammar_name_strategy()) {
        let r = build_json_ok(&repeat1_json(&name, "rv"));
        prop_assert!(r.build_stats.state_count > 0);
    }

    #[test]
    fn edge_ir_path_matches_json_name(name in grammar_name_strategy()) {
        let ir_r = build_ir_ok(&name, &[("tok", "hello")], &[("s", vec!["tok"])], "s");
        let json_r = build_json_ok(&minimal_json(&name));
        prop_assert_eq!(ir_r.grammar_name, json_r.grammar_name);
    }

    #[test]
    fn edge_single_char_token(name in grammar_name_strategy()) {
        let g = json!({
            "name": name,
            "rules": {
                "source_file": { "type": "STRING", "value": "x" }
            }
        });
        let r = build_json_ok(&g);
        prop_assert!(!r.parser_code.is_empty());
    }

    #[test]
    fn edge_prec_left_grammar_builds(name in grammar_name_strategy()) {
        let g = json!({
            "name": name,
            "rules": {
                "source_file": {
                    "type": "PREC_LEFT",
                    "value": 1,
                    "content": { "type": "STRING", "value": "tok" }
                }
            }
        });
        let r = build_json_ok(&g);
        prop_assert!(r.build_stats.state_count > 0);
    }
}
