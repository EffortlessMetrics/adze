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
