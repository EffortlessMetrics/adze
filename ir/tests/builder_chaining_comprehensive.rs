//! Comprehensive tests for GrammarBuilder fluent chaining patterns.

use adze_ir::Associativity;
use adze_ir::builder::GrammarBuilder;

#[test]
fn chain_minimal() {
    let g = GrammarBuilder::new("ch1")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    assert_eq!(g.name, "ch1");
}

#[test]
fn chain_two_tokens() {
    let g = GrammarBuilder::new("ch2")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    assert!(g.all_rules().count() >= 1);
}

#[test]
fn chain_two_rules_same_lhs() {
    let g = GrammarBuilder::new("ch3")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a"])
        .rule("s", vec!["b"])
        .start("s")
        .build();
    assert!(g.all_rules().count() >= 2);
}

#[test]
fn chain_with_nonterminal() {
    let g = GrammarBuilder::new("ch4")
        .token("x", "x")
        .rule("inner", vec!["x"])
        .rule("s", vec!["inner"])
        .start("s")
        .build();
    assert!(g.all_rules().count() >= 2);
}

#[test]
fn chain_multi_symbol_rhs() {
    let g = GrammarBuilder::new("ch5")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a", "b"])
        .start("s")
        .build();
    let r = g.all_rules().next().unwrap();
    assert_eq!(r.rhs.len(), 2);
}

#[test]
fn chain_prec_left() {
    let g = GrammarBuilder::new("ch6")
        .token("a", "a")
        .rule_with_precedence("s", vec!["a"], 1, Associativity::Left)
        .start("s")
        .build();
    assert!(g.all_rules().count() >= 1);
}

#[test]
fn chain_prec_right() {
    let g = GrammarBuilder::new("ch7")
        .token("a", "a")
        .rule_with_precedence("s", vec!["a"], 2, Associativity::Right)
        .start("s")
        .build();
    assert!(g.all_rules().count() >= 1);
}

#[test]
fn chain_mixed_plain_prec() {
    let g = GrammarBuilder::new("ch8")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a"])
        .rule_with_precedence("s", vec!["b"], 1, Associativity::Left)
        .start("s")
        .build();
    assert!(g.all_rules().count() >= 2);
}

#[test]
fn chain_name_is_set() {
    let g = GrammarBuilder::new("mygrammar")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    assert_eq!(g.name, "mygrammar");
}

#[test]
fn chain_start_symbol_valid() {
    let g = GrammarBuilder::new("ch9")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    assert!(g.start_symbol().is_some());
}

#[test]
fn chain_empty_rhs() {
    let g = GrammarBuilder::new("empty")
        .token("a", "a")
        .rule("s", vec![])
        .start("s")
        .build();
    let _ = g.all_rules().count();
}

#[test]
fn chain_many_tokens_loop() {
    let mut b = GrammarBuilder::new("loop");
    for i in 0..30 {
        let n = format!("tok{}", i);
        b = b.token(&n, &n);
    }
    b = b.rule("s", vec!["tok0"]).start("s");
    let g = b.build();
    assert!(g.all_rules().count() >= 1);
}

#[test]
fn chain_many_rules_loop() {
    let mut b = GrammarBuilder::new("rloop");
    for i in 0..30 {
        let n = format!("tok{}", i);
        b = b.token(&n, &n);
    }
    for i in 0..30 {
        let tok = format!("tok{}", i);
        b = b.rule("s", vec![&tok]);
    }
    b = b.start("s");
    let g = b.build();
    assert!(g.all_rules().count() >= 30);
}

#[test]
fn chain_long_rhs_from_loop() {
    let mut b = GrammarBuilder::new("lrhs");
    for i in 0..8 {
        let n = format!("t{}", i);
        b = b.token(&n, &n);
    }
    let rhs: Vec<&str> = (0..8)
        .map(|i| Box::leak(format!("t{}", i).into_boxed_str()) as &str)
        .collect();
    b = b.rule("s", rhs).start("s");
    let g = b.build();
    let r = g.all_rules().next().unwrap();
    assert_eq!(r.rhs.len(), 8);
}

#[test]
fn chain_diamond_shape() {
    let g = GrammarBuilder::new("dia")
        .token("a", "a")
        .rule("l", vec!["a"])
        .rule("r", vec!["a"])
        .rule("s", vec!["l"])
        .rule("s", vec!["r"])
        .start("s")
        .build();
    assert!(g.all_rules().count() >= 4);
}

#[test]
fn chain_deep_nesting() {
    let mut b = GrammarBuilder::new("deep");
    b = b.token("leaf", "x");
    b = b.rule("n0", vec!["leaf"]);
    for i in 1..8 {
        let prev = format!("n{}", i - 1);
        let curr = format!("n{}", i);
        b = b.rule(&curr, vec![&prev]);
    }
    b = b.start("n7");
    let g = b.build();
    assert!(g.all_rules().count() >= 8);
}

#[test]
fn chain_regex_tokens() {
    let g = GrammarBuilder::new("regex")
        .token("num", r"\d+")
        .token("id", r"[a-z]+")
        .rule("s", vec!["num"])
        .start("s")
        .build();
    assert!(g.all_rules().count() >= 1);
}
