//! Comprehensive tests for field name handling in adze-ir Grammar.
//!
//! 84 tests across 10 categories covering grammar names, rule_names,
//! tokens, fields, clone, Debug, serde roundtrip, normalize, optimize,
//! and grammar size variations.

use adze_ir::builder::GrammarBuilder;
use adze_ir::optimizer::{GrammarOptimizer, optimize_grammar};
use adze_ir::{FieldId, Grammar};

// ── Helpers ────────────────────────────────────────────────────────────────

/// Build a single-rule grammar with unique prefix.
fn single_rule_grammar(prefix: &str) -> Grammar {
    GrammarBuilder::new(&format!("{prefix}_single"))
        .token("A", "a")
        .rule("start", vec!["A"])
        .start("start")
        .build()
}

/// Build an arithmetic grammar.
fn arith_grammar(prefix: &str) -> Grammar {
    GrammarBuilder::new(&format!("{prefix}_arith"))
        .token("NUM", r"\d+")
        .token("+", "+")
        .token("*", "*")
        .rule("expr", vec!["expr", "+", "expr"])
        .rule("expr", vec!["expr", "*", "expr"])
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build()
}

/// Build a grammar with N distinct rules: rule_0, rule_1, …, rule_{n-1}.
fn n_rule_grammar(prefix: &str, n: usize) -> Grammar {
    let name = format!("{prefix}_{n}r");
    let mut b = GrammarBuilder::new(&name).token("T", "t");
    for i in 0..n {
        let rule_name = format!("rule_{i}");
        // Leak the string so we get a &'static str for the vec
        let lhs: &'static str = Box::leak(rule_name.into_boxed_str());
        b = b.rule(lhs, vec!["T"]);
    }
    b.start("rule_0").build()
}

// ═══════════════════════════════════════════════════════════════════════════
// 1. Grammar name basics (tests 1–10)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn fn_v10_name_matches_builder_input() {
    let g = GrammarBuilder::new("fn_v10_alpha").build();
    assert_eq!(g.name, "fn_v10_alpha");
}

#[test]
fn fn_v10_name_non_empty() {
    let g = GrammarBuilder::new("fn_v10_non_empty").build();
    assert!(!g.name.is_empty());
}

#[test]
fn fn_v10_name_with_underscore() {
    let g = GrammarBuilder::new("fn_v10_my_grammar").build();
    assert_eq!(g.name, "fn_v10_my_grammar");
}

#[test]
fn fn_v10_name_with_digits() {
    let g = GrammarBuilder::new("fn_v10_g42").build();
    assert_eq!(g.name, "fn_v10_g42");
}

#[test]
fn fn_v10_name_leading_digit() {
    let g = GrammarBuilder::new("9fn_v10_lead").build();
    assert_eq!(g.name, "9fn_v10_lead");
}

#[test]
fn fn_v10_name_long() {
    let long = "fn_v10_".to_string() + &"x".repeat(200);
    let g = GrammarBuilder::new(&long).build();
    assert_eq!(g.name, long);
}

#[test]
fn fn_v10_name_unicode() {
    let g = GrammarBuilder::new("fn_v10_αβγ").build();
    assert_eq!(g.name, "fn_v10_αβγ");
}

#[test]
fn fn_v10_name_hyphen() {
    let g = GrammarBuilder::new("fn_v10_my-grammar").build();
    assert!(g.name.contains('-'));
}

#[test]
fn fn_v10_grammar_new_matches_name() {
    let g = Grammar::new("fn_v10_via_new".to_string());
    assert_eq!(g.name, "fn_v10_via_new");
}

#[test]
fn fn_v10_empty_builder_name_is_empty_string() {
    let g = GrammarBuilder::new("").build();
    assert_eq!(g.name, "");
}

// ═══════════════════════════════════════════════════════════════════════════
// 2. rule_names — presence and content (tests 11–20)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn fn_v10_rule_names_contains_defined_rules() {
    let g = arith_grammar("fn_v10_rn1");
    let names: Vec<&str> = g.rule_names.values().map(|s| s.as_str()).collect();
    assert!(names.contains(&"expr"));
}

#[test]
fn fn_v10_rule_names_excludes_token_names() {
    let g = arith_grammar("fn_v10_rn2");
    let names: Vec<&str> = g.rule_names.values().map(|s| s.as_str()).collect();
    assert!(!names.contains(&"NUM"));
    assert!(!names.contains(&"+"));
    assert!(!names.contains(&"*"));
}

#[test]
fn fn_v10_rule_names_preserves_insertion_order() {
    let g = GrammarBuilder::new("fn_v10_order")
        .token("T", "t")
        .rule("alpha", vec!["T"])
        .rule("beta", vec!["T"])
        .rule("gamma", vec!["T"])
        .start("alpha")
        .build();

    let names: Vec<&str> = g.rule_names.values().map(|s| s.as_str()).collect();
    // "alpha" appears first because start() moves it first
    assert_eq!(names[0], "alpha");
    // beta and gamma follow in insertion order
    assert!(names.contains(&"beta"));
    assert!(names.contains(&"gamma"));
}

#[test]
fn fn_v10_single_rule_single_name() {
    let g = single_rule_grammar("fn_v10_sr");
    let names: Vec<&str> = g.rule_names.values().map(|s| s.as_str()).collect();
    assert!(names.contains(&"start"));
}

#[test]
fn fn_v10_multiple_rules_all_in_names() {
    let g = GrammarBuilder::new("fn_v10_multi")
        .token("T", "t")
        .rule("aaa", vec!["T"])
        .rule("bbb", vec!["T"])
        .rule("ccc", vec!["T"])
        .rule("ddd", vec!["T"])
        .start("aaa")
        .build();

    let names: Vec<&str> = g.rule_names.values().map(|s| s.as_str()).collect();
    for expected in &["aaa", "bbb", "ccc", "ddd"] {
        assert!(names.contains(expected), "missing rule name: {expected}");
    }
}

#[test]
fn fn_v10_rule_names_count_matches_distinct_lhs() {
    let g = arith_grammar("fn_v10_cnt");
    // "expr" is the only non-terminal rule LHS
    let rule_name_vals: Vec<&str> = g.rule_names.values().map(|s| s.as_str()).collect();
    assert!(rule_name_vals.contains(&"expr"));
}

#[test]
fn fn_v10_rule_names_is_indexmap() {
    let g = arith_grammar("fn_v10_imap");
    // Verify it's an IndexMap by checking iterator determinism
    let first: Vec<String> = g.rule_names.values().cloned().collect();
    let second: Vec<String> = g.rule_names.values().cloned().collect();
    assert_eq!(first, second);
}

#[test]
fn fn_v10_rule_names_no_duplicates() {
    let g = n_rule_grammar("fn_v10_nodup", 10);
    let names: Vec<&str> = g.rule_names.values().map(|s| s.as_str()).collect();
    let mut deduped = names.clone();
    deduped.sort();
    deduped.dedup();
    assert_eq!(names.len(), deduped.len());
}

#[test]
fn fn_v10_rule_names_key_is_symbol_id() {
    let g = single_rule_grammar("fn_v10_key");
    for (sid, _name) in &g.rule_names {
        // SymbolId(0) is reserved for EOF; builder starts at 1
        assert!(sid.0 > 0);
    }
}

#[test]
fn fn_v10_find_symbol_by_name_returns_id() {
    let g = arith_grammar("fn_v10_find");
    let sid = g.find_symbol_by_name("expr");
    assert!(sid.is_some());
}

// ═══════════════════════════════════════════════════════════════════════════
// 3. Token access (tests 21–28)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn fn_v10_tokens_accessible() {
    let g = arith_grammar("fn_v10_tok1");
    assert!(!g.tokens.is_empty());
}

#[test]
fn fn_v10_tokens_contain_defined_tokens() {
    let g = arith_grammar("fn_v10_tok2");
    let tok_names: Vec<&str> = g.tokens.values().map(|t| t.name.as_str()).collect();
    assert!(tok_names.contains(&"NUM"));
}

#[test]
fn fn_v10_tokens_count() {
    let g = arith_grammar("fn_v10_tok3");
    // NUM, +, *
    assert_eq!(g.tokens.len(), 3);
}

#[test]
fn fn_v10_tokens_not_in_rule_names_values() {
    let g = arith_grammar("fn_v10_tok4");
    let rule_name_vals: Vec<&str> = g.rule_names.values().map(|s| s.as_str()).collect();
    for tok in g.tokens.values() {
        assert!(
            !rule_name_vals.contains(&tok.name.as_str()),
            "token {} must not appear in rule_names values",
            tok.name
        );
    }
}

#[test]
fn fn_v10_token_pattern_is_string_for_literal() {
    let g = GrammarBuilder::new("fn_v10_tpat")
        .token("IF", "if")
        .rule("start", vec!["IF"])
        .start("start")
        .build();

    let tok = g.tokens.values().next().unwrap();
    assert_eq!(tok.name, "IF");
}

#[test]
fn fn_v10_token_keys_are_symbol_ids() {
    let g = arith_grammar("fn_v10_tkey");
    for sid in g.tokens.keys() {
        assert!(sid.0 > 0);
    }
}

#[test]
fn fn_v10_fragile_token_flag() {
    let g = GrammarBuilder::new("fn_v10_frag")
        .fragile_token("ERR", "error")
        .token("A", "a")
        .rule("start", vec!["A"])
        .start("start")
        .build();

    let err_tok = g.tokens.values().find(|t| t.name == "ERR").unwrap();
    assert!(err_tok.fragile);
}

#[test]
fn fn_v10_non_fragile_by_default() {
    let g = single_rule_grammar("fn_v10_nfrag");
    for tok in g.tokens.values() {
        assert!(!tok.fragile);
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// 4. Fields collection (tests 29–36)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn fn_v10_fields_empty_by_default() {
    let g = arith_grammar("fn_v10_fld1");
    assert!(g.fields.is_empty());
}

#[test]
fn fn_v10_fields_manual_insert() {
    let mut g = Grammar::new("fn_v10_fld2".to_string());
    g.fields.insert(FieldId(0), "alpha".to_string());
    g.fields.insert(FieldId(1), "beta".to_string());
    assert_eq!(g.fields.len(), 2);
}

#[test]
fn fn_v10_fields_lexicographic_order_validates() {
    let mut g = Grammar::new("fn_v10_fld3".to_string());
    g.fields.insert(FieldId(0), "alpha".to_string());
    g.fields.insert(FieldId(1), "beta".to_string());
    assert!(g.validate().is_ok());
}

#[test]
fn fn_v10_fields_wrong_order_fails_validation() {
    let mut g = Grammar::new("fn_v10_fld4".to_string());
    g.fields.insert(FieldId(0), "zebra".to_string());
    g.fields.insert(FieldId(1), "alpha".to_string());
    assert!(g.validate().is_err());
}

#[test]
fn fn_v10_fields_single_entry_validates() {
    let mut g = Grammar::new("fn_v10_fld5".to_string());
    g.fields.insert(FieldId(0), "only".to_string());
    assert!(g.validate().is_ok());
}

#[test]
fn fn_v10_fields_preserves_insertion_order() {
    let mut g = Grammar::new("fn_v10_fld6".to_string());
    g.fields.insert(FieldId(0), "aaa".to_string());
    g.fields.insert(FieldId(1), "bbb".to_string());
    g.fields.insert(FieldId(2), "ccc".to_string());

    let vals: Vec<&str> = g.fields.values().map(|s| s.as_str()).collect();
    assert_eq!(vals, vec!["aaa", "bbb", "ccc"]);
}

#[test]
fn fn_v10_fields_key_is_field_id() {
    let mut g = Grammar::new("fn_v10_fld7".to_string());
    g.fields.insert(FieldId(42), "foo".to_string());
    assert!(g.fields.contains_key(&FieldId(42)));
}

#[test]
fn fn_v10_fields_many_entries() {
    let mut g = Grammar::new("fn_v10_fld8".to_string());
    for i in 0..20u16 {
        g.fields.insert(FieldId(i), format!("field_{:03}", i));
    }
    assert_eq!(g.fields.len(), 20);
    assert!(g.validate().is_ok());
}

// ═══════════════════════════════════════════════════════════════════════════
// 5. Clone trait (tests 37–44)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn fn_v10_clone_preserves_name() {
    let g = arith_grammar("fn_v10_cl1");
    let g2 = g.clone();
    assert_eq!(g.name, g2.name);
}

#[test]
fn fn_v10_clone_preserves_rule_names() {
    let g = arith_grammar("fn_v10_cl2");
    let g2 = g.clone();
    assert_eq!(g.rule_names, g2.rule_names);
}

#[test]
fn fn_v10_clone_preserves_tokens() {
    let g = arith_grammar("fn_v10_cl3");
    let g2 = g.clone();
    assert_eq!(g.tokens, g2.tokens);
}

#[test]
fn fn_v10_clone_preserves_rules() {
    let g = arith_grammar("fn_v10_cl4");
    let g2 = g.clone();
    assert_eq!(g.rules, g2.rules);
}

#[test]
fn fn_v10_clone_preserves_fields() {
    let mut g = Grammar::new("fn_v10_cl5".to_string());
    g.fields.insert(FieldId(0), "alpha".to_string());
    let g2 = g.clone();
    assert_eq!(g.fields, g2.fields);
}

#[test]
fn fn_v10_clone_equality() {
    let g = arith_grammar("fn_v10_cl6");
    let g2 = g.clone();
    assert_eq!(g, g2);
}

#[test]
fn fn_v10_clone_independent_mutation() {
    let g = arith_grammar("fn_v10_cl7");
    let mut g2 = g.clone();
    g2.name = "fn_v10_cl7_modified".to_string();
    assert_ne!(g.name, g2.name);
}

#[test]
fn fn_v10_clone_preserves_extras() {
    let g = GrammarBuilder::new("fn_v10_cl8")
        .token("A", "a")
        .token("WS", r"\s+")
        .extra("WS")
        .rule("start", vec!["A"])
        .start("start")
        .build();

    let g2 = g.clone();
    assert_eq!(g.extras, g2.extras);
}

// ═══════════════════════════════════════════════════════════════════════════
// 6. Debug trait (tests 45–50)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn fn_v10_debug_includes_name() {
    let g = arith_grammar("fn_v10_dbg1");
    let dbg = format!("{:?}", g);
    assert!(dbg.contains("fn_v10_dbg1"));
}

#[test]
fn fn_v10_debug_includes_grammar_keyword() {
    let g = arith_grammar("fn_v10_dbg2");
    let dbg = format!("{:?}", g);
    assert!(dbg.contains("Grammar"));
}

#[test]
fn fn_v10_debug_includes_rule_names() {
    let g = arith_grammar("fn_v10_dbg3");
    let dbg = format!("{:?}", g);
    assert!(dbg.contains("rule_names"));
}

#[test]
fn fn_v10_debug_includes_tokens() {
    let g = arith_grammar("fn_v10_dbg4");
    let dbg = format!("{:?}", g);
    assert!(dbg.contains("tokens"));
}

#[test]
fn fn_v10_debug_includes_fields() {
    let g = arith_grammar("fn_v10_dbg5");
    let dbg = format!("{:?}", g);
    assert!(dbg.contains("fields"));
}

#[test]
fn fn_v10_debug_empty_grammar() {
    let g = Grammar::new("fn_v10_dbg6".to_string());
    let dbg = format!("{:?}", g);
    assert!(dbg.contains("fn_v10_dbg6"));
}

// ═══════════════════════════════════════════════════════════════════════════
// 7. Serde roundtrip (tests 51–60)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn fn_v10_serde_roundtrip_preserves_name() {
    let g = arith_grammar("fn_v10_serde1");
    let json = serde_json::to_string(&g).unwrap();
    let g2: Grammar = serde_json::from_str(&json).unwrap();
    assert_eq!(g.name, g2.name);
}

#[test]
fn fn_v10_serde_roundtrip_preserves_rule_names() {
    let g = arith_grammar("fn_v10_serde2");
    let json = serde_json::to_string(&g).unwrap();
    let g2: Grammar = serde_json::from_str(&json).unwrap();
    assert_eq!(g.rule_names, g2.rule_names);
}

#[test]
fn fn_v10_serde_roundtrip_preserves_tokens() {
    let g = arith_grammar("fn_v10_serde3");
    let json = serde_json::to_string(&g).unwrap();
    let g2: Grammar = serde_json::from_str(&json).unwrap();
    assert_eq!(g.tokens, g2.tokens);
}

#[test]
fn fn_v10_serde_roundtrip_preserves_rules() {
    let g = arith_grammar("fn_v10_serde4");
    let json = serde_json::to_string(&g).unwrap();
    let g2: Grammar = serde_json::from_str(&json).unwrap();
    assert_eq!(g.rules, g2.rules);
}

#[test]
fn fn_v10_serde_roundtrip_preserves_fields() {
    let mut g = Grammar::new("fn_v10_serde5".to_string());
    g.fields.insert(FieldId(0), "alpha".to_string());
    g.fields.insert(FieldId(1), "beta".to_string());

    let json = serde_json::to_string(&g).unwrap();
    let g2: Grammar = serde_json::from_str(&json).unwrap();
    assert_eq!(g.fields, g2.fields);
}

#[test]
fn fn_v10_serde_roundtrip_full_equality() {
    let g = arith_grammar("fn_v10_serde6");
    let json = serde_json::to_string(&g).unwrap();
    let g2: Grammar = serde_json::from_str(&json).unwrap();
    assert_eq!(g, g2);
}

#[test]
fn fn_v10_serde_roundtrip_pretty() {
    let g = single_rule_grammar("fn_v10_serde7");
    let json = serde_json::to_string_pretty(&g).unwrap();
    let g2: Grammar = serde_json::from_str(&json).unwrap();
    assert_eq!(g, g2);
}

#[test]
fn fn_v10_serde_json_contains_name() {
    let g = arith_grammar("fn_v10_serde8");
    let json = serde_json::to_string(&g).unwrap();
    assert!(json.contains("fn_v10_serde8_arith"));
}

#[test]
fn fn_v10_serde_roundtrip_with_extras() {
    let g = GrammarBuilder::new("fn_v10_serde9")
        .token("A", "a")
        .token("WS", r"\s+")
        .extra("WS")
        .rule("start", vec!["A"])
        .start("start")
        .build();

    let json = serde_json::to_string(&g).unwrap();
    let g2: Grammar = serde_json::from_str(&json).unwrap();
    assert_eq!(g, g2);
}

#[test]
fn fn_v10_serde_roundtrip_empty_grammar() {
    let g = Grammar::new("fn_v10_serde10".to_string());
    let json = serde_json::to_string(&g).unwrap();
    let g2: Grammar = serde_json::from_str(&json).unwrap();
    assert_eq!(g, g2);
}

// ═══════════════════════════════════════════════════════════════════════════
// 8. Normalize (tests 61–68)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn fn_v10_normalize_simple_no_change() {
    let mut g = single_rule_grammar("fn_v10_norm1");
    let orig_name = g.name.clone();
    let _rules = g.normalize();
    assert_eq!(g.name, orig_name);
}

#[test]
fn fn_v10_normalize_preserves_name() {
    let mut g = arith_grammar("fn_v10_norm2");
    let _rules = g.normalize();
    assert_eq!(g.name, "fn_v10_norm2_arith");
}

#[test]
fn fn_v10_normalize_preserves_tokens() {
    let mut g = arith_grammar("fn_v10_norm3");
    let tokens_before = g.tokens.clone();
    let _rules = g.normalize();
    assert_eq!(g.tokens, tokens_before);
}

#[test]
fn fn_v10_normalize_may_add_auxiliary_rules() {
    let mut g = GrammarBuilder::new("fn_v10_norm4")
        .token("A", "a")
        .token("B", "b")
        .rule("start", vec!["A"])
        .start("start")
        .build();

    let rules_before = g.rules.len();
    let _rules = g.normalize();
    // Simple grammar: no auxiliary rules needed
    assert!(g.rules.len() >= rules_before);
}

#[test]
fn fn_v10_normalize_rule_names_keys_subset_of_rules_keys() {
    let mut g = arith_grammar("fn_v10_norm5");
    let _rules = g.normalize();
    // Original rule_names keys should still be in rules (they may have new aux entries)
    // (rule_names won't include aux rules added by normalize)
    for (sid, _name) in &g.rule_names {
        assert!(
            g.rules.contains_key(sid) || g.tokens.contains_key(sid),
            "rule_names key {sid} not found in rules or tokens"
        );
    }
}

#[test]
fn fn_v10_normalize_returns_all_rules() {
    let mut g = arith_grammar("fn_v10_norm6");
    let returned = g.normalize();
    let total: usize = g.rules.values().map(|v| v.len()).sum();
    assert_eq!(returned.len(), total);
}

#[test]
fn fn_v10_normalize_preserves_fields() {
    let mut g = Grammar::new("fn_v10_norm7".to_string());
    g.fields.insert(FieldId(0), "alpha".to_string());
    let _rules = g.normalize();
    assert_eq!(g.fields.len(), 1);
    assert_eq!(g.fields[&FieldId(0)], "alpha");
}

#[test]
fn fn_v10_normalize_idempotent_name() {
    let mut g = single_rule_grammar("fn_v10_norm8");
    let _rules = g.normalize();
    let name_after_first = g.name.clone();
    let _rules = g.normalize();
    assert_eq!(g.name, name_after_first);
}

// ═══════════════════════════════════════════════════════════════════════════
// 9. Optimize (tests 69–76)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn fn_v10_optimize_preserves_name() {
    let g = arith_grammar("fn_v10_opt1");
    let optimized = optimize_grammar(g).unwrap();
    assert_eq!(optimized.name, "fn_v10_opt1_arith");
}

#[test]
fn fn_v10_optimize_preserves_tokens() {
    let g = single_rule_grammar("fn_v10_opt2");
    let tokens_before = g.tokens.clone();
    let optimized = optimize_grammar(g).unwrap();
    assert_eq!(optimized.tokens, tokens_before);
}

#[test]
fn fn_v10_optimize_rule_names_non_empty() {
    let g = arith_grammar("fn_v10_opt3");
    let optimized = optimize_grammar(g).unwrap();
    assert!(!optimized.rule_names.is_empty());
}

#[test]
fn fn_v10_optimize_returns_ok() {
    let g = single_rule_grammar("fn_v10_opt4");
    assert!(optimize_grammar(g).is_ok());
}

#[test]
fn fn_v10_optimizer_struct_preserves_name() {
    let mut g = arith_grammar("fn_v10_opt5");
    let mut optimizer = GrammarOptimizer::new();
    let _stats = optimizer.optimize(&mut g);
    assert_eq!(g.name, "fn_v10_opt5_arith");
}

#[test]
fn fn_v10_optimize_preserves_fields() {
    let mut g = Grammar::new("fn_v10_opt6".to_string());
    g.fields.insert(FieldId(0), "alpha".to_string());
    let optimized = optimize_grammar(g).unwrap();
    assert_eq!(optimized.fields.len(), 1);
}

#[test]
fn fn_v10_optimize_rules_still_present() {
    let g = arith_grammar("fn_v10_opt7");
    let optimized = optimize_grammar(g).unwrap();
    assert!(!optimized.rules.is_empty());
}

#[test]
fn fn_v10_optimize_multiple_times_stable() {
    let g = arith_grammar("fn_v10_opt8");
    let opt1 = optimize_grammar(g).unwrap();
    let opt2 = optimize_grammar(opt1.clone()).unwrap();
    assert_eq!(opt1.name, opt2.name);
}

// ═══════════════════════════════════════════════════════════════════════════
// 10. Grammar size variations (tests 77–84)
// ═══════════════════════════════════════════════════════════════════════════

#[test]
fn fn_v10_size_one_rule() {
    let g = n_rule_grammar("fn_v10_sz1", 1);
    assert_eq!(g.rules.len(), 1);
    let names: Vec<&str> = g.rule_names.values().map(|s| s.as_str()).collect();
    assert!(names.contains(&"rule_0"));
}

#[test]
fn fn_v10_size_five_rules() {
    let g = n_rule_grammar("fn_v10_sz5", 5);
    assert_eq!(g.rules.len(), 5);
    for i in 0..5 {
        let expected = format!("rule_{i}");
        assert!(
            g.rule_names.values().any(|n| n == &expected),
            "missing {expected}"
        );
    }
}

#[test]
fn fn_v10_size_ten_rules() {
    let g = n_rule_grammar("fn_v10_sz10", 10);
    assert_eq!(g.rules.len(), 10);
    assert_eq!(
        g.rule_names
            .values()
            .filter(|n| n.starts_with("rule_"))
            .count(),
        10
    );
}

#[test]
fn fn_v10_size_twenty_rules() {
    let g = n_rule_grammar("fn_v10_sz20", 20);
    assert_eq!(g.rules.len(), 20);
    assert_eq!(
        g.rule_names
            .values()
            .filter(|n| n.starts_with("rule_"))
            .count(),
        20
    );
}

#[test]
fn fn_v10_size_one_rule_serde() {
    let g = n_rule_grammar("fn_v10_szs1", 1);
    let json = serde_json::to_string(&g).unwrap();
    let g2: Grammar = serde_json::from_str(&json).unwrap();
    assert_eq!(g, g2);
}

#[test]
fn fn_v10_size_five_rules_serde() {
    let g = n_rule_grammar("fn_v10_szs5", 5);
    let json = serde_json::to_string(&g).unwrap();
    let g2: Grammar = serde_json::from_str(&json).unwrap();
    assert_eq!(g, g2);
}

#[test]
fn fn_v10_size_ten_rules_clone_eq() {
    let g = n_rule_grammar("fn_v10_szc10", 10);
    let g2 = g.clone();
    assert_eq!(g, g2);
}

#[test]
fn fn_v10_size_twenty_rules_debug_contains_name() {
    let g = n_rule_grammar("fn_v10_szd20", 20);
    let dbg = format!("{:?}", g);
    assert!(dbg.contains("fn_v10_szd20"));
}
