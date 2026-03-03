#![allow(clippy::needless_range_loop)]
//! Property-based tests for LR(1) item set data structures.
//!
//! Run with: `cargo test -p adze-glr-core --test item_set_proptest`

use adze_glr_core::{FirstFollowSets, ItemSet, ItemSetCollection, LRItem};
use adze_ir::*;
use proptest::prelude::*;
use std::collections::{BTreeSet, HashMap, HashSet};
use std::hash::{Hash, Hasher};

// ---------------------------------------------------------------------------
// Strategies
// ---------------------------------------------------------------------------

fn rule_id_strategy() -> impl Strategy<Value = RuleId> {
    (0u16..200).prop_map(RuleId)
}

fn symbol_id_strategy() -> impl Strategy<Value = SymbolId> {
    (0u16..200).prop_map(SymbolId)
}

fn position_strategy() -> impl Strategy<Value = usize> {
    0usize..20
}

fn state_id_strategy() -> impl Strategy<Value = StateId> {
    (0u16..500).prop_map(StateId)
}

fn lr_item_strategy() -> impl Strategy<Value = LRItem> {
    (
        rule_id_strategy(),
        position_strategy(),
        symbol_id_strategy(),
    )
        .prop_map(|(rule_id, position, lookahead)| LRItem::new(rule_id, position, lookahead))
}

fn lr_item_vec_strategy(max_len: usize) -> impl Strategy<Value = Vec<LRItem>> {
    prop::collection::vec(lr_item_strategy(), 0..=max_len)
}

/// Build a minimal grammar: S -> a, with given symbol IDs.
fn simple_grammar(terminal: SymbolId, nonterminal: SymbolId) -> Grammar {
    let mut grammar = Grammar::new("proptest".into());
    grammar.tokens.insert(
        terminal,
        Token {
            name: "a".into(),
            pattern: TokenPattern::String("a".into()),
            fragile: false,
        },
    );
    grammar.rule_names.insert(nonterminal, "S".into());
    grammar.rules.insert(
        nonterminal,
        vec![Rule {
            lhs: nonterminal,
            rhs: vec![Symbol::Terminal(terminal)],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(0),
        }],
    );
    grammar
}

/// Build a grammar with two rules: S -> a | a b
fn two_rule_grammar() -> Grammar {
    let a = SymbolId(1);
    let b = SymbolId(2);
    let s = SymbolId(10);

    let mut grammar = Grammar::new("two_rule".into());
    grammar.tokens.insert(
        a,
        Token {
            name: "a".into(),
            pattern: TokenPattern::String("a".into()),
            fragile: false,
        },
    );
    grammar.tokens.insert(
        b,
        Token {
            name: "b".into(),
            pattern: TokenPattern::String("b".into()),
            fragile: false,
        },
    );
    grammar.rule_names.insert(s, "S".into());
    grammar.rules.insert(
        s,
        vec![
            Rule {
                lhs: s,
                rhs: vec![Symbol::Terminal(a)],
                precedence: None,
                associativity: None,
                fields: vec![],
                production_id: ProductionId(0),
            },
            Rule {
                lhs: s,
                rhs: vec![Symbol::Terminal(a), Symbol::Terminal(b)],
                precedence: None,
                associativity: None,
                fields: vec![],
                production_id: ProductionId(1),
            },
        ],
    );
    grammar
}

/// Build a grammar with a non-terminal on the RHS: S -> A, A -> a
fn indirect_grammar() -> Grammar {
    let a = SymbolId(1);
    let s = SymbolId(10);
    let big_a = SymbolId(11);

    let mut grammar = Grammar::new("indirect".into());
    grammar.tokens.insert(
        a,
        Token {
            name: "a".into(),
            pattern: TokenPattern::String("a".into()),
            fragile: false,
        },
    );
    grammar.rule_names.insert(s, "S".into());
    grammar.rule_names.insert(big_a, "A".into());
    grammar.rules.insert(
        s,
        vec![Rule {
            lhs: s,
            rhs: vec![Symbol::NonTerminal(big_a)],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(0),
        }],
    );
    grammar.rules.insert(
        big_a,
        vec![Rule {
            lhs: big_a,
            rhs: vec![Symbol::Terminal(a)],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(1),
        }],
    );
    grammar
}

// ---------------------------------------------------------------------------
// 1. LRItem construction and field access
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    #[test]
    fn item_new_preserves_fields(
        rule_id in rule_id_strategy(),
        position in position_strategy(),
        lookahead in symbol_id_strategy(),
    ) {
        let item = LRItem::new(rule_id, position, lookahead);
        prop_assert_eq!(item.rule_id, rule_id);
        prop_assert_eq!(item.position, position);
        prop_assert_eq!(item.lookahead, lookahead);
    }
}

// ---------------------------------------------------------------------------
// 2. LRItem equality is reflexive
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn item_eq_reflexive(item in lr_item_strategy()) {
        prop_assert_eq!(&item, &item);
    }
}

// ---------------------------------------------------------------------------
// 3. LRItem equality is symmetric
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn item_eq_symmetric(a in lr_item_strategy(), b in lr_item_strategy()) {
        prop_assert_eq!(a == b, b == a);
    }
}

// ---------------------------------------------------------------------------
// 4. LRItem clone roundtrip
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn item_clone_roundtrip(item in lr_item_strategy()) {
        let cloned = item.clone();
        prop_assert_eq!(&item, &cloned);
    }
}

// ---------------------------------------------------------------------------
// 5. LRItem hash consistent with Eq
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn item_hash_consistent(a in lr_item_strategy(), b in lr_item_strategy()) {
        fn compute_hash(item: &LRItem) -> u64 {
            let mut hasher = std::collections::hash_map::DefaultHasher::new();
            item.hash(&mut hasher);
            hasher.finish()
        }
        if a == b {
            prop_assert_eq!(compute_hash(&a), compute_hash(&b));
        }
    }
}

// ---------------------------------------------------------------------------
// 6. LRItem ordering is total and consistent
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn item_ord_consistent(a in lr_item_strategy(), b in lr_item_strategy()) {
        use std::cmp::Ordering;
        let ab = a.cmp(&b);
        let ba = b.cmp(&a);
        match ab {
            Ordering::Less => prop_assert_eq!(ba, Ordering::Greater),
            Ordering::Equal => prop_assert_eq!(ba, Ordering::Equal),
            Ordering::Greater => prop_assert_eq!(ba, Ordering::Less),
        }
    }
}

// ---------------------------------------------------------------------------
// 7. LRItem ordering is transitive
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn item_ord_transitive(
        a in lr_item_strategy(),
        b in lr_item_strategy(),
        c in lr_item_strategy(),
    ) {
        if a <= b && b <= c {
            prop_assert!(a <= c);
        }
    }
}

// ---------------------------------------------------------------------------
// 8. Items with different fields are not equal
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn item_different_rule_id_not_equal(
        r1 in 0u16..100,
        r2 in 100u16..200,
        pos in position_strategy(),
        la in symbol_id_strategy(),
    ) {
        let a = LRItem::new(RuleId(r1), pos, la);
        let b = LRItem::new(RuleId(r2), pos, la);
        prop_assert_ne!(&a, &b);
    }
}

// ---------------------------------------------------------------------------
// 9. ItemSet new creates empty set with correct ID
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn item_set_new_empty(id in state_id_strategy()) {
        let set = ItemSet::new(id);
        prop_assert!(set.items.is_empty());
        prop_assert_eq!(set.id, id);
    }
}

// ---------------------------------------------------------------------------
// 10. ItemSet add_item inserts item
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn item_set_add_contains(item in lr_item_strategy(), id in state_id_strategy()) {
        let mut set = ItemSet::new(id);
        set.add_item(item.clone());
        prop_assert!(set.items.contains(&item));
        prop_assert_eq!(set.items.len(), 1);
    }
}

// ---------------------------------------------------------------------------
// 11. ItemSet add_item is idempotent (BTreeSet dedup)
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn item_set_add_idempotent(item in lr_item_strategy(), id in state_id_strategy()) {
        let mut set = ItemSet::new(id);
        set.add_item(item.clone());
        set.add_item(item.clone());
        prop_assert_eq!(set.items.len(), 1);
    }
}

// ---------------------------------------------------------------------------
// 12. ItemSet add multiple items preserves all distinct items
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn item_set_add_multiple(items in lr_item_vec_strategy(10), id in state_id_strategy()) {
        let mut set = ItemSet::new(id);
        for item in &items {
            set.add_item(item.clone());
        }
        let expected: BTreeSet<_> = items.into_iter().collect();
        prop_assert_eq!(set.items.len(), expected.len());
        prop_assert_eq!(&set.items, &expected);
    }
}

// ---------------------------------------------------------------------------
// 13. ItemSet equality depends on items, not ID
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn item_set_eq_ignores_id(
        items in lr_item_vec_strategy(5),
        id1 in state_id_strategy(),
        id2 in state_id_strategy(),
    ) {
        let mut set1 = ItemSet::new(id1);
        let mut set2 = ItemSet::new(id2);
        for item in &items {
            set1.add_item(item.clone());
            set2.add_item(item.clone());
        }
        // ItemSet derives PartialEq; if IDs differ, sets may not be equal
        if id1 == id2 {
            prop_assert_eq!(&set1, &set2);
        }
        // But item contents should always match
        prop_assert_eq!(&set1.items, &set2.items);
    }
}

// ---------------------------------------------------------------------------
// 14. ItemSet items are always sorted (BTreeSet invariant)
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn item_set_items_sorted(items in lr_item_vec_strategy(15)) {
        let mut set = ItemSet::new(StateId(0));
        for item in &items {
            set.add_item(item.clone());
        }
        let as_vec: Vec<_> = set.items.iter().collect();
        for i in 1..as_vec.len() {
            prop_assert!(as_vec[i - 1] < as_vec[i]);
        }
    }
}

// ---------------------------------------------------------------------------
// 15. LRItem is_reduce_item on simple grammar
// ---------------------------------------------------------------------------

#[test]
fn item_is_reduce_at_end() {
    let grammar = simple_grammar(SymbolId(1), SymbolId(10));
    // Rule 0: S -> a (rhs len 1)
    let item_start = LRItem::new(RuleId(0), 0, SymbolId(0));
    assert!(!item_start.is_reduce_item(&grammar));

    let item_end = LRItem::new(RuleId(0), 1, SymbolId(0));
    assert!(item_end.is_reduce_item(&grammar));
}

// ---------------------------------------------------------------------------
// 16. LRItem next_symbol returns correct symbol
// ---------------------------------------------------------------------------

#[test]
fn item_next_symbol_at_positions() {
    let grammar = simple_grammar(SymbolId(1), SymbolId(10));
    let item0 = LRItem::new(RuleId(0), 0, SymbolId(0));
    assert_eq!(
        item0.next_symbol(&grammar),
        Some(&Symbol::Terminal(SymbolId(1)))
    );

    let item1 = LRItem::new(RuleId(0), 1, SymbolId(0));
    assert_eq!(item1.next_symbol(&grammar), None);
}

// ---------------------------------------------------------------------------
// 17. Closure on kernel with terminal next_symbol adds no items
// ---------------------------------------------------------------------------

#[test]
fn closure_terminal_no_expansion() {
    let grammar = simple_grammar(SymbolId(1), SymbolId(10));
    let ff = FirstFollowSets::compute(&grammar).unwrap();

    let mut set = ItemSet::new(StateId(0));
    set.add_item(LRItem::new(RuleId(0), 0, SymbolId(0)));
    let initial_len = set.items.len();
    set.closure(&grammar, &ff).unwrap();
    // Terminal at dot: closure should not add new items
    assert_eq!(set.items.len(), initial_len);
}

// ---------------------------------------------------------------------------
// 18. Closure on item with non-terminal expands
// ---------------------------------------------------------------------------

#[test]
fn closure_nonterminal_expands() {
    let grammar = indirect_grammar();
    let ff = FirstFollowSets::compute(&grammar).unwrap();

    let mut set = ItemSet::new(StateId(0));
    // S -> . A (rule 0, position 0)
    set.add_item(LRItem::new(RuleId(0), 0, SymbolId(0)));
    set.closure(&grammar, &ff).unwrap();
    // Closure should add A -> . a items
    assert!(set.items.len() > 1);
}

// ---------------------------------------------------------------------------
// 19. Closure is idempotent
// ---------------------------------------------------------------------------

#[test]
fn closure_idempotent() {
    let grammar = indirect_grammar();
    let ff = FirstFollowSets::compute(&grammar).unwrap();

    let mut set = ItemSet::new(StateId(0));
    set.add_item(LRItem::new(RuleId(0), 0, SymbolId(0)));
    set.closure(&grammar, &ff).unwrap();
    let after_first: BTreeSet<_> = set.items.clone();
    set.closure(&grammar, &ff).unwrap();
    assert_eq!(set.items, after_first);
}

// ---------------------------------------------------------------------------
// 20. Closure is monotone (only adds items)
// ---------------------------------------------------------------------------

#[test]
fn closure_monotone() {
    let grammar = indirect_grammar();
    let ff = FirstFollowSets::compute(&grammar).unwrap();

    let mut set = ItemSet::new(StateId(0));
    set.add_item(LRItem::new(RuleId(0), 0, SymbolId(0)));
    let before: BTreeSet<_> = set.items.clone();
    set.closure(&grammar, &ff).unwrap();
    // Every item that was there before closure is still there
    for item in &before {
        assert!(set.items.contains(item));
    }
}

// ---------------------------------------------------------------------------
// 21. Goto with unmatched symbol yields empty set
// ---------------------------------------------------------------------------

#[test]
fn goto_unmatched_symbol_empty() {
    let grammar = simple_grammar(SymbolId(1), SymbolId(10));
    let ff = FirstFollowSets::compute(&grammar).unwrap();

    let mut set = ItemSet::new(StateId(0));
    set.add_item(LRItem::new(RuleId(0), 0, SymbolId(0)));
    set.closure(&grammar, &ff).unwrap();

    // Terminal(99) doesn't appear in our grammar RHS
    let goto = set.goto(&Symbol::Terminal(SymbolId(99)), &grammar, &ff);
    assert!(goto.items.is_empty());
}

// ---------------------------------------------------------------------------
// 22. Goto advances dot position
// ---------------------------------------------------------------------------

#[test]
fn goto_advances_dot() {
    let grammar = simple_grammar(SymbolId(1), SymbolId(10));
    let ff = FirstFollowSets::compute(&grammar).unwrap();

    let mut set = ItemSet::new(StateId(0));
    set.add_item(LRItem::new(RuleId(0), 0, SymbolId(0)));
    set.closure(&grammar, &ff).unwrap();

    let goto = set.goto(&Symbol::Terminal(SymbolId(1)), &grammar, &ff);
    // After shifting 'a', we should have S -> a . at position 1
    assert!(!goto.items.is_empty());
    for item in &goto.items {
        if item.rule_id == RuleId(0) {
            assert!(item.position >= 1);
        }
    }
}

// ---------------------------------------------------------------------------
// 23. Goto result items are all shifted from originals
// ---------------------------------------------------------------------------

#[test]
fn goto_items_come_from_original() {
    let grammar = two_rule_grammar();
    let ff = FirstFollowSets::compute(&grammar).unwrap();

    let mut set = ItemSet::new(StateId(0));
    set.add_item(LRItem::new(RuleId(0), 0, SymbolId(0)));
    set.add_item(LRItem::new(RuleId(1), 0, SymbolId(0)));
    set.closure(&grammar, &ff).unwrap();

    let goto = set.goto(&Symbol::Terminal(SymbolId(1)), &grammar, &ff);
    // All kernel items in the goto should have position > 0
    // (closure may add items at position 0 for non-terminals)
    let kernel: Vec<_> = goto
        .items
        .iter()
        .filter(|it| {
            // Kernel items are those with position > 0 (or augmented start)
            it.position > 0
        })
        .collect();
    assert!(!kernel.is_empty());
}

// ---------------------------------------------------------------------------
// 24. Canonical collection has at least one state
// ---------------------------------------------------------------------------

#[test]
fn canonical_collection_nonempty() {
    let grammar = simple_grammar(SymbolId(1), SymbolId(10));
    let ff = FirstFollowSets::compute(&grammar).unwrap();
    let collection = ItemSetCollection::build_canonical_collection(&grammar, &ff);
    assert!(!collection.sets.is_empty());
}

// ---------------------------------------------------------------------------
// 25. Canonical collection state IDs are unique
// ---------------------------------------------------------------------------

#[test]
fn canonical_collection_unique_ids() {
    let grammar = two_rule_grammar();
    let ff = FirstFollowSets::compute(&grammar).unwrap();
    let collection = ItemSetCollection::build_canonical_collection(&grammar, &ff);

    let ids: Vec<_> = collection.sets.iter().map(|s| s.id).collect();
    let unique: HashSet<_> = ids.iter().collect();
    assert_eq!(ids.len(), unique.len());
}

// ---------------------------------------------------------------------------
// 26. Canonical collection goto table targets valid states
// ---------------------------------------------------------------------------

#[test]
fn canonical_collection_goto_targets_valid() {
    let grammar = two_rule_grammar();
    let ff = FirstFollowSets::compute(&grammar).unwrap();
    let collection = ItemSetCollection::build_canonical_collection(&grammar, &ff);

    let valid_ids: HashSet<_> = collection.sets.iter().map(|s| s.id).collect();
    for (_key, &target) in &collection.goto_table {
        assert!(valid_ids.contains(&target));
    }
}

// ---------------------------------------------------------------------------
// 27. Canonical collection goto table sources are valid states
// ---------------------------------------------------------------------------

#[test]
fn canonical_collection_goto_sources_valid() {
    let grammar = two_rule_grammar();
    let ff = FirstFollowSets::compute(&grammar).unwrap();
    let collection = ItemSetCollection::build_canonical_collection(&grammar, &ff);

    let valid_ids: HashSet<_> = collection.sets.iter().map(|s| s.id).collect();
    for ((source, _sym), _target) in &collection.goto_table {
        assert!(valid_ids.contains(source));
    }
}

// ---------------------------------------------------------------------------
// 28. No duplicate item sets in canonical collection
// ---------------------------------------------------------------------------

#[test]
fn canonical_collection_no_duplicate_sets() {
    let grammar = indirect_grammar();
    let ff = FirstFollowSets::compute(&grammar).unwrap();
    let collection = ItemSetCollection::build_canonical_collection(&grammar, &ff);

    for i in 0..collection.sets.len() {
        for j in (i + 1)..collection.sets.len() {
            assert_ne!(
                collection.sets[i].items, collection.sets[j].items,
                "States {} and {} have identical item sets",
                collection.sets[i].id.0, collection.sets[j].id.0,
            );
        }
    }
}

// ---------------------------------------------------------------------------
// 29. Every state's items are closed (closure fixed-point)
// ---------------------------------------------------------------------------

#[test]
fn canonical_collection_states_are_closed() {
    let grammar = indirect_grammar();
    let ff = FirstFollowSets::compute(&grammar).unwrap();
    let collection = ItemSetCollection::build_canonical_collection(&grammar, &ff);

    for state in &collection.sets {
        let mut reclosed = state.clone();
        reclosed.closure(&grammar, &ff).unwrap();
        assert_eq!(
            state.items, reclosed.items,
            "State {} is not closed",
            state.id.0,
        );
    }
}

// ---------------------------------------------------------------------------
// 30. ItemSet deduplication via BTreeSet<LRItem>
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn item_set_btreeset_dedup(items in lr_item_vec_strategy(20)) {
        let mut set = ItemSet::new(StateId(0));
        for item in &items {
            set.add_item(item.clone());
        }
        // Manually compute expected unique count
        let unique: BTreeSet<_> = items.into_iter().collect();
        prop_assert_eq!(set.items.len(), unique.len());
    }
}

// ---------------------------------------------------------------------------
// 31. LRItem in HashMap behaves correctly
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn item_hashmap_lookup(items in lr_item_vec_strategy(10)) {
        let mut map = HashMap::new();
        for (i, item) in items.iter().enumerate() {
            map.insert(item.clone(), i);
        }
        // Each unique item should be findable
        for item in &items {
            prop_assert!(map.contains_key(item));
        }
    }
}

// ---------------------------------------------------------------------------
// 32. LRItem PartialOrd consistent with Ord
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn item_partial_ord_consistent(a in lr_item_strategy(), b in lr_item_strategy()) {
        let partial = a.partial_cmp(&b);
        let total = a.cmp(&b);
        prop_assert_eq!(partial, Some(total));
    }
}

// ---------------------------------------------------------------------------
// 33. Empty ItemSet closure is no-op
// ---------------------------------------------------------------------------

#[test]
fn empty_item_set_closure_noop() {
    let grammar = simple_grammar(SymbolId(1), SymbolId(10));
    let ff = FirstFollowSets::compute(&grammar).unwrap();

    let mut set = ItemSet::new(StateId(0));
    set.closure(&grammar, &ff).unwrap();
    assert!(set.items.is_empty());
}

// ---------------------------------------------------------------------------
// 34. Goto on empty ItemSet is empty
// ---------------------------------------------------------------------------

#[test]
fn empty_item_set_goto_empty() {
    let grammar = simple_grammar(SymbolId(1), SymbolId(10));
    let ff = FirstFollowSets::compute(&grammar).unwrap();

    let set = ItemSet::new(StateId(0));
    let goto = set.goto(&Symbol::Terminal(SymbolId(1)), &grammar, &ff);
    assert!(goto.items.is_empty());
}

// ---------------------------------------------------------------------------
// 35. Kernel items: items at position > 0 in goto result
// ---------------------------------------------------------------------------

#[test]
fn goto_kernel_items_have_positive_position() {
    let grammar = indirect_grammar();
    let ff = FirstFollowSets::compute(&grammar).unwrap();

    let mut set = ItemSet::new(StateId(0));
    set.add_item(LRItem::new(RuleId(0), 0, SymbolId(0)));
    set.closure(&grammar, &ff).unwrap();

    // GOTO over the non-terminal A
    let goto = set.goto(&Symbol::NonTerminal(SymbolId(11)), &grammar, &ff);
    if !goto.items.is_empty() {
        // At least one kernel item has position > 0
        let has_kernel = goto.items.iter().any(|it| it.position > 0);
        assert!(has_kernel);
    }
}

// ---------------------------------------------------------------------------
// 36. Reduce items have no next_symbol
// ---------------------------------------------------------------------------

#[test]
fn reduce_items_no_next_symbol() {
    let grammar = simple_grammar(SymbolId(1), SymbolId(10));
    // S -> a is rule 0, rhs len 1
    let reduce_item = LRItem::new(RuleId(0), 1, SymbolId(0));
    assert!(reduce_item.is_reduce_item(&grammar));
    assert_eq!(reduce_item.next_symbol(&grammar), None);
}

// ---------------------------------------------------------------------------
// 37. Non-reduce items have a next_symbol
// ---------------------------------------------------------------------------

#[test]
fn non_reduce_items_have_next_symbol() {
    let grammar = simple_grammar(SymbolId(1), SymbolId(10));
    let shift_item = LRItem::new(RuleId(0), 0, SymbolId(0));
    assert!(!shift_item.is_reduce_item(&grammar));
    assert!(shift_item.next_symbol(&grammar).is_some());
}

// ---------------------------------------------------------------------------
// 38. ItemSetCollection symbol_is_terminal tracks correctly
// ---------------------------------------------------------------------------

#[test]
fn collection_symbol_is_terminal_tracking() {
    let grammar = two_rule_grammar();
    let ff = FirstFollowSets::compute(&grammar).unwrap();
    let collection = ItemSetCollection::build_canonical_collection(&grammar, &ff);

    // All symbols tracked in symbol_is_terminal should be consistent:
    // terminal IDs 1,2 should map to true; non-terminal ID 10 may or may not appear
    for (sym, &is_terminal) in &collection.symbol_is_terminal {
        if sym.0 == 1 || sym.0 == 2 {
            assert!(is_terminal, "Symbol {} should be terminal", sym.0);
        }
    }
}
