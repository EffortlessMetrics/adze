//! Advanced comprehensive tests for `SymbolRegistry`.
//!
//! 80+ tests covering: creation, lookup, registration, SymbolInfo fields,
//! registry from Grammar, determinism, large registries, Unicode names,
//! duplicate handling, Debug/Clone traits, and iteration edge cases.

use adze_ir::builder::GrammarBuilder;
use adze_ir::symbol_registry::{SymbolInfo, SymbolRegistry};
use adze_ir::{SymbolId, SymbolMetadata};
use std::collections::{HashMap, HashSet};

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

// ===================================================================
// 1. SymbolRegistry creation and basic operations
// ===================================================================

#[test]
fn new_registry_contains_eof() {
    let reg = SymbolRegistry::new();
    assert!(reg.get_id("end").is_some());
    assert_eq!(reg.get_id("end"), Some(SymbolId(0)));
}

#[test]
fn new_registry_len_is_one() {
    let reg = SymbolRegistry::new();
    assert_eq!(reg.len(), 1); // only "end"
}

#[test]
fn new_registry_is_not_empty() {
    let reg = SymbolRegistry::new();
    assert!(!reg.is_empty());
}

#[test]
fn default_equals_new() {
    let a = SymbolRegistry::new();
    let b = SymbolRegistry::default();
    assert_eq!(a, b);
}

#[test]
fn eof_metadata_is_terminal_visible() {
    let reg = SymbolRegistry::new();
    let meta = reg.get_metadata(SymbolId(0)).unwrap();
    assert!(meta.visible);
    assert!(!meta.named);
    assert!(!meta.hidden);
    assert!(meta.terminal);
}

// ===================================================================
// 2. Symbol lookup by name and by ID
// ===================================================================

#[test]
fn lookup_nonexistent_name_returns_none() {
    let reg = SymbolRegistry::new();
    assert_eq!(reg.get_id("nonexistent"), None);
}

#[test]
fn lookup_nonexistent_id_returns_none() {
    let reg = SymbolRegistry::new();
    assert_eq!(reg.get_name(SymbolId(999)), None);
}

#[test]
fn lookup_after_register_by_name() {
    let mut reg = SymbolRegistry::new();
    let id = reg.register("foo", terminal_meta());
    assert_eq!(reg.get_id("foo"), Some(id));
}

#[test]
fn lookup_after_register_by_id() {
    let mut reg = SymbolRegistry::new();
    let id = reg.register("bar", terminal_meta());
    assert_eq!(reg.get_name(id), Some("bar"));
}

#[test]
fn get_metadata_for_registered_symbol() {
    let mut reg = SymbolRegistry::new();
    let id = reg.register("expr", nonterminal_meta());
    let meta = reg.get_metadata(id).unwrap();
    assert!(meta.named);
    assert!(!meta.terminal);
}

#[test]
fn get_metadata_for_missing_id_returns_none() {
    let reg = SymbolRegistry::new();
    assert!(reg.get_metadata(SymbolId(42)).is_none());
}

#[test]
fn contains_id_true_for_eof() {
    let reg = SymbolRegistry::new();
    assert!(reg.contains_id(SymbolId(0)));
}

#[test]
fn contains_id_false_for_unregistered() {
    let reg = SymbolRegistry::new();
    assert!(!reg.contains_id(SymbolId(100)));
}

#[test]
fn contains_id_true_after_register() {
    let mut reg = SymbolRegistry::new();
    let id = reg.register("x", terminal_meta());
    assert!(reg.contains_id(id));
}

// ===================================================================
// 3. Symbol registration (tokens, nonterminals)
// ===================================================================

#[test]
fn register_assigns_sequential_ids() {
    let mut reg = SymbolRegistry::new();
    let a = reg.register("a", terminal_meta());
    let b = reg.register("b", terminal_meta());
    let c = reg.register("c", terminal_meta());
    assert_eq!(a.0 + 1, b.0);
    assert_eq!(b.0 + 1, c.0);
}

#[test]
fn first_user_symbol_is_id_1() {
    let mut reg = SymbolRegistry::new();
    let id = reg.register("first", terminal_meta());
    assert_eq!(id, SymbolId(1));
}

#[test]
fn register_terminal_and_nonterminal() {
    let mut reg = SymbolRegistry::new();
    let tok = reg.register("plus", terminal_meta());
    let nt = reg.register("expr", nonterminal_meta());
    assert!(reg.get_metadata(tok).unwrap().terminal);
    assert!(!reg.get_metadata(nt).unwrap().terminal);
}

#[test]
fn register_hidden_symbol() {
    let mut reg = SymbolRegistry::new();
    let id = reg.register("_ws", hidden_meta());
    let meta = reg.get_metadata(id).unwrap();
    assert!(meta.hidden);
    assert!(!meta.visible);
}

#[test]
fn register_many_symbols_increments_len() {
    let mut reg = SymbolRegistry::new();
    for i in 0..50 {
        reg.register(&format!("sym_{i}"), terminal_meta());
    }
    assert_eq!(reg.len(), 51); // 50 + "end"
}

// ===================================================================
// 4. SymbolInfo fields
// ===================================================================

#[test]
fn symbol_info_id_matches_registered() {
    let mut reg = SymbolRegistry::new();
    let id = reg.register("tok", terminal_meta());
    let info = reg.iter().find(|(n, _)| *n == "tok").unwrap().1;
    assert_eq!(info.id, id);
}

#[test]
fn symbol_info_metadata_matches() {
    let mut reg = SymbolRegistry::new();
    let meta = nonterminal_meta();
    reg.register("rule", meta);
    let info = reg.iter().find(|(n, _)| *n == "rule").unwrap().1;
    assert_eq!(info.metadata, meta);
}

#[test]
fn symbol_info_for_eof() {
    let reg = SymbolRegistry::new();
    let (name, info) = reg.iter().next().unwrap();
    assert_eq!(name, "end");
    assert_eq!(info.id, SymbolId(0));
    assert!(info.metadata.terminal);
}

#[test]
fn symbol_info_clone() {
    let info = SymbolInfo {
        id: SymbolId(5),
        metadata: terminal_meta(),
    };
    let cloned = info;
    assert_eq!(cloned.id, info.id);
    assert_eq!(cloned.metadata, info.metadata);
}

#[test]
fn symbol_info_debug_contains_id() {
    let info = SymbolInfo {
        id: SymbolId(7),
        metadata: terminal_meta(),
    };
    let dbg = format!("{info:?}");
    assert!(
        dbg.contains("7"),
        "Debug output should contain the ID: {dbg}"
    );
}

#[test]
fn symbol_info_debug_contains_metadata() {
    let info = SymbolInfo {
        id: SymbolId(3),
        metadata: nonterminal_meta(),
    };
    let dbg = format!("{info:?}");
    assert!(
        dbg.contains("metadata"),
        "Debug output should mention metadata: {dbg}"
    );
}

// ===================================================================
// 5. Registry from Grammar (builder → build → registry access)
// ===================================================================

#[test]
fn grammar_registry_contains_tokens() {
    let mut grammar = GrammarBuilder::new("test")
        .token("number", r"\d+")
        .token("plus", r"\+")
        .rule("expr", vec!["number", "plus", "number"])
        .start("expr")
        .build();
    let reg = grammar.get_or_build_registry();
    assert!(reg.get_id("number").is_some());
    assert!(reg.get_id("plus").is_some());
}

#[test]
fn grammar_registry_contains_nonterminals() {
    let mut grammar = GrammarBuilder::new("test")
        .token("number", r"\d+")
        .rule("expr", vec!["number"])
        .start("expr")
        .build();
    let reg = grammar.get_or_build_registry();
    assert!(reg.get_id("expr").is_some());
}

#[test]
fn grammar_registry_has_eof() {
    let mut grammar = GrammarBuilder::new("minimal")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let reg = grammar.get_or_build_registry();
    assert_eq!(reg.get_id("end"), Some(SymbolId(0)));
}

#[test]
fn grammar_registry_tokens_are_terminal() {
    let mut grammar = GrammarBuilder::new("test")
        .token("num", r"\d+")
        .rule("e", vec!["num"])
        .start("e")
        .build();
    let reg = grammar.get_or_build_registry();
    let id = reg.get_id("num").unwrap();
    assert!(reg.get_metadata(id).unwrap().terminal);
}

#[test]
fn grammar_registry_rules_are_nonterminal() {
    let mut grammar = GrammarBuilder::new("test")
        .token("num", r"\d+")
        .rule("expr", vec!["num"])
        .start("expr")
        .build();
    let reg = grammar.get_or_build_registry();
    let id = reg.get_id("expr").unwrap();
    assert!(!reg.get_metadata(id).unwrap().terminal);
}

#[test]
fn grammar_registry_underscore_prefixed_hidden() {
    let mut grammar = GrammarBuilder::new("test")
        .token("a", "a")
        .rule("_internal", vec!["a"])
        .start("_internal")
        .build();
    let reg = grammar.get_or_build_registry();
    let id = reg.get_id("_internal").unwrap();
    let meta = reg.get_metadata(id).unwrap();
    assert!(meta.hidden);
    assert!(!meta.visible);
}

#[test]
fn build_registry_is_idempotent() {
    let grammar = GrammarBuilder::new("test")
        .token("x", "x")
        .rule("s", vec!["x"])
        .start("s")
        .build();
    let r1 = grammar.build_registry();
    let r2 = grammar.build_registry();
    assert_eq!(r1, r2);
}

// ===================================================================
// 6. Registry determinism
// ===================================================================

#[test]
fn same_order_produces_same_ids() {
    let names = ["alpha", "beta", "gamma", "delta"];
    let mut r1 = SymbolRegistry::new();
    let mut r2 = SymbolRegistry::new();
    for name in &names {
        r1.register(name, terminal_meta());
        r2.register(name, terminal_meta());
    }
    for name in &names {
        assert_eq!(r1.get_id(name), r2.get_id(name));
    }
}

#[test]
fn different_order_produces_different_ids() {
    let mut r1 = SymbolRegistry::new();
    let mut r2 = SymbolRegistry::new();
    r1.register("a", terminal_meta());
    r1.register("b", terminal_meta());
    r2.register("b", terminal_meta());
    r2.register("a", terminal_meta());
    // "a" gets different IDs in different orders
    assert_ne!(r1.get_id("a"), r2.get_id("a"));
}

#[test]
fn iteration_order_matches_insertion() {
    let mut reg = SymbolRegistry::new();
    let names = ["first", "second", "third"];
    for n in &names {
        reg.register(n, terminal_meta());
    }
    let iter_names: Vec<&str> = reg.iter().map(|(n, _)| n).collect();
    assert_eq!(iter_names[0], "end"); // eof always first
    assert_eq!(iter_names[1], "first");
    assert_eq!(iter_names[2], "second");
    assert_eq!(iter_names[3], "third");
}

#[test]
fn to_index_map_preserves_insertion_order() {
    let mut reg = SymbolRegistry::new();
    let id_a = reg.register("a", terminal_meta());
    let id_b = reg.register("b", terminal_meta());
    let map = reg.to_index_map();
    // "end" is index 0, "a" is index 1, "b" is index 2
    assert_eq!(map[&SymbolId(0)], 0);
    assert_eq!(map[&id_a], 1);
    assert_eq!(map[&id_b], 2);
}

#[test]
fn to_symbol_map_is_inverse_of_to_index_map() {
    let mut reg = SymbolRegistry::new();
    reg.register("x", terminal_meta());
    reg.register("y", terminal_meta());
    let idx_map = reg.to_index_map();
    let sym_map = reg.to_symbol_map();
    for (&sym_id, &idx) in &idx_map {
        assert_eq!(sym_map[&idx], sym_id);
    }
}

#[test]
fn clone_registry_equals_original() {
    let mut reg = SymbolRegistry::new();
    reg.register("tok", terminal_meta());
    let cloned = reg.clone();
    assert_eq!(reg, cloned);
}

// ===================================================================
// 7. Large registry (many symbols)
// ===================================================================

#[test]
fn register_200_symbols() {
    let mut reg = SymbolRegistry::new();
    for i in 0..200 {
        reg.register(&format!("sym_{i}"), terminal_meta());
    }
    assert_eq!(reg.len(), 201); // 200 + "end"
}

#[test]
fn large_registry_all_ids_unique() {
    let mut reg = SymbolRegistry::new();
    let mut ids = HashSet::new();
    for i in 0..300 {
        let id = reg.register(&format!("s{i}"), terminal_meta());
        assert!(ids.insert(id), "Duplicate ID for s{i}");
    }
}

#[test]
fn large_registry_all_lookups_work() {
    let mut reg = SymbolRegistry::new();
    let mut expected: Vec<(String, SymbolId)> = Vec::new();
    for i in 0..150 {
        let name = format!("sym_{i}");
        let id = reg.register(&name, terminal_meta());
        expected.push((name, id));
    }
    for (name, id) in &expected {
        assert_eq!(reg.get_id(name), Some(*id));
        assert_eq!(reg.get_name(*id), Some(name.as_str()));
    }
}

#[test]
fn large_registry_index_map_has_correct_size() {
    let mut reg = SymbolRegistry::new();
    for i in 0..100 {
        reg.register(&format!("t{i}"), terminal_meta());
    }
    assert_eq!(reg.to_index_map().len(), 101);
}

#[test]
fn large_registry_symbol_map_has_correct_size() {
    let mut reg = SymbolRegistry::new();
    for i in 0..100 {
        reg.register(&format!("t{i}"), terminal_meta());
    }
    assert_eq!(reg.to_symbol_map().len(), 101);
}

// ===================================================================
// 8. Unicode symbol names
// ===================================================================

#[test]
fn unicode_name_registered_and_looked_up() {
    let mut reg = SymbolRegistry::new();
    let id = reg.register("αβγ", terminal_meta());
    assert_eq!(reg.get_id("αβγ"), Some(id));
    assert_eq!(reg.get_name(id), Some("αβγ"));
}

#[test]
fn emoji_name() {
    let mut reg = SymbolRegistry::new();
    let id = reg.register("🎉", terminal_meta());
    assert_eq!(reg.get_id("🎉"), Some(id));
}

#[test]
fn cjk_name() {
    let mut reg = SymbolRegistry::new();
    let id = reg.register("变量", terminal_meta());
    assert_eq!(reg.get_name(id), Some("变量"));
}

#[test]
fn mixed_ascii_unicode_names() {
    let mut reg = SymbolRegistry::new();
    let a = reg.register("café", terminal_meta());
    let b = reg.register("cafe", terminal_meta());
    assert_ne!(a, b);
    assert_eq!(reg.get_id("café"), Some(a));
    assert_eq!(reg.get_id("cafe"), Some(b));
}

#[test]
fn empty_string_name() {
    let mut reg = SymbolRegistry::new();
    let id = reg.register("", terminal_meta());
    assert_eq!(reg.get_id(""), Some(id));
    assert_eq!(reg.get_name(id), Some(""));
}

#[test]
fn whitespace_name() {
    let mut reg = SymbolRegistry::new();
    let id = reg.register(" ", terminal_meta());
    assert_eq!(reg.get_id(" "), Some(id));
}

#[test]
fn name_with_special_chars() {
    let mut reg = SymbolRegistry::new();
    let id = reg.register("a->b", terminal_meta());
    assert_eq!(reg.get_name(id), Some("a->b"));
}

// ===================================================================
// 9. Duplicate symbol handling
// ===================================================================

#[test]
fn duplicate_register_returns_same_id() {
    let mut reg = SymbolRegistry::new();
    let id1 = reg.register("dup", terminal_meta());
    let id2 = reg.register("dup", terminal_meta());
    assert_eq!(id1, id2);
}

#[test]
fn duplicate_register_does_not_increase_len() {
    let mut reg = SymbolRegistry::new();
    reg.register("x", terminal_meta());
    let len_before = reg.len();
    reg.register("x", terminal_meta());
    assert_eq!(reg.len(), len_before);
}

#[test]
fn duplicate_register_updates_metadata() {
    let mut reg = SymbolRegistry::new();
    let id = reg.register("sym", terminal_meta());
    let new_meta = nonterminal_meta();
    reg.register("sym", new_meta);
    assert_eq!(reg.get_metadata(id).unwrap(), new_meta);
}

#[test]
fn duplicate_eof_returns_id_zero() {
    let mut reg = SymbolRegistry::new();
    let id = reg.register("end", terminal_meta());
    assert_eq!(id, SymbolId(0));
}

#[test]
fn duplicate_does_not_affect_other_symbols() {
    let mut reg = SymbolRegistry::new();
    let a = reg.register("a", terminal_meta());
    let b = reg.register("b", terminal_meta());
    reg.register("a", nonterminal_meta()); // re-register "a"
    assert_eq!(reg.get_id("b"), Some(b));
    assert_eq!(reg.get_metadata(b).unwrap(), terminal_meta());
    assert_eq!(reg.get_id("a"), Some(a));
}

#[test]
fn multiple_duplicates_same_id() {
    let mut reg = SymbolRegistry::new();
    let id1 = reg.register("r", terminal_meta());
    let id2 = reg.register("r", nonterminal_meta());
    let id3 = reg.register("r", hidden_meta());
    assert_eq!(id1, id2);
    assert_eq!(id2, id3);
    // metadata should reflect last registration
    assert_eq!(reg.get_metadata(id1).unwrap(), hidden_meta());
}

// ===================================================================
// 10. SymbolInfo Debug/Clone traits
// ===================================================================

#[test]
fn symbol_info_copy_semantics() {
    let info = SymbolInfo {
        id: SymbolId(10),
        metadata: terminal_meta(),
    };
    let copied = info;
    // Both should be usable (Copy)
    assert_eq!(info.id, copied.id);
}

#[test]
fn symbol_info_debug_format_is_nonempty() {
    let info = SymbolInfo {
        id: SymbolId(0),
        metadata: terminal_meta(),
    };
    assert!(!format!("{info:?}").is_empty());
}

#[test]
fn symbol_info_eq_same_values() {
    let a = SymbolInfo {
        id: SymbolId(5),
        metadata: terminal_meta(),
    };
    let b = SymbolInfo {
        id: SymbolId(5),
        metadata: terminal_meta(),
    };
    assert_eq!(a, b);
}

#[test]
fn symbol_info_ne_different_id() {
    let a = SymbolInfo {
        id: SymbolId(1),
        metadata: terminal_meta(),
    };
    let b = SymbolInfo {
        id: SymbolId(2),
        metadata: terminal_meta(),
    };
    assert_ne!(a, b);
}

#[test]
fn symbol_info_ne_different_metadata() {
    let a = SymbolInfo {
        id: SymbolId(1),
        metadata: terminal_meta(),
    };
    let b = SymbolInfo {
        id: SymbolId(1),
        metadata: nonterminal_meta(),
    };
    assert_ne!(a, b);
}

// ===================================================================
// 11. SymbolRegistry Debug trait
// ===================================================================

#[test]
fn registry_debug_is_nonempty() {
    let reg = SymbolRegistry::new();
    let dbg = format!("{reg:?}");
    assert!(!dbg.is_empty());
}

#[test]
fn registry_debug_contains_struct_name() {
    let reg = SymbolRegistry::new();
    let dbg = format!("{reg:?}");
    assert!(
        dbg.contains("SymbolRegistry"),
        "Debug should mention SymbolRegistry: {dbg}"
    );
}

#[test]
fn registry_debug_with_symbols() {
    let mut reg = SymbolRegistry::new();
    reg.register("my_token", terminal_meta());
    let dbg = format!("{reg:?}");
    assert!(
        dbg.contains("my_token"),
        "Debug should contain registered symbol name: {dbg}"
    );
}

#[test]
fn registry_clone_is_independent() {
    let mut reg = SymbolRegistry::new();
    reg.register("original", terminal_meta());
    let mut cloned = reg.clone();
    cloned.register("only_in_clone", terminal_meta());
    assert!(reg.get_id("only_in_clone").is_none());
    assert!(cloned.get_id("only_in_clone").is_some());
}

#[test]
fn registry_eq_after_same_operations() {
    let mut a = SymbolRegistry::new();
    let mut b = SymbolRegistry::new();
    a.register("x", terminal_meta());
    b.register("x", terminal_meta());
    assert_eq!(a, b);
}

#[test]
fn registry_ne_different_symbols() {
    let mut a = SymbolRegistry::new();
    let mut b = SymbolRegistry::new();
    a.register("x", terminal_meta());
    b.register("y", terminal_meta());
    assert_ne!(a, b);
}

// ===================================================================
// 12. Registry iteration
// ===================================================================

#[test]
fn iter_empty_registry_has_eof_only() {
    let reg = SymbolRegistry::new();
    let items: Vec<_> = reg.iter().collect();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].0, "end");
}

#[test]
fn iter_count_matches_len() {
    let mut reg = SymbolRegistry::new();
    for i in 0..10 {
        reg.register(&format!("s{i}"), terminal_meta());
    }
    assert_eq!(reg.iter().count(), reg.len());
}

#[test]
fn iter_all_names_present() {
    let mut reg = SymbolRegistry::new();
    let names = ["apple", "banana", "cherry"];
    for n in &names {
        reg.register(n, terminal_meta());
    }
    let iter_names: HashSet<&str> = reg.iter().map(|(n, _)| n).collect();
    for n in &names {
        assert!(iter_names.contains(n));
    }
    assert!(iter_names.contains("end"));
}

#[test]
fn iter_all_ids_match_lookup() {
    let mut reg = SymbolRegistry::new();
    reg.register("p", terminal_meta());
    reg.register("q", nonterminal_meta());
    for (name, info) in reg.iter() {
        assert_eq!(reg.get_id(name), Some(info.id));
    }
}

#[test]
fn iter_metadata_matches_get_metadata() {
    let mut reg = SymbolRegistry::new();
    reg.register("tok", terminal_meta());
    reg.register("nt", nonterminal_meta());
    for (_name, info) in reg.iter() {
        assert_eq!(reg.get_metadata(info.id), Some(info.metadata));
    }
}

#[test]
fn to_index_map_ids_match_iter_order() {
    let mut reg = SymbolRegistry::new();
    reg.register("m", terminal_meta());
    reg.register("n", terminal_meta());
    let idx_map = reg.to_index_map();
    for (i, (_name, info)) in reg.iter().enumerate() {
        assert_eq!(idx_map[&info.id], i);
    }
}

#[test]
fn to_symbol_map_indices_match_iter_order() {
    let mut reg = SymbolRegistry::new();
    reg.register("u", terminal_meta());
    reg.register("v", terminal_meta());
    let sym_map = reg.to_symbol_map();
    for (i, (_name, info)) in reg.iter().enumerate() {
        assert_eq!(sym_map[&i], info.id);
    }
}

// ===================================================================
// Additional edge cases
// ===================================================================

#[test]
fn register_after_clone_diverges() {
    let mut reg = SymbolRegistry::new();
    reg.register("shared", terminal_meta());
    let mut clone = reg.clone();
    let id_orig = reg.register("only_orig", terminal_meta());
    let id_clone = clone.register("only_clone", terminal_meta());
    // Both get the same next ID since they diverged from the same state
    assert_eq!(id_orig.0, id_clone.0);
    // But they refer to different names
    assert_eq!(reg.get_name(id_orig), Some("only_orig"));
    assert_eq!(clone.get_name(id_clone), Some("only_clone"));
}

#[test]
fn index_map_and_symbol_map_same_size() {
    let mut reg = SymbolRegistry::new();
    for i in 0..20 {
        reg.register(&format!("s{i}"), terminal_meta());
    }
    assert_eq!(reg.to_index_map().len(), reg.to_symbol_map().len());
}

#[test]
fn all_metadata_combinations() {
    let mut reg = SymbolRegistry::new();
    // Enumerate all 16 combinations of 4 bools
    for visible in [false, true] {
        for named in [false, true] {
            for hidden in [false, true] {
                for term in [false, true] {
                    let name = format!("s_{visible}_{named}_{hidden}_{term}");
                    let meta = SymbolMetadata {
                        visible,
                        named,
                        hidden,
                        terminal: term,
                    };
                    let id = reg.register(&name, meta);
                    assert_eq!(reg.get_metadata(id).unwrap(), meta);
                }
            }
        }
    }
    assert_eq!(reg.len(), 17); // 16 + "end"
}

#[test]
fn symbol_id_ordering() {
    assert!(SymbolId(0) < SymbolId(1));
    assert!(SymbolId(100) > SymbolId(50));
    assert_eq!(SymbolId(5), SymbolId(5));
}

#[test]
fn symbol_id_hash_consistency() {
    let mut set = HashSet::new();
    set.insert(SymbolId(10));
    assert!(set.contains(&SymbolId(10)));
    assert!(!set.contains(&SymbolId(11)));
}

#[test]
fn symbol_metadata_eq() {
    assert_eq!(terminal_meta(), terminal_meta());
    assert_ne!(terminal_meta(), nonterminal_meta());
}

#[test]
fn symbol_metadata_clone() {
    let meta = terminal_meta();
    let cloned = meta;
    assert_eq!(meta, cloned);
}

#[test]
fn symbol_metadata_debug_nonempty() {
    let meta = terminal_meta();
    assert!(!format!("{meta:?}").is_empty());
}

#[test]
fn registry_serde_roundtrip() {
    let mut reg = SymbolRegistry::new();
    reg.register("tok_a", terminal_meta());
    reg.register("rule_b", nonterminal_meta());
    let json = serde_json::to_string(&reg).unwrap();
    let deserialized: SymbolRegistry = serde_json::from_str(&json).unwrap();
    assert_eq!(reg, deserialized);
}

#[test]
fn registry_serde_preserves_ids() {
    let mut reg = SymbolRegistry::new();
    let id = reg.register("x", terminal_meta());
    let json = serde_json::to_string(&reg).unwrap();
    let deserialized: SymbolRegistry = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.get_id("x"), Some(id));
}

#[test]
fn registry_serde_preserves_metadata() {
    let mut reg = SymbolRegistry::new();
    let meta = hidden_meta();
    let id = reg.register("h", meta);
    let json = serde_json::to_string(&reg).unwrap();
    let deserialized: SymbolRegistry = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.get_metadata(id), Some(meta));
}

#[test]
fn names_with_dots_and_slashes() {
    let mut reg = SymbolRegistry::new();
    let a = reg.register("a.b.c", terminal_meta());
    let b = reg.register("x/y/z", terminal_meta());
    assert_eq!(reg.get_name(a), Some("a.b.c"));
    assert_eq!(reg.get_name(b), Some("x/y/z"));
}

#[test]
fn very_long_name() {
    let mut reg = SymbolRegistry::new();
    let long_name = "a".repeat(1000);
    let id = reg.register(&long_name, terminal_meta());
    assert_eq!(reg.get_name(id), Some(long_name.as_str()));
}

#[test]
fn grammar_builder_multiple_rules_registry() {
    let mut grammar = GrammarBuilder::new("calc")
        .token("num", r"\d+")
        .token("plus", r"\+")
        .token("star", r"\*")
        .rule("expr", vec!["term", "plus", "term"])
        .rule("term", vec!["num"])
        .start("expr")
        .build();
    let reg = grammar.get_or_build_registry();
    // All symbols should be resolvable
    for name in &["num", "plus", "star", "expr", "term", "end"] {
        assert!(reg.get_id(name).is_some(), "Missing symbol: {name}");
    }
}

#[test]
fn grammar_registry_len_covers_all_symbols() {
    let mut grammar = GrammarBuilder::new("g")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a", "b"])
        .start("s")
        .build();
    let reg = grammar.get_or_build_registry();
    // "end" + "a" + "b" + "s" = 4
    assert!(reg.len() >= 4);
}

#[test]
fn get_or_build_registry_caches() {
    let mut grammar = GrammarBuilder::new("g")
        .token("x", "x")
        .rule("s", vec!["x"])
        .start("s")
        .build();
    let r1_id = grammar.get_or_build_registry().get_id("x");
    let r2_id = grammar.get_or_build_registry().get_id("x");
    assert_eq!(r1_id, r2_id);
}

#[test]
fn iter_collected_as_hashmap() {
    let mut reg = SymbolRegistry::new();
    reg.register("k1", terminal_meta());
    reg.register("k2", nonterminal_meta());
    let map: HashMap<&str, SymbolInfo> = reg.iter().collect();
    assert!(map.contains_key("k1"));
    assert!(map.contains_key("k2"));
    assert!(map.contains_key("end"));
}

#[test]
fn consecutive_ids_no_gaps() {
    let mut reg = SymbolRegistry::new();
    let mut ids = Vec::new();
    ids.push(SymbolId(0)); // "end"
    for i in 0..10 {
        ids.push(reg.register(&format!("s{i}"), terminal_meta()));
    }
    for window in ids.windows(2) {
        assert_eq!(
            window[0].0 + 1,
            window[1].0,
            "Gap between {:?} and {:?}",
            window[0],
            window[1]
        );
    }
}

#[test]
fn to_index_map_values_are_contiguous() {
    let mut reg = SymbolRegistry::new();
    for i in 0..5 {
        reg.register(&format!("t{i}"), terminal_meta());
    }
    let map = reg.to_index_map();
    let mut indices: Vec<usize> = map.values().copied().collect();
    indices.sort();
    for (i, &idx) in indices.iter().enumerate() {
        assert_eq!(i, idx, "Index map should be contiguous");
    }
}

#[test]
fn to_symbol_map_keys_are_contiguous() {
    let mut reg = SymbolRegistry::new();
    for i in 0..5 {
        reg.register(&format!("t{i}"), terminal_meta());
    }
    let map = reg.to_symbol_map();
    let mut keys: Vec<usize> = map.keys().copied().collect();
    keys.sort();
    for (i, &k) in keys.iter().enumerate() {
        assert_eq!(i, k, "Symbol map keys should be contiguous");
    }
}

#[test]
fn register_preserves_existing_symbols() {
    let mut reg = SymbolRegistry::new();
    let id_a = reg.register("a", terminal_meta());
    reg.register("b", terminal_meta());
    reg.register("c", terminal_meta());
    // "a" should still be at its original id
    assert_eq!(reg.get_id("a"), Some(id_a));
    assert_eq!(reg.get_name(id_a), Some("a"));
}

#[test]
fn metadata_all_false() {
    let meta = SymbolMetadata {
        visible: false,
        named: false,
        hidden: false,
        terminal: false,
    };
    let mut reg = SymbolRegistry::new();
    let id = reg.register("ghost", meta);
    assert_eq!(reg.get_metadata(id).unwrap(), meta);
}

#[test]
fn metadata_all_true() {
    let meta = SymbolMetadata {
        visible: true,
        named: true,
        hidden: true,
        terminal: true,
    };
    let mut reg = SymbolRegistry::new();
    let id = reg.register("everything", meta);
    assert_eq!(reg.get_metadata(id).unwrap(), meta);
}
