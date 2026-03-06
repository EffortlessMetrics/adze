//! Comprehensive tests for adze-tool grammar converter (Rust types → JSON grammar).
//!
//! 55+ tests covering:
//! 1. Simple grammar conversion (8 tests)
//! 2. Multi-rule conversion (8 tests)
//! 3. Pattern types (8 tests)
//! 4. Choice/alternatives (7 tests)
//! 5. Sequence/concatenation (8 tests)
//! 6. Repeat/optional (8 tests)
//! 7. Error cases (8 tests)

use adze_tool::pure_rust_builder::{BuildOptions, build_parser_from_json};
use serde_json::json;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn opts() -> BuildOptions {
    BuildOptions {
        out_dir: "/tmp/adze-converter-v3".to_string(),
        emit_artifacts: false,
        compress_tables: false,
    }
}

fn build_json(
    value: &serde_json::Value,
) -> anyhow::Result<adze_tool::pure_rust_builder::BuildResult> {
    build_parser_from_json(serde_json::to_string(value).unwrap(), opts())
}

fn minimal_grammar(name: &str, rule_type: &str, rule_value: &str) -> serde_json::Value {
    json!({
        "name": name,
        "rules": {
            "start": { "type": rule_type, "value": rule_value }
        }
    })
}

// ===========================================================================
// 1. Simple grammar conversion (8 tests)
// ===========================================================================

#[test]
fn simple_string_literal_builds() {
    let g = minimal_grammar("simple_str", "STRING", "hello");
    assert!(build_json(&g).is_ok());
}

#[test]
fn simple_string_literal_name() {
    let g = minimal_grammar("named_str", "STRING", "world");
    let r = build_json(&g).unwrap();
    assert_eq!(r.grammar_name, "named_str");
}

#[test]
fn simple_string_literal_produces_code() {
    let g = minimal_grammar("code_str", "STRING", "abc");
    let r = build_json(&g).unwrap();
    assert!(!r.parser_code.is_empty());
}

#[test]
fn simple_string_literal_produces_node_types() {
    let g = minimal_grammar("nt_str", "STRING", "x");
    let r = build_json(&g).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&r.node_types_json).unwrap();
    assert!(parsed.is_array());
}

#[test]
fn simple_pattern_rule_builds() {
    let g = minimal_grammar("simple_pat", "PATTERN", "[a-z]+");
    assert!(build_json(&g).is_ok());
}

#[test]
fn simple_pattern_has_states() {
    let g = minimal_grammar("pat_states", "PATTERN", "[0-9]+");
    let r = build_json(&g).unwrap();
    assert!(r.build_stats.state_count > 0);
}

#[test]
fn simple_pattern_has_symbols() {
    let g = minimal_grammar("pat_syms", "PATTERN", "\\w+");
    let r = build_json(&g).unwrap();
    assert!(r.build_stats.symbol_count > 0);
}

#[test]
fn simple_blank_rule_builds() {
    let g = json!({
        "name": "blank_lang",
        "rules": {
            "start": { "type": "BLANK" }
        }
    });
    assert!(build_json(&g).is_ok());
}

// ===========================================================================
// 2. Multi-rule conversion (8 tests)
// ===========================================================================

#[test]
fn multi_rule_symbol_ref_builds() {
    let g = json!({
        "name": "multi_sym",
        "rules": {
            "start": { "type": "SYMBOL", "name": "item" },
            "item": { "type": "STRING", "value": "x" }
        }
    });
    assert!(build_json(&g).is_ok());
}

#[test]
fn multi_rule_preserves_grammar_name() {
    let g = json!({
        "name": "multi_named",
        "rules": {
            "program": { "type": "SYMBOL", "name": "token" },
            "token": { "type": "PATTERN", "value": "[a-z]+" }
        }
    });
    let r = build_json(&g).unwrap();
    assert_eq!(r.grammar_name, "multi_named");
}

#[test]
fn multi_rule_three_levels() {
    let g = json!({
        "name": "three_lvl",
        "rules": {
            "start": { "type": "SYMBOL", "name": "middle" },
            "middle": { "type": "SYMBOL", "name": "leaf" },
            "leaf": { "type": "STRING", "value": "z" }
        }
    });
    assert!(build_json(&g).is_ok());
}

#[test]
fn multi_rule_two_tokens() {
    let g = json!({
        "name": "two_tok",
        "rules": {
            "start": {
                "type": "CHOICE",
                "members": [
                    { "type": "SYMBOL", "name": "alpha" },
                    { "type": "SYMBOL", "name": "digit" }
                ]
            },
            "alpha": { "type": "PATTERN", "value": "[a-z]+" },
            "digit": { "type": "PATTERN", "value": "[0-9]+" }
        }
    });
    assert!(build_json(&g).is_ok());
}

#[test]
fn multi_rule_code_nonempty() {
    let g = json!({
        "name": "mr_code",
        "rules": {
            "root": { "type": "SYMBOL", "name": "val" },
            "val": { "type": "STRING", "value": "ok" }
        }
    });
    let r = build_json(&g).unwrap();
    assert!(!r.parser_code.is_empty());
}

#[test]
fn multi_rule_stats_have_states() {
    let g = json!({
        "name": "mr_stats",
        "rules": {
            "root": { "type": "SYMBOL", "name": "val" },
            "val": { "type": "STRING", "value": "v" }
        }
    });
    let r = build_json(&g).unwrap();
    assert!(r.build_stats.state_count > 0);
}

#[test]
fn multi_rule_deterministic() {
    let g = json!({
        "name": "mr_det",
        "rules": {
            "root": { "type": "SYMBOL", "name": "child" },
            "child": { "type": "STRING", "value": "d" }
        }
    });
    let r1 = build_json(&g).unwrap();
    let r2 = build_json(&g).unwrap();
    assert_eq!(r1.grammar_name, r2.grammar_name);
    assert_eq!(r1.build_stats.state_count, r2.build_stats.state_count);
}

#[test]
fn multi_rule_with_extras() {
    let g = json!({
        "name": "mr_extras",
        "rules": {
            "root": { "type": "SYMBOL", "name": "word" },
            "word": { "type": "PATTERN", "value": "[a-z]+" }
        },
        "extras": [
            { "type": "PATTERN", "value": "\\s+" }
        ]
    });
    assert!(build_json(&g).is_ok());
}

// ===========================================================================
// 3. Pattern types (8 tests)
// ===========================================================================

#[test]
fn pattern_digits() {
    let g = minimal_grammar("pat_dig", "PATTERN", "[0-9]+");
    assert!(build_json(&g).is_ok());
}

#[test]
fn pattern_word_chars() {
    let g = minimal_grammar("pat_word", "PATTERN", "\\w+");
    assert!(build_json(&g).is_ok());
}

#[test]
fn pattern_alpha_lower() {
    let g = minimal_grammar("pat_alpha", "PATTERN", "[a-z]+");
    assert!(build_json(&g).is_ok());
}

#[test]
fn pattern_identifier() {
    let g = minimal_grammar("pat_ident", "PATTERN", "[a-zA-Z_][a-zA-Z0-9_]*");
    assert!(build_json(&g).is_ok());
}

#[test]
fn pattern_single_char() {
    let g = minimal_grammar("pat_char", "PATTERN", ".");
    assert!(build_json(&g).is_ok());
}

#[test]
fn string_keyword_if() {
    let g = minimal_grammar("kw_if", "STRING", "if");
    assert!(build_json(&g).is_ok());
}

#[test]
fn string_keyword_return() {
    let g = minimal_grammar("kw_ret", "STRING", "return");
    assert!(build_json(&g).is_ok());
}

#[test]
fn token_wrapping_pattern() {
    let g = json!({
        "name": "tok_wrap",
        "rules": {
            "start": {
                "type": "TOKEN",
                "content": { "type": "PATTERN", "value": "[0-9]+" }
            }
        }
    });
    assert!(build_json(&g).is_ok());
}

// ===========================================================================
// 4. Choice/alternatives (7 tests)
// ===========================================================================

#[test]
fn choice_two_strings() {
    let g = json!({
        "name": "ch_two",
        "rules": {
            "start": {
                "type": "CHOICE",
                "members": [
                    { "type": "STRING", "value": "a" },
                    { "type": "STRING", "value": "b" }
                ]
            }
        }
    });
    assert!(build_json(&g).is_ok());
}

#[test]
fn choice_three_alternatives() {
    let g = json!({
        "name": "ch_three",
        "rules": {
            "start": {
                "type": "CHOICE",
                "members": [
                    { "type": "STRING", "value": "x" },
                    { "type": "STRING", "value": "y" },
                    { "type": "STRING", "value": "z" }
                ]
            }
        }
    });
    assert!(build_json(&g).is_ok());
}

#[test]
fn choice_mixed_string_pattern() {
    let g = json!({
        "name": "ch_mix",
        "rules": {
            "start": {
                "type": "CHOICE",
                "members": [
                    { "type": "STRING", "value": "kw" },
                    { "type": "PATTERN", "value": "[a-z]+" }
                ]
            }
        }
    });
    assert!(build_json(&g).is_ok());
}

#[test]
fn choice_with_symbols() {
    let g = json!({
        "name": "ch_sym",
        "rules": {
            "start": {
                "type": "CHOICE",
                "members": [
                    { "type": "SYMBOL", "name": "alpha" },
                    { "type": "SYMBOL", "name": "num" }
                ]
            },
            "alpha": { "type": "PATTERN", "value": "[a-z]+" },
            "num": { "type": "PATTERN", "value": "[0-9]+" }
        }
    });
    assert!(build_json(&g).is_ok());
}

#[test]
fn choice_with_blank() {
    let g = json!({
        "name": "ch_blank",
        "rules": {
            "start": {
                "type": "CHOICE",
                "members": [
                    { "type": "STRING", "value": "x" },
                    { "type": "BLANK" }
                ]
            }
        }
    });
    assert!(build_json(&g).is_ok());
}

#[test]
fn choice_nested_in_choice() {
    let g = json!({
        "name": "ch_nest",
        "rules": {
            "start": {
                "type": "CHOICE",
                "members": [
                    {
                        "type": "CHOICE",
                        "members": [
                            { "type": "STRING", "value": "a" },
                            { "type": "STRING", "value": "b" }
                        ]
                    },
                    { "type": "STRING", "value": "c" }
                ]
            }
        }
    });
    assert!(build_json(&g).is_ok());
}

#[test]
fn choice_produces_code() {
    let g = json!({
        "name": "ch_code",
        "rules": {
            "start": {
                "type": "CHOICE",
                "members": [
                    { "type": "STRING", "value": "yes" },
                    { "type": "STRING", "value": "no" }
                ]
            }
        }
    });
    let r = build_json(&g).unwrap();
    assert!(!r.parser_code.is_empty());
}

// ===========================================================================
// 5. Sequence/concatenation (8 tests)
// ===========================================================================

#[test]
fn seq_two_strings() {
    let g = json!({
        "name": "seq_two",
        "rules": {
            "start": {
                "type": "SEQ",
                "members": [
                    { "type": "STRING", "value": "a" },
                    { "type": "STRING", "value": "b" }
                ]
            }
        }
    });
    assert!(build_json(&g).is_ok());
}

#[test]
fn seq_three_elements() {
    let g = json!({
        "name": "seq_three",
        "rules": {
            "start": {
                "type": "SEQ",
                "members": [
                    { "type": "STRING", "value": "x" },
                    { "type": "STRING", "value": "y" },
                    { "type": "STRING", "value": "z" }
                ]
            }
        }
    });
    assert!(build_json(&g).is_ok());
}

#[test]
fn seq_string_and_pattern() {
    let g = json!({
        "name": "seq_sp",
        "rules": {
            "start": {
                "type": "SEQ",
                "members": [
                    { "type": "STRING", "value": "let" },
                    { "type": "PATTERN", "value": "[a-z]+" }
                ]
            }
        }
    });
    assert!(build_json(&g).is_ok());
}

#[test]
fn seq_with_symbol_refs() {
    let g = json!({
        "name": "seq_ref",
        "rules": {
            "start": {
                "type": "SEQ",
                "members": [
                    { "type": "SYMBOL", "name": "lhs" },
                    { "type": "STRING", "value": "=" },
                    { "type": "SYMBOL", "name": "rhs" }
                ]
            },
            "lhs": { "type": "PATTERN", "value": "[a-z]+" },
            "rhs": { "type": "PATTERN", "value": "[0-9]+" }
        }
    });
    assert!(build_json(&g).is_ok());
}

#[test]
fn seq_nested_in_seq() {
    let g = json!({
        "name": "seq_nest",
        "rules": {
            "start": {
                "type": "SEQ",
                "members": [
                    {
                        "type": "SEQ",
                        "members": [
                            { "type": "STRING", "value": "a" },
                            { "type": "STRING", "value": "b" }
                        ]
                    },
                    { "type": "STRING", "value": "c" }
                ]
            }
        }
    });
    assert!(build_json(&g).is_ok());
}

#[test]
fn seq_produces_code() {
    let g = json!({
        "name": "seq_code",
        "rules": {
            "start": {
                "type": "SEQ",
                "members": [
                    { "type": "STRING", "value": "(" },
                    { "type": "STRING", "value": ")" }
                ]
            }
        }
    });
    let r = build_json(&g).unwrap();
    assert!(!r.parser_code.is_empty());
}

#[test]
fn seq_has_positive_state_count() {
    let g = json!({
        "name": "seq_st",
        "rules": {
            "start": {
                "type": "SEQ",
                "members": [
                    { "type": "STRING", "value": "{" },
                    { "type": "STRING", "value": "}" }
                ]
            }
        }
    });
    let r = build_json(&g).unwrap();
    assert!(r.build_stats.state_count > 0);
}

#[test]
fn seq_inside_choice() {
    let g = json!({
        "name": "seq_ch",
        "rules": {
            "start": {
                "type": "CHOICE",
                "members": [
                    {
                        "type": "SEQ",
                        "members": [
                            { "type": "STRING", "value": "a" },
                            { "type": "STRING", "value": "b" }
                        ]
                    },
                    { "type": "STRING", "value": "c" }
                ]
            }
        }
    });
    assert!(build_json(&g).is_ok());
}

// ===========================================================================
// 6. Repeat/optional (8 tests)
// ===========================================================================

#[test]
fn repeat_zero_or_more() {
    let g = json!({
        "name": "rep_zero",
        "rules": {
            "start": {
                "type": "REPEAT",
                "content": { "type": "STRING", "value": "a" }
            }
        }
    });
    assert!(build_json(&g).is_ok());
}

#[test]
fn repeat1_one_or_more() {
    let g = json!({
        "name": "rep_one",
        "rules": {
            "start": {
                "type": "REPEAT1",
                "content": { "type": "STRING", "value": "b" }
            }
        }
    });
    assert!(build_json(&g).is_ok());
}

#[test]
fn optional_rule() {
    let g = json!({
        "name": "opt_rule",
        "rules": {
            "start": {
                "type": "SEQ",
                "members": [
                    { "type": "STRING", "value": "x" },
                    {
                        "type": "OPTIONAL",
                        "value": { "type": "STRING", "value": "y" }
                    }
                ]
            }
        }
    });
    assert!(build_json(&g).is_ok());
}

#[test]
fn repeat_with_pattern() {
    let g = json!({
        "name": "rep_pat",
        "rules": {
            "start": {
                "type": "REPEAT",
                "content": { "type": "PATTERN", "value": "[a-z]+" }
            }
        }
    });
    assert!(build_json(&g).is_ok());
}

#[test]
fn repeat1_with_symbol_ref() {
    let g = json!({
        "name": "rep1_sym",
        "rules": {
            "start": {
                "type": "REPEAT1",
                "content": { "type": "SYMBOL", "name": "item" }
            },
            "item": { "type": "STRING", "value": "i" }
        }
    });
    assert!(build_json(&g).is_ok());
}

#[test]
fn optional_standalone() {
    let g = json!({
        "name": "opt_sa",
        "rules": {
            "start": {
                "type": "OPTIONAL",
                "value": { "type": "STRING", "value": "maybe" }
            }
        }
    });
    assert!(build_json(&g).is_ok());
}

#[test]
fn repeat_nested_in_seq() {
    let g = json!({
        "name": "rep_seq",
        "rules": {
            "start": {
                "type": "SEQ",
                "members": [
                    { "type": "STRING", "value": "[" },
                    {
                        "type": "REPEAT",
                        "content": { "type": "PATTERN", "value": "[0-9]+" }
                    },
                    { "type": "STRING", "value": "]" }
                ]
            }
        }
    });
    assert!(build_json(&g).is_ok());
}

#[test]
fn repeat_produces_valid_stats() {
    let g = json!({
        "name": "rep_stats",
        "rules": {
            "start": {
                "type": "REPEAT1",
                "content": { "type": "STRING", "value": "r" }
            }
        }
    });
    let r = build_json(&g).unwrap();
    assert!(r.build_stats.state_count > 0);
    assert!(r.build_stats.symbol_count > 0);
}

// ===========================================================================
// 7. Error cases (8 tests)
// ===========================================================================

#[test]
fn error_invalid_json() {
    let r = build_parser_from_json("{bad json".to_string(), opts());
    assert!(r.is_err());
}

#[test]
fn error_empty_string() {
    let r = build_parser_from_json(String::new(), opts());
    assert!(r.is_err());
}

#[test]
fn error_json_array_not_object() {
    let r = build_parser_from_json("[]".to_string(), opts());
    assert!(r.is_err());
}

#[test]
fn error_json_number() {
    let r = build_parser_from_json("42".to_string(), opts());
    assert!(r.is_err());
}

#[test]
fn error_missing_name() {
    let g = json!({
        "rules": {
            "start": { "type": "STRING", "value": "x" }
        }
    });
    let r = build_json(&g);
    assert!(r.is_err());
}

#[test]
fn error_missing_rules() {
    let g = json!({ "name": "norules" });
    let r = build_json(&g);
    assert!(r.is_err());
}

#[test]
fn error_empty_object() {
    let r = build_parser_from_json("{}".to_string(), opts());
    assert!(r.is_err());
}

#[test]
fn error_descriptive_message() {
    let r = build_parser_from_json("not valid json".to_string(), opts());
    let err = r.unwrap_err();
    let msg = format!("{err}");
    assert!(!msg.is_empty());
}
