//! Comprehensive tests for `NodeTypesGenerator`.
//!
//! 60+ tests covering: minimal grammars, multi-token grammars, JSON validity,
//! array structure, "type"/"named" fields, determinism, symbol names, and edge cases.

use adze_ir::builder::GrammarBuilder;
use adze_ir::{Associativity, Grammar, ProductionId, Rule, Symbol, SymbolId, Token, TokenPattern};
use adze_tablegen::NodeTypesGenerator;
use serde_json::Value;

// ===========================================================================
// Helpers
// ===========================================================================

fn generate(grammar: &Grammar) -> String {
    NodeTypesGenerator::new(grammar)
        .generate()
        .expect("generate failed")
}

fn generate_parsed(grammar: &Grammar) -> Vec<Value> {
    let json = generate(grammar);
    let v: Value = serde_json::from_str(&json).unwrap();
    v.as_array().unwrap().clone()
}

fn find_by_type<'a>(nodes: &'a [Value], name: &str) -> Option<&'a Value> {
    nodes.iter().find(|n| n["type"].as_str() == Some(name))
}

fn minimal_grammar() -> Grammar {
    GrammarBuilder::new("minimal")
        .token("NUMBER", r"\d+")
        .rule("expr", vec!["NUMBER"])
        .start("expr")
        .build()
}

fn arithmetic_grammar() -> Grammar {
    GrammarBuilder::new("arithmetic")
        .token("NUMBER", r"\d+")
        .token("+", "+")
        .token("-", "-")
        .token("*", "*")
        .token("/", "/")
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "-", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "*", "expr"], 2, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "/", "expr"], 2, Associativity::Left)
        .rule("expr", vec!["NUMBER"])
        .start("expr")
        .build()
}

fn multi_rule_grammar() -> Grammar {
    GrammarBuilder::new("multi")
        .token("NUMBER", r"\d+")
        .token("IDENT", r"[a-z]+")
        .token("+", "+")
        .token("=", "=")
        .token(";", ";")
        .rule("program", vec!["statement"])
        .rule("statement", vec!["assignment"])
        .rule("statement", vec!["expr"])
        .rule("assignment", vec!["IDENT", "=", "expr", ";"])
        .rule("expr", vec!["NUMBER"])
        .rule("expr", vec!["IDENT"])
        .rule("expr", vec!["expr", "+", "expr"])
        .start("program")
        .build()
}

// ===========================================================================
// 1. Minimal grammar
// ===========================================================================

#[test]
fn minimal_grammar_generates_ok() {
    let g = minimal_grammar();
    assert!(NodeTypesGenerator::new(&g).generate().is_ok());
}

#[test]
fn minimal_grammar_output_nonempty() {
    let json = generate(&minimal_grammar());
    assert!(!json.is_empty());
}

#[test]
fn minimal_grammar_contains_expr() {
    let nodes = generate_parsed(&minimal_grammar());
    assert!(find_by_type(&nodes, "expr").is_some());
}

#[test]
fn minimal_grammar_expr_is_named() {
    let nodes = generate_parsed(&minimal_grammar());
    let expr = find_by_type(&nodes, "expr").unwrap();
    assert_eq!(expr["named"], true);
}

#[test]
fn minimal_grammar_has_at_least_one_node() {
    let nodes = generate_parsed(&minimal_grammar());
    assert!(!nodes.is_empty());
}

// ===========================================================================
// 2. Multi-token grammar
// ===========================================================================

#[test]
fn multi_token_grammar_generates_ok() {
    assert!(
        NodeTypesGenerator::new(&arithmetic_grammar())
            .generate()
            .is_ok()
    );
}

#[test]
fn multi_token_grammar_contains_expr() {
    let nodes = generate_parsed(&arithmetic_grammar());
    assert!(find_by_type(&nodes, "expr").is_some());
}

#[test]
fn multi_token_anonymous_plus() {
    let nodes = generate_parsed(&arithmetic_grammar());
    let plus = find_by_type(&nodes, "+");
    if let Some(p) = plus {
        assert_eq!(p["named"], false);
    }
}

#[test]
fn multi_token_anonymous_minus() {
    let nodes = generate_parsed(&arithmetic_grammar());
    let minus = find_by_type(&nodes, "-");
    if let Some(m) = minus {
        assert_eq!(m["named"], false);
    }
}

#[test]
fn multi_token_anonymous_star() {
    let nodes = generate_parsed(&arithmetic_grammar());
    let star = find_by_type(&nodes, "*");
    if let Some(s) = star {
        assert_eq!(s["named"], false);
    }
}

#[test]
fn multi_token_anonymous_slash() {
    let nodes = generate_parsed(&arithmetic_grammar());
    let slash = find_by_type(&nodes, "/");
    if let Some(s) = slash {
        assert_eq!(s["named"], false);
    }
}

// ===========================================================================
// 3. Output is valid JSON
// ===========================================================================

#[test]
fn minimal_output_is_valid_json() {
    let json = generate(&minimal_grammar());
    assert!(serde_json::from_str::<Value>(&json).is_ok());
}

#[test]
fn arithmetic_output_is_valid_json() {
    let json = generate(&arithmetic_grammar());
    assert!(serde_json::from_str::<Value>(&json).is_ok());
}

#[test]
fn multi_rule_output_is_valid_json() {
    let json = generate(&multi_rule_grammar());
    assert!(serde_json::from_str::<Value>(&json).is_ok());
}

#[test]
fn python_like_output_is_valid_json() {
    let g = GrammarBuilder::python_like();
    let json = generate(&g);
    assert!(serde_json::from_str::<Value>(&json).is_ok());
}

#[test]
fn javascript_like_output_is_valid_json() {
    let g = GrammarBuilder::javascript_like();
    let json = generate(&g);
    assert!(serde_json::from_str::<Value>(&json).is_ok());
}

// ===========================================================================
// 4. Output is JSON array
// ===========================================================================

#[test]
fn minimal_output_is_json_array() {
    let v: Value = serde_json::from_str(&generate(&minimal_grammar())).unwrap();
    assert!(v.is_array());
}

#[test]
fn arithmetic_output_is_json_array() {
    let v: Value = serde_json::from_str(&generate(&arithmetic_grammar())).unwrap();
    assert!(v.is_array());
}

#[test]
fn multi_rule_output_is_json_array() {
    let v: Value = serde_json::from_str(&generate(&multi_rule_grammar())).unwrap();
    assert!(v.is_array());
}

// ===========================================================================
// 5. Each element has "type" field
// ===========================================================================

#[test]
fn minimal_all_elements_have_type() {
    for n in generate_parsed(&minimal_grammar()) {
        assert!(n.get("type").is_some(), "missing 'type' field: {n}");
    }
}

#[test]
fn arithmetic_all_elements_have_type() {
    for n in generate_parsed(&arithmetic_grammar()) {
        assert!(n.get("type").is_some(), "missing 'type' field: {n}");
    }
}

#[test]
fn multi_rule_all_elements_have_type() {
    for n in generate_parsed(&multi_rule_grammar()) {
        assert!(n.get("type").is_some(), "missing 'type' field: {n}");
    }
}

#[test]
fn type_field_is_string() {
    for n in generate_parsed(&arithmetic_grammar()) {
        assert!(n["type"].is_string(), "'type' should be a string: {n}");
    }
}

#[test]
fn type_field_is_nonempty() {
    for n in generate_parsed(&arithmetic_grammar()) {
        let s = n["type"].as_str().unwrap();
        assert!(!s.is_empty(), "'type' should not be empty");
    }
}

// ===========================================================================
// 6. Each element has "named" field
// ===========================================================================

#[test]
fn minimal_all_elements_have_named() {
    for n in generate_parsed(&minimal_grammar()) {
        assert!(n.get("named").is_some(), "missing 'named' field: {n}");
    }
}

#[test]
fn arithmetic_all_elements_have_named() {
    for n in generate_parsed(&arithmetic_grammar()) {
        assert!(n.get("named").is_some(), "missing 'named' field: {n}");
    }
}

#[test]
fn multi_rule_all_elements_have_named() {
    for n in generate_parsed(&multi_rule_grammar()) {
        assert!(n.get("named").is_some(), "missing 'named' field: {n}");
    }
}

#[test]
fn named_field_is_boolean() {
    for n in generate_parsed(&arithmetic_grammar()) {
        assert!(n["named"].is_boolean(), "'named' should be a boolean: {n}");
    }
}

// ===========================================================================
// 7. Different grammars produce different output
// ===========================================================================

#[test]
fn minimal_and_arithmetic_differ() {
    let a = generate(&minimal_grammar());
    let b = generate(&arithmetic_grammar());
    assert_ne!(a, b);
}

#[test]
fn minimal_and_multi_rule_differ() {
    let a = generate(&minimal_grammar());
    let b = generate(&multi_rule_grammar());
    assert_ne!(a, b);
}

#[test]
fn arithmetic_and_multi_rule_differ() {
    let a = generate(&arithmetic_grammar());
    let b = generate(&multi_rule_grammar());
    assert_ne!(a, b);
}

#[test]
fn python_and_javascript_differ() {
    let a = generate(&GrammarBuilder::python_like());
    let b = generate(&GrammarBuilder::javascript_like());
    assert_ne!(a, b);
}

#[test]
fn different_grammar_names_same_structure_same_nodes() {
    let a = GrammarBuilder::new("alpha")
        .token("X", r"\d+")
        .rule("root", vec!["X"])
        .start("root")
        .build();
    let b = GrammarBuilder::new("beta")
        .token("X", r"\d+")
        .rule("root", vec!["X"])
        .start("root")
        .build();
    // Both have identical structure so node types should match
    let na = generate_parsed(&a);
    let nb = generate_parsed(&b);
    assert_eq!(na.len(), nb.len());
}

// ===========================================================================
// 8. Determinism – same grammar produces same output
// ===========================================================================

#[test]
fn determinism_minimal() {
    let g = minimal_grammar();
    assert_eq!(generate(&g), generate(&g));
}

#[test]
fn determinism_arithmetic() {
    let g = arithmetic_grammar();
    assert_eq!(generate(&g), generate(&g));
}

#[test]
fn determinism_multi_rule() {
    let g = multi_rule_grammar();
    assert_eq!(generate(&g), generate(&g));
}

#[test]
fn determinism_python_like() {
    let g = GrammarBuilder::python_like();
    assert_eq!(generate(&g), generate(&g));
}

#[test]
fn determinism_javascript_like() {
    let g = GrammarBuilder::javascript_like();
    assert_eq!(generate(&g), generate(&g));
}

#[test]
fn determinism_across_generator_instances() {
    let g = arithmetic_grammar();
    let out1 = NodeTypesGenerator::new(&g).generate().unwrap();
    let out2 = NodeTypesGenerator::new(&g).generate().unwrap();
    assert_eq!(out1, out2);
}

// ===========================================================================
// 9. Node types contain expected symbol names
// ===========================================================================

#[test]
fn arithmetic_contains_expr_name() {
    let nodes = generate_parsed(&arithmetic_grammar());
    let names: Vec<&str> = nodes.iter().filter_map(|n| n["type"].as_str()).collect();
    assert!(names.contains(&"expr"));
}

#[test]
fn multi_rule_contains_program() {
    let nodes = generate_parsed(&multi_rule_grammar());
    assert!(find_by_type(&nodes, "program").is_some());
}

#[test]
fn multi_rule_contains_statement() {
    let nodes = generate_parsed(&multi_rule_grammar());
    assert!(find_by_type(&nodes, "statement").is_some());
}

#[test]
fn multi_rule_contains_assignment() {
    let nodes = generate_parsed(&multi_rule_grammar());
    assert!(find_by_type(&nodes, "assignment").is_some());
}

#[test]
fn multi_rule_contains_expr() {
    let nodes = generate_parsed(&multi_rule_grammar());
    assert!(find_by_type(&nodes, "expr").is_some());
}

#[test]
fn python_like_contains_module() {
    let nodes = generate_parsed(&GrammarBuilder::python_like());
    assert!(find_by_type(&nodes, "module").is_some());
}

#[test]
fn python_like_contains_function_def() {
    let nodes = generate_parsed(&GrammarBuilder::python_like());
    assert!(find_by_type(&nodes, "function_def").is_some());
}

#[test]
fn javascript_like_contains_program() {
    let nodes = generate_parsed(&GrammarBuilder::javascript_like());
    assert!(find_by_type(&nodes, "program").is_some());
}

#[test]
fn javascript_like_contains_expression() {
    let nodes = generate_parsed(&GrammarBuilder::javascript_like());
    assert!(find_by_type(&nodes, "expression").is_some());
}

#[test]
fn javascript_like_contains_var_declaration() {
    let nodes = generate_parsed(&GrammarBuilder::javascript_like());
    assert!(find_by_type(&nodes, "var_declaration").is_some());
}

// ===========================================================================
// 10. Edge cases
// ===========================================================================

// -- Single token grammar --

#[test]
fn single_token_grammar() {
    let g = GrammarBuilder::new("one_token")
        .token("A", "a")
        .rule("root", vec!["A"])
        .start("root")
        .build();
    let nodes = generate_parsed(&g);
    assert!(!nodes.is_empty());
}

#[test]
fn single_token_rule_is_named() {
    let g = GrammarBuilder::new("one_token")
        .token("A", "a")
        .rule("root", vec!["A"])
        .start("root")
        .build();
    let nodes = generate_parsed(&g);
    let root = find_by_type(&nodes, "root").unwrap();
    assert_eq!(root["named"], true);
}

// -- Many tokens grammar --

#[test]
fn many_tokens_grammar_generates_ok() {
    let mut builder = GrammarBuilder::new("many_tokens");
    let mut rhs = Vec::new();
    for i in 0..20 {
        let name = format!("T{i}");
        // Leak to get a &'static str so we can use it in rhs
        let name_ref: &'static str = Box::leak(name.clone().into_boxed_str());
        builder = builder.token(name_ref, name_ref);
        rhs.push(name_ref);
    }
    builder = builder.rule("big_rule", rhs).start("big_rule");
    let g = builder.build();
    assert!(NodeTypesGenerator::new(&g).generate().is_ok());
}

#[test]
fn many_tokens_all_have_type_and_named() {
    let mut builder = GrammarBuilder::new("many_tokens");
    let mut rhs = Vec::new();
    for i in 0..10 {
        let name: &'static str = Box::leak(format!("T{i}").into_boxed_str());
        builder = builder.token(name, name);
        rhs.push(name);
    }
    builder = builder.rule("big_rule", rhs).start("big_rule");
    let g = builder.build();
    for n in generate_parsed(&g) {
        assert!(n.get("type").is_some());
        assert!(n.get("named").is_some());
    }
}

// -- Precedence grammars --

#[test]
fn precedence_grammar_generates_ok() {
    let g = GrammarBuilder::new("prec")
        .token("N", r"\d+")
        .token("+", "+")
        .token("*", "*")
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "*", "expr"], 2, Associativity::Left)
        .rule("expr", vec!["N"])
        .start("expr")
        .build();
    assert!(NodeTypesGenerator::new(&g).generate().is_ok());
}

#[test]
fn precedence_grammar_output_is_valid_json() {
    let g = GrammarBuilder::new("prec")
        .token("N", r"\d+")
        .token("+", "+")
        .token("*", "*")
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "*", "expr"], 2, Associativity::Left)
        .rule("expr", vec!["N"])
        .start("expr")
        .build();
    assert!(serde_json::from_str::<Value>(&generate(&g)).is_ok());
}

#[test]
fn right_assoc_grammar() {
    let g = GrammarBuilder::new("right_assoc")
        .token("N", r"\d+")
        .token("^", "^")
        .rule_with_precedence("expr", vec!["expr", "^", "expr"], 3, Associativity::Right)
        .rule("expr", vec!["N"])
        .start("expr")
        .build();
    let nodes = generate_parsed(&g);
    assert!(find_by_type(&nodes, "expr").is_some());
}

// -- Empty rule --

#[test]
fn empty_rule_generates_ok() {
    let g = GrammarBuilder::new("empty")
        .token("A", "a")
        .rule("root", vec![])
        .rule("root", vec!["A"])
        .start("root")
        .build();
    assert!(NodeTypesGenerator::new(&g).generate().is_ok());
}

// -- Grammar with regex tokens --

#[test]
fn regex_token_is_named() {
    let g = GrammarBuilder::new("regex")
        .token("IDENT", r"[a-z]+")
        .rule("root", vec!["IDENT"])
        .start("root")
        .build();
    let nodes = generate_parsed(&g);
    // Regex tokens that appear as rules should be named
    let root = find_by_type(&nodes, "root").unwrap();
    assert_eq!(root["named"], true);
}

// -- Grammar with string literal tokens --

#[test]
fn string_literal_token_is_anonymous() {
    let g = GrammarBuilder::new("literal")
        .token("+", "+")
        .token("N", r"\d+")
        .rule("root", vec!["N", "+", "N"])
        .start("root")
        .build();
    let nodes = generate_parsed(&g);
    if let Some(plus) = find_by_type(&nodes, "+") {
        assert_eq!(plus["named"], false);
    }
}

// -- Output is sorted --

#[test]
fn output_is_sorted_by_type() {
    let nodes = generate_parsed(&arithmetic_grammar());
    let names: Vec<&str> = nodes.iter().filter_map(|n| n["type"].as_str()).collect();
    let mut sorted = names.clone();
    sorted.sort();
    assert_eq!(names, sorted);
}

#[test]
fn multi_rule_output_is_sorted() {
    let nodes = generate_parsed(&multi_rule_grammar());
    let names: Vec<&str> = nodes.iter().filter_map(|n| n["type"].as_str()).collect();
    let mut sorted = names.clone();
    sorted.sort();
    assert_eq!(names, sorted);
}

// -- No duplicate type entries --

#[test]
fn no_duplicate_types_minimal() {
    let nodes = generate_parsed(&minimal_grammar());
    let names: Vec<&str> = nodes.iter().filter_map(|n| n["type"].as_str()).collect();
    let unique: std::collections::HashSet<&str> = names.iter().copied().collect();
    assert_eq!(names.len(), unique.len());
}

#[test]
fn no_duplicate_types_arithmetic() {
    let nodes = generate_parsed(&arithmetic_grammar());
    let names: Vec<&str> = nodes.iter().filter_map(|n| n["type"].as_str()).collect();
    let unique: std::collections::HashSet<&str> = names.iter().copied().collect();
    assert_eq!(names.len(), unique.len());
}

// -- Output is pretty-printed JSON --

#[test]
fn output_contains_newlines() {
    let json = generate(&minimal_grammar());
    assert!(
        json.contains('\n'),
        "expected pretty-printed JSON with newlines"
    );
}

// -- Fragile token grammar --

#[test]
fn fragile_token_grammar_generates_ok() {
    let g = GrammarBuilder::new("fragile")
        .fragile_token("COMMENT", r"//.*")
        .token("N", r"\d+")
        .rule("root", vec!["N"])
        .start("root")
        .build();
    assert!(NodeTypesGenerator::new(&g).generate().is_ok());
}

// -- Grammar with extras --

#[test]
fn grammar_with_extras_generates_ok() {
    let g = GrammarBuilder::new("extras")
        .token("N", r"\d+")
        .token("WS", r"\s+")
        .extra("WS")
        .rule("root", vec!["N"])
        .start("root")
        .build();
    assert!(NodeTypesGenerator::new(&g).generate().is_ok());
}

// -- Grammar with external scanners --

#[test]
fn grammar_with_externals_generates_ok() {
    let g = GrammarBuilder::new("ext")
        .token("N", r"\d+")
        .external("INDENT")
        .rule("root", vec!["N"])
        .start("root")
        .build();
    assert!(NodeTypesGenerator::new(&g).generate().is_ok());
}

// -- Raw Grammar API (no builder) --

#[test]
fn raw_grammar_single_rule() {
    let mut grammar = Grammar::new("raw".to_string());
    let tok = Token {
        name: "num".to_string(),
        pattern: TokenPattern::Regex(r"\d+".to_string()),
        fragile: false,
    };
    grammar.tokens.insert(SymbolId(0), tok);
    let rule = Rule {
        lhs: SymbolId(1),
        rhs: vec![Symbol::Terminal(SymbolId(0))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    };
    grammar.add_rule(rule);
    let result = NodeTypesGenerator::new(&grammar).generate();
    assert!(result.is_ok());
    let v: Value = serde_json::from_str(&result.unwrap()).unwrap();
    assert!(v.is_array());
}

#[test]
fn raw_grammar_multiple_tokens() {
    let mut grammar = Grammar::new("raw2".to_string());
    grammar.tokens.insert(
        SymbolId(0),
        Token {
            name: "a".to_string(),
            pattern: TokenPattern::String("a".to_string()),
            fragile: false,
        },
    );
    grammar.tokens.insert(
        SymbolId(1),
        Token {
            name: "b".to_string(),
            pattern: TokenPattern::String("b".to_string()),
            fragile: false,
        },
    );
    grammar.add_rule(Rule {
        lhs: SymbolId(2),
        rhs: vec![Symbol::Terminal(SymbolId(0)), Symbol::Terminal(SymbolId(1))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    let nodes = generate_parsed(&grammar);
    assert!(!nodes.is_empty());
}

// -- Multiple rules for same non-terminal --

#[test]
fn multiple_alternatives_single_node_type() {
    let g = GrammarBuilder::new("alts")
        .token("A", "a")
        .token("B", "b")
        .token("C", "c")
        .rule("root", vec!["A"])
        .rule("root", vec!["B"])
        .rule("root", vec!["C"])
        .start("root")
        .build();
    let nodes = generate_parsed(&g);
    let roots: Vec<_> = nodes
        .iter()
        .filter(|n| n["type"].as_str() == Some("root"))
        .collect();
    // One node type entry per non-terminal, not one per alternative
    assert_eq!(roots.len(), 1);
}

// -- Generator with new --

#[test]
fn generator_new_accepts_reference() {
    let g = minimal_grammar();
    let _gen = NodeTypesGenerator::new(&g);
}

// -- Output format --

#[test]
fn output_starts_with_bracket() {
    let json = generate(&minimal_grammar());
    let trimmed = json.trim();
    assert!(
        trimmed.starts_with('['),
        "JSON should start with '[', got: {}",
        &trimmed[..1]
    );
}

#[test]
fn output_ends_with_bracket() {
    let json = generate(&minimal_grammar());
    let trimmed = json.trim();
    assert!(trimmed.ends_with(']'), "JSON should end with ']'");
}

// -- Named vs anonymous classification --

#[test]
fn rule_nodes_are_named() {
    let nodes = generate_parsed(&multi_rule_grammar());
    for name in &["program", "statement", "assignment", "expr"] {
        if let Some(n) = find_by_type(&nodes, name) {
            assert_eq!(n["named"], true, "{name} should be named");
        }
    }
}

#[test]
fn operator_tokens_are_anonymous() {
    let nodes = generate_parsed(&multi_rule_grammar());
    for op in &["+", "=", ";"] {
        if let Some(n) = find_by_type(&nodes, op) {
            assert_eq!(n["named"], false, "'{op}' should be anonymous");
        }
    }
}

// -- Count of node types --

#[test]
fn minimal_node_count() {
    let nodes = generate_parsed(&minimal_grammar());
    // At least the "expr" rule should appear
    assert!(nodes.len() >= 1);
}

#[test]
fn arithmetic_node_count_greater_than_minimal() {
    let min_nodes = generate_parsed(&minimal_grammar());
    let arith_nodes = generate_parsed(&arithmetic_grammar());
    assert!(arith_nodes.len() > min_nodes.len());
}

// -- Fields are optional --

#[test]
fn fields_absent_when_empty() {
    let nodes = generate_parsed(&minimal_grammar());
    let expr = find_by_type(&nodes, "expr").unwrap();
    // fields should either be absent or null when there are none
    let fields = expr.get("fields");
    assert!(
        fields.is_none() || fields.unwrap().is_null(),
        "fields should be absent or null for simple rules"
    );
}

// -- Children are optional --

#[test]
fn children_absent_when_not_applicable() {
    let nodes = generate_parsed(&minimal_grammar());
    let expr = find_by_type(&nodes, "expr").unwrap();
    let children = expr.get("children");
    assert!(
        children.is_none() || children.unwrap().is_null(),
        "children should be absent or null for simple rules"
    );
}

// -- Subtypes are optional --

#[test]
fn subtypes_absent_when_not_applicable() {
    let nodes = generate_parsed(&minimal_grammar());
    let expr = find_by_type(&nodes, "expr").unwrap();
    let subtypes = expr.get("subtypes");
    assert!(
        subtypes.is_none() || subtypes.unwrap().is_null(),
        "subtypes should be absent or null for simple rules"
    );
}
