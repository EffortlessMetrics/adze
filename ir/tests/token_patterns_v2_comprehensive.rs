// Comprehensive tests for Token and TokenPattern in adze-ir
// Tests token construction and pattern matching

use adze_ir::builder::GrammarBuilder;

#[test]
fn token_simple_string() {
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    assert_eq!(g.tokens.len(), 1);
}

#[test]
fn token_regex_pattern() {
    let g = GrammarBuilder::new("t")
        .token("num", "[0-9]+")
        .rule("s", vec!["num"])
        .start("s")
        .build();
    assert_eq!(g.tokens.len(), 1);
}

#[test]
fn token_multiple() {
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    assert_eq!(g.tokens.len(), 3);
}

#[test]
fn token_special_chars() {
    let g = GrammarBuilder::new("t")
        .token("plus", r"\+")
        .token("star", r"\*")
        .rule("s", vec!["plus"])
        .start("s")
        .build();
    assert_eq!(g.tokens.len(), 2);
}

#[test]
fn token_preserved_after_clone() {
    let g = GrammarBuilder::new("t")
        .token("x", "x")
        .rule("s", vec!["x"])
        .start("s")
        .build();
    let g2 = g.clone();
    assert_eq!(g.tokens.len(), g2.tokens.len());
}

#[test]
fn token_preserved_after_normalize() {
    let mut g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a", "b"])
        .start("s")
        .build();
    let before = g.tokens.len();
    g.normalize();
    assert_eq!(g.tokens.len(), before);
}

#[test]
fn token_serde_roundtrip() {
    let g = GrammarBuilder::new("t")
        .token("x", "[a-z]+")
        .rule("s", vec!["x"])
        .start("s")
        .build();
    let json = serde_json::to_string(&g).unwrap();
    let g2: adze_ir::Grammar = serde_json::from_str(&json).unwrap();
    assert_eq!(g.tokens.len(), g2.tokens.len());
}

#[test]
fn token_debug_format() {
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    for (_, tok) in &g.tokens {
        let dbg = format!("{:?}", tok);
        assert!(!dbg.is_empty());
    }
}

#[test]
fn token_pattern_long_regex() {
    let g = GrammarBuilder::new("t")
        .token("complex", "[a-zA-Z_][a-zA-Z0-9_]*")
        .rule("s", vec!["complex"])
        .start("s")
        .build();
    assert_eq!(g.tokens.len(), 1);
}

#[test]
fn token_ids_are_unique() {
    let g = GrammarBuilder::new("t")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let ids: Vec<_> = g.tokens.keys().collect();
    let unique: std::collections::HashSet<_> = ids.iter().collect();
    assert_eq!(ids.len(), unique.len());
}
