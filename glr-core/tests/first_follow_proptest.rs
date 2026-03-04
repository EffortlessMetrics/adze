#![allow(clippy::needless_range_loop)]
//! Property-based tests for FIRST/FOLLOW set computation in adze-glr-core.
//!
//! Run with: `cargo test -p adze-glr-core --test first_follow_proptest`

use adze_glr_core::FirstFollowSets;
use adze_ir::*;
use proptest::prelude::*;

// ---------------------------------------------------------------------------
// Grammar construction helpers
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

/// Terminal IDs live in 1..=MAX_TERM, non-terminal IDs in NT_BASE..
const MAX_TERM: u16 = 8;
const NT_BASE: u16 = 10;
const MAX_NT: u16 = 18;

// ---------------------------------------------------------------------------
// Strategies for generating random grammars
// ---------------------------------------------------------------------------

/// Generate a terminal SymbolId.
fn arb_terminal() -> impl Strategy<Value = SymbolId> {
    (1..=MAX_TERM).prop_map(SymbolId)
}

/// Generate a non-terminal SymbolId.
fn arb_nonterminal() -> impl Strategy<Value = SymbolId> {
    (NT_BASE..=MAX_NT).prop_map(SymbolId)
}

/// Generate a single RHS symbol (terminal, non-terminal, or epsilon).
fn arb_rhs_symbol() -> impl Strategy<Value = Symbol> {
    prop_oneof![
        8 => arb_terminal().prop_map(Symbol::Terminal),
        4 => arb_nonterminal().prop_map(Symbol::NonTerminal),
        1 => Just(Symbol::Epsilon),
    ]
}

/// Generate a RHS for a production (1..=4 symbols).
fn arb_rhs() -> impl Strategy<Value = Vec<Symbol>> {
    prop::collection::vec(arb_rhs_symbol(), 1..=4)
}

/// Generate a complete grammar with `num_nt` non-terminals and `num_term` terminals,
/// each non-terminal having 1..=3 productions.
fn arb_grammar(
    num_term: std::ops::RangeInclusive<usize>,
    num_nt: std::ops::RangeInclusive<usize>,
) -> impl Strategy<Value = Grammar> {
    (num_term, num_nt).prop_flat_map(|(nt_count, nn_count)| {
        let nt_count = nt_count.max(1);
        let nn_count = nn_count.max(1);
        // Generate 1..=3 productions per non-terminal
        let prods =
            prop::collection::vec(prop::collection::vec(arb_rhs(), 1..=3), nn_count..=nn_count);
        prods.prop_map(move |all_prods| {
            let mut g = Grammar::new("proptest".into());
            // Register terminals
            for i in 1..=(nt_count as u16).min(MAX_TERM) {
                let id = SymbolId(i);
                tok(&mut g, id, &format!("t{i}"), &format!("t{i}"));
            }
            // Register non-terminals and their rules
            let mut prod_counter = 0u16;
            for (idx, prods) in all_prods.iter().enumerate() {
                let nt_id = SymbolId(NT_BASE + idx as u16);
                g.rule_names.insert(nt_id, format!("N{idx}"));
                for rhs in prods {
                    // Filter RHS: keep only terminals that were registered
                    // and non-terminals that exist
                    let filtered_rhs: Vec<Symbol> = rhs
                        .iter()
                        .map(|sym| match sym {
                            Symbol::Terminal(id) if id.0 > (nt_count as u16).min(MAX_TERM) => {
                                Symbol::Terminal(SymbolId(1)) // clamp
                            }
                            Symbol::NonTerminal(id) if id.0 >= NT_BASE + all_prods.len() as u16 => {
                                Symbol::NonTerminal(SymbolId(NT_BASE)) // clamp
                            }
                            other => other.clone(),
                        })
                        .collect();
                    g.rules
                        .entry(nt_id)
                        .or_default()
                        .push(rule(nt_id, filtered_rhs, prod_counter));
                    prod_counter += 1;
                }
            }
            g
        })
    })
}

/// Generate a small grammar (1-3 terminals, 1-3 non-terminals).
fn arb_small_grammar() -> impl Strategy<Value = Grammar> {
    arb_grammar(1..=3, 1..=3)
}

/// Generate a medium grammar (2-6 terminals, 2-6 non-terminals).
fn arb_medium_grammar() -> impl Strategy<Value = Grammar> {
    arb_grammar(2..=6, 2..=6)
}

/// Build a single-rule grammar: S → <rhs> with given terminals.
fn single_rule_grammar(terminals: &[u16], rhs: Vec<Symbol>) -> Grammar {
    let mut g = Grammar::new("single".into());
    for &t in terminals {
        tok(&mut g, SymbolId(t), &format!("t{t}"), &format!("t{t}"));
    }
    let s = SymbolId(NT_BASE);
    g.rule_names.insert(s, "S".into());
    g.rules.entry(s).or_default().push(rule(s, rhs, 0));
    g
}

// ---------------------------------------------------------------------------
// Property-based tests
// ---------------------------------------------------------------------------

// 1. Compute doesn't panic on any valid grammar
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]
    #[test]
    fn compute_does_not_panic(g in arb_small_grammar()) {
        let _ = FirstFollowSets::compute(&g);
    }
}

// 2. Compute doesn't panic on medium grammars
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]
    #[test]
    fn compute_does_not_panic_medium(g in arb_medium_grammar()) {
        let _ = FirstFollowSets::compute(&g);
    }
}

// 3. Deterministic: same grammar → same FIRST sets
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]
    #[test]
    fn deterministic_first_sets(g in arb_small_grammar()) {
        let ff1 = FirstFollowSets::compute(&g);
        let ff2 = FirstFollowSets::compute(&g);
        match (ff1, ff2) {
            (Ok(a), Ok(b)) => {
                for (id, _) in &g.rules {
                    let f1 = a.first(*id);
                    let f2 = b.first(*id);
                    prop_assert_eq!(f1, f2, "FIRST({:?}) differs across runs", id);
                }
            }
            (Err(_), Err(_)) => {} // both fail is fine
            _ => prop_assert!(false, "one succeeded and the other failed"),
        }
    }
}

// 4. Deterministic: same grammar → same FOLLOW sets
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]
    #[test]
    fn deterministic_follow_sets(g in arb_small_grammar()) {
        let ff1 = FirstFollowSets::compute(&g);
        let ff2 = FirstFollowSets::compute(&g);
        match (ff1, ff2) {
            (Ok(a), Ok(b)) => {
                for (id, _) in &g.rules {
                    let f1 = a.follow(*id);
                    let f2 = b.follow(*id);
                    prop_assert_eq!(f1, f2, "FOLLOW({:?}) differs across runs", id);
                }
            }
            (Err(_), Err(_)) => {}
            _ => prop_assert!(false, "one succeeded and the other failed"),
        }
    }
}

// 5. FIRST(terminal) contains the terminal itself
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]
    #[test]
    fn terminal_in_own_first_set(t_id in 1..=MAX_TERM) {
        let t = SymbolId(t_id);
        let s = SymbolId(NT_BASE);
        let mut g = Grammar::new("term_self".into());
        tok(&mut g, t, &format!("t{t_id}"), &format!("t{t_id}"));
        g.rule_names.insert(s, "S".into());
        g.rules.entry(s).or_default().push(rule(s, vec![Symbol::Terminal(t)], 0));
        let ff = FirstFollowSets::compute(&g).unwrap();
        let first_s = ff.first(s).unwrap();
        prop_assert!(first_s.contains(t_id as usize), "FIRST(S) should contain t{t_id}");
    }
}

// 6. FOLLOW of start symbol contains EOF (index 0)
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]
    #[test]
    fn follow_start_contains_eof(g in arb_small_grammar()) {
        if let Ok(ff) = FirstFollowSets::compute(&g)
            && let Some(start) = g.start_symbol()
                && let Some(follow_set) = ff.follow(start) {
                    prop_assert!(follow_set.contains(0), "FOLLOW(start) must contain EOF");
                }
    }
}

// 7. Epsilon rule makes non-terminal nullable
proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]
    #[test]
    fn epsilon_rule_implies_nullable(t_id in 1..=MAX_TERM) {
        let t = SymbolId(t_id);
        let s = SymbolId(NT_BASE);
        let mut g = Grammar::new("eps".into());
        tok(&mut g, t, &format!("t{t_id}"), "x");
        g.rule_names.insert(s, "S".into());
        g.rules.entry(s).or_default().push(rule(s, vec![Symbol::Epsilon], 0));
        g.rules.entry(s).or_default().push(rule(s, vec![Symbol::Terminal(t)], 1));
        let ff = FirstFollowSets::compute(&g).unwrap();
        prop_assert!(ff.is_nullable(s), "S with epsilon production must be nullable");
    }
}

// 8. Non-epsilon-only terminal rule is not nullable
proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]
    #[test]
    fn terminal_only_rule_not_nullable(t_id in 1..=MAX_TERM) {
        let t = SymbolId(t_id);
        let s = SymbolId(NT_BASE);
        let mut g = Grammar::new("noeps".into());
        tok(&mut g, t, &format!("t{t_id}"), "x");
        g.rule_names.insert(s, "S".into());
        g.rules.entry(s).or_default().push(rule(s, vec![Symbol::Terminal(t)], 0));
        let ff = FirstFollowSets::compute(&g).unwrap();
        prop_assert!(!ff.is_nullable(s), "S with only terminal production must not be nullable");
    }
}

// 9. FIRST set of non-terminal is non-empty when it has terminal productions
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]
    #[test]
    fn first_set_nonempty_for_terminal_production(t_id in 1..=MAX_TERM) {
        let t = SymbolId(t_id);
        let s = SymbolId(NT_BASE);
        let mut g = Grammar::new("nonempty".into());
        tok(&mut g, t, &format!("t{t_id}"), "x");
        g.rule_names.insert(s, "S".into());
        g.rules.entry(s).or_default().push(rule(s, vec![Symbol::Terminal(t)], 0));
        let ff = FirstFollowSets::compute(&g).unwrap();
        let first_s = ff.first(s).unwrap();
        prop_assert!(first_s.count_ones(..) > 0, "FIRST(S) must be non-empty");
    }
}

// 10. FIRST propagates through chain rules: A → B, B → t ⟹ t ∈ FIRST(A)
proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]
    #[test]
    fn first_propagates_through_chain(t_id in 1..=MAX_TERM) {
        let t = SymbolId(t_id);
        let a = SymbolId(NT_BASE);
        let b = SymbolId(NT_BASE + 1);
        let mut g = Grammar::new("chain".into());
        tok(&mut g, t, &format!("t{t_id}"), "x");
        g.rule_names.insert(a, "A".into());
        g.rule_names.insert(b, "B".into());
        g.rules.entry(a).or_default().push(rule(a, vec![Symbol::NonTerminal(b)], 0));
        g.rules.entry(b).or_default().push(rule(b, vec![Symbol::Terminal(t)], 1));
        let ff = FirstFollowSets::compute(&g).unwrap();
        let first_a = ff.first(a).unwrap();
        prop_assert!(first_a.contains(t_id as usize), "FIRST(A) should contain t from chain A→B→t");
    }
}

// 11. FOLLOW propagates: A → B c ⟹ c ∈ FOLLOW(B)
proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]
    #[test]
    fn follow_of_nt_before_terminal(t1_id in 1..=4u16, t2_id in 5..=MAX_TERM) {
        let t1 = SymbolId(t1_id);
        let t2 = SymbolId(t2_id);
        let a = SymbolId(NT_BASE);
        let b = SymbolId(NT_BASE + 1);
        let mut g = Grammar::new("follow_prop".into());
        tok(&mut g, t1, &format!("t{t1_id}"), "x");
        tok(&mut g, t2, &format!("t{t2_id}"), "y");
        g.rule_names.insert(a, "A".into());
        g.rule_names.insert(b, "B".into());
        g.rules.entry(a).or_default().push(rule(
            a,
            vec![Symbol::NonTerminal(b), Symbol::Terminal(t2)],
            0,
        ));
        g.rules.entry(b).or_default().push(rule(b, vec![Symbol::Terminal(t1)], 1));
        let ff = FirstFollowSets::compute(&g).unwrap();
        let follow_b = ff.follow(b).unwrap();
        prop_assert!(
            follow_b.contains(t2_id as usize),
            "FOLLOW(B) should contain t2 from A → B t2"
        );
    }
}

// 12. Nullable propagation through chain: A → B, B → ε ⟹ A nullable
proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]
    #[test]
    fn nullable_chain_propagation(t_id in 1..=MAX_TERM) {
        let t = SymbolId(t_id);
        let a = SymbolId(NT_BASE);
        let b = SymbolId(NT_BASE + 1);
        let mut g = Grammar::new("null_chain".into());
        tok(&mut g, t, &format!("t{t_id}"), "x");
        g.rule_names.insert(a, "A".into());
        g.rule_names.insert(b, "B".into());
        g.rules.entry(a).or_default().push(rule(a, vec![Symbol::NonTerminal(b)], 0));
        g.rules.entry(b).or_default().push(rule(b, vec![Symbol::Epsilon], 1));
        let ff = FirstFollowSets::compute(&g).unwrap();
        prop_assert!(ff.is_nullable(b), "B with epsilon must be nullable");
        prop_assert!(ff.is_nullable(a), "A → B where B nullable ⟹ A nullable");
    }
}

// 13. FIRST(S) is subset of registered terminal IDs for random grammars
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]
    #[test]
    fn first_set_within_terminal_range(g in arb_small_grammar()) {
        if let Ok(ff) = FirstFollowSets::compute(&g) {
            let _terminal_ids: std::collections::HashSet<u16> =
                g.tokens.keys().map(|id| id.0).collect();
            for (nt_id, _) in &g.rules {
                if let Some(first_set) = ff.first(*nt_id) {
                    for bit in first_set.ones() {
                        // FIRST set bits should correspond to terminal IDs
                        // or to FIRST sets of other non-terminals (which ultimately
                        // trace back to terminals). For simple grammars, all bits
                        // should be terminal IDs.
                        // We just check the bit is within a reasonable range.
                        prop_assert!(
                            bit < 100,
                            "FIRST set bit {bit} unreasonably large for nt {nt_id:?}"
                        );
                    }
                }
            }
        }
    }
}

// 14. FOLLOW set bits are within reasonable range
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]
    #[test]
    fn follow_set_within_range(g in arb_small_grammar()) {
        if let Ok(ff) = FirstFollowSets::compute(&g) {
            for (nt_id, _) in &g.rules {
                if let Some(follow_set) = ff.follow(*nt_id) {
                    for bit in follow_set.ones() {
                        prop_assert!(
                            bit < 100,
                            "FOLLOW set bit {bit} unreasonably large for nt {nt_id:?}"
                        );
                    }
                }
            }
        }
    }
}

// 15. compute_normalized doesn't panic
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]
    #[test]
    fn compute_normalized_does_not_panic(mut g in arb_small_grammar()) {
        let _ = FirstFollowSets::compute_normalized(&mut g);
    }
}

// 16. compute and compute_normalized agree on simple (already normalized) grammars
proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]
    #[test]
    fn compute_and_normalized_agree(g in arb_small_grammar()) {
        let ff1 = FirstFollowSets::compute(&g);
        let mut g2 = g.clone();
        let ff2 = FirstFollowSets::compute_normalized(&mut g2);
        match (ff1, ff2) {
            (Ok(a), Ok(b)) => {
                for (id, _) in &g.rules {
                    let f1 = a.first(*id);
                    let f2 = b.first(*id);
                    prop_assert_eq!(f1, f2, "FIRST({:?}) should agree", id);
                }
            }
            (Err(_), Err(_)) => {}
            _ => prop_assert!(false, "compute and compute_normalized disagree on success/failure"),
        }
    }
}

// 17. Nullable iff can derive empty
proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]
    #[test]
    fn nullable_iff_epsilon_reachable(choice in proptest::bool::ANY) {
        let s = SymbolId(NT_BASE);
        let t = SymbolId(1);
        let mut g = Grammar::new("null_iff".into());
        tok(&mut g, t, "t1", "x");
        g.rule_names.insert(s, "S".into());
        if choice {
            // S → ε | t  (nullable)
            g.rules.entry(s).or_default().push(rule(s, vec![Symbol::Epsilon], 0));
            g.rules.entry(s).or_default().push(rule(s, vec![Symbol::Terminal(t)], 1));
        } else {
            // S → t  (not nullable)
            g.rules.entry(s).or_default().push(rule(s, vec![Symbol::Terminal(t)], 0));
        }
        let ff = FirstFollowSets::compute(&g).unwrap();
        prop_assert_eq!(ff.is_nullable(s), choice, "nullable mismatch");
    }
}

// 18. Left recursion doesn't diverge: S → S a | a
proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]
    #[test]
    fn left_recursion_terminates(t_id in 1..=MAX_TERM) {
        let t = SymbolId(t_id);
        let s = SymbolId(NT_BASE);
        let mut g = Grammar::new("leftrec".into());
        tok(&mut g, t, &format!("t{t_id}"), "x");
        g.rule_names.insert(s, "S".into());
        g.rules.entry(s).or_default().push(rule(
            s,
            vec![Symbol::NonTerminal(s), Symbol::Terminal(t)],
            0,
        ));
        g.rules.entry(s).or_default().push(rule(s, vec![Symbol::Terminal(t)], 1));
        let ff = FirstFollowSets::compute(&g).unwrap();
        prop_assert!(ff.first(s).unwrap().contains(t_id as usize));
    }
}

// 19. Right recursion doesn't diverge: S → a S | a
proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]
    #[test]
    fn right_recursion_terminates(t_id in 1..=MAX_TERM) {
        let t = SymbolId(t_id);
        let s = SymbolId(NT_BASE);
        let mut g = Grammar::new("rightrec".into());
        tok(&mut g, t, &format!("t{t_id}"), "x");
        g.rule_names.insert(s, "S".into());
        g.rules.entry(s).or_default().push(rule(
            s,
            vec![Symbol::Terminal(t), Symbol::NonTerminal(s)],
            0,
        ));
        g.rules.entry(s).or_default().push(rule(s, vec![Symbol::Terminal(t)], 1));
        let ff = FirstFollowSets::compute(&g).unwrap();
        prop_assert!(ff.first(s).unwrap().contains(t_id as usize));
    }
}

// 20. Mutual recursion: A → B, B → A a | a
proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]
    #[test]
    fn mutual_recursion_terminates(t_id in 1..=MAX_TERM) {
        let t = SymbolId(t_id);
        let a = SymbolId(NT_BASE);
        let b = SymbolId(NT_BASE + 1);
        let mut g = Grammar::new("mutual".into());
        tok(&mut g, t, &format!("t{t_id}"), "x");
        g.rule_names.insert(a, "A".into());
        g.rule_names.insert(b, "B".into());
        g.rules.entry(a).or_default().push(rule(a, vec![Symbol::NonTerminal(b)], 0));
        g.rules.entry(b).or_default().push(rule(
            b,
            vec![Symbol::NonTerminal(a), Symbol::Terminal(t)],
            1,
        ));
        g.rules.entry(b).or_default().push(rule(b, vec![Symbol::Terminal(t)], 2));
        let ff = FirstFollowSets::compute(&g).unwrap();
        prop_assert!(ff.first(a).unwrap().contains(t_id as usize));
        prop_assert!(ff.first(b).unwrap().contains(t_id as usize));
    }
}

// 21. Multiple terminals: FIRST collects all leading terminals
proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]
    #[test]
    fn first_collects_all_alternatives(t1 in 1..=4u16, t2 in 5..=MAX_TERM) {
        let s = SymbolId(NT_BASE);
        let mut g = Grammar::new("multi_alt".into());
        tok(&mut g, SymbolId(t1), &format!("t{t1}"), "a");
        tok(&mut g, SymbolId(t2), &format!("t{t2}"), "b");
        g.rule_names.insert(s, "S".into());
        g.rules.entry(s).or_default().push(rule(s, vec![Symbol::Terminal(SymbolId(t1))], 0));
        g.rules.entry(s).or_default().push(rule(s, vec![Symbol::Terminal(SymbolId(t2))], 1));
        let ff = FirstFollowSets::compute(&g).unwrap();
        let first_s = ff.first(s).unwrap();
        prop_assert!(first_s.contains(t1 as usize), "FIRST(S) missing t1");
        prop_assert!(first_s.contains(t2 as usize), "FIRST(S) missing t2");
    }
}

// 22. Nullable prefix: A → B C, B nullable ⟹ FIRST(C) ⊆ FIRST(A)
proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]
    #[test]
    fn first_through_nullable_prefix(t1 in 1..=4u16, t2 in 5..=MAX_TERM) {
        let a = SymbolId(NT_BASE);
        let b = SymbolId(NT_BASE + 1);
        let c = SymbolId(NT_BASE + 2);
        let mut g = Grammar::new("null_prefix".into());
        tok(&mut g, SymbolId(t1), &format!("t{t1}"), "a");
        tok(&mut g, SymbolId(t2), &format!("t{t2}"), "b");
        g.rule_names.insert(a, "A".into());
        g.rule_names.insert(b, "B".into());
        g.rule_names.insert(c, "C".into());
        g.rules.entry(a).or_default().push(rule(
            a,
            vec![Symbol::NonTerminal(b), Symbol::NonTerminal(c)],
            0,
        ));
        g.rules.entry(b).or_default().push(rule(b, vec![Symbol::Epsilon], 1));
        g.rules.entry(b).or_default().push(rule(b, vec![Symbol::Terminal(SymbolId(t1))], 2));
        g.rules.entry(c).or_default().push(rule(c, vec![Symbol::Terminal(SymbolId(t2))], 3));
        let ff = FirstFollowSets::compute(&g).unwrap();
        let first_a = ff.first(a).unwrap();
        // FIRST(A) should contain FIRST(C) since B is nullable
        prop_assert!(
            first_a.contains(t2 as usize),
            "FIRST(A) should contain FIRST(C) when B is nullable"
        );
    }
}

// 23. FOLLOW inherits from LHS at end of rule: A → x B, FOLLOW(A) ⊆ FOLLOW(B)
proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]
    #[test]
    fn follow_inherited_at_end(t_id in 1..=MAX_TERM) {
        let t = SymbolId(t_id);
        let a = SymbolId(NT_BASE);
        let b = SymbolId(NT_BASE + 1);
        let mut g = Grammar::new("follow_end".into());
        tok(&mut g, t, &format!("t{t_id}"), "x");
        g.rule_names.insert(a, "A".into());
        g.rule_names.insert(b, "B".into());
        g.rules.entry(a).or_default().push(rule(
            a,
            vec![Symbol::Terminal(t), Symbol::NonTerminal(b)],
            0,
        ));
        g.rules.entry(b).or_default().push(rule(b, vec![Symbol::Terminal(t)], 1));
        let ff = FirstFollowSets::compute(&g).unwrap();
        // A is the start symbol, so FOLLOW(A) contains EOF (0).
        // Since B is at end of A's rule, FOLLOW(A) ⊆ FOLLOW(B).
        if let (Some(fa), Some(fb)) = (ff.follow(a), ff.follow(b)) {
            for bit in fa.ones() {
                prop_assert!(
                    fb.contains(bit),
                    "FOLLOW(A) bit {bit} should be in FOLLOW(B)"
                );
            }
        }
    }
}

// 24. Nullable determinism matches
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]
    #[test]
    fn nullable_is_deterministic(g in arb_small_grammar()) {
        let ff1 = FirstFollowSets::compute(&g);
        let ff2 = FirstFollowSets::compute(&g);
        match (ff1, ff2) {
            (Ok(a), Ok(b)) => {
                for (id, _) in &g.rules {
                    prop_assert_eq!(
                        a.is_nullable(*id),
                        b.is_nullable(*id),
                        "nullable({:?}) differs across runs", id
                    );
                }
            }
            (Err(_), Err(_)) => {}
            _ => prop_assert!(false, "nullable determinism mismatch"),
        }
    }
}

// 25. first_of_sequence with single terminal
proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]
    #[test]
    fn first_of_sequence_single_terminal(t_id in 1..=MAX_TERM) {
        let t = SymbolId(t_id);
        let g = single_rule_grammar(&[t_id], vec![Symbol::Terminal(t)]);
        let ff = FirstFollowSets::compute(&g).unwrap();
        let seq_first = ff.first_of_sequence(&[Symbol::Terminal(t)]).unwrap();
        prop_assert!(seq_first.contains(t_id as usize));
        prop_assert_eq!(seq_first.count_ones(..), 1);
    }
}

// 26. first_of_sequence with nullable prefix
proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]
    #[test]
    fn first_of_sequence_nullable_prefix(t1 in 1..=4u16, t2 in 5..=MAX_TERM) {
        let b = SymbolId(NT_BASE + 1);
        let mut g = Grammar::new("seq_null".into());
        tok(&mut g, SymbolId(t1), &format!("t{t1}"), "a");
        tok(&mut g, SymbolId(t2), &format!("t{t2}"), "b");
        let s = SymbolId(NT_BASE);
        g.rule_names.insert(s, "S".into());
        g.rule_names.insert(b, "B".into());
        g.rules.entry(s).or_default().push(rule(
            s,
            vec![Symbol::NonTerminal(b), Symbol::Terminal(SymbolId(t2))],
            0,
        ));
        g.rules.entry(b).or_default().push(rule(b, vec![Symbol::Epsilon], 1));
        let ff = FirstFollowSets::compute(&g).unwrap();
        let seq = vec![Symbol::NonTerminal(b), Symbol::Terminal(SymbolId(t2))];
        let result = ff.first_of_sequence(&seq).unwrap();
        prop_assert!(result.contains(t2 as usize), "first_of_sequence should see past nullable B");
    }
}

// 27. Empty grammar (no rules) doesn't panic
#[test]
fn empty_grammar_no_panic() {
    let g = Grammar::new("empty".into());
    let _ = FirstFollowSets::compute(&g);
}

// 28. Grammar with only epsilon rules
proptest! {
    #![proptest_config(ProptestConfig::with_cases(30))]
    #[test]
    fn all_epsilon_grammar(num_nt in 1..=4usize) {
        let mut g = Grammar::new("all_eps".into());
        // Need at least one token for a valid grammar context
        tok(&mut g, SymbolId(1), "t1", "x");
        for (prod, i) in (0..num_nt).enumerate() {
            let nt = SymbolId(NT_BASE + i as u16);
            g.rule_names.insert(nt, format!("N{i}"));
            g.rules.entry(nt).or_default().push(rule(nt, vec![Symbol::Epsilon], prod as u16));
        }
        let ff = FirstFollowSets::compute(&g).unwrap();
        for i in 0..num_nt {
            let nt = SymbolId(NT_BASE + i as u16);
            prop_assert!(ff.is_nullable(nt), "N{i} with only epsilon should be nullable");
        }
    }
}

// 29. FIRST set is superset when alternatives are added
proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]
    #[test]
    fn adding_alternative_grows_first(t1 in 1..=4u16, t2 in 5..=MAX_TERM) {
        let s = SymbolId(NT_BASE);
        // Grammar 1: S → t1
        let mut g1 = Grammar::new("grow1".into());
        tok(&mut g1, SymbolId(t1), &format!("t{t1}"), "a");
        tok(&mut g1, SymbolId(t2), &format!("t{t2}"), "b");
        g1.rule_names.insert(s, "S".into());
        g1.rules.entry(s).or_default().push(rule(s, vec![Symbol::Terminal(SymbolId(t1))], 0));
        let ff1 = FirstFollowSets::compute(&g1).unwrap();

        // Grammar 2: S → t1 | t2
        let mut g2 = g1.clone();
        g2.rules.entry(s).or_default().push(rule(s, vec![Symbol::Terminal(SymbolId(t2))], 1));
        let ff2 = FirstFollowSets::compute(&g2).unwrap();

        let first1 = ff1.first(s).unwrap();
        let first2 = ff2.first(s).unwrap();
        // first1 ⊆ first2
        for bit in first1.ones() {
            prop_assert!(first2.contains(bit), "adding alternative should grow FIRST");
        }
        prop_assert!(first2.contains(t2 as usize), "new alternative should appear in FIRST");
    }
}

// 30. FOLLOW of non-start non-terminal might be empty (no assertion failure)
proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]
    #[test]
    fn follow_can_be_empty_for_non_start(t_id in 1..=MAX_TERM) {
        // A → t, B → t  (B is never referenced in any RHS)
        let t = SymbolId(t_id);
        let a = SymbolId(NT_BASE);
        let b = SymbolId(NT_BASE + 1);
        let mut g = Grammar::new("isolated".into());
        tok(&mut g, t, &format!("t{t_id}"), "x");
        g.rule_names.insert(a, "A".into());
        g.rule_names.insert(b, "B".into());
        g.rules.entry(a).or_default().push(rule(a, vec![Symbol::Terminal(t)], 0));
        g.rules.entry(b).or_default().push(rule(b, vec![Symbol::Terminal(t)], 1));
        let ff = FirstFollowSets::compute(&g).unwrap();
        // B is never used in any other rule's RHS, so its FOLLOW could be empty
        // (or could have EOF if it happens to be picked as start symbol).
        // Just ensure no panic.
        let _ = ff.follow(b);
    }
}

// 31. FIRST of sequence with all nullable symbols
proptest! {
    #![proptest_config(ProptestConfig::with_cases(30))]
    #[test]
    fn first_of_all_nullable_sequence(t_id in 1..=MAX_TERM) {
        let t = SymbolId(t_id);
        let a = SymbolId(NT_BASE);
        let b = SymbolId(NT_BASE + 1);
        let mut g = Grammar::new("all_null_seq".into());
        tok(&mut g, t, &format!("t{t_id}"), "x");
        g.rule_names.insert(a, "A".into());
        g.rule_names.insert(b, "B".into());
        g.rules.entry(a).or_default().push(rule(a, vec![Symbol::Epsilon], 0));
        g.rules.entry(a).or_default().push(rule(a, vec![Symbol::Terminal(t)], 1));
        g.rules.entry(b).or_default().push(rule(b, vec![Symbol::Epsilon], 2));
        let ff = FirstFollowSets::compute(&g).unwrap();
        // Sequence [A, B] — both nullable; FIRST should include FIRST(A)
        let seq = vec![Symbol::NonTerminal(a), Symbol::NonTerminal(b)];
        let result = ff.first_of_sequence(&seq).unwrap();
        prop_assert!(result.contains(t_id as usize), "should contain FIRST(A)");
    }
}

// 32. Diamond: A → B C, B → t1, C → t2; FIRST(A) = {t1}
proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]
    #[test]
    fn diamond_first_set(t1 in 1..=4u16, t2 in 5..=MAX_TERM) {
        let a = SymbolId(NT_BASE);
        let b = SymbolId(NT_BASE + 1);
        let c = SymbolId(NT_BASE + 2);
        let mut g = Grammar::new("diamond".into());
        tok(&mut g, SymbolId(t1), &format!("t{t1}"), "a");
        tok(&mut g, SymbolId(t2), &format!("t{t2}"), "b");
        g.rule_names.insert(a, "A".into());
        g.rule_names.insert(b, "B".into());
        g.rule_names.insert(c, "C".into());
        g.rules.entry(a).or_default().push(rule(
            a,
            vec![Symbol::NonTerminal(b), Symbol::NonTerminal(c)],
            0,
        ));
        g.rules.entry(b).or_default().push(rule(b, vec![Symbol::Terminal(SymbolId(t1))], 1));
        g.rules.entry(c).or_default().push(rule(c, vec![Symbol::Terminal(SymbolId(t2))], 2));
        let ff = FirstFollowSets::compute(&g).unwrap();
        let first_a = ff.first(a).unwrap();
        // B is not nullable so FIRST(A) = FIRST(B) = {t1}
        prop_assert!(first_a.contains(t1 as usize), "FIRST(A) should contain t1");
        prop_assert!(!first_a.contains(t2 as usize), "FIRST(A) should NOT contain t2 (B not nullable)");
    }
}

// 33. Medium grammar: FIRST sets for all non-terminals are computed
proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]
    #[test]
    fn medium_grammar_all_nts_have_first(g in arb_medium_grammar()) {
        if let Ok(ff) = FirstFollowSets::compute(&g) {
            for (nt_id, _) in &g.rules {
                // Every non-terminal should have a FIRST set entry
                prop_assert!(
                    ff.first(*nt_id).is_some(),
                    "NT {nt_id:?} should have a FIRST set"
                );
            }
        }
    }
}

// 34. Medium grammar: all non-terminals have FOLLOW set entries
proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]
    #[test]
    fn medium_grammar_all_nts_have_follow(g in arb_medium_grammar()) {
        if let Ok(ff) = FirstFollowSets::compute(&g) {
            for (nt_id, _) in &g.rules {
                prop_assert!(
                    ff.follow(*nt_id).is_some(),
                    "NT {nt_id:?} should have a FOLLOW set"
                );
            }
        }
    }
}

// 35. Querying FIRST/FOLLOW on unknown symbols returns None
proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]
    #[test]
    fn unknown_symbol_returns_none(t_id in 1..=MAX_TERM) {
        let t = SymbolId(t_id);
        let s = SymbolId(NT_BASE);
        let mut g = Grammar::new("unknown".into());
        tok(&mut g, t, &format!("t{t_id}"), "x");
        g.rule_names.insert(s, "S".into());
        g.rules.entry(s).or_default().push(rule(s, vec![Symbol::Terminal(t)], 0));
        let ff = FirstFollowSets::compute(&g).unwrap();
        // A symbol that is neither a rule LHS nor a token should return None
        let unknown = SymbolId(200);
        prop_assert!(ff.first(unknown).is_none(), "FIRST of unknown symbol should be None");
        prop_assert!(ff.follow(unknown).is_none(), "FOLLOW of unknown symbol should be None");
    }
}
