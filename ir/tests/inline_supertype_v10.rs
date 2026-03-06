//! Comprehensive tests for inline rules and supertypes in adze-ir Grammar.

use adze_ir::builder::GrammarBuilder;
use adze_ir::{Associativity, Grammar};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Build a small grammar with an `expr` rule and two tokens.
fn base_grammar(name: &str) -> GrammarBuilder {
    GrammarBuilder::new(name)
        .token("ID", r"[a-z]+")
        .token(";", ";")
        .rule("start", vec!["expr", ";"])
        .rule("expr", vec!["ID"])
        .start("start")
}

/// Build a grammar that has several non-terminals handy.
fn rich_grammar(name: &str) -> GrammarBuilder {
    GrammarBuilder::new(name)
        .token("ID", r"[a-z]+")
        .token("NUM", r"[0-9]+")
        .token("+", "+")
        .token(";", ";")
        .rule("start", vec!["stmt"])
        .rule("stmt", vec!["expr", ";"])
        .rule("expr", vec!["term"])
        .rule("expr", vec!["expr", "+", "term"])
        .rule("term", vec!["ID"])
        .rule("term", vec!["NUM"])
        .start("start")
}

// ===========================================================================
// 1. No inline rules → inline_rules empty
// ===========================================================================

#[test]
fn is_v10_no_inline_rules_vec_is_empty() {
    let g = base_grammar("is_v10_no_inline_empty").build();
    assert!(g.inline_rules.is_empty());
}

#[test]
fn is_v10_no_inline_rules_len_zero() {
    let g = base_grammar("is_v10_no_inline_len").build();
    assert_eq!(g.inline_rules.len(), 0);
}

#[test]
fn is_v10_default_grammar_no_inline() {
    let g = Grammar::new("is_v10_default_no_inline".to_string());
    assert!(g.inline_rules.is_empty());
}

#[test]
fn is_v10_rich_grammar_no_inline_by_default() {
    let g = rich_grammar("is_v10_rich_no_inline").build();
    assert!(g.inline_rules.is_empty());
}

// ===========================================================================
// 2. One inline rule → inline_rules has 1
// ===========================================================================

#[test]
fn is_v10_one_inline_rule_len() {
    let g = base_grammar("is_v10_one_inline_len").inline("expr").build();
    assert_eq!(g.inline_rules.len(), 1);
}

#[test]
fn is_v10_one_inline_rule_contains_symbol() {
    let g = base_grammar("is_v10_one_inline_contains")
        .inline("expr")
        .build();
    let expr_id = g.find_symbol_by_name("expr").unwrap();
    assert!(g.inline_rules.contains(&expr_id));
}

#[test]
fn is_v10_one_inline_rule_exact_id() {
    let g = base_grammar("is_v10_one_inline_exact")
        .inline("expr")
        .build();
    let expr_id = g.find_symbol_by_name("expr").unwrap();
    assert_eq!(g.inline_rules[0], expr_id);
}

#[test]
fn is_v10_one_inline_non_inline_not_present() {
    let g = base_grammar("is_v10_one_inline_absent")
        .inline("expr")
        .build();
    let start_id = g.find_symbol_by_name("start").unwrap();
    assert!(!g.inline_rules.contains(&start_id));
}

// ===========================================================================
// 3. Multiple inline rules → all present
// ===========================================================================

#[test]
fn is_v10_multiple_inline_two() {
    let g = rich_grammar("is_v10_multi_inline_two")
        .inline("expr")
        .inline("term")
        .build();
    assert_eq!(g.inline_rules.len(), 2);
}

#[test]
fn is_v10_multiple_inline_three() {
    let g = rich_grammar("is_v10_multi_inline_three")
        .rule("helper", vec!["ID"])
        .inline("expr")
        .inline("term")
        .inline("helper")
        .build();
    assert_eq!(g.inline_rules.len(), 3);
}

#[test]
fn is_v10_multiple_inline_all_resolvable() {
    let g = rich_grammar("is_v10_multi_inline_resolve")
        .inline("expr")
        .inline("term")
        .build();
    let expr_id = g.find_symbol_by_name("expr").unwrap();
    let term_id = g.find_symbol_by_name("term").unwrap();
    assert!(g.inline_rules.contains(&expr_id));
    assert!(g.inline_rules.contains(&term_id));
}

#[test]
fn is_v10_multiple_inline_preserves_order() {
    let g = rich_grammar("is_v10_multi_inline_order")
        .inline("term")
        .inline("expr")
        .build();
    let expr_id = g.find_symbol_by_name("expr").unwrap();
    let term_id = g.find_symbol_by_name("term").unwrap();
    assert_eq!(g.inline_rules[0], term_id);
    assert_eq!(g.inline_rules[1], expr_id);
}

// ===========================================================================
// 4. No supertypes → supertypes empty
// ===========================================================================

#[test]
fn is_v10_no_supertypes_vec_is_empty() {
    let g = base_grammar("is_v10_no_super_empty").build();
    assert!(g.supertypes.is_empty());
}

#[test]
fn is_v10_no_supertypes_len_zero() {
    let g = base_grammar("is_v10_no_super_len").build();
    assert_eq!(g.supertypes.len(), 0);
}

#[test]
fn is_v10_default_grammar_no_supertypes() {
    let g = Grammar::new("is_v10_default_no_super".to_string());
    assert!(g.supertypes.is_empty());
}

#[test]
fn is_v10_rich_grammar_no_supertypes_by_default() {
    let g = rich_grammar("is_v10_rich_no_super").build();
    assert!(g.supertypes.is_empty());
}

// ===========================================================================
// 5. One supertype → supertypes has 1
// ===========================================================================

#[test]
fn is_v10_one_supertype_len() {
    let g = base_grammar("is_v10_one_super_len")
        .supertype("expr")
        .build();
    assert_eq!(g.supertypes.len(), 1);
}

#[test]
fn is_v10_one_supertype_contains_symbol() {
    let g = base_grammar("is_v10_one_super_contains")
        .supertype("expr")
        .build();
    let expr_id = g.find_symbol_by_name("expr").unwrap();
    assert!(g.supertypes.contains(&expr_id));
}

#[test]
fn is_v10_one_supertype_exact_id() {
    let g = base_grammar("is_v10_one_super_exact")
        .supertype("expr")
        .build();
    let expr_id = g.find_symbol_by_name("expr").unwrap();
    assert_eq!(g.supertypes[0], expr_id);
}

#[test]
fn is_v10_one_supertype_non_super_absent() {
    let g = base_grammar("is_v10_one_super_absent")
        .supertype("expr")
        .build();
    let start_id = g.find_symbol_by_name("start").unwrap();
    assert!(!g.supertypes.contains(&start_id));
}

// ===========================================================================
// 6. Multiple supertypes → all present
// ===========================================================================

#[test]
fn is_v10_multiple_supertypes_two() {
    let g = rich_grammar("is_v10_multi_super_two")
        .supertype("expr")
        .supertype("stmt")
        .build();
    assert_eq!(g.supertypes.len(), 2);
}

#[test]
fn is_v10_multiple_supertypes_three() {
    let g = rich_grammar("is_v10_multi_super_three")
        .rule("decl", vec!["ID"])
        .supertype("expr")
        .supertype("stmt")
        .supertype("decl")
        .build();
    assert_eq!(g.supertypes.len(), 3);
}

#[test]
fn is_v10_multiple_supertypes_all_resolvable() {
    let g = rich_grammar("is_v10_multi_super_resolve")
        .supertype("expr")
        .supertype("stmt")
        .build();
    let expr_id = g.find_symbol_by_name("expr").unwrap();
    let stmt_id = g.find_symbol_by_name("stmt").unwrap();
    assert!(g.supertypes.contains(&expr_id));
    assert!(g.supertypes.contains(&stmt_id));
}

#[test]
fn is_v10_multiple_supertypes_preserves_order() {
    let g = rich_grammar("is_v10_multi_super_order")
        .supertype("stmt")
        .supertype("expr")
        .build();
    let expr_id = g.find_symbol_by_name("expr").unwrap();
    let stmt_id = g.find_symbol_by_name("stmt").unwrap();
    assert_eq!(g.supertypes[0], stmt_id);
    assert_eq!(g.supertypes[1], expr_id);
}

// ===========================================================================
// 7. Inline + supertype on same grammar
// ===========================================================================

#[test]
fn is_v10_inline_and_supertype_coexist() {
    let g = rich_grammar("is_v10_both_coexist")
        .inline("term")
        .supertype("expr")
        .build();
    assert_eq!(g.inline_rules.len(), 1);
    assert_eq!(g.supertypes.len(), 1);
}

#[test]
fn is_v10_inline_and_supertype_different_symbols() {
    let g = rich_grammar("is_v10_both_diff_syms")
        .inline("term")
        .supertype("expr")
        .build();
    let term_id = g.find_symbol_by_name("term").unwrap();
    let expr_id = g.find_symbol_by_name("expr").unwrap();
    assert!(g.inline_rules.contains(&term_id));
    assert!(g.supertypes.contains(&expr_id));
}

#[test]
fn is_v10_same_symbol_inline_and_supertype() {
    let g = base_grammar("is_v10_both_same_sym")
        .inline("expr")
        .supertype("expr")
        .build();
    let expr_id = g.find_symbol_by_name("expr").unwrap();
    assert!(g.inline_rules.contains(&expr_id));
    assert!(g.supertypes.contains(&expr_id));
}

#[test]
fn is_v10_inline_and_supertype_independent_counts() {
    let g = rich_grammar("is_v10_both_indep_counts")
        .inline("term")
        .inline("expr")
        .supertype("stmt")
        .build();
    assert_eq!(g.inline_rules.len(), 2);
    assert_eq!(g.supertypes.len(), 1);
}

#[test]
fn is_v10_inline_does_not_appear_in_supertypes() {
    let g = rich_grammar("is_v10_inline_not_super")
        .inline("term")
        .supertype("expr")
        .build();
    let term_id = g.find_symbol_by_name("term").unwrap();
    assert!(!g.supertypes.contains(&term_id));
}

#[test]
fn is_v10_supertype_does_not_appear_in_inline() {
    let g = rich_grammar("is_v10_super_not_inline")
        .inline("term")
        .supertype("expr")
        .build();
    let expr_id = g.find_symbol_by_name("expr").unwrap();
    assert!(!g.inline_rules.contains(&expr_id));
}

// ===========================================================================
// 8. normalize expands inline rules
// ===========================================================================

#[test]
fn is_v10_normalize_returns_rules() {
    let mut g = base_grammar("is_v10_norm_returns").build();
    let rules = g.normalize();
    assert!(!rules.is_empty());
}

#[test]
fn is_v10_normalize_keeps_all_basic_rules() {
    let mut g = base_grammar("is_v10_norm_keeps_basic").build();
    let rules_before = g.all_rules().count();
    let _rules = g.normalize();
    let rules_after = g.all_rules().count();
    assert_eq!(rules_before, rules_after);
}

#[test]
fn is_v10_normalize_with_inline_still_runs() {
    let mut g = base_grammar("is_v10_norm_inline_runs")
        .inline("expr")
        .build();
    let rules = g.normalize();
    assert!(!rules.is_empty());
}

#[test]
fn is_v10_normalize_idempotent_on_simple_grammar() {
    let mut g = base_grammar("is_v10_norm_idempotent").build();
    let _r1 = g.normalize();
    let count_after_first = g.all_rules().count();
    let _r2 = g.normalize();
    let count_after_second = g.all_rules().count();
    assert_eq!(count_after_first, count_after_second);
}

// ===========================================================================
// 9. After normalize, inline symbols may be removed
// ===========================================================================

#[test]
fn is_v10_normalize_does_not_crash_with_inline() {
    let mut g = rich_grammar("is_v10_norm_no_crash_inline")
        .inline("term")
        .build();
    let _rules = g.normalize();
    // Just verifying no panic.
}

#[test]
fn is_v10_normalize_inline_rules_vec_persists() {
    let mut g = base_grammar("is_v10_norm_inline_vec_persists")
        .inline("expr")
        .build();
    let _rules = g.normalize();
    // inline_rules metadata itself is not cleared by normalize
    assert!(!g.inline_rules.is_empty());
}

#[test]
fn is_v10_normalize_inline_id_still_in_vec() {
    let mut g = base_grammar("is_v10_norm_inline_id_persists")
        .inline("expr")
        .build();
    let expr_id = g.find_symbol_by_name("expr").unwrap();
    let _rules = g.normalize();
    assert!(g.inline_rules.contains(&expr_id));
}

#[test]
fn is_v10_normalize_grammar_name_unchanged() {
    let mut g = base_grammar("is_v10_norm_name_unchanged").build();
    let _rules = g.normalize();
    assert_eq!(g.name, "is_v10_norm_name_unchanged");
}

// ===========================================================================
// 10. Normalize changes rule count (when complex symbols exist)
// ===========================================================================

#[test]
fn is_v10_normalize_no_change_without_complex() {
    let mut g = base_grammar("is_v10_norm_no_change").build();
    let before = g.all_rules().count();
    let _rules = g.normalize();
    assert_eq!(g.all_rules().count(), before);
}

#[test]
fn is_v10_normalize_rich_grammar_stable() {
    let mut g = rich_grammar("is_v10_norm_rich_stable").build();
    let before = g.all_rules().count();
    let _rules = g.normalize();
    assert_eq!(g.all_rules().count(), before);
}

#[test]
fn is_v10_normalize_with_inline_rule_count_unchanged() {
    let mut g = rich_grammar("is_v10_norm_inline_count")
        .inline("term")
        .build();
    let before = g.all_rules().count();
    let _rules = g.normalize();
    // normalize does not inline-expand; it normalizes complex symbols
    assert_eq!(g.all_rules().count(), before);
}

// ===========================================================================
// 11. Normalize preserves start symbol
// ===========================================================================

#[test]
fn is_v10_normalize_preserves_start() {
    let mut g = rich_grammar("is_v10_norm_start_preserved").build();
    let start_before = g.start_symbol();
    let _rules = g.normalize();
    assert_eq!(g.start_symbol(), start_before);
}

#[test]
fn is_v10_normalize_start_rules_still_present() {
    let mut g = base_grammar("is_v10_norm_start_rules").build();
    let start = g.start_symbol().unwrap();
    let _rules = g.normalize();
    assert!(g.rules.contains_key(&start));
}

#[test]
fn is_v10_normalize_start_first_in_rules() {
    let mut g = rich_grammar("is_v10_norm_start_first").build();
    let start = g.start_symbol().unwrap();
    let _rules = g.normalize();
    let first_key = g.rules.keys().next().copied();
    assert_eq!(first_key, Some(start));
}

// ===========================================================================
// 12. Normalize preserves tokens
// ===========================================================================

#[test]
fn is_v10_normalize_tokens_count_unchanged() {
    let mut g = base_grammar("is_v10_norm_tokens_count").build();
    let tok_count = g.tokens.len();
    let _rules = g.normalize();
    assert_eq!(g.tokens.len(), tok_count);
}

#[test]
fn is_v10_normalize_token_names_preserved() {
    let mut g = base_grammar("is_v10_norm_token_names").build();
    let names_before: Vec<String> = g.tokens.values().map(|t| t.name.clone()).collect();
    let _rules = g.normalize();
    let names_after: Vec<String> = g.tokens.values().map(|t| t.name.clone()).collect();
    assert_eq!(names_before, names_after);
}

#[test]
fn is_v10_normalize_token_patterns_preserved() {
    let mut g = rich_grammar("is_v10_norm_tok_pat").build();
    let pats_before: Vec<_> = g.tokens.values().map(|t| t.pattern.clone()).collect();
    let _rules = g.normalize();
    let pats_after: Vec<_> = g.tokens.values().map(|t| t.pattern.clone()).collect();
    assert_eq!(pats_before, pats_after);
}

// ===========================================================================
// 13. Supertype survives normalize
// ===========================================================================

#[test]
fn is_v10_supertype_survives_normalize() {
    let mut g = rich_grammar("is_v10_super_norm").supertype("expr").build();
    let _rules = g.normalize();
    assert_eq!(g.supertypes.len(), 1);
}

#[test]
fn is_v10_supertype_id_same_after_normalize() {
    let mut g = rich_grammar("is_v10_super_norm_id")
        .supertype("expr")
        .build();
    let expr_id = g.find_symbol_by_name("expr").unwrap();
    let _rules = g.normalize();
    assert_eq!(g.supertypes[0], expr_id);
}

#[test]
fn is_v10_multiple_supertypes_survive_normalize() {
    let mut g = rich_grammar("is_v10_multi_super_norm")
        .supertype("expr")
        .supertype("stmt")
        .build();
    let _rules = g.normalize();
    assert_eq!(g.supertypes.len(), 2);
}

#[test]
fn is_v10_supertype_and_inline_survive_normalize() {
    let mut g = rich_grammar("is_v10_both_survive_norm")
        .inline("term")
        .supertype("expr")
        .build();
    let _rules = g.normalize();
    assert_eq!(g.inline_rules.len(), 1);
    assert_eq!(g.supertypes.len(), 1);
}

// ===========================================================================
// 14. Supertype survives optimize
// ===========================================================================

#[test]
fn is_v10_supertype_survives_optimize() {
    let mut g = rich_grammar("is_v10_super_opt").supertype("expr").build();
    g.optimize();
    assert_eq!(g.supertypes.len(), 1);
}

#[test]
fn is_v10_supertype_id_same_after_optimize() {
    let mut g = rich_grammar("is_v10_super_opt_id")
        .supertype("expr")
        .build();
    let expr_id = g.find_symbol_by_name("expr").unwrap();
    g.optimize();
    assert_eq!(g.supertypes[0], expr_id);
}

#[test]
fn is_v10_inline_rules_survive_optimize() {
    let mut g = rich_grammar("is_v10_inline_opt").inline("term").build();
    g.optimize();
    assert_eq!(g.inline_rules.len(), 1);
}

#[test]
fn is_v10_both_survive_optimize() {
    let mut g = rich_grammar("is_v10_both_opt")
        .inline("term")
        .supertype("expr")
        .build();
    g.optimize();
    assert_eq!(g.inline_rules.len(), 1);
    assert_eq!(g.supertypes.len(), 1);
}

// ===========================================================================
// 15. Clone preserves inline_rules
// ===========================================================================

#[test]
fn is_v10_clone_preserves_inline_len() {
    let g = rich_grammar("is_v10_clone_inline_len")
        .inline("term")
        .build();
    let g2 = g.clone();
    assert_eq!(g.inline_rules.len(), g2.inline_rules.len());
}

#[test]
fn is_v10_clone_preserves_inline_ids() {
    let g = rich_grammar("is_v10_clone_inline_ids")
        .inline("term")
        .inline("expr")
        .build();
    let g2 = g.clone();
    assert_eq!(g.inline_rules, g2.inline_rules);
}

#[test]
fn is_v10_clone_inline_independent() {
    let g = rich_grammar("is_v10_clone_inline_indep")
        .inline("term")
        .build();
    let mut g2 = g.clone();
    g2.inline_rules.clear();
    assert_eq!(g.inline_rules.len(), 1);
    assert!(g2.inline_rules.is_empty());
}

#[test]
fn is_v10_clone_empty_inline_preserved() {
    let g = base_grammar("is_v10_clone_empty_inline").build();
    let g2 = g.clone();
    assert!(g2.inline_rules.is_empty());
}

// ===========================================================================
// 16. Clone preserves supertypes
// ===========================================================================

#[test]
fn is_v10_clone_preserves_supertype_len() {
    let g = rich_grammar("is_v10_clone_super_len")
        .supertype("expr")
        .build();
    let g2 = g.clone();
    assert_eq!(g.supertypes.len(), g2.supertypes.len());
}

#[test]
fn is_v10_clone_preserves_supertype_ids() {
    let g = rich_grammar("is_v10_clone_super_ids")
        .supertype("expr")
        .supertype("stmt")
        .build();
    let g2 = g.clone();
    assert_eq!(g.supertypes, g2.supertypes);
}

#[test]
fn is_v10_clone_supertype_independent() {
    let g = rich_grammar("is_v10_clone_super_indep")
        .supertype("expr")
        .build();
    let mut g2 = g.clone();
    g2.supertypes.clear();
    assert_eq!(g.supertypes.len(), 1);
    assert!(g2.supertypes.is_empty());
}

#[test]
fn is_v10_clone_empty_supertypes_preserved() {
    let g = base_grammar("is_v10_clone_empty_super").build();
    let g2 = g.clone();
    assert!(g2.supertypes.is_empty());
}

// ===========================================================================
// 17. Debug includes inline info
// ===========================================================================

#[test]
fn is_v10_debug_mentions_inline_rules_field() {
    let g = base_grammar("is_v10_debug_inline_field").build();
    let dbg = format!("{:?}", g);
    assert!(dbg.contains("inline_rules"));
}

#[test]
fn is_v10_debug_inline_empty_shown() {
    let g = base_grammar("is_v10_debug_inline_empty").build();
    let dbg = format!("{:?}", g);
    assert!(dbg.contains("inline_rules: []"));
}

#[test]
fn is_v10_debug_inline_with_entries() {
    let g = base_grammar("is_v10_debug_inline_entries")
        .inline("expr")
        .build();
    let dbg = format!("{:?}", g);
    assert!(dbg.contains("inline_rules: [SymbolId("));
}

#[test]
fn is_v10_debug_inline_symbol_id_value() {
    let g = base_grammar("is_v10_debug_inline_sid")
        .inline("expr")
        .build();
    let expr_id = g.find_symbol_by_name("expr").unwrap();
    let dbg = format!("{:?}", g);
    let expected = format!("SymbolId({})", expr_id.0);
    assert!(dbg.contains(&expected));
}

// ===========================================================================
// 18. Debug includes supertype info
// ===========================================================================

#[test]
fn is_v10_debug_mentions_supertypes_field() {
    let g = base_grammar("is_v10_debug_super_field").build();
    let dbg = format!("{:?}", g);
    assert!(dbg.contains("supertypes"));
}

#[test]
fn is_v10_debug_supertypes_empty_shown() {
    let g = base_grammar("is_v10_debug_super_empty").build();
    let dbg = format!("{:?}", g);
    assert!(dbg.contains("supertypes: []"));
}

#[test]
fn is_v10_debug_supertypes_with_entries() {
    let g = base_grammar("is_v10_debug_super_entries")
        .supertype("expr")
        .build();
    let dbg = format!("{:?}", g);
    assert!(dbg.contains("supertypes: [SymbolId("));
}

#[test]
fn is_v10_debug_supertype_symbol_id_value() {
    let g = base_grammar("is_v10_debug_super_sid")
        .supertype("expr")
        .build();
    let expr_id = g.find_symbol_by_name("expr").unwrap();
    let dbg = format!("{:?}", g);
    let expected = format!("SymbolId({})", expr_id.0);
    assert!(dbg.contains(&expected));
}

// ===========================================================================
// 19. Validate with inline rules → Ok
// ===========================================================================

#[test]
fn is_v10_validate_ok_with_inline() {
    let g = base_grammar("is_v10_validate_inline_ok")
        .inline("expr")
        .build();
    assert!(g.validate().is_ok());
}

#[test]
fn is_v10_validate_ok_with_supertype() {
    let g = base_grammar("is_v10_validate_super_ok")
        .supertype("expr")
        .build();
    assert!(g.validate().is_ok());
}

#[test]
fn is_v10_validate_ok_with_both() {
    let g = rich_grammar("is_v10_validate_both_ok")
        .inline("term")
        .supertype("expr")
        .build();
    assert!(g.validate().is_ok());
}

#[test]
fn is_v10_validate_ok_after_normalize() {
    let mut g = base_grammar("is_v10_validate_after_norm")
        .inline("expr")
        .build();
    let _rules = g.normalize();
    assert!(g.validate().is_ok());
}

// ===========================================================================
// 20. Grammar with inline chain rules
// ===========================================================================

#[test]
fn is_v10_inline_chain_a_to_b_to_c() {
    let g = GrammarBuilder::new("is_v10_chain_abc")
        .token("ID", r"[a-z]+")
        .rule("start", vec!["a"])
        .rule("a", vec!["b"])
        .rule("b", vec!["c"])
        .rule("c", vec!["ID"])
        .inline("a")
        .inline("b")
        .start("start")
        .build();
    assert_eq!(g.inline_rules.len(), 2);
}

#[test]
fn is_v10_inline_chain_ids_correct() {
    let g = GrammarBuilder::new("is_v10_chain_ids")
        .token("ID", r"[a-z]+")
        .rule("start", vec!["a"])
        .rule("a", vec!["b"])
        .rule("b", vec!["c"])
        .rule("c", vec!["ID"])
        .inline("a")
        .inline("b")
        .start("start")
        .build();
    let a_id = g.find_symbol_by_name("a").unwrap();
    let b_id = g.find_symbol_by_name("b").unwrap();
    assert!(g.inline_rules.contains(&a_id));
    assert!(g.inline_rules.contains(&b_id));
}

#[test]
fn is_v10_inline_chain_non_inlined_untouched() {
    let g = GrammarBuilder::new("is_v10_chain_not_inlined")
        .token("ID", r"[a-z]+")
        .rule("start", vec!["a"])
        .rule("a", vec!["b"])
        .rule("b", vec!["c"])
        .rule("c", vec!["ID"])
        .inline("a")
        .start("start")
        .build();
    let c_id = g.find_symbol_by_name("c").unwrap();
    assert!(!g.inline_rules.contains(&c_id));
}

#[test]
fn is_v10_inline_chain_validate_ok() {
    let g = GrammarBuilder::new("is_v10_chain_valid")
        .token("ID", r"[a-z]+")
        .rule("start", vec!["a"])
        .rule("a", vec!["b"])
        .rule("b", vec!["c"])
        .rule("c", vec!["ID"])
        .inline("a")
        .inline("b")
        .start("start")
        .build();
    assert!(g.validate().is_ok());
}

// ===========================================================================
// Additional coverage: edge cases, interactions, builder ergonomics
// ===========================================================================

#[test]
fn is_v10_inline_does_not_modify_token_map() {
    let g = base_grammar("is_v10_inline_no_tok_mod")
        .inline("expr")
        .build();
    // "expr" should NOT appear in tokens
    for tok in g.tokens.values() {
        assert_ne!(tok.name, "expr");
    }
}

#[test]
fn is_v10_supertype_does_not_modify_token_map() {
    let g = base_grammar("is_v10_super_no_tok_mod")
        .supertype("expr")
        .build();
    for tok in g.tokens.values() {
        assert_ne!(tok.name, "expr");
    }
}

#[test]
fn is_v10_inline_rule_still_has_productions() {
    let g = rich_grammar("is_v10_inline_has_prods")
        .inline("term")
        .build();
    let term_id = g.find_symbol_by_name("term").unwrap();
    let prods = g.get_rules_for_symbol(term_id).unwrap();
    assert!(!prods.is_empty());
}

#[test]
fn is_v10_supertype_rule_still_has_productions() {
    let g = rich_grammar("is_v10_super_has_prods")
        .supertype("expr")
        .build();
    let expr_id = g.find_symbol_by_name("expr").unwrap();
    let prods = g.get_rules_for_symbol(expr_id).unwrap();
    assert!(!prods.is_empty());
}

#[test]
fn is_v10_inline_with_precedence_rule() {
    let g = GrammarBuilder::new("is_v10_inline_prec")
        .token("ID", r"[a-z]+")
        .token("+", "+")
        .rule("start", vec!["expr"])
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .rule("expr", vec!["ID"])
        .inline("expr")
        .start("start")
        .build();
    let expr_id = g.find_symbol_by_name("expr").unwrap();
    assert!(g.inline_rules.contains(&expr_id));
}

#[test]
fn is_v10_supertype_with_precedence_rule() {
    let g = GrammarBuilder::new("is_v10_super_prec")
        .token("ID", r"[a-z]+")
        .token("+", "+")
        .rule("start", vec!["expr"])
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .rule("expr", vec!["ID"])
        .supertype("expr")
        .start("start")
        .build();
    let expr_id = g.find_symbol_by_name("expr").unwrap();
    assert!(g.supertypes.contains(&expr_id));
}

#[test]
fn is_v10_inline_with_extra_token() {
    let g = GrammarBuilder::new("is_v10_inline_extra")
        .token("ID", r"[a-z]+")
        .token("WS", r"\s+")
        .rule("start", vec!["expr"])
        .rule("expr", vec!["ID"])
        .extra("WS")
        .inline("expr")
        .start("start")
        .build();
    assert_eq!(g.inline_rules.len(), 1);
    assert_eq!(g.extras.len(), 1);
}

#[test]
fn is_v10_grammar_name_unaffected_by_inline() {
    let g = base_grammar("is_v10_name_inline").inline("expr").build();
    assert_eq!(g.name, "is_v10_name_inline");
}

#[test]
fn is_v10_grammar_name_unaffected_by_supertype() {
    let g = base_grammar("is_v10_name_super").supertype("expr").build();
    assert_eq!(g.name, "is_v10_name_super");
}

#[test]
fn is_v10_builder_chaining_inline_then_supertype() {
    let g = rich_grammar("is_v10_chain_in_sup")
        .inline("term")
        .supertype("expr")
        .build();
    assert_eq!(g.inline_rules.len(), 1);
    assert_eq!(g.supertypes.len(), 1);
}

#[test]
fn is_v10_builder_chaining_supertype_then_inline() {
    let g = rich_grammar("is_v10_chain_sup_in")
        .supertype("expr")
        .inline("term")
        .build();
    assert_eq!(g.supertypes.len(), 1);
    assert_eq!(g.inline_rules.len(), 1);
}

#[test]
fn is_v10_inline_rules_not_in_externals() {
    let g = base_grammar("is_v10_inline_no_ext").inline("expr").build();
    let expr_id = g.find_symbol_by_name("expr").unwrap();
    assert!(!g.externals.iter().any(|e| e.symbol_id == expr_id));
}

#[test]
fn is_v10_supertype_not_in_externals() {
    let g = base_grammar("is_v10_super_no_ext")
        .supertype("expr")
        .build();
    let expr_id = g.find_symbol_by_name("expr").unwrap();
    assert!(!g.externals.iter().any(|e| e.symbol_id == expr_id));
}

#[test]
fn is_v10_all_rules_iter_includes_inline_symbol() {
    let g = rich_grammar("is_v10_all_rules_inline")
        .inline("term")
        .build();
    let term_id = g.find_symbol_by_name("term").unwrap();
    assert!(g.all_rules().any(|r| r.lhs == term_id));
}

#[test]
fn is_v10_all_rules_iter_includes_supertype_symbol() {
    let g = rich_grammar("is_v10_all_rules_super")
        .supertype("expr")
        .build();
    let expr_id = g.find_symbol_by_name("expr").unwrap();
    assert!(g.all_rules().any(|r| r.lhs == expr_id));
}

#[test]
fn is_v10_serde_roundtrip_inline_rules() {
    let g = rich_grammar("is_v10_serde_inline").inline("term").build();
    let json = serde_json::to_string(&g).unwrap();
    let g2: Grammar = serde_json::from_str(&json).unwrap();
    assert_eq!(g.inline_rules, g2.inline_rules);
}

#[test]
fn is_v10_serde_roundtrip_supertypes() {
    let g = rich_grammar("is_v10_serde_super").supertype("expr").build();
    let json = serde_json::to_string(&g).unwrap();
    let g2: Grammar = serde_json::from_str(&json).unwrap();
    assert_eq!(g.supertypes, g2.supertypes);
}

#[test]
fn is_v10_serde_roundtrip_both() {
    let g = rich_grammar("is_v10_serde_both")
        .inline("term")
        .supertype("expr")
        .build();
    let json = serde_json::to_string(&g).unwrap();
    let g2: Grammar = serde_json::from_str(&json).unwrap();
    assert_eq!(g.inline_rules, g2.inline_rules);
    assert_eq!(g.supertypes, g2.supertypes);
}

#[test]
fn is_v10_eq_grammars_same_inline() {
    let g1 = base_grammar("is_v10_eq_inline1").inline("expr").build();
    let g2 = base_grammar("is_v10_eq_inline1").inline("expr").build();
    assert_eq!(g1.inline_rules, g2.inline_rules);
}

#[test]
fn is_v10_eq_grammars_same_supertypes() {
    let g1 = base_grammar("is_v10_eq_super1").supertype("expr").build();
    let g2 = base_grammar("is_v10_eq_super1").supertype("expr").build();
    assert_eq!(g1.supertypes, g2.supertypes);
}

#[test]
fn is_v10_symbol_id_is_copy() {
    let g = base_grammar("is_v10_sid_copy").inline("expr").build();
    let id = g.inline_rules[0];
    let id2 = id; // Copy, not move
    assert_eq!(id, id2);
}

#[test]
fn is_v10_inline_after_rule_definition() {
    let g = GrammarBuilder::new("is_v10_inline_after_def")
        .token("ID", r"[a-z]+")
        .rule("start", vec!["helper"])
        .rule("helper", vec!["ID"])
        .start("start")
        .inline("helper")
        .build();
    let helper_id = g.find_symbol_by_name("helper").unwrap();
    assert!(g.inline_rules.contains(&helper_id));
}

#[test]
fn is_v10_supertype_after_rule_definition() {
    let g = GrammarBuilder::new("is_v10_super_after_def")
        .token("ID", r"[a-z]+")
        .rule("start", vec!["wrapper"])
        .rule("wrapper", vec!["ID"])
        .start("start")
        .supertype("wrapper")
        .build();
    let wrapper_id = g.find_symbol_by_name("wrapper").unwrap();
    assert!(g.supertypes.contains(&wrapper_id));
}

#[test]
fn is_v10_inline_before_rule_definition() {
    let g = GrammarBuilder::new("is_v10_inline_before_def")
        .token("ID", r"[a-z]+")
        .inline("helper")
        .rule("start", vec!["helper"])
        .rule("helper", vec!["ID"])
        .start("start")
        .build();
    let helper_id = g.find_symbol_by_name("helper").unwrap();
    assert!(g.inline_rules.contains(&helper_id));
}

#[test]
fn is_v10_supertype_before_rule_definition() {
    let g = GrammarBuilder::new("is_v10_super_before_def")
        .token("ID", r"[a-z]+")
        .supertype("wrapper")
        .rule("start", vec!["wrapper"])
        .rule("wrapper", vec!["ID"])
        .start("start")
        .build();
    let wrapper_id = g.find_symbol_by_name("wrapper").unwrap();
    assert!(g.supertypes.contains(&wrapper_id));
}

#[test]
fn is_v10_multiple_alternatives_inline() {
    let g = GrammarBuilder::new("is_v10_multi_alt_inline")
        .token("ID", r"[a-z]+")
        .token("NUM", r"[0-9]+")
        .rule("start", vec!["primary"])
        .rule("primary", vec!["ID"])
        .rule("primary", vec!["NUM"])
        .inline("primary")
        .start("start")
        .build();
    let primary_id = g.find_symbol_by_name("primary").unwrap();
    let prods = g.get_rules_for_symbol(primary_id).unwrap();
    assert_eq!(prods.len(), 2);
    assert!(g.inline_rules.contains(&primary_id));
}

#[test]
fn is_v10_normalize_then_validate_with_inline() {
    let mut g = rich_grammar("is_v10_norm_then_validate")
        .inline("term")
        .supertype("expr")
        .build();
    let _rules = g.normalize();
    assert!(g.validate().is_ok());
}

#[test]
fn is_v10_optimize_then_validate_with_inline() {
    let mut g = rich_grammar("is_v10_opt_then_validate")
        .inline("term")
        .supertype("expr")
        .build();
    g.optimize();
    assert!(g.validate().is_ok());
}

#[test]
fn is_v10_normalize_then_optimize_preserves_both() {
    let mut g = rich_grammar("is_v10_norm_opt_both")
        .inline("term")
        .supertype("expr")
        .build();
    let _rules = g.normalize();
    g.optimize();
    assert!(!g.inline_rules.is_empty());
    assert!(!g.supertypes.is_empty());
}
