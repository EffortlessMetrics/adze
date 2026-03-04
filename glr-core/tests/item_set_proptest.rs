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

// ===========================================================================
// 36-65: ADDITIONAL TESTS
// ===========================================================================

// ---------------------------------------------------------------------------
// Additional grammar helpers
// ---------------------------------------------------------------------------

/// Build a grammar with multiple non-terminals: S -> A B, A -> a, B -> b
fn multi_nt_grammar() -> Grammar {
    let a = SymbolId(1);
    let b = SymbolId(2);
    let s = SymbolId(10);
    let big_a = SymbolId(11);
    let big_b = SymbolId(12);

    let mut grammar = Grammar::new("multi_nt".into());
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
    grammar.rule_names.insert(big_a, "A".into());
    grammar.rule_names.insert(big_b, "B".into());
    grammar.rules.insert(
        s,
        vec![Rule {
            lhs: s,
            rhs: vec![Symbol::NonTerminal(big_a), Symbol::NonTerminal(big_b)],
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
    grammar.rules.insert(
        big_b,
        vec![Rule {
            lhs: big_b,
            rhs: vec![Symbol::Terminal(b)],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(2),
        }],
    );
    grammar
}

/// Build an ambiguous grammar: S -> a | a (two identical alternatives)
fn ambiguous_grammar() -> Grammar {
    let a = SymbolId(1);
    let s = SymbolId(10);

    let mut grammar = Grammar::new("ambiguous".into());
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
                rhs: vec![Symbol::Terminal(a)],
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

/// Build a chain grammar: S -> A, A -> B, B -> a
fn chain_grammar() -> Grammar {
    let a = SymbolId(1);
    let s = SymbolId(10);
    let big_a = SymbolId(11);
    let big_b = SymbolId(12);

    let mut grammar = Grammar::new("chain".into());
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
    grammar.rule_names.insert(big_b, "B".into());
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
            rhs: vec![Symbol::NonTerminal(big_b)],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(1),
        }],
    );
    grammar.rules.insert(
        big_b,
        vec![Rule {
            lhs: big_b,
            rhs: vec![Symbol::Terminal(a)],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(2),
        }],
    );
    grammar
}

/// Build a right-recursive grammar: S -> a S | a
fn right_recursive_grammar() -> Grammar {
    let a = SymbolId(1);
    let s = SymbolId(10);

    let mut grammar = Grammar::new("right_rec".into());
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
                rhs: vec![Symbol::Terminal(a), Symbol::NonTerminal(s)],
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

// ===========================================================================
// ITEM CREATION — additional property tests
// ===========================================================================

// 36. LRItem clone produces equal item
proptest! {
    #[test]
    fn item_clone_is_equal(item in lr_item_strategy()) {
        let cloned = item.clone();
        prop_assert_eq!(item, cloned);
    }
}

// 37. LRItem ordering is transitive
proptest! {
    #[test]
    fn item_ord_transitive(
        a in lr_item_strategy(),
        b in lr_item_strategy(),
        c in lr_item_strategy(),
    ) {
        use std::cmp::Ordering;
        if a.cmp(&b) == Ordering::Less && b.cmp(&c) == Ordering::Less {
            prop_assert_eq!(a.cmp(&c), Ordering::Less);
        }
    }
}

// 38. LRItem with position 0 is never a reduce item for non-empty, non-epsilon rules
#[test]
fn item_position_zero_not_reduce_for_nonempty_rule() {
    let grammar = simple_grammar(SymbolId(1), SymbolId(10));
    let item = LRItem::new(RuleId(0), 0, SymbolId(0));
    assert!(
        !item.is_reduce_item(&grammar),
        "position 0 in non-empty rule must not be a reduce item"
    );
}

// 39. LRItem at end position is a reduce item
#[test]
fn item_at_end_position_is_reduce() {
    let grammar = simple_grammar(SymbolId(1), SymbolId(10));
    // S -> a is rule 0, rhs len=1, so position=1 is reduce
    let item = LRItem::new(RuleId(0), 1, SymbolId(0));
    assert!(item.is_reduce_item(&grammar));
}

// 40. LRItem next_symbol returns None for reduce items
#[test]
fn item_next_symbol_none_at_end() {
    let grammar = simple_grammar(SymbolId(1), SymbolId(10));
    let item = LRItem::new(RuleId(0), 1, SymbolId(0));
    assert!(item.next_symbol(&grammar).is_none());
}

// 41. LRItem next_symbol returns correct terminal
#[test]
fn item_next_symbol_returns_terminal() {
    let grammar = simple_grammar(SymbolId(1), SymbolId(10));
    let item = LRItem::new(RuleId(0), 0, SymbolId(0));
    assert_eq!(
        item.next_symbol(&grammar),
        Some(&Symbol::Terminal(SymbolId(1)))
    );
}

// 42. LRItem next_symbol returns non-terminal
#[test]
fn item_next_symbol_returns_nonterminal() {
    let grammar = indirect_grammar();
    // S -> . A (rule 0, position 0)
    let item = LRItem::new(RuleId(0), 0, SymbolId(0));
    assert_eq!(
        item.next_symbol(&grammar),
        Some(&Symbol::NonTerminal(SymbolId(11)))
    );
}

// 43. LRItem with unknown rule_id is not a reduce item
#[test]
fn item_unknown_rule_not_reduce() {
    let grammar = simple_grammar(SymbolId(1), SymbolId(10));
    let item = LRItem::new(RuleId(999), 0, SymbolId(0));
    assert!(!item.is_reduce_item(&grammar));
}

// 44. LRItem with unknown rule_id returns None for next_symbol
#[test]
fn item_unknown_rule_next_symbol_none() {
    let grammar = simple_grammar(SymbolId(1), SymbolId(10));
    let item = LRItem::new(RuleId(999), 0, SymbolId(0));
    assert!(item.next_symbol(&grammar).is_none());
}

// ===========================================================================
// ITEM SET CREATION — additional tests
// ===========================================================================

// 45. New ItemSet is empty
#[test]
fn item_set_new_is_empty() {
    let set = ItemSet::new(StateId(42));
    assert!(set.items.is_empty());
    assert_eq!(set.id, StateId(42));
}

// 46. ItemSet equality is based on items, not id
#[test]
fn item_set_equality_based_on_items() {
    let mut set_a = ItemSet::new(StateId(0));
    let mut set_b = ItemSet::new(StateId(1));
    let item = LRItem::new(RuleId(0), 0, SymbolId(0));
    set_a.add_item(item.clone());
    set_b.add_item(item);
    // ItemSet derives PartialEq, Eq — both id and items are compared
    // Different IDs means they are not equal even with same items
    assert_ne!(set_a, set_b);
    // But the items field itself should be equal
    assert_eq!(set_a.items, set_b.items);
}

// 47. ItemSet with same id and items is equal
#[test]
fn item_set_equal_same_id_and_items() {
    let mut set_a = ItemSet::new(StateId(5));
    let mut set_b = ItemSet::new(StateId(5));
    let item = LRItem::new(RuleId(1), 2, SymbolId(3));
    set_a.add_item(item.clone());
    set_b.add_item(item);
    assert_eq!(set_a, set_b);
}

// ===========================================================================
// CLOSURE — additional tests
// ===========================================================================

// 48. Closure through chain grammar expands transitively
#[test]
fn closure_chain_grammar_transitive() {
    let grammar = chain_grammar();
    let ff = FirstFollowSets::compute(&grammar).unwrap();

    let mut set = ItemSet::new(StateId(0));
    // S -> . A (rule 0)
    set.add_item(LRItem::new(RuleId(0), 0, SymbolId(0)));
    set.closure(&grammar, &ff).unwrap();
    // Should expand through A -> . B and B -> . a
    assert!(
        set.items.len() >= 3,
        "chain closure must add items transitively, got {}",
        set.items.len()
    );
}

// 49. Closure of multi-nt grammar adds items for both non-terminals
#[test]
fn closure_multi_nt_expands_first_nt() {
    let grammar = multi_nt_grammar();
    let ff = FirstFollowSets::compute(&grammar).unwrap();

    let mut set = ItemSet::new(StateId(0));
    // S -> . A B (rule 0)
    set.add_item(LRItem::new(RuleId(0), 0, SymbolId(0)));
    set.closure(&grammar, &ff).unwrap();
    // At minimum, closure adds A -> . a
    assert!(set.items.len() > 1);
}

// 50. Closure on a reduce item (dot at end) adds nothing
#[test]
fn closure_reduce_item_no_expansion() {
    let grammar = simple_grammar(SymbolId(1), SymbolId(10));
    let ff = FirstFollowSets::compute(&grammar).unwrap();

    let mut set = ItemSet::new(StateId(0));
    set.add_item(LRItem::new(RuleId(0), 1, SymbolId(0)));
    let before = set.items.len();
    set.closure(&grammar, &ff).unwrap();
    assert_eq!(set.items.len(), before);
}

// 51. Closure of right-recursive grammar terminates
#[test]
fn closure_right_recursive_terminates() {
    let grammar = right_recursive_grammar();
    let ff = FirstFollowSets::compute(&grammar).unwrap();

    let mut set = ItemSet::new(StateId(0));
    // S -> a . S (rule 0, position 1 — dot before non-terminal S)
    set.add_item(LRItem::new(RuleId(0), 1, SymbolId(0)));
    set.closure(&grammar, &ff).unwrap();
    // Closure must add S -> . a S and S -> . a
    assert!(set.items.len() > 1);
}

// ===========================================================================
// GOTO — additional tests
// ===========================================================================

// 52. Goto preserves lookahead from source items
#[test]
fn goto_preserves_lookahead() {
    let grammar = simple_grammar(SymbolId(1), SymbolId(10));
    let ff = FirstFollowSets::compute(&grammar).unwrap();

    let mut set = ItemSet::new(StateId(0));
    let la = SymbolId(7);
    set.add_item(LRItem::new(RuleId(0), 0, la));
    set.closure(&grammar, &ff).unwrap();

    let goto = set.goto(&Symbol::Terminal(SymbolId(1)), &grammar, &ff);
    // The advanced item should preserve the original lookahead
    let has_la = goto.items.iter().any(|it| it.lookahead == la);
    assert!(has_la, "goto must preserve lookahead");
}

// 53. Goto with non-terminal on multi-nt grammar
#[test]
fn goto_nonterminal_multi_nt() {
    let grammar = multi_nt_grammar();
    let ff = FirstFollowSets::compute(&grammar).unwrap();

    let mut set = ItemSet::new(StateId(0));
    set.add_item(LRItem::new(RuleId(0), 0, SymbolId(0)));
    set.closure(&grammar, &ff).unwrap();

    // GOTO on A should produce S -> A . B which then closes over B -> . b
    let goto = set.goto(&Symbol::NonTerminal(SymbolId(11)), &grammar, &ff);
    assert!(
        !goto.items.is_empty(),
        "goto on A should produce non-empty set"
    );
    let has_pos1 = goto
        .items
        .iter()
        .any(|it| it.rule_id == RuleId(0) && it.position == 1);
    assert!(has_pos1, "goto should advance S -> A . B");
}

// 54. Goto result is already closed
#[test]
fn goto_result_is_closed() {
    let grammar = indirect_grammar();
    let ff = FirstFollowSets::compute(&grammar).unwrap();

    let mut set = ItemSet::new(StateId(0));
    set.add_item(LRItem::new(RuleId(0), 0, SymbolId(0)));
    set.closure(&grammar, &ff).unwrap();

    let goto = set.goto(&Symbol::NonTerminal(SymbolId(11)), &grammar, &ff);
    let mut reclosed = goto.clone();
    reclosed.closure(&grammar, &ff).unwrap();
    assert_eq!(
        goto.items, reclosed.items,
        "goto result must already be closed"
    );
}

// 55. Goto on different symbols yields different sets
#[test]
fn goto_different_symbols_different_sets() {
    let grammar = two_rule_grammar();
    let ff = FirstFollowSets::compute(&grammar).unwrap();
    let collection = ItemSetCollection::build_canonical_collection(&grammar, &ff);

    // Find initial state
    let initial = &collection.sets[0];
    let goto_a = initial.goto(&Symbol::Terminal(SymbolId(1)), &grammar, &ff);
    let goto_b = initial.goto(&Symbol::Terminal(SymbolId(2)), &grammar, &ff);
    // If both are non-empty, they should differ
    if !goto_a.items.is_empty() && !goto_b.items.is_empty() {
        assert_ne!(goto_a.items, goto_b.items);
    }
}

// ===========================================================================
// CANONICAL COLLECTION — additional tests
// ===========================================================================

// 56. Multi-nt grammar produces valid collection
#[test]
fn multi_nt_grammar_collection_valid() {
    let grammar = multi_nt_grammar();
    let ff = FirstFollowSets::compute(&grammar).unwrap();
    let collection = ItemSetCollection::build_canonical_collection(&grammar, &ff);
    assert!(!collection.sets.is_empty());
    // Every goto target must be a valid state
    let valid_ids: HashSet<_> = collection.sets.iter().map(|s| s.id).collect();
    for (_, &target) in &collection.goto_table {
        assert!(valid_ids.contains(&target));
    }
}

// 57. Chain grammar produces more states than simple grammar
#[test]
fn chain_grammar_more_states_than_simple() {
    let simple = simple_grammar(SymbolId(1), SymbolId(10));
    let chain = chain_grammar();
    let ff_simple = FirstFollowSets::compute(&simple).unwrap();
    let ff_chain = FirstFollowSets::compute(&chain).unwrap();
    let c_simple = ItemSetCollection::build_canonical_collection(&simple, &ff_simple);
    let c_chain = ItemSetCollection::build_canonical_collection(&chain, &ff_chain);
    assert!(
        c_chain.sets.len() >= c_simple.sets.len(),
        "chain grammar should have at least as many states as simple grammar"
    );
}

// 58. Right-recursive grammar builds without panic
#[test]
fn right_recursive_grammar_builds_collection() {
    let grammar = right_recursive_grammar();
    let ff = FirstFollowSets::compute(&grammar).unwrap();
    let collection = ItemSetCollection::build_canonical_collection(&grammar, &ff);
    assert!(!collection.sets.is_empty());
    assert!(collection.sets.len() > 1);
}

// 59. Ambiguous grammar builds without panic
#[test]
fn ambiguous_grammar_builds_collection() {
    let grammar = ambiguous_grammar();
    let ff = FirstFollowSets::compute(&grammar).unwrap();
    let collection = ItemSetCollection::build_canonical_collection(&grammar, &ff);
    assert!(!collection.sets.is_empty());
}

// 60. State IDs in collection are sequential from 0
#[test]
fn collection_state_ids_sequential() {
    let grammar = indirect_grammar();
    let ff = FirstFollowSets::compute(&grammar).unwrap();
    let collection = ItemSetCollection::build_canonical_collection(&grammar, &ff);
    for (i, state) in collection.sets.iter().enumerate() {
        assert_eq!(
            state.id,
            StateId(i as u16),
            "state {} has id {} instead of {}",
            i,
            state.id.0,
            i
        );
    }
}

// ===========================================================================
// DETERMINISM — additional tests
// ===========================================================================

// 61. Canonical collection items are identical across runs (multi-nt)
#[test]
fn collection_items_deterministic_multi_nt() {
    let grammar = multi_nt_grammar();
    let ff = FirstFollowSets::compute(&grammar).unwrap();
    let c1 = ItemSetCollection::build_canonical_collection(&grammar, &ff);
    let c2 = ItemSetCollection::build_canonical_collection(&grammar, &ff);
    assert_eq!(c1.sets.len(), c2.sets.len());
    for i in 0..c1.sets.len() {
        assert_eq!(c1.sets[i].items, c2.sets[i].items, "state {} differs", i);
    }
}

// 62. Goto table is deterministic across runs
#[test]
fn goto_table_deterministic() {
    let grammar = recursive_grammar();
    let ff = FirstFollowSets::compute(&grammar).unwrap();
    let c1 = ItemSetCollection::build_canonical_collection(&grammar, &ff);
    let c2 = ItemSetCollection::build_canonical_collection(&grammar, &ff);
    assert_eq!(c1.goto_table.len(), c2.goto_table.len());
    for (key, &val) in &c1.goto_table {
        assert_eq!(
            c2.goto_table.get(key).copied(),
            Some(val),
            "goto table differs for {:?}",
            key
        );
    }
}

// 63. symbol_is_terminal map is consistent with grammar
#[test]
fn symbol_is_terminal_consistent() {
    let grammar = multi_nt_grammar();
    let ff = FirstFollowSets::compute(&grammar).unwrap();
    let collection = ItemSetCollection::build_canonical_collection(&grammar, &ff);
    for (sym, &is_term) in &collection.symbol_is_terminal {
        if grammar.tokens.contains_key(sym) {
            assert!(is_term, "token {:?} should be marked terminal", sym);
        }
        if grammar.rules.contains_key(sym) {
            assert!(
                !is_term,
                "non-terminal {:?} should not be marked terminal",
                sym
            );
        }
    }
}

// 64. Every non-initial state is reachable via goto from some state
#[test]
fn all_states_reachable_via_goto() {
    let grammar = indirect_grammar();
    let ff = FirstFollowSets::compute(&grammar).unwrap();
    let collection = ItemSetCollection::build_canonical_collection(&grammar, &ff);

    let goto_targets: HashSet<_> = collection.goto_table.values().copied().collect();
    for state in &collection.sets {
        if state.id != StateId(0) {
            assert!(
                goto_targets.contains(&state.id),
                "state {} is not reachable via any goto transition",
                state.id.0
            );
        }
    }
}

// 65. Augmented collection with known EOF/start produces at least one state
#[test]
fn augmented_collection_basic() {
    let a = SymbolId(1);
    let s = SymbolId(10);
    let s_prime = SymbolId(11);
    let eof = SymbolId(12);

    let mut grammar = Grammar::new("augmented_test".into());
    grammar.tokens.insert(
        a,
        Token {
            name: "a".into(),
            pattern: TokenPattern::String("a".into()),
            fragile: false,
        },
    );
    grammar.rule_names.insert(s, "S".into());
    grammar.rule_names.insert(s_prime, "S'".into());
    grammar.rules.insert(
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
    grammar.rules.insert(
        s_prime,
        vec![Rule {
            lhs: s_prime,
            rhs: vec![Symbol::NonTerminal(s)],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(1),
        }],
    );

    let ff = FirstFollowSets::compute(&grammar).unwrap();
    let collection =
        ItemSetCollection::build_canonical_collection_augmented(&grammar, &ff, s_prime, s, eof);
    assert!(
        !collection.sets.is_empty(),
        "augmented collection must have at least one state"
    );
}
