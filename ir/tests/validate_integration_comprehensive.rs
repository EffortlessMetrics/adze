//! Comprehensive tests for Grammar validate() integration patterns.

use adze_ir::builder::GrammarBuilder;
use adze_ir::validation::GrammarValidator;

fn validate_grammar(
    name: &str,
    tokens: &[(&str, &str)],
    rules: &[(&str, Vec<&str>)],
    start: &str,
) -> adze_ir::validation::ValidationResult {
    let mut b = GrammarBuilder::new(name);
    for (n, p) in tokens {
        b = b.token(n, p);
    }
    for (lhs, rhs) in rules {
        b = b.rule(lhs, rhs.clone());
    }
    b = b.start(start);
    let g = b.build();
    let mut v = GrammarValidator::new();
    v.validate(&g)
}

#[test]
fn valid_minimal() {
    let r = validate_grammar("min", &[("a", "a")], &[("s", vec!["a"])], "s");
    assert!(r.errors.is_empty());
}

#[test]
fn valid_two_alts() {
    let r = validate_grammar(
        "two",
        &[("a", "a"), ("b", "b")],
        &[("s", vec!["a"]), ("s", vec!["b"])],
        "s",
    );
    assert!(r.errors.is_empty());
}

#[test]
fn valid_chain() {
    let r = validate_grammar(
        "chain",
        &[("x", "x")],
        &[("inner", vec!["x"]), ("s", vec!["inner"])],
        "s",
    );
    assert!(r.errors.is_empty());
}

#[test]
fn valid_multi_rhs() {
    let r = validate_grammar(
        "multi",
        &[("a", "a"), ("b", "b")],
        &[("s", vec!["a", "b"])],
        "s",
    );
    assert!(r.errors.is_empty());
}

#[test]
fn valid_diamond() {
    let r = validate_grammar(
        "dia",
        &[("a", "a")],
        &[
            ("l", vec!["a"]),
            ("r", vec!["a"]),
            ("s", vec!["l"]),
            ("s", vec!["r"]),
        ],
        "s",
    );
    assert!(r.errors.is_empty());
}

#[test]
fn valid_many_tokens() {
    let mut b = GrammarBuilder::new("many");
    for i in 0..20 {
        let n = format!("t{}", i);
        b = b.token(&n, &n);
    }
    b = b.rule("s", vec!["t0"]).start("s");
    let g = b.build();
    let mut v = GrammarValidator::new();
    let r = v.validate(&g);
    assert!(r.errors.is_empty());
}

#[test]
fn validation_result_has_warnings() {
    let r = validate_grammar("w", &[("a", "a")], &[("s", vec!["a"])], "s");
    let _ = r.warnings;
}

#[test]
fn validation_result_has_stats() {
    let r = validate_grammar("st", &[("a", "a")], &[("s", vec!["a"])], "s");
    let _ = r.stats;
}

#[test]
fn validator_reusable() {
    let mut v = GrammarValidator::new();
    let g1 = GrammarBuilder::new("g1")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let g2 = GrammarBuilder::new("g2")
        .token("b", "b")
        .rule("s", vec!["b"])
        .start("s")
        .build();
    let r1 = v.validate(&g1);
    let r2 = v.validate(&g2);
    assert!(r1.errors.is_empty());
    assert!(r2.errors.is_empty());
}

#[test]
fn valid_with_regex_token() {
    let r = validate_grammar("regex", &[("num", r"\d+")], &[("s", vec!["num"])], "s");
    assert!(r.errors.is_empty());
}

#[test]
fn valid_deep_chain() {
    let r = validate_grammar(
        "deep",
        &[("x", "x")],
        &[("a", vec!["x"]), ("b", vec!["a"]), ("s", vec!["b"])],
        "s",
    );
    assert!(r.errors.is_empty());
}

#[test]
fn validate_after_normalize() {
    let mut g = GrammarBuilder::new("norm")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    g.normalize();
    let mut v = GrammarValidator::new();
    let r = v.validate(&g);
    assert!(r.errors.is_empty());
}
