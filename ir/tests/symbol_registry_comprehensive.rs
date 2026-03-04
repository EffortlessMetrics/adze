//! Comprehensive tests for `SymbolRegistry`.
//!
//! Covers: creation, allocation, name resolution, pre-registered symbols,
//! duplicate handling, edge cases, capacity, iteration, mapping, serde,
//! clone/eq semantics, and property-based invariants.

use adze_ir::symbol_registry::{SymbolInfo, SymbolRegistry};
use adze_ir::{SymbolId, SymbolMetadata};
use std::collections::{HashMap, HashSet};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn terminal() -> SymbolMetadata {
    SymbolMetadata {
        visible: true,
        named: false,
        hidden: false,
        terminal: true,
    }
}

fn nonterminal() -> SymbolMetadata {
    SymbolMetadata {
        visible: true,
        named: true,
        hidden: false,
        terminal: false,
    }
}

fn hidden_terminal() -> SymbolMetadata {
    SymbolMetadata {
        visible: false,
        named: false,
        hidden: true,
        terminal: true,
    }
}

// ===========================================================================
// 1. Creation & pre-registered symbols
// ===========================================================================

#[test]
fn new_registry_contains_eof() {
    let reg = SymbolRegistry::new();
    assert_eq!(reg.len(), 1);
    assert!(!reg.is_empty());
    assert_eq!(reg.get_id("end"), Some(SymbolId(0)));
}

#[test]
fn eof_name_is_end() {
    let reg = SymbolRegistry::new();
    assert_eq!(reg.get_name(SymbolId(0)), Some("end"));
}

#[test]
fn eof_metadata_is_visible_unnamed_terminal() {
    let reg = SymbolRegistry::new();
    let meta = reg.get_metadata(SymbolId(0)).unwrap();
    assert!(meta.visible);
    assert!(!meta.named);
    assert!(!meta.hidden);
    assert!(meta.terminal);
}

#[test]
fn default_equals_new() {
    let a = SymbolRegistry::new();
    let b = SymbolRegistry::default();
    assert_eq!(a, b);
}

// ===========================================================================
// 2. Basic registration & allocation
// ===========================================================================

#[test]
fn first_user_symbol_gets_id_one() {
    let mut reg = SymbolRegistry::new();
    let id = reg.register("number", terminal());
    assert_eq!(id, SymbolId(1));
}

#[test]
fn sequential_ids_after_eof() {
    let mut reg = SymbolRegistry::new();
    let names = ["a", "b", "c", "d", "e"];
    for (i, &name) in names.iter().enumerate() {
        let id = reg.register(name, terminal());
        assert_eq!(id, SymbolId((i as u16) + 1));
    }
}

#[test]
fn len_increases_with_each_new_symbol() {
    let mut reg = SymbolRegistry::new();
    assert_eq!(reg.len(), 1); // EOF only
    reg.register("x", terminal());
    assert_eq!(reg.len(), 2);
    reg.register("y", terminal());
    assert_eq!(reg.len(), 3);
}

#[test]
fn contains_id_for_registered_symbol() {
    let mut reg = SymbolRegistry::new();
    let id = reg.register("tok", terminal());
    assert!(reg.contains_id(id));
}

#[test]
fn contains_id_false_for_unregistered() {
    let reg = SymbolRegistry::new();
    assert!(!reg.contains_id(SymbolId(42)));
    assert!(!reg.contains_id(SymbolId(u16::MAX)));
}

// ===========================================================================
// 3. Name resolution (name → id, id → name)
// ===========================================================================

#[test]
fn get_id_returns_correct_id() {
    let mut reg = SymbolRegistry::new();
    let id = reg.register("identifier", nonterminal());
    assert_eq!(reg.get_id("identifier"), Some(id));
}

#[test]
fn get_name_returns_correct_name() {
    let mut reg = SymbolRegistry::new();
    let id = reg.register("plus", terminal());
    assert_eq!(reg.get_name(id), Some("plus"));
}

#[test]
fn get_id_returns_none_for_unknown() {
    let reg = SymbolRegistry::new();
    assert_eq!(reg.get_id("unknown"), None);
}

#[test]
fn get_name_returns_none_for_unknown() {
    let reg = SymbolRegistry::new();
    assert_eq!(reg.get_name(SymbolId(999)), None);
}

#[test]
fn get_metadata_returns_none_for_unknown() {
    let reg = SymbolRegistry::new();
    assert_eq!(reg.get_metadata(SymbolId(999)), None);
}

#[test]
fn roundtrip_name_id_name() {
    let mut reg = SymbolRegistry::new();
    let id = reg.register("expr", nonterminal());
    let name = reg.get_name(id).unwrap();
    assert_eq!(reg.get_id(name), Some(id));
}

#[test]
fn roundtrip_id_name_id() {
    let mut reg = SymbolRegistry::new();
    reg.register("stmt", nonterminal());
    let id = reg.get_id("stmt").unwrap();
    assert_eq!(reg.get_name(id), Some("stmt"));
}

// ===========================================================================
// 4. Duplicate name handling
// ===========================================================================

#[test]
fn duplicate_registration_returns_same_id() {
    let mut reg = SymbolRegistry::new();
    let id1 = reg.register("tok", terminal());
    let id2 = reg.register("tok", terminal());
    assert_eq!(id1, id2);
}

#[test]
fn duplicate_registration_does_not_increase_len() {
    let mut reg = SymbolRegistry::new();
    reg.register("tok", terminal());
    let len_before = reg.len();
    reg.register("tok", terminal());
    assert_eq!(reg.len(), len_before);
}

#[test]
fn duplicate_registration_updates_metadata() {
    let mut reg = SymbolRegistry::new();
    let id = reg.register("tok", terminal());
    let new_meta = nonterminal();
    reg.register("tok", new_meta);
    assert_eq!(reg.get_metadata(id), Some(new_meta));
}

#[test]
fn duplicate_registration_preserves_name_lookup() {
    let mut reg = SymbolRegistry::new();
    let id = reg.register("tok", terminal());
    reg.register("tok", nonterminal());
    assert_eq!(reg.get_id("tok"), Some(id));
    assert_eq!(reg.get_name(id), Some("tok"));
}

#[test]
fn reregister_eof_updates_metadata() {
    let mut reg = SymbolRegistry::new();
    let custom = hidden_terminal();
    let id = reg.register("end", custom);
    assert_eq!(id, SymbolId(0));
    assert_eq!(reg.get_metadata(id), Some(custom));
    assert_eq!(reg.len(), 1); // no extra symbol created
}

// ===========================================================================
// 5. Edge cases: empty names, special characters
// ===========================================================================

#[test]
fn empty_string_name_can_be_registered() {
    let mut reg = SymbolRegistry::new();
    let id = reg.register("", terminal());
    assert_eq!(reg.get_id(""), Some(id));
    assert_eq!(reg.get_name(id), Some(""));
}

#[test]
fn special_character_names() {
    let mut reg = SymbolRegistry::new();
    for name in ["+", "-", "*", "/", "==", "!=", "&&", "||", "(", ")", ";"] {
        let id = reg.register(name, terminal());
        assert_eq!(reg.get_id(name), Some(id));
        assert_eq!(reg.get_name(id), Some(name));
    }
}

#[test]
fn unicode_names() {
    let mut reg = SymbolRegistry::new();
    let id = reg.register("αβγ", terminal());
    assert_eq!(reg.get_id("αβγ"), Some(id));
    assert_eq!(reg.get_name(id), Some("αβγ"));
}

#[test]
fn whitespace_in_name() {
    let mut reg = SymbolRegistry::new();
    let id = reg.register("hello world", terminal());
    assert_eq!(reg.get_id("hello world"), Some(id));
}

#[test]
fn very_long_name() {
    let mut reg = SymbolRegistry::new();
    let long_name = "x".repeat(10_000);
    let id = reg.register(&long_name, terminal());
    assert_eq!(reg.get_id(&long_name), Some(id));
    assert_eq!(reg.get_name(id), Some(long_name.as_str()));
}

// ===========================================================================
// 6. Metadata variants
// ===========================================================================

#[test]
fn all_metadata_combinations_stored_correctly() {
    let mut reg = SymbolRegistry::new();
    let combos: Vec<SymbolMetadata> = (0..16)
        .map(|bits| SymbolMetadata {
            visible: bits & 1 != 0,
            named: bits & 2 != 0,
            hidden: bits & 4 != 0,
            terminal: bits & 8 != 0,
        })
        .collect();

    for (i, meta) in combos.iter().enumerate() {
        let name = format!("sym_{i}");
        let id = reg.register(&name, *meta);
        assert_eq!(reg.get_metadata(id), Some(*meta));
    }
}

#[test]
fn mixed_terminal_and_nonterminal() {
    let mut reg = SymbolRegistry::new();
    let t = reg.register("+", terminal());
    let nt = reg.register("expression", nonterminal());
    assert!(reg.get_metadata(t).unwrap().terminal);
    assert!(!reg.get_metadata(nt).unwrap().terminal);
    assert!(!reg.get_metadata(t).unwrap().named);
    assert!(reg.get_metadata(nt).unwrap().named);
}

// ===========================================================================
// 7. Iteration
// ===========================================================================

#[test]
fn iter_starts_with_eof() {
    let reg = SymbolRegistry::new();
    let (name, info) = reg.iter().next().unwrap();
    assert_eq!(name, "end");
    assert_eq!(info.id, SymbolId(0));
}

#[test]
fn iter_preserves_insertion_order() {
    let mut reg = SymbolRegistry::new();
    let names = ["alpha", "beta", "gamma"];
    for &n in &names {
        reg.register(n, terminal());
    }
    let collected: Vec<&str> = reg.iter().map(|(n, _)| n).collect();
    assert_eq!(collected, vec!["end", "alpha", "beta", "gamma"]);
}

#[test]
fn iter_count_matches_len() {
    let mut reg = SymbolRegistry::new();
    for i in 0..10 {
        reg.register(&format!("s{i}"), terminal());
    }
    assert_eq!(reg.iter().count(), reg.len());
}

#[test]
fn iter_symbol_info_ids_match_lookups() {
    let mut reg = SymbolRegistry::new();
    for name in ["x", "y", "z"] {
        reg.register(name, nonterminal());
    }
    for (name, info) in reg.iter() {
        assert_eq!(reg.get_id(name), Some(info.id));
        assert_eq!(reg.get_metadata(info.id), Some(info.metadata));
    }
}

#[test]
fn iter_all_registered_ids_present() {
    let mut reg = SymbolRegistry::new();
    let mut expected_ids: HashSet<SymbolId> = HashSet::new();
    expected_ids.insert(SymbolId(0));
    for i in 0..5 {
        let id = reg.register(&format!("t{i}"), terminal());
        expected_ids.insert(id);
    }
    let iter_ids: HashSet<SymbolId> = reg.iter().map(|(_, info)| info.id).collect();
    assert_eq!(expected_ids, iter_ids);
}

// ===========================================================================
// 8. to_index_map / to_symbol_map
// ===========================================================================

#[test]
fn index_map_size_matches_len() {
    let mut reg = SymbolRegistry::new();
    for n in ["a", "b", "c"] {
        reg.register(n, terminal());
    }
    assert_eq!(reg.to_index_map().len(), reg.len());
}

#[test]
fn symbol_map_size_matches_len() {
    let mut reg = SymbolRegistry::new();
    for n in ["a", "b", "c"] {
        reg.register(n, terminal());
    }
    assert_eq!(reg.to_symbol_map().len(), reg.len());
}

#[test]
fn index_map_indices_are_contiguous() {
    let mut reg = SymbolRegistry::new();
    for n in ["p", "q", "r", "s"] {
        reg.register(n, terminal());
    }
    let idx_map = reg.to_index_map();
    let mut indices: Vec<usize> = idx_map.values().copied().collect();
    indices.sort();
    let expected: Vec<usize> = (0..reg.len()).collect();
    assert_eq!(indices, expected);
}

#[test]
fn index_map_and_symbol_map_are_inverses() {
    let mut reg = SymbolRegistry::new();
    for n in ["a", "b", "c"] {
        reg.register(n, terminal());
    }
    let idx_map = reg.to_index_map();
    let sym_map = reg.to_symbol_map();
    for (&sym_id, &idx) in &idx_map {
        assert_eq!(sym_map.get(&idx), Some(&sym_id));
    }
    for (&idx, &sym_id) in &sym_map {
        assert_eq!(idx_map.get(&sym_id), Some(&idx));
    }
}

#[test]
fn index_map_eof_present() {
    let reg = SymbolRegistry::new();
    let idx_map = reg.to_index_map();
    assert!(idx_map.contains_key(&SymbolId(0)));
}

#[test]
fn symbol_map_index_zero_present() {
    let reg = SymbolRegistry::new();
    let sym_map = reg.to_symbol_map();
    assert!(sym_map.contains_key(&0));
}

// ===========================================================================
// 9. Clone semantics
// ===========================================================================

#[test]
fn clone_equals_original() {
    let mut reg = SymbolRegistry::new();
    reg.register("tok", terminal());
    let cloned = reg.clone();
    assert_eq!(reg, cloned);
}

#[test]
fn clone_is_independent() {
    let mut reg = SymbolRegistry::new();
    reg.register("a", terminal());
    let original_len = reg.len();
    let mut cloned = reg.clone();
    cloned.register("b", terminal());
    assert_eq!(reg.len(), original_len);
    assert_eq!(reg.get_id("b"), None);
}

#[test]
fn clone_preserves_all_lookups() {
    let mut reg = SymbolRegistry::new();
    for n in ["x", "y", "z"] {
        reg.register(n, nonterminal());
    }
    let cloned = reg.clone();
    for n in ["end", "x", "y", "z"] {
        assert_eq!(reg.get_id(n), cloned.get_id(n));
    }
}

// ===========================================================================
// 10. PartialEq
// ===========================================================================

#[test]
fn eq_same_registrations_same_order() {
    let meta = terminal();
    let mut r1 = SymbolRegistry::new();
    let mut r2 = SymbolRegistry::new();
    for n in ["a", "b", "c"] {
        r1.register(n, meta);
        r2.register(n, meta);
    }
    assert_eq!(r1, r2);
}

#[test]
fn ne_different_metadata() {
    let mut r1 = SymbolRegistry::new();
    let mut r2 = SymbolRegistry::new();
    r1.register("tok", terminal());
    r2.register("tok", nonterminal());
    assert_ne!(r1, r2);
}

#[test]
fn ne_different_symbols() {
    let mut r1 = SymbolRegistry::new();
    let mut r2 = SymbolRegistry::new();
    r1.register("foo", terminal());
    r2.register("bar", terminal());
    assert_ne!(r1, r2);
}

#[test]
fn ne_different_lengths() {
    let mut r1 = SymbolRegistry::new();
    let mut r2 = SymbolRegistry::new();
    r1.register("a", terminal());
    r1.register("b", terminal());
    r2.register("a", terminal());
    assert_ne!(r1, r2);
}

#[test]
fn eq_reflexive() {
    let mut reg = SymbolRegistry::new();
    reg.register("a", terminal());
    assert_eq!(reg, reg);
}

// ===========================================================================
// 11. Determinism
// ===========================================================================

#[test]
fn deterministic_ids_across_independent_registries() {
    let names = ["number", "plus", "minus", "expr", "term"];
    let mut r1 = SymbolRegistry::new();
    let mut r2 = SymbolRegistry::new();
    for &n in &names {
        r1.register(n, terminal());
        r2.register(n, terminal());
    }
    for &n in &names {
        assert_eq!(r1.get_id(n), r2.get_id(n));
    }
}

#[test]
fn ids_are_monotonically_increasing() {
    let mut reg = SymbolRegistry::new();
    let mut prev = SymbolId(0);
    for i in 0..20 {
        let id = reg.register(&format!("s{i}"), terminal());
        assert!(id.0 > prev.0);
        prev = id;
    }
}

// ===========================================================================
// 12. Large / stress registries
// ===========================================================================

#[test]
fn register_200_symbols() {
    let mut reg = SymbolRegistry::new();
    for i in 0u16..200 {
        let name = format!("sym_{i}");
        let id = reg.register(&name, terminal());
        assert_eq!(id, SymbolId(i + 1));
    }
    assert_eq!(reg.len(), 201);
    assert_eq!(reg.get_id("sym_0"), Some(SymbolId(1)));
    assert_eq!(reg.get_id("sym_199"), Some(SymbolId(200)));
    assert_eq!(reg.get_name(SymbolId(100)), Some("sym_99"));
}

#[test]
fn stress_500_symbols_no_collision() {
    let mut reg = SymbolRegistry::new();
    let mut ids = HashSet::new();
    ids.insert(SymbolId(0));
    for i in 0..500 {
        let id = reg.register(&format!("s{i}"), terminal());
        assert!(ids.insert(id), "collision at symbol {i}");
    }
    assert_eq!(reg.len(), 501);
}

#[test]
fn large_registry_all_lookups_valid() {
    let mut reg = SymbolRegistry::new();
    let mut pairs = Vec::new();
    for i in 0..100 {
        let name = format!("token_{i}");
        let id = reg.register(&name, terminal());
        pairs.push((name, id));
    }
    for (name, id) in &pairs {
        assert_eq!(reg.get_id(name), Some(*id));
        assert_eq!(reg.get_name(*id), Some(name.as_str()));
        assert!(reg.contains_id(*id));
    }
}

// ===========================================================================
// 13. Serde roundtrip
// ===========================================================================

#[test]
fn serde_json_roundtrip_empty_ish() {
    let reg = SymbolRegistry::new();
    let json = serde_json::to_string(&reg).unwrap();
    let deser: SymbolRegistry = serde_json::from_str(&json).unwrap();
    assert_eq!(reg, deser);
}

#[test]
fn serde_json_roundtrip_with_symbols() {
    let mut reg = SymbolRegistry::new();
    reg.register("plus", terminal());
    reg.register("expr", nonterminal());
    reg.register("_ws", hidden_terminal());
    let json = serde_json::to_string(&reg).unwrap();
    let deser: SymbolRegistry = serde_json::from_str(&json).unwrap();
    assert_eq!(reg, deser);
}

#[test]
fn serde_json_preserves_lookups() {
    let mut reg = SymbolRegistry::new();
    let id = reg.register("foo", terminal());
    let json = serde_json::to_string(&reg).unwrap();
    let deser: SymbolRegistry = serde_json::from_str(&json).unwrap();
    assert_eq!(deser.get_id("foo"), Some(id));
    assert_eq!(deser.get_name(id), Some("foo"));
    assert_eq!(deser.get_metadata(id), Some(terminal()));
}

#[test]
fn serde_json_roundtrip_large() {
    let mut reg = SymbolRegistry::new();
    for i in 0..50 {
        reg.register(&format!("sym_{i}"), terminal());
    }
    let json = serde_json::to_string(&reg).unwrap();
    let deser: SymbolRegistry = serde_json::from_str(&json).unwrap();
    assert_eq!(reg, deser);
    assert_eq!(deser.len(), 51);
}

// ===========================================================================
// 14. SymbolInfo struct
// ===========================================================================

#[test]
fn symbol_info_from_iter_has_correct_fields() {
    let mut reg = SymbolRegistry::new();
    let meta = nonterminal();
    let id = reg.register("stmt", meta);
    let info = reg
        .iter()
        .find(|(n, _)| *n == "stmt")
        .map(|(_, info)| info)
        .unwrap();
    assert_eq!(info.id, id);
    assert_eq!(info.metadata, meta);
}

#[test]
fn symbol_info_debug_format() {
    let info = SymbolInfo {
        id: SymbolId(42),
        metadata: terminal(),
    };
    let dbg = format!("{info:?}");
    assert!(dbg.contains("42"));
}

#[test]
fn symbol_info_clone() {
    let info = SymbolInfo {
        id: SymbolId(1),
        metadata: terminal(),
    };
    let cloned = info;
    assert_eq!(info.id, cloned.id);
    assert_eq!(info.metadata, cloned.metadata);
}

// ===========================================================================
// 15. Registration invariants (property-style, manual)
// ===========================================================================

/// After registering N unique names, len == N + 1 (for EOF).
#[test]
fn invariant_len_equals_unique_count_plus_eof() {
    let mut reg = SymbolRegistry::new();
    let names: Vec<String> = (0..30).map(|i| format!("sym_{i}")).collect();
    for n in &names {
        reg.register(n, terminal());
    }
    assert_eq!(reg.len(), names.len() + 1);
}

/// Every registered ID can be found by name and vice-versa.
#[test]
fn invariant_bidirectional_lookup() {
    let mut reg = SymbolRegistry::new();
    let mut map: HashMap<String, SymbolId> = HashMap::new();
    for i in 0..25 {
        let name = format!("t_{i}");
        let id = reg.register(&name, terminal());
        map.insert(name, id);
    }
    for (name, id) in &map {
        assert_eq!(reg.get_id(name), Some(*id));
        assert_eq!(reg.get_name(*id), Some(name.as_str()));
    }
}

/// IDs assigned are always contiguous starting from 1 (0 is EOF).
#[test]
fn invariant_contiguous_ids() {
    let mut reg = SymbolRegistry::new();
    let mut ids = Vec::new();
    for i in 0..20 {
        ids.push(reg.register(&format!("v{i}"), terminal()));
    }
    for (i, id) in ids.iter().enumerate() {
        assert_eq!(id.0, (i as u16) + 1);
    }
}

/// Duplicate registration never creates gaps in ID sequence.
#[test]
fn invariant_no_gaps_after_duplicates() {
    let mut reg = SymbolRegistry::new();
    reg.register("a", terminal()); // id=1
    reg.register("a", nonterminal()); // still id=1
    let id_b = reg.register("b", terminal()); // should be id=2
    assert_eq!(id_b, SymbolId(2));
}

/// iter() covers every symbol exactly once.
#[test]
fn invariant_iter_covers_all_once() {
    let mut reg = SymbolRegistry::new();
    for i in 0..15 {
        reg.register(&format!("s{i}"), terminal());
    }
    let names: Vec<&str> = reg.iter().map(|(n, _)| n).collect();
    let unique: HashSet<&str> = names.iter().copied().collect();
    assert_eq!(names.len(), unique.len(), "iter should not have duplicates");
    assert_eq!(names.len(), reg.len());
}

/// to_index_map values cover exactly [0, len).
#[test]
fn invariant_index_map_contiguous_range() {
    let mut reg = SymbolRegistry::new();
    for i in 0..10 {
        reg.register(&format!("x{i}"), terminal());
    }
    let idx_map = reg.to_index_map();
    let mut vals: Vec<usize> = idx_map.values().copied().collect();
    vals.sort();
    let expected: Vec<usize> = (0..reg.len()).collect();
    assert_eq!(vals, expected);
}

/// to_symbol_map keys cover exactly [0, len).
#[test]
fn invariant_symbol_map_contiguous_keys() {
    let mut reg = SymbolRegistry::new();
    for i in 0..10 {
        reg.register(&format!("x{i}"), terminal());
    }
    let sym_map = reg.to_symbol_map();
    let mut keys: Vec<usize> = sym_map.keys().copied().collect();
    keys.sort();
    let expected: Vec<usize> = (0..reg.len()).collect();
    assert_eq!(keys, expected);
}

// ===========================================================================
// 16. Interaction between multiple operations
// ===========================================================================

#[test]
fn register_then_clone_then_register_more() {
    let mut reg = SymbolRegistry::new();
    reg.register("a", terminal());
    let snapshot = reg.clone();
    reg.register("b", terminal());
    reg.register("c", terminal());

    assert_eq!(snapshot.len(), 2);
    assert_eq!(reg.len(), 4);
    assert_eq!(snapshot.get_id("b"), None);
    assert_eq!(reg.get_id("b"), Some(SymbolId(2)));
}

#[test]
fn interleaved_register_and_lookup() {
    let mut reg = SymbolRegistry::new();
    let id_a = reg.register("a", terminal());
    assert_eq!(reg.get_id("a"), Some(id_a));

    let id_b = reg.register("b", nonterminal());
    // a still accessible
    assert_eq!(reg.get_id("a"), Some(id_a));
    assert_eq!(reg.get_id("b"), Some(id_b));
    assert_ne!(id_a, id_b);
}

#[test]
fn iter_after_metadata_update() {
    let mut reg = SymbolRegistry::new();
    reg.register("tok", terminal());
    reg.register("tok", nonterminal()); // update metadata
    let info = reg
        .iter()
        .find(|(n, _)| *n == "tok")
        .map(|(_, i)| i)
        .unwrap();
    assert_eq!(info.metadata, nonterminal());
}

#[test]
fn index_map_after_metadata_update() {
    let mut reg = SymbolRegistry::new();
    let id = reg.register("tok", terminal());
    reg.register("tok", nonterminal());
    let idx_map = reg.to_index_map();
    assert!(idx_map.contains_key(&id));
    assert_eq!(idx_map.len(), 2); // eof + tok
}

// ===========================================================================
// 17. Boundary / capacity edge cases
// ===========================================================================

#[test]
fn register_up_to_1000_symbols() {
    let mut reg = SymbolRegistry::new();
    for i in 0u16..1000 {
        reg.register(&format!("s{i}"), terminal());
    }
    assert_eq!(reg.len(), 1001);
    assert_eq!(reg.get_id("s0"), Some(SymbolId(1)));
    assert_eq!(reg.get_id("s999"), Some(SymbolId(1000)));
}

#[test]
fn symbol_id_zero_always_eof_after_many_registrations() {
    let mut reg = SymbolRegistry::new();
    for i in 0..50 {
        reg.register(&format!("t{i}"), terminal());
    }
    assert_eq!(reg.get_id("end"), Some(SymbolId(0)));
    assert_eq!(reg.get_name(SymbolId(0)), Some("end"));
}

#[test]
fn contains_id_max_u16_false() {
    let reg = SymbolRegistry::new();
    assert!(!reg.contains_id(SymbolId(u16::MAX)));
}

// ===========================================================================
// 18. Debug formatting
// ===========================================================================

#[test]
fn registry_debug_format_not_empty() {
    let reg = SymbolRegistry::new();
    let dbg = format!("{reg:?}");
    assert!(!dbg.is_empty());
    assert!(dbg.contains("SymbolRegistry"));
}

#[test]
fn registry_debug_shows_end() {
    let reg = SymbolRegistry::new();
    let dbg = format!("{reg:?}");
    assert!(dbg.contains("end"));
}
