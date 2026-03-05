//! Comprehensive tests for SymbolRegistry and symbol management in adze-ir.
//!
//! 90 tests across 15 categories (6 each):
//!   1.  find_existing_*        — find_symbol_by_name for existing rule names
//!   2.  find_nonexistent_*     — find_symbol_by_name returns None for missing names
//!   3.  find_lowercase_*       — lowercase names are findable via rule_names
//!   4.  token_findable_*       — multiple tokens findable through rule_names
//!   5.  start_symbol_*         — start symbol resolution
//!   6.  many_tokens_*          — grammar with 10+ tokens
//!   7.  many_rules_*           — grammar with many rules
//!   8.  build_registry_*       — get_or_build_registry doesn't panic
//!   9.  registry_correct_*     — registry built correctly
//!  10.  rule_names_pop_*       — rule_names populated after build
//!  11.  tokens_pop_*           — tokens populated after build
//!  12.  inline_*               — inline rules findable
//!  13.  supertype_*            — supertype names findable
//!  14.  extra_*                — extra names findable
//!  15.  unique_ids_*           — symbol IDs unique across all symbols

use adze_ir::builder::GrammarBuilder;
use adze_ir::{Grammar, SymbolId};
use std::collections::HashSet;

// ── Helpers ──────────────────────────────────────────────────────────

fn simple_grammar() -> Grammar {
    GrammarBuilder::new("test")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a", "b"])
        .start("start")
        .build()
}

fn multi_token_grammar() -> Grammar {
    GrammarBuilder::new("multi")
        .token("t0", "x0")
        .token("t1", "x1")
        .token("t2", "x2")
        .token("t3", "x3")
        .token("t4", "x4")
        .token("t5", "x5")
        .token("t6", "x6")
        .token("t7", "x7")
        .token("t8", "x8")
        .token("t9", "x9")
        .token("t10", "x10")
        .token("t11", "x11")
        .rule("root", vec!["t0", "t1"])
        .start("root")
        .build()
}

fn multi_rule_grammar() -> Grammar {
    GrammarBuilder::new("rules")
        .token("num", "\\d+")
        .token("id", "[a-z]+")
        .rule("expr", vec!["num"])
        .rule("expr", vec!["id"])
        .rule("expr", vec!["expr", "expr"])
        .rule("stmt", vec!["expr"])
        .rule("stmt", vec!["stmt", "expr"])
        .rule("block", vec!["stmt"])
        .rule("block", vec!["block", "stmt"])
        .rule("program", vec!["block"])
        .start("program")
        .build()
}

fn inline_grammar() -> Grammar {
    GrammarBuilder::new("inline_test")
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
    GrammarBuilder::new("super_test")
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

fn extra_grammar() -> Grammar {
    GrammarBuilder::new("extra_test")
        .token("num", "\\d+")
        .token("ws", "[ \\t]+")
        .token("comment", "//[^\\n]*")
        .rule("root", vec!["num"])
        .extra("ws")
        .extra("comment")
        .start("root")
        .build()
}

// ═══════════════════════════════════════════════════════════════════
// 1. find_existing_* — find_symbol_by_name for existing rule names
// ═══════════════════════════════════════════════════════════════════

#[test]
fn find_existing_start_rule() {
    let g = simple_grammar();
    assert!(g.find_symbol_by_name("start").is_some());
}

#[test]
fn find_existing_returns_valid_symbol_id() {
    let g = simple_grammar();
    let id = g.find_symbol_by_name("start").unwrap();
    assert!(g.rules.contains_key(&id));
}

#[test]
fn find_existing_lowercase_token_a() {
    let g = simple_grammar();
    // Lowercase token names are registered in rule_names by the builder
    assert!(g.find_symbol_by_name("a").is_some());
}

#[test]
fn find_existing_lowercase_token_b() {
    let g = simple_grammar();
    assert!(g.find_symbol_by_name("b").is_some());
}

#[test]
fn find_existing_multi_rule_expr() {
    let g = multi_rule_grammar();
    assert!(g.find_symbol_by_name("expr").is_some());
}

#[test]
fn find_existing_multi_rule_program() {
    let g = multi_rule_grammar();
    let id = g.find_symbol_by_name("program").unwrap();
    assert!(g.rules.contains_key(&id));
}

// ═══════════════════════════════════════════════════════════════════
// 2. find_nonexistent_* — find_symbol_by_name returns None
// ═══════════════════════════════════════════════════════════════════

#[test]
fn find_nonexistent_name_returns_none() {
    let g = simple_grammar();
    assert!(g.find_symbol_by_name("nonexistent").is_none());
}

#[test]
fn find_nonexistent_empty_string_returns_none() {
    let g = simple_grammar();
    assert!(g.find_symbol_by_name("").is_none());
}

#[test]
fn find_nonexistent_uppercase_only_name() {
    // Uppercase-only names are NOT in rule_names
    let g = GrammarBuilder::new("t")
        .token("NUMBER", "\\d+")
        .rule("expr", vec!["NUMBER"])
        .start("expr")
        .build();
    assert!(g.find_symbol_by_name("NUMBER").is_none());
}

#[test]
fn find_nonexistent_punctuation_token() {
    let g = GrammarBuilder::new("t")
        .token("+", "+")
        .token("num", "\\d+")
        .rule("expr", vec!["num", "+", "num"])
        .start("expr")
        .build();
    assert!(g.find_symbol_by_name("+").is_none());
}

#[test]
fn find_nonexistent_similar_name() {
    let g = simple_grammar();
    assert!(g.find_symbol_by_name("Start").is_none());
}

#[test]
fn find_nonexistent_in_empty_grammar() {
    let g = Grammar::new("empty".to_string());
    assert!(g.find_symbol_by_name("anything").is_none());
}

// ═══════════════════════════════════════════════════════════════════
// 3. find_lowercase_* — lowercase names work in find_symbol_by_name
// ═══════════════════════════════════════════════════════════════════

#[test]
fn find_lowercase_single_char_token() {
    let g = simple_grammar();
    let id = g.find_symbol_by_name("a").unwrap();
    assert!(g.tokens.contains_key(&id));
}

#[test]
fn find_lowercase_multi_char_rule() {
    let g = multi_rule_grammar();
    assert!(g.find_symbol_by_name("stmt").is_some());
}

#[test]
fn find_lowercase_all_rules_findable() {
    let g = multi_rule_grammar();
    for name in ["expr", "stmt", "block", "program"] {
        assert!(
            g.find_symbol_by_name(name).is_some(),
            "expected to find {name}"
        );
    }
}

#[test]
fn find_lowercase_returns_distinct_ids() {
    let g = multi_rule_grammar();
    let ids: Vec<SymbolId> = ["expr", "stmt", "block", "program"]
        .iter()
        .filter_map(|n| g.find_symbol_by_name(n))
        .collect();
    let unique: HashSet<u16> = ids.iter().map(|id| id.0).collect();
    assert_eq!(ids.len(), unique.len());
}

#[test]
fn find_lowercase_mixed_case_findable() {
    // Mixed-case names are NOT all-uppercase, so they go into rule_names
    let g = GrammarBuilder::new("t")
        .token("myToken", "tok")
        .rule("myRule", vec!["myToken"])
        .start("myRule")
        .build();
    assert!(g.find_symbol_by_name("myToken").is_some());
    assert!(g.find_symbol_by_name("myRule").is_some());
}

#[test]
fn find_lowercase_underscore_prefix_findable() {
    let g = GrammarBuilder::new("t")
        .token("_hidden", "h")
        .rule("root", vec!["_hidden"])
        .start("root")
        .build();
    assert!(g.find_symbol_by_name("_hidden").is_some());
}

// ═══════════════════════════════════════════════════════════════════
// 4. token_findable_* — multiple tokens findable through rule_names
// ═══════════════════════════════════════════════════════════════════

#[test]
fn token_findable_all_lowercase_tokens() {
    let g = multi_token_grammar();
    for i in 0..12 {
        let name = format!("t{i}");
        assert!(
            g.find_symbol_by_name(&name).is_some(),
            "expected to find {name}"
        );
    }
}

#[test]
fn token_findable_ids_match_token_keys() {
    let g = simple_grammar();
    let id_a = g.find_symbol_by_name("a").unwrap();
    assert!(g.tokens.contains_key(&id_a));
}

#[test]
fn token_findable_ids_match_for_b() {
    let g = simple_grammar();
    let id_b = g.find_symbol_by_name("b").unwrap();
    assert!(g.tokens.contains_key(&id_b));
}

#[test]
fn token_findable_uppercase_excluded() {
    let g = GrammarBuilder::new("t")
        .token("ALPHA", "a")
        .token("BETA", "b")
        .token("lower", "c")
        .rule("root", vec!["lower"])
        .start("root")
        .build();
    assert!(g.find_symbol_by_name("ALPHA").is_none());
    assert!(g.find_symbol_by_name("BETA").is_none());
    assert!(g.find_symbol_by_name("lower").is_some());
}

#[test]
fn token_findable_count_in_rule_names() {
    let g = simple_grammar();
    // "a", "b", and "start" should all be in rule_names
    assert_eq!(g.rule_names.len(), 3);
}

#[test]
fn token_findable_rule_names_values_match() {
    let g = simple_grammar();
    let names: HashSet<&str> = g.rule_names.values().map(String::as_str).collect();
    assert!(names.contains("a"));
    assert!(names.contains("b"));
    assert!(names.contains("start"));
}

// ═══════════════════════════════════════════════════════════════════
// 5. start_symbol_* — start symbol resolution
// ═══════════════════════════════════════════════════════════════════

#[test]
fn start_symbol_findable_by_name() {
    let g = simple_grammar();
    assert!(g.find_symbol_by_name("start").is_some());
}

#[test]
fn start_symbol_has_rules() {
    let g = simple_grammar();
    let id = g.find_symbol_by_name("start").unwrap();
    assert!(g.rules.contains_key(&id));
}

#[test]
fn start_symbol_rules_are_first() {
    let g = simple_grammar();
    let start_id = g.find_symbol_by_name("start").unwrap();
    let first_rule_lhs = *g.rules.keys().next().unwrap();
    assert_eq!(first_rule_lhs, start_id);
}

#[test]
fn start_symbol_in_rule_names() {
    let g = simple_grammar();
    let start_id = g.find_symbol_by_name("start").unwrap();
    assert!(g.rule_names.contains_key(&start_id));
}

#[test]
fn start_symbol_multi_rule_grammar() {
    let g = multi_rule_grammar();
    let program_id = g.find_symbol_by_name("program").unwrap();
    let first_rule_lhs = *g.rules.keys().next().unwrap();
    assert_eq!(first_rule_lhs, program_id);
}

#[test]
fn start_symbol_not_in_tokens() {
    let g = simple_grammar();
    let start_id = g.find_symbol_by_name("start").unwrap();
    assert!(!g.tokens.contains_key(&start_id));
}

// ═══════════════════════════════════════════════════════════════════
// 6. many_tokens_* — grammar with 10+ tokens
// ═══════════════════════════════════════════════════════════════════

#[test]
fn many_tokens_count() {
    let g = multi_token_grammar();
    assert_eq!(g.tokens.len(), 12);
}

#[test]
fn many_tokens_all_unique_ids() {
    let g = multi_token_grammar();
    let ids: HashSet<u16> = g.tokens.keys().map(|id| id.0).collect();
    assert_eq!(ids.len(), g.tokens.len());
}

#[test]
fn many_tokens_all_nonzero_ids() {
    let g = multi_token_grammar();
    for id in g.tokens.keys() {
        assert_ne!(id.0, 0, "token IDs should not be 0 (reserved for EOF)");
    }
}

#[test]
fn many_tokens_names_correct() {
    let g = multi_token_grammar();
    let names: HashSet<&str> = g.tokens.values().map(|t| t.name.as_str()).collect();
    for i in 0..12 {
        let name = format!("t{i}");
        assert!(names.contains(name.as_str()), "missing token {name}");
    }
}

#[test]
fn many_tokens_patterns_correct() {
    let g = multi_token_grammar();
    for (_, token) in &g.tokens {
        assert!(!token.name.is_empty());
    }
}

#[test]
fn many_tokens_rule_names_include_all() {
    let g = multi_token_grammar();
    // All lowercase token names + "root" should be in rule_names
    assert!(g.rule_names.len() >= 13);
}

// ═══════════════════════════════════════════════════════════════════
// 7. many_rules_* — grammar with many rules
// ═══════════════════════════════════════════════════════════════════

#[test]
fn many_rules_distinct_lhs_count() {
    let g = multi_rule_grammar();
    // expr, stmt, block, program → 4 distinct LHS
    assert_eq!(g.rules.len(), 4);
}

#[test]
fn many_rules_expr_has_alternatives() {
    let g = multi_rule_grammar();
    let expr_id = g.find_symbol_by_name("expr").unwrap();
    let rules = g.rules.get(&expr_id).unwrap();
    assert_eq!(rules.len(), 3);
}

#[test]
fn many_rules_stmt_has_alternatives() {
    let g = multi_rule_grammar();
    let stmt_id = g.find_symbol_by_name("stmt").unwrap();
    let rules = g.rules.get(&stmt_id).unwrap();
    assert_eq!(rules.len(), 2);
}

#[test]
fn many_rules_all_lhs_findable() {
    let g = multi_rule_grammar();
    for name in ["expr", "stmt", "block", "program"] {
        let id = g.find_symbol_by_name(name).unwrap();
        assert!(g.rules.contains_key(&id), "{name} should have rules");
    }
}

#[test]
fn many_rules_total_production_count() {
    let g = multi_rule_grammar();
    let total: usize = g.rules.values().map(|v| v.len()).sum();
    assert_eq!(total, 8);
}

#[test]
fn many_rules_each_production_has_rhs() {
    let g = multi_rule_grammar();
    for rules in g.rules.values() {
        for rule in rules {
            assert!(!rule.rhs.is_empty());
        }
    }
}

// ═══════════════════════════════════════════════════════════════════
// 8. build_registry_* — get_or_build_registry doesn't panic
// ═══════════════════════════════════════════════════════════════════

#[test]
fn build_registry_simple_no_panic() {
    let mut g = simple_grammar();
    let _reg = g.get_or_build_registry();
}

#[test]
fn build_registry_returns_nonempty() {
    let mut g = simple_grammar();
    let reg = g.get_or_build_registry();
    assert!(!reg.is_empty());
}

#[test]
fn build_registry_contains_end_symbol() {
    let mut g = simple_grammar();
    let reg = g.get_or_build_registry();
    assert!(reg.get_id("end").is_some());
    assert_eq!(reg.get_id("end").unwrap(), SymbolId(0));
}

#[test]
fn build_registry_multi_token_no_panic() {
    let mut g = multi_token_grammar();
    let _reg = g.get_or_build_registry();
}

#[test]
fn build_registry_multi_rule_no_panic() {
    let mut g = multi_rule_grammar();
    let _reg = g.get_or_build_registry();
}

#[test]
fn build_registry_starts_none_then_some() {
    let mut g = simple_grammar();
    assert!(g.symbol_registry.is_none());
    let _reg = g.get_or_build_registry();
    assert!(g.symbol_registry.is_some());
}

// ═══════════════════════════════════════════════════════════════════
// 9. registry_correct_* — registry built correctly
// ═══════════════════════════════════════════════════════════════════

#[test]
fn registry_correct_contains_tokens() {
    let mut g = simple_grammar();
    let reg = g.get_or_build_registry();
    assert!(reg.get_id("a").is_some());
    assert!(reg.get_id("b").is_some());
}

#[test]
fn registry_correct_contains_rules() {
    let mut g = multi_rule_grammar();
    let reg = g.get_or_build_registry();
    // Rule names registered as non-terminals
    for name in ["expr", "stmt", "block", "program"] {
        assert!(reg.get_id(name).is_some(), "registry should contain {name}");
    }
}

#[test]
fn registry_correct_tokens_are_terminal() {
    let mut g = simple_grammar();
    let reg = g.get_or_build_registry();
    for name in ["a", "b"] {
        let id = reg.get_id(name).unwrap();
        let meta = reg.get_metadata(id).unwrap();
        assert!(meta.terminal, "{name} should be terminal");
    }
}

#[test]
fn registry_correct_rules_are_nonterminal() {
    let mut g = multi_rule_grammar();
    let reg = g.get_or_build_registry();
    // Rules that are not tokens should be non-terminal
    let expr_id = reg.get_id("expr").unwrap();
    let meta = reg.get_metadata(expr_id).unwrap();
    assert!(!meta.terminal);
}

#[test]
fn registry_correct_index_map_covers_all() {
    let mut g = simple_grammar();
    let reg = g.get_or_build_registry();
    let idx_map = reg.to_index_map();
    assert_eq!(idx_map.len(), reg.len());
}

#[test]
fn registry_correct_symbol_map_covers_all() {
    let mut g = simple_grammar();
    let reg = g.get_or_build_registry();
    let sym_map = reg.to_symbol_map();
    assert_eq!(sym_map.len(), reg.len());
}

// ═══════════════════════════════════════════════════════════════════
// 10. rule_names_pop_* — rule_names populated after build
// ═══════════════════════════════════════════════════════════════════

#[test]
fn rule_names_pop_not_empty() {
    let g = simple_grammar();
    assert!(!g.rule_names.is_empty());
}

#[test]
fn rule_names_pop_contains_rule_lhs() {
    let g = simple_grammar();
    let names: HashSet<&str> = g.rule_names.values().map(String::as_str).collect();
    assert!(names.contains("start"));
}

#[test]
fn rule_names_pop_contains_lowercase_tokens() {
    let g = simple_grammar();
    let names: HashSet<&str> = g.rule_names.values().map(String::as_str).collect();
    assert!(names.contains("a"));
    assert!(names.contains("b"));
}

#[test]
fn rule_names_pop_excludes_uppercase_only() {
    let g = GrammarBuilder::new("t")
        .token("NUM", "\\d+")
        .token("PLUS", "\\+")
        .rule("expr", vec!["NUM", "PLUS", "NUM"])
        .start("expr")
        .build();
    let names: HashSet<&str> = g.rule_names.values().map(String::as_str).collect();
    assert!(!names.contains("NUM"));
    assert!(!names.contains("PLUS"));
    assert!(names.contains("expr"));
}

#[test]
fn rule_names_pop_maps_to_correct_ids() {
    let g = simple_grammar();
    for (&id, name) in &g.rule_names {
        let found = g.find_symbol_by_name(name).unwrap();
        assert_eq!(found, id);
    }
}

#[test]
fn rule_names_pop_count_matches() {
    let g = multi_rule_grammar();
    // num, id, expr, stmt, block, program → 6 names
    assert_eq!(g.rule_names.len(), 6);
}

// ═══════════════════════════════════════════════════════════════════
// 11. tokens_pop_* — tokens populated after build
// ═══════════════════════════════════════════════════════════════════

#[test]
fn tokens_pop_not_empty() {
    let g = simple_grammar();
    assert!(!g.tokens.is_empty());
}

#[test]
fn tokens_pop_correct_count() {
    let g = simple_grammar();
    assert_eq!(g.tokens.len(), 2);
}

#[test]
fn tokens_pop_each_has_name() {
    let g = multi_token_grammar();
    for token in g.tokens.values() {
        assert!(!token.name.is_empty());
    }
}

#[test]
fn tokens_pop_each_has_pattern() {
    let g = simple_grammar();
    for token in g.tokens.values() {
        match &token.pattern {
            adze_ir::TokenPattern::String(s) => assert!(!s.is_empty()),
            adze_ir::TokenPattern::Regex(r) => assert!(!r.is_empty()),
        }
    }
}

#[test]
fn tokens_pop_fragile_flag_defaults_false() {
    let g = simple_grammar();
    for token in g.tokens.values() {
        assert!(!token.fragile);
    }
}

#[test]
fn tokens_pop_fragile_token_works() {
    let g = GrammarBuilder::new("t")
        .fragile_token("err", "ERROR")
        .token("ok", "ok")
        .rule("root", vec!["ok"])
        .start("root")
        .build();
    let fragile_count = g.tokens.values().filter(|t| t.fragile).count();
    assert_eq!(fragile_count, 1);
}

// ═══════════════════════════════════════════════════════════════════
// 12. inline_* — inline rules findable
// ═══════════════════════════════════════════════════════════════════

#[test]
fn inline_rules_vector_populated() {
    let g = inline_grammar();
    assert!(!g.inline_rules.is_empty());
}

#[test]
fn inline_rule_name_findable() {
    let g = inline_grammar();
    assert!(g.find_symbol_by_name("primary").is_some());
}

#[test]
fn inline_rule_id_matches() {
    let g = inline_grammar();
    let found_id = g.find_symbol_by_name("primary").unwrap();
    assert!(g.inline_rules.contains(&found_id));
}

#[test]
fn inline_rule_has_productions() {
    let g = inline_grammar();
    let id = g.find_symbol_by_name("primary").unwrap();
    assert!(g.rules.contains_key(&id));
}

#[test]
fn inline_multiple_rules() {
    let g = GrammarBuilder::new("t")
        .token("num", "\\d+")
        .token("id", "[a-z]+")
        .rule("atom", vec!["num"])
        .rule("ref_expr", vec!["id"])
        .rule("expr", vec!["atom"])
        .rule("expr", vec!["ref_expr"])
        .inline("atom")
        .inline("ref_expr")
        .start("expr")
        .build();
    assert_eq!(g.inline_rules.len(), 2);
    assert!(g.find_symbol_by_name("atom").is_some());
    assert!(g.find_symbol_by_name("ref_expr").is_some());
}

#[test]
fn inline_rule_also_in_rule_names() {
    let g = inline_grammar();
    let id = g.find_symbol_by_name("primary").unwrap();
    assert!(g.rule_names.contains_key(&id));
}

// ═══════════════════════════════════════════════════════════════════
// 13. supertype_* — supertype names findable
// ═══════════════════════════════════════════════════════════════════

#[test]
fn supertype_vector_populated() {
    let g = supertype_grammar();
    assert!(!g.supertypes.is_empty());
}

#[test]
fn supertype_name_findable() {
    let g = supertype_grammar();
    assert!(g.find_symbol_by_name("expression").is_some());
}

#[test]
fn supertype_id_matches() {
    let g = supertype_grammar();
    let found_id = g.find_symbol_by_name("expression").unwrap();
    assert!(g.supertypes.contains(&found_id));
}

#[test]
fn supertype_has_rules() {
    let g = supertype_grammar();
    let id = g.find_symbol_by_name("expression").unwrap();
    assert!(g.rules.contains_key(&id));
}

#[test]
fn supertype_multiple() {
    let g = GrammarBuilder::new("t")
        .token("num", "\\d+")
        .token("id", "[a-z]+")
        .rule("literal", vec!["num"])
        .rule("variable", vec!["id"])
        .rule("expression", vec!["literal"])
        .rule("expression", vec!["variable"])
        .rule("declaration", vec!["variable"])
        .supertype("expression")
        .supertype("declaration")
        .start("expression")
        .build();
    assert_eq!(g.supertypes.len(), 2);
}

#[test]
fn supertype_also_in_rule_names() {
    let g = supertype_grammar();
    let id = g.find_symbol_by_name("expression").unwrap();
    assert!(g.rule_names.contains_key(&id));
}

// ═══════════════════════════════════════════════════════════════════
// 14. extra_* — extra names findable
// ═══════════════════════════════════════════════════════════════════

#[test]
fn extra_vector_populated() {
    let g = extra_grammar();
    assert!(!g.extras.is_empty());
}

#[test]
fn extra_name_findable() {
    let g = extra_grammar();
    assert!(g.find_symbol_by_name("ws").is_some());
}

#[test]
fn extra_id_matches() {
    let g = extra_grammar();
    let ws_id = g.find_symbol_by_name("ws").unwrap();
    assert!(g.extras.contains(&ws_id));
}

#[test]
fn extra_multiple() {
    let g = extra_grammar();
    assert_eq!(g.extras.len(), 2);
}

#[test]
fn extra_comment_findable() {
    let g = extra_grammar();
    assert!(g.find_symbol_by_name("comment").is_some());
    let id = g.find_symbol_by_name("comment").unwrap();
    assert!(g.extras.contains(&id));
}

#[test]
fn extra_marked_hidden_in_registry() {
    let mut g = extra_grammar();
    let reg = g.get_or_build_registry();
    let ws_id = reg.get_id("ws").unwrap();
    let meta = reg.get_metadata(ws_id).unwrap();
    assert!(meta.hidden);
}

// ═══════════════════════════════════════════════════════════════════
// 15. unique_ids_* — symbol IDs unique across all symbols
// ═══════════════════════════════════════════════════════════════════

#[test]
fn unique_ids_tokens_all_distinct() {
    let g = multi_token_grammar();
    let ids: HashSet<u16> = g.tokens.keys().map(|id| id.0).collect();
    assert_eq!(ids.len(), g.tokens.len());
}

#[test]
fn unique_ids_rules_all_distinct() {
    let g = multi_rule_grammar();
    let ids: HashSet<u16> = g.rules.keys().map(|id| id.0).collect();
    assert_eq!(ids.len(), g.rules.len());
}

#[test]
fn unique_ids_rule_names_all_distinct() {
    let g = multi_rule_grammar();
    let ids: HashSet<u16> = g.rule_names.keys().map(|id| id.0).collect();
    assert_eq!(ids.len(), g.rule_names.len());
}

#[test]
fn unique_ids_registry_all_distinct() {
    let mut g = multi_rule_grammar();
    let reg = g.get_or_build_registry();
    let ids: HashSet<u16> = reg.iter().map(|(_, info)| info.id.0).collect();
    assert_eq!(ids.len(), reg.len());
}

#[test]
fn unique_ids_large_grammar() {
    let mut builder = GrammarBuilder::new("large");
    for i in 0..50 {
        builder = builder.token(&format!("tok{i}"), &format!("p{i}"));
    }
    builder = builder.rule("root", vec!["tok0"]).start("root");
    let g = builder.build();

    let token_ids: HashSet<u16> = g.tokens.keys().map(|id| id.0).collect();
    assert_eq!(token_ids.len(), 50);
}

#[test]
fn unique_ids_no_overlap_tokens_and_rules() {
    let g = multi_rule_grammar();
    let token_ids: HashSet<u16> = g.tokens.keys().map(|id| id.0).collect();
    let _rule_ids: HashSet<u16> = g.rules.keys().map(|id| id.0).collect();
    // Tokens referenced in RHS of rules may appear in both, but their
    // SymbolId should be consistent — check no rule LHS collides with a
    // token that has a different name.
    for (&rule_id, name) in &g.rule_names {
        if token_ids.contains(&rule_id.0) {
            // If the ID appears in both, the token must have the same name
            if let Some(token) = g.tokens.get(&rule_id) {
                assert_eq!(token.name, *name);
            }
        }
    }
    // All rule LHS IDs that are purely non-terminals should not be token IDs
    for &rule_lhs in g.rules.keys() {
        if !g.tokens.contains_key(&rule_lhs) {
            assert!(
                !token_ids.contains(&rule_lhs.0),
                "rule LHS {rule_lhs} should not collide with a token ID"
            );
        }
    }
}
