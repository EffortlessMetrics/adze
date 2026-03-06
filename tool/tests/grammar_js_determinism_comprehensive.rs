//! Comprehensive tests for grammar_js module: JSON generation, determinism,
//! validity, and conversion of various grammar types.
//!
//! Validates the HashMap→IndexMap fix (commit 449033f9) ensuring that
//! identical grammars always produce identical JSON output.

use adze_ir::builder::GrammarBuilder;
use adze_ir::{Associativity, Grammar};
use adze_tool::grammar_js::{ExternalToken, GrammarJs, GrammarJsConverter, Rule, from_json};
use serde_json::{Value, json};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Build a minimal grammar with a single string rule.
fn minimal_grammar_js() -> GrammarJs {
    let mut g = GrammarJs::new("minimal".to_string());
    g.rules.insert(
        "start".to_string(),
        Rule::String {
            value: "x".to_string(),
        },
    );
    g
}

/// Serialize a GrammarJs to a pretty-printed JSON string.
fn to_json_string(g: &GrammarJs) -> String {
    serde_json::to_string_pretty(g).expect("GrammarJs should serialize")
}

/// Build the "arithmetic" GrammarJs from JSON, the way the pipeline does.
fn arithmetic_grammar_json() -> Value {
    json!({
        "name": "arithmetic",
        "rules": {
            "expression": {
                "type": "CHOICE",
                "members": [
                    { "type": "SYMBOL", "name": "sum" },
                    { "type": "SYMBOL", "name": "product" },
                    { "type": "SYMBOL", "name": "number" }
                ]
            },
            "sum": {
                "type": "SEQ",
                "members": [
                    { "type": "SYMBOL", "name": "expression" },
                    { "type": "STRING", "value": "+" },
                    { "type": "SYMBOL", "name": "expression" }
                ]
            },
            "product": {
                "type": "SEQ",
                "members": [
                    { "type": "SYMBOL", "name": "expression" },
                    { "type": "STRING", "value": "*" },
                    { "type": "SYMBOL", "name": "expression" }
                ]
            },
            "number": {
                "type": "PATTERN",
                "value": "\\d+"
            }
        },
        "extras": [
            { "type": "PATTERN", "value": "\\s" }
        ]
    })
}

// ===========================================================================
// 1. Simple grammar JSON generation
// ===========================================================================

#[test]
fn minimal_grammar_serializes_to_valid_json() {
    let g = minimal_grammar_js();
    let json_str = to_json_string(&g);
    let parsed: Value = serde_json::from_str(&json_str).expect("should be valid JSON");
    assert_eq!(parsed["name"], "minimal");
    assert!(parsed["rules"]["start"].is_object());
}

#[test]
fn grammar_json_contains_all_top_level_fields() {
    let g = minimal_grammar_js();
    let v: Value = serde_json::to_value(&g).unwrap();
    let obj = v.as_object().unwrap();
    for key in &[
        "name",
        "rules",
        "extras",
        "inline",
        "conflicts",
        "supertypes",
    ] {
        assert!(obj.contains_key(*key), "missing top-level key '{key}'");
    }
}

#[test]
fn grammar_json_name_matches() {
    let g = GrammarJs::new("my_lang".to_string());
    let v: Value = serde_json::to_value(&g).unwrap();
    assert_eq!(v["name"], "my_lang");
}

#[test]
fn grammar_json_rules_are_indexed_map() {
    // Verify rules are serialized as a JSON object (not array)
    let g = minimal_grammar_js();
    let v: Value = serde_json::to_value(&g).unwrap();
    assert!(v["rules"].is_object());
}

// ===========================================================================
// 2. Determinism — same grammar → same JSON
// ===========================================================================

#[test]
fn determinism_identical_grammars_produce_identical_json() {
    let g1 = minimal_grammar_js();
    let g2 = minimal_grammar_js();
    assert_eq!(to_json_string(&g1), to_json_string(&g2));
}

#[test]
fn determinism_multi_rule_grammar_stable_across_iterations() {
    // Build a grammar with many rules and check 50 iterations produce the same output
    let build = || {
        let mut g = GrammarJs::new("multi".to_string());
        for i in 0..20 {
            g.rules.insert(
                format!("rule_{i}"),
                Rule::Pattern {
                    value: format!("[a-z]{{{i}}}"),
                },
            );
        }
        to_json_string(&g)
    };
    let reference = build();
    for _ in 0..50 {
        assert_eq!(
            build(),
            reference,
            "JSON output should be identical across iterations"
        );
    }
}

#[test]
fn determinism_rule_insertion_order_preserved() {
    let mut g = GrammarJs::new("order".to_string());
    g.rules.insert("alpha".to_string(), Rule::Blank);
    g.rules.insert("beta".to_string(), Rule::Blank);
    g.rules.insert("gamma".to_string(), Rule::Blank);

    let v: Value = serde_json::to_value(&g).unwrap();
    let keys: Vec<&String> = v["rules"].as_object().unwrap().keys().collect();
    assert_eq!(keys, &["alpha", "beta", "gamma"]);
}

#[test]
fn determinism_roundtrip_preserves_rule_order() {
    let json_val = arithmetic_grammar_json();
    let g = from_json(&json_val).unwrap();
    let serialized = serde_json::to_value(&g).unwrap();
    let keys: Vec<&String> = serialized["rules"].as_object().unwrap().keys().collect();
    // The order from the original JSON should be preserved
    assert_eq!(keys, &["expression", "sum", "product", "number"]);
}

#[test]
fn determinism_serialize_deserialize_roundtrip() {
    let original = arithmetic_grammar_json();
    let g1 = from_json(&original).unwrap();
    let json_str = serde_json::to_string(&g1).unwrap();
    let g2: GrammarJs = serde_json::from_str(&json_str).unwrap();
    // Re-serialize and compare
    let json_str2 = serde_json::to_string(&g2).unwrap();
    assert_eq!(
        json_str, json_str2,
        "roundtrip serialization should be stable"
    );
}

// ===========================================================================
// 3. Valid JSON output
// ===========================================================================

#[test]
fn grammar_with_all_rule_types_serializes_to_valid_json() {
    let mut g = GrammarJs::new("all_types".to_string());
    g.rules.insert(
        "s".to_string(),
        Rule::String {
            value: "hello".to_string(),
        },
    );
    g.rules.insert(
        "p".to_string(),
        Rule::Pattern {
            value: r"\d+".to_string(),
        },
    );
    g.rules.insert(
        "sym".to_string(),
        Rule::Symbol {
            name: "s".to_string(),
        },
    );
    g.rules.insert("blank".to_string(), Rule::Blank);
    g.rules.insert(
        "seq".to_string(),
        Rule::Seq {
            members: vec![
                Rule::Symbol {
                    name: "s".to_string(),
                },
                Rule::Symbol {
                    name: "p".to_string(),
                },
            ],
        },
    );
    g.rules.insert(
        "choice".to_string(),
        Rule::Choice {
            members: vec![
                Rule::Symbol {
                    name: "s".to_string(),
                },
                Rule::Blank,
            ],
        },
    );
    g.rules.insert(
        "opt".to_string(),
        Rule::Optional {
            value: Box::new(Rule::Symbol {
                name: "s".to_string(),
            }),
        },
    );
    g.rules.insert(
        "rep".to_string(),
        Rule::Repeat {
            content: Box::new(Rule::Symbol {
                name: "p".to_string(),
            }),
        },
    );
    g.rules.insert(
        "rep1".to_string(),
        Rule::Repeat1 {
            content: Box::new(Rule::Symbol {
                name: "p".to_string(),
            }),
        },
    );
    g.rules.insert(
        "prec".to_string(),
        Rule::Prec {
            value: 5,
            content: Box::new(Rule::Symbol {
                name: "s".to_string(),
            }),
        },
    );
    g.rules.insert(
        "prec_left".to_string(),
        Rule::PrecLeft {
            value: 2,
            content: Box::new(Rule::Symbol {
                name: "seq".to_string(),
            }),
        },
    );
    g.rules.insert(
        "prec_right".to_string(),
        Rule::PrecRight {
            value: 3,
            content: Box::new(Rule::Symbol {
                name: "seq".to_string(),
            }),
        },
    );
    g.rules.insert(
        "prec_dynamic".to_string(),
        Rule::PrecDynamic {
            value: 1,
            content: Box::new(Rule::Blank),
        },
    );
    g.rules.insert(
        "tok".to_string(),
        Rule::Token {
            content: Box::new(Rule::Pattern {
                value: r"[0-9]+".to_string(),
            }),
        },
    );
    g.rules.insert(
        "imm_tok".to_string(),
        Rule::ImmediateToken {
            content: Box::new(Rule::String {
                value: ".".to_string(),
            }),
        },
    );
    g.rules.insert(
        "alias".to_string(),
        Rule::Alias {
            content: Box::new(Rule::Symbol {
                name: "s".to_string(),
            }),
            value: "aliased_name".to_string(),
            named: true,
        },
    );
    g.rules.insert(
        "field".to_string(),
        Rule::Field {
            name: "my_field".to_string(),
            content: Box::new(Rule::Symbol {
                name: "p".to_string(),
            }),
        },
    );

    let json_str = serde_json::to_string_pretty(&g).unwrap();
    let parsed: Value =
        serde_json::from_str(&json_str).expect("all-types grammar must be valid JSON");
    assert_eq!(parsed["rules"].as_object().unwrap().len(), 17);
}

#[test]
fn grammar_json_extras_are_array() {
    let mut g = GrammarJs::new("with_extras".to_string());
    g.rules.insert("start".to_string(), Rule::Blank);
    g.extras.push(Rule::Pattern {
        value: r"\s".to_string(),
    });
    g.extras.push(Rule::Symbol {
        name: "start".to_string(),
    });
    let v: Value = serde_json::to_value(&g).unwrap();
    assert!(v["extras"].is_array());
    assert_eq!(v["extras"].as_array().unwrap().len(), 2);
}

#[test]
fn grammar_json_externals_serialize() {
    let mut g = GrammarJs::new("ext".to_string());
    g.rules.insert("start".to_string(), Rule::Blank);
    g.externals.push(ExternalToken {
        name: "indent".to_string(),
        symbol: "external_0".to_string(),
    });
    let v: Value = serde_json::to_value(&g).unwrap();
    assert_eq!(v["externals"][0]["name"], "indent");
}

// ===========================================================================
// 4. Various grammar types
// ===========================================================================

#[test]
fn simple_string_grammar_roundtrip() {
    let v = json!({
        "name": "simple_str",
        "rules": {
            "start": { "type": "STRING", "value": "hello" }
        }
    });
    let g = from_json(&v).unwrap();
    assert_eq!(g.name, "simple_str");
    assert_eq!(g.rules.len(), 1);
    match &g.rules["start"] {
        Rule::String { value } => assert_eq!(value, "hello"),
        other => panic!("expected STRING rule, got {other:?}"),
    }
}

#[test]
fn recursive_grammar_parses_correctly() {
    let v = json!({
        "name": "recursive",
        "rules": {
            "expr": {
                "type": "CHOICE",
                "members": [
                    {
                        "type": "SEQ",
                        "members": [
                            { "type": "SYMBOL", "name": "expr" },
                            { "type": "STRING", "value": "+" },
                            { "type": "SYMBOL", "name": "expr" }
                        ]
                    },
                    { "type": "PATTERN", "value": "\\d+" }
                ]
            }
        }
    });
    let g = from_json(&v).unwrap();
    assert_eq!(g.rules.len(), 1);
    match &g.rules["expr"] {
        Rule::Choice { members } => assert_eq!(members.len(), 2),
        other => panic!("expected CHOICE, got {other:?}"),
    }
}

#[test]
fn grammar_with_precedence_rules() {
    let v = json!({
        "name": "prec_test",
        "rules": {
            "expr": {
                "type": "CHOICE",
                "members": [
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
                    },
                    {
                        "type": "PREC_LEFT",
                        "value": 2,
                        "content": {
                            "type": "SEQ",
                            "members": [
                                { "type": "SYMBOL", "name": "expr" },
                                { "type": "STRING", "value": "*" },
                                { "type": "SYMBOL", "name": "expr" }
                            ]
                        }
                    },
                    {
                        "type": "PREC_RIGHT",
                        "value": 3,
                        "content": {
                            "type": "SEQ",
                            "members": [
                                { "type": "SYMBOL", "name": "expr" },
                                { "type": "STRING", "value": "^" },
                                { "type": "SYMBOL", "name": "expr" }
                            ]
                        }
                    },
                    { "type": "PATTERN", "value": "\\d+" }
                ]
            }
        }
    });
    let g = from_json(&v).unwrap();
    let rule = &g.rules["expr"];
    match rule {
        Rule::Choice { members } => {
            assert_eq!(members.len(), 4);
            // Check the precedence types
            assert!(matches!(&members[0], Rule::PrecLeft { value: 1, .. }));
            assert!(matches!(&members[1], Rule::PrecLeft { value: 2, .. }));
            assert!(matches!(&members[2], Rule::PrecRight { value: 3, .. }));
            assert!(matches!(&members[3], Rule::Pattern { .. }));
        }
        other => panic!("expected CHOICE, got {other:?}"),
    }
}

#[test]
fn grammar_with_optional_and_repeat() {
    let v = json!({
        "name": "rep_opt",
        "rules": {
            "list": {
                "type": "SEQ",
                "members": [
                    { "type": "STRING", "value": "[" },
                    {
                        "type": "OPTIONAL",
                        "value": {
                            "type": "SEQ",
                            "members": [
                                { "type": "SYMBOL", "name": "item" },
                                {
                                    "type": "REPEAT",
                                    "content": {
                                        "type": "SEQ",
                                        "members": [
                                            { "type": "STRING", "value": "," },
                                            { "type": "SYMBOL", "name": "item" }
                                        ]
                                    }
                                }
                            ]
                        }
                    },
                    { "type": "STRING", "value": "]" }
                ]
            },
            "item": { "type": "PATTERN", "value": "\\w+" }
        }
    });
    let g = from_json(&v).unwrap();
    assert_eq!(g.rules.len(), 2);
    match &g.rules["list"] {
        Rule::Seq { members } => assert_eq!(members.len(), 3),
        other => panic!("expected SEQ, got {other:?}"),
    }
}

#[test]
fn grammar_with_field_and_alias() {
    let v = json!({
        "name": "field_alias",
        "rules": {
            "assignment": {
                "type": "SEQ",
                "members": [
                    {
                        "type": "FIELD",
                        "name": "left",
                        "content": { "type": "SYMBOL", "name": "identifier" }
                    },
                    { "type": "STRING", "value": "=" },
                    {
                        "type": "FIELD",
                        "name": "right",
                        "content": {
                            "type": "ALIAS",
                            "content": { "type": "SYMBOL", "name": "identifier" },
                            "value": "value",
                            "named": true
                        }
                    }
                ]
            },
            "identifier": { "type": "PATTERN", "value": "[a-z]+" }
        }
    });
    let g = from_json(&v).unwrap();
    match &g.rules["assignment"] {
        Rule::Seq { members } => {
            assert!(matches!(&members[0], Rule::Field { name, .. } if name == "left"));
            assert!(matches!(&members[2], Rule::Field { name, .. } if name == "right"));
        }
        other => panic!("expected SEQ, got {other:?}"),
    }
}

// ===========================================================================
// 5. Python-like and JavaScript-like preset grammars
// ===========================================================================

#[test]
fn python_like_grammar_converts_to_grammar_js() {
    let grammar = GrammarBuilder::python_like();
    let converter = GrammarJsConverter::new(build_grammar_js_from_ir(&grammar));
    let ir = converter.convert().unwrap();
    assert_eq!(ir.name, "python_like");
    assert!(!ir.rules.is_empty());
}

#[test]
fn javascript_like_grammar_converts_to_grammar_js() {
    let grammar = GrammarBuilder::javascript_like();
    let converter = GrammarJsConverter::new(build_grammar_js_from_ir(&grammar));
    let ir = converter.convert().unwrap();
    assert_eq!(ir.name, "javascript_like");
    assert!(!ir.rules.is_empty());
}

#[test]
fn python_like_grammar_json_is_deterministic() {
    let g1 = build_grammar_js_from_ir(&GrammarBuilder::python_like());
    let g2 = build_grammar_js_from_ir(&GrammarBuilder::python_like());
    assert_eq!(to_json_string(&g1), to_json_string(&g2));
}

#[test]
fn javascript_like_grammar_json_is_deterministic() {
    let g1 = build_grammar_js_from_ir(&GrammarBuilder::javascript_like());
    let g2 = build_grammar_js_from_ir(&GrammarBuilder::javascript_like());
    assert_eq!(to_json_string(&g1), to_json_string(&g2));
}

#[test]
fn python_like_grammar_has_externals() {
    let g = build_grammar_js_from_ir(&GrammarBuilder::python_like());
    // Python-like grammar should have INDENT/DEDENT externals
    assert!(
        !g.externals.is_empty(),
        "python-like grammar should have external tokens"
    );
}

#[test]
fn javascript_like_grammar_has_many_rules() {
    let g = build_grammar_js_from_ir(&GrammarBuilder::javascript_like());
    // JS-like grammar has: program, statement, var_declaration, function_declaration,
    // block, statements, expression_statement, expression — at least 8 non-terminal rules
    assert!(
        g.rules.len() >= 5,
        "javascript-like grammar should have at least 5 rules, got {}",
        g.rules.len()
    );
}

// ===========================================================================
// 6. Validation
// ===========================================================================

#[test]
fn validate_accepts_valid_grammar() {
    let v = json!({
        "name": "valid",
        "rules": {
            "start": { "type": "SYMBOL", "name": "token" },
            "token": { "type": "PATTERN", "value": "\\w+" }
        }
    });
    let g = from_json(&v).unwrap();
    assert!(g.validate().is_ok());
}

#[test]
fn validate_rejects_missing_symbol() {
    let mut g = GrammarJs::new("invalid".to_string());
    g.rules.insert(
        "start".to_string(),
        Rule::Symbol {
            name: "nonexistent".to_string(),
        },
    );
    assert!(g.validate().is_err());
}

#[test]
fn validate_rejects_invalid_word_token() {
    let mut g = GrammarJs::new("bad_word".to_string());
    g.rules.insert("start".to_string(), Rule::Blank);
    g.word = Some("missing_token".to_string());
    assert!(g.validate().is_err());
}

#[test]
fn validate_rejects_invalid_inline_rule() {
    let mut g = GrammarJs::new("bad_inline".to_string());
    g.rules.insert("start".to_string(), Rule::Blank);
    g.inline.push("not_a_rule".to_string());
    assert!(g.validate().is_err());
}

#[test]
fn validate_rejects_invalid_conflict_rule() {
    let mut g = GrammarJs::new("bad_conflict".to_string());
    g.rules.insert("start".to_string(), Rule::Blank);
    g.conflicts
        .push(vec!["start".to_string(), "missing".to_string()]);
    assert!(g.validate().is_err());
}

// ===========================================================================
// 7. Edge cases
// ===========================================================================

#[test]
fn empty_grammar_serializes() {
    let g = GrammarJs::new("empty".to_string());
    let json_str = to_json_string(&g);
    let v: Value = serde_json::from_str(&json_str).unwrap();
    assert_eq!(v["name"], "empty");
    assert!(v["rules"].as_object().unwrap().is_empty());
}

#[test]
fn deeply_nested_rule_serializes() {
    let mut g = GrammarJs::new("deep".to_string());
    // Build a deeply nested optional(repeat(prec_left(string)))
    let inner = Rule::String {
        value: "x".to_string(),
    };
    let prec = Rule::PrecLeft {
        value: 1,
        content: Box::new(inner),
    };
    let rep = Rule::Repeat {
        content: Box::new(prec),
    };
    let opt = Rule::Optional {
        value: Box::new(rep),
    };
    g.rules.insert("deep_rule".to_string(), opt);

    let json_str = to_json_string(&g);
    let parsed: Value = serde_json::from_str(&json_str).unwrap();
    // Verify the nesting: rules.deep_rule.type == OPTIONAL
    assert_eq!(parsed["rules"]["deep_rule"]["type"], "OPTIONAL");
}

#[test]
fn grammar_with_unicode_rule_names() {
    let mut g = GrammarJs::new("unicode_lang".to_string());
    g.rules.insert(
        "日本語".to_string(),
        Rule::Pattern {
            value: r"\p{Katakana}+".to_string(),
        },
    );
    let json_str = to_json_string(&g);
    let parsed: Value = serde_json::from_str(&json_str).unwrap();
    assert!(parsed["rules"]["日本語"].is_object());
}

// ===========================================================================
// Helper: Convert Grammar IR → GrammarJs (via JSON roundtrip)
// ===========================================================================

/// Converts a Grammar IR (from GrammarBuilder) into a GrammarJs by going through
/// the same JSON path the real pipeline uses.
fn build_grammar_js_from_ir(grammar: &Grammar) -> GrammarJs {
    // Build a Tree-sitter-style JSON object from the IR
    let mut rules = serde_json::Map::new();

    // Collect rule names and their symbol IDs
    let rule_names: Vec<(adze_ir::SymbolId, String)> = grammar
        .rule_names
        .iter()
        .map(|(id, name)| (*id, name.clone()))
        .collect();

    for (symbol_id, name) in &rule_names {
        if let Some(productions) = grammar.rules.get(symbol_id) {
            let rule_json = productions_to_json(productions, grammar);
            rules.insert(name.clone(), rule_json);
        }
    }

    // Build extras
    let mut extras = Vec::new();
    for extra_id in &grammar.extras {
        if let Some(token) = grammar.tokens.get(extra_id) {
            extras.push(json!({ "type": "PATTERN", "value": token.pattern.as_str() }));
        }
    }

    // Build externals
    let mut externals = Vec::new();
    for ext in &grammar.externals {
        externals.push(json!({ "type": "SYMBOL", "name": ext.name }));
    }

    let grammar_json = json!({
        "name": grammar.name,
        "rules": rules,
        "extras": extras,
        "externals": externals,
    });

    from_json(&grammar_json).unwrap()
}

/// Convert a list of productions for a single non-terminal into a JSON rule.
fn productions_to_json(productions: &[adze_ir::Rule], grammar: &Grammar) -> Value {
    if productions.len() == 1 {
        production_to_json(&productions[0], grammar)
    } else {
        let members: Vec<Value> = productions
            .iter()
            .map(|p| production_to_json(p, grammar))
            .collect();
        json!({ "type": "CHOICE", "members": members })
    }
}

/// Convert a single production (Rule) into a JSON rule.
fn production_to_json(production: &adze_ir::Rule, grammar: &Grammar) -> Value {
    let rhs_json: Vec<Value> = production
        .rhs
        .iter()
        .filter_map(|sym| symbol_to_json(sym, grammar))
        .collect();

    let base = if rhs_json.is_empty() {
        json!({ "type": "BLANK" })
    } else if rhs_json.len() == 1 {
        rhs_json.into_iter().next().unwrap()
    } else {
        json!({ "type": "SEQ", "members": rhs_json })
    };

    // Wrap in precedence if present
    match (&production.precedence, &production.associativity) {
        (Some(adze_ir::PrecedenceKind::Static(level)), Some(Associativity::Left)) => {
            json!({ "type": "PREC_LEFT", "value": level, "content": base })
        }
        (Some(adze_ir::PrecedenceKind::Static(level)), Some(Associativity::Right)) => {
            json!({ "type": "PREC_RIGHT", "value": level, "content": base })
        }
        (Some(adze_ir::PrecedenceKind::Static(level)), _) => {
            json!({ "type": "PREC", "value": level, "content": base })
        }
        _ => base,
    }
}

/// Convert a Symbol to its JSON representation.
fn symbol_to_json(symbol: &adze_ir::Symbol, grammar: &Grammar) -> Option<Value> {
    match symbol {
        adze_ir::Symbol::Terminal(id) => grammar.tokens.get(id).map(|token| match &token.pattern {
            adze_ir::TokenPattern::String(s) => json!({ "type": "STRING", "value": s }),
            adze_ir::TokenPattern::Regex(r) => json!({ "type": "PATTERN", "value": r }),
        }),
        adze_ir::Symbol::NonTerminal(id) => grammar
            .rule_names
            .get(id)
            .map(|name| json!({ "type": "SYMBOL", "name": name })),
        adze_ir::Symbol::Epsilon => None,
        _ => None,
    }
}

// ===========================================================================
// Trait impl for TokenPattern helper
// ===========================================================================

trait TokenPatternExt {
    fn as_str(&self) -> &str;
}

impl TokenPatternExt for adze_ir::TokenPattern {
    fn as_str(&self) -> &str {
        match self {
            adze_ir::TokenPattern::String(s) => s,
            adze_ir::TokenPattern::Regex(r) => r,
        }
    }
}
