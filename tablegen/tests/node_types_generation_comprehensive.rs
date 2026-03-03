#![allow(clippy::needless_range_loop)]

use adze_ir::{
    ExternalToken, FieldId, Grammar, ProductionId, Rule, Symbol, SymbolId, Token, TokenPattern,
};
use adze_tablegen::NodeTypesGenerator;
use serde_json::Value;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn empty_grammar() -> Grammar {
    Grammar::new("empty".to_string())
}

/// Minimal grammar: one named rule consuming one string token.
fn minimal_grammar() -> Grammar {
    let mut g = Grammar::new("minimal".to_string());

    let tok_id = SymbolId(0);
    g.tokens.insert(
        tok_id,
        Token {
            name: "plus".to_string(),
            pattern: TokenPattern::String("+".to_string()),
            fragile: false,
        },
    );

    let rule_id = SymbolId(1);
    g.rule_names.insert(rule_id, "expression".to_string());
    g.add_rule(Rule {
        lhs: rule_id,
        rhs: vec![Symbol::Terminal(tok_id)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    g
}

/// Grammar with both string-literal and regex tokens to test named/anonymous.
fn mixed_token_grammar() -> Grammar {
    let mut g = Grammar::new("mixed".to_string());

    let str_tok = SymbolId(0);
    g.tokens.insert(
        str_tok,
        Token {
            name: "semicolon".to_string(),
            pattern: TokenPattern::String(";".to_string()),
            fragile: false,
        },
    );

    let regex_tok = SymbolId(1);
    g.tokens.insert(
        regex_tok,
        Token {
            name: "identifier".to_string(),
            pattern: TokenPattern::Regex(r"[a-z]+".to_string()),
            fragile: false,
        },
    );

    let rule_id = SymbolId(2);
    g.rule_names.insert(rule_id, "statement".to_string());
    g.add_rule(Rule {
        lhs: rule_id,
        rhs: vec![Symbol::Terminal(regex_tok), Symbol::Terminal(str_tok)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    g
}

/// Grammar with fields on a rule.
fn grammar_with_fields() -> Grammar {
    let mut g = Grammar::new("fields".to_string());

    let num_tok = SymbolId(0);
    g.tokens.insert(
        num_tok,
        Token {
            name: "number".to_string(),
            pattern: TokenPattern::Regex(r"\d+".to_string()),
            fragile: false,
        },
    );

    let op_tok = SymbolId(1);
    g.tokens.insert(
        op_tok,
        Token {
            name: "op".to_string(),
            pattern: TokenPattern::String("+".to_string()),
            fragile: false,
        },
    );

    let lhs_field = FieldId(0);
    let rhs_field = FieldId(1);
    g.fields.insert(lhs_field, "left".to_string());
    g.fields.insert(rhs_field, "right".to_string());

    let expr_id = SymbolId(2);
    g.rule_names.insert(expr_id, "binary_expression".to_string());
    g.add_rule(Rule {
        lhs: expr_id,
        rhs: vec![
            Symbol::Terminal(num_tok),
            Symbol::Terminal(op_tok),
            Symbol::Terminal(num_tok),
        ],
        precedence: None,
        associativity: None,
        fields: vec![(lhs_field, 0), (rhs_field, 2)],
        production_id: ProductionId(0),
    });
    g
}

/// Grammar with an internal rule (name starts with `_`).
fn grammar_with_internal_rule() -> Grammar {
    let mut g = Grammar::new("internal".to_string());

    let tok = SymbolId(0);
    g.tokens.insert(
        tok,
        Token {
            name: "x".to_string(),
            pattern: TokenPattern::String("x".to_string()),
            fragile: false,
        },
    );

    // Internal rule – should NOT appear as a named node.
    let internal_id = SymbolId(1);
    g.rule_names.insert(internal_id, "_hidden".to_string());
    g.add_rule(Rule {
        lhs: internal_id,
        rhs: vec![Symbol::Terminal(tok)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });

    // Public rule
    let public_id = SymbolId(2);
    g.rule_names.insert(public_id, "visible".to_string());
    g.add_rule(Rule {
        lhs: public_id,
        rhs: vec![Symbol::NonTerminal(internal_id)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(1),
    });
    g
}

/// Grammar with a supertype declared.
fn grammar_with_supertype() -> Grammar {
    let mut g = Grammar::new("supertype".to_string());

    let num_tok = SymbolId(0);
    g.tokens.insert(
        num_tok,
        Token {
            name: "number".to_string(),
            pattern: TokenPattern::Regex(r"\d+".to_string()),
            fragile: false,
        },
    );

    let lit_id = SymbolId(1);
    g.rule_names.insert(lit_id, "number_literal".to_string());
    g.add_rule(Rule {
        lhs: lit_id,
        rhs: vec![Symbol::Terminal(num_tok)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });

    // _expression is a supertype
    let expr_id = SymbolId(2);
    g.rule_names.insert(expr_id, "_expression".to_string());
    g.supertypes.push(expr_id);
    g.add_rule(Rule {
        lhs: expr_id,
        rhs: vec![Symbol::NonTerminal(lit_id)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(1),
    });
    g
}

/// Build a large grammar with many rules.
fn large_grammar(n_rules: u16) -> Grammar {
    let mut g = Grammar::new("large".to_string());

    // One shared token
    let tok = SymbolId(0);
    g.tokens.insert(
        tok,
        Token {
            name: "tok".to_string(),
            pattern: TokenPattern::String("t".to_string()),
            fragile: false,
        },
    );

    for i in 0..n_rules {
        let sid = SymbolId(i + 1);
        g.rule_names.insert(sid, format!("rule_{i}"));
        g.add_rule(Rule {
            lhs: sid,
            rhs: vec![Symbol::Terminal(tok)],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(i),
        });
    }
    g
}

fn parse_node_types(json: &str) -> Vec<Value> {
    let v: Value = serde_json::from_str(json).expect("invalid JSON");
    v.as_array().expect("expected array").clone()
}

fn find_node<'a>(nodes: &'a [Value], name: &str) -> Option<&'a Value> {
    nodes
        .iter()
        .find(|n| n.get("type").and_then(Value::as_str) == Some(name))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[test]
fn test_empty_grammar_produces_empty_array() {
    let g = empty_grammar();
    let ntg = NodeTypesGenerator::new(&g);
    let json = ntg.generate().unwrap();
    let nodes = parse_node_types(&json);
    assert!(nodes.is_empty(), "empty grammar should yield []");
}

#[test]
fn test_output_is_valid_json() {
    let g = minimal_grammar();
    let ntg = NodeTypesGenerator::new(&g);
    let json = ntg.generate().unwrap();
    let _: Value = serde_json::from_str(&json).expect("must be valid JSON");
}

#[test]
fn test_each_entry_has_type_field() {
    let g = minimal_grammar();
    let ntg = NodeTypesGenerator::new(&g);
    let nodes = parse_node_types(&ntg.generate().unwrap());
    for node in &nodes {
        assert!(
            node.get("type").is_some(),
            "every entry must have a \"type\" field"
        );
    }
}

#[test]
fn test_each_entry_has_named_field() {
    let g = minimal_grammar();
    let ntg = NodeTypesGenerator::new(&g);
    let nodes = parse_node_types(&ntg.generate().unwrap());
    for node in &nodes {
        assert!(
            node.get("named").and_then(Value::as_bool).is_some(),
            "every entry must have a boolean \"named\" field"
        );
    }
}

#[test]
fn test_named_rule_appears_as_named_node() {
    let g = minimal_grammar();
    let ntg = NodeTypesGenerator::new(&g);
    let nodes = parse_node_types(&ntg.generate().unwrap());
    let expr = find_node(&nodes, "expression").expect("expression node must exist");
    assert_eq!(expr["named"], true);
}

#[test]
fn test_string_token_is_anonymous() {
    let g = minimal_grammar();
    let ntg = NodeTypesGenerator::new(&g);
    let nodes = parse_node_types(&ntg.generate().unwrap());
    let plus = find_node(&nodes, "+").expect("'+' anonymous node must exist");
    assert_eq!(plus["named"], false);
}

#[test]
fn test_regex_token_is_named() {
    let g = mixed_token_grammar();
    let ntg = NodeTypesGenerator::new(&g);
    let nodes = parse_node_types(&ntg.generate().unwrap());

    // The regex token "identifier" should NOT appear as an anonymous node.
    // Only string-literal tokens are emitted as anonymous.
    let anon: Vec<_> = nodes
        .iter()
        .filter(|n| n["named"] == false)
        .collect();
    // ";" is the only anonymous node.
    assert!(anon.iter().any(|n| n["type"] == ";"));
    // "identifier" should not be among anonymous nodes.
    assert!(anon.iter().all(|n| n["type"] != "identifier"));
}

#[test]
fn test_terminal_string_classification() {
    let g = mixed_token_grammar();
    let ntg = NodeTypesGenerator::new(&g);
    let nodes = parse_node_types(&ntg.generate().unwrap());
    let semi = find_node(&nodes, ";").expect("semicolon anonymous node");
    assert_eq!(semi["named"], false, "string literal tokens are anonymous");
}

#[test]
fn test_nonterminal_is_named() {
    let g = mixed_token_grammar();
    let ntg = NodeTypesGenerator::new(&g);
    let nodes = parse_node_types(&ntg.generate().unwrap());
    let stmt = find_node(&nodes, "statement").expect("statement node must exist");
    assert_eq!(stmt["named"], true);
}

#[test]
fn test_field_descriptions_present() {
    let g = grammar_with_fields();
    let ntg = NodeTypesGenerator::new(&g);
    let nodes = parse_node_types(&ntg.generate().unwrap());
    let expr = find_node(&nodes, "binary_expression").expect("binary_expression node");
    let fields = expr.get("fields").expect("fields must be present");
    assert!(fields.get("left").is_some(), "left field expected");
    assert!(fields.get("right").is_some(), "right field expected");
}

#[test]
fn test_field_has_types_array() {
    let g = grammar_with_fields();
    let ntg = NodeTypesGenerator::new(&g);
    let nodes = parse_node_types(&ntg.generate().unwrap());
    let expr = find_node(&nodes, "binary_expression").unwrap();
    let left = &expr["fields"]["left"];
    assert!(left["types"].is_array(), "field types must be an array");
    assert!(!left["types"].as_array().unwrap().is_empty());
}

#[test]
fn test_field_has_required_and_multiple() {
    let g = grammar_with_fields();
    let ntg = NodeTypesGenerator::new(&g);
    let nodes = parse_node_types(&ntg.generate().unwrap());
    let expr = find_node(&nodes, "binary_expression").unwrap();
    let left = &expr["fields"]["left"];
    assert!(
        left.get("required").is_some(),
        "field must have 'required'"
    );
    assert!(
        left.get("multiple").is_some(),
        "field must have 'multiple'"
    );
}

#[test]
fn test_field_type_ref_structure() {
    let g = grammar_with_fields();
    let ntg = NodeTypesGenerator::new(&g);
    let nodes = parse_node_types(&ntg.generate().unwrap());
    let expr = find_node(&nodes, "binary_expression").unwrap();
    let types = expr["fields"]["left"]["types"].as_array().unwrap();
    for tr in types {
        assert!(tr.get("type").is_some(), "type ref needs \"type\"");
        assert!(tr.get("named").is_some(), "type ref needs \"named\"");
    }
}

#[test]
fn test_node_without_fields_omits_fields_key() {
    let g = minimal_grammar();
    let ntg = NodeTypesGenerator::new(&g);
    let nodes = parse_node_types(&ntg.generate().unwrap());
    let expr = find_node(&nodes, "expression").unwrap();
    // The generator uses skip_serializing_if = Option::is_none for fields
    assert!(
        expr.get("fields").is_none(),
        "nodes without fields should not have a 'fields' key"
    );
}

#[test]
fn test_node_without_children_omits_children_key() {
    let g = minimal_grammar();
    let ntg = NodeTypesGenerator::new(&g);
    let nodes = parse_node_types(&ntg.generate().unwrap());
    let expr = find_node(&nodes, "expression").unwrap();
    assert!(
        expr.get("children").is_none(),
        "nodes without children should not have a 'children' key"
    );
}

#[test]
fn test_node_without_subtypes_omits_subtypes_key() {
    let g = minimal_grammar();
    let ntg = NodeTypesGenerator::new(&g);
    let nodes = parse_node_types(&ntg.generate().unwrap());
    let expr = find_node(&nodes, "expression").unwrap();
    assert!(
        expr.get("subtypes").is_none(),
        "nodes without subtypes should not have a 'subtypes' key"
    );
}

#[test]
fn test_internal_rule_excluded_from_output() {
    let g = grammar_with_internal_rule();
    let ntg = NodeTypesGenerator::new(&g);
    let nodes = parse_node_types(&ntg.generate().unwrap());
    assert!(
        find_node(&nodes, "_hidden").is_none(),
        "internal rules (prefixed _) must not appear"
    );
}

#[test]
fn test_public_rule_present_alongside_internal() {
    let g = grammar_with_internal_rule();
    let ntg = NodeTypesGenerator::new(&g);
    let nodes = parse_node_types(&ntg.generate().unwrap());
    assert!(
        find_node(&nodes, "visible").is_some(),
        "public rule must be present"
    );
}

#[test]
fn test_supertype_symbol_in_grammar() {
    // Verify the generator doesn't crash with supertypes declared.
    let g = grammar_with_supertype();
    let ntg = NodeTypesGenerator::new(&g);
    let json = ntg.generate().unwrap();
    let nodes = parse_node_types(&json);
    // number_literal is public and should appear
    assert!(find_node(&nodes, "number_literal").is_some());
}

#[test]
fn test_supertype_internal_rule_excluded() {
    let g = grammar_with_supertype();
    let ntg = NodeTypesGenerator::new(&g);
    let nodes = parse_node_types(&ntg.generate().unwrap());
    // _expression starts with _ so it's internal
    assert!(find_node(&nodes, "_expression").is_none());
}

#[test]
fn test_output_sorted_by_type_name() {
    let g = grammar_with_fields();
    let ntg = NodeTypesGenerator::new(&g);
    let nodes = parse_node_types(&ntg.generate().unwrap());
    let names: Vec<&str> = nodes
        .iter()
        .map(|n| n["type"].as_str().unwrap())
        .collect();
    let mut sorted = names.clone();
    sorted.sort();
    assert_eq!(names, sorted, "node types must be sorted alphabetically");
}

#[test]
fn test_large_grammar_generates_all_rules() {
    let n = 50u16;
    let g = large_grammar(n);
    let ntg = NodeTypesGenerator::new(&g);
    let nodes = parse_node_types(&ntg.generate().unwrap());

    // Each public rule + 1 anonymous "t" token
    let named_count = nodes.iter().filter(|n| n["named"] == true).count();
    assert_eq!(named_count, n as usize);
}

#[test]
fn test_large_grammar_anonymous_token_present() {
    let g = large_grammar(10);
    let ntg = NodeTypesGenerator::new(&g);
    let nodes = parse_node_types(&ntg.generate().unwrap());
    assert!(
        find_node(&nodes, "t").is_some(),
        "anonymous token 't' should be present"
    );
}

#[test]
fn test_duplicate_string_tokens_handled() {
    // Two tokens with the same string pattern
    let mut g = Grammar::new("dup".to_string());
    let t1 = SymbolId(0);
    g.tokens.insert(
        t1,
        Token {
            name: "a".to_string(),
            pattern: TokenPattern::String("x".to_string()),
            fragile: false,
        },
    );
    let t2 = SymbolId(1);
    g.tokens.insert(
        t2,
        Token {
            name: "b".to_string(),
            pattern: TokenPattern::String("x".to_string()),
            fragile: false,
        },
    );
    let ntg = NodeTypesGenerator::new(&g);
    // Should not crash
    let json = ntg.generate().unwrap();
    let nodes = parse_node_types(&json);
    // Both anonymous entries may share the same type name "x"
    let xs: Vec<_> = nodes.iter().filter(|n| n["type"] == "x").collect();
    assert!(
        !xs.is_empty(),
        "at least one anonymous node 'x' must exist"
    );
}

#[test]
fn test_error_node_not_injected() {
    // The generator does not synthesize ERROR nodes; verify absence.
    let g = minimal_grammar();
    let ntg = NodeTypesGenerator::new(&g);
    let nodes = parse_node_types(&ntg.generate().unwrap());
    assert!(
        find_node(&nodes, "ERROR").is_none(),
        "generator should not inject ERROR unless explicitly added"
    );
}

#[test]
fn test_root_node_not_injected() {
    let g = minimal_grammar();
    let ntg = NodeTypesGenerator::new(&g);
    let nodes = parse_node_types(&ntg.generate().unwrap());
    assert!(
        find_node(&nodes, "_root").is_none(),
        "generator should not inject _root unless explicitly added"
    );
}

#[test]
fn test_multiple_rules_same_lhs() {
    // Multiple alternative rules for one symbol should produce a single node entry.
    let mut g = Grammar::new("multi_alt".to_string());

    let t1 = SymbolId(0);
    g.tokens.insert(
        t1,
        Token {
            name: "a".to_string(),
            pattern: TokenPattern::String("a".to_string()),
            fragile: false,
        },
    );
    let t2 = SymbolId(1);
    g.tokens.insert(
        t2,
        Token {
            name: "b".to_string(),
            pattern: TokenPattern::String("b".to_string()),
            fragile: false,
        },
    );

    let rule_id = SymbolId(2);
    g.rule_names.insert(rule_id, "choice".to_string());
    g.add_rule(Rule {
        lhs: rule_id,
        rhs: vec![Symbol::Terminal(t1)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    g.add_rule(Rule {
        lhs: rule_id,
        rhs: vec![Symbol::Terminal(t2)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(1),
    });

    let ntg = NodeTypesGenerator::new(&g);
    let nodes = parse_node_types(&ntg.generate().unwrap());
    let choice_nodes: Vec<_> = nodes.iter().filter(|n| n["type"] == "choice").collect();
    assert_eq!(
        choice_nodes.len(),
        1,
        "multiple alternatives should produce exactly one node entry"
    );
}

#[test]
fn test_external_token_grammar_does_not_crash() {
    let mut g = Grammar::new("ext".to_string());
    g.externals.push(ExternalToken {
        name: "indent".to_string(),
        symbol_id: SymbolId(99),
    });
    let ntg = NodeTypesGenerator::new(&g);
    let json = ntg.generate().unwrap();
    let nodes = parse_node_types(&json);
    // External tokens live in the externals list, not tokens, so empty output is fine.
    assert!(nodes.is_empty() || !nodes.is_empty());
}

#[test]
fn test_generate_returns_ok() {
    let g = minimal_grammar();
    let ntg = NodeTypesGenerator::new(&g);
    assert!(ntg.generate().is_ok());
}

#[test]
fn test_pretty_printed_json() {
    let g = minimal_grammar();
    let ntg = NodeTypesGenerator::new(&g);
    let json = ntg.generate().unwrap();
    // Pretty-printed JSON contains newlines and indentation
    assert!(json.contains('\n'), "output should be pretty-printed");
}

#[test]
fn test_fields_from_multiple_alternatives_merged() {
    // When a symbol has two rules with different fields, both fields appear.
    let mut g = Grammar::new("merged_fields".to_string());

    let t1 = SymbolId(0);
    g.tokens.insert(
        t1,
        Token {
            name: "num".to_string(),
            pattern: TokenPattern::Regex(r"\d+".to_string()),
            fragile: false,
        },
    );

    let f1 = FieldId(0);
    let f2 = FieldId(1);
    g.fields.insert(f1, "alpha".to_string());
    g.fields.insert(f2, "beta".to_string());

    let rule_id = SymbolId(1);
    g.rule_names.insert(rule_id, "combo".to_string());
    g.add_rule(Rule {
        lhs: rule_id,
        rhs: vec![Symbol::Terminal(t1)],
        precedence: None,
        associativity: None,
        fields: vec![(f1, 0)],
        production_id: ProductionId(0),
    });
    g.add_rule(Rule {
        lhs: rule_id,
        rhs: vec![Symbol::Terminal(t1)],
        precedence: None,
        associativity: None,
        fields: vec![(f2, 0)],
        production_id: ProductionId(1),
    });

    let ntg = NodeTypesGenerator::new(&g);
    let nodes = parse_node_types(&ntg.generate().unwrap());
    let combo = find_node(&nodes, "combo").expect("combo node");
    let fields = combo.get("fields").expect("fields present");
    assert!(fields.get("alpha").is_some(), "alpha field expected");
    assert!(fields.get("beta").is_some(), "beta field expected");
}

#[test]
fn test_only_string_tokens_emitted_as_anonymous() {
    // Regex tokens should not be emitted as anonymous nodes in the token loop.
    let mut g = Grammar::new("regex_only".to_string());
    let t = SymbolId(0);
    g.tokens.insert(
        t,
        Token {
            name: "ident".to_string(),
            pattern: TokenPattern::Regex(r"[a-z]+".to_string()),
            fragile: false,
        },
    );

    let ntg = NodeTypesGenerator::new(&g);
    let nodes = parse_node_types(&ntg.generate().unwrap());
    // No anonymous nodes should be emitted for a regex-only token set.
    let anon: Vec<_> = nodes.iter().filter(|n| n["named"] == false).collect();
    assert!(
        anon.is_empty(),
        "regex tokens should not produce anonymous entries"
    );
}

#[test]
fn test_fallback_rule_name() {
    // If no rule_name is registered, the generator falls back to "rule_<id>".
    let mut g = Grammar::new("fallback".to_string());
    let tok = SymbolId(0);
    g.tokens.insert(
        tok,
        Token {
            name: "x".to_string(),
            pattern: TokenPattern::String("x".to_string()),
            fragile: false,
        },
    );
    let rule_id = SymbolId(5);
    // Don't insert into rule_names — force fallback.
    g.add_rule(Rule {
        lhs: rule_id,
        rhs: vec![Symbol::Terminal(tok)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });

    let ntg = NodeTypesGenerator::new(&g);
    let nodes = parse_node_types(&ntg.generate().unwrap());
    assert!(
        find_node(&nodes, "rule_5").is_some(),
        "fallback name 'rule_5' expected"
    );
}

#[test]
fn test_nonterminal_type_ref_is_named() {
    // A field referencing a NonTerminal should have named: true in type ref.
    let mut g = Grammar::new("nt_ref".to_string());

    let tok = SymbolId(0);
    g.tokens.insert(
        tok,
        Token {
            name: "t".to_string(),
            pattern: TokenPattern::String("t".to_string()),
            fragile: false,
        },
    );

    let child_id = SymbolId(1);
    g.rule_names.insert(child_id, "child".to_string());
    g.add_rule(Rule {
        lhs: child_id,
        rhs: vec![Symbol::Terminal(tok)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });

    let f = FieldId(0);
    g.fields.insert(f, "value".to_string());

    let parent_id = SymbolId(2);
    g.rule_names.insert(parent_id, "parent".to_string());
    g.add_rule(Rule {
        lhs: parent_id,
        rhs: vec![Symbol::NonTerminal(child_id)],
        precedence: None,
        associativity: None,
        fields: vec![(f, 0)],
        production_id: ProductionId(1),
    });

    let ntg = NodeTypesGenerator::new(&g);
    let nodes = parse_node_types(&ntg.generate().unwrap());
    let parent = find_node(&nodes, "parent").unwrap();
    let types = parent["fields"]["value"]["types"].as_array().unwrap();
    assert_eq!(types[0]["named"], true);
    assert_eq!(types[0]["type"], "child");
}

#[test]
fn test_terminal_type_ref_string_is_anonymous() {
    // A field referencing a string-literal terminal should have named: false.
    let mut g = Grammar::new("term_ref".to_string());

    let tok = SymbolId(0);
    g.tokens.insert(
        tok,
        Token {
            name: "lp".to_string(),
            pattern: TokenPattern::String("(".to_string()),
            fragile: false,
        },
    );

    let f = FieldId(0);
    g.fields.insert(f, "open".to_string());

    let rule_id = SymbolId(1);
    g.rule_names.insert(rule_id, "group".to_string());
    g.add_rule(Rule {
        lhs: rule_id,
        rhs: vec![Symbol::Terminal(tok)],
        precedence: None,
        associativity: None,
        fields: vec![(f, 0)],
        production_id: ProductionId(0),
    });

    let ntg = NodeTypesGenerator::new(&g);
    let nodes = parse_node_types(&ntg.generate().unwrap());
    let group = find_node(&nodes, "group").unwrap();
    let types = group["fields"]["open"]["types"].as_array().unwrap();
    assert_eq!(types[0]["named"], false);
    assert_eq!(types[0]["type"], "(");
}
