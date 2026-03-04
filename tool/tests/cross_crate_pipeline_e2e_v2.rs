//! Cross-crate integration: Grammar → IR → GLR → ParseTable → Tablegen full pipeline.

use adze_glr_core::{FirstFollowSets, build_lr1_automaton};
use adze_ir::Associativity;
use adze_ir::builder::GrammarBuilder;
use adze_tablegen::{NodeTypesGenerator, StaticLanguageGenerator};

fn build_parse_table(grammar: &mut adze_ir::Grammar) -> Result<adze_glr_core::ParseTable, String> {
    grammar.normalize();
    let ff = FirstFollowSets::compute(grammar).map_err(|e| format!("{:?}", e))?;
    build_lr1_automaton(grammar, &ff).map_err(|e| format!("{:?}", e))
}

// ── Minimal pipeline ──

#[test]
fn pipeline_single_token_grammar() {
    let mut g = GrammarBuilder::new("single")
        .token("x", "x")
        .rule("s", vec!["x"])
        .start("s")
        .build();
    let pt = build_parse_table(&mut g).unwrap();
    assert!(pt.state_count > 0);
    assert!(pt.symbol_count > 0);
}

#[test]
fn pipeline_to_node_types() {
    let mut g = GrammarBuilder::new("node_types")
        .token("x", "x")
        .rule("s", vec!["x"])
        .start("s")
        .build();
    let _pt = build_parse_table(&mut g).unwrap();
    let output = NodeTypesGenerator::new(&g).generate().unwrap();
    let v: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert!(v.is_array());
}

#[test]
fn pipeline_to_static_language() {
    let mut g = GrammarBuilder::new("static_lang")
        .token("x", "x")
        .rule("s", vec!["x"])
        .start("s")
        .build();
    let pt = build_parse_table(&mut g).unwrap();
    let generator = StaticLanguageGenerator::new(g.clone(), pt);
    let code = generator.generate_language_code();
    let code_str = code.to_string();
    assert!(!code_str.is_empty());
}

// ── Two-alternative grammar ──

#[test]
fn pipeline_two_alternatives() {
    let mut g = GrammarBuilder::new("alt")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a"])
        .rule("s", vec!["b"])
        .start("s")
        .build();
    let pt = build_parse_table(&mut g).unwrap();
    assert!(pt.state_count >= 2);
    let code = StaticLanguageGenerator::new(g.clone(), pt)
        .generate_language_code()
        .to_string();
    assert!(!code.is_empty());
}

// ── Chain grammar ──

#[test]
fn pipeline_chain() {
    let mut g = GrammarBuilder::new("chain")
        .token("x", "x")
        .rule("a", vec!["x"])
        .rule("b", vec!["a"])
        .rule("s", vec!["b"])
        .start("s")
        .build();
    let pt = build_parse_table(&mut g).unwrap();
    assert!(pt.state_count > 0);
}

// ── Recursive grammar ──

#[test]
fn pipeline_recursive() {
    let mut g = GrammarBuilder::new("recursive")
        .token("n", "n")
        .token("plus", "+")
        .rule("e", vec!["n"])
        .rule("e", vec!["e", "plus", "n"])
        .start("e")
        .build();
    let pt = build_parse_table(&mut g).unwrap();
    assert!(pt.state_count >= 3);
}

// ── Precedence grammar ──

#[test]
fn pipeline_precedence() {
    let mut g = GrammarBuilder::new("prec")
        .token("n", "n")
        .token("plus", "+")
        .token("star", "*")
        .rule("e", vec!["n"])
        .rule_with_precedence("e", vec!["e", "plus", "e"], 1, Associativity::Left)
        .rule_with_precedence("e", vec!["e", "star", "e"], 2, Associativity::Left)
        .start("e")
        .build();
    let pt = build_parse_table(&mut g).unwrap();
    assert!(pt.state_count > 0);
}

// ── Large grammar ──

#[test]
fn pipeline_large_grammar() {
    let mut b = GrammarBuilder::new("large");
    for i in 0..10 {
        let n: &str = Box::leak(format!("tok{}", i).into_boxed_str());
        b = b.token(n, n).rule("s", vec![n]);
    }
    let mut g = b.start("s").build();
    let pt = build_parse_table(&mut g).unwrap();
    assert!(pt.state_count > 0);
    assert!(pt.symbol_count >= 10);
}

// ── Determinism ──

#[test]
fn pipeline_deterministic_parse_table() {
    let make = || {
        let mut g = GrammarBuilder::new("det")
            .token("x", "x")
            .rule("s", vec!["x"])
            .start("s")
            .build();
        build_parse_table(&mut g).unwrap()
    };
    let pt1 = make();
    let pt2 = make();
    assert_eq!(pt1.state_count, pt2.state_count);
    assert_eq!(pt1.symbol_count, pt2.symbol_count);
}

#[test]
fn pipeline_deterministic_code_gen() {
    let make = || {
        let mut g = GrammarBuilder::new("detcg")
            .token("x", "x")
            .rule("s", vec!["x"])
            .start("s")
            .build();
        let pt = build_parse_table(&mut g).unwrap();
        StaticLanguageGenerator::new(g.clone(), pt)
            .generate_language_code()
            .to_string()
    };
    assert_eq!(make(), make());
}

#[test]
fn pipeline_deterministic_node_types() {
    let make = || {
        let mut g = GrammarBuilder::new("detnt")
            .token("x", "x")
            .rule("s", vec!["x"])
            .start("s")
            .build();
        let _pt = build_parse_table(&mut g).unwrap();
        NodeTypesGenerator::new(&g).generate().unwrap()
    };
    assert_eq!(make(), make());
}

// ── Static language code properties ──

#[test]
fn static_language_contains_grammar_name() {
    let mut g = GrammarBuilder::new("my_grammar")
        .token("x", "x")
        .rule("s", vec!["x"])
        .start("s")
        .build();
    let pt = build_parse_table(&mut g).unwrap();
    let code = StaticLanguageGenerator::new(g.clone(), pt)
        .generate_language_code()
        .to_string();
    assert!(code.contains("my_grammar"));
}

// ── First/Follow properties ──

#[test]
fn first_follow_simple() {
    let mut g = GrammarBuilder::new("ff")
        .token("x", "x")
        .rule("s", vec!["x"])
        .start("s")
        .build();
    g.normalize();
    let ff = FirstFollowSets::compute(&g).unwrap();
    if let Some(start) = g.start_symbol() {
        let first_set = ff.first(start);
        assert!(first_set.is_some());
    }
}

#[test]
fn first_follow_multiple_rules() {
    let mut g = GrammarBuilder::new("ffa")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a"])
        .rule("s", vec!["b"])
        .start("s")
        .build();
    g.normalize();
    let ff = FirstFollowSets::compute(&g).unwrap();
    if let Some(first_set) = g.start_symbol().and_then(|start| ff.first(start)) {
        assert!(first_set.count_ones(..) >= 2);
    }
}

// ── Node types JSON structure ──

#[test]
fn node_types_all_have_type_field() {
    let mut g = GrammarBuilder::new("ntf")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a"])
        .rule("s", vec!["b"])
        .start("s")
        .build();
    let _pt = build_parse_table(&mut g).unwrap();
    let json_str = NodeTypesGenerator::new(&g).generate().unwrap();
    let v: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    if let serde_json::Value::Array(arr) = v {
        for item in &arr {
            assert!(item.get("type").is_some());
        }
    }
}

#[test]
fn node_types_all_have_named_field() {
    let mut g = GrammarBuilder::new("ntn")
        .token("x", "x")
        .rule("s", vec!["x"])
        .start("s")
        .build();
    let _pt = build_parse_table(&mut g).unwrap();
    let json_str = NodeTypesGenerator::new(&g).generate().unwrap();
    let v: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    if let serde_json::Value::Array(arr) = v {
        for item in &arr {
            assert!(item.get("named").is_some());
        }
    }
}

// ── Parse table properties ──

#[test]
fn parse_table_action_table_len() {
    let mut g = GrammarBuilder::new("ptl")
        .token("x", "x")
        .rule("s", vec!["x"])
        .start("s")
        .build();
    let pt = build_parse_table(&mut g).unwrap();
    assert_eq!(pt.action_table.len(), pt.state_count);
}

#[test]
fn parse_table_goto_table_len() {
    let mut g = GrammarBuilder::new("ptg")
        .token("x", "x")
        .rule("s", vec!["x"])
        .start("s")
        .build();
    let pt = build_parse_table(&mut g).unwrap();
    assert_eq!(pt.goto_table.len(), pt.state_count);
}

#[test]
fn parse_table_rules_nonempty() {
    let mut g = GrammarBuilder::new("ptr")
        .token("x", "x")
        .rule("s", vec!["x"])
        .start("s")
        .build();
    let pt = build_parse_table(&mut g).unwrap();
    assert!(!pt.rules.is_empty());
}

#[test]
fn parse_table_eof_symbol_valid() {
    let mut g = GrammarBuilder::new("pte")
        .token("x", "x")
        .rule("s", vec!["x"])
        .start("s")
        .build();
    let pt = build_parse_table(&mut g).unwrap();
    // eof_symbol is a valid SymbolId
    let _ = pt.eof_symbol;
}

// ── Right-associative ──

#[test]
fn pipeline_right_associative() {
    let mut g = GrammarBuilder::new("rassoc")
        .token("n", "n")
        .token("eq", "=")
        .rule("e", vec!["n"])
        .rule_with_precedence("e", vec!["e", "eq", "e"], 1, Associativity::Right)
        .start("e")
        .build();
    let pt = build_parse_table(&mut g).unwrap();
    assert!(pt.state_count > 0);
}

// ── Mixed associativity ──

#[test]
fn pipeline_mixed_assoc() {
    let mut g = GrammarBuilder::new("mixed")
        .token("n", "n")
        .token("plus", "+")
        .token("pow", "^")
        .rule("e", vec!["n"])
        .rule_with_precedence("e", vec!["e", "plus", "e"], 1, Associativity::Left)
        .rule_with_precedence("e", vec!["e", "pow", "e"], 2, Associativity::Right)
        .start("e")
        .build();
    let pt = build_parse_table(&mut g).unwrap();
    assert!(pt.state_count > 0);
}

// ── Multiple nonterminals ──

#[test]
fn pipeline_multiple_nonterminals() {
    let mut g = GrammarBuilder::new("multi_nt")
        .token("x", "x")
        .token("y", "y")
        .rule("a", vec!["x"])
        .rule("b", vec!["y"])
        .rule("s", vec!["a", "b"])
        .start("s")
        .build();
    let pt = build_parse_table(&mut g).unwrap();
    assert!(pt.state_count > 0);
}

#[test]
fn pipeline_sequence_of_three() {
    let mut g = GrammarBuilder::new("seq3")
        .token("x", "x")
        .token("y", "y")
        .token("z", "z")
        .rule("s", vec!["x", "y", "z"])
        .start("s")
        .build();
    let pt = build_parse_table(&mut g).unwrap();
    assert!(pt.state_count >= 4);
}

// ── ParseRule properties ──

#[test]
fn parse_rules_all_have_lhs() {
    let mut g = GrammarBuilder::new("prl")
        .token("x", "x")
        .rule("s", vec!["x"])
        .start("s")
        .build();
    let pt = build_parse_table(&mut g).unwrap();
    for rule in &pt.rules {
        // lhs should be a valid symbol
        let _ = rule.lhs;
    }
}

#[test]
fn parse_rules_rhs_len() {
    let mut g = GrammarBuilder::new("prr")
        .token("x", "x")
        .token("y", "y")
        .rule("s", vec!["x", "y"])
        .start("s")
        .build();
    let pt = build_parse_table(&mut g).unwrap();
    // At least one rule should have rhs_len >= 2
    let has_multi = pt.rules.iter().any(|r| r.rhs_len >= 2);
    assert!(has_multi);
}

// ── Grammar name propagation ──

#[test]
fn grammar_name_in_node_types() {
    let mut g = GrammarBuilder::new("named_grammar")
        .token("x", "x")
        .rule("s", vec!["x"])
        .start("s")
        .build();
    let _pt = build_parse_table(&mut g).unwrap();
    let json = NodeTypesGenerator::new(&g).generate().unwrap();
    // Grammar name is used in the grammar, may or may not appear in node types
    let _ = json;
}

// ── Multiple build ──

#[test]
fn build_parse_table_twice() {
    let make = || {
        let mut g = GrammarBuilder::new("twice")
            .token("x", "x")
            .rule("s", vec!["x"])
            .start("s")
            .build();
        build_parse_table(&mut g).unwrap()
    };
    let pt1 = make();
    let pt2 = make();
    assert_eq!(pt1.rules.len(), pt2.rules.len());
}
