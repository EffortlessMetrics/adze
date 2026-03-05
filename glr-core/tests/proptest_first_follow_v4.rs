//! Property-based tests for FIRST/FOLLOW set computation (v4).
//!
//! Run with: `cargo test -p adze-glr-core --test proptest_first_follow_v4`

use adze_glr_core::FirstFollowSets;
use adze_ir::builder::GrammarBuilder;
use adze_ir::*;
use proptest::prelude::*;

// ---------------------------------------------------------------------------
// Grammar construction helpers (manual, low-level)
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

/// Find a symbol by name in either rule_names or tokens.
fn sym(g: &Grammar, name: &str) -> SymbolId {
    if let Some(id) = g.find_symbol_by_name(name) {
        return id;
    }
    for (id, token) in &g.tokens {
        if token.name == name {
            return *id;
        }
    }
    panic!("symbol {name:?} not found in grammar");
}

const MAX_TERM: u16 = 8;
const NT_BASE: u16 = 10;

// ---------------------------------------------------------------------------
// Proptest strategies
// ---------------------------------------------------------------------------

fn arb_terminal() -> impl Strategy<Value = SymbolId> {
    (1..=MAX_TERM).prop_map(SymbolId)
}

fn arb_nonterminal() -> impl Strategy<Value = SymbolId> {
    (NT_BASE..NT_BASE + 4).prop_map(SymbolId)
}

fn arb_rhs_symbol() -> impl Strategy<Value = Symbol> {
    prop_oneof![
        8 => arb_terminal().prop_map(Symbol::Terminal),
        4 => arb_nonterminal().prop_map(Symbol::NonTerminal),
        1 => Just(Symbol::Epsilon),
    ]
}

fn arb_rhs() -> impl Strategy<Value = Vec<Symbol>> {
    prop::collection::vec(arb_rhs_symbol(), 1..=3)
}

/// Generate a small grammar: 2-4 terminals, 1-3 nonterminals, 1-3 prods each.
fn arb_small_grammar() -> impl Strategy<Value = Grammar> {
    (2..=4usize, 1..=3usize).prop_flat_map(|(num_t, num_nt)| {
        let prods = prop::collection::vec(prop::collection::vec(arb_rhs(), 1..=3), num_nt..=num_nt);
        prods.prop_map(move |all_prods| {
            let mut g = Grammar::new("proptest".into());
            for i in 1..=(num_t as u16).min(MAX_TERM) {
                tok(&mut g, SymbolId(i), &format!("t{i}"), &format!("t{i}"));
            }
            let mut prod_counter = 0u16;
            for (idx, prod_list) in all_prods.iter().enumerate() {
                let nt_id = SymbolId(NT_BASE + idx as u16);
                g.rule_names.insert(nt_id, format!("N{idx}"));
                for rhs in prod_list {
                    let filtered: Vec<Symbol> = rhs
                        .iter()
                        .map(|sym| match sym {
                            Symbol::Terminal(id) if id.0 > (num_t as u16).min(MAX_TERM) => {
                                Symbol::Terminal(SymbolId(1))
                            }
                            Symbol::NonTerminal(id) if id.0 >= NT_BASE + all_prods.len() as u16 => {
                                Symbol::NonTerminal(SymbolId(NT_BASE))
                            }
                            other => other.clone(),
                        })
                        .collect();
                    g.rules.entry(nt_id).or_default().push(make_rule(
                        nt_id,
                        filtered,
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
// 1. FIRST set properties (8 proptest tests)
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// FIRST(S) is non-empty when S has at least one terminal production.
    #[test]
    fn first_nonempty_for_terminal_production(t_id in 1..=MAX_TERM) {
        let s = SymbolId(NT_BASE);
        let mut g = Grammar::new("nonempty".into());
        tok(&mut g, SymbolId(t_id), &format!("t{t_id}"), "x");
        g.rule_names.insert(s, "S".into());
        g.rules.entry(s).or_default().push(
            make_rule(s, vec![Symbol::Terminal(SymbolId(t_id))], 0),
        );
        let ff = FirstFollowSets::compute(&g).unwrap();
        let first_s = ff.first(s).unwrap();
        prop_assert!(first_s.count_ones(..) > 0, "FIRST(S) must be non-empty");
    }

    /// A terminal appears in its own FIRST set (via the nonterminal that produces it).
    #[test]
    fn terminal_in_first_of_producing_nt(t_id in 1..=MAX_TERM) {
        let t = SymbolId(t_id);
        let s = SymbolId(NT_BASE);
        let mut g = Grammar::new("term_first".into());
        tok(&mut g, t, &format!("t{t_id}"), "x");
        g.rule_names.insert(s, "S".into());
        g.rules.entry(s).or_default().push(
            make_rule(s, vec![Symbol::Terminal(t)], 0),
        );
        let ff = FirstFollowSets::compute(&g).unwrap();
        let first_s = ff.first(s).unwrap();
        prop_assert!(
            first_s.contains(t_id as usize),
            "FIRST(S) should contain terminal t{t_id}"
        );
    }

    /// Epsilon production makes the nonterminal nullable.
    #[test]
    fn epsilon_makes_nullable(t_id in 1..=MAX_TERM) {
        let s = SymbolId(NT_BASE);
        let mut g = Grammar::new("eps".into());
        tok(&mut g, SymbolId(t_id), &format!("t{t_id}"), "x");
        g.rule_names.insert(s, "S".into());
        g.rules.entry(s).or_default().push(make_rule(s, vec![Symbol::Epsilon], 0));
        g.rules.entry(s).or_default().push(
            make_rule(s, vec![Symbol::Terminal(SymbolId(t_id))], 1),
        );
        let ff = FirstFollowSets::compute(&g).unwrap();
        prop_assert!(ff.is_nullable(s), "S with epsilon must be nullable");
    }

    /// Non-epsilon terminal-only rule is not nullable.
    #[test]
    fn terminal_rule_not_nullable(t_id in 1..=MAX_TERM) {
        let s = SymbolId(NT_BASE);
        let mut g = Grammar::new("noeps".into());
        tok(&mut g, SymbolId(t_id), &format!("t{t_id}"), "x");
        g.rule_names.insert(s, "S".into());
        g.rules.entry(s).or_default().push(
            make_rule(s, vec![Symbol::Terminal(SymbolId(t_id))], 0),
        );
        let ff = FirstFollowSets::compute(&g).unwrap();
        prop_assert!(!ff.is_nullable(s), "S with only terminal must not be nullable");
    }

    /// FIRST propagates through chain: A → B, B → t ⟹ t ∈ FIRST(A).
    #[test]
    fn first_propagates_through_chain(t_id in 1..=MAX_TERM) {
        let t = SymbolId(t_id);
        let a = SymbolId(NT_BASE);
        let b = SymbolId(NT_BASE + 1);
        let mut g = Grammar::new("chain".into());
        tok(&mut g, t, &format!("t{t_id}"), "x");
        g.rule_names.insert(a, "A".into());
        g.rule_names.insert(b, "B".into());
        g.rules.entry(a).or_default().push(
            make_rule(a, vec![Symbol::NonTerminal(b)], 0),
        );
        g.rules.entry(b).or_default().push(
            make_rule(b, vec![Symbol::Terminal(t)], 1),
        );
        let ff = FirstFollowSets::compute(&g).unwrap();
        let first_a = ff.first(a).unwrap();
        prop_assert!(
            first_a.contains(t_id as usize),
            "FIRST(A) should contain t via A→B→t"
        );
    }

    /// Epsilon propagation through chain: A → B, B → ε ⟹ A nullable.
    #[test]
    fn nullable_propagates_through_chain(t_id in 1..=MAX_TERM) {
        let t = SymbolId(t_id);
        let a = SymbolId(NT_BASE);
        let b = SymbolId(NT_BASE + 1);
        let mut g = Grammar::new("null_chain".into());
        tok(&mut g, t, &format!("t{t_id}"), "x");
        g.rule_names.insert(a, "A".into());
        g.rule_names.insert(b, "B".into());
        g.rules.entry(a).or_default().push(
            make_rule(a, vec![Symbol::NonTerminal(b)], 0),
        );
        g.rules.entry(b).or_default().push(make_rule(b, vec![Symbol::Epsilon], 1));
        let ff = FirstFollowSets::compute(&g).unwrap();
        prop_assert!(ff.is_nullable(b), "B with epsilon must be nullable");
        prop_assert!(ff.is_nullable(a), "A→B where B nullable ⟹ A nullable");
    }

    /// FIRST of multi-alternative nonterminal is union of FIRST of each alternative.
    #[test]
    fn first_is_union_of_alternatives(t1 in 1..=4u16, t2 in 5..=MAX_TERM) {
        let s = SymbolId(NT_BASE);
        let mut g = Grammar::new("union".into());
        tok(&mut g, SymbolId(t1), &format!("t{t1}"), "a");
        tok(&mut g, SymbolId(t2), &format!("t{t2}"), "b");
        g.rule_names.insert(s, "S".into());
        g.rules.entry(s).or_default().push(
            make_rule(s, vec![Symbol::Terminal(SymbolId(t1))], 0),
        );
        g.rules.entry(s).or_default().push(
            make_rule(s, vec![Symbol::Terminal(SymbolId(t2))], 1),
        );
        let ff = FirstFollowSets::compute(&g).unwrap();
        let first_s = ff.first(s).unwrap();
        prop_assert!(first_s.contains(t1 as usize), "FIRST(S) must contain t1");
        prop_assert!(first_s.contains(t2 as usize), "FIRST(S) must contain t2");
    }

    /// When A → B C and B is nullable, FIRST(A) includes FIRST(C).
    #[test]
    fn first_skips_nullable_prefix(t1 in 1..=4u16, t2 in 5..=MAX_TERM) {
        let a = SymbolId(NT_BASE);
        let b = SymbolId(NT_BASE + 1);
        let mut g = Grammar::new("skipnull".into());
        tok(&mut g, SymbolId(t1), &format!("t{t1}"), "a");
        tok(&mut g, SymbolId(t2), &format!("t{t2}"), "b");
        g.rule_names.insert(a, "A".into());
        g.rule_names.insert(b, "B".into());
        g.rules.entry(b).or_default().push(make_rule(b, vec![Symbol::Epsilon], 0));
        g.rules.entry(b).or_default().push(
            make_rule(b, vec![Symbol::Terminal(SymbolId(t1))], 1),
        );
        g.rules.entry(a).or_default().push(
            make_rule(a, vec![Symbol::NonTerminal(b), Symbol::Terminal(SymbolId(t2))], 2),
        );
        let ff = FirstFollowSets::compute(&g).unwrap();
        let first_a = ff.first(a).unwrap();
        // B is nullable so FIRST(A) includes FIRST(B) ∪ {t2}
        prop_assert!(first_a.contains(t1 as usize), "FIRST(A) must include FIRST(B)");
        prop_assert!(first_a.contains(t2 as usize), "FIRST(A) must include t2 past nullable B");
    }
}

// =========================================================================
// 2. FOLLOW set properties (8 proptest tests)
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// FOLLOW(start) contains EOF for any grammar with a start symbol.
    #[test]
    fn follow_start_has_eof(g in arb_small_grammar()) {
        if let Ok(ff) = FirstFollowSets::compute(&g)
            && let Some(start) = g.start_symbol()
            && let Some(follow) = ff.follow(start)
        {
            prop_assert!(follow.contains(0), "FOLLOW(start) must contain EOF");
        }
    }

    /// A → B t ⟹ t ∈ FOLLOW(B).
    #[test]
    fn follow_nt_before_terminal(t1 in 1..=4u16, t2 in 5..=MAX_TERM) {
        let a = SymbolId(NT_BASE);
        let b = SymbolId(NT_BASE + 1);
        let mut g = Grammar::new("follow".into());
        tok(&mut g, SymbolId(t1), &format!("t{t1}"), "x");
        tok(&mut g, SymbolId(t2), &format!("t{t2}"), "y");
        g.rule_names.insert(a, "A".into());
        g.rule_names.insert(b, "B".into());
        g.rules.entry(a).or_default().push(
            make_rule(a, vec![Symbol::NonTerminal(b), Symbol::Terminal(SymbolId(t2))], 0),
        );
        g.rules.entry(b).or_default().push(
            make_rule(b, vec![Symbol::Terminal(SymbolId(t1))], 1),
        );
        let ff = FirstFollowSets::compute(&g).unwrap();
        let follow_b = ff.follow(b).unwrap();
        prop_assert!(
            follow_b.contains(t2 as usize),
            "FOLLOW(B) must contain t2 from A → B t2"
        );
    }

    /// A → B C ⟹ FIRST(C) ⊆ FOLLOW(B).
    #[test]
    fn follow_contains_first_of_successor(t1 in 1..=4u16, t2 in 5..=MAX_TERM) {
        let a = SymbolId(NT_BASE);
        let b = SymbolId(NT_BASE + 1);
        let c = SymbolId(NT_BASE + 2);
        let mut g = Grammar::new("follow_first".into());
        tok(&mut g, SymbolId(t1), &format!("t{t1}"), "x");
        tok(&mut g, SymbolId(t2), &format!("t{t2}"), "y");
        g.rule_names.insert(a, "A".into());
        g.rule_names.insert(b, "B".into());
        g.rule_names.insert(c, "C".into());
        g.rules.entry(a).or_default().push(
            make_rule(a, vec![Symbol::NonTerminal(b), Symbol::NonTerminal(c)], 0),
        );
        g.rules.entry(b).or_default().push(
            make_rule(b, vec![Symbol::Terminal(SymbolId(t1))], 1),
        );
        g.rules.entry(c).or_default().push(
            make_rule(c, vec![Symbol::Terminal(SymbolId(t2))], 2),
        );
        let ff = FirstFollowSets::compute(&g).unwrap();
        let follow_b = ff.follow(b).unwrap();
        let first_c = ff.first(c).unwrap();
        for bit in first_c.ones() {
            prop_assert!(
                follow_b.contains(bit),
                "FOLLOW(B) must contain all of FIRST(C), missing bit {bit}"
            );
        }
    }

    /// A → B C, C nullable ⟹ FOLLOW(A) ⊆ FOLLOW(B).
    #[test]
    fn follow_propagates_when_suffix_nullable(t1 in 1..=4u16, t2 in 5..=MAX_TERM) {
        let a = SymbolId(NT_BASE);
        let b = SymbolId(NT_BASE + 1);
        let c = SymbolId(NT_BASE + 2);
        let mut g = Grammar::new("follow_null".into());
        tok(&mut g, SymbolId(t1), &format!("t{t1}"), "x");
        tok(&mut g, SymbolId(t2), &format!("t{t2}"), "y");
        g.rule_names.insert(a, "A".into());
        g.rule_names.insert(b, "B".into());
        g.rule_names.insert(c, "C".into());
        g.rules.entry(a).or_default().push(
            make_rule(a, vec![Symbol::NonTerminal(b), Symbol::NonTerminal(c)], 0),
        );
        g.rules.entry(b).or_default().push(
            make_rule(b, vec![Symbol::Terminal(SymbolId(t1))], 1),
        );
        g.rules.entry(c).or_default().push(make_rule(c, vec![Symbol::Epsilon], 2));
        g.rules.entry(c).or_default().push(
            make_rule(c, vec![Symbol::Terminal(SymbolId(t2))], 3),
        );
        let ff = FirstFollowSets::compute(&g).unwrap();
        let follow_a = ff.follow(a).unwrap();
        let follow_b = ff.follow(b).unwrap();
        for bit in follow_a.ones() {
            prop_assert!(
                follow_b.contains(bit),
                "FOLLOW(B) must include FOLLOW(A) when suffix is nullable, missing bit {bit}"
            );
        }
    }

    /// FOLLOW sets only contain bits within a reasonable range.
    #[test]
    fn follow_bits_within_range(g in arb_small_grammar()) {
        if let Ok(ff) = FirstFollowSets::compute(&g) {
            for (nt_id, _) in &g.rules {
                if let Some(follow_set) = ff.follow(*nt_id) {
                    for bit in follow_set.ones() {
                        prop_assert!(bit < 100, "FOLLOW bit {bit} out of range for {nt_id:?}");
                    }
                }
            }
        }
    }

    /// FIRST set bits are within a reasonable range.
    #[test]
    fn first_bits_within_range(g in arb_small_grammar()) {
        if let Ok(ff) = FirstFollowSets::compute(&g) {
            for (nt_id, _) in &g.rules {
                if let Some(first_set) = ff.first(*nt_id) {
                    for bit in first_set.ones() {
                        prop_assert!(bit < 100, "FIRST bit {bit} out of range for {nt_id:?}");
                    }
                }
            }
        }
    }

    /// A at end of production: A → ... B ⟹ FOLLOW(A) ⊆ FOLLOW(B).
    #[test]
    fn follow_propagates_to_trailing_nt(t1 in 1..=4u16, t2 in 5..=MAX_TERM) {
        let a = SymbolId(NT_BASE);
        let b = SymbolId(NT_BASE + 1);
        let mut g = Grammar::new("trail".into());
        tok(&mut g, SymbolId(t1), &format!("t{t1}"), "x");
        tok(&mut g, SymbolId(t2), &format!("t{t2}"), "y");
        g.rule_names.insert(a, "A".into());
        g.rule_names.insert(b, "B".into());
        // A → t1 B (B is at the end)
        g.rules.entry(a).or_default().push(
            make_rule(a, vec![Symbol::Terminal(SymbolId(t1)), Symbol::NonTerminal(b)], 0),
        );
        g.rules.entry(b).or_default().push(
            make_rule(b, vec![Symbol::Terminal(SymbolId(t2))], 1),
        );
        let ff = FirstFollowSets::compute(&g).unwrap();
        let follow_a = ff.follow(a).unwrap();
        let follow_b = ff.follow(b).unwrap();
        for bit in follow_a.ones() {
            prop_assert!(
                follow_b.contains(bit),
                "FOLLOW(B) must include FOLLOW(A) for trailing NT, missing bit {bit}"
            );
        }
    }

    /// Computation doesn't panic on arbitrary small grammars.
    #[test]
    fn compute_never_panics(g in arb_small_grammar()) {
        let _ = FirstFollowSets::compute(&g);
    }
}

// =========================================================================
// 3. FIRST/FOLLOW interaction (5 proptest tests)
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Deterministic: same grammar yields identical FIRST sets.
    #[test]
    fn deterministic_first(g in arb_small_grammar()) {
        let ff1 = FirstFollowSets::compute(&g);
        let ff2 = FirstFollowSets::compute(&g);
        match (ff1, ff2) {
            (Ok(a), Ok(b)) => {
                for (id, _) in &g.rules {
                    prop_assert_eq!(a.first(*id), b.first(*id), "FIRST({:?}) differs", id);
                }
            }
            (Err(_), Err(_)) => {}
            _ => prop_assert!(false, "inconsistent success/failure"),
        }
    }

    /// Deterministic: same grammar yields identical FOLLOW sets.
    #[test]
    fn deterministic_follow(g in arb_small_grammar()) {
        let ff1 = FirstFollowSets::compute(&g);
        let ff2 = FirstFollowSets::compute(&g);
        match (ff1, ff2) {
            (Ok(a), Ok(b)) => {
                for (id, _) in &g.rules {
                    prop_assert_eq!(a.follow(*id), b.follow(*id), "FOLLOW({:?}) differs", id);
                }
            }
            (Err(_), Err(_)) => {}
            _ => prop_assert!(false, "inconsistent success/failure"),
        }
    }

    /// Deterministic: nullable is consistent across runs.
    #[test]
    fn deterministic_nullable(g in arb_small_grammar()) {
        let ff1 = FirstFollowSets::compute(&g);
        let ff2 = FirstFollowSets::compute(&g);
        match (ff1, ff2) {
            (Ok(a), Ok(b)) => {
                for (id, _) in &g.rules {
                    prop_assert_eq!(
                        a.is_nullable(*id),
                        b.is_nullable(*id),
                        "nullable({:?}) differs", id
                    );
                }
            }
            (Err(_), Err(_)) => {}
            _ => prop_assert!(false, "inconsistent success/failure"),
        }
    }

    /// FIRST/FOLLOW sets exist for all declared nonterminals when compute succeeds.
    #[test]
    fn sets_exist_for_all_nts(g in arb_small_grammar()) {
        if let Ok(ff) = FirstFollowSets::compute(&g) {
            for (nt_id, _) in &g.rules {
                prop_assert!(ff.first(*nt_id).is_some(), "FIRST missing for {nt_id:?}");
                prop_assert!(ff.follow(*nt_id).is_some(), "FOLLOW missing for {nt_id:?}");
            }
        }
    }

    /// If A is nullable and has FOLLOW(A), then FOLLOW elements are terminal-range.
    #[test]
    fn nullable_follow_consistency(g in arb_small_grammar()) {
        if let Ok(ff) = FirstFollowSets::compute(&g) {
            for (nt_id, _) in &g.rules {
                if ff.is_nullable(*nt_id)
                    && let Some(follow_set) = ff.follow(*nt_id)
                {
                    for bit in follow_set.ones() {
                        prop_assert!(bit < 50, "FOLLOW bit {bit} too large for nullable {nt_id:?}");
                    }
                }
            }
        }
    }
}

// =========================================================================
// 4. Deterministic specific grammars (8 regular tests)
// =========================================================================

#[test]
fn specific_single_terminal_rule() {
    // S → a
    let mut g = GrammarBuilder::new("single")
        .token("a", "a")
        .rule("start_s", vec!["a"])
        .start("start_s")
        .build();
    let ff = FirstFollowSets::compute_normalized(&mut g).unwrap();
    let s = sym(&g, "start_s");
    let a = sym(&g, "a");
    assert!(ff.first(s).unwrap().contains(a.0 as usize));
    assert!(!ff.is_nullable(s));
    assert!(ff.follow(s).unwrap().contains(0)); // EOF
}

#[test]
fn specific_two_alternatives() {
    // S → a | b
    let mut g = GrammarBuilder::new("alt")
        .token("a", "a")
        .token("b", "b")
        .rule("start_s", vec!["a"])
        .rule("start_s", vec!["b"])
        .start("start_s")
        .build();
    let ff = FirstFollowSets::compute_normalized(&mut g).unwrap();
    let s = sym(&g, "start_s");
    let a = sym(&g, "a");
    let b = sym(&g, "b");
    assert!(ff.first(s).unwrap().contains(a.0 as usize));
    assert!(ff.first(s).unwrap().contains(b.0 as usize));
}

#[test]
fn specific_sequence_two_terminals() {
    // S → a b
    let mut g = GrammarBuilder::new("seq")
        .token("a", "a")
        .token("b", "b")
        .rule("start_s", vec!["a", "b"])
        .start("start_s")
        .build();
    let ff = FirstFollowSets::compute_normalized(&mut g).unwrap();
    let s = sym(&g, "start_s");
    let a = sym(&g, "a");
    let b = sym(&g, "b");
    let first_s = ff.first(s).unwrap();
    assert!(first_s.contains(a.0 as usize));
    assert!(
        !first_s.contains(b.0 as usize),
        "b not in FIRST(S) for S→ab"
    );
}

#[test]
fn specific_chain_three_deep() {
    // S → A, A → B, B → x
    let mut g = GrammarBuilder::new("deep")
        .token("x", "x")
        .rule("start_s", vec!["nt_a"])
        .rule("nt_a", vec!["nt_b"])
        .rule("nt_b", vec!["x"])
        .start("start_s")
        .build();
    let ff = FirstFollowSets::compute_normalized(&mut g).unwrap();
    let s = sym(&g, "start_s");
    let x = sym(&g, "x");
    assert!(ff.first(s).unwrap().contains(x.0 as usize));
}

#[test]
fn specific_follow_middle_nt() {
    // S → A b, A → a
    let mut g = GrammarBuilder::new("mid")
        .token("a", "a")
        .token("b", "b")
        .rule("start_s", vec!["nt_a", "b"])
        .rule("nt_a", vec!["a"])
        .start("start_s")
        .build();
    let ff = FirstFollowSets::compute_normalized(&mut g).unwrap();
    let a_nt = sym(&g, "nt_a");
    let b = sym(&g, "b");
    assert!(
        ff.follow(a_nt).unwrap().contains(b.0 as usize),
        "FOLLOW(A) must contain b"
    );
}

#[test]
fn specific_follow_eof_only_for_start() {
    // S → a  (only start has EOF in FOLLOW)
    let mut g = GrammarBuilder::new("eof")
        .token("a", "a")
        .rule("start_s", vec!["a"])
        .start("start_s")
        .build();
    let ff = FirstFollowSets::compute_normalized(&mut g).unwrap();
    let s = sym(&g, "start_s");
    assert!(ff.follow(s).unwrap().contains(0));
}

#[test]
fn specific_multiple_tokens_first() {
    // S → a | b | c
    let mut g = GrammarBuilder::new("multi")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("start_s", vec!["a"])
        .rule("start_s", vec!["b"])
        .rule("start_s", vec!["c"])
        .start("start_s")
        .build();
    let ff = FirstFollowSets::compute_normalized(&mut g).unwrap();
    let s = sym(&g, "start_s");
    assert_eq!(ff.first(s).unwrap().count_ones(..), 3);
}

#[test]
fn specific_two_nts_follow_propagation() {
    // S → A B, A → a, B → b
    let mut g = GrammarBuilder::new("twofollow")
        .token("a", "a")
        .token("b", "b")
        .rule("start_s", vec!["nt_a", "nt_b"])
        .rule("nt_a", vec!["a"])
        .rule("nt_b", vec!["b"])
        .start("start_s")
        .build();
    let ff = FirstFollowSets::compute_normalized(&mut g).unwrap();
    let a_nt = sym(&g, "nt_a");
    let b_nt = sym(&g, "nt_b");
    let b_tok = sym(&g, "b");
    // FOLLOW(A) must contain FIRST(B) = {b}
    assert!(ff.follow(a_nt).unwrap().contains(b_tok.0 as usize));
    // FOLLOW(B) must contain FOLLOW(S) which includes EOF
    assert!(ff.follow(b_nt).unwrap().contains(0));
}

// =========================================================================
// 5. Linear grammar FIRST/FOLLOW (5 tests)
// =========================================================================

#[test]
fn linear_right_linear() {
    // S → a S | a  (right-linear)
    let mut g = GrammarBuilder::new("rlin")
        .token("a", "a")
        .rule("start_s", vec!["a", "start_s"])
        .rule("start_s", vec!["a"])
        .start("start_s")
        .build();
    let ff = FirstFollowSets::compute_normalized(&mut g).unwrap();
    let s = sym(&g, "start_s");
    let a = sym(&g, "a");
    assert!(ff.first(s).unwrap().contains(a.0 as usize));
    assert_eq!(
        ff.first(s).unwrap().count_ones(..),
        1,
        "only 'a' in FIRST(S)"
    );
}

#[test]
fn linear_left_linear() {
    // S → S a | a  (left-linear)
    let mut g = GrammarBuilder::new("llin")
        .token("a", "a")
        .rule("start_s", vec!["start_s", "a"])
        .rule("start_s", vec!["a"])
        .start("start_s")
        .build();
    let ff = FirstFollowSets::compute_normalized(&mut g).unwrap();
    let s = sym(&g, "start_s");
    let a = sym(&g, "a");
    assert!(ff.first(s).unwrap().contains(a.0 as usize));
}

#[test]
fn linear_two_token_chain() {
    // S → a B, B → b
    let mut g = GrammarBuilder::new("chain2")
        .token("a", "a")
        .token("b", "b")
        .rule("start_s", vec!["a", "nt_b"])
        .rule("nt_b", vec!["b"])
        .start("start_s")
        .build();
    let ff = FirstFollowSets::compute_normalized(&mut g).unwrap();
    let s = sym(&g, "start_s");
    let a = sym(&g, "a");
    let b = sym(&g, "b");
    assert!(ff.first(s).unwrap().contains(a.0 as usize));
    assert!(!ff.first(s).unwrap().contains(b.0 as usize));
}

#[test]
fn linear_three_token_sequence() {
    // S → a b c
    let mut g = GrammarBuilder::new("seq3")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("start_s", vec!["a", "b", "c"])
        .start("start_s")
        .build();
    let ff = FirstFollowSets::compute_normalized(&mut g).unwrap();
    let s = sym(&g, "start_s");
    let a = sym(&g, "a");
    // FIRST(S) = {a} only
    assert!(ff.first(s).unwrap().contains(a.0 as usize));
    assert_eq!(ff.first(s).unwrap().count_ones(..), 1);
}

#[test]
fn linear_right_recursive_follow() {
    // S → a S | b
    let mut g = GrammarBuilder::new("rrf")
        .token("a", "a")
        .token("b", "b")
        .rule("start_s", vec!["a", "start_s"])
        .rule("start_s", vec!["b"])
        .start("start_s")
        .build();
    let ff = FirstFollowSets::compute_normalized(&mut g).unwrap();
    let s = sym(&g, "start_s");
    let a = sym(&g, "a");
    let b = sym(&g, "b");
    let first_s = ff.first(s).unwrap();
    assert!(first_s.contains(a.0 as usize));
    assert!(first_s.contains(b.0 as usize));
}

// =========================================================================
// 6. Recursive grammar FIRST/FOLLOW (5 tests)
// =========================================================================

#[test]
fn recursive_direct_left() {
    // E → E plus T | T, T → num
    let mut g = GrammarBuilder::new("dlr")
        .token("plus", "\\+")
        .token("num", "[0-9]+")
        .rule("nt_e", vec!["nt_e", "plus", "nt_t"])
        .rule("nt_e", vec!["nt_t"])
        .rule("nt_t", vec!["num"])
        .start("nt_e")
        .build();
    let ff = FirstFollowSets::compute_normalized(&mut g).unwrap();
    let e = sym(&g, "nt_e");
    let num = sym(&g, "num");
    let plus = sym(&g, "plus");
    assert!(ff.first(e).unwrap().contains(num.0 as usize));
    // FOLLOW(E) should contain {plus, EOF}
    let follow_e = ff.follow(e).unwrap();
    assert!(follow_e.contains(0), "FOLLOW(E) must have EOF");
    assert!(
        follow_e.contains(plus.0 as usize),
        "FOLLOW(E) must have plus"
    );
}

#[test]
fn recursive_indirect() {
    // A → B c, B → A d | e
    let mut g = GrammarBuilder::new("indirect")
        .token("c", "c")
        .token("d", "d")
        .token("e", "e")
        .rule("nt_a", vec!["nt_b", "c"])
        .rule("nt_b", vec!["nt_a", "d"])
        .rule("nt_b", vec!["e"])
        .start("nt_a")
        .build();
    let ff = FirstFollowSets::compute_normalized(&mut g).unwrap();
    let a = sym(&g, "nt_a");
    let e_tok = sym(&g, "e");
    // FIRST(A) = FIRST(B) = {e}
    assert!(ff.first(a).unwrap().contains(e_tok.0 as usize));
}

#[test]
fn recursive_mutual() {
    // A → B a, B → A b | c
    let mut g = GrammarBuilder::new("mutual")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("nt_a", vec!["nt_b", "a"])
        .rule("nt_b", vec!["nt_a", "b"])
        .rule("nt_b", vec!["c"])
        .start("nt_a")
        .build();
    let ff = FirstFollowSets::compute_normalized(&mut g).unwrap();
    let a_nt = sym(&g, "nt_a");
    let b_nt = sym(&g, "nt_b");
    let c = sym(&g, "c");
    assert!(ff.first(a_nt).unwrap().contains(c.0 as usize));
    assert!(ff.first(b_nt).unwrap().contains(c.0 as usize));
}

#[test]
fn recursive_right() {
    // S → a S | a
    let mut g = GrammarBuilder::new("right_rec")
        .token("a", "a")
        .rule("start_s", vec!["a", "start_s"])
        .rule("start_s", vec!["a"])
        .start("start_s")
        .build();
    let ff = FirstFollowSets::compute_normalized(&mut g).unwrap();
    let s = sym(&g, "start_s");
    assert!(!ff.is_nullable(s));
    assert!(ff.follow(s).unwrap().contains(0));
}

#[test]
fn recursive_nested_expression() {
    // E → lp E rp | num
    let mut g = GrammarBuilder::new("nested")
        .token("lp", "\\(")
        .token("rp", "\\)")
        .token("num", "[0-9]+")
        .rule("nt_e", vec!["lp", "nt_e", "rp"])
        .rule("nt_e", vec!["num"])
        .start("nt_e")
        .build();
    let ff = FirstFollowSets::compute_normalized(&mut g).unwrap();
    let e = sym(&g, "nt_e");
    let lp = sym(&g, "lp");
    let rp = sym(&g, "rp");
    let num = sym(&g, "num");
    let first_e = ff.first(e).unwrap();
    assert!(first_e.contains(lp.0 as usize));
    assert!(first_e.contains(num.0 as usize));
    // FOLLOW(E) should contain {rp, EOF}
    let follow_e = ff.follow(e).unwrap();
    assert!(follow_e.contains(rp.0 as usize));
    assert!(follow_e.contains(0));
}

// =========================================================================
// 7. Complex grammar FIRST/FOLLOW (5 tests)
// =========================================================================

#[test]
fn complex_expression_with_terms_and_factors() {
    // E → E plus T | T
    // T → T star F | F
    // F → lp E rp | id
    let mut g = GrammarBuilder::new("expr")
        .token("plus", "\\+")
        .token("star", "\\*")
        .token("lp", "\\(")
        .token("rp", "\\)")
        .token("id", "[a-z]+")
        .rule("nt_e", vec!["nt_e", "plus", "nt_t"])
        .rule("nt_e", vec!["nt_t"])
        .rule("nt_t", vec!["nt_t", "star", "nt_f"])
        .rule("nt_t", vec!["nt_f"])
        .rule("nt_f", vec!["lp", "nt_e", "rp"])
        .rule("nt_f", vec!["id"])
        .start("nt_e")
        .build();
    let ff = FirstFollowSets::compute_normalized(&mut g).unwrap();
    let e = sym(&g, "nt_e");
    let t = sym(&g, "nt_t");
    let f_nt = sym(&g, "nt_f");
    let lp = sym(&g, "lp");
    let id = sym(&g, "id");
    let plus = sym(&g, "plus");
    let star = sym(&g, "star");
    let rp = sym(&g, "rp");

    // FIRST(E) = FIRST(T) = FIRST(F) = {lp, id}
    for nt in [e, t, f_nt] {
        let first = ff.first(nt).unwrap();
        assert!(first.contains(lp.0 as usize), "FIRST should contain lp");
        assert!(first.contains(id.0 as usize), "FIRST should contain id");
    }
    // FOLLOW(E) = {plus, rp, EOF}
    let follow_e = ff.follow(e).unwrap();
    assert!(follow_e.contains(plus.0 as usize));
    assert!(follow_e.contains(rp.0 as usize));
    assert!(follow_e.contains(0));
    // FOLLOW(T) = {plus, star, rp, EOF}
    let follow_t = ff.follow(t).unwrap();
    assert!(follow_t.contains(plus.0 as usize));
    assert!(follow_t.contains(star.0 as usize));
}

#[test]
fn complex_if_then_else() {
    // S → if E then S else S | if E then S | assign
    // E → id
    let mut g = GrammarBuilder::new("ite")
        .token("if_kw", "if")
        .token("then_kw", "then")
        .token("else_kw", "else")
        .token("assign", "assign")
        .token("id", "[a-z]+")
        .rule(
            "start_s",
            vec!["if_kw", "nt_e", "then_kw", "start_s", "else_kw", "start_s"],
        )
        .rule("start_s", vec!["if_kw", "nt_e", "then_kw", "start_s"])
        .rule("start_s", vec!["assign"])
        .rule("nt_e", vec!["id"])
        .start("start_s")
        .build();
    let ff = FirstFollowSets::compute_normalized(&mut g).unwrap();
    let s = sym(&g, "start_s");
    let if_kw = sym(&g, "if_kw");
    let assign = sym(&g, "assign");
    let first_s = ff.first(s).unwrap();
    assert!(first_s.contains(if_kw.0 as usize));
    assert!(first_s.contains(assign.0 as usize));
}

#[test]
fn complex_list_grammar() {
    // L → L comma item | item
    let mut g = GrammarBuilder::new("list")
        .token("comma", ",")
        .token("item", "[a-z]+")
        .rule("nt_l", vec!["nt_l", "comma", "item"])
        .rule("nt_l", vec!["item"])
        .start("nt_l")
        .build();
    let ff = FirstFollowSets::compute_normalized(&mut g).unwrap();
    let l = sym(&g, "nt_l");
    let item = sym(&g, "item");
    let comma = sym(&g, "comma");
    assert!(ff.first(l).unwrap().contains(item.0 as usize));
    let follow_l = ff.follow(l).unwrap();
    assert!(follow_l.contains(comma.0 as usize));
    assert!(follow_l.contains(0));
}

#[test]
fn complex_multiple_nonterminals_follow() {
    // S → A B C, A → a, B → b, C → c
    let mut g = GrammarBuilder::new("abc")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("start_s", vec!["nt_a", "nt_b", "nt_c"])
        .rule("nt_a", vec!["a"])
        .rule("nt_b", vec!["b"])
        .rule("nt_c", vec!["c"])
        .start("start_s")
        .build();
    let ff = FirstFollowSets::compute_normalized(&mut g).unwrap();
    let a_nt = sym(&g, "nt_a");
    let b_nt = sym(&g, "nt_b");
    let c_nt = sym(&g, "nt_c");
    let b_tok = sym(&g, "b");
    let c_tok = sym(&g, "c");
    // FOLLOW(A) = {b}, FOLLOW(B) = {c}, FOLLOW(C) = {EOF}
    assert!(ff.follow(a_nt).unwrap().contains(b_tok.0 as usize));
    assert!(ff.follow(b_nt).unwrap().contains(c_tok.0 as usize));
    assert!(ff.follow(c_nt).unwrap().contains(0));
}

#[test]
fn complex_arithmetic_with_unary() {
    // E → E plus T | T
    // T → minus T | F
    // F → num
    let mut g = GrammarBuilder::new("unary")
        .token("plus", "\\+")
        .token("minus", "-")
        .token("num", "[0-9]+")
        .rule("nt_e", vec!["nt_e", "plus", "nt_t"])
        .rule("nt_e", vec!["nt_t"])
        .rule("nt_t", vec!["minus", "nt_t"])
        .rule("nt_t", vec!["nt_f"])
        .rule("nt_f", vec!["num"])
        .start("nt_e")
        .build();
    let ff = FirstFollowSets::compute_normalized(&mut g).unwrap();
    let e = sym(&g, "nt_e");
    let t = sym(&g, "nt_t");
    let minus = sym(&g, "minus");
    let num = sym(&g, "num");
    let first_e = ff.first(e).unwrap();
    assert!(
        first_e.contains(minus.0 as usize),
        "FIRST(E) includes minus"
    );
    assert!(first_e.contains(num.0 as usize), "FIRST(E) includes num");
    let first_t = ff.first(t).unwrap();
    assert!(first_t.contains(minus.0 as usize));
    assert!(first_t.contains(num.0 as usize));
}

// =========================================================================
// 8. Edge cases (6 tests)
// =========================================================================

#[test]
fn edge_single_token_grammar() {
    let mut g = GrammarBuilder::new("one")
        .token("x", "x")
        .rule("start_s", vec!["x"])
        .start("start_s")
        .build();
    let ff = FirstFollowSets::compute_normalized(&mut g).unwrap();
    let s = sym(&g, "start_s");
    assert!(ff.first(s).unwrap().count_ones(..) == 1);
}

#[test]
fn edge_deeply_nested_chain() {
    // S → A, A → B, B → C, C → D, D → x
    let mut g = GrammarBuilder::new("deep5")
        .token("x", "x")
        .rule("start_s", vec!["nt_a"])
        .rule("nt_a", vec!["nt_b"])
        .rule("nt_b", vec!["nt_c"])
        .rule("nt_c", vec!["nt_d"])
        .rule("nt_d", vec!["x"])
        .start("start_s")
        .build();
    let ff = FirstFollowSets::compute_normalized(&mut g).unwrap();
    let s = sym(&g, "start_s");
    let x = sym(&g, "x");
    assert!(ff.first(s).unwrap().contains(x.0 as usize));
}

#[test]
fn edge_many_alternatives() {
    // S → a | b | c | d
    let mut g = GrammarBuilder::new("many")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .token("d", "d")
        .rule("start_s", vec!["a"])
        .rule("start_s", vec!["b"])
        .rule("start_s", vec!["c"])
        .rule("start_s", vec!["d"])
        .start("start_s")
        .build();
    let ff = FirstFollowSets::compute_normalized(&mut g).unwrap();
    let s = sym(&g, "start_s");
    assert_eq!(ff.first(s).unwrap().count_ones(..), 4);
}

#[test]
fn edge_identical_productions() {
    // S → a, S → a (duplicated)
    let mut g = GrammarBuilder::new("dup")
        .token("a", "a")
        .rule("start_s", vec!["a"])
        .rule("start_s", vec!["a"])
        .start("start_s")
        .build();
    let ff = FirstFollowSets::compute_normalized(&mut g).unwrap();
    let s = sym(&g, "start_s");
    let a = sym(&g, "a");
    assert!(ff.first(s).unwrap().contains(a.0 as usize));
    assert_eq!(ff.first(s).unwrap().count_ones(..), 1);
}

#[test]
fn edge_long_rhs() {
    // S → a b c d
    let mut g = GrammarBuilder::new("long")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .token("d", "d")
        .rule("start_s", vec!["a", "b", "c", "d"])
        .start("start_s")
        .build();
    let ff = FirstFollowSets::compute_normalized(&mut g).unwrap();
    let s = sym(&g, "start_s");
    let a = sym(&g, "a");
    let first_s = ff.first(s).unwrap();
    assert!(first_s.contains(a.0 as usize));
    assert_eq!(first_s.count_ones(..), 1, "only first terminal in FIRST");
}

#[test]
fn edge_self_recursive_only() {
    // S → S a | b (direct left recursion)
    let mut g = GrammarBuilder::new("selfrec")
        .token("a", "a")
        .token("b", "b")
        .rule("start_s", vec!["start_s", "a"])
        .rule("start_s", vec!["b"])
        .start("start_s")
        .build();
    let ff = FirstFollowSets::compute_normalized(&mut g).unwrap();
    let s = sym(&g, "start_s");
    let b = sym(&g, "b");
    let a = sym(&g, "a");
    // FIRST(S) = {b} (from the base case)
    assert!(ff.first(s).unwrap().contains(b.0 as usize));
    // FOLLOW(S) should contain {a, EOF}
    let follow_s = ff.follow(s).unwrap();
    assert!(follow_s.contains(a.0 as usize));
    assert!(follow_s.contains(0));
}
