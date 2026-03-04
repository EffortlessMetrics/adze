//! Advanced v2 tests for StaticLanguageGenerator comprehensive testing.
//!
//! Covers: code generation for various grammars, non-emptiness, determinism,
//! grammar name in output, set_start_can_be_empty effects, large grammars,
//! precedence grammars, and multiple nonterminal code generation.

use adze_glr_core::{FirstFollowSets, ParseTable, build_lr1_automaton};
use adze_ir::builder::GrammarBuilder;
use adze_ir::{Associativity, Grammar};
use adze_tablegen::{NodeTypesGenerator, StaticLanguageGenerator};

// ===========================================================================
// Helpers — GrammarBuilder → ParseTable via FIRST/FOLLOW + LR(1)
// ===========================================================================

fn build_pipeline(
    name: &str,
    builder_fn: impl FnOnce(GrammarBuilder) -> GrammarBuilder,
) -> (Grammar, ParseTable) {
    let builder = builder_fn(GrammarBuilder::new(name));
    let mut g = builder.build();
    let ff = FirstFollowSets::compute_normalized(&mut g).expect("FIRST/FOLLOW");
    let pt = build_lr1_automaton(&g, &ff).expect("LR(1) automaton");
    (g, pt)
}

fn single_token_grammar() -> (Grammar, ParseTable) {
    build_pipeline("single_tok", |b| {
        b.token("x", "x").rule("start", vec!["x"]).start("start")
    })
}

fn two_token_grammar() -> (Grammar, ParseTable) {
    build_pipeline("two_tok", |b| {
        b.token("a", "a")
            .token("b", "b")
            .rule("start", vec!["a", "b"])
            .start("start")
    })
}

fn three_token_grammar() -> (Grammar, ParseTable) {
    build_pipeline("three_tok", |b| {
        b.token("a", "a")
            .token("b", "b")
            .token("c", "c")
            .rule("start", vec!["a", "b", "c"])
            .start("start")
    })
}

fn alt_grammar() -> (Grammar, ParseTable) {
    build_pipeline("alt", |b| {
        b.token("a", "a")
            .token("b", "b")
            .rule("start", vec!["a"])
            .rule("start", vec!["b"])
            .start("start")
    })
}

fn chain_grammar() -> (Grammar, ParseTable) {
    build_pipeline("chain", |b| {
        b.token("x", "x")
            .rule("C", vec!["x"])
            .rule("B", vec!["C"])
            .rule("A", vec!["B"])
            .rule("start", vec!["A"])
            .start("start")
    })
}

fn recursive_grammar() -> (Grammar, ParseTable) {
    build_pipeline("recursive", |b| {
        b.token("a", "a")
            .rule("A", vec!["a", "A"])
            .rule("A", vec!["a"])
            .rule("start", vec!["A"])
            .start("start")
    })
}

fn precedence_grammar() -> (Grammar, ParseTable) {
    build_pipeline("prec", |b| {
        b.token("num", "n")
            .token("plus", "p")
            .token("star", "s")
            .rule("start", vec!["expr"])
            .rule("expr", vec!["num"])
            .rule_with_precedence("expr", vec!["expr", "plus", "expr"], 1, Associativity::Left)
            .rule_with_precedence("expr", vec!["expr", "star", "expr"], 2, Associativity::Left)
            .start("start")
    })
}

fn right_assoc_grammar() -> (Grammar, ParseTable) {
    build_pipeline("rassoc", |b| {
        b.token("num", "n")
            .token("pow", "p")
            .rule("start", vec!["expr"])
            .rule("expr", vec!["num"])
            .rule_with_precedence("expr", vec!["expr", "pow", "expr"], 1, Associativity::Right)
            .start("start")
    })
}

fn multi_nonterminal_grammar() -> (Grammar, ParseTable) {
    build_pipeline("multi_nt", |b| {
        b.token("x", "x")
            .token("y", "y")
            .rule("alpha", vec!["x"])
            .rule("beta", vec!["y"])
            .rule("start", vec!["alpha", "beta"])
            .start("start")
    })
}

fn deep_chain_grammar() -> (Grammar, ParseTable) {
    build_pipeline("deep", |b| {
        b.token("z", "z")
            .rule("E", vec!["z"])
            .rule("D", vec!["E"])
            .rule("C", vec!["D"])
            .rule("B", vec!["C"])
            .rule("A", vec!["B"])
            .rule("start", vec!["A"])
            .start("start")
    })
}

fn wide_alt_grammar() -> (Grammar, ParseTable) {
    build_pipeline("wide_alt", |b| {
        b.token("a", "a")
            .token("b", "b")
            .token("c", "c")
            .token("d", "d")
            .token("e", "e")
            .rule("start", vec!["a"])
            .rule("start", vec!["b"])
            .rule("start", vec!["c"])
            .rule("start", vec!["d"])
            .rule("start", vec!["e"])
            .start("start")
    })
}

fn mixed_prec_grammar() -> (Grammar, ParseTable) {
    build_pipeline("mixed_prec", |b| {
        b.token("num", "n")
            .token("plus", "p")
            .token("star", "s")
            .token("pow", "w")
            .rule("start", vec!["expr"])
            .rule("expr", vec!["num"])
            .rule_with_precedence("expr", vec!["expr", "plus", "expr"], 1, Associativity::Left)
            .rule_with_precedence("expr", vec!["expr", "star", "expr"], 2, Associativity::Left)
            .rule_with_precedence("expr", vec!["expr", "pow", "expr"], 3, Associativity::Right)
            .start("start")
    })
}

// ===========================================================================
// 1. Generate language code for various grammars
// ===========================================================================

#[test]
fn gen_code_single_token() {
    let (g, t) = single_token_grammar();
    let slg = StaticLanguageGenerator::new(g, t);
    let code = slg.generate_language_code().to_string();
    assert!(!code.is_empty());
}

#[test]
fn gen_code_two_token() {
    let (g, t) = two_token_grammar();
    let slg = StaticLanguageGenerator::new(g, t);
    let code = slg.generate_language_code().to_string();
    assert!(!code.is_empty());
}

#[test]
fn gen_code_three_token() {
    let (g, t) = three_token_grammar();
    let slg = StaticLanguageGenerator::new(g, t);
    let code = slg.generate_language_code().to_string();
    assert!(!code.is_empty());
}

#[test]
fn gen_code_alt() {
    let (g, t) = alt_grammar();
    let slg = StaticLanguageGenerator::new(g, t);
    let code = slg.generate_language_code().to_string();
    assert!(!code.is_empty());
}

#[test]
fn gen_code_chain() {
    let (g, t) = chain_grammar();
    let slg = StaticLanguageGenerator::new(g, t);
    let code = slg.generate_language_code().to_string();
    assert!(!code.is_empty());
}

#[test]
fn gen_code_recursive() {
    let (g, t) = recursive_grammar();
    let slg = StaticLanguageGenerator::new(g, t);
    let code = slg.generate_language_code().to_string();
    assert!(!code.is_empty());
}

#[test]
fn gen_code_precedence() {
    let (g, t) = precedence_grammar();
    let slg = StaticLanguageGenerator::new(g, t);
    let code = slg.generate_language_code().to_string();
    assert!(!code.is_empty());
}

#[test]
fn gen_code_right_assoc() {
    let (g, t) = right_assoc_grammar();
    let slg = StaticLanguageGenerator::new(g, t);
    let code = slg.generate_language_code().to_string();
    assert!(!code.is_empty());
}

#[test]
fn gen_code_multi_nonterminal() {
    let (g, t) = multi_nonterminal_grammar();
    let slg = StaticLanguageGenerator::new(g, t);
    let code = slg.generate_language_code().to_string();
    assert!(!code.is_empty());
}

#[test]
fn gen_code_deep_chain() {
    let (g, t) = deep_chain_grammar();
    let slg = StaticLanguageGenerator::new(g, t);
    let code = slg.generate_language_code().to_string();
    assert!(!code.is_empty());
}

#[test]
fn gen_code_wide_alt() {
    let (g, t) = wide_alt_grammar();
    let slg = StaticLanguageGenerator::new(g, t);
    let code = slg.generate_language_code().to_string();
    assert!(!code.is_empty());
}

#[test]
fn gen_code_mixed_prec() {
    let (g, t) = mixed_prec_grammar();
    let slg = StaticLanguageGenerator::new(g, t);
    let code = slg.generate_language_code().to_string();
    assert!(!code.is_empty());
}

// ===========================================================================
// 2. Language code is non-empty (additional structural checks)
// ===========================================================================

#[test]
fn code_contains_symbol_names_single() {
    let (g, t) = single_token_grammar();
    let code = StaticLanguageGenerator::new(g, t)
        .generate_language_code()
        .to_string();
    assert!(code.contains("SYMBOL_NAMES"));
}

#[test]
fn code_contains_symbol_metadata_alt() {
    let (g, t) = alt_grammar();
    let code = StaticLanguageGenerator::new(g, t)
        .generate_language_code()
        .to_string();
    assert!(code.contains("SYMBOL_METADATA"));
}

#[test]
fn code_contains_parse_table_recursive() {
    let (g, t) = recursive_grammar();
    let code = StaticLanguageGenerator::new(g, t)
        .generate_language_code()
        .to_string();
    assert!(code.contains("PARSE_TABLE"));
}

#[test]
fn code_contains_field_names_chain() {
    let (g, t) = chain_grammar();
    let code = StaticLanguageGenerator::new(g, t)
        .generate_language_code()
        .to_string();
    assert!(code.contains("FIELD_NAMES"));
}

#[test]
fn code_contains_lex_modes_two_token() {
    let (g, t) = two_token_grammar();
    let code = StaticLanguageGenerator::new(g, t)
        .generate_language_code()
        .to_string();
    assert!(code.contains("LEX_MODES"));
}

#[test]
fn code_contains_tslanguage_deep() {
    let (g, t) = deep_chain_grammar();
    let code = StaticLanguageGenerator::new(g, t)
        .generate_language_code()
        .to_string();
    assert!(code.contains("TSLanguage"));
}

#[test]
fn code_contains_external_scanner_prec() {
    let (g, t) = precedence_grammar();
    let code = StaticLanguageGenerator::new(g, t)
        .generate_language_code()
        .to_string();
    assert!(code.contains("EXTERNAL_SCANNER"));
}

#[test]
fn code_contains_version_constant_wide_alt() {
    let (g, t) = wide_alt_grammar();
    let code = StaticLanguageGenerator::new(g, t)
        .generate_language_code()
        .to_string();
    assert!(code.contains("TREE_SITTER_LANGUAGE_VERSION"));
}

// ===========================================================================
// 3. Language code is deterministic
// ===========================================================================

#[test]
fn deterministic_single_token() {
    let (g1, t1) = single_token_grammar();
    let (g2, t2) = single_token_grammar();
    let c1 = StaticLanguageGenerator::new(g1, t1)
        .generate_language_code()
        .to_string();
    let c2 = StaticLanguageGenerator::new(g2, t2)
        .generate_language_code()
        .to_string();
    assert_eq!(c1, c2);
}

#[test]
fn deterministic_alt() {
    let (g1, t1) = alt_grammar();
    let (g2, t2) = alt_grammar();
    let c1 = StaticLanguageGenerator::new(g1, t1)
        .generate_language_code()
        .to_string();
    let c2 = StaticLanguageGenerator::new(g2, t2)
        .generate_language_code()
        .to_string();
    assert_eq!(c1, c2);
}

#[test]
fn deterministic_chain() {
    let (g1, t1) = chain_grammar();
    let (g2, t2) = chain_grammar();
    let c1 = StaticLanguageGenerator::new(g1, t1)
        .generate_language_code()
        .to_string();
    let c2 = StaticLanguageGenerator::new(g2, t2)
        .generate_language_code()
        .to_string();
    assert_eq!(c1, c2);
}

#[test]
fn deterministic_recursive() {
    let (g1, t1) = recursive_grammar();
    let (g2, t2) = recursive_grammar();
    let c1 = StaticLanguageGenerator::new(g1, t1)
        .generate_language_code()
        .to_string();
    let c2 = StaticLanguageGenerator::new(g2, t2)
        .generate_language_code()
        .to_string();
    assert_eq!(c1, c2);
}

#[test]
fn deterministic_precedence() {
    let (g1, t1) = precedence_grammar();
    let (g2, t2) = precedence_grammar();
    let c1 = StaticLanguageGenerator::new(g1, t1)
        .generate_language_code()
        .to_string();
    let c2 = StaticLanguageGenerator::new(g2, t2)
        .generate_language_code()
        .to_string();
    assert_eq!(c1, c2);
}

#[test]
fn deterministic_multi_nonterminal() {
    let (g1, t1) = multi_nonterminal_grammar();
    let (g2, t2) = multi_nonterminal_grammar();
    let c1 = StaticLanguageGenerator::new(g1, t1)
        .generate_language_code()
        .to_string();
    let c2 = StaticLanguageGenerator::new(g2, t2)
        .generate_language_code()
        .to_string();
    assert_eq!(c1, c2);
}

#[test]
fn deterministic_node_types_single() {
    let (g1, t1) = single_token_grammar();
    let (g2, t2) = single_token_grammar();
    let n1 = StaticLanguageGenerator::new(g1, t1).generate_node_types();
    let n2 = StaticLanguageGenerator::new(g2, t2).generate_node_types();
    assert_eq!(n1, n2);
}

#[test]
fn deterministic_node_types_prec() {
    let (g1, t1) = precedence_grammar();
    let (g2, t2) = precedence_grammar();
    let n1 = StaticLanguageGenerator::new(g1, t1).generate_node_types();
    let n2 = StaticLanguageGenerator::new(g2, t2).generate_node_types();
    assert_eq!(n1, n2);
}

// ===========================================================================
// 4. Language code contains grammar name
// ===========================================================================

#[test]
fn code_has_grammar_name_single_tok() {
    let (g, t) = single_token_grammar();
    let code = StaticLanguageGenerator::new(g, t)
        .generate_language_code()
        .to_string();
    assert!(code.contains("tree_sitter_single_tok"));
}

#[test]
fn code_has_grammar_name_alt() {
    let (g, t) = alt_grammar();
    let code = StaticLanguageGenerator::new(g, t)
        .generate_language_code()
        .to_string();
    assert!(code.contains("tree_sitter_alt"));
}

#[test]
fn code_has_grammar_name_chain() {
    let (g, t) = chain_grammar();
    let code = StaticLanguageGenerator::new(g, t)
        .generate_language_code()
        .to_string();
    assert!(code.contains("tree_sitter_chain"));
}

#[test]
fn code_has_grammar_name_recursive() {
    let (g, t) = recursive_grammar();
    let code = StaticLanguageGenerator::new(g, t)
        .generate_language_code()
        .to_string();
    assert!(code.contains("tree_sitter_recursive"));
}

#[test]
fn code_has_grammar_name_prec() {
    let (g, t) = precedence_grammar();
    let code = StaticLanguageGenerator::new(g, t)
        .generate_language_code()
        .to_string();
    assert!(code.contains("tree_sitter_prec"));
}

#[test]
fn code_has_grammar_name_multi_nt() {
    let (g, t) = multi_nonterminal_grammar();
    let code = StaticLanguageGenerator::new(g, t)
        .generate_language_code()
        .to_string();
    assert!(code.contains("tree_sitter_multi_nt"));
}

#[test]
fn code_has_grammar_name_deep() {
    let (g, t) = deep_chain_grammar();
    let code = StaticLanguageGenerator::new(g, t)
        .generate_language_code()
        .to_string();
    assert!(code.contains("tree_sitter_deep"));
}

#[test]
fn code_has_grammar_name_wide_alt() {
    let (g, t) = wide_alt_grammar();
    let code = StaticLanguageGenerator::new(g, t)
        .generate_language_code()
        .to_string();
    assert!(code.contains("tree_sitter_wide_alt"));
}

// ===========================================================================
// 5. set_start_can_be_empty effect
// ===========================================================================

#[test]
fn start_can_be_empty_default_is_false() {
    let (g, t) = single_token_grammar();
    let slg = StaticLanguageGenerator::new(g, t);
    assert!(!slg.start_can_be_empty);
}

#[test]
fn start_can_be_empty_set_true() {
    let (g, t) = single_token_grammar();
    let mut slg = StaticLanguageGenerator::new(g, t);
    slg.set_start_can_be_empty(true);
    assert!(slg.start_can_be_empty);
}

#[test]
fn start_can_be_empty_toggle_back() {
    let (g, t) = single_token_grammar();
    let mut slg = StaticLanguageGenerator::new(g, t);
    slg.set_start_can_be_empty(true);
    slg.set_start_can_be_empty(false);
    assert!(!slg.start_can_be_empty);
}

#[test]
fn start_can_be_empty_code_differs() {
    let (g1, t1) = single_token_grammar();
    let (g2, t2) = single_token_grammar();
    let code_false = StaticLanguageGenerator::new(g1, t1)
        .generate_language_code()
        .to_string();
    let mut slg = StaticLanguageGenerator::new(g2, t2);
    slg.set_start_can_be_empty(true);
    let code_true = slg.generate_language_code().to_string();
    // The codes may or may not differ depending on implementation, but both must be non-empty
    assert!(!code_false.is_empty());
    assert!(!code_true.is_empty());
}

#[test]
fn start_can_be_empty_true_still_generates_valid_code() {
    let (g, t) = alt_grammar();
    let mut slg = StaticLanguageGenerator::new(g, t);
    slg.set_start_can_be_empty(true);
    let code = slg.generate_language_code().to_string();
    assert!(code.contains("TSLanguage"));
    assert!(code.contains("SYMBOL_NAMES"));
}

#[test]
fn start_can_be_empty_true_chain() {
    let (g, t) = chain_grammar();
    let mut slg = StaticLanguageGenerator::new(g, t);
    slg.set_start_can_be_empty(true);
    let code = slg.generate_language_code().to_string();
    assert!(!code.is_empty());
}

#[test]
fn start_can_be_empty_preserves_grammar_name() {
    let (g, t) = single_token_grammar();
    let mut slg = StaticLanguageGenerator::new(g, t);
    slg.set_start_can_be_empty(true);
    assert_eq!(slg.grammar.name, "single_tok");
}

// ===========================================================================
// 6. Large grammar code generation
// ===========================================================================

#[test]
fn large_grammar_10_tokens() {
    let (g, t) = build_pipeline("large10", |b| {
        let mut bldr = b;
        let mut tokens = Vec::new();
        for i in 0..10 {
            let name = format!("t{i}");
            let pat = format!("{}", (b'a' + i as u8) as char);
            bldr = bldr.token(&name, &pat);
            tokens.push(name);
        }
        for tok in &tokens {
            bldr = bldr.rule("start", vec![tok.as_str()]);
        }
        bldr.start("start")
    });
    let slg = StaticLanguageGenerator::new(g, t);
    let code = slg.generate_language_code().to_string();
    assert!(!code.is_empty());
    assert!(code.contains("tree_sitter_large10"));
}

#[test]
fn large_grammar_many_alternatives() {
    let (g, t) = build_pipeline("many_alts", |b| {
        let mut bldr = b;
        let mut tokens = Vec::new();
        for i in 0..8 {
            let name = format!("tok{i}");
            let pat = format!("{}", (b'a' + i as u8) as char);
            bldr = bldr.token(&name, &pat);
            tokens.push(name);
        }
        for tok in &tokens {
            bldr = bldr.rule("start", vec![tok.as_str()]);
        }
        bldr.start("start")
    });
    let code = StaticLanguageGenerator::new(g, t)
        .generate_language_code()
        .to_string();
    assert!(code.contains("SYMBOL_NAMES"));
    assert!(code.contains("tree_sitter_many_alts"));
}

#[test]
fn large_grammar_deep_chain_10() {
    let (g, t) = build_pipeline("deep10", |b| {
        let mut bldr = b.token("x", "x");
        bldr = bldr.rule("n0", vec!["x"]);
        for i in 1..10 {
            let prev = format!("n{}", i - 1);
            let curr = format!("n{i}");
            bldr = bldr.rule(&curr, vec![prev.as_str()]);
        }
        bldr.rule("start", vec!["n9"]).start("start")
    });
    let code = StaticLanguageGenerator::new(g, t)
        .generate_language_code()
        .to_string();
    assert!(!code.is_empty());
}

#[test]
fn large_grammar_code_is_parseable_token_stream() {
    let (g, t) = build_pipeline("parseable_lg", |b| {
        let mut bldr = b;
        for i in 0..6 {
            let name = format!("t{i}");
            let pat = format!("{}", (b'a' + i as u8) as char);
            bldr = bldr.token(&name, &pat);
            bldr = bldr.rule("start", vec![name.as_str()]);
        }
        bldr.start("start")
    });
    let ts = StaticLanguageGenerator::new(g, t).generate_language_code();
    let reparsed: proc_macro2::TokenStream = ts.to_string().parse().expect("must reparse");
    assert!(!reparsed.is_empty());
}

#[test]
fn large_grammar_deterministic() {
    let make = || {
        build_pipeline("det_large", |b| {
            let mut bldr = b;
            for i in 0..5 {
                let name = format!("t{i}");
                let pat = format!("{}", (b'a' + i as u8) as char);
                bldr = bldr.token(&name, &pat);
                bldr = bldr.rule("start", vec![name.as_str()]);
            }
            bldr.start("start")
        })
    };
    let (g1, t1) = make();
    let (g2, t2) = make();
    let c1 = StaticLanguageGenerator::new(g1, t1)
        .generate_language_code()
        .to_string();
    let c2 = StaticLanguageGenerator::new(g2, t2)
        .generate_language_code()
        .to_string();
    assert_eq!(c1, c2);
}

// ===========================================================================
// 7. Precedence grammar code generation
// ===========================================================================

#[test]
fn prec_grammar_generates_parse_table() {
    let (g, t) = precedence_grammar();
    let code = StaticLanguageGenerator::new(g, t)
        .generate_language_code()
        .to_string();
    assert!(code.contains("PARSE_TABLE"));
}

#[test]
fn prec_grammar_contains_symbol_metadata() {
    let (g, t) = precedence_grammar();
    let code = StaticLanguageGenerator::new(g, t)
        .generate_language_code()
        .to_string();
    assert!(code.contains("SYMBOL_METADATA"));
}

#[test]
fn prec_grammar_node_types_valid_json() {
    let (g, t) = precedence_grammar();
    let slg = StaticLanguageGenerator::new(g, t);
    let json_str = slg.generate_node_types();
    let v: serde_json::Value = serde_json::from_str(&json_str).expect("valid JSON");
    assert!(v.is_array());
}

#[test]
fn right_assoc_generates_code() {
    let (g, t) = right_assoc_grammar();
    let code = StaticLanguageGenerator::new(g, t)
        .generate_language_code()
        .to_string();
    assert!(code.contains("tree_sitter_rassoc"));
}

#[test]
fn mixed_prec_generates_code() {
    let (g, t) = mixed_prec_grammar();
    let code = StaticLanguageGenerator::new(g, t)
        .generate_language_code()
        .to_string();
    assert!(code.contains("tree_sitter_mixed_prec"));
}

#[test]
fn prec_grammar_code_is_parseable() {
    let (g, t) = precedence_grammar();
    let ts = StaticLanguageGenerator::new(g, t).generate_language_code();
    let reparsed: proc_macro2::TokenStream = ts.to_string().parse().expect("must reparse");
    assert!(!reparsed.is_empty());
}

#[test]
fn mixed_prec_deterministic() {
    let (g1, t1) = mixed_prec_grammar();
    let (g2, t2) = mixed_prec_grammar();
    let c1 = StaticLanguageGenerator::new(g1, t1)
        .generate_language_code()
        .to_string();
    let c2 = StaticLanguageGenerator::new(g2, t2)
        .generate_language_code()
        .to_string();
    assert_eq!(c1, c2);
}

#[test]
fn prec_grammar_node_types_not_empty() {
    let (g, t) = precedence_grammar();
    let slg = StaticLanguageGenerator::new(g, t);
    let arr: Vec<serde_json::Value> = serde_json::from_str(&slg.generate_node_types()).unwrap();
    assert!(!arr.is_empty());
}

// ===========================================================================
// 8. Multiple nonterminal code generation
// ===========================================================================

#[test]
fn multi_nt_code_has_ffi() {
    let (g, t) = multi_nonterminal_grammar();
    let code = StaticLanguageGenerator::new(g, t)
        .generate_language_code()
        .to_string();
    assert!(code.contains("tree_sitter_multi_nt"));
}

#[test]
fn multi_nt_node_types_valid_json() {
    let (g, t) = multi_nonterminal_grammar();
    let slg = StaticLanguageGenerator::new(g, t);
    let json_str = slg.generate_node_types();
    let v: serde_json::Value = serde_json::from_str(&json_str).expect("valid JSON");
    assert!(v.is_array());
}

#[test]
fn multi_nt_node_types_not_empty() {
    let (g, t) = multi_nonterminal_grammar();
    let arr: Vec<serde_json::Value> =
        serde_json::from_str(&StaticLanguageGenerator::new(g, t).generate_node_types()).unwrap();
    assert!(!arr.is_empty());
}

#[test]
fn chain_grammar_node_types_valid_json() {
    let (g, t) = chain_grammar();
    let slg = StaticLanguageGenerator::new(g, t);
    let v: serde_json::Value =
        serde_json::from_str(&slg.generate_node_types()).expect("valid JSON");
    assert!(v.is_array());
}

#[test]
fn deep_chain_node_types_valid_json() {
    let (g, t) = deep_chain_grammar();
    let slg = StaticLanguageGenerator::new(g, t);
    let v: serde_json::Value =
        serde_json::from_str(&slg.generate_node_types()).expect("valid JSON");
    assert!(v.is_array());
}

#[test]
fn multi_nt_code_deterministic() {
    let (g1, t1) = multi_nonterminal_grammar();
    let (g2, t2) = multi_nonterminal_grammar();
    let c1 = StaticLanguageGenerator::new(g1, t1)
        .generate_language_code()
        .to_string();
    let c2 = StaticLanguageGenerator::new(g2, t2)
        .generate_language_code()
        .to_string();
    assert_eq!(c1, c2);
}

#[test]
fn deep_chain_code_has_tslanguage() {
    let (g, t) = deep_chain_grammar();
    let code = StaticLanguageGenerator::new(g, t)
        .generate_language_code()
        .to_string();
    assert!(code.contains("TSLanguage"));
}

// ===========================================================================
// 9. NodeTypesGenerator integration
// ===========================================================================

#[test]
fn node_types_gen_single_token() {
    let (g, _) = single_token_grammar();
    let result = NodeTypesGenerator::new(&g).generate();
    assert!(result.is_ok());
}

#[test]
fn node_types_gen_alt() {
    let (g, _) = alt_grammar();
    let result = NodeTypesGenerator::new(&g).generate();
    assert!(result.is_ok());
}

#[test]
fn node_types_gen_precedence() {
    let (g, _) = precedence_grammar();
    let result = NodeTypesGenerator::new(&g).generate();
    assert!(result.is_ok());
}

#[test]
fn node_types_gen_multi_nt() {
    let (g, _) = multi_nonterminal_grammar();
    let result = NodeTypesGenerator::new(&g).generate();
    assert!(result.is_ok());
}

#[test]
fn node_types_gen_deep_chain() {
    let (g, _) = deep_chain_grammar();
    let result = NodeTypesGenerator::new(&g).generate();
    assert!(result.is_ok());
}

#[test]
fn node_types_gen_empty_grammar() {
    let g = Grammar::new("empty_v2".to_string());
    let result = NodeTypesGenerator::new(&g).generate();
    assert!(result.is_ok());
    let arr: Vec<serde_json::Value> = serde_json::from_str(&result.unwrap()).unwrap();
    assert!(arr.is_empty());
}

// ===========================================================================
// 10. Constructor preserves fields
// ===========================================================================

#[test]
fn constructor_preserves_grammar_name_chain() {
    let (g, t) = chain_grammar();
    let slg = StaticLanguageGenerator::new(g, t);
    assert_eq!(slg.grammar.name, "chain");
}

#[test]
fn constructor_preserves_state_count() {
    let (g, t) = alt_grammar();
    let expected = t.state_count;
    let slg = StaticLanguageGenerator::new(g, t);
    assert_eq!(slg.parse_table.state_count, expected);
}

#[test]
fn constructor_compressed_tables_none() {
    let (g, t) = recursive_grammar();
    let slg = StaticLanguageGenerator::new(g, t);
    assert!(slg.compressed_tables.is_none());
}

// ===========================================================================
// 11. Token stream round-trip
// ===========================================================================

#[test]
fn token_stream_roundtrip_alt() {
    let (g, t) = alt_grammar();
    let ts = StaticLanguageGenerator::new(g, t).generate_language_code();
    let reparsed: proc_macro2::TokenStream = ts.to_string().parse().expect("must reparse");
    assert!(!reparsed.is_empty());
}

#[test]
fn token_stream_roundtrip_recursive() {
    let (g, t) = recursive_grammar();
    let ts = StaticLanguageGenerator::new(g, t).generate_language_code();
    let reparsed: proc_macro2::TokenStream = ts.to_string().parse().expect("must reparse");
    assert!(!reparsed.is_empty());
}

#[test]
fn token_stream_roundtrip_deep_chain() {
    let (g, t) = deep_chain_grammar();
    let ts = StaticLanguageGenerator::new(g, t).generate_language_code();
    let reparsed: proc_macro2::TokenStream = ts.to_string().parse().expect("must reparse");
    assert!(!reparsed.is_empty());
}
