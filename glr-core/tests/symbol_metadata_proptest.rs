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
        let table = ParseTable {
            symbol_metadata: v.clone(),
            ..Default::default()
        };
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
        let table = ParseTable {
            symbol_metadata: v.clone(),
            symbol_count: v.len(),
            ..Default::default()
        };
        prop_assert_eq!(table.symbol_count, table.symbol_metadata.len());
    }
}

// ---------------------------------------------------------------------------
// 17. ParseTable: filter terminals in table context
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn parse_table_filter_terminals(v in arb_metadata_vec(15)) {
        let table = ParseTable {
            symbol_metadata: v,
            ..Default::default()
        };
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
        let table = ParseTable {
            symbol_metadata: v,
            ..Default::default()
        };
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

// ===========================================================================
// Additional tests (36–63)
// ===========================================================================

// ---------------------------------------------------------------------------
// 36. Default ParseTable has zero state_count and symbol_count
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn default_table_counts_are_zero(_dummy in 0..1u8) {
        let table = ParseTable::default();
        prop_assert_eq!(table.state_count, 0);
        prop_assert_eq!(table.symbol_count, 0);
        prop_assert_eq!(table.token_count, 0);
    }
}

// ---------------------------------------------------------------------------
// 37. Manually constructed terminal metadata has is_terminal true
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn terminal_metadata_flag_is_true(name in arb_symbol_name(), id in arb_symbol_id()) {
        let m = SymbolMetadata {
            name,
            is_visible: true,
            is_named: true,
            is_supertype: false,
            is_terminal: true,
            is_extra: false,
            is_fragile: false,
            symbol_id: id,
        };
        prop_assert!(m.is_terminal);
    }
}

// ---------------------------------------------------------------------------
// 38. Manually constructed non-terminal metadata has is_terminal false
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn nonterminal_metadata_flag_is_false(name in arb_symbol_name(), id in arb_symbol_id()) {
        let m = SymbolMetadata {
            name,
            is_visible: true,
            is_named: true,
            is_supertype: false,
            is_terminal: false,
            is_extra: false,
            is_fragile: false,
            symbol_id: id,
        };
        prop_assert!(!m.is_terminal);
    }
}

// ---------------------------------------------------------------------------
// 39. is_visible true metadata stays visible after clone
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn visible_metadata_stays_visible_after_clone(name in arb_symbol_name()) {
        let m = SymbolMetadata {
            name,
            is_visible: true,
            is_named: false,
            is_supertype: false,
            is_terminal: false,
            is_extra: false,
            is_fragile: false,
            symbol_id: SymbolId(42),
        };
        let c = m.clone();
        prop_assert!(c.is_visible);
    }
}

// ---------------------------------------------------------------------------
// 40. is_visible false metadata stays invisible after clone
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn invisible_metadata_stays_invisible_after_clone(name in arb_symbol_name()) {
        let m = SymbolMetadata {
            name,
            is_visible: false,
            is_named: false,
            is_supertype: false,
            is_terminal: false,
            is_extra: false,
            is_fragile: false,
            symbol_id: SymbolId(42),
        };
        let c = m.clone();
        prop_assert!(!c.is_visible);
    }
}

// ---------------------------------------------------------------------------
// 41. is_supertype true metadata preserved through clone
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn supertype_preserved_through_clone(name in arb_symbol_name()) {
        let m = SymbolMetadata {
            name,
            is_visible: true,
            is_named: true,
            is_supertype: true,
            is_terminal: false,
            is_extra: false,
            is_fragile: false,
            symbol_id: SymbolId(7),
        };
        prop_assert!(m.clone().is_supertype);
    }
}

// ---------------------------------------------------------------------------
// 42. is_supertype false metadata preserved through clone
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn non_supertype_preserved_through_clone(name in arb_symbol_name()) {
        let m = SymbolMetadata {
            name,
            is_visible: true,
            is_named: true,
            is_supertype: false,
            is_terminal: true,
            is_extra: false,
            is_fragile: false,
            symbol_id: SymbolId(8),
        };
        prop_assert!(!m.clone().is_supertype);
    }
}

// ---------------------------------------------------------------------------
// 43. Terminal and non-terminal have distinct is_terminal values
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn terminal_vs_nonterminal_distinct(name in arb_symbol_name()) {
        let term = SymbolMetadata {
            name: name.clone(),
            is_visible: true,
            is_named: true,
            is_supertype: false,
            is_terminal: true,
            is_extra: false,
            is_fragile: false,
            symbol_id: SymbolId(1),
        };
        let nonterm = SymbolMetadata {
            name,
            is_visible: true,
            is_named: true,
            is_supertype: false,
            is_terminal: false,
            is_extra: false,
            is_fragile: false,
            symbol_id: SymbolId(2),
        };
        prop_assert_ne!(term.is_terminal, nonterm.is_terminal);
    }
}

// ---------------------------------------------------------------------------
// 44. ParseTable::is_terminal based on token_count boundary
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn parse_table_is_terminal_boundary(token_count in 1usize..20, extra in 0usize..5) {
        let table = ParseTable {
            token_count,
            external_token_count: extra,
            ..Default::default()
        };
        let boundary = token_count + extra;
        // Symbols below boundary are terminals
        for i in 0..boundary {
            prop_assert!(table.is_terminal(SymbolId(i as u16)));
        }
        // Symbols at or above boundary are non-terminals
        prop_assert!(!table.is_terminal(SymbolId(boundary as u16)));
    }
}

// ---------------------------------------------------------------------------
// 45. ParseTable metadata can be replaced entirely
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn parse_table_metadata_replacement(
        v1 in arb_metadata_vec(5),
        v2 in arb_metadata_vec(8),
    ) {
        let mut table = ParseTable {
            symbol_metadata: v1,
            ..Default::default()
        };
        let old_len = table.symbol_metadata.len();
        table.symbol_metadata = v2.clone();
        prop_assert_eq!(table.symbol_metadata.len(), v2.len());
        prop_assert!(table.symbol_metadata.len() != old_len || v2.len() == old_len);
    }
}

// ---------------------------------------------------------------------------
// 46. Metadata fields are independent (toggling one doesn't affect others)
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn field_independence(m in arb_metadata()) {
        let mut modified = m.clone();
        let orig_visible = m.is_visible;
        let orig_named = m.is_named;
        let orig_supertype = m.is_supertype;

        modified.is_terminal = !modified.is_terminal;
        // Other fields remain unchanged
        prop_assert_eq!(modified.is_visible, orig_visible);
        prop_assert_eq!(modified.is_named, orig_named);
        prop_assert_eq!(modified.is_supertype, orig_supertype);
    }
}

// ---------------------------------------------------------------------------
// 47. Debug output reflects actual is_terminal value
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn debug_reflects_is_terminal_value(m in arb_metadata()) {
        let dbg = format!("{:?}", m);
        let expected = format!("is_terminal: {}", m.is_terminal);
        prop_assert!(dbg.contains(&expected),
            "Debug should contain '{}', got: {}", expected, dbg);
    }
}

// ---------------------------------------------------------------------------
// 48. Debug output reflects actual is_visible value
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn debug_reflects_is_visible_value(m in arb_metadata()) {
        let dbg = format!("{:?}", m);
        let expected = format!("is_visible: {}", m.is_visible);
        prop_assert!(dbg.contains(&expected),
            "Debug should contain '{}', got: {}", expected, dbg);
    }
}

// ---------------------------------------------------------------------------
// 49. Debug output reflects actual is_supertype value
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn debug_reflects_is_supertype_value(m in arb_metadata()) {
        let dbg = format!("{:?}", m);
        let expected = format!("is_supertype: {}", m.is_supertype);
        prop_assert!(dbg.contains(&expected),
            "Debug should contain '{}', got: {}", expected, dbg);
    }
}

// ---------------------------------------------------------------------------
// 50. Collection: visible terminals are a subset of terminals
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn visible_terminals_subset_of_terminals(v in arb_metadata_vec(15)) {
        let visible_terms = v.iter().filter(|m| m.is_visible && m.is_terminal).count();
        let all_terms = v.iter().filter(|m| m.is_terminal).count();
        prop_assert!(visible_terms <= all_terms);
    }
}

// ---------------------------------------------------------------------------
// 51. Collection: supertype non-terminals are a subset of non-terminals
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn supertype_nonterms_subset_of_nonterms(v in arb_metadata_vec(15)) {
        let super_nt = v.iter().filter(|m| m.is_supertype && !m.is_terminal).count();
        let all_nt = v.iter().filter(|m| !m.is_terminal).count();
        prop_assert!(super_nt <= all_nt);
    }
}

// ---------------------------------------------------------------------------
// 52. Metadata with all flags true is valid
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn all_flags_true_is_valid(name in arb_symbol_name(), id in arb_symbol_id()) {
        let m = SymbolMetadata {
            name,
            is_visible: true,
            is_named: true,
            is_supertype: true,
            is_terminal: true,
            is_extra: true,
            is_fragile: true,
            symbol_id: id,
        };
        let c = m.clone();
        prop_assert!(c.is_visible);
        prop_assert!(c.is_named);
        prop_assert!(c.is_supertype);
        prop_assert!(c.is_terminal);
        prop_assert!(c.is_extra);
        prop_assert!(c.is_fragile);
    }
}

// ---------------------------------------------------------------------------
// 53. Metadata with all flags false is valid
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn all_flags_false_is_valid(name in arb_symbol_name(), id in arb_symbol_id()) {
        let m = SymbolMetadata {
            name,
            is_visible: false,
            is_named: false,
            is_supertype: false,
            is_terminal: false,
            is_extra: false,
            is_fragile: false,
            symbol_id: id,
        };
        let c = m.clone();
        prop_assert!(!c.is_visible);
        prop_assert!(!c.is_named);
        prop_assert!(!c.is_supertype);
        prop_assert!(!c.is_terminal);
        prop_assert!(!c.is_extra);
        prop_assert!(!c.is_fragile);
    }
}

// ---------------------------------------------------------------------------
// 54. ParseTable metadata lookup by index matches insertion order
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn parse_table_metadata_order_preserved(v in arb_metadata_vec(12)) {
        let table = ParseTable {
            symbol_metadata: v.clone(),
            ..Default::default()
        };
        for i in 0..v.len() {
            prop_assert_eq!(&table.symbol_metadata[i].name, &v[i].name);
            prop_assert_eq!(table.symbol_metadata[i].is_terminal, v[i].is_terminal);
            prop_assert_eq!(table.symbol_metadata[i].is_visible, v[i].is_visible);
            prop_assert_eq!(table.symbol_metadata[i].is_supertype, v[i].is_supertype);
        }
    }
}

// ---------------------------------------------------------------------------
// 55. Determinism: constructing identical metadata yields identical Debug
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn identical_construction_yields_identical_debug(
        name in arb_symbol_name(),
        vis in any::<bool>(),
        named in any::<bool>(),
        sup in any::<bool>(),
        term in any::<bool>(),
        extra in any::<bool>(),
        frag in any::<bool>(),
        id in arb_symbol_id(),
    ) {
        let m1 = SymbolMetadata {
            name: name.clone(),
            is_visible: vis,
            is_named: named,
            is_supertype: sup,
            is_terminal: term,
            is_extra: extra,
            is_fragile: frag,
            symbol_id: id,
        };
        let m2 = SymbolMetadata {
            name,
            is_visible: vis,
            is_named: named,
            is_supertype: sup,
            is_terminal: term,
            is_extra: extra,
            is_fragile: frag,
            symbol_id: id,
        };
        prop_assert_eq!(format!("{:?}", m1), format!("{:?}", m2));
    }
}

// ---------------------------------------------------------------------------
// 56. Collection: map to (id, is_terminal) pairs preserves data
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn map_to_id_terminal_pairs(v in arb_metadata_vec(10)) {
        let pairs: Vec<(SymbolId, bool)> = v.iter().map(|m| (m.symbol_id, m.is_terminal)).collect();
        prop_assert_eq!(pairs.len(), v.len());
        for i in 0..v.len() {
            prop_assert_eq!(pairs[i].0, v[i].symbol_id);
            prop_assert_eq!(pairs[i].1, v[i].is_terminal);
        }
    }
}

// ---------------------------------------------------------------------------
// 57. Collection: counting fragile symbols <= total
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn fragile_count_lte_total(v in arb_metadata_vec(20)) {
        let fragile = v.iter().filter(|m| m.is_fragile).count();
        prop_assert!(fragile <= v.len());
    }
}

// ---------------------------------------------------------------------------
// 58. ParseTable metadata survives multiple mutations
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn parse_table_metadata_survives_mutations(v in arb_metadata_vec(6)) {
        let mut table = ParseTable {
            symbol_metadata: v.clone(),
            ..Default::default()
        };
        // Mutate state_count and symbol_count, metadata should be unaffected
        table.state_count = 99;
        table.symbol_count = 999;
        prop_assert_eq!(table.symbol_metadata.len(), v.len());
        for i in 0..v.len() {
            prop_assert_eq!(&table.symbol_metadata[i].name, &v[i].name);
        }
    }
}

// ---------------------------------------------------------------------------
// 59. Collection: windows of 2 have consecutive symbol_ids
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn windows_have_consecutive_ids(v in arb_metadata_vec(10)) {
        // arb_metadata_vec assigns sequential IDs
        for window in v.windows(2) {
            prop_assert_eq!(window[1].symbol_id.0, window[0].symbol_id.0 + 1);
        }
    }
}

// ---------------------------------------------------------------------------
// 60. Metadata name with unicode is valid
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn unicode_name_is_valid(id in arb_symbol_id()) {
        let names = vec!["αβγ", "日本語", "émoji🎉", "über"];
        for name in names {
            let m = SymbolMetadata {
                name: name.to_string(),
                is_visible: true,
                is_named: true,
                is_supertype: false,
                is_terminal: true,
                is_extra: false,
                is_fragile: false,
                symbol_id: id,
            };
            let dbg = format!("{:?}", m);
            prop_assert!(!dbg.is_empty());
            prop_assert_eq!(&m.clone().name, name);
        }
    }
}

// ---------------------------------------------------------------------------
// 61. ParseTable default has empty action_table and goto_table
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn default_table_tables_empty(_dummy in 0..1u8) {
        let table = ParseTable::default();
        prop_assert!(table.action_table.is_empty());
        prop_assert!(table.goto_table.is_empty());
        prop_assert!(table.symbol_to_index.is_empty());
        prop_assert!(table.index_to_symbol.is_empty());
    }
}

// ---------------------------------------------------------------------------
// 62. Determinism: sorting metadata vec by name is stable and repeatable
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn sort_by_name_deterministic(v in arb_metadata_vec(10)) {
        let mut s1 = v.clone();
        let mut s2 = v.clone();
        s1.sort_by(|a, b| a.name.cmp(&b.name));
        s2.sort_by(|a, b| a.name.cmp(&b.name));
        for i in 0..s1.len() {
            prop_assert_eq!(&s1[i].name, &s2[i].name);
            prop_assert_eq!(s1[i].symbol_id, s2[i].symbol_id);
        }
    }
}

// ---------------------------------------------------------------------------
// 63. Collection: extra terminals are a subset of all extras
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn extra_terminals_subset_of_extras(v in arb_metadata_vec(15)) {
        let extra_terms = v.iter().filter(|m| m.is_extra && m.is_terminal).count();
        let all_extras = v.iter().filter(|m| m.is_extra).count();
        prop_assert!(extra_terms <= all_extras);
    }
}
