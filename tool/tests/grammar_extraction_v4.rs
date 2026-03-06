//! Grammar extraction pattern tests for adze-tool (v4).
//!
//! Covers 55+ extraction patterns grouped into seven categories:
//! 1. Simple rule extraction
//! 2. Token pattern extraction
//! 3. Nested rule extraction (CHOICE, SEQ, REPEAT)
//! 4. Precedence extraction (PREC, PREC_LEFT, PREC_RIGHT)
//! 5. Field name extraction
//! 6. Extras extraction (whitespace, comments)
//! 7. Error extraction (malformed grammars)

use adze_tool::pure_rust_builder::{BuildOptions, BuildResult, build_parser_from_json};
use serde_json::{Value, json};
use tempfile::TempDir;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn build_from_json(grammar: Value) -> anyhow::Result<BuildResult> {
    let json_str = serde_json::to_string(&grammar)?;
    let dir = TempDir::new()?;
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().into(),
        emit_artifacts: false,
        compress_tables: false,
    };
    build_parser_from_json(json_str, opts)
}

fn build_ok(grammar: Value) -> BuildResult {
    build_from_json(grammar).expect("build_parser_from_json should succeed")
}

fn assert_valid(result: &BuildResult, expected_name: &str) {
    assert_eq!(result.grammar_name, expected_name);
    assert!(!result.parser_path.is_empty(), "parser_path is empty");
    assert!(!result.parser_code.is_empty(), "parser_code is empty");
    assert!(
        !result.node_types_json.is_empty(),
        "node_types_json is empty"
    );
    assert!(result.build_stats.state_count > 0, "state_count is zero");
    assert!(result.build_stats.symbol_count > 0, "symbol_count is zero");
}

fn assert_node_types_valid(result: &BuildResult) {
    let v: Value =
        serde_json::from_str(&result.node_types_json).expect("node_types_json is invalid JSON");
    assert!(v.is_array(), "node_types should be a JSON array");
}

// =========================================================================
// 1. Extract simple rules (8 tests)
// =========================================================================

#[test]
fn simple_single_string_rule() {
    let g = json!({
        "name": "simple_str",
        "rules": {
            "source_file": { "type": "STRING", "value": "hello" }
        }
    });
    let r = build_ok(g);
    assert_valid(&r, "simple_str");
}

#[test]
fn simple_single_pattern_rule() {
    let g = json!({
        "name": "simple_pat",
        "rules": {
            "source_file": { "type": "PATTERN", "value": "[a-z]+" }
        }
    });
    let r = build_ok(g);
    assert_valid(&r, "simple_pat");
}

#[test]
fn simple_symbol_reference() {
    let g = json!({
        "name": "sym_ref",
        "rules": {
            "program": { "type": "SYMBOL", "name": "item" },
            "item": { "type": "STRING", "value": "x" }
        }
    });
    let r = build_ok(g);
    assert_valid(&r, "sym_ref");
}

#[test]
fn simple_blank_rule() {
    let g = json!({
        "name": "blank_rule",
        "rules": {
            "source_file": {
                "type": "CHOICE",
                "members": [
                    { "type": "STRING", "value": "a" },
                    { "type": "BLANK" }
                ]
            }
        }
    });
    let r = build_ok(g);
    assert_valid(&r, "blank_rule");
}

#[test]
fn simple_grammar_name_matches_output() {
    let g = json!({
        "name": "name_check",
        "rules": {
            "root": { "type": "STRING", "value": "v" }
        }
    });
    let r = build_ok(g);
    assert_eq!(r.grammar_name, "name_check");
}

#[test]
fn simple_two_independent_rules() {
    let g = json!({
        "name": "two_rules",
        "rules": {
            "alpha": { "type": "STRING", "value": "a" },
            "beta": { "type": "STRING", "value": "b" }
        }
    });
    let r = build_ok(g);
    assert_valid(&r, "two_rules");
    assert_node_types_valid(&r);
}

#[test]
fn simple_three_rules_with_refs() {
    let g = json!({
        "name": "three_rules",
        "rules": {
            "program": {
                "type": "SEQ",
                "members": [
                    { "type": "SYMBOL", "name": "header" },
                    { "type": "SYMBOL", "name": "body" }
                ]
            },
            "header": { "type": "STRING", "value": "H" },
            "body": { "type": "STRING", "value": "B" }
        }
    });
    let r = build_ok(g);
    assert_valid(&r, "three_rules");
}

#[test]
fn simple_stats_populated() {
    let g = json!({
        "name": "stats_pop",
        "rules": {
            "root": { "type": "STRING", "value": "z" }
        }
    });
    let r = build_ok(g);
    assert!(r.build_stats.state_count > 0);
    assert!(r.build_stats.symbol_count > 0);
    // conflict_cells is valid for any value
    let _ = r.build_stats.conflict_cells;
}

// =========================================================================
// 2. Extract token patterns (8 tests)
// =========================================================================

#[test]
fn token_regex_digit_pattern() {
    let g = json!({
        "name": "tok_digit",
        "rules": {
            "source_file": {
                "type": "TOKEN",
                "content": { "type": "PATTERN", "value": "\\d+" }
            }
        }
    });
    let r = build_ok(g);
    assert_valid(&r, "tok_digit");
}

#[test]
fn token_string_literal() {
    let g = json!({
        "name": "tok_str",
        "rules": {
            "source_file": {
                "type": "TOKEN",
                "content": { "type": "STRING", "value": "keyword" }
            }
        }
    });
    let r = build_ok(g);
    assert_valid(&r, "tok_str");
}

#[test]
fn token_word_pattern() {
    let g = json!({
        "name": "tok_word",
        "rules": {
            "source_file": {
                "type": "TOKEN",
                "content": { "type": "PATTERN", "value": "\\w+" }
            }
        }
    });
    let r = build_ok(g);
    assert_valid(&r, "tok_word");
}

#[test]
fn token_immediate_string() {
    let g = json!({
        "name": "tok_imm",
        "rules": {
            "source_file": {
                "type": "IMMEDIATE_TOKEN",
                "content": { "type": "STRING", "value": "." }
            }
        }
    });
    let r = build_ok(g);
    assert_valid(&r, "tok_imm");
}

#[test]
fn token_character_class_pattern() {
    let g = json!({
        "name": "tok_charclass",
        "rules": {
            "source_file": {
                "type": "TOKEN",
                "content": { "type": "PATTERN", "value": "[A-Za-z_][A-Za-z0-9_]*" }
            }
        }
    });
    let r = build_ok(g);
    assert_valid(&r, "tok_charclass");
}

#[test]
fn token_seq_inside_token() {
    let g = json!({
        "name": "tok_seq",
        "rules": {
            "source_file": {
                "type": "TOKEN",
                "content": {
                    "type": "SEQ",
                    "members": [
                        { "type": "PATTERN", "value": "[0-9]+" },
                        { "type": "STRING", "value": "." },
                        { "type": "PATTERN", "value": "[0-9]+" }
                    ]
                }
            }
        }
    });
    let r = build_ok(g);
    assert_valid(&r, "tok_seq");
}

#[test]
fn token_choice_inside_token() {
    let g = json!({
        "name": "tok_choice",
        "rules": {
            "source_file": {
                "type": "TOKEN",
                "content": {
                    "type": "CHOICE",
                    "members": [
                        { "type": "STRING", "value": "true" },
                        { "type": "STRING", "value": "false" }
                    ]
                }
            }
        }
    });
    let r = build_ok(g);
    assert_valid(&r, "tok_choice");
}

#[test]
fn token_generates_parser_code() {
    let g = json!({
        "name": "tok_code",
        "rules": {
            "source_file": {
                "type": "TOKEN",
                "content": { "type": "PATTERN", "value": "[0-9]+" }
            }
        }
    });
    let r = build_ok(g);
    assert!(
        r.parser_code.len() > 50,
        "parser code should be non-trivial"
    );
}

// =========================================================================
// 3. Extract nested rules — CHOICE, SEQ, REPEAT (8 tests)
// =========================================================================

#[test]
fn nested_choice_two_alternatives() {
    let g = json!({
        "name": "nest_choice2",
        "rules": {
            "source_file": {
                "type": "CHOICE",
                "members": [
                    { "type": "STRING", "value": "yes" },
                    { "type": "STRING", "value": "no" }
                ]
            }
        }
    });
    let r = build_ok(g);
    assert_valid(&r, "nest_choice2");
}

#[test]
fn nested_choice_three_alternatives() {
    let g = json!({
        "name": "nest_choice3",
        "rules": {
            "source_file": {
                "type": "CHOICE",
                "members": [
                    { "type": "STRING", "value": "a" },
                    { "type": "STRING", "value": "b" },
                    { "type": "STRING", "value": "c" }
                ]
            }
        }
    });
    let r = build_ok(g);
    assert_valid(&r, "nest_choice3");
}

#[test]
fn nested_seq_two_members() {
    let g = json!({
        "name": "nest_seq2",
        "rules": {
            "source_file": {
                "type": "SEQ",
                "members": [
                    { "type": "STRING", "value": "(" },
                    { "type": "STRING", "value": ")" }
                ]
            }
        }
    });
    let r = build_ok(g);
    assert_valid(&r, "nest_seq2");
}

#[test]
fn nested_repeat_star() {
    let g = json!({
        "name": "nest_repeat",
        "rules": {
            "source_file": {
                "type": "REPEAT",
                "content": { "type": "STRING", "value": "item" }
            }
        }
    });
    let r = build_ok(g);
    assert_valid(&r, "nest_repeat");
}

#[test]
fn nested_repeat1_plus() {
    let g = json!({
        "name": "nest_repeat1",
        "rules": {
            "source_file": {
                "type": "REPEAT1",
                "content": { "type": "STRING", "value": "elem" }
            }
        }
    });
    let r = build_ok(g);
    assert_valid(&r, "nest_repeat1");
}

#[test]
fn nested_seq_inside_choice() {
    let g = json!({
        "name": "nest_seq_in_choice",
        "rules": {
            "source_file": {
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
    let r = build_ok(g);
    assert_valid(&r, "nest_seq_in_choice");
}

#[test]
fn nested_repeat_inside_seq() {
    let g = json!({
        "name": "nest_rep_in_seq",
        "rules": {
            "source_file": {
                "type": "SEQ",
                "members": [
                    { "type": "STRING", "value": "start" },
                    {
                        "type": "REPEAT",
                        "content": { "type": "STRING", "value": "mid" }
                    },
                    { "type": "STRING", "value": "end" }
                ]
            }
        }
    });
    let r = build_ok(g);
    assert_valid(&r, "nest_rep_in_seq");
}

#[test]
fn nested_choice_inside_repeat() {
    let g = json!({
        "name": "nest_choice_rep",
        "rules": {
            "source_file": {
                "type": "REPEAT",
                "content": {
                    "type": "CHOICE",
                    "members": [
                        { "type": "STRING", "value": "x" },
                        { "type": "STRING", "value": "y" }
                    ]
                }
            }
        }
    });
    let r = build_ok(g);
    assert_valid(&r, "nest_choice_rep");
}

// =========================================================================
// 4. Extract precedence (7 tests)
// =========================================================================

#[test]
fn prec_basic_value() {
    let g = json!({
        "name": "prec_basic",
        "rules": {
            "source_file": {
                "type": "PREC",
                "value": 1,
                "content": { "type": "STRING", "value": "a" }
            }
        }
    });
    let r = build_ok(g);
    assert_valid(&r, "prec_basic");
}

#[test]
fn prec_left_associative() {
    let g = json!({
        "name": "prec_left",
        "rules": {
            "source_file": {
                "type": "PREC_LEFT",
                "value": 2,
                "content": { "type": "STRING", "value": "op" }
            }
        }
    });
    let r = build_ok(g);
    assert_valid(&r, "prec_left");
}

#[test]
fn prec_right_associative() {
    let g = json!({
        "name": "prec_right",
        "rules": {
            "source_file": {
                "type": "PREC_RIGHT",
                "value": 3,
                "content": { "type": "STRING", "value": "assign" }
            }
        }
    });
    let r = build_ok(g);
    assert_valid(&r, "prec_right");
}

#[test]
fn prec_zero_value() {
    let g = json!({
        "name": "prec_zero",
        "rules": {
            "source_file": {
                "type": "PREC",
                "value": 0,
                "content": { "type": "STRING", "value": "z" }
            }
        }
    });
    let r = build_ok(g);
    assert_valid(&r, "prec_zero");
}

#[test]
fn prec_negative_value() {
    let g = json!({
        "name": "prec_neg",
        "rules": {
            "source_file": {
                "type": "PREC",
                "value": -1,
                "content": { "type": "STRING", "value": "low" }
            }
        }
    });
    let r = build_ok(g);
    assert_valid(&r, "prec_neg");
}

#[test]
fn prec_left_wrapping_seq() {
    let g = json!({
        "name": "prec_left_seq",
        "rules": {
            "source_file": {
                "type": "PREC_LEFT",
                "value": 5,
                "content": {
                    "type": "SEQ",
                    "members": [
                        { "type": "STRING", "value": "a" },
                        { "type": "STRING", "value": "+" },
                        { "type": "STRING", "value": "b" }
                    ]
                }
            }
        }
    });
    let r = build_ok(g);
    assert_valid(&r, "prec_left_seq");
}

#[test]
fn prec_right_wrapping_choice() {
    let g = json!({
        "name": "prec_right_ch",
        "rules": {
            "source_file": {
                "type": "SEQ",
                "members": [
                    { "type": "STRING", "value": "x" },
                    {
                        "type": "PREC_RIGHT",
                        "value": 4,
                        "content": {
                            "type": "CHOICE",
                            "members": [
                                { "type": "STRING", "value": "=" },
                                { "type": "STRING", "value": "+=" }
                            ]
                        }
                    }
                ]
            }
        }
    });
    let r = build_ok(g);
    assert_valid(&r, "prec_right_ch");
}

// =========================================================================
// 5. Extract field names (8 tests)
// =========================================================================

#[test]
fn field_basic_extraction() {
    let g = json!({
        "name": "field_basic",
        "rules": {
            "source_file": {
                "type": "FIELD",
                "name": "body",
                "content": { "type": "STRING", "value": "x" }
            }
        }
    });
    let r = build_ok(g);
    assert_valid(&r, "field_basic");
}

#[test]
fn field_with_symbol_content() {
    let g = json!({
        "name": "field_sym",
        "rules": {
            "source_file": {
                "type": "FIELD",
                "name": "value",
                "content": { "type": "SYMBOL", "name": "item" }
            },
            "item": { "type": "STRING", "value": "v" }
        }
    });
    let r = build_ok(g);
    assert_valid(&r, "field_sym");
}

#[test]
fn field_with_pattern_content() {
    let g = json!({
        "name": "field_pat",
        "rules": {
            "source_file": {
                "type": "FIELD",
                "name": "identifier",
                "content": { "type": "PATTERN", "value": "[a-z]+" }
            }
        }
    });
    let r = build_ok(g);
    assert_valid(&r, "field_pat");
}

#[test]
fn field_inside_seq() {
    let g = json!({
        "name": "field_in_seq",
        "rules": {
            "source_file": {
                "type": "SEQ",
                "members": [
                    {
                        "type": "FIELD",
                        "name": "left",
                        "content": { "type": "STRING", "value": "L" }
                    },
                    { "type": "STRING", "value": "=" },
                    {
                        "type": "FIELD",
                        "name": "right",
                        "content": { "type": "STRING", "value": "R" }
                    }
                ]
            }
        }
    });
    let r = build_ok(g);
    assert_valid(&r, "field_in_seq");
}

#[test]
fn field_inside_choice() {
    let g = json!({
        "name": "field_in_ch",
        "rules": {
            "source_file": {
                "type": "CHOICE",
                "members": [
                    {
                        "type": "FIELD",
                        "name": "first",
                        "content": { "type": "STRING", "value": "A" }
                    },
                    {
                        "type": "FIELD",
                        "name": "second",
                        "content": { "type": "STRING", "value": "B" }
                    }
                ]
            }
        }
    });
    let r = build_ok(g);
    assert_valid(&r, "field_in_ch");
}

#[test]
fn field_nested_in_prec() {
    let g = json!({
        "name": "field_prec",
        "rules": {
            "source_file": {
                "type": "PREC_LEFT",
                "value": 1,
                "content": {
                    "type": "FIELD",
                    "name": "operand",
                    "content": { "type": "STRING", "value": "n" }
                }
            }
        }
    });
    let r = build_ok(g);
    assert_valid(&r, "field_prec");
}

#[test]
fn field_wrapping_repeat() {
    let g = json!({
        "name": "field_rep",
        "rules": {
            "source_file": {
                "type": "SEQ",
                "members": [
                    {
                        "type": "FIELD",
                        "name": "items",
                        "content": {
                            "type": "REPEAT",
                            "content": { "type": "STRING", "value": "i" }
                        }
                    },
                    { "type": "STRING", "value": ";" }
                ]
            }
        }
    });
    let r = build_ok(g);
    assert_valid(&r, "field_rep");
}

#[test]
fn field_node_types_json_valid() {
    let g = json!({
        "name": "field_ntj",
        "rules": {
            "source_file": {
                "type": "SEQ",
                "members": [
                    {
                        "type": "FIELD",
                        "name": "name",
                        "content": { "type": "PATTERN", "value": "[a-z]+" }
                    },
                    { "type": "STRING", "value": ";" }
                ]
            }
        }
    });
    let r = build_ok(g);
    assert_valid(&r, "field_ntj");
    assert_node_types_valid(&r);
}

// =========================================================================
// 6. Extract extras — whitespace, comments (8 tests)
// =========================================================================

#[test]
fn extras_whitespace_pattern() {
    let g = json!({
        "name": "ext_ws",
        "rules": {
            "source_file": { "type": "STRING", "value": "x" }
        },
        "extras": [
            { "type": "PATTERN", "value": "\\s+" }
        ]
    });
    let r = build_ok(g);
    assert_valid(&r, "ext_ws");
}

#[test]
fn extras_single_char_whitespace() {
    let g = json!({
        "name": "ext_ws1",
        "rules": {
            "source_file": { "type": "STRING", "value": "y" }
        },
        "extras": [
            { "type": "PATTERN", "value": "\\s" }
        ]
    });
    let r = build_ok(g);
    assert_valid(&r, "ext_ws1");
}

#[test]
fn extras_line_comment_pattern() {
    let g = json!({
        "name": "ext_lc",
        "rules": {
            "source_file": { "type": "STRING", "value": "z" }
        },
        "extras": [
            { "type": "PATTERN", "value": "//[^\\n]*" }
        ]
    });
    let r = build_ok(g);
    assert_valid(&r, "ext_lc");
}

#[test]
fn extras_whitespace_plus_comment() {
    let g = json!({
        "name": "ext_ws_cmt",
        "rules": {
            "source_file": { "type": "STRING", "value": "a" }
        },
        "extras": [
            { "type": "PATTERN", "value": "\\s+" },
            { "type": "PATTERN", "value": "//[^\\n]*" }
        ]
    });
    let r = build_ok(g);
    assert_valid(&r, "ext_ws_cmt");
}

#[test]
fn extras_empty_array() {
    let g = json!({
        "name": "ext_empty",
        "rules": {
            "source_file": { "type": "STRING", "value": "e" }
        },
        "extras": []
    });
    let r = build_ok(g);
    assert_valid(&r, "ext_empty");
}

#[test]
fn extras_string_literal_extra() {
    let g = json!({
        "name": "ext_str",
        "rules": {
            "source_file": { "type": "STRING", "value": "f" }
        },
        "extras": [
            { "type": "STRING", "value": " " }
        ]
    });
    let r = build_ok(g);
    assert_valid(&r, "ext_str");
}

#[test]
fn extras_do_not_break_stats() {
    let g = json!({
        "name": "ext_stats",
        "rules": {
            "source_file": { "type": "STRING", "value": "s" }
        },
        "extras": [
            { "type": "PATTERN", "value": "\\s+" }
        ]
    });
    let r = build_ok(g);
    assert!(r.build_stats.state_count > 0);
    assert!(r.build_stats.symbol_count > 0);
}

#[test]
fn extras_with_complex_grammar() {
    let g = json!({
        "name": "ext_complex",
        "rules": {
            "source_file": {
                "type": "REPEAT",
                "content": {
                    "type": "CHOICE",
                    "members": [
                        { "type": "SYMBOL", "name": "stmt" },
                        { "type": "SYMBOL", "name": "decl" }
                    ]
                }
            },
            "stmt": { "type": "STRING", "value": "stmt" },
            "decl": { "type": "STRING", "value": "decl" }
        },
        "extras": [
            { "type": "PATTERN", "value": "\\s+" },
            { "type": "PATTERN", "value": "#[^\\n]*" }
        ]
    });
    let r = build_ok(g);
    assert_valid(&r, "ext_complex");
    assert_node_types_valid(&r);
}

// =========================================================================
// 7. Error extraction — malformed grammars (8 tests)
// =========================================================================

#[test]
fn error_invalid_json_string() {
    let dir = TempDir::new().unwrap();
    let opts = BuildOptions {
        out_dir: dir.path().to_string_lossy().into(),
        emit_artifacts: false,
        compress_tables: false,
    };
    let result = build_parser_from_json("{ not valid json }".to_string(), opts);
    assert!(result.is_err(), "invalid JSON should produce an error");
}

#[test]
fn error_missing_name() {
    let g = json!({
        "rules": {
            "source_file": { "type": "STRING", "value": "a" }
        }
    });
    let result = build_from_json(g);
    // Either fails or defaults to "unknown"
    if let Ok(r) = result {
        assert_eq!(r.grammar_name, "unknown");
    }
}

#[test]
fn error_missing_rules() {
    let g = json!({ "name": "no_rules" });
    let result = build_from_json(g);
    assert!(result.is_err(), "grammar with no rules should fail");
}

#[test]
fn error_empty_rules_object() {
    let g = json!({
        "name": "empty_rules",
        "rules": {}
    });
    let result = build_from_json(g);
    assert!(result.is_err(), "empty rules object should fail");
}

#[test]
fn error_rule_is_bare_string() {
    let g = json!({
        "name": "bad_struct",
        "rules": {
            "source_file": "not_a_rule_object"
        }
    });
    let result = build_from_json(g);
    assert!(result.is_err(), "bare string rule should produce an error");
}

#[test]
fn error_rule_is_number() {
    let g = json!({
        "name": "bad_num",
        "rules": {
            "source_file": 42
        }
    });
    let result = build_from_json(g);
    assert!(result.is_err(), "numeric rule should produce an error");
}

#[test]
fn error_unknown_rule_type() {
    let g = json!({
        "name": "bad_type",
        "rules": {
            "source_file": { "type": "NONEXISTENT_TYPE", "value": "x" }
        }
    });
    let result = build_from_json(g);
    assert!(result.is_err(), "unknown rule type should produce an error");
}

#[test]
fn error_rules_is_array_not_object() {
    let g = json!({
        "name": "rules_arr",
        "rules": [
            { "type": "STRING", "value": "a" }
        ]
    });
    let result = build_from_json(g);
    assert!(result.is_err(), "rules as array should produce an error");
}
