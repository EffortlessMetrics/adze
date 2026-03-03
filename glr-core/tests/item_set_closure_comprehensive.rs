#![allow(clippy::needless_range_loop)]
//! Comprehensive tests for LR(1) item set closure computation.
//!
//! Run with: `cargo test -p adze-glr-core --test item_set_closure_comprehensive`

use adze_glr_core::{FirstFollowSets, ItemSet, ItemSetCollection, LRItem};
use adze_ir::*;
use std::collections::BTreeSet;
use std::hash::{Hash, Hasher};

// ---------------------------------------------------------------------------
// Helper: build a grammar with S -> a
// ---------------------------------------------------------------------------
fn simple_grammar() -> Grammar {
    let a = SymbolId(1);
    let s = SymbolId(10);

    let mut g = Grammar::new("simple".into());
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

// ---------------------------------------------------------------------------
// Helper: grammar with epsilon production  S -> ε | a
// ---------------------------------------------------------------------------
fn epsilon_grammar() -> Grammar {
    let a = SymbolId(1);
    let s = SymbolId(10);

    let mut g = Grammar::new("epsilon".into());
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
        vec![
            Rule {
                lhs: s,
                rhs: vec![Symbol::Epsilon],
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
    g
}

// ---------------------------------------------------------------------------
// Helper: grammar with left-recursion  E -> E '+' a | a
// ---------------------------------------------------------------------------
fn left_recursive_grammar() -> Grammar {
    let a = SymbolId(1);
    let plus = SymbolId(2);
    let e = SymbolId(10);

    let mut g = Grammar::new("left_rec".into());
    g.tokens.insert(
        a,
        Token {
            name: "a".into(),
            pattern: TokenPattern::String("a".into()),
            fragile: false,
        },
    );
    g.tokens.insert(
        plus,
        Token {
            name: "+".into(),
            pattern: TokenPattern::String("+".into()),
            fragile: false,
        },
    );
    g.rule_names.insert(e, "E".into());
    g.rules.insert(
        e,
        vec![
            Rule {
                lhs: e,
                rhs: vec![
                    Symbol::NonTerminal(e),
                    Symbol::Terminal(plus),
                    Symbol::Terminal(a),
                ],
                precedence: None,
                associativity: None,
                fields: vec![],
                production_id: ProductionId(0),
            },
            Rule {
                lhs: e,
                rhs: vec![Symbol::Terminal(a)],
                precedence: None,
                associativity: None,
                fields: vec![],
                production_id: ProductionId(1),
            },
        ],
    );
    g
}

// ---------------------------------------------------------------------------
// Helper: two-nonterminal grammar  S -> A a ;  A -> b
// ---------------------------------------------------------------------------
fn two_nonterminal_grammar() -> Grammar {
    let a = SymbolId(1);
    let b = SymbolId(2);
    let s = SymbolId(10);
    let a_nt = SymbolId(11);

    let mut g = Grammar::new("two_nt".into());
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
    g.rule_names.insert(a_nt, "A".into());
    g.rules.insert(
        s,
        vec![Rule {
            lhs: s,
            rhs: vec![Symbol::NonTerminal(a_nt), Symbol::Terminal(a)],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(0),
        }],
    );
    g.rules.insert(
        a_nt,
        vec![Rule {
            lhs: a_nt,
            rhs: vec![Symbol::Terminal(b)],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(1),
        }],
    );
    g
}

// ---------------------------------------------------------------------------
// Helper: right-recursive grammar  L -> a L | a
// ---------------------------------------------------------------------------
fn right_recursive_grammar() -> Grammar {
    let a = SymbolId(1);
    let l = SymbolId(10);

    let mut g = Grammar::new("right_rec".into());
    g.tokens.insert(
        a,
        Token {
            name: "a".into(),
            pattern: TokenPattern::String("a".into()),
            fragile: false,
        },
    );
    g.rule_names.insert(l, "L".into());
    g.rules.insert(
        l,
        vec![
            Rule {
                lhs: l,
                rhs: vec![Symbol::Terminal(a), Symbol::NonTerminal(l)],
                precedence: None,
                associativity: None,
                fields: vec![],
                production_id: ProductionId(0),
            },
            Rule {
                lhs: l,
                rhs: vec![Symbol::Terminal(a)],
                precedence: None,
                associativity: None,
                fields: vec![],
                production_id: ProductionId(1),
            },
        ],
    );
    g
}

// ===========================================================================
// Tests
// ===========================================================================

// ---- Empty item set closure ----

#[test]
fn empty_item_set_closure_is_empty() {
    let g = simple_grammar();
    let ff = FirstFollowSets::compute(&g).unwrap();

    let mut set = ItemSet::new(StateId(0));
    set.closure(&g, &ff).unwrap();
    assert!(set.items.is_empty(), "closure of empty set should be empty");
}

// ---- Single kernel item closure ----

#[test]
fn single_terminal_item_no_prediction() {
    // S -> • a   (no nonterminal after dot → closure adds nothing)
    let g = simple_grammar();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let s = SymbolId(10);

    let mut set = ItemSet::new(StateId(0));
    let item = LRItem::new(RuleId(0), 0, SymbolId(0)); // production 0, pos 0, lookahead EOF
    set.add_item(item.clone());
    set.closure(&g, &ff).unwrap();

    // Only the kernel item should remain – no nonterminal after dot.
    assert_eq!(set.items.len(), 1);
    assert!(set.items.contains(&item));

    // Verify the item relates to a real rule
    let rule = g.get_rules_for_symbol(s).unwrap().first().unwrap();
    assert_eq!(rule.rhs, vec![Symbol::Terminal(SymbolId(1))]);
}

#[test]
fn single_nonterminal_item_predicts() {
    // S -> • A a  should predict  A -> • b
    let g = two_nonterminal_grammar();
    let ff = FirstFollowSets::compute(&g).unwrap();

    let mut set = ItemSet::new(StateId(0));
    // S -> • A a , lookahead $
    set.add_item(LRItem::new(RuleId(0), 0, SymbolId(0)));
    set.closure(&g, &ff).unwrap();

    // We expect the kernel item plus A -> • b with some lookahead
    assert!(set.items.len() >= 2, "closure should predict A's rules");

    // A -> • b should be present (production_id 1, position 0)
    let has_a_prediction = set.items.iter().any(|i| i.rule_id == RuleId(1) && i.position == 0);
    assert!(has_a_prediction, "should contain A -> • b");
}

// ---- Closure with epsilon productions ----

#[test]
fn epsilon_production_is_reduce_item() {
    let g = epsilon_grammar();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let s = SymbolId(10);

    let mut set = ItemSet::new(StateId(0));
    // S -> • ε , $   (production 0)
    let eps_item = LRItem::new(RuleId(0), 0, SymbolId(0));
    set.add_item(eps_item.clone());
    set.closure(&g, &ff).unwrap();

    assert!(eps_item.is_reduce_item(&g), "epsilon item should be a reduce item at position 0");
}

#[test]
fn closure_with_epsilon_alternative() {
    let g = epsilon_grammar();
    let ff = FirstFollowSets::compute(&g).unwrap();

    let mut set = ItemSet::new(StateId(0));
    // Add both S productions as kernel items
    set.add_item(LRItem::new(RuleId(0), 0, SymbolId(0))); // S -> • ε
    set.add_item(LRItem::new(RuleId(1), 0, SymbolId(0))); // S -> • a
    set.closure(&g, &ff).unwrap();

    // Both items should be present
    assert!(set.items.len() >= 2);
}

// ---- Closure with recursive rules ----

#[test]
fn left_recursive_closure_terminates() {
    let g = left_recursive_grammar();
    let ff = FirstFollowSets::compute(&g).unwrap();

    let mut set = ItemSet::new(StateId(0));
    // E -> • E '+' a , $
    set.add_item(LRItem::new(RuleId(0), 0, SymbolId(0)));
    set.closure(&g, &ff).unwrap();

    // Should have predicted E -> • a as well
    let has_base = set.items.iter().any(|i| i.rule_id == RuleId(1) && i.position == 0);
    assert!(has_base, "left-recursive closure should predict base case E -> • a");
}

#[test]
fn left_recursive_closure_has_recursive_prediction() {
    let g = left_recursive_grammar();
    let ff = FirstFollowSets::compute(&g).unwrap();

    let mut set = ItemSet::new(StateId(0));
    set.add_item(LRItem::new(RuleId(0), 0, SymbolId(0)));
    set.closure(&g, &ff).unwrap();

    // E -> • E '+' a should also be re-predicted (same as kernel)
    let has_recursive = set.items.iter().any(|i| i.rule_id == RuleId(0) && i.position == 0);
    assert!(has_recursive, "recursive prediction should appear");
}

#[test]
fn right_recursive_closure_terminates() {
    let g = right_recursive_grammar();
    let ff = FirstFollowSets::compute(&g).unwrap();

    let mut set = ItemSet::new(StateId(0));
    // L -> • a L , $  (production 0)
    set.add_item(LRItem::new(RuleId(0), 0, SymbolId(0)));
    set.closure(&g, &ff).unwrap();

    // No nonterminal after dot at position 0 of production 0 (it's terminal 'a'),
    // so closure should not add predictions.
    // Only kernel + L -> • a (production 1) might appear if also seeded.
    // With only production 0 seeded and 'a' being terminal, closure adds nothing.
    assert!(set.items.len() >= 1);
}

// ---- Multiple kernel items ----

#[test]
fn multiple_kernel_items_all_present_after_closure() {
    let g = left_recursive_grammar();
    let ff = FirstFollowSets::compute(&g).unwrap();

    let mut set = ItemSet::new(StateId(0));
    let k1 = LRItem::new(RuleId(0), 0, SymbolId(0)); // E -> • E '+' a , $
    let k2 = LRItem::new(RuleId(1), 0, SymbolId(0)); // E -> • a , $
    set.add_item(k1.clone());
    set.add_item(k2.clone());
    set.closure(&g, &ff).unwrap();

    assert!(set.items.contains(&k1));
    assert!(set.items.contains(&k2));
}

#[test]
fn multiple_kernel_items_may_produce_different_lookaheads() {
    let g = left_recursive_grammar();
    let ff = FirstFollowSets::compute(&g).unwrap();

    let plus = SymbolId(2);
    let eof = SymbolId(0);

    let mut set = ItemSet::new(StateId(0));
    set.add_item(LRItem::new(RuleId(0), 0, eof));
    set.add_item(LRItem::new(RuleId(0), 0, plus));
    set.closure(&g, &ff).unwrap();

    // Both kernel items (different lookaheads) should survive
    assert!(set.items.contains(&LRItem::new(RuleId(0), 0, eof)));
    assert!(set.items.contains(&LRItem::new(RuleId(0), 0, plus)));
}

// ---- GOTO computation ----

#[test]
fn goto_on_terminal_advances_dot() {
    let g = simple_grammar();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let a = SymbolId(1);

    let mut set = ItemSet::new(StateId(0));
    set.add_item(LRItem::new(RuleId(0), 0, SymbolId(0))); // S -> • a , $
    set.closure(&g, &ff).unwrap();

    let goto_set = set.goto(&Symbol::Terminal(a), &g, &ff);
    // Should contain S -> a • , $
    let has_advanced = goto_set
        .items
        .iter()
        .any(|i| i.rule_id == RuleId(0) && i.position == 1);
    assert!(has_advanced, "GOTO(I, a) should advance dot past 'a'");
}

#[test]
fn goto_on_nonterminal_advances_dot() {
    let g = two_nonterminal_grammar();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let a_nt = SymbolId(11);

    let mut set = ItemSet::new(StateId(0));
    set.add_item(LRItem::new(RuleId(0), 0, SymbolId(0))); // S -> • A a , $
    set.closure(&g, &ff).unwrap();

    let goto_set = set.goto(&Symbol::NonTerminal(a_nt), &g, &ff);
    // Should contain S -> A • a , $
    let has_advanced = goto_set
        .items
        .iter()
        .any(|i| i.rule_id == RuleId(0) && i.position == 1);
    assert!(has_advanced, "GOTO(I, A) should advance dot past A");
}

#[test]
fn goto_on_unrelated_symbol_is_empty() {
    let g = simple_grammar();
    let ff = FirstFollowSets::compute(&g).unwrap();

    let mut set = ItemSet::new(StateId(0));
    set.add_item(LRItem::new(RuleId(0), 0, SymbolId(0))); // S -> • a , $
    set.closure(&g, &ff).unwrap();

    // Symbol b (id=99) does not appear in the grammar rhs at the dot
    let goto_set = set.goto(&Symbol::Terminal(SymbolId(99)), &g, &ff);
    assert!(goto_set.items.is_empty(), "GOTO on unrelated symbol should be empty");
}

#[test]
fn goto_of_empty_set_is_empty() {
    let g = simple_grammar();
    let ff = FirstFollowSets::compute(&g).unwrap();

    let set = ItemSet::new(StateId(0));
    let goto_set = set.goto(&Symbol::Terminal(SymbolId(1)), &g, &ff);
    assert!(goto_set.items.is_empty());
}

#[test]
fn goto_computes_closure_of_result() {
    // S -> • A a ;  A -> • b
    // GOTO(I0, b) should produce { A -> b • } (with closure, though nothing to predict)
    // But GOTO(I0, A) should produce { S -> A • a } which also has no prediction.
    // Use two_nonterminal_grammar to check that goto includes closure.
    let g = two_nonterminal_grammar();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let b = SymbolId(2);

    let mut set = ItemSet::new(StateId(0));
    set.add_item(LRItem::new(RuleId(0), 0, SymbolId(0))); // S -> • A a , $
    set.closure(&g, &ff).unwrap();

    // After closure, should have A -> • b
    let goto_b = set.goto(&Symbol::Terminal(b), &g, &ff);
    // A -> b •
    let has_reduce = goto_b
        .items
        .iter()
        .any(|i| i.rule_id == RuleId(1) && i.position == 1);
    assert!(has_reduce, "GOTO(I, b) should yield A -> b •");
}

// ---- Item equality and hashing ----

#[test]
fn lr_item_equality_same_fields() {
    let a = LRItem::new(RuleId(5), 2, SymbolId(3));
    let b = LRItem::new(RuleId(5), 2, SymbolId(3));
    assert_eq!(a, b);
}

#[test]
fn lr_item_inequality_different_rule() {
    let a = LRItem::new(RuleId(5), 2, SymbolId(3));
    let b = LRItem::new(RuleId(6), 2, SymbolId(3));
    assert_ne!(a, b);
}

#[test]
fn lr_item_inequality_different_position() {
    let a = LRItem::new(RuleId(5), 2, SymbolId(3));
    let b = LRItem::new(RuleId(5), 3, SymbolId(3));
    assert_ne!(a, b);
}

#[test]
fn lr_item_inequality_different_lookahead() {
    let a = LRItem::new(RuleId(5), 2, SymbolId(3));
    let b = LRItem::new(RuleId(5), 2, SymbolId(4));
    assert_ne!(a, b);
}

#[test]
fn lr_item_hash_consistent_with_eq() {
    use std::collections::hash_map::DefaultHasher;

    let a = LRItem::new(RuleId(1), 0, SymbolId(0));
    let b = LRItem::new(RuleId(1), 0, SymbolId(0));

    let hash_of = |item: &LRItem| {
        let mut h = DefaultHasher::new();
        item.hash(&mut h);
        h.finish()
    };

    assert_eq!(a, b);
    assert_eq!(hash_of(&a), hash_of(&b), "equal items must have equal hashes");
}

#[test]
fn lr_item_btreeset_deduplication() {
    let mut set = BTreeSet::new();
    set.insert(LRItem::new(RuleId(0), 0, SymbolId(0)));
    set.insert(LRItem::new(RuleId(0), 0, SymbolId(0)));
    assert_eq!(set.len(), 1, "BTreeSet should deduplicate identical items");
}

// ---- Closure convergence (idempotence) ----

#[test]
fn closure_is_idempotent_simple() {
    let g = simple_grammar();
    let ff = FirstFollowSets::compute(&g).unwrap();

    let mut set = ItemSet::new(StateId(0));
    set.add_item(LRItem::new(RuleId(0), 0, SymbolId(0)));
    set.closure(&g, &ff).unwrap();
    let after_first: BTreeSet<_> = set.items.clone();

    set.closure(&g, &ff).unwrap();
    assert_eq!(set.items, after_first, "second closure should not change set");
}

#[test]
fn closure_is_idempotent_two_nonterminal() {
    let g = two_nonterminal_grammar();
    let ff = FirstFollowSets::compute(&g).unwrap();

    let mut set = ItemSet::new(StateId(0));
    set.add_item(LRItem::new(RuleId(0), 0, SymbolId(0)));
    set.closure(&g, &ff).unwrap();
    let after_first: BTreeSet<_> = set.items.clone();

    set.closure(&g, &ff).unwrap();
    assert_eq!(set.items, after_first, "closure should be idempotent");
}

#[test]
fn closure_is_idempotent_left_recursive() {
    let g = left_recursive_grammar();
    let ff = FirstFollowSets::compute(&g).unwrap();

    let mut set = ItemSet::new(StateId(0));
    set.add_item(LRItem::new(RuleId(0), 0, SymbolId(0)));
    set.closure(&g, &ff).unwrap();
    let after_first: BTreeSet<_> = set.items.clone();

    // Run closure two more times
    set.closure(&g, &ff).unwrap();
    set.closure(&g, &ff).unwrap();
    assert_eq!(set.items, after_first);
}

#[test]
fn closure_is_idempotent_epsilon() {
    let g = epsilon_grammar();
    let ff = FirstFollowSets::compute(&g).unwrap();

    let mut set = ItemSet::new(StateId(0));
    set.add_item(LRItem::new(RuleId(0), 0, SymbolId(0)));
    set.add_item(LRItem::new(RuleId(1), 0, SymbolId(0)));
    set.closure(&g, &ff).unwrap();
    let after_first: BTreeSet<_> = set.items.clone();

    set.closure(&g, &ff).unwrap();
    assert_eq!(set.items, after_first);
}

// ---- Large item sets ----

#[test]
fn large_grammar_many_alternatives() {
    // S -> a1 | a2 | ... | a20  (20 alternatives)
    let s = SymbolId(100);
    let mut g = Grammar::new("large".into());
    g.rule_names.insert(s, "S".into());

    let mut rules = Vec::new();
    for i in 0..20 {
        let t = SymbolId(i + 1);
        g.tokens.insert(
            t,
            Token {
                name: format!("t{}", i),
                pattern: TokenPattern::String(format!("t{}", i)),
                fragile: false,
            },
        );
        rules.push(Rule {
            lhs: s,
            rhs: vec![Symbol::Terminal(t)],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(i as u16),
        });
    }
    g.rules.insert(s, rules);

    let ff = FirstFollowSets::compute(&g).unwrap();
    let collection = ItemSetCollection::build_canonical_collection(&g, &ff);
    assert!(!collection.sets.is_empty());
    // State 0 should have all 20 kernel items plus no extra predictions (all terminal)
    assert!(collection.sets[0].items.len() >= 20);
}

#[test]
fn large_chain_grammar() {
    // Chain: S -> A1 ; A1 -> A2 ; ... ; A9 -> a
    let a = SymbolId(1);
    let mut g = Grammar::new("chain".into());
    g.tokens.insert(
        a,
        Token {
            name: "a".into(),
            pattern: TokenPattern::String("a".into()),
            fragile: false,
        },
    );

    let chain_len = 10;
    let base_nt = 10u16;
    for i in 0..chain_len {
        let nt = SymbolId(base_nt + i);
        g.rule_names.insert(nt, format!("A{}", i));
        let rhs = if i == chain_len - 1 {
            vec![Symbol::Terminal(a)]
        } else {
            vec![Symbol::NonTerminal(SymbolId(base_nt + i + 1))]
        };
        g.rules.insert(
            nt,
            vec![Rule {
                lhs: nt,
                rhs,
                precedence: None,
                associativity: None,
                fields: vec![],
                production_id: ProductionId(i as u16),
            }],
        );
    }

    let ff = FirstFollowSets::compute(&g).unwrap();

    // Closure of S -> • A1 should predict through the entire chain
    let mut set = ItemSet::new(StateId(0));
    set.add_item(LRItem::new(RuleId(0), 0, SymbolId(0)));
    set.closure(&g, &ff).unwrap();

    // Each link in the chain should generate a predicted item
    assert!(
        set.items.len() >= chain_len as usize,
        "chain closure should propagate through {} levels, got {} items",
        chain_len,
        set.items.len()
    );
}

#[test]
fn large_item_set_with_many_lookaheads() {
    // S -> A a ; S -> A b ; A -> c
    // Two rules mean A -> • c might appear with two different lookaheads
    let a = SymbolId(1);
    let b = SymbolId(2);
    let c = SymbolId(3);
    let s = SymbolId(10);
    let a_nt = SymbolId(11);

    let mut g = Grammar::new("multi_la".into());
    for (id, name) in [(a, "a"), (b, "b"), (c, "c")] {
        g.tokens.insert(
            id,
            Token {
                name: name.into(),
                pattern: TokenPattern::String(name.into()),
                fragile: false,
            },
        );
    }
    g.rule_names.insert(s, "S".into());
    g.rule_names.insert(a_nt, "A".into());
    g.rules.insert(
        s,
        vec![
            Rule {
                lhs: s,
                rhs: vec![Symbol::NonTerminal(a_nt), Symbol::Terminal(a)],
                precedence: None,
                associativity: None,
                fields: vec![],
                production_id: ProductionId(0),
            },
            Rule {
                lhs: s,
                rhs: vec![Symbol::NonTerminal(a_nt), Symbol::Terminal(b)],
                precedence: None,
                associativity: None,
                fields: vec![],
                production_id: ProductionId(1),
            },
        ],
    );
    g.rules.insert(
        a_nt,
        vec![Rule {
            lhs: a_nt,
            rhs: vec![Symbol::Terminal(c)],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(2),
        }],
    );

    let ff = FirstFollowSets::compute(&g).unwrap();

    let mut set = ItemSet::new(StateId(0));
    set.add_item(LRItem::new(RuleId(0), 0, SymbolId(0))); // S -> • A a , $
    set.add_item(LRItem::new(RuleId(1), 0, SymbolId(0))); // S -> • A b , $
    set.closure(&g, &ff).unwrap();

    // A -> • c should appear with lookaheads 'a' and 'b'
    let a_items: Vec<_> = set
        .items
        .iter()
        .filter(|i| i.rule_id == RuleId(2) && i.position == 0)
        .collect();
    assert!(
        a_items.len() >= 2,
        "A -> • c should appear with at least 2 lookaheads, got {}",
        a_items.len()
    );
}

// ---- Reduce item detection ----

#[test]
fn reduce_item_at_end_of_rule() {
    let g = simple_grammar();
    // S -> a •  (position 1, past the only symbol)
    let item = LRItem::new(RuleId(0), 1, SymbolId(0));
    assert!(item.is_reduce_item(&g), "dot at end should be reduce item");
}

#[test]
fn non_reduce_item_at_start() {
    let g = simple_grammar();
    let item = LRItem::new(RuleId(0), 0, SymbolId(0));
    assert!(!item.is_reduce_item(&g), "dot at start should not be reduce item");
}

// ---- next_symbol ----

#[test]
fn next_symbol_at_start() {
    let g = simple_grammar();
    let item = LRItem::new(RuleId(0), 0, SymbolId(0));
    let sym = item.next_symbol(&g);
    assert_eq!(sym, Some(&Symbol::Terminal(SymbolId(1))));
}

#[test]
fn next_symbol_at_end_is_none() {
    let g = simple_grammar();
    let item = LRItem::new(RuleId(0), 1, SymbolId(0));
    assert!(item.next_symbol(&g).is_none());
}

// ---- Canonical collection ----

#[test]
fn canonical_collection_simple_grammar() {
    let g = simple_grammar();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let collection = ItemSetCollection::build_canonical_collection(&g, &ff);

    // Simple grammar S -> a should produce a small number of states
    assert!(
        collection.sets.len() >= 2,
        "should have at least initial and accept states, got {}",
        collection.sets.len()
    );
}

#[test]
fn canonical_collection_states_have_unique_ids() {
    let g = left_recursive_grammar();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let collection = ItemSetCollection::build_canonical_collection(&g, &ff);

    let ids: Vec<_> = collection.sets.iter().map(|s| s.id).collect();
    let unique: BTreeSet<_> = ids.iter().collect();
    assert_eq!(ids.len(), unique.len(), "all state IDs should be unique");
}

#[test]
fn canonical_collection_goto_table_populated() {
    let g = two_nonterminal_grammar();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let collection = ItemSetCollection::build_canonical_collection(&g, &ff);

    assert!(
        !collection.goto_table.is_empty(),
        "GOTO table should have entries for a multi-rule grammar"
    );
}
