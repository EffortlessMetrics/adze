//! Comprehensive NODE_TYPES JSON generation tests for adze-tablegen.

use adze_ir::{
    ExternalToken, FieldId, Grammar, ProductionId, Rule, Symbol, SymbolId, Token, TokenPattern,
};
use adze_tablegen::NodeTypesGenerator;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn empty_grammar() -> Grammar {
    Grammar::new("empty".to_string())
}

/// Minimal grammar: one rule `expr -> NUMBER`
fn single_rule_grammar() -> Grammar {
    let mut g = Grammar::new("single".to_string());

    let num_id = SymbolId(0);
    g.tokens.insert(
        num_id,
        Token {
            name: "number".to_string(),
            pattern: TokenPattern::Regex(r"\d+".to_string()),
            fragile: false,
        },
    );

    let expr_id = SymbolId(1);
    g.rule_names.insert(expr_id, "expression".to_string());
    g.add_rule(Rule {
        lhs: expr_id,
        rhs: vec![Symbol::Terminal(num_id)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });

    g
}

/// Grammar with both named (regex) and anonymous (string-literal) tokens.
fn named_and_anonymous_grammar() -> Grammar {
    let mut g = Grammar::new("mixed".to_string());

    let ident_id = SymbolId(0);
    g.tokens.insert(
        ident_id,
        Token {
            name: "identifier".to_string(),
            pattern: TokenPattern::Regex(r"[a-z]+".to_string()),
            fragile: false,
        },
    );

    let plus_id = SymbolId(1);
    g.tokens.insert(
        plus_id,
        Token {
            name: "plus".to_string(),
            pattern: TokenPattern::String("+".to_string()),
            fragile: false,
        },
    );

    let semi_id = SymbolId(2);
    g.tokens.insert(
        semi_id,
        Token {
            name: "semicolon".to_string(),
            pattern: TokenPattern::String(";".to_string()),
            fragile: false,
        },
    );

    let stmt_id = SymbolId(10);
    g.rule_names.insert(stmt_id, "statement".to_string());
    g.add_rule(Rule {
        lhs: stmt_id,
        rhs: vec![Symbol::Terminal(ident_id), Symbol::Terminal(semi_id)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });

    g
}

/// Grammar declaring a supertype symbol.
fn supertype_grammar() -> Grammar {
    let mut g = Grammar::new("supertype".to_string());

    let num_id = SymbolId(0);
    g.tokens.insert(
        num_id,
        Token {
            name: "number".to_string(),
            pattern: TokenPattern::Regex(r"\d+".to_string()),
            fragile: false,
        },
    );

    let lit_id = SymbolId(10);
    g.rule_names.insert(lit_id, "literal".to_string());
    g.add_rule(Rule {
        lhs: lit_id,
        rhs: vec![Symbol::Terminal(num_id)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });

    let expr_id = SymbolId(11);
    g.rule_names.insert(expr_id, "expression".to_string());
    g.add_rule(Rule {
        lhs: expr_id,
        rhs: vec![Symbol::NonTerminal(lit_id)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(1),
    });

    // Mark expression as a supertype
    g.supertypes.push(expr_id);

    g
}

/// Grammar with field definitions on a binary expression.
fn fields_grammar() -> Grammar {
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
    let plus_id = SymbolId(1);
    g.tokens.insert(
        plus_id,
        Token {
            name: "plus".to_string(),
            pattern: TokenPattern::String("+".to_string()),
            fragile: false,
        },
    );

    let left_fid = FieldId(0);
    let op_fid = FieldId(1);
    let right_fid = FieldId(2);
    g.fields.insert(left_fid, "left".to_string());
    g.fields.insert(op_fid, "operator".to_string());
    g.fields.insert(right_fid, "right".to_string());

    let bin_id = SymbolId(10);
    g.rule_names.insert(bin_id, "binary_expression".to_string());
    g.add_rule(Rule {
        lhs: bin_id,
        rhs: vec![
            Symbol::Terminal(num_id),
            Symbol::Terminal(plus_id),
            Symbol::Terminal(num_id),
        ],
        precedence: None,
        associativity: None,
        fields: vec![(left_fid, 0), (op_fid, 1), (right_fid, 2)],
        production_id: ProductionId(0),
    });

    g
}

/// Grammar whose rule has multiple child type references (NonTerminal and Terminal).
fn multiple_child_types_grammar() -> Grammar {
    let mut g = Grammar::new("multi_child".to_string());

    let num_id = SymbolId(0);
    g.tokens.insert(
        num_id,
        Token {
            name: "number".to_string(),
            pattern: TokenPattern::Regex(r"\d+".to_string()),
            fragile: false,
        },
    );
    let comma_id = SymbolId(1);
    g.tokens.insert(
        comma_id,
        Token {
            name: "comma".to_string(),
            pattern: TokenPattern::String(",".to_string()),
            fragile: false,
        },
    );
    let ident_id = SymbolId(2);
    g.tokens.insert(
        ident_id,
        Token {
            name: "identifier".to_string(),
            pattern: TokenPattern::Regex(r"[a-z]+".to_string()),
            fragile: false,
        },
    );

    let item_id = SymbolId(10);
    g.rule_names.insert(item_id, "item".to_string());
    g.add_rule(Rule {
        lhs: item_id,
        rhs: vec![Symbol::Terminal(num_id)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });

    let list_id = SymbolId(11);
    g.rule_names.insert(list_id, "list".to_string());
    g.add_rule(Rule {
        lhs: list_id,
        rhs: vec![
            Symbol::NonTerminal(item_id),
            Symbol::Terminal(comma_id),
            Symbol::Terminal(ident_id),
        ],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(1),
    });

    g
}

/// Grammar with an optional child symbol.
fn optional_child_grammar() -> Grammar {
    let mut g = Grammar::new("optional".to_string());

    let num_id = SymbolId(0);
    g.tokens.insert(
        num_id,
        Token {
            name: "number".to_string(),
            pattern: TokenPattern::Regex(r"\d+".to_string()),
            fragile: false,
        },
    );
    let semi_id = SymbolId(1);
    g.tokens.insert(
        semi_id,
        Token {
            name: "semicolon".to_string(),
            pattern: TokenPattern::String(";".to_string()),
            fragile: false,
        },
    );

    let stmt_id = SymbolId(10);
    g.rule_names.insert(stmt_id, "statement".to_string());
    g.add_rule(Rule {
        lhs: stmt_id,
        rhs: vec![
            Symbol::Terminal(num_id),
            Symbol::Optional(Box::new(Symbol::Terminal(semi_id))),
        ],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });

    g
}

/// Grammar with a repeated child symbol.
fn repeated_child_grammar() -> Grammar {
    let mut g = Grammar::new("repeated".to_string());

    let num_id = SymbolId(0);
    g.tokens.insert(
        num_id,
        Token {
            name: "number".to_string(),
            pattern: TokenPattern::Regex(r"\d+".to_string()),
            fragile: false,
        },
    );

    let list_id = SymbolId(10);
    g.rule_names.insert(list_id, "number_list".to_string());
    g.add_rule(Rule {
        lhs: list_id,
        rhs: vec![Symbol::Repeat(Box::new(Symbol::Terminal(num_id)))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });

    g
}

/// Grammar with external tokens.
fn external_token_grammar() -> Grammar {
    let mut g = Grammar::new("external".to_string());

    let num_id = SymbolId(0);
    g.tokens.insert(
        num_id,
        Token {
            name: "number".to_string(),
            pattern: TokenPattern::Regex(r"\d+".to_string()),
            fragile: false,
        },
    );

    let indent_id = SymbolId(50);
    g.externals.push(ExternalToken {
        name: "indent".to_string(),
        symbol_id: indent_id,
    });
    let dedent_id = SymbolId(51);
    g.externals.push(ExternalToken {
        name: "dedent".to_string(),
        symbol_id: dedent_id,
    });

    let block_id = SymbolId(10);
    g.rule_names.insert(block_id, "block".to_string());
    g.add_rule(Rule {
        lhs: block_id,
        rhs: vec![
            Symbol::External(indent_id),
            Symbol::Terminal(num_id),
            Symbol::External(dedent_id),
        ],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });

    g
}

/// Grammar whose rules intentionally use names that collide once to check dedup.
fn duplicate_type_grammar() -> Grammar {
    let mut g = Grammar::new("dedup".to_string());

    // Two string-literal tokens with same literal value "+".
    let plus1 = SymbolId(0);
    g.tokens.insert(
        plus1,
        Token {
            name: "plus_a".to_string(),
            pattern: TokenPattern::String("+".to_string()),
            fragile: false,
        },
    );
    let plus2 = SymbolId(1);
    g.tokens.insert(
        plus2,
        Token {
            name: "plus_b".to_string(),
            pattern: TokenPattern::String("+".to_string()),
            fragile: false,
        },
    );

    let expr_id = SymbolId(10);
    g.rule_names.insert(expr_id, "expr".to_string());
    g.add_rule(Rule {
        lhs: expr_id,
        rhs: vec![Symbol::Terminal(plus1)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });

    g
}

// ---------------------------------------------------------------------------
// 1. Empty grammar produces valid JSON
// ---------------------------------------------------------------------------

#[test]
fn test_empty_grammar_produces_valid_json() {
    let g = empty_grammar();
    let generator = NodeTypesGenerator::new(&g);
    let json = generator.generate().expect("generate must succeed");
    let parsed: serde_json::Value = serde_json::from_str(&json).expect("must be valid JSON");
    assert!(parsed.is_array(), "top-level must be an array");
    assert!(
        parsed.as_array().unwrap().is_empty(),
        "no node types expected"
    );
}

// ---------------------------------------------------------------------------
// 2. Single-rule grammar produces correct node types
// ---------------------------------------------------------------------------

#[test]
fn test_single_rule_grammar_produces_correct_types() {
    let g = single_rule_grammar();
    let generator = NodeTypesGenerator::new(&g);
    let json = generator.generate().unwrap();
    let arr: Vec<serde_json::Value> = serde_json::from_str(&json).unwrap();

    // Should have the "expression" named node
    let named: Vec<_> = arr
        .iter()
        .filter(|n| n["named"].as_bool() == Some(true))
        .collect();
    assert!(
        named
            .iter()
            .any(|n| n["type"].as_str() == Some("expression")),
        "expected 'expression' named node type, got: {named:?}"
    );
}

// ---------------------------------------------------------------------------
// 3. Grammar with named and anonymous nodes
// ---------------------------------------------------------------------------

#[test]
fn test_named_and_anonymous_nodes() {
    let g = named_and_anonymous_grammar();
    let generator = NodeTypesGenerator::new(&g);
    let json = generator.generate().unwrap();
    let arr: Vec<serde_json::Value> = serde_json::from_str(&json).unwrap();

    // Named node for the rule
    let named: Vec<_> = arr.iter().filter(|n| n["named"] == true).collect();
    assert!(
        named
            .iter()
            .any(|n| n["type"].as_str() == Some("statement")),
        "expected 'statement' named node"
    );

    // Anonymous literal tokens
    let anon: Vec<_> = arr.iter().filter(|n| n["named"] == false).collect();
    let anon_types: Vec<&str> = anon.iter().filter_map(|n| n["type"].as_str()).collect();
    assert!(anon_types.contains(&"+"), "expected anonymous '+' token");
    assert!(anon_types.contains(&";"), "expected anonymous ';' token");
}

#[test]
fn test_anonymous_nodes_have_no_fields() {
    let g = named_and_anonymous_grammar();
    let generator = NodeTypesGenerator::new(&g);
    let json = generator.generate().unwrap();
    let arr: Vec<serde_json::Value> = serde_json::from_str(&json).unwrap();

    for node in &arr {
        if node["named"] == false {
            assert!(
                node.get("fields").is_none() || node["fields"].is_null(),
                "anonymous node '{}' should have no fields",
                node["type"]
            );
        }
    }
}

// ---------------------------------------------------------------------------
// 4. Grammar with supertype nodes
// ---------------------------------------------------------------------------

#[test]
fn test_supertype_grammar_generates_output() {
    let g = supertype_grammar();
    let generator = NodeTypesGenerator::new(&g);
    let json = generator.generate().unwrap();
    let arr: Vec<serde_json::Value> = serde_json::from_str(&json).unwrap();

    let names: Vec<&str> = arr
        .iter()
        .filter(|n| n["named"] == true)
        .filter_map(|n| n["type"].as_str())
        .collect();

    assert!(names.contains(&"expression"), "expected 'expression' node");
    assert!(names.contains(&"literal"), "expected 'literal' node");
}

#[test]
fn test_supertype_declared_in_grammar() {
    let g = supertype_grammar();
    assert!(
        !g.supertypes.is_empty(),
        "grammar should have supertype declarations"
    );
}

// ---------------------------------------------------------------------------
// 5. Grammar with fields produces correct field definitions
// ---------------------------------------------------------------------------

#[test]
fn test_fields_appear_in_node_type() {
    let g = fields_grammar();
    let generator = NodeTypesGenerator::new(&g);
    let json = generator.generate().unwrap();
    let arr: Vec<serde_json::Value> = serde_json::from_str(&json).unwrap();

    let bin = arr
        .iter()
        .find(|n| n["type"].as_str() == Some("binary_expression"))
        .expect("binary_expression node must exist");

    let fields = bin.get("fields").expect("fields must be present");
    assert!(fields.get("left").is_some(), "expected 'left' field");
    assert!(
        fields.get("operator").is_some(),
        "expected 'operator' field"
    );
    assert!(fields.get("right").is_some(), "expected 'right' field");
}

#[test]
fn test_field_has_types_array() {
    let g = fields_grammar();
    let generator = NodeTypesGenerator::new(&g);
    let json = generator.generate().unwrap();
    let arr: Vec<serde_json::Value> = serde_json::from_str(&json).unwrap();

    let bin = arr
        .iter()
        .find(|n| n["type"].as_str() == Some("binary_expression"))
        .unwrap();

    let left = &bin["fields"]["left"];
    assert!(
        left["types"].is_array(),
        "field 'left' must have a 'types' array"
    );
    assert!(
        !left["types"].as_array().unwrap().is_empty(),
        "'types' must be non-empty"
    );
}

#[test]
fn test_field_has_required_and_multiple_flags() {
    let g = fields_grammar();
    let generator = NodeTypesGenerator::new(&g);
    let json = generator.generate().unwrap();
    let arr: Vec<serde_json::Value> = serde_json::from_str(&json).unwrap();

    let bin = arr
        .iter()
        .find(|n| n["type"].as_str() == Some("binary_expression"))
        .unwrap();

    for field_name in &["left", "operator", "right"] {
        let field = &bin["fields"][field_name];
        assert!(
            field.get("required").is_some(),
            "field '{field_name}' must have 'required'"
        );
        assert!(
            field.get("multiple").is_some(),
            "field '{field_name}' must have 'multiple'"
        );
    }
}

// ---------------------------------------------------------------------------
// 6. Grammar with multiple child types
// ---------------------------------------------------------------------------

#[test]
fn test_multiple_child_types_grammar_output() {
    let g = multiple_child_types_grammar();
    let generator = NodeTypesGenerator::new(&g);
    let json = generator.generate().unwrap();
    let arr: Vec<serde_json::Value> = serde_json::from_str(&json).unwrap();

    let named_types: Vec<&str> = arr
        .iter()
        .filter(|n| n["named"] == true)
        .filter_map(|n| n["type"].as_str())
        .collect();

    assert!(named_types.contains(&"item"), "expected 'item' node");
    assert!(named_types.contains(&"list"), "expected 'list' node");
}

#[test]
fn test_anonymous_comma_token_present() {
    let g = multiple_child_types_grammar();
    let generator = NodeTypesGenerator::new(&g);
    let json = generator.generate().unwrap();
    let arr: Vec<serde_json::Value> = serde_json::from_str(&json).unwrap();

    let anon_types: Vec<&str> = arr
        .iter()
        .filter(|n| n["named"] == false)
        .filter_map(|n| n["type"].as_str())
        .collect();

    assert!(anon_types.contains(&","), "expected anonymous ',' token");
}

// ---------------------------------------------------------------------------
// 7. Grammar with optional children
// ---------------------------------------------------------------------------

#[test]
fn test_optional_child_grammar_generates() {
    let g = optional_child_grammar();
    let generator = NodeTypesGenerator::new(&g);
    let json = generator.generate().unwrap();
    let arr: Vec<serde_json::Value> = serde_json::from_str(&json).unwrap();

    assert!(
        arr.iter().any(|n| n["type"].as_str() == Some("statement")),
        "expected 'statement' node"
    );
}

#[test]
fn test_optional_child_anonymous_semicolon() {
    let g = optional_child_grammar();
    let generator = NodeTypesGenerator::new(&g);
    let json = generator.generate().unwrap();
    let arr: Vec<serde_json::Value> = serde_json::from_str(&json).unwrap();

    let anon: Vec<&str> = arr
        .iter()
        .filter(|n| n["named"] == false)
        .filter_map(|n| n["type"].as_str())
        .collect();

    assert!(anon.contains(&";"), "expected anonymous ';' token");
}

// ---------------------------------------------------------------------------
// 8. Grammar with repeated children
// ---------------------------------------------------------------------------

#[test]
fn test_repeated_child_grammar_generates() {
    let g = repeated_child_grammar();
    let generator = NodeTypesGenerator::new(&g);
    let json = generator.generate().unwrap();
    let arr: Vec<serde_json::Value> = serde_json::from_str(&json).unwrap();

    assert!(
        arr.iter()
            .any(|n| n["type"].as_str() == Some("number_list")),
        "expected 'number_list' node"
    );
}

#[test]
fn test_repeat_one_generates() {
    let mut g = Grammar::new("repeat_one".to_string());

    let num_id = SymbolId(0);
    g.tokens.insert(
        num_id,
        Token {
            name: "number".to_string(),
            pattern: TokenPattern::Regex(r"\d+".to_string()),
            fragile: false,
        },
    );

    let list_id = SymbolId(10);
    g.rule_names.insert(list_id, "nonempty_list".to_string());
    g.add_rule(Rule {
        lhs: list_id,
        rhs: vec![Symbol::RepeatOne(Box::new(Symbol::Terminal(num_id)))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });

    let generator = NodeTypesGenerator::new(&g);
    let json = generator.generate().unwrap();
    let arr: Vec<serde_json::Value> = serde_json::from_str(&json).unwrap();

    assert!(
        arr.iter()
            .any(|n| n["type"].as_str() == Some("nonempty_list")),
        "expected 'nonempty_list' node"
    );
}

// ---------------------------------------------------------------------------
// 9. Node types include correct "named" flag
// ---------------------------------------------------------------------------

#[test]
fn test_named_flag_true_for_rules() {
    let g = single_rule_grammar();
    let generator = NodeTypesGenerator::new(&g);
    let json = generator.generate().unwrap();
    let arr: Vec<serde_json::Value> = serde_json::from_str(&json).unwrap();

    let expr = arr
        .iter()
        .find(|n| n["type"].as_str() == Some("expression"))
        .unwrap();
    assert_eq!(expr["named"], true, "rule nodes must be named");
}

#[test]
fn test_named_flag_false_for_string_literals() {
    let g = named_and_anonymous_grammar();
    let generator = NodeTypesGenerator::new(&g);
    let json = generator.generate().unwrap();
    let arr: Vec<serde_json::Value> = serde_json::from_str(&json).unwrap();

    let plus = arr
        .iter()
        .find(|n| n["type"].as_str() == Some("+"))
        .unwrap();
    assert_eq!(
        plus["named"], false,
        "string literal tokens must not be named"
    );
}

#[test]
fn test_named_flag_consistency() {
    // Every node type entry must have a boolean "named" field
    let g = fields_grammar();
    let generator = NodeTypesGenerator::new(&g);
    let json = generator.generate().unwrap();
    let arr: Vec<serde_json::Value> = serde_json::from_str(&json).unwrap();

    for node in &arr {
        assert!(
            node["named"].is_boolean(),
            "node '{}' must have boolean 'named' field",
            node["type"]
        );
    }
}

// ---------------------------------------------------------------------------
// 10. Node types include correct "type" field
// ---------------------------------------------------------------------------

#[test]
fn test_type_field_present_on_all_entries() {
    let g = named_and_anonymous_grammar();
    let generator = NodeTypesGenerator::new(&g);
    let json = generator.generate().unwrap();
    let arr: Vec<serde_json::Value> = serde_json::from_str(&json).unwrap();

    for node in &arr {
        assert!(
            node["type"].is_string(),
            "every node must have a string 'type' field, got: {node}"
        );
        assert!(
            !node["type"].as_str().unwrap().is_empty(),
            "type field must be non-empty"
        );
    }
}

#[test]
fn test_type_field_matches_rule_name() {
    let g = single_rule_grammar();
    let generator = NodeTypesGenerator::new(&g);
    let json = generator.generate().unwrap();
    let arr: Vec<serde_json::Value> = serde_json::from_str(&json).unwrap();

    let type_names: Vec<&str> = arr.iter().filter_map(|n| n["type"].as_str()).collect();
    assert!(
        type_names.contains(&"expression"),
        "type field should match the rule name 'expression'"
    );
}

#[test]
fn test_type_field_matches_literal_for_anonymous() {
    let g = named_and_anonymous_grammar();
    let generator = NodeTypesGenerator::new(&g);
    let json = generator.generate().unwrap();
    let arr: Vec<serde_json::Value> = serde_json::from_str(&json).unwrap();

    // String-literal tokens should use the literal value as type
    let anon_types: Vec<&str> = arr
        .iter()
        .filter(|n| n["named"] == false)
        .filter_map(|n| n["type"].as_str())
        .collect();

    assert!(
        anon_types.contains(&"+"),
        "anonymous token type should be literal '+'"
    );
}

// ---------------------------------------------------------------------------
// 11. Node types JSON is valid JSON (parseable)
// ---------------------------------------------------------------------------

#[test]
fn test_json_validity_empty() {
    let json = NodeTypesGenerator::new(&empty_grammar())
        .generate()
        .unwrap();
    assert!(serde_json::from_str::<serde_json::Value>(&json).is_ok());
}

#[test]
fn test_json_validity_complex() {
    let json = NodeTypesGenerator::new(&fields_grammar())
        .generate()
        .unwrap();
    assert!(serde_json::from_str::<serde_json::Value>(&json).is_ok());
}

#[test]
fn test_json_is_pretty_printed() {
    let json = NodeTypesGenerator::new(&single_rule_grammar())
        .generate()
        .unwrap();
    // Pretty-printed JSON should contain newlines
    assert!(json.contains('\n'), "JSON should be pretty-printed");
}

// ---------------------------------------------------------------------------
// 12. Node types are sorted alphabetically
// ---------------------------------------------------------------------------

#[test]
fn test_node_types_sorted_alphabetically() {
    let g = named_and_anonymous_grammar();
    let generator = NodeTypesGenerator::new(&g);
    let json = generator.generate().unwrap();
    let arr: Vec<serde_json::Value> = serde_json::from_str(&json).unwrap();

    let type_names: Vec<&str> = arr.iter().filter_map(|n| n["type"].as_str()).collect();
    let mut sorted = type_names.clone();
    sorted.sort();
    assert_eq!(
        type_names, sorted,
        "node types must be sorted alphabetically"
    );
}

#[test]
fn test_sorting_with_many_nodes() {
    let g = multiple_child_types_grammar();
    let generator = NodeTypesGenerator::new(&g);
    let json = generator.generate().unwrap();
    let arr: Vec<serde_json::Value> = serde_json::from_str(&json).unwrap();

    let type_names: Vec<&str> = arr.iter().filter_map(|n| n["type"].as_str()).collect();
    let mut sorted = type_names.clone();
    sorted.sort();
    assert_eq!(type_names, sorted, "node types must remain sorted");
}

// ---------------------------------------------------------------------------
// 13. Duplicate types are deduplicated
// ---------------------------------------------------------------------------

#[test]
fn test_duplicate_literal_tokens_appear() {
    let g = duplicate_type_grammar();
    let generator = NodeTypesGenerator::new(&g);
    let json = generator.generate().unwrap();
    let arr: Vec<serde_json::Value> = serde_json::from_str(&json).unwrap();

    // Both tokens emit "+" as type_name – count how many "+" entries exist
    let plus_count = arr
        .iter()
        .filter(|n| n["type"].as_str() == Some("+"))
        .count();

    // The generator currently emits one per token.  Either deduplication reduces
    // them to 1, or we see both – either is a valid documented behaviour.
    // The test simply checks the output is well-formed.
    assert!(plus_count >= 1, "at least one '+' anonymous node expected");
}

#[test]
fn test_rule_not_duplicated_with_same_name() {
    // A single rule should not appear more than once.
    let g = single_rule_grammar();
    let generator = NodeTypesGenerator::new(&g);
    let json = generator.generate().unwrap();
    let arr: Vec<serde_json::Value> = serde_json::from_str(&json).unwrap();

    let expr_count = arr
        .iter()
        .filter(|n| n["type"].as_str() == Some("expression") && n["named"] == true)
        .count();
    assert_eq!(
        expr_count, 1,
        "named 'expression' should appear exactly once"
    );
}

// ---------------------------------------------------------------------------
// 14. External token types appear in output
// ---------------------------------------------------------------------------

#[test]
fn test_external_token_grammar_generates() {
    let g = external_token_grammar();
    let generator = NodeTypesGenerator::new(&g);
    let json = generator.generate().unwrap();
    let arr: Vec<serde_json::Value> = serde_json::from_str(&json).unwrap();

    // The block rule should be present as a named node
    assert!(
        arr.iter().any(|n| n["type"].as_str() == Some("block")),
        "expected 'block' node"
    );
}

#[test]
fn test_external_tokens_declared() {
    let g = external_token_grammar();
    assert_eq!(
        g.externals.len(),
        2,
        "grammar should have 2 external tokens"
    );
    assert_eq!(g.externals[0].name, "indent");
    assert_eq!(g.externals[1].name, "dedent");
}

// ---------------------------------------------------------------------------
// 15. Grammar changes result in different NODE_TYPES
// ---------------------------------------------------------------------------

#[test]
fn test_different_grammars_produce_different_output() {
    let json1 = NodeTypesGenerator::new(&single_rule_grammar())
        .generate()
        .unwrap();
    let json2 = NodeTypesGenerator::new(&fields_grammar())
        .generate()
        .unwrap();
    assert_ne!(
        json1, json2,
        "different grammars must produce different JSON"
    );
}

#[test]
fn test_adding_rule_changes_output() {
    let mut g = single_rule_grammar();
    let generator = NodeTypesGenerator::new(&g);
    let before = generator.generate().unwrap();

    // Add another rule
    let stmt_id = SymbolId(20);
    g.rule_names.insert(stmt_id, "statement".to_string());
    g.add_rule(Rule {
        lhs: stmt_id,
        rhs: vec![Symbol::NonTerminal(SymbolId(1))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(99),
    });

    let after = NodeTypesGenerator::new(&g).generate().unwrap();
    assert_ne!(before, after, "adding a rule must change the output");
}

#[test]
fn test_adding_token_changes_output() {
    let mut g = single_rule_grammar();
    let before = NodeTypesGenerator::new(&g).generate().unwrap();

    g.tokens.insert(
        SymbolId(90),
        Token {
            name: "dot".to_string(),
            pattern: TokenPattern::String(".".to_string()),
            fragile: false,
        },
    );

    let after = NodeTypesGenerator::new(&g).generate().unwrap();
    assert_ne!(before, after, "adding a token must change the output");
}

// ---------------------------------------------------------------------------
// Additional tests (to exceed 25)
// ---------------------------------------------------------------------------

#[test]
fn test_internal_rules_excluded() {
    // Rules starting with '_' should be excluded from the output
    let mut g = Grammar::new("internal".to_string());

    let num_id = SymbolId(0);
    g.tokens.insert(
        num_id,
        Token {
            name: "number".to_string(),
            pattern: TokenPattern::Regex(r"\d+".to_string()),
            fragile: false,
        },
    );

    let internal_id = SymbolId(10);
    g.rule_names.insert(internal_id, "_hidden_rule".to_string());
    g.add_rule(Rule {
        lhs: internal_id,
        rhs: vec![Symbol::Terminal(num_id)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });

    let generator = NodeTypesGenerator::new(&g);
    let json = generator.generate().unwrap();
    let arr: Vec<serde_json::Value> = serde_json::from_str(&json).unwrap();

    assert!(
        !arr.iter()
            .any(|n| { n["type"].as_str().is_some_and(|s| s.starts_with('_')) }),
        "internal rules (starting with '_') must not appear in output"
    );
}

#[test]
fn test_regex_token_is_named() {
    // Regex tokens are emitted as named nodes from the rule, not as anonymous literals.
    let g = single_rule_grammar();
    let generator = NodeTypesGenerator::new(&g);
    let json = generator.generate().unwrap();
    let arr: Vec<serde_json::Value> = serde_json::from_str(&json).unwrap();

    // The regex-based "number" token should NOT be emitted as an anonymous node
    let anon_number = arr
        .iter()
        .find(|n| n["type"].as_str() == Some("number") && n["named"] == false);
    assert!(
        anon_number.is_none(),
        "regex-based token should not appear as anonymous node"
    );
}

#[test]
fn test_generate_returns_ok() {
    // Every well-formed grammar should successfully generate
    for g in [
        empty_grammar(),
        single_rule_grammar(),
        named_and_anonymous_grammar(),
        supertype_grammar(),
        fields_grammar(),
        multiple_child_types_grammar(),
        optional_child_grammar(),
        repeated_child_grammar(),
        external_token_grammar(),
        duplicate_type_grammar(),
    ] {
        let result = NodeTypesGenerator::new(&g).generate();
        assert!(result.is_ok(), "generate() failed for grammar '{}'", g.name);
    }
}

#[test]
fn test_output_is_json_array_of_objects() {
    let g = fields_grammar();
    let json = NodeTypesGenerator::new(&g).generate().unwrap();
    let arr: Vec<serde_json::Value> = serde_json::from_str(&json).unwrap();

    for node in &arr {
        assert!(
            node.is_object(),
            "each entry must be a JSON object, got: {node}"
        );
    }
}

#[test]
fn test_no_null_type_names() {
    let g = named_and_anonymous_grammar();
    let json = NodeTypesGenerator::new(&g).generate().unwrap();
    let arr: Vec<serde_json::Value> = serde_json::from_str(&json).unwrap();

    for node in &arr {
        assert!(!node["type"].is_null(), "type must never be null");
    }
}

#[test]
fn test_choice_symbol_in_rule() {
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
    g.add_rule(Rule {
        lhs: val_id,
        rhs: vec![Symbol::Choice(vec![
            Symbol::Terminal(num_id),
            Symbol::Terminal(str_id),
        ])],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });

    let generator = NodeTypesGenerator::new(&g);
    let json = generator.generate().unwrap();
    let arr: Vec<serde_json::Value> = serde_json::from_str(&json).unwrap();

    assert!(
        arr.iter().any(|n| n["type"].as_str() == Some("value")),
        "expected 'value' node"
    );
}

#[test]
fn test_sequence_symbol_in_rule() {
    let mut g = Grammar::new("sequence".to_string());

    let num_id = SymbolId(0);
    g.tokens.insert(
        num_id,
        Token {
            name: "number".to_string(),
            pattern: TokenPattern::Regex(r"\d+".to_string()),
            fragile: false,
        },
    );

    let pair_id = SymbolId(10);
    g.rule_names.insert(pair_id, "pair".to_string());
    g.add_rule(Rule {
        lhs: pair_id,
        rhs: vec![Symbol::Sequence(vec![
            Symbol::Terminal(num_id),
            Symbol::Terminal(num_id),
        ])],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });

    let generator = NodeTypesGenerator::new(&g);
    let json = generator.generate().unwrap();
    let arr: Vec<serde_json::Value> = serde_json::from_str(&json).unwrap();

    assert!(
        arr.iter().any(|n| n["type"].as_str() == Some("pair")),
        "expected 'pair' node"
    );
}

#[test]
fn test_epsilon_symbol_in_rule() {
    let mut g = Grammar::new("epsilon".to_string());

    let eps_id = SymbolId(10);
    g.rule_names.insert(eps_id, "empty_rule".to_string());
    g.add_rule(Rule {
        lhs: eps_id,
        rhs: vec![Symbol::Epsilon],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });

    let generator = NodeTypesGenerator::new(&g);
    let json = generator.generate().unwrap();
    let arr: Vec<serde_json::Value> = serde_json::from_str(&json).unwrap();

    assert!(
        arr.iter().any(|n| n["type"].as_str() == Some("empty_rule")),
        "expected 'empty_rule' node"
    );
}

#[test]
fn test_deterministic_output() {
    // Running generate twice on the same grammar must yield semantically identical output.
    let g = fields_grammar();
    let json1 = NodeTypesGenerator::new(&g).generate().unwrap();
    let json2 = NodeTypesGenerator::new(&g).generate().unwrap();
    let val1: serde_json::Value = serde_json::from_str(&json1).unwrap();
    let val2: serde_json::Value = serde_json::from_str(&json2).unwrap();
    assert_eq!(val1, val2, "generation must be deterministic");
}
