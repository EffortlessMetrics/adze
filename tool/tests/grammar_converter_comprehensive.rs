//! Comprehensive tests for the grammar converter functionality.
//!
//! Covers `GrammarConverter`, `GrammarJsConverter`, `from_tree_sitter_json`,
//! `GrammarJs::validate`, and edge cases in grammar construction and conversion.

use adze_tool::grammar_js::json_converter::from_tree_sitter_json;
use adze_tool::grammar_js::{GrammarJs, Rule};
use adze_tool::{GrammarConverter, GrammarJsConverter};
use serde_json::json;

// ---------------------------------------------------------------------------
// GrammarConverter (IR-level sample grammar)
// ---------------------------------------------------------------------------

#[test]
fn sample_grammar_has_correct_name() {
    let grammar = GrammarConverter::create_sample_grammar();
    assert_eq!(grammar.name, "sample");
}

#[test]
fn sample_grammar_tokens_contain_expected_names() {
    let grammar = GrammarConverter::create_sample_grammar();
    let names: Vec<&str> = grammar.tokens.values().map(|t| t.name.as_str()).collect();
    assert!(names.contains(&"identifier"), "missing identifier token");
    assert!(names.contains(&"number"), "missing number token");
    assert!(names.contains(&"plus"), "missing plus token");
}

#[test]
fn sample_grammar_has_three_productions_for_expr() {
    let grammar = GrammarConverter::create_sample_grammar();
    // There is exactly one non-terminal (expr) with three alternative rules.
    let total_rules: usize = grammar.rules.values().map(|v| v.len()).sum();
    assert_eq!(total_rules, 3, "expected 3 productions");
}

#[test]
fn sample_grammar_fields_mapped() {
    let grammar = GrammarConverter::create_sample_grammar();
    let field_names: Vec<&str> = grammar.fields.values().map(|s| s.as_str()).collect();
    assert!(field_names.contains(&"left"));
    assert!(field_names.contains(&"right"));
}

#[test]
fn sample_grammar_normalize_preserves_rules() {
    let mut grammar = GrammarConverter::create_sample_grammar();
    let before = grammar.rules.values().map(|v| v.len()).sum::<usize>();
    let _ = grammar.normalize();
    let after = grammar.rules.values().map(|v| v.len()).sum::<usize>();
    assert!(after >= before, "normalize should not lose rules");
}

// ---------------------------------------------------------------------------
// from_tree_sitter_json — basic happy-path
// ---------------------------------------------------------------------------

fn minimal_json_grammar() -> serde_json::Value {
    json!({
        "name": "minimal",
        "rules": {
            "source": {
                "type": "STRING",
                "value": "hello"
            }
        }
    })
}

#[test]
fn json_minimal_grammar_name() {
    let g = from_tree_sitter_json(&minimal_json_grammar()).unwrap();
    assert_eq!(g.name, "minimal");
}

#[test]
fn json_minimal_grammar_has_one_rule() {
    let g = from_tree_sitter_json(&minimal_json_grammar()).unwrap();
    assert_eq!(g.rules.len(), 1);
    assert!(g.rules.contains_key("source"));
}

#[test]
fn json_word_token_parsed() {
    let v = json!({
        "name": "lang",
        "word": "identifier",
        "rules": {
            "source": { "type": "BLANK" },
            "identifier": { "type": "PATTERN", "value": "[a-z]+" }
        }
    });
    let g = from_tree_sitter_json(&v).unwrap();
    assert_eq!(g.word.as_deref(), Some("identifier"));
}

#[test]
fn json_inline_rules_parsed() {
    let v = json!({
        "name": "lang",
        "inline": ["_expr"],
        "rules": {
            "source": { "type": "BLANK" },
            "_expr": { "type": "BLANK" }
        }
    });
    let g = from_tree_sitter_json(&v).unwrap();
    assert_eq!(g.inline, vec!["_expr"]);
}

#[test]
fn json_conflicts_parsed() {
    let v = json!({
        "name": "lang",
        "conflicts": [["a", "b"]],
        "rules": {
            "source": { "type": "BLANK" },
            "a": { "type": "BLANK" },
            "b": { "type": "BLANK" }
        }
    });
    let g = from_tree_sitter_json(&v).unwrap();
    assert_eq!(g.conflicts.len(), 1);
    assert_eq!(g.conflicts[0], vec!["a", "b"]);
}

#[test]
fn json_extras_parsed() {
    let v = json!({
        "name": "lang",
        "extras": [
            { "type": "PATTERN", "value": "\\s" }
        ],
        "rules": {
            "source": { "type": "BLANK" }
        }
    });
    let g = from_tree_sitter_json(&v).unwrap();
    assert_eq!(g.extras.len(), 1);
}

#[test]
fn json_supertypes_parsed() {
    let v = json!({
        "name": "lang",
        "supertypes": ["expression", "statement"],
        "rules": {
            "source": { "type": "BLANK" }
        }
    });
    let g = from_tree_sitter_json(&v).unwrap();
    assert_eq!(g.supertypes, vec!["expression", "statement"]);
}

// ---------------------------------------------------------------------------
// from_tree_sitter_json — rule types
// ---------------------------------------------------------------------------

#[test]
fn json_seq_rule_parsed() {
    let v = json!({
        "name": "lang",
        "rules": {
            "pair": {
                "type": "SEQ",
                "members": [
                    { "type": "STRING", "value": "(" },
                    { "type": "STRING", "value": ")" }
                ]
            }
        }
    });
    let g = from_tree_sitter_json(&v).unwrap();
    assert!(matches!(g.rules["pair"], Rule::Seq { .. }));
}

#[test]
fn json_choice_rule_parsed() {
    let v = json!({
        "name": "lang",
        "rules": {
            "item": {
                "type": "CHOICE",
                "members": [
                    { "type": "STRING", "value": "a" },
                    { "type": "STRING", "value": "b" }
                ]
            }
        }
    });
    let g = from_tree_sitter_json(&v).unwrap();
    assert!(matches!(g.rules["item"], Rule::Choice { .. }));
}

#[test]
fn json_repeat_and_repeat1_parsed() {
    let v = json!({
        "name": "lang",
        "rules": {
            "many": {
                "type": "REPEAT",
                "content": { "type": "STRING", "value": "x" }
            },
            "many1": {
                "type": "REPEAT1",
                "content": { "type": "STRING", "value": "y" }
            }
        }
    });
    let g = from_tree_sitter_json(&v).unwrap();
    assert!(matches!(g.rules["many"], Rule::Repeat { .. }));
    assert!(matches!(g.rules["many1"], Rule::Repeat1 { .. }));
}

#[test]
fn json_precedence_variants_parsed() {
    let v = json!({
        "name": "lang",
        "rules": {
            "p": {
                "type": "PREC",
                "value": 1,
                "content": { "type": "BLANK" }
            },
            "pl": {
                "type": "PREC_LEFT",
                "value": 2,
                "content": { "type": "BLANK" }
            },
            "pr": {
                "type": "PREC_RIGHT",
                "value": 3,
                "content": { "type": "BLANK" }
            },
            "pd": {
                "type": "PREC_DYNAMIC",
                "value": -1,
                "content": { "type": "BLANK" }
            }
        }
    });
    let g = from_tree_sitter_json(&v).unwrap();
    assert!(matches!(g.rules["p"], Rule::Prec { value: 1, .. }));
    assert!(matches!(g.rules["pl"], Rule::PrecLeft { value: 2, .. }));
    assert!(matches!(g.rules["pr"], Rule::PrecRight { value: 3, .. }));
    assert!(matches!(g.rules["pd"], Rule::PrecDynamic { value: -1, .. }));
}

#[test]
fn json_field_and_alias_parsed() {
    let v = json!({
        "name": "lang",
        "rules": {
            "f": {
                "type": "FIELD",
                "name": "lhs",
                "content": { "type": "BLANK" }
            },
            "a": {
                "type": "ALIAS",
                "value": "alias_name",
                "named": true,
                "content": { "type": "BLANK" }
            }
        }
    });
    let g = from_tree_sitter_json(&v).unwrap();
    match &g.rules["f"] {
        Rule::Field { name, .. } => assert_eq!(name, "lhs"),
        other => panic!("expected Field, got {other:?}"),
    }
    match &g.rules["a"] {
        Rule::Alias { value, named, .. } => {
            assert_eq!(value, "alias_name");
            assert!(named);
        }
        other => panic!("expected Alias, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// from_tree_sitter_json — error cases
// ---------------------------------------------------------------------------

#[test]
fn json_missing_name_is_error() {
    let v = json!({ "rules": {} });
    assert!(from_tree_sitter_json(&v).is_err());
}

#[test]
fn json_not_an_object_is_error() {
    let v = json!("just a string");
    assert!(from_tree_sitter_json(&v).is_err());
}

// ---------------------------------------------------------------------------
// GrammarJs::validate
// ---------------------------------------------------------------------------

#[test]
fn validate_empty_grammar_passes() {
    let g = GrammarJs::new("empty".into());
    assert!(g.validate().is_ok());
}

#[test]
fn validate_bad_word_token_fails() {
    let mut g = GrammarJs::new("bad".into());
    g.word = Some("missing".into());
    assert!(g.validate().is_err());
}

#[test]
fn validate_bad_inline_reference_fails() {
    let mut g = GrammarJs::new("bad".into());
    g.inline.push("ghost".into());
    assert!(g.validate().is_err());
}

#[test]
fn validate_bad_conflict_reference_fails() {
    let mut g = GrammarJs::new("bad".into());
    g.conflicts.push(vec!["nope".into()]);
    assert!(g.validate().is_err());
}

#[test]
fn validate_undefined_symbol_in_rule_fails() {
    let mut g = GrammarJs::new("bad".into());
    g.rules.insert(
        "start".into(),
        Rule::Symbol {
            name: "undefined_sym".into(),
        },
    );
    assert!(g.validate().is_err());
}

// ---------------------------------------------------------------------------
// GrammarConverter: IR-level sample grammar — deeper checks
// ---------------------------------------------------------------------------

#[test]
fn sample_grammar_identifier_token_is_regex() {
    let grammar = GrammarConverter::create_sample_grammar();
    let id_token = grammar
        .tokens
        .values()
        .find(|t| t.name == "identifier")
        .unwrap();
    assert!(
        matches!(&id_token.pattern, adze_ir::TokenPattern::Regex(_)),
        "identifier should be regex"
    );
}

#[test]
fn sample_grammar_plus_token_is_string() {
    let grammar = GrammarConverter::create_sample_grammar();
    let plus_token = grammar.tokens.values().find(|t| t.name == "plus").unwrap();
    assert!(
        matches!(&plus_token.pattern, adze_ir::TokenPattern::String(s) if s == "+"),
        "plus should be string literal"
    );
}

#[test]
fn sample_grammar_tokens_not_fragile() {
    let grammar = GrammarConverter::create_sample_grammar();
    for token in grammar.tokens.values() {
        assert!(!token.fragile, "sample tokens should not be fragile");
    }
}

#[test]
fn sample_grammar_has_exactly_three_tokens() {
    let grammar = GrammarConverter::create_sample_grammar();
    assert_eq!(grammar.tokens.len(), 3);
}

#[test]
fn sample_grammar_expr_has_left_associative_rule() {
    let grammar = GrammarConverter::create_sample_grammar();
    let has_left = grammar
        .all_rules()
        .any(|r| r.associativity == Some(adze_ir::Associativity::Left));
    assert!(has_left, "expected a left-associative rule");
}

#[test]
fn sample_grammar_expr_plus_rule_has_precedence() {
    let grammar = GrammarConverter::create_sample_grammar();
    let has_prec = grammar.all_rules().any(|r| r.precedence.is_some());
    assert!(has_prec, "expected a rule with precedence");
}

#[test]
fn sample_grammar_production_ids_are_unique() {
    let grammar = GrammarConverter::create_sample_grammar();
    let ids: Vec<_> = grammar.all_rules().map(|r| r.production_id).collect();
    let mut dedup = ids.clone();
    dedup.sort();
    dedup.dedup();
    assert_eq!(ids.len(), dedup.len(), "production IDs must be unique");
}

#[test]
fn sample_grammar_no_externals() {
    let grammar = GrammarConverter::create_sample_grammar();
    assert!(grammar.externals.is_empty());
}

#[test]
fn sample_grammar_no_extras() {
    let grammar = GrammarConverter::create_sample_grammar();
    assert!(grammar.extras.is_empty());
}

#[test]
fn sample_grammar_no_precedences_declared() {
    let grammar = GrammarConverter::create_sample_grammar();
    assert!(grammar.precedences.is_empty());
}

#[test]
fn sample_grammar_no_conflicts_declared() {
    let grammar = GrammarConverter::create_sample_grammar();
    assert!(grammar.conflicts.is_empty());
}

#[test]
fn sample_grammar_rules_all_have_matching_lhs() {
    let grammar = GrammarConverter::create_sample_grammar();
    for (lhs, rules) in &grammar.rules {
        for rule in rules {
            assert_eq!(&rule.lhs, lhs, "rule.lhs must match its key in the map");
        }
    }
}

#[test]
fn sample_grammar_expr_binary_rule_has_three_symbols() {
    let grammar = GrammarConverter::create_sample_grammar();
    let bin_rule = grammar.all_rules().find(|r| r.rhs.len() == 3).unwrap();
    assert_eq!(bin_rule.rhs.len(), 3);
}

#[test]
fn sample_grammar_binary_rule_fields_reference_valid_field_ids() {
    let grammar = GrammarConverter::create_sample_grammar();
    let bin_rule = grammar.all_rules().find(|r| !r.fields.is_empty()).unwrap();
    for (field_id, _pos) in &bin_rule.fields {
        assert!(
            grammar.fields.contains_key(field_id),
            "field id {:?} not in grammar.fields",
            field_id
        );
    }
}

#[test]
fn sample_grammar_clone_equals_original() {
    let g1 = GrammarConverter::create_sample_grammar();
    let g2 = g1.clone();
    assert_eq!(g1, g2);
}

// ---------------------------------------------------------------------------
// from_tree_sitter_json — additional rule types and edge cases
// ---------------------------------------------------------------------------

#[test]
fn json_optional_rule_parsed() {
    let v = json!({
        "name": "lang",
        "rules": {
            "maybe": {
                "type": "CHOICE",
                "members": [
                    { "type": "STRING", "value": "x" },
                    { "type": "BLANK" }
                ]
            }
        }
    });
    let g = from_tree_sitter_json(&v).unwrap();
    assert!(g.rules.contains_key("maybe"));
}

#[test]
fn json_blank_rule_parsed() {
    let v = json!({
        "name": "lang",
        "rules": {
            "empty": { "type": "BLANK" }
        }
    });
    let g = from_tree_sitter_json(&v).unwrap();
    assert!(matches!(g.rules["empty"], Rule::Blank));
}

#[test]
fn json_pattern_rule_parsed() {
    let v = json!({
        "name": "lang",
        "rules": {
            "number": { "type": "PATTERN", "value": "[0-9]+" }
        }
    });
    let g = from_tree_sitter_json(&v).unwrap();
    match &g.rules["number"] {
        Rule::Pattern { value } => assert_eq!(value, "[0-9]+"),
        other => panic!("expected Pattern, got {other:?}"),
    }
}

#[test]
fn json_string_rule_parsed() {
    let v = json!({
        "name": "lang",
        "rules": {
            "kw": { "type": "STRING", "value": "return" }
        }
    });
    let g = from_tree_sitter_json(&v).unwrap();
    match &g.rules["kw"] {
        Rule::String { value } => assert_eq!(value, "return"),
        other => panic!("expected String, got {other:?}"),
    }
}

#[test]
fn json_symbol_rule_parsed() {
    let v = json!({
        "name": "lang",
        "rules": {
            "start": { "type": "SYMBOL", "name": "other" },
            "other": { "type": "BLANK" }
        }
    });
    let g = from_tree_sitter_json(&v).unwrap();
    match &g.rules["start"] {
        Rule::Symbol { name } => assert_eq!(name, "other"),
        other => panic!("expected Symbol, got {other:?}"),
    }
}

#[test]
fn json_token_rule_parsed() {
    let v = json!({
        "name": "lang",
        "rules": {
            "tok": {
                "type": "TOKEN",
                "content": { "type": "STRING", "value": "abc" }
            }
        }
    });
    let g = from_tree_sitter_json(&v).unwrap();
    assert!(matches!(g.rules["tok"], Rule::Token { .. }));
}

#[test]
fn json_immediate_token_rule_parsed() {
    let v = json!({
        "name": "lang",
        "rules": {
            "imm": {
                "type": "IMMEDIATE_TOKEN",
                "content": { "type": "STRING", "value": "!" }
            }
        }
    });
    let g = from_tree_sitter_json(&v).unwrap();
    assert!(matches!(g.rules["imm"], Rule::ImmediateToken { .. }));
}

#[test]
fn json_nested_seq_parsed() {
    let v = json!({
        "name": "lang",
        "rules": {
            "nested": {
                "type": "SEQ",
                "members": [
                    { "type": "STRING", "value": "a" },
                    {
                        "type": "SEQ",
                        "members": [
                            { "type": "STRING", "value": "b" },
                            { "type": "STRING", "value": "c" }
                        ]
                    }
                ]
            }
        }
    });
    let g = from_tree_sitter_json(&v).unwrap();
    assert!(matches!(g.rules["nested"], Rule::Seq { .. }));
}

#[test]
fn json_empty_extras_defaults_to_empty() {
    let v = json!({
        "name": "lang",
        "extras": [],
        "rules": { "s": { "type": "BLANK" } }
    });
    let g = from_tree_sitter_json(&v).unwrap();
    assert!(g.extras.is_empty());
}

#[test]
fn json_multiple_extras_parsed() {
    let v = json!({
        "name": "lang",
        "extras": [
            { "type": "PATTERN", "value": "\\s" },
            { "type": "PATTERN", "value": "//[^\\n]*" }
        ],
        "rules": { "s": { "type": "BLANK" } }
    });
    let g = from_tree_sitter_json(&v).unwrap();
    assert_eq!(g.extras.len(), 2);
}

#[test]
fn json_externals_parsed() {
    let v = json!({
        "name": "lang",
        "externals": [
            { "type": "SYMBOL", "name": "indent" },
            { "type": "SYMBOL", "name": "dedent" }
        ],
        "rules": { "s": { "type": "BLANK" } }
    });
    let g = from_tree_sitter_json(&v).unwrap();
    assert_eq!(g.externals.len(), 2);
}

#[test]
fn json_missing_rules_produces_empty_grammar() {
    let v = json!({ "name": "bad" });
    let g = from_tree_sitter_json(&v).unwrap();
    assert!(g.rules.is_empty());
}

#[test]
fn json_null_value_is_error() {
    let v = serde_json::Value::Null;
    assert!(from_tree_sitter_json(&v).is_err());
}

#[test]
fn json_array_value_is_error() {
    let v = json!([1, 2, 3]);
    assert!(from_tree_sitter_json(&v).is_err());
}

// ---------------------------------------------------------------------------
// GrammarJs::validate — additional cases
// ---------------------------------------------------------------------------

#[test]
fn validate_grammar_with_valid_symbol_passes() {
    let mut g = GrammarJs::new("ok".into());
    g.rules.insert(
        "start".into(),
        Rule::Symbol {
            name: "other".into(),
        },
    );
    g.rules.insert("other".into(), Rule::Blank);
    assert!(g.validate().is_ok());
}

#[test]
fn validate_grammar_with_nested_invalid_symbol_fails() {
    let mut g = GrammarJs::new("bad".into());
    g.rules.insert(
        "start".into(),
        Rule::Seq {
            members: vec![Rule::Symbol {
                name: "ghost".into(),
            }],
        },
    );
    assert!(g.validate().is_err());
}

#[test]
fn validate_grammar_with_valid_word_token_passes() {
    let mut g = GrammarJs::new("ok".into());
    g.rules.insert(
        "identifier".into(),
        Rule::Pattern {
            value: "[a-z]+".into(),
        },
    );
    g.word = Some("identifier".into());
    assert!(g.validate().is_ok());
}

#[test]
fn validate_grammar_with_valid_inline_passes() {
    let mut g = GrammarJs::new("ok".into());
    g.rules.insert("_expr".into(), Rule::Blank);
    g.inline.push("_expr".into());
    assert!(g.validate().is_ok());
}

#[test]
fn validate_grammar_with_valid_conflicts_passes() {
    let mut g = GrammarJs::new("ok".into());
    g.rules.insert("a".into(), Rule::Blank);
    g.rules.insert("b".into(), Rule::Blank);
    g.conflicts.push(vec!["a".into(), "b".into()]);
    assert!(g.validate().is_ok());
}

#[test]
fn validate_symbol_in_optional_is_checked() {
    let mut g = GrammarJs::new("bad".into());
    g.rules.insert(
        "start".into(),
        Rule::Optional {
            value: Box::new(Rule::Symbol {
                name: "missing".into(),
            }),
        },
    );
    assert!(g.validate().is_err());
}

#[test]
fn validate_symbol_in_repeat_is_checked() {
    let mut g = GrammarJs::new("bad".into());
    g.rules.insert(
        "start".into(),
        Rule::Repeat {
            content: Box::new(Rule::Symbol {
                name: "missing".into(),
            }),
        },
    );
    assert!(g.validate().is_err());
}

#[test]
fn validate_symbol_in_repeat1_is_checked() {
    let mut g = GrammarJs::new("bad".into());
    g.rules.insert(
        "start".into(),
        Rule::Repeat1 {
            content: Box::new(Rule::Symbol {
                name: "missing".into(),
            }),
        },
    );
    assert!(g.validate().is_err());
}

#[test]
fn validate_symbol_in_prec_is_checked() {
    let mut g = GrammarJs::new("bad".into());
    g.rules.insert(
        "start".into(),
        Rule::Prec {
            value: 1,
            content: Box::new(Rule::Symbol {
                name: "missing".into(),
            }),
        },
    );
    assert!(g.validate().is_err());
}

#[test]
fn validate_symbol_in_field_is_checked() {
    let mut g = GrammarJs::new("bad".into());
    g.rules.insert(
        "start".into(),
        Rule::Field {
            name: "f".into(),
            content: Box::new(Rule::Symbol {
                name: "missing".into(),
            }),
        },
    );
    assert!(g.validate().is_err());
}

#[test]
fn validate_symbol_in_alias_is_checked() {
    let mut g = GrammarJs::new("bad".into());
    g.rules.insert(
        "start".into(),
        Rule::Alias {
            value: "a".into(),
            named: true,
            content: Box::new(Rule::Symbol {
                name: "missing".into(),
            }),
        },
    );
    assert!(g.validate().is_err());
}

#[test]
fn validate_external_symbol_passes() {
    use adze_tool::grammar_js::ExternalToken;
    let mut g = GrammarJs::new("ok".into());
    g.externals.push(ExternalToken {
        name: "indent".into(),
        symbol: "indent".into(),
    });
    g.rules.insert(
        "start".into(),
        Rule::Symbol {
            name: "indent".into(),
        },
    );
    assert!(g.validate().is_ok());
}

// ---------------------------------------------------------------------------
// GrammarJsConverter round-trip
// ---------------------------------------------------------------------------

fn expression_grammar_js() -> GrammarJs {
    let v = json!({
        "name": "expr",
        "rules": {
            "source_file": {
                "type": "SYMBOL",
                "name": "expression"
            },
            "expression": {
                "type": "CHOICE",
                "members": [
                    { "type": "SYMBOL", "name": "number" },
                    { "type": "SYMBOL", "name": "identifier" }
                ]
            },
            "number": {
                "type": "PATTERN",
                "value": "\\d+"
            },
            "identifier": {
                "type": "PATTERN",
                "value": "[a-zA-Z_]\\w*"
            }
        }
    });
    from_tree_sitter_json(&v).unwrap()
}

#[test]
fn converter_produces_grammar_with_correct_name() {
    let ir = GrammarJsConverter::new(expression_grammar_js())
        .convert()
        .unwrap();
    assert_eq!(ir.name, "expr");
}

#[test]
fn converter_produces_nonempty_rules_and_tokens() {
    let ir = GrammarJsConverter::new(expression_grammar_js())
        .convert()
        .unwrap();
    assert!(!ir.rules.is_empty(), "expected rules");
    assert!(!ir.tokens.is_empty(), "expected tokens");
}

#[test]
fn converter_complex_grammar_with_precedence() {
    let v = json!({
        "name": "calc",
        "rules": {
            "expression": {
                "type": "CHOICE",
                "members": [
                    { "type": "SYMBOL", "name": "number" },
                    {
                        "type": "PREC_LEFT",
                        "value": 1,
                        "content": {
                            "type": "SEQ",
                            "members": [
                                { "type": "SYMBOL", "name": "expression" },
                                { "type": "STRING", "value": "+" },
                                { "type": "SYMBOL", "name": "expression" }
                            ]
                        }
                    }
                ]
            },
            "number": {
                "type": "PATTERN",
                "value": "[0-9]+"
            }
        }
    });
    let g = from_tree_sitter_json(&v).unwrap();
    let ir = GrammarJsConverter::new(g).convert().unwrap();
    assert_eq!(ir.name, "calc");
    assert!(!ir.rules.is_empty());
}

#[test]
fn converter_preserves_pattern_tokens() {
    let ir = GrammarJsConverter::new(expression_grammar_js())
        .convert()
        .unwrap();
    let has_regex = ir
        .tokens
        .values()
        .any(|t| matches!(&t.pattern, adze_ir::TokenPattern::Regex(_)));
    assert!(has_regex, "expected at least one regex token");
}

#[test]
fn converter_grammar_with_repeat() {
    let v = json!({
        "name": "rep",
        "rules": {
            "source_file": {
                "type": "REPEAT",
                "content": { "type": "SYMBOL", "name": "item" }
            },
            "item": { "type": "STRING", "value": "x" }
        }
    });
    let g = from_tree_sitter_json(&v).unwrap();
    let ir = GrammarJsConverter::new(g).convert().unwrap();
    assert_eq!(ir.name, "rep");
}

#[test]
fn converter_grammar_with_optional() {
    let v = json!({
        "name": "opt",
        "rules": {
            "source_file": {
                "type": "SEQ",
                "members": [
                    { "type": "STRING", "value": "a" },
                    {
                        "type": "CHOICE",
                        "members": [
                            { "type": "STRING", "value": "b" },
                            { "type": "BLANK" }
                        ]
                    }
                ]
            }
        }
    });
    let g = from_tree_sitter_json(&v).unwrap();
    let ir = GrammarJsConverter::new(g).convert().unwrap();
    assert_eq!(ir.name, "opt");
}

#[test]
fn converter_grammar_with_field() {
    let v = json!({
        "name": "fld",
        "rules": {
            "pair": {
                "type": "SEQ",
                "members": [
                    {
                        "type": "FIELD",
                        "name": "key",
                        "content": { "type": "SYMBOL", "name": "identifier" }
                    },
                    { "type": "STRING", "value": ":" },
                    {
                        "type": "FIELD",
                        "name": "value",
                        "content": { "type": "SYMBOL", "name": "identifier" }
                    }
                ]
            },
            "identifier": { "type": "PATTERN", "value": "[a-z]+" }
        }
    });
    let g = from_tree_sitter_json(&v).unwrap();
    let ir = GrammarJsConverter::new(g).convert().unwrap();
    assert_eq!(ir.name, "fld");
}

// ---------------------------------------------------------------------------
// ToolError coverage
// ---------------------------------------------------------------------------

#[test]
fn tool_error_multiple_word_rules_display() {
    let e = adze_tool::ToolError::MultipleWordRules;
    let msg = e.to_string();
    assert!(msg.contains("multiple word rules"));
}

#[test]
fn tool_error_multiple_precedence_display() {
    let e = adze_tool::ToolError::MultiplePrecedenceAttributes;
    let msg = e.to_string();
    assert!(msg.contains("prec"));
}

#[test]
fn tool_error_string_too_long() {
    let e = adze_tool::ToolError::string_too_long("test_op", 9999);
    let msg = e.to_string();
    assert!(msg.contains("9999"));
    assert!(msg.contains("test_op"));
}

#[test]
fn tool_error_complex_symbols_not_normalized() {
    let e = adze_tool::ToolError::complex_symbols_not_normalized("FIRST set");
    let msg = e.to_string();
    assert!(msg.contains("FIRST set"));
}

#[test]
fn tool_error_expected_symbol_type() {
    let e = adze_tool::ToolError::expected_symbol_type("terminal");
    let msg = e.to_string();
    assert!(msg.contains("terminal"));
}

#[test]
fn tool_error_expected_action_type() {
    let e = adze_tool::ToolError::expected_action_type("shift");
    let msg = e.to_string();
    assert!(msg.contains("shift"));
}

#[test]
fn tool_error_expected_error_type() {
    let e = adze_tool::ToolError::expected_error_type("parse");
    let msg = e.to_string();
    assert!(msg.contains("parse"));
}

#[test]
fn tool_error_grammar_validation() {
    let e = adze_tool::ToolError::grammar_validation("missing start symbol");
    let msg = e.to_string();
    assert!(msg.contains("missing start symbol"));
}

#[test]
fn tool_error_from_string() {
    let e: adze_tool::ToolError = "custom error".into();
    assert!(e.to_string().contains("custom error"));
}

#[test]
fn tool_error_from_owned_string() {
    let e: adze_tool::ToolError = String::from("owned error").into();
    assert!(e.to_string().contains("owned error"));
}

#[test]
fn tool_error_other_variant() {
    let e = adze_tool::ToolError::Other("other msg".into());
    assert!(e.to_string().contains("other msg"));
}

#[test]
fn tool_error_nested_option() {
    let e = adze_tool::ToolError::NestedOptionType;
    assert!(e.to_string().contains("Option<Option"));
}

#[test]
fn tool_error_struct_no_fields() {
    let e = adze_tool::ToolError::StructHasNoFields { name: "Foo".into() };
    assert!(e.to_string().contains("Foo"));
}

#[test]
fn tool_error_expected_string_literal() {
    let e = adze_tool::ToolError::ExpectedStringLiteral {
        context: "pattern".into(),
        actual: "42".into(),
    };
    let msg = e.to_string();
    assert!(msg.contains("pattern"));
    assert!(msg.contains("42"));
}

#[test]
fn tool_error_expected_integer_literal() {
    let e = adze_tool::ToolError::ExpectedIntegerLiteral {
        actual: "abc".into(),
    };
    assert!(e.to_string().contains("abc"));
}

#[test]
fn tool_error_expected_path_type() {
    let e = adze_tool::ToolError::ExpectedPathType {
        actual: "Fn()".into(),
    };
    assert!(e.to_string().contains("Fn()"));
}

#[test]
fn tool_error_expected_single_segment_path() {
    let e = adze_tool::ToolError::ExpectedSingleSegmentPath {
        actual: "a::b::c".into(),
    };
    assert!(e.to_string().contains("a::b::c"));
}

#[test]
fn tool_error_invalid_production() {
    let e = adze_tool::ToolError::InvalidProduction {
        details: "empty rhs".into(),
    };
    assert!(e.to_string().contains("empty rhs"));
}

// ---------------------------------------------------------------------------
// GrammarVisualizer via GrammarConverter sample grammar
// ---------------------------------------------------------------------------

#[test]
fn visualizer_text_contains_grammar_name() {
    let grammar = GrammarConverter::create_sample_grammar();
    let vis = adze_tool::GrammarVisualizer::new(grammar);
    let text = vis.to_text();
    assert!(text.contains("Grammar: sample"));
}

#[test]
fn visualizer_text_lists_all_tokens() {
    let grammar = GrammarConverter::create_sample_grammar();
    let vis = adze_tool::GrammarVisualizer::new(grammar);
    let text = vis.to_text();
    assert!(text.contains("identifier"));
    assert!(text.contains("number"));
    assert!(text.contains("plus"));
}

#[test]
fn visualizer_dot_output_is_valid_structure() {
    let grammar = GrammarConverter::create_sample_grammar();
    let vis = adze_tool::GrammarVisualizer::new(grammar);
    let dot = vis.to_dot();
    assert!(dot.starts_with("digraph Grammar {"));
    assert!(dot.contains("}"));
    assert!(dot.contains("rankdir=LR"));
}

#[test]
fn visualizer_dot_references_terminals() {
    let grammar = GrammarConverter::create_sample_grammar();
    let vis = adze_tool::GrammarVisualizer::new(grammar);
    let dot = vis.to_dot();
    assert!(dot.contains("shape=ellipse"));
    assert!(dot.contains("lightblue"));
}

#[test]
fn visualizer_dot_references_nonterminals() {
    let grammar = GrammarConverter::create_sample_grammar();
    let vis = adze_tool::GrammarVisualizer::new(grammar);
    let dot = vis.to_dot();
    assert!(dot.contains("lightgreen"));
}

#[test]
fn visualizer_railroad_svg_has_svg_tags() {
    let grammar = GrammarConverter::create_sample_grammar();
    let vis = adze_tool::GrammarVisualizer::new(grammar);
    let svg = vis.to_railroad_svg();
    assert!(svg.contains("<svg"));
    assert!(svg.contains("</svg>"));
}

#[test]
fn visualizer_dependency_graph_output() {
    let grammar = GrammarConverter::create_sample_grammar();
    let vis = adze_tool::GrammarVisualizer::new(grammar);
    let deps = vis.dependency_graph();
    assert!(deps.contains("Symbol Dependencies"));
}

#[test]
fn visualizer_empty_grammar_does_not_panic() {
    let grammar = adze_ir::Grammar::new("empty".to_string());
    let vis = adze_tool::GrammarVisualizer::new(grammar);
    let _text = vis.to_text();
    let _dot = vis.to_dot();
    let _svg = vis.to_railroad_svg();
    let _deps = vis.dependency_graph();
}

// ---------------------------------------------------------------------------
// BuildOptions defaults
// ---------------------------------------------------------------------------

#[test]
fn build_options_default_compress_tables() {
    let opts = adze_tool::BuildOptions::default();
    assert!(opts.compress_tables);
}

#[test]
fn build_options_default_emit_artifacts_false() {
    // Unless ADZE_EMIT_ARTIFACTS env is set, should be false
    let opts = adze_tool::BuildOptions::default();
    assert!(!opts.emit_artifacts);
}
