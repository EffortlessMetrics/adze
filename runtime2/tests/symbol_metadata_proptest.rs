#![allow(clippy::needless_range_loop)]

use proptest::prelude::*;

use adze_runtime::language::{Language, SymbolMetadata};

// ---------------------------------------------------------------------------
// Strategies
// ---------------------------------------------------------------------------

fn arb_symbol_metadata() -> impl Strategy<Value = SymbolMetadata> {
    (any::<bool>(), any::<bool>(), any::<bool>()).prop_map(|(t, v, s)| SymbolMetadata {
        is_terminal: t,
        is_visible: v,
        is_supertype: s,
    })
}

fn arb_symbol_metadata_vec(max_len: usize) -> impl Strategy<Value = Vec<SymbolMetadata>> {
    proptest::collection::vec(arb_symbol_metadata(), 0..=max_len)
}

fn leak_table() -> &'static adze_glr_core::ParseTable {
    Box::leak(Box::new(adze_glr_core::ParseTable::default()))
}

fn build_language_from_metadata(meta: Vec<SymbolMetadata>) -> Language {
    let names: Vec<String> = (0..meta.len()).map(|i| format!("sym_{i}")).collect();
    Language::builder()
        .version(15)
        .parse_table(leak_table())
        .symbol_names(names)
        .symbol_metadata(meta)
        .build()
        .unwrap()
}

// ---------------------------------------------------------------------------
// 1 – All 8 boolean combinations
// ---------------------------------------------------------------------------

#[test]
fn all_eight_boolean_combinations_are_constructible() {
    for t in [false, true] {
        for v in [false, true] {
            for s in [false, true] {
                let m = SymbolMetadata {
                    is_terminal: t,
                    is_visible: v,
                    is_supertype: s,
                };
                assert_eq!(m.is_terminal, t);
                assert_eq!(m.is_visible, v);
                assert_eq!(m.is_supertype, s);
            }
        }
    }
}

#[test]
fn all_eight_combinations_debug_output_differs() {
    let mut outputs = std::collections::HashSet::new();
    for t in [false, true] {
        for v in [false, true] {
            for s in [false, true] {
                let m = SymbolMetadata {
                    is_terminal: t,
                    is_visible: v,
                    is_supertype: s,
                };
                let dbg = format!("{m:?}");
                outputs.insert(dbg);
            }
        }
    }
    assert_eq!(outputs.len(), 8);
}

#[test]
fn all_eight_combinations_in_language() {
    let all: Vec<SymbolMetadata> = [false, true]
        .iter()
        .flat_map(|&t| {
            [false, true].iter().flat_map(move |&v| {
                [false, true].iter().map(move |&s| SymbolMetadata {
                    is_terminal: t,
                    is_visible: v,
                    is_supertype: s,
                })
            })
        })
        .collect();
    assert_eq!(all.len(), 8);
    let lang = build_language_from_metadata(all.clone());
    assert_eq!(lang.symbol_count, 8);
    for i in 0..8 {
        assert_eq!(lang.symbol_metadata[i].is_terminal, all[i].is_terminal);
        assert_eq!(lang.symbol_metadata[i].is_visible, all[i].is_visible);
        assert_eq!(lang.symbol_metadata[i].is_supertype, all[i].is_supertype);
    }
}

// ---------------------------------------------------------------------------
// 2 – Clone / Copy semantics
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn clone_preserves_all_fields(m in arb_symbol_metadata()) {
        let cloned = m;
        prop_assert_eq!(cloned.is_terminal, m.is_terminal);
        prop_assert_eq!(cloned.is_visible, m.is_visible);
        prop_assert_eq!(cloned.is_supertype, m.is_supertype);
    }

    #[test]
    fn copy_preserves_all_fields(m in arb_symbol_metadata()) {
        let copied = m;
        let again = m; // still usable after copy
        prop_assert_eq!(copied.is_terminal, m.is_terminal);
        prop_assert_eq!(copied.is_visible, m.is_visible);
        prop_assert_eq!(again.is_supertype, m.is_supertype);
    }

    #[test]
    fn clone_and_copy_agree(m in arb_symbol_metadata()) {
        let cloned = m;
        let copied = m;
        prop_assert_eq!(cloned.is_terminal, copied.is_terminal);
        prop_assert_eq!(cloned.is_visible, copied.is_visible);
        prop_assert_eq!(cloned.is_supertype, copied.is_supertype);
    }
}

// ---------------------------------------------------------------------------
// 3 – Debug formatting
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn debug_contains_struct_name(m in arb_symbol_metadata()) {
        let dbg = format!("{m:?}");
        prop_assert!(dbg.contains("SymbolMetadata"));
    }

    #[test]
    fn debug_contains_field_names(m in arb_symbol_metadata()) {
        let dbg = format!("{m:?}");
        prop_assert!(dbg.contains("is_terminal"));
        prop_assert!(dbg.contains("is_visible"));
        prop_assert!(dbg.contains("is_supertype"));
    }

    #[test]
    fn debug_contains_field_values(m in arb_symbol_metadata()) {
        let dbg = format!("{m:?}");
        prop_assert!(dbg.contains(&m.is_terminal.to_string()));
        prop_assert!(dbg.contains(&m.is_visible.to_string()));
        prop_assert!(dbg.contains(&m.is_supertype.to_string()));
    }

    #[test]
    fn debug_is_not_empty(m in arb_symbol_metadata()) {
        let dbg = format!("{m:?}");
        prop_assert!(!dbg.is_empty());
    }
}

// ---------------------------------------------------------------------------
// 4 – Default-like baseline (SymbolMetadata has no Default, so test
//     the all-false combo which acts as a canonical zero value)
// ---------------------------------------------------------------------------

#[test]
fn all_false_is_valid_metadata() {
    let m = SymbolMetadata {
        is_terminal: false,
        is_visible: false,
        is_supertype: false,
    };
    assert!(!m.is_terminal);
    assert!(!m.is_visible);
    assert!(!m.is_supertype);
}

#[test]
fn all_true_is_valid_metadata() {
    let m = SymbolMetadata {
        is_terminal: true,
        is_visible: true,
        is_supertype: true,
    };
    assert!(m.is_terminal);
    assert!(m.is_visible);
    assert!(m.is_supertype);
}

#[test]
fn all_false_clone_matches() {
    let m = SymbolMetadata {
        is_terminal: false,
        is_visible: false,
        is_supertype: false,
    };
    let c = m;
    assert_eq!(c.is_terminal, m.is_terminal);
    assert_eq!(c.is_visible, m.is_visible);
    assert_eq!(c.is_supertype, m.is_supertype);
}

// ---------------------------------------------------------------------------
// 5 – SymbolMetadata in Language
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn language_symbol_count_matches_metadata_len(meta in arb_symbol_metadata_vec(20)) {
        let lang = build_language_from_metadata(meta.clone());
        prop_assert_eq!(lang.symbol_count as usize, meta.len());
    }

    #[test]
    fn language_is_terminal_matches_metadata(meta in arb_symbol_metadata_vec(20)) {
        let lang = build_language_from_metadata(meta.clone());
        for i in 0..meta.len() {
            prop_assert_eq!(lang.is_terminal(i as u16), meta[i].is_terminal);
        }
    }

    #[test]
    fn language_is_visible_matches_metadata(meta in arb_symbol_metadata_vec(20)) {
        let lang = build_language_from_metadata(meta.clone());
        for i in 0..meta.len() {
            prop_assert_eq!(lang.is_visible(i as u16), meta[i].is_visible);
        }
    }

    #[test]
    fn language_symbol_name_matches_generated_names(meta in arb_symbol_metadata_vec(20)) {
        let lang = build_language_from_metadata(meta.clone());
        for i in 0..meta.len() {
            let expected_name = format!("sym_{i}");
            prop_assert_eq!(lang.symbol_name(i as u16), Some(expected_name.as_str()));
        }
    }

    #[test]
    fn language_out_of_bounds_is_terminal_returns_false(meta in arb_symbol_metadata_vec(10)) {
        let lang = build_language_from_metadata(meta.clone());
        let oob = meta.len() as u16;
        prop_assert!(!lang.is_terminal(oob));
        prop_assert!(!lang.is_terminal(oob.saturating_add(100)));
    }

    #[test]
    fn language_out_of_bounds_is_visible_returns_false(meta in arb_symbol_metadata_vec(10)) {
        let lang = build_language_from_metadata(meta.clone());
        let oob = meta.len() as u16;
        prop_assert!(!lang.is_visible(oob));
        prop_assert!(!lang.is_visible(oob.saturating_add(100)));
    }
}

// ---------------------------------------------------------------------------
// 6 – Multiple metadata entries consistency
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn vec_of_metadata_preserves_order(entries in arb_symbol_metadata_vec(30)) {
        let cloned: Vec<SymbolMetadata> = entries.to_vec();
        for i in 0..entries.len() {
            prop_assert_eq!(cloned[i].is_terminal, entries[i].is_terminal);
            prop_assert_eq!(cloned[i].is_visible, entries[i].is_visible);
            prop_assert_eq!(cloned[i].is_supertype, entries[i].is_supertype);
        }
    }

    #[test]
    fn metadata_vec_len_is_stable_after_clone(entries in arb_symbol_metadata_vec(30)) {
        let cloned = entries.clone();
        prop_assert_eq!(cloned.len(), entries.len());
    }

    #[test]
    fn two_languages_with_same_metadata_agree(meta in arb_symbol_metadata_vec(15)) {
        let lang1 = build_language_from_metadata(meta.clone());
        let lang2 = build_language_from_metadata(meta.clone());
        prop_assert_eq!(lang1.symbol_count, lang2.symbol_count);
        for i in 0..meta.len() {
            let idx = i as u16;
            prop_assert_eq!(lang1.is_terminal(idx), lang2.is_terminal(idx));
            prop_assert_eq!(lang1.is_visible(idx), lang2.is_visible(idx));
        }
    }

    #[test]
    fn language_clone_preserves_metadata(meta in arb_symbol_metadata_vec(15)) {
        let lang = build_language_from_metadata(meta.clone());
        let cloned_lang = lang.clone();
        prop_assert_eq!(cloned_lang.symbol_count, lang.symbol_count);
        for i in 0..meta.len() {
            let idx = i as u16;
            prop_assert_eq!(cloned_lang.is_terminal(idx), lang.is_terminal(idx));
            prop_assert_eq!(cloned_lang.is_visible(idx), lang.is_visible(idx));
            prop_assert_eq!(
                cloned_lang.symbol_metadata[i].is_supertype,
                lang.symbol_metadata[i].is_supertype
            );
        }
    }
}

// ---------------------------------------------------------------------------
// 7 – Random metadata generation
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn random_metadata_fields_roundtrip(
        t in any::<bool>(), v in any::<bool>(), s in any::<bool>()
    ) {
        let m = SymbolMetadata {
            is_terminal: t,
            is_visible: v,
            is_supertype: s,
        };
        prop_assert_eq!(m.is_terminal, t);
        prop_assert_eq!(m.is_visible, v);
        prop_assert_eq!(m.is_supertype, s);
    }

    #[test]
    fn random_metadata_debug_roundtrip_stable(m in arb_symbol_metadata()) {
        let dbg1 = format!("{m:?}");
        let dbg2 = format!("{m:?}");
        prop_assert_eq!(dbg1, dbg2);
    }

    #[test]
    fn random_metadata_copy_is_idempotent(m in arb_symbol_metadata()) {
        let a = m;
        let b = a;
        let c = b;
        prop_assert_eq!(c.is_terminal, m.is_terminal);
        prop_assert_eq!(c.is_visible, m.is_visible);
        prop_assert_eq!(c.is_supertype, m.is_supertype);
    }

    #[test]
    fn random_metadata_in_singleton_language(m in arb_symbol_metadata()) {
        let lang = build_language_from_metadata(vec![m]);
        prop_assert_eq!(lang.symbol_count, 1);
        prop_assert_eq!(lang.is_terminal(0), m.is_terminal);
        prop_assert_eq!(lang.is_visible(0), m.is_visible);
        prop_assert_eq!(lang.symbol_metadata[0].is_supertype, m.is_supertype);
    }

    #[test]
    fn random_pair_metadata_independent(
        m1 in arb_symbol_metadata(), m2 in arb_symbol_metadata()
    ) {
        let lang = build_language_from_metadata(vec![m1, m2]);
        prop_assert_eq!(lang.is_terminal(0), m1.is_terminal);
        prop_assert_eq!(lang.is_terminal(1), m2.is_terminal);
        prop_assert_eq!(lang.is_visible(0), m1.is_visible);
        prop_assert_eq!(lang.is_visible(1), m2.is_visible);
        prop_assert_eq!(lang.symbol_metadata[0].is_supertype, m1.is_supertype);
        prop_assert_eq!(lang.symbol_metadata[1].is_supertype, m2.is_supertype);
    }
}

// ---------------------------------------------------------------------------
// 8 – symbol_for_name interaction with metadata
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn symbol_for_name_respects_visibility(m in arb_symbol_metadata()) {
        let lang = build_language_from_metadata(vec![m]);
        let found_named = lang.symbol_for_name("sym_0", true);
        let found_anon = lang.symbol_for_name("sym_0", false);
        if m.is_visible {
            prop_assert_eq!(found_named, Some(0));
            prop_assert_eq!(found_anon, None);
        } else {
            prop_assert_eq!(found_named, None);
            prop_assert_eq!(found_anon, Some(0));
        }
    }

    #[test]
    fn symbol_for_name_nonexistent_returns_none(m in arb_symbol_metadata()) {
        let lang = build_language_from_metadata(vec![m]);
        prop_assert_eq!(lang.symbol_for_name("does_not_exist", true), None);
        prop_assert_eq!(lang.symbol_for_name("does_not_exist", false), None);
    }
}

// ---------------------------------------------------------------------------
// 9 – Edge cases and size_of
// ---------------------------------------------------------------------------

#[test]
fn symbol_metadata_is_small() {
    // 3 bools → should be at most 3 bytes (likely 3 with no padding)
    assert!(std::mem::size_of::<SymbolMetadata>() <= 4);
}

#[test]
fn empty_metadata_vec_builds_language_with_zero_symbols() {
    let lang = build_language_from_metadata(vec![]);
    assert_eq!(lang.symbol_count, 0);
    assert!(!lang.is_terminal(0));
    assert!(!lang.is_visible(0));
    assert_eq!(lang.symbol_name(0), None);
}

#[test]
fn large_metadata_vec_builds_successfully() {
    let meta: Vec<SymbolMetadata> = (0..256)
        .map(|i| SymbolMetadata {
            is_terminal: i % 2 == 0,
            is_visible: i % 3 == 0,
            is_supertype: i % 5 == 0,
        })
        .collect();
    let lang = build_language_from_metadata(meta.clone());
    assert_eq!(lang.symbol_count, 256);
    for i in 0..256 {
        assert_eq!(lang.symbol_metadata[i].is_terminal, meta[i].is_terminal);
        assert_eq!(lang.symbol_metadata[i].is_visible, meta[i].is_visible);
        assert_eq!(lang.symbol_metadata[i].is_supertype, meta[i].is_supertype);
    }
}
