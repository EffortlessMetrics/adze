//! Comprehensive tests for Grammar clone and serde behavior.

use adze_ir::builder::GrammarBuilder;

#[test]
fn grammar_clone_equal_name() {
    let g = GrammarBuilder::new("test")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let c = g.clone();
    assert_eq!(g.name, c.name);
}

#[test]
fn grammar_clone_equal_rules_count() {
    let g = GrammarBuilder::new("test")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let c = g.clone();
    assert_eq!(g.all_rules().count(), c.all_rules().count());
}

#[test]
fn grammar_clone_equal_start() {
    let g = GrammarBuilder::new("test")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let c = g.clone();
    assert_eq!(g.start_symbol(), c.start_symbol());
}

#[test]
fn grammar_clone_independent() {
    let g = GrammarBuilder::new("test")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let mut c = g.clone();
    c.normalize();
    // Original is not affected
    assert_eq!(g.name, "test");
}

#[test]
fn grammar_serde_json_roundtrip() {
    let g = GrammarBuilder::new("test")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let json = serde_json::to_string(&g).unwrap();
    let g2: adze_ir::Grammar = serde_json::from_str(&json).unwrap();
    assert_eq!(g.name, g2.name);
}

#[test]
fn grammar_serde_json_roundtrip_rules() {
    let g = GrammarBuilder::new("rt")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a"])
        .rule("s", vec!["b"])
        .start("s")
        .build();
    let json = serde_json::to_string(&g).unwrap();
    let g2: adze_ir::Grammar = serde_json::from_str(&json).unwrap();
    assert_eq!(g.all_rules().count(), g2.all_rules().count());
}

#[test]
fn grammar_serde_json_roundtrip_start() {
    let g = GrammarBuilder::new("start")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let json = serde_json::to_string(&g).unwrap();
    let g2: adze_ir::Grammar = serde_json::from_str(&json).unwrap();
    assert_eq!(g.start_symbol(), g2.start_symbol());
}

#[test]
fn grammar_serde_bincode_roundtrip() {
    let g = GrammarBuilder::new("bincode")
        .token("x", "x")
        .rule("s", vec!["x"])
        .start("s")
        .build();
    let bytes = postcard::to_stdvec(&g).unwrap();
    let g2: adze_ir::Grammar = postcard::from_bytes(&bytes).unwrap();
    assert_eq!(g.name, g2.name);
}

#[test]
fn grammar_serde_preserves_tokens() {
    let g = GrammarBuilder::new("tok")
        .token("plus", r"\+")
        .token("num", r"\d+")
        .rule("s", vec!["num"])
        .start("s")
        .build();
    let json = serde_json::to_string(&g).unwrap();
    assert!(json.contains("plus"));
    assert!(json.contains("num"));
}

#[test]
fn grammar_serde_preserves_name() {
    let g = GrammarBuilder::new("myname")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let json = serde_json::to_string(&g).unwrap();
    assert!(json.contains("myname"));
}

#[test]
fn grammar_clone_large() {
    let mut b = GrammarBuilder::new("big");
    for i in 0..50 {
        let n = format!("t{}", i);
        b = b.token(&n, &n);
    }
    for i in 0..50 {
        let tok = format!("t{}", i);
        b = b.rule("s", vec![&tok]);
    }
    b = b.start("s");
    let g = b.build();
    let c = g.clone();
    assert_eq!(g.all_rules().count(), c.all_rules().count());
}

#[test]
fn grammar_serde_large() {
    let mut b = GrammarBuilder::new("big");
    for i in 0..50 {
        let n = format!("t{}", i);
        b = b.token(&n, &n);
    }
    for i in 0..50 {
        let tok = format!("t{}", i);
        b = b.rule("s", vec![&tok]);
    }
    b = b.start("s");
    let g = b.build();
    let json = serde_json::to_string(&g).unwrap();
    let g2: adze_ir::Grammar = serde_json::from_str(&json).unwrap();
    assert_eq!(g.all_rules().count(), g2.all_rules().count());
}

#[test]
fn grammar_normalize_then_clone() {
    let mut g = GrammarBuilder::new("nc")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    g.normalize();
    let c = g.clone();
    assert_eq!(g.name, c.name);
}

#[test]
fn grammar_normalize_then_serde() {
    let mut g = GrammarBuilder::new("ns")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    g.normalize();
    let json = serde_json::to_string(&g).unwrap();
    let g2: adze_ir::Grammar = serde_json::from_str(&json).unwrap();
    assert_eq!(g.name, g2.name);
}

#[test]
fn grammar_serde_json_pretty() {
    let g = GrammarBuilder::new("pretty")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let json = serde_json::to_string_pretty(&g).unwrap();
    assert!(json.contains('\n'));
}
