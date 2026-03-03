#![allow(clippy::needless_range_loop)]

use adze_ir::symbol_registry::{SymbolInfo, SymbolRegistry};
use adze_ir::{SymbolId, SymbolMetadata};
use proptest::prelude::*;
use std::collections::HashSet;

// ---------------------------------------------------------------------------
// Strategies
// ---------------------------------------------------------------------------

fn arb_metadata() -> impl Strategy<Value = SymbolMetadata> {
    (any::<bool>(), any::<bool>(), any::<bool>(), any::<bool>()).prop_map(
        |(visible, named, hidden, terminal)| SymbolMetadata {
            visible,
            named,
            hidden,
            terminal,
        },
    )
}

/// Non-empty alphanumeric symbol name (avoids collisions with "end").
fn arb_name() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9_]{0,15}".prop_filter("must not be `end`", |s| s != "end")
}

/// Vec of unique symbol names (1..=count).
fn arb_unique_names(count: usize) -> impl Strategy<Value = Vec<String>> {
    proptest::collection::hash_set(arb_name(), 1..=count)
        .prop_map(|s| s.into_iter().collect::<Vec<_>>())
}

// ---------------------------------------------------------------------------
// 1. Register returns a unique ID for each new symbol
// ---------------------------------------------------------------------------
proptest! {
    #[test]
    fn register_gives_unique_id(names in arb_unique_names(20)) {
        let mut reg = SymbolRegistry::new();
        let mut ids = HashSet::new();
        for name in &names {
            let id = reg.register(name, SymbolMetadata { visible: true, named: true, hidden: false, terminal: false });
            prop_assert!(ids.insert(id), "duplicate id {id:?} for {name}");
        }
    }
}

// ---------------------------------------------------------------------------
// 2. Re-registration of the same name returns the same ID
// ---------------------------------------------------------------------------
proptest! {
    #[test]
    fn reregister_same_name_same_id(name in arb_name(), m1 in arb_metadata(), m2 in arb_metadata()) {
        let mut reg = SymbolRegistry::new();
        let id1 = reg.register(&name, m1);
        let id2 = reg.register(&name, m2);
        prop_assert_eq!(id1, id2);
    }
}

// ---------------------------------------------------------------------------
// 3. ID lookup returns the correct name
// ---------------------------------------------------------------------------
proptest! {
    #[test]
    fn id_lookup_returns_correct_name(name in arb_name(), meta in arb_metadata()) {
        let mut reg = SymbolRegistry::new();
        let id = reg.register(&name, meta);
        prop_assert_eq!(reg.get_name(id), Some(name.as_str()));
    }
}

// ---------------------------------------------------------------------------
// 4. Name lookup returns the correct ID
// ---------------------------------------------------------------------------
proptest! {
    #[test]
    fn name_lookup_returns_correct_id(name in arb_name(), meta in arb_metadata()) {
        let mut reg = SymbolRegistry::new();
        let id = reg.register(&name, meta);
        prop_assert_eq!(reg.get_id(&name), Some(id));
    }
}

// ---------------------------------------------------------------------------
// 5. Registering multiple symbols preserves all lookups
// ---------------------------------------------------------------------------
proptest! {
    #[test]
    fn register_multiple_all_lookups_valid(names in arb_unique_names(15)) {
        let mut reg = SymbolRegistry::new();
        let mut pairs = Vec::new();
        for name in &names {
            let id = reg.register(name, SymbolMetadata { visible: true, named: false, hidden: false, terminal: true });
            pairs.push((name.clone(), id));
        }
        for (name, id) in &pairs {
            prop_assert_eq!(reg.get_id(name), Some(*id));
            prop_assert_eq!(reg.get_name(*id), Some(name.as_str()));
        }
    }
}

// ---------------------------------------------------------------------------
// 6. Symbol count matches number of unique registrations + EOF
// ---------------------------------------------------------------------------
proptest! {
    #[test]
    fn count_matches_registrations(names in arb_unique_names(20)) {
        let mut reg = SymbolRegistry::new();
        for name in &names {
            reg.register(name, SymbolMetadata { visible: true, named: true, hidden: false, terminal: false });
        }
        prop_assert_eq!(reg.len(), names.len() + 1); // +1 for EOF
    }
}

// ---------------------------------------------------------------------------
// 7. Clone preserves all entries
// ---------------------------------------------------------------------------
proptest! {
    #[test]
    fn clone_preserves_entries(names in arb_unique_names(12)) {
        let mut reg = SymbolRegistry::new();
        for name in &names {
            reg.register(name, SymbolMetadata { visible: true, named: true, hidden: false, terminal: false });
        }
        let cloned = reg.clone();
        prop_assert_eq!(reg.len(), cloned.len());
        for name in &names {
            prop_assert_eq!(reg.get_id(name), cloned.get_id(name));
        }
        prop_assert_eq!(reg.get_id("end"), cloned.get_id("end"));
    }
}

// ---------------------------------------------------------------------------
// 8. PartialEq: identical registries are equal
// ---------------------------------------------------------------------------
proptest! {
    #[test]
    fn partial_eq_identical_registries(names in arb_unique_names(10)) {
        let mut reg1 = SymbolRegistry::new();
        let mut reg2 = SymbolRegistry::new();
        let meta = SymbolMetadata { visible: true, named: true, hidden: false, terminal: false };
        for name in &names {
            reg1.register(name, meta);
            reg2.register(name, meta);
        }
        prop_assert_eq!(&reg1, &reg2);
    }
}

// ---------------------------------------------------------------------------
// 9. PartialEq: different metadata makes registries unequal
// ---------------------------------------------------------------------------
proptest! {
    #[test]
    fn partial_eq_different_metadata(name in arb_name()) {
        let mut reg1 = SymbolRegistry::new();
        let mut reg2 = SymbolRegistry::new();
        reg1.register(&name, SymbolMetadata { visible: true, named: true, hidden: false, terminal: false });
        reg2.register(&name, SymbolMetadata { visible: false, named: false, hidden: true, terminal: true });
        prop_assert_ne!(reg1, reg2);
    }
}

// ---------------------------------------------------------------------------
// 10. PartialEq: different symbols makes registries unequal
// ---------------------------------------------------------------------------
proptest! {
    #[test]
    fn partial_eq_different_symbols(
        n1 in "[a-m][a-z]{1,5}".prop_filter("not end", |s| s != "end"),
        n2 in "[n-z][a-z]{1,5}".prop_filter("not end", |s| s != "end"),
    ) {
        let meta = SymbolMetadata { visible: true, named: true, hidden: false, terminal: false };
        let mut reg1 = SymbolRegistry::new();
        let mut reg2 = SymbolRegistry::new();
        reg1.register(&n1, meta);
        reg2.register(&n2, meta);
        prop_assert_ne!(reg1, reg2);
    }
}

// ---------------------------------------------------------------------------
// 11. contains_id returns true for registered, false for unregistered
// ---------------------------------------------------------------------------
proptest! {
    #[test]
    fn contains_id_correctness(name in arb_name(), meta in arb_metadata()) {
        let mut reg = SymbolRegistry::new();
        let id = reg.register(&name, meta);
        prop_assert!(reg.contains_id(id));
        prop_assert!(!reg.contains_id(SymbolId(1000)));
    }
}

// ---------------------------------------------------------------------------
// 12. get_metadata returns the last-registered metadata
// ---------------------------------------------------------------------------
proptest! {
    #[test]
    fn metadata_updated_on_reregister(name in arb_name(), m1 in arb_metadata(), m2 in arb_metadata()) {
        let mut reg = SymbolRegistry::new();
        reg.register(&name, m1);
        let id = reg.register(&name, m2);
        prop_assert_eq!(reg.get_metadata(id), Some(m2));
    }
}

// ---------------------------------------------------------------------------
// 13. IDs are sequential starting from 1 (0 is EOF)
// ---------------------------------------------------------------------------
proptest! {
    #[test]
    fn ids_are_sequential(names in arb_unique_names(20)) {
        let mut reg = SymbolRegistry::new();
        for (i, name) in names.iter().enumerate() {
            let id = reg.register(name, SymbolMetadata { visible: true, named: false, hidden: false, terminal: true });
            prop_assert_eq!(id, SymbolId((i + 1) as u16));
        }
    }
}

// ---------------------------------------------------------------------------
// 14. Re-registration does NOT increase count
// ---------------------------------------------------------------------------
proptest! {
    #[test]
    fn reregister_does_not_increase_count(name in arb_name(), m1 in arb_metadata(), m2 in arb_metadata()) {
        let mut reg = SymbolRegistry::new();
        reg.register(&name, m1);
        let before = reg.len();
        reg.register(&name, m2);
        prop_assert_eq!(reg.len(), before);
    }
}

// ---------------------------------------------------------------------------
// 15. iter length equals len()
// ---------------------------------------------------------------------------
proptest! {
    #[test]
    fn iter_len_matches_len(names in arb_unique_names(15)) {
        let mut reg = SymbolRegistry::new();
        for name in &names {
            reg.register(name, SymbolMetadata { visible: true, named: true, hidden: false, terminal: false });
        }
        prop_assert_eq!(reg.iter().count(), reg.len());
    }
}

// ---------------------------------------------------------------------------
// 16. iter first entry is always EOF
// ---------------------------------------------------------------------------
proptest! {
    #[test]
    fn iter_first_is_eof(names in arb_unique_names(5)) {
        let mut reg = SymbolRegistry::new();
        for name in &names {
            reg.register(name, SymbolMetadata { visible: true, named: true, hidden: false, terminal: false });
        }
        let first = reg.iter().next().unwrap();
        prop_assert_eq!(first.0, "end");
        prop_assert_eq!(first.1.id, SymbolId(0));
    }
}

// ---------------------------------------------------------------------------
// 17. iter yields SymbolInfo with correct ids
// ---------------------------------------------------------------------------
proptest! {
    #[test]
    fn iter_symbol_info_ids_match(names in arb_unique_names(10)) {
        let mut reg = SymbolRegistry::new();
        let meta = SymbolMetadata { visible: true, named: true, hidden: false, terminal: false };
        for name in &names {
            reg.register(name, meta);
        }
        for (name, info) in reg.iter() {
            prop_assert_eq!(reg.get_id(name), Some(info.id));
        }
    }
}

// ---------------------------------------------------------------------------
// 18. to_index_map has entry for every registered symbol
// ---------------------------------------------------------------------------
proptest! {
    #[test]
    fn index_map_covers_all_symbols(names in arb_unique_names(10)) {
        let mut reg = SymbolRegistry::new();
        let meta = SymbolMetadata { visible: true, named: false, hidden: false, terminal: true };
        let mut ids = vec![SymbolId(0)]; // EOF
        for name in &names {
            ids.push(reg.register(name, meta));
        }
        let idx_map = reg.to_index_map();
        for id in &ids {
            prop_assert!(idx_map.contains_key(id), "missing {id:?} in index_map");
        }
        prop_assert_eq!(idx_map.len(), reg.len());
    }
}

// ---------------------------------------------------------------------------
// 19. to_symbol_map has entry for every index
// ---------------------------------------------------------------------------
proptest! {
    #[test]
    fn symbol_map_covers_all_indices(names in arb_unique_names(10)) {
        let mut reg = SymbolRegistry::new();
        let meta = SymbolMetadata { visible: true, named: false, hidden: false, terminal: true };
        for name in &names {
            reg.register(name, meta);
        }
        let sym_map = reg.to_symbol_map();
        for i in 0..reg.len() {
            prop_assert!(sym_map.contains_key(&i), "missing index {i} in symbol_map");
        }
        prop_assert_eq!(sym_map.len(), reg.len());
    }
}

// ---------------------------------------------------------------------------
// 20. to_index_map and to_symbol_map are inverses
// ---------------------------------------------------------------------------
proptest! {
    #[test]
    fn index_map_symbol_map_inverse(names in arb_unique_names(15)) {
        let mut reg = SymbolRegistry::new();
        let meta = SymbolMetadata { visible: true, named: false, hidden: false, terminal: true };
        for name in &names {
            reg.register(name, meta);
        }
        let idx_map = reg.to_index_map();
        let sym_map = reg.to_symbol_map();
        for (&sym_id, &idx) in &idx_map {
            prop_assert_eq!(sym_map.get(&idx), Some(&sym_id));
        }
        for (&idx, &sym_id) in &sym_map {
            prop_assert_eq!(idx_map.get(&sym_id), Some(&idx));
        }
    }
}

// ---------------------------------------------------------------------------
// 21. Nonexistent name lookup returns None
// ---------------------------------------------------------------------------
proptest! {
    #[test]
    fn nonexistent_name_returns_none(name in arb_name()) {
        let reg = SymbolRegistry::new();
        if name != "end" {
            prop_assert_eq!(reg.get_id(&name), None);
        }
    }
}

// ---------------------------------------------------------------------------
// 22. Nonexistent id lookup returns None
// ---------------------------------------------------------------------------
proptest! {
    #[test]
    fn nonexistent_id_returns_none(raw_id in 500u16..=u16::MAX) {
        let reg = SymbolRegistry::new();
        prop_assert_eq!(reg.get_name(SymbolId(raw_id)), None);
        prop_assert_eq!(reg.get_metadata(SymbolId(raw_id)), None);
        prop_assert!(!reg.contains_id(SymbolId(raw_id)));
    }
}

// ---------------------------------------------------------------------------
// 23. Clone is independent — mutating clone does not affect original
// ---------------------------------------------------------------------------
proptest! {
    #[test]
    fn clone_is_independent(names in arb_unique_names(8)) {
        let mut reg = SymbolRegistry::new();
        let meta = SymbolMetadata { visible: true, named: true, hidden: false, terminal: false };
        for name in &names {
            reg.register(name, meta);
        }
        let original_len = reg.len();
        let mut cloned = reg.clone();
        cloned.register("extra_clone_only", meta);
        prop_assert_eq!(reg.len(), original_len);
        prop_assert_eq!(reg.get_id("extra_clone_only"), None);
    }
}

// ---------------------------------------------------------------------------
// 24. Determinism: two registries with same insertion order are equal
// ---------------------------------------------------------------------------
proptest! {
    #[test]
    fn determinism_same_order_equal(names in arb_unique_names(12)) {
        let meta = SymbolMetadata { visible: true, named: true, hidden: false, terminal: false };
        let mut r1 = SymbolRegistry::new();
        let mut r2 = SymbolRegistry::new();
        for name in &names {
            r1.register(name, meta);
            r2.register(name, meta);
        }
        prop_assert_eq!(&r1, &r2);
        for name in &names {
            prop_assert_eq!(r1.get_id(name), r2.get_id(name));
        }
    }
}

// ---------------------------------------------------------------------------
// 25. is_empty is false for a fresh (default) registry
// ---------------------------------------------------------------------------
proptest! {
    #[test]
    fn fresh_registry_not_empty(_dummy in 0..1u8) {
        let reg = SymbolRegistry::new();
        prop_assert!(!reg.is_empty());
    }
}

// ---------------------------------------------------------------------------
// 26. Metadata round-trip: register then query returns identical metadata
// ---------------------------------------------------------------------------
proptest! {
    #[test]
    fn metadata_roundtrip(name in arb_name(), meta in arb_metadata()) {
        let mut reg = SymbolRegistry::new();
        let id = reg.register(&name, meta);
        prop_assert_eq!(reg.get_metadata(id), Some(meta));
    }
}

// ---------------------------------------------------------------------------
// 27. All registered IDs appear in iter
// ---------------------------------------------------------------------------
proptest! {
    #[test]
    fn all_ids_appear_in_iter(names in arb_unique_names(12)) {
        let mut reg = SymbolRegistry::new();
        let meta = SymbolMetadata { visible: true, named: true, hidden: false, terminal: false };
        let mut expected_ids: HashSet<SymbolId> = HashSet::new();
        expected_ids.insert(SymbolId(0)); // EOF
        for name in &names {
            expected_ids.insert(reg.register(name, meta));
        }
        let iter_ids: HashSet<SymbolId> = reg.iter().map(|(_, info)| info.id).collect();
        prop_assert_eq!(expected_ids, iter_ids);
    }
}

// ---------------------------------------------------------------------------
// 28. Iter metadata matches get_metadata
// ---------------------------------------------------------------------------
proptest! {
    #[test]
    fn iter_metadata_matches_get(names in arb_unique_names(10)) {
        let mut reg = SymbolRegistry::new();
        let meta = SymbolMetadata { visible: false, named: true, hidden: true, terminal: false };
        for name in &names {
            reg.register(name, meta);
        }
        for (_, SymbolInfo { id, metadata }) in reg.iter() {
            prop_assert_eq!(reg.get_metadata(id), Some(metadata));
        }
    }
}

// ---------------------------------------------------------------------------
// 29. index_map indices are contiguous 0..len
// ---------------------------------------------------------------------------
proptest! {
    #[test]
    fn index_map_indices_contiguous(names in arb_unique_names(10)) {
        let mut reg = SymbolRegistry::new();
        let meta = SymbolMetadata { visible: true, named: false, hidden: false, terminal: true };
        for name in &names {
            reg.register(name, meta);
        }
        let idx_map = reg.to_index_map();
        let mut indices: Vec<usize> = idx_map.values().copied().collect();
        indices.sort();
        let expected: Vec<usize> = (0..reg.len()).collect();
        prop_assert_eq!(indices, expected);
    }
}

// ---------------------------------------------------------------------------
// 30. EOF always has SymbolId(0) regardless of other registrations
// ---------------------------------------------------------------------------
proptest! {
    #[test]
    fn eof_always_zero(names in arb_unique_names(15)) {
        let mut reg = SymbolRegistry::new();
        for name in &names {
            reg.register(name, SymbolMetadata { visible: true, named: true, hidden: false, terminal: false });
        }
        prop_assert_eq!(reg.get_id("end"), Some(SymbolId(0)));
        prop_assert_eq!(reg.get_name(SymbolId(0)), Some("end"));
    }
}
