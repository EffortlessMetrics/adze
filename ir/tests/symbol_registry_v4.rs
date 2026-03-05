//! SymbolRegistry v4 — focused on cross-cutting concerns:
//!   registry cloning & equality, builder↔registry coherence, index-map bijectivity,
//!   metadata field combinations, multi-production grammars, ID arithmetic,
//!   and boundary conditions not covered by earlier suites.

use adze_ir::builder::GrammarBuilder;
use adze_ir::symbol_registry::{SymbolInfo, SymbolRegistry};
use adze_ir::{Associativity, Grammar, SymbolId, SymbolMetadata};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn terminal_meta() -> SymbolMetadata {
    SymbolMetadata {
        visible: true,
        named: false,
        hidden: false,
        terminal: true,
    }
}

fn nonterminal_meta() -> SymbolMetadata {
    SymbolMetadata {
        visible: true,
        named: true,
        hidden: false,
        terminal: false,
    }
}

fn hidden_meta() -> SymbolMetadata {
    SymbolMetadata {
        visible: false,
        named: false,
        hidden: true,
        terminal: true,
    }
}

// ===========================================================================
// 1. Clone & equality semantics (5 tests)
// ===========================================================================

#[test]
fn test_clone_registry_equals_original() {
    let mut reg = SymbolRegistry::new();
    reg.register("foo", terminal_meta());
    reg.register("bar", nonterminal_meta());
    let cloned = reg.clone();
    assert_eq!(reg, cloned);
}

#[test]
fn test_clone_registry_independent_mutation() {
    let mut reg = SymbolRegistry::new();
    reg.register("tok", terminal_meta());
    let mut cloned = reg.clone();
    cloned.register("extra", terminal_meta());
    assert_ne!(reg.len(), cloned.len());
}

#[test]
fn test_two_registries_same_symbols_equal() {
    let mut a = SymbolRegistry::new();
    let mut b = SymbolRegistry::new();
    for name in ["x", "y", "z"] {
        a.register(name, terminal_meta());
        b.register(name, terminal_meta());
    }
    assert_eq!(a, b);
}

#[test]
fn test_registries_different_order_not_equal() {
    let mut a = SymbolRegistry::new();
    a.register("first", terminal_meta());
    a.register("second", terminal_meta());

    let mut b = SymbolRegistry::new();
    b.register("second", terminal_meta());
    b.register("first", terminal_meta());
    // Different insertion order → different IDs → not equal
    assert_ne!(a, b);
}

#[test]
fn test_registries_different_metadata_not_equal() {
    let mut a = SymbolRegistry::new();
    a.register("sym", terminal_meta());
    let mut b = SymbolRegistry::new();
    b.register("sym", nonterminal_meta());
    assert_ne!(a, b);
}

// ===========================================================================
// 2. Index-map bijectivity & coverage (6 tests)
// ===========================================================================

#[test]
fn test_index_map_size_matches_len() {
    let mut reg = SymbolRegistry::new();
    for i in 0..10 {
        reg.register(&format!("s{i}"), terminal_meta());
    }
    let idx = reg.to_index_map();
    assert_eq!(idx.len(), reg.len());
}

#[test]
fn test_symbol_map_size_matches_len() {
    let mut reg = SymbolRegistry::new();
    for i in 0..10 {
        reg.register(&format!("s{i}"), terminal_meta());
    }
    let sym = reg.to_symbol_map();
    assert_eq!(sym.len(), reg.len());
}

#[test]
fn test_index_map_eof_has_index_zero() {
    let reg = SymbolRegistry::new();
    let idx = reg.to_index_map();
    assert_eq!(idx[&SymbolId(0)], 0);
}

#[test]
fn test_symbol_map_index_zero_is_eof() {
    let reg = SymbolRegistry::new();
    let sym = reg.to_symbol_map();
    assert_eq!(sym[&0], SymbolId(0));
}

#[test]
fn test_index_map_indices_contiguous() {
    let mut reg = SymbolRegistry::new();
    for i in 0..5 {
        reg.register(&format!("t{i}"), terminal_meta());
    }
    let idx = reg.to_index_map();
    let mut indices: Vec<usize> = idx.values().copied().collect();
    indices.sort();
    let expected: Vec<usize> = (0..reg.len()).collect();
    assert_eq!(indices, expected);
}

#[test]
fn test_index_and_symbol_maps_inverse_on_grammar() {
    let grammar = GrammarBuilder::new("inv")
        .token("A", "a")
        .token("B", "b")
        .rule("r1", vec!["A", "B"])
        .rule("r2", vec!["A"])
        .start("r1")
        .build();
    let reg = grammar.build_registry();
    let idx_map = reg.to_index_map();
    let sym_map = reg.to_symbol_map();
    for (&sid, &idx) in &idx_map {
        assert_eq!(sym_map[&idx], sid);
    }
}

// ===========================================================================
// 3. Metadata field combinations (6 tests)
// ===========================================================================

#[test]
fn test_metadata_all_true() {
    let mut reg = SymbolRegistry::new();
    let meta = SymbolMetadata {
        visible: true,
        named: true,
        hidden: true,
        terminal: true,
    };
    let id = reg.register("all_true", meta);
    let got = reg.get_metadata(id).unwrap();
    assert!(got.visible && got.named && got.hidden && got.terminal);
}

#[test]
fn test_metadata_all_false() {
    let mut reg = SymbolRegistry::new();
    let meta = SymbolMetadata {
        visible: false,
        named: false,
        hidden: false,
        terminal: false,
    };
    let id = reg.register("all_false", meta);
    let got = reg.get_metadata(id).unwrap();
    assert!(!got.visible && !got.named && !got.hidden && !got.terminal);
}

#[test]
fn test_metadata_copy_semantics() {
    let meta = terminal_meta();
    let copy = meta;
    assert_eq!(meta, copy);
}

#[test]
fn test_metadata_update_replaces_all_fields() {
    let mut reg = SymbolRegistry::new();
    reg.register("m", terminal_meta());

    let updated = SymbolMetadata {
        visible: false,
        named: true,
        hidden: true,
        terminal: false,
    };
    reg.register("m", updated);

    let id = reg.get_id("m").unwrap();
    let got = reg.get_metadata(id).unwrap();
    assert_eq!(got, updated);
}

#[test]
fn test_metadata_eof_is_visible_unnamed_unhidden_terminal() {
    let reg = SymbolRegistry::new();
    let meta = reg.get_metadata(SymbolId(0)).unwrap();
    assert!(meta.visible);
    assert!(!meta.named);
    assert!(!meta.hidden);
    assert!(meta.terminal);
}

#[test]
fn test_metadata_hidden_via_underscore_prefix_in_grammar() {
    let mut grammar = Grammar::new("hidden_prefix".to_string());
    let id = SymbolId(50);
    grammar.rule_names.insert(id, "_helper".to_string());
    let reg = grammar.build_registry();
    if let Some(rid) = reg.get_id("_helper") {
        let meta = reg.get_metadata(rid).unwrap();
        assert!(meta.hidden, "underscore-prefixed rule should be hidden");
        assert!(!meta.visible, "underscore-prefixed rule should not be visible");
    }
}

// ===========================================================================
// 4. Multi-production / multi-alternative grammars (5 tests)
// ===========================================================================

#[test]
fn test_multi_alt_same_lhs_single_registry_entry() {
    let grammar = GrammarBuilder::new("multi")
        .token("A", "a")
        .token("B", "b")
        .token("C", "c")
        .rule("item", vec!["A"])
        .rule("item", vec!["B"])
        .rule("item", vec!["C"])
        .start("item")
        .build();
    let reg = grammar.build_registry();
    // "item" should appear once despite 3 productions
    let count = reg.iter().filter(|(n, _)| *n == "item").count();
    assert_eq!(count, 1);
}

#[test]
fn test_multi_level_grammar_all_symbols_registered() {
    let grammar = GrammarBuilder::new("levels")
        .token("NUM", r"\d+")
        .token("+", "+")
        .token("*", "*")
        .rule("factor", vec!["NUM"])
        .rule("term", vec!["factor", "*", "factor"])
        .rule("expr", vec!["term", "+", "term"])
        .start("expr")
        .build();
    let reg = grammar.build_registry();
    for name in ["NUM", "+", "*", "factor", "term", "expr"] {
        assert!(reg.get_id(name).is_some(), "missing symbol: {name}");
    }
}

#[test]
fn test_grammar_with_precedence_registry_unaffected() {
    let grammar = GrammarBuilder::new("prec")
        .token("+", "+")
        .token("*", "*")
        .token("NUM", r"\d+")
        .rule("expr", vec!["expr", "+", "expr"])
        .rule("expr", vec!["expr", "*", "expr"])
        .rule("expr", vec!["NUM"])
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .start("expr")
        .build();
    let reg = grammar.build_registry();
    // Precedence doesn't create extra registry entries
    assert!(reg.get_id("expr").is_some());
    assert!(reg.get_id("+").is_some());
    assert!(reg.get_id("*").is_some());
}

#[test]
fn test_grammar_extras_do_not_create_extra_symbols() {
    let grammar = GrammarBuilder::new("extras")
        .token("WS", r"\s+")
        .token("COMMENT", "//.*")
        .extra("WS")
        .extra("COMMENT")
        .token("ID", r"[a-z]+")
        .rule("r", vec!["ID"])
        .start("r")
        .build();
    let reg = grammar.build_registry();
    // WS and COMMENT are tokens that are also extras — still single entries each
    let ws_count = reg.iter().filter(|(n, _)| *n == "WS").count();
    let cm_count = reg.iter().filter(|(n, _)| *n == "COMMENT").count();
    assert_eq!(ws_count, 1);
    assert_eq!(cm_count, 1);
}

#[test]
fn test_grammar_externals_appear_in_registry() {
    let grammar = GrammarBuilder::new("ext")
        .token("INDENT", "INDENT")
        .token("DEDENT", "DEDENT")
        .external("INDENT")
        .external("DEDENT")
        .token("ID", r"[a-z]+")
        .rule("block", vec!["ID"])
        .start("block")
        .build();
    let reg = grammar.build_registry();
    assert!(reg.get_id("INDENT").is_some());
    assert!(reg.get_id("DEDENT").is_some());
}

// ===========================================================================
// 5. SymbolId arithmetic & properties (6 tests)
// ===========================================================================

#[test]
fn test_symbol_id_inner_value() {
    let id = SymbolId(42);
    assert_eq!(id.0, 42);
}

#[test]
fn test_symbol_id_copy_semantics() {
    let a = SymbolId(7);
    let b = a; // Copy
    assert_eq!(a, b);
}

#[test]
fn test_symbol_id_ord_consistent() {
    let ids: Vec<SymbolId> = (0..10).map(SymbolId).collect();
    for pair in ids.windows(2) {
        assert!(pair[0] < pair[1]);
    }
}

#[test]
fn test_symbol_id_max_u16() {
    let id = SymbolId(u16::MAX);
    assert_eq!(id.0, 65535);
    assert_eq!(format!("{id}"), "Symbol(65535)");
}

#[test]
fn test_symbol_id_zero_display() {
    assert_eq!(format!("{}", SymbolId(0)), "Symbol(0)");
}

#[test]
fn test_symbol_id_hash_deterministic() {
    use std::collections::HashSet;
    let mut set = HashSet::new();
    set.insert(SymbolId(1));
    set.insert(SymbolId(1));
    assert_eq!(set.len(), 1);
}

// ===========================================================================
// 6. SymbolInfo correctness (4 tests)
// ===========================================================================

#[test]
fn test_symbol_info_from_iter_has_correct_id() {
    let mut reg = SymbolRegistry::new();
    let id = reg.register("tok", terminal_meta());
    let info: SymbolInfo = reg
        .iter()
        .find(|(n, _)| *n == "tok")
        .map(|(_, i)| i)
        .unwrap();
    assert_eq!(info.id, id);
}

#[test]
fn test_symbol_info_from_iter_has_correct_metadata() {
    let mut reg = SymbolRegistry::new();
    let meta = hidden_meta();
    reg.register("hidden_sym", meta);
    let info: SymbolInfo = reg
        .iter()
        .find(|(n, _)| *n == "hidden_sym")
        .map(|(_, i)| i)
        .unwrap();
    assert_eq!(info.metadata, meta);
}

#[test]
fn test_symbol_info_eof_entry() {
    let reg = SymbolRegistry::new();
    let (name, info) = reg.iter().next().unwrap();
    assert_eq!(name, "end");
    assert_eq!(info.id, SymbolId(0));
    assert!(info.metadata.terminal);
}

#[test]
fn test_symbol_info_count_matches_len() {
    let mut reg = SymbolRegistry::new();
    for i in 0..5 {
        reg.register(&format!("s{i}"), terminal_meta());
    }
    assert_eq!(reg.iter().count(), reg.len());
}

// ===========================================================================
// 7. builder ↔ registry coherence (6 tests)
// ===========================================================================

#[test]
fn test_build_registry_idempotent() {
    let grammar = GrammarBuilder::new("idem")
        .token("A", "a")
        .rule("r", vec!["A"])
        .start("r")
        .build();
    let r1 = grammar.build_registry();
    let r2 = grammar.build_registry();
    assert_eq!(r1, r2);
}

#[test]
fn test_get_or_build_populates_field() {
    let mut grammar = GrammarBuilder::new("populate")
        .token("X", "x")
        .rule("p", vec!["X"])
        .start("p")
        .build();
    assert!(grammar.symbol_registry.is_none());
    let _ = grammar.get_or_build_registry();
    assert!(grammar.symbol_registry.is_some());
}

#[test]
fn test_get_or_build_returns_same_as_build() {
    let mut grammar = GrammarBuilder::new("same")
        .token("X", "x")
        .rule("p", vec!["X"])
        .start("p")
        .build();
    let built = grammar.build_registry();
    let cached = grammar.get_or_build_registry();
    assert_eq!(&built, cached);
}

#[test]
fn test_find_symbol_by_name_matches_registry() {
    let grammar = GrammarBuilder::new("match")
        .token("N", r"\d+")
        .rule("expr", vec!["N"])
        .start("expr")
        .build();
    // find_symbol_by_name searches rule_names
    let from_find = grammar.find_symbol_by_name("expr");
    assert!(from_find.is_some());
}

#[test]
fn test_find_symbol_by_name_none_for_token() {
    let grammar = GrammarBuilder::new("no_tok")
        .token("NUM", r"\d+")
        .rule("r", vec!["NUM"])
        .start("r")
        .build();
    // find_symbol_by_name only searches rule_names, tokens are not there
    assert!(grammar.find_symbol_by_name("NUM").is_none());
}

#[test]
fn test_builder_inline_and_supertype_in_registry() {
    let grammar = GrammarBuilder::new("inline_super")
        .token("A", "a")
        .token("B", "b")
        .rule("base", vec!["A"])
        .rule("wrapper", vec!["base"])
        .inline("base")
        .supertype("wrapper")
        .start("wrapper")
        .build();
    let reg = grammar.build_registry();
    assert!(reg.get_id("base").is_some());
    assert!(reg.get_id("wrapper").is_some());
}

// ===========================================================================
// 8. Serialization corner cases (5 tests)
// ===========================================================================

#[test]
fn test_serde_empty_grammar_registry_roundtrip() {
    let grammar = Grammar::new("empty".to_string());
    let reg = grammar.build_registry();
    let json = serde_json::to_string(&reg).unwrap();
    let back: SymbolRegistry = serde_json::from_str(&json).unwrap();
    assert_eq!(reg, back);
}

#[test]
fn test_serde_large_registry_roundtrip() {
    let mut reg = SymbolRegistry::new();
    for i in 0..300u16 {
        reg.register(&format!("sym_{i}"), terminal_meta());
    }
    let json = serde_json::to_string(&reg).unwrap();
    let back: SymbolRegistry = serde_json::from_str(&json).unwrap();
    assert_eq!(reg.len(), back.len());
    assert_eq!(back.get_id("sym_0"), Some(SymbolId(1)));
    assert_eq!(back.get_id("sym_299"), Some(SymbolId(300)));
}

#[test]
fn test_serde_registry_preserves_order() {
    let mut reg = SymbolRegistry::new();
    let names = ["zz", "aa", "mm"];
    for n in &names {
        reg.register(n, terminal_meta());
    }
    let json = serde_json::to_string(&reg).unwrap();
    let back: SymbolRegistry = serde_json::from_str(&json).unwrap();
    let orig_order: Vec<&str> = reg.iter().map(|(n, _)| n).collect();
    let back_order: Vec<&str> = back.iter().map(|(n, _)| n).collect();
    assert_eq!(orig_order, back_order);
}

#[test]
fn test_serde_hidden_metadata_roundtrip() {
    let mut reg = SymbolRegistry::new();
    let id = reg.register("_internal", hidden_meta());
    let json = serde_json::to_string(&reg).unwrap();
    let back: SymbolRegistry = serde_json::from_str(&json).unwrap();
    let meta = back.get_metadata(id).unwrap();
    assert!(meta.hidden);
    assert!(!meta.visible);
}

#[test]
fn test_serde_symbol_id_roundtrip() {
    let id = SymbolId(1234);
    let json = serde_json::to_string(&id).unwrap();
    let back: SymbolId = serde_json::from_str(&json).unwrap();
    assert_eq!(id, back);
}

// ===========================================================================
// 9. Boundary & stress conditions (6 tests)
// ===========================================================================

#[test]
fn test_register_unicode_name() {
    let mut reg = SymbolRegistry::new();
    let id = reg.register("αβγ", terminal_meta());
    assert_eq!(reg.get_id("αβγ"), Some(id));
    assert_eq!(reg.get_name(id), Some("αβγ"));
}

#[test]
fn test_register_empty_string_name() {
    let mut reg = SymbolRegistry::new();
    let id = reg.register("", terminal_meta());
    assert_eq!(reg.get_id(""), Some(id));
    assert_eq!(reg.get_name(id), Some(""));
}

#[test]
fn test_register_whitespace_name() {
    let mut reg = SymbolRegistry::new();
    let id = reg.register("  ", terminal_meta());
    assert_eq!(reg.get_id("  "), Some(id));
}

#[test]
fn test_register_special_chars_name() {
    let mut reg = SymbolRegistry::new();
    for name in ["->", "=>", "&&", "||", "!=", "::"] {
        let id = reg.register(name, terminal_meta());
        assert_eq!(reg.get_id(name), Some(id));
    }
}

#[test]
fn test_stress_500_symbols_all_lookups_valid() {
    let mut reg = SymbolRegistry::new();
    for i in 0..500u16 {
        reg.register(&format!("s{i}"), terminal_meta());
    }
    assert_eq!(reg.len(), 501); // 500 + EOF
    for i in 0..500u16 {
        let name = format!("s{i}");
        let id = reg.get_id(&name).expect("should exist");
        assert_eq!(reg.get_name(id), Some(name.as_str()));
    }
}

#[test]
fn test_contains_id_after_many_registrations() {
    let mut reg = SymbolRegistry::new();
    let mut ids = Vec::new();
    for i in 0..20 {
        ids.push(reg.register(&format!("c{i}"), terminal_meta()));
    }
    for id in &ids {
        assert!(reg.contains_id(*id));
    }
    // One past the end should not exist
    assert!(!reg.contains_id(SymbolId(reg.len() as u16 + 100)));
}

// ===========================================================================
// 10. Grammar-level integration (6 tests)
// ===========================================================================

#[test]
fn test_grammar_tokens_count() {
    let grammar = GrammarBuilder::new("tc")
        .token("A", "a")
        .token("B", "b")
        .token("C", "c")
        .rule("r", vec!["A"])
        .start("r")
        .build();
    assert_eq!(grammar.tokens.len(), 3);
}

#[test]
fn test_grammar_rules_count() {
    let grammar = GrammarBuilder::new("rc")
        .token("X", "x")
        .rule("a", vec!["X"])
        .rule("b", vec!["X"])
        .start("a")
        .build();
    assert_eq!(grammar.rules.len(), 2);
}

#[test]
fn test_grammar_rule_names_contains_all_lhs() {
    let grammar = GrammarBuilder::new("lhs")
        .token("T", "t")
        .rule("alpha", vec!["T"])
        .rule("beta", vec!["T"])
        .start("alpha")
        .build();
    let names: Vec<&str> = grammar.rule_names.values().map(|s| s.as_str()).collect();
    assert!(names.contains(&"alpha"));
    assert!(names.contains(&"beta"));
}

#[test]
fn test_grammar_start_symbol_accessible() {
    let grammar = GrammarBuilder::new("ss")
        .token("T", "t")
        .rule("program", vec!["T"])
        .start("program")
        .build();
    let start = grammar.start_symbol();
    assert!(start.is_some());
}

#[test]
fn test_grammar_python_like_preset_has_registry_symbols() {
    let grammar = GrammarBuilder::python_like();
    let reg = grammar.build_registry();
    // Must have at least EOF plus some symbols
    assert!(reg.len() > 1);
    assert_eq!(reg.get_id("end"), Some(SymbolId(0)));
}

#[test]
fn test_grammar_javascript_like_preset_has_registry_symbols() {
    let grammar = GrammarBuilder::javascript_like();
    let reg = grammar.build_registry();
    assert!(reg.len() > 1);
    assert!(reg.get_id("NUMBER").is_some());
}
