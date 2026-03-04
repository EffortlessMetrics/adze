// Wave 132: Comprehensive tests for GrammarBuilder presets and edge cases
use adze_ir::builder::GrammarBuilder;
use adze_ir::*;

// =====================================================================
// Builder basic construction
// =====================================================================

#[test]
fn builder_creates_grammar_with_name() {
    let g = GrammarBuilder::new("test").build();
    assert_eq!(g.name, "test");
}

#[test]
fn builder_empty_grammar() {
    let g = GrammarBuilder::new("empty").build();
    assert!(g.tokens.is_empty());
    assert!(g.rules.is_empty());
}

#[test]
fn builder_single_token() {
    let g = GrammarBuilder::new("t").token("x", r"x").build();
    assert_eq!(g.tokens.len(), 1);
    let tok = g.tokens.values().next().unwrap();
    assert_eq!(tok.name, "x");
}

#[test]
fn builder_multiple_tokens() {
    let g = GrammarBuilder::new("t")
        .token("a", r"a")
        .token("b", r"b")
        .token("c", r"c")
        .build();
    assert_eq!(g.tokens.len(), 3);
}

#[test]
fn builder_token_names_preserved() {
    let g = GrammarBuilder::new("t")
        .token("identifier", r"[a-z]+")
        .token("number", r"\d+")
        .build();
    let names: Vec<String> = g.tokens.values().map(|t| t.name.clone()).collect();
    assert!(names.contains(&"identifier".to_string()));
    assert!(names.contains(&"number".to_string()));
}

#[test]
fn builder_single_rule() {
    let g = GrammarBuilder::new("r")
        .token("x", r"x")
        .rule("start", vec!["x"])
        .start("start")
        .build();
    assert!(!g.rules.is_empty());
}

#[test]
fn builder_multiple_rules_same_lhs() {
    let g = GrammarBuilder::new("r")
        .token("a", r"a")
        .token("b", r"b")
        .rule("start", vec!["a"])
        .rule("start", vec!["b"])
        .start("start")
        .build();
    // Should have 2 rules for the same LHS
    let total_rules: usize = g.rules.values().map(|v| v.len()).sum();
    assert!(total_rules >= 2);
}

#[test]
fn builder_empty_rhs_becomes_epsilon() {
    let g = GrammarBuilder::new("eps")
        .rule("start", vec![])
        .start("start")
        .build();
    let rules: Vec<&Rule> = g.rules.values().flatten().collect();
    let has_epsilon = rules.iter().any(|r| r.rhs.contains(&Symbol::Epsilon));
    assert!(has_epsilon, "Empty RHS should become Epsilon rule");
}

// =====================================================================
// Builder with start symbol
// =====================================================================

#[test]
fn builder_start_symbol_set() {
    let g = GrammarBuilder::new("s")
        .token("x", r"x")
        .rule("start", vec!["x"])
        .start("start")
        .build();
    assert!(g.start_symbol().is_some());
}

#[test]
fn builder_no_start_symbol() {
    let g = GrammarBuilder::new("ns")
        .token("x", r"x")
        .rule("expr", vec!["x"])
        .build();
    // Without .start(), start_symbol may be None or first rule
    let _ = g.start_symbol();
}

// =====================================================================
// Builder with precedence
// =====================================================================

#[test]
fn builder_rule_with_precedence() {
    let g = GrammarBuilder::new("prec")
        .token("num", r"\d+")
        .token("plus", r"\+")
        .rule("expr", vec!["num"])
        .rule_with_precedence("expr", vec!["expr", "plus", "expr"], 1, Associativity::Left)
        .start("expr")
        .build();
    let has_prec = g.rules.values().flatten().any(|r| r.precedence.is_some());
    assert!(has_prec);
}

// =====================================================================
// Builder fragile tokens
// =====================================================================

#[test]
fn builder_fragile_token() {
    let g = GrammarBuilder::new("frag")
        .token("ws", r"\s+")
        .fragile_token("comment", r"//.*")
        .rule("start", vec!["ws"])
        .start("start")
        .build();
    let fragile_tokens: Vec<&Token> = g.tokens.values().filter(|t| t.fragile).collect();
    assert_eq!(fragile_tokens.len(), 1);
    assert_eq!(fragile_tokens[0].name, "comment");
}

// =====================================================================
// Preset grammars
// =====================================================================

#[test]
fn python_like_preset_has_name() {
    let g = GrammarBuilder::python_like();
    assert_eq!(g.name, "python_like");
}

#[test]
fn python_like_preset_has_tokens() {
    let g = GrammarBuilder::python_like();
    assert!(!g.tokens.is_empty());
}

#[test]
fn python_like_preset_has_rules() {
    let g = GrammarBuilder::python_like();
    assert!(!g.rules.is_empty());
}

#[test]
fn python_like_preset_has_start_symbol() {
    let g = GrammarBuilder::python_like();
    assert!(g.start_symbol().is_some());
}

#[test]
fn javascript_like_preset_has_name() {
    let g = GrammarBuilder::javascript_like();
    assert_eq!(g.name, "javascript_like");
}

#[test]
fn javascript_like_preset_has_tokens() {
    let g = GrammarBuilder::javascript_like();
    assert!(!g.tokens.is_empty());
}

#[test]
fn javascript_like_preset_has_rules() {
    let g = GrammarBuilder::javascript_like();
    assert!(!g.rules.is_empty());
}

#[test]
fn javascript_like_preset_has_start_symbol() {
    let g = GrammarBuilder::javascript_like();
    assert!(g.start_symbol().is_some());
}

// =====================================================================
// Builder determinism
// =====================================================================

#[test]
fn builder_is_deterministic() {
    let build = || {
        GrammarBuilder::new("det")
            .token("a", r"a")
            .token("b", r"b")
            .rule("start", vec!["a"])
            .rule("start", vec!["b"])
            .start("start")
            .build()
    };
    let g1 = build();
    let g2 = build();
    assert_eq!(
        serde_json::to_string(&g1).unwrap(),
        serde_json::to_string(&g2).unwrap()
    );
}

// =====================================================================
// Rule_names populated correctly
// =====================================================================

#[test]
fn builder_rule_names_lowercase() {
    let g = GrammarBuilder::new("rn")
        .token("x", r"x")
        .rule("expression", vec!["x"])
        .start("expression")
        .build();
    let names: Vec<&String> = g.rule_names.values().collect();
    assert!(names.contains(&&"expression".to_string()));
}

#[test]
fn builder_rule_names_multiple() {
    let g = GrammarBuilder::new("rn")
        .token("x", r"x")
        .rule("start", vec!["item"])
        .rule("item", vec!["x"])
        .start("start")
        .build();
    let names: Vec<&String> = g.rule_names.values().collect();
    assert!(names.contains(&&"start".to_string()));
    assert!(names.contains(&&"item".to_string()));
}

// =====================================================================
// Grammar::new
// =====================================================================

#[test]
fn grammar_new() {
    let g = Grammar::new("test".to_string());
    assert_eq!(g.name, "test");
    assert!(g.rules.is_empty());
    assert!(g.tokens.is_empty());
    assert!(g.externals.is_empty());
    assert!(g.extras.is_empty());
    assert!(g.conflicts.is_empty());
}

// =====================================================================
// Grammar normalize
// =====================================================================

#[test]
fn normalize_simple_grammar() {
    let mut g = GrammarBuilder::new("norm")
        .token("x", r"x")
        .rule("start", vec!["x"])
        .start("start")
        .build();
    g.normalize();
    // Should not panic, grammar should still be valid
    assert!(!g.rules.is_empty());
}

#[test]
fn normalize_idempotent() {
    let mut g = GrammarBuilder::new("idem")
        .token("x", r"x")
        .rule("start", vec!["x"])
        .start("start")
        .build();
    g.normalize();
    let rules_after_first = g.rules.values().flatten().count();
    g.normalize();
    let rules_after_second = g.rules.values().flatten().count();
    assert_eq!(rules_after_first, rules_after_second);
}

// =====================================================================
// Complex builder scenarios
// =====================================================================

#[test]
fn builder_many_tokens() {
    let mut b = GrammarBuilder::new("many");
    for i in 0..50 {
        b = b.token(&format!("t{}", i), &format!("t{}", i));
    }
    let g = b.build();
    assert_eq!(g.tokens.len(), 50);
}

#[test]
fn builder_many_rules() {
    let mut b = GrammarBuilder::new("many");
    b = b.token("x", r"x");
    for _ in 0..20 {
        b = b.rule("start", vec!["x"]);
    }
    b = b.start("start");
    let g = b.build();
    let total: usize = g.rules.values().map(|v| v.len()).sum();
    assert!(total >= 20);
}

#[test]
fn builder_chain_all_methods() {
    let g = GrammarBuilder::new("chain")
        .token("num", r"\d+")
        .token("plus", r"\+")
        .token("star", r"\*")
        .token("lparen", r"\(")
        .token("rparen", r"\)")
        .fragile_token("ws", r"\s+")
        .rule("expr", vec!["num"])
        .rule("expr", vec!["expr", "plus", "expr"])
        .rule("expr", vec!["expr", "star", "expr"])
        .rule("expr", vec!["lparen", "expr", "rparen"])
        .rule_with_precedence("expr", vec!["expr", "plus", "expr"], 1, Associativity::Left)
        .start("expr")
        .build();
    assert!(g.tokens.len() >= 5);
    assert!(!g.rules.is_empty());
    assert!(g.start_symbol().is_some());
}
