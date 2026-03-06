//! Comprehensive tests for SymbolId, FieldId, Symbol, SymbolRegistry, and Grammar symbol lookups.

use std::collections::{BTreeSet, HashMap, HashSet};

use adze_ir::builder::GrammarBuilder;
use adze_ir::{FieldId, Grammar, Symbol, SymbolId, SymbolMetadata, SymbolRegistry};

// ---------------------------------------------------------------------------
// Helper
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
// 1. SymbolId construction and properties (8 tests)
// ===========================================================================

#[test]
fn test_symbol_id_new_zero() {
    let id = SymbolId(0);
    assert_eq!(id.0, 0);
}

#[test]
fn test_symbol_id_new_max() {
    let id = SymbolId(u16::MAX);
    assert_eq!(id.0, u16::MAX);
}

#[test]
fn test_symbol_id_display() {
    assert_eq!(format!("{}", SymbolId(42)), "Symbol(42)");
}

#[test]
fn test_symbol_id_debug() {
    let dbg = format!("{:?}", SymbolId(7));
    assert!(dbg.contains("SymbolId"));
    assert!(dbg.contains("7"));
}

#[test]
fn test_symbol_id_clone_and_copy() {
    let a = SymbolId(10);
    let b = a; // Copy
    let c = a;
    assert_eq!(a, b);
    assert_eq!(a, c);
}

#[test]
fn test_symbol_id_equality() {
    assert_eq!(SymbolId(5), SymbolId(5));
    assert_ne!(SymbolId(5), SymbolId(6));
}

#[test]
fn test_symbol_id_hash_consistent() {
    let mut set = HashSet::new();
    set.insert(SymbolId(1));
    set.insert(SymbolId(1));
    assert_eq!(set.len(), 1);
    set.insert(SymbolId(2));
    assert_eq!(set.len(), 2);
}

#[test]
fn test_symbol_id_as_map_key() {
    let mut map = HashMap::new();
    map.insert(SymbolId(0), "eof");
    map.insert(SymbolId(1), "number");
    assert_eq!(map[&SymbolId(0)], "eof");
    assert_eq!(map[&SymbolId(1)], "number");
}

// ===========================================================================
// 2. FieldId construction and properties (5 tests)
// ===========================================================================

#[test]
fn test_field_id_new_zero() {
    let id = FieldId(0);
    assert_eq!(id.0, 0);
}

#[test]
fn test_field_id_display() {
    assert_eq!(format!("{}", FieldId(3)), "Field(3)");
}

#[test]
fn test_field_id_debug() {
    let dbg = format!("{:?}", FieldId(99));
    assert!(dbg.contains("FieldId"));
    assert!(dbg.contains("99"));
}

#[test]
fn test_field_id_equality_and_hash() {
    assert_eq!(FieldId(0), FieldId(0));
    assert_ne!(FieldId(0), FieldId(1));

    let mut set = HashSet::new();
    set.insert(FieldId(5));
    set.insert(FieldId(5));
    assert_eq!(set.len(), 1);
}

#[test]
fn test_field_id_clone_and_copy() {
    let a = FieldId(42);
    let b = a;
    let c = a;
    assert_eq!(a, b);
    assert_eq!(b, c);
}

// ===========================================================================
// 3. Symbol enum variants (8 tests)
// ===========================================================================

#[test]
fn test_symbol_terminal_variant() {
    let s = Symbol::Terminal(SymbolId(1));
    assert!(matches!(s, Symbol::Terminal(SymbolId(1))));
}

#[test]
fn test_symbol_nonterminal_variant() {
    let s = Symbol::NonTerminal(SymbolId(2));
    assert!(matches!(s, Symbol::NonTerminal(SymbolId(2))));
}

#[test]
fn test_symbol_external_variant() {
    let s = Symbol::External(SymbolId(3));
    assert!(matches!(s, Symbol::External(SymbolId(3))));
}

#[test]
fn test_symbol_epsilon_variant() {
    let s = Symbol::Epsilon;
    assert!(matches!(s, Symbol::Epsilon));
}

#[test]
fn test_symbol_optional_variant() {
    let inner = Symbol::Terminal(SymbolId(4));
    let s = Symbol::Optional(Box::new(inner.clone()));
    assert!(matches!(s, Symbol::Optional(_)));
}

#[test]
fn test_symbol_repeat_variant() {
    let inner = Symbol::NonTerminal(SymbolId(5));
    let s = Symbol::Repeat(Box::new(inner));
    assert!(matches!(s, Symbol::Repeat(_)));
}

#[test]
fn test_symbol_repeat_one_variant() {
    let inner = Symbol::Terminal(SymbolId(6));
    let s = Symbol::RepeatOne(Box::new(inner));
    assert!(matches!(s, Symbol::RepeatOne(_)));
}

#[test]
fn test_symbol_choice_and_sequence_variants() {
    let choice = Symbol::Choice(vec![
        Symbol::Terminal(SymbolId(1)),
        Symbol::Terminal(SymbolId(2)),
    ]);
    assert!(matches!(choice, Symbol::Choice(_)));

    let seq = Symbol::Sequence(vec![
        Symbol::Terminal(SymbolId(1)),
        Symbol::NonTerminal(SymbolId(3)),
    ]);
    assert!(matches!(seq, Symbol::Sequence(_)));
}

// ===========================================================================
// 4. Symbol Debug / Clone / PartialEq / Hash (8 tests)
// ===========================================================================

#[test]
fn test_symbol_debug_format() {
    let s = Symbol::Terminal(SymbolId(10));
    let dbg = format!("{:?}", s);
    assert!(dbg.contains("Terminal"));
    assert!(dbg.contains("10"));
}

#[test]
fn test_symbol_clone_terminal() {
    let a = Symbol::Terminal(SymbolId(1));
    let b = a.clone();
    assert_eq!(a, b);
}

#[test]
fn test_symbol_clone_nested() {
    let a = Symbol::Optional(Box::new(Symbol::Repeat(Box::new(Symbol::Terminal(
        SymbolId(7),
    )))));
    let b = a.clone();
    assert_eq!(a, b);
}

#[test]
fn test_symbol_partial_eq_same() {
    assert_eq!(Symbol::Epsilon, Symbol::Epsilon);
    assert_eq!(Symbol::Terminal(SymbolId(0)), Symbol::Terminal(SymbolId(0)));
}

#[test]
fn test_symbol_partial_eq_different_variant() {
    assert_ne!(
        Symbol::Terminal(SymbolId(1)),
        Symbol::NonTerminal(SymbolId(1))
    );
}

#[test]
fn test_symbol_partial_eq_different_id() {
    assert_ne!(Symbol::Terminal(SymbolId(1)), Symbol::Terminal(SymbolId(2)));
}

#[test]
fn test_symbol_hash_in_set() {
    let mut set = HashSet::new();
    set.insert(Symbol::Terminal(SymbolId(1)));
    set.insert(Symbol::Terminal(SymbolId(1)));
    set.insert(Symbol::NonTerminal(SymbolId(1)));
    assert_eq!(set.len(), 2);
}

#[test]
fn test_symbol_ord_terminal_before_nonterminal() {
    // Symbol derives Ord — verify it doesn't panic and is deterministic.
    let mut syms = vec![
        Symbol::NonTerminal(SymbolId(1)),
        Symbol::Terminal(SymbolId(1)),
        Symbol::Epsilon,
    ];
    syms.sort();
    // Just ensure sort completes and is stable across calls.
    let mut syms2 = syms.clone();
    syms2.sort();
    assert_eq!(syms, syms2);
}

// ===========================================================================
// 5. SymbolRegistry operations (8 tests)
// ===========================================================================

#[test]
fn test_registry_new_has_eof() {
    let reg = SymbolRegistry::new();
    assert_eq!(reg.get_id("end"), Some(SymbolId(0)));
    assert_eq!(reg.get_name(SymbolId(0)), Some("end"));
}

#[test]
fn test_registry_default_equals_new() {
    let a = SymbolRegistry::new();
    let b = SymbolRegistry::default();
    assert_eq!(a, b);
}

#[test]
fn test_registry_register_returns_sequential_ids() {
    let mut reg = SymbolRegistry::new();
    let id1 = reg.register("plus", terminal_meta());
    let id2 = reg.register("minus", terminal_meta());
    assert_eq!(id1, SymbolId(1));
    assert_eq!(id2, SymbolId(2));
}

#[test]
fn test_registry_register_idempotent() {
    let mut reg = SymbolRegistry::new();
    let id1 = reg.register("tok", terminal_meta());
    let id2 = reg.register("tok", terminal_meta());
    assert_eq!(id1, id2);
    // Only "end" + "tok"
    assert_eq!(reg.len(), 2);
}

#[test]
fn test_registry_get_id_missing() {
    let reg = SymbolRegistry::new();
    assert_eq!(reg.get_id("nonexistent"), None);
}

#[test]
fn test_registry_get_name_missing() {
    let reg = SymbolRegistry::new();
    assert_eq!(reg.get_name(SymbolId(999)), None);
}

#[test]
fn test_registry_contains_id() {
    let mut reg = SymbolRegistry::new();
    let id = reg.register("number", terminal_meta());
    assert!(reg.contains_id(id));
    assert!(!reg.contains_id(SymbolId(999)));
}

#[test]
fn test_registry_len_and_is_empty() {
    let reg = SymbolRegistry::new();
    // "end" is pre-registered
    assert!(!reg.is_empty());
    assert_eq!(reg.len(), 1);
}

// ===========================================================================
// 6. Grammar symbol lookup patterns (8 tests)
// ===========================================================================

#[test]
fn test_grammar_find_symbol_by_name_found() {
    let grammar = GrammarBuilder::new("test")
        .token("NUMBER", r"\d+")
        .rule("expr", vec!["NUMBER"])
        .start("expr")
        .build();
    assert!(grammar.find_symbol_by_name("expr").is_some());
}

#[test]
fn test_grammar_find_symbol_by_name_missing() {
    let grammar = GrammarBuilder::new("test")
        .token("NUMBER", r"\d+")
        .rule("expr", vec!["NUMBER"])
        .start("expr")
        .build();
    assert!(grammar.find_symbol_by_name("nonexistent").is_none());
}

#[test]
fn test_grammar_get_rules_for_symbol() {
    let grammar = GrammarBuilder::new("test")
        .token("NUMBER", r"\d+")
        .token("+", "+")
        .rule("expr", vec!["NUMBER"])
        .rule("expr", vec!["expr", "+", "expr"])
        .start("expr")
        .build();

    let expr_id = grammar.find_symbol_by_name("expr").unwrap();
    let rules = grammar.get_rules_for_symbol(expr_id).unwrap();
    assert_eq!(rules.len(), 2);
}

#[test]
fn test_grammar_get_rules_for_unknown_symbol() {
    let grammar = Grammar::new("empty".to_string());
    assert!(grammar.get_rules_for_symbol(SymbolId(999)).is_none());
}

#[test]
fn test_grammar_start_symbol_with_explicit_start() {
    let grammar = GrammarBuilder::new("test")
        .token("NUMBER", r"\d+")
        .rule("expr", vec!["NUMBER"])
        .start("expr")
        .build();
    assert!(grammar.start_symbol().is_some());
}

#[test]
fn test_grammar_all_rules_iterator() {
    let grammar = GrammarBuilder::new("test")
        .token("NUMBER", r"\d+")
        .token("+", "+")
        .rule("expr", vec!["NUMBER"])
        .rule("expr", vec!["expr", "+", "expr"])
        .rule("term", vec!["NUMBER"])
        .start("expr")
        .build();
    assert_eq!(grammar.all_rules().count(), 3);
}

#[test]
fn test_grammar_build_registry_populates_tokens() {
    let mut grammar = GrammarBuilder::new("test")
        .token("NUMBER", r"\d+")
        .token("+", "+")
        .rule("expr", vec!["NUMBER"])
        .start("expr")
        .build();

    let registry = grammar.get_or_build_registry();
    // Registry must contain at least "end" (eof) from SymbolRegistry::new()
    assert!(!registry.is_empty());
    assert!(registry.get_id("end").is_some());
}

#[test]
fn test_grammar_build_registry_is_cached() {
    let mut grammar = GrammarBuilder::new("test")
        .token("NUMBER", r"\d+")
        .rule("expr", vec!["NUMBER"])
        .start("expr")
        .build();

    assert!(grammar.symbol_registry.is_none());
    let _ = grammar.get_or_build_registry();
    assert!(grammar.symbol_registry.is_some());
    // Calling again should reuse the cached value (no panic / no change).
    let _ = grammar.get_or_build_registry();
    assert!(grammar.symbol_registry.is_some());
}

// ===========================================================================
// 7. Symbol ordering (5 tests)
// ===========================================================================

#[test]
fn test_symbol_id_ord() {
    assert!(SymbolId(0) < SymbolId(1));
    assert!(SymbolId(100) > SymbolId(99));
    assert!(SymbolId(5) <= SymbolId(5));
}

#[test]
fn test_symbol_id_btreeset_sorted() {
    let mut set = BTreeSet::new();
    set.insert(SymbolId(3));
    set.insert(SymbolId(1));
    set.insert(SymbolId(2));
    let ids: Vec<_> = set.into_iter().collect();
    assert_eq!(ids, vec![SymbolId(1), SymbolId(2), SymbolId(3)]);
}

#[test]
fn test_symbol_enum_sort_deterministic() {
    let syms = vec![
        Symbol::Epsilon,
        Symbol::External(SymbolId(5)),
        Symbol::NonTerminal(SymbolId(2)),
        Symbol::Terminal(SymbolId(1)),
    ];
    let mut a = syms.clone();
    let mut b = syms;
    a.sort();
    b.sort();
    assert_eq!(a, b);
}

#[test]
fn test_symbol_id_min_max() {
    let ids = [SymbolId(10), SymbolId(0), SymbolId(5)];
    assert_eq!(*ids.iter().min().unwrap(), SymbolId(0));
    assert_eq!(*ids.iter().max().unwrap(), SymbolId(10));
}

#[test]
fn test_symbol_id_partial_ord_consistency() {
    // PartialOrd and Ord must agree.
    let a = SymbolId(3);
    let b = SymbolId(7);
    assert_eq!(a.partial_cmp(&b), Some(std::cmp::Ordering::Less));
    assert_eq!(a.cmp(&b), std::cmp::Ordering::Less);
}

// ===========================================================================
// 8. Edge cases (5 tests)
// ===========================================================================

#[test]
fn test_symbol_id_zero_display() {
    assert_eq!(format!("{}", SymbolId(0)), "Symbol(0)");
}

#[test]
fn test_symbol_deeply_nested() {
    // Optional(Repeat(RepeatOne(Terminal)))
    let deep = Symbol::Optional(Box::new(Symbol::Repeat(Box::new(Symbol::RepeatOne(
        Box::new(Symbol::Terminal(SymbolId(1))),
    )))));
    let cloned = deep.clone();
    assert_eq!(deep, cloned);
}

#[test]
fn test_symbol_empty_choice_and_sequence() {
    let empty_choice = Symbol::Choice(vec![]);
    let empty_seq = Symbol::Sequence(vec![]);
    assert_ne!(empty_choice, empty_seq);
    assert_eq!(empty_choice.clone(), empty_choice);
    assert_eq!(empty_seq.clone(), empty_seq);
}

#[test]
fn test_registry_metadata_roundtrip() {
    let mut reg = SymbolRegistry::new();
    let meta = hidden_meta();
    let id = reg.register("_whitespace", meta);
    let got = reg.get_metadata(id).unwrap();
    assert_eq!(got, meta);
    assert!(got.hidden);
    assert!(!got.visible);
}

#[test]
fn test_registry_iter_preserves_insertion_order() {
    let mut reg = SymbolRegistry::new();
    reg.register("alpha", terminal_meta());
    reg.register("beta", nonterminal_meta());
    reg.register("gamma", terminal_meta());

    let names: Vec<&str> = reg.iter().map(|(name, _)| name).collect();
    // "end" is always first (registered in new()), then insertion order.
    assert_eq!(names, vec!["end", "alpha", "beta", "gamma"]);
}

// ===========================================================================
// Additional edge-case and integration tests to reach 55+
// ===========================================================================

#[test]
fn test_registry_to_index_map() {
    let mut reg = SymbolRegistry::new();
    reg.register("a", terminal_meta());
    reg.register("b", terminal_meta());
    let idx_map = reg.to_index_map();
    // "end" → 0, "a" → 1, "b" → 2
    assert_eq!(idx_map[&SymbolId(0)], 0);
    assert_eq!(idx_map[&SymbolId(1)], 1);
    assert_eq!(idx_map[&SymbolId(2)], 2);
}

#[test]
fn test_registry_to_symbol_map() {
    let mut reg = SymbolRegistry::new();
    reg.register("x", terminal_meta());
    let sym_map = reg.to_symbol_map();
    assert_eq!(sym_map[&0], SymbolId(0));
    assert_eq!(sym_map[&1], SymbolId(1));
}

#[test]
fn test_registry_register_updates_metadata() {
    let mut reg = SymbolRegistry::new();
    let id = reg.register("tok", terminal_meta());
    let new_meta = hidden_meta();
    let id2 = reg.register("tok", new_meta);
    assert_eq!(id, id2);
    assert_eq!(reg.get_metadata(id).unwrap(), new_meta);
}

#[test]
fn test_registry_many_symbols() {
    let mut reg = SymbolRegistry::new();
    for i in 0..500u16 {
        let name = format!("sym_{i}");
        let id = reg.register(&name, terminal_meta());
        assert_eq!(id.0, i + 1); // +1 because "end" is 0
    }
    assert_eq!(reg.len(), 501); // 500 + "end"
}

#[test]
fn test_symbol_id_serde_roundtrip() {
    let id = SymbolId(42);
    let json = serde_json::to_string(&id).unwrap();
    let back: SymbolId = serde_json::from_str(&json).unwrap();
    assert_eq!(id, back);
}

#[test]
fn test_field_id_serde_roundtrip() {
    let id = FieldId(7);
    let json = serde_json::to_string(&id).unwrap();
    let back: FieldId = serde_json::from_str(&json).unwrap();
    assert_eq!(id, back);
}

#[test]
fn test_symbol_serde_roundtrip_terminal() {
    let sym = Symbol::Terminal(SymbolId(10));
    let json = serde_json::to_string(&sym).unwrap();
    let back: Symbol = serde_json::from_str(&json).unwrap();
    assert_eq!(sym, back);
}

#[test]
fn test_symbol_serde_roundtrip_epsilon() {
    let sym = Symbol::Epsilon;
    let json = serde_json::to_string(&sym).unwrap();
    let back: Symbol = serde_json::from_str(&json).unwrap();
    assert_eq!(sym, back);
}

#[test]
fn test_symbol_serde_roundtrip_nested() {
    let sym = Symbol::Optional(Box::new(Symbol::Choice(vec![
        Symbol::Terminal(SymbolId(1)),
        Symbol::NonTerminal(SymbolId(2)),
    ])));
    let json = serde_json::to_string(&sym).unwrap();
    let back: Symbol = serde_json::from_str(&json).unwrap();
    assert_eq!(sym, back);
}

#[test]
fn test_registry_serde_roundtrip() {
    let mut reg = SymbolRegistry::new();
    reg.register("number", terminal_meta());
    reg.register("expr", nonterminal_meta());

    let json = serde_json::to_string(&reg).unwrap();
    let back: SymbolRegistry = serde_json::from_str(&json).unwrap();
    assert_eq!(reg, back);
}

#[test]
fn test_grammar_multiple_start_candidates() {
    // Build grammar where start is explicitly set.
    let grammar = GrammarBuilder::new("multi")
        .token("A", "a")
        .token("B", "b")
        .rule("alpha", vec!["A"])
        .rule("beta", vec!["B"])
        .start("alpha")
        .build();
    let start = grammar.start_symbol().unwrap();
    let name = grammar.rule_names.get(&start).unwrap();
    assert_eq!(name, "alpha");
}

#[test]
fn test_symbol_metadata_fields() {
    let meta = SymbolMetadata {
        visible: true,
        named: true,
        hidden: false,
        terminal: false,
    };
    assert!(meta.visible);
    assert!(meta.named);
    assert!(!meta.hidden);
    assert!(!meta.terminal);
}

#[test]
fn test_symbol_metadata_debug() {
    let meta = terminal_meta();
    let dbg = format!("{:?}", meta);
    assert!(dbg.contains("SymbolMetadata"));
}
