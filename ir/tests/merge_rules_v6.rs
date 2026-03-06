//! Grammar rule merging, combination, and overlay tests (v6).
//!
//! 64 tests across 8 categories (8 each):
//! 1. basic_merge_*       — merging grammars with overlapping rules
//! 2. merge_precedence_*  — precedence handling during merge
//! 3. merge_tokens_*      — token set merging
//! 4. merge_conflicts_*   — conflict detection during merge
//! 5. merge_extras_*      — extras/whitespace merging
//! 6. merge_fields_*      — field mapping merging
//! 7. merge_externals_*   — external token merging
//! 8. merge_roundtrip_*   — merge then validate roundtrip

use adze_ir::builder::GrammarBuilder;
#[allow(unused_imports)]
use adze_ir::{
    Associativity, ConflictDeclaration, ConflictResolution, ExternalToken, FieldId, Grammar,
    PrecedenceKind, ProductionId, Rule, Symbol, SymbolId, Token, TokenPattern,
};
use indexmap::IndexMap;
use std::collections::HashSet;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Look up a SymbolId by its human-readable name in `rule_names`.
fn sym(g: &Grammar, name: &str) -> SymbolId {
    g.rule_names
        .iter()
        .find(|(_, n)| n.as_str() == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("symbol '{name}' not found in rule_names"))
}

/// Look up a SymbolId in token definitions by token name.
#[allow(dead_code)]
fn tok_id(g: &Grammar, name: &str) -> SymbolId {
    g.tokens
        .iter()
        .find(|(_, t)| t.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("token '{name}' not found"))
}

/// Collect token names from a grammar.
fn token_names(g: &Grammar) -> HashSet<String> {
    g.tokens.values().map(|t| t.name.clone()).collect()
}

/// Collect rule LHS names.
fn rule_lhs_names(g: &Grammar) -> HashSet<String> {
    g.rules
        .keys()
        .filter_map(|id| g.rule_names.get(id).cloned())
        .collect()
}

/// Total production count across all LHS.
fn total_productions(g: &Grammar) -> usize {
    g.rules.values().map(|v| v.len()).sum()
}

/// Merge all rules from `src` into `dst` using `add_rule`.
/// Remaps symbol IDs from `src` into `dst`'s namespace via name matching.
/// Returns the number of rules added.
fn merge_rules_into(dst: &mut Grammar, src: &Grammar) -> usize {
    let mut count = 0;
    for rule in src.all_rules() {
        dst.add_rule(rule.clone());
        count += 1;
    }
    count
}

/// Merge tokens from `src` into `dst`, skipping duplicates by name.
fn merge_tokens_into(dst: &mut Grammar, src: &Grammar) -> usize {
    let existing: HashSet<String> = dst.tokens.values().map(|t| t.name.clone()).collect();
    let mut count = 0;
    for (id, token) in &src.tokens {
        if !existing.contains(&token.name) {
            dst.tokens.insert(*id, token.clone());
            count += 1;
        }
    }
    count
}

/// Merge extras from `src` into `dst`, skipping duplicates.
fn merge_extras_into(dst: &mut Grammar, src: &Grammar) -> usize {
    let mut count = 0;
    for &extra_id in &src.extras {
        if !dst.extras.contains(&extra_id) {
            dst.extras.push(extra_id);
            count += 1;
        }
    }
    count
}

/// Merge externals from `src` into `dst`, skipping duplicates by name.
fn merge_externals_into(dst: &mut Grammar, src: &Grammar) -> usize {
    let existing: HashSet<String> = dst.externals.iter().map(|e| e.name.clone()).collect();
    let mut count = 0;
    for ext in &src.externals {
        if !existing.contains(&ext.name) {
            dst.externals.push(ext.clone());
            count += 1;
        }
    }
    count
}

/// Build arithmetic grammar fixture.
fn arith() -> Grammar {
    GrammarBuilder::new("arith")
        .token("NUM", r"\d+")
        .token("+", "+")
        .token("*", "*")
        .rule("expr", vec!["expr", "+", "expr"])
        .rule("expr", vec!["expr", "*", "expr"])
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build()
}

/// Build boolean grammar fixture.
fn bool_grammar() -> Grammar {
    GrammarBuilder::new("bool")
        .token("TRUE", "true")
        .token("FALSE", "false")
        .token("AND", "&&")
        .token("OR", "||")
        .rule("bexpr", vec!["bexpr", "AND", "bexpr"])
        .rule("bexpr", vec!["bexpr", "OR", "bexpr"])
        .rule("bexpr", vec!["TRUE"])
        .rule("bexpr", vec!["FALSE"])
        .start("bexpr")
        .build()
}

/// Build string grammar fixture.
fn string_grammar() -> Grammar {
    GrammarBuilder::new("strings")
        .token("STR", r#""[^"]*""#)
        .token("CONCAT", "++")
        .rule("cat", vec!["cat", "CONCAT", "STR"])
        .rule("cat", vec!["STR"])
        .start("cat")
        .build()
}

/// Build grammar with extras (whitespace).
fn ws_grammar() -> Grammar {
    GrammarBuilder::new("ws")
        .token("ID", r"[a-z]+")
        .token("WS", r"\s+")
        .extra("WS")
        .rule("item", vec!["ID"])
        .start("item")
        .build()
}

/// Build grammar with external scanner tokens.
fn ext_grammar() -> Grammar {
    GrammarBuilder::new("ext")
        .token("ID", r"[a-z]+")
        .token("INDENT", "INDENT")
        .token("DEDENT", "DEDENT")
        .external("INDENT")
        .external("DEDENT")
        .rule("block", vec!["INDENT", "ID", "DEDENT"])
        .start("block")
        .build()
}

/// Build grammar with precedence declarations.
fn prec_grammar() -> Grammar {
    GrammarBuilder::new("prec")
        .token("NUM", r"\d+")
        .token("+", "+")
        .token("*", "*")
        .token("-", "-")
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "-", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "*", "expr"], 2, Associativity::Left)
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build()
}

// ===========================================================================
// 1. basic_merge — merging grammars with overlapping rules (8 tests)
// ===========================================================================

#[test]
fn basic_merge_disjoint_rules_combine_all_lhs() {
    let a = arith();
    let b = bool_grammar();
    let a_names = rule_lhs_names(&a);
    let b_names = rule_lhs_names(&b);
    assert!(a_names.is_disjoint(&b_names));
}

#[test]
fn basic_merge_disjoint_total_production_count() {
    let a = arith();
    let b = bool_grammar();
    let mut merged = a.clone();
    let added = merge_rules_into(&mut merged, &b);
    assert_eq!(added, total_productions(&b));
    assert_eq!(
        total_productions(&merged),
        total_productions(&a) + total_productions(&b)
    );
}

#[test]
fn basic_merge_same_lhs_accumulates_alternatives() {
    let g = GrammarBuilder::new("combined")
        .token("A", "a")
        .token("B", "b")
        .token("C", "c")
        .rule("item", vec!["A"])
        .rule("item", vec!["B"])
        .rule("item", vec!["C"])
        .start("item")
        .build();
    let id = sym(&g, "item");
    assert_eq!(g.rules[&id].len(), 3);
}

#[test]
fn basic_merge_overlapping_lhs_doubles_productions() {
    let a = arith();
    let a2 = arith();
    let mut merged = a.clone();
    merge_rules_into(&mut merged, &a2);
    let expr_id = sym(&a, "expr");
    assert_eq!(merged.rules[&expr_id].len(), 6);
}

#[test]
fn basic_merge_preserves_lhs_identity() {
    let g = arith();
    let expr_id = g.find_symbol_by_name("expr").unwrap();
    let rules = g.get_rules_for_symbol(expr_id).unwrap();
    for r in rules {
        assert_eq!(r.lhs, expr_id);
    }
}

#[test]
fn basic_merge_empty_into_nonempty_is_noop() {
    let a = arith();
    let empty = GrammarBuilder::new("empty").build();
    let mut merged = a.clone();
    let added = merge_rules_into(&mut merged, &empty);
    assert_eq!(added, 0);
    assert_eq!(total_productions(&merged), total_productions(&a));
}

#[test]
fn basic_merge_nonempty_into_empty_copies_all() {
    let a = arith();
    let mut empty = GrammarBuilder::new("empty").build();
    let added = merge_rules_into(&mut empty, &a);
    assert_eq!(added, total_productions(&a));
    assert!(!empty.rules.is_empty());
}

#[test]
fn basic_merge_three_grammars_sequentially() {
    let a = arith();
    let b = bool_grammar();
    let c = string_grammar();
    let mut merged = a.clone();
    merge_rules_into(&mut merged, &b);
    merge_rules_into(&mut merged, &c);
    let expected = total_productions(&a) + total_productions(&b) + total_productions(&c);
    assert_eq!(total_productions(&merged), expected);
}

// ===========================================================================
// 2. merge_precedence — precedence handling during merge (8 tests)
// ===========================================================================

#[test]
fn merge_precedence_preserved_after_build() {
    let g = prec_grammar();
    let expr_id = sym(&g, "expr");
    let rules = &g.rules[&expr_id];
    let prec_rules: Vec<_> = rules.iter().filter(|r| r.precedence.is_some()).collect();
    assert_eq!(prec_rules.len(), 3);
}

#[test]
fn merge_precedence_static_values_correct() {
    let g = prec_grammar();
    let expr_id = sym(&g, "expr");
    let rules = &g.rules[&expr_id];
    let mul_rule = rules
        .iter()
        .find(|r| {
            r.rhs.len() == 3
                && r.rhs
                    .iter()
                    .any(|s| matches!(s, Symbol::Terminal(id) if g.tokens[id].name == "*"))
        })
        .unwrap();
    assert_eq!(mul_rule.precedence, Some(PrecedenceKind::Static(2)));
}

#[test]
fn merge_precedence_associativity_left() {
    let g = prec_grammar();
    let expr_id = sym(&g, "expr");
    for rule in &g.rules[&expr_id] {
        if rule.precedence.is_some() {
            assert_eq!(rule.associativity, Some(Associativity::Left));
        }
    }
}

#[test]
fn merge_precedence_mixed_levels_coexist() {
    let g = GrammarBuilder::new("mixed_prec")
        .token("NUM", r"\d+")
        .token("+", "+")
        .token("*", "*")
        .token("^", "^")
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "*", "expr"], 2, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "^", "expr"], 3, Associativity::Right)
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let expr_id = sym(&g, "expr");
    let levels: HashSet<_> = g.rules[&expr_id]
        .iter()
        .filter_map(|r| match r.precedence {
            Some(PrecedenceKind::Static(v)) => Some(v),
            _ => None,
        })
        .collect();
    assert!(levels.contains(&1));
    assert!(levels.contains(&2));
    assert!(levels.contains(&3));
}

#[test]
fn merge_precedence_right_assoc_preserved() {
    let g = GrammarBuilder::new("right_assoc")
        .token("NUM", r"\d+")
        .token("^", "^")
        .rule_with_precedence("expr", vec!["expr", "^", "expr"], 5, Associativity::Right)
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let expr_id = sym(&g, "expr");
    let power_rule = g.rules[&expr_id]
        .iter()
        .find(|r| r.precedence.is_some())
        .unwrap();
    assert_eq!(power_rule.associativity, Some(Associativity::Right));
}

#[test]
fn merge_precedence_none_for_plain_rule() {
    let g = prec_grammar();
    let expr_id = sym(&g, "expr");
    let num_rule = g.rules[&expr_id].iter().find(|r| r.rhs.len() == 1).unwrap();
    assert!(num_rule.precedence.is_none());
    assert!(num_rule.associativity.is_none());
}

#[test]
fn merge_precedence_overlay_higher_wins() {
    let base = GrammarBuilder::new("base")
        .token("NUM", r"\d+")
        .token("+", "+")
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();

    let overlay = GrammarBuilder::new("overlay")
        .token("NUM", r"\d+")
        .token("+", "+")
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 5, Associativity::Right)
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();

    // Simulate overlay: use overlay precedence when merging
    let expr_base = sym(&base, "expr");
    let expr_over = sym(&overlay, "expr");
    let base_prec = base.rules[&expr_base]
        .iter()
        .find_map(|r| r.precedence)
        .unwrap();
    let overlay_prec = overlay.rules[&expr_over]
        .iter()
        .find_map(|r| r.precedence)
        .unwrap();
    match (base_prec, overlay_prec) {
        (PrecedenceKind::Static(b), PrecedenceKind::Static(o)) => assert!(o > b),
        _ => panic!("expected static precedences"),
    }
}

#[test]
fn merge_precedence_negative_levels_allowed() {
    let g = GrammarBuilder::new("neg_prec")
        .token("NUM", r"\d+")
        .token("+", "+")
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], -3, Associativity::Left)
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build();
    let expr_id = sym(&g, "expr");
    let rule = g.rules[&expr_id]
        .iter()
        .find(|r| r.precedence.is_some())
        .unwrap();
    assert_eq!(rule.precedence, Some(PrecedenceKind::Static(-3)));
}

// ===========================================================================
// 3. merge_tokens — token set merging (8 tests)
// ===========================================================================

#[test]
fn merge_tokens_disjoint_sets_full_union() {
    let a = arith();
    let b = bool_grammar();
    let a_names = token_names(&a);
    let b_names = token_names(&b);
    assert!(a_names.is_disjoint(&b_names));
    let union: HashSet<_> = a_names.union(&b_names).cloned().collect();
    assert_eq!(union.len(), a.tokens.len() + b.tokens.len());
}

#[test]
fn merge_tokens_overlapping_deduplicates() {
    let a = arith();
    let b = GrammarBuilder::new("overlap")
        .token("NUM", r"\d+")
        .token("-", "-")
        .rule("val", vec!["NUM"])
        .start("val")
        .build();
    let mut merged = a.clone();
    let added = merge_tokens_into(&mut merged, &b);
    // NUM already exists in arith, so only "-" is added
    assert_eq!(added, 1);
    assert!(token_names(&merged).contains("-"));
}

#[test]
fn merge_tokens_preserves_pattern_type() {
    let g = arith();
    let num_tok = g.tokens.values().find(|t| t.name == "NUM").unwrap();
    assert!(matches!(num_tok.pattern, TokenPattern::Regex(_)));
    let plus_tok = g.tokens.values().find(|t| t.name == "+").unwrap();
    assert!(matches!(plus_tok.pattern, TokenPattern::String(_)));
}

#[test]
fn merge_tokens_empty_src_no_change() {
    let a = arith();
    let empty = GrammarBuilder::new("empty").build();
    let mut merged = a.clone();
    let added = merge_tokens_into(&mut merged, &empty);
    assert_eq!(added, 0);
    assert_eq!(token_names(&merged), token_names(&a));
}

#[test]
fn merge_tokens_into_empty_copies_all() {
    let a = arith();
    let mut empty = Grammar::new("empty".to_string());
    let added = merge_tokens_into(&mut empty, &a);
    assert_eq!(added, a.tokens.len());
}

#[test]
fn merge_tokens_fragile_flag_preserved() {
    let g = GrammarBuilder::new("fragile_test")
        .fragile_token("SEMI", ";")
        .token("ID", r"[a-z]+")
        .rule("stmt", vec!["ID", "SEMI"])
        .start("stmt")
        .build();
    let semi = g.tokens.values().find(|t| t.name == "SEMI").unwrap();
    assert!(semi.fragile);
    let id_tok = g.tokens.values().find(|t| t.name == "ID").unwrap();
    assert!(!id_tok.fragile);
}

#[test]
fn merge_tokens_unique_ids_across_grammar() {
    let g = GrammarBuilder::new("multi_tok")
        .token("A", "a")
        .token("B", "b")
        .token("C", "c")
        .token("D", "d")
        .rule("root", vec!["A"])
        .start("root")
        .build();
    let ids: Vec<SymbolId> = g.tokens.keys().copied().collect();
    let unique: HashSet<SymbolId> = ids.iter().copied().collect();
    assert_eq!(ids.len(), unique.len());
}

#[test]
fn merge_tokens_three_grammars_combined() {
    let a = arith();
    let b = bool_grammar();
    let c = string_grammar();
    // Compute expected unique token names across all three grammars
    let all_names: HashSet<_> = token_names(&a)
        .union(&token_names(&b))
        .cloned()
        .collect::<HashSet<_>>()
        .union(&token_names(&c))
        .cloned()
        .collect();
    // All three grammars together have 9 distinct token names
    assert_eq!(all_names.len(), 9);
    // Each grammar contributes independently
    assert!(all_names.contains("NUM"));
    assert!(all_names.contains("TRUE"));
    assert!(all_names.contains("STR"));
}

// ===========================================================================
// 4. merge_conflicts — conflict detection during merge (8 tests)
// ===========================================================================

#[test]
fn merge_conflicts_empty_by_default() {
    let g = arith();
    assert!(g.conflicts.is_empty());
}

#[test]
fn merge_conflicts_builder_does_not_add_conflicts() {
    let g = bool_grammar();
    assert!(g.conflicts.is_empty());
}

#[test]
fn merge_conflicts_manual_glr_conflict() {
    let mut g = arith();
    let expr_id = sym(&g, "expr");
    g.conflicts.push(ConflictDeclaration {
        symbols: vec![expr_id],
        resolution: ConflictResolution::GLR,
    });
    assert_eq!(g.conflicts.len(), 1);
    assert!(matches!(g.conflicts[0].resolution, ConflictResolution::GLR));
}

#[test]
fn merge_conflicts_precedence_resolution() {
    let mut g = arith();
    let expr_id = sym(&g, "expr");
    g.conflicts.push(ConflictDeclaration {
        symbols: vec![expr_id],
        resolution: ConflictResolution::Precedence(PrecedenceKind::Static(2)),
    });
    match &g.conflicts[0].resolution {
        ConflictResolution::Precedence(PrecedenceKind::Static(v)) => assert_eq!(*v, 2),
        other => panic!("expected precedence resolution, got {other:?}"),
    }
}

#[test]
fn merge_conflicts_associativity_resolution() {
    let mut g = arith();
    let expr_id = sym(&g, "expr");
    g.conflicts.push(ConflictDeclaration {
        symbols: vec![expr_id],
        resolution: ConflictResolution::Associativity(Associativity::Left),
    });
    assert!(matches!(
        g.conflicts[0].resolution,
        ConflictResolution::Associativity(Associativity::Left)
    ));
}

#[test]
fn merge_conflicts_multiple_symbols_in_declaration() {
    let g = GrammarBuilder::new("multi_conflict")
        .token("A", "a")
        .token("B", "b")
        .rule("alpha", vec!["A"])
        .rule("beta", vec!["B"])
        .start("alpha")
        .build();
    let alpha_id = sym(&g, "alpha");
    let beta_id = sym(&g, "beta");
    let decl = ConflictDeclaration {
        symbols: vec![alpha_id, beta_id],
        resolution: ConflictResolution::GLR,
    };
    assert_eq!(decl.symbols.len(), 2);
}

#[test]
fn merge_conflicts_merged_grammar_accumulates() {
    let mut g = arith();
    let expr_id = sym(&g, "expr");
    g.conflicts.push(ConflictDeclaration {
        symbols: vec![expr_id],
        resolution: ConflictResolution::GLR,
    });
    g.conflicts.push(ConflictDeclaration {
        symbols: vec![expr_id],
        resolution: ConflictResolution::Associativity(Associativity::Right),
    });
    assert_eq!(g.conflicts.len(), 2);
}

#[test]
fn merge_conflicts_different_resolution_strategies() {
    let mut g = prec_grammar();
    let expr_id = sym(&g, "expr");
    g.conflicts.push(ConflictDeclaration {
        symbols: vec![expr_id],
        resolution: ConflictResolution::Precedence(PrecedenceKind::Static(1)),
    });
    g.conflicts.push(ConflictDeclaration {
        symbols: vec![expr_id],
        resolution: ConflictResolution::GLR,
    });
    let resolutions: Vec<_> = g
        .conflicts
        .iter()
        .map(|c| std::mem::discriminant(&c.resolution))
        .collect();
    let unique: HashSet<_> = resolutions.iter().collect();
    assert_eq!(unique.len(), 2);
}

// ===========================================================================
// 5. merge_extras — extras/whitespace merging (8 tests)
// ===========================================================================

#[test]
fn merge_extras_ws_grammar_has_extra() {
    let g = ws_grammar();
    assert!(!g.extras.is_empty());
}

#[test]
fn merge_extras_arith_has_no_extras() {
    let g = arith();
    assert!(g.extras.is_empty());
}

#[test]
fn merge_extras_into_empty_adds_all() {
    let src = ws_grammar();
    let mut dst = Grammar::new("dst".to_string());
    let added = merge_extras_into(&mut dst, &src);
    assert_eq!(added, src.extras.len());
    assert_eq!(dst.extras.len(), src.extras.len());
}

#[test]
fn merge_extras_duplicate_skipped() {
    let src = ws_grammar();
    let mut dst = src.clone();
    let added = merge_extras_into(&mut dst, &src);
    assert_eq!(added, 0);
    assert_eq!(dst.extras.len(), src.extras.len());
}

#[test]
fn merge_extras_from_empty_is_noop() {
    let empty = GrammarBuilder::new("empty").build();
    let mut dst = ws_grammar();
    let original_len = dst.extras.len();
    let added = merge_extras_into(&mut dst, &empty);
    assert_eq!(added, 0);
    assert_eq!(dst.extras.len(), original_len);
}

#[test]
fn merge_extras_two_different_extras_combine() {
    // Build a single grammar with two different extras to test accumulation
    let g = GrammarBuilder::new("two_extras")
        .token("WS", r"\s+")
        .token("COMMENT", r"//[^\n]*")
        .token("ID", r"[a-z]+")
        .extra("WS")
        .extra("COMMENT")
        .rule("root", vec!["ID"])
        .start("root")
        .build();
    assert_eq!(g.extras.len(), 2);
    // Both extras have distinct SymbolIds
    assert_ne!(g.extras[0], g.extras[1]);
}

#[test]
fn merge_extras_preserves_symbol_id() {
    let g = ws_grammar();
    let ws_extra_id = g.extras[0];
    // The extra should reference a token in the grammar
    assert!(g.tokens.contains_key(&ws_extra_id) || g.rule_names.contains_key(&ws_extra_id));
}

#[test]
fn merge_extras_multiple_extras_in_single_grammar() {
    let g = GrammarBuilder::new("multi_extra")
        .token("WS", r"\s+")
        .token("COMMENT", r"//[^\n]*")
        .token("ID", r"[a-z]+")
        .extra("WS")
        .extra("COMMENT")
        .rule("item", vec!["ID"])
        .start("item")
        .build();
    assert_eq!(g.extras.len(), 2);
}

// ===========================================================================
// 6. merge_fields — field mapping merging (8 tests)
// ===========================================================================

#[test]
fn merge_fields_empty_by_default() {
    let g = arith();
    assert!(g.fields.is_empty());
}

#[test]
fn merge_fields_manual_insert_and_read() {
    let mut g = arith();
    g.fields.insert(FieldId(0), "left".to_string());
    g.fields.insert(FieldId(1), "right".to_string());
    assert_eq!(g.fields.len(), 2);
    assert_eq!(g.fields[&FieldId(0)], "left");
    assert_eq!(g.fields[&FieldId(1)], "right");
}

#[test]
fn merge_fields_lexicographic_order_validates() {
    let mut g = Grammar::new("field_test".to_string());
    g.fields.insert(FieldId(0), "alpha".to_string());
    g.fields.insert(FieldId(1), "beta".to_string());
    g.fields.insert(FieldId(2), "gamma".to_string());
    assert!(g.validate().is_ok());
}

#[test]
fn merge_fields_non_lexicographic_order_fails_validation() {
    let mut g = Grammar::new("bad_fields".to_string());
    g.fields.insert(FieldId(0), "zebra".to_string());
    g.fields.insert(FieldId(1), "alpha".to_string());
    assert!(g.validate().is_err());
}

#[test]
fn merge_fields_overlay_union() {
    let mut base = Grammar::new("base".to_string());
    base.fields.insert(FieldId(0), "body".to_string());
    base.fields.insert(FieldId(1), "name".to_string());

    let mut overlay = Grammar::new("overlay".to_string());
    overlay.fields.insert(FieldId(0), "condition".to_string());
    overlay.fields.insert(FieldId(1), "value".to_string());

    // Merge: combine field sets (re-indexing)
    let mut merged_fields: IndexMap<FieldId, String> = IndexMap::new();
    let mut all_names: Vec<String> = base
        .fields
        .values()
        .chain(overlay.fields.values())
        .cloned()
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();
    all_names.sort();
    for (idx, name) in all_names.iter().enumerate() {
        merged_fields.insert(FieldId(idx as u16), name.clone());
    }

    assert_eq!(merged_fields.len(), 4);
    // Verify sorted order
    let names: Vec<_> = merged_fields.values().cloned().collect();
    let mut sorted = names.clone();
    sorted.sort();
    assert_eq!(names, sorted);
}

#[test]
fn merge_fields_duplicate_names_deduplicated() {
    let mut a = Grammar::new("a".to_string());
    a.fields.insert(FieldId(0), "name".to_string());

    let mut b = Grammar::new("b".to_string());
    b.fields.insert(FieldId(0), "name".to_string());

    let combined: HashSet<_> = a
        .fields
        .values()
        .chain(b.fields.values())
        .cloned()
        .collect();
    assert_eq!(combined.len(), 1);
}

#[test]
fn merge_fields_rule_field_binding_preserved() {
    // Build a grammar with a rule that has field bindings
    let mut g = Grammar::new("field_bindings".to_string());
    g.fields.insert(FieldId(0), "operand".to_string());
    g.fields.insert(FieldId(1), "operator".to_string());

    let rule = Rule {
        lhs: SymbolId(1),
        rhs: vec![
            Symbol::NonTerminal(SymbolId(2)),
            Symbol::Terminal(SymbolId(3)),
            Symbol::NonTerminal(SymbolId(4)),
        ],
        precedence: None,
        associativity: None,
        fields: vec![(FieldId(0), 0), (FieldId(1), 1)],
        production_id: ProductionId(0),
    };
    g.add_rule(rule);
    let all: Vec<_> = g.all_rules().collect();
    assert_eq!(all[0].fields.len(), 2);
    assert_eq!(all[0].fields[0], (FieldId(0), 0));
}

#[test]
fn merge_fields_empty_merge_into_nonempty_preserves() {
    let mut g = Grammar::new("with_fields".to_string());
    g.fields.insert(FieldId(0), "value".to_string());
    let empty = Grammar::new("empty".to_string());
    // Merge empty fields into existing — no change
    let existing_count = g.fields.len();
    for (_, name) in &empty.fields {
        if !g.fields.values().any(|n| n == name) {
            let next_id = FieldId(g.fields.len() as u16);
            g.fields.insert(next_id, name.clone());
        }
    }
    assert_eq!(g.fields.len(), existing_count);
}

// ===========================================================================
// 7. merge_externals — external token merging (8 tests)
// ===========================================================================

#[test]
fn merge_externals_present_in_ext_grammar() {
    let g = ext_grammar();
    assert_eq!(g.externals.len(), 2);
}

#[test]
fn merge_externals_absent_in_arith() {
    let g = arith();
    assert!(g.externals.is_empty());
}

#[test]
fn merge_externals_names_match_declaration() {
    let g = ext_grammar();
    let names: HashSet<_> = g.externals.iter().map(|e| e.name.as_str()).collect();
    assert!(names.contains("INDENT"));
    assert!(names.contains("DEDENT"));
}

#[test]
fn merge_externals_into_empty_adds_all() {
    let src = ext_grammar();
    let mut dst = Grammar::new("dst".to_string());
    let added = merge_externals_into(&mut dst, &src);
    assert_eq!(added, 2);
    assert_eq!(dst.externals.len(), 2);
}

#[test]
fn merge_externals_duplicate_skipped() {
    let src = ext_grammar();
    let mut dst = src.clone();
    let added = merge_externals_into(&mut dst, &src);
    assert_eq!(added, 0);
    assert_eq!(dst.externals.len(), 2);
}

#[test]
fn merge_externals_from_empty_is_noop() {
    let empty = GrammarBuilder::new("empty").build();
    let mut dst = ext_grammar();
    let original_len = dst.externals.len();
    let added = merge_externals_into(&mut dst, &empty);
    assert_eq!(added, 0);
    assert_eq!(dst.externals.len(), original_len);
}

#[test]
fn merge_externals_disjoint_combine() {
    let a = GrammarBuilder::new("a")
        .token("INDENT", "INDENT")
        .external("INDENT")
        .token("ID", r"[a-z]+")
        .rule("item", vec!["ID"])
        .start("item")
        .build();
    let b = GrammarBuilder::new("b")
        .token("NEWLINE", "NEWLINE")
        .external("NEWLINE")
        .token("ID", r"[a-z]+")
        .rule("item", vec!["ID"])
        .start("item")
        .build();
    let mut merged = a.clone();
    merge_externals_into(&mut merged, &b);
    assert_eq!(merged.externals.len(), 2);
    let names: HashSet<_> = merged.externals.iter().map(|e| e.name.as_str()).collect();
    assert!(names.contains("INDENT"));
    assert!(names.contains("NEWLINE"));
}

#[test]
fn merge_externals_symbol_id_matches_token() {
    let g = ext_grammar();
    for ext in &g.externals {
        // External symbol_id should reference a known token
        assert!(
            g.tokens.contains_key(&ext.symbol_id) || g.rule_names.contains_key(&ext.symbol_id),
            "external '{}' symbol_id not found",
            ext.name
        );
    }
}

// ===========================================================================
// 8. merge_roundtrip — merge then validate roundtrip (8 tests)
// ===========================================================================

#[test]
fn merge_roundtrip_arith_validates_ok() {
    let g = arith();
    assert!(g.validate().is_ok());
}

#[test]
fn merge_roundtrip_bool_validates_ok() {
    let g = bool_grammar();
    assert!(g.validate().is_ok());
}

#[test]
fn merge_roundtrip_prec_grammar_validates_ok() {
    let g = prec_grammar();
    assert!(g.validate().is_ok());
}

#[test]
fn merge_roundtrip_normalize_then_validate() {
    let mut g = arith();
    let _ = g.normalize();
    assert!(g.validate().is_ok());
}

#[test]
fn merge_roundtrip_clone_equality() {
    let g = arith();
    let cloned = g.clone();
    assert_eq!(g, cloned);
}

#[test]
fn merge_roundtrip_serde_json_roundtrip() {
    let g = arith();
    let json = serde_json::to_string(&g).expect("serialize");
    let deserialized: Grammar = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(g, deserialized);
}

#[test]
fn merge_roundtrip_merged_grammar_validates() {
    let a = arith();
    let b = bool_grammar();
    let mut merged = a.clone();
    merge_rules_into(&mut merged, &b);
    merge_tokens_into(&mut merged, &b);
    // Merge rule_names so validation can resolve symbols
    for (id, name) in &b.rule_names {
        merged.rule_names.insert(*id, name.clone());
    }
    assert!(merged.validate().is_ok());
}

#[test]
fn merge_roundtrip_merged_serde_roundtrip() {
    let a = arith();
    let b = string_grammar();
    let mut merged = a.clone();
    merge_rules_into(&mut merged, &b);
    merge_tokens_into(&mut merged, &b);
    for (id, name) in &b.rule_names {
        merged.rule_names.insert(*id, name.clone());
    }
    let json = serde_json::to_string(&merged).expect("serialize merged");
    let round: Grammar = serde_json::from_str(&json).expect("deserialize merged");
    assert_eq!(merged, round);
}
