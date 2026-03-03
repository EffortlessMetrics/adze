#![allow(clippy::needless_range_loop)]
//! Property-based tests for canonical LR(1) collection building.
//!
//! Run with: `cargo test -p adze-glr-core --test canonical_collection_proptest`

use adze_glr_core::{FirstFollowSets, ItemSet, ItemSetCollection, LRItem};
use adze_ir::*;
use proptest::prelude::*;
use std::collections::BTreeSet;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

const MAX_TERM: u16 = 8;
const NT_BASE: u16 = 10;

fn tok(g: &mut Grammar, id: SymbolId, name: &str, pat: &str) {
    g.tokens.insert(
        id,
        Token {
            name: name.into(),
            pattern: TokenPattern::String(pat.into()),
            fragile: false,
        },
    );
}

fn rule(lhs: SymbolId, rhs: Vec<Symbol>, prod: u16) -> Rule {
    Rule {
        lhs,
        rhs,
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(prod),
    }
}

/// Build canonical collection from a mutable grammar.
fn build(grammar: &mut Grammar) -> (ItemSetCollection, FirstFollowSets) {
    let ff = FirstFollowSets::compute_normalized(grammar)
        .expect("FIRST/FOLLOW should succeed");
    let col = ItemSetCollection::build_canonical_collection(grammar, &ff);
    (col, ff)
}

/// Build augmented canonical collection.
fn build_augmented(
    grammar: &mut Grammar,
    augmented_start: SymbolId,
    original_start: SymbolId,
    eof: SymbolId,
) -> (ItemSetCollection, FirstFollowSets) {
    let ff = FirstFollowSets::compute_normalized(grammar)
        .expect("FIRST/FOLLOW should succeed");
    let col = ItemSetCollection::build_canonical_collection_augmented(
        grammar,
        &ff,
        augmented_start,
        original_start,
        eof,
    );
    (col, ff)
}

// ---------------------------------------------------------------------------
// Grammar strategies
// ---------------------------------------------------------------------------

fn arb_rhs_symbol() -> impl Strategy<Value = Symbol> {
    prop_oneof![
        8 => (1..=MAX_TERM).prop_map(|i| Symbol::Terminal(SymbolId(i))),
        4 => (NT_BASE..=NT_BASE + 3).prop_map(|i| Symbol::NonTerminal(SymbolId(i))),
    ]
}

fn arb_rhs() -> impl Strategy<Value = Vec<Symbol>> {
    prop::collection::vec(arb_rhs_symbol(), 1..=4)
}

fn arb_small_grammar() -> impl Strategy<Value = Grammar> {
    let num_nt = 1..=3usize;
    let num_term = 1..=3usize;
    (num_term, num_nt).prop_flat_map(|(nt, nn)| {
        let nt = nt.max(1);
        let nn = nn.max(1);
        let prods = prop::collection::vec(
            prop::collection::vec(arb_rhs(), 1..=3),
            nn..=nn,
        );
        prods.prop_map(move |all_prods| {
            let mut g = Grammar::new("proptest".into());
            for i in 1..=(nt as u16).min(MAX_TERM) {
                tok(&mut g, SymbolId(i), &format!("t{i}"), &format!("t{i}"));
            }
            let mut prod_counter = 0u16;
            for (idx, prods) in all_prods.iter().enumerate() {
                let nt_id = SymbolId(NT_BASE + idx as u16);
                g.rule_names.insert(nt_id, format!("N{idx}"));
                for rhs in prods {
                    let filtered_rhs: Vec<Symbol> = rhs
                        .iter()
                        .map(|sym| match sym {
                            Symbol::Terminal(id) if id.0 > (nt as u16).min(MAX_TERM) => {
                                Symbol::Terminal(SymbolId(1))
                            }
                            Symbol::NonTerminal(id) if id.0 >= NT_BASE + all_prods.len() as u16 => {
                                Symbol::NonTerminal(SymbolId(NT_BASE))
                            }
                            other => other.clone(),
                        })
                        .collect();
                    g.rules.entry(nt_id).or_default().push(rule(
                        nt_id,
                        filtered_rhs,
                        prod_counter,
                    ));
                    prod_counter += 1;
                }
            }
            g
        })
    })
}

/// Simple grammar: S → a
fn simple_grammar() -> Grammar {
    let mut g = Grammar::new("simple".into());
    let a = SymbolId(1);
    let s = SymbolId(NT_BASE);
    tok(&mut g, a, "a", "a");
    g.rule_names.insert(s, "S".into());
    g.rules.entry(s).or_default().push(rule(s, vec![Symbol::Terminal(a)], 0));
    g
}

/// Left-recursive grammar: E → E '+' a | a
fn left_recursive_grammar() -> Grammar {
    let mut g = Grammar::new("left_rec".into());
    let a = SymbolId(1);
    let plus = SymbolId(2);
    let e = SymbolId(NT_BASE);
    tok(&mut g, a, "a", "a");
    tok(&mut g, plus, "plus", "+");
    g.rule_names.insert(e, "E".into());
    g.rules.entry(e).or_default().push(rule(
        e,
        vec![Symbol::NonTerminal(e), Symbol::Terminal(plus), Symbol::Terminal(a)],
        0,
    ));
    g.rules.entry(e).or_default().push(rule(e, vec![Symbol::Terminal(a)], 1));
    g
}

/// Right-recursive grammar: E → a '+' E | a
fn right_recursive_grammar() -> Grammar {
    let mut g = Grammar::new("right_rec".into());
    let a = SymbolId(1);
    let plus = SymbolId(2);
    let e = SymbolId(NT_BASE);
    tok(&mut g, a, "a", "a");
    tok(&mut g, plus, "plus", "+");
    g.rule_names.insert(e, "E".into());
    g.rules.entry(e).or_default().push(rule(
        e,
        vec![Symbol::Terminal(a), Symbol::Terminal(plus), Symbol::NonTerminal(e)],
        0,
    ));
    g.rules.entry(e).or_default().push(rule(e, vec![Symbol::Terminal(a)], 1));
    g
}

/// Two-nonterminal grammar: S → A a, A → b
fn two_nt_grammar() -> Grammar {
    let mut g = Grammar::new("two_nt".into());
    let a = SymbolId(1);
    let b = SymbolId(2);
    let s = SymbolId(NT_BASE);
    let big_a = SymbolId(NT_BASE + 1);
    tok(&mut g, a, "a", "a");
    tok(&mut g, b, "b", "b");
    g.rule_names.insert(s, "S".into());
    g.rule_names.insert(big_a, "A".into());
    g.rules.entry(s).or_default().push(rule(
        s,
        vec![Symbol::NonTerminal(big_a), Symbol::Terminal(a)],
        0,
    ));
    g.rules.entry(big_a).or_default().push(rule(big_a, vec![Symbol::Terminal(b)], 1));
    g
}

/// Augmented grammar: S' → S, S → a
fn augmented_grammar() -> (Grammar, SymbolId, SymbolId, SymbolId) {
    let mut g = Grammar::new("augmented".into());
    let a = SymbolId(1);
    let eof = SymbolId(2);
    let s = SymbolId(NT_BASE);
    let s_prime = SymbolId(NT_BASE + 1);
    tok(&mut g, a, "a", "a");
    tok(&mut g, eof, "EOF", "$");
    g.rule_names.insert(s, "S".into());
    g.rule_names.insert(s_prime, "S_prime".into());
    g.rules.entry(s).or_default().push(rule(s, vec![Symbol::Terminal(a)], 0));
    g.rules.entry(s_prime).or_default().push(rule(
        s_prime,
        vec![Symbol::NonTerminal(s)],
        1,
    ));
    (g, s_prime, s, eof)
}

// ===========================================================================
// 1. build_canonical_collection produces non-empty collection
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn collection_non_empty_random(mut g in arb_small_grammar()) {
        let (col, _) = build(&mut g);
        prop_assert!(!col.sets.is_empty(), "canonical collection must have ≥1 state");
    }
}

#[test]
fn collection_non_empty_simple() {
    let mut g = simple_grammar();
    let (col, _) = build(&mut g);
    assert!(!col.sets.is_empty());
}

#[test]
fn collection_non_empty_left_recursive() {
    let mut g = left_recursive_grammar();
    let (col, _) = build(&mut g);
    assert!(!col.sets.is_empty());
}

#[test]
fn collection_non_empty_right_recursive() {
    let mut g = right_recursive_grammar();
    let (col, _) = build(&mut g);
    assert!(!col.sets.is_empty());
}

// ===========================================================================
// 2. build_canonical_collection_augmented includes augmented start
// ===========================================================================

#[test]
fn augmented_initial_state_has_augmented_start_item() {
    let (mut g, s_prime, s, eof) = augmented_grammar();
    let (col, _) = build_augmented(&mut g, s_prime, s, eof);

    assert!(!col.sets.is_empty(), "augmented collection must have states");
    let initial = &col.sets[0];
    // The initial state should contain an item whose rule_id corresponds to S' → S
    let has_augmented = initial.items.iter().any(|item| {
        if let Some(r) = g.all_rules().find(|r| r.production_id.0 == item.rule_id.0) {
            r.lhs == s_prime
        } else {
            false
        }
    });
    assert!(has_augmented, "initial state must contain augmented start item");
}

#[test]
fn augmented_collection_has_more_states_than_empty() {
    let (mut g, s_prime, s, eof) = augmented_grammar();
    let (col, _) = build_augmented(&mut g, s_prime, s, eof);
    assert!(col.sets.len() >= 2, "augmented grammar needs ≥2 states");
}

#[test]
fn augmented_initial_item_lookahead_is_eof() {
    let (mut g, s_prime, s, eof) = augmented_grammar();
    let (col, _) = build_augmented(&mut g, s_prime, s, eof);

    let initial = &col.sets[0];
    let aug_items: Vec<_> = initial.items.iter().filter(|item| {
        g.all_rules().any(|r| r.production_id.0 == item.rule_id.0 && r.lhs == s_prime)
    }).collect();
    assert!(!aug_items.is_empty());
    for item in &aug_items {
        assert_eq!(item.lookahead, eof, "augmented start items should have EOF lookahead");
    }
}

// ===========================================================================
// 3. Collection state count determinism
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    #[test]
    fn state_count_deterministic(mut g in arb_small_grammar()) {
        let mut g2 = g.clone();
        let (col1, _) = build(&mut g);
        let (col2, _) = build(&mut g2);
        prop_assert_eq!(col1.sets.len(), col2.sets.len(), "same grammar must yield same state count");
    }
}

#[test]
fn state_count_deterministic_simple() {
    let mut g1 = simple_grammar();
    let mut g2 = simple_grammar();
    let (c1, _) = build(&mut g1);
    let (c2, _) = build(&mut g2);
    assert_eq!(c1.sets.len(), c2.sets.len());
}


// ===========================================================================
// 4. Collection with simple grammar
// ===========================================================================

#[test]
fn simple_grammar_at_least_two_states() {
    let mut g = simple_grammar();
    let (col, _) = build(&mut g);
    // S → a requires at least: initial state and state after shifting 'a'
    assert!(col.sets.len() >= 2, "S→a needs ≥2 states, got {}", col.sets.len());
}

#[test]
fn simple_grammar_initial_state_is_zero() {
    let mut g = simple_grammar();
    let (col, _) = build(&mut g);
    assert_eq!(col.sets[0].id, StateId(0));
}

#[test]
fn simple_grammar_unique_state_ids() {
    let mut g = simple_grammar();
    let (col, _) = build(&mut g);
    let ids: BTreeSet<_> = col.sets.iter().map(|s| s.id).collect();
    assert_eq!(ids.len(), col.sets.len(), "state IDs must be unique");
}

#[test]
fn simple_grammar_has_goto_entries() {
    let mut g = simple_grammar();
    let (col, _) = build(&mut g);
    assert!(!col.goto_table.is_empty(), "S→a must have ≥1 goto entry");
}

// ===========================================================================
// 5. Collection with left-recursive grammar
// ===========================================================================

#[test]
fn left_recursive_multiple_states() {
    let mut g = left_recursive_grammar();
    let (col, _) = build(&mut g);
    // E → E '+' a | a requires more states than the simple grammar
    assert!(col.sets.len() >= 3, "left-recursive grammar needs ≥3 states, got {}", col.sets.len());
}

#[test]
fn left_recursive_has_self_referential_items() {
    let mut g = left_recursive_grammar();
    let e = SymbolId(NT_BASE);
    let (col, _) = build(&mut g);

    // At least one state should have an item referencing E on the RHS
    let has_self_ref = col.sets.iter().any(|set| {
        set.items.iter().any(|item| {
            if let Some(r) = g.all_rules().find(|r| r.production_id.0 == item.rule_id.0) {
                r.rhs.contains(&Symbol::NonTerminal(e)) && r.lhs == e
            } else {
                false
            }
        })
    });
    assert!(has_self_ref, "left-recursive grammar must have self-referential items");
}

#[test]
fn left_recursive_goto_targets_valid() {
    let mut g = left_recursive_grammar();
    let (col, _) = build(&mut g);
    let max_state = col.sets.len();
    for ((src, _sym), tgt) in &col.goto_table {
        assert!((src.0 as usize) < max_state, "goto source out of range");
        assert!((tgt.0 as usize) < max_state, "goto target out of range");
    }
}

// ===========================================================================
// 6. Collection with right-recursive grammar
// ===========================================================================

#[test]
fn right_recursive_multiple_states() {
    let mut g = right_recursive_grammar();
    let (col, _) = build(&mut g);
    assert!(col.sets.len() >= 3, "right-recursive grammar needs ≥3 states, got {}", col.sets.len());
}

#[test]
fn right_recursive_has_self_referential_items() {
    let mut g = right_recursive_grammar();
    let e = SymbolId(NT_BASE);
    let (col, _) = build(&mut g);

    let has_self_ref = col.sets.iter().any(|set| {
        set.items.iter().any(|item| {
            if let Some(r) = g.all_rules().find(|r| r.production_id.0 == item.rule_id.0) {
                r.rhs.contains(&Symbol::NonTerminal(e)) && r.lhs == e
            } else {
                false
            }
        })
    });
    assert!(has_self_ref, "right-recursive grammar must have self-referential items");
}


// ===========================================================================
// 7. Collection closure correctness
// ===========================================================================

#[test]
fn closure_on_terminal_only_rule_no_expansion() {
    let mut g = simple_grammar();
    let ff = FirstFollowSets::compute_normalized(&mut g).unwrap();
    let s = SymbolId(NT_BASE);
    let start_rules = g.get_rules_for_symbol(s).unwrap();

    let mut set = ItemSet::new(StateId(0));
    for r in start_rules {
        set.add_item(LRItem::new(RuleId(r.production_id.0), 0, SymbolId(0)));
    }
    let before = set.items.len();
    set.closure(&g, &ff).unwrap();
    // Terminal-only rule: closure should not add new items
    assert_eq!(set.items.len(), before, "closure on terminal-only items should not expand");
}

#[test]
fn closure_on_nonterminal_expands() {
    let mut g = two_nt_grammar();
    let ff = FirstFollowSets::compute_normalized(&mut g).unwrap();
    let s = SymbolId(NT_BASE);
    let start_rules = g.get_rules_for_symbol(s).unwrap();

    let mut set = ItemSet::new(StateId(0));
    for r in start_rules {
        set.add_item(LRItem::new(RuleId(r.production_id.0), 0, SymbolId(0)));
    }
    let before = set.items.len();
    set.closure(&g, &ff).unwrap();
    assert!(set.items.len() > before, "closure on NT should add items for A → b");
}

#[test]
fn closure_idempotent() {
    let mut g = left_recursive_grammar();
    let ff = FirstFollowSets::compute_normalized(&mut g).unwrap();
    let e = SymbolId(NT_BASE);
    let start_rules = g.get_rules_for_symbol(e).unwrap();

    let mut set = ItemSet::new(StateId(0));
    for r in start_rules {
        set.add_item(LRItem::new(RuleId(r.production_id.0), 0, SymbolId(0)));
    }
    set.closure(&g, &ff).unwrap();
    let after_first: BTreeSet<_> = set.items.clone();
    set.closure(&g, &ff).unwrap();
    assert_eq!(set.items, after_first, "closure must be idempotent");
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(60))]

    #[test]
    fn closure_idempotent_random(mut g in arb_small_grammar()) {
        let ff = match FirstFollowSets::compute_normalized(&mut g) {
            Ok(ff) => ff,
            Err(_) => return Ok(()),
        };
        if let Some(start) = g.start_symbol() {
            if let Some(rules) = g.get_rules_for_symbol(start) {
                let mut set = ItemSet::new(StateId(0));
                for r in rules {
                    set.add_item(LRItem::new(RuleId(r.production_id.0), 0, SymbolId(0)));
                }
                let _ = set.closure(&g, &ff);
                let snapshot: BTreeSet<_> = set.items.clone();
                let _ = set.closure(&g, &ff);
                prop_assert_eq!(set.items, snapshot, "closure must be idempotent");
            }
        }
    }
}

// ===========================================================================
// 8. Collection goto correctness
// ===========================================================================

#[test]
fn goto_on_terminal_produces_advanced_position() {
    let mut g = simple_grammar();
    let ff = FirstFollowSets::compute_normalized(&mut g).unwrap();
    let s = SymbolId(NT_BASE);
    let a = SymbolId(1);
    let start_rules = g.get_rules_for_symbol(s).unwrap();

    let mut set = ItemSet::new(StateId(0));
    for r in start_rules {
        set.add_item(LRItem::new(RuleId(r.production_id.0), 0, SymbolId(0)));
    }
    set.closure(&g, &ff).unwrap();

    let goto_set = set.goto(&Symbol::Terminal(a), &g, &ff);
    assert!(!goto_set.items.is_empty(), "goto on 'a' should produce items");
    // All items in the goto set should have position > 0
    for item in &goto_set.items {
        if let Some(r) = g.all_rules().find(|r| r.production_id.0 == item.rule_id.0) {
            if r.lhs == s {
                assert!(item.position > 0, "goto items should have advanced dot position");
            }
        }
    }
}

#[test]
fn goto_on_absent_symbol_is_empty() {
    let mut g = simple_grammar();
    let ff = FirstFollowSets::compute_normalized(&mut g).unwrap();
    let s = SymbolId(NT_BASE);
    let start_rules = g.get_rules_for_symbol(s).unwrap();

    let mut set = ItemSet::new(StateId(0));
    for r in start_rules {
        set.add_item(LRItem::new(RuleId(r.production_id.0), 0, SymbolId(0)));
    }
    set.closure(&g, &ff).unwrap();

    // Symbol 99 doesn't exist in the grammar
    let goto_set = set.goto(&Symbol::Terminal(SymbolId(99)), &g, &ff);
    assert!(goto_set.items.is_empty(), "goto on absent symbol should be empty");
}

#[test]
fn goto_table_sources_match_states() {
    let mut g = left_recursive_grammar();
    let (col, _) = build(&mut g);
    let state_ids: BTreeSet<_> = col.sets.iter().map(|s| s.id).collect();
    for ((src, _), _) in &col.goto_table {
        assert!(state_ids.contains(src), "goto source must be a known state");
    }
}

#[test]
fn goto_table_targets_match_states() {
    let mut g = right_recursive_grammar();
    let (col, _) = build(&mut g);
    let state_ids: BTreeSet<_> = col.sets.iter().map(|s| s.id).collect();
    for (_, tgt) in &col.goto_table {
        assert!(state_ids.contains(tgt), "goto target must be a known state");
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(60))]

    #[test]
    fn goto_table_all_targets_valid(mut g in arb_small_grammar()) {
        let (col, _) = build(&mut g);
        let max_state = col.sets.len();
        for ((src, _), tgt) in &col.goto_table {
            prop_assert!((src.0 as usize) < max_state, "goto source out of range");
            prop_assert!((tgt.0 as usize) < max_state, "goto target out of range");
        }
    }
}

// ===========================================================================
// Additional structural property tests
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(60))]

    #[test]
    fn all_state_ids_sequential(mut g in arb_small_grammar()) {
        let (col, _) = build(&mut g);
        for (i, set) in col.sets.iter().enumerate() {
            prop_assert_eq!(set.id, StateId(i as u16), "state IDs must be sequential");
        }
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(60))]

    #[test]
    fn no_empty_item_sets(mut g in arb_small_grammar()) {
        let (col, _) = build(&mut g);
        for set in &col.sets {
            prop_assert!(!set.items.is_empty(), "no state should be empty");
        }
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(60))]

    #[test]
    fn no_duplicate_item_sets(mut g in arb_small_grammar()) {
        let (col, _) = build(&mut g);
        for i in 0..col.sets.len() {
            for j in (i + 1)..col.sets.len() {
                prop_assert!(
                    col.sets[i].items != col.sets[j].items,
                    "states {} and {} have identical item sets",
                    i, j
                );
            }
        }
    }
}

#[test]
fn two_nt_grammar_closure_includes_derived_items() {
    let mut g = two_nt_grammar();
    let (col, _) = build(&mut g);
    let big_a = SymbolId(NT_BASE + 1);

    // Some state should reference the A → b rule
    let has_a_rule = col.sets.iter().any(|set| {
        set.items.iter().any(|item| {
            g.all_rules().any(|r| r.production_id.0 == item.rule_id.0 && r.lhs == big_a)
        })
    });
    assert!(has_a_rule, "collection should contain items from A → b");
}

#[test]
fn two_nt_grammar_has_goto_for_nonterminal() {
    let mut g = two_nt_grammar();
    let (col, _) = build(&mut g);
    let big_a = SymbolId(NT_BASE + 1);

    // There should be a goto on non-terminal A
    let has_nt_goto = col.goto_table.keys().any(|(_, sym)| *sym == big_a);
    assert!(has_nt_goto, "should have goto entry for non-terminal A");
}

#[test]
fn symbol_is_terminal_tracking() {
    let mut g = two_nt_grammar();
    let (col, _) = build(&mut g);
    let a = SymbolId(1);
    let big_a = SymbolId(NT_BASE + 1);

    if let Some(&is_term) = col.symbol_is_terminal.get(&a) {
        assert!(is_term, "terminal 'a' should be marked as terminal");
    }
    if let Some(&is_term) = col.symbol_is_terminal.get(&big_a) {
        assert!(!is_term, "non-terminal 'A' should not be marked as terminal");
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(60))]

    #[test]
    fn initial_state_always_zero(mut g in arb_small_grammar()) {
        let (col, _) = build(&mut g);
        prop_assert_eq!(col.sets[0].id, StateId(0), "initial state must be StateId(0)");
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(60))]

    #[test]
    fn goto_table_consistent_with_symbol_is_terminal(mut g in arb_small_grammar()) {
        let (col, _) = build(&mut g);
        for ((_, sym), _) in &col.goto_table {
            prop_assert!(
                col.symbol_is_terminal.contains_key(sym),
                "every symbol in goto_table should have is_terminal entry"
            );
        }
    }
}
