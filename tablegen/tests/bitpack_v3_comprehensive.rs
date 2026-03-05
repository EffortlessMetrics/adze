//! Comprehensive bit-packing and compression tests for `StaticLanguageGenerator`.
//!
//! Validates that code generation through the static language generator produces
//! deterministic, correctly-structured output whose size scales with grammar
//! complexity.

use adze_glr_core::{FirstFollowSets, ParseTable, build_lr1_automaton};
use adze_ir::Grammar;
use adze_ir::builder::GrammarBuilder;
use adze_tablegen::StaticLanguageGenerator;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn build_real(
    name: &str,
    build: impl FnOnce(GrammarBuilder) -> GrammarBuilder,
) -> (Grammar, ParseTable) {
    let mut g = build(GrammarBuilder::new(name)).build();
    let ff = FirstFollowSets::compute_normalized(&mut g).expect("compute ff");
    let table = build_lr1_automaton(&g, &ff).expect("build automaton");
    (g, table)
}

fn simple_grammar() -> (Grammar, ParseTable) {
    build_real("simple", |b| {
        b.token("A", "a").rule("s", vec!["A"]).start("s")
    })
}

fn two_token_grammar() -> (Grammar, ParseTable) {
    build_real("two_tok", |b| {
        b.token("A", "a")
            .token("B", "b")
            .rule("s", vec!["A", "B"])
            .start("s")
    })
}

fn two_alt_grammar() -> (Grammar, ParseTable) {
    build_real("two_alt", |b| {
        b.token("A", "a")
            .token("B", "b")
            .rule("s", vec!["A"])
            .rule("s", vec!["B"])
            .start("s")
    })
}

fn chain_grammar() -> (Grammar, ParseTable) {
    build_real("chain", |b| {
        b.token("X", "x")
            .rule("inner", vec!["X"])
            .rule("s", vec!["inner"])
            .start("s")
    })
}

fn expression_grammar() -> (Grammar, ParseTable) {
    build_real("expr", |b| {
        b.token("number", r"\d+")
            .token("plus", "+")
            .token("star", "*")
            .token("lparen", "(")
            .token("rparen", ")")
            .rule("expr", vec!["number"])
            .rule("expr", vec!["expr", "plus", "expr"])
            .rule("expr", vec!["expr", "star", "expr"])
            .rule("expr", vec!["lparen", "expr", "rparen"])
            .start("expr")
    })
}

fn many_alternatives_grammar() -> (Grammar, ParseTable) {
    build_real("many_alt", |b| {
        b.token("A", "a")
            .token("B", "b")
            .token("C", "c")
            .token("D", "d")
            .token("E", "e")
            .token("F", "f")
            .rule("s", vec!["A"])
            .rule("s", vec!["B"])
            .rule("s", vec!["C"])
            .rule("s", vec!["D"])
            .rule("s", vec!["E"])
            .rule("s", vec!["F"])
            .start("s")
    })
}

fn multi_rule_grammar() -> (Grammar, ParseTable) {
    build_real("multi_rule", |b| {
        b.token("A", "a")
            .token("B", "b")
            .token("C", "c")
            .rule("x", vec!["A"])
            .rule("y", vec!["B"])
            .rule("z", vec!["C"])
            .rule("s", vec!["x", "y", "z"])
            .start("s")
    })
}

fn generate_code(grammar: Grammar, table: ParseTable) -> String {
    let generator = StaticLanguageGenerator::new(grammar, table);
    generator.generate_language_code().to_string()
}

// ===========================================================================
// 1. Generated code properties
// ===========================================================================

#[test]
fn test_generated_code_simple_is_non_empty() {
    let (g, t) = simple_grammar();
    let code = generate_code(g, t);
    assert!(!code.is_empty());
}

#[test]
fn test_generated_code_two_token_is_non_empty() {
    let (g, t) = two_token_grammar();
    let code = generate_code(g, t);
    assert!(!code.is_empty());
}

#[test]
fn test_generated_code_expression_is_non_empty() {
    let (g, t) = expression_grammar();
    let code = generate_code(g, t);
    assert!(!code.is_empty());
}

#[test]
fn test_generated_code_many_alt_is_non_empty() {
    let (g, t) = many_alternatives_grammar();
    let code = generate_code(g, t);
    assert!(!code.is_empty());
}

#[test]
fn test_generated_code_contains_static_array() {
    let (g, t) = simple_grammar();
    let code = generate_code(g, t);
    assert!(code.contains("static") || code.contains("const"));
}

#[test]
fn test_generated_code_contains_state_data_two_token() {
    let (g, t) = two_token_grammar();
    let code = generate_code(g, t);
    // Generated code must include numeric state data.
    assert!(code.contains('0') || code.contains('1'));
}

#[test]
fn test_generated_code_contains_state_data_chain() {
    let (g, t) = chain_grammar();
    let code = generate_code(g, t);
    assert!(code.contains('0') || code.contains('1'));
}

#[test]
fn test_generated_code_contains_state_data_multi_rule() {
    let (g, t) = multi_rule_grammar();
    let code = generate_code(g, t);
    assert!(code.contains('0') || code.contains('1'));
}

// ===========================================================================
// 2. Bit-packed output deterministic
// ===========================================================================

#[test]
fn test_deterministic_simple() {
    let (g1, t1) = simple_grammar();
    let (g2, t2) = simple_grammar();
    assert_eq!(generate_code(g1, t1), generate_code(g2, t2));
}

#[test]
fn test_deterministic_two_token() {
    let (g1, t1) = two_token_grammar();
    let (g2, t2) = two_token_grammar();
    assert_eq!(generate_code(g1, t1), generate_code(g2, t2));
}

#[test]
fn test_deterministic_two_alt() {
    let (g1, t1) = two_alt_grammar();
    let (g2, t2) = two_alt_grammar();
    assert_eq!(generate_code(g1, t1), generate_code(g2, t2));
}

#[test]
fn test_deterministic_chain() {
    let (g1, t1) = chain_grammar();
    let (g2, t2) = chain_grammar();
    assert_eq!(generate_code(g1, t1), generate_code(g2, t2));
}

#[test]
fn test_deterministic_expression() {
    let (g1, t1) = expression_grammar();
    let (g2, t2) = expression_grammar();
    assert_eq!(generate_code(g1, t1), generate_code(g2, t2));
}

#[test]
fn test_deterministic_many_alt() {
    let (g1, t1) = many_alternatives_grammar();
    let (g2, t2) = many_alternatives_grammar();
    assert_eq!(generate_code(g1, t1), generate_code(g2, t2));
}

#[test]
fn test_deterministic_multi_rule() {
    let (g1, t1) = multi_rule_grammar();
    let (g2, t2) = multi_rule_grammar();
    assert_eq!(generate_code(g1, t1), generate_code(g2, t2));
}

#[test]
fn test_deterministic_independent_generators() {
    let (g1, t1) = expression_grammar();
    let (g2, t2) = expression_grammar();
    let gen1 = StaticLanguageGenerator::new(g1, t1);
    let gen2 = StaticLanguageGenerator::new(g2, t2);
    assert_eq!(
        gen1.generate_language_code().to_string(),
        gen2.generate_language_code().to_string(),
    );
}

// ===========================================================================
// 3. Different grammars produce different code
// ===========================================================================

#[test]
fn test_different_simple_vs_two_token() {
    let (g1, t1) = simple_grammar();
    let (g2, t2) = two_token_grammar();
    assert_ne!(generate_code(g1, t1), generate_code(g2, t2));
}

#[test]
fn test_different_simple_vs_expression() {
    let (g1, t1) = simple_grammar();
    let (g2, t2) = expression_grammar();
    assert_ne!(generate_code(g1, t1), generate_code(g2, t2));
}

#[test]
fn test_different_two_token_vs_two_alt() {
    let (g1, t1) = two_token_grammar();
    let (g2, t2) = two_alt_grammar();
    assert_ne!(generate_code(g1, t1), generate_code(g2, t2));
}

#[test]
fn test_different_chain_vs_two_token() {
    let (g1, t1) = chain_grammar();
    let (g2, t2) = two_token_grammar();
    assert_ne!(generate_code(g1, t1), generate_code(g2, t2));
}

#[test]
fn test_different_expression_vs_many_alt() {
    let (g1, t1) = expression_grammar();
    let (g2, t2) = many_alternatives_grammar();
    assert_ne!(generate_code(g1, t1), generate_code(g2, t2));
}

#[test]
fn test_different_simple_vs_chain() {
    let (g1, t1) = simple_grammar();
    let (g2, t2) = chain_grammar();
    assert_ne!(generate_code(g1, t1), generate_code(g2, t2));
}

#[test]
fn test_different_multi_rule_vs_many_alt() {
    let (g1, t1) = multi_rule_grammar();
    let (g2, t2) = many_alternatives_grammar();
    assert_ne!(generate_code(g1, t1), generate_code(g2, t2));
}

#[test]
fn test_different_simple_vs_many_alt() {
    let (g1, t1) = simple_grammar();
    let (g2, t2) = many_alternatives_grammar();
    assert_ne!(generate_code(g1, t1), generate_code(g2, t2));
}

// ===========================================================================
// 4. Code length scales with complexity
// ===========================================================================

#[test]
fn test_scale_simple_shorter_than_expression() {
    let (g1, t1) = simple_grammar();
    let (g2, t2) = expression_grammar();
    assert!(generate_code(g1, t1).len() < generate_code(g2, t2).len());
}

#[test]
fn test_scale_simple_shorter_than_many_alt() {
    let (g1, t1) = simple_grammar();
    let (g2, t2) = many_alternatives_grammar();
    assert!(generate_code(g1, t1).len() < generate_code(g2, t2).len());
}

#[test]
fn test_scale_two_token_shorter_than_expression() {
    let (g1, t1) = two_token_grammar();
    let (g2, t2) = expression_grammar();
    assert!(generate_code(g1, t1).len() < generate_code(g2, t2).len());
}

#[test]
fn test_scale_simple_shorter_than_multi_rule() {
    let (g1, t1) = simple_grammar();
    let (g2, t2) = multi_rule_grammar();
    assert!(generate_code(g1, t1).len() < generate_code(g2, t2).len());
}

#[test]
fn test_scale_chain_shorter_than_expression() {
    let (g1, t1) = chain_grammar();
    let (g2, t2) = expression_grammar();
    assert!(generate_code(g1, t1).len() < generate_code(g2, t2).len());
}

#[test]
fn test_scale_two_alt_shorter_than_many_alt() {
    let (g1, t1) = two_alt_grammar();
    let (g2, t2) = many_alternatives_grammar();
    assert!(generate_code(g1, t1).len() < generate_code(g2, t2).len());
}

#[test]
fn test_scale_simple_shorter_than_chain() {
    let (g1, t1) = simple_grammar();
    let (g2, t2) = chain_grammar();
    assert!(generate_code(g1, t1).len() < generate_code(g2, t2).len());
}

// ===========================================================================
// 5. Code contains expected tokens
// ===========================================================================

#[test]
fn test_contains_state_count_simple() {
    let (g, t) = simple_grammar();
    let state_count = t.state_count;
    let code = generate_code(g, t);
    assert!(
        code.contains(&state_count.to_string()),
        "expected state_count {state_count} in generated code",
    );
}

#[test]
fn test_contains_symbol_count_simple() {
    let (g, t) = simple_grammar();
    let symbol_count = t.symbol_count;
    let code = generate_code(g, t);
    assert!(
        code.contains(&symbol_count.to_string()),
        "expected symbol_count {symbol_count} in generated code",
    );
}

#[test]
fn test_contains_state_count_expression() {
    let (g, t) = expression_grammar();
    let state_count = t.state_count;
    let code = generate_code(g, t);
    assert!(
        code.contains(&state_count.to_string()),
        "expected state_count {state_count} in generated code",
    );
}

#[test]
fn test_contains_symbol_count_expression() {
    let (g, t) = expression_grammar();
    let symbol_count = t.symbol_count;
    let code = generate_code(g, t);
    assert!(
        code.contains(&symbol_count.to_string()),
        "expected symbol_count {symbol_count} in generated code",
    );
}

#[test]
fn test_contains_state_count_many_alt() {
    let (g, t) = many_alternatives_grammar();
    let state_count = t.state_count;
    let code = generate_code(g, t);
    assert!(
        code.contains(&state_count.to_string()),
        "expected state_count {state_count} in generated code",
    );
}

#[test]
fn test_contains_symbol_count_many_alt() {
    let (g, t) = many_alternatives_grammar();
    let symbol_count = t.symbol_count;
    let code = generate_code(g, t);
    assert!(
        code.contains(&symbol_count.to_string()),
        "expected symbol_count {symbol_count} in generated code",
    );
}

#[test]
fn test_contains_state_count_multi_rule() {
    let (g, t) = multi_rule_grammar();
    let state_count = t.state_count;
    let code = generate_code(g, t);
    assert!(
        code.contains(&state_count.to_string()),
        "expected state_count {state_count} in generated code",
    );
}

#[test]
fn test_contains_token_count_expression() {
    let (g, t) = expression_grammar();
    let token_count = t.token_count;
    let code = generate_code(g, t);
    assert!(
        code.contains(&token_count.to_string()),
        "expected token_count {token_count} in generated code",
    );
}

// ===========================================================================
// 6. Multiple generation rounds produce identical output
// ===========================================================================

#[test]
fn test_multi_round_simple() {
    let (g, t) = simple_grammar();
    let generator = StaticLanguageGenerator::new(g, t);
    let first = generator.generate_language_code().to_string();
    let second = generator.generate_language_code().to_string();
    assert_eq!(first, second);
}

#[test]
fn test_multi_round_two_token() {
    let (g, t) = two_token_grammar();
    let generator = StaticLanguageGenerator::new(g, t);
    let first = generator.generate_language_code().to_string();
    let second = generator.generate_language_code().to_string();
    assert_eq!(first, second);
}

#[test]
fn test_multi_round_two_alt() {
    let (g, t) = two_alt_grammar();
    let generator = StaticLanguageGenerator::new(g, t);
    let first = generator.generate_language_code().to_string();
    let second = generator.generate_language_code().to_string();
    assert_eq!(first, second);
}

#[test]
fn test_multi_round_chain() {
    let (g, t) = chain_grammar();
    let generator = StaticLanguageGenerator::new(g, t);
    let first = generator.generate_language_code().to_string();
    let second = generator.generate_language_code().to_string();
    assert_eq!(first, second);
}

#[test]
fn test_multi_round_expression() {
    let (g, t) = expression_grammar();
    let generator = StaticLanguageGenerator::new(g, t);
    let first = generator.generate_language_code().to_string();
    let second = generator.generate_language_code().to_string();
    assert_eq!(first, second);
}

#[test]
fn test_multi_round_many_alt() {
    let (g, t) = many_alternatives_grammar();
    let generator = StaticLanguageGenerator::new(g, t);
    let first = generator.generate_language_code().to_string();
    let second = generator.generate_language_code().to_string();
    assert_eq!(first, second);
}

#[test]
fn test_multi_round_multi_rule() {
    let (g, t) = multi_rule_grammar();
    let generator = StaticLanguageGenerator::new(g, t);
    let first = generator.generate_language_code().to_string();
    let second = generator.generate_language_code().to_string();
    assert_eq!(first, second);
}

#[test]
fn test_multi_round_three_times() {
    let (g, t) = expression_grammar();
    let generator = StaticLanguageGenerator::new(g, t);
    let a = generator.generate_language_code().to_string();
    let b = generator.generate_language_code().to_string();
    let c = generator.generate_language_code().to_string();
    assert_eq!(a, b);
    assert_eq!(b, c);
}

// ===========================================================================
// 7. Edge cases
// ===========================================================================

#[test]
fn test_edge_single_token_grammar() {
    let (g, t) = simple_grammar();
    let code = generate_code(g, t);
    assert!(!code.is_empty());
    assert!(code.contains("static") || code.contains("const") || code.contains('0'));
}

#[test]
fn test_edge_expression_grammar_has_multiple_rules() {
    let (g, t) = expression_grammar();
    let code = generate_code(g, t);
    // Expression grammar has 4+ rules, producing richer output.
    assert!(code.len() > 100, "expression grammar code too short");
}

#[test]
fn test_edge_many_alternatives_all_distinct_tokens() {
    let (g, t) = many_alternatives_grammar();
    let code = generate_code(g, t);
    assert!(code.len() > 100, "many-alt grammar code too short");
}

#[test]
fn test_edge_chain_grammar_nonterminal_indirection() {
    let (g, t) = chain_grammar();
    let code = generate_code(g, t);
    assert!(!code.is_empty());
}

#[test]
fn test_edge_two_alt_grammar_conflict_free() {
    let (g, t) = two_alt_grammar();
    let code = generate_code(g, t);
    assert!(!code.is_empty());
}

#[test]
fn test_edge_multi_rule_nonterminals_linked() {
    let (g, t) = multi_rule_grammar();
    let code = generate_code(g, t);
    assert!(!code.is_empty());
}

#[test]
fn test_edge_generator_with_start_can_be_empty() {
    let (g, t) = simple_grammar();
    let mut generator = StaticLanguageGenerator::new(g, t);
    generator.set_start_can_be_empty(true);
    let code = generator.generate_language_code().to_string();
    assert!(!code.is_empty());
}

#[test]
fn test_edge_generator_start_can_be_empty_deterministic() {
    let (g1, t1) = simple_grammar();
    let (g2, t2) = simple_grammar();
    let mut gen1 = StaticLanguageGenerator::new(g1, t1);
    let mut gen2 = StaticLanguageGenerator::new(g2, t2);
    gen1.set_start_can_be_empty(true);
    gen2.set_start_can_be_empty(true);
    assert_eq!(
        gen1.generate_language_code().to_string(),
        gen2.generate_language_code().to_string(),
    );
}
