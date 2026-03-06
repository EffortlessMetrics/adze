//! Property-based tests for FIRST/FOLLOW set computation (v5).
//!
//! Run with: `cargo test -p adze-glr-core --test proptest_first_follow_v5 -- --test-threads=2`

use adze_glr_core::FirstFollowSets;
use adze_ir::builder::GrammarBuilder;
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
                        .map(|s| match s {
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

/// Grammar with explicit start symbol set (first NT).
fn arb_grammar_with_start() -> impl Strategy<Value = Grammar> {
    arb_small_grammar().prop_map(|mut g| {
        if let Some(&first_nt) = g.rules.keys().next() {
            g.rule_names
                .entry(first_nt)
                .or_insert_with(|| "source_file".into());
            // Ensure start symbol heuristic works by naming it source_file
            let old_name = g.rule_names.get(&first_nt).cloned();
            if let Some(name) = old_name
                && name != "source_file"
            {
                g.rule_names.insert(first_nt, "source_file".into());
            }
        }
        g
    })
}

// =========================================================================
// 1. FIRST sets are non-empty for non-epsilon rules (5 properties)
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    /// FIRST(S) is non-empty when S has a terminal production.
    #[test]
    fn v5_first_nonempty_single_terminal(t_id in 1..=MAX_TERM) {
        let s = SymbolId(NT_BASE);
        let mut g = Grammar::new("ne1".into());
        tok(&mut g, SymbolId(t_id), &format!("t{t_id}"), "x");
        g.rule_names.insert(s, "S".into());
        g.rules.entry(s).or_default().push(
            make_rule(s, vec![Symbol::Terminal(SymbolId(t_id))], 0),
        );
        let ff = FirstFollowSets::compute(&g).unwrap();
        let first_s = ff.first(s).unwrap();
        prop_assert!(first_s.count_ones(..) > 0);
    }

    /// FIRST(S) is non-empty when S has two terminal alternatives.
    #[test]
    fn v5_first_nonempty_two_alternatives(t1 in 1..=4u16, t2 in 5..=MAX_TERM) {
        let s = SymbolId(NT_BASE);
        let mut g = Grammar::new("ne2".into());
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
        prop_assert!(ff.first(s).unwrap().count_ones(..) >= 2);
    }

    /// FIRST(A) is non-empty when A chains to a terminal through B.
    #[test]
    fn v5_first_nonempty_chain(t_id in 1..=MAX_TERM) {
        let a = SymbolId(NT_BASE);
        let b = SymbolId(NT_BASE + 1);
        let mut g = Grammar::new("ne3".into());
        tok(&mut g, SymbolId(t_id), &format!("t{t_id}"), "x");
        g.rule_names.insert(a, "A".into());
        g.rule_names.insert(b, "B".into());
        g.rules.entry(a).or_default().push(
            make_rule(a, vec![Symbol::NonTerminal(b)], 0),
        );
        g.rules.entry(b).or_default().push(
            make_rule(b, vec![Symbol::Terminal(SymbolId(t_id))], 1),
        );
        let ff = FirstFollowSets::compute(&g).unwrap();
        prop_assert!(ff.first(a).unwrap().count_ones(..) > 0);
    }

    /// FIRST(S) is non-empty for S with sequence starting with terminal.
    #[test]
    fn v5_first_nonempty_sequence_start(t1 in 1..=4u16, t2 in 5..=MAX_TERM) {
        let s = SymbolId(NT_BASE);
        let mut g = Grammar::new("ne4".into());
        tok(&mut g, SymbolId(t1), &format!("t{t1}"), "a");
        tok(&mut g, SymbolId(t2), &format!("t{t2}"), "b");
        g.rule_names.insert(s, "S".into());
        g.rules.entry(s).or_default().push(
            make_rule(s, vec![Symbol::Terminal(SymbolId(t1)), Symbol::Terminal(SymbolId(t2))], 0),
        );
        let ff = FirstFollowSets::compute(&g).unwrap();
        prop_assert!(ff.first(s).unwrap().count_ones(..) > 0);
    }

    /// FIRST(S) is non-empty even when S also has epsilon (the terminal alt contributes).
    #[test]
    fn v5_first_nonempty_with_epsilon_alt(t_id in 1..=MAX_TERM) {
        let s = SymbolId(NT_BASE);
        let mut g = Grammar::new("ne5".into());
        tok(&mut g, SymbolId(t_id), &format!("t{t_id}"), "x");
        g.rule_names.insert(s, "S".into());
        g.rules.entry(s).or_default().push(make_rule(s, vec![Symbol::Epsilon], 0));
        g.rules.entry(s).or_default().push(
            make_rule(s, vec![Symbol::Terminal(SymbolId(t_id))], 1),
        );
        let ff = FirstFollowSets::compute(&g).unwrap();
        prop_assert!(ff.first(s).unwrap().count_ones(..) > 0);
    }
}

// =========================================================================
// 2. FIRST sets contain only terminal symbols (5 properties)
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    /// All bits in FIRST(S) correspond to valid terminal IDs.
    #[test]
    fn v5_first_only_terminals_single_rule(t_id in 1..=MAX_TERM) {
        let s = SymbolId(NT_BASE);
        let mut g = Grammar::new("fo1".into());
        tok(&mut g, SymbolId(t_id), &format!("t{t_id}"), "x");
        g.rule_names.insert(s, "S".into());
        g.rules.entry(s).or_default().push(
            make_rule(s, vec![Symbol::Terminal(SymbolId(t_id))], 0),
        );
        let ff = FirstFollowSets::compute(&g).unwrap();
        for bit in ff.first(s).unwrap().ones() {
            prop_assert!(
                g.tokens.contains_key(&SymbolId(bit as u16)),
                "FIRST bit {bit} is not a terminal"
            );
        }
    }

    /// FIRST bits on a chain grammar only contain terminals.
    #[test]
    fn v5_first_only_terminals_chain(t_id in 1..=MAX_TERM) {
        let a = SymbolId(NT_BASE);
        let b = SymbolId(NT_BASE + 1);
        let mut g = Grammar::new("fo2".into());
        tok(&mut g, SymbolId(t_id), &format!("t{t_id}"), "x");
        g.rule_names.insert(a, "A".into());
        g.rule_names.insert(b, "B".into());
        g.rules.entry(a).or_default().push(
            make_rule(a, vec![Symbol::NonTerminal(b)], 0),
        );
        g.rules.entry(b).or_default().push(
            make_rule(b, vec![Symbol::Terminal(SymbolId(t_id))], 1),
        );
        let ff = FirstFollowSets::compute(&g).unwrap();
        for bit in ff.first(a).unwrap().ones() {
            prop_assert!(
                g.tokens.contains_key(&SymbolId(bit as u16)),
                "FIRST(A) bit {bit} is not a terminal"
            );
        }
    }

    /// FIRST bits for a multi-alt grammar are all terminals.
    #[test]
    fn v5_first_only_terminals_multi_alt(t1 in 1..=4u16, t2 in 5..=MAX_TERM) {
        let s = SymbolId(NT_BASE);
        let mut g = Grammar::new("fo3".into());
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
        for bit in ff.first(s).unwrap().ones() {
            prop_assert!(g.tokens.contains_key(&SymbolId(bit as u16)));
        }
    }

    /// FIRST bits through nullable prefix are terminals.
    #[test]
    fn v5_first_only_terminals_nullable_prefix(t1 in 1..=4u16, t2 in 5..=MAX_TERM) {
        let a = SymbolId(NT_BASE);
        let b = SymbolId(NT_BASE + 1);
        let mut g = Grammar::new("fo4".into());
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
        for bit in ff.first(a).unwrap().ones() {
            prop_assert!(g.tokens.contains_key(&SymbolId(bit as u16)));
        }
    }

    /// FIRST bits for arbitrary small grammars are all terminals.
    #[test]
    fn v5_first_only_terminals_arb_grammar(g in arb_small_grammar()) {
        if let Ok(ff) = FirstFollowSets::compute(&g) {
            for (nt_id, _) in &g.rules {
                if let Some(first_set) = ff.first(*nt_id) {
                    for bit in first_set.ones() {
                        prop_assert!(
                            g.tokens.contains_key(&SymbolId(bit as u16)),
                            "FIRST({nt_id:?}) bit {bit} is not a declared terminal"
                        );
                    }
                }
            }
        }
    }
}

// =========================================================================
// 3. FOLLOW set of start symbol contains EOF (5 properties)
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    /// FOLLOW(start) contains EOF for a single-rule grammar.
    #[test]
    fn v5_follow_start_eof_single(t_id in 1..=MAX_TERM) {
        let mut g = GrammarBuilder::new("eof1")
            .token("tk", "x")
            .rule("source_file", vec!["tk"])
            .start("source_file")
            .build();
        let ff = FirstFollowSets::compute_normalized(&mut g).unwrap();
        let s = sym(&g, "source_file");
        prop_assert!(ff.follow(s).unwrap().contains(0), "FOLLOW(start) must have EOF, t_id={t_id}");
    }

    /// FOLLOW(start) contains EOF for a two-alt grammar.
    #[test]
    fn v5_follow_start_eof_two_alt(t1 in 1..=4u16, t2 in 5..=MAX_TERM) {
        let mut g = GrammarBuilder::new("eof2")
            .token("a", "a")
            .token("b", "b")
            .rule("source_file", vec!["a"])
            .rule("source_file", vec!["b"])
            .start("source_file")
            .build();
        let ff = FirstFollowSets::compute_normalized(&mut g).unwrap();
        let s = sym(&g, "source_file");
        prop_assert!(ff.follow(s).unwrap().contains(0), "t1={t1}, t2={t2}");
    }

    /// FOLLOW(start) contains EOF for recursive grammar.
    #[test]
    fn v5_follow_start_eof_recursive(t_id in 1..=MAX_TERM) {
        let mut g = GrammarBuilder::new("eof3")
            .token("a", "a")
            .rule("source_file", vec!["a", "source_file"])
            .rule("source_file", vec!["a"])
            .start("source_file")
            .build();
        let ff = FirstFollowSets::compute_normalized(&mut g).unwrap();
        let s = sym(&g, "source_file");
        prop_assert!(ff.follow(s).unwrap().contains(0), "t_id={t_id}");
    }

    /// FOLLOW(start) contains EOF for chain grammar.
    #[test]
    fn v5_follow_start_eof_chain(t_id in 1..=MAX_TERM) {
        let mut g = GrammarBuilder::new("eof4")
            .token("x", "x")
            .rule("source_file", vec!["nt_inner"])
            .rule("nt_inner", vec!["x"])
            .start("source_file")
            .build();
        let ff = FirstFollowSets::compute_normalized(&mut g).unwrap();
        let s = sym(&g, "source_file");
        prop_assert!(ff.follow(s).unwrap().contains(0), "t_id={t_id}");
    }

    /// FOLLOW(start) contains EOF for arbitrary grammar with start set.
    #[test]
    fn v5_follow_start_eof_arb(g in arb_grammar_with_start()) {
        if let Ok(ff) = FirstFollowSets::compute(&g)
            && let Some(start) = g.start_symbol()
            && let Some(follow) = ff.follow(start)
        {
            prop_assert!(follow.contains(0), "FOLLOW(start) must contain EOF");
        }
    }
}

// =========================================================================
// 4. FOLLOW sets are non-empty for reachable symbols (5 properties)
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    /// FOLLOW(B) is non-empty when B appears before a terminal: A → B t.
    #[test]
    fn v5_follow_nonempty_before_terminal(t1 in 1..=4u16, t2 in 5..=MAX_TERM) {
        let a = SymbolId(NT_BASE);
        let b = SymbolId(NT_BASE + 1);
        let mut g = Grammar::new("fn1".into());
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
        prop_assert!(ff.follow(b).unwrap().count_ones(..) > 0);
    }

    /// FOLLOW(B) is non-empty when B is at end: A → t B.
    #[test]
    fn v5_follow_nonempty_trailing(t1 in 1..=4u16, t2 in 5..=MAX_TERM) {
        let a = SymbolId(NT_BASE);
        let b = SymbolId(NT_BASE + 1);
        let mut g = Grammar::new("fn2".into());
        tok(&mut g, SymbolId(t1), &format!("t{t1}"), "x");
        tok(&mut g, SymbolId(t2), &format!("t{t2}"), "y");
        g.rule_names.insert(a, "source_file".into());
        g.rule_names.insert(b, "B".into());
        g.rules.entry(a).or_default().push(
            make_rule(a, vec![Symbol::Terminal(SymbolId(t1)), Symbol::NonTerminal(b)], 0),
        );
        g.rules.entry(b).or_default().push(
            make_rule(b, vec![Symbol::Terminal(SymbolId(t2))], 1),
        );
        let ff = FirstFollowSets::compute(&g).unwrap();
        // B is trailing in start rule, so FOLLOW(B) ⊇ FOLLOW(A) ⊇ {EOF}
        prop_assert!(ff.follow(b).unwrap().count_ones(..) > 0);
    }

    /// FOLLOW(B) non-empty when B is between two NTs: A → C B D.
    #[test]
    fn v5_follow_nonempty_middle(t1 in 1..=3u16, t2 in 4..=6u16, t3 in 7..=MAX_TERM) {
        let a = SymbolId(NT_BASE);
        let b = SymbolId(NT_BASE + 1);
        let c = SymbolId(NT_BASE + 2);
        let d = SymbolId(NT_BASE + 3);
        let mut g = Grammar::new("fn3".into());
        tok(&mut g, SymbolId(t1), &format!("t{t1}"), "a");
        tok(&mut g, SymbolId(t2), &format!("t{t2}"), "b");
        tok(&mut g, SymbolId(t3), &format!("t{t3}"), "c");
        g.rule_names.insert(a, "A".into());
        g.rule_names.insert(b, "B".into());
        g.rule_names.insert(c, "C".into());
        g.rule_names.insert(d, "D".into());
        g.rules.entry(a).or_default().push(
            make_rule(a, vec![
                Symbol::NonTerminal(c), Symbol::NonTerminal(b), Symbol::NonTerminal(d),
            ], 0),
        );
        g.rules.entry(b).or_default().push(make_rule(b, vec![Symbol::Terminal(SymbolId(t2))], 1));
        g.rules.entry(c).or_default().push(make_rule(c, vec![Symbol::Terminal(SymbolId(t1))], 2));
        g.rules.entry(d).or_default().push(make_rule(d, vec![Symbol::Terminal(SymbolId(t3))], 3));
        let ff = FirstFollowSets::compute(&g).unwrap();
        // FOLLOW(B) ⊇ FIRST(D) = {t3}
        prop_assert!(ff.follow(b).unwrap().count_ones(..) > 0);
    }

    /// FOLLOW(start) is always non-empty (contains EOF).
    #[test]
    fn v5_follow_nonempty_start(g in arb_grammar_with_start()) {
        if let Ok(ff) = FirstFollowSets::compute(&g)
            && let Some(start) = g.start_symbol()
            && let Some(follow) = ff.follow(start)
        {
            prop_assert!(follow.count_ones(..) > 0);
        }
    }

    /// FOLLOW(B) is non-empty when A → B C and C has a terminal.
    #[test]
    fn v5_follow_nonempty_before_nt(t1 in 1..=4u16, t2 in 5..=MAX_TERM) {
        let a = SymbolId(NT_BASE);
        let b = SymbolId(NT_BASE + 1);
        let c = SymbolId(NT_BASE + 2);
        let mut g = Grammar::new("fn5".into());
        tok(&mut g, SymbolId(t1), &format!("t{t1}"), "x");
        tok(&mut g, SymbolId(t2), &format!("t{t2}"), "y");
        g.rule_names.insert(a, "A".into());
        g.rule_names.insert(b, "B".into());
        g.rule_names.insert(c, "C".into());
        g.rules.entry(a).or_default().push(
            make_rule(a, vec![Symbol::NonTerminal(b), Symbol::NonTerminal(c)], 0),
        );
        g.rules.entry(b).or_default().push(make_rule(b, vec![Symbol::Terminal(SymbolId(t1))], 1));
        g.rules.entry(c).or_default().push(make_rule(c, vec![Symbol::Terminal(SymbolId(t2))], 2));
        let ff = FirstFollowSets::compute(&g).unwrap();
        // FOLLOW(B) ⊇ FIRST(C) = {t2}
        prop_assert!(ff.follow(b).unwrap().count_ones(..) > 0);
    }
}

// =========================================================================
// 5. FIRST/FOLLOW computation is deterministic (5 properties)
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    /// Two computations on the same grammar yield identical FIRST sets.
    #[test]
    fn v5_deterministic_first(g in arb_small_grammar()) {
        let r1 = FirstFollowSets::compute(&g);
        let r2 = FirstFollowSets::compute(&g);
        match (r1, r2) {
            (Ok(a), Ok(b)) => {
                for (id, _) in &g.rules {
                    prop_assert_eq!(a.first(*id), b.first(*id));
                }
            }
            (Err(_), Err(_)) => {}
            _ => prop_assert!(false, "inconsistent success/failure"),
        }
    }

    /// Two computations on the same grammar yield identical FOLLOW sets.
    #[test]
    fn v5_deterministic_follow(g in arb_small_grammar()) {
        let r1 = FirstFollowSets::compute(&g);
        let r2 = FirstFollowSets::compute(&g);
        match (r1, r2) {
            (Ok(a), Ok(b)) => {
                for (id, _) in &g.rules {
                    prop_assert_eq!(a.follow(*id), b.follow(*id));
                }
            }
            (Err(_), Err(_)) => {}
            _ => prop_assert!(false, "inconsistent success/failure"),
        }
    }

    /// Nullable is deterministic across runs.
    #[test]
    fn v5_deterministic_nullable(g in arb_small_grammar()) {
        let r1 = FirstFollowSets::compute(&g);
        let r2 = FirstFollowSets::compute(&g);
        match (r1, r2) {
            (Ok(a), Ok(b)) => {
                for (id, _) in &g.rules {
                    prop_assert_eq!(a.is_nullable(*id), b.is_nullable(*id));
                }
            }
            (Err(_), Err(_)) => {}
            _ => prop_assert!(false, "inconsistent success/failure"),
        }
    }

    /// compute_normalized is deterministic on cloned grammars.
    #[test]
    fn v5_deterministic_normalized(_t_id in 1..=MAX_TERM) {
        let base = GrammarBuilder::new("det_norm")
            .token("x", "x")
            .rule("source_file", vec!["x"])
            .start("source_file")
            .build();
        let mut g1 = base.clone();
        let mut g2 = base;
        let ff1 = FirstFollowSets::compute_normalized(&mut g1).unwrap();
        let ff2 = FirstFollowSets::compute_normalized(&mut g2).unwrap();
        let s1 = sym(&g1, "source_file");
        let s2 = sym(&g2, "source_file");
        prop_assert_eq!(ff1.first(s1), ff2.first(s2));
        prop_assert_eq!(ff1.follow(s1), ff2.follow(s2));
    }

    /// FIRST/FOLLOW sets exist for all NTs when compute succeeds.
    #[test]
    fn v5_deterministic_sets_exist(g in arb_small_grammar()) {
        if let Ok(ff) = FirstFollowSets::compute(&g) {
            for (nt_id, _) in &g.rules {
                prop_assert!(ff.first(*nt_id).is_some());
                prop_assert!(ff.follow(*nt_id).is_some());
            }
        }
    }
}

// =========================================================================
// 6. Terminal FIRST set is the terminal itself (5 properties)
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    /// S → t  ⟹  FIRST(S) = {t}.
    #[test]
    fn v5_terminal_first_exact(t_id in 1..=MAX_TERM) {
        let s = SymbolId(NT_BASE);
        let mut g = Grammar::new("tf1".into());
        tok(&mut g, SymbolId(t_id), &format!("t{t_id}"), "x");
        g.rule_names.insert(s, "S".into());
        g.rules.entry(s).or_default().push(
            make_rule(s, vec![Symbol::Terminal(SymbolId(t_id))], 0),
        );
        let ff = FirstFollowSets::compute(&g).unwrap();
        let first_s = ff.first(s).unwrap();
        prop_assert!(first_s.contains(t_id as usize));
        prop_assert_eq!(first_s.count_ones(..), 1, "only the one terminal should be in FIRST");
    }

    /// A → t1 t2  ⟹  FIRST(A) = {t1}.
    #[test]
    fn v5_terminal_first_sequence(t1 in 1..=4u16, t2 in 5..=MAX_TERM) {
        let s = SymbolId(NT_BASE);
        let mut g = Grammar::new("tf2".into());
        tok(&mut g, SymbolId(t1), &format!("t{t1}"), "a");
        tok(&mut g, SymbolId(t2), &format!("t{t2}"), "b");
        g.rule_names.insert(s, "S".into());
        g.rules.entry(s).or_default().push(
            make_rule(s, vec![Symbol::Terminal(SymbolId(t1)), Symbol::Terminal(SymbolId(t2))], 0),
        );
        let ff = FirstFollowSets::compute(&g).unwrap();
        let first_s = ff.first(s).unwrap();
        prop_assert!(first_s.contains(t1 as usize));
        prop_assert!(!first_s.contains(t2 as usize), "t2 should not be in FIRST(S→t1 t2)");
    }

    /// B → t  ⟹  t ∈ FIRST(A) when A → B.
    #[test]
    fn v5_terminal_first_through_unit(t_id in 1..=MAX_TERM) {
        let a = SymbolId(NT_BASE);
        let b = SymbolId(NT_BASE + 1);
        let t = SymbolId(t_id);
        let mut g = Grammar::new("tf3".into());
        tok(&mut g, t, &format!("t{t_id}"), "x");
        g.rule_names.insert(a, "A".into());
        g.rule_names.insert(b, "B".into());
        g.rules.entry(a).or_default().push(make_rule(a, vec![Symbol::NonTerminal(b)], 0));
        g.rules.entry(b).or_default().push(make_rule(b, vec![Symbol::Terminal(t)], 1));
        let ff = FirstFollowSets::compute(&g).unwrap();
        prop_assert!(ff.first(a).unwrap().contains(t_id as usize));
        prop_assert_eq!(ff.first(a).unwrap().count_ones(..), 1);
    }

    /// S → t | ε  ⟹  t ∈ FIRST(S) and |FIRST(S)| = 1.
    #[test]
    fn v5_terminal_first_with_epsilon(t_id in 1..=MAX_TERM) {
        let s = SymbolId(NT_BASE);
        let mut g = Grammar::new("tf4".into());
        tok(&mut g, SymbolId(t_id), &format!("t{t_id}"), "x");
        g.rule_names.insert(s, "S".into());
        g.rules.entry(s).or_default().push(make_rule(s, vec![Symbol::Epsilon], 0));
        g.rules.entry(s).or_default().push(
            make_rule(s, vec![Symbol::Terminal(SymbolId(t_id))], 1),
        );
        let ff = FirstFollowSets::compute(&g).unwrap();
        let first_s = ff.first(s).unwrap();
        prop_assert!(first_s.contains(t_id as usize));
        // Epsilon is not a terminal symbol; FIRST set should only have the terminal.
        prop_assert_eq!(first_s.count_ones(..), 1);
    }

    /// Right-recursive: S → t S | t  ⟹  FIRST(S) = {t}.
    #[test]
    fn v5_terminal_first_right_recursive(t_id in 1..=MAX_TERM) {
        let s = SymbolId(NT_BASE);
        let mut g = Grammar::new("tf5".into());
        tok(&mut g, SymbolId(t_id), &format!("t{t_id}"), "x");
        g.rule_names.insert(s, "S".into());
        g.rules.entry(s).or_default().push(
            make_rule(s, vec![Symbol::Terminal(SymbolId(t_id)), Symbol::NonTerminal(s)], 0),
        );
        g.rules.entry(s).or_default().push(
            make_rule(s, vec![Symbol::Terminal(SymbolId(t_id))], 1),
        );
        let ff = FirstFollowSets::compute(&g).unwrap();
        prop_assert_eq!(ff.first(s).unwrap().count_ones(..), 1);
        prop_assert!(ff.first(s).unwrap().contains(t_id as usize));
    }
}

// =========================================================================
// 7. FIRST/FOLLOW survive normalize (5 properties)
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    /// compute and compute_normalized agree on FIRST for simple grammars.
    #[test]
    fn v5_normalize_preserves_first(t_id in 1..=MAX_TERM) {
        let s = SymbolId(NT_BASE);
        let mut g = Grammar::new("np1".into());
        tok(&mut g, SymbolId(t_id), &format!("t{t_id}"), "x");
        g.rule_names.insert(s, "S".into());
        g.rules.entry(s).or_default().push(
            make_rule(s, vec![Symbol::Terminal(SymbolId(t_id))], 0),
        );
        let ff_plain = FirstFollowSets::compute(&g).unwrap();
        let ff_norm = FirstFollowSets::compute_normalized(&mut g.clone()).unwrap();
        prop_assert_eq!(
            ff_plain.first(s).unwrap().count_ones(..),
            ff_norm.first(s).unwrap().count_ones(..),
        );
    }

    /// compute and compute_normalized agree on nullable.
    #[test]
    fn v5_normalize_preserves_nullable(t_id in 1..=MAX_TERM) {
        let s = SymbolId(NT_BASE);
        let mut g = Grammar::new("np2".into());
        tok(&mut g, SymbolId(t_id), &format!("t{t_id}"), "x");
        g.rule_names.insert(s, "S".into());
        g.rules.entry(s).or_default().push(make_rule(s, vec![Symbol::Epsilon], 0));
        g.rules.entry(s).or_default().push(
            make_rule(s, vec![Symbol::Terminal(SymbolId(t_id))], 1),
        );
        let ff_plain = FirstFollowSets::compute(&g).unwrap();
        let ff_norm = FirstFollowSets::compute_normalized(&mut g.clone()).unwrap();
        prop_assert_eq!(ff_plain.is_nullable(s), ff_norm.is_nullable(s));
    }

    /// compute_normalized doesn't panic on chain grammars.
    #[test]
    fn v5_normalize_chain_no_panic(t_id in 1..=MAX_TERM) {
        let mut g = GrammarBuilder::new("np3")
            .token("x", "x")
            .rule("source_file", vec!["nt_inner"])
            .rule("nt_inner", vec!["x"])
            .start("source_file")
            .build();
        let _ = FirstFollowSets::compute_normalized(&mut g);
        prop_assert!(true, "t_id={t_id}");
    }

    /// FIRST cardinality is preserved through normalization for terminal rules.
    #[test]
    fn v5_normalize_preserves_first_card(t1 in 1..=4u16, t2 in 5..=MAX_TERM) {
        let s = SymbolId(NT_BASE);
        let mut g = Grammar::new("np4".into());
        tok(&mut g, SymbolId(t1), &format!("t{t1}"), "a");
        tok(&mut g, SymbolId(t2), &format!("t{t2}"), "b");
        g.rule_names.insert(s, "S".into());
        g.rules.entry(s).or_default().push(
            make_rule(s, vec![Symbol::Terminal(SymbolId(t1))], 0),
        );
        g.rules.entry(s).or_default().push(
            make_rule(s, vec![Symbol::Terminal(SymbolId(t2))], 1),
        );
        let ff_plain = FirstFollowSets::compute(&g).unwrap();
        let ff_norm = FirstFollowSets::compute_normalized(&mut g.clone()).unwrap();
        prop_assert_eq!(
            ff_plain.first(s).unwrap().count_ones(..),
            ff_norm.first(s).unwrap().count_ones(..),
        );
    }

    /// FOLLOW(start) has EOF after normalization.
    #[test]
    fn v5_normalize_follow_eof(t_id in 1..=MAX_TERM) {
        let mut g = GrammarBuilder::new("np5")
            .token("x", "x")
            .rule("source_file", vec!["x"])
            .start("source_file")
            .build();
        let ff = FirstFollowSets::compute_normalized(&mut g).unwrap();
        let s = sym(&g, "source_file");
        prop_assert!(ff.follow(s).unwrap().contains(0), "t_id={t_id}");
    }
}

// =========================================================================
// 8. Consistency: FIRST(A) subset of possible tokens in A-rules (5 properties)
// =========================================================================

/// Collect all terminal SymbolIds that appear (directly or transitively) in the
/// RHS of rules for a given nonterminal, using a simple BFS.
fn reachable_terminals(g: &Grammar, nt: SymbolId) -> Vec<u16> {
    let mut visited = std::collections::HashSet::new();
    let mut queue = std::collections::VecDeque::new();
    let mut terminals = Vec::new();
    queue.push_back(nt);
    visited.insert(nt);
    while let Some(current) = queue.pop_front() {
        if let Some(rules) = g.rules.get(&current) {
            for rule in rules {
                for sym in &rule.rhs {
                    match sym {
                        Symbol::Terminal(id) => {
                            terminals.push(id.0);
                        }
                        Symbol::NonTerminal(id) => {
                            if visited.insert(*id) {
                                queue.push_back(*id);
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }
    terminals.sort();
    terminals.dedup();
    terminals
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    /// FIRST(S) ⊆ reachable terminals from S (single terminal rule).
    #[test]
    fn v5_first_subset_reachable_single(t_id in 1..=MAX_TERM) {
        let s = SymbolId(NT_BASE);
        let mut g = Grammar::new("sub1".into());
        tok(&mut g, SymbolId(t_id), &format!("t{t_id}"), "x");
        g.rule_names.insert(s, "S".into());
        g.rules.entry(s).or_default().push(
            make_rule(s, vec![Symbol::Terminal(SymbolId(t_id))], 0),
        );
        let ff = FirstFollowSets::compute(&g).unwrap();
        let reach = reachable_terminals(&g, s);
        for bit in ff.first(s).unwrap().ones() {
            prop_assert!(
                reach.contains(&(bit as u16)),
                "FIRST bit {bit} not in reachable terminals"
            );
        }
    }

    /// FIRST(A) ⊆ reachable terminals from A (chain rule).
    #[test]
    fn v5_first_subset_reachable_chain(t_id in 1..=MAX_TERM) {
        let a = SymbolId(NT_BASE);
        let b = SymbolId(NT_BASE + 1);
        let mut g = Grammar::new("sub2".into());
        tok(&mut g, SymbolId(t_id), &format!("t{t_id}"), "x");
        g.rule_names.insert(a, "A".into());
        g.rule_names.insert(b, "B".into());
        g.rules.entry(a).or_default().push(make_rule(a, vec![Symbol::NonTerminal(b)], 0));
        g.rules.entry(b).or_default().push(make_rule(b, vec![Symbol::Terminal(SymbolId(t_id))], 1));
        let ff = FirstFollowSets::compute(&g).unwrap();
        let reach = reachable_terminals(&g, a);
        for bit in ff.first(a).unwrap().ones() {
            prop_assert!(reach.contains(&(bit as u16)));
        }
    }

    /// FIRST(S) ⊆ reachable terminals from S (multi-alternative).
    #[test]
    fn v5_first_subset_reachable_multi(t1 in 1..=4u16, t2 in 5..=MAX_TERM) {
        let s = SymbolId(NT_BASE);
        let mut g = Grammar::new("sub3".into());
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
        let reach = reachable_terminals(&g, s);
        for bit in ff.first(s).unwrap().ones() {
            prop_assert!(reach.contains(&(bit as u16)));
        }
    }

    /// FIRST(A) ⊆ reachable terminals for nullable-prefix grammar.
    #[test]
    fn v5_first_subset_reachable_nullable(t1 in 1..=4u16, t2 in 5..=MAX_TERM) {
        let a = SymbolId(NT_BASE);
        let b = SymbolId(NT_BASE + 1);
        let mut g = Grammar::new("sub4".into());
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
        let reach = reachable_terminals(&g, a);
        for bit in ff.first(a).unwrap().ones() {
            prop_assert!(reach.contains(&(bit as u16)));
        }
    }

    /// FIRST ⊆ reachable terminals for arbitrary grammars.
    #[test]
    fn v5_first_subset_reachable_arb(g in arb_small_grammar()) {
        if let Ok(ff) = FirstFollowSets::compute(&g) {
            for (nt_id, _) in &g.rules {
                if let Some(first_set) = ff.first(*nt_id) {
                    let reach = reachable_terminals(&g, *nt_id);
                    for bit in first_set.ones() {
                        prop_assert!(
                            reach.contains(&(bit as u16)),
                            "FIRST({nt_id:?}) bit {bit} not reachable"
                        );
                    }
                }
            }
        }
    }
}

// =========================================================================
// 9. Edge cases (6 properties)
// =========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    /// Epsilon-only grammar: S → ε. S is nullable, FIRST(S) is empty.
    #[test]
    fn v5_edge_epsilon_only(t_id in 1..=MAX_TERM) {
        let s = SymbolId(NT_BASE);
        let mut g = Grammar::new("edge1".into());
        // Need at least one token to make it a valid grammar structure
        tok(&mut g, SymbolId(t_id), &format!("t{t_id}"), "x");
        g.rule_names.insert(s, "S".into());
        g.rules.entry(s).or_default().push(make_rule(s, vec![Symbol::Epsilon], 0));
        let ff = FirstFollowSets::compute(&g).unwrap();
        prop_assert!(ff.is_nullable(s));
        prop_assert_eq!(ff.first(s).unwrap().count_ones(..), 0, "epsilon-only FIRST should be empty");
    }

    /// Self-recursive rule S → S does not cause infinite loop.
    #[test]
    fn v5_edge_self_recursive_no_hang(t_id in 1..=MAX_TERM) {
        let s = SymbolId(NT_BASE);
        let mut g = Grammar::new("edge2".into());
        tok(&mut g, SymbolId(t_id), &format!("t{t_id}"), "x");
        g.rule_names.insert(s, "S".into());
        g.rules.entry(s).or_default().push(
            make_rule(s, vec![Symbol::NonTerminal(s)], 0),
        );
        g.rules.entry(s).or_default().push(
            make_rule(s, vec![Symbol::Terminal(SymbolId(t_id))], 1),
        );
        // Should not hang
        let ff = FirstFollowSets::compute(&g).unwrap();
        prop_assert!(ff.first(s).unwrap().contains(t_id as usize));
    }

    /// Multiple epsilon rules: S → ε | ε | t.
    #[test]
    fn v5_edge_multi_epsilon(t_id in 1..=MAX_TERM) {
        let s = SymbolId(NT_BASE);
        let mut g = Grammar::new("edge3".into());
        tok(&mut g, SymbolId(t_id), &format!("t{t_id}"), "x");
        g.rule_names.insert(s, "S".into());
        g.rules.entry(s).or_default().push(make_rule(s, vec![Symbol::Epsilon], 0));
        g.rules.entry(s).or_default().push(make_rule(s, vec![Symbol::Epsilon], 1));
        g.rules.entry(s).or_default().push(
            make_rule(s, vec![Symbol::Terminal(SymbolId(t_id))], 2),
        );
        let ff = FirstFollowSets::compute(&g).unwrap();
        prop_assert!(ff.is_nullable(s));
        prop_assert!(ff.first(s).unwrap().contains(t_id as usize));
    }

    /// Long chain: A→B, B→C, C→D, D→t. FIRST(A) = {t}.
    #[test]
    fn v5_edge_long_chain(t_id in 1..=MAX_TERM) {
        let a = SymbolId(NT_BASE);
        let b = SymbolId(NT_BASE + 1);
        let c = SymbolId(NT_BASE + 2);
        let d = SymbolId(NT_BASE + 3);
        let mut g = Grammar::new("edge4".into());
        tok(&mut g, SymbolId(t_id), &format!("t{t_id}"), "x");
        g.rule_names.insert(a, "A".into());
        g.rule_names.insert(b, "B".into());
        g.rule_names.insert(c, "C".into());
        g.rule_names.insert(d, "D".into());
        g.rules.entry(a).or_default().push(make_rule(a, vec![Symbol::NonTerminal(b)], 0));
        g.rules.entry(b).or_default().push(make_rule(b, vec![Symbol::NonTerminal(c)], 1));
        g.rules.entry(c).or_default().push(make_rule(c, vec![Symbol::NonTerminal(d)], 2));
        g.rules.entry(d).or_default().push(make_rule(d, vec![Symbol::Terminal(SymbolId(t_id))], 3));
        let ff = FirstFollowSets::compute(&g).unwrap();
        prop_assert!(ff.first(a).unwrap().contains(t_id as usize));
        prop_assert_eq!(ff.first(a).unwrap().count_ones(..), 1);
    }

    /// Mutual recursion: A→B t1, B→A t2 | t3. Doesn't hang.
    #[test]
    fn v5_edge_mutual_recursion(t1 in 1..=3u16, t2 in 4..=6u16, t3 in 7..=MAX_TERM) {
        let a = SymbolId(NT_BASE);
        let b = SymbolId(NT_BASE + 1);
        let mut g = Grammar::new("edge5".into());
        tok(&mut g, SymbolId(t1), &format!("t{t1}"), "a");
        tok(&mut g, SymbolId(t2), &format!("t{t2}"), "b");
        tok(&mut g, SymbolId(t3), &format!("t{t3}"), "c");
        g.rule_names.insert(a, "A".into());
        g.rule_names.insert(b, "B".into());
        g.rules.entry(a).or_default().push(
            make_rule(a, vec![Symbol::NonTerminal(b), Symbol::Terminal(SymbolId(t1))], 0),
        );
        g.rules.entry(b).or_default().push(
            make_rule(b, vec![Symbol::NonTerminal(a), Symbol::Terminal(SymbolId(t2))], 1),
        );
        g.rules.entry(b).or_default().push(
            make_rule(b, vec![Symbol::Terminal(SymbolId(t3))], 2),
        );
        let ff = FirstFollowSets::compute(&g).unwrap();
        // FIRST(A) = FIRST(B) = {t3}
        prop_assert!(ff.first(a).unwrap().contains(t3 as usize));
        prop_assert!(ff.first(b).unwrap().contains(t3 as usize));
    }

    /// Arbitrary grammar: compute never panics.
    #[test]
    fn v5_edge_no_panic(g in arb_small_grammar()) {
        let _ = FirstFollowSets::compute(&g);
    }
}
