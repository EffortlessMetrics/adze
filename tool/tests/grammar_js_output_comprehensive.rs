//! Comprehensive tests for grammar_js output generation.
//!
//! Covers JSON output format validation, grammar rules serialization,
//! token serialization, precedence handling, special character escaping,
//! edge cases, deterministic output, and field mapping in output.

use adze_tool::grammar_js::json_converter::from_tree_sitter_json;
use adze_tool::grammar_js::{ExternalToken, GrammarJs, Rule, from_json};
use serde_json::{Value, json};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Build a minimal grammar with a single string rule.
fn minimal_grammar() -> GrammarJs {
    let mut g = GrammarJs::new("test_lang".to_string());
    g.rules.insert(
        "start".to_string(),
        Rule::String {
            value: "hello".to_string(),
        },
    );
    g
}

/// Build a grammar with multiple rules referencing each other.
fn multi_rule_grammar() -> GrammarJs {
    let mut g = GrammarJs::new("multi".to_string());
    g.rules.insert(
        "source".to_string(),
        Rule::Symbol {
            name: "expression".to_string(),
        },
    );
    g.rules.insert(
        "expression".to_string(),
        Rule::Choice {
            members: vec![
                Rule::Symbol {
                    name: "number".to_string(),
                },
                Rule::Symbol {
                    name: "identifier".to_string(),
                },
            ],
        },
    );
    g.rules.insert(
        "number".to_string(),
        Rule::Pattern {
            value: r"\d+".to_string(),
        },
    );
    g.rules.insert(
        "identifier".to_string(),
        Rule::Pattern {
            value: r"[a-zA-Z_]\w*".to_string(),
        },
    );
    g
}

// ===========================================================================
// 1. JSON Output Format Validation
// ===========================================================================

#[test]
fn json_output_grammar_has_name_field() {
    let g = minimal_grammar();
    let json_val = serde_json::to_value(&g).unwrap();
    assert_eq!(json_val["name"].as_str().unwrap(), "test_lang");
}

#[test]
fn json_output_grammar_has_rules_object() {
    let g = minimal_grammar();
    let json_val = serde_json::to_value(&g).unwrap();
    assert!(json_val["rules"].is_object());
}

#[test]
fn json_output_top_level_keys_present() {
    let g = minimal_grammar();
    let json_val = serde_json::to_value(&g).unwrap();
    let obj = json_val.as_object().unwrap();
    assert!(obj.contains_key("name"));
    assert!(obj.contains_key("rules"));
    assert!(obj.contains_key("extras"));
    assert!(obj.contains_key("conflicts"));
    assert!(obj.contains_key("externals"));
    assert!(obj.contains_key("inline"));
    assert!(obj.contains_key("supertypes"));
}

#[test]
fn json_output_is_valid_json_string() {
    let g = multi_rule_grammar();
    let s = serde_json::to_string(&g).unwrap();
    let reparsed: Value = serde_json::from_str(&s).unwrap();
    assert_eq!(reparsed["name"].as_str().unwrap(), "multi");
}

#[test]
fn json_output_pretty_is_valid() {
    let g = minimal_grammar();
    let pretty = serde_json::to_string_pretty(&g).unwrap();
    let reparsed: Value = serde_json::from_str(&pretty).unwrap();
    assert_eq!(reparsed["name"].as_str().unwrap(), "test_lang");
}

// ===========================================================================
// 2. Grammar Rules Serialization
// ===========================================================================

#[test]
fn serialize_string_rule() {
    let rule = Rule::String {
        value: "foo".to_string(),
    };
    let v = serde_json::to_value(&rule).unwrap();
    assert_eq!(v["type"].as_str().unwrap(), "STRING");
    assert_eq!(v["value"].as_str().unwrap(), "foo");
}

#[test]
fn serialize_pattern_rule() {
    let rule = Rule::Pattern {
        value: r"\d+".to_string(),
    };
    let v = serde_json::to_value(&rule).unwrap();
    assert_eq!(v["type"].as_str().unwrap(), "PATTERN");
    assert_eq!(v["value"].as_str().unwrap(), r"\d+");
}

#[test]
fn serialize_symbol_rule() {
    let rule = Rule::Symbol {
        name: "expr".to_string(),
    };
    let v = serde_json::to_value(&rule).unwrap();
    assert_eq!(v["type"].as_str().unwrap(), "SYMBOL");
    assert_eq!(v["name"].as_str().unwrap(), "expr");
}

#[test]
fn serialize_blank_rule() {
    let rule = Rule::Blank;
    let v = serde_json::to_value(&rule).unwrap();
    assert_eq!(v["type"].as_str().unwrap(), "BLANK");
}

#[test]
fn serialize_seq_rule() {
    let rule = Rule::Seq {
        members: vec![
            Rule::String {
                value: "a".to_string(),
            },
            Rule::String {
                value: "b".to_string(),
            },
        ],
    };
    let v = serde_json::to_value(&rule).unwrap();
    assert_eq!(v["type"].as_str().unwrap(), "SEQ");
    assert_eq!(v["members"].as_array().unwrap().len(), 2);
}

#[test]
fn serialize_choice_rule() {
    let rule = Rule::Choice {
        members: vec![
            Rule::String {
                value: "x".to_string(),
            },
            Rule::String {
                value: "y".to_string(),
            },
            Rule::String {
                value: "z".to_string(),
            },
        ],
    };
    let v = serde_json::to_value(&rule).unwrap();
    assert_eq!(v["type"].as_str().unwrap(), "CHOICE");
    assert_eq!(v["members"].as_array().unwrap().len(), 3);
}

#[test]
fn serialize_optional_rule() {
    let rule = Rule::Optional {
        value: Box::new(Rule::Symbol {
            name: "item".to_string(),
        }),
    };
    let v = serde_json::to_value(&rule).unwrap();
    assert_eq!(v["type"].as_str().unwrap(), "OPTIONAL");
    assert_eq!(v["value"]["name"].as_str().unwrap(), "item");
}

#[test]
fn serialize_repeat_rule() {
    let rule = Rule::Repeat {
        content: Box::new(Rule::Symbol {
            name: "stmt".to_string(),
        }),
    };
    let v = serde_json::to_value(&rule).unwrap();
    assert_eq!(v["type"].as_str().unwrap(), "REPEAT");
    assert_eq!(v["content"]["name"].as_str().unwrap(), "stmt");
}

#[test]
fn serialize_repeat1_rule() {
    let rule = Rule::Repeat1 {
        content: Box::new(Rule::Symbol {
            name: "arg".to_string(),
        }),
    };
    let v = serde_json::to_value(&rule).unwrap();
    assert_eq!(v["type"].as_str().unwrap(), "REPEAT1");
    assert_eq!(v["content"]["name"].as_str().unwrap(), "arg");
}

// ===========================================================================
// 3. Token Serialization
// ===========================================================================

#[test]
fn serialize_token_rule() {
    let rule = Rule::Token {
        content: Box::new(Rule::Pattern {
            value: r"[0-9]+".to_string(),
        }),
    };
    let v = serde_json::to_value(&rule).unwrap();
    assert_eq!(v["type"].as_str().unwrap(), "TOKEN");
    assert_eq!(v["content"]["type"].as_str().unwrap(), "PATTERN");
}

#[test]
fn serialize_immediate_token_rule() {
    let rule = Rule::ImmediateToken {
        content: Box::new(Rule::String {
            value: ".".to_string(),
        }),
    };
    let v = serde_json::to_value(&rule).unwrap();
    assert_eq!(v["type"].as_str().unwrap(), "IMMEDIATE_TOKEN");
    assert_eq!(v["content"]["value"].as_str().unwrap(), ".");
}

#[test]
fn serialize_token_wrapping_choice() {
    let rule = Rule::Token {
        content: Box::new(Rule::Choice {
            members: vec![
                Rule::String {
                    value: "+".to_string(),
                },
                Rule::String {
                    value: "-".to_string(),
                },
            ],
        }),
    };
    let v = serde_json::to_value(&rule).unwrap();
    assert_eq!(v["type"].as_str().unwrap(), "TOKEN");
    assert_eq!(v["content"]["type"].as_str().unwrap(), "CHOICE");
    assert_eq!(v["content"]["members"].as_array().unwrap().len(), 2);
}

// ===========================================================================
// 4. Precedence Handling in Output
// ===========================================================================

#[test]
fn serialize_prec_rule() {
    let rule = Rule::Prec {
        value: 5,
        content: Box::new(Rule::Symbol {
            name: "expr".to_string(),
        }),
    };
    let v = serde_json::to_value(&rule).unwrap();
    assert_eq!(v["type"].as_str().unwrap(), "PREC");
    assert_eq!(v["value"].as_i64().unwrap(), 5);
    assert_eq!(v["content"]["name"].as_str().unwrap(), "expr");
}

#[test]
fn serialize_prec_left_rule() {
    let rule = Rule::PrecLeft {
        value: 2,
        content: Box::new(Rule::Seq {
            members: vec![
                Rule::Symbol {
                    name: "expr".to_string(),
                },
                Rule::String {
                    value: "+".to_string(),
                },
                Rule::Symbol {
                    name: "expr".to_string(),
                },
            ],
        }),
    };
    let v = serde_json::to_value(&rule).unwrap();
    assert_eq!(v["type"].as_str().unwrap(), "PREC_LEFT");
    assert_eq!(v["value"].as_i64().unwrap(), 2);
    assert_eq!(v["content"]["type"].as_str().unwrap(), "SEQ");
}

#[test]
fn serialize_prec_right_rule() {
    let rule = Rule::PrecRight {
        value: 3,
        content: Box::new(Rule::Symbol {
            name: "assign".to_string(),
        }),
    };
    let v = serde_json::to_value(&rule).unwrap();
    assert_eq!(v["type"].as_str().unwrap(), "PREC_RIGHT");
    assert_eq!(v["value"].as_i64().unwrap(), 3);
}

#[test]
fn serialize_prec_dynamic_rule() {
    let rule = Rule::PrecDynamic {
        value: -1,
        content: Box::new(Rule::Symbol {
            name: "fallback".to_string(),
        }),
    };
    let v = serde_json::to_value(&rule).unwrap();
    assert_eq!(v["type"].as_str().unwrap(), "PREC_DYNAMIC");
    assert_eq!(v["value"].as_i64().unwrap(), -1);
}

#[test]
fn serialize_prec_zero_value() {
    let rule = Rule::Prec {
        value: 0,
        content: Box::new(Rule::Blank),
    };
    let v = serde_json::to_value(&rule).unwrap();
    assert_eq!(v["value"].as_i64().unwrap(), 0);
}

#[test]
fn serialize_prec_negative_value() {
    let rule = Rule::Prec {
        value: -10,
        content: Box::new(Rule::Blank),
    };
    let v = serde_json::to_value(&rule).unwrap();
    assert_eq!(v["value"].as_i64().unwrap(), -10);
}

#[test]
fn serialize_nested_prec_rules() {
    let rule = Rule::PrecLeft {
        value: 1,
        content: Box::new(Rule::PrecRight {
            value: 2,
            content: Box::new(Rule::Symbol {
                name: "inner".to_string(),
            }),
        }),
    };
    let v = serde_json::to_value(&rule).unwrap();
    assert_eq!(v["type"].as_str().unwrap(), "PREC_LEFT");
    assert_eq!(v["content"]["type"].as_str().unwrap(), "PREC_RIGHT");
    assert_eq!(v["content"]["value"].as_i64().unwrap(), 2);
}

// ===========================================================================
// 5. Special Character Escaping
// ===========================================================================

#[test]
fn serialize_string_with_quotes() {
    let rule = Rule::String {
        value: r#"say "hi""#.to_string(),
    };
    let s = serde_json::to_string(&rule).unwrap();
    // The JSON string must contain escaped quotes
    assert!(s.contains(r#"say \"hi\""#));
}

#[test]
fn serialize_string_with_backslash() {
    let rule = Rule::String {
        value: r"path\to\file".to_string(),
    };
    let v = serde_json::to_value(&rule).unwrap();
    assert_eq!(v["value"].as_str().unwrap(), r"path\to\file");
}

#[test]
fn serialize_pattern_with_regex_chars() {
    let rule = Rule::Pattern {
        value: r"[a-zA-Z_]\w*".to_string(),
    };
    let v = serde_json::to_value(&rule).unwrap();
    assert_eq!(v["value"].as_str().unwrap(), r"[a-zA-Z_]\w*");
}

#[test]
fn serialize_string_with_newline() {
    let rule = Rule::String {
        value: "line1\nline2".to_string(),
    };
    let s = serde_json::to_string(&rule).unwrap();
    assert!(s.contains(r"\n"));
}

#[test]
fn serialize_string_with_tab() {
    let rule = Rule::String {
        value: "col1\tcol2".to_string(),
    };
    let v = serde_json::to_value(&rule).unwrap();
    assert_eq!(v["value"].as_str().unwrap(), "col1\tcol2");
}

#[test]
fn serialize_string_with_unicode() {
    let rule = Rule::String {
        value: "λ→∀".to_string(),
    };
    let v = serde_json::to_value(&rule).unwrap();
    assert_eq!(v["value"].as_str().unwrap(), "λ→∀");
}

#[test]
fn serialize_empty_string_value() {
    let rule = Rule::String {
        value: "".to_string(),
    };
    let v = serde_json::to_value(&rule).unwrap();
    assert_eq!(v["value"].as_str().unwrap(), "");
}

// ===========================================================================
// 6. Edge Cases
// ===========================================================================

#[test]
fn edge_case_empty_grammar_no_rules() {
    let g = GrammarJs::new("empty".to_string());
    let v = serde_json::to_value(&g).unwrap();
    assert_eq!(v["name"].as_str().unwrap(), "empty");
    assert!(v["rules"].as_object().unwrap().is_empty());
}

#[test]
fn edge_case_single_blank_rule() {
    let mut g = GrammarJs::new("blank_only".to_string());
    g.rules.insert("start".to_string(), Rule::Blank);
    let v = serde_json::to_value(&g).unwrap();
    assert_eq!(v["rules"]["start"]["type"].as_str().unwrap(), "BLANK");
}

#[test]
fn edge_case_many_rules() {
    let mut g = GrammarJs::new("many_rules".to_string());
    for i in 0..50 {
        g.rules.insert(
            format!("rule_{}", i),
            Rule::String {
                value: format!("val_{}", i),
            },
        );
    }
    let v = serde_json::to_value(&g).unwrap();
    assert_eq!(v["rules"].as_object().unwrap().len(), 50);
}

#[test]
fn edge_case_deeply_nested_seq() {
    // seq(seq(seq("a", "b"), "c"), "d")
    let rule = Rule::Seq {
        members: vec![
            Rule::Seq {
                members: vec![
                    Rule::Seq {
                        members: vec![
                            Rule::String {
                                value: "a".to_string(),
                            },
                            Rule::String {
                                value: "b".to_string(),
                            },
                        ],
                    },
                    Rule::String {
                        value: "c".to_string(),
                    },
                ],
            },
            Rule::String {
                value: "d".to_string(),
            },
        ],
    };
    let v = serde_json::to_value(&rule).unwrap();
    assert_eq!(v["type"].as_str().unwrap(), "SEQ");
    assert_eq!(v["members"][0]["type"].as_str().unwrap(), "SEQ");
    assert_eq!(
        v["members"][0]["members"][0]["type"].as_str().unwrap(),
        "SEQ"
    );
}

#[test]
fn edge_case_choice_with_single_member() {
    let rule = Rule::Choice {
        members: vec![Rule::String {
            value: "only".to_string(),
        }],
    };
    let v = serde_json::to_value(&rule).unwrap();
    assert_eq!(v["members"].as_array().unwrap().len(), 1);
}

#[test]
fn edge_case_empty_seq() {
    let rule = Rule::Seq { members: vec![] };
    let v = serde_json::to_value(&rule).unwrap();
    assert!(v["members"].as_array().unwrap().is_empty());
}

#[test]
fn edge_case_repeat_of_optional() {
    let rule = Rule::Repeat {
        content: Box::new(Rule::Optional {
            value: Box::new(Rule::String {
                value: "maybe".to_string(),
            }),
        }),
    };
    let v = serde_json::to_value(&rule).unwrap();
    assert_eq!(v["content"]["type"].as_str().unwrap(), "OPTIONAL");
    assert_eq!(v["content"]["value"]["value"].as_str().unwrap(), "maybe");
}

#[test]
fn edge_case_grammar_with_all_optional_fields() {
    let mut g = GrammarJs::new("full".to_string());
    g.word = Some("ident".to_string());
    g.inline = vec!["_inline".to_string()];
    g.conflicts = vec![vec!["rule_a".to_string(), "rule_b".to_string()]];
    g.extras = vec![Rule::Pattern {
        value: r"\s".to_string(),
    }];
    g.externals = vec![ExternalToken {
        name: "ext_tok".to_string(),
        symbol: "external_0".to_string(),
    }];
    g.supertypes = vec!["_expression".to_string()];
    g.rules.insert(
        "ident".to_string(),
        Rule::Pattern {
            value: "[a-z]+".to_string(),
        },
    );
    g.rules.insert(
        "_inline".to_string(),
        Rule::Symbol {
            name: "ident".to_string(),
        },
    );
    g.rules.insert(
        "rule_a".to_string(),
        Rule::Symbol {
            name: "ident".to_string(),
        },
    );
    g.rules.insert(
        "rule_b".to_string(),
        Rule::Symbol {
            name: "ident".to_string(),
        },
    );

    let v = serde_json::to_value(&g).unwrap();
    assert_eq!(v["word"].as_str().unwrap(), "ident");
    assert_eq!(v["inline"].as_array().unwrap().len(), 1);
    assert_eq!(v["conflicts"].as_array().unwrap().len(), 1);
    assert_eq!(v["extras"].as_array().unwrap().len(), 1);
    assert_eq!(v["externals"].as_array().unwrap().len(), 1);
    assert_eq!(v["supertypes"].as_array().unwrap().len(), 1);
}

// ===========================================================================
// 7. Deterministic Output
// ===========================================================================

#[test]
fn deterministic_serialization_same_grammar() {
    let g1 = multi_rule_grammar();
    let g2 = multi_rule_grammar();
    let s1 = serde_json::to_string(&g1).unwrap();
    let s2 = serde_json::to_string(&g2).unwrap();
    assert_eq!(s1, s2);
}

#[test]
fn deterministic_rule_order_preserved() {
    let g = multi_rule_grammar();
    let v = serde_json::to_value(&g).unwrap();
    let keys: Vec<&str> = v["rules"]
        .as_object()
        .unwrap()
        .keys()
        .map(|k| k.as_str())
        .collect();
    assert_eq!(keys, vec!["source", "expression", "number", "identifier"]);
}

#[test]
fn deterministic_multiple_serialization_cycles() {
    let g = minimal_grammar();
    let first = serde_json::to_string(&g).unwrap();
    for _ in 0..10 {
        let output = serde_json::to_string(&g).unwrap();
        assert_eq!(first, output);
    }
}

// ===========================================================================
// 8. Field Mapping in Output
// ===========================================================================

#[test]
fn serialize_field_rule() {
    let rule = Rule::Field {
        name: "operator".to_string(),
        content: Box::new(Rule::String {
            value: "+".to_string(),
        }),
    };
    let v = serde_json::to_value(&rule).unwrap();
    assert_eq!(v["type"].as_str().unwrap(), "FIELD");
    assert_eq!(v["name"].as_str().unwrap(), "operator");
    assert_eq!(v["content"]["value"].as_str().unwrap(), "+");
}

#[test]
fn serialize_field_wrapping_symbol() {
    let rule = Rule::Field {
        name: "left".to_string(),
        content: Box::new(Rule::Symbol {
            name: "expression".to_string(),
        }),
    };
    let v = serde_json::to_value(&rule).unwrap();
    assert_eq!(v["name"].as_str().unwrap(), "left");
    assert_eq!(v["content"]["type"].as_str().unwrap(), "SYMBOL");
}

#[test]
fn serialize_multiple_fields_in_seq() {
    let rule = Rule::Seq {
        members: vec![
            Rule::Field {
                name: "left".to_string(),
                content: Box::new(Rule::Symbol {
                    name: "expr".to_string(),
                }),
            },
            Rule::Field {
                name: "op".to_string(),
                content: Box::new(Rule::String {
                    value: "+".to_string(),
                }),
            },
            Rule::Field {
                name: "right".to_string(),
                content: Box::new(Rule::Symbol {
                    name: "expr".to_string(),
                }),
            },
        ],
    };
    let v = serde_json::to_value(&rule).unwrap();
    let members = v["members"].as_array().unwrap();
    assert_eq!(members.len(), 3);
    assert_eq!(members[0]["name"].as_str().unwrap(), "left");
    assert_eq!(members[1]["name"].as_str().unwrap(), "op");
    assert_eq!(members[2]["name"].as_str().unwrap(), "right");
}

// ===========================================================================
// 9. Alias Serialization
// ===========================================================================

#[test]
fn serialize_alias_named() {
    let rule = Rule::Alias {
        content: Box::new(Rule::Symbol {
            name: "ident".to_string(),
        }),
        value: "name".to_string(),
        named: true,
    };
    let v = serde_json::to_value(&rule).unwrap();
    assert_eq!(v["type"].as_str().unwrap(), "ALIAS");
    assert_eq!(v["value"].as_str().unwrap(), "name");
    assert!(v["named"].as_bool().unwrap());
}

#[test]
fn serialize_alias_unnamed() {
    let rule = Rule::Alias {
        content: Box::new(Rule::String {
            value: "=>".to_string(),
        }),
        value: "arrow".to_string(),
        named: false,
    };
    let v = serde_json::to_value(&rule).unwrap();
    assert!(!v["named"].as_bool().unwrap());
    assert_eq!(v["value"].as_str().unwrap(), "arrow");
}

// ===========================================================================
// 10. Roundtrip (serialize -> deserialize) via from_json
// ===========================================================================

#[test]
fn roundtrip_minimal_grammar() {
    let g = minimal_grammar();
    let v = serde_json::to_value(&g).unwrap();
    let g2 = from_json(&v).unwrap();
    assert_eq!(g2.name, g.name);
    assert_eq!(g2.rules.len(), g.rules.len());
}

#[test]
fn roundtrip_multi_rule_grammar() {
    let g = multi_rule_grammar();
    let v = serde_json::to_value(&g).unwrap();
    let g2 = from_json(&v).unwrap();
    assert_eq!(g2.name, "multi");
    assert_eq!(g2.rules.len(), 4);
}

#[test]
fn roundtrip_grammar_with_extras() {
    let mut g = GrammarJs::new("with_extras".to_string());
    g.rules.insert(
        "start".to_string(),
        Rule::String {
            value: "x".to_string(),
        },
    );
    g.extras = vec![Rule::Pattern {
        value: r"\s".to_string(),
    }];
    let v = serde_json::to_value(&g).unwrap();
    let g2 = from_json(&v).unwrap();
    assert_eq!(g2.extras.len(), 1);
}

#[test]
fn roundtrip_grammar_with_word() {
    let mut g = GrammarJs::new("word_test".to_string());
    g.word = Some("ident".to_string());
    g.rules.insert(
        "ident".to_string(),
        Rule::Pattern {
            value: "[a-z]+".to_string(),
        },
    );
    let v = serde_json::to_value(&g).unwrap();
    let g2 = from_json(&v).unwrap();
    assert_eq!(g2.word, Some("ident".to_string()));
}

#[test]
fn roundtrip_preserves_rule_types() {
    let json_input = json!({
        "name": "types_test",
        "rules": {
            "start": {
                "type": "SEQ",
                "members": [
                    { "type": "STRING", "value": "fn" },
                    { "type": "PATTERN", "value": "[a-z]+" },
                    { "type": "BLANK" }
                ]
            }
        }
    });
    let g = from_json(&json_input).unwrap();
    let v = serde_json::to_value(&g).unwrap();
    let rules = &v["rules"]["start"];
    assert_eq!(rules["type"].as_str().unwrap(), "SEQ");
    let members = rules["members"].as_array().unwrap();
    assert_eq!(members[0]["type"].as_str().unwrap(), "STRING");
    assert_eq!(members[1]["type"].as_str().unwrap(), "PATTERN");
    assert_eq!(members[2]["type"].as_str().unwrap(), "BLANK");
}

// ===========================================================================
// 11. from_tree_sitter_json specific tests
// ===========================================================================

#[test]
fn from_ts_json_with_conflicts() {
    let json_input = json!({
        "name": "conflict_test",
        "rules": {
            "a": { "type": "STRING", "value": "a" },
            "b": { "type": "STRING", "value": "b" }
        },
        "conflicts": [["a", "b"]]
    });
    let g = from_tree_sitter_json(&json_input).unwrap();
    assert_eq!(g.conflicts.len(), 1);
    assert_eq!(g.conflicts[0], vec!["a", "b"]);
}

#[test]
fn from_ts_json_with_inline() {
    let json_input = json!({
        "name": "inline_test",
        "rules": {
            "_inline_rule": { "type": "BLANK" }
        },
        "inline": ["_inline_rule"]
    });
    let g = from_tree_sitter_json(&json_input).unwrap();
    assert_eq!(g.inline, vec!["_inline_rule"]);
}

#[test]
fn from_ts_json_with_supertypes() {
    let json_input = json!({
        "name": "super_test",
        "rules": {
            "_expression": { "type": "CHOICE", "members": [
                { "type": "SYMBOL", "name": "number" }
            ]},
            "number": { "type": "PATTERN", "value": "\\d+" }
        },
        "supertypes": ["_expression"]
    });
    let g = from_tree_sitter_json(&json_input).unwrap();
    assert_eq!(g.supertypes, vec!["_expression"]);
}

#[test]
fn from_ts_json_missing_name_is_error() {
    let json_input = json!({
        "rules": {
            "start": { "type": "BLANK" }
        }
    });
    assert!(from_tree_sitter_json(&json_input).is_err());
}

// ===========================================================================
// 12. Grammar Validation with Serialization
// ===========================================================================

#[test]
fn validation_passes_for_valid_grammar() {
    let g = multi_rule_grammar();
    assert!(g.validate().is_ok());
}

#[test]
fn validation_fails_for_missing_symbol() {
    let mut g = GrammarJs::new("bad".to_string());
    g.rules.insert(
        "start".to_string(),
        Rule::Symbol {
            name: "missing".to_string(),
        },
    );
    assert!(g.validate().is_err());
}

#[test]
fn validation_passes_with_external_symbol() {
    let mut g = GrammarJs::new("ext".to_string());
    g.externals.push(ExternalToken {
        name: "ext_tok".to_string(),
        symbol: "external_0".to_string(),
    });
    g.rules.insert(
        "start".to_string(),
        Rule::Symbol {
            name: "ext_tok".to_string(),
        },
    );
    assert!(g.validate().is_ok());
}

#[test]
fn validation_checks_word_token_exists() {
    let mut g = GrammarJs::new("word".to_string());
    g.word = Some("identifier".to_string());
    g.rules.insert(
        "start".to_string(),
        Rule::String {
            value: "x".to_string(),
        },
    );
    // "identifier" not in rules -> should fail
    assert!(g.validate().is_err());
}

#[test]
fn validation_checks_inline_rules_exist() {
    let mut g = GrammarJs::new("inline".to_string());
    g.inline = vec!["_missing".to_string()];
    g.rules.insert("start".to_string(), Rule::Blank);
    assert!(g.validate().is_err());
}

#[test]
fn validation_checks_conflict_rules_exist() {
    let mut g = GrammarJs::new("conflict".to_string());
    g.conflicts = vec![vec!["start".to_string(), "missing".to_string()]];
    g.rules.insert("start".to_string(), Rule::Blank);
    assert!(g.validate().is_err());
}
