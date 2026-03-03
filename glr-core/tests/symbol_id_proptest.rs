#![allow(clippy::needless_range_loop)]

//! Property-based tests for `SymbolId` usage in adze-glr-core.
//!
//! Run with: `cargo test -p adze-glr-core --test symbol_id_proptest`

use adze_glr_core::{Action, GotoIndexing, ParseRule, ParseTable, SymbolMetadata};
use adze_ir::{RuleId, StateId, SymbolId};
use proptest::prelude::*;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

// ---------------------------------------------------------------------------
// Strategies
// ---------------------------------------------------------------------------

/// Random SymbolId in range 0..=1000.
fn arb_symbol_id() -> impl Strategy<Value = SymbolId> {
    (0..=1000u16).prop_map(SymbolId)
}

/// Random SymbolId covering the full u16 range.
fn arb_symbol_id_full() -> impl Strategy<Value = SymbolId> {
    any::<u16>().prop_map(SymbolId)
}

/// Generate a sorted, deduplicated vec of SymbolId.
fn arb_symbol_id_set(max_len: usize) -> impl Strategy<Value = Vec<SymbolId>> {
    prop::collection::vec(arb_symbol_id(), 0..=max_len).prop_map(|mut v| {
        v.sort();
        v.dedup();
        v
    })
}

/// Build a minimal ParseTable with given terminal SymbolIds mapped.
fn build_table_with_terminals(terminals: &[SymbolId], states: usize) -> ParseTable {
    let mut table = ParseTable::default();
    table.state_count = states;
    for (idx, &sym) in terminals.iter().enumerate() {
        table.symbol_to_index.insert(sym, idx);
        if idx >= table.index_to_symbol.len() {
            table.index_to_symbol.resize(idx + 1, SymbolId(0));
        }
        table.index_to_symbol[idx] = sym;
    }
    let ncols = terminals.len();
    table.action_table = vec![vec![vec![]; ncols]; states];
    table.token_count = terminals.len();
    table
}

/// Build a minimal ParseTable with terminal and nonterminal SymbolIds.
fn build_table_with_both(
    terminals: &[SymbolId],
    nonterminals: &[SymbolId],
    states: usize,
) -> ParseTable {
    let mut table = build_table_with_terminals(terminals, states);
    for (idx, &sym) in nonterminals.iter().enumerate() {
        table.nonterminal_to_index.insert(sym, idx);
    }
    let nt_cols = nonterminals.len();
    table.goto_table = vec![vec![StateId(u16::MAX); nt_cols]; states];
    table
}

fn hash_of<T: Hash>(val: &T) -> u64 {
    let mut h = DefaultHasher::new();
    val.hash(&mut h);
    h.finish()
}

// ---------------------------------------------------------------------------
// 1. SymbolId conversion from/to u16
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn roundtrip_u16(raw in any::<u16>()) {
        let id = SymbolId(raw);
        prop_assert_eq!(id.0, raw);
    }
}

// ---------------------------------------------------------------------------
// 2. SymbolId Display format
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn display_format(raw in any::<u16>()) {
        let id = SymbolId(raw);
        let s = format!("{id}");
        prop_assert_eq!(s, format!("Symbol({raw})"));
    }
}

// ---------------------------------------------------------------------------
// 3. SymbolId Debug format contains inner value
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn debug_contains_value(raw in any::<u16>()) {
        let id = SymbolId(raw);
        let dbg = format!("{id:?}");
        prop_assert!(dbg.contains(&raw.to_string()));
    }
}

// ---------------------------------------------------------------------------
// 4. SymbolId equality is reflexive
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn equality_reflexive(raw in any::<u16>()) {
        let id = SymbolId(raw);
        prop_assert_eq!(id, id);
    }
}

// ---------------------------------------------------------------------------
// 5. SymbolId equality matches inner value
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn equality_by_inner(a in any::<u16>(), b in any::<u16>()) {
        let sa = SymbolId(a);
        let sb = SymbolId(b);
        prop_assert_eq!(sa == sb, a == b);
    }
}

// ---------------------------------------------------------------------------
// 6. SymbolId hash consistency: equal ids have equal hashes
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn hash_consistency(raw in any::<u16>()) {
        let a = SymbolId(raw);
        let b = SymbolId(raw);
        prop_assert_eq!(hash_of(&a), hash_of(&b));
    }
}

// ---------------------------------------------------------------------------
// 7. SymbolId hash in HashMap: insert and retrieve
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn hashmap_insert_retrieve(ids in arb_symbol_id_set(50)) {
        let mut map: HashMap<SymbolId, usize> = HashMap::new();
        for (i, &id) in ids.iter().enumerate() {
            map.insert(id, i);
        }
        for (i, &id) in ids.iter().enumerate() {
            prop_assert_eq!(map.get(&id), Some(&i));
        }
    }
}

// ---------------------------------------------------------------------------
// 8. SymbolId hash in HashSet: deduplication
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn hashset_dedup(raw in any::<u16>()) {
        let mut set = HashSet::new();
        set.insert(SymbolId(raw));
        set.insert(SymbolId(raw));
        prop_assert_eq!(set.len(), 1);
    }
}

// ---------------------------------------------------------------------------
// 9. SymbolId ordering is consistent with inner u16
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn ordering_consistent(a in any::<u16>(), b in any::<u16>()) {
        let sa = SymbolId(a);
        let sb = SymbolId(b);
        prop_assert_eq!(sa.cmp(&sb), a.cmp(&b));
    }
}

// ---------------------------------------------------------------------------
// 10. SymbolId in BTreeMap preserves sorted order
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn btreemap_sorted_order(ids in arb_symbol_id_set(50)) {
        let mut map = BTreeMap::new();
        for &id in &ids {
            map.insert(id, ());
        }
        let keys: Vec<u16> = map.keys().map(|s| s.0).collect();
        for i in 1..keys.len() {
            prop_assert!(keys[i - 1] < keys[i], "BTreeMap keys must be sorted");
        }
    }
}

// ---------------------------------------------------------------------------
// 11. SymbolId in BTreeSet preserves sorted order
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn btreeset_sorted_order(raws in prop::collection::vec(0..500u16, 0..60)) {
        let set: BTreeSet<SymbolId> = raws.iter().copied().map(SymbolId).collect();
        let vals: Vec<u16> = set.iter().map(|s| s.0).collect();
        for i in 1..vals.len() {
            prop_assert!(vals[i - 1] < vals[i]);
        }
    }
}

// ---------------------------------------------------------------------------
// 12. symbol_to_index: every mapped symbol resolves via actions()
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn action_lookup_mapped_symbol(
        terminals in arb_symbol_id_set(10),
    ) {
        prop_assume!(!terminals.is_empty());
        let table = build_table_with_terminals(&terminals, 2);
        let state = StateId(0);
        for &sym in &terminals {
            // Should not panic; may return empty slice
            let _actions = table.actions(state, sym);
        }
    }
}

// ---------------------------------------------------------------------------
// 13. actions() returns empty for unmapped symbol
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn action_lookup_unmapped_returns_empty(
        terminals in arb_symbol_id_set(5),
        extra in arb_symbol_id(),
    ) {
        prop_assume!(!terminals.contains(&extra));
        let table = build_table_with_terminals(&terminals, 1);
        let actions = table.actions(StateId(0), extra);
        prop_assert!(actions.is_empty(), "unmapped symbol should yield empty actions");
    }
}

// ---------------------------------------------------------------------------
// 14. actions() returns empty for out-of-range state
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn action_lookup_out_of_range_state(
        terminals in arb_symbol_id_set(3),
        state_raw in 10..100u16,
    ) {
        prop_assume!(!terminals.is_empty());
        let table = build_table_with_terminals(&terminals, 2);
        let actions = table.actions(StateId(state_raw), terminals[0]);
        prop_assert!(actions.is_empty());
    }
}

// ---------------------------------------------------------------------------
// 15. Shift action inserted and retrieved via symbol_to_index
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn shift_action_roundtrip(
        sym_raw in 1..200u16,
        target_state in 0..50u16,
    ) {
        let sym = SymbolId(sym_raw);
        let mut table = build_table_with_terminals(&[sym], 1);
        table.action_table[0][0] = vec![Action::Shift(StateId(target_state))];
        let actions = table.actions(StateId(0), sym);
        prop_assert!(actions.contains(&Action::Shift(StateId(target_state))));
    }
}

// ---------------------------------------------------------------------------
// 16. goto() returns None for unmapped nonterminal
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn goto_unmapped_returns_none(
        nt in arb_symbol_id(),
        extra in arb_symbol_id(),
    ) {
        prop_assume!(nt != extra);
        let table = build_table_with_both(&[], &[nt], 1);
        let result = table.goto(StateId(0), extra);
        prop_assert!(result.is_none(), "unmapped nonterminal should yield None");
    }
}

// ---------------------------------------------------------------------------
// 17. goto() returns None for sentinel (u16::MAX)
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn goto_sentinel_returns_none(
        nts in arb_symbol_id_set(5),
    ) {
        prop_assume!(!nts.is_empty());
        // Default goto_table is filled with StateId(u16::MAX)
        let table = build_table_with_both(&[], &nts, 1);
        for &nt in &nts {
            let result = table.goto(StateId(0), nt);
            prop_assert!(result.is_none(), "sentinel value should be treated as no-edge");
        }
    }
}

// ---------------------------------------------------------------------------
// 18. goto() returns valid state when set
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn goto_valid_state(
        nt_raw in 10..200u16,
        target in 0..50u16,
    ) {
        let nt = SymbolId(nt_raw);
        let mut table = build_table_with_both(&[], &[nt], 2);
        table.goto_table[0][0] = StateId(target);
        let result = table.goto(StateId(0), nt);
        if target == u16::MAX {
            prop_assert!(result.is_none());
        } else {
            prop_assert_eq!(result, Some(StateId(target)));
        }
    }
}

// ---------------------------------------------------------------------------
// 19. goto() out-of-range state returns None
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn goto_out_of_range_state(
        nts in arb_symbol_id_set(3),
        state_raw in 10..100u16,
    ) {
        prop_assume!(!nts.is_empty());
        let table = build_table_with_both(&[], &nts, 2);
        let result = table.goto(StateId(state_raw), nts[0]);
        prop_assert!(result.is_none());
    }
}

// ---------------------------------------------------------------------------
// 20. symbol_to_index is bijective with index_to_symbol
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn symbol_index_bijection(terminals in arb_symbol_id_set(20)) {
        let table = build_table_with_terminals(&terminals, 1);
        for (&sym, &idx) in &table.symbol_to_index {
            prop_assert!(idx < table.index_to_symbol.len());
            prop_assert_eq!(table.index_to_symbol[idx], sym);
        }
    }
}

// ---------------------------------------------------------------------------
// 21. nonterminal_to_index maps are disjoint from symbol_to_index
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn terminal_nonterminal_maps_disjoint(
        terms in arb_symbol_id_set(10),
        nts_raw in prop::collection::vec(500..700u16, 0..10),
    ) {
        let nts: Vec<SymbolId> = {
            let mut v: Vec<SymbolId> = nts_raw.into_iter().map(SymbolId).collect();
            v.sort();
            v.dedup();
            v.retain(|s| !terms.contains(s));
            v
        };
        let table = build_table_with_both(&terms, &nts, 1);
        for key in table.nonterminal_to_index.keys() {
            prop_assert!(
                !table.symbol_to_index.contains_key(key),
                "nonterminal {key} should not appear in symbol_to_index"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// 22. ParseRule.lhs is a valid SymbolId
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn parse_rule_lhs_roundtrip(lhs_raw in any::<u16>(), rhs_len in 0..20u16) {
        let rule = ParseRule {
            lhs: SymbolId(lhs_raw),
            rhs_len,
        };
        prop_assert_eq!(rule.lhs.0, lhs_raw);
        prop_assert_eq!(rule.rhs_len, rhs_len);
    }
}

// ---------------------------------------------------------------------------
// 23. ParseTable.rule() returns correct SymbolId for lhs
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn table_rule_lhs_lookup(
        lhs_raw in 0..500u16,
        rhs_len in 0..10u16,
    ) {
        let mut table = ParseTable::default();
        table.rules.push(ParseRule {
            lhs: SymbolId(lhs_raw),
            rhs_len,
        });
        let (lhs, len) = table.rule(RuleId(0));
        prop_assert_eq!(lhs, SymbolId(lhs_raw));
        prop_assert_eq!(len, rhs_len);
    }
}

// ---------------------------------------------------------------------------
// 24. eof_symbol and start_symbol are valid SymbolIds
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn eof_and_start_symbol_valid(eof in arb_symbol_id(), start in arb_symbol_id()) {
        let mut table = ParseTable::default();
        table.eof_symbol = eof;
        table.start_symbol = start;
        prop_assert_eq!(table.eof(), eof);
        prop_assert_eq!(table.start_symbol(), start);
    }
}

// ---------------------------------------------------------------------------
// 25. is_terminal checks against terminal_boundary
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn is_terminal_boundary(
        tok_count in 1..50usize,
        ext_count in 0..10usize,
        sym_raw in 0..100u16,
    ) {
        let mut table = ParseTable::default();
        table.token_count = tok_count;
        table.external_token_count = ext_count;
        let boundary = tok_count + ext_count;
        let sym = SymbolId(sym_raw);
        prop_assert_eq!(
            table.is_terminal(sym),
            (sym_raw as usize) < boundary,
        );
    }
}

// ---------------------------------------------------------------------------
// 26. SymbolId clone produces identical value
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn clone_identical(raw in any::<u16>()) {
        let original = SymbolId(raw);
        let cloned = original;
        prop_assert_eq!(original, cloned);
        prop_assert_eq!(hash_of(&original), hash_of(&cloned));
    }
}

// ---------------------------------------------------------------------------
// 27. SymbolId in extras vec round-trips
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn extras_roundtrip(extras in arb_symbol_id_set(10)) {
        let mut table = ParseTable::default();
        table.extras = extras.clone();
        prop_assert_eq!(table.extras.len(), extras.len());
        for i in 0..extras.len() {
            prop_assert_eq!(table.extras[i], extras[i]);
        }
    }
}

// ---------------------------------------------------------------------------
// 28. SymbolMetadata.symbol_id matches stored SymbolId
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn symbol_metadata_id_matches(raw in any::<u16>()) {
        let meta = SymbolMetadata {
            name: String::from("test"),
            is_visible: true,
            is_named: true,
            is_supertype: false,
            is_terminal: true,
            is_extra: false,
            is_fragile: false,
            symbol_id: SymbolId(raw),
        };
        prop_assert_eq!(meta.symbol_id, SymbolId(raw));
    }
}

// ---------------------------------------------------------------------------
// 29. Multiple actions for same symbol in one state
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn multiple_actions_per_symbol(
        sym_raw in 1..200u16,
        shift_target in 0..50u16,
        reduce_rule in 0..50u16,
    ) {
        let sym = SymbolId(sym_raw);
        let mut table = build_table_with_terminals(&[sym], 1);
        table.action_table[0][0] = vec![
            Action::Shift(StateId(shift_target)),
            Action::Reduce(RuleId(reduce_rule)),
        ];
        let actions = table.actions(StateId(0), sym);
        prop_assert_eq!(actions.len(), 2);
        prop_assert!(actions.contains(&Action::Shift(StateId(shift_target))));
        prop_assert!(actions.contains(&Action::Reduce(RuleId(reduce_rule))));
    }
}

// ---------------------------------------------------------------------------
// 30. SymbolId range: full u16 range is valid
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn full_u16_range_valid(raw in any::<u16>()) {
        let id = SymbolId(raw);
        // Must not panic; Display must work
        let _ = format!("{id}");
        let _ = format!("{id:?}");
        // Equality and hash must work
        prop_assert_eq!(id, SymbolId(raw));
        let _ = hash_of(&id);
    }
}

// ---------------------------------------------------------------------------
// 31. BTreeMap key ordering matches SymbolId natural order
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn btreemap_key_ordering_matches_natural(
        pairs in prop::collection::vec((arb_symbol_id(), 0..1000u32), 1..30),
    ) {
        let map: BTreeMap<SymbolId, u32> = pairs.into_iter().collect();
        let keys: Vec<u16> = map.keys().map(|k| k.0).collect();
        // BTreeMap guarantees sorted keys; verify SymbolId Ord is u16-natural
        let mut sorted = keys.clone();
        sorted.sort();
        prop_assert_eq!(keys, sorted);
    }
}

// ---------------------------------------------------------------------------
// 32. SymbolId partial ordering is total
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn partial_ord_is_total(a in any::<u16>(), b in any::<u16>()) {
        let sa = SymbolId(a);
        let sb = SymbolId(b);
        // PartialOrd should always return Some
        prop_assert!(sa.partial_cmp(&sb).is_some());
    }
}

// ---------------------------------------------------------------------------
// 33. alias_sequences stores SymbolId options correctly
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn alias_sequences_roundtrip(
        ids in prop::collection::vec(prop::option::of(arb_symbol_id()), 0..10),
    ) {
        let mut table = ParseTable::default();
        table.alias_sequences.push(ids.clone());
        prop_assert_eq!(table.alias_sequences[0].len(), ids.len());
        for i in 0..ids.len() {
            prop_assert_eq!(table.alias_sequences[0][i], ids[i]);
        }
    }
}
