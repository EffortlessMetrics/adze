//! Determinism tests for `AbiLanguageBuilder` output in adze-tablegen.
//!
//! 80+ tests across 20 categories verifying that `AbiLanguageBuilder::generate()`
//! always produces identical output for the same grammar input.
//!
//! Categories:
//!   twice_*         — build twice → same output
//!   thrice_*        — build 3 times → all identical
//!   five_*          — build 5 times → all identical
//!   ten_*           — build 10 times → all identical
//!   minimal_*       — minimal grammar deterministic
//!   arith_*         — arithmetic grammar deterministic
//!   prec_*          — precedence grammar deterministic
//!   extras_*        — extras grammar deterministic
//!   inline_*        — inline grammar deterministic
//!   extern_*        — externals grammar deterministic
//!   super_*         — supertypes grammar deterministic
//!   conflict_*      — conflict grammar deterministic
//!   alt_*           — alternatives grammar deterministic
//!   chain_*         — chain rule grammar deterministic
//!   recurse_*       — recursive grammar deterministic
//!   large_*         — large grammar deterministic
//!   len_*           — code length deterministic
//!   nondegen_*      — different grammars → different output
//!   name_*          — output contains same grammar name
//!   byte_*          — byte-for-byte identical

use adze_glr_core::{FirstFollowSets, ParseTable, build_lr1_automaton};
use adze_ir::builder::GrammarBuilder;
use adze_ir::{Associativity, ConflictDeclaration, ConflictResolution, Grammar};
use adze_tablegen::AbiLanguageBuilder;

// ============================================================================
// Helpers
// ============================================================================

fn build_table(grammar: &Grammar) -> ParseTable {
    let mut g = grammar.clone();
    let ff = FirstFollowSets::compute_normalized(&mut g).expect("FIRST/FOLLOW");
    build_lr1_automaton(&g, &ff).expect("LR(1)")
}

fn generate(grammar: &Grammar) -> String {
    let pt = build_table(grammar);
    AbiLanguageBuilder::new(grammar, &pt).generate().to_string()
}

// --- grammar factories ---

fn minimal_grammar() -> Grammar {
    GrammarBuilder::new("ad_v9_minimal")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build()
}

fn arith_grammar() -> Grammar {
    GrammarBuilder::new("ad_v9_arith")
        .token("num", r"\d+")
        .token("plus", "+")
        .token("star", "*")
        .rule("expr", vec!["num"])
        .rule("expr", vec!["expr", "plus", "expr"])
        .rule("expr", vec!["expr", "star", "expr"])
        .start("expr")
        .build()
}

fn prec_grammar() -> Grammar {
    GrammarBuilder::new("ad_v9_prec")
        .token("num", r"\d+")
        .token("plus", "+")
        .token("star", "*")
        .rule("expr", vec!["num"])
        .rule_with_precedence("expr", vec!["expr", "plus", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "star", "expr"], 2, Associativity::Left)
        .start("expr")
        .build()
}

fn extras_grammar() -> Grammar {
    GrammarBuilder::new("ad_v9_extras")
        .token("id", r"[a-z]+")
        .token("ws", r"[ \t]+")
        .extra("ws")
        .rule("start", vec!["id"])
        .start("start")
        .build()
}

fn inline_grammar() -> Grammar {
    GrammarBuilder::new("ad_v9_inline")
        .token("x", "x")
        .token("y", "y")
        .rule("inner", vec!["x"])
        .rule("outer", vec!["inner", "y"])
        .inline("inner")
        .start("outer")
        .build()
}

fn extern_grammar() -> Grammar {
    GrammarBuilder::new("ad_v9_extern")
        .token("id", r"[a-z]+")
        .token("colon", ":")
        .external("indent")
        .external("dedent")
        .rule("block", vec!["id", "colon", "indent", "id", "dedent"])
        .start("block")
        .build()
}

fn super_grammar() -> Grammar {
    GrammarBuilder::new("ad_v9_super")
        .token("num", r"\d+")
        .token("id", r"[a-z]+")
        .rule("literal", vec!["num"])
        .rule("ident", vec!["id"])
        .rule("expr", vec!["literal"])
        .rule("expr", vec!["ident"])
        .supertype("expr")
        .start("expr")
        .build()
}

fn conflict_grammar() -> Grammar {
    let mut g = GrammarBuilder::new("ad_v9_conflict")
        .token("num", r"\d+")
        .token("plus", "+")
        .rule("expr", vec!["num"])
        .rule("expr", vec!["expr", "plus", "expr"])
        .start("expr")
        .build();
    // Manually add a GLR conflict declaration
    let sym_ids: Vec<_> = g.rule_names.keys().copied().take(1).collect();
    if !sym_ids.is_empty() {
        g.conflicts.push(ConflictDeclaration {
            symbols: sym_ids,
            resolution: ConflictResolution::GLR,
        });
    }
    g
}

fn alt_grammar() -> Grammar {
    GrammarBuilder::new("ad_v9_alt")
        .token("num", r"\d+")
        .token("id", r"[a-z]+")
        .token("str", r#""[^"]*""#)
        .rule("value", vec!["num"])
        .rule("value", vec!["id"])
        .rule("value", vec!["str"])
        .start("value")
        .build()
}

fn chain_grammar() -> Grammar {
    GrammarBuilder::new("ad_v9_chain")
        .token("x", "x")
        .rule("a", vec!["x"])
        .rule("b", vec!["a"])
        .rule("c", vec!["b"])
        .rule("d", vec!["c"])
        .start("d")
        .build()
}

fn recurse_grammar() -> Grammar {
    GrammarBuilder::new("ad_v9_recurse")
        .token("lp", "(")
        .token("rp", ")")
        .token("x", "x")
        .rule("expr", vec!["x"])
        .rule("expr", vec!["lp", "expr", "rp"])
        .start("expr")
        .build()
}

fn large_grammar() -> Grammar {
    let mut builder = GrammarBuilder::new("ad_v9_large");
    for i in 0..20 {
        builder = builder.token(&format!("tok_{i}"), &format!("t{i}"));
    }
    for i in 0..15 {
        let tok = format!("tok_{i}");
        let rule_name = format!("rule_{i}");
        builder = builder.rule(&rule_name, vec![Box::leak(tok.into_boxed_str())]);
    }
    builder = builder
        .rule("top", vec!["rule_0"])
        .rule("top", vec!["rule_1"])
        .start("top");
    builder.build()
}

fn second_minimal_grammar() -> Grammar {
    GrammarBuilder::new("ad_v9_second")
        .token("b", "b")
        .rule("root", vec!["b"])
        .start("root")
        .build()
}

fn right_assoc_grammar() -> Grammar {
    GrammarBuilder::new("ad_v9_rassoc")
        .token("num", r"\d+")
        .token("pow", "^")
        .rule("expr", vec!["num"])
        .rule_with_precedence("expr", vec!["expr", "pow", "expr"], 3, Associativity::Right)
        .start("expr")
        .build()
}

fn multi_prec_grammar() -> Grammar {
    GrammarBuilder::new("ad_v9_mprec")
        .token("num", r"\d+")
        .token("plus", "+")
        .token("star", "*")
        .token("pow", "^")
        .rule("expr", vec!["num"])
        .rule_with_precedence("expr", vec!["expr", "plus", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "star", "expr"], 2, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "pow", "expr"], 3, Associativity::Right)
        .start("expr")
        .build()
}

fn deep_recurse_grammar() -> Grammar {
    GrammarBuilder::new("ad_v9_deep")
        .token("lp", "(")
        .token("rp", ")")
        .token("lb", "[")
        .token("rb", "]")
        .token("x", "x")
        .rule("expr", vec!["x"])
        .rule("expr", vec!["lp", "expr", "rp"])
        .rule("expr", vec!["lb", "expr", "rb"])
        .start("expr")
        .build()
}

fn multi_extras_grammar() -> Grammar {
    GrammarBuilder::new("ad_v9_mext")
        .token("id", r"[a-z]+")
        .token("ws", r"[ \t]+")
        .token("nl", r"\n")
        .extra("ws")
        .extra("nl")
        .rule("start", vec!["id"])
        .start("start")
        .build()
}

fn multi_extern_grammar() -> Grammar {
    GrammarBuilder::new("ad_v9_mextern")
        .token("id", r"[a-z]+")
        .external("indent")
        .external("dedent")
        .external("newline")
        .rule("block", vec!["indent", "id", "dedent"])
        .start("block")
        .build()
}

// ============================================================================
// 1. Build twice → same output
// ============================================================================

#[test]
fn twice_minimal() {
    let g = minimal_grammar();
    assert_eq!(generate(&g), generate(&g));
}

#[test]
fn twice_arith() {
    let g = arith_grammar();
    assert_eq!(generate(&g), generate(&g));
}

#[test]
fn twice_prec() {
    let g = prec_grammar();
    assert_eq!(generate(&g), generate(&g));
}

#[test]
fn twice_extras() {
    let g = extras_grammar();
    assert_eq!(generate(&g), generate(&g));
}

// ============================================================================
// 2. Build 3 times → all identical
// ============================================================================

#[test]
fn thrice_minimal() {
    let g = minimal_grammar();
    let a = generate(&g);
    let b = generate(&g);
    let c = generate(&g);
    assert_eq!(a, b);
    assert_eq!(b, c);
}

#[test]
fn thrice_arith() {
    let g = arith_grammar();
    let a = generate(&g);
    let b = generate(&g);
    let c = generate(&g);
    assert_eq!(a, b);
    assert_eq!(b, c);
}

#[test]
fn thrice_prec() {
    let g = prec_grammar();
    let a = generate(&g);
    let b = generate(&g);
    let c = generate(&g);
    assert_eq!(a, b);
    assert_eq!(b, c);
}

#[test]
fn thrice_large() {
    let g = large_grammar();
    let a = generate(&g);
    let b = generate(&g);
    let c = generate(&g);
    assert_eq!(a, b);
    assert_eq!(b, c);
}

// ============================================================================
// 3. Build 5 times → all identical
// ============================================================================

#[test]
fn five_minimal() {
    let g = minimal_grammar();
    let first = generate(&g);
    for _ in 0..4 {
        assert_eq!(first, generate(&g));
    }
}

#[test]
fn five_arith() {
    let g = arith_grammar();
    let first = generate(&g);
    for _ in 0..4 {
        assert_eq!(first, generate(&g));
    }
}

#[test]
fn five_extras() {
    let g = extras_grammar();
    let first = generate(&g);
    for _ in 0..4 {
        assert_eq!(first, generate(&g));
    }
}

#[test]
fn five_extern() {
    let g = extern_grammar();
    let first = generate(&g);
    for _ in 0..4 {
        assert_eq!(first, generate(&g));
    }
}

// ============================================================================
// 4. Build 10 times → all identical
// ============================================================================

#[test]
fn ten_minimal() {
    let g = minimal_grammar();
    let first = generate(&g);
    for _ in 0..9 {
        assert_eq!(first, generate(&g));
    }
}

#[test]
fn ten_arith() {
    let g = arith_grammar();
    let first = generate(&g);
    for _ in 0..9 {
        assert_eq!(first, generate(&g));
    }
}

#[test]
fn ten_large() {
    let g = large_grammar();
    let first = generate(&g);
    for _ in 0..9 {
        assert_eq!(first, generate(&g));
    }
}

#[test]
fn ten_prec() {
    let g = prec_grammar();
    let first = generate(&g);
    for _ in 0..9 {
        assert_eq!(first, generate(&g));
    }
}

// ============================================================================
// 5. Minimal grammar → deterministic
// ============================================================================

#[test]
fn minimal_fresh_builders_match() {
    let a = generate(&minimal_grammar());
    let b = generate(&minimal_grammar());
    assert_eq!(a, b);
}

#[test]
fn minimal_output_nonempty() {
    let code = generate(&minimal_grammar());
    assert!(!code.is_empty());
}

#[test]
fn minimal_stable_across_tables() {
    let g = minimal_grammar();
    let pt1 = build_table(&g);
    let pt2 = build_table(&g);
    let a = AbiLanguageBuilder::new(&g, &pt1).generate().to_string();
    let b = AbiLanguageBuilder::new(&g, &pt2).generate().to_string();
    assert_eq!(a, b);
}

#[test]
fn minimal_bytes_identical() {
    let g = minimal_grammar();
    let a = generate(&g);
    let b = generate(&g);
    assert_eq!(a.as_bytes(), b.as_bytes());
}

// ============================================================================
// 6. Arithmetic grammar → deterministic
// ============================================================================

#[test]
fn arith_fresh_builders_match() {
    let a = generate(&arith_grammar());
    let b = generate(&arith_grammar());
    assert_eq!(a, b);
}

#[test]
fn arith_output_nonempty() {
    let code = generate(&arith_grammar());
    assert!(!code.is_empty());
}

#[test]
fn arith_stable_across_tables() {
    let g = arith_grammar();
    let pt1 = build_table(&g);
    let pt2 = build_table(&g);
    let a = AbiLanguageBuilder::new(&g, &pt1).generate().to_string();
    let b = AbiLanguageBuilder::new(&g, &pt2).generate().to_string();
    assert_eq!(a, b);
}

#[test]
fn arith_bytes_identical() {
    let g = arith_grammar();
    let a = generate(&g);
    let b = generate(&g);
    assert_eq!(a.as_bytes(), b.as_bytes());
}

// ============================================================================
// 7. Grammar with precedence → deterministic
// ============================================================================

#[test]
fn prec_fresh_builders_match() {
    let a = generate(&prec_grammar());
    let b = generate(&prec_grammar());
    assert_eq!(a, b);
}

#[test]
fn prec_right_assoc_deterministic() {
    let g = right_assoc_grammar();
    assert_eq!(generate(&g), generate(&g));
}

#[test]
fn prec_multi_level_deterministic() {
    let g = multi_prec_grammar();
    assert_eq!(generate(&g), generate(&g));
}

#[test]
fn prec_multi_five_times() {
    let g = multi_prec_grammar();
    let first = generate(&g);
    for _ in 0..4 {
        assert_eq!(first, generate(&g));
    }
}

// ============================================================================
// 8. Grammar with extras → deterministic
// ============================================================================

#[test]
fn extras_fresh_builders_match() {
    let a = generate(&extras_grammar());
    let b = generate(&extras_grammar());
    assert_eq!(a, b);
}

#[test]
fn extras_multi_deterministic() {
    let g = multi_extras_grammar();
    assert_eq!(generate(&g), generate(&g));
}

#[test]
fn extras_five_times() {
    let g = extras_grammar();
    let first = generate(&g);
    for _ in 0..4 {
        assert_eq!(first, generate(&g));
    }
}

#[test]
fn extras_multi_five_times() {
    let g = multi_extras_grammar();
    let first = generate(&g);
    for _ in 0..4 {
        assert_eq!(first, generate(&g));
    }
}

// ============================================================================
// 9. Grammar with inline → deterministic
// ============================================================================

#[test]
fn inline_fresh_builders_match() {
    let a = generate(&inline_grammar());
    let b = generate(&inline_grammar());
    assert_eq!(a, b);
}

#[test]
fn inline_three_times() {
    let g = inline_grammar();
    let a = generate(&g);
    let b = generate(&g);
    let c = generate(&g);
    assert_eq!(a, b);
    assert_eq!(b, c);
}

#[test]
fn inline_bytes_identical() {
    let g = inline_grammar();
    let a = generate(&g);
    let b = generate(&g);
    assert_eq!(a.as_bytes(), b.as_bytes());
}

#[test]
fn inline_stable_across_tables() {
    let g = inline_grammar();
    let pt1 = build_table(&g);
    let pt2 = build_table(&g);
    let a = AbiLanguageBuilder::new(&g, &pt1).generate().to_string();
    let b = AbiLanguageBuilder::new(&g, &pt2).generate().to_string();
    assert_eq!(a, b);
}

// ============================================================================
// 10. Grammar with externals → deterministic
// ============================================================================

#[test]
fn extern_fresh_builders_match() {
    let a = generate(&extern_grammar());
    let b = generate(&extern_grammar());
    assert_eq!(a, b);
}

#[test]
fn extern_multi_deterministic() {
    let g = multi_extern_grammar();
    assert_eq!(generate(&g), generate(&g));
}

#[test]
fn extern_five_times() {
    let g = extern_grammar();
    let first = generate(&g);
    for _ in 0..4 {
        assert_eq!(first, generate(&g));
    }
}

#[test]
fn extern_multi_five_times() {
    let g = multi_extern_grammar();
    let first = generate(&g);
    for _ in 0..4 {
        assert_eq!(first, generate(&g));
    }
}

// ============================================================================
// 11. Grammar with supertypes → deterministic
// ============================================================================

#[test]
fn super_fresh_builders_match() {
    let a = generate(&super_grammar());
    let b = generate(&super_grammar());
    assert_eq!(a, b);
}

#[test]
fn super_three_times() {
    let g = super_grammar();
    let a = generate(&g);
    let b = generate(&g);
    let c = generate(&g);
    assert_eq!(a, b);
    assert_eq!(b, c);
}

#[test]
fn super_bytes_identical() {
    let g = super_grammar();
    let a = generate(&g);
    let b = generate(&g);
    assert_eq!(a.as_bytes(), b.as_bytes());
}

#[test]
fn super_stable_across_tables() {
    let g = super_grammar();
    let pt1 = build_table(&g);
    let pt2 = build_table(&g);
    let a = AbiLanguageBuilder::new(&g, &pt1).generate().to_string();
    let b = AbiLanguageBuilder::new(&g, &pt2).generate().to_string();
    assert_eq!(a, b);
}

// ============================================================================
// 12. Grammar with conflicts → deterministic
// ============================================================================

#[test]
fn conflict_fresh_builders_match() {
    let a = generate(&conflict_grammar());
    let b = generate(&conflict_grammar());
    assert_eq!(a, b);
}

#[test]
fn conflict_three_times() {
    let g = conflict_grammar();
    let a = generate(&g);
    let b = generate(&g);
    let c = generate(&g);
    assert_eq!(a, b);
    assert_eq!(b, c);
}

#[test]
fn conflict_five_times() {
    let g = conflict_grammar();
    let first = generate(&g);
    for _ in 0..4 {
        assert_eq!(first, generate(&g));
    }
}

#[test]
fn conflict_bytes_identical() {
    let g = conflict_grammar();
    let a = generate(&g);
    let b = generate(&g);
    assert_eq!(a.as_bytes(), b.as_bytes());
}

// ============================================================================
// 13. Grammar with alternatives → deterministic
// ============================================================================

#[test]
fn alt_fresh_builders_match() {
    let a = generate(&alt_grammar());
    let b = generate(&alt_grammar());
    assert_eq!(a, b);
}

#[test]
fn alt_three_times() {
    let g = alt_grammar();
    let a = generate(&g);
    let b = generate(&g);
    let c = generate(&g);
    assert_eq!(a, b);
    assert_eq!(b, c);
}

#[test]
fn alt_five_times() {
    let g = alt_grammar();
    let first = generate(&g);
    for _ in 0..4 {
        assert_eq!(first, generate(&g));
    }
}

#[test]
fn alt_bytes_identical() {
    let g = alt_grammar();
    let a = generate(&g);
    let b = generate(&g);
    assert_eq!(a.as_bytes(), b.as_bytes());
}

// ============================================================================
// 14. Grammar with chain rules → deterministic
// ============================================================================

#[test]
fn chain_fresh_builders_match() {
    let a = generate(&chain_grammar());
    let b = generate(&chain_grammar());
    assert_eq!(a, b);
}

#[test]
fn chain_three_times() {
    let g = chain_grammar();
    let a = generate(&g);
    let b = generate(&g);
    let c = generate(&g);
    assert_eq!(a, b);
    assert_eq!(b, c);
}

#[test]
fn chain_five_times() {
    let g = chain_grammar();
    let first = generate(&g);
    for _ in 0..4 {
        assert_eq!(first, generate(&g));
    }
}

#[test]
fn chain_bytes_identical() {
    let g = chain_grammar();
    let a = generate(&g);
    let b = generate(&g);
    assert_eq!(a.as_bytes(), b.as_bytes());
}

// ============================================================================
// 15. Grammar with recursion → deterministic
// ============================================================================

#[test]
fn recurse_fresh_builders_match() {
    let a = generate(&recurse_grammar());
    let b = generate(&recurse_grammar());
    assert_eq!(a, b);
}

#[test]
fn recurse_deep_deterministic() {
    let g = deep_recurse_grammar();
    assert_eq!(generate(&g), generate(&g));
}

#[test]
fn recurse_five_times() {
    let g = recurse_grammar();
    let first = generate(&g);
    for _ in 0..4 {
        assert_eq!(first, generate(&g));
    }
}

#[test]
fn recurse_deep_five_times() {
    let g = deep_recurse_grammar();
    let first = generate(&g);
    for _ in 0..4 {
        assert_eq!(first, generate(&g));
    }
}

// ============================================================================
// 16. Large grammar → deterministic
// ============================================================================

#[test]
fn large_fresh_builders_match() {
    let a = generate(&large_grammar());
    let b = generate(&large_grammar());
    assert_eq!(a, b);
}

#[test]
fn large_five_times() {
    let g = large_grammar();
    let first = generate(&g);
    for _ in 0..4 {
        assert_eq!(first, generate(&g));
    }
}

#[test]
fn large_bytes_identical() {
    let g = large_grammar();
    let a = generate(&g);
    let b = generate(&g);
    assert_eq!(a.as_bytes(), b.as_bytes());
}

#[test]
fn large_stable_across_tables() {
    let g = large_grammar();
    let pt1 = build_table(&g);
    let pt2 = build_table(&g);
    let a = AbiLanguageBuilder::new(&g, &pt1).generate().to_string();
    let b = AbiLanguageBuilder::new(&g, &pt2).generate().to_string();
    assert_eq!(a, b);
}

// ============================================================================
// 17. Code length is deterministic
// ============================================================================

#[test]
fn len_minimal() {
    let g = minimal_grammar();
    assert_eq!(generate(&g).len(), generate(&g).len());
}

#[test]
fn len_arith() {
    let g = arith_grammar();
    assert_eq!(generate(&g).len(), generate(&g).len());
}

#[test]
fn len_prec() {
    let g = prec_grammar();
    assert_eq!(generate(&g).len(), generate(&g).len());
}

#[test]
fn len_large() {
    let g = large_grammar();
    assert_eq!(generate(&g).len(), generate(&g).len());
}

#[test]
fn len_extras() {
    let g = extras_grammar();
    assert_eq!(generate(&g).len(), generate(&g).len());
}

#[test]
fn len_extern() {
    let g = extern_grammar();
    assert_eq!(generate(&g).len(), generate(&g).len());
}

#[test]
fn len_chain() {
    let g = chain_grammar();
    assert_eq!(generate(&g).len(), generate(&g).len());
}

#[test]
fn len_recurse() {
    let g = recurse_grammar();
    assert_eq!(generate(&g).len(), generate(&g).len());
}

// ============================================================================
// 18. Different grammars → different output (non-degeneracy)
// ============================================================================

#[test]
fn nondegen_minimal_vs_arith() {
    let a = generate(&minimal_grammar());
    let b = generate(&arith_grammar());
    assert_ne!(a, b);
}

#[test]
fn nondegen_minimal_vs_second() {
    let a = generate(&minimal_grammar());
    let b = generate(&second_minimal_grammar());
    assert_ne!(a, b);
}

#[test]
fn nondegen_arith_vs_prec() {
    let a = generate(&arith_grammar());
    let b = generate(&prec_grammar());
    assert_ne!(a, b);
}

#[test]
fn nondegen_chain_vs_recurse() {
    let a = generate(&chain_grammar());
    let b = generate(&recurse_grammar());
    assert_ne!(a, b);
}

#[test]
fn nondegen_extras_vs_extern() {
    let a = generate(&extras_grammar());
    let b = generate(&extern_grammar());
    assert_ne!(a, b);
}

#[test]
fn nondegen_alt_vs_chain() {
    let a = generate(&alt_grammar());
    let b = generate(&chain_grammar());
    assert_ne!(a, b);
}

#[test]
fn nondegen_inline_vs_super() {
    let a = generate(&inline_grammar());
    let b = generate(&super_grammar());
    assert_ne!(a, b);
}

#[test]
fn nondegen_large_vs_minimal() {
    let a = generate(&large_grammar());
    let b = generate(&minimal_grammar());
    assert_ne!(a, b);
}

// ============================================================================
// 19. Output contains same grammar name each time
// ============================================================================

#[test]
fn name_minimal_stable() {
    let g = minimal_grammar();
    let a = generate(&g);
    let b = generate(&g);
    let count_a = a.matches("ad_v9_minimal").count();
    let count_b = b.matches("ad_v9_minimal").count();
    assert!(count_a > 0, "grammar name must appear in output");
    assert_eq!(count_a, count_b);
}

#[test]
fn name_arith_stable() {
    let g = arith_grammar();
    let a = generate(&g);
    let b = generate(&g);
    let count_a = a.matches("ad_v9_arith").count();
    let count_b = b.matches("ad_v9_arith").count();
    assert!(count_a > 0, "grammar name must appear in output");
    assert_eq!(count_a, count_b);
}

#[test]
fn name_prec_stable() {
    let g = prec_grammar();
    let a = generate(&g);
    let b = generate(&g);
    let count_a = a.matches("ad_v9_prec").count();
    let count_b = b.matches("ad_v9_prec").count();
    assert!(count_a > 0, "grammar name must appear in output");
    assert_eq!(count_a, count_b);
}

#[test]
fn name_large_stable() {
    let g = large_grammar();
    let a = generate(&g);
    let b = generate(&g);
    let count_a = a.matches("ad_v9_large").count();
    let count_b = b.matches("ad_v9_large").count();
    assert!(count_a > 0, "grammar name must appear in output");
    assert_eq!(count_a, count_b);
}

// ============================================================================
// 20. Output byte-for-byte identical
// ============================================================================

#[test]
fn byte_minimal() {
    let g = minimal_grammar();
    let a = generate(&g).into_bytes();
    let b = generate(&g).into_bytes();
    assert_eq!(a.len(), b.len());
    assert!(a.iter().zip(b.iter()).all(|(x, y)| x == y));
}

#[test]
fn byte_arith() {
    let g = arith_grammar();
    let a = generate(&g).into_bytes();
    let b = generate(&g).into_bytes();
    assert_eq!(a.len(), b.len());
    assert!(a.iter().zip(b.iter()).all(|(x, y)| x == y));
}

#[test]
fn byte_prec() {
    let g = prec_grammar();
    let a = generate(&g).into_bytes();
    let b = generate(&g).into_bytes();
    assert_eq!(a.len(), b.len());
    assert!(a.iter().zip(b.iter()).all(|(x, y)| x == y));
}

#[test]
fn byte_large() {
    let g = large_grammar();
    let a = generate(&g).into_bytes();
    let b = generate(&g).into_bytes();
    assert_eq!(a.len(), b.len());
    assert!(a.iter().zip(b.iter()).all(|(x, y)| x == y));
}
