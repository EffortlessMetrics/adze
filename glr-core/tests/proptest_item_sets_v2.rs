//! Property-based tests for GLR item set properties (v2).
//!
//! Run with: `cargo test -p adze-glr-core --test proptest_item_sets_v2`

use adze_glr_core::{FirstFollowSets, ItemSetCollection};
use adze_ir::builder::GrammarBuilder;
use adze_ir::*;
use proptest::prelude::*;
use std::collections::BTreeSet;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

const NT_BASE: u16 = 10;
const MAX_TERM: u16 = 8;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

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

fn make_rule(lhs: SymbolId, rhs: Vec<Symbol>, prod: u16) -> Rule {
    Rule {
        lhs,
        rhs,
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(prod),
    }
}

fn build_collection(grammar: &mut Grammar) -> (ItemSetCollection, FirstFollowSets) {
    let ff = FirstFollowSets::compute_normalized(grammar).expect("FIRST/FOLLOW should succeed");
    let col = ItemSetCollection::build_canonical_collection(grammar, &ff);
    (col, ff)
}

// ---------------------------------------------------------------------------
// Concrete grammar builders
// ---------------------------------------------------------------------------

fn single_token_grammar() -> Grammar {
    let mut g = Grammar::new("single".into());
    let a = SymbolId(1);
    let s = SymbolId(NT_BASE);
    tok(&mut g, a, "a", "a");
    g.rule_names.insert(s, "S".into());
    g.rules
        .entry(s)
        .or_default()
        .push(make_rule(s, vec![Symbol::Terminal(a)], 0));
    g
}

fn two_alt_grammar() -> Grammar {
    let mut g = Grammar::new("two_alt".into());
    let a = SymbolId(1);
    let b = SymbolId(2);
    let s = SymbolId(NT_BASE);
    tok(&mut g, a, "a", "a");
    tok(&mut g, b, "b", "b");
    g.rule_names.insert(s, "S".into());
    g.rules
        .entry(s)
        .or_default()
        .push(make_rule(s, vec![Symbol::Terminal(a)], 0));
    g.rules
        .entry(s)
        .or_default()
        .push(make_rule(s, vec![Symbol::Terminal(b)], 1));
    g
}

fn chain_grammar() -> Grammar {
    let mut g = Grammar::new("chain".into());
    let a = SymbolId(1);
    let s = SymbolId(NT_BASE);
    let t = SymbolId(NT_BASE + 1);
    tok(&mut g, a, "a", "a");
    g.rule_names.insert(s, "S".into());
    g.rule_names.insert(t, "T".into());
    g.rules
        .entry(s)
        .or_default()
        .push(make_rule(s, vec![Symbol::NonTerminal(t)], 0));
    g.rules
        .entry(t)
        .or_default()
        .push(make_rule(t, vec![Symbol::Terminal(a)], 1));
    g
}

fn left_recursive_grammar() -> Grammar {
    let mut g = Grammar::new("left_rec".into());
    let a = SymbolId(1);
    let plus = SymbolId(2);
    let e = SymbolId(NT_BASE);
    tok(&mut g, a, "a", "a");
    tok(&mut g, plus, "plus", "+");
    g.rule_names.insert(e, "E".into());
    g.rules.entry(e).or_default().push(make_rule(
        e,
        vec![
            Symbol::NonTerminal(e),
            Symbol::Terminal(plus),
            Symbol::Terminal(a),
        ],
        0,
    ));
    g.rules
        .entry(e)
        .or_default()
        .push(make_rule(e, vec![Symbol::Terminal(a)], 1));
    g
}

fn right_recursive_grammar() -> Grammar {
    let mut g = Grammar::new("right_rec".into());
    let a = SymbolId(1);
    let s = SymbolId(NT_BASE);
    tok(&mut g, a, "a", "a");
    g.rule_names.insert(s, "S".into());
    g.rules.entry(s).or_default().push(make_rule(
        s,
        vec![Symbol::Terminal(a), Symbol::NonTerminal(s)],
        0,
    ));
    g.rules
        .entry(s)
        .or_default()
        .push(make_rule(s, vec![Symbol::Terminal(a)], 1));
    g
}

fn multi_nonterminal_grammar() -> Grammar {
    let mut g = Grammar::new("multi_nt".into());
    let a = SymbolId(1);
    let b = SymbolId(2);
    let s = SymbolId(NT_BASE);
    let x = SymbolId(NT_BASE + 1);
    let y = SymbolId(NT_BASE + 2);
    tok(&mut g, a, "a", "a");
    tok(&mut g, b, "b", "b");
    g.rule_names.insert(s, "S".into());
    g.rule_names.insert(x, "X".into());
    g.rule_names.insert(y, "Y".into());
    g.rules
        .entry(s)
        .or_default()
        .push(make_rule(s, vec![Symbol::NonTerminal(x)], 0));
    g.rules
        .entry(s)
        .or_default()
        .push(make_rule(s, vec![Symbol::NonTerminal(y)], 1));
    g.rules
        .entry(x)
        .or_default()
        .push(make_rule(x, vec![Symbol::Terminal(a)], 2));
    g.rules
        .entry(y)
        .or_default()
        .push(make_rule(y, vec![Symbol::Terminal(b)], 3));
    g
}

fn expression_grammar() -> Grammar {
    let mut g = Grammar::new("expr".into());
    let num = SymbolId(1);
    let plus = SymbolId(2);
    let star = SymbolId(3);
    let lparen = SymbolId(4);
    let rparen = SymbolId(5);
    let e = SymbolId(NT_BASE);
    let t = SymbolId(NT_BASE + 1);
    let f = SymbolId(NT_BASE + 2);
    tok(&mut g, num, "num", "[0-9]+");
    tok(&mut g, plus, "plus", "\\+");
    tok(&mut g, star, "star", "\\*");
    tok(&mut g, lparen, "lparen", "\\(");
    tok(&mut g, rparen, "rparen", "\\)");
    g.rule_names.insert(e, "E".into());
    g.rule_names.insert(t, "T".into());
    g.rule_names.insert(f, "F".into());
    // E → E + T | T
    g.rules.entry(e).or_default().push(make_rule(
        e,
        vec![
            Symbol::NonTerminal(e),
            Symbol::Terminal(plus),
            Symbol::NonTerminal(t),
        ],
        0,
    ));
    g.rules
        .entry(e)
        .or_default()
        .push(make_rule(e, vec![Symbol::NonTerminal(t)], 1));
    // T → T * F | F
    g.rules.entry(t).or_default().push(make_rule(
        t,
        vec![
            Symbol::NonTerminal(t),
            Symbol::Terminal(star),
            Symbol::NonTerminal(f),
        ],
        2,
    ));
    g.rules
        .entry(t)
        .or_default()
        .push(make_rule(t, vec![Symbol::NonTerminal(f)], 3));
    // F → ( E ) | num
    g.rules.entry(f).or_default().push(make_rule(
        f,
        vec![
            Symbol::Terminal(lparen),
            Symbol::NonTerminal(e),
            Symbol::Terminal(rparen),
        ],
        4,
    ));
    g.rules
        .entry(f)
        .or_default()
        .push(make_rule(f, vec![Symbol::Terminal(num)], 5));
    g
}

// ---------------------------------------------------------------------------
// Proptest strategies
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
        let prods = prop::collection::vec(prop::collection::vec(arb_rhs(), 1..=3), nn..=nn);
        prods.prop_map(move |all_prods| {
            let mut g = Grammar::new("random".into());
            // Register terminals
            for i in 1..=(nt as u16) {
                let id = SymbolId(i);
                tok(&mut g, id, &format!("t{i}"), &format!("t{i}"));
            }
            // Register nonterminals and rules
            let mut prod_counter = 0u16;
            for (ni, prods_for_nt) in all_prods.iter().enumerate() {
                let nt_id = SymbolId(NT_BASE + ni as u16);
                g.rule_names.insert(nt_id, format!("N{ni}"));
                for rhs in prods_for_nt {
                    let filtered: Vec<Symbol> = rhs
                        .iter()
                        .filter(|sym| match sym {
                            Symbol::Terminal(id) => id.0 >= 1 && id.0 <= nt as u16,
                            Symbol::NonTerminal(id) => {
                                id.0 >= NT_BASE && id.0 < NT_BASE + nn as u16
                            }
                            _ => false,
                        })
                        .cloned()
                        .collect();
                    if !filtered.is_empty() {
                        g.rules.entry(nt_id).or_default().push(make_rule(
                            nt_id,
                            filtered,
                            prod_counter,
                        ));
                        prod_counter += 1;
                    }
                }
                // Ensure at least one terminal production
                if g.rules.get(&nt_id).is_none_or(|r| r.is_empty()) {
                    let tid = SymbolId(1);
                    g.rules.entry(nt_id).or_default().push(make_rule(
                        nt_id,
                        vec![Symbol::Terminal(tid)],
                        prod_counter,
                    ));
                    prod_counter += 1;
                }
            }
            g
        })
    })
}

// =========================================================================
// 1. State count ≥ 1 proptest (5 tests)
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    #[test]
    fn state_count_at_least_one_random(mut g in arb_small_grammar()) {
        let (col, _) = build_collection(&mut g);
        prop_assert!(!col.sets.is_empty(), "canonical collection must have ≥1 state");
    }

    #[test]
    fn state_count_at_least_one_random_v2(mut g in arb_small_grammar()) {
        let (col, _) = build_collection(&mut g);
        prop_assert!(!col.sets.is_empty(), "must have at least the initial state");
    }

    #[test]
    fn state_count_positive_with_terminals(
        term_count in 1..=4u16,
    ) {
        let mut g = Grammar::new("multi_tok".into());
        for i in 1..=term_count {
            let id = SymbolId(i);
            tok(&mut g, id, &format!("t{i}"), &format!("t{i}"));
        }
        let s = SymbolId(NT_BASE);
        g.rule_names.insert(s, "S".into());
        for i in 1..=term_count {
            g.rules.entry(s).or_default().push(make_rule(
                s,
                vec![Symbol::Terminal(SymbolId(i))],
                i - 1,
            ));
        }
        let (col, _) = build_collection(&mut g);
        prop_assert!(!col.sets.is_empty());
    }

    #[test]
    fn state_count_nonzero_chain(depth in 1..=4usize) {
        let mut g = Grammar::new("chain_depth".into());
        let a = SymbolId(1);
        tok(&mut g, a, "a", "a");
        for d in 0..depth {
            let nt = SymbolId(NT_BASE + d as u16);
            g.rule_names.insert(nt, format!("N{d}"));
            if d + 1 < depth {
                let next = SymbolId(NT_BASE + (d + 1) as u16);
                g.rules.entry(nt).or_default().push(make_rule(
                    nt,
                    vec![Symbol::NonTerminal(next)],
                    d as u16,
                ));
            } else {
                g.rules.entry(nt).or_default().push(make_rule(
                    nt,
                    vec![Symbol::Terminal(a)],
                    d as u16,
                ));
            }
        }
        let (col, _) = build_collection(&mut g);
        prop_assert!(!col.sets.is_empty());
    }

    #[test]
    fn state_count_nonzero_builder(
        alt_count in 1..=5usize,
    ) {
        let mut builder = GrammarBuilder::new("dyn");
        for i in 0..alt_count {
            builder = builder.token(&format!("t{i}"), &format!("t{i}"));
        }
        for i in 0..alt_count {
            builder = builder.rule("S", vec![&format!("t{}", i)]);
        }
        let mut g = builder.start("S").build();
        let (col, _) = build_collection(&mut g);
        prop_assert!(!col.sets.is_empty());
    }
}

// =========================================================================
// 2. Collection determinism proptest (5 tests)
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(60))]

    #[test]
    fn determinism_state_count(mut g in arb_small_grammar()) {
        let mut g2 = g.clone();
        let (col1, _) = build_collection(&mut g);
        let (col2, _) = build_collection(&mut g2);
        prop_assert_eq!(col1.sets.len(), col2.sets.len());
    }

    #[test]
    fn determinism_goto_count(mut g in arb_small_grammar()) {
        let mut g2 = g.clone();
        let (col1, _) = build_collection(&mut g);
        let (col2, _) = build_collection(&mut g2);
        prop_assert_eq!(col1.goto_table.len(), col2.goto_table.len());
    }

    #[test]
    fn determinism_item_count_per_state(mut g in arb_small_grammar()) {
        let mut g2 = g.clone();
        let (col1, _) = build_collection(&mut g);
        let (col2, _) = build_collection(&mut g2);
        let counts1: Vec<usize> = col1.sets.iter().map(|s| s.items.len()).collect();
        let counts2: Vec<usize> = col2.sets.iter().map(|s| s.items.len()).collect();
        prop_assert_eq!(counts1, counts2);
    }

    #[test]
    fn determinism_goto_keys(mut g in arb_small_grammar()) {
        let mut g2 = g.clone();
        let (col1, _) = build_collection(&mut g);
        let (col2, _) = build_collection(&mut g2);
        let keys1: BTreeSet<_> = col1.goto_table.keys().collect();
        let keys2: BTreeSet<_> = col2.goto_table.keys().collect();
        prop_assert_eq!(keys1, keys2);
    }

    #[test]
    fn determinism_state_ids(mut g in arb_small_grammar()) {
        let mut g2 = g.clone();
        let (col1, _) = build_collection(&mut g);
        let (col2, _) = build_collection(&mut g2);
        let ids1: Vec<StateId> = col1.sets.iter().map(|s| s.id).collect();
        let ids2: Vec<StateId> = col2.sets.iter().map(|s| s.id).collect();
        prop_assert_eq!(ids1, ids2);
    }
}

// =========================================================================
// 3. More rules → more states proptest (5 tests)
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(60))]

    #[test]
    fn more_alts_weakly_more_states(extra_alts in 1..=4u16) {
        // Base: S → a
        let mut g1 = Grammar::new("base".into());
        let a = SymbolId(1);
        tok(&mut g1, a, "a", "a");
        let s = SymbolId(NT_BASE);
        g1.rule_names.insert(s, "S".into());
        g1.rules.entry(s).or_default().push(make_rule(
            s, vec![Symbol::Terminal(a)], 0,
        ));
        let (col1, _) = build_collection(&mut g1);

        // Extended: S → a | t2 | t3 | ...
        let mut g2 = g1.clone();
        for i in 1..=extra_alts {
            let tid = SymbolId(1 + i);
            tok(&mut g2, tid, &format!("t{}", 1 + i), &format!("t{}", 1 + i));
            g2.rules.entry(s).or_default().push(make_rule(
                s, vec![Symbol::Terminal(tid)], i,
            ));
        }
        let (col2, _) = build_collection(&mut g2);
        prop_assert!(col2.sets.len() >= col1.sets.len());
    }

    #[test]
    fn deeper_chain_weakly_more_states(depth in 1..=4usize) {
        let mut g_shallow = Grammar::new("shallow".into());
        let a = SymbolId(1);
        tok(&mut g_shallow, a, "a", "a");
        let s = SymbolId(NT_BASE);
        g_shallow.rule_names.insert(s, "S".into());
        g_shallow.rules.entry(s).or_default().push(make_rule(
            s, vec![Symbol::Terminal(a)], 0,
        ));
        let (col_shallow, _) = build_collection(&mut g_shallow);

        // Build chain of depth `depth`: S → N1, N1 → N2, ..., Nn → a
        let mut g_deep = Grammar::new("deep".into());
        tok(&mut g_deep, a, "a", "a");
        for d in 0..=depth {
            let nt = SymbolId(NT_BASE + d as u16);
            g_deep.rule_names.insert(nt, format!("N{d}"));
            if d < depth {
                let next = SymbolId(NT_BASE + (d + 1) as u16);
                g_deep.rules.entry(nt).or_default().push(make_rule(
                    nt, vec![Symbol::NonTerminal(next)], d as u16,
                ));
            } else {
                g_deep.rules.entry(nt).or_default().push(make_rule(
                    nt, vec![Symbol::Terminal(a)], d as u16,
                ));
            }
        }
        let (col_deep, _) = build_collection(&mut g_deep);
        prop_assert!(col_deep.sets.len() >= col_shallow.sets.len());
    }

    #[test]
    fn extra_nonterminal_weakly_more_states(extra in 1..=3u16) {
        let a = SymbolId(1);
        let b = SymbolId(2);
        let s = SymbolId(NT_BASE);

        let mut g1 = Grammar::new("base_nt".into());
        tok(&mut g1, a, "a", "a");
        tok(&mut g1, b, "b", "b");
        g1.rule_names.insert(s, "S".into());
        g1.rules.entry(s).or_default().push(make_rule(
            s, vec![Symbol::Terminal(a)], 0,
        ));
        let (col1, _) = build_collection(&mut g1);

        let mut g2 = g1.clone();
        for i in 1..=extra {
            let nt = SymbolId(NT_BASE + i);
            g2.rule_names.insert(nt, format!("X{i}"));
            g2.rules.entry(nt).or_default().push(make_rule(
                nt, vec![Symbol::Terminal(b)], i,
            ));
            g2.rules.entry(s).or_default().push(make_rule(
                s, vec![Symbol::NonTerminal(nt)], extra + i,
            ));
        }
        let (col2, _) = build_collection(&mut g2);
        prop_assert!(col2.sets.len() >= col1.sets.len());
    }

    #[test]
    fn longer_rhs_weakly_more_states(rhs_len in 1..=4usize) {
        let mut g = Grammar::new("rhs_len".into());
        let a = SymbolId(1);
        tok(&mut g, a, "a", "a");
        let s = SymbolId(NT_BASE);
        g.rule_names.insert(s, "S".into());
        let rhs: Vec<Symbol> = (0..rhs_len).map(|_| Symbol::Terminal(a)).collect();
        g.rules.entry(s).or_default().push(make_rule(s, rhs, 0));
        let (col, _) = build_collection(&mut g);
        // States = at least rhs_len + 1 (initial state + one per dot position after shift)
        prop_assert!(col.sets.len() > rhs_len);
    }

    #[test]
    fn builder_more_rules_weakly_more_states(extra in 1..=3usize) {
        let b1 = GrammarBuilder::new("b1")
            .token("a", "a")
            .rule("S", vec!["a"])
            .start("S");
        let mut g1 = b1.build();
        let (col1, _) = build_collection(&mut g1);

        let mut b2 = GrammarBuilder::new("b2").token("a", "a");
        for i in 0..=extra {
            let tname = format!("t{i}");
            b2 = b2.token(&tname, &tname);
        }
        for i in 0..=extra {
            let tname = format!("t{i}");
            b2 = b2.rule("S", vec![&tname]);
        }
        b2 = b2.rule("S", vec!["a"]);
        let mut g2 = b2.start("S").build();
        let (col2, _) = build_collection(&mut g2);
        prop_assert!(col2.sets.len() >= col1.sets.len());
    }
}

// =========================================================================
// 4. Canonical collection properties (5 proptest)
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(60))]

    #[test]
    fn goto_targets_are_valid_states(mut g in arb_small_grammar()) {
        let (col, _) = build_collection(&mut g);
        let state_ids: BTreeSet<StateId> = col.sets.iter().map(|s| s.id).collect();
        for (_, tgt) in &col.goto_table {
            prop_assert!(state_ids.contains(tgt), "goto target must be a known state");
        }
    }

    #[test]
    fn goto_sources_are_valid_states(mut g in arb_small_grammar()) {
        let (col, _) = build_collection(&mut g);
        let state_ids: BTreeSet<StateId> = col.sets.iter().map(|s| s.id).collect();
        for ((src, _), _) in &col.goto_table {
            prop_assert!(state_ids.contains(src), "goto source must be a known state");
        }
    }

    #[test]
    fn state_ids_are_sequential(mut g in arb_small_grammar()) {
        let (col, _) = build_collection(&mut g);
        for (i, set) in col.sets.iter().enumerate() {
            prop_assert_eq!(set.id, StateId(i as u16), "state IDs must be sequential");
        }
    }

    #[test]
    fn every_state_has_items(mut g in arb_small_grammar()) {
        let (col, _) = build_collection(&mut g);
        for set in &col.sets {
            prop_assert!(!set.items.is_empty(), "every state must have at least one item");
        }
    }

    #[test]
    fn initial_state_exists(mut g in arb_small_grammar()) {
        let (col, _) = build_collection(&mut g);
        prop_assert!(
            col.sets.iter().any(|s| s.id == StateId(0)),
            "state 0 must exist"
        );
    }
}

// =========================================================================
// 5. Regular item set tests (10 tests)
// =========================================================================

#[test]
fn single_token_grammar_has_states() {
    let mut g = single_token_grammar();
    let (col, _) = build_collection(&mut g);
    assert!(!col.sets.is_empty());
}

#[test]
fn single_token_grammar_goto_nonempty() {
    let mut g = single_token_grammar();
    let (col, _) = build_collection(&mut g);
    assert!(!col.goto_table.is_empty(), "S→a must have goto entries");
}

#[test]
fn two_alt_grammar_has_multiple_states() {
    let mut g = two_alt_grammar();
    let (col, _) = build_collection(&mut g);
    assert!(col.sets.len() >= 3, "S→a|b needs ≥3 states");
}

#[test]
fn chain_grammar_states() {
    let mut g = chain_grammar();
    let (col, _) = build_collection(&mut g);
    assert!(col.sets.len() >= 3, "S→T, T→a needs ≥3 states");
}

#[test]
fn left_recursive_grammar_states() {
    let mut g = left_recursive_grammar();
    let (col, _) = build_collection(&mut g);
    assert!(col.sets.len() >= 4, "E→E+a|a needs ≥4 states");
}

#[test]
fn right_recursive_grammar_states() {
    let mut g = right_recursive_grammar();
    let (col, _) = build_collection(&mut g);
    assert!(col.sets.len() >= 3, "S→aS|a needs ≥3 states");
}

#[test]
fn multi_nonterminal_grammar_states() {
    let mut g = multi_nonterminal_grammar();
    let (col, _) = build_collection(&mut g);
    assert!(col.sets.len() >= 4);
}

#[test]
fn goto_table_targets_match_states() {
    let mut g = left_recursive_grammar();
    let (col, _) = build_collection(&mut g);
    let state_ids: BTreeSet<_> = col.sets.iter().map(|s| s.id).collect();
    for (_, tgt) in &col.goto_table {
        assert!(state_ids.contains(tgt), "goto target must be a known state");
    }
}

#[test]
fn all_items_have_valid_positions() {
    let mut g = expression_grammar();
    let (col, _) = build_collection(&mut g);
    for set in &col.sets {
        for item in &set.items {
            // position must be reasonable (≤ some max rhs length)
            assert!(item.position <= 20, "position must be bounded");
        }
    }
}

#[test]
fn determinism_fixed_grammar() {
    let mut g1 = left_recursive_grammar();
    let mut g2 = left_recursive_grammar();
    let (col1, _) = build_collection(&mut g1);
    let (col2, _) = build_collection(&mut g2);
    assert_eq!(col1.sets.len(), col2.sets.len());
    assert_eq!(col1.goto_table.len(), col2.goto_table.len());
}

// =========================================================================
// 6. Item set for expression grammars (5 tests)
// =========================================================================

#[test]
fn expression_grammar_has_many_states() {
    let mut g = expression_grammar();
    let (col, _) = build_collection(&mut g);
    // Classic expression grammar: E→E+T|T, T→T*F|F, F→(E)|n
    // Typically produces ~12 states
    assert!(
        col.sets.len() >= 10,
        "expression grammar should have ≥10 states, got {}",
        col.sets.len()
    );
}

#[test]
fn expression_grammar_goto_entries() {
    let mut g = expression_grammar();
    let (col, _) = build_collection(&mut g);
    assert!(
        col.goto_table.len() >= 10,
        "expression grammar goto table should have ≥10 entries, got {}",
        col.goto_table.len()
    );
}

#[test]
fn expression_grammar_terminal_tracking() {
    let mut g = expression_grammar();
    let (col, _) = build_collection(&mut g);
    // symbol_is_terminal should classify terminals and nonterminals
    for (sym, is_term) in &col.symbol_is_terminal {
        if sym.0 < NT_BASE {
            assert!(is_term, "symbol {} should be terminal", sym.0);
        } else {
            assert!(!is_term, "symbol {} should be nonterminal", sym.0);
        }
    }
}

#[test]
fn expression_grammar_sequential_ids() {
    let mut g = expression_grammar();
    let (col, _) = build_collection(&mut g);
    for (i, set) in col.sets.iter().enumerate() {
        assert_eq!(set.id, StateId(i as u16));
    }
}

#[test]
fn expression_grammar_no_empty_states() {
    let mut g = expression_grammar();
    let (col, _) = build_collection(&mut g);
    for set in &col.sets {
        assert!(!set.items.is_empty(), "state {} has no items", set.id.0);
    }
}

// =========================================================================
// 7. Collection size bounds (5 tests)
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(40))]

    #[test]
    fn state_count_bounded_above(mut g in arb_small_grammar()) {
        let (col, _) = build_collection(&mut g);
        // For small random grammars, state count should be bounded
        prop_assert!(col.sets.len() <= 500, "state count should be reasonable");
    }

    #[test]
    fn goto_count_bounded_by_states_times_symbols(mut g in arb_small_grammar()) {
        let (col, _) = build_collection(&mut g);
        let n_states = col.sets.len();
        let n_symbols = col.symbol_is_terminal.len();
        prop_assert!(
            col.goto_table.len() <= n_states * n_symbols,
            "goto entries ≤ states × symbols"
        );
    }

    #[test]
    fn item_count_per_state_bounded(mut g in arb_small_grammar()) {
        let (col, _) = build_collection(&mut g);
        for set in &col.sets {
            prop_assert!(
                set.items.len() <= 1000,
                "item count per state should be bounded"
            );
        }
    }

    #[test]
    fn symbol_is_terminal_covers_goto_symbols(mut g in arb_small_grammar()) {
        let (col, _) = build_collection(&mut g);
        for ((_, sym), _) in &col.goto_table {
            prop_assert!(
                col.symbol_is_terminal.contains_key(sym),
                "goto symbol {} must be in symbol_is_terminal",
                sym.0
            );
        }
    }

    #[test]
    fn total_items_bounded(mut g in arb_small_grammar()) {
        let (col, _) = build_collection(&mut g);
        let total: usize = col.sets.iter().map(|s| s.items.len()).sum();
        prop_assert!(total <= 10000, "total items across all states should be bounded");
    }
}

// =========================================================================
// 8. Edge cases (10 tests)
// =========================================================================

#[test]
fn single_terminal_single_rule() {
    let mut g = GrammarBuilder::new("minimal")
        .token("x", "x")
        .rule("S", vec!["x"])
        .start("S")
        .build();
    let (col, _) = build_collection(&mut g);
    assert!(!col.sets.is_empty());
    assert!(!col.goto_table.is_empty());
}

#[test]
fn many_alternatives_same_lhs() {
    let mut b = GrammarBuilder::new("many_alts");
    for i in 0..8 {
        let name = format!("t{i}");
        b = b.token(&name, &name);
    }
    for i in 0..8 {
        let name = format!("t{i}");
        b = b.rule("S", vec![&name]);
    }
    let mut g = b.start("S").build();
    let (col, _) = build_collection(&mut g);
    // 8 alternatives: initial state + 8 shift states + accept ≥ 10
    assert!(
        col.sets.len() >= 9,
        "8 alts should produce ≥9 states, got {}",
        col.sets.len()
    );
}

#[test]
fn long_rhs_single_rule() {
    let mut g = Grammar::new("long_rhs".into());
    let a = SymbolId(1);
    tok(&mut g, a, "a", "a");
    let s = SymbolId(NT_BASE);
    g.rule_names.insert(s, "S".into());
    let rhs: Vec<Symbol> = (0..6).map(|_| Symbol::Terminal(a)).collect();
    g.rules.entry(s).or_default().push(make_rule(s, rhs, 0));
    let (col, _) = build_collection(&mut g);
    // S → a a a a a a produces at least 7 states (one per dot position + initial)
    assert!(
        col.sets.len() >= 7,
        "long RHS should produce ≥7 states, got {}",
        col.sets.len()
    );
}

#[test]
fn self_recursive_rule() {
    let mut g = Grammar::new("self_rec".into());
    let a = SymbolId(1);
    let s = SymbolId(NT_BASE);
    tok(&mut g, a, "a", "a");
    g.rule_names.insert(s, "S".into());
    // S → S a | a (left recursive)
    g.rules.entry(s).or_default().push(make_rule(
        s,
        vec![Symbol::NonTerminal(s), Symbol::Terminal(a)],
        0,
    ));
    g.rules
        .entry(s)
        .or_default()
        .push(make_rule(s, vec![Symbol::Terminal(a)], 1));
    let (col, _) = build_collection(&mut g);
    assert!(col.sets.len() >= 3);
}

#[test]
fn mutual_recursion() {
    let mut g = Grammar::new("mutual".into());
    let a = SymbolId(1);
    let b = SymbolId(2);
    let x = SymbolId(NT_BASE);
    let y = SymbolId(NT_BASE + 1);
    tok(&mut g, a, "a", "a");
    tok(&mut g, b, "b", "b");
    g.rule_names.insert(x, "X".into());
    g.rule_names.insert(y, "Y".into());
    // X → a Y | a
    g.rules.entry(x).or_default().push(make_rule(
        x,
        vec![Symbol::Terminal(a), Symbol::NonTerminal(y)],
        0,
    ));
    g.rules
        .entry(x)
        .or_default()
        .push(make_rule(x, vec![Symbol::Terminal(a)], 1));
    // Y → b X | b
    g.rules.entry(y).or_default().push(make_rule(
        y,
        vec![Symbol::Terminal(b), Symbol::NonTerminal(x)],
        2,
    ));
    g.rules
        .entry(y)
        .or_default()
        .push(make_rule(y, vec![Symbol::Terminal(b)], 3));
    let (col, _) = build_collection(&mut g);
    assert!(col.sets.len() >= 4);
}

#[test]
fn duplicate_rules_same_grammar() {
    let mut g = Grammar::new("dup".into());
    let a = SymbolId(1);
    let s = SymbolId(NT_BASE);
    tok(&mut g, a, "a", "a");
    g.rule_names.insert(s, "S".into());
    // Two identical rules
    g.rules
        .entry(s)
        .or_default()
        .push(make_rule(s, vec![Symbol::Terminal(a)], 0));
    g.rules
        .entry(s)
        .or_default()
        .push(make_rule(s, vec![Symbol::Terminal(a)], 1));
    let (col, _) = build_collection(&mut g);
    assert!(!col.sets.is_empty());
}

#[test]
fn builder_grammar_has_goto() {
    let mut g = GrammarBuilder::new("builder_test")
        .token("a", "a")
        .token("b", "b")
        .rule("S", vec!["a"])
        .rule("S", vec!["b"])
        .start("S")
        .build();
    let (col, _) = build_collection(&mut g);
    assert!(!col.goto_table.is_empty());
}

#[test]
fn three_level_chain() {
    let mut g = Grammar::new("chain3".into());
    let a = SymbolId(1);
    let s = SymbolId(NT_BASE);
    let m = SymbolId(NT_BASE + 1);
    let leaf = SymbolId(NT_BASE + 2);
    tok(&mut g, a, "a", "a");
    g.rule_names.insert(s, "S".into());
    g.rule_names.insert(m, "M".into());
    g.rule_names.insert(leaf, "L".into());
    g.rules
        .entry(s)
        .or_default()
        .push(make_rule(s, vec![Symbol::NonTerminal(m)], 0));
    g.rules
        .entry(m)
        .or_default()
        .push(make_rule(m, vec![Symbol::NonTerminal(leaf)], 1));
    g.rules
        .entry(leaf)
        .or_default()
        .push(make_rule(leaf, vec![Symbol::Terminal(a)], 2));
    let (col, _) = build_collection(&mut g);
    assert!(
        col.sets.len() >= 4,
        "3-level chain needs ≥4 states, got {}",
        col.sets.len()
    );
}

#[test]
fn mixed_terminal_nonterminal_rhs() {
    let mut g = Grammar::new("mixed".into());
    let a = SymbolId(1);
    let b = SymbolId(2);
    let s = SymbolId(NT_BASE);
    let t = SymbolId(NT_BASE + 1);
    tok(&mut g, a, "a", "a");
    tok(&mut g, b, "b", "b");
    g.rule_names.insert(s, "S".into());
    g.rule_names.insert(t, "T".into());
    // S → a T b
    g.rules.entry(s).or_default().push(make_rule(
        s,
        vec![
            Symbol::Terminal(a),
            Symbol::NonTerminal(t),
            Symbol::Terminal(b),
        ],
        0,
    ));
    // T → a
    g.rules
        .entry(t)
        .or_default()
        .push(make_rule(t, vec![Symbol::Terminal(a)], 1));
    let (col, _) = build_collection(&mut g);
    assert!(col.sets.len() >= 4);
}

#[test]
fn goto_entry_count_grows_with_grammar_size() {
    let mut g_small = single_token_grammar();
    let (col_small, _) = build_collection(&mut g_small);

    let mut g_big = expression_grammar();
    let (col_big, _) = build_collection(&mut g_big);

    assert!(
        col_big.goto_table.len() >= col_small.goto_table.len(),
        "bigger grammar should have ≥ goto entries"
    );
}
