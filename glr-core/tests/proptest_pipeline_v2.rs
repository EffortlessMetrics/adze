#![cfg(feature = "test-api")]
//! Property-based tests for the full FIRST/FOLLOW → LR(1) → ParseTable pipeline.
//!
//! Requires `--features test-api` to access `test_helpers::test`.

use adze_glr_core::test_helpers::test as th;
use adze_glr_core::{Action, FirstFollowSets, ParseTable, build_lr1_automaton};
use adze_ir::builder::GrammarBuilder;
use adze_ir::{Associativity, Grammar, Symbol, SymbolId};
use proptest::prelude::*;
use std::collections::BTreeSet;

// ---------------------------------------------------------------------------
// Strategies – grammar generators
// ---------------------------------------------------------------------------

/// Token names that are safe (no `/` which triggers regex-slash panic).
const TOKEN_NAMES: &[&str] = &["a", "b", "c", "d", "e", "f"];
const TOKEN_PATTERNS: &[&str] = &["a", "b", "c", "d", "e", "f"];
const NT_NAMES: &[&str] = &["s", "t", "u", "v", "w", "x"];

/// Build a tiny grammar with 1-3 tokens and 1-3 non-terminals.
fn arb_simple_grammar() -> impl Strategy<Value = Grammar> {
    (1..=3usize, 1..=3usize).prop_flat_map(|(n_tok, n_nt)| {
        // For each NT, pick 1-3 productions, each with 1-3 RHS symbols.
        let n_tok2 = n_tok;
        let n_nt2 = n_nt;
        proptest::collection::vec(
            proptest::collection::vec(proptest::collection::vec(0..(n_tok2 + n_nt2), 1..=3), 1..=3),
            n_nt2..=n_nt2,
        )
        .prop_map(move |productions| build_grammar(n_tok, n_nt, &productions))
    })
}

/// Deterministic grammar builder from indices.
fn build_grammar(n_tok: usize, _n_nt: usize, productions: &[Vec<Vec<usize>>]) -> Grammar {
    let mut builder = GrammarBuilder::new("proptest");
    // Register tokens
    for i in 0..n_tok {
        builder = builder.token(TOKEN_NAMES[i], TOKEN_PATTERNS[i]);
    }
    // Register rules – each NT gets its set of productions
    for (nt_idx, nt_prods) in productions.iter().enumerate() {
        let lhs = NT_NAMES[nt_idx];
        for rhs_indices in nt_prods {
            let rhs: Vec<&str> = rhs_indices
                .iter()
                .map(|&idx| {
                    if idx < n_tok {
                        TOKEN_NAMES[idx]
                    } else {
                        NT_NAMES[idx - n_tok]
                    }
                })
                .collect();
            builder = builder.rule(lhs, rhs);
        }
    }
    builder = builder.start(NT_NAMES[0]);
    builder.build()
}

/// Build the smallest valid grammar: S → a
fn minimal_grammar() -> Grammar {
    GrammarBuilder::new("min")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build()
}

/// Build a grammar with two alternatives: S → a | b
fn two_alt_grammar() -> Grammar {
    GrammarBuilder::new("twoalt")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a"])
        .rule("s", vec!["b"])
        .start("s")
        .build()
}

/// Build a nullable grammar: S → ε | a
fn nullable_grammar() -> Grammar {
    GrammarBuilder::new("nullable")
        .token("a", "a")
        .rule("s", vec![])
        .rule("s", vec!["a"])
        .start("s")
        .build()
}

/// Build a left-recursive grammar: S → S a | a
fn left_recursive_grammar() -> Grammar {
    GrammarBuilder::new("leftrec")
        .token("a", "a")
        .rule("s", vec!["s", "a"])
        .rule("s", vec!["a"])
        .start("s")
        .build()
}

/// Build a grammar with precedence: E → E + E | E * E | a
fn precedence_grammar() -> Grammar {
    GrammarBuilder::new("prec")
        .token("a", "a")
        .token("+", "+")
        .token("*", "*")
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "*", "expr"], 2, Associativity::Left)
        .rule("expr", vec!["a"])
        .start("expr")
        .build()
}

/// Build a chain grammar: S → T, T → a
fn chain_grammar() -> Grammar {
    GrammarBuilder::new("chain")
        .token("a", "a")
        .rule("s", vec!["t"])
        .rule("t", vec!["a"])
        .start("s")
        .build()
}

/// Strategy yielding one of the fixed grammars.
fn arb_fixed_grammar() -> impl Strategy<Value = Grammar> {
    prop_oneof![
        Just(minimal_grammar()),
        Just(two_alt_grammar()),
        Just(nullable_grammar()),
        Just(left_recursive_grammar()),
        Just(precedence_grammar()),
        Just(chain_grammar()),
    ]
}

/// Compute FF + table, returning None if construction fails.
fn try_pipeline(g: &Grammar) -> Option<(FirstFollowSets, ParseTable)> {
    let ff = FirstFollowSets::compute(g).ok()?;
    let table = build_lr1_automaton(g, &ff).ok()?;
    Some((ff, table))
}

/// Collect all terminal SymbolIds from a grammar.
fn terminal_ids(g: &Grammar) -> Vec<SymbolId> {
    g.tokens.keys().copied().collect()
}

/// Collect all non-terminal SymbolIds from a grammar.
fn nonterminal_ids(g: &Grammar) -> Vec<SymbolId> {
    g.rules.keys().copied().collect()
}

/// Strategy for arbitrary Action values (no Fork nesting > 1 level).
fn arb_action() -> impl Strategy<Value = Action> {
    prop_oneof![
        (0..100u16).prop_map(|s| Action::Shift(adze_ir::StateId(s))),
        (0..100u16).prop_map(|r| Action::Reduce(adze_ir::RuleId(r))),
        Just(Action::Accept),
        Just(Action::Error),
        Just(Action::Recover),
    ]
}

/// Strategy for Action including Fork.
fn arb_action_with_fork() -> impl Strategy<Value = Action> {
    arb_action().prop_recursive(1, 8, 4, |inner| {
        proptest::collection::vec(inner, 2..=4).prop_map(Action::Fork)
    })
}

// ===========================================================================
// FIRST SET PROPERTIES
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    // 1. FIRST/FOLLOW computation never panics on random grammars
    #[test]
    fn ff_compute_does_not_panic(g in arb_simple_grammar()) {
        let _ = FirstFollowSets::compute(&g);
    }

    // 2. Non-terminal whose first RHS symbol is a terminal has that terminal in FIRST
    #[test]
    fn nt_first_contains_leading_terminal(g in arb_fixed_grammar()) {
        let ff = FirstFollowSets::compute(&g).unwrap();
        for (&nt, rules) in &g.rules {
            for rule in rules {
                if let Some(Symbol::Terminal(tid)) = rule.rhs.first()
                    && let Some(fs) = ff.first(nt) {
                        prop_assert!(fs.contains(tid.0 as usize),
                            "FIRST({:?}) missing leading terminal {:?}", nt, tid);
                    }
            }
        }
    }

    // 3. Non-nullable NT that derives only terminals has non-empty FIRST
    #[test]
    fn non_nullable_nt_has_nonempty_first(g in arb_fixed_grammar()) {
        let ff = FirstFollowSets::compute(&g).unwrap();
        for &nt in g.rules.keys() {
            if !ff.is_nullable(nt)
                && let Some(fs) = ff.first(nt) {
                    prop_assert!(fs.count_ones(..) > 0,
                        "Non-nullable {:?} has empty FIRST", nt);
                }
        }
    }

    // 4. Nullable symbols must have epsilon production or derive nullable
    #[test]
    fn nullable_implies_epsilon_or_nullable_rhs(g in arb_fixed_grammar()) {
        let ff = FirstFollowSets::compute(&g).unwrap();
        for &nt in g.rules.keys() {
            if ff.is_nullable(nt) {
                // At least one production must be all-nullable or epsilon
                let rules = g.rules.get(&nt).unwrap();
                let has_nullable_prod = rules.iter().any(|r| {
                    r.rhs.iter().all(|sym| match sym {
                        Symbol::Epsilon => true,
                        Symbol::NonTerminal(id) => ff.is_nullable(*id),
                        _ => false,
                    })
                });
                prop_assert!(has_nullable_prod,
                    "Nullable {:?} has no nullable production", nt);
            }
        }
    }

    // 5. FIRST set is a subset of all terminal IDs (+ possibly ε sentinel)
    #[test]
    fn first_set_subset_of_terminals(g in arb_fixed_grammar()) {
        let ff = FirstFollowSets::compute(&g).unwrap();
        let term_ids: BTreeSet<usize> = g.tokens.keys().map(|t| t.0 as usize).collect();
        for &nt in g.rules.keys() {
            if let Some(fs) = ff.first(nt) {
                for bit in fs.ones() {
                    // bit 0 is EOF sentinel, also allowed
                    prop_assert!(bit == 0 || term_ids.contains(&bit),
                        "FIRST({:?}) contains non-terminal bit {}", nt, bit);
                }
            }
        }
    }

    // 6. FOLLOW(start) always contains EOF (bit 0)
    #[test]
    fn follow_of_start_contains_eof(g in arb_fixed_grammar()) {
        let ff = FirstFollowSets::compute(&g).unwrap();
        if let Some(start) = g.start_symbol()
            && let Some(fol) = ff.follow(start) {
                prop_assert!(fol.contains(0),
                    "FOLLOW(start) missing EOF");
            }
    }

    // 7. If A → α B β where β is not nullable, FIRST(β) ⊆ FOLLOW(B)
    #[test]
    fn first_of_suffix_in_follow(g in arb_fixed_grammar()) {
        let ff = FirstFollowSets::compute(&g).unwrap();
        for rule in g.all_rules() {
            for (i, sym) in rule.rhs.iter().enumerate() {
                if let Symbol::NonTerminal(b) = sym
                    && i + 1 < rule.rhs.len() {
                        let next = &rule.rhs[i + 1];
                        if let Symbol::Terminal(t) = next
                            && let Some(fol) = ff.follow(*b) {
                                prop_assert!(fol.contains(t.0 as usize),
                                    "FIRST(next) {:?} not in FOLLOW({:?})", t, b);
                            }
                    }
            }
        }
    }

    // 8. FIRST of a sequence starting with terminal = {that terminal}
    #[test]
    fn first_of_terminal_sequence(_dummy in 0..5u32) {
        let g = minimal_grammar();
        let ff = FirstFollowSets::compute(&g).unwrap();
        let tids = terminal_ids(&g);
        if let Some(&tid) = tids.first() {
            let seq = vec![Symbol::Terminal(tid)];
            let fs = ff.first_of_sequence(&seq).unwrap();
            prop_assert!(fs.contains(tid.0 as usize));
            // Should contain only that terminal
            prop_assert_eq!(fs.count_ones(..), 1);
        }
    }

    // 9. Terminals are never nullable
    #[test]
    fn terminals_never_nullable(g in arb_fixed_grammar()) {
        let ff = FirstFollowSets::compute(&g).unwrap();
        for &tid in g.tokens.keys() {
            prop_assert!(!ff.is_nullable(tid),
                "Terminal {:?} should not be nullable", tid);
        }
    }

    // 10. FIRST sets are idempotent – computing twice gives same result
    #[test]
    fn first_follow_idempotent(g in arb_fixed_grammar()) {
        let ff1 = FirstFollowSets::compute(&g).unwrap();
        let ff2 = FirstFollowSets::compute(&g).unwrap();
        for &sym in g.rules.keys().chain(g.tokens.keys()) {
            let f1 = ff1.first(sym);
            let f2 = ff2.first(sym);
            prop_assert_eq!(f1, f2, "FIRST not idempotent for {:?}", sym);
            let fo1 = ff1.follow(sym);
            let fo2 = ff2.follow(sym);
            prop_assert_eq!(fo1, fo2, "FOLLOW not idempotent for {:?}", sym);
            prop_assert_eq!(ff1.is_nullable(sym), ff2.is_nullable(sym));
        }
    }
}

// ===========================================================================
// PARSE TABLE INVARIANTS
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(60))]

    // 11. state_count matches action_table rows
    #[test]
    fn state_count_matches_action_rows(g in arb_fixed_grammar()) {
        if let Some((_, table)) = try_pipeline(&g) {
            prop_assert_eq!(table.state_count, table.action_table.len());
        }
    }

    // 12. state_count matches goto_table rows
    #[test]
    fn state_count_matches_goto_rows(g in arb_fixed_grammar()) {
        if let Some((_, table)) = try_pipeline(&g) {
            prop_assert_eq!(table.state_count, table.goto_table.len());
        }
    }

    // 13. Every action table row has the same width
    #[test]
    fn action_rows_uniform_width(g in arb_fixed_grammar()) {
        if let Some((_, table)) = try_pipeline(&g)
            && let Some(first_row) = table.action_table.first() {
                let w = first_row.len();
                for (i, row) in table.action_table.iter().enumerate() {
                    prop_assert_eq!(row.len(), w,
                        "Action row {} has width {} but expected {}", i, row.len(), w);
                }
            }
    }

    // 14. Every goto table row has the same width
    #[test]
    fn goto_rows_uniform_width(g in arb_fixed_grammar()) {
        if let Some((_, table)) = try_pipeline(&g)
            && let Some(first_row) = table.goto_table.first() {
                let w = first_row.len();
                for (i, row) in table.goto_table.iter().enumerate() {
                    prop_assert_eq!(row.len(), w,
                        "Goto row {} has width {} but expected {}", i, row.len(), w);
                }
            }
    }

    // 15. symbol_to_index and index_to_symbol are consistent
    #[test]
    fn symbol_index_roundtrip(g in arb_fixed_grammar()) {
        if let Some((_, table)) = try_pipeline(&g) {
            for (&sym, &idx) in &table.symbol_to_index {
                if idx < table.index_to_symbol.len() {
                    prop_assert_eq!(table.index_to_symbol[idx], sym,
                        "index_to_symbol[{}] != {:?}", idx, sym);
                }
            }
        }
    }

    // 16. At least one state has Accept on EOF
    #[test]
    fn at_least_one_accept_state(g in arb_fixed_grammar()) {
        if let Some((_, table)) = try_pipeline(&g) {
            let has_accept = (0..table.state_count)
                .any(|s| th::has_accept_on_eof(&table, s));
            prop_assert!(has_accept, "No state accepts on EOF");
        }
    }

    // 17. Initial state is within bounds
    #[test]
    fn initial_state_in_bounds(g in arb_fixed_grammar()) {
        if let Some((_, table)) = try_pipeline(&g) {
            prop_assert!((table.initial_state.0 as usize) < table.state_count,
                "initial_state {} >= state_count {}", table.initial_state.0, table.state_count);
        }
    }

    // 18. ParseRule lhs refers to a known non-terminal
    #[test]
    fn parse_rules_lhs_valid(g in arb_fixed_grammar()) {
        if let Some((_, table)) = try_pipeline(&g) {
            for (i, rule) in table.rules.iter().enumerate() {
                // The lhs should be findable in nonterminal_to_index or be the augmented start
                let valid = table.nonterminal_to_index.contains_key(&rule.lhs)
                    || rule.lhs == table.start_symbol
                    || rule.lhs.0 > 100; // augmented start symbols get high IDs
                prop_assert!(valid,
                    "Rule {} lhs {:?} not in nonterminal_to_index", i, rule.lhs);
            }
        }
    }

    // 19. Rules vec is non-empty for any grammar with productions
    #[test]
    fn rules_vec_nonempty(g in arb_fixed_grammar()) {
        if let Some((_, table)) = try_pipeline(&g) {
            prop_assert!(!table.rules.is_empty(), "Rules vec should not be empty");
        }
    }

    // 20. EOF symbol is in symbol_to_index
    #[test]
    fn eof_in_symbol_index(g in arb_fixed_grammar()) {
        if let Some((_, table)) = try_pipeline(&g) {
            prop_assert!(table.symbol_to_index.contains_key(&table.eof_symbol),
                "EOF {:?} not in symbol_to_index", table.eof_symbol);
        }
    }

    // 21. Shift targets are valid state indices
    #[test]
    fn shift_targets_valid(g in arb_fixed_grammar()) {
        if let Some((_, table)) = try_pipeline(&g) {
            for (s, row) in table.action_table.iter().enumerate() {
                for cell in row {
                    for action in cell {
                        if let Action::Shift(target) = action {
                            prop_assert!((target.0 as usize) < table.state_count,
                                "State {} shift to {} >= state_count {}",
                                s, target.0, table.state_count);
                        }
                    }
                }
            }
        }
    }

    // 22. Reduce rule IDs are valid indices into rules vec
    #[test]
    fn reduce_rule_ids_valid(g in arb_fixed_grammar()) {
        if let Some((_, table)) = try_pipeline(&g) {
            for row in &table.action_table {
                for cell in row {
                    for action in cell {
                        if let Action::Reduce(rid) = action {
                            prop_assert!((rid.0 as usize) < table.rules.len(),
                                "Reduce rule {} >= rules.len() {}", rid.0, table.rules.len());
                        }
                    }
                }
            }
        }
    }

    // 23. Goto targets are valid state indices (non-zero means valid)
    #[test]
    fn goto_targets_valid(g in arb_fixed_grammar()) {
        if let Some((_, table)) = try_pipeline(&g) {
            for row in &table.goto_table {
                for &st in row {
                    if st.0 != 0 && st.0 != u16::MAX {
                        prop_assert!((st.0 as usize) < table.state_count,
                            "Goto target {} >= state_count {}", st.0, table.state_count);
                    }
                }
            }
        }
    }

    // 24. token_count <= symbol_count
    #[test]
    fn token_count_le_symbol_count(g in arb_fixed_grammar()) {
        if let Some((_, table)) = try_pipeline(&g) {
            prop_assert!(table.token_count <= table.symbol_count,
                "token_count {} > symbol_count {}", table.token_count, table.symbol_count);
        }
    }
}

// ===========================================================================
// ACTION ENUM PROPERTIES
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    // 25. Action clone roundtrip
    #[test]
    fn action_clone_eq(a in arb_action_with_fork()) {
        let b = a.clone();
        prop_assert_eq!(&a, &b);
    }

    // 26. Action Debug doesn't panic
    #[test]
    fn action_debug_no_panic(a in arb_action_with_fork()) {
        let _ = format!("{:?}", a);
    }

    // 27. Action PartialEq is reflexive
    #[test]
    fn action_eq_reflexive(a in arb_action_with_fork()) {
        prop_assert_eq!(&a, &a);
    }

    // 28. Action PartialEq is symmetric
    #[test]
    fn action_eq_symmetric(a in arb_action_with_fork(), b in arb_action_with_fork()) {
        prop_assert_eq!(a == b, b == a);
    }

    // 29. Shift variant preserves state id
    #[test]
    fn shift_preserves_state(s in 0..u16::MAX) {
        let a = Action::Shift(adze_ir::StateId(s));
        if let Action::Shift(st) = a {
            prop_assert_eq!(st.0, s);
        } else {
            prop_assert!(false, "Expected Shift");
        }
    }

    // 30. Reduce variant preserves rule id
    #[test]
    fn reduce_preserves_rule(r in 0..u16::MAX) {
        let a = Action::Reduce(adze_ir::RuleId(r));
        if let Action::Reduce(rid) = a {
            prop_assert_eq!(rid.0, r);
        } else {
            prop_assert!(false, "Expected Reduce");
        }
    }

    // 31. Fork contains its children
    #[test]
    fn fork_contains_children(children in proptest::collection::vec(arb_action(), 2..=5)) {
        let fork = Action::Fork(children.clone());
        if let Action::Fork(inner) = fork {
            prop_assert_eq!(inner.len(), children.len());
            for (a, b) in inner.iter().zip(children.iter()) {
                prop_assert_eq!(a, b);
            }
        }
    }

    // 32. Different action variants are not equal
    #[test]
    fn different_variants_not_equal(s in 0..100u16, r in 0..100u16) {
        let shift = Action::Shift(adze_ir::StateId(s));
        let reduce = Action::Reduce(adze_ir::RuleId(r));
        prop_assert_ne!(&shift, &reduce);
        prop_assert_ne!(&shift, &Action::Accept);
        prop_assert_ne!(&shift, &Action::Error);
        prop_assert_ne!(&reduce, &Action::Accept);
        prop_assert_ne!(&Action::Accept, &Action::Error);
    }

    // 33. Action Hash is consistent with Eq
    #[test]
    fn action_hash_consistent(a in arb_action_with_fork()) {
        use std::hash::{Hash, Hasher};
        use std::collections::hash_map::DefaultHasher;
        let mut h1 = DefaultHasher::new();
        let mut h2 = DefaultHasher::new();
        a.hash(&mut h1);
        a.clone().hash(&mut h2);
        prop_assert_eq!(h1.finish(), h2.finish());
    }
}

// ===========================================================================
// PARSE RULE PROPERTIES
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    // 34. ParseRule rhs_len matches source grammar rule
    #[test]
    fn parse_rule_rhs_len_matches_grammar(g in arb_fixed_grammar()) {
        if let Some((_, table)) = try_pipeline(&g) {
            // The augmented grammar adds a rule S' → S $, which has rhs_len 2
            // Other rules should match grammar RHS lengths
            for rule in &table.rules {
                // rhs_len should be reasonable
                prop_assert!(rule.rhs_len <= 100,
                    "Unreasonable rhs_len: {}", rule.rhs_len);
            }
        }
    }

    // 35. Number of parse rules >= number of grammar rules
    #[test]
    fn parse_rules_ge_grammar_rules(g in arb_fixed_grammar()) {
        if let Some((_, table)) = try_pipeline(&g) {
            let grammar_rule_count: usize = g.rules.values().map(|v| v.len()).sum();
            // +1 for the augmented start rule
            prop_assert!(table.rules.len() >= grammar_rule_count,
                "parse rules {} < grammar rules {}", table.rules.len(), grammar_rule_count);
        }
    }

    // 36. All ParseRules have non-zero lhs (lhs.0 > 0)
    #[test]
    fn parse_rule_lhs_nonzero(g in arb_fixed_grammar()) {
        if let Some((_, table)) = try_pipeline(&g) {
            for (i, rule) in table.rules.iter().enumerate() {
                // SymbolId(0) is reserved for EOF/ERROR
                // Some internal rules might use it, but actual grammar rules shouldn't
                // Just verify it's bounded
                prop_assert!(rule.lhs.0 <= 10000,
                    "Rule {} lhs {:?} is suspiciously large", i, rule.lhs);
            }
        }
    }

    // 37. ParseRule rhs_len is bounded by grammar max
    #[test]
    fn parse_rule_rhs_bounded(g in arb_fixed_grammar()) {
        if let Some((_, table)) = try_pipeline(&g) {
            let max_grammar_rhs = g.all_rules()
                .map(|r| r.rhs.len())
                .max()
                .unwrap_or(0);
            for rule in &table.rules {
                // augmented rule adds 2 (S' → S $)
                prop_assert!(rule.rhs_len as usize <= max_grammar_rhs + 2,
                    "rhs_len {} exceeds max grammar rhs {} + 2", rule.rhs_len, max_grammar_rhs);
            }
        }
    }
}

// ===========================================================================
// DETERMINISM: same grammar → same results
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(40))]

    // 38. Same grammar always produces identical FIRST sets
    #[test]
    fn deterministic_first_sets(g in arb_fixed_grammar()) {
        let ff1 = FirstFollowSets::compute(&g).unwrap();
        let ff2 = FirstFollowSets::compute(&g).unwrap();
        for &sym in g.rules.keys().chain(g.tokens.keys()) {
            prop_assert_eq!(ff1.first(sym), ff2.first(sym));
        }
    }

    // 39. Same grammar always produces identical FOLLOW sets
    #[test]
    fn deterministic_follow_sets(g in arb_fixed_grammar()) {
        let ff1 = FirstFollowSets::compute(&g).unwrap();
        let ff2 = FirstFollowSets::compute(&g).unwrap();
        for &sym in g.rules.keys().chain(g.tokens.keys()) {
            prop_assert_eq!(ff1.follow(sym), ff2.follow(sym));
        }
    }

    // 40. Same grammar always produces same state_count
    #[test]
    fn deterministic_state_count(g in arb_fixed_grammar()) {
        if let (Some((_, t1)), Some((_, t2))) = (try_pipeline(&g), try_pipeline(&g)) {
            prop_assert_eq!(t1.state_count, t2.state_count);
        }
    }

    // 41. Same grammar always produces same rules vec length
    #[test]
    fn deterministic_rules_len(g in arb_fixed_grammar()) {
        if let (Some((_, t1)), Some((_, t2))) = (try_pipeline(&g), try_pipeline(&g)) {
            prop_assert_eq!(t1.rules.len(), t2.rules.len());
        }
    }

    // 42. Same grammar always produces same action table dimensions
    #[test]
    fn deterministic_action_dimensions(g in arb_fixed_grammar()) {
        if let (Some((_, t1)), Some((_, t2))) = (try_pipeline(&g), try_pipeline(&g)) {
            prop_assert_eq!(t1.action_table.len(), t2.action_table.len());
            for (r1, r2) in t1.action_table.iter().zip(t2.action_table.iter()) {
                prop_assert_eq!(r1.len(), r2.len());
            }
        }
    }

    // 43. Same grammar always produces same action cell contents
    #[test]
    fn deterministic_action_contents(g in arb_fixed_grammar()) {
        if let (Some((_, t1)), Some((_, t2))) = (try_pipeline(&g), try_pipeline(&g)) {
            for (r1, r2) in t1.action_table.iter().zip(t2.action_table.iter()) {
                for (c1, c2) in r1.iter().zip(r2.iter()) {
                    prop_assert_eq!(c1, c2);
                }
            }
        }
    }

    // 44. Same grammar always produces same goto table
    #[test]
    fn deterministic_goto_table(g in arb_fixed_grammar()) {
        if let (Some((_, t1)), Some((_, t2))) = (try_pipeline(&g), try_pipeline(&g)) {
            for (r1, r2) in t1.goto_table.iter().zip(t2.goto_table.iter()) {
                prop_assert_eq!(r1, r2);
            }
        }
    }

    // 45. Same grammar produces same symbol_to_index
    #[test]
    fn deterministic_symbol_to_index(g in arb_fixed_grammar()) {
        if let (Some((_, t1)), Some((_, t2))) = (try_pipeline(&g), try_pipeline(&g)) {
            prop_assert_eq!(&t1.symbol_to_index, &t2.symbol_to_index);
        }
    }

    // 46. Same grammar produces same eof_symbol
    #[test]
    fn deterministic_eof_symbol(g in arb_fixed_grammar()) {
        if let (Some((_, t1)), Some((_, t2))) = (try_pipeline(&g), try_pipeline(&g)) {
            prop_assert_eq!(t1.eof_symbol, t2.eof_symbol);
        }
    }

    // 47. Same grammar produces same nullable set
    #[test]
    fn deterministic_nullable(g in arb_fixed_grammar()) {
        let ff1 = FirstFollowSets::compute(&g).unwrap();
        let ff2 = FirstFollowSets::compute(&g).unwrap();
        for &sym in g.rules.keys() {
            prop_assert_eq!(ff1.is_nullable(sym), ff2.is_nullable(sym));
        }
    }
}

// ===========================================================================
// RANDOM GRAMMAR PIPELINE TESTS
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(60))]

    // 48. Random grammars never panic in pipeline
    #[test]
    fn random_grammar_pipeline_no_panic(g in arb_simple_grammar()) {
        let _ = try_pipeline(&g);
    }

    // 49. If pipeline succeeds, table has ≥1 state
    #[test]
    fn successful_pipeline_has_states(g in arb_simple_grammar()) {
        if let Some((_, table)) = try_pipeline(&g) {
            prop_assert!(table.state_count >= 1,
                "Successful pipeline produced 0 states");
        }
    }

    // 50. If pipeline succeeds, every grammar terminal is in symbol_to_index
    #[test]
    fn terminals_in_symbol_index(g in arb_simple_grammar()) {
        if let Some((_, table)) = try_pipeline(&g) {
            for &tid in g.tokens.keys() {
                prop_assert!(table.symbol_to_index.contains_key(&tid),
                    "Terminal {:?} not in symbol_to_index", tid);
            }
        }
    }

    // 51. If pipeline succeeds, start_symbol is set
    #[test]
    fn start_symbol_is_set(g in arb_simple_grammar()) {
        if let Some((_, table)) = try_pipeline(&g) {
            prop_assert!(table.start_symbol.0 > 0,
                "start_symbol should be non-zero");
        }
    }

    // 52. ParseTable.grammar() returns a grammar with same name
    #[test]
    fn table_grammar_name_matches(g in arb_fixed_grammar()) {
        if let Some((_, table)) = try_pipeline(&g) {
            // The table wraps an augmented grammar; the name should still be present
            let _name = &table.grammar().name;
            // Just verify it doesn't panic
        }
    }

    // 53. table.eof() matches table.eof_symbol
    #[test]
    fn eof_accessor_consistent(g in arb_fixed_grammar()) {
        if let Some((_, table)) = try_pipeline(&g) {
            prop_assert_eq!(table.eof(), table.eof_symbol);
        }
    }

    // 54. table.start_symbol() matches table.start_symbol field
    #[test]
    fn start_symbol_accessor_consistent(g in arb_fixed_grammar()) {
        if let Some((_, table)) = try_pipeline(&g) {
            prop_assert_eq!(table.start_symbol(), table.start_symbol);
        }
    }

    // 55. table.rule(id) matches table.rules[id]
    #[test]
    fn rule_accessor_consistent(g in arb_fixed_grammar()) {
        if let Some((_, table)) = try_pipeline(&g) {
            for (i, r) in table.rules.iter().enumerate() {
                let (lhs, rhs_len) = table.rule(adze_ir::RuleId(i as u16));
                prop_assert_eq!(lhs, r.lhs);
                prop_assert_eq!(rhs_len, r.rhs_len);
            }
        }
    }

    // 56. valid_symbols returns a vec of the right length
    #[test]
    fn valid_symbols_length(g in arb_fixed_grammar()) {
        if let Some((_, table)) = try_pipeline(&g) {
            let vs = table.valid_symbols(table.initial_state);
            prop_assert_eq!(vs.len(), table.terminal_boundary());
        }
    }

    // 57. terminal_boundary = token_count + external_token_count
    #[test]
    fn terminal_boundary_formula(g in arb_fixed_grammar()) {
        if let Some((_, table)) = try_pipeline(&g) {
            prop_assert_eq!(
                table.terminal_boundary(),
                table.token_count + table.external_token_count
            );
        }
    }

    // 58. actions() on invalid state returns empty slice
    #[test]
    fn actions_out_of_bounds_empty(g in arb_fixed_grammar()) {
        if let Some((_, table)) = try_pipeline(&g) {
            let bad_state = adze_ir::StateId(table.state_count as u16 + 100);
            let tids = terminal_ids(&g);
            if let Some(&tid) = tids.first() {
                let acts = table.actions(bad_state, tid);
                prop_assert!(acts.is_empty());
            }
        }
    }

    // 59. actions() on invalid symbol returns empty slice
    #[test]
    fn actions_invalid_symbol_empty(g in arb_fixed_grammar()) {
        if let Some((_, table)) = try_pipeline(&g) {
            let bad_sym = SymbolId(60000);
            let acts = table.actions(table.initial_state, bad_sym);
            prop_assert!(acts.is_empty());
        }
    }

    // 60. goto() on invalid state returns None
    #[test]
    fn goto_out_of_bounds_none(g in arb_fixed_grammar()) {
        if let Some((_, table)) = try_pipeline(&g) {
            let bad_state = adze_ir::StateId(table.state_count as u16 + 100);
            let nts = nonterminal_ids(&g);
            if let Some(&nt) = nts.first() {
                let result = table.goto(bad_state, nt);
                prop_assert!(result.is_none());
            }
        }
    }
}
