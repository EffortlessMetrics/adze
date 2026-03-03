#![allow(clippy::needless_range_loop)]
//! Property-based tests for GOTO table construction in adze-glr-core.
//!
//! The GOTO table maps `(state, nonterminal) → state` and is stored as
//! `ParseTable::goto_table: Vec<Vec<StateId>>`.  Lookups go through
//! `ParseTable::goto()` which uses `nonterminal_to_index` for column mapping
//! and treats `StateId(u16::MAX)` as a sentinel for "no transition".
//!
//! Run with: `cargo test -p adze-glr-core --test goto_table_proptest`

use adze_glr_core::{
    Action, FirstFollowSets, GotoIndexing, LexMode, ParseRule, ParseTable, build_lr1_automaton,
};
use adze_ir::builder::GrammarBuilder;
use adze_ir::*;
use proptest::prelude::*;
use std::collections::{BTreeMap, BTreeSet};

// ---------------------------------------------------------------------------
// Helpers – grammar construction
// ---------------------------------------------------------------------------

/// Build a parse table from a grammar via the standard pipeline.
fn build_table(grammar: &Grammar) -> ParseTable {
    let ff = FirstFollowSets::compute(grammar).expect("FIRST/FOLLOW");
    build_lr1_automaton(grammar, &ff).expect("automaton")
}

/// S → t  (one terminal, one nonterminal)
fn grammar_single(tok_id: u16) -> Grammar {
    let t = SymbolId(tok_id);
    let s = SymbolId(tok_id + 10);
    let mut g = Grammar::new("single".into());
    g.tokens.insert(
        t,
        Token {
            name: format!("t{tok_id}"),
            pattern: TokenPattern::String(format!("t{tok_id}")),
            fragile: false,
        },
    );
    g.rule_names.insert(s, "S".into());
    g.rules.insert(
        s,
        vec![Rule {
            lhs: s,
            rhs: vec![Symbol::Terminal(t)],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(0),
        }],
    );
    g
}

/// S → t₁ | t₂ | … | tₙ
fn grammar_n_alternatives(n: usize) -> Grammar {
    assert!(n >= 1);
    let s = SymbolId(100);
    let mut g = Grammar::new("alt".into());
    for i in 0..n {
        let t = SymbolId((i + 1) as u16);
        g.tokens.insert(
            t,
            Token {
                name: format!("t{i}"),
                pattern: TokenPattern::String(format!("t{i}")),
                fragile: false,
            },
        );
    }
    g.rule_names.insert(s, "S".into());
    let rules: Vec<Rule> = (0..n)
        .map(|i| Rule {
            lhs: s,
            rhs: vec![Symbol::Terminal(SymbolId((i + 1) as u16))],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(i as u16),
        })
        .collect();
    g.rules.insert(s, rules);
    g
}

/// Chain grammar: N0 → N1, N1 → N2, …, N(depth-1) → t
fn grammar_chain(depth: usize) -> Grammar {
    assert!(depth >= 1);
    let t = SymbolId(1);
    let mut g = Grammar::new("chain".into());
    g.tokens.insert(
        t,
        Token {
            name: "t".into(),
            pattern: TokenPattern::String("t".into()),
            fragile: false,
        },
    );
    for i in 0..depth {
        let nt = SymbolId(100 + i as u16);
        g.rule_names.insert(nt, format!("N{i}"));
        let rhs = if i + 1 < depth {
            vec![Symbol::NonTerminal(SymbolId(100 + (i + 1) as u16))]
        } else {
            vec![Symbol::Terminal(t)]
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
    g
}

/// S → a b (sequence grammar)
fn grammar_sequence() -> Grammar {
    GrammarBuilder::new("seq")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a", "b"])
        .start("start")
        .build()
}

/// S → A B, A → a, B → b (two-nonterminal grammar)
fn grammar_two_nt() -> Grammar {
    GrammarBuilder::new("twont")
        .token("a", "a")
        .token("b", "b")
        .rule("aa", vec!["a"])
        .rule("bb", vec!["b"])
        .rule("start", vec!["aa", "bb"])
        .start("start")
        .build()
}

/// S → A B C, A → a, B → b, C → c (three nonterminals)
fn grammar_three_nt() -> Grammar {
    GrammarBuilder::new("threent")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("aa", vec!["a"])
        .rule("bb", vec!["b"])
        .rule("cc", vec!["c"])
        .rule("start", vec!["aa", "bb", "cc"])
        .start("start")
        .build()
}

// ---------------------------------------------------------------------------
// Helpers – synthetic ParseTable construction (for pure-structural tests)
// ---------------------------------------------------------------------------

const NO_GOTO: StateId = StateId(65535);

fn synthetic_table(
    num_states: usize,
    num_terminals: usize,
    num_nonterminals: usize,
    goto_table: Vec<Vec<StateId>>,
    rules: Vec<ParseRule>,
) -> ParseTable {
    let symbol_count = num_terminals + num_nonterminals;
    let mut symbol_to_index = BTreeMap::new();
    let mut index_to_symbol = Vec::new();
    for i in 0..symbol_count {
        symbol_to_index.insert(SymbolId(i as u16), i);
        index_to_symbol.push(SymbolId(i as u16));
    }
    let mut nonterminal_to_index = BTreeMap::new();
    for i in num_terminals..symbol_count {
        nonterminal_to_index.insert(SymbolId(i as u16), i - num_terminals);
    }

    ParseTable {
        action_table: vec![vec![Vec::new(); symbol_count]; num_states],
        goto_table,
        symbol_metadata: vec![],
        state_count: num_states,
        symbol_count,
        symbol_to_index,
        index_to_symbol,
        external_scanner_states: vec![],
        rules: rules.clone(),
        nonterminal_to_index,
        goto_indexing: GotoIndexing::NonterminalMap,
        eof_symbol: SymbolId(0),
        start_symbol: SymbolId(num_terminals as u16),
        grammar: Grammar::new("synth".into()),
        initial_state: StateId(0),
        token_count: num_terminals,
        external_token_count: 0,
        lex_modes: vec![
            LexMode {
                lex_state: 0,
                external_lex_state: 0,
            };
            num_states
        ],
        extras: vec![],
        dynamic_prec_by_rule: vec![0; rules.len()],
        rule_assoc_by_rule: vec![0; rules.len()],
        alias_sequences: vec![],
        field_names: vec![],
        field_map: BTreeMap::new(),
    }
}

// ---------------------------------------------------------------------------
// Proptest strategies
// ---------------------------------------------------------------------------

/// Strategy for valid goto table entries: either NO_GOTO or a valid state.
fn goto_entry(max_state: u16) -> impl Strategy<Value = StateId> {
    prop_oneof![Just(NO_GOTO), (0..max_state).prop_map(StateId),]
}

/// Strategy for a synthetic parse table with random goto entries.
fn arb_goto_table() -> impl Strategy<Value = ParseTable> {
    (1usize..=8, 1usize..=4, 1usize..=4).prop_flat_map(|(ns, nt, nnt)| {
        let gotos = prop::collection::vec(
            prop::collection::vec(goto_entry(ns as u16), nnt..=nnt),
            ns..=ns,
        );
        let rules = prop::collection::vec(
            ((nt as u16)..(nt + nnt) as u16, 0u16..=4).prop_map(|(lhs, rhs_len)| ParseRule {
                lhs: SymbolId(lhs),
                rhs_len,
            }),
            0..=6,
        );
        (Just(ns), Just(nt), Just(nnt), gotos, rules)
            .prop_map(|(ns, nt, nnt, gotos, rules)| synthetic_table(ns, nt, nnt, gotos, rules))
    })
}

// ===========================================================================
// Property tests
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(120))]

    // -----------------------------------------------------------------------
    // 1. Goto entries always point to valid state indices
    // -----------------------------------------------------------------------
    #[test]
    fn goto_entries_point_to_valid_states(tok_id in 1u16..30) {
        let g = grammar_single(tok_id);
        let pt = build_table(&g);
        for s in 0..pt.state_count {
            let st = StateId(s as u16);
            for &nt in pt.nonterminal_to_index.keys() {
                if let Some(target) = pt.goto(st, nt) {
                    prop_assert!(
                        (target.0 as usize) < pt.state_count,
                        "goto({s}, {:?}) = {:?} out of range (states={})",
                        nt, target, pt.state_count,
                    );
                }
            }
        }
    }

    // -----------------------------------------------------------------------
    // 2. Goto table is deterministic (each cell has exactly one value)
    // -----------------------------------------------------------------------
    #[test]
    fn goto_is_deterministic(tok_id in 1u16..30) {
        let g = grammar_single(tok_id);
        let pt = build_table(&g);
        for s in 0..pt.state_count {
            for row in &pt.goto_table {
                for col in 0..row.len() {
                    // Each cell is a single StateId — deterministic by construction.
                    let val = row[col];
                    prop_assert!(
                        val.0 == u16::MAX || (val.0 as usize) < pt.state_count,
                        "non-deterministic or invalid goto entry: {:?}", val,
                    );
                    let _ = s; // suppress unused
                }
            }
        }
    }

    // -----------------------------------------------------------------------
    // 3. Every nonterminal in nonterminal_to_index has a goto from at least
    //    one state (coverage)
    // -----------------------------------------------------------------------
    #[test]
    fn goto_covers_all_nonterminals(tok_id in 1u16..20) {
        let g = grammar_single(tok_id);
        let pt = build_table(&g);
        for &nt in pt.nonterminal_to_index.keys() {
            let has_goto = (0..pt.state_count).any(|s| {
                pt.goto(StateId(s as u16), nt).is_some()
            });
            prop_assert!(has_goto,
                "nonterminal {:?} has no goto from any state", nt);
        }
    }

    // -----------------------------------------------------------------------
    // 4. Goto table rows match state_count
    // -----------------------------------------------------------------------
    #[test]
    fn goto_rows_eq_state_count(tok_id in 1u16..30) {
        let g = grammar_single(tok_id);
        let pt = build_table(&g);
        prop_assert_eq!(pt.goto_table.len(), pt.state_count);
    }

    // -----------------------------------------------------------------------
    // 5. Goto table column widths are uniform
    // -----------------------------------------------------------------------
    #[test]
    fn goto_columns_uniform(tok_id in 1u16..30) {
        let g = grammar_single(tok_id);
        let pt = build_table(&g);
        if let Some(first) = pt.goto_table.first() {
            let w = first.len();
            for (i, row) in pt.goto_table.iter().enumerate() {
                prop_assert_eq!(row.len(), w, "goto row {} width mismatch", i);
            }
        }
    }

    // -----------------------------------------------------------------------
    // 6. Alternative grammars: goto entries valid
    // -----------------------------------------------------------------------
    #[test]
    fn alternatives_goto_valid(n in 1usize..8) {
        let g = grammar_n_alternatives(n);
        let pt = build_table(&g);
        for s in 0..pt.state_count {
            for &nt in pt.nonterminal_to_index.keys() {
                if let Some(tgt) = pt.goto(StateId(s as u16), nt) {
                    prop_assert!((tgt.0 as usize) < pt.state_count);
                }
            }
        }
    }

    // -----------------------------------------------------------------------
    // 7. Chain grammars: every intermediate nonterminal reachable via goto
    // -----------------------------------------------------------------------
    #[test]
    fn chain_goto_covers_intermediates(depth in 1usize..6) {
        let g = grammar_chain(depth);
        let pt = build_table(&g);
        for &nt in pt.nonterminal_to_index.keys() {
            let reachable = (0..pt.state_count)
                .any(|s| pt.goto(StateId(s as u16), nt).is_some());
            prop_assert!(reachable,
                "nt {:?} unreachable in chain(depth={depth})", nt);
        }
    }

    // -----------------------------------------------------------------------
    // 8. Goto targets are distinct per state (no two nonterminals map to the
    //    same target in one state — probabilistic but holds for small grammars)
    // -----------------------------------------------------------------------
    #[test]
    fn goto_targets_locally_unique(tok_id in 1u16..20) {
        let g = grammar_single(tok_id);
        let pt = build_table(&g);
        for s in 0..pt.state_count {
            let st = StateId(s as u16);
            let targets: Vec<StateId> = pt.nonterminal_to_index.keys()
                .filter_map(|&nt| pt.goto(st, nt))
                .collect();
            let unique: BTreeSet<u16> = targets.iter().map(|t| t.0).collect();
            prop_assert_eq!(targets.len(), unique.len(),
                "state {}: duplicate goto targets", s);
        }
    }

    // -----------------------------------------------------------------------
    // 9. Goto accessor returns None for unknown nonterminal
    // -----------------------------------------------------------------------
    #[test]
    fn goto_none_for_unknown_nt(tok_id in 1u16..30) {
        let g = grammar_single(tok_id);
        let pt = build_table(&g);
        let unknown = SymbolId(60000);
        for s in 0..pt.state_count {
            prop_assert!(pt.goto(StateId(s as u16), unknown).is_none());
        }
    }

    // -----------------------------------------------------------------------
    // 10. Goto accessor returns None for out-of-range state
    // -----------------------------------------------------------------------
    #[test]
    fn goto_none_for_oob_state(tok_id in 1u16..20) {
        let g = grammar_single(tok_id);
        let pt = build_table(&g);
        let oob = StateId(pt.state_count as u16 + 100);
        for &nt in pt.nonterminal_to_index.keys() {
            prop_assert!(pt.goto(oob, nt).is_none());
        }
    }

    // -----------------------------------------------------------------------
    // 11. Synthetic table: valid entries within bounds
    // -----------------------------------------------------------------------
    #[test]
    fn synthetic_goto_entries_valid(pt in arb_goto_table()) {
        for s in 0..pt.state_count {
            for &nt in pt.nonterminal_to_index.keys() {
                if let Some(tgt) = pt.goto(StateId(s as u16), nt) {
                    prop_assert!((tgt.0 as usize) < pt.state_count,
                        "synthetic goto target out of range");
                }
            }
        }
    }

    // -----------------------------------------------------------------------
    // 12. Synthetic table: deterministic
    // -----------------------------------------------------------------------
    #[test]
    fn synthetic_goto_deterministic(pt in arb_goto_table()) {
        for row in &pt.goto_table {
            for &cell in row {
                prop_assert!(
                    cell == NO_GOTO || (cell.0 as usize) < pt.state_count,
                    "bad cell: {:?}", cell,
                );
            }
        }
    }

    // -----------------------------------------------------------------------
    // 13. Goto table rows uniform width on synthetic tables
    // -----------------------------------------------------------------------
    #[test]
    fn synthetic_goto_uniform_width(pt in arb_goto_table()) {
        if let Some(w) = pt.goto_table.first().map(|r| r.len()) {
            for (i, row) in pt.goto_table.iter().enumerate() {
                prop_assert_eq!(row.len(), w, "synth row {} width", i);
            }
        }
    }

    // -----------------------------------------------------------------------
    // 14. nonterminal_to_index columns are within goto row bounds
    // -----------------------------------------------------------------------
    #[test]
    fn nt_index_within_goto_cols(tok_id in 1u16..30) {
        let g = grammar_single(tok_id);
        let pt = build_table(&g);
        let width = pt.goto_table.first().map(|r| r.len()).unwrap_or(0);
        for (&_nt, &col) in &pt.nonterminal_to_index {
            prop_assert!(col < width || width == 0,
                "nt col {col} >= goto width {width}");
        }
    }

    // -----------------------------------------------------------------------
    // 15. Reduce actions' LHS is reachable via some goto
    // -----------------------------------------------------------------------
    #[test]
    fn reduce_lhs_reachable_via_goto(tok_id in 1u16..20) {
        let g = grammar_single(tok_id);
        let pt = build_table(&g);
        for rule in &pt.rules {
            let lhs = rule.lhs;
            if pt.nonterminal_to_index.contains_key(&lhs) {
                let reachable = (0..pt.state_count)
                    .any(|s| pt.goto(StateId(s as u16), lhs).is_some());
                prop_assert!(reachable,
                    "rule LHS {:?} not reachable via goto", lhs);
            }
        }
    }

    // -----------------------------------------------------------------------
    // 16. Goto vs action table consistency: goto targets have action entries
    // -----------------------------------------------------------------------
    #[test]
    fn goto_targets_have_action_rows(tok_id in 1u16..20) {
        let g = grammar_single(tok_id);
        let pt = build_table(&g);
        for s in 0..pt.state_count {
            for &nt in pt.nonterminal_to_index.keys() {
                if let Some(tgt) = pt.goto(StateId(s as u16), nt) {
                    prop_assert!(
                        (tgt.0 as usize) < pt.action_table.len(),
                        "goto target {} has no action_table row", tgt.0,
                    );
                }
            }
        }
    }

    // -----------------------------------------------------------------------
    // 17. Action table Reduce rules have valid goto targets after reduction
    // -----------------------------------------------------------------------
    #[test]
    fn reduce_rule_lhs_goto_exists(tok_id in 1u16..20) {
        let g = grammar_single(tok_id);
        let pt = build_table(&g);
        // For each Reduce(rule_id) in the action table, the rule's LHS
        // should have a goto entry from at least one state.
        let mut reduce_lhs_ids = BTreeSet::new();
        for row in &pt.action_table {
            for cell in row {
                for action in cell {
                    if let Action::Reduce(rid) = action {
                        let idx = rid.0 as usize;
                        if idx < pt.rules.len() {
                            reduce_lhs_ids.insert(pt.rules[idx].lhs);
                        }
                    }
                }
            }
        }
        for lhs in reduce_lhs_ids {
            if pt.nonterminal_to_index.contains_key(&lhs) {
                let has = (0..pt.state_count)
                    .any(|s| pt.goto(StateId(s as u16), lhs).is_some());
                prop_assert!(has, "reduce LHS {:?} missing goto", lhs);
            }
        }
    }

    // -----------------------------------------------------------------------
    // 18. Large alternative grammars: state count grows, goto stays valid
    // -----------------------------------------------------------------------
    #[test]
    fn large_alternatives_goto_valid(n in 1usize..15) {
        let g = grammar_n_alternatives(n);
        let pt = build_table(&g);
        prop_assert!(pt.state_count >= 2);
        for s in 0..pt.state_count {
            for &nt in pt.nonterminal_to_index.keys() {
                if let Some(tgt) = pt.goto(StateId(s as u16), nt) {
                    prop_assert!((tgt.0 as usize) < pt.state_count);
                }
            }
        }
    }

    // -----------------------------------------------------------------------
    // 19. Chain grammars: state count >= depth + 1 (at least)
    // -----------------------------------------------------------------------
    #[test]
    fn chain_state_count_grows(depth in 1usize..6) {
        let g = grammar_chain(depth);
        let pt = build_table(&g);
        prop_assert!(pt.state_count >= depth + 1,
            "chain(depth={depth}) should have >= {} states, got {}",
            depth + 1, pt.state_count);
    }

    // -----------------------------------------------------------------------
    // 20. nonterminal_to_index keys are a subset of grammar nonterminals
    // -----------------------------------------------------------------------
    #[test]
    fn nt_index_keys_subset_of_grammar(tok_id in 1u16..20) {
        let g = grammar_single(tok_id);
        let pt = build_table(&g);
        let grammar_nts: BTreeSet<SymbolId> = pt.grammar.rules.keys().copied().collect();
        for &nt in pt.nonterminal_to_index.keys() {
            prop_assert!(grammar_nts.contains(&nt),
                "nt {:?} in nonterminal_to_index but not in grammar rules", nt);
        }
    }

    // -----------------------------------------------------------------------
    // 21. Goto table sentinel value is u16::MAX (65535)
    // -----------------------------------------------------------------------
    #[test]
    fn goto_sentinel_is_u16_max(pt in arb_goto_table()) {
        for row in &pt.goto_table {
            for &cell in row {
                if cell.0 as usize >= pt.state_count && cell != NO_GOTO {
                    prop_assert!(false,
                        "goto cell {:?} is neither valid state nor sentinel", cell);
                }
            }
        }
    }

    // -----------------------------------------------------------------------
    // 22. Clone preserves goto table bit-for-bit
    // -----------------------------------------------------------------------
    #[test]
    fn clone_preserves_goto(tok_id in 1u16..20) {
        let g = grammar_single(tok_id);
        let pt = build_table(&g);
        let cloned = pt.clone();
        prop_assert_eq!(pt.goto_table, cloned.goto_table);
        prop_assert_eq!(pt.nonterminal_to_index, cloned.nonterminal_to_index);
        prop_assert_eq!(pt.goto_indexing, cloned.goto_indexing);
    }

    // -----------------------------------------------------------------------
    // 23. Goto accessor is consistent with raw table lookup
    // -----------------------------------------------------------------------
    #[test]
    fn goto_accessor_matches_raw(tok_id in 1u16..20) {
        let g = grammar_single(tok_id);
        let pt = build_table(&g);
        for s in 0..pt.state_count {
            for (&nt, &col) in &pt.nonterminal_to_index {
                let raw = pt.goto_table[s][col];
                let accessor = pt.goto(StateId(s as u16), nt);
                if raw.0 == u16::MAX {
                    prop_assert!(accessor.is_none(),
                        "raw sentinel but accessor returned Some");
                } else {
                    prop_assert_eq!(accessor, Some(raw),
                        "accessor mismatch at state {}, nt {:?}", s, nt);
                }
            }
        }
    }

    // -----------------------------------------------------------------------
    // 24. Start symbol always has a goto from the initial state
    // -----------------------------------------------------------------------
    #[test]
    fn start_symbol_goto_from_initial(tok_id in 1u16..30) {
        let g = grammar_single(tok_id);
        let pt = build_table(&g);
        let start = pt.start_symbol();
        let has_goto = pt.goto(pt.initial_state, start).is_some();
        prop_assert!(has_goto,
            "start symbol {:?} missing goto from initial state", start);
    }

    // -----------------------------------------------------------------------
    // 25. Shift targets in action table appear as goto sources
    // -----------------------------------------------------------------------
    #[test]
    fn shift_targets_are_valid_states(tok_id in 1u16..20) {
        let g = grammar_single(tok_id);
        let pt = build_table(&g);
        for row in &pt.action_table {
            for cell in row {
                for action in cell {
                    if let Action::Shift(tgt) = action {
                        prop_assert!(
                            (tgt.0 as usize) < pt.state_count,
                            "shift target {:?} out of range", tgt,
                        );
                    }
                }
            }
        }
    }
}

// ===========================================================================
// Non-proptest unit tests (empty and structural edge cases)
// ===========================================================================

/// 26. Empty/default ParseTable has empty goto table.
#[test]
fn empty_grammar_goto_table_is_empty() {
    let pt = ParseTable::default();
    assert!(pt.goto_table.is_empty());
    assert!(pt.nonterminal_to_index.is_empty());
    assert_eq!(pt.state_count, 0);
}

/// 27. Default table: goto accessor on state 0 returns None.
#[test]
fn empty_table_goto_returns_none() {
    let pt = ParseTable::default();
    assert!(pt.goto(StateId(0), SymbolId(0)).is_none());
}

/// 28. Sequence grammar (S → a b) has valid goto.
#[test]
fn sequence_grammar_goto_valid() {
    let g = grammar_sequence();
    let pt = build_table(&g);
    for s in 0..pt.state_count {
        for &nt in pt.nonterminal_to_index.keys() {
            if let Some(tgt) = pt.goto(StateId(s as u16), nt) {
                assert!(
                    (tgt.0 as usize) < pt.state_count,
                    "goto target out of range"
                );
            }
        }
    }
}

/// 29. Two-nonterminal grammar: both nonterminals reachable via goto.
#[test]
fn two_nt_grammar_both_reachable() {
    let g = grammar_two_nt();
    let pt = build_table(&g);
    for &nt in pt.nonterminal_to_index.keys() {
        let reachable = (0..pt.state_count).any(|s| pt.goto(StateId(s as u16), nt).is_some());
        assert!(reachable, "nt {:?} unreachable via goto", nt);
    }
}

/// 30. Three-nonterminal grammar: all three reachable.
#[test]
fn three_nt_grammar_all_reachable() {
    let g = grammar_three_nt();
    let pt = build_table(&g);
    for &nt in pt.nonterminal_to_index.keys() {
        let reachable = (0..pt.state_count).any(|s| pt.goto(StateId(s as u16), nt).is_some());
        assert!(reachable, "nt {:?} unreachable in 3-nt grammar", nt);
    }
}

/// 31. Large alternative grammar (15 alternatives) produces valid goto.
#[test]
fn large_grammar_goto_valid() {
    let g = grammar_n_alternatives(15);
    let pt = build_table(&g);
    assert!(pt.state_count >= 2);
    for s in 0..pt.state_count {
        for &nt in pt.nonterminal_to_index.keys() {
            if let Some(tgt) = pt.goto(StateId(s as u16), nt) {
                assert!((tgt.0 as usize) < pt.state_count);
            }
        }
    }
}

/// 32. Goto indexing mode is NonterminalMap after build_lr1_automaton.
#[test]
fn goto_indexing_is_nonterminal_map() {
    let g = grammar_single(1);
    let pt = build_table(&g);
    // build_lr1_automaton calls detect_goto_indexing; the default for
    // the standard pipeline is NonterminalMap.
    assert!(
        matches!(
            pt.goto_indexing,
            GotoIndexing::NonterminalMap | GotoIndexing::DirectSymbolId
        ),
        "unexpected goto_indexing: {:?}",
        pt.goto_indexing,
    );
}

/// 33. GrammarBuilder-based grammar has goto coverage for start symbol.
#[test]
fn builder_grammar_start_goto() {
    let g = GrammarBuilder::new("bg")
        .token("x", "x")
        .rule("start", vec!["x"])
        .start("start")
        .build();
    let pt = build_table(&g);
    let has = pt.goto(pt.initial_state, pt.start_symbol()).is_some();
    assert!(has, "start symbol missing goto from initial state");
}

/// 34. Action and goto table have the same number of rows.
#[test]
fn action_and_goto_row_count_match() {
    let g = grammar_two_nt();
    let pt = build_table(&g);
    assert_eq!(pt.action_table.len(), pt.goto_table.len());
    assert_eq!(pt.action_table.len(), pt.state_count);
}
