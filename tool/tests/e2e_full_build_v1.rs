//! End-to-end integration tests for the full adze-tool build pipeline.
//!
//! Pipeline: JSON grammar string → `build_parser_from_json()` → `BuildResult`
//! → verify grammar name, parse table stats, parser code, node types JSON.

use adze_tool::pure_rust_builder::{BuildOptions, build_parser_from_json};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn default_opts() -> BuildOptions {
    BuildOptions {
        out_dir: "target/debug".to_string(),
        emit_artifacts: false,
        compress_tables: false,
    }
}

fn build_ok(json: &str) -> adze_tool::pure_rust_builder::BuildResult {
    build_parser_from_json(json.to_string(), default_opts())
        .unwrap_or_else(|e| panic!("build_parser_from_json failed: {e}"))
}

/// Convenience: parse `node_types_json` into a `serde_json::Value` array.
fn node_types(r: &adze_tool::pure_rust_builder::BuildResult) -> Vec<serde_json::Value> {
    let v: serde_json::Value =
        serde_json::from_str(&r.node_types_json).expect("node_types_json is not valid JSON");
    v.as_array()
        .expect("node_types_json is not an array")
        .clone()
}

/// Collect the `"type"` strings from every entry in node_types_json.
fn node_type_names(r: &adze_tool::pure_rust_builder::BuildResult) -> Vec<String> {
    node_types(r)
        .iter()
        .filter_map(|entry| entry.get("type").and_then(|v| v.as_str()).map(String::from))
        .collect()
}

/// Collect the `"type"` strings for *named* entries only.
fn named_node_type_names(r: &adze_tool::pure_rust_builder::BuildResult) -> Vec<String> {
    node_types(r)
        .iter()
        .filter(|e| e.get("named").and_then(|v| v.as_bool()) == Some(true))
        .filter_map(|e| e.get("type").and_then(|v| v.as_str()).map(String::from))
        .collect()
}

// ---------------------------------------------------------------------------
// Grammars used across categories
// ---------------------------------------------------------------------------

const SIMPLE_STRING: &str = r#"{
    "name": "simple",
    "rules": {
        "start": { "type": "STRING", "value": "hello" }
    }
}"#;

const SIMPLE_PATTERN: &str = r#"{
    "name": "pat",
    "rules": {
        "identifier": { "type": "PATTERN", "value": "[a-z]+" }
    }
}"#;

const SIMPLE_SEQ: &str = r#"{
    "name": "seqgrammar",
    "rules": {
        "pair": {
            "type": "SEQ",
            "members": [
                { "type": "STRING", "value": "(" },
                { "type": "STRING", "value": ")" }
            ]
        }
    }
}"#;

const SIMPLE_CHOICE: &str = r#"{
    "name": "choicegrammar",
    "rules": {
        "token": {
            "type": "CHOICE",
            "members": [
                { "type": "STRING", "value": "a" },
                { "type": "STRING", "value": "b" }
            ]
        }
    }
}"#;

const SIMPLE_REPEAT: &str = r#"{
    "name": "repgrammar",
    "rules": {
        "items": {
            "type": "REPEAT",
            "content": { "type": "SYMBOL", "name": "item" }
        },
        "item": { "type": "PATTERN", "value": "[a-z]+" }
    }
}"#;

const SIMPLE_REPEAT1: &str = r#"{
    "name": "rep1grammar",
    "rules": {
        "items": {
            "type": "REPEAT1",
            "content": { "type": "PATTERN", "value": "[a-z]+" }
        }
    }
}"#;

const SIMPLE_OPTIONAL: &str = r#"{
    "name": "optgrammar",
    "rules": {
        "maybe_word": {
            "type": "OPTIONAL",
            "content": { "type": "PATTERN", "value": "[a-z]+" }
        }
    }
}"#;

const ARITHMETIC: &str = r#"{
    "name": "arithmetic",
    "rules": {
        "expression": {
            "type": "CHOICE",
            "members": [
                {
                    "type": "SEQ",
                    "members": [
                        { "type": "SYMBOL", "name": "expression" },
                        { "type": "STRING", "value": "+" },
                        { "type": "SYMBOL", "name": "expression" }
                    ]
                },
                {
                    "type": "SEQ",
                    "members": [
                        { "type": "SYMBOL", "name": "expression" },
                        { "type": "STRING", "value": "*" },
                        { "type": "SYMBOL", "name": "expression" }
                    ]
                },
                { "type": "PATTERN", "value": "[0-9]+" }
            ]
        }
    }
}"#;

const JSONLIKE: &str = r#"{
    "name": "jsonlike",
    "rules": {
        "document": {
            "type": "CHOICE",
            "members": [
                { "type": "SYMBOL", "name": "object" },
                { "type": "SYMBOL", "name": "array" },
                { "type": "SYMBOL", "name": "value" }
            ]
        },
        "object": {
            "type": "SEQ",
            "members": [
                { "type": "STRING", "value": "{" },
                { "type": "OPTIONAL", "content": { "type": "SYMBOL", "name": "pair_item" } },
                { "type": "STRING", "value": "}" }
            ]
        },
        "array": {
            "type": "SEQ",
            "members": [
                { "type": "STRING", "value": "[" },
                { "type": "OPTIONAL", "content": { "type": "SYMBOL", "name": "value" } },
                { "type": "STRING", "value": "]" }
            ]
        },
        "pair_item": {
            "type": "SEQ",
            "members": [
                { "type": "SYMBOL", "name": "value" },
                { "type": "STRING", "value": ":" },
                { "type": "SYMBOL", "name": "value" }
            ]
        },
        "value": {
            "type": "CHOICE",
            "members": [
                { "type": "PATTERN", "value": "[a-zA-Z_]+" },
                { "type": "PATTERN", "value": "[0-9]+" }
            ]
        }
    }
}"#;

// ===========================================================================
// Category 1: Full build from simple JSON grammars (8 tests)
// ===========================================================================

#[test]
fn simple_string_grammar_builds_successfully() {
    let r = build_ok(SIMPLE_STRING);
    assert_eq!(r.grammar_name, "simple");
}

#[test]
fn simple_string_grammar_produces_nonempty_parser_code() {
    let r = build_ok(SIMPLE_STRING);
    assert!(!r.parser_code.is_empty());
}

#[test]
fn simple_pattern_grammar_builds() {
    let r = build_ok(SIMPLE_PATTERN);
    assert_eq!(r.grammar_name, "pat");
    assert!(!r.parser_code.is_empty());
}

#[test]
fn simple_seq_grammar_builds() {
    let r = build_ok(SIMPLE_SEQ);
    assert_eq!(r.grammar_name, "seqgrammar");
    assert!(r.build_stats.state_count > 0);
}

#[test]
fn simple_choice_grammar_builds() {
    let r = build_ok(SIMPLE_CHOICE);
    assert_eq!(r.grammar_name, "choicegrammar");
    assert!(r.build_stats.symbol_count > 0);
}

#[test]
fn simple_repeat_grammar_builds() {
    let r = build_ok(SIMPLE_REPEAT);
    assert_eq!(r.grammar_name, "repgrammar");
}

#[test]
fn simple_repeat1_grammar_builds() {
    let r = build_ok(SIMPLE_REPEAT1);
    assert_eq!(r.grammar_name, "rep1grammar");
}

#[test]
fn simple_optional_grammar_builds() {
    let r = build_ok(SIMPLE_OPTIONAL);
    assert_eq!(r.grammar_name, "optgrammar");
    assert!(!r.node_types_json.is_empty());
}

// ===========================================================================
// Category 2: Full build from complex JSON grammars (8 tests)
// ===========================================================================

#[test]
fn arithmetic_grammar_builds() {
    let r = build_ok(ARITHMETIC);
    assert_eq!(r.grammar_name, "arithmetic");
}

#[test]
fn arithmetic_grammar_has_positive_state_count() {
    let r = build_ok(ARITHMETIC);
    assert!(
        r.build_stats.state_count >= 3,
        "arithmetic should have ≥3 states, got {}",
        r.build_stats.state_count
    );
}

#[test]
fn arithmetic_grammar_has_conflict_cells() {
    let r = build_ok(ARITHMETIC);
    assert!(
        r.build_stats.conflict_cells >= 1,
        "ambiguous arithmetic grammar should have ≥1 conflict cell, got {}",
        r.build_stats.conflict_cells
    );
}

#[test]
fn jsonlike_grammar_builds() {
    let r = build_ok(JSONLIKE);
    assert_eq!(r.grammar_name, "jsonlike");
}

#[test]
fn jsonlike_grammar_has_many_states() {
    let r = build_ok(JSONLIKE);
    assert!(
        r.build_stats.state_count >= 5,
        "jsonlike should have ≥5 states, got {}",
        r.build_stats.state_count
    );
}

#[test]
fn jsonlike_grammar_has_many_symbols() {
    let r = build_ok(JSONLIKE);
    assert!(
        r.build_stats.symbol_count >= 5,
        "jsonlike should have ≥5 symbols, got {}",
        r.build_stats.symbol_count
    );
}

#[test]
fn complex_nested_seq_choice_builds() {
    let json = r#"{
        "name": "nested",
        "rules": {
            "root": {
                "type": "SEQ",
                "members": [
                    {
                        "type": "CHOICE",
                        "members": [
                            { "type": "STRING", "value": "x" },
                            { "type": "STRING", "value": "y" }
                        ]
                    },
                    { "type": "STRING", "value": ";" }
                ]
            }
        }
    }"#;
    let r = build_ok(json);
    assert_eq!(r.grammar_name, "nested");
    assert!(r.build_stats.state_count >= 2);
}

#[test]
fn complex_repeat_with_seq_builds() {
    let json = r#"{
        "name": "repseq",
        "rules": {
            "list": {
                "type": "REPEAT",
                "content": {
                    "type": "SEQ",
                    "members": [
                        { "type": "PATTERN", "value": "[a-z]+" },
                        { "type": "STRING", "value": "," }
                    ]
                }
            }
        }
    }"#;
    let r = build_ok(json);
    assert_eq!(r.grammar_name, "repseq");
    assert!(!r.parser_code.is_empty());
}

// ===========================================================================
// Category 3: Build output consistency — grammar matches parse table (8 tests)
// ===========================================================================

#[test]
fn string_grammar_state_count_is_positive() {
    let r = build_ok(SIMPLE_STRING);
    assert!(r.build_stats.state_count > 0);
}

#[test]
fn string_grammar_symbol_count_is_positive() {
    let r = build_ok(SIMPLE_STRING);
    assert!(r.build_stats.symbol_count > 0);
}

#[test]
fn pattern_grammar_symbol_count_positive() {
    let r = build_ok(SIMPLE_PATTERN);
    assert!(r.build_stats.symbol_count > 0);
}

#[test]
fn choice_grammar_symbol_count_at_least_three() {
    // CHOICE { "a", "b" } → at least END + two tokens + nonterminal
    let r = build_ok(SIMPLE_CHOICE);
    assert!(
        r.build_stats.symbol_count >= 3,
        "choice grammar should have ≥3 symbols, got {}",
        r.build_stats.symbol_count
    );
}

#[test]
fn more_rules_means_more_symbols() {
    let r_small = build_ok(SIMPLE_STRING);
    let r_big = build_ok(JSONLIKE);
    assert!(
        r_big.build_stats.symbol_count > r_small.build_stats.symbol_count,
        "jsonlike ({}) should have more symbols than simple ({})",
        r_big.build_stats.symbol_count,
        r_small.build_stats.symbol_count
    );
}

#[test]
fn more_rules_means_more_states() {
    let r_small = build_ok(SIMPLE_STRING);
    let r_big = build_ok(JSONLIKE);
    assert!(
        r_big.build_stats.state_count > r_small.build_stats.state_count,
        "jsonlike ({}) should have more states than simple ({})",
        r_big.build_stats.state_count,
        r_small.build_stats.state_count
    );
}

#[test]
fn simple_grammar_has_zero_or_few_conflicts() {
    let r = build_ok(SIMPLE_STRING);
    assert!(
        r.build_stats.conflict_cells <= 1,
        "simple STRING grammar should have ≤1 conflict, got {}",
        r.build_stats.conflict_cells
    );
}

#[test]
fn parser_code_length_scales_with_grammar_size() {
    let r_small = build_ok(SIMPLE_STRING);
    let r_big = build_ok(JSONLIKE);
    assert!(
        r_big.parser_code.len() > r_small.parser_code.len(),
        "jsonlike code ({}) should be longer than simple ({})",
        r_big.parser_code.len(),
        r_small.parser_code.len()
    );
}

// ===========================================================================
// Category 4: Parser code contains expected patterns (8 tests)
// ===========================================================================

#[test]
fn parser_code_contains_language_constant() {
    let r = build_ok(SIMPLE_STRING);
    assert!(
        r.parser_code.contains("LANGUAGE"),
        "parser code must define LANGUAGE"
    );
}

#[test]
fn parser_code_contains_tslanguage_type() {
    let r = build_ok(SIMPLE_STRING);
    assert!(
        r.parser_code.contains("TSLanguage"),
        "parser code must reference TSLanguage"
    );
}

#[test]
fn parser_code_contains_tree_sitter_function() {
    let r = build_ok(SIMPLE_STRING);
    assert!(
        r.parser_code.contains("tree_sitter_"),
        "parser code must have a tree_sitter_<name> function"
    );
}

#[test]
fn parser_code_contains_extern_c() {
    let r = build_ok(SIMPLE_STRING);
    assert!(
        r.parser_code.contains("extern"),
        "parser code must use extern calling convention"
    );
}

#[test]
fn parser_code_contains_symbol_metadata() {
    let r = build_ok(SIMPLE_STRING);
    assert!(
        r.parser_code.contains("SYMBOL_METADATA"),
        "parser code must contain SYMBOL_METADATA table"
    );
}

#[test]
fn parser_code_contains_parse_table() {
    let r = build_ok(SIMPLE_STRING);
    assert!(
        r.parser_code.contains("PARSE_TABLE") || r.parser_code.contains("SMALL_PARSE_TABLE"),
        "parser code must contain a parse table"
    );
}

#[test]
fn parser_code_contains_lexer_fn() {
    let r = build_ok(SIMPLE_STRING);
    assert!(
        r.parser_code.contains("lexer_fn"),
        "parser code must contain lexer_fn"
    );
}

#[test]
fn parser_code_references_grammar_name() {
    let r = build_ok(SIMPLE_STRING);
    assert!(
        r.parser_code.contains("tree_sitter_simple"),
        "parser code should reference tree_sitter_simple"
    );
}

// ===========================================================================
// Category 5: Node types JSON matches grammar symbols (8 tests)
// ===========================================================================

#[test]
fn node_types_json_is_valid_json_array() {
    let r = build_ok(SIMPLE_STRING);
    let parsed: serde_json::Value = serde_json::from_str(&r.node_types_json).unwrap();
    assert!(parsed.is_array());
}

#[test]
fn node_types_entries_have_type_field() {
    let r = build_ok(SIMPLE_STRING);
    let entries = node_types(&r);
    assert!(!entries.is_empty());
    for entry in &entries {
        assert!(
            entry.get("type").is_some(),
            "every node type entry must have a \"type\" field"
        );
    }
}

#[test]
fn node_types_entries_have_named_field() {
    let r = build_ok(SIMPLE_STRING);
    for entry in &node_types(&r) {
        assert!(
            entry.get("named").is_some(),
            "every node type entry must have a \"named\" field"
        );
    }
}

#[test]
fn simple_grammar_includes_start_rule_in_node_types() {
    let r = build_ok(SIMPLE_STRING);
    let names = node_type_names(&r);
    assert!(
        names.contains(&"start".to_string()),
        "node types should include the 'start' rule, got: {names:?}"
    );
}

#[test]
fn jsonlike_grammar_includes_all_named_rules() {
    let r = build_ok(JSONLIKE);
    let named = named_node_type_names(&r);
    for expected in ["document", "object", "array", "pair_item", "value"] {
        assert!(
            named.contains(&expected.to_string()),
            "node types should include named rule '{expected}', got: {named:?}"
        );
    }
}

#[test]
fn arithmetic_node_types_include_expression() {
    let r = build_ok(ARITHMETIC);
    let named = named_node_type_names(&r);
    assert!(
        named.contains(&"expression".to_string()),
        "arithmetic node types should include 'expression', got: {named:?}"
    );
}

#[test]
fn repeat_grammar_node_types_include_item() {
    let r = build_ok(SIMPLE_REPEAT);
    let named = named_node_type_names(&r);
    assert!(
        named.contains(&"item".to_string()),
        "repeat grammar should include 'item' in node types, got: {named:?}"
    );
}

#[test]
fn jsonlike_node_types_include_punctuation() {
    let r = build_ok(JSONLIKE);
    let all = node_type_names(&r);
    for punct in ["{", "}", "[", "]", ":"] {
        assert!(
            all.contains(&punct.to_string()),
            "jsonlike node types should include punctuation '{punct}', got: {all:?}"
        );
    }
}

// ===========================================================================
// Category 6: Build stats match actual grammar/table properties (8 tests)
// ===========================================================================

#[test]
fn stats_state_count_is_nonzero_for_any_grammar() {
    for json in [SIMPLE_STRING, SIMPLE_PATTERN, SIMPLE_SEQ, SIMPLE_CHOICE] {
        let r = build_ok(json);
        assert!(r.build_stats.state_count > 0, "state_count must be > 0");
    }
}

#[test]
fn stats_symbol_count_is_nonzero_for_any_grammar() {
    for json in [SIMPLE_STRING, SIMPLE_PATTERN, SIMPLE_SEQ, SIMPLE_CHOICE] {
        let r = build_ok(json);
        assert!(r.build_stats.symbol_count > 0, "symbol_count must be > 0");
    }
}

#[test]
fn stats_conflict_cells_zero_for_unambiguous_grammar() {
    let r = build_ok(SIMPLE_SEQ);
    assert_eq!(
        r.build_stats.conflict_cells, 0,
        "deterministic SEQ grammar should have 0 conflict cells"
    );
}

#[test]
fn stats_conflict_cells_positive_for_ambiguous_grammar() {
    let r = build_ok(ARITHMETIC);
    assert!(
        r.build_stats.conflict_cells > 0,
        "ambiguous arithmetic grammar should have conflict cells"
    );
}

#[test]
fn stats_symbol_count_includes_end_token() {
    // Every grammar has at least END + one token + one nonterminal
    let r = build_ok(SIMPLE_STRING);
    assert!(
        r.build_stats.symbol_count >= 3,
        "symbol_count should be ≥3 (END + token + nonterminal), got {}",
        r.build_stats.symbol_count
    );
}

#[test]
fn stats_jsonlike_has_many_symbols() {
    let r = build_ok(JSONLIKE);
    // jsonlike has 5 rules + several punctuation tokens + END
    assert!(
        r.build_stats.symbol_count >= 10,
        "jsonlike should have ≥10 symbols, got {}",
        r.build_stats.symbol_count
    );
}

#[test]
fn stats_repeat_grammar_has_moderate_states() {
    let r = build_ok(SIMPLE_REPEAT);
    // REPEAT over a single token symbol: should be small
    assert!(
        r.build_stats.state_count >= 2,
        "repeat grammar should have ≥2 states, got {}",
        r.build_stats.state_count
    );
    assert!(
        r.build_stats.state_count <= 20,
        "simple repeat should have ≤20 states, got {}",
        r.build_stats.state_count
    );
}

#[test]
fn stats_node_types_count_matches_symbol_count_roughly() {
    let r = build_ok(JSONLIKE);
    let nt_count = node_types(&r).len();
    // node_types has named + anonymous entries; symbol_count also includes END
    // They should be in the same ballpark
    assert!(
        nt_count >= 2,
        "node_types should have ≥2 entries, got {nt_count}"
    );
    assert!(
        (nt_count as isize - r.build_stats.symbol_count as isize).unsigned_abs() <= 10,
        "node_types count ({nt_count}) and symbol_count ({}) should be within 10",
        r.build_stats.symbol_count
    );
}

// ===========================================================================
// Category 7: Full build determinism (8 tests)
// ===========================================================================

#[test]
fn determinism_grammar_name_is_stable() {
    let r1 = build_ok(SIMPLE_STRING);
    let r2 = build_ok(SIMPLE_STRING);
    assert_eq!(r1.grammar_name, r2.grammar_name);
}

#[test]
fn determinism_parser_code_is_identical() {
    let r1 = build_ok(SIMPLE_STRING);
    let r2 = build_ok(SIMPLE_STRING);
    assert_eq!(r1.parser_code, r2.parser_code);
}

#[test]
fn determinism_node_types_json_is_identical() {
    let r1 = build_ok(SIMPLE_STRING);
    let r2 = build_ok(SIMPLE_STRING);
    assert_eq!(r1.node_types_json, r2.node_types_json);
}

#[test]
fn determinism_state_count_is_stable() {
    let r1 = build_ok(ARITHMETIC);
    let r2 = build_ok(ARITHMETIC);
    assert_eq!(r1.build_stats.state_count, r2.build_stats.state_count);
}

#[test]
fn determinism_symbol_count_is_stable() {
    let r1 = build_ok(ARITHMETIC);
    let r2 = build_ok(ARITHMETIC);
    assert_eq!(r1.build_stats.symbol_count, r2.build_stats.symbol_count);
}

#[test]
fn determinism_conflict_cells_is_stable() {
    let r1 = build_ok(ARITHMETIC);
    let r2 = build_ok(ARITHMETIC);
    assert_eq!(r1.build_stats.conflict_cells, r2.build_stats.conflict_cells);
}

#[test]
fn determinism_complex_grammar_code_is_stable() {
    let r1 = build_ok(JSONLIKE);
    let r2 = build_ok(JSONLIKE);
    assert_eq!(r1.parser_code, r2.parser_code);
}

#[test]
fn determinism_complex_grammar_node_types_is_stable() {
    let r1 = build_ok(JSONLIKE);
    let r2 = build_ok(JSONLIKE);
    assert_eq!(r1.node_types_json, r2.node_types_json);
}

// ===========================================================================
// Category 8: Error propagation — invalid inputs give good errors (8 tests)
// ===========================================================================

#[test]
fn error_empty_string() {
    let r = build_parser_from_json(String::new(), default_opts());
    assert!(r.is_err(), "empty string should fail");
}

#[test]
fn error_invalid_json_syntax() {
    let r = build_parser_from_json("not json {{{".to_string(), default_opts());
    assert!(r.is_err(), "invalid JSON syntax should fail");
}

#[test]
fn error_json_number() {
    let r = build_parser_from_json("42".to_string(), default_opts());
    assert!(r.is_err(), "bare number should fail");
}

#[test]
fn error_json_array() {
    let r = build_parser_from_json("[]".to_string(), default_opts());
    assert!(r.is_err(), "bare array should fail");
}

#[test]
fn error_json_null() {
    let r = build_parser_from_json("null".to_string(), default_opts());
    assert!(r.is_err(), "null should fail");
}

#[test]
fn error_missing_rules_key() {
    let r = build_parser_from_json(r#"{"name":"t"}"#.to_string(), default_opts());
    assert!(r.is_err(), "missing 'rules' key should fail");
}

#[test]
fn error_empty_rules_object() {
    let r = build_parser_from_json(r#"{"name":"t","rules":{}}"#.to_string(), default_opts());
    assert!(r.is_err(), "empty rules should fail");
}

#[test]
fn error_unknown_rule_type() {
    let json = r#"{
        "name": "bad",
        "rules": {
            "start": { "type": "NONEXISTENT_TYPE", "value": "x" }
        }
    }"#;
    let r = build_parser_from_json(json.to_string(), default_opts());
    assert!(r.is_err(), "unknown rule type should fail");
}
