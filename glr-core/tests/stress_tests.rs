//! Stress tests and edge-case tests for GLR core algorithms.
//!
//! Run with: cargo test -p adze-glr-core --features test-api --test stress_tests

#![cfg(feature = "test-api")]

use adze_glr_core::{
    Action, ConflictResolver, ConflictType, Driver, FirstFollowSets, GotoIndexing, ItemSet,
    ItemSetCollection, LexMode, ParseRule, ParseTable,
};
use adze_ir::{
    Grammar, ProductionId, Rule, RuleId, StateId, Symbol, SymbolId, Token, TokenPattern,
};
use std::collections::BTreeMap;

// ---------------------------------------------------------------------------
// Helper: build a grammar with N chain rules  A_0 → A_1 → … → A_{n-1} → 'a'
// ---------------------------------------------------------------------------
fn chain_grammar(n: usize) -> Grammar {
    assert!(n >= 1);
    let mut g = Grammar::new("chain".into());

    let a_tok = SymbolId(1);
    g.tokens.insert(
        a_tok,
        Token {
            name: "a".into(),
            pattern: TokenPattern::String("a".into()),
            fragile: false,
        },
    );

    let base_nt = 10u16;
    for i in 0..n {
        let lhs = SymbolId(base_nt + i as u16);
        g.rule_names.insert(lhs, format!("A{}", i));

        let rhs = if i + 1 < n {
            vec![Symbol::NonTerminal(SymbolId(base_nt + i as u16 + 1))]
        } else {
            vec![Symbol::Terminal(a_tok)]
        };

        g.rules.insert(
            lhs,
            vec![Rule {
                lhs,
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

/// Build a grammar with `n` alternative productions for a single nonterminal.
/// E → t_0 | t_1 | … | t_{n-1}
fn wide_grammar(n: usize) -> Grammar {
    let mut g = Grammar::new("wide".into());
    let e = SymbolId(200);
    g.rule_names.insert(e, "E".into());

    let mut rules = Vec::with_capacity(n);
    for i in 0..n {
        let tok = SymbolId(1 + i as u16);
        g.tokens.insert(
            tok,
            Token {
                name: format!("t{}", i),
                pattern: TokenPattern::String(format!("t{}", i)),
                fragile: false,
            },
        );
        rules.push(Rule {
            lhs: e,
            rhs: vec![Symbol::Terminal(tok)],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(i as u16),
        });
    }
    g.rules.insert(e, rules);
    g
}

/// Ambiguous grammar: E → 'a' | E E
fn ambiguous_ee_grammar() -> Grammar {
    let mut g = Grammar::new("ambig".into());
    let a = SymbolId(1);
    let e = SymbolId(10);

    g.tokens.insert(
        a,
        Token {
            name: "a".into(),
            pattern: TokenPattern::String("a".into()),
            fragile: false,
        },
    );
    g.rule_names.insert(e, "E".into());
    g.rules.insert(
        e,
        vec![
            Rule {
                lhs: e,
                rhs: vec![Symbol::Terminal(a)],
                precedence: None,
                associativity: None,
                fields: vec![],
                production_id: ProductionId(0),
            },
            Rule {
                lhs: e,
                rhs: vec![Symbol::NonTerminal(e), Symbol::NonTerminal(e)],
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
// 1. FIRST/FOLLOW computation on grammars with 100+ rules
// ---------------------------------------------------------------------------

#[test]
fn first_follow_100_chain_rules() {
    let g = chain_grammar(120);
    let ff = FirstFollowSets::compute(&g).expect("FIRST/FOLLOW should succeed on 120-rule chain");

    // The start symbol (A0) should have 'a' (SymbolId 1) in its FIRST set
    let start = SymbolId(10);
    let first = ff.first(start).expect("FIRST(A0) must exist");
    assert!(
        first.contains(1),
        "terminal 'a' (id=1) must be in FIRST(A0)"
    );

    // No nonterminal in this chain should be nullable
    for i in 0..120u16 {
        assert!(
            !ff.is_nullable(SymbolId(10 + i)),
            "A{} must not be nullable",
            i
        );
    }
}

#[test]
fn first_follow_wide_grammar_150_alternatives() {
    let g = wide_grammar(150);
    let ff = FirstFollowSets::compute(&g).expect("FIRST/FOLLOW should succeed on 150-alt grammar");

    let e = SymbolId(200);
    let first = ff.first(e).expect("FIRST(E) must exist");
    // Every token t_0..t_149 should be in FIRST(E)
    for i in 0..150u16 {
        assert!(
            first.contains((1 + i) as usize),
            "t{} must be in FIRST(E)",
            i
        );
    }
}

// ---------------------------------------------------------------------------
// 2. LR(1) canonical collection for grammars with many states
// ---------------------------------------------------------------------------

#[test]
fn canonical_collection_chain_grammar() {
    let g = chain_grammar(50);
    let ff = FirstFollowSets::compute(&g).unwrap();
    let coll = ItemSetCollection::build_canonical_collection(&g, &ff);

    // A chain of 50 rules should produce at least 50 states (one per symbol transition)
    assert!(
        coll.sets.len() >= 50,
        "expected ≥50 states, got {}",
        coll.sets.len()
    );

    // Every item set should be non-empty
    for (i, set) in coll.sets.iter().enumerate() {
        assert!(
            !set.items.is_empty(),
            "state {} must have at least one item",
            i
        );
    }
}

#[test]
fn canonical_collection_wide_grammar() {
    let g = wide_grammar(80);
    let ff = FirstFollowSets::compute(&g).unwrap();
    let coll = ItemSetCollection::build_canonical_collection(&g, &ff);

    // Wide grammar: 80 alternatives should produce at least 80 shift-states + initial + reduce
    assert!(
        coll.sets.len() >= 80,
        "expected ≥80 states, got {}",
        coll.sets.len()
    );
}

// ---------------------------------------------------------------------------
// 3. GLR conflict detection on highly ambiguous grammars
// ---------------------------------------------------------------------------

#[test]
fn conflict_detection_ambiguous_grammar() {
    let g = ambiguous_ee_grammar();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let coll = ItemSetCollection::build_canonical_collection(&g, &ff);
    let resolver = ConflictResolver::detect_conflicts(&coll, &g, &ff);

    assert!(
        !resolver.conflicts.is_empty(),
        "E → a | E E must produce conflicts"
    );

    // Should contain at least one shift/reduce conflict
    let has_sr = resolver
        .conflicts
        .iter()
        .any(|c| c.conflict_type == ConflictType::ShiftReduce);
    assert!(has_sr, "should contain a shift/reduce conflict");
}

#[test]
fn conflict_detection_multi_ambiguous() {
    // E → 'a' | E E | E E E  —  triple ambiguity
    let mut g = Grammar::new("triple_ambig".into());
    let a = SymbolId(1);
    let e = SymbolId(10);

    g.tokens.insert(
        a,
        Token {
            name: "a".into(),
            pattern: TokenPattern::String("a".into()),
            fragile: false,
        },
    );
    g.rule_names.insert(e, "E".into());
    g.rules.insert(
        e,
        vec![
            Rule {
                lhs: e,
                rhs: vec![Symbol::Terminal(a)],
                precedence: None,
                associativity: None,
                fields: vec![],
                production_id: ProductionId(0),
            },
            Rule {
                lhs: e,
                rhs: vec![Symbol::NonTerminal(e), Symbol::NonTerminal(e)],
                precedence: None,
                associativity: None,
                fields: vec![],
                production_id: ProductionId(1),
            },
            Rule {
                lhs: e,
                rhs: vec![
                    Symbol::NonTerminal(e),
                    Symbol::NonTerminal(e),
                    Symbol::NonTerminal(e),
                ],
                precedence: None,
                associativity: None,
                fields: vec![],
                production_id: ProductionId(2),
            },
        ],
    );

    let ff = FirstFollowSets::compute(&g).unwrap();
    let coll = ItemSetCollection::build_canonical_collection(&g, &ff);
    let resolver = ConflictResolver::detect_conflicts(&coll, &g, &ff);

    assert!(
        resolver.conflicts.len() >= 2,
        "triple-ambiguous grammar should have multiple conflicts, got {}",
        resolver.conflicts.len()
    );
}

// ---------------------------------------------------------------------------
// 4. Memory: verify no excessive allocation for large item sets
// ---------------------------------------------------------------------------

#[test]
fn large_item_set_memory_bounded() {
    // Build a grammar that produces a large closure: S → A B C … with many nonterminals
    let mut g = Grammar::new("fat_closure".into());
    let tok = SymbolId(1);
    g.tokens.insert(
        tok,
        Token {
            name: "x".into(),
            pattern: TokenPattern::String("x".into()),
            fragile: false,
        },
    );

    let n = 60usize;
    let base = 10u16;

    // S → A_0 A_1 … A_{n-1}
    let s = SymbolId(base);
    g.rule_names.insert(s, "S".into());
    let rhs: Vec<Symbol> = (1..=n)
        .map(|i| Symbol::NonTerminal(SymbolId(base + i as u16)))
        .collect();
    g.rules.insert(
        s,
        vec![Rule {
            lhs: s,
            rhs,
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(0),
        }],
    );

    // Each A_i → 'x'
    for i in 1..=n {
        let nt = SymbolId(base + i as u16);
        g.rule_names.insert(nt, format!("A{}", i));
        g.rules.insert(
            nt,
            vec![Rule {
                lhs: nt,
                rhs: vec![Symbol::Terminal(tok)],
                precedence: None,
                associativity: None,
                fields: vec![],
                production_id: ProductionId(i as u16),
            }],
        );
    }

    let ff = FirstFollowSets::compute(&g).unwrap();

    // Build initial item set and closure
    let mut initial = ItemSet::new(StateId(0));
    let start_rules = g.get_rules_for_symbol(s).unwrap();
    for r in start_rules {
        initial.add_item(adze_glr_core::LRItem::new(
            RuleId(r.production_id.0),
            0,
            SymbolId(0),
        ));
    }
    let _ = initial.closure(&g, &ff);

    // The closure expands A_1 since it's at position 0, so we should see at
    // least the kernel item + the closure-expanded A_1 → • x item
    assert!(
        initial.items.len() >= 2,
        "closure should expand, got {} items",
        initial.items.len()
    );
    // But it shouldn't explode (bound by O(rules * terminals) for lookaheads)
    assert!(
        initial.items.len() < 500,
        "closure too large: {} items (potential blowup)",
        initial.items.len()
    );
}

// ---------------------------------------------------------------------------
// 5. Edge cases: empty grammar, single-token grammar, left-recursive grammar
// ---------------------------------------------------------------------------

#[test]
fn edge_case_single_token_grammar() {
    // S → 'a'
    let mut g = Grammar::new("single".into());
    let a = SymbolId(1);
    let s = SymbolId(10);

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

    let ff = FirstFollowSets::compute(&g).unwrap();
    assert!(ff.first(s).unwrap().contains(1));
    assert!(!ff.is_nullable(s));

    let coll = ItemSetCollection::build_canonical_collection(&g, &ff);
    // Minimal grammar: initial state, after-shift state, possibly accept state
    assert!(
        coll.sets.len() >= 2,
        "single-token grammar should have at least 2 states, got {}",
        coll.sets.len()
    );

    // Full pipeline: build parse table
    let pt = adze_glr_core::build_lr1_automaton(&g, &ff).unwrap();
    // The table should have an Accept action somewhere on EOF
    let eof_col = pt.symbol_to_index.get(&pt.eof_symbol);
    assert!(eof_col.is_some(), "EOF must be in symbol_to_index");
    let has_accept = pt.action_table.iter().any(|row| {
        row.get(*eof_col.unwrap())
            .map(|cell| cell.iter().any(|a| matches!(a, Action::Accept)))
            .unwrap_or(false)
    });
    assert!(
        has_accept,
        "single-token grammar must produce an Accept action"
    );
}

#[test]
fn edge_case_left_recursive_grammar() {
    // L → L 'a' | 'a'  (direct left recursion)
    let mut g = Grammar::new("left_rec".into());
    let a = SymbolId(1);
    let l = SymbolId(10);

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
                rhs: vec![Symbol::NonTerminal(l), Symbol::Terminal(a)],
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

    let ff = FirstFollowSets::compute(&g).unwrap();
    // FIRST(L) should contain 'a'
    assert!(ff.first(l).unwrap().contains(1));
    // L is not nullable (both alternatives start with a terminal or L which starts with 'a')
    assert!(!ff.is_nullable(l));

    let coll = ItemSetCollection::build_canonical_collection(&g, &ff);
    assert!(!coll.sets.is_empty());

    // The pipeline should handle left recursion without infinite loop
    let pt = adze_glr_core::build_lr1_automaton(&g, &ff).unwrap();
    assert!(pt.state_count > 0);
}

#[test]
fn edge_case_epsilon_production() {
    // S → A 'b'; A → ε | 'a'
    let mut g = Grammar::new("eps".into());
    let a_tok = SymbolId(1);
    let b_tok = SymbolId(2);
    let s = SymbolId(10);
    let a_nt = SymbolId(11);

    g.tokens.insert(
        a_tok,
        Token {
            name: "a".into(),
            pattern: TokenPattern::String("a".into()),
            fragile: false,
        },
    );
    g.tokens.insert(
        b_tok,
        Token {
            name: "b".into(),
            pattern: TokenPattern::String("b".into()),
            fragile: false,
        },
    );
    g.rule_names.insert(s, "S".into());
    g.rule_names.insert(a_nt, "A".into());

    g.rules.insert(
        a_nt,
        vec![
            Rule {
                lhs: a_nt,
                rhs: vec![],
                precedence: None,
                associativity: None,
                fields: vec![],
                production_id: ProductionId(0),
            },
            Rule {
                lhs: a_nt,
                rhs: vec![Symbol::Terminal(a_tok)],
                precedence: None,
                associativity: None,
                fields: vec![],
                production_id: ProductionId(1),
            },
        ],
    );
    g.rules.insert(
        s,
        vec![Rule {
            lhs: s,
            rhs: vec![Symbol::NonTerminal(a_nt), Symbol::Terminal(b_tok)],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(2),
        }],
    );

    let ff = FirstFollowSets::compute(&g).unwrap();
    assert!(
        ff.is_nullable(a_nt),
        "A must be nullable (has ε production)"
    );
    assert!(!ff.is_nullable(s), "S must not be nullable");

    // FIRST(S) should contain 'a' and 'b' (because A is nullable)
    let first_s = ff.first(s).unwrap();
    assert!(first_s.contains(1), "'a' in FIRST(S)");
    assert!(first_s.contains(2), "'b' in FIRST(S) because A is nullable");
}

// ---------------------------------------------------------------------------
// 6. Driver execution with multi-way ambiguity (3+ forks)
// ---------------------------------------------------------------------------

/// Build a hand-crafted parse table that forces 3-way forking on a single symbol.
fn three_way_fork_table() -> ParseTable {
    // Symbols: 0=EOF, 1='a', 2=S(start)
    // Rules: R0: S → 'a' (rhs_len 1), R1: S → 'a' (rhs_len 1), R2: S → 'a' (rhs_len 1)
    //   (three different reductions for the same input to force 3 forks)
    //
    // States:
    //   0: on 'a' → Shift(1)
    //   1: on EOF → [Reduce(R0), Reduce(R1), Reduce(R2)]  (3-way fork)
    //   2: on EOF → Accept  (after goto on S)

    let s_sym = SymbolId(2);
    let eof = SymbolId(0);

    let rules = vec![
        ParseRule {
            lhs: s_sym,
            rhs_len: 1,
        },
        ParseRule {
            lhs: s_sym,
            rhs_len: 1,
        },
        ParseRule {
            lhs: s_sym,
            rhs_len: 1,
        },
    ];

    let sym_count = 3; // 0=EOF, 1='a', 2=S
    let state_count = 3;

    // action_table[state][symbol_index]
    let mut actions = vec![vec![vec![]; sym_count]; state_count];

    // State 0: shift 'a' → state 1
    actions[0][1] = vec![Action::Shift(StateId(1))];

    // State 1: 3-way fork on EOF
    actions[1][0] = vec![
        Action::Reduce(RuleId(0)),
        Action::Reduce(RuleId(1)),
        Action::Reduce(RuleId(2)),
    ];

    // State 2: accept on EOF
    actions[2][0] = vec![Action::Accept];

    let invalid = StateId(65535);
    let mut gotos = vec![vec![invalid; sym_count]; state_count];
    gotos[0][2] = StateId(2); // goto S after reduce in state 0
    gotos[1][2] = StateId(2); // goto S after reduce in state 1

    let mut symbol_to_index = BTreeMap::new();
    symbol_to_index.insert(SymbolId(0), 0);
    symbol_to_index.insert(SymbolId(1), 1);
    symbol_to_index.insert(SymbolId(2), 2);

    let mut nonterminal_to_index = BTreeMap::new();
    nonterminal_to_index.insert(s_sym, 2);

    ParseTable {
        action_table: actions,
        goto_table: gotos,
        rules,
        state_count,
        symbol_count: sym_count,
        symbol_to_index,
        index_to_symbol: vec![SymbolId(0), SymbolId(1), SymbolId(2)],
        external_scanner_states: vec![vec![]; state_count],
        nonterminal_to_index,
        goto_indexing: GotoIndexing::NonterminalMap,
        eof_symbol: eof,
        start_symbol: s_sym,
        grammar: Grammar::new("fork3".into()),
        symbol_metadata: vec![],
        initial_state: StateId(0),
        token_count: 2,
        external_token_count: 0,
        lex_modes: vec![
            LexMode {
                lex_state: 0,
                external_lex_state: 0
            };
            state_count
        ],
        extras: vec![],
        dynamic_prec_by_rule: vec![0; 3],
        rule_assoc_by_rule: vec![0; 3],
        alias_sequences: vec![],
        field_names: vec![],
        field_map: BTreeMap::new(),
    }
}

#[test]
fn driver_three_way_fork() {
    let table = three_way_fork_table();
    let mut driver = Driver::new(&table);

    // Input: single token 'a' at byte range [0,1)
    let tokens = vec![(1u32, 0u32, 1u32)];
    let result = driver.parse_tokens(tokens);

    assert!(
        result.is_ok(),
        "3-way fork parse should succeed: {:?}",
        result.err()
    );
    let forest = result.unwrap();
    let view = forest.view();
    let roots = view.roots();
    // The driver should produce at least one root (it picks the best among forks)
    assert!(!roots.is_empty(), "forest must have at least one root");
}

// ---------------------------------------------------------------------------
// 7. ParseTable serialization/deserialization with large tables
// ---------------------------------------------------------------------------

#[cfg(feature = "serialization")]
mod serialization_stress {
    use super::*;

    /// Build a large parse table (many states, symbols, multi-action cells).
    fn large_parse_table(states: usize, syms: usize) -> ParseTable {
        let mut actions = Vec::with_capacity(states);
        for s in 0..states {
            let mut row = Vec::with_capacity(syms);
            for c in 0..syms {
                if s == 0 && c == 0 {
                    row.push(vec![Action::Accept]);
                } else if c < syms / 2 {
                    row.push(vec![Action::Shift(StateId((s + 1) as u16 % states as u16))]);
                } else {
                    // Multi-action cell to exercise GLR serialization
                    row.push(vec![Action::Shift(StateId(0)), Action::Reduce(RuleId(0))]);
                }
            }
            actions.push(row);
        }

        let gotos = vec![vec![StateId(0); syms]; states];

        let mut symbol_to_index = BTreeMap::new();
        for i in 0..syms {
            symbol_to_index.insert(SymbolId(i as u16), i);
        }

        ParseTable {
            action_table: actions,
            goto_table: gotos,
            rules: vec![ParseRule {
                lhs: SymbolId(0),
                rhs_len: 1,
            }],
            state_count: states,
            symbol_count: syms,
            symbol_to_index,
            index_to_symbol: (0..syms).map(|i| SymbolId(i as u16)).collect(),
            external_scanner_states: vec![vec![]; states],
            nonterminal_to_index: BTreeMap::new(),
            goto_indexing: GotoIndexing::NonterminalMap,
            eof_symbol: SymbolId(0),
            start_symbol: SymbolId(1),
            grammar: Grammar::new("large".into()),
            symbol_metadata: vec![],
            initial_state: StateId(0),
            token_count: syms / 2,
            external_token_count: 0,
            lex_modes: vec![
                LexMode {
                    lex_state: 0,
                    external_lex_state: 0
                };
                states
            ],
            extras: vec![],
            dynamic_prec_by_rule: vec![0],
            rule_assoc_by_rule: vec![0],
            alias_sequences: vec![],
            field_names: vec![],
            field_map: BTreeMap::new(),
        }
    }

    #[test]
    fn roundtrip_large_table_200_states_100_symbols() {
        let table = large_parse_table(200, 100);
        let bytes = table.to_bytes().expect("serialize large table");
        let restored = ParseTable::from_bytes(&bytes).expect("deserialize large table");

        assert_eq!(restored.state_count, 200);
        assert_eq!(restored.symbol_count, 100);
        assert_eq!(restored.action_table.len(), 200);
        assert_eq!(restored.action_table[0].len(), 100);
    }

    #[test]
    fn roundtrip_preserves_multi_action_cells() {
        let table = large_parse_table(50, 50);
        let bytes = table.to_bytes().unwrap();
        let restored = ParseTable::from_bytes(&bytes).unwrap();

        // Check a multi-action cell in the upper-right quadrant
        let cell = &restored.action_table[1][26]; // sym 26 ≥ 25 = syms/2
        assert_eq!(cell.len(), 2, "multi-action cell should survive roundtrip");
    }

    #[test]
    fn serialization_size_bounded() {
        let table = large_parse_table(100, 80);
        let bytes = table.to_bytes().unwrap();
        // 100 states × 80 symbols × ~2 actions × ~3 bytes/action < 200 KB is generous
        assert!(
            bytes.len() < 500_000,
            "serialized size {} bytes seems excessive",
            bytes.len()
        );
    }
}

// ---------------------------------------------------------------------------
// 8. Symbol resolution with overlapping terminal/nonterminal ranges
// ---------------------------------------------------------------------------

#[test]
fn symbol_resolution_overlapping_ranges() {
    // Set up a grammar where terminal IDs and nonterminal IDs are in adjacent
    // ranges, verifying the pipeline distinguishes them correctly.
    let mut g = Grammar::new("overlap".into());

    // Terminals: 1, 2, 3
    for i in 1..=3u16 {
        g.tokens.insert(
            SymbolId(i),
            Token {
                name: format!("t{}", i),
                pattern: TokenPattern::String(format!("{}", i)),
                fragile: false,
            },
        );
    }

    // Nonterminals: 4, 5 (immediately after terminals)
    let s = SymbolId(4);
    let a = SymbolId(5);
    g.rule_names.insert(s, "S".into());
    g.rule_names.insert(a, "A".into());

    g.rules.insert(
        s,
        vec![Rule {
            lhs: s,
            rhs: vec![Symbol::NonTerminal(a), Symbol::Terminal(SymbolId(3))],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(0),
        }],
    );
    g.rules.insert(
        a,
        vec![
            Rule {
                lhs: a,
                rhs: vec![Symbol::Terminal(SymbolId(1))],
                precedence: None,
                associativity: None,
                fields: vec![],
                production_id: ProductionId(1),
            },
            Rule {
                lhs: a,
                rhs: vec![Symbol::Terminal(SymbolId(2))],
                precedence: None,
                associativity: None,
                fields: vec![],
                production_id: ProductionId(2),
            },
        ],
    );

    let ff = FirstFollowSets::compute(&g).unwrap();
    // FIRST(A) = {t1, t2}
    let first_a = ff.first(a).unwrap();
    assert!(first_a.contains(1));
    assert!(first_a.contains(2));
    assert!(!first_a.contains(3), "t3 must not be in FIRST(A)");

    // Build the full automaton
    let pt = adze_glr_core::build_lr1_automaton(&g, &ff).unwrap();

    // Verify the table distinguishes terminals from nonterminals
    assert!(
        pt.is_terminal(SymbolId(1)),
        "SymbolId(1) should be terminal"
    );
    assert!(
        pt.is_terminal(SymbolId(2)),
        "SymbolId(2) should be terminal"
    );
    assert!(
        pt.is_terminal(SymbolId(3)),
        "SymbolId(3) should be terminal"
    );

    // Nonterminals should have goto entries
    let has_goto_s = pt.goto_table.iter().any(|row| {
        let col = match pt.goto_indexing {
            GotoIndexing::NonterminalMap => pt.nonterminal_to_index.get(&s).copied(),
            GotoIndexing::DirectSymbolId => Some(s.0 as usize),
        };
        col.and_then(|c| row.get(c))
            .is_some_and(|st| st.0 != 0 && st.0 != 65535)
    });
    assert!(has_goto_s, "S should have a goto entry");
}

// ---------------------------------------------------------------------------
// Additional: full pipeline stress test (end-to-end)
// ---------------------------------------------------------------------------

#[test]
fn full_pipeline_100_rule_chain() {
    let g = chain_grammar(100);
    let ff = FirstFollowSets::compute(&g).unwrap();
    let pt = adze_glr_core::build_lr1_automaton(&g, &ff).unwrap();

    // Sanity: the table must have an Accept action somewhere
    let eof_col = pt.symbol_to_index[&pt.eof_symbol];
    let has_accept = pt.action_table.iter().any(|row| {
        row.get(eof_col)
            .map(|cell| cell.iter().any(|a| matches!(a, Action::Accept)))
            .unwrap_or(false)
    });
    assert!(has_accept, "100-rule chain must have Accept on EOF");
}

#[test]
fn full_pipeline_ambiguous_grammar_produces_forks() {
    let g = ambiguous_ee_grammar();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let pt = adze_glr_core::build_lr1_automaton(&g, &ff).unwrap();

    // The table should contain at least one multi-action cell (Fork or multiple actions)
    let has_multi = pt
        .action_table
        .iter()
        .any(|row| row.iter().any(|cell| cell.len() > 1));
    assert!(
        has_multi,
        "ambiguous E → a | E E must produce multi-action cells"
    );
}
