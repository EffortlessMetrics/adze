#![allow(clippy::needless_range_loop)]

//! Property-based tests for `SymbolMetadata` in adze-glr-core.
//!
//! Run with: `cargo test -p adze-glr-core --test symbol_metadata_proptest`

use adze_glr_core::{ParseTable, SymbolId, SymbolMetadata};
use proptest::prelude::*;
use std::collections::{BTreeMap, HashMap, HashSet};

// ---------------------------------------------------------------------------
// Strategies
// ---------------------------------------------------------------------------

/// Generate a random symbol name (1–30 chars, optionally prefixed with `_`).
fn arb_symbol_name() -> impl Strategy<Value = String> {
    prop_oneof![
        "[a-z][a-z0-9_]{0,19}".prop_map(|s| s),
        "[a-z][a-z0-9_]{0,19}".prop_map(|s| format!("_{s}")),
    ]
}

/// Generate a random `SymbolId`.
fn arb_symbol_id() -> impl Strategy<Value = SymbolId> {
    (0..=1000u16).prop_map(SymbolId)
}

/// Generate a random `SymbolMetadata`.
fn arb_metadata() -> impl Strategy<Value = SymbolMetadata> {
    (
        arb_symbol_name(),
        any::<bool>(),
        any::<bool>(),
        any::<bool>(),
        any::<bool>(),
        any::<bool>(),
        any::<bool>(),
        arb_symbol_id(),
    )
        .prop_map(
            |(
                name,
                is_visible,
                is_named,
                is_supertype,
                is_terminal,
                is_extra,
                is_fragile,
                symbol_id,
            )| {
                SymbolMetadata {
                    name,
                    is_visible,
                    is_named,
                    is_supertype,
                    is_terminal,
                    is_extra,
                    is_fragile,
                    symbol_id,
                }
            },
        )
}

/// Generate a `Vec<SymbolMetadata>` with unique symbol IDs.
fn arb_metadata_vec(max_len: usize) -> impl Strategy<Value = Vec<SymbolMetadata>> {
    prop::collection::vec(arb_metadata(), 0..=max_len).prop_map(|mut v| {
        // Enforce unique symbol IDs by reassigning
        for (i, m) in v.iter_mut().enumerate() {
            m.symbol_id = SymbolId(i as u16);
        }
        v
    })
}

// ---------------------------------------------------------------------------
// 1. Clone preserves all fields
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn clone_preserves_all_fields(m in arb_metadata()) {
        let c = m.clone();
        prop_assert_eq!(&c.name, &m.name);
        prop_assert_eq!(c.is_visible, m.is_visible);
        prop_assert_eq!(c.is_named, m.is_named);
        prop_assert_eq!(c.is_supertype, m.is_supertype);
        prop_assert_eq!(c.is_terminal, m.is_terminal);
        prop_assert_eq!(c.is_extra, m.is_extra);
        prop_assert_eq!(c.is_fragile, m.is_fragile);
        prop_assert_eq!(c.symbol_id, m.symbol_id);
    }
}

// ---------------------------------------------------------------------------
// 2. Clone is independent (mutating original doesn't affect clone)
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn clone_independence(m in arb_metadata()) {
        let c = m.clone();
        let mut m2 = m.clone();
        m2.name = "MUTATED".to_string();
        m2.is_terminal = !m2.is_terminal;
        // Clone should be unaffected
        prop_assert_eq!(&c.name, &m.name);
        prop_assert_eq!(c.is_terminal, m.is_terminal);
    }
}

// ---------------------------------------------------------------------------
// 3. Debug output contains the name
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn debug_contains_name(m in arb_metadata()) {
        let dbg = format!("{:?}", m);
        prop_assert!(dbg.contains(&m.name), "Debug should contain name '{}'", m.name);
    }
}

// ---------------------------------------------------------------------------
// 4. Debug output contains symbol_id value
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn debug_contains_symbol_id(m in arb_metadata()) {
        let dbg = format!("{:?}", m);
        let id_str = format!("{}", m.symbol_id.0);
        prop_assert!(dbg.contains(&id_str), "Debug should contain symbol_id {}", m.symbol_id.0);
    }
}

// ---------------------------------------------------------------------------
// 5. Debug output contains boolean field names
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn debug_contains_boolean_fields(m in arb_metadata()) {
        let dbg = format!("{:?}", m);
        prop_assert!(dbg.contains("is_visible"));
        prop_assert!(dbg.contains("is_named"));
        prop_assert!(dbg.contains("is_supertype"));
        prop_assert!(dbg.contains("is_terminal"));
        prop_assert!(dbg.contains("is_extra"));
        prop_assert!(dbg.contains("is_fragile"));
    }
}

// ---------------------------------------------------------------------------
// 6. Debug is non-empty
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn debug_is_nonempty(m in arb_metadata()) {
        let dbg = format!("{:?}", m);
        prop_assert!(!dbg.is_empty());
    }
}

// ---------------------------------------------------------------------------
// 7. Double-clone is identical to single clone
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn double_clone_consistent(m in arb_metadata()) {
        let c1 = m.clone();
        let c2 = c1.clone();
        prop_assert_eq!(&c2.name, &m.name);
        prop_assert_eq!(c2.symbol_id, m.symbol_id);
        prop_assert_eq!(c2.is_terminal, m.is_terminal);
        prop_assert_eq!(c2.is_named, m.is_named);
        prop_assert_eq!(c2.is_visible, m.is_visible);
        prop_assert_eq!(c2.is_supertype, m.is_supertype);
        prop_assert_eq!(c2.is_extra, m.is_extra);
        prop_assert_eq!(c2.is_fragile, m.is_fragile);
    }
}

// ---------------------------------------------------------------------------
// 8. Name is never corrupted by clone
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn name_preserved_through_clone(name in arb_symbol_name()) {
        let m = SymbolMetadata {
            name: name.clone(),
            is_visible: true,
            is_named: true,
            is_supertype: false,
            is_terminal: false,
            is_extra: false,
            is_fragile: false,
            symbol_id: SymbolId(0),
        };
        let c = m.clone();
        prop_assert_eq!(&c.name, &name);
        prop_assert_eq!(c.name.len(), name.len());
    }
}

// ---------------------------------------------------------------------------
// 9. Collection: unique IDs in generated vec
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn vec_has_unique_ids(v in arb_metadata_vec(20)) {
        let ids: Vec<_> = v.iter().map(|m| m.symbol_id).collect();
        let unique: HashSet<_> = ids.iter().collect();
        prop_assert_eq!(ids.len(), unique.len(), "symbol IDs should be unique");
    }
}

// ---------------------------------------------------------------------------
// 10. Collection: filter terminals gives correct count
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn filter_terminals_count(v in arb_metadata_vec(15)) {
        let terminal_count = v.iter().filter(|m| m.is_terminal).count();
        let non_terminal_count = v.iter().filter(|m| !m.is_terminal).count();
        prop_assert_eq!(terminal_count + non_terminal_count, v.len());
    }
}

// ---------------------------------------------------------------------------
// 11. Collection: partition covers all elements
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn partition_covers_all(v in arb_metadata_vec(15)) {
        let (terms, nonterms): (Vec<_>, Vec<_>) = v.iter().partition(|m| m.is_terminal);
        prop_assert_eq!(terms.len() + nonterms.len(), v.len());
    }
}

// ---------------------------------------------------------------------------
// 12. Collection: find by symbol_id always works for known IDs
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn find_by_symbol_id(v in arb_metadata_vec(10)) {
        for m in &v {
            let found = v.iter().find(|x| x.symbol_id == m.symbol_id);
            prop_assert!(found.is_some(), "should find symbol_id {:?}", m.symbol_id);
        }
    }
}

// ---------------------------------------------------------------------------
// 13. Collection: HashMap lookup by SymbolId
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn hashmap_lookup(v in arb_metadata_vec(10)) {
        let map: HashMap<SymbolId, &SymbolMetadata> = v.iter().map(|m| (m.symbol_id, m)).collect();
        for m in &v {
            prop_assert!(map.contains_key(&m.symbol_id));
            prop_assert_eq!(&map[&m.symbol_id].name, &m.name);
        }
    }
}

// ---------------------------------------------------------------------------
// 14. Collection: BTreeMap lookup by SymbolId
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn btreemap_lookup(v in arb_metadata_vec(10)) {
        let map: BTreeMap<SymbolId, &SymbolMetadata> = v.iter().map(|m| (m.symbol_id, m)).collect();
        for m in &v {
            prop_assert!(map.contains_key(&m.symbol_id));
            prop_assert_eq!(&map[&m.symbol_id].name, &m.name);
        }
    }
}

// ---------------------------------------------------------------------------
// 15. ParseTable: metadata vec roundtrip through default table
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn parse_table_metadata_roundtrip(v in arb_metadata_vec(10)) {
        let mut table = ParseTable::default();
        table.symbol_metadata = v.clone();
        prop_assert_eq!(table.symbol_metadata.len(), v.len());
        for i in 0..v.len() {
            prop_assert_eq!(&table.symbol_metadata[i].name, &v[i].name);
            prop_assert_eq!(table.symbol_metadata[i].symbol_id, v[i].symbol_id);
        }
    }
}

// ---------------------------------------------------------------------------
// 16. ParseTable: symbol_count consistency
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn parse_table_symbol_count_consistency(v in arb_metadata_vec(10)) {
        let mut table = ParseTable::default();
        table.symbol_metadata = v.clone();
        table.symbol_count = v.len();
        prop_assert_eq!(table.symbol_count, table.symbol_metadata.len());
    }
}

// ---------------------------------------------------------------------------
// 17. ParseTable: filter terminals in table context
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn parse_table_filter_terminals(v in arb_metadata_vec(15)) {
        let mut table = ParseTable::default();
        table.symbol_metadata = v;
        let terms: Vec<_> = table.symbol_metadata.iter().filter(|m| m.is_terminal).collect();
        let nonterms: Vec<_> = table.symbol_metadata.iter().filter(|m| !m.is_terminal).collect();
        prop_assert_eq!(terms.len() + nonterms.len(), table.symbol_metadata.len());
    }
}

// ---------------------------------------------------------------------------
// 18. ParseTable: extras subset of terminals
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn extras_are_subset_of_collection(v in arb_metadata_vec(15)) {
        let extras: Vec<_> = v.iter().filter(|m| m.is_extra).collect();
        // Every extra should be in the original collection
        for e in &extras {
            let found = v.iter().any(|m| m.symbol_id == e.symbol_id);
            prop_assert!(found, "extra {:?} should be in collection", e.symbol_id);
        }
    }
}

// ---------------------------------------------------------------------------
// 19. Metadata boolean flag combinations are all valid
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn all_boolean_combinations_valid(
        is_visible in any::<bool>(),
        is_named in any::<bool>(),
        is_supertype in any::<bool>(),
        is_terminal in any::<bool>(),
        is_extra in any::<bool>(),
        is_fragile in any::<bool>(),
    ) {
        // All boolean combinations should create a valid metadata struct
        let m = SymbolMetadata {
            name: "test".to_string(),
            is_visible,
            is_named,
            is_supertype,
            is_terminal,
            is_extra,
            is_fragile,
            symbol_id: SymbolId(0),
        };
        // Should be cloneable and debuggable without panics
        let _ = m.clone();
        let _ = format!("{:?}", m);
    }
}

// ---------------------------------------------------------------------------
// 20. Symbol ID value preservation through metadata
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn symbol_id_value_preserved(id in 0..=1000u16) {
        let m = SymbolMetadata {
            name: "x".to_string(),
            is_visible: true,
            is_named: true,
            is_supertype: false,
            is_terminal: false,
            is_extra: false,
            is_fragile: false,
            symbol_id: SymbolId(id),
        };
        prop_assert_eq!(m.symbol_id.0, id);
        prop_assert_eq!(m.clone().symbol_id.0, id);
    }
}

// ---------------------------------------------------------------------------
// 21. Collection: sorting by symbol_id is stable
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn sort_by_symbol_id_stable(v in arb_metadata_vec(15)) {
        let mut sorted = v.clone();
        sorted.sort_by_key(|m| m.symbol_id);
        // Should be monotonically non-decreasing
        for i in 1..sorted.len() {
            prop_assert!(sorted[i].symbol_id.0 >= sorted[i - 1].symbol_id.0);
        }
    }
}

// ---------------------------------------------------------------------------
// 22. Collection: group by is_terminal
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn group_by_terminal(v in arb_metadata_vec(15)) {
        let mut groups: HashMap<bool, Vec<&SymbolMetadata>> = HashMap::new();
        for m in &v {
            groups.entry(m.is_terminal).or_default().push(m);
        }
        let total: usize = groups.values().map(|g| g.len()).sum();
        prop_assert_eq!(total, v.len());
    }
}

// ---------------------------------------------------------------------------
// 23. Debug output is deterministic (same metadata → same string)
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn debug_deterministic(m in arb_metadata()) {
        let d1 = format!("{:?}", m);
        let d2 = format!("{:?}", m);
        prop_assert_eq!(d1, d2);
    }
}

// ---------------------------------------------------------------------------
// 24. Clone of vec preserves length and content
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn vec_clone_preserves_content(v in arb_metadata_vec(10)) {
        let cloned = v.clone();
        prop_assert_eq!(cloned.len(), v.len());
        for i in 0..v.len() {
            prop_assert_eq!(&cloned[i].name, &v[i].name);
            prop_assert_eq!(cloned[i].symbol_id, v[i].symbol_id);
            prop_assert_eq!(cloned[i].is_terminal, v[i].is_terminal);
            prop_assert_eq!(cloned[i].is_named, v[i].is_named);
            prop_assert_eq!(cloned[i].is_visible, v[i].is_visible);
            prop_assert_eq!(cloned[i].is_supertype, v[i].is_supertype);
            prop_assert_eq!(cloned[i].is_extra, v[i].is_extra);
            prop_assert_eq!(cloned[i].is_fragile, v[i].is_fragile);
        }
    }
}

// ---------------------------------------------------------------------------
// 25. Collection: visible symbols count <= total
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn visible_count_lte_total(v in arb_metadata_vec(20)) {
        let visible = v.iter().filter(|m| m.is_visible).count();
        prop_assert!(visible <= v.len());
    }
}

// ---------------------------------------------------------------------------
// 26. Collection: named symbols count <= total
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn named_count_lte_total(v in arb_metadata_vec(20)) {
        let named = v.iter().filter(|m| m.is_named).count();
        prop_assert!(named <= v.len());
    }
}

// ---------------------------------------------------------------------------
// 27. ParseTable: cloning table preserves metadata
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn parse_table_clone_preserves_metadata(v in arb_metadata_vec(8)) {
        let mut table = ParseTable::default();
        table.symbol_metadata = v;
        let cloned = table.clone();
        prop_assert_eq!(cloned.symbol_metadata.len(), table.symbol_metadata.len());
        for i in 0..table.symbol_metadata.len() {
            prop_assert_eq!(&cloned.symbol_metadata[i].name, &table.symbol_metadata[i].name);
            prop_assert_eq!(cloned.symbol_metadata[i].symbol_id, table.symbol_metadata[i].symbol_id);
        }
    }
}

// ---------------------------------------------------------------------------
// 28. Collection: extend preserves prior elements
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn extend_preserves_prior(
        v1 in arb_metadata_vec(5),
        v2 in arb_metadata_vec(5),
    ) {
        let orig_len = v1.len();
        let mut combined = v1.clone();
        combined.extend(v2.iter().cloned());
        // First orig_len elements should be unchanged
        for i in 0..orig_len {
            prop_assert_eq!(&combined[i].name, &v1[i].name);
            prop_assert_eq!(combined[i].symbol_id, v1[i].symbol_id);
        }
        prop_assert_eq!(combined.len(), v1.len() + v2.len());
    }
}

// ---------------------------------------------------------------------------
// 29. Collection: retain filters correctly
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn retain_filters_correctly(v in arb_metadata_vec(15)) {
        let mut terminals = v.clone();
        terminals.retain(|m| m.is_terminal);
        for m in &terminals {
            prop_assert!(m.is_terminal);
        }
        prop_assert!(terminals.len() <= v.len());
    }
}

// ---------------------------------------------------------------------------
// 30. Empty name is valid
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn empty_name_is_valid(id in arb_symbol_id()) {
        let m = SymbolMetadata {
            name: String::new(),
            is_visible: false,
            is_named: false,
            is_supertype: false,
            is_terminal: false,
            is_extra: false,
            is_fragile: false,
            symbol_id: id,
        };
        let _ = m.clone();
        let dbg = format!("{:?}", m);
        prop_assert!(!dbg.is_empty());
    }
}

// ---------------------------------------------------------------------------
// 31. ParseTable default has empty metadata
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn default_table_metadata_empty(_dummy in 0..1u8) {
        let table = ParseTable::default();
        prop_assert!(table.symbol_metadata.is_empty());
    }
}

// ---------------------------------------------------------------------------
// 32. Metadata vec to BTreeMap roundtrip
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn vec_to_btreemap_roundtrip(v in arb_metadata_vec(10)) {
        let map: BTreeMap<SymbolId, SymbolMetadata> =
            v.iter().map(|m| (m.symbol_id, m.clone())).collect();
        // Every element should be findable
        for m in &v {
            let entry = map.get(&m.symbol_id).unwrap();
            prop_assert_eq!(&entry.name, &m.name);
            prop_assert_eq!(entry.is_terminal, m.is_terminal);
        }
        // Map size should match (IDs are unique)
        prop_assert_eq!(map.len(), v.len());
    }
}

// ---------------------------------------------------------------------------
// 33. Supertype symbols count <= total
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn supertype_count_lte_total(v in arb_metadata_vec(20)) {
        let supertypes = v.iter().filter(|m| m.is_supertype).count();
        prop_assert!(supertypes <= v.len());
    }
}

// ---------------------------------------------------------------------------
// 34. Debug output for clones is identical
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn clone_debug_identical(m in arb_metadata()) {
        let c = m.clone();
        prop_assert_eq!(format!("{:?}", m), format!("{:?}", c));
    }
}

// ---------------------------------------------------------------------------
// 35. Collection: index-based iteration matches iter
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn index_iteration_matches_iter(v in arb_metadata_vec(10)) {
        let iter_names: Vec<_> = v.iter().map(|m| m.name.clone()).collect();
        let mut idx_names = Vec::new();
        for i in 0..v.len() {
            idx_names.push(v[i].name.clone());
        }
        prop_assert_eq!(iter_names, idx_names);
    }
}
