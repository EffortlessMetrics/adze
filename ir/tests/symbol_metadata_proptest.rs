#![allow(clippy::needless_range_loop)]

//! Property-based tests for SymbolMetadata in adze-ir.

use adze_ir::{Grammar, SymbolMetadata, SymbolRegistry};
use proptest::prelude::*;

// ---------------------------------------------------------------------------
// Strategies
// ---------------------------------------------------------------------------

fn symbol_metadata_strategy() -> impl Strategy<Value = SymbolMetadata> {
    (any::<bool>(), any::<bool>(), any::<bool>(), any::<bool>()).prop_map(
        |(visible, named, hidden, terminal)| SymbolMetadata {
            visible,
            named,
            hidden,
            terminal,
        },
    )
}

fn named_metadata_strategy() -> impl Strategy<Value = SymbolMetadata> {
    (any::<bool>(), any::<bool>(), any::<bool>()).prop_map(|(visible, hidden, terminal)| {
        SymbolMetadata {
            visible,
            named: true,
            hidden,
            terminal,
        }
    })
}

fn anonymous_metadata_strategy() -> impl Strategy<Value = SymbolMetadata> {
    (any::<bool>(), any::<bool>(), any::<bool>()).prop_map(|(visible, hidden, terminal)| {
        SymbolMetadata {
            visible,
            named: false,
            hidden,
            terminal,
        }
    })
}

fn visible_metadata_strategy() -> impl Strategy<Value = SymbolMetadata> {
    (any::<bool>(), any::<bool>(), any::<bool>()).prop_map(|(named, hidden, terminal)| {
        SymbolMetadata {
            visible: true,
            named,
            hidden,
            terminal,
        }
    })
}

fn hidden_metadata_strategy() -> impl Strategy<Value = SymbolMetadata> {
    (any::<bool>(), any::<bool>(), any::<bool>()).prop_map(|(visible, named, terminal)| {
        SymbolMetadata {
            visible,
            named,
            hidden: true,
            terminal,
        }
    })
}

fn symbol_name_strategy() -> impl Strategy<Value = String> {
    prop::string::string_regex("[a-z][a-z0-9_]{0,19}")
        .unwrap()
        .prop_filter("non-empty", |s| !s.is_empty())
}

// ---------------------------------------------------------------------------
// 1. SymbolMetadata creation preserves all fields
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn creation_preserves_fields(
        visible in any::<bool>(),
        named in any::<bool>(),
        hidden in any::<bool>(),
        terminal in any::<bool>(),
    ) {
        let meta = SymbolMetadata { visible, named, hidden, terminal };
        prop_assert_eq!(meta.visible, visible);
        prop_assert_eq!(meta.named, named);
        prop_assert_eq!(meta.hidden, hidden);
        prop_assert_eq!(meta.terminal, terminal);
    }
}

// ---------------------------------------------------------------------------
// 2. SymbolMetadata serde JSON roundtrip
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn serde_json_roundtrip(meta in symbol_metadata_strategy()) {
        let json = serde_json::to_string(&meta).unwrap();
        let restored: SymbolMetadata = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(meta, restored);
    }
}

// ---------------------------------------------------------------------------
// 3. SymbolMetadata serde bincode roundtrip
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn serde_bincode_roundtrip(meta in symbol_metadata_strategy()) {
        let bytes = bincode::serialize(&meta).unwrap();
        let restored: SymbolMetadata = bincode::deserialize(&bytes).unwrap();
        prop_assert_eq!(meta, restored);
    }
}

// ---------------------------------------------------------------------------
// 4. SymbolMetadata Copy semantics
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn copy_semantics(meta in symbol_metadata_strategy()) {
        let copied = meta;
        prop_assert_eq!(meta, copied);
        prop_assert_eq!(meta.visible, copied.visible);
        prop_assert_eq!(meta.named, copied.named);
        prop_assert_eq!(meta.hidden, copied.hidden);
        prop_assert_eq!(meta.terminal, copied.terminal);
    }
}

// ---------------------------------------------------------------------------
// 5. SymbolMetadata Clone equals original
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn clone_equals_original(meta in symbol_metadata_strategy()) {
        let cloned = meta;
        prop_assert_eq!(meta, cloned);
    }
}

// ---------------------------------------------------------------------------
// 6. SymbolMetadata equality is reflexive
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn equality_reflexive(meta in symbol_metadata_strategy()) {
        prop_assert_eq!(meta, meta);
    }
}

// ---------------------------------------------------------------------------
// 7. SymbolMetadata equality is symmetric
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn equality_symmetric(a in symbol_metadata_strategy(), b in symbol_metadata_strategy()) {
        prop_assert_eq!(a == b, b == a);
    }
}

// ---------------------------------------------------------------------------
// 8. SymbolMetadata equality is transitive
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn equality_transitive(meta in symbol_metadata_strategy()) {
        let b = meta;
        let c = b;
        if meta == b && b == c {
            prop_assert_eq!(meta, c);
        }
    }
}

// ---------------------------------------------------------------------------
// 9. SymbolMetadata Debug output contains field values
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn debug_contains_field_names(meta in symbol_metadata_strategy()) {
        let dbg = format!("{:?}", meta);
        prop_assert!(dbg.contains("visible"));
        prop_assert!(dbg.contains("named"));
        prop_assert!(dbg.contains("hidden"));
        prop_assert!(dbg.contains("terminal"));
    }
}

// ---------------------------------------------------------------------------
// 10. SymbolMetadata in Grammar via symbol_registry
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn metadata_stored_in_grammar_registry(meta in symbol_metadata_strategy()) {
        let mut grammar = Grammar::new("test".to_string());
        let mut reg = SymbolRegistry::new();
        let id = reg.register("sym", meta);
        grammar.symbol_registry = Some(reg);

        let stored = grammar.symbol_registry.as_ref().unwrap().get_metadata(id).unwrap();
        prop_assert_eq!(stored, meta);
    }
}

// ---------------------------------------------------------------------------
// 11. Named symbol metadata always has named=true
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn named_metadata_has_named_true(meta in named_metadata_strategy()) {
        prop_assert!(meta.named);
    }
}

// ---------------------------------------------------------------------------
// 12. Anonymous symbol metadata always has named=false
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn anonymous_metadata_has_named_false(meta in anonymous_metadata_strategy()) {
        prop_assert!(!meta.named);
    }
}

// ---------------------------------------------------------------------------
// 13. Named and anonymous metadata are never equal when other fields match
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn named_vs_anonymous_differ(
        visible in any::<bool>(),
        hidden in any::<bool>(),
        terminal in any::<bool>(),
    ) {
        let named = SymbolMetadata { visible, named: true, hidden, terminal };
        let anon = SymbolMetadata { visible, named: false, hidden, terminal };
        prop_assert_ne!(named, anon);
    }
}

// ---------------------------------------------------------------------------
// 14. Visible metadata always has visible=true
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn visible_metadata_has_visible_true(meta in visible_metadata_strategy()) {
        prop_assert!(meta.visible);
    }
}

// ---------------------------------------------------------------------------
// 15. Hidden metadata always has hidden=true
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn hidden_metadata_has_hidden_true(meta in hidden_metadata_strategy()) {
        prop_assert!(meta.hidden);
    }
}

// ---------------------------------------------------------------------------
// 16. Visible vs non-visible metadata differ when other fields match
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn visible_vs_invisible_differ(
        named in any::<bool>(),
        hidden in any::<bool>(),
        terminal in any::<bool>(),
    ) {
        let vis = SymbolMetadata { visible: true, named, hidden, terminal };
        let invis = SymbolMetadata { visible: false, named, hidden, terminal };
        prop_assert_ne!(vis, invis);
    }
}

// ---------------------------------------------------------------------------
// 17. Multiple metadata consistency in registry
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn multiple_metadata_in_registry(
        m1 in symbol_metadata_strategy(),
        m2 in symbol_metadata_strategy(),
        m3 in symbol_metadata_strategy(),
    ) {
        let mut reg = SymbolRegistry::new();
        let id1 = reg.register("alpha", m1);
        let id2 = reg.register("beta", m2);
        let id3 = reg.register("gamma", m3);

        prop_assert_eq!(reg.get_metadata(id1).unwrap(), m1);
        prop_assert_eq!(reg.get_metadata(id2).unwrap(), m2);
        prop_assert_eq!(reg.get_metadata(id3).unwrap(), m3);
        prop_assert_ne!(id1, id2);
        prop_assert_ne!(id2, id3);
    }
}

// ---------------------------------------------------------------------------
// 18. Default metadata has all fields false
// ---------------------------------------------------------------------------

#[test]
fn default_metadata_all_false() {
    let meta = SymbolMetadata {
        visible: false,
        named: false,
        hidden: false,
        terminal: false,
    };
    assert!(!meta.visible);
    assert!(!meta.named);
    assert!(!meta.hidden);
    assert!(!meta.terminal);
}

// ---------------------------------------------------------------------------
// 19. All-true metadata
// ---------------------------------------------------------------------------

#[test]
fn all_true_metadata() {
    let meta = SymbolMetadata {
        visible: true,
        named: true,
        hidden: true,
        terminal: true,
    };
    assert!(meta.visible);
    assert!(meta.named);
    assert!(meta.hidden);
    assert!(meta.terminal);
}

// ---------------------------------------------------------------------------
// 20. Exactly 16 distinct SymbolMetadata values
// ---------------------------------------------------------------------------

#[test]
fn exhaustive_sixteen_combinations() {
    let mut seen = std::collections::HashSet::new();
    for v in [false, true] {
        for n in [false, true] {
            for h in [false, true] {
                for t in [false, true] {
                    let meta = SymbolMetadata {
                        visible: v,
                        named: n,
                        hidden: h,
                        terminal: t,
                    };
                    let json = serde_json::to_string(&meta).unwrap();
                    seen.insert(json);
                }
            }
        }
    }
    assert_eq!(seen.len(), 16);
}

// ---------------------------------------------------------------------------
// 21. JSON roundtrip for all 16 combinations
// ---------------------------------------------------------------------------

#[test]
fn exhaustive_roundtrip_all_sixteen() {
    for v in [false, true] {
        for n in [false, true] {
            for h in [false, true] {
                for t in [false, true] {
                    let meta = SymbolMetadata {
                        visible: v,
                        named: n,
                        hidden: h,
                        terminal: t,
                    };
                    let json = serde_json::to_string(&meta).unwrap();
                    let restored: SymbolMetadata = serde_json::from_str(&json).unwrap();
                    assert_eq!(meta, restored);
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// 22. Metadata survives grammar serde roundtrip
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn metadata_survives_grammar_serde_roundtrip(meta in symbol_metadata_strategy()) {
        let mut grammar = Grammar::new("roundtrip_test".to_string());
        let mut reg = SymbolRegistry::new();
        reg.register("test_sym", meta);
        grammar.symbol_registry = Some(reg);

        let json = serde_json::to_string(&grammar).unwrap();
        let restored: Grammar = serde_json::from_str(&json).unwrap();

        let restored_reg = restored.symbol_registry.as_ref().unwrap();
        let id = restored_reg.get_id("test_sym").unwrap();
        prop_assert_eq!(restored_reg.get_metadata(id).unwrap(), meta);
    }
}

// ---------------------------------------------------------------------------
// 23. Metadata update in registry overwrites previous
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn registry_metadata_update(
        m1 in symbol_metadata_strategy(),
        m2 in symbol_metadata_strategy(),
    ) {
        let mut reg = SymbolRegistry::new();
        let id1 = reg.register("sym", m1);
        let id2 = reg.register("sym", m2);

        prop_assert_eq!(id1, id2);
        prop_assert_eq!(reg.get_metadata(id1).unwrap(), m2);
    }
}

// ---------------------------------------------------------------------------
// 24. Terminal vs non-terminal metadata differ when other fields match
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn terminal_vs_nonterminal_differ(
        visible in any::<bool>(),
        named in any::<bool>(),
        hidden in any::<bool>(),
    ) {
        let term = SymbolMetadata { visible, named, hidden, terminal: true };
        let nonterm = SymbolMetadata { visible, named, hidden, terminal: false };
        prop_assert_ne!(term, nonterm);
    }
}

// ---------------------------------------------------------------------------
// 25. Hidden vs non-hidden metadata differ when other fields match
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn hidden_vs_nonhidden_differ(
        visible in any::<bool>(),
        named in any::<bool>(),
        terminal in any::<bool>(),
    ) {
        let hid = SymbolMetadata { visible, named, hidden: true, terminal };
        let nothid = SymbolMetadata { visible, named, hidden: false, terminal };
        prop_assert_ne!(hid, nothid);
    }
}

// ---------------------------------------------------------------------------
// 26. JSON field order does not affect deserialization
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn json_field_order_irrelevant(meta in symbol_metadata_strategy()) {
        let manual_json = format!(
            r#"{{"terminal":{},"hidden":{},"named":{},"visible":{}}}"#,
            meta.terminal, meta.hidden, meta.named, meta.visible
        );
        let restored: SymbolMetadata = serde_json::from_str(&manual_json).unwrap();
        prop_assert_eq!(meta, restored);
    }
}

// ---------------------------------------------------------------------------
// 27. Metadata with dynamic name in registry
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn registry_with_generated_name(
        name in symbol_name_strategy(),
        meta in symbol_metadata_strategy(),
    ) {
        let mut reg = SymbolRegistry::new();
        let id = reg.register(&name, meta);
        prop_assert_eq!(reg.get_id(&name), Some(id));
        prop_assert_eq!(reg.get_name(id), Some(name.as_str()));
        prop_assert_eq!(reg.get_metadata(id).unwrap(), meta);
    }
}

// ---------------------------------------------------------------------------
// 28. Multiple registrations preserve distinct metadata
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn distinct_names_get_distinct_ids(
        m1 in symbol_metadata_strategy(),
        m2 in symbol_metadata_strategy(),
    ) {
        let mut reg = SymbolRegistry::new();
        let id1 = reg.register("aaa", m1);
        let id2 = reg.register("bbb", m2);

        prop_assert_ne!(id1, id2);
        prop_assert_eq!(reg.get_metadata(id1).unwrap(), m1);
        prop_assert_eq!(reg.get_metadata(id2).unwrap(), m2);
    }
}

// ---------------------------------------------------------------------------
// 29. Metadata equality: flipping any single field changes equality
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn single_field_flip_changes_equality(meta in symbol_metadata_strategy()) {
        let flip_visible = SymbolMetadata { visible: !meta.visible, ..meta };
        let flip_named = SymbolMetadata { named: !meta.named, ..meta };
        let flip_hidden = SymbolMetadata { hidden: !meta.hidden, ..meta };
        let flip_terminal = SymbolMetadata { terminal: !meta.terminal, ..meta };

        prop_assert_ne!(meta, flip_visible);
        prop_assert_ne!(meta, flip_named);
        prop_assert_ne!(meta, flip_hidden);
        prop_assert_ne!(meta, flip_terminal);
    }
}

// ---------------------------------------------------------------------------
// 30. Bincode serialization size is consistent
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn bincode_size_consistent(
        a in symbol_metadata_strategy(),
        b in symbol_metadata_strategy(),
    ) {
        let bytes_a = bincode::serialize(&a).unwrap();
        let bytes_b = bincode::serialize(&b).unwrap();
        prop_assert_eq!(bytes_a.len(), bytes_b.len());
    }
}

// ---------------------------------------------------------------------------
// 31. Grammar with multiple registry entries preserves all metadata
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn grammar_multiple_registry_entries(
        metas in proptest::collection::vec(symbol_metadata_strategy(), 1..10),
    ) {
        let mut grammar = Grammar::new("multi".to_string());
        let mut reg = SymbolRegistry::new();
        let mut ids = Vec::new();

        for (i, meta) in metas.iter().enumerate() {
            let name = format!("sym_{}", i);
            ids.push(reg.register(&name, *meta));
        }
        grammar.symbol_registry = Some(reg);

        let reg_ref = grammar.symbol_registry.as_ref().unwrap();
        for i in 0..metas.len() {
            prop_assert_eq!(reg_ref.get_metadata(ids[i]).unwrap(), metas[i]);
        }
    }
}

// ---------------------------------------------------------------------------
// 32. JSON pretty-print roundtrip
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn serde_json_pretty_roundtrip(meta in symbol_metadata_strategy()) {
        let json = serde_json::to_string_pretty(&meta).unwrap();
        let restored: SymbolMetadata = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(meta, restored);
    }
}

// ---------------------------------------------------------------------------
// 33. Metadata inequality when all fields differ
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn all_fields_flipped_not_equal(meta in symbol_metadata_strategy()) {
        let flipped = SymbolMetadata {
            visible: !meta.visible,
            named: !meta.named,
            hidden: !meta.hidden,
            terminal: !meta.terminal,
        };
        prop_assert_ne!(meta, flipped);
    }
}
