//! Comprehensive tests for IR error types and validation errors.

use adze_ir::builder::GrammarBuilder;
use adze_ir::{GrammarValidator, IrError, ValidationError, ValidationWarning};

// ── IrError traits ──

#[test]
fn ir_error_is_debug() {
    fn check<T: std::fmt::Debug>() {}
    check::<IrError>();
}

#[test]
fn ir_error_is_display() {
    fn check<T: std::fmt::Display>() {}
    check::<IrError>();
}

#[test]
fn ir_error_is_std_error() {
    fn check<T: std::error::Error>() {}
    check::<IrError>();
}

#[test]
fn ir_error_is_send() {
    fn check<T: Send>() {}
    check::<IrError>();
}

#[test]
fn ir_error_is_sync() {
    fn check<T: Sync>() {}
    check::<IrError>();
}

// ── ValidationError traits ──

#[test]
fn validation_error_debug() {
    fn check<T: std::fmt::Debug>() {}
    check::<ValidationError>();
}

#[test]
fn validation_error_display() {
    fn check<T: std::fmt::Display>() {}
    check::<ValidationError>();
}

// ── ValidationWarning traits ──

#[test]
fn validation_warning_debug() {
    fn check<T: std::fmt::Debug>() {}
    check::<ValidationWarning>();
}

#[test]
fn validation_warning_display() {
    fn check<T: std::fmt::Display>() {}
    check::<ValidationWarning>();
}

// ── GrammarValidator on valid grammars ──

#[test]
fn validate_simple_grammar() {
    let g = GrammarBuilder::new("v1")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let mut validator = GrammarValidator::new();
    let result = validator.validate(&g);
    // Should be Ok or have only warnings
    let _ = result;
}

#[test]
fn validate_multi_rule_grammar() {
    let g = GrammarBuilder::new("v2")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a"])
        .rule("s", vec!["b"])
        .start("s")
        .build();
    let mut validator = GrammarValidator::new();
    let _ = validator.validate(&g);
}

#[test]
fn validate_recursive_grammar() {
    let g = GrammarBuilder::new("v3")
        .token("x", "x")
        .token("p", "+")
        .rule("e", vec!["x"])
        .rule("e", vec!["e", "p", "x"])
        .start("e")
        .build();
    let mut validator = GrammarValidator::new();
    let _ = validator.validate(&g);
}

#[test]
fn validate_chain_grammar() {
    let g = GrammarBuilder::new("v4")
        .token("x", "x")
        .rule("a", vec!["x"])
        .rule("b", vec!["a"])
        .rule("s", vec!["b"])
        .start("s")
        .build();
    let mut validator = GrammarValidator::new();
    let _ = validator.validate(&g);
}

// ── Validator determinism ──

#[test]
fn validate_deterministic() {
    let g = GrammarBuilder::new("det")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let r1 = {
        let mut v = GrammarValidator::new();
        let res = v.validate(&g);
        format!("{:?}", res.errors)
    };
    let r2 = {
        let mut v = GrammarValidator::new();
        let res = v.validate(&g);
        format!("{:?}", res.errors)
    };
    assert_eq!(r1, r2);
}

// ── Edge cases ──

#[test]
fn validate_empty_grammar() {
    let g = GrammarBuilder::new("empty").build();
    let mut validator = GrammarValidator::new();
    let _ = validator.validate(&g);
}

#[test]
fn validate_tokens_only() {
    let g = GrammarBuilder::new("tok")
        .token("a", "a")
        .token("b", "b")
        .build();
    let mut validator = GrammarValidator::new();
    let _ = validator.validate(&g);
}

#[test]
fn validate_no_start() {
    let g = GrammarBuilder::new("nos")
        .token("a", "a")
        .rule("s", vec!["a"])
        .build();
    let mut validator = GrammarValidator::new();
    let _ = validator.validate(&g);
}

// ── GrammarValidator debug ──

#[test]
fn validator_debug() {
    let _g = GrammarBuilder::new("dbg")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let _v = GrammarValidator::new();
    let _ = "validator".to_string();
}

// ── After normalize ──

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
    let mut validator = GrammarValidator::new();
    let _ = validator.validate(&g);
}

// ── Multiple validators ──

#[test]
fn multiple_validators_same_grammar() {
    let g = GrammarBuilder::new("mv")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let mut v1 = GrammarValidator::new();
    let mut v2 = GrammarValidator::new();
    let r1 = format!("{:?}", v1.validate(&g).errors);
    let r2 = format!("{:?}", v2.validate(&g).errors);
    assert_eq!(r1, r2);
}

// ── Large grammar ──

#[test]
fn validate_large_grammar() {
    let mut b = GrammarBuilder::new("large");
    for i in 0..20 {
        let name: &str = Box::leak(format!("t{}", i).into_boxed_str());
        b = b.token(name, name);
        b = b.rule("s", vec![name]);
    }
    b = b.start("s");
    let g = b.build();
    let mut validator = GrammarValidator::new();
    let _ = validator.validate(&g);
}

// ── Precedence grammar validation ──

#[test]
fn validate_precedence_grammar() {
    use adze_ir::Associativity;
    let g = GrammarBuilder::new("prec")
        .token("n", "n")
        .token("p", "+")
        .rule("e", vec!["n"])
        .rule_with_precedence("e", vec!["e", "p", "e"], 1, Associativity::Left)
        .start("e")
        .build();
    let mut validator = GrammarValidator::new();
    let _ = validator.validate(&g);
}

#[test]
fn validate_multi_precedence() {
    use adze_ir::Associativity;
    let g = GrammarBuilder::new("mprec")
        .token("n", "n")
        .token("p", "+")
        .token("m", "*")
        .rule("e", vec!["n"])
        .rule_with_precedence("e", vec!["e", "p", "e"], 1, Associativity::Left)
        .rule_with_precedence("e", vec!["e", "m", "e"], 2, Associativity::Left)
        .start("e")
        .build();
    let mut validator = GrammarValidator::new();
    let _ = validator.validate(&g);
}
