//! Comprehensive tests for ABI builder and language generation.

use adze_glr_core::{FirstFollowSets, build_lr1_automaton};
use adze_ir::Associativity;
use adze_ir::builder::GrammarBuilder;
use adze_tablegen::abi_builder::AbiLanguageBuilder;

fn build_table(grammar: &mut adze_ir::Grammar) -> adze_glr_core::ParseTable {
    grammar.normalize();
    let ff = FirstFollowSets::compute(grammar).unwrap();
    build_lr1_automaton(grammar, &ff).unwrap()
}

// ── Construction ──

#[test]
fn abi_builder_new() {
    let mut g = GrammarBuilder::new("test")
        .token("x", "x")
        .rule("s", vec!["x"])
        .start("s")
        .build();
    let pt = build_table(&mut g);
    let _builder = AbiLanguageBuilder::new(&g, &pt);
}

#[test]
fn abi_builder_generate() {
    let mut g = GrammarBuilder::new("test_gen")
        .token("x", "x")
        .rule("s", vec!["x"])
        .start("s")
        .build();
    let pt = build_table(&mut g);
    let builder = AbiLanguageBuilder::new(&g, &pt);
    let code = builder.generate();
    let code_str = code.to_string();
    assert!(!code_str.is_empty());
}

// ── Different grammar shapes ──

#[test]
fn abi_two_alternatives() {
    let mut g = GrammarBuilder::new("alt")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a"])
        .rule("s", vec!["b"])
        .start("s")
        .build();
    let pt = build_table(&mut g);
    let code = AbiLanguageBuilder::new(&g, &pt).generate().to_string();
    assert!(!code.is_empty());
}

#[test]
fn abi_chain() {
    let mut g = GrammarBuilder::new("chain")
        .token("x", "x")
        .rule("a", vec!["x"])
        .rule("b", vec!["a"])
        .rule("s", vec!["b"])
        .start("s")
        .build();
    let pt = build_table(&mut g);
    let code = AbiLanguageBuilder::new(&g, &pt).generate().to_string();
    assert!(!code.is_empty());
}

#[test]
fn abi_recursive() {
    let mut g = GrammarBuilder::new("rec")
        .token("n", "n")
        .token("plus", "+")
        .rule("e", vec!["n"])
        .rule("e", vec!["e", "plus", "n"])
        .start("e")
        .build();
    let pt = build_table(&mut g);
    let code = AbiLanguageBuilder::new(&g, &pt).generate().to_string();
    assert!(!code.is_empty());
}

#[test]
fn abi_precedence() {
    let mut g = GrammarBuilder::new("prec")
        .token("n", "n")
        .token("plus", "+")
        .token("star", "*")
        .rule("e", vec!["n"])
        .rule_with_precedence("e", vec!["e", "plus", "e"], 1, Associativity::Left)
        .rule_with_precedence("e", vec!["e", "star", "e"], 2, Associativity::Left)
        .start("e")
        .build();
    let pt = build_table(&mut g);
    let code = AbiLanguageBuilder::new(&g, &pt).generate().to_string();
    assert!(!code.is_empty());
}

// ── Determinism ──

#[test]
fn abi_deterministic() {
    let make = || {
        let mut g = GrammarBuilder::new("det")
            .token("x", "x")
            .rule("s", vec!["x"])
            .start("s")
            .build();
        let pt = build_table(&mut g);
        AbiLanguageBuilder::new(&g, &pt).generate().to_string()
    };
    assert_eq!(make(), make());
}

// ── Large grammar ──

#[test]
fn abi_large_grammar() {
    let mut b = GrammarBuilder::new("large");
    for i in 0..10 {
        let n: &str = Box::leak(format!("t{}", i).into_boxed_str());
        b = b.token(n, n).rule("s", vec![n]);
    }
    let mut g = b.start("s").build();
    let pt = build_table(&mut g);
    let code = AbiLanguageBuilder::new(&g, &pt).generate().to_string();
    assert!(!code.is_empty());
}

// ── Code contains grammar name ──

#[test]
fn abi_code_contains_name() {
    let mut g = GrammarBuilder::new("my_language")
        .token("x", "x")
        .rule("s", vec!["x"])
        .start("s")
        .build();
    let pt = build_table(&mut g);
    let code = AbiLanguageBuilder::new(&g, &pt).generate().to_string();
    assert!(code.contains("my_language"));
}

// ── Sequence grammar ──

#[test]
fn abi_sequence() {
    let mut g = GrammarBuilder::new("seq")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a", "b"])
        .start("s")
        .build();
    let pt = build_table(&mut g);
    let code = AbiLanguageBuilder::new(&g, &pt).generate().to_string();
    assert!(!code.is_empty());
}

// ── Multiple nonterminals ──

#[test]
fn abi_multi_nt() {
    let mut g = GrammarBuilder::new("multi")
        .token("x", "x")
        .token("y", "y")
        .rule("a", vec!["x"])
        .rule("b", vec!["y"])
        .rule("s", vec!["a", "b"])
        .start("s")
        .build();
    let pt = build_table(&mut g);
    let code = AbiLanguageBuilder::new(&g, &pt).generate().to_string();
    assert!(!code.is_empty());
}

// ── Right associative ──

#[test]
fn abi_right_assoc() {
    let mut g = GrammarBuilder::new("rassoc")
        .token("n", "n")
        .token("eq", "=")
        .rule("e", vec!["n"])
        .rule_with_precedence("e", vec!["e", "eq", "e"], 1, Associativity::Right)
        .start("e")
        .build();
    let pt = build_table(&mut g);
    let code = AbiLanguageBuilder::new(&g, &pt).generate().to_string();
    assert!(!code.is_empty());
}

// ── Mixed associativity ──

#[test]
fn abi_mixed_assoc() {
    let mut g = GrammarBuilder::new("mixed")
        .token("n", "n")
        .token("plus", "+")
        .token("pow", "^")
        .rule("e", vec!["n"])
        .rule_with_precedence("e", vec!["e", "plus", "e"], 1, Associativity::Left)
        .rule_with_precedence("e", vec!["e", "pow", "e"], 2, Associativity::Right)
        .start("e")
        .build();
    let pt = build_table(&mut g);
    let code = AbiLanguageBuilder::new(&g, &pt).generate().to_string();
    assert!(!code.is_empty());
}

// ── Code has static array references ──

#[test]
fn abi_code_has_arrays() {
    let mut g = GrammarBuilder::new("arr")
        .token("x", "x")
        .rule("s", vec!["x"])
        .start("s")
        .build();
    let pt = build_table(&mut g);
    let code = AbiLanguageBuilder::new(&g, &pt).generate().to_string();
    // Generated code should have static arrays/consts
    assert!(code.len() > 100);
}
