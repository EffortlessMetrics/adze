//! Comprehensive tests for externals, extras, and inline_rules in adze-ir Grammar.
//!
//! 80+ tests covering:
//!   ee_v10_ext_*        – external token behaviour
//!   ee_v10_extra_*      – extras (whitespace, comments)
//!   ee_v10_inline_*     – inline rules
//!   ee_v10_super_*      – supertypes
//!   ee_v10_conflict_*   – conflict declarations
//!   ee_v10_clone_*      – clone preserves all fields
//!   ee_v10_norm_*       – normalize/optimize don't crash
//!   ee_v10_combo_*      – combined features
//!   ee_v10_debug_*      – Debug formatting
//!   ee_v10_edge_*       – edge cases

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

/// Resolve a symbol name to its SymbolId (searches rule_names).
fn sym(g: &Grammar, name: &str) -> SymbolId {
    g.find_symbol_by_name(name)
        .unwrap_or_else(|| panic!("symbol `{name}` not found"))
}

/// Resolve a token name to its SymbolId (searches tokens map).
fn tok(g: &Grammar, name: &str) -> SymbolId {
    g.tokens
        .iter()
        .find(|(_, t)| t.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("token `{name}` not found"))
}

// ===========================================================================
// 1. ee_v10_ext_* – external tokens (15 tests)
// ===========================================================================

#[test]
fn ee_v10_ext_empty_by_default() {
    let g = base("ee_v10_ext01").rule("stmt", vec!["ID", ";"]).build();
    assert!(g.externals.is_empty());
}

#[test]
fn ee_v10_ext_single() {
    let g = base("ee_v10_ext02")
        .rule("stmt", vec!["ID", ";"])
        .external("INDENT")
        .build();
    assert_eq!(g.externals.len(), 1);
}

#[test]
fn ee_v10_ext_single_name() {
    let g = base("ee_v10_ext03")
        .rule("stmt", vec!["ID", ";"])
        .external("INDENT")
        .build();
    assert_eq!(g.externals[0].name, "INDENT");
}

#[test]
fn ee_v10_ext_single_symbol_id_valid() {
    let g = base("ee_v10_ext04")
        .rule("stmt", vec!["ID", ";"])
        .external("INDENT")
        .build();
    // SymbolId should be non-zero (0 is typically start symbol)
    assert_ne!(g.externals[0].symbol_id, SymbolId(u16::MAX));
}

#[test]
fn ee_v10_ext_three() {
    let g = base("ee_v10_ext05")
        .rule("stmt", vec!["ID", ";"])
        .external("INDENT")
        .external("DEDENT")
        .external("NEWLINE")
        .build();
    assert_eq!(g.externals.len(), 3);
}

#[test]
fn ee_v10_ext_three_names() {
    let g = base("ee_v10_ext06")
        .rule("stmt", vec!["ID", ";"])
        .external("INDENT")
        .external("DEDENT")
        .external("NEWLINE")
        .build();
    let names: Vec<&str> = g.externals.iter().map(|e| e.name.as_str()).collect();
    assert_eq!(names, vec!["INDENT", "DEDENT", "NEWLINE"]);
}

#[test]
fn ee_v10_ext_unique_symbol_ids() {
    let g = base("ee_v10_ext07")
        .rule("stmt", vec!["ID", ";"])
        .external("INDENT")
        .external("DEDENT")
        .build();
    assert_ne!(g.externals[0].symbol_id, g.externals[1].symbol_id);
}

#[test]
fn ee_v10_ext_five() {
    let g = base("ee_v10_ext08")
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
fn ee_v10_ext_preserves_order() {
    let g = base("ee_v10_ext09")
        .rule("stmt", vec!["ID", ";"])
        .external("ZZZ")
        .external("AAA")
        .external("MMM")
        .build();
    let names: Vec<&str> = g.externals.iter().map(|e| e.name.as_str()).collect();
    assert_eq!(names, vec!["ZZZ", "AAA", "MMM"]);
}

#[test]
fn ee_v10_ext_does_not_appear_in_tokens() {
    let g = base("ee_v10_ext10")
        .rule("stmt", vec!["ID", ";"])
        .external("INDENT")
        .build();
    let token_names: Vec<&str> = g.tokens.values().map(|t| t.name.as_str()).collect();
    assert!(!token_names.contains(&"INDENT"));
}

#[test]
fn ee_v10_ext_symbol_id_not_in_tokens() {
    let g = base("ee_v10_ext11")
        .rule("stmt", vec!["ID", ";"])
        .external("INDENT")
        .build();
    let ext_id = g.externals[0].symbol_id;
    // External tokens are not registered as regular tokens
    assert!(!g.tokens.contains_key(&ext_id));
}

#[test]
fn ee_v10_ext_grammar_name_preserved() {
    let g = base("ee_v10_ext12")
        .rule("stmt", vec!["ID", ";"])
        .external("INDENT")
        .build();
    assert_eq!(g.name, "ee_v10_ext12");
}

#[test]
fn ee_v10_ext_rules_still_present() {
    let g = base("ee_v10_ext13")
        .rule("stmt", vec!["ID", ";"])
        .external("INDENT")
        .build();
    assert!(!g.rules.is_empty());
}

#[test]
fn ee_v10_ext_tokens_still_present() {
    let g = base("ee_v10_ext14")
        .rule("stmt", vec!["ID", ";"])
        .external("INDENT")
        .build();
    assert!(!g.tokens.is_empty());
}

#[test]
fn ee_v10_ext_two_externals_different_names() {
    let g = base("ee_v10_ext15")
        .rule("stmt", vec!["ID", ";"])
        .external("HEREDOC_START")
        .external("HEREDOC_END")
        .build();
    assert_ne!(g.externals[0].name, g.externals[1].name);
}

// ===========================================================================
// 2. ee_v10_extra_* – extras (15 tests)
// ===========================================================================

#[test]
fn ee_v10_extra_empty_by_default() {
    let g = base("ee_v10_xtra01").rule("stmt", vec!["ID", ";"]).build();
    assert!(g.extras.is_empty());
}

#[test]
fn ee_v10_extra_single() {
    let g = base("ee_v10_xtra02")
        .rule("stmt", vec!["ID", ";"])
        .token("WS", r"\s+")
        .extra("WS")
        .build();
    assert_eq!(g.extras.len(), 1);
}

#[test]
fn ee_v10_extra_single_resolves() {
    let g = base("ee_v10_xtra03")
        .rule("stmt", vec!["ID", ";"])
        .token("WS", r"\s+")
        .extra("WS")
        .build();
    assert_eq!(g.extras[0], tok(&g, "WS"));
}

#[test]
fn ee_v10_extra_three() {
    let g = base("ee_v10_xtra04")
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
fn ee_v10_extra_preserves_order() {
    let g = base("ee_v10_xtra05")
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
fn ee_v10_extra_ids_are_valid_symbols() {
    let g = base("ee_v10_xtra06")
        .rule("stmt", vec!["ID", ";"])
        .token("WS", r"\s+")
        .extra("WS")
        .build();
    let ws_id = g.extras[0];
    assert!(g.tokens.contains_key(&ws_id) || g.rule_names.contains_key(&ws_id));
}

#[test]
fn ee_v10_extra_two_distinct_ids() {
    let g = base("ee_v10_xtra07")
        .rule("stmt", vec!["ID", ";"])
        .token("WS", r"\s+")
        .token("NL", r"\n")
        .extra("WS")
        .extra("NL")
        .build();
    assert_ne!(g.extras[0], g.extras[1]);
}

#[test]
fn ee_v10_extra_does_not_affect_rules() {
    let g = base("ee_v10_xtra08")
        .rule("stmt", vec!["ID", ";"])
        .token("WS", r"\s+")
        .extra("WS")
        .build();
    assert!(!g.rules.is_empty());
}

#[test]
fn ee_v10_extra_grammar_name() {
    let g = base("ee_v10_xtra09")
        .rule("stmt", vec!["ID", ";"])
        .token("WS", r"\s+")
        .extra("WS")
        .build();
    assert_eq!(g.name, "ee_v10_xtra09");
}

#[test]
fn ee_v10_extra_five() {
    let g = base("ee_v10_xtra10")
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
fn ee_v10_extra_tokens_still_present() {
    let g = base("ee_v10_xtra11")
        .rule("stmt", vec!["ID", ";"])
        .token("WS", r"\s+")
        .extra("WS")
        .build();
    // The WS token should still exist in the token map
    assert!(g.tokens.contains_key(&tok(&g, "WS")));
}

#[test]
fn ee_v10_extra_not_in_externals() {
    let g = base("ee_v10_xtra12")
        .rule("stmt", vec!["ID", ";"])
        .token("WS", r"\s+")
        .extra("WS")
        .build();
    assert!(g.externals.is_empty());
}

#[test]
fn ee_v10_extra_not_in_inline() {
    let g = base("ee_v10_xtra13")
        .rule("stmt", vec!["ID", ";"])
        .token("WS", r"\s+")
        .extra("WS")
        .build();
    assert!(g.inline_rules.is_empty());
}

#[test]
fn ee_v10_extra_not_in_supertypes() {
    let g = base("ee_v10_xtra14")
        .rule("stmt", vec!["ID", ";"])
        .token("WS", r"\s+")
        .extra("WS")
        .build();
    assert!(g.supertypes.is_empty());
}

#[test]
fn ee_v10_extra_nonterminal() {
    // extras can also be non-terminal symbols
    let g = base("ee_v10_xtra15")
        .rule("stmt", vec!["ID", ";"])
        .rule("comment", vec!["ID"])
        .extra("comment")
        .build();
    assert_eq!(g.extras.len(), 1);
}

// ===========================================================================
// 3. ee_v10_inline_* – inline rules (12 tests)
// ===========================================================================

#[test]
fn ee_v10_inline_empty_by_default() {
    let g = base("ee_v10_inl01").rule("stmt", vec!["ID", ";"]).build();
    assert!(g.inline_rules.is_empty());
}

#[test]
fn ee_v10_inline_single() {
    let g = base("ee_v10_inl02")
        .rule("stmt", vec!["helper"])
        .rule("helper", vec!["ID", ";"])
        .inline("helper")
        .build();
    assert_eq!(g.inline_rules.len(), 1);
}

#[test]
fn ee_v10_inline_single_resolves() {
    let g = base("ee_v10_inl03")
        .rule("stmt", vec!["helper"])
        .rule("helper", vec!["ID", ";"])
        .inline("helper")
        .build();
    assert_eq!(g.inline_rules[0], sym(&g, "helper"));
}

#[test]
fn ee_v10_inline_three() {
    let g = base("ee_v10_inl04")
        .rule("stmt", vec!["h1"])
        .rule("h1", vec!["ID"])
        .rule("h2", vec!["NUM"])
        .rule("h3", vec!["ID", ";"])
        .inline("h1")
        .inline("h2")
        .inline("h3")
        .build();
    assert_eq!(g.inline_rules.len(), 3);
}

#[test]
fn ee_v10_inline_preserves_order() {
    let g = base("ee_v10_inl05")
        .rule("stmt", vec!["a1"])
        .rule("a1", vec!["ID"])
        .rule("b1", vec!["NUM"])
        .inline("a1")
        .inline("b1")
        .build();
    assert_eq!(g.inline_rules[0], sym(&g, "a1"));
    assert_eq!(g.inline_rules[1], sym(&g, "b1"));
}

#[test]
fn ee_v10_inline_rule_still_in_rules() {
    let g = base("ee_v10_inl06")
        .rule("stmt", vec!["helper"])
        .rule("helper", vec!["ID", ";"])
        .inline("helper")
        .build();
    assert!(g.rules.contains_key(&sym(&g, "helper")));
}

#[test]
fn ee_v10_inline_does_not_affect_extras() {
    let g = base("ee_v10_inl07")
        .rule("stmt", vec!["helper"])
        .rule("helper", vec!["ID", ";"])
        .inline("helper")
        .build();
    assert!(g.extras.is_empty());
}

#[test]
fn ee_v10_inline_does_not_affect_externals() {
    let g = base("ee_v10_inl08")
        .rule("stmt", vec!["helper"])
        .rule("helper", vec!["ID", ";"])
        .inline("helper")
        .build();
    assert!(g.externals.is_empty());
}

#[test]
fn ee_v10_inline_five() {
    let g = base("ee_v10_inl09")
        .rule("stmt", vec!["r1"])
        .rule("r1", vec!["ID"])
        .rule("r2", vec!["NUM"])
        .rule("r3", vec!["ID", ";"])
        .rule("r4", vec!["NUM", ";"])
        .rule("r5", vec!["ID", "NUM"])
        .inline("r1")
        .inline("r2")
        .inline("r3")
        .inline("r4")
        .inline("r5")
        .build();
    assert_eq!(g.inline_rules.len(), 5);
}

#[test]
fn ee_v10_inline_ids_are_unique() {
    let g = base("ee_v10_inl10")
        .rule("stmt", vec!["h1"])
        .rule("h1", vec!["ID"])
        .rule("h2", vec!["NUM"])
        .inline("h1")
        .inline("h2")
        .build();
    assert_ne!(g.inline_rules[0], g.inline_rules[1]);
}

#[test]
fn ee_v10_inline_grammar_name() {
    let g = base("ee_v10_inl11")
        .rule("stmt", vec!["helper"])
        .rule("helper", vec!["ID", ";"])
        .inline("helper")
        .build();
    assert_eq!(g.name, "ee_v10_inl11");
}

#[test]
fn ee_v10_inline_not_in_supertypes() {
    let g = base("ee_v10_inl12")
        .rule("stmt", vec!["helper"])
        .rule("helper", vec!["ID", ";"])
        .inline("helper")
        .build();
    assert!(g.supertypes.is_empty());
}

// ===========================================================================
// 4. ee_v10_super_* – supertypes (10 tests)
// ===========================================================================

#[test]
fn ee_v10_super_empty_by_default() {
    let g = base("ee_v10_sup01").rule("stmt", vec!["ID", ";"]).build();
    assert!(g.supertypes.is_empty());
}

#[test]
fn ee_v10_super_single() {
    let g = base("ee_v10_sup02")
        .rule("stmt", vec!["ID", ";"])
        .rule("expr", vec!["ID"])
        .supertype("expr")
        .build();
    assert_eq!(g.supertypes.len(), 1);
}

#[test]
fn ee_v10_super_single_resolves() {
    let g = base("ee_v10_sup03")
        .rule("stmt", vec!["ID", ";"])
        .rule("expr", vec!["ID"])
        .supertype("expr")
        .build();
    assert_eq!(g.supertypes[0], sym(&g, "expr"));
}

#[test]
fn ee_v10_super_three() {
    let g = base("ee_v10_sup04")
        .rule("stmt", vec!["ID", ";"])
        .rule("expr", vec!["ID"])
        .rule("literal", vec!["NUM"])
        .rule("declaration", vec!["ID", ";"])
        .supertype("expr")
        .supertype("literal")
        .supertype("declaration")
        .build();
    assert_eq!(g.supertypes.len(), 3);
}

#[test]
fn ee_v10_super_preserves_order() {
    let g = base("ee_v10_sup05")
        .rule("stmt", vec!["ID", ";"])
        .rule("expr", vec!["ID"])
        .rule("literal", vec!["NUM"])
        .supertype("expr")
        .supertype("literal")
        .build();
    assert_eq!(g.supertypes[0], sym(&g, "expr"));
    assert_eq!(g.supertypes[1], sym(&g, "literal"));
}

#[test]
fn ee_v10_super_does_not_affect_inline() {
    let g = base("ee_v10_sup06")
        .rule("stmt", vec!["ID", ";"])
        .rule("expr", vec!["ID"])
        .supertype("expr")
        .build();
    assert!(g.inline_rules.is_empty());
}

#[test]
fn ee_v10_super_does_not_affect_extras() {
    let g = base("ee_v10_sup07")
        .rule("stmt", vec!["ID", ";"])
        .rule("expr", vec!["ID"])
        .supertype("expr")
        .build();
    assert!(g.extras.is_empty());
}

#[test]
fn ee_v10_super_does_not_affect_externals() {
    let g = base("ee_v10_sup08")
        .rule("stmt", vec!["ID", ";"])
        .rule("expr", vec!["ID"])
        .supertype("expr")
        .build();
    assert!(g.externals.is_empty());
}

#[test]
fn ee_v10_super_five() {
    let g = base("ee_v10_sup09")
        .rule("stmt", vec!["ID", ";"])
        .rule("s1", vec!["ID"])
        .rule("s2", vec!["NUM"])
        .rule("s3", vec!["ID", ";"])
        .rule("s4", vec!["NUM", ";"])
        .rule("s5", vec!["ID", "NUM"])
        .supertype("s1")
        .supertype("s2")
        .supertype("s3")
        .supertype("s4")
        .supertype("s5")
        .build();
    assert_eq!(g.supertypes.len(), 5);
}

#[test]
fn ee_v10_super_ids_unique() {
    let g = base("ee_v10_sup10")
        .rule("stmt", vec!["ID", ";"])
        .rule("s1", vec!["ID"])
        .rule("s2", vec!["NUM"])
        .supertype("s1")
        .supertype("s2")
        .build();
    assert_ne!(g.supertypes[0], g.supertypes[1]);
}

// ===========================================================================
// 5. ee_v10_conflict_* – conflicts (5 tests)
// ===========================================================================

#[test]
fn ee_v10_conflict_empty_by_default() {
    let g = base("ee_v10_conf01").rule("stmt", vec!["ID", ";"]).build();
    assert!(g.conflicts.is_empty());
}

#[test]
fn ee_v10_conflict_empty_count() {
    let g = base("ee_v10_conf02").rule("stmt", vec!["ID", ";"]).build();
    assert_eq!(g.conflicts.len(), 0);
}

#[test]
fn ee_v10_conflict_manual_single() {
    let mut g = Grammar::new("ee_v10_conf03".into());
    assert!(g.conflicts.is_empty());
    // Conflicts are empty in a fresh grammar
    assert_eq!(g.conflicts.len(), 0);
    // They can be pushed manually
    g.conflicts.push(adze_ir::ConflictDeclaration {
        symbols: vec![SymbolId(1), SymbolId(2)],
        resolution: adze_ir::ConflictResolution::GLR,
    });
    assert_eq!(g.conflicts.len(), 1);
}

#[test]
fn ee_v10_conflict_manual_symbols() {
    let mut g = Grammar::new("ee_v10_conf04".into());
    g.conflicts.push(adze_ir::ConflictDeclaration {
        symbols: vec![SymbolId(3), SymbolId(4), SymbolId(5)],
        resolution: adze_ir::ConflictResolution::GLR,
    });
    assert_eq!(g.conflicts[0].symbols.len(), 3);
}

#[test]
fn ee_v10_conflict_manual_resolution() {
    let mut g = Grammar::new("ee_v10_conf05".into());
    g.conflicts.push(adze_ir::ConflictDeclaration {
        symbols: vec![SymbolId(1)],
        resolution: adze_ir::ConflictResolution::GLR,
    });
    assert_eq!(g.conflicts[0].resolution, adze_ir::ConflictResolution::GLR);
}

// ===========================================================================
// 6. ee_v10_clone_* – clone preserves fields (8 tests)
// ===========================================================================

#[test]
fn ee_v10_clone_externals() {
    let g = base("ee_v10_cln01")
        .rule("stmt", vec!["ID", ";"])
        .external("INDENT")
        .external("DEDENT")
        .build();
    let g2 = g.clone();
    assert_eq!(g2.externals.len(), g.externals.len());
    assert_eq!(g2.externals[0].name, g.externals[0].name);
    assert_eq!(g2.externals[1].name, g.externals[1].name);
}

#[test]
fn ee_v10_clone_extras() {
    let g = base("ee_v10_cln02")
        .rule("stmt", vec!["ID", ";"])
        .token("WS", r"\s+")
        .extra("WS")
        .build();
    let g2 = g.clone();
    assert_eq!(g2.extras.len(), g.extras.len());
    assert_eq!(g2.extras[0], g.extras[0]);
}

#[test]
fn ee_v10_clone_inline_rules() {
    let g = base("ee_v10_cln03")
        .rule("stmt", vec!["helper"])
        .rule("helper", vec!["ID", ";"])
        .inline("helper")
        .build();
    let g2 = g.clone();
    assert_eq!(g2.inline_rules.len(), g.inline_rules.len());
    assert_eq!(g2.inline_rules[0], g.inline_rules[0]);
}

#[test]
fn ee_v10_clone_supertypes() {
    let g = base("ee_v10_cln04")
        .rule("stmt", vec!["ID", ";"])
        .rule("expr", vec!["ID"])
        .supertype("expr")
        .build();
    let g2 = g.clone();
    assert_eq!(g2.supertypes.len(), g.supertypes.len());
    assert_eq!(g2.supertypes[0], g.supertypes[0]);
}

#[test]
fn ee_v10_clone_conflicts() {
    let mut g = Grammar::new("ee_v10_cln05".into());
    g.conflicts.push(adze_ir::ConflictDeclaration {
        symbols: vec![SymbolId(1), SymbolId(2)],
        resolution: adze_ir::ConflictResolution::GLR,
    });
    let g2 = g.clone();
    assert_eq!(g2.conflicts.len(), 1);
    assert_eq!(g2.conflicts[0].symbols.len(), 2);
}

#[test]
fn ee_v10_clone_name() {
    let g = base("ee_v10_cln06")
        .rule("stmt", vec!["ID", ";"])
        .external("INDENT")
        .build();
    let g2 = g.clone();
    assert_eq!(g2.name, g.name);
}

#[test]
fn ee_v10_clone_external_symbol_ids() {
    let g = base("ee_v10_cln07")
        .rule("stmt", vec!["ID", ";"])
        .external("INDENT")
        .external("DEDENT")
        .build();
    let g2 = g.clone();
    assert_eq!(g2.externals[0].symbol_id, g.externals[0].symbol_id);
    assert_eq!(g2.externals[1].symbol_id, g.externals[1].symbol_id);
}

#[test]
fn ee_v10_clone_independent() {
    let g = base("ee_v10_cln08")
        .rule("stmt", vec!["ID", ";"])
        .token("WS", r"\s+")
        .extra("WS")
        .external("INDENT")
        .build();
    let mut g2 = g.clone();
    g2.extras.clear();
    // Mutating clone should not affect original
    assert_eq!(g.extras.len(), 1);
    assert!(g2.extras.is_empty());
}

// ===========================================================================
// 7. ee_v10_norm_* – normalize/optimize don't crash (8 tests)
// ===========================================================================

#[test]
fn ee_v10_norm_with_externals() {
    let mut g = base("ee_v10_nrm01")
        .rule("stmt", vec!["ID", ";"])
        .external("INDENT")
        .build();
    g.normalize();
    assert_eq!(g.externals.len(), 1);
}

#[test]
fn ee_v10_norm_with_extras() {
    let mut g = base("ee_v10_nrm02")
        .rule("stmt", vec!["ID", ";"])
        .token("WS", r"\s+")
        .extra("WS")
        .build();
    g.normalize();
    assert_eq!(g.extras.len(), 1);
}

#[test]
fn ee_v10_norm_with_inline() {
    let mut g = base("ee_v10_nrm03")
        .rule("stmt", vec!["helper"])
        .rule("helper", vec!["ID", ";"])
        .inline("helper")
        .build();
    g.normalize();
    assert_eq!(g.inline_rules.len(), 1);
}

#[test]
fn ee_v10_norm_with_supertypes() {
    let mut g = base("ee_v10_nrm04")
        .rule("stmt", vec!["ID", ";"])
        .rule("expr", vec!["ID"])
        .supertype("expr")
        .build();
    g.normalize();
    assert_eq!(g.supertypes.len(), 1);
}

#[test]
fn ee_v10_norm_preserves_external_name() {
    let mut g = base("ee_v10_nrm05")
        .rule("stmt", vec!["ID", ";"])
        .external("INDENT")
        .build();
    g.normalize();
    assert_eq!(g.externals[0].name, "INDENT");
}

#[test]
fn ee_v10_opt_with_externals() {
    let mut g = base("ee_v10_nrm06")
        .rule("stmt", vec!["ID", ";"])
        .external("INDENT")
        .build();
    g.optimize();
    assert_eq!(g.externals.len(), 1);
}

#[test]
fn ee_v10_opt_with_extras() {
    let mut g = base("ee_v10_nrm07")
        .rule("stmt", vec!["ID", ";"])
        .token("WS", r"\s+")
        .extra("WS")
        .build();
    g.optimize();
    assert_eq!(g.extras.len(), 1);
}

#[test]
fn ee_v10_opt_with_all_features() {
    let mut g = base("ee_v10_nrm08")
        .rule("stmt", vec!["helper"])
        .rule("helper", vec!["ID", ";"])
        .rule("expr", vec!["ID"])
        .token("WS", r"\s+")
        .external("INDENT")
        .extra("WS")
        .inline("helper")
        .supertype("expr")
        .build();
    g.optimize();
    assert_eq!(g.externals.len(), 1);
    assert_eq!(g.extras.len(), 1);
    assert_eq!(g.inline_rules.len(), 1);
    assert_eq!(g.supertypes.len(), 1);
}

// ===========================================================================
// 8. ee_v10_combo_* – combined features (12 tests)
// ===========================================================================

#[test]
fn ee_v10_combo_ext_and_extra() {
    let g = base("ee_v10_cmb01")
        .rule("stmt", vec!["ID", ";"])
        .token("WS", r"\s+")
        .external("INDENT")
        .extra("WS")
        .build();
    assert_eq!(g.externals.len(), 1);
    assert_eq!(g.extras.len(), 1);
}

#[test]
fn ee_v10_combo_ext_and_inline() {
    let g = base("ee_v10_cmb02")
        .rule("stmt", vec!["helper"])
        .rule("helper", vec!["ID", ";"])
        .external("INDENT")
        .inline("helper")
        .build();
    assert_eq!(g.externals.len(), 1);
    assert_eq!(g.inline_rules.len(), 1);
}

#[test]
fn ee_v10_combo_extra_and_inline() {
    let g = base("ee_v10_cmb03")
        .rule("stmt", vec!["helper"])
        .rule("helper", vec!["ID", ";"])
        .token("WS", r"\s+")
        .extra("WS")
        .inline("helper")
        .build();
    assert_eq!(g.extras.len(), 1);
    assert_eq!(g.inline_rules.len(), 1);
}

#[test]
fn ee_v10_combo_all_three() {
    let g = base("ee_v10_cmb04")
        .rule("stmt", vec!["helper"])
        .rule("helper", vec!["ID", ";"])
        .token("WS", r"\s+")
        .external("INDENT")
        .extra("WS")
        .inline("helper")
        .build();
    assert_eq!(g.externals.len(), 1);
    assert_eq!(g.extras.len(), 1);
    assert_eq!(g.inline_rules.len(), 1);
}

#[test]
fn ee_v10_combo_all_four() {
    let g = base("ee_v10_cmb05")
        .rule("stmt", vec!["helper"])
        .rule("helper", vec!["ID", ";"])
        .rule("expr", vec!["ID"])
        .token("WS", r"\s+")
        .external("INDENT")
        .extra("WS")
        .inline("helper")
        .supertype("expr")
        .build();
    assert_eq!(g.externals.len(), 1);
    assert_eq!(g.extras.len(), 1);
    assert_eq!(g.inline_rules.len(), 1);
    assert_eq!(g.supertypes.len(), 1);
}

#[test]
fn ee_v10_combo_multiple_of_each() {
    let g = base("ee_v10_cmb06")
        .rule("stmt", vec!["h1"])
        .rule("h1", vec!["ID"])
        .rule("h2", vec!["NUM"])
        .rule("expr", vec!["ID"])
        .rule("literal", vec!["NUM"])
        .token("WS", r"\s+")
        .token("COMMENT", "//[^\n]*")
        .external("INDENT")
        .external("DEDENT")
        .extra("WS")
        .extra("COMMENT")
        .inline("h1")
        .inline("h2")
        .supertype("expr")
        .supertype("literal")
        .build();
    assert_eq!(g.externals.len(), 2);
    assert_eq!(g.extras.len(), 2);
    assert_eq!(g.inline_rules.len(), 2);
    assert_eq!(g.supertypes.len(), 2);
}

#[test]
fn ee_v10_combo_ext_extra_separate_symbols() {
    let g = base("ee_v10_cmb07")
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
fn ee_v10_combo_inline_supertype_separate() {
    let g = base("ee_v10_cmb08")
        .rule("stmt", vec!["helper"])
        .rule("helper", vec!["ID", ";"])
        .rule("expr", vec!["ID"])
        .inline("helper")
        .supertype("expr")
        .build();
    assert_ne!(g.inline_rules[0], g.supertypes[0]);
}

#[test]
fn ee_v10_combo_build_succeeds() {
    // Ensure complex grammar with many features builds without panic
    let g = base("ee_v10_cmb09")
        .rule("stmt", vec!["h1"])
        .rule("h1", vec!["ID", ";"])
        .rule("h2", vec!["NUM", ";"])
        .rule("expr", vec!["ID"])
        .rule("literal", vec!["NUM"])
        .token("WS", r"\s+")
        .token("COMMENT", "//[^\n]*")
        .token("NL", r"\n")
        .external("INDENT")
        .external("DEDENT")
        .external("NEWLINE")
        .extra("WS")
        .extra("COMMENT")
        .extra("NL")
        .inline("h1")
        .inline("h2")
        .supertype("expr")
        .supertype("literal")
        .build();
    assert_eq!(g.name, "ee_v10_cmb09");
}

#[test]
fn ee_v10_combo_rules_not_empty() {
    let g = base("ee_v10_cmb10")
        .rule("stmt", vec!["ID", ";"])
        .external("INDENT")
        .token("WS", r"\s+")
        .extra("WS")
        .build();
    assert!(!g.rules.is_empty());
}

#[test]
fn ee_v10_combo_tokens_not_empty() {
    let g = base("ee_v10_cmb11")
        .rule("stmt", vec!["ID", ";"])
        .external("INDENT")
        .token("WS", r"\s+")
        .extra("WS")
        .build();
    assert!(!g.tokens.is_empty());
}

#[test]
fn ee_v10_combo_normalize_all() {
    let mut g = base("ee_v10_cmb12")
        .rule("stmt", vec!["helper"])
        .rule("helper", vec!["ID", ";"])
        .rule("expr", vec!["ID"])
        .token("WS", r"\s+")
        .external("INDENT")
        .extra("WS")
        .inline("helper")
        .supertype("expr")
        .build();
    g.normalize();
    // All metadata survives normalization
    assert_eq!(g.externals.len(), 1);
    assert_eq!(g.extras.len(), 1);
    assert_eq!(g.inline_rules.len(), 1);
    assert_eq!(g.supertypes.len(), 1);
}

// ===========================================================================
// 9. ee_v10_debug_* – Debug formatting (5 tests)
// ===========================================================================

#[test]
fn ee_v10_debug_grammar_with_externals() {
    let g = base("ee_v10_dbg01")
        .rule("stmt", vec!["ID", ";"])
        .external("INDENT")
        .build();
    let dbg = format!("{g:?}");
    assert!(dbg.contains("INDENT"));
}

#[test]
fn ee_v10_debug_grammar_with_extras() {
    let g = base("ee_v10_dbg02")
        .rule("stmt", vec!["ID", ";"])
        .token("WS", r"\s+")
        .extra("WS")
        .build();
    let dbg = format!("{g:?}");
    assert!(dbg.contains("extras"));
}

#[test]
fn ee_v10_debug_grammar_name() {
    let g = base("ee_v10_dbg03").rule("stmt", vec!["ID", ";"]).build();
    let dbg = format!("{g:?}");
    assert!(dbg.contains("ee_v10_dbg03"));
}

#[test]
fn ee_v10_debug_external_token() {
    let et = adze_ir::ExternalToken {
        name: "HEREDOC".into(),
        symbol_id: SymbolId(99),
    };
    let dbg = format!("{et:?}");
    assert!(dbg.contains("HEREDOC"));
    assert!(dbg.contains("99"));
}

#[test]
fn ee_v10_debug_conflict_declaration() {
    let cd = adze_ir::ConflictDeclaration {
        symbols: vec![SymbolId(1), SymbolId(2)],
        resolution: adze_ir::ConflictResolution::GLR,
    };
    let dbg = format!("{cd:?}");
    assert!(dbg.contains("GLR"));
}

// ===========================================================================
// 10. ee_v10_edge_* – edge cases (5 tests)
// ===========================================================================

#[test]
fn ee_v10_edge_grammar_new_all_empty() {
    let g = Grammar::new("ee_v10_edge01".into());
    assert!(g.externals.is_empty());
    assert!(g.extras.is_empty());
    assert!(g.inline_rules.is_empty());
    assert!(g.supertypes.is_empty());
    assert!(g.conflicts.is_empty());
}

#[test]
fn ee_v10_edge_grammar_new_name() {
    let g = Grammar::new("ee_v10_edge02".into());
    assert_eq!(g.name, "ee_v10_edge02");
}

#[test]
fn ee_v10_edge_many_externals() {
    let mut builder = base("ee_v10_edge03").rule("stmt", vec!["ID", ";"]);
    for i in 0..20 {
        builder = builder.external(&format!("EXT_{i}"));
    }
    let g = builder.build();
    assert_eq!(g.externals.len(), 20);
}

#[test]
fn ee_v10_edge_many_extras() {
    let mut builder = base("ee_v10_edge04").rule("stmt", vec!["ID", ";"]);
    for i in 0..10 {
        let tok_name = format!("TOK_{i}");
        builder = builder.token(&tok_name, &format!("t{i}"));
    }
    for i in 0..10 {
        let tok_name = format!("TOK_{i}");
        builder = builder.extra(&tok_name);
    }
    let g = builder.build();
    assert_eq!(g.extras.len(), 10);
}

#[test]
fn ee_v10_edge_external_symbol_id_copy() {
    let g = base("ee_v10_edge05")
        .rule("stmt", vec!["ID", ";"])
        .external("INDENT")
        .build();
    let id1 = g.externals[0].symbol_id;
    let id2 = g.externals[0].symbol_id;
    assert_eq!(id1, id2);
}
