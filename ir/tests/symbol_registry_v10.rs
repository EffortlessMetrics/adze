//! Comprehensive tests for SymbolRegistry integration with Grammar in adze-ir.
//!
//! 105 tests across 20 categories:
//!   1.  empty_grammar           — empty grammar has no symbols found by name
//!   2.  one_token               — find_symbol_by_name finds a single token
//!   3.  one_rule                — find non-terminal by name
//!   4.  multiple_tokens         — all tokens findable
//!   5.  nonexistent_name        — returns None for missing names
//!   6.  token_terminal          — token metadata is terminal
//!   7.  nonterminal_metadata    — non-terminal metadata is non-terminal
//!   8.  find_after_normalize    — still works after normalize
//!   9.  find_after_optimize     — still works after optimize
//!  10.  clone_preserves         — clone preserves registry
//!  11.  debug_shows             — Debug format includes registry info
//!  12.  serde_roundtrip         — serde roundtrip preserves registry
//!  13.  deterministic_names     — symbol names are deterministic
//!  14.  different_grammars      — different grammars have different registries
//!  15.  extras_entries          — extra symbols have entries
//!  16.  external_entries        — external symbols have entries
//!  17.  inline_entries          — inline symbols have entries
//!  18.  supertype_entries       — supertype symbols have entries
//!  19.  many_tokens             — registry after adding many tokens
//!  20.  many_rules              — registry after adding many rules

use adze_ir::builder::GrammarBuilder;
use adze_ir::{Grammar, SymbolId};
use std::collections::HashSet;

// ── Helpers ──────────────────────────────────────────────────────────

fn empty_grammar() -> Grammar {
    GrammarBuilder::new("sr_v10_empty").build()
}

fn one_token_grammar() -> Grammar {
    GrammarBuilder::new("sr_v10_one_tok")
        .token("kw", "kw")
        .rule("root", vec!["kw"])
        .start("root")
        .build()
}

fn one_rule_grammar() -> Grammar {
    GrammarBuilder::new("sr_v10_one_rule")
        .token("num", "\\d+")
        .rule("expr", vec!["num"])
        .start("expr")
        .build()
}

fn multi_token_grammar() -> Grammar {
    GrammarBuilder::new("sr_v10_multi_tok")
        .token("alpha", "a")
        .token("beta", "b")
        .token("gamma", "c")
        .token("delta", "d")
        .token("epsilon", "e")
        .rule("seq", vec!["alpha", "beta", "gamma", "delta", "epsilon"])
        .start("seq")
        .build()
}

fn extra_grammar() -> Grammar {
    GrammarBuilder::new("sr_v10_extra")
        .token("num", "\\d+")
        .token("ws", "[ \\t]+")
        .token("comment", "//[^\\n]*")
        .rule("root", vec!["num"])
        .extra("ws")
        .extra("comment")
        .start("root")
        .build()
}

fn external_grammar() -> Grammar {
    GrammarBuilder::new("sr_v10_external")
        .token("num", "\\d+")
        .rule("root", vec!["num"])
        .external("indent")
        .external("dedent")
        .start("root")
        .build()
}

fn inline_grammar() -> Grammar {
    GrammarBuilder::new("sr_v10_inline")
        .token("num", "\\d+")
        .token("id", "[a-z]+")
        .rule("primary", vec!["num"])
        .rule("primary", vec!["id"])
        .rule("expr", vec!["primary"])
        .inline("primary")
        .start("expr")
        .build()
}

fn supertype_grammar() -> Grammar {
    GrammarBuilder::new("sr_v10_super")
        .token("num", "\\d+")
        .token("id", "[a-z]+")
        .rule("literal", vec!["num"])
        .rule("variable", vec!["id"])
        .rule("expression", vec!["literal"])
        .rule("expression", vec!["variable"])
        .supertype("expression")
        .start("expression")
        .build()
}

fn many_token_grammar() -> Grammar {
    let mut b = GrammarBuilder::new("sr_v10_many_tok");
    let mut names = Vec::new();
    for i in 0..20 {
        let name = format!("tok{i}");
        let pat = format!("p{i}");
        b = b.token(
            Box::leak(name.clone().into_boxed_str()),
            Box::leak(pat.into_boxed_str()),
        );
        names.push(name);
    }
    b = b.rule("root", vec![Box::leak(names[0].clone().into_boxed_str())]);
    b = b.start("root");
    b.build()
}

fn many_rule_grammar() -> Grammar {
    GrammarBuilder::new("sr_v10_many_rules")
        .token("num", "\\d+")
        .token("id", "[a-z]+")
        .rule("leaf", vec!["num"])
        .rule("leaf", vec!["id"])
        .rule("atom", vec!["leaf"])
        .rule("factor", vec!["atom"])
        .rule("term", vec!["factor"])
        .rule("sum", vec!["term"])
        .rule("comparison", vec!["sum"])
        .rule("equality", vec!["comparison"])
        .rule("conjunction", vec!["equality"])
        .rule("disjunction", vec!["conjunction"])
        .rule("expression", vec!["disjunction"])
        .rule("statement", vec!["expression"])
        .rule("block", vec!["statement"])
        .rule("program", vec!["block"])
        .start("program")
        .build()
}

// ═══════════════════════════════════════════════════════════════════
// 1. empty_grammar — empty grammar has no symbols found by name
// ═══════════════════════════════════════════════════════════════════

#[test]
fn sr_v10_empty_find_returns_none() {
    let g = empty_grammar();
    assert!(g.find_symbol_by_name("anything").is_none());
}

#[test]
fn sr_v10_empty_rule_names_empty() {
    let g = empty_grammar();
    assert!(g.rule_names.is_empty());
}

#[test]
fn sr_v10_empty_tokens_empty() {
    let g = empty_grammar();
    assert!(g.tokens.is_empty());
}

#[test]
fn sr_v10_empty_build_registry_has_eof() {
    let g = empty_grammar();
    let reg = g.build_registry();
    // EOF ("end") is always registered
    assert!(reg.get_id("end").is_some());
}

#[test]
fn sr_v10_empty_registry_len_is_one() {
    let g = empty_grammar();
    let reg = g.build_registry();
    // Only EOF
    assert_eq!(reg.len(), 1);
}

// ═══════════════════════════════════════════════════════════════════
// 2. one_token — find_symbol_by_name finds a single token
// ═══════════════════════════════════════════════════════════════════

#[test]
fn sr_v10_one_token_findable() {
    let g = one_token_grammar();
    assert!(g.find_symbol_by_name("kw").is_some());
}

#[test]
fn sr_v10_one_token_id_nonzero() {
    let g = one_token_grammar();
    let id = g.find_symbol_by_name("kw").unwrap();
    assert!(id.0 > 0);
}

#[test]
fn sr_v10_one_token_in_tokens_map() {
    let g = one_token_grammar();
    let id = g.find_symbol_by_name("kw").unwrap();
    assert!(g.tokens.contains_key(&id));
}

#[test]
fn sr_v10_one_token_registry_contains() {
    let g = one_token_grammar();
    let reg = g.build_registry();
    assert!(reg.get_id("kw").is_some());
}

#[test]
fn sr_v10_one_token_registry_name_roundtrip() {
    let g = one_token_grammar();
    let reg = g.build_registry();
    let id = reg.get_id("kw").unwrap();
    assert_eq!(reg.get_name(id), Some("kw"));
}

// ═══════════════════════════════════════════════════════════════════
// 3. one_rule — find non-terminal by name
// ═══════════════════════════════════════════════════════════════════

#[test]
fn sr_v10_one_rule_find_nonterminal() {
    let g = one_rule_grammar();
    assert!(g.find_symbol_by_name("expr").is_some());
}

#[test]
fn sr_v10_one_rule_has_rules() {
    let g = one_rule_grammar();
    let id = g.find_symbol_by_name("expr").unwrap();
    assert!(g.rules.contains_key(&id));
}

#[test]
fn sr_v10_one_rule_registry_nonterminal() {
    let g = one_rule_grammar();
    let reg = g.build_registry();
    let id = reg.get_id("expr").unwrap();
    let meta = reg.get_metadata(id).unwrap();
    assert!(!meta.terminal);
}

#[test]
fn sr_v10_one_rule_registry_named() {
    let g = one_rule_grammar();
    let reg = g.build_registry();
    let id = reg.get_id("expr").unwrap();
    let meta = reg.get_metadata(id).unwrap();
    assert!(meta.named);
}

#[test]
fn sr_v10_one_rule_token_also_findable() {
    let g = one_rule_grammar();
    assert!(g.find_symbol_by_name("num").is_some());
}

// ═══════════════════════════════════════════════════════════════════
// 4. multiple_tokens — all tokens findable
// ═══════════════════════════════════════════════════════════════════

#[test]
fn sr_v10_multi_tok_alpha_findable() {
    let g = multi_token_grammar();
    assert!(g.find_symbol_by_name("alpha").is_some());
}

#[test]
fn sr_v10_multi_tok_beta_findable() {
    let g = multi_token_grammar();
    assert!(g.find_symbol_by_name("beta").is_some());
}

#[test]
fn sr_v10_multi_tok_gamma_findable() {
    let g = multi_token_grammar();
    assert!(g.find_symbol_by_name("gamma").is_some());
}

#[test]
fn sr_v10_multi_tok_all_different_ids() {
    let g = multi_token_grammar();
    let ids: HashSet<u16> = ["alpha", "beta", "gamma", "delta", "epsilon"]
        .iter()
        .filter_map(|n| g.find_symbol_by_name(n))
        .map(|id| id.0)
        .collect();
    assert_eq!(ids.len(), 5);
}

#[test]
fn sr_v10_multi_tok_registry_has_all() {
    let g = multi_token_grammar();
    let reg = g.build_registry();
    for name in &["alpha", "beta", "gamma", "delta", "epsilon"] {
        assert!(reg.get_id(name).is_some(), "missing: {name}");
    }
}

#[test]
fn sr_v10_multi_tok_seq_findable() {
    let g = multi_token_grammar();
    assert!(g.find_symbol_by_name("seq").is_some());
}

// ═══════════════════════════════════════════════════════════════════
// 5. nonexistent_name — returns None for missing names
// ═══════════════════════════════════════════════════════════════════

#[test]
fn sr_v10_nonexistent_returns_none() {
    let g = one_token_grammar();
    assert!(g.find_symbol_by_name("zzz_missing").is_none());
}

#[test]
fn sr_v10_nonexistent_empty_string() {
    let g = one_token_grammar();
    assert!(g.find_symbol_by_name("").is_none());
}

#[test]
fn sr_v10_nonexistent_case_sensitive() {
    let g = one_token_grammar();
    // "kw" exists but "KW" does not
    assert!(g.find_symbol_by_name("KW").is_none());
}

#[test]
fn sr_v10_nonexistent_partial_match() {
    let g = multi_token_grammar();
    assert!(g.find_symbol_by_name("alph").is_none());
}

#[test]
fn sr_v10_nonexistent_registry_get_id_none() {
    let g = one_token_grammar();
    let reg = g.build_registry();
    assert!(reg.get_id("nonexistent").is_none());
}

// ═══════════════════════════════════════════════════════════════════
// 6. token_terminal — token metadata is terminal
// ═══════════════════════════════════════════════════════════════════

#[test]
fn sr_v10_token_terminal_flag() {
    let g = one_token_grammar();
    let reg = g.build_registry();
    let id = reg.get_id("kw").unwrap();
    let meta = reg.get_metadata(id).unwrap();
    assert!(meta.terminal);
}

#[test]
fn sr_v10_token_terminal_not_named() {
    let g = one_token_grammar();
    let reg = g.build_registry();
    let id = reg.get_id("kw").unwrap();
    let meta = reg.get_metadata(id).unwrap();
    assert!(!meta.named);
}

#[test]
fn sr_v10_multi_token_all_terminal() {
    let g = multi_token_grammar();
    let reg = g.build_registry();
    for name in &["alpha", "beta", "gamma", "delta", "epsilon"] {
        let id = reg.get_id(name).unwrap();
        let meta = reg.get_metadata(id).unwrap();
        assert!(meta.terminal, "{name} should be terminal");
    }
}

#[test]
fn sr_v10_token_terminal_visible() {
    let g = one_token_grammar();
    let reg = g.build_registry();
    let id = reg.get_id("kw").unwrap();
    let meta = reg.get_metadata(id).unwrap();
    assert!(meta.visible);
}

#[test]
fn sr_v10_eof_is_terminal() {
    let g = one_token_grammar();
    let reg = g.build_registry();
    let eof_id = reg.get_id("end").unwrap();
    let meta = reg.get_metadata(eof_id).unwrap();
    assert!(meta.terminal);
}

// ═══════════════════════════════════════════════════════════════════
// 7. nonterminal_metadata — non-terminal metadata is non-terminal
// ═══════════════════════════════════════════════════════════════════

#[test]
fn sr_v10_nonterminal_not_terminal() {
    let g = one_rule_grammar();
    let reg = g.build_registry();
    let id = reg.get_id("expr").unwrap();
    let meta = reg.get_metadata(id).unwrap();
    assert!(!meta.terminal);
}

#[test]
fn sr_v10_nonterminal_is_named() {
    let g = one_rule_grammar();
    let reg = g.build_registry();
    let id = reg.get_id("expr").unwrap();
    let meta = reg.get_metadata(id).unwrap();
    assert!(meta.named);
}

#[test]
fn sr_v10_nonterminal_is_visible() {
    let g = one_rule_grammar();
    let reg = g.build_registry();
    let id = reg.get_id("expr").unwrap();
    let meta = reg.get_metadata(id).unwrap();
    assert!(meta.visible);
}

#[test]
fn sr_v10_nonterminal_not_hidden() {
    let g = one_rule_grammar();
    let reg = g.build_registry();
    let id = reg.get_id("expr").unwrap();
    let meta = reg.get_metadata(id).unwrap();
    assert!(!meta.hidden);
}

#[test]
fn sr_v10_rule_and_token_differ_in_terminal() {
    let g = one_rule_grammar();
    let reg = g.build_registry();
    let expr_meta = reg.get_metadata(reg.get_id("expr").unwrap()).unwrap();
    let num_meta = reg.get_metadata(reg.get_id("num").unwrap()).unwrap();
    assert!(!expr_meta.terminal);
    assert!(num_meta.terminal);
}

// ═══════════════════════════════════════════════════════════════════
// 8. find_after_normalize — still works after normalize
// ═══════════════════════════════════════════════════════════════════

#[test]
fn sr_v10_normalize_preserves_find() {
    let mut g = one_rule_grammar();
    let _aux = g.normalize();
    assert!(g.find_symbol_by_name("expr").is_some());
}

#[test]
fn sr_v10_normalize_preserves_token_find() {
    let mut g = one_rule_grammar();
    let _aux = g.normalize();
    assert!(g.find_symbol_by_name("num").is_some());
}

#[test]
fn sr_v10_normalize_preserves_registry_build() {
    let mut g = multi_token_grammar();
    let _aux = g.normalize();
    let reg = g.build_registry();
    assert!(reg.get_id("alpha").is_some());
}

#[test]
fn sr_v10_normalize_multi_rule_find() {
    let mut g = many_rule_grammar();
    let _aux = g.normalize();
    assert!(g.find_symbol_by_name("program").is_some());
}

#[test]
fn sr_v10_normalize_preserves_metadata() {
    let mut g = one_rule_grammar();
    let _aux = g.normalize();
    let reg = g.build_registry();
    let id = reg.get_id("expr").unwrap();
    let meta = reg.get_metadata(id).unwrap();
    assert!(!meta.terminal);
}

// ═══════════════════════════════════════════════════════════════════
// 9. find_after_optimize — still works after optimize
// ═══════════════════════════════════════════════════════════════════

#[test]
fn sr_v10_optimize_preserves_find() {
    let mut g = one_rule_grammar();
    g.optimize();
    assert!(g.find_symbol_by_name("expr").is_some());
}

#[test]
fn sr_v10_optimize_preserves_token_find() {
    let mut g = one_rule_grammar();
    g.optimize();
    assert!(g.find_symbol_by_name("num").is_some());
}

#[test]
fn sr_v10_optimize_preserves_registry_build() {
    let mut g = multi_token_grammar();
    g.optimize();
    let reg = g.build_registry();
    assert!(reg.get_id("beta").is_some());
}

#[test]
fn sr_v10_optimize_many_rules_find() {
    let mut g = many_rule_grammar();
    g.optimize();
    assert!(g.find_symbol_by_name("statement").is_some());
}

#[test]
fn sr_v10_optimize_preserves_metadata() {
    let mut g = one_rule_grammar();
    g.optimize();
    let reg = g.build_registry();
    let id = reg.get_id("num").unwrap();
    let meta = reg.get_metadata(id).unwrap();
    assert!(meta.terminal);
}

// ═══════════════════════════════════════════════════════════════════
// 10. clone_preserves — clone preserves registry
// ═══════════════════════════════════════════════════════════════════

#[test]
fn sr_v10_clone_preserves_find() {
    let g = one_rule_grammar();
    let g2 = g.clone();
    assert_eq!(
        g.find_symbol_by_name("expr"),
        g2.find_symbol_by_name("expr")
    );
}

#[test]
fn sr_v10_clone_preserves_rule_names() {
    let g = multi_token_grammar();
    let g2 = g.clone();
    assert_eq!(g.rule_names, g2.rule_names);
}

#[test]
fn sr_v10_clone_preserves_tokens() {
    let g = multi_token_grammar();
    let g2 = g.clone();
    assert_eq!(g.tokens.len(), g2.tokens.len());
}

#[test]
fn sr_v10_clone_registry_equal() {
    let mut g = one_rule_grammar();
    let _reg = g.get_or_build_registry();
    let g2 = g.clone();
    assert_eq!(g.symbol_registry, g2.symbol_registry);
}

#[test]
fn sr_v10_clone_independent_mutation() {
    let g = one_rule_grammar();
    let mut g2 = g.clone();
    g2.optimize();
    // Original unaffected
    assert!(g.find_symbol_by_name("expr").is_some());
}

// ═══════════════════════════════════════════════════════════════════
// 11. debug_shows — Debug format includes registry info
// ═══════════════════════════════════════════════════════════════════

#[test]
fn sr_v10_debug_contains_grammar_name() {
    let g = one_token_grammar();
    let dbg = format!("{g:?}");
    assert!(dbg.contains("sr_v10_one_tok"));
}

#[test]
fn sr_v10_debug_contains_rule_names() {
    let g = one_rule_grammar();
    let dbg = format!("{g:?}");
    assert!(dbg.contains("rule_names"));
}

#[test]
fn sr_v10_debug_contains_tokens() {
    let g = one_token_grammar();
    let dbg = format!("{g:?}");
    assert!(dbg.contains("tokens"));
}

#[test]
fn sr_v10_debug_contains_symbol_registry_field() {
    let g = one_token_grammar();
    let dbg = format!("{g:?}");
    assert!(dbg.contains("symbol_registry"));
}

#[test]
fn sr_v10_debug_registry_when_built() {
    let mut g = one_token_grammar();
    let _reg = g.get_or_build_registry();
    let dbg = format!("{g:?}");
    assert!(dbg.contains("SymbolRegistry"));
}

// ═══════════════════════════════════════════════════════════════════
// 12. serde_roundtrip — serde roundtrip preserves registry
// ═══════════════════════════════════════════════════════════════════

#[test]
fn sr_v10_serde_roundtrip_basic() {
    let g = one_rule_grammar();
    let json = serde_json::to_string(&g).unwrap();
    let g2: Grammar = serde_json::from_str(&json).unwrap();
    assert_eq!(g, g2);
}

#[test]
fn sr_v10_serde_roundtrip_preserves_find() {
    let g = one_rule_grammar();
    let json = serde_json::to_string(&g).unwrap();
    let g2: Grammar = serde_json::from_str(&json).unwrap();
    assert_eq!(
        g.find_symbol_by_name("expr"),
        g2.find_symbol_by_name("expr")
    );
}

#[test]
fn sr_v10_serde_roundtrip_multi_token() {
    let g = multi_token_grammar();
    let json = serde_json::to_string(&g).unwrap();
    let g2: Grammar = serde_json::from_str(&json).unwrap();
    assert_eq!(g.tokens.len(), g2.tokens.len());
}

#[test]
fn sr_v10_serde_roundtrip_with_registry() {
    let mut g = one_rule_grammar();
    let _reg = g.get_or_build_registry();
    let json = serde_json::to_string(&g).unwrap();
    let g2: Grammar = serde_json::from_str(&json).unwrap();
    assert_eq!(g.symbol_registry, g2.symbol_registry);
}

#[test]
fn sr_v10_serde_roundtrip_registry_ids() {
    let mut g = multi_token_grammar();
    let _reg = g.get_or_build_registry();
    let json = serde_json::to_string(&g).unwrap();
    let mut g2: Grammar = serde_json::from_str(&json).unwrap();
    let reg2 = g2.get_or_build_registry();
    assert!(reg2.get_id("alpha").is_some());
}

// ═══════════════════════════════════════════════════════════════════
// 13. deterministic_names — symbol names are deterministic
// ═══════════════════════════════════════════════════════════════════

#[test]
fn sr_v10_deterministic_build_twice() {
    let g = one_rule_grammar();
    let reg1 = g.build_registry();
    let reg2 = g.build_registry();
    assert_eq!(reg1, reg2);
}

#[test]
fn sr_v10_deterministic_same_builder() {
    let g1 = GrammarBuilder::new("sr_v10_det_a")
        .token("x", "x")
        .rule("r", vec!["x"])
        .start("r")
        .build();
    let g2 = GrammarBuilder::new("sr_v10_det_a")
        .token("x", "x")
        .rule("r", vec!["x"])
        .start("r")
        .build();
    let reg1 = g1.build_registry();
    let reg2 = g2.build_registry();
    for (name1, info1) in reg1.iter() {
        let info2 = reg2.iter().find(|(n, _)| *n == name1).unwrap().1;
        assert_eq!(info1.id, info2.id, "mismatch for {name1}");
    }
}

#[test]
fn sr_v10_deterministic_ids_ordered() {
    let g = multi_token_grammar();
    let reg = g.build_registry();
    let ids: Vec<u16> = reg.iter().map(|(_, info)| info.id.0).collect();
    // EOF should be id 0
    assert_eq!(ids[0], 0);
}

#[test]
fn sr_v10_deterministic_eof_always_zero() {
    let g = many_rule_grammar();
    let reg = g.build_registry();
    assert_eq!(reg.get_id("end"), Some(SymbolId(0)));
}

#[test]
fn sr_v10_deterministic_iter_order_stable() {
    let g = multi_token_grammar();
    let reg = g.build_registry();
    let names1: Vec<&str> = reg.iter().map(|(n, _)| n).collect();
    let names2: Vec<&str> = reg.iter().map(|(n, _)| n).collect();
    assert_eq!(names1, names2);
}

// ═══════════════════════════════════════════════════════════════════
// 14. different_grammars — different grammars have different registries
// ═══════════════════════════════════════════════════════════════════

#[test]
fn sr_v10_different_grammars_differ() {
    let g1 = one_token_grammar();
    let g2 = one_rule_grammar();
    let reg1 = g1.build_registry();
    let reg2 = g2.build_registry();
    assert_ne!(reg1, reg2);
}

#[test]
fn sr_v10_different_names_differ() {
    let g1 = GrammarBuilder::new("sr_v10_diff_a")
        .token("aa", "a")
        .rule("ra", vec!["aa"])
        .start("ra")
        .build();
    let g2 = GrammarBuilder::new("sr_v10_diff_b")
        .token("bb", "b")
        .rule("rb", vec!["bb"])
        .start("rb")
        .build();
    let reg1 = g1.build_registry();
    let reg2 = g2.build_registry();
    assert!(reg1.get_id("aa").is_some());
    assert!(reg2.get_id("aa").is_none());
}

#[test]
fn sr_v10_different_token_count_differs() {
    let g1 = one_token_grammar();
    let g2 = multi_token_grammar();
    let reg1 = g1.build_registry();
    let reg2 = g2.build_registry();
    assert_ne!(reg1.len(), reg2.len());
}

#[test]
fn sr_v10_different_rule_count_differs() {
    let g1 = one_rule_grammar();
    let g2 = many_rule_grammar();
    assert_ne!(g1.rules.len(), g2.rules.len());
}

#[test]
fn sr_v10_different_registry_names_differ() {
    let g1 = one_token_grammar();
    let g2 = multi_token_grammar();
    let reg1 = g1.build_registry();
    let reg2 = g2.build_registry();
    let names1: HashSet<String> = reg1.iter().map(|(n, _)| n.to_string()).collect();
    let names2: HashSet<String> = reg2.iter().map(|(n, _)| n.to_string()).collect();
    assert_ne!(names1, names2);
}

// ═══════════════════════════════════════════════════════════════════
// 15. extras_entries — extra symbols have entries
// ═══════════════════════════════════════════════════════════════════

#[test]
fn sr_v10_extras_in_grammar() {
    let g = extra_grammar();
    assert!(!g.extras.is_empty());
}

#[test]
fn sr_v10_extras_ws_findable() {
    let g = extra_grammar();
    assert!(g.find_symbol_by_name("ws").is_some());
}

#[test]
fn sr_v10_extras_comment_findable() {
    let g = extra_grammar();
    assert!(g.find_symbol_by_name("comment").is_some());
}

#[test]
fn sr_v10_extras_in_registry() {
    let g = extra_grammar();
    let reg = g.build_registry();
    assert!(reg.get_id("ws").is_some());
    assert!(reg.get_id("comment").is_some());
}

#[test]
fn sr_v10_extras_marked_hidden() {
    let g = extra_grammar();
    let reg = g.build_registry();
    let ws_id = reg.get_id("ws").unwrap();
    let meta = reg.get_metadata(ws_id).unwrap();
    assert!(meta.hidden);
}

// ═══════════════════════════════════════════════════════════════════
// 16. external_entries — external symbols have entries
// ═══════════════════════════════════════════════════════════════════

#[test]
fn sr_v10_external_in_grammar() {
    let g = external_grammar();
    assert!(!g.externals.is_empty());
}

#[test]
fn sr_v10_external_indent_findable() {
    let g = external_grammar();
    assert!(g.find_symbol_by_name("indent").is_some());
}

#[test]
fn sr_v10_external_dedent_findable() {
    let g = external_grammar();
    assert!(g.find_symbol_by_name("dedent").is_some());
}

#[test]
fn sr_v10_external_in_registry() {
    let g = external_grammar();
    let reg = g.build_registry();
    assert!(reg.get_id("indent").is_some());
    assert!(reg.get_id("dedent").is_some());
}

#[test]
fn sr_v10_external_count() {
    let g = external_grammar();
    assert_eq!(g.externals.len(), 2);
}

// ═══════════════════════════════════════════════════════════════════
// 17. inline_entries — inline symbols have entries
// ═══════════════════════════════════════════════════════════════════

#[test]
fn sr_v10_inline_in_grammar() {
    let g = inline_grammar();
    assert!(!g.inline_rules.is_empty());
}

#[test]
fn sr_v10_inline_primary_findable() {
    let g = inline_grammar();
    assert!(g.find_symbol_by_name("primary").is_some());
}

#[test]
fn sr_v10_inline_expr_findable() {
    let g = inline_grammar();
    assert!(g.find_symbol_by_name("expr").is_some());
}

#[test]
fn sr_v10_inline_in_registry() {
    let g = inline_grammar();
    let reg = g.build_registry();
    assert!(reg.get_id("primary").is_some());
}

#[test]
fn sr_v10_inline_is_nonterminal() {
    let g = inline_grammar();
    let reg = g.build_registry();
    let id = reg.get_id("primary").unwrap();
    let meta = reg.get_metadata(id).unwrap();
    assert!(!meta.terminal);
}

// ═══════════════════════════════════════════════════════════════════
// 18. supertype_entries — supertype symbols have entries
// ═══════════════════════════════════════════════════════════════════

#[test]
fn sr_v10_supertype_in_grammar() {
    let g = supertype_grammar();
    assert!(!g.supertypes.is_empty());
}

#[test]
fn sr_v10_supertype_expression_findable() {
    let g = supertype_grammar();
    assert!(g.find_symbol_by_name("expression").is_some());
}

#[test]
fn sr_v10_supertype_literal_findable() {
    let g = supertype_grammar();
    assert!(g.find_symbol_by_name("literal").is_some());
}

#[test]
fn sr_v10_supertype_in_registry() {
    let g = supertype_grammar();
    let reg = g.build_registry();
    assert!(reg.get_id("expression").is_some());
}

#[test]
fn sr_v10_supertype_is_nonterminal() {
    let g = supertype_grammar();
    let reg = g.build_registry();
    let id = reg.get_id("expression").unwrap();
    let meta = reg.get_metadata(id).unwrap();
    assert!(!meta.terminal);
}

// ═══════════════════════════════════════════════════════════════════
// 19. many_tokens — registry after adding many tokens
// ═══════════════════════════════════════════════════════════════════

#[test]
fn sr_v10_many_tokens_all_in_registry() {
    let g = many_token_grammar();
    let reg = g.build_registry();
    for i in 0..20 {
        let name = format!("tok{i}");
        assert!(reg.get_id(&name).is_some(), "missing {name}");
    }
}

#[test]
fn sr_v10_many_tokens_unique_ids() {
    let g = many_token_grammar();
    let reg = g.build_registry();
    let ids: HashSet<SymbolId> = (0..20)
        .filter_map(|i| reg.get_id(&format!("tok{i}")))
        .collect();
    assert_eq!(ids.len(), 20);
}

#[test]
fn sr_v10_many_tokens_registry_len() {
    let g = many_token_grammar();
    let reg = g.build_registry();
    // 20 tokens + EOF + "root" non-terminal = at least 22
    assert!(reg.len() >= 22);
}

#[test]
fn sr_v10_many_tokens_name_roundtrip() {
    let g = many_token_grammar();
    let reg = g.build_registry();
    for i in 0..20 {
        let name = format!("tok{i}");
        let id = reg.get_id(&name).unwrap();
        assert_eq!(reg.get_name(id), Some(name.as_str()));
    }
}

#[test]
fn sr_v10_many_tokens_all_terminal() {
    let g = many_token_grammar();
    let reg = g.build_registry();
    for i in 0..20 {
        let name = format!("tok{i}");
        let id = reg.get_id(&name).unwrap();
        let meta = reg.get_metadata(id).unwrap();
        assert!(meta.terminal, "{name} should be terminal");
    }
}

// ═══════════════════════════════════════════════════════════════════
// 20. many_rules — registry after adding many rules
// ═══════════════════════════════════════════════════════════════════

#[test]
fn sr_v10_many_rules_all_findable() {
    let g = many_rule_grammar();
    for name in &[
        "leaf",
        "atom",
        "factor",
        "term",
        "sum",
        "comparison",
        "equality",
        "conjunction",
        "disjunction",
        "expression",
        "statement",
        "block",
        "program",
    ] {
        assert!(g.find_symbol_by_name(name).is_some(), "missing: {name}");
    }
}

#[test]
fn sr_v10_many_rules_in_registry() {
    let g = many_rule_grammar();
    let reg = g.build_registry();
    for name in &[
        "leaf",
        "atom",
        "factor",
        "term",
        "sum",
        "comparison",
        "equality",
        "conjunction",
        "disjunction",
        "expression",
        "statement",
        "block",
        "program",
    ] {
        assert!(reg.get_id(name).is_some(), "missing: {name}");
    }
}

#[test]
fn sr_v10_many_rules_unique_ids() {
    let g = many_rule_grammar();
    let reg = g.build_registry();
    let names = [
        "leaf",
        "atom",
        "factor",
        "term",
        "sum",
        "comparison",
        "equality",
        "conjunction",
        "disjunction",
        "expression",
        "statement",
        "block",
        "program",
    ];
    let ids: HashSet<SymbolId> = names.iter().filter_map(|n| reg.get_id(n)).collect();
    assert_eq!(ids.len(), names.len());
}

#[test]
fn sr_v10_many_rules_all_nonterminal() {
    let g = many_rule_grammar();
    let reg = g.build_registry();
    for name in &[
        "leaf",
        "atom",
        "factor",
        "term",
        "sum",
        "comparison",
        "equality",
        "conjunction",
        "disjunction",
        "expression",
        "statement",
        "block",
        "program",
    ] {
        let id = reg.get_id(name).unwrap();
        let meta = reg.get_metadata(id).unwrap();
        assert!(!meta.terminal, "{name} should be non-terminal");
    }
}

#[test]
fn sr_v10_many_rules_tokens_still_terminal() {
    let g = many_rule_grammar();
    let reg = g.build_registry();
    for name in &["num", "id"] {
        let id = reg.get_id(name).unwrap();
        let meta = reg.get_metadata(id).unwrap();
        assert!(meta.terminal, "{name} should be terminal");
    }
}

// ═══════════════════════════════════════════════════════════════════
// Bonus: additional edge-case and integration tests
// ═══════════════════════════════════════════════════════════════════

#[test]
fn sr_v10_get_or_build_registry_caches() {
    let mut g = one_rule_grammar();
    assert!(g.symbol_registry.is_none());
    let _reg = g.get_or_build_registry();
    assert!(g.symbol_registry.is_some());
}

#[test]
fn sr_v10_registry_contains_id() {
    let g = one_rule_grammar();
    let reg = g.build_registry();
    let id = reg.get_id("expr").unwrap();
    assert!(reg.contains_id(id));
}

#[test]
fn sr_v10_registry_not_contains_bogus_id() {
    let g = one_rule_grammar();
    let reg = g.build_registry();
    assert!(!reg.contains_id(SymbolId(9999)));
}

#[test]
fn sr_v10_registry_get_name_bogus_none() {
    let g = one_rule_grammar();
    let reg = g.build_registry();
    assert!(reg.get_name(SymbolId(9999)).is_none());
}

#[test]
fn sr_v10_registry_get_metadata_bogus_none() {
    let g = one_rule_grammar();
    let reg = g.build_registry();
    assert!(reg.get_metadata(SymbolId(9999)).is_none());
}

#[test]
fn sr_v10_registry_not_empty() {
    let g = one_rule_grammar();
    let reg = g.build_registry();
    assert!(!reg.is_empty());
}

#[test]
fn sr_v10_registry_to_index_map() {
    let g = one_rule_grammar();
    let reg = g.build_registry();
    let map = reg.to_index_map();
    assert!(!map.is_empty());
}

#[test]
fn sr_v10_registry_to_symbol_map() {
    let g = one_rule_grammar();
    let reg = g.build_registry();
    let map = reg.to_symbol_map();
    assert!(!map.is_empty());
}

#[test]
fn sr_v10_registry_index_symbol_roundtrip() {
    let g = multi_token_grammar();
    let reg = g.build_registry();
    let idx_map = reg.to_index_map();
    let sym_map = reg.to_symbol_map();
    for (&sym_id, &idx) in &idx_map {
        assert_eq!(sym_map[&idx], sym_id);
    }
}

#[test]
fn sr_v10_registry_iter_all_have_metadata() {
    let g = multi_token_grammar();
    let reg = g.build_registry();
    for (name, info) in reg.iter() {
        assert!(
            reg.get_metadata(info.id).is_some(),
            "no metadata for {name}"
        );
    }
}
