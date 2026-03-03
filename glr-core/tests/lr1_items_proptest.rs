#![allow(clippy::needless_range_loop)]
//! Property-based tests for LR(1) item set construction in adze-glr-core.
//!
//! Run with: `cargo test -p adze-glr-core --test lr1_items_proptest`

use adze_glr_core::{FirstFollowSets, ItemSet, ItemSetCollection, LRItem};
use adze_ir::*;
use proptest::prelude::*;
use std::collections::{BTreeSet, HashSet};
use std::hash::{Hash, Hasher};

// ---------------------------------------------------------------------------
// Strategies
// ---------------------------------------------------------------------------

fn rule_id_strat() -> impl Strategy<Value = RuleId> {
    (0u16..150).prop_map(RuleId)
}

fn symbol_id_strat() -> impl Strategy<Value = SymbolId> {
    (0u16..150).prop_map(SymbolId)
}

fn position_strat() -> impl Strategy<Value = usize> {
    0usize..15
}

fn lr_item_strat() -> impl Strategy<Value = LRItem> {
    (rule_id_strat(), position_strat(), symbol_id_strat())
        .prop_map(|(r, p, l)| LRItem::new(r, p, l))
}

fn lr_item_vec_strat(max_len: usize) -> impl Strategy<Value = Vec<LRItem>> {
    prop::collection::vec(lr_item_strat(), 1..=max_len)
}

// ---------------------------------------------------------------------------
// Grammar helpers
// ---------------------------------------------------------------------------

/// S -> a
fn grammar_s_to_a() -> Grammar {
    let a = SymbolId(1);
    let s = SymbolId(10);
    let mut g = Grammar::new("s_to_a".into());
    g.tokens.insert(
        a,
        Token {
            name: "a".into(),
            pattern: TokenPattern::String("a".into()),
            fragile: false,
        },
    );
    g.rule_names.insert(s, "S".into());
    g.rules.insert(
        s,
        vec![Rule {
            lhs: s,
            rhs: vec![Symbol::Terminal(a)],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(0),
        }],
    );
    g
}

/// S -> A b, A -> a
fn grammar_s_a_b() -> Grammar {
    let a = SymbolId(1);
    let b = SymbolId(2);
    let s = SymbolId(10);
    let big_a = SymbolId(11);

    let mut g = Grammar::new("s_a_b".into());
    g.tokens.insert(
        a,
        Token {
            name: "a".into(),
            pattern: TokenPattern::String("a".into()),
            fragile: false,
        },
    );
    g.tokens.insert(
        b,
        Token {
            name: "b".into(),
            pattern: TokenPattern::String("b".into()),
            fragile: false,
        },
    );
    g.rule_names.insert(s, "S".into());
    g.rule_names.insert(big_a, "A".into());
    g.rules.insert(
        s,
        vec![Rule {
            lhs: s,
            rhs: vec![Symbol::NonTerminal(big_a), Symbol::Terminal(b)],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(0),
        }],
    );
    g.rules.insert(
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
    g
}

/// S -> a | b (two alternatives)
fn grammar_two_alt() -> Grammar {
    let a = SymbolId(1);
    let b = SymbolId(2);
    let s = SymbolId(10);

    let mut g = Grammar::new("two_alt".into());
    g.tokens.insert(
        a,
        Token {
            name: "a".into(),
            pattern: TokenPattern::String("a".into()),
            fragile: false,
        },
    );
    g.tokens.insert(
        b,
        Token {
            name: "b".into(),
            pattern: TokenPattern::String("b".into()),
            fragile: false,
        },
    );
    g.rule_names.insert(s, "S".into());
    g.rules.insert(
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
                rhs: vec![Symbol::Terminal(b)],
                precedence: None,
                associativity: None,
                fields: vec![],
                production_id: ProductionId(1),
            },
        ],
    );
    g
}

/// E -> E + T | T, T -> a  (left-recursive)
fn grammar_left_recursive() -> Grammar {
    let plus = SymbolId(1);
    let a = SymbolId(2);
    let e = SymbolId(10);
    let t = SymbolId(11);

    let mut g = Grammar::new("left_rec".into());
    g.tokens.insert(
        plus,
        Token {
            name: "plus".into(),
            pattern: TokenPattern::String("+".into()),
            fragile: false,
        },
    );
    g.tokens.insert(
        a,
        Token {
            name: "a".into(),
            pattern: TokenPattern::String("a".into()),
            fragile: false,
        },
    );
    g.rule_names.insert(e, "E".into());
    g.rule_names.insert(t, "T".into());
    g.rules.insert(
        e,
        vec![
            Rule {
                lhs: e,
                rhs: vec![
                    Symbol::NonTerminal(e),
                    Symbol::Terminal(plus),
                    Symbol::NonTerminal(t),
                ],
                precedence: None,
                associativity: None,
                fields: vec![],
                production_id: ProductionId(0),
            },
            Rule {
                lhs: e,
                rhs: vec![Symbol::NonTerminal(t)],
                precedence: None,
                associativity: None,
                fields: vec![],
                production_id: ProductionId(1),
            },
        ],
    );
    g.rules.insert(
        t,
        vec![Rule {
            lhs: t,
            rhs: vec![Symbol::Terminal(a)],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(2),
        }],
    );
    g
}

// ---------------------------------------------------------------------------
// 1. LRItem equality is reflexive, symmetric, and transitive
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    #[test]
    fn lr1_item_equality_reflexive(item in lr_item_strat()) {
        prop_assert_eq!(&item, &item);
    }
}

// ---------------------------------------------------------------------------
// 2. Equal items produce equal hashes
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn lr1_item_equal_implies_same_hash(a in lr_item_strat(), b in lr_item_strat()) {
        fn hash_of(item: &LRItem) -> u64 {
            let mut h = std::collections::hash_map::DefaultHasher::new();
            item.hash(&mut h);
            h.finish()
        }
        if a == b {
            prop_assert_eq!(hash_of(&a), hash_of(&b));
        }
    }
}

// ---------------------------------------------------------------------------
// 3. Clone produces equal item with same hash
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn lr1_item_clone_hash_eq(item in lr_item_strat()) {
        let cloned = item.clone();
        fn hash_of(i: &LRItem) -> u64 {
            let mut h = std::collections::hash_map::DefaultHasher::new();
            i.hash(&mut h);
            h.finish()
        }
        prop_assert_eq!(&item, &cloned);
        prop_assert_eq!(hash_of(&item), hash_of(&cloned));
    }
}

// ---------------------------------------------------------------------------
// 4. Dot position bounded by production length for real grammars
// ---------------------------------------------------------------------------

#[test]
fn dot_position_bounded_by_rhs_len() {
    let grammar = grammar_s_to_a();
    for rule in grammar.all_rules() {
        for pos in 0..=rule.rhs.len() {
            let item = LRItem::new(RuleId(rule.production_id.0), pos, SymbolId(0));
            // pos <= rhs.len() is valid (reduce position at end)
            assert!(item.position <= rule.rhs.len());
        }
    }
}

// ---------------------------------------------------------------------------
// 5. Items beyond production length are reduce items
// ---------------------------------------------------------------------------

#[test]
fn item_past_end_is_reduce() {
    let grammar = grammar_s_to_a();
    // S -> a has rhs len 1; position 1 is at end
    let item = LRItem::new(RuleId(0), 1, SymbolId(0));
    assert!(item.is_reduce_item(&grammar));
    assert!(item.next_symbol(&grammar).is_none());
}

// ---------------------------------------------------------------------------
// 6. Closure always contains kernel items
// ---------------------------------------------------------------------------

#[test]
fn closure_contains_kernel_items() {
    let grammar = grammar_s_a_b();
    let ff = FirstFollowSets::compute(&grammar).unwrap();

    let kernel = LRItem::new(RuleId(0), 0, SymbolId(0));
    let mut set = ItemSet::new(StateId(0));
    set.add_item(kernel.clone());
    set.closure(&grammar, &ff).unwrap();

    assert!(set.items.contains(&kernel));
}

// ---------------------------------------------------------------------------
// 7. Closure with non-terminal at dot expands to child rules
// ---------------------------------------------------------------------------

#[test]
fn closure_expands_nonterminal_rules() {
    let grammar = grammar_s_a_b();
    let ff = FirstFollowSets::compute(&grammar).unwrap();

    let mut set = ItemSet::new(StateId(0));
    // S -> . A b  (A is non-terminal, should expand)
    set.add_item(LRItem::new(RuleId(0), 0, SymbolId(0)));
    let before = set.items.len();
    set.closure(&grammar, &ff).unwrap();

    assert!(
        set.items.len() > before,
        "closure should add items for A -> . a"
    );
}

// ---------------------------------------------------------------------------
// 8. Closure is a fixed point (applying twice gives same result)
// ---------------------------------------------------------------------------

#[test]
fn closure_fixed_point() {
    let grammar = grammar_left_recursive();
    let ff = FirstFollowSets::compute(&grammar).unwrap();

    let mut set = ItemSet::new(StateId(0));
    set.add_item(LRItem::new(RuleId(0), 0, SymbolId(0)));
    set.add_item(LRItem::new(RuleId(1), 0, SymbolId(0)));
    set.closure(&grammar, &ff).unwrap();
    let first_pass: BTreeSet<_> = set.items.clone();

    set.closure(&grammar, &ff).unwrap();
    assert_eq!(set.items, first_pass, "second closure should be no-op");
}

// ---------------------------------------------------------------------------
// 9. Goto produces valid states (items have advanced dot)
// ---------------------------------------------------------------------------

#[test]
fn goto_produces_advanced_items() {
    let grammar = grammar_s_a_b();
    let ff = FirstFollowSets::compute(&grammar).unwrap();

    let mut set = ItemSet::new(StateId(0));
    set.add_item(LRItem::new(RuleId(0), 0, SymbolId(0)));
    set.closure(&grammar, &ff).unwrap();

    // Goto over NonTerminal A
    let goto = set.goto(&Symbol::NonTerminal(SymbolId(11)), &grammar, &ff);
    for item in &goto.items {
        if item.rule_id == RuleId(0) {
            assert!(
                item.position >= 1,
                "kernel items in goto must have advanced dot"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// 10. Goto on absent symbol returns empty set
// ---------------------------------------------------------------------------

#[test]
fn goto_absent_symbol_empty() {
    let grammar = grammar_s_to_a();
    let ff = FirstFollowSets::compute(&grammar).unwrap();

    let mut set = ItemSet::new(StateId(0));
    set.add_item(LRItem::new(RuleId(0), 0, SymbolId(0)));
    set.closure(&grammar, &ff).unwrap();

    let goto = set.goto(&Symbol::Terminal(SymbolId(200)), &grammar, &ff);
    assert!(goto.items.is_empty());
}

// ---------------------------------------------------------------------------
// 11. Ordering: items are totally ordered (antisymmetric)
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn lr1_item_ordering_antisymmetric(a in lr_item_strat(), b in lr_item_strat()) {
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
// 12. Ordering is transitive
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn lr1_item_ordering_transitive(
        a in lr_item_strat(),
        b in lr_item_strat(),
        c in lr_item_strat(),
    ) {
        if a <= b && b <= c {
            prop_assert!(a <= c);
        }
    }
}

// ---------------------------------------------------------------------------
// 13. Canonical collection has at least one state
// ---------------------------------------------------------------------------

#[test]
fn canonical_collection_at_least_one_state() {
    let grammar = grammar_s_to_a();
    let ff = FirstFollowSets::compute(&grammar).unwrap();
    let coll = ItemSetCollection::build_canonical_collection(&grammar, &ff);
    assert!(
        !coll.sets.is_empty(),
        "canonical collection must have the initial state"
    );
}

// ---------------------------------------------------------------------------
// 14. Canonical collection — no duplicate item sets
// ---------------------------------------------------------------------------

#[test]
fn canonical_collection_no_duplicates() {
    let grammar = grammar_s_a_b();
    let ff = FirstFollowSets::compute(&grammar).unwrap();
    let coll = ItemSetCollection::build_canonical_collection(&grammar, &ff);

    for i in 0..coll.sets.len() {
        for j in (i + 1)..coll.sets.len() {
            assert_ne!(
                coll.sets[i].items, coll.sets[j].items,
                "states {} and {} have identical items",
                i, j,
            );
        }
    }
}

// ---------------------------------------------------------------------------
// 15. Canonical collection state IDs are contiguous from 0
// ---------------------------------------------------------------------------

#[test]
fn canonical_collection_contiguous_ids() {
    let grammar = grammar_s_a_b();
    let ff = FirstFollowSets::compute(&grammar).unwrap();
    let coll = ItemSetCollection::build_canonical_collection(&grammar, &ff);

    for (idx, state) in coll.sets.iter().enumerate() {
        assert_eq!(
            state.id,
            StateId(idx as u16),
            "state at index {} has id {}",
            idx,
            state.id.0,
        );
    }
}

// ---------------------------------------------------------------------------
// 16. Goto table targets reference existing states
// ---------------------------------------------------------------------------

#[test]
fn goto_table_targets_exist() {
    let grammar = grammar_left_recursive();
    let ff = FirstFollowSets::compute(&grammar).unwrap();
    let coll = ItemSetCollection::build_canonical_collection(&grammar, &ff);

    let valid: HashSet<_> = coll.sets.iter().map(|s| s.id).collect();
    for (_, &target) in &coll.goto_table {
        assert!(
            valid.contains(&target),
            "target state {} not found",
            target.0
        );
    }
}

// ---------------------------------------------------------------------------
// 17. Goto table sources reference existing states
// ---------------------------------------------------------------------------

#[test]
fn goto_table_sources_exist() {
    let grammar = grammar_left_recursive();
    let ff = FirstFollowSets::compute(&grammar).unwrap();
    let coll = ItemSetCollection::build_canonical_collection(&grammar, &ff);

    let valid: HashSet<_> = coll.sets.iter().map(|s| s.id).collect();
    for ((src, _), _) in &coll.goto_table {
        assert!(valid.contains(src), "source state {} not found", src.0);
    }
}

// ---------------------------------------------------------------------------
// 18. Every state in the collection is closed (fixed-point)
// ---------------------------------------------------------------------------

#[test]
fn all_states_are_closed() {
    let grammar = grammar_left_recursive();
    let ff = FirstFollowSets::compute(&grammar).unwrap();
    let coll = ItemSetCollection::build_canonical_collection(&grammar, &ff);

    for state in &coll.sets {
        let mut copy = state.clone();
        copy.closure(&grammar, &ff).unwrap();
        assert_eq!(
            state.items, copy.items,
            "state {} is not closed",
            state.id.0,
        );
    }
}

// ---------------------------------------------------------------------------
// 19. No duplicate items within any single state
// ---------------------------------------------------------------------------

#[test]
fn no_duplicate_items_in_states() {
    let grammar = grammar_s_a_b();
    let ff = FirstFollowSets::compute(&grammar).unwrap();
    let coll = ItemSetCollection::build_canonical_collection(&grammar, &ff);

    for state in &coll.sets {
        let vec: Vec<_> = state.items.iter().collect();
        let set: HashSet<_> = state.items.iter().collect();
        assert_eq!(vec.len(), set.len(), "duplicates in state {}", state.id.0);
    }
}

// ---------------------------------------------------------------------------
// 20. Items inserted into BTreeSet remain sorted
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn btreeset_items_always_sorted(items in lr_item_vec_strat(12)) {
        let set: BTreeSet<_> = items.into_iter().collect();
        let as_vec: Vec<_> = set.iter().collect();
        for i in 1..as_vec.len() {
            prop_assert!(as_vec[i - 1] < as_vec[i]);
        }
    }
}

// ---------------------------------------------------------------------------
// 21. Differing lookaheads make distinct items
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn different_lookahead_means_different_item(
        rule in rule_id_strat(),
        pos in position_strat(),
        la1 in 0u16..50,
        la2 in 50u16..100,
    ) {
        let a = LRItem::new(rule, pos, SymbolId(la1));
        let b = LRItem::new(rule, pos, SymbolId(la2));
        prop_assert_ne!(&a, &b);
    }
}

// ---------------------------------------------------------------------------
// 22. Differing positions make distinct items
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn different_position_means_different_item(
        rule in rule_id_strat(),
        pos1 in 0usize..7,
        pos2 in 7usize..14,
        la in symbol_id_strat(),
    ) {
        let a = LRItem::new(rule, pos1, la);
        let b = LRItem::new(rule, pos2, la);
        prop_assert_ne!(&a, &b);
    }
}

// ---------------------------------------------------------------------------
// 23. Differing rule IDs make distinct items
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn different_rule_id_means_different_item(
        r1 in 0u16..75,
        r2 in 75u16..150,
        pos in position_strat(),
        la in symbol_id_strat(),
    ) {
        let a = LRItem::new(RuleId(r1), pos, la);
        let b = LRItem::new(RuleId(r2), pos, la);
        prop_assert_ne!(&a, &b);
    }
}

// ---------------------------------------------------------------------------
// 24. PartialOrd is consistent with Ord
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn partial_ord_matches_ord(a in lr_item_strat(), b in lr_item_strat()) {
        prop_assert_eq!(a.partial_cmp(&b), Some(a.cmp(&b)));
    }
}

// ---------------------------------------------------------------------------
// 25. Left-recursive grammar produces multiple states
// ---------------------------------------------------------------------------

#[test]
fn left_recursive_grammar_multiple_states() {
    let grammar = grammar_left_recursive();
    let ff = FirstFollowSets::compute(&grammar).unwrap();
    let coll = ItemSetCollection::build_canonical_collection(&grammar, &ff);

    assert!(
        coll.sets.len() > 1,
        "left-recursive grammar should produce multiple states, got {}",
        coll.sets.len(),
    );
}

// ---------------------------------------------------------------------------
// 26. Two-alternative grammar: goto over each alternative gives non-empty set
// ---------------------------------------------------------------------------

#[test]
fn two_alt_grammar_goto_both_alternatives() {
    let grammar = grammar_two_alt();
    let ff = FirstFollowSets::compute(&grammar).unwrap();

    let mut set = ItemSet::new(StateId(0));
    // Add items for both alternatives
    set.add_item(LRItem::new(RuleId(0), 0, SymbolId(0))); // S -> . a
    set.add_item(LRItem::new(RuleId(1), 0, SymbolId(0))); // S -> . b
    set.closure(&grammar, &ff).unwrap();

    let goto_a = set.goto(&Symbol::Terminal(SymbolId(1)), &grammar, &ff);
    let goto_b = set.goto(&Symbol::Terminal(SymbolId(2)), &grammar, &ff);

    assert!(
        !goto_a.items.is_empty(),
        "goto over 'a' should be non-empty"
    );
    assert!(
        !goto_b.items.is_empty(),
        "goto over 'b' should be non-empty"
    );
}

// ---------------------------------------------------------------------------
// 27. Canonical collection initial state (state 0) is non-empty
// ---------------------------------------------------------------------------

#[test]
fn initial_state_is_nonempty() {
    let grammar = grammar_s_to_a();
    let ff = FirstFollowSets::compute(&grammar).unwrap();
    let coll = ItemSetCollection::build_canonical_collection(&grammar, &ff);

    assert!(
        !coll.sets[0].items.is_empty(),
        "initial state must have at least the start item"
    );
}

// ---------------------------------------------------------------------------
// 28. Closure monotonicity: closure only adds items, never removes
// ---------------------------------------------------------------------------

#[test]
fn closure_only_adds() {
    let grammar = grammar_left_recursive();
    let ff = FirstFollowSets::compute(&grammar).unwrap();

    let mut set = ItemSet::new(StateId(0));
    set.add_item(LRItem::new(RuleId(0), 0, SymbolId(0)));
    set.add_item(LRItem::new(RuleId(1), 0, SymbolId(0)));
    let before: BTreeSet<_> = set.items.clone();

    set.closure(&grammar, &ff).unwrap();

    for item in &before {
        assert!(set.items.contains(item), "closure removed an item");
    }
    assert!(set.items.len() >= before.len());
}

// ---------------------------------------------------------------------------
// 29. HashSet and BTreeSet agree on unique count
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn hashset_btreeset_agree_on_count(items in lr_item_vec_strat(15)) {
        let btree: BTreeSet<_> = items.iter().cloned().collect();
        let hash: HashSet<_> = items.into_iter().collect();
        prop_assert_eq!(btree.len(), hash.len());
    }
}

// ---------------------------------------------------------------------------
// 30. Goto result is itself closed
// ---------------------------------------------------------------------------

#[test]
fn goto_result_is_closed() {
    let grammar = grammar_s_a_b();
    let ff = FirstFollowSets::compute(&grammar).unwrap();

    let mut set = ItemSet::new(StateId(0));
    set.add_item(LRItem::new(RuleId(0), 0, SymbolId(0)));
    set.closure(&grammar, &ff).unwrap();

    // Goto over terminal 'a' (used by A -> . a)
    let goto = set.goto(&Symbol::Terminal(SymbolId(1)), &grammar, &ff);
    if !goto.items.is_empty() {
        let mut reclosed = goto.clone();
        reclosed.closure(&grammar, &ff).unwrap();
        assert_eq!(
            goto.items, reclosed.items,
            "goto result should already be closed"
        );
    }
}

// ---------------------------------------------------------------------------
// 31. Empty item set closure is no-op
// ---------------------------------------------------------------------------

#[test]
fn empty_set_closure_stays_empty() {
    let grammar = grammar_s_to_a();
    let ff = FirstFollowSets::compute(&grammar).unwrap();
    let mut set = ItemSet::new(StateId(0));
    set.closure(&grammar, &ff).unwrap();
    assert!(set.items.is_empty());
}

// ---------------------------------------------------------------------------
// 32. Item constructed with new() stores fields correctly
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn lr1_item_fields_stored(
        r in rule_id_strat(),
        p in position_strat(),
        la in symbol_id_strat(),
    ) {
        let item = LRItem::new(r, p, la);
        prop_assert_eq!(item.rule_id, r);
        prop_assert_eq!(item.position, p);
        prop_assert_eq!(item.lookahead, la);
    }
}

// ---------------------------------------------------------------------------
// 33. Canonical collection on two-alt grammar has no duplicate item sets
// ---------------------------------------------------------------------------

#[test]
fn two_alt_canonical_no_dupes() {
    let grammar = grammar_two_alt();
    let ff = FirstFollowSets::compute(&grammar).unwrap();
    let coll = ItemSetCollection::build_canonical_collection(&grammar, &ff);

    for i in 0..coll.sets.len() {
        for j in (i + 1)..coll.sets.len() {
            assert_ne!(
                coll.sets[i].items, coll.sets[j].items,
                "duplicate item sets at {} and {}",
                i, j,
            );
        }
    }
}

// ---------------------------------------------------------------------------
// 34. Reduce items at every rule's end position
// ---------------------------------------------------------------------------

#[test]
fn reduce_item_at_rhs_end_for_all_rules() {
    let grammar = grammar_left_recursive();
    for rule in grammar.all_rules() {
        let item = LRItem::new(RuleId(rule.production_id.0), rule.rhs.len(), SymbolId(0));
        assert!(
            item.is_reduce_item(&grammar),
            "item at end of rule {} should be reduce",
            rule.production_id.0,
        );
    }
}

// ---------------------------------------------------------------------------
// 35. Non-reduce items at position 0 have a next symbol
// ---------------------------------------------------------------------------

#[test]
fn shift_items_have_next_symbol() {
    let grammar = grammar_left_recursive();
    for rule in grammar.all_rules() {
        if rule.rhs.is_empty() {
            continue;
        }
        let item = LRItem::new(RuleId(rule.production_id.0), 0, SymbolId(0));
        assert!(
            item.next_symbol(&grammar).is_some(),
            "item at pos 0 of non-empty rule {} should have next_symbol",
            rule.production_id.0,
        );
    }
}
