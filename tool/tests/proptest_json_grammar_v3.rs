//! Property-based tests for adze-tool JSON grammar parsing via `build_parser_from_json`.
//!
//! Covers:
//! 1. Grammar name preserved proptest (5)
//! 2. Build is deterministic proptest (5)
//! 3. Stats positive proptest (5)
//! 4. More tokens → more symbols proptest (5)
//! 5. Token patterns proptest (5)
//! 6. Regular JSON grammar tests (15)
//! 7. Error handling tests (10)

use adze_tool::pure_rust_builder::{BuildOptions, BuildResult, build_parser_from_json};
use proptest::prelude::*;
use serde_json::json;

// ===========================================================================
// Helpers
// ===========================================================================

fn test_opts() -> BuildOptions {
    BuildOptions {
        out_dir: "/tmp/proptest_json_v3".to_string(),
        emit_artifacts: false,
        compress_tables: false,
    }
}

/// Build a grammar from a JSON value, expecting success.
fn build_json_ok(value: &serde_json::Value) -> BuildResult {
    build_parser_from_json(value.to_string(), test_opts()).expect("build_json_ok: should succeed")
}

/// Build a minimal grammar: one STRING rule named `source_file`.
fn minimal_grammar(name: &str) -> serde_json::Value {
    json!({
        "name": name,
        "rules": {
            "source_file": { "type": "STRING", "value": "hello" }
        }
    })
}

/// Build a grammar with N CHOICE alternatives for `source_file`.
fn choice_grammar(name: &str, alternatives: &[&str]) -> serde_json::Value {
    let members: Vec<serde_json::Value> = alternatives
        .iter()
        .map(|s| json!({ "type": "STRING", "value": s }))
        .collect();
    json!({
        "name": name,
        "rules": {
            "source_file": {
                "type": "CHOICE",
                "members": members
            }
        }
    })
}

/// Build a SEQ grammar with N string tokens in sequence.
fn seq_grammar(name: &str, tokens: &[&str]) -> serde_json::Value {
    let members: Vec<serde_json::Value> = tokens
        .iter()
        .map(|s| json!({ "type": "STRING", "value": s }))
        .collect();
    json!({
        "name": name,
        "rules": {
            "source_file": {
                "type": "SEQ",
                "members": members
            }
        }
    })
}

/// Build a grammar with a PATTERN rule.
fn pattern_grammar(name: &str, pattern: &str) -> serde_json::Value {
    json!({
        "name": name,
        "rules": {
            "source_file": { "type": "PATTERN", "value": pattern }
        }
    })
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
    ]
}

fn token_value_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("foo".to_string()),
        Just("bar".to_string()),
        Just("baz".to_string()),
        Just("qux".to_string()),
        Just("quux".to_string()),
    ]
}

fn pattern_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("[a-z]+".to_string()),
        Just("[0-9]+".to_string()),
        Just("[A-Z][a-z]*".to_string()),
        Just("[a-zA-Z_]+".to_string()),
        Just("[0-9a-fA-F]+".to_string()),
    ]
}

// ===========================================================================
// 1. Grammar name preserved proptest (5 tests)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(8))]

    #[test]
    fn name_preserved_minimal(name in grammar_name_strategy()) {
        let g = minimal_grammar(&name);
        let r = build_json_ok(&g);
        prop_assert_eq!(r.grammar_name, name);
    }

    #[test]
    fn name_preserved_choice(name in grammar_name_strategy()) {
        let g = choice_grammar(&name, &["x", "y"]);
        let r = build_json_ok(&g);
        prop_assert_eq!(r.grammar_name, name);
    }

    #[test]
    fn name_preserved_pattern(name in grammar_name_strategy()) {
        let g = pattern_grammar(&name, "[a-z]+");
        let r = build_json_ok(&g);
        prop_assert_eq!(r.grammar_name, name);
    }

    #[test]
    fn name_preserved_seq(name in grammar_name_strategy()) {
        let g = seq_grammar(&name, &["a", "b"]);
        let r = build_json_ok(&g);
        prop_assert_eq!(r.grammar_name, name);
    }

    #[test]
    fn name_appears_in_code(name in grammar_name_strategy()) {
        let g = minimal_grammar(&name);
        let r = build_json_ok(&g);
        prop_assert!(
            r.parser_code.contains(&name),
            "parser_code should contain grammar name '{}'",
            name
        );
    }
}

// ===========================================================================
// 2. Build is deterministic proptest (5 tests)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(6))]

    #[test]
    fn deterministic_parser_code(name in grammar_name_strategy()) {
        let g = minimal_grammar(&name);
        let r1 = build_json_ok(&g);
        let r2 = build_json_ok(&g);
        prop_assert_eq!(r1.parser_code, r2.parser_code);
    }

    #[test]
    fn deterministic_node_types(name in grammar_name_strategy()) {
        let g = minimal_grammar(&name);
        let r1 = build_json_ok(&g);
        let r2 = build_json_ok(&g);
        prop_assert_eq!(r1.node_types_json, r2.node_types_json);
    }

    #[test]
    fn deterministic_state_count(name in grammar_name_strategy()) {
        let g = minimal_grammar(&name);
        let r1 = build_json_ok(&g);
        let r2 = build_json_ok(&g);
        prop_assert_eq!(r1.build_stats.state_count, r2.build_stats.state_count);
    }

    #[test]
    fn deterministic_symbol_count(name in grammar_name_strategy()) {
        let g = minimal_grammar(&name);
        let r1 = build_json_ok(&g);
        let r2 = build_json_ok(&g);
        prop_assert_eq!(r1.build_stats.symbol_count, r2.build_stats.symbol_count);
    }

    #[test]
    fn deterministic_choice_grammar(name in grammar_name_strategy()) {
        let g = choice_grammar(&name, &["x", "y", "z"]);
        let r1 = build_json_ok(&g);
        let r2 = build_json_ok(&g);
        prop_assert_eq!(r1.parser_code, r2.parser_code);
        prop_assert_eq!(r1.build_stats.state_count, r2.build_stats.state_count);
    }
}

// ===========================================================================
// 3. Stats positive proptest (5 tests)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(8))]

    #[test]
    fn stats_state_count_positive(name in grammar_name_strategy()) {
        let g = minimal_grammar(&name);
        let r = build_json_ok(&g);
        prop_assert!(r.build_stats.state_count > 0);
    }

    #[test]
    fn stats_symbol_count_positive(name in grammar_name_strategy()) {
        let g = minimal_grammar(&name);
        let r = build_json_ok(&g);
        prop_assert!(r.build_stats.symbol_count > 0);
    }

    #[test]
    fn stats_choice_has_multiple_symbols(name in grammar_name_strategy()) {
        let g = choice_grammar(&name, &["a", "b", "c"]);
        let r = build_json_ok(&g);
        // 3 terminal strings + EOF + nonterminal(s)
        prop_assert!(r.build_stats.symbol_count >= 3);
    }

    #[test]
    fn stats_parser_code_nonempty(name in grammar_name_strategy()) {
        let g = minimal_grammar(&name);
        let r = build_json_ok(&g);
        prop_assert!(!r.parser_code.is_empty());
    }

    #[test]
    fn stats_node_types_valid_json(name in grammar_name_strategy()) {
        let g = minimal_grammar(&name);
        let r = build_json_ok(&g);
        let parsed: Result<serde_json::Value, _> = serde_json::from_str(&r.node_types_json);
        prop_assert!(parsed.is_ok(), "node_types_json must be valid JSON");
    }
}

// ===========================================================================
// 4. More tokens → more symbols proptest (5 tests)
// ===========================================================================

const ALTS_A: [&str; 5] = ["a", "b", "c", "d", "e"];
const ALTS_F: [&str; 5] = ["f", "g", "h", "i", "j"];
const ALTS_U: [&str; 5] = ["u", "v", "w", "x", "y"];
const SEQ_TOKENS: [&str; 5] = ["p", "q", "r", "ss", "tt"];

proptest! {
    #![proptest_config(ProptestConfig::with_cases(6))]

    #[test]
    fn monotonic_choice_symbols(n in 2..=4usize) {
        let small_alts = &ALTS_A[..n];
        let large_alts = &ALTS_A[..n + 1];
        let r_small = build_json_ok(&choice_grammar("mono_s", small_alts));
        let r_large = build_json_ok(&choice_grammar("mono_l", large_alts));
        prop_assert!(
            r_large.build_stats.symbol_count >= r_small.build_stats.symbol_count,
            "More alternatives should not decrease symbol count: {} vs {}",
            r_small.build_stats.symbol_count,
            r_large.build_stats.symbol_count,
        );
    }

    #[test]
    fn monotonic_choice_states(n in 2..=4usize) {
        let small_alts = &ALTS_F[..n];
        let large_alts = &ALTS_F[..n + 1];
        let r_small = build_json_ok(&choice_grammar("ms", small_alts));
        let r_large = build_json_ok(&choice_grammar("ml", large_alts));
        prop_assert!(
            r_large.build_stats.state_count >= r_small.build_stats.state_count,
            "More alternatives should not decrease state count: {} vs {}",
            r_small.build_stats.state_count,
            r_large.build_stats.state_count,
        );
    }

    #[test]
    fn monotonic_seq_code_grows(n in 2..=4usize) {
        let small_tokens = &SEQ_TOKENS[..n];
        let large_tokens = &SEQ_TOKENS[..n + 1];
        let r_small = build_json_ok(&seq_grammar("seq_s", small_tokens));
        let r_large = build_json_ok(&seq_grammar("seq_l", large_tokens));
        prop_assert!(
            r_large.parser_code.len() >= r_small.parser_code.len(),
            "Longer sequence should produce equal or longer code"
        );
    }

    #[test]
    fn monotonic_single_vs_multi_choice(n in 3..=5usize) {
        let r_one = build_json_ok(&choice_grammar("one", &ALTS_A[..2]));
        let r_many = build_json_ok(&choice_grammar("many", &ALTS_A[..n]));
        prop_assert!(
            r_many.build_stats.symbol_count >= r_one.build_stats.symbol_count,
        );
    }

    #[test]
    fn monotonic_node_types_grows(n in 2..=4usize) {
        let small_alts = &ALTS_U[..n];
        let large_alts = &ALTS_U[..n + 1];
        let r_small = build_json_ok(&choice_grammar("nt_s", small_alts));
        let r_large = build_json_ok(&choice_grammar("nt_l", large_alts));
        prop_assert!(
            r_large.node_types_json.len() >= r_small.node_types_json.len(),
            "More alternatives should produce equal or longer node_types"
        );
    }
}

// ===========================================================================
// 5. Token patterns proptest (5 tests)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(8))]

    #[test]
    fn pattern_builds_successfully(pat in pattern_strategy()) {
        let g = pattern_grammar("pat_test", &pat);
        let r = build_json_ok(&g);
        prop_assert!(r.build_stats.state_count > 0);
    }

    #[test]
    fn pattern_name_preserved(pat in pattern_strategy()) {
        let g = pattern_grammar("pat_name", &pat);
        let r = build_json_ok(&g);
        prop_assert_eq!(r.grammar_name, "pat_name");
    }

    #[test]
    fn pattern_has_symbols(pat in pattern_strategy()) {
        let g = pattern_grammar("pat_sym", &pat);
        let r = build_json_ok(&g);
        prop_assert!(r.build_stats.symbol_count > 0);
    }

    #[test]
    fn pattern_deterministic(pat in pattern_strategy()) {
        let g = pattern_grammar("pat_det", &pat);
        let r1 = build_json_ok(&g);
        let r2 = build_json_ok(&g);
        prop_assert_eq!(r1.build_stats.state_count, r2.build_stats.state_count);
    }

    #[test]
    fn string_token_builds(tok in token_value_strategy()) {
        let g = json!({
            "name": "tok_test",
            "rules": {
                "source_file": { "type": "STRING", "value": tok }
            }
        });
        let r = build_json_ok(&g);
        prop_assert!(!r.parser_code.is_empty());
    }
}

// ===========================================================================
// 6. Regular JSON grammar tests (15 tests)
// ===========================================================================

#[test]
fn regular_minimal_string_grammar() {
    let g = minimal_grammar("reg_min");
    let r = build_json_ok(&g);
    assert_eq!(r.grammar_name, "reg_min");
    assert!(r.build_stats.state_count > 0);
}

#[test]
fn regular_pattern_grammar() {
    let g = pattern_grammar("reg_pat", "[0-9]+");
    let r = build_json_ok(&g);
    assert!(!r.parser_code.is_empty());
}

#[test]
fn regular_choice_two_strings() {
    let g = choice_grammar("reg_ch2", &["yes", "no"]);
    let r = build_json_ok(&g);
    assert!(r.build_stats.symbol_count >= 2);
}

#[test]
fn regular_choice_five_strings() {
    let g = choice_grammar("reg_ch5", &["a", "b", "c", "d", "e"]);
    let r = build_json_ok(&g);
    assert!(r.build_stats.symbol_count >= 5);
}

#[test]
fn regular_seq_two_tokens() {
    let g = seq_grammar("reg_seq2", &["hello", "world"]);
    let r = build_json_ok(&g);
    assert!(r.build_stats.state_count > 0);
}

#[test]
fn regular_seq_three_tokens() {
    let g = seq_grammar("reg_seq3", &["a", "b", "c"]);
    let r = build_json_ok(&g);
    assert!(!r.parser_code.is_empty());
}

#[test]
fn regular_nested_choice_in_seq() {
    let g = json!({
        "name": "reg_nested",
        "rules": {
            "source_file": {
                "type": "SEQ",
                "members": [
                    {
                        "type": "CHOICE",
                        "members": [
                            { "type": "STRING", "value": "x" },
                            { "type": "STRING", "value": "y" }
                        ]
                    },
                    { "type": "STRING", "value": "z" }
                ]
            }
        }
    });
    let r = build_json_ok(&g);
    assert!(r.build_stats.state_count > 0);
}

#[test]
fn regular_symbol_reference() {
    let g = json!({
        "name": "reg_sym",
        "rules": {
            "source_file": { "type": "SYMBOL", "name": "item" },
            "item": { "type": "STRING", "value": "tok" }
        }
    });
    let r = build_json_ok(&g);
    assert!(r.build_stats.symbol_count > 0);
}

#[test]
fn regular_repeat_rule() {
    let g = json!({
        "name": "reg_rep",
        "rules": {
            "source_file": {
                "type": "REPEAT",
                "content": { "type": "STRING", "value": "item" }
            }
        }
    });
    let r = build_json_ok(&g);
    assert!(r.build_stats.state_count > 0);
}

#[test]
fn regular_repeat1_rule() {
    let g = json!({
        "name": "reg_rep1",
        "rules": {
            "source_file": {
                "type": "REPEAT1",
                "content": { "type": "STRING", "value": "thing" }
            }
        }
    });
    let r = build_json_ok(&g);
    assert!(!r.parser_code.is_empty());
}

#[test]
fn regular_optional_rule() {
    let g = json!({
        "name": "reg_opt",
        "rules": {
            "source_file": {
                "type": "SEQ",
                "members": [
                    { "type": "STRING", "value": "start" },
                    {
                        "type": "CHOICE",
                        "members": [
                            { "type": "STRING", "value": "mid" },
                            { "type": "BLANK" }
                        ]
                    },
                    { "type": "STRING", "value": "end" }
                ]
            }
        }
    });
    let r = build_json_ok(&g);
    assert!(r.build_stats.state_count > 0);
}

#[test]
fn regular_with_extras() {
    let g = json!({
        "name": "reg_extras",
        "rules": {
            "source_file": { "type": "STRING", "value": "hello" }
        },
        "extras": [
            { "type": "PATTERN", "value": "\\s" }
        ]
    });
    let r = build_json_ok(&g);
    assert!(r.build_stats.symbol_count > 0);
}

#[test]
fn regular_node_types_is_array() {
    let g = minimal_grammar("reg_nt");
    let r = build_json_ok(&g);
    let v: serde_json::Value = serde_json::from_str(&r.node_types_json).unwrap();
    assert!(v.is_array(), "node_types_json should be a JSON array");
}

#[test]
fn regular_compressed_tables() {
    let opts = BuildOptions {
        out_dir: "/tmp/proptest_json_v3_comp".to_string(),
        emit_artifacts: false,
        compress_tables: true,
    };
    let g = minimal_grammar("reg_comp");
    let r = build_parser_from_json(g.to_string(), opts).unwrap();
    assert!(!r.parser_code.is_empty());
}

#[test]
fn regular_prec_left() {
    let g = json!({
        "name": "reg_prec",
        "rules": {
            "source_file": { "type": "SYMBOL", "name": "expr" },
            "expr": {
                "type": "CHOICE",
                "members": [
                    { "type": "PATTERN", "value": "[0-9]+" },
                    {
                        "type": "PREC_LEFT",
                        "value": 1,
                        "content": {
                            "type": "SEQ",
                            "members": [
                                { "type": "SYMBOL", "name": "expr" },
                                { "type": "STRING", "value": "+" },
                                { "type": "SYMBOL", "name": "expr" }
                            ]
                        }
                    }
                ]
            }
        }
    });
    let r = build_json_ok(&g);
    assert!(r.build_stats.state_count > 0);
}

// ===========================================================================
// 7. Error handling tests (10 tests)
// ===========================================================================

#[test]
fn error_empty_string() {
    let r = build_parser_from_json(String::new(), test_opts());
    assert!(r.is_err());
}

#[test]
fn error_not_json() {
    let r = build_parser_from_json("this is not json".to_string(), test_opts());
    assert!(r.is_err());
}

#[test]
fn error_json_number() {
    let r = build_parser_from_json("42".to_string(), test_opts());
    assert!(r.is_err());
}

#[test]
fn error_json_array() {
    let r = build_parser_from_json("[]".to_string(), test_opts());
    assert!(r.is_err());
}

#[test]
fn error_json_null() {
    let r = build_parser_from_json("null".to_string(), test_opts());
    assert!(r.is_err());
}

#[test]
fn error_json_bool() {
    let r = build_parser_from_json("true".to_string(), test_opts());
    assert!(r.is_err());
}

#[test]
fn error_empty_object() {
    let r = build_parser_from_json("{}".to_string(), test_opts());
    assert!(r.is_err());
}

#[test]
fn error_name_only_no_rules() {
    let g = json!({"name": "orphan"});
    let r = build_parser_from_json(g.to_string(), test_opts());
    assert!(r.is_err());
}

#[test]
fn error_rules_wrong_type() {
    let g = json!({"name": "bad", "rules": "not_an_object"});
    let r = build_parser_from_json(g.to_string(), test_opts());
    assert!(r.is_err());
}

#[test]
fn error_message_is_nonempty() {
    let r = build_parser_from_json(String::new(), test_opts());
    if let Err(e) = r {
        let msg = format!("{e}");
        assert!(!msg.is_empty(), "Error message should not be empty");
    }
}
