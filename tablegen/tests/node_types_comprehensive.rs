//! Comprehensive tests for NODE_TYPES JSON generation in adze-tablegen.
//!
//! Covers: schema validation, roundtrip serialization, edge cases,
//! field handling, symbol variants, and deterministic output.

use adze_ir::builder::GrammarBuilder;
use adze_ir::{
    Associativity, ExternalToken, FieldId, Grammar, ProductionId, Rule, Symbol, SymbolId, Token,
    TokenPattern,
};
use adze_tablegen::NodeTypesGenerator;
use serde_json::Value;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Parse generated JSON and return the array of node type objects.
fn generate_and_parse(grammar: &Grammar) -> Vec<Value> {
    let generator = NodeTypesGenerator::new(grammar);
    let json = generator
        .generate()
        .expect("NodeTypesGenerator::generate() failed");
    let val: Value = serde_json::from_str(&json).expect("output is not valid JSON");
    val.as_array().expect("output is not a JSON array").to_vec()
}

/// Find a node type entry by its `type` field.
fn find_node<'a>(nodes: &'a [Value], type_name: &str) -> Option<&'a Value> {
    nodes.iter().find(|n| n["type"].as_str() == Some(type_name))
}

// ---------------------------------------------------------------------------
// 1. Schema shape — every entry must have `type` (string) + `named` (bool)
// ---------------------------------------------------------------------------

#[test]
fn schema_required_fields_present() {
    let g = GrammarBuilder::new("schema")
        .token("num", r"\d+")
        .token("+", "+")
        .rule("expr", vec!["num"])
        .build();

    for entry in generate_and_parse(&g) {
        assert!(
            entry.get("type").and_then(Value::as_str).is_some(),
            "entry missing 'type': {entry}"
        );
        assert!(
            entry.get("named").and_then(Value::as_bool).is_some(),
            "entry missing 'named': {entry}"
        );
    }
}

// ---------------------------------------------------------------------------
// 2. Optional keys are absent when not applicable
// ---------------------------------------------------------------------------

#[test]
fn optional_keys_absent_when_empty() {
    let g = GrammarBuilder::new("opt")
        .token("x", "x")
        .rule("start", vec!["x"])
        .build();

    let nodes = generate_and_parse(&g);
    // The literal "x" token should have no fields, children, or subtypes.
    let x_node = find_node(&nodes, "x").expect("missing 'x'");
    assert!(x_node.get("fields").is_none(), "unexpected 'fields'");
    assert!(x_node.get("children").is_none(), "unexpected 'children'");
    assert!(x_node.get("subtypes").is_none(), "unexpected 'subtypes'");
}

// ---------------------------------------------------------------------------
// 3. Named vs anonymous classification
// ---------------------------------------------------------------------------

#[test]
fn regex_tokens_are_named() {
    let g = GrammarBuilder::new("named_tok")
        .token("identifier", r"[a-z]+")
        .rule("start", vec!["identifier"])
        .build();

    let nodes = generate_and_parse(&g);
    // Regex tokens should NOT appear as anonymous nodes in the output.
    // They appear as named when referenced via rules. Verify no unnamed 'identifier' entry.
    let anon: Vec<_> = nodes.iter().filter(|n| n["named"] == false).collect();
    for n in &anon {
        assert_ne!(
            n["type"].as_str(),
            Some("identifier"),
            "regex token should not be anonymous"
        );
    }
}

#[test]
fn string_literal_tokens_are_anonymous() {
    let g = GrammarBuilder::new("anon_tok")
        .token("+", "+")
        .token(";", ";")
        .token("id", r"[a-z]+")
        .rule("stmt", vec!["id", ";"])
        .build();

    let nodes = generate_and_parse(&g);
    let semi = find_node(&nodes, ";").expect("missing ';'");
    assert_eq!(semi["named"], false, "literal ';' should be anonymous");
}

// ---------------------------------------------------------------------------
// 4. Internal rules (prefixed with _) are excluded
// ---------------------------------------------------------------------------

#[test]
fn internal_rules_excluded_from_output() {
    let mut g = Grammar::new("internal".to_string());

    let tok_id = SymbolId(0);
    g.tokens.insert(
        tok_id,
        Token {
            name: "a".to_string(),
            pattern: TokenPattern::String("a".to_string()),
            fragile: false,
        },
    );

    // Public rule
    let pub_id = SymbolId(1);
    g.rule_names.insert(pub_id, "public_rule".to_string());
    g.add_rule(Rule {
        lhs: pub_id,
        rhs: vec![Symbol::Terminal(tok_id)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });

    // Internal rule (starts with _)
    let priv_id = SymbolId(2);
    g.rule_names.insert(priv_id, "_hidden".to_string());
    g.add_rule(Rule {
        lhs: priv_id,
        rhs: vec![Symbol::Terminal(tok_id)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(1),
    });

    let nodes = generate_and_parse(&g);
    assert!(
        find_node(&nodes, "_hidden").is_none(),
        "internal rule should be excluded"
    );
    assert!(
        find_node(&nodes, "public_rule").is_some(),
        "public rule should be present"
    );
}

// ---------------------------------------------------------------------------
// 5. Fields appear on node types
// ---------------------------------------------------------------------------

#[test]
fn fields_attached_to_node_type() {
    let mut g = Grammar::new("fields".to_string());

    let num_id = SymbolId(0);
    g.tokens.insert(
        num_id,
        Token {
            name: "number".to_string(),
            pattern: TokenPattern::Regex(r"\d+".to_string()),
            fragile: false,
        },
    );

    let op_id = SymbolId(1);
    g.tokens.insert(
        op_id,
        Token {
            name: "op".to_string(),
            pattern: TokenPattern::String("+".to_string()),
            fragile: false,
        },
    );

    let expr_id = SymbolId(10);
    g.rule_names
        .insert(expr_id, "binary_expression".to_string());

    let left_field = FieldId(0);
    let right_field = FieldId(1);
    g.fields.insert(left_field, "left".to_string());
    g.fields.insert(right_field, "right".to_string());

    g.add_rule(Rule {
        lhs: expr_id,
        rhs: vec![
            Symbol::Terminal(num_id),
            Symbol::Terminal(op_id),
            Symbol::Terminal(num_id),
        ],
        precedence: None,
        associativity: None,
        fields: vec![(left_field, 0), (right_field, 2)],
        production_id: ProductionId(0),
    });

    let nodes = generate_and_parse(&g);
    let expr = find_node(&nodes, "binary_expression").expect("missing binary_expression");
    let fields_obj = expr
        .get("fields")
        .expect("binary_expression should have fields");
    assert!(fields_obj.get("left").is_some(), "missing 'left' field");
    assert!(fields_obj.get("right").is_some(), "missing 'right' field");
}

// ---------------------------------------------------------------------------
// 6. Field types array contains correct type references
// ---------------------------------------------------------------------------

#[test]
fn field_types_reference_correct_symbols() {
    let mut g = Grammar::new("field_types".to_string());

    let num_id = SymbolId(0);
    g.tokens.insert(
        num_id,
        Token {
            name: "number".to_string(),
            pattern: TokenPattern::Regex(r"\d+".to_string()),
            fragile: false,
        },
    );

    let expr_id = SymbolId(10);
    g.rule_names.insert(expr_id, "wrapper".to_string());
    let val_field = FieldId(0);
    g.fields.insert(val_field, "value".to_string());

    g.add_rule(Rule {
        lhs: expr_id,
        rhs: vec![Symbol::Terminal(num_id)],
        precedence: None,
        associativity: None,
        fields: vec![(val_field, 0)],
        production_id: ProductionId(0),
    });

    let nodes = generate_and_parse(&g);
    let wrapper = find_node(&nodes, "wrapper").expect("missing wrapper");
    let value_field = &wrapper["fields"]["value"];
    let types = value_field["types"]
        .as_array()
        .expect("types should be array");
    assert!(!types.is_empty(), "types array should not be empty");

    // The type reference should point to the number token (named regex token)
    let first_type = &types[0];
    assert_eq!(first_type["type"].as_str(), Some("number"));
    assert_eq!(first_type["named"], true);
}

// ---------------------------------------------------------------------------
// 7. JSON roundtrip: parse → serialize → parse yields identical value
// ---------------------------------------------------------------------------

#[test]
fn json_roundtrip_identity() {
    let g = GrammarBuilder::new("roundtrip")
        .token("num", r"\d+")
        .token("+", "+")
        .token("-", "-")
        .rule("expr", vec!["num"])
        .rule("add", vec!["expr", "+", "expr"])
        .rule("sub", vec!["expr", "-", "expr"])
        .build();

    let generator = NodeTypesGenerator::new(&g);
    let json1 = generator.generate().unwrap();
    let val1: Value = serde_json::from_str(&json1).unwrap();

    // Serialize back to string, then parse again
    let json2 = serde_json::to_string_pretty(&val1).unwrap();
    let val2: Value = serde_json::from_str(&json2).unwrap();

    assert_eq!(val1, val2, "roundtrip must be identity");
}

// ---------------------------------------------------------------------------
// 8. Output is sorted alphabetically by type name
// ---------------------------------------------------------------------------

#[test]
fn output_sorted_by_type_name() {
    let g = GrammarBuilder::new("sort")
        .token("z_tok", r"z+")
        .token("a_tok", r"a+")
        .token("m_tok", r"m+")
        .rule("zebra", vec!["z_tok"])
        .rule("alpha", vec!["a_tok"])
        .rule("middle", vec!["m_tok"])
        .build();

    let nodes = generate_and_parse(&g);
    let names: Vec<&str> = nodes.iter().filter_map(|n| n["type"].as_str()).collect();
    let mut sorted = names.clone();
    sorted.sort();
    assert_eq!(names, sorted, "entries should be alphabetically sorted");
}

// ---------------------------------------------------------------------------
// 9. Determinism — same grammar → same output across invocations
// ---------------------------------------------------------------------------

#[test]
fn deterministic_across_invocations() {
    let make = || {
        GrammarBuilder::new("det")
            .token("num", r"\d+")
            .token("+", "+")
            .rule("expr", vec!["num", "+", "num"])
            .build()
    };

    let j1 = NodeTypesGenerator::new(&make()).generate().unwrap();
    let j2 = NodeTypesGenerator::new(&make()).generate().unwrap();
    let v1: Value = serde_json::from_str(&j1).unwrap();
    let v2: Value = serde_json::from_str(&j2).unwrap();
    assert_eq!(v1, v2, "identical grammars must produce identical JSON");
}

// ---------------------------------------------------------------------------
// 10. Empty grammar → valid empty JSON array
// ---------------------------------------------------------------------------

#[test]
fn empty_grammar_yields_empty_array() {
    let g = Grammar::new("empty".to_string());
    let nodes = generate_and_parse(&g);
    assert!(nodes.is_empty(), "empty grammar should produce []");
}

// ---------------------------------------------------------------------------
// 11. Rules referencing NonTerminal symbols
// ---------------------------------------------------------------------------

#[test]
fn nonterminal_reference_in_fields() {
    let mut g = Grammar::new("nt_ref".to_string());

    let tok_id = SymbolId(0);
    g.tokens.insert(
        tok_id,
        Token {
            name: "id".to_string(),
            pattern: TokenPattern::Regex(r"[a-z]+".to_string()),
            fragile: false,
        },
    );

    let child_id = SymbolId(1);
    g.rule_names.insert(child_id, "child".to_string());
    g.add_rule(Rule {
        lhs: child_id,
        rhs: vec![Symbol::Terminal(tok_id)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });

    let parent_id = SymbolId(2);
    g.rule_names.insert(parent_id, "parent".to_string());
    let body_field = FieldId(0);
    g.fields.insert(body_field, "body".to_string());
    g.add_rule(Rule {
        lhs: parent_id,
        rhs: vec![Symbol::NonTerminal(child_id)],
        precedence: None,
        associativity: None,
        fields: vec![(body_field, 0)],
        production_id: ProductionId(1),
    });

    let nodes = generate_and_parse(&g);
    let parent = find_node(&nodes, "parent").expect("missing 'parent'");
    let body_types = &parent["fields"]["body"]["types"];
    let first = &body_types[0];
    assert_eq!(first["type"].as_str(), Some("child"));
    assert_eq!(
        first["named"], true,
        "non-terminal reference should be named"
    );
}

// ---------------------------------------------------------------------------
// 12. External symbol produces named "external" type reference
// ---------------------------------------------------------------------------

#[test]
fn external_symbol_type_ref() {
    let mut g = Grammar::new("ext".to_string());

    let ext_sym = SymbolId(50);
    g.externals.push(ExternalToken {
        name: "indent".to_string(),
        symbol_id: ext_sym,
    });

    let rule_id = SymbolId(1);
    g.rule_names.insert(rule_id, "block".to_string());
    let indent_field = FieldId(0);
    g.fields.insert(indent_field, "indent".to_string());
    g.add_rule(Rule {
        lhs: rule_id,
        rhs: vec![Symbol::External(ext_sym)],
        precedence: None,
        associativity: None,
        fields: vec![(indent_field, 0)],
        production_id: ProductionId(0),
    });

    let nodes = generate_and_parse(&g);
    let block = find_node(&nodes, "block").expect("missing 'block'");
    let indent_types = &block["fields"]["indent"]["types"];
    let first = &indent_types[0];
    assert_eq!(first["type"].as_str(), Some("external"));
    assert_eq!(first["named"], true);
}

// ---------------------------------------------------------------------------
// 13. Choice symbol uses first alternative's type reference
// ---------------------------------------------------------------------------

#[test]
fn choice_symbol_uses_first_alternative() {
    let mut g = Grammar::new("choice".to_string());

    let num_id = SymbolId(0);
    g.tokens.insert(
        num_id,
        Token {
            name: "number".to_string(),
            pattern: TokenPattern::Regex(r"\d+".to_string()),
            fragile: false,
        },
    );

    let str_id = SymbolId(1);
    g.tokens.insert(
        str_id,
        Token {
            name: "string".to_string(),
            pattern: TokenPattern::Regex(r#""[^"]*""#.to_string()),
            fragile: false,
        },
    );

    let val_id = SymbolId(10);
    g.rule_names.insert(val_id, "value".to_string());
    let inner_field = FieldId(0);
    g.fields.insert(inner_field, "inner".to_string());

    g.add_rule(Rule {
        lhs: val_id,
        rhs: vec![Symbol::Choice(vec![
            Symbol::Terminal(num_id),
            Symbol::Terminal(str_id),
        ])],
        precedence: None,
        associativity: None,
        fields: vec![(inner_field, 0)],
        production_id: ProductionId(0),
    });

    let nodes = generate_and_parse(&g);
    let value = find_node(&nodes, "value").expect("missing 'value'");
    let inner_types = &value["fields"]["inner"]["types"];
    let first = &inner_types[0];
    // Choice uses the first alternative
    assert_eq!(first["type"].as_str(), Some("number"));
}

// ---------------------------------------------------------------------------
// 14. Sequence symbol uses first element's type reference
// ---------------------------------------------------------------------------

#[test]
fn sequence_symbol_uses_first_element() {
    let mut g = Grammar::new("seq".to_string());

    let a_id = SymbolId(0);
    g.tokens.insert(
        a_id,
        Token {
            name: "alpha".to_string(),
            pattern: TokenPattern::Regex(r"[a-z]+".to_string()),
            fragile: false,
        },
    );

    let b_id = SymbolId(1);
    g.tokens.insert(
        b_id,
        Token {
            name: "beta".to_string(),
            pattern: TokenPattern::Regex(r"[A-Z]+".to_string()),
            fragile: false,
        },
    );

    let rule_id = SymbolId(10);
    g.rule_names.insert(rule_id, "pair".to_string());
    let combo_field = FieldId(0);
    g.fields.insert(combo_field, "combo".to_string());

    g.add_rule(Rule {
        lhs: rule_id,
        rhs: vec![Symbol::Sequence(vec![
            Symbol::Terminal(a_id),
            Symbol::Terminal(b_id),
        ])],
        precedence: None,
        associativity: None,
        fields: vec![(combo_field, 0)],
        production_id: ProductionId(0),
    });

    let nodes = generate_and_parse(&g);
    let pair = find_node(&nodes, "pair").expect("missing 'pair'");
    let combo_types = &pair["fields"]["combo"]["types"];
    assert_eq!(combo_types[0]["type"].as_str(), Some("alpha"));
}

// ---------------------------------------------------------------------------
// 15. Epsilon symbol produces "empty" type reference
// ---------------------------------------------------------------------------

#[test]
fn epsilon_symbol_type_ref() {
    let mut g = Grammar::new("eps".to_string());

    let rule_id = SymbolId(1);
    g.rule_names.insert(rule_id, "void_rule".to_string());
    let phantom_field = FieldId(0);
    g.fields.insert(phantom_field, "phantom".to_string());

    g.add_rule(Rule {
        lhs: rule_id,
        rhs: vec![Symbol::Epsilon],
        precedence: None,
        associativity: None,
        fields: vec![(phantom_field, 0)],
        production_id: ProductionId(0),
    });

    let nodes = generate_and_parse(&g);
    let void_rule = find_node(&nodes, "void_rule").expect("missing 'void_rule'");
    let types = &void_rule["fields"]["phantom"]["types"];
    assert_eq!(types[0]["type"].as_str(), Some("empty"));
    assert_eq!(types[0]["named"], false);
}

// ---------------------------------------------------------------------------
// 16. Optional symbol delegates to inner type
// ---------------------------------------------------------------------------

#[test]
fn optional_symbol_delegates_to_inner() {
    let mut g = Grammar::new("opt_sym".to_string());

    let tok_id = SymbolId(0);
    g.tokens.insert(
        tok_id,
        Token {
            name: "kw".to_string(),
            pattern: TokenPattern::String("return".to_string()),
            fragile: false,
        },
    );

    let rule_id = SymbolId(10);
    g.rule_names.insert(rule_id, "maybe_return".to_string());
    let kw_field = FieldId(0);
    g.fields.insert(kw_field, "keyword".to_string());

    g.add_rule(Rule {
        lhs: rule_id,
        rhs: vec![Symbol::Optional(Box::new(Symbol::Terminal(tok_id)))],
        precedence: None,
        associativity: None,
        fields: vec![(kw_field, 0)],
        production_id: ProductionId(0),
    });

    let nodes = generate_and_parse(&g);
    let mr = find_node(&nodes, "maybe_return").expect("missing 'maybe_return'");
    let types = &mr["fields"]["keyword"]["types"];
    // Optional delegates to the inner token's type
    assert_eq!(types[0]["type"].as_str(), Some("return"));
}

// ---------------------------------------------------------------------------
// 17. Repeat / RepeatOne delegate to inner type
// ---------------------------------------------------------------------------

#[test]
fn repeat_symbols_delegate_to_inner() {
    let mut g = Grammar::new("rep".to_string());

    let tok_id = SymbolId(0);
    g.tokens.insert(
        tok_id,
        Token {
            name: "digit".to_string(),
            pattern: TokenPattern::Regex(r"\d".to_string()),
            fragile: false,
        },
    );

    let rule_id = SymbolId(10);
    g.rule_names.insert(rule_id, "digits".to_string());
    let items_field = FieldId(0);
    g.fields.insert(items_field, "items".to_string());

    g.add_rule(Rule {
        lhs: rule_id,
        rhs: vec![Symbol::Repeat(Box::new(Symbol::Terminal(tok_id)))],
        precedence: None,
        associativity: None,
        fields: vec![(items_field, 0)],
        production_id: ProductionId(0),
    });

    let nodes = generate_and_parse(&g);
    let digits = find_node(&nodes, "digits").expect("missing 'digits'");
    let types = &digits["fields"]["items"]["types"];
    assert_eq!(types[0]["type"].as_str(), Some("digit"));
    assert_eq!(types[0]["named"], true);
}

// ---------------------------------------------------------------------------
// 18. Multiple rules for same LHS do not duplicate node type
// ---------------------------------------------------------------------------

#[test]
fn multiple_rules_same_lhs_no_duplicate() {
    let g = GrammarBuilder::new("multi")
        .token("a", r"a+")
        .token("b", r"b+")
        .rule("expr", vec!["a"])
        .rule("expr", vec!["b"])
        .build();

    let nodes = generate_and_parse(&g);
    let count = nodes
        .iter()
        .filter(|n| n["type"].as_str() == Some("expr"))
        .count();
    assert_eq!(count, 1, "same LHS should produce exactly one node type");
}

// ---------------------------------------------------------------------------
// 19. Large grammar — many rules and tokens, still valid JSON
// ---------------------------------------------------------------------------

#[test]
fn large_grammar_valid_json() {
    let mut builder = GrammarBuilder::new("large");
    for i in 0..50 {
        let tok_name = format!("tok_{i}");
        let rule_name = format!("rule_{i}");
        builder = builder
            .token(&tok_name, &format!(r"t{i}"))
            .rule(&rule_name, vec![&tok_name]);
        // Borrow checker workaround: we must convert the string back for the vec
        // Actually, builder consumes self, so the above won't work directly.
        // Let's use the raw Grammar API instead.
    }
    // The builder approach above has lifetime issues in a loop.
    // Use raw Grammar API for the large test.

    let mut g = Grammar::new("large".to_string());
    for i in 0u16..50 {
        let tok_id = SymbolId(i);
        g.tokens.insert(
            tok_id,
            Token {
                name: format!("tok_{i}"),
                pattern: TokenPattern::Regex(format!("t{i}")),
                fragile: false,
            },
        );

        let rule_id = SymbolId(100 + i);
        g.rule_names.insert(rule_id, format!("rule_{i}"));
        g.add_rule(Rule {
            lhs: rule_id,
            rhs: vec![Symbol::Terminal(tok_id)],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(i),
        });
    }

    let generator = NodeTypesGenerator::new(&g);
    let json = generator.generate().expect("large grammar should succeed");
    let val: Value = serde_json::from_str(&json).expect("output must be valid JSON");
    let arr = val.as_array().expect("must be array");
    // At least 50 rules should appear
    assert!(arr.len() >= 50, "expected ≥50 entries, got {}", arr.len());
}

// ---------------------------------------------------------------------------
// 20. No null type names in output
// ---------------------------------------------------------------------------

#[test]
fn no_null_type_names() {
    let g = GrammarBuilder::new("nullcheck")
        .token("x", r"x+")
        .token("y", "y")
        .rule("start", vec!["x"])
        .rule("alt", vec!["y"])
        .build();

    for entry in generate_and_parse(&g) {
        let t = entry.get("type").expect("missing type");
        assert!(!t.is_null(), "type must not be null: {entry}");
        let s = t.as_str().expect("type must be string");
        assert!(!s.is_empty(), "type must not be empty");
    }
}

// ---------------------------------------------------------------------------
// 21. Pretty-printed JSON (contains newlines and indentation)
// ---------------------------------------------------------------------------

#[test]
fn output_is_pretty_printed() {
    let g = GrammarBuilder::new("pretty")
        .token("a", r"a+")
        .rule("start", vec!["a"])
        .build();

    let generator = NodeTypesGenerator::new(&g);
    let json = generator.generate().unwrap();
    assert!(json.contains('\n'), "JSON should be pretty-printed");
    assert!(json.contains("  "), "JSON should have indentation");
}

// ---------------------------------------------------------------------------
// 22. Supertypes declared on Grammar appear in output
// ---------------------------------------------------------------------------

#[test]
fn supertype_grammar_generates_node_types() {
    let mut g = Grammar::new("super".to_string());

    let num_id = SymbolId(0);
    g.tokens.insert(
        num_id,
        Token {
            name: "number".to_string(),
            pattern: TokenPattern::Regex(r"\d+".to_string()),
            fragile: false,
        },
    );

    let str_id = SymbolId(1);
    g.tokens.insert(
        str_id,
        Token {
            name: "string".to_string(),
            pattern: TokenPattern::Regex(r#""[^"]*""#.to_string()),
            fragile: false,
        },
    );

    // `literal` is a supertype over number | string
    let literal_id = SymbolId(10);
    g.rule_names.insert(literal_id, "literal".to_string());
    g.supertypes.push(literal_id);
    g.add_rule(Rule {
        lhs: literal_id,
        rhs: vec![Symbol::Terminal(num_id)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });

    let nodes = generate_and_parse(&g);
    let lit = find_node(&nodes, "literal");
    assert!(lit.is_some(), "supertype 'literal' should appear in output");
}

// ---------------------------------------------------------------------------
// 23. Adding a token changes the output
// ---------------------------------------------------------------------------

#[test]
fn adding_token_changes_output() {
    let g1 = GrammarBuilder::new("diff")
        .token("a", "a")
        .rule("start", vec!["a"])
        .build();

    let mut g2 = Grammar::new("diff".to_string());
    // Copy g1 manually
    let a_id = SymbolId(0);
    g2.tokens.insert(
        a_id,
        Token {
            name: "a".to_string(),
            pattern: TokenPattern::String("a".to_string()),
            fragile: false,
        },
    );
    let start_id = SymbolId(1);
    g2.rule_names.insert(start_id, "start".to_string());
    g2.add_rule(Rule {
        lhs: start_id,
        rhs: vec![Symbol::Terminal(a_id)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    // Add extra token
    let b_id = SymbolId(2);
    g2.tokens.insert(
        b_id,
        Token {
            name: "b".to_string(),
            pattern: TokenPattern::String("b".to_string()),
            fragile: false,
        },
    );

    let j1 = NodeTypesGenerator::new(&g1).generate().unwrap();
    let j2 = NodeTypesGenerator::new(&g2).generate().unwrap();
    let v1: Value = serde_json::from_str(&j1).unwrap();
    let v2: Value = serde_json::from_str(&j2).unwrap();
    assert_ne!(v1, v2, "adding a token should change the output");
}

// ---------------------------------------------------------------------------
// 24. Rules are named=true
// ---------------------------------------------------------------------------

#[test]
fn rules_are_named() {
    let g = GrammarBuilder::new("named_rules")
        .token("x", r"x+")
        .rule("statement", vec!["x"])
        .rule("expression", vec!["x"])
        .build();

    let nodes = generate_and_parse(&g);
    let stmt = find_node(&nodes, "statement").expect("missing 'statement'");
    assert_eq!(stmt["named"], true, "rules should be named=true");
    let expr = find_node(&nodes, "expression").expect("missing 'expression'");
    assert_eq!(expr["named"], true, "rules should be named=true");
}

// ---------------------------------------------------------------------------
// 25. GrammarBuilder with extras still generates valid output
// ---------------------------------------------------------------------------

#[test]
fn grammar_with_extras_generates_valid_output() {
    let g = GrammarBuilder::new("extras")
        .token("num", r"\d+")
        .token("ws", r"\s+")
        .rule("expr", vec!["num"])
        .extra("ws")
        .build();

    let nodes = generate_and_parse(&g);
    // Should still have at least the rule
    assert!(
        find_node(&nodes, "expr").is_some(),
        "expr should be present"
    );
}

// ---------------------------------------------------------------------------
// 26. GrammarBuilder with precedence rules
// ---------------------------------------------------------------------------

#[test]
fn precedence_rules_produce_valid_output() {
    let g = GrammarBuilder::new("prec")
        .token("num", r"\d+")
        .token("+", "+")
        .token("*", "*")
        .rule("expr", vec!["num"])
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "*", "expr"], 2, Associativity::Left)
        .build();

    let nodes = generate_and_parse(&g);
    let expr = find_node(&nodes, "expr").expect("missing 'expr'");
    assert_eq!(expr["named"], true);

    // Literal tokens should be present
    assert!(find_node(&nodes, "+").is_some(), "'+' token missing");
    assert!(find_node(&nodes, "*").is_some(), "'*' token missing");
}

// ---------------------------------------------------------------------------
// 27. Multiple fields on one rule all appear
// ---------------------------------------------------------------------------

#[test]
fn multiple_fields_all_present() {
    let mut g = Grammar::new("multi_field".to_string());

    let a_id = SymbolId(0);
    g.tokens.insert(
        a_id,
        Token {
            name: "a".to_string(),
            pattern: TokenPattern::Regex(r"a+".to_string()),
            fragile: false,
        },
    );
    let b_id = SymbolId(1);
    g.tokens.insert(
        b_id,
        Token {
            name: "b".to_string(),
            pattern: TokenPattern::Regex(r"b+".to_string()),
            fragile: false,
        },
    );
    let c_id = SymbolId(2);
    g.tokens.insert(
        c_id,
        Token {
            name: "c".to_string(),
            pattern: TokenPattern::Regex(r"c+".to_string()),
            fragile: false,
        },
    );

    let rule_id = SymbolId(10);
    g.rule_names.insert(rule_id, "triple".to_string());

    let f1 = FieldId(0);
    let f2 = FieldId(1);
    let f3 = FieldId(2);
    g.fields.insert(f1, "first".to_string());
    g.fields.insert(f2, "second".to_string());
    g.fields.insert(f3, "third".to_string());

    g.add_rule(Rule {
        lhs: rule_id,
        rhs: vec![
            Symbol::Terminal(a_id),
            Symbol::Terminal(b_id),
            Symbol::Terminal(c_id),
        ],
        precedence: None,
        associativity: None,
        fields: vec![(f1, 0), (f2, 1), (f3, 2)],
        production_id: ProductionId(0),
    });

    let nodes = generate_and_parse(&g);
    let triple = find_node(&nodes, "triple").expect("missing 'triple'");
    let fields = triple.get("fields").expect("should have fields");
    assert!(fields.get("first").is_some(), "missing 'first'");
    assert!(fields.get("second").is_some(), "missing 'second'");
    assert!(fields.get("third").is_some(), "missing 'third'");
}
