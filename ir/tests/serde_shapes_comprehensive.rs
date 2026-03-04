//! Comprehensive tests for Grammar serde with various shapes.

use adze_ir::Grammar;
use adze_ir::builder::GrammarBuilder;

fn roundtrip_json(g: &Grammar) -> Grammar {
    let json = serde_json::to_string(g).unwrap();
    serde_json::from_str(&json).unwrap()
}

fn roundtrip_bincode(g: &Grammar) -> Grammar {
    let bytes = bincode::serialize(g).unwrap();
    bincode::deserialize(&bytes).unwrap()
}

#[test]
fn json_simple() {
    let g = GrammarBuilder::new("s1")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let g2 = roundtrip_json(&g);
    assert_eq!(g.name, g2.name);
}

#[test]
fn bincode_simple() {
    let g = GrammarBuilder::new("s2")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let g2 = roundtrip_bincode(&g);
    assert_eq!(g.name, g2.name);
}

#[test]
fn json_two_alts() {
    let g = GrammarBuilder::new("a2")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a"])
        .rule("s", vec!["b"])
        .start("s")
        .build();
    let g2 = roundtrip_json(&g);
    assert_eq!(g.all_rules().count(), g2.all_rules().count());
}

#[test]
fn bincode_two_alts() {
    let g = GrammarBuilder::new("a3")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a"])
        .rule("s", vec!["b"])
        .start("s")
        .build();
    let g2 = roundtrip_bincode(&g);
    assert_eq!(g.all_rules().count(), g2.all_rules().count());
}

#[test]
fn json_preserves_start() {
    let g = GrammarBuilder::new("st")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let g2 = roundtrip_json(&g);
    assert_eq!(g.start_symbol(), g2.start_symbol());
}

#[test]
fn bincode_preserves_start() {
    let g = GrammarBuilder::new("st2")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let g2 = roundtrip_bincode(&g);
    assert_eq!(g.start_symbol(), g2.start_symbol());
}

#[test]
fn json_chain() {
    let g = GrammarBuilder::new("ch")
        .token("x", "x")
        .rule("inner", vec!["x"])
        .rule("s", vec!["inner"])
        .start("s")
        .build();
    let g2 = roundtrip_json(&g);
    assert_eq!(g.name, g2.name);
    assert_eq!(g.all_rules().count(), g2.all_rules().count());
}

#[test]
fn json_large() {
    let mut b = GrammarBuilder::new("big");
    for i in 0..30 {
        let n = format!("t{}", i);
        b = b.token(&n, &n);
    }
    for i in 0..30 {
        let tok = format!("t{}", i);
        b = b.rule("s", vec![&tok]);
    }
    b = b.start("s");
    let g = b.build();
    let g2 = roundtrip_json(&g);
    assert_eq!(g.all_rules().count(), g2.all_rules().count());
}

#[test]
fn bincode_large() {
    let mut b = GrammarBuilder::new("big2");
    for i in 0..30 {
        let n = format!("t{}", i);
        b = b.token(&n, &n);
    }
    for i in 0..30 {
        let tok = format!("t{}", i);
        b = b.rule("s", vec![&tok]);
    }
    b = b.start("s");
    let g = b.build();
    let g2 = roundtrip_bincode(&g);
    assert_eq!(g.all_rules().count(), g2.all_rules().count());
}

#[test]
fn json_after_normalize() {
    let mut g = GrammarBuilder::new("norm")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    g.normalize();
    let g2 = roundtrip_json(&g);
    assert_eq!(g.name, g2.name);
}

#[test]
fn json_pretty() {
    let g = GrammarBuilder::new("pretty")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let json = serde_json::to_string_pretty(&g).unwrap();
    assert!(json.contains('\n'));
    let g2: Grammar = serde_json::from_str(&json).unwrap();
    assert_eq!(g.name, g2.name);
}

#[test]
fn json_deterministic() {
    let build = || {
        GrammarBuilder::new("det")
            .token("a", "a")
            .rule("s", vec!["a"])
            .start("s")
            .build()
    };
    let j1 = serde_json::to_string(&build()).unwrap();
    let j2 = serde_json::to_string(&build()).unwrap();
    assert_eq!(j1, j2);
}
