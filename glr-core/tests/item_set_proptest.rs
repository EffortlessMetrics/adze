#![allow(clippy::needless_range_loop)]
//! Property-based tests for LR(1) item set data structures.
//!
//! Run with: `cargo test -p adze-glr-core --test item_set_proptest`

use adze_glr_core::{FirstFollowSets, ItemSet, ItemSetCollection, LRItem};
use adze_ir::*;
use proptest::prelude::*;
use std::collections::{BTreeSet, HashSet};
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

// ---------------------------------------------------------------------------
// Grammar helpers
// ---------------------------------------------------------------------------

/// Build a minimal grammar: S -> a
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

/// Build a left-recursive grammar: S -> S a | a
fn recursive_grammar() -> Grammar {
    let a = SymbolId(1);
    let s = SymbolId(10);

    let mut grammar = Grammar::new("recursive".into());
    grammar.tokens.insert(
        a,
        Token {
            name: "a".into(),
            pattern: TokenPattern::String("a".into()),
            fragile: false,
        },
    );
    grammar.rule_names.insert(s, "S".into());
    grammar.rules.insert(
        s,
        vec![
            Rule {
                lhs: s,
                rhs: vec![Symbol::NonTerminal(s), Symbol::Terminal(a)],
                precedence: None,
                associativity: None,
                fields: vec![],
                production_id: ProductionId(0),
            },
            Rule {
                lhs: s,
                rhs: vec![Symbol::Terminal(a)],
                precedence: None,
                associativity: None,
                fields: vec![],
                production_id: ProductionId(1),
            },
        ],
    );
    grammar
}

/// Build a grammar with Symbol::Epsilon in the RHS (for item-level tests).
/// A -> ε is represented as `rhs: vec![Symbol::Epsilon]`.
fn epsilon_symbol_grammar() -> Grammar {
    let s = SymbolId(10);
    let big_a = SymbolId(11);

    let mut grammar = Grammar::new("epsilon_sym".into());
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
            rhs: vec![Symbol::Epsilon],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(1),
        }],
    );
    grammar
}

/// Build a grammar with an empty production: S -> A, A -> (empty rhs).
/// Empty rhs is the canonical way to represent nullable rules for collection building.
fn epsilon_grammar() -> Grammar {
    let s = SymbolId(10);
    let big_a = SymbolId(11);

    let mut grammar = Grammar::new("epsilon".into());
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
            rhs: vec![],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(1),
        }],
    );
    grammar
}

/// Build a grammar with both empty and terminal alternatives: S -> A, A -> a | (empty)
fn epsilon_alt_grammar() -> Grammar {
    let a = SymbolId(1);
    let s = SymbolId(10);
    let big_a = SymbolId(11);

    let mut grammar = Grammar::new("epsilon_alt".into());
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
        vec![
            Rule {
                lhs: big_a,
                rhs: vec![Symbol::Terminal(a)],
                precedence: None,
                associativity: None,
                fields: vec![],
                production_id: ProductionId(1),
            },
            Rule {
                lhs: big_a,
                rhs: vec![],
                precedence: None,
                associativity: None,
                fields: vec![],
                production_id: ProductionId(2),
            },
        ],
    );
    grammar
}

// ===========================================================================
// 1. LRItem construction preserves fields (proptest)
// ===========================================================================

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

// ===========================================================================
// 2. LRItem hash consistent with Eq (proptest)
// ===========================================================================

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

// ===========================================================================
// 3. LRItem ordering is antisymmetric (proptest)
// ===========================================================================

proptest! {
    #[test]
    fn item_ord_antisymmetric(a in lr_item_strategy(), b in lr_item_strategy()) {
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

// ===========================================================================
// 4. ItemSet add_item is idempotent (proptest)
// ===========================================================================

proptest! {
    #[test]
    fn item_set_add_idempotent(item in lr_item_strategy(), id in state_id_strategy()) {
        let mut set = ItemSet::new(id);
        set.add_item(item.clone());
        set.add_item(item.clone());
        prop_assert_eq!(set.items.len(), 1);
    }
}

// ===========================================================================
// 5. ItemSet preserves all distinct items (proptest)
// ===========================================================================

proptest! {
    #[test]
    fn item_set_preserves_distinct(items in lr_item_vec_strategy(15)) {
        let mut set = ItemSet::new(StateId(0));
        for item in &items {
            set.add_item(item.clone());
        }
        let expected: BTreeSet<_> = items.into_iter().collect();
        prop_assert_eq!(set.items.len(), expected.len());
        prop_assert_eq!(&set.items, &expected);
    }
}

// ===========================================================================
// 6. ItemSet items are always sorted (proptest)
// ===========================================================================

proptest! {
    #[test]
    fn item_set_items_always_sorted(items in lr_item_vec_strategy(15)) {
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

// ===========================================================================
// ITEM SET COLLECTION — AT LEAST 1 STATE
// ===========================================================================

// 7. Simple grammar produces at least one state
#[test]
fn collection_has_at_least_one_state_simple() {
    let grammar = simple_grammar(SymbolId(1), SymbolId(10));
    let ff = FirstFollowSets::compute(&grammar).unwrap();
    let collection = ItemSetCollection::build_canonical_collection(&grammar, &ff);
    assert!(
        !collection.sets.is_empty(),
        "simple grammar must produce at least one state"
    );
}

// 8. Two-rule grammar produces at least one state
#[test]
fn collection_has_at_least_one_state_two_rule() {
    let grammar = two_rule_grammar();
    let ff = FirstFollowSets::compute(&grammar).unwrap();
    let collection = ItemSetCollection::build_canonical_collection(&grammar, &ff);
    assert!(!collection.sets.is_empty());
}

// 9. Indirect grammar produces at least one state
#[test]
fn collection_has_at_least_one_state_indirect() {
    let grammar = indirect_grammar();
    let ff = FirstFollowSets::compute(&grammar).unwrap();
    let collection = ItemSetCollection::build_canonical_collection(&grammar, &ff);
    assert!(!collection.sets.is_empty());
}

// ===========================================================================
// ITEM SET COUNT IS DETERMINISTIC
// ===========================================================================

// 10. Running build_canonical_collection twice yields the same state count
#[test]
fn collection_count_deterministic_simple() {
    let grammar = simple_grammar(SymbolId(1), SymbolId(10));
    let ff = FirstFollowSets::compute(&grammar).unwrap();
    let c1 = ItemSetCollection::build_canonical_collection(&grammar, &ff);
    let c2 = ItemSetCollection::build_canonical_collection(&grammar, &ff);
    assert_eq!(c1.sets.len(), c2.sets.len());
}

// 11. Deterministic count for two-rule grammar
#[test]
fn collection_count_deterministic_two_rule() {
    let grammar = two_rule_grammar();
    let ff = FirstFollowSets::compute(&grammar).unwrap();
    let c1 = ItemSetCollection::build_canonical_collection(&grammar, &ff);
    let c2 = ItemSetCollection::build_canonical_collection(&grammar, &ff);
    assert_eq!(c1.sets.len(), c2.sets.len());
    assert_eq!(c1.goto_table.len(), c2.goto_table.len());
}

// 12. Deterministic count for indirect grammar
#[test]
fn collection_count_deterministic_indirect() {
    let grammar = indirect_grammar();
    let ff = FirstFollowSets::compute(&grammar).unwrap();
    let c1 = ItemSetCollection::build_canonical_collection(&grammar, &ff);
    let c2 = ItemSetCollection::build_canonical_collection(&grammar, &ff);
    assert_eq!(c1.sets.len(), c2.sets.len());
    // Item sets themselves must match
    for i in 0..c1.sets.len() {
        assert_eq!(c1.sets[i].items, c2.sets[i].items);
    }
}

// ===========================================================================
// ITEM SET WITH SIMPLE GRAMMAR
// ===========================================================================

// 13. Simple grammar: unique state IDs
#[test]
fn simple_grammar_unique_state_ids() {
    let grammar = simple_grammar(SymbolId(1), SymbolId(10));
    let ff = FirstFollowSets::compute(&grammar).unwrap();
    let collection = ItemSetCollection::build_canonical_collection(&grammar, &ff);

    let ids: Vec<_> = collection.sets.iter().map(|s| s.id).collect();
    let unique: HashSet<_> = ids.iter().collect();
    assert_eq!(ids.len(), unique.len());
}

// 14. Simple grammar: has at least one reduce item somewhere
#[test]
fn simple_grammar_has_reduce_item() {
    let grammar = simple_grammar(SymbolId(1), SymbolId(10));
    let ff = FirstFollowSets::compute(&grammar).unwrap();
    let collection = ItemSetCollection::build_canonical_collection(&grammar, &ff);

    let has_reduce = collection
        .sets
        .iter()
        .any(|s| s.items.iter().any(|it| it.is_reduce_item(&grammar)));
    assert!(has_reduce, "some state must contain a reduce item");
}

// 15. Simple grammar: no duplicate item sets
#[test]
fn simple_grammar_no_duplicate_sets() {
    let grammar = simple_grammar(SymbolId(1), SymbolId(10));
    let ff = FirstFollowSets::compute(&grammar).unwrap();
    let collection = ItemSetCollection::build_canonical_collection(&grammar, &ff);

    for i in 0..collection.sets.len() {
        for j in (i + 1)..collection.sets.len() {
            assert_ne!(
                collection.sets[i].items, collection.sets[j].items,
                "States {} and {} have identical item sets",
                i, j,
            );
        }
    }
}

// ===========================================================================
// ITEM SET WITH RECURSIVE GRAMMAR
// ===========================================================================

// 16. Left-recursive grammar builds without panic
#[test]
fn recursive_grammar_builds_collection() {
    let grammar = recursive_grammar();
    let ff = FirstFollowSets::compute(&grammar).unwrap();
    let collection = ItemSetCollection::build_canonical_collection(&grammar, &ff);
    assert!(
        !collection.sets.is_empty(),
        "recursive grammar must produce states"
    );
}

// 17. Recursive grammar closure expands self-referencing non-terminal
#[test]
fn recursive_grammar_closure_expands() {
    let grammar = recursive_grammar();
    let ff = FirstFollowSets::compute(&grammar).unwrap();

    let mut set = ItemSet::new(StateId(0));
    // S -> . S a (rule 0)
    set.add_item(LRItem::new(RuleId(0), 0, SymbolId(0)));
    let before = set.items.len();
    set.closure(&grammar, &ff).unwrap();
    // Closure must add S -> . a via the non-terminal S on the RHS
    assert!(
        set.items.len() > before,
        "closure of recursive rule must add items"
    );
}

// 18. Recursive grammar produces multiple states
#[test]
fn recursive_grammar_multiple_states() {
    let grammar = recursive_grammar();
    let ff = FirstFollowSets::compute(&grammar).unwrap();
    let collection = ItemSetCollection::build_canonical_collection(&grammar, &ff);
    assert!(
        collection.sets.len() > 1,
        "recursive grammar needs at least shift + reduce states"
    );
}

// 19. Recursive grammar goto targets are valid
#[test]
fn recursive_grammar_goto_targets_valid() {
    let grammar = recursive_grammar();
    let ff = FirstFollowSets::compute(&grammar).unwrap();
    let collection = ItemSetCollection::build_canonical_collection(&grammar, &ff);

    let valid_ids: HashSet<_> = collection.sets.iter().map(|s| s.id).collect();
    for (_, &target) in &collection.goto_table {
        assert!(valid_ids.contains(&target));
    }
}

// ===========================================================================
// ITEM SET CLOSURES ARE COMPLETE
// ===========================================================================

// 20. Closure on terminal-at-dot adds no items
#[test]
fn closure_terminal_no_expansion() {
    let grammar = simple_grammar(SymbolId(1), SymbolId(10));
    let ff = FirstFollowSets::compute(&grammar).unwrap();

    let mut set = ItemSet::new(StateId(0));
    set.add_item(LRItem::new(RuleId(0), 0, SymbolId(0)));
    let before = set.items.len();
    set.closure(&grammar, &ff).unwrap();
    assert_eq!(set.items.len(), before);
}

// 21. Closure on non-terminal-at-dot expands
#[test]
fn closure_nonterminal_expands() {
    let grammar = indirect_grammar();
    let ff = FirstFollowSets::compute(&grammar).unwrap();

    let mut set = ItemSet::new(StateId(0));
    set.add_item(LRItem::new(RuleId(0), 0, SymbolId(0)));
    set.closure(&grammar, &ff).unwrap();
    assert!(set.items.len() > 1);
}

// 22. Closure is idempotent
#[test]
fn closure_idempotent() {
    let grammar = indirect_grammar();
    let ff = FirstFollowSets::compute(&grammar).unwrap();

    let mut set = ItemSet::new(StateId(0));
    set.add_item(LRItem::new(RuleId(0), 0, SymbolId(0)));
    set.closure(&grammar, &ff).unwrap();
    let snapshot: BTreeSet<_> = set.items.clone();
    set.closure(&grammar, &ff).unwrap();
    assert_eq!(set.items, snapshot);
}

// 23. Closure is monotone (only adds, never removes)
#[test]
fn closure_monotone() {
    let grammar = indirect_grammar();
    let ff = FirstFollowSets::compute(&grammar).unwrap();

    let mut set = ItemSet::new(StateId(0));
    set.add_item(LRItem::new(RuleId(0), 0, SymbolId(0)));
    let before: BTreeSet<_> = set.items.clone();
    set.closure(&grammar, &ff).unwrap();
    for item in &before {
        assert!(set.items.contains(item));
    }
}

// 24. Every state in a canonical collection is already closed
#[test]
fn all_collection_states_are_closed() {
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

// ===========================================================================
// ITEM SET GOTO TRANSITIONS
// ===========================================================================

// 25. Goto with unmatched symbol yields empty set
#[test]
fn goto_unmatched_symbol_empty() {
    let grammar = simple_grammar(SymbolId(1), SymbolId(10));
    let ff = FirstFollowSets::compute(&grammar).unwrap();

    let mut set = ItemSet::new(StateId(0));
    set.add_item(LRItem::new(RuleId(0), 0, SymbolId(0)));
    set.closure(&grammar, &ff).unwrap();

    let goto = set.goto(&Symbol::Terminal(SymbolId(99)), &grammar, &ff);
    assert!(goto.items.is_empty());
}

// 26. Goto advances dot position
#[test]
fn goto_advances_dot() {
    let grammar = simple_grammar(SymbolId(1), SymbolId(10));
    let ff = FirstFollowSets::compute(&grammar).unwrap();

    let mut set = ItemSet::new(StateId(0));
    set.add_item(LRItem::new(RuleId(0), 0, SymbolId(0)));
    set.closure(&grammar, &ff).unwrap();

    let goto = set.goto(&Symbol::Terminal(SymbolId(1)), &grammar, &ff);
    assert!(!goto.items.is_empty());
    for item in &goto.items {
        if item.rule_id == RuleId(0) {
            assert!(item.position >= 1);
        }
    }
}

// 27. Goto on empty set yields empty
#[test]
fn goto_empty_set_yields_empty() {
    let grammar = simple_grammar(SymbolId(1), SymbolId(10));
    let ff = FirstFollowSets::compute(&grammar).unwrap();

    let set = ItemSet::new(StateId(0));
    let goto = set.goto(&Symbol::Terminal(SymbolId(1)), &grammar, &ff);
    assert!(goto.items.is_empty());
}

// 28. Goto kernel items have position > 0
#[test]
fn goto_kernel_items_positive_position() {
    let grammar = indirect_grammar();
    let ff = FirstFollowSets::compute(&grammar).unwrap();

    let mut set = ItemSet::new(StateId(0));
    set.add_item(LRItem::new(RuleId(0), 0, SymbolId(0)));
    set.closure(&grammar, &ff).unwrap();

    let goto = set.goto(&Symbol::NonTerminal(SymbolId(11)), &grammar, &ff);
    if !goto.items.is_empty() {
        let has_kernel = goto.items.iter().any(|it| it.position > 0);
        assert!(has_kernel);
    }
}

// 29. Goto table sources and targets are all valid state IDs
#[test]
fn goto_table_refs_valid() {
    let grammar = two_rule_grammar();
    let ff = FirstFollowSets::compute(&grammar).unwrap();
    let collection = ItemSetCollection::build_canonical_collection(&grammar, &ff);

    let valid_ids: HashSet<_> = collection.sets.iter().map(|s| s.id).collect();
    for ((source, _), &target) in &collection.goto_table {
        assert!(valid_ids.contains(source));
        assert!(valid_ids.contains(&target));
    }
}

// ===========================================================================
// ITEM SET WITH EPSILON PRODUCTIONS
// ===========================================================================

// 30. Epsilon grammar builds without panic
#[test]
fn epsilon_grammar_builds_collection() {
    let grammar = epsilon_grammar();
    let ff = FirstFollowSets::compute(&grammar).unwrap();
    let collection = ItemSetCollection::build_canonical_collection(&grammar, &ff);
    assert!(!collection.sets.is_empty());
}

// 31. Epsilon item at position 0 is always a reduce item (Symbol::Epsilon variant)
#[test]
fn epsilon_item_is_reduce_and_has_next_symbol() {
    let grammar = epsilon_symbol_grammar();
    // A -> ε is production 1
    let item = LRItem::new(RuleId(1), 0, SymbolId(0));
    assert!(
        item.is_reduce_item(&grammar),
        "epsilon rule at position 0 must be a reduce item"
    );
    assert_eq!(item.next_symbol(&grammar), Some(&Symbol::Epsilon));
}

// 32. Empty-rhs item at position 0 is a reduce item
#[test]
fn empty_rhs_item_is_reduce() {
    let grammar = epsilon_grammar();
    // A -> (empty) is production 1
    let item = LRItem::new(RuleId(1), 0, SymbolId(0));
    assert!(
        item.is_reduce_item(&grammar),
        "empty-rhs rule at position 0 must be a reduce item"
    );
    assert_eq!(item.next_symbol(&grammar), None);
}

// 33. Closure through epsilon-producing non-terminal expands
#[test]
fn epsilon_closure_expands() {
    let grammar = epsilon_alt_grammar();
    let ff = FirstFollowSets::compute(&grammar).unwrap();

    let mut set = ItemSet::new(StateId(0));
    // S -> . A (rule 0)
    set.add_item(LRItem::new(RuleId(0), 0, SymbolId(0)));
    let before = set.items.len();
    set.closure(&grammar, &ff).unwrap();
    assert!(
        set.items.len() > before,
        "closure through nullable non-terminal must expand"
    );
}

// 34. Epsilon-alt grammar produces valid canonical collection
#[test]
fn epsilon_alt_grammar_states_are_closed() {
    let grammar = epsilon_alt_grammar();
    let ff = FirstFollowSets::compute(&grammar).unwrap();
    let collection = ItemSetCollection::build_canonical_collection(&grammar, &ff);

    for state in &collection.sets {
        let mut reclosed = state.clone();
        reclosed.closure(&grammar, &ff).unwrap();
        assert_eq!(
            state.items, reclosed.items,
            "State {} is not closed in epsilon-alt grammar",
            state.id.0,
        );
    }
}

// 35. Epsilon-alt grammar deterministic count
#[test]
fn epsilon_alt_grammar_deterministic_count() {
    let grammar = epsilon_alt_grammar();
    let ff = FirstFollowSets::compute(&grammar).unwrap();
    let c1 = ItemSetCollection::build_canonical_collection(&grammar, &ff);
    let c2 = ItemSetCollection::build_canonical_collection(&grammar, &ff);
    assert_eq!(c1.sets.len(), c2.sets.len());
}
