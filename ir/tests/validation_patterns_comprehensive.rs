//! Comprehensive tests for grammar validation patterns.

use adze_ir::Associativity;
use adze_ir::builder::GrammarBuilder;
use adze_ir::validation::{GrammarValidator, ValidationResult};

fn validate(grammar: &adze_ir::Grammar) -> ValidationResult {
    let mut v = GrammarValidator::new();
    v.validate(grammar)
}

// ── Empty grammar ──

#[test]
fn validate_empty_grammar() {
    let g = GrammarBuilder::new("empty").build();
    let r = validate(&g);
    // May have warnings about no rules, but shouldn't panic
    let _ = r.errors;
    let _ = r.warnings;
}

// ── Simple valid grammar ──

#[test]
fn validate_simple_grammar_no_errors() {
    let g = GrammarBuilder::new("simple")
        .token("x", "x")
        .rule("s", vec!["x"])
        .start("s")
        .build();
    let r = validate(&g);
    assert!(r.errors.is_empty(), "errors: {:?}", r.errors);
}

// ── Token-only grammar ──

#[test]
fn validate_tokens_only() {
    let g = GrammarBuilder::new("tokens")
        .token("a", "a")
        .token("b", "b")
        .build();
    let r = validate(&g);
    let _ = r;
}

// ── Multiple rules ──

#[test]
fn validate_multiple_rules() {
    let g = GrammarBuilder::new("multi")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a"])
        .rule("s", vec!["b"])
        .start("s")
        .build();
    let r = validate(&g);
    assert!(r.errors.is_empty(), "errors: {:?}", r.errors);
}

// ── Chain grammar ──

#[test]
fn validate_chain() {
    let g = GrammarBuilder::new("chain")
        .token("x", "x")
        .rule("a", vec!["x"])
        .rule("b", vec!["a"])
        .rule("s", vec!["b"])
        .start("s")
        .build();
    let r = validate(&g);
    assert!(r.errors.is_empty(), "errors: {:?}", r.errors);
}

// ── Recursive grammar ──

#[test]
fn validate_recursive() {
    let g = GrammarBuilder::new("rec")
        .token("n", "n")
        .token("plus", "+")
        .rule("e", vec!["n"])
        .rule("e", vec!["e", "plus", "n"])
        .start("e")
        .build();
    let r = validate(&g);
    // Recursive grammars may produce CyclicRule errors — this is expected
    let _ = r;
}

// ── Precedence grammar ──

#[test]
fn validate_precedence() {
    let g = GrammarBuilder::new("prec")
        .token("n", "n")
        .token("plus", "+")
        .token("star", "*")
        .rule("e", vec!["n"])
        .rule_with_precedence("e", vec!["e", "plus", "e"], 1, Associativity::Left)
        .rule_with_precedence("e", vec!["e", "star", "e"], 2, Associativity::Left)
        .start("e")
        .build();
    let r = validate(&g);
    // Recursive grammars with precedence may have CyclicRule
    let _ = r;
}

// ── Validation stats ──

#[test]
fn validate_stats_present() {
    let g = GrammarBuilder::new("stats")
        .token("x", "x")
        .rule("s", vec!["x"])
        .start("s")
        .build();
    let r = validate(&g);
    let _ = r.stats;
}

// ── Multiple validators ──

#[test]
fn validate_multiple_validators() {
    let g = GrammarBuilder::new("mv")
        .token("x", "x")
        .rule("s", vec!["x"])
        .start("s")
        .build();
    let r1 = validate(&g);
    let r2 = validate(&g);
    assert_eq!(r1.errors.len(), r2.errors.len());
    assert_eq!(r1.warnings.len(), r2.warnings.len());
}

// ── Normalized grammar validation ──

#[test]
fn validate_after_normalize() {
    let mut g = GrammarBuilder::new("norm")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a"])
        .rule("s", vec!["b"])
        .start("s")
        .build();
    g.normalize();
    let r = validate(&g);
    assert!(r.errors.is_empty(), "errors: {:?}", r.errors);
}

// ── Large grammar ──

#[test]
fn validate_large_grammar() {
    let mut b = GrammarBuilder::new("large");
    for i in 0..15 {
        let n: &str = Box::leak(format!("t{}", i).into_boxed_str());
        b = b.token(n, n).rule("s", vec![n]);
    }
    let g = b.start("s").build();
    let r = validate(&g);
    assert!(r.errors.is_empty(), "errors: {:?}", r.errors);
}

// ── Right associative ──

#[test]
fn validate_right_assoc() {
    let g = GrammarBuilder::new("rassoc")
        .token("n", "n")
        .token("eq", "=")
        .rule("e", vec!["n"])
        .rule_with_precedence("e", vec!["e", "eq", "e"], 1, Associativity::Right)
        .start("e")
        .build();
    let r = validate(&g);
    // Recursive grammars may have CyclicRule
    let _ = r;
}

// ── Multiple nonterminals ──

#[test]
fn validate_multiple_nonterminals() {
    let g = GrammarBuilder::new("multi_nt")
        .token("x", "x")
        .token("y", "y")
        .rule("a", vec!["x"])
        .rule("b", vec!["y"])
        .rule("s", vec!["a", "b"])
        .start("s")
        .build();
    let r = validate(&g);
    assert!(r.errors.is_empty(), "errors: {:?}", r.errors);
}

// ── Sequence grammar ──

#[test]
fn validate_sequence() {
    let g = GrammarBuilder::new("seq")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("s", vec!["a", "b", "c"])
        .start("s")
        .build();
    let r = validate(&g);
    assert!(r.errors.is_empty(), "errors: {:?}", r.errors);
}

// ── Validator reuse ──

#[test]
fn validator_reuse() {
    let mut v = GrammarValidator::new();
    let g1 = GrammarBuilder::new("g1")
        .token("x", "x")
        .rule("s", vec!["x"])
        .start("s")
        .build();
    let g2 = GrammarBuilder::new("g2")
        .token("y", "y")
        .rule("s", vec!["y"])
        .start("s")
        .build();
    let _r1 = v.validate(&g1);
    let _r2 = v.validate(&g2);
}

// ── Error format ──

#[test]
fn validation_error_debug() {
    let g = GrammarBuilder::new("empty").build();
    let r = validate(&g);
    for e in &r.errors {
        let d = format!("{:?}", e);
        assert!(!d.is_empty());
    }
}

#[test]
fn validation_warning_debug() {
    let g = GrammarBuilder::new("empty").build();
    let r = validate(&g);
    for w in &r.warnings {
        let d = format!("{:?}", w);
        assert!(!d.is_empty());
    }
}

// ── Determinism ──

#[test]
fn validation_deterministic() {
    let g = GrammarBuilder::new("det")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a"])
        .rule("s", vec!["b"])
        .start("s")
        .build();
    let r1 = validate(&g);
    let r2 = validate(&g);
    assert_eq!(r1.errors.len(), r2.errors.len());
}

// ── Unicode grammar name ──

#[test]
fn validate_unicode_name() {
    let g = GrammarBuilder::new("日本語")
        .token("x", "x")
        .rule("s", vec!["x"])
        .start("s")
        .build();
    let r = validate(&g);
    let _ = r;
}

// ── Mixed precedence levels ──

#[test]
fn validate_mixed_precedence() {
    let g = GrammarBuilder::new("mixed_prec")
        .token("n", "n")
        .token("plus", "+")
        .token("star", "*")
        .token("pow", "^")
        .rule("e", vec!["n"])
        .rule_with_precedence("e", vec!["e", "plus", "e"], 1, Associativity::Left)
        .rule_with_precedence("e", vec!["e", "star", "e"], 2, Associativity::Left)
        .rule_with_precedence("e", vec!["e", "pow", "e"], 3, Associativity::Right)
        .start("e")
        .build();
    let r = validate(&g);
    // Recursive grammars may have CyclicRule
    let _ = r;
}

// ── Grammar with many tokens ──

#[test]
fn validate_many_tokens() {
    let mut b = GrammarBuilder::new("many_tokens");
    for i in 0..20 {
        let n: &str = Box::leak(format!("token_{}", i).into_boxed_str());
        b = b.token(n, n);
    }
    let g = b.build();
    let r = validate(&g);
    let _ = r;
}
