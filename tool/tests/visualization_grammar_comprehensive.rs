//! Comprehensive tests for tool visualization module.

use adze_ir::builder::GrammarBuilder;

// ── Grammar construction for visualization ──

#[test]
fn viz_simple_grammar() {
    let g = GrammarBuilder::new("viz")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let dbg = format!("{:?}", g);
    assert!(dbg.contains("viz"));
}

#[test]
fn viz_multi_token_grammar() {
    let g = GrammarBuilder::new("multi")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("s", vec!["a", "b", "c"])
        .start("s")
        .build();
    assert_eq!(g.tokens.len(), 3);
}

#[test]
fn viz_alternative_grammar() {
    let g = GrammarBuilder::new("alts")
        .token("x", "x")
        .token("y", "y")
        .rule("s", vec!["x"])
        .rule("s", vec!["y"])
        .start("s")
        .build();
    assert!(g.all_rules().count() >= 2);
}

#[test]
fn viz_chain_grammar() {
    let g = GrammarBuilder::new("chain")
        .token("a", "a")
        .rule("x", vec!["a"])
        .rule("s", vec!["x"])
        .start("s")
        .build();
    assert!(g.all_rules().count() >= 2);
}

#[test]
fn viz_recursive_grammar() {
    let g = GrammarBuilder::new("rec")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a", "s"])
        .rule("s", vec!["b"])
        .start("s")
        .build();
    assert!(g.all_rules().count() >= 2);
}

// ── Grammar properties for visualization ──

#[test]
fn viz_grammar_name() {
    let g = GrammarBuilder::new("my_lang")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    assert_eq!(g.name, "my_lang");
}

#[test]
fn viz_grammar_token_names() {
    let g = GrammarBuilder::new("names")
        .token("number", "[0-9]+")
        .token("string", "\"[^\"]*\"")
        .rule("s", vec!["number"])
        .start("s")
        .build();
    assert_eq!(g.tokens.len(), 2);
}

#[test]
fn viz_grammar_rule_names() {
    let g = GrammarBuilder::new("rnames")
        .token("a", "a")
        .rule("expr", vec!["a"])
        .rule("start", vec!["expr"])
        .start("start")
        .build();
    assert!(g.rule_names.len() >= 2);
}

// ── Normalized grammar visualization ──

#[test]
fn viz_normalized_grammar() {
    let mut g = GrammarBuilder::new("norm")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a", "b"])
        .start("s")
        .build();
    g.normalize();
    let dbg = format!("{:?}", g);
    assert!(dbg.contains("norm"));
}

#[test]
fn viz_normalized_preserves_tokens() {
    let mut g = GrammarBuilder::new("normtok")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a", "b"])
        .start("s")
        .build();
    let before = g.tokens.len();
    g.normalize();
    assert_eq!(g.tokens.len(), before);
}

// ── Grammar serialization for visualization ──

#[test]
fn viz_json_output() {
    let g = GrammarBuilder::new("json")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let json = serde_json::to_string_pretty(&g).unwrap();
    assert!(json.contains("json"));
}

#[test]
fn viz_json_compact() {
    let g = GrammarBuilder::new("compact")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let json = serde_json::to_string(&g).unwrap();
    assert!(!json.contains("\n"));
}

// ── Large grammar visualization ──

#[test]
fn viz_many_tokens() {
    let mut b = GrammarBuilder::new("many");
    for i in 0..20 {
        let n = format!("tok{}", i);
        b = b.token(&n, &n);
    }
    b = b.rule("s", vec!["tok0"]).start("s");
    let g = b.build();
    assert_eq!(g.tokens.len(), 20);
}

#[test]
fn viz_many_rules() {
    let mut b = GrammarBuilder::new("manyrules");
    for i in 0..10 {
        let n = format!("t{}", i);
        b = b.token(&n, &n);
    }
    for i in 0..10 {
        let tok = format!("t{}", i);
        b = b.rule("s", vec![&tok]);
    }
    b = b.start("s");
    let g = b.build();
    assert!(g.all_rules().count() >= 10);
}

// ── Debug format tests ──

#[test]
fn debug_format_nonempty() {
    let g = GrammarBuilder::new("dbg")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let dbg = format!("{:?}", g);
    assert!(dbg.len() > 10);
}

#[test]
fn debug_format_contains_name() {
    let g = GrammarBuilder::new("myname")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let dbg = format!("{:?}", g);
    assert!(dbg.contains("myname"));
}

// ── Clone for visualization ──

#[test]
fn clone_for_viz() {
    let g = GrammarBuilder::new("clone")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let g2 = g.clone();
    assert_eq!(format!("{:?}", g), format!("{:?}", g2));
}
