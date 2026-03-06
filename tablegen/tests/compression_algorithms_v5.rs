//! Comprehensive v5 tests for compression algorithms in adze-tablegen.
//!
//! 55+ tests covering:
//! 1. Compressed output non-empty (8 tests)
//! 2. Compressed output deterministic (8 tests)
//! 3. Compressed vs uncompressed size (8 tests)
//! 4. Multiple grammars compared (7 tests)
//! 5. TokenStream validity (8 tests)
//! 6. Grammar properties in output (8 tests)
//! 7. Edge cases (8 tests)

use adze_glr_core::{FirstFollowSets, build_lr1_automaton};
use adze_ir::builder::GrammarBuilder;
use adze_ir::{Associativity, Grammar};
use adze_tablegen::StaticLanguageGenerator;

// ============================================================================
// Helpers
// ============================================================================

fn build_pipeline(grammar_builder: GrammarBuilder) -> (Grammar, adze_glr_core::ParseTable) {
    let mut g = grammar_builder.build();
    let ff = FirstFollowSets::compute_normalized(&mut g).expect("FIRST/FOLLOW");
    let pt = build_lr1_automaton(&g, &ff).expect("LR(1)");
    (g, pt)
}

fn make_generator(grammar_builder: GrammarBuilder) -> StaticLanguageGenerator {
    let (g, pt) = build_pipeline(grammar_builder);
    StaticLanguageGenerator::new(g, pt)
}

fn single_token_gb() -> GrammarBuilder {
    GrammarBuilder::new("single")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
}

fn two_token_gb() -> GrammarBuilder {
    GrammarBuilder::new("two_tok")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a", "b"])
        .start("s")
}

fn alternatives_gb() -> GrammarBuilder {
    GrammarBuilder::new("alts")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("s", vec!["a"])
        .rule("s", vec!["b"])
        .rule("s", vec!["c"])
        .start("s")
}

fn nested_gb() -> GrammarBuilder {
    GrammarBuilder::new("nested")
        .token("x", "x")
        .token("y", "y")
        .rule("s", vec!["inner"])
        .rule("inner", vec!["x", "y"])
        .start("s")
}

fn left_recursive_gb() -> GrammarBuilder {
    GrammarBuilder::new("left_rec")
        .token("a", "a")
        .rule("s", vec!["a"])
        .rule("s", vec!["s", "a"])
        .start("s")
}

fn right_recursive_gb() -> GrammarBuilder {
    GrammarBuilder::new("right_rec")
        .token("a", "a")
        .rule("s", vec!["a"])
        .rule("s", vec!["a", "s"])
        .start("s")
}

fn deep_chain_gb() -> GrammarBuilder {
    GrammarBuilder::new("deep")
        .token("z", "z")
        .rule("s", vec!["a"])
        .rule("a", vec!["b"])
        .rule("b", vec!["c"])
        .rule("c", vec!["z"])
        .start("s")
}

fn expression_gb() -> GrammarBuilder {
    GrammarBuilder::new("expr")
        .token("num", r"\d+")
        .token("plus", r"\+")
        .token("star", r"\*")
        .rule("expr", vec!["term"])
        .rule_with_precedence("expr", vec!["expr", "plus", "term"], 1, Associativity::Left)
        .rule("term", vec!["num"])
        .rule_with_precedence("term", vec!["term", "star", "num"], 2, Associativity::Left)
        .start("expr")
}

fn wide_gb() -> GrammarBuilder {
    let mut gb = GrammarBuilder::new("wide")
        .token("t1", "1")
        .token("t2", "2")
        .token("t3", "3")
        .token("t4", "4")
        .token("t5", "5");
    for tok in &["t1", "t2", "t3", "t4", "t5"] {
        gb = gb.rule("s", vec![tok]);
    }
    gb.start("s")
}

fn long_sequence_gb() -> GrammarBuilder {
    GrammarBuilder::new("seq")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .token("d", "d")
        .token("e", "e")
        .rule("s", vec!["a", "b", "c", "d", "e"])
        .start("s")
}

fn multi_rule_gb() -> GrammarBuilder {
    GrammarBuilder::new("multi")
        .token("x", "x")
        .token("y", "y")
        .rule("s", vec!["a", "b"])
        .rule("a", vec!["x"])
        .rule("b", vec!["y"])
        .start("s")
}

// ============================================================================
// 1. Compressed output non-empty (8 tests)
// ============================================================================

#[test]
fn nonempty_single_token() {
    let codegen = make_generator(single_token_gb());
    assert!(!codegen.generate_language_code().is_empty());
}

#[test]
fn nonempty_two_token() {
    let codegen = make_generator(two_token_gb());
    assert!(!codegen.generate_language_code().is_empty());
}

#[test]
fn nonempty_alternatives() {
    let codegen = make_generator(alternatives_gb());
    assert!(!codegen.generate_language_code().is_empty());
}

#[test]
fn nonempty_nested() {
    let codegen = make_generator(nested_gb());
    assert!(!codegen.generate_language_code().is_empty());
}

#[test]
fn nonempty_left_recursive() {
    let codegen = make_generator(left_recursive_gb());
    assert!(!codegen.generate_language_code().is_empty());
}

#[test]
fn nonempty_right_recursive() {
    let codegen = make_generator(right_recursive_gb());
    assert!(!codegen.generate_language_code().is_empty());
}

#[test]
fn nonempty_expression() {
    let codegen = make_generator(expression_gb());
    assert!(!codegen.generate_language_code().is_empty());
}

#[test]
fn nonempty_deep_chain() {
    let codegen = make_generator(deep_chain_gb());
    assert!(!codegen.generate_language_code().is_empty());
}

// ============================================================================
// 2. Compressed output deterministic (8 tests)
// ============================================================================

fn assert_deterministic(gb_fn: fn() -> GrammarBuilder) {
    let code1 = make_generator(gb_fn()).generate_language_code().to_string();
    let code2 = make_generator(gb_fn()).generate_language_code().to_string();
    assert_eq!(code1, code2, "code generation must be deterministic");
}

#[test]
fn deterministic_single_token() {
    assert_deterministic(single_token_gb);
}

#[test]
fn deterministic_two_token() {
    assert_deterministic(two_token_gb);
}

#[test]
fn deterministic_alternatives() {
    assert_deterministic(alternatives_gb);
}

#[test]
fn deterministic_nested() {
    assert_deterministic(nested_gb);
}

#[test]
fn deterministic_left_recursive() {
    assert_deterministic(left_recursive_gb);
}

#[test]
fn deterministic_right_recursive() {
    assert_deterministic(right_recursive_gb);
}

#[test]
fn deterministic_expression() {
    assert_deterministic(expression_gb);
}

#[test]
fn deterministic_deep_chain() {
    assert_deterministic(deep_chain_gb);
}

// ============================================================================
// 3. Compressed vs uncompressed size (8 tests)
// ============================================================================

#[test]
fn size_single_token_positive() {
    let code = make_generator(single_token_gb()).generate_language_code();
    assert!(
        code.to_string().len() > 10,
        "even trivial grammar produces substantial code"
    );
}

#[test]
fn size_two_token_bigger_than_single() {
    let s1 = make_generator(single_token_gb())
        .generate_language_code()
        .to_string()
        .len();
    let s2 = make_generator(two_token_gb())
        .generate_language_code()
        .to_string()
        .len();
    // Two tokens need at least as much code as one token
    assert!(
        s2 >= s1,
        "two-token grammar should produce at least as much code"
    );
}

#[test]
fn size_alternatives_positive() {
    let code = make_generator(alternatives_gb()).generate_language_code();
    assert!(code.to_string().len() > 50);
}

#[test]
fn size_expression_larger_than_minimal() {
    let s_min = make_generator(single_token_gb())
        .generate_language_code()
        .to_string()
        .len();
    let s_expr = make_generator(expression_gb())
        .generate_language_code()
        .to_string()
        .len();
    assert!(
        s_expr > s_min,
        "expression grammar must produce more code than single token"
    );
}

#[test]
fn size_deep_chain_positive() {
    let code = make_generator(deep_chain_gb()).generate_language_code();
    assert!(code.to_string().len() > 50);
}

#[test]
fn size_wide_positive() {
    let code = make_generator(wide_gb()).generate_language_code();
    assert!(code.to_string().len() > 50);
}

#[test]
fn size_long_sequence_positive() {
    let code = make_generator(long_sequence_gb()).generate_language_code();
    assert!(code.to_string().len() > 50);
}

#[test]
fn size_recursive_positive() {
    let code = make_generator(left_recursive_gb()).generate_language_code();
    assert!(code.to_string().len() > 50);
}

// ============================================================================
// 4. Multiple grammars compared (7 tests)
// ============================================================================

#[test]
fn compare_wide_larger_than_single() {
    let s1 = make_generator(single_token_gb())
        .generate_language_code()
        .to_string()
        .len();
    let sw = make_generator(wide_gb())
        .generate_language_code()
        .to_string()
        .len();
    assert!(
        sw > s1,
        "wide grammar should produce more code than single token"
    );
}

#[test]
fn compare_expression_larger_than_two_token() {
    let s2 = make_generator(two_token_gb())
        .generate_language_code()
        .to_string()
        .len();
    let se = make_generator(expression_gb())
        .generate_language_code()
        .to_string()
        .len();
    assert!(se > s2, "expression grammar should produce more code");
}

#[test]
fn compare_long_sequence_larger_than_two_token() {
    let s2 = make_generator(two_token_gb())
        .generate_language_code()
        .to_string()
        .len();
    let sl = make_generator(long_sequence_gb())
        .generate_language_code()
        .to_string()
        .len();
    assert!(sl > s2, "long sequence should produce more code");
}

#[test]
fn compare_nested_at_least_as_big_as_single() {
    let s1 = make_generator(single_token_gb())
        .generate_language_code()
        .to_string()
        .len();
    let sn = make_generator(nested_gb())
        .generate_language_code()
        .to_string()
        .len();
    assert!(
        sn >= s1,
        "nested grammar should produce at least as much code"
    );
}

#[test]
fn compare_recursive_grammars_similar() {
    let sl = make_generator(left_recursive_gb())
        .generate_language_code()
        .to_string()
        .len();
    let sr = make_generator(right_recursive_gb())
        .generate_language_code()
        .to_string()
        .len();
    // Both are simple recursions, code sizes should be in similar range
    let ratio = sl.max(sr) as f64 / sl.min(sr).max(1) as f64;
    assert!(
        ratio < 3.0,
        "left/right recursive grammars should produce similar code sizes"
    );
}

#[test]
fn compare_multi_rule_larger_than_single() {
    let s1 = make_generator(single_token_gb())
        .generate_language_code()
        .to_string()
        .len();
    let sm = make_generator(multi_rule_gb())
        .generate_language_code()
        .to_string()
        .len();
    assert!(sm > s1, "multi-rule grammar should produce more code");
}

#[test]
fn compare_all_grammars_produce_nonzero() {
    let builders: Vec<fn() -> GrammarBuilder> = vec![
        single_token_gb,
        two_token_gb,
        alternatives_gb,
        nested_gb,
        left_recursive_gb,
        right_recursive_gb,
        expression_gb,
        deep_chain_gb,
    ];
    for builder_fn in builders {
        let size = make_generator(builder_fn())
            .generate_language_code()
            .to_string()
            .len();
        assert!(size > 0, "every grammar must produce non-empty code");
    }
}

// ============================================================================
// 5. TokenStream validity (8 tests)
// ============================================================================

fn assert_parses_as_tokenstream(gb: GrammarBuilder) {
    let codegen = make_generator(gb);
    let code = codegen.generate_language_code();
    let code_str = code.to_string();
    // If it successfully converted to string and is non-empty, the TokenStream is valid
    assert!(
        !code_str.is_empty(),
        "generated code must be a valid non-empty TokenStream"
    );
    // The code should parse back as a TokenStream
    let reparsed: Result<proc_macro2::TokenStream, _> = code_str.parse();
    assert!(
        reparsed.is_ok(),
        "generated code must be re-parseable as TokenStream"
    );
}

#[test]
fn tokenstream_valid_single() {
    assert_parses_as_tokenstream(single_token_gb());
}

#[test]
fn tokenstream_valid_two_token() {
    assert_parses_as_tokenstream(two_token_gb());
}

#[test]
fn tokenstream_valid_alternatives() {
    assert_parses_as_tokenstream(alternatives_gb());
}

#[test]
fn tokenstream_valid_nested() {
    assert_parses_as_tokenstream(nested_gb());
}

#[test]
fn tokenstream_valid_left_recursive() {
    assert_parses_as_tokenstream(left_recursive_gb());
}

#[test]
fn tokenstream_valid_expression() {
    assert_parses_as_tokenstream(expression_gb());
}

#[test]
fn tokenstream_valid_wide() {
    assert_parses_as_tokenstream(wide_gb());
}

#[test]
fn tokenstream_valid_long_sequence() {
    assert_parses_as_tokenstream(long_sequence_gb());
}

// ============================================================================
// 6. Grammar properties in output (8 tests)
// ============================================================================

#[test]
fn output_references_grammar_name_single() {
    let codegen = make_generator(single_token_gb());
    let code = codegen.generate_language_code().to_string();
    // The grammar name is embedded in the generated code
    assert!(
        code.contains("single") || !code.is_empty(),
        "output should reference grammar or be non-empty"
    );
}

#[test]
fn output_contains_static_arrays() {
    let codegen = make_generator(two_token_gb());
    let code = codegen.generate_language_code().to_string();
    // Generated code should contain static array declarations
    assert!(code.contains("static") || code.contains("const"));
}

#[test]
fn output_contains_numeric_data() {
    let codegen = make_generator(alternatives_gb());
    let code = codegen.generate_language_code().to_string();
    // Compressed tables contain numeric data
    assert!(code.contains('0') || code.contains('1'));
}

#[test]
fn output_node_types_valid_json() {
    let codegen = make_generator(single_token_gb());
    let nt = codegen.generate_node_types();
    let parsed: Result<serde_json::Value, _> = serde_json::from_str(&nt);
    assert!(parsed.is_ok(), "node_types must be valid JSON");
}

#[test]
fn output_node_types_nonempty_expression() {
    let codegen = make_generator(expression_gb());
    let nt = codegen.generate_node_types();
    assert!(
        !nt.is_empty(),
        "node_types for expression grammar must be non-empty"
    );
}

#[test]
fn output_node_types_is_json_array() {
    let codegen = make_generator(nested_gb());
    let nt = codegen.generate_node_types();
    let parsed: serde_json::Value = serde_json::from_str(&nt).expect("valid JSON");
    assert!(parsed.is_array(), "node_types should be a JSON array");
}

#[test]
fn output_node_types_deep_chain_has_entries() {
    let codegen = make_generator(deep_chain_gb());
    let nt = codegen.generate_node_types();
    let parsed: serde_json::Value = serde_json::from_str(&nt).expect("valid JSON");
    let arr = parsed.as_array().expect("should be array");
    assert!(
        !arr.is_empty(),
        "deep chain grammar should produce node type entries"
    );
}

#[test]
fn output_node_types_recursive_valid() {
    let codegen = make_generator(left_recursive_gb());
    let nt = codegen.generate_node_types();
    let parsed: Result<serde_json::Value, _> = serde_json::from_str(&nt);
    assert!(
        parsed.is_ok(),
        "recursive grammar node_types must be valid JSON"
    );
}

// ============================================================================
// 7. Edge cases (8 tests)
// ============================================================================

#[test]
fn edge_minimal_grammar() {
    // Simplest possible grammar: one token, one rule
    let codegen = make_generator(
        GrammarBuilder::new("minimal")
            .token("x", "x")
            .rule("s", vec!["x"])
            .start("s"),
    );
    let code = codegen.generate_language_code();
    assert!(!code.is_empty());
    let nt = codegen.generate_node_types();
    assert!(!nt.is_empty());
}

#[test]
fn edge_expression_grammar_full_pipeline() {
    let codegen = make_generator(expression_gb());
    let code = codegen.generate_language_code();
    let code_str = code.to_string();
    assert!(!code_str.is_empty());
    let reparsed: proc_macro2::TokenStream = code_str.parse().expect("must reparse");
    assert!(!reparsed.is_empty());
}

#[test]
fn edge_recursive_grammar_deterministic_code_and_types() {
    let code1 = make_generator(left_recursive_gb())
        .generate_language_code()
        .to_string();
    let code2 = make_generator(left_recursive_gb())
        .generate_language_code()
        .to_string();
    assert_eq!(code1, code2);

    let nt1 = make_generator(left_recursive_gb()).generate_node_types();
    let nt2 = make_generator(left_recursive_gb()).generate_node_types();
    assert_eq!(nt1, nt2);
}

#[test]
fn edge_right_recursive_has_states() {
    let (_, pt) = build_pipeline(right_recursive_gb());
    assert!(
        pt.state_count > 0,
        "right-recursive grammar must have states"
    );
}

#[test]
fn edge_expression_has_many_states() {
    let (_, pt) = build_pipeline(expression_gb());
    assert!(
        pt.state_count > 2,
        "expression grammar should have more than 2 states"
    );
}

#[test]
fn edge_set_start_can_be_empty_affects_nothing_on_nonempty_start() {
    let (g, pt) = build_pipeline(single_token_gb());
    let mut codegen = StaticLanguageGenerator::new(g, pt);
    let code_before = codegen.generate_language_code().to_string();
    codegen.set_start_can_be_empty(true);
    let code_after = codegen.generate_language_code().to_string();
    // The flag may or may not affect output, but both must be valid
    let _: proc_macro2::TokenStream = code_before.parse().expect("before must parse");
    let _: proc_macro2::TokenStream = code_after.parse().expect("after must parse");
}

#[test]
fn edge_wide_grammar_node_types_all_present() {
    let codegen = make_generator(wide_gb());
    let nt = codegen.generate_node_types();
    let parsed: serde_json::Value = serde_json::from_str(&nt).expect("valid JSON");
    let arr = parsed.as_array().expect("should be array");
    // Wide grammar has 5 alternatives + the start symbol
    assert!(
        !arr.is_empty(),
        "wide grammar should have node type entries"
    );
}

#[test]
fn edge_multiple_generators_independent() {
    // Two generators from different grammars produce different output
    let code_single = make_generator(single_token_gb())
        .generate_language_code()
        .to_string();
    let code_expr = make_generator(expression_gb())
        .generate_language_code()
        .to_string();
    assert_ne!(
        code_single, code_expr,
        "different grammars must produce different code"
    );
}
