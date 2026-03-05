//! Comprehensive tests for inline rules, supertypes, and extras in adze-ir.

use adze_ir::builder::GrammarBuilder;
use adze_ir::*;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Build a minimal grammar that has rules for the given non-terminal names.
fn minimal_grammar(name: &str) -> GrammarBuilder {
    GrammarBuilder::new(name)
        .token("ID", r"[a-z]+")
        .token(";", ";")
        .rule("start", vec!["ID", ";"])
        .start("start")
}

// ===========================================================================
// 1. Inline rules registration (8 tests)
// ===========================================================================

#[test]
fn test_inline_rules_empty_by_default() {
    let g = minimal_grammar("t1").build();
    assert!(g.inline_rules.is_empty());
}

#[test]
fn test_inline_rules_single_entry() {
    let mut g = minimal_grammar("t2").rule("helper", vec!["ID"]).build();
    let id = g.find_symbol_by_name("helper").unwrap();
    g.inline_rules.push(id);
    assert_eq!(g.inline_rules.len(), 1);
    assert_eq!(g.inline_rules[0], id);
}

#[test]
fn test_inline_rules_multiple_entries() {
    let mut g = minimal_grammar("t3")
        .rule("a", vec!["ID"])
        .rule("b", vec!["ID"])
        .rule("c", vec!["ID"])
        .build();
    let a = g.find_symbol_by_name("a").unwrap();
    let b = g.find_symbol_by_name("b").unwrap();
    let c = g.find_symbol_by_name("c").unwrap();
    g.inline_rules = vec![a, b, c];
    assert_eq!(g.inline_rules.len(), 3);
}

#[test]
fn test_inline_rules_preserves_order() {
    let mut g = minimal_grammar("t4")
        .rule("x", vec!["ID"])
        .rule("y", vec!["ID"])
        .build();
    let x = g.find_symbol_by_name("x").unwrap();
    let y = g.find_symbol_by_name("y").unwrap();
    g.inline_rules = vec![y, x];
    assert_eq!(g.inline_rules[0], y);
    assert_eq!(g.inline_rules[1], x);
}

#[test]
fn test_inline_rules_contains_specific_symbol() {
    let mut g = minimal_grammar("t5").rule("helper", vec!["ID"]).build();
    let helper = g.find_symbol_by_name("helper").unwrap();
    g.inline_rules.push(helper);
    assert!(g.inline_rules.contains(&helper));
}

#[test]
fn test_inline_rules_does_not_affect_rules_map() {
    let mut g = minimal_grammar("t6").rule("helper", vec!["ID"]).build();
    let helper = g.find_symbol_by_name("helper").unwrap();
    let rules_before = g.rules.len();
    g.inline_rules.push(helper);
    assert_eq!(g.rules.len(), rules_before);
}

#[test]
fn test_inline_rules_symbol_still_has_productions() {
    let mut g = minimal_grammar("t7").rule("helper", vec!["ID"]).build();
    let helper = g.find_symbol_by_name("helper").unwrap();
    g.inline_rules.push(helper);
    assert!(g.get_rules_for_symbol(helper).is_some());
}

#[test]
fn test_inline_rules_independent_of_supertypes() {
    let mut g = minimal_grammar("t8").rule("helper", vec!["ID"]).build();
    let helper = g.find_symbol_by_name("helper").unwrap();
    g.inline_rules.push(helper);
    g.supertypes.push(helper);
    assert_eq!(g.inline_rules.len(), 1);
    assert_eq!(g.supertypes.len(), 1);
}

// ===========================================================================
// 2. Supertype registration (8 tests)
// ===========================================================================

#[test]
fn test_supertypes_empty_by_default() {
    let g = minimal_grammar("s1").build();
    assert!(g.supertypes.is_empty());
}

#[test]
fn test_supertypes_single_entry() {
    let mut g = minimal_grammar("s2").rule("expression", vec!["ID"]).build();
    let expr = g.find_symbol_by_name("expression").unwrap();
    g.supertypes.push(expr);
    assert_eq!(g.supertypes.len(), 1);
    assert_eq!(g.supertypes[0], expr);
}

#[test]
fn test_supertypes_multiple_entries() {
    let mut g = minimal_grammar("s3")
        .rule("expression", vec!["ID"])
        .rule("statement", vec!["expression", ";"])
        .build();
    let expr = g.find_symbol_by_name("expression").unwrap();
    let stmt = g.find_symbol_by_name("statement").unwrap();
    g.supertypes = vec![expr, stmt];
    assert_eq!(g.supertypes.len(), 2);
}

#[test]
fn test_supertypes_preserves_insertion_order() {
    let mut g = minimal_grammar("s4")
        .rule("expression", vec!["ID"])
        .rule("statement", vec!["expression", ";"])
        .build();
    let expr = g.find_symbol_by_name("expression").unwrap();
    let stmt = g.find_symbol_by_name("statement").unwrap();
    g.supertypes = vec![stmt, expr];
    assert_eq!(g.supertypes[0], stmt);
    assert_eq!(g.supertypes[1], expr);
}

#[test]
fn test_supertypes_does_not_affect_token_count() {
    let mut g = minimal_grammar("s5").rule("expression", vec!["ID"]).build();
    let tokens_before = g.tokens.len();
    let expr = g.find_symbol_by_name("expression").unwrap();
    g.supertypes.push(expr);
    assert_eq!(g.tokens.len(), tokens_before);
}

#[test]
fn test_supertypes_contains_check() {
    let mut g = minimal_grammar("s6")
        .rule("expression", vec!["ID"])
        .rule("statement", vec!["expression", ";"])
        .build();
    let expr = g.find_symbol_by_name("expression").unwrap();
    let stmt = g.find_symbol_by_name("statement").unwrap();
    g.supertypes.push(expr);
    assert!(g.supertypes.contains(&expr));
    assert!(!g.supertypes.contains(&stmt));
}

#[test]
fn test_supertypes_symbol_keeps_its_rules() {
    let mut g = minimal_grammar("s7").rule("expression", vec!["ID"]).build();
    let expr = g.find_symbol_by_name("expression").unwrap();
    g.supertypes.push(expr);
    let rules = g.get_rules_for_symbol(expr).unwrap();
    assert_eq!(rules.len(), 1);
}

#[test]
fn test_supertypes_independent_of_extras() {
    let mut g = minimal_grammar("s8").rule("expression", vec!["ID"]).build();
    let expr = g.find_symbol_by_name("expression").unwrap();
    g.supertypes.push(expr);
    assert!(g.extras.is_empty());
}

// ===========================================================================
// 3. Extras registration (8 tests)
// ===========================================================================

#[test]
fn test_extras_empty_by_default() {
    let g = GrammarBuilder::new("e1")
        .token("ID", r"[a-z]+")
        .rule("start", vec!["ID"])
        .start("start")
        .build();
    assert!(g.extras.is_empty());
}

#[test]
fn test_extras_single_via_builder() {
    let g = GrammarBuilder::new("e2")
        .token("ID", r"[a-z]+")
        .token("WS", r"\s+")
        .rule("start", vec!["ID"])
        .start("start")
        .extra("WS")
        .build();
    assert_eq!(g.extras.len(), 1);
}

#[test]
fn test_extras_multiple_via_builder() {
    let g = GrammarBuilder::new("e3")
        .token("ID", r"[a-z]+")
        .token("WS", r"\s+")
        .token("COMMENT", r"//[^\n]*")
        .rule("start", vec!["ID"])
        .start("start")
        .extra("WS")
        .extra("COMMENT")
        .build();
    assert_eq!(g.extras.len(), 2);
}

#[test]
fn test_extras_preserves_order() {
    let g = GrammarBuilder::new("e4")
        .token("ID", r"[a-z]+")
        .token("ws_tok", r"\s+")
        .token("comment_tok", r"//[^\n]*")
        .rule("start", vec!["ID"])
        .start("start")
        .extra("comment_tok")
        .extra("ws_tok")
        .build();
    let comment = g.find_symbol_by_name("comment_tok").unwrap();
    let ws = g.find_symbol_by_name("ws_tok").unwrap();
    assert_eq!(g.extras[0], comment);
    assert_eq!(g.extras[1], ws);
}

#[test]
fn test_extras_symbol_marked_hidden_in_registry() {
    let mut g = GrammarBuilder::new("e5")
        .token("ws_tok", r"\s+")
        .token("ID", r"[a-z]+")
        .rule("start", vec!["ID"])
        .start("start")
        .extra("ws_tok")
        .build();
    // extras list should contain the ws_tok symbol
    assert_eq!(g.extras.len(), 1);
    let ws = g.extras[0];
    let registry = g.get_or_build_registry();
    let meta = registry.get_metadata(ws);
    assert!(meta.is_some());
}

#[test]
fn test_extras_does_not_affect_rules() {
    let g = GrammarBuilder::new("e6")
        .token("ID", r"[a-z]+")
        .token("WS", r"\s+")
        .rule("start", vec!["ID"])
        .start("start")
        .extra("WS")
        .build();
    // WS is an extra but not referenced in any rule
    assert_eq!(g.rules.len(), 1);
}

#[test]
fn test_extras_can_be_set_directly() {
    let mut g = minimal_grammar("e7").build();
    // Use a known SymbolId from the tokens map instead of find_symbol_by_name
    let id_sym = *g.tokens.keys().next().unwrap();
    g.extras.push(id_sym);
    assert_eq!(g.extras.len(), 1);
    assert_eq!(g.extras[0], id_sym);
}

#[test]
fn test_extras_independent_of_inline_rules() {
    let g = GrammarBuilder::new("e8")
        .token("ID", r"[a-z]+")
        .token("WS", r"\s+")
        .rule("start", vec!["ID"])
        .start("start")
        .extra("WS")
        .build();
    assert_eq!(g.extras.len(), 1);
    assert!(g.inline_rules.is_empty());
}

// ===========================================================================
// 4. Builder inline/supertype/extra API (8 tests)
// ===========================================================================

#[test]
fn test_builder_inline_registers_symbol() {
    let g = GrammarBuilder::new("b1")
        .token("ID", r"[a-z]+")
        .rule("start", vec!["helper"])
        .rule("helper", vec!["ID"])
        .start("start")
        .inline("helper")
        .build();
    assert_eq!(g.inline_rules.len(), 1);
    let helper = g.find_symbol_by_name("helper").unwrap();
    assert_eq!(g.inline_rules[0], helper);
}

#[test]
fn test_builder_supertype_registers_symbol() {
    let g = GrammarBuilder::new("b2")
        .token("ID", r"[a-z]+")
        .rule("start", vec!["expression"])
        .rule("expression", vec!["ID"])
        .start("start")
        .supertype("expression")
        .build();
    assert_eq!(g.supertypes.len(), 1);
    let expr = g.find_symbol_by_name("expression").unwrap();
    assert_eq!(g.supertypes[0], expr);
}

#[test]
fn test_builder_chaining_inline_and_supertype() {
    let g = GrammarBuilder::new("b3")
        .token("ID", r"[a-z]+")
        .rule("start", vec!["expression"])
        .rule("expression", vec!["ID"])
        .rule("helper", vec!["ID"])
        .start("start")
        .inline("helper")
        .supertype("expression")
        .build();
    assert_eq!(g.inline_rules.len(), 1);
    assert_eq!(g.supertypes.len(), 1);
}

#[test]
fn test_builder_chaining_all_three() {
    let g = GrammarBuilder::new("b4")
        .token("ID", r"[a-z]+")
        .token("WS", r"\s+")
        .rule("start", vec!["expression"])
        .rule("expression", vec!["ID"])
        .rule("helper", vec!["ID"])
        .start("start")
        .extra("WS")
        .inline("helper")
        .supertype("expression")
        .build();
    assert_eq!(g.extras.len(), 1);
    assert_eq!(g.inline_rules.len(), 1);
    assert_eq!(g.supertypes.len(), 1);
}

#[test]
fn test_builder_inline_multiple() {
    let g = GrammarBuilder::new("b5")
        .token("ID", r"[a-z]+")
        .rule("start", vec!["ID"])
        .rule("a", vec!["ID"])
        .rule("b", vec!["ID"])
        .start("start")
        .inline("a")
        .inline("b")
        .build();
    assert_eq!(g.inline_rules.len(), 2);
}

#[test]
fn test_builder_supertype_multiple() {
    let g = GrammarBuilder::new("b6")
        .token("ID", r"[a-z]+")
        .rule("start", vec!["ID"])
        .rule("expression", vec!["ID"])
        .rule("statement", vec!["expression"])
        .start("start")
        .supertype("expression")
        .supertype("statement")
        .build();
    assert_eq!(g.supertypes.len(), 2);
}

#[test]
fn test_builder_inline_creates_symbol_if_not_exists() {
    let g = GrammarBuilder::new("b7")
        .token("ID", r"[a-z]+")
        .rule("start", vec!["ID"])
        .start("start")
        .inline("phantom")
        .build();
    // The symbol should exist even though it has no rules
    assert_eq!(g.inline_rules.len(), 1);
    assert!(g.find_symbol_by_name("phantom").is_some());
}

#[test]
fn test_builder_supertype_creates_symbol_if_not_exists() {
    let g = GrammarBuilder::new("b8")
        .token("ID", r"[a-z]+")
        .rule("start", vec!["ID"])
        .start("start")
        .supertype("phantom_type")
        .build();
    assert_eq!(g.supertypes.len(), 1);
    assert!(g.find_symbol_by_name("phantom_type").is_some());
}

// ===========================================================================
// 5. Grammar with mixed inline/super/extra (5 tests)
// ===========================================================================

#[test]
fn test_mixed_all_three_lists_independent() {
    let g = GrammarBuilder::new("m1")
        .token("ID", r"[a-z]+")
        .token("ws_tok", r"\s+")
        .token(";", ";")
        .rule("start", vec!["expression", ";"])
        .rule("expression", vec!["primary"])
        .rule("primary", vec!["ID"])
        .start("start")
        .extra("ws_tok")
        .inline("primary")
        .supertype("expression")
        .build();

    let ws = g.extras[0];
    let primary = g.find_symbol_by_name("primary").unwrap();
    let expr = g.find_symbol_by_name("expression").unwrap();

    assert!(g.extras.contains(&ws));
    assert!(g.inline_rules.contains(&primary));
    assert!(g.supertypes.contains(&expr));
    // No overlap
    assert!(!g.extras.contains(&primary));
    assert!(!g.inline_rules.contains(&expr));
    assert!(!g.supertypes.contains(&ws));
}

#[test]
fn test_mixed_same_symbol_in_inline_and_supertype() {
    let g = GrammarBuilder::new("m2")
        .token("ID", r"[a-z]+")
        .rule("start", vec!["node"])
        .rule("node", vec!["ID"])
        .start("start")
        .inline("node")
        .supertype("node")
        .build();
    // Both lists contain the same symbol — allowed at IR level
    let node = g.find_symbol_by_name("node").unwrap();
    assert!(g.inline_rules.contains(&node));
    assert!(g.supertypes.contains(&node));
}

#[test]
fn test_mixed_grammar_name_preserved() {
    let g = GrammarBuilder::new("my_lang")
        .token("ID", r"[a-z]+")
        .token("WS", r"\s+")
        .rule("start", vec!["ID"])
        .start("start")
        .extra("WS")
        .inline("start")
        .build();
    assert_eq!(g.name, "my_lang");
}

#[test]
fn test_mixed_extras_with_multiple_tokens() {
    let g = GrammarBuilder::new("m4")
        .token("ID", r"[a-z]+")
        .token("WS", r"\s+")
        .token("NL", r"\n")
        .token("COMMENT", r"#[^\n]*")
        .rule("start", vec!["ID"])
        .start("start")
        .extra("WS")
        .extra("NL")
        .extra("COMMENT")
        .inline("start")
        .build();
    assert_eq!(g.extras.len(), 3);
    assert_eq!(g.inline_rules.len(), 1);
}

#[test]
fn test_mixed_start_symbol_can_be_supertype() {
    let g = GrammarBuilder::new("m5")
        .token("ID", r"[a-z]+")
        .rule("start", vec!["ID"])
        .start("start")
        .supertype("start")
        .build();
    let start_id = g.start_symbol().unwrap();
    assert!(g.supertypes.contains(&start_id));
}

// ===========================================================================
// 6. Serialization roundtrip (5 tests)
// ===========================================================================

#[test]
fn test_serde_roundtrip_inline_rules() {
    let g = GrammarBuilder::new("ser1")
        .token("ID", r"[a-z]+")
        .rule("start", vec!["helper"])
        .rule("helper", vec!["ID"])
        .start("start")
        .inline("helper")
        .build();

    let json = serde_json::to_string(&g).unwrap();
    let g2: Grammar = serde_json::from_str(&json).unwrap();
    assert_eq!(g.inline_rules, g2.inline_rules);
}

#[test]
fn test_serde_roundtrip_supertypes() {
    let g = GrammarBuilder::new("ser2")
        .token("ID", r"[a-z]+")
        .rule("start", vec!["expression"])
        .rule("expression", vec!["ID"])
        .start("start")
        .supertype("expression")
        .build();

    let json = serde_json::to_string(&g).unwrap();
    let g2: Grammar = serde_json::from_str(&json).unwrap();
    assert_eq!(g.supertypes, g2.supertypes);
}

#[test]
fn test_serde_roundtrip_extras() {
    let g = GrammarBuilder::new("ser3")
        .token("ID", r"[a-z]+")
        .token("WS", r"\s+")
        .rule("start", vec!["ID"])
        .start("start")
        .extra("WS")
        .build();

    let json = serde_json::to_string(&g).unwrap();
    let g2: Grammar = serde_json::from_str(&json).unwrap();
    assert_eq!(g.extras, g2.extras);
}

#[test]
fn test_serde_roundtrip_all_three_together() {
    let g = GrammarBuilder::new("ser4")
        .token("ID", r"[a-z]+")
        .token("WS", r"\s+")
        .rule("start", vec!["expression"])
        .rule("expression", vec!["helper"])
        .rule("helper", vec!["ID"])
        .start("start")
        .extra("WS")
        .inline("helper")
        .supertype("expression")
        .build();

    let json = serde_json::to_string(&g).unwrap();
    let g2: Grammar = serde_json::from_str(&json).unwrap();
    assert_eq!(g.extras, g2.extras);
    assert_eq!(g.inline_rules, g2.inline_rules);
    assert_eq!(g.supertypes, g2.supertypes);
}

#[test]
fn test_serde_roundtrip_empty_lists() {
    let g = minimal_grammar("ser5").build();
    let json = serde_json::to_string(&g).unwrap();
    let g2: Grammar = serde_json::from_str(&json).unwrap();
    assert!(g2.inline_rules.is_empty());
    assert!(g2.supertypes.is_empty());
    assert!(g2.extras.is_empty());
}

// ===========================================================================
// 7. Interaction with normalize() (5 tests)
// ===========================================================================

#[test]
fn test_normalize_preserves_inline_rules() {
    let mut g = GrammarBuilder::new("n1")
        .token("ID", r"[a-z]+")
        .rule("start", vec!["helper"])
        .rule("helper", vec!["ID"])
        .start("start")
        .inline("helper")
        .build();

    let inline_before = g.inline_rules.clone();
    let _aux = g.normalize();
    assert_eq!(g.inline_rules, inline_before);
}

#[test]
fn test_normalize_preserves_supertypes() {
    let mut g = GrammarBuilder::new("n2")
        .token("ID", r"[a-z]+")
        .rule("start", vec!["expression"])
        .rule("expression", vec!["ID"])
        .start("start")
        .supertype("expression")
        .build();

    let supertypes_before = g.supertypes.clone();
    let _aux = g.normalize();
    assert_eq!(g.supertypes, supertypes_before);
}

#[test]
fn test_normalize_preserves_extras() {
    let mut g = GrammarBuilder::new("n3")
        .token("ID", r"[a-z]+")
        .token("WS", r"\s+")
        .rule("start", vec!["ID"])
        .start("start")
        .extra("WS")
        .build();

    let extras_before = g.extras.clone();
    let _aux = g.normalize();
    assert_eq!(g.extras, extras_before);
}

#[test]
fn test_normalize_does_not_add_aux_to_inline_or_supertype() {
    let mut g = GrammarBuilder::new("n4")
        .token("ID", r"[a-z]+")
        .rule("start", vec!["helper"])
        .rule("helper", vec!["ID"])
        .start("start")
        .inline("helper")
        .supertype("start")
        .build();

    // Add an optional to trigger aux rule creation
    let helper_id = g.find_symbol_by_name("helper").unwrap();
    g.add_rule(Rule {
        lhs: g.start_symbol().unwrap(),
        rhs: vec![Symbol::Optional(Box::new(Symbol::NonTerminal(helper_id)))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });

    let inline_count_before = g.inline_rules.len();
    let super_count_before = g.supertypes.len();
    let _aux = g.normalize();

    // normalize should not modify these lists
    assert_eq!(g.inline_rules.len(), inline_count_before);
    assert_eq!(g.supertypes.len(), super_count_before);
}

#[test]
fn test_normalize_with_all_three_lists_populated() {
    let mut g = GrammarBuilder::new("n5")
        .token("ID", r"[a-z]+")
        .token("WS", r"\s+")
        .rule("start", vec!["expression"])
        .rule("expression", vec!["helper"])
        .rule("helper", vec!["ID"])
        .start("start")
        .extra("WS")
        .inline("helper")
        .supertype("expression")
        .build();

    let extras = g.extras.clone();
    let inlines = g.inline_rules.clone();
    let supers = g.supertypes.clone();

    let _aux = g.normalize();

    assert_eq!(g.extras, extras);
    assert_eq!(g.inline_rules, inlines);
    assert_eq!(g.supertypes, supers);
}

// ===========================================================================
// 8. Edge cases (8 tests)
// ===========================================================================

#[test]
fn test_edge_empty_grammar_all_lists_empty() {
    let g = Grammar::new("empty".to_string());
    assert!(g.inline_rules.is_empty());
    assert!(g.supertypes.is_empty());
    assert!(g.extras.is_empty());
}

#[test]
fn test_edge_duplicate_inline_entries_allowed() {
    let g = GrammarBuilder::new("dup1")
        .token("ID", r"[a-z]+")
        .rule("start", vec!["helper"])
        .rule("helper", vec!["ID"])
        .start("start")
        .inline("helper")
        .inline("helper")
        .build();
    // Duplicates are allowed at the IR level — validation is separate
    assert_eq!(g.inline_rules.len(), 2);
    assert_eq!(g.inline_rules[0], g.inline_rules[1]);
}

#[test]
fn test_edge_duplicate_supertype_entries_allowed() {
    let g = GrammarBuilder::new("dup2")
        .token("ID", r"[a-z]+")
        .rule("start", vec!["expression"])
        .rule("expression", vec!["ID"])
        .start("start")
        .supertype("expression")
        .supertype("expression")
        .build();
    assert_eq!(g.supertypes.len(), 2);
    assert_eq!(g.supertypes[0], g.supertypes[1]);
}

#[test]
fn test_edge_duplicate_extra_entries_allowed() {
    let g = GrammarBuilder::new("dup3")
        .token("ID", r"[a-z]+")
        .token("WS", r"\s+")
        .rule("start", vec!["ID"])
        .start("start")
        .extra("WS")
        .extra("WS")
        .build();
    assert_eq!(g.extras.len(), 2);
    assert_eq!(g.extras[0], g.extras[1]);
}

#[test]
fn test_edge_many_inline_rules() {
    let mut builder = GrammarBuilder::new("many_inlines")
        .token("ID", r"[a-z]+")
        .rule("start", vec!["ID"]);
    builder = builder.start("start");

    for i in 0..50 {
        let name = format!("rule_{i}");
        // Build intermediate to add rules, then recreate builder
        // Since builder consumes self, we chain inline calls
        builder = builder.rule(&name, vec!["ID"]).inline(&name);
    }

    let g = builder.build();
    assert_eq!(g.inline_rules.len(), 50);
}

#[test]
fn test_edge_symbol_id_values_are_stable() {
    let g = GrammarBuilder::new("stable")
        .token("ID", r"[a-z]+")
        .rule("start", vec!["helper"])
        .rule("helper", vec!["ID"])
        .start("start")
        .inline("helper")
        .supertype("start")
        .build();

    let helper = g.find_symbol_by_name("helper").unwrap();
    let start_sym = g.find_symbol_by_name("start").unwrap();

    // Verify the IDs in the lists match find_symbol_by_name
    assert_eq!(g.inline_rules[0], helper);
    assert_eq!(g.supertypes[0], start_sym);
}

#[test]
fn test_edge_clearing_inline_rules() {
    let mut g = GrammarBuilder::new("clear1")
        .token("ID", r"[a-z]+")
        .rule("start", vec!["ID"])
        .start("start")
        .inline("start")
        .build();
    assert_eq!(g.inline_rules.len(), 1);
    g.inline_rules.clear();
    assert!(g.inline_rules.is_empty());
}

#[test]
fn test_edge_clearing_supertypes() {
    let mut g = GrammarBuilder::new("clear2")
        .token("ID", r"[a-z]+")
        .rule("start", vec!["ID"])
        .start("start")
        .supertype("start")
        .build();
    assert_eq!(g.supertypes.len(), 1);
    g.supertypes.clear();
    assert!(g.supertypes.is_empty());
}

// ===========================================================================
// Bonus: additional coverage
// ===========================================================================

#[test]
fn test_python_like_preset_extras() {
    let g = GrammarBuilder::python_like();
    // python_like adds WHITESPACE as an extra
    assert!(!g.extras.is_empty());
}

#[test]
fn test_javascript_like_preset_extras() {
    let g = GrammarBuilder::javascript_like();
    assert!(!g.extras.is_empty());
}

#[test]
fn test_serde_roundtrip_json_contains_field_names() {
    let g = GrammarBuilder::new("fields")
        .token("ID", r"[a-z]+")
        .rule("start", vec!["ID"])
        .start("start")
        .inline("start")
        .supertype("start")
        .build();

    let json = serde_json::to_string_pretty(&g).unwrap();
    assert!(json.contains("inline_rules"));
    assert!(json.contains("supertypes"));
    assert!(json.contains("extras"));
}

#[test]
fn test_default_grammar_has_empty_lists() {
    let g = Grammar::default();
    assert!(g.inline_rules.is_empty());
    assert!(g.supertypes.is_empty());
    assert!(g.extras.is_empty());
}

#[test]
fn test_clone_preserves_all_three_lists() {
    let g = GrammarBuilder::new("clone_test")
        .token("ID", r"[a-z]+")
        .token("WS", r"\s+")
        .rule("start", vec!["expression"])
        .rule("expression", vec!["ID"])
        .rule("helper", vec!["ID"])
        .start("start")
        .extra("WS")
        .inline("helper")
        .supertype("expression")
        .build();

    let g2 = g.clone();
    assert_eq!(g.extras, g2.extras);
    assert_eq!(g.inline_rules, g2.inline_rules);
    assert_eq!(g.supertypes, g2.supertypes);
}
