//! Comprehensive tests for extras and externals handling in adze-ir Grammar.
//!
//! 80+ tests covering:
//!   ee_v10_extras_*      – extras (whitespace, comments)
//!   ee_v10_externals_*   – external scanner tokens
//!   ee_v10_combined_*    – extras + externals together
//!   ee_v10_normalize_*   – normalize preserves extras/externals
//!   ee_v10_optimize_*    – optimize preserves extras/externals
//!   ee_v10_clone_*       – clone preserves extras/externals
//!   ee_v10_debug_*       – Debug formatting for extras/externals
//!   ee_v10_serde_*       – serde roundtrip for extras/externals
//!   ee_v10_symid_*       – SymbolId validity for extras/externals
//!   ee_v10_validate_*    – validate with extras/externals

use adze_ir::builder::GrammarBuilder;
use adze_ir::{Grammar, SymbolId};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Build a small grammar with common tokens and a start rule.
fn base(name: &str) -> GrammarBuilder {
    GrammarBuilder::new(name)
        .token("ID", r"[a-z]+")
        .token("NUM", r"[0-9]+")
        .token(";", ";")
        .rule("program", vec!["stmt"])
        .start("program")
}

/// Resolve a token name to its SymbolId (searches tokens map).
fn tok(g: &Grammar, name: &str) -> SymbolId {
    g.tokens
        .iter()
        .find(|(_, t)| t.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("token `{name}` not found"))
}

/// Serde roundtrip helper.
fn roundtrip(g: &Grammar) -> Grammar {
    let json = serde_json::to_string(g).expect("serialize");
    serde_json::from_str(&json).expect("deserialize")
}

// ===========================================================================
// 1. No extras → extras empty (tests 1-4)
// ===========================================================================

#[test]
fn ee_v10_extras_empty_by_default() {
    let g = base("ee_v10_no_ext01")
        .rule("stmt", vec!["ID", ";"])
        .build();
    assert!(g.extras.is_empty());
}

#[test]
fn ee_v10_extras_empty_len_zero() {
    let g = base("ee_v10_no_ext02")
        .rule("stmt", vec!["ID", ";"])
        .build();
    assert_eq!(g.extras.len(), 0);
}

#[test]
fn ee_v10_extras_empty_grammar_new() {
    let g = Grammar::new("ee_v10_no_ext03".into());
    assert!(g.extras.is_empty());
}

#[test]
fn ee_v10_extras_empty_with_externals_only() {
    let g = base("ee_v10_no_ext04")
        .rule("stmt", vec!["ID", ";"])
        .external("INDENT")
        .build();
    assert!(g.extras.is_empty());
}

// ===========================================================================
// 2. One extra → extras has 1 (tests 5-9)
// ===========================================================================

#[test]
fn ee_v10_extras_single_len() {
    let g = base("ee_v10_one_ext01")
        .rule("stmt", vec!["ID", ";"])
        .token("WS", r"\s+")
        .extra("WS")
        .build();
    assert_eq!(g.extras.len(), 1);
}

#[test]
fn ee_v10_extras_single_resolves_to_token() {
    let g = base("ee_v10_one_ext02")
        .rule("stmt", vec!["ID", ";"])
        .token("WS", r"\s+")
        .extra("WS")
        .build();
    assert_eq!(g.extras[0], tok(&g, "WS"));
}

#[test]
fn ee_v10_extras_single_token_still_in_map() {
    let g = base("ee_v10_one_ext03")
        .rule("stmt", vec!["ID", ";"])
        .token("WS", r"\s+")
        .extra("WS")
        .build();
    assert!(g.tokens.contains_key(&g.extras[0]));
}

#[test]
fn ee_v10_extras_single_not_in_externals() {
    let g = base("ee_v10_one_ext04")
        .rule("stmt", vec!["ID", ";"])
        .token("WS", r"\s+")
        .extra("WS")
        .build();
    assert!(g.externals.is_empty());
}

#[test]
fn ee_v10_extras_single_grammar_name_preserved() {
    let g = base("ee_v10_one_ext05")
        .rule("stmt", vec!["ID", ";"])
        .token("WS", r"\s+")
        .extra("WS")
        .build();
    assert_eq!(g.name, "ee_v10_one_ext05");
}

// ===========================================================================
// 3. Multiple extras → all present (tests 10-15)
// ===========================================================================

#[test]
fn ee_v10_extras_two_len() {
    let g = base("ee_v10_mul_ext01")
        .rule("stmt", vec!["ID", ";"])
        .token("WS", r"\s+")
        .token("COMMENT", "//[^\n]*")
        .extra("WS")
        .extra("COMMENT")
        .build();
    assert_eq!(g.extras.len(), 2);
}

#[test]
fn ee_v10_extras_three_len() {
    let g = base("ee_v10_mul_ext02")
        .rule("stmt", vec!["ID", ";"])
        .token("WS", r"\s+")
        .token("COMMENT", "//[^\n]*")
        .token("BLOCK_COMMENT", r"/\*.*?\*/")
        .extra("WS")
        .extra("COMMENT")
        .extra("BLOCK_COMMENT")
        .build();
    assert_eq!(g.extras.len(), 3);
}

#[test]
fn ee_v10_extras_preserves_insertion_order() {
    let g = base("ee_v10_mul_ext03")
        .rule("stmt", vec!["ID", ";"])
        .token("WS", r"\s+")
        .token("COMMENT", "//[^\n]*")
        .extra("WS")
        .extra("COMMENT")
        .build();
    assert_eq!(g.extras[0], tok(&g, "WS"));
    assert_eq!(g.extras[1], tok(&g, "COMMENT"));
}

#[test]
fn ee_v10_extras_ids_are_distinct() {
    let g = base("ee_v10_mul_ext04")
        .rule("stmt", vec!["ID", ";"])
        .token("WS", r"\s+")
        .token("NL", r"\n")
        .extra("WS")
        .extra("NL")
        .build();
    assert_ne!(g.extras[0], g.extras[1]);
}

#[test]
fn ee_v10_extras_five() {
    let g = base("ee_v10_mul_ext05")
        .rule("stmt", vec!["ID", ";"])
        .token("E1", "a")
        .token("E2", "b")
        .token("E3", "c")
        .token("E4", "d")
        .token("E5", "e")
        .extra("E1")
        .extra("E2")
        .extra("E3")
        .extra("E4")
        .extra("E5")
        .build();
    assert_eq!(g.extras.len(), 5);
}

#[test]
fn ee_v10_extras_nonterminal_as_extra() {
    let g = base("ee_v10_mul_ext06")
        .rule("stmt", vec!["ID", ";"])
        .rule("comment", vec!["ID"])
        .extra("comment")
        .build();
    assert_eq!(g.extras.len(), 1);
}

// ===========================================================================
// 4. No externals → externals empty (tests 16-19)
// ===========================================================================

#[test]
fn ee_v10_externals_empty_by_default() {
    let g = base("ee_v10_no_xtn01")
        .rule("stmt", vec!["ID", ";"])
        .build();
    assert!(g.externals.is_empty());
}

#[test]
fn ee_v10_externals_empty_len_zero() {
    let g = base("ee_v10_no_xtn02")
        .rule("stmt", vec!["ID", ";"])
        .build();
    assert_eq!(g.externals.len(), 0);
}

#[test]
fn ee_v10_externals_empty_grammar_new() {
    let g = Grammar::new("ee_v10_no_xtn03".into());
    assert!(g.externals.is_empty());
}

#[test]
fn ee_v10_externals_empty_with_extras_only() {
    let g = base("ee_v10_no_xtn04")
        .rule("stmt", vec!["ID", ";"])
        .token("WS", r"\s+")
        .extra("WS")
        .build();
    assert!(g.externals.is_empty());
}

// ===========================================================================
// 5. One external → externals has 1 (tests 20-24)
// ===========================================================================

#[test]
fn ee_v10_externals_single_len() {
    let g = base("ee_v10_one_xtn01")
        .rule("stmt", vec!["ID", ";"])
        .external("INDENT")
        .build();
    assert_eq!(g.externals.len(), 1);
}

#[test]
fn ee_v10_externals_single_name() {
    let g = base("ee_v10_one_xtn02")
        .rule("stmt", vec!["ID", ";"])
        .external("INDENT")
        .build();
    assert_eq!(g.externals[0].name, "INDENT");
}

#[test]
fn ee_v10_externals_single_not_in_tokens() {
    let g = base("ee_v10_one_xtn03")
        .rule("stmt", vec!["ID", ";"])
        .external("INDENT")
        .build();
    let token_names: Vec<&str> = g.tokens.values().map(|t| t.name.as_str()).collect();
    assert!(!token_names.contains(&"INDENT"));
}

#[test]
fn ee_v10_externals_single_symbol_id_not_in_tokens() {
    let g = base("ee_v10_one_xtn04")
        .rule("stmt", vec!["ID", ";"])
        .external("INDENT")
        .build();
    let ext_id = g.externals[0].symbol_id;
    assert!(!g.tokens.contains_key(&ext_id));
}

#[test]
fn ee_v10_externals_single_grammar_name_preserved() {
    let g = base("ee_v10_one_xtn05")
        .rule("stmt", vec!["ID", ";"])
        .external("INDENT")
        .build();
    assert_eq!(g.name, "ee_v10_one_xtn05");
}

// ===========================================================================
// 6. Multiple externals → all present (tests 25-30)
// ===========================================================================

#[test]
fn ee_v10_externals_two_len() {
    let g = base("ee_v10_mul_xtn01")
        .rule("stmt", vec!["ID", ";"])
        .external("INDENT")
        .external("DEDENT")
        .build();
    assert_eq!(g.externals.len(), 2);
}

#[test]
fn ee_v10_externals_three_names() {
    let g = base("ee_v10_mul_xtn02")
        .rule("stmt", vec!["ID", ";"])
        .external("INDENT")
        .external("DEDENT")
        .external("NEWLINE")
        .build();
    let names: Vec<&str> = g.externals.iter().map(|e| e.name.as_str()).collect();
    assert_eq!(names, vec!["INDENT", "DEDENT", "NEWLINE"]);
}

#[test]
fn ee_v10_externals_preserves_insertion_order() {
    let g = base("ee_v10_mul_xtn03")
        .rule("stmt", vec!["ID", ";"])
        .external("ZZZ")
        .external("AAA")
        .external("MMM")
        .build();
    let names: Vec<&str> = g.externals.iter().map(|e| e.name.as_str()).collect();
    assert_eq!(names, vec!["ZZZ", "AAA", "MMM"]);
}

#[test]
fn ee_v10_externals_unique_symbol_ids() {
    let g = base("ee_v10_mul_xtn04")
        .rule("stmt", vec!["ID", ";"])
        .external("INDENT")
        .external("DEDENT")
        .build();
    assert_ne!(g.externals[0].symbol_id, g.externals[1].symbol_id);
}

#[test]
fn ee_v10_externals_five() {
    let g = base("ee_v10_mul_xtn05")
        .rule("stmt", vec!["ID", ";"])
        .external("E1")
        .external("E2")
        .external("E3")
        .external("E4")
        .external("E5")
        .build();
    assert_eq!(g.externals.len(), 5);
}

#[test]
fn ee_v10_externals_different_names() {
    let g = base("ee_v10_mul_xtn06")
        .rule("stmt", vec!["ID", ";"])
        .external("HEREDOC_START")
        .external("HEREDOC_END")
        .build();
    assert_ne!(g.externals[0].name, g.externals[1].name);
}

// ===========================================================================
// 7. Extras + externals on same grammar (tests 31-37)
// ===========================================================================

#[test]
fn ee_v10_combined_both_present() {
    let g = base("ee_v10_comb01")
        .rule("stmt", vec!["ID", ";"])
        .token("WS", r"\s+")
        .external("INDENT")
        .extra("WS")
        .build();
    assert_eq!(g.externals.len(), 1);
    assert_eq!(g.extras.len(), 1);
}

#[test]
fn ee_v10_combined_multiple_of_each() {
    let g = base("ee_v10_comb02")
        .rule("stmt", vec!["ID", ";"])
        .token("WS", r"\s+")
        .token("COMMENT", "//[^\n]*")
        .external("INDENT")
        .external("DEDENT")
        .extra("WS")
        .extra("COMMENT")
        .build();
    assert_eq!(g.externals.len(), 2);
    assert_eq!(g.extras.len(), 2);
}

#[test]
fn ee_v10_combined_separate_symbol_ids() {
    let g = base("ee_v10_comb03")
        .rule("stmt", vec!["ID", ";"])
        .token("WS", r"\s+")
        .external("INDENT")
        .extra("WS")
        .build();
    let ext_id = g.externals[0].symbol_id;
    let extra_id = g.extras[0];
    assert_ne!(ext_id, extra_id);
}

#[test]
fn ee_v10_combined_rules_still_present() {
    let g = base("ee_v10_comb04")
        .rule("stmt", vec!["ID", ";"])
        .token("WS", r"\s+")
        .external("INDENT")
        .extra("WS")
        .build();
    assert!(!g.rules.is_empty());
}

#[test]
fn ee_v10_combined_tokens_still_present() {
    let g = base("ee_v10_comb05")
        .rule("stmt", vec!["ID", ";"])
        .token("WS", r"\s+")
        .external("INDENT")
        .extra("WS")
        .build();
    assert!(!g.tokens.is_empty());
}

#[test]
fn ee_v10_combined_extras_not_in_externals() {
    let g = base("ee_v10_comb06")
        .rule("stmt", vec!["ID", ";"])
        .token("WS", r"\s+")
        .external("INDENT")
        .extra("WS")
        .build();
    let extra_id = g.extras[0];
    let ext_ids: Vec<SymbolId> = g.externals.iter().map(|e| e.symbol_id).collect();
    assert!(!ext_ids.contains(&extra_id));
}

#[test]
fn ee_v10_combined_complex_grammar() {
    let g = base("ee_v10_comb07")
        .rule("stmt", vec!["ID", ";"])
        .rule("expr", vec!["NUM"])
        .token("WS", r"\s+")
        .token("COMMENT", "//[^\n]*")
        .token("NL", r"\n")
        .external("INDENT")
        .external("DEDENT")
        .external("NEWLINE")
        .extra("WS")
        .extra("COMMENT")
        .extra("NL")
        .build();
    assert_eq!(g.externals.len(), 3);
    assert_eq!(g.extras.len(), 3);
    assert_eq!(g.name, "ee_v10_comb07");
}

// ===========================================================================
// 8. Extras preserved after normalize (tests 38-42)
// ===========================================================================

#[test]
fn ee_v10_normalize_preserves_extras_single() {
    let mut g = base("ee_v10_norm_ext01")
        .rule("stmt", vec!["ID", ";"])
        .token("WS", r"\s+")
        .extra("WS")
        .build();
    g.normalize();
    assert_eq!(g.extras.len(), 1);
}

#[test]
fn ee_v10_normalize_preserves_extras_value() {
    let mut g = base("ee_v10_norm_ext02")
        .rule("stmt", vec!["ID", ";"])
        .token("WS", r"\s+")
        .extra("WS")
        .build();
    let before = g.extras[0];
    g.normalize();
    assert_eq!(g.extras[0], before);
}

#[test]
fn ee_v10_normalize_preserves_extras_multiple() {
    let mut g = base("ee_v10_norm_ext03")
        .rule("stmt", vec!["ID", ";"])
        .token("WS", r"\s+")
        .token("COMMENT", "//[^\n]*")
        .extra("WS")
        .extra("COMMENT")
        .build();
    g.normalize();
    assert_eq!(g.extras.len(), 2);
}

#[test]
fn ee_v10_normalize_preserves_extras_order() {
    let mut g = base("ee_v10_norm_ext04")
        .rule("stmt", vec!["ID", ";"])
        .token("WS", r"\s+")
        .token("COMMENT", "//[^\n]*")
        .extra("WS")
        .extra("COMMENT")
        .build();
    let before: Vec<SymbolId> = g.extras.clone();
    g.normalize();
    assert_eq!(g.extras, before);
}

#[test]
fn ee_v10_normalize_preserves_extras_with_complex_rules() {
    let mut g = base("ee_v10_norm_ext05")
        .rule("stmt", vec!["expr"])
        .rule("expr", vec!["ID"])
        .rule("expr", vec!["NUM"])
        .token("WS", r"\s+")
        .extra("WS")
        .build();
    g.normalize();
    assert_eq!(g.extras.len(), 1);
}

// ===========================================================================
// 9. Extras preserved after optimize (tests 43-47)
// ===========================================================================

#[test]
fn ee_v10_optimize_preserves_extras_single() {
    let mut g = base("ee_v10_opt_ext01")
        .rule("stmt", vec!["ID", ";"])
        .token("WS", r"\s+")
        .extra("WS")
        .build();
    g.optimize();
    assert_eq!(g.extras.len(), 1);
}

#[test]
fn ee_v10_optimize_preserves_extras_value() {
    let mut g = base("ee_v10_opt_ext02")
        .rule("stmt", vec!["ID", ";"])
        .token("WS", r"\s+")
        .extra("WS")
        .build();
    let before = g.extras[0];
    g.optimize();
    assert_eq!(g.extras[0], before);
}

#[test]
fn ee_v10_optimize_preserves_extras_multiple() {
    let mut g = base("ee_v10_opt_ext03")
        .rule("stmt", vec!["ID", ";"])
        .token("WS", r"\s+")
        .token("COMMENT", "//[^\n]*")
        .extra("WS")
        .extra("COMMENT")
        .build();
    g.optimize();
    assert_eq!(g.extras.len(), 2);
}

#[test]
fn ee_v10_optimize_preserves_extras_order() {
    let mut g = base("ee_v10_opt_ext04")
        .rule("stmt", vec!["ID", ";"])
        .token("WS", r"\s+")
        .token("COMMENT", "//[^\n]*")
        .extra("WS")
        .extra("COMMENT")
        .build();
    let before: Vec<SymbolId> = g.extras.clone();
    g.optimize();
    assert_eq!(g.extras, before);
}

#[test]
fn ee_v10_optimize_preserves_extras_empty() {
    let mut g = base("ee_v10_opt_ext05")
        .rule("stmt", vec!["ID", ";"])
        .build();
    g.optimize();
    assert!(g.extras.is_empty());
}

// ===========================================================================
// 10. Externals preserved after normalize (tests 48-52)
// ===========================================================================

#[test]
fn ee_v10_normalize_preserves_externals_single() {
    let mut g = base("ee_v10_norm_xtn01")
        .rule("stmt", vec!["ID", ";"])
        .external("INDENT")
        .build();
    g.normalize();
    assert_eq!(g.externals.len(), 1);
}

#[test]
fn ee_v10_normalize_preserves_externals_name() {
    let mut g = base("ee_v10_norm_xtn02")
        .rule("stmt", vec!["ID", ";"])
        .external("INDENT")
        .build();
    g.normalize();
    assert_eq!(g.externals[0].name, "INDENT");
}

#[test]
fn ee_v10_normalize_preserves_externals_multiple() {
    let mut g = base("ee_v10_norm_xtn03")
        .rule("stmt", vec!["ID", ";"])
        .external("INDENT")
        .external("DEDENT")
        .build();
    g.normalize();
    assert_eq!(g.externals.len(), 2);
}

#[test]
fn ee_v10_normalize_preserves_externals_order() {
    let mut g = base("ee_v10_norm_xtn04")
        .rule("stmt", vec!["ID", ";"])
        .external("INDENT")
        .external("DEDENT")
        .build();
    g.normalize();
    assert_eq!(g.externals[0].name, "INDENT");
    assert_eq!(g.externals[1].name, "DEDENT");
}

#[test]
fn ee_v10_normalize_preserves_externals_symbol_ids() {
    let mut g = base("ee_v10_norm_xtn05")
        .rule("stmt", vec!["ID", ";"])
        .external("INDENT")
        .build();
    let before = g.externals[0].symbol_id;
    g.normalize();
    assert_eq!(g.externals[0].symbol_id, before);
}

// ===========================================================================
// 11. Externals preserved after optimize (tests 53-57)
// ===========================================================================

#[test]
fn ee_v10_optimize_preserves_externals_single() {
    let mut g = base("ee_v10_opt_xtn01")
        .rule("stmt", vec!["ID", ";"])
        .external("INDENT")
        .build();
    g.optimize();
    assert_eq!(g.externals.len(), 1);
}

#[test]
fn ee_v10_optimize_preserves_externals_name() {
    let mut g = base("ee_v10_opt_xtn02")
        .rule("stmt", vec!["ID", ";"])
        .external("INDENT")
        .build();
    g.optimize();
    assert_eq!(g.externals[0].name, "INDENT");
}

#[test]
fn ee_v10_optimize_preserves_externals_multiple() {
    let mut g = base("ee_v10_opt_xtn03")
        .rule("stmt", vec!["ID", ";"])
        .external("INDENT")
        .external("DEDENT")
        .build();
    g.optimize();
    assert_eq!(g.externals.len(), 2);
}

#[test]
fn ee_v10_optimize_preserves_externals_symbol_ids() {
    let mut g = base("ee_v10_opt_xtn04")
        .rule("stmt", vec!["ID", ";"])
        .external("INDENT")
        .build();
    let before = g.externals[0].symbol_id;
    g.optimize();
    assert_eq!(g.externals[0].symbol_id, before);
}

#[test]
fn ee_v10_optimize_preserves_externals_empty() {
    let mut g = base("ee_v10_opt_xtn05")
        .rule("stmt", vec!["ID", ";"])
        .build();
    g.optimize();
    assert!(g.externals.is_empty());
}

// ===========================================================================
// 12. Clone preserves extras (tests 58-61)
// ===========================================================================

#[test]
fn ee_v10_clone_extras_single() {
    let g = base("ee_v10_cln_ext01")
        .rule("stmt", vec!["ID", ";"])
        .token("WS", r"\s+")
        .extra("WS")
        .build();
    let g2 = g.clone();
    assert_eq!(g2.extras.len(), g.extras.len());
    assert_eq!(g2.extras[0], g.extras[0]);
}

#[test]
fn ee_v10_clone_extras_multiple() {
    let g = base("ee_v10_cln_ext02")
        .rule("stmt", vec!["ID", ";"])
        .token("WS", r"\s+")
        .token("COMMENT", "//[^\n]*")
        .extra("WS")
        .extra("COMMENT")
        .build();
    let g2 = g.clone();
    assert_eq!(g2.extras, g.extras);
}

#[test]
fn ee_v10_clone_extras_independent() {
    let g = base("ee_v10_cln_ext03")
        .rule("stmt", vec!["ID", ";"])
        .token("WS", r"\s+")
        .extra("WS")
        .build();
    let mut g2 = g.clone();
    g2.extras.clear();
    assert_eq!(g.extras.len(), 1);
    assert!(g2.extras.is_empty());
}

#[test]
fn ee_v10_clone_extras_empty() {
    let g = base("ee_v10_cln_ext04")
        .rule("stmt", vec!["ID", ";"])
        .build();
    let g2 = g.clone();
    assert!(g2.extras.is_empty());
}

// ===========================================================================
// 13. Clone preserves externals (tests 62-65)
// ===========================================================================

#[test]
fn ee_v10_clone_externals_single() {
    let g = base("ee_v10_cln_xtn01")
        .rule("stmt", vec!["ID", ";"])
        .external("INDENT")
        .build();
    let g2 = g.clone();
    assert_eq!(g2.externals.len(), 1);
    assert_eq!(g2.externals[0].name, "INDENT");
}

#[test]
fn ee_v10_clone_externals_multiple() {
    let g = base("ee_v10_cln_xtn02")
        .rule("stmt", vec!["ID", ";"])
        .external("INDENT")
        .external("DEDENT")
        .build();
    let g2 = g.clone();
    assert_eq!(g2.externals.len(), 2);
    assert_eq!(g2.externals[0].name, g.externals[0].name);
    assert_eq!(g2.externals[1].name, g.externals[1].name);
}

#[test]
fn ee_v10_clone_externals_symbol_ids_match() {
    let g = base("ee_v10_cln_xtn03")
        .rule("stmt", vec!["ID", ";"])
        .external("INDENT")
        .external("DEDENT")
        .build();
    let g2 = g.clone();
    assert_eq!(g2.externals[0].symbol_id, g.externals[0].symbol_id);
    assert_eq!(g2.externals[1].symbol_id, g.externals[1].symbol_id);
}

#[test]
fn ee_v10_clone_externals_independent() {
    let g = base("ee_v10_cln_xtn04")
        .rule("stmt", vec!["ID", ";"])
        .external("INDENT")
        .build();
    let mut g2 = g.clone();
    g2.externals.clear();
    assert_eq!(g.externals.len(), 1);
    assert!(g2.externals.is_empty());
}

// ===========================================================================
// 14. Debug includes extras info (tests 66-68)
// ===========================================================================

#[test]
fn ee_v10_debug_extras_field_present() {
    let g = base("ee_v10_dbg_ext01")
        .rule("stmt", vec!["ID", ";"])
        .token("WS", r"\s+")
        .extra("WS")
        .build();
    let dbg = format!("{g:?}");
    assert!(dbg.contains("extras"));
}

#[test]
fn ee_v10_debug_extras_empty_visible() {
    let g = base("ee_v10_dbg_ext02")
        .rule("stmt", vec!["ID", ";"])
        .build();
    let dbg = format!("{g:?}");
    assert!(dbg.contains("extras"));
}

#[test]
fn ee_v10_debug_extras_name_present() {
    let g = base("ee_v10_dbg_ext03")
        .rule("stmt", vec!["ID", ";"])
        .build();
    let dbg = format!("{g:?}");
    assert!(dbg.contains("ee_v10_dbg_ext03"));
}

// ===========================================================================
// 15. Debug includes externals info (tests 69-71)
// ===========================================================================

#[test]
fn ee_v10_debug_externals_field_present() {
    let g = base("ee_v10_dbg_xtn01")
        .rule("stmt", vec!["ID", ";"])
        .external("INDENT")
        .build();
    let dbg = format!("{g:?}");
    assert!(dbg.contains("externals"));
}

#[test]
fn ee_v10_debug_externals_name_in_output() {
    let g = base("ee_v10_dbg_xtn02")
        .rule("stmt", vec!["ID", ";"])
        .external("INDENT")
        .build();
    let dbg = format!("{g:?}");
    assert!(dbg.contains("INDENT"));
}

#[test]
fn ee_v10_debug_external_token_struct() {
    let et = adze_ir::ExternalToken {
        name: "HEREDOC".into(),
        symbol_id: SymbolId(99),
    };
    let dbg = format!("{et:?}");
    assert!(dbg.contains("HEREDOC"));
    assert!(dbg.contains("99"));
}

// ===========================================================================
// 16. Serde roundtrip preserves extras (tests 72-75)
// ===========================================================================

#[test]
fn ee_v10_serde_extras_single() {
    let g = base("ee_v10_ser_ext01")
        .rule("stmt", vec!["ID", ";"])
        .token("WS", r"\s+")
        .extra("WS")
        .build();
    let g2 = roundtrip(&g);
    assert_eq!(g2.extras, g.extras);
}

#[test]
fn ee_v10_serde_extras_multiple() {
    let g = base("ee_v10_ser_ext02")
        .rule("stmt", vec!["ID", ";"])
        .token("WS", r"\s+")
        .token("COMMENT", "//[^\n]*")
        .extra("WS")
        .extra("COMMENT")
        .build();
    let g2 = roundtrip(&g);
    assert_eq!(g2.extras.len(), 2);
    assert_eq!(g2.extras, g.extras);
}

#[test]
fn ee_v10_serde_extras_empty() {
    let g = base("ee_v10_ser_ext03")
        .rule("stmt", vec!["ID", ";"])
        .build();
    let g2 = roundtrip(&g);
    assert!(g2.extras.is_empty());
}

#[test]
fn ee_v10_serde_extras_order_preserved() {
    let g = base("ee_v10_ser_ext04")
        .rule("stmt", vec!["ID", ";"])
        .token("WS", r"\s+")
        .token("COMMENT", "//[^\n]*")
        .token("NL", r"\n")
        .extra("WS")
        .extra("COMMENT")
        .extra("NL")
        .build();
    let g2 = roundtrip(&g);
    assert_eq!(g2.extras, g.extras);
}

// ===========================================================================
// 17. Serde roundtrip preserves externals (tests 76-79)
// ===========================================================================

#[test]
fn ee_v10_serde_externals_single() {
    let g = base("ee_v10_ser_xtn01")
        .rule("stmt", vec!["ID", ";"])
        .external("INDENT")
        .build();
    let g2 = roundtrip(&g);
    assert_eq!(g2.externals.len(), 1);
    assert_eq!(g2.externals[0].name, "INDENT");
    assert_eq!(g2.externals[0].symbol_id, g.externals[0].symbol_id);
}

#[test]
fn ee_v10_serde_externals_multiple() {
    let g = base("ee_v10_ser_xtn02")
        .rule("stmt", vec!["ID", ";"])
        .external("INDENT")
        .external("DEDENT")
        .external("NEWLINE")
        .build();
    let g2 = roundtrip(&g);
    assert_eq!(g2.externals.len(), 3);
    for i in 0..3 {
        assert_eq!(g2.externals[i].name, g.externals[i].name);
        assert_eq!(g2.externals[i].symbol_id, g.externals[i].symbol_id);
    }
}

#[test]
fn ee_v10_serde_externals_empty() {
    let g = base("ee_v10_ser_xtn03")
        .rule("stmt", vec!["ID", ";"])
        .build();
    let g2 = roundtrip(&g);
    assert!(g2.externals.is_empty());
}

#[test]
fn ee_v10_serde_externals_order_preserved() {
    let g = base("ee_v10_ser_xtn04")
        .rule("stmt", vec!["ID", ";"])
        .external("ZZZ")
        .external("AAA")
        .external("MMM")
        .build();
    let g2 = roundtrip(&g);
    let names: Vec<&str> = g2.externals.iter().map(|e| e.name.as_str()).collect();
    assert_eq!(names, vec!["ZZZ", "AAA", "MMM"]);
}

// ===========================================================================
// 18. Extra is valid SymbolId (tests 80-82)
// ===========================================================================

#[test]
fn ee_v10_symid_extra_exists_in_tokens() {
    let g = base("ee_v10_sid_ext01")
        .rule("stmt", vec!["ID", ";"])
        .token("WS", r"\s+")
        .extra("WS")
        .build();
    let ws_id = g.extras[0];
    assert!(g.tokens.contains_key(&ws_id) || g.rule_names.contains_key(&ws_id));
}

#[test]
fn ee_v10_symid_extra_not_max() {
    let g = base("ee_v10_sid_ext02")
        .rule("stmt", vec!["ID", ";"])
        .token("WS", r"\s+")
        .extra("WS")
        .build();
    assert_ne!(g.extras[0], SymbolId(u16::MAX));
}

#[test]
fn ee_v10_symid_extras_all_valid() {
    let g = base("ee_v10_sid_ext03")
        .rule("stmt", vec!["ID", ";"])
        .token("WS", r"\s+")
        .token("COMMENT", "//[^\n]*")
        .extra("WS")
        .extra("COMMENT")
        .build();
    for extra_id in &g.extras {
        assert!(g.tokens.contains_key(extra_id) || g.rule_names.contains_key(extra_id));
    }
}

// ===========================================================================
// 19. External is valid SymbolId (tests 83-85)
// ===========================================================================

#[test]
fn ee_v10_symid_external_not_max() {
    let g = base("ee_v10_sid_xtn01")
        .rule("stmt", vec!["ID", ";"])
        .external("INDENT")
        .build();
    assert_ne!(g.externals[0].symbol_id, SymbolId(u16::MAX));
}

#[test]
fn ee_v10_symid_external_is_copy() {
    let g = base("ee_v10_sid_xtn02")
        .rule("stmt", vec!["ID", ";"])
        .external("INDENT")
        .build();
    let id1 = g.externals[0].symbol_id;
    let id2 = g.externals[0].symbol_id;
    assert_eq!(id1, id2);
}

#[test]
fn ee_v10_symid_externals_all_distinct() {
    let g = base("ee_v10_sid_xtn03")
        .rule("stmt", vec!["ID", ";"])
        .external("INDENT")
        .external("DEDENT")
        .external("NEWLINE")
        .build();
    let ids: Vec<SymbolId> = g.externals.iter().map(|e| e.symbol_id).collect();
    for i in 0..ids.len() {
        for j in (i + 1)..ids.len() {
            assert_ne!(ids[i], ids[j], "external ids at {i} and {j} should differ");
        }
    }
}

// ===========================================================================
// 20. Validate with extras/externals → Ok (tests 86-90)
// ===========================================================================

#[test]
fn ee_v10_validate_with_extras() {
    let g = base("ee_v10_val01")
        .rule("stmt", vec!["ID", ";"])
        .token("WS", r"\s+")
        .extra("WS")
        .build();
    assert!(g.validate().is_ok());
}

#[test]
fn ee_v10_validate_with_externals() {
    let g = base("ee_v10_val02")
        .rule("stmt", vec!["ID", ";"])
        .external("INDENT")
        .build();
    assert!(g.validate().is_ok());
}

#[test]
fn ee_v10_validate_with_both() {
    let g = base("ee_v10_val03")
        .rule("stmt", vec!["ID", ";"])
        .token("WS", r"\s+")
        .external("INDENT")
        .extra("WS")
        .build();
    assert!(g.validate().is_ok());
}

#[test]
fn ee_v10_validate_with_multiple_extras_and_externals() {
    let g = base("ee_v10_val04")
        .rule("stmt", vec!["ID", ";"])
        .token("WS", r"\s+")
        .token("COMMENT", "//[^\n]*")
        .external("INDENT")
        .external("DEDENT")
        .extra("WS")
        .extra("COMMENT")
        .build();
    assert!(g.validate().is_ok());
}

#[test]
fn ee_v10_validate_after_normalize_with_extras() {
    let mut g = base("ee_v10_val05")
        .rule("stmt", vec!["ID", ";"])
        .token("WS", r"\s+")
        .extra("WS")
        .external("INDENT")
        .build();
    g.normalize();
    assert!(g.validate().is_ok());
}

// ===========================================================================
// Bonus: additional coverage (tests 91+)
// ===========================================================================

#[test]
fn ee_v10_serde_roundtrip_both_extras_and_externals() {
    let g = base("ee_v10_bonus01")
        .rule("stmt", vec!["ID", ";"])
        .token("WS", r"\s+")
        .token("COMMENT", "//[^\n]*")
        .external("INDENT")
        .external("DEDENT")
        .extra("WS")
        .extra("COMMENT")
        .build();
    let g2 = roundtrip(&g);
    assert_eq!(g2.extras, g.extras);
    assert_eq!(g2.externals.len(), g.externals.len());
    assert_eq!(g2.externals[0].name, g.externals[0].name);
    assert_eq!(g2.externals[1].name, g.externals[1].name);
    assert_eq!(g2.name, g.name);
}

#[test]
fn ee_v10_normalize_then_optimize_preserves_extras() {
    let mut g = base("ee_v10_bonus02")
        .rule("stmt", vec!["ID", ";"])
        .token("WS", r"\s+")
        .extra("WS")
        .build();
    let before: Vec<SymbolId> = g.extras.clone();
    g.normalize();
    g.optimize();
    assert_eq!(g.extras, before);
}

#[test]
fn ee_v10_normalize_then_optimize_preserves_externals() {
    let mut g = base("ee_v10_bonus03")
        .rule("stmt", vec!["ID", ";"])
        .external("INDENT")
        .build();
    let before_name = g.externals[0].name.clone();
    let before_id = g.externals[0].symbol_id;
    g.normalize();
    g.optimize();
    assert_eq!(g.externals.len(), 1);
    assert_eq!(g.externals[0].name, before_name);
    assert_eq!(g.externals[0].symbol_id, before_id);
}

#[test]
fn ee_v10_many_externals_loop() {
    let mut builder = base("ee_v10_bonus04").rule("stmt", vec!["ID", ";"]);
    for i in 0..20 {
        builder = builder.external(&format!("EXT_{i}"));
    }
    let g = builder.build();
    assert_eq!(g.externals.len(), 20);
}

#[test]
fn ee_v10_many_extras_loop() {
    let mut builder = base("ee_v10_bonus05").rule("stmt", vec!["ID", ";"]);
    for i in 0..10 {
        let name = format!("TOK_{i}");
        builder = builder.token(&name, &format!("t{i}"));
    }
    for i in 0..10 {
        let name = format!("TOK_{i}");
        builder = builder.extra(&name);
    }
    let g = builder.build();
    assert_eq!(g.extras.len(), 10);
}

#[test]
fn ee_v10_clone_then_serde_extras() {
    let g = base("ee_v10_bonus06")
        .rule("stmt", vec!["ID", ";"])
        .token("WS", r"\s+")
        .extra("WS")
        .build();
    let cloned = g.clone();
    let g2 = roundtrip(&cloned);
    assert_eq!(g2.extras, g.extras);
}

#[test]
fn ee_v10_clone_then_serde_externals() {
    let g = base("ee_v10_bonus07")
        .rule("stmt", vec!["ID", ";"])
        .external("INDENT")
        .build();
    let cloned = g.clone();
    let g2 = roundtrip(&cloned);
    assert_eq!(g2.externals.len(), 1);
    assert_eq!(g2.externals[0].name, "INDENT");
    assert_eq!(g2.externals[0].symbol_id, g.externals[0].symbol_id);
}

#[test]
fn ee_v10_serde_json_extras_field_exists() {
    let g = base("ee_v10_bonus08")
        .rule("stmt", vec!["ID", ";"])
        .token("WS", r"\s+")
        .extra("WS")
        .build();
    let json = serde_json::to_string(&g).expect("serialize");
    assert!(json.contains("\"extras\""));
}

#[test]
fn ee_v10_serde_json_externals_field_exists() {
    let g = base("ee_v10_bonus09")
        .rule("stmt", vec!["ID", ";"])
        .external("INDENT")
        .build();
    let json = serde_json::to_string(&g).expect("serialize");
    assert!(json.contains("\"externals\""));
}

#[test]
fn ee_v10_extras_does_not_affect_inline_rules() {
    let g = base("ee_v10_bonus10")
        .rule("stmt", vec!["ID", ";"])
        .token("WS", r"\s+")
        .extra("WS")
        .build();
    assert!(g.inline_rules.is_empty());
}

#[test]
fn ee_v10_extras_does_not_affect_supertypes() {
    let g = base("ee_v10_bonus11")
        .rule("stmt", vec!["ID", ";"])
        .token("WS", r"\s+")
        .extra("WS")
        .build();
    assert!(g.supertypes.is_empty());
}

#[test]
fn ee_v10_externals_does_not_affect_inline_rules() {
    let g = base("ee_v10_bonus12")
        .rule("stmt", vec!["ID", ";"])
        .external("INDENT")
        .build();
    assert!(g.inline_rules.is_empty());
}

#[test]
fn ee_v10_externals_does_not_affect_supertypes() {
    let g = base("ee_v10_bonus13")
        .rule("stmt", vec!["ID", ";"])
        .external("INDENT")
        .build();
    assert!(g.supertypes.is_empty());
}
