#![cfg(feature = "test-api")]

//! Comprehensive tests for GLR parse forest operations including:
//! - ParseTable construction and field defaults
//! - Action resolution (shift, reduce, accept, error, fork)
//! - Goto table behavior and remapping
//! - ParseForest / ForestNode creation and error stats
//! - Disambiguation (to_single_tree)
//! - Symbol metadata and terminal boundary checks
//!
//! Run with: cargo test -p adze-glr-core --test parse_forest_tests --features test-api

use adze_glr_core::parse_forest::{ERROR_SYMBOL, ErrorMeta, ForestAlternative, ForestNode};
use adze_glr_core::{
    Action, FirstFollowSets, GotoIndexing, LexMode, ParseForest, ParseRule, ParseTable,
    SymbolMetadata, build_lr1_automaton, sanity_check_tables,
};
use adze_ir::*;
use std::collections::{BTreeMap, HashMap};

// ═══════════════════════════════════════════════════════════════════════
// Helpers
// ═══════════════════════════════════════════════════════════════════════

const NO_GOTO: StateId = StateId(65535);
type ActionCell = Vec<Action>;

/// Build a `ParseTable` from raw action/goto matrices.
fn build_table(
    actions: Vec<Vec<ActionCell>>,
    gotos: Vec<Vec<StateId>>,
    rules: Vec<ParseRule>,
    start: SymbolId,
    eof: SymbolId,
    num_terminals: usize,
) -> ParseTable {
    let symbol_count = actions.first().map(|r| r.len()).unwrap_or(0);
    let state_count = actions.len();

    let mut symbol_to_index = BTreeMap::new();
    let mut index_to_symbol = Vec::new();
    for i in 0..symbol_count {
        symbol_to_index.insert(SymbolId(i as u16), i);
        index_to_symbol.push(SymbolId(i as u16));
    }

    let mut nonterminal_to_index = BTreeMap::new();
    for i in 0..symbol_count {
        for row in &gotos {
            if i < row.len() && row[i] != NO_GOTO {
                nonterminal_to_index.insert(SymbolId(i as u16), i);
                break;
            }
        }
    }

    ParseTable {
        action_table: actions,
        goto_table: gotos,
        rules: rules.clone(),
        state_count,
        symbol_count,
        symbol_to_index,
        index_to_symbol,
        external_scanner_states: vec![],
        nonterminal_to_index,
        eof_symbol: eof,
        start_symbol: start,
        grammar: Grammar::new("test".to_string()),
        symbol_metadata: vec![],
        initial_state: StateId(0),
        token_count: num_terminals,
        external_token_count: 0,
        lex_modes: vec![
            LexMode {
                lex_state: 0,
                external_lex_state: 0,
            };
            state_count
        ],
        extras: vec![],
        dynamic_prec_by_rule: vec![0; rules.len()],
        rule_assoc_by_rule: vec![0; rules.len()],
        alias_sequences: vec![],
        field_names: vec![],
        goto_indexing: GotoIndexing::NonterminalMap,
        field_map: BTreeMap::new(),
    }
}

/// Build a minimal grammar: S → a
fn grammar_s_to_a() -> Grammar {
    let mut g = Grammar::new("s_to_a".into());
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
    g
}

/// Build a grammar: S → a b
fn grammar_s_to_ab() -> Grammar {
    let mut g = Grammar::new("s_to_ab".into());
    let a = SymbolId(1);
    let b = SymbolId(2);
    let s = SymbolId(10);
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
    g.rules.insert(
        s,
        vec![Rule {
            lhs: s,
            rhs: vec![Symbol::Terminal(a), Symbol::Terminal(b)],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(0),
        }],
    );
    g
}

/// Resolve a symbol name to its SymbolId inside a built grammar.
fn sym_id(grammar: &Grammar, name: &str) -> SymbolId {
    for (&id, tok) in &grammar.tokens {
        if tok.name == name {
            return id;
        }
    }
    for (&id, n) in &grammar.rule_names {
        if n == name {
            return id;
        }
    }
    panic!("symbol '{}' not found in grammar", name);
}

// ═══════════════════════════════════════════════════════════════════════
// 1. ParseTable default construction
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_parse_table_default_has_empty_tables() {
    let table = ParseTable::default();
    assert!(table.action_table.is_empty());
    assert!(table.goto_table.is_empty());
    assert!(table.rules.is_empty());
    assert_eq!(table.state_count, 0);
    assert_eq!(table.symbol_count, 0);
    assert_eq!(table.eof_symbol, SymbolId(0));
    assert!(matches!(table.goto_indexing, GotoIndexing::NonterminalMap));
}

// ═══════════════════════════════════════════════════════════════════════
// 2. Action cell operations
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_action_shift_equality() {
    let a1 = Action::Shift(StateId(5));
    let a2 = Action::Shift(StateId(5));
    let a3 = Action::Shift(StateId(6));
    assert_eq!(a1, a2);
    assert_ne!(a1, a3);
}

#[test]
fn test_action_reduce_equality() {
    let a1 = Action::Reduce(RuleId(0));
    let a2 = Action::Reduce(RuleId(0));
    let a3 = Action::Reduce(RuleId(1));
    assert_eq!(a1, a2);
    assert_ne!(a1, a3);
}

#[test]
fn test_action_variants_are_distinct() {
    let shift = Action::Shift(StateId(0));
    let reduce = Action::Reduce(RuleId(0));
    let accept = Action::Accept;
    let error = Action::Error;
    let recover = Action::Recover;
    assert_ne!(shift, reduce);
    assert_ne!(reduce, accept);
    assert_ne!(accept, error);
    assert_ne!(error, recover);
}

#[test]
fn test_fork_action_holds_multiple_actions() {
    let fork = Action::Fork(vec![Action::Shift(StateId(1)), Action::Reduce(RuleId(0))]);
    if let Action::Fork(inner) = &fork {
        assert_eq!(inner.len(), 2);
        assert!(matches!(inner[0], Action::Shift(StateId(1))));
        assert!(matches!(inner[1], Action::Reduce(RuleId(0))));
    } else {
        panic!("expected Fork");
    }
}

// ═══════════════════════════════════════════════════════════════════════
// 3. ParseTable.actions() lookup
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_actions_returns_shift_for_terminal() {
    // S → a grammar, build automaton
    let g = grammar_s_to_a();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let table = build_lr1_automaton(&g, &ff).unwrap();

    let a = sym_id(&g, "a");
    let actions = table.actions(table.initial_state, a);
    assert!(
        actions.iter().any(|act| matches!(act, Action::Shift(_))),
        "initial state should shift on terminal 'a'"
    );
}

#[test]
fn test_actions_empty_for_unmapped_symbol() {
    let g = grammar_s_to_a();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let table = build_lr1_automaton(&g, &ff).unwrap();

    // SymbolId(999) is not in the grammar
    let actions = table.actions(table.initial_state, SymbolId(999));
    assert!(actions.is_empty());
}

#[test]
fn test_actions_empty_for_out_of_range_state() {
    let g = grammar_s_to_a();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let table = build_lr1_automaton(&g, &ff).unwrap();

    let a = sym_id(&g, "a");
    let bogus_state = StateId(table.state_count as u16 + 100);
    let actions = table.actions(bogus_state, a);
    assert!(actions.is_empty());
}

// ═══════════════════════════════════════════════════════════════════════
// 4. Goto table behavior
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_goto_returns_some_for_start_symbol() {
    let g = grammar_s_to_a();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let table = build_lr1_automaton(&g, &ff).unwrap();

    let s = sym_id(&g, "S");
    let target = table.goto(table.initial_state, s);
    assert!(target.is_some(), "goto(initial, S) should exist");
}

#[test]
fn test_goto_returns_none_for_terminal() {
    let g = grammar_s_to_a();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let table = build_lr1_automaton(&g, &ff).unwrap();

    let a = sym_id(&g, "a");
    // Terminals don't appear in the nonterminal_to_index map
    let target = table.goto(table.initial_state, a);
    assert!(target.is_none(), "goto for a terminal should be None");
}

#[test]
fn test_goto_remap_to_direct_symbol_id_and_back() {
    let g = grammar_s_to_a();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let table = build_lr1_automaton(&g, &ff).unwrap();

    let s = sym_id(&g, "S");
    let original_target = table.goto(table.initial_state, s);

    // Remap to direct symbol ID layout
    let table = table.remap_goto_to_direct_symbol_id();
    assert!(matches!(table.goto_indexing, GotoIndexing::DirectSymbolId));

    // Remap back to nonterminal map
    let table = table.remap_goto_to_nonterminal_map();
    assert!(matches!(table.goto_indexing, GotoIndexing::NonterminalMap));

    let round_trip_target = table.goto(table.initial_state, s);
    assert_eq!(original_target, round_trip_target);
}

#[test]
fn test_remap_noop_when_already_correct_indexing() {
    let g = grammar_s_to_a();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let table = build_lr1_automaton(&g, &ff).unwrap();

    // Already NonterminalMap, remap should be a no-op
    assert!(matches!(table.goto_indexing, GotoIndexing::NonterminalMap));
    let table = table.remap_goto_to_nonterminal_map();
    assert!(matches!(table.goto_indexing, GotoIndexing::NonterminalMap));

    // Remap to direct and then try again — should be a no-op
    let table = table.remap_goto_to_direct_symbol_id();
    assert!(matches!(table.goto_indexing, GotoIndexing::DirectSymbolId));
    let table = table.remap_goto_to_direct_symbol_id();
    assert!(matches!(table.goto_indexing, GotoIndexing::DirectSymbolId));
}

// ═══════════════════════════════════════════════════════════════════════
// 5. Terminal boundary & is_terminal
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_terminal_boundary_no_external_tokens() {
    let g = grammar_s_to_a();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let table = build_lr1_automaton(&g, &ff).unwrap();

    assert_eq!(table.external_token_count, 0);
    assert_eq!(table.terminal_boundary(), table.token_count);
}

#[test]
fn test_is_terminal_for_known_symbols() {
    let g = grammar_s_to_a();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let table = build_lr1_automaton(&g, &ff).unwrap();

    let a = sym_id(&g, "a");
    let s = sym_id(&g, "S");

    assert!(table.is_terminal(a), "terminal 'a' should be terminal");
    assert!(
        !table.is_terminal(s),
        "nonterminal 'S' should not be terminal"
    );
}

// ═══════════════════════════════════════════════════════════════════════
// 6. ForestNode creation and completeness
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_forest_node_is_complete_with_alternative() {
    let node = ForestNode {
        id: 0,
        symbol: SymbolId(1),
        span: (0, 5),
        alternatives: vec![ForestAlternative {
            children: vec![1, 2],
        }],
        error_meta: ErrorMeta::default(),
    };
    assert!(node.is_complete());
}

#[test]
fn test_forest_node_incomplete_no_alternatives() {
    let node = ForestNode {
        id: 0,
        symbol: SymbolId(1),
        span: (0, 5),
        alternatives: vec![],
        error_meta: ErrorMeta::default(),
    };
    assert!(!node.is_complete());
}

#[test]
fn test_forest_node_multiple_alternatives() {
    let node = ForestNode {
        id: 0,
        symbol: SymbolId(1),
        span: (0, 10),
        alternatives: vec![
            ForestAlternative { children: vec![1] },
            ForestAlternative {
                children: vec![2, 3],
            },
        ],
        error_meta: ErrorMeta::default(),
    };
    assert!(node.is_complete());
    assert_eq!(node.alternatives.len(), 2);
}

// ═══════════════════════════════════════════════════════════════════════
// 7. ParseForest error stats
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_parse_forest_push_error_chunk() {
    let mut forest = ParseForest {
        roots: vec![],
        nodes: HashMap::new(),
        grammar: Grammar::new("test".into()),
        source: "abc".into(),
        next_node_id: 0,
    };

    let id = forest.push_error_chunk((0, 3));
    assert_eq!(id, 0);

    let node = &forest.nodes[&id];
    assert_eq!(node.symbol, ERROR_SYMBOL);
    assert_eq!(node.span, (0, 3));
    assert!(node.error_meta.is_error);
    assert!(!node.error_meta.missing);
    assert_eq!(node.error_meta.cost, 1);
}

#[test]
fn test_parse_forest_error_stats_no_errors() {
    let forest = ParseForest {
        roots: vec![],
        nodes: HashMap::new(),
        grammar: Grammar::new("test".into()),
        source: "".into(),
        next_node_id: 0,
    };

    let (has_error, missing, cost) = forest.debug_error_stats();
    assert!(!has_error);
    assert_eq!(missing, 0);
    assert_eq!(cost, 0);
}

#[test]
fn test_parse_forest_error_stats_with_error_chunk() {
    let mut forest = ParseForest {
        roots: vec![],
        nodes: HashMap::new(),
        grammar: Grammar::new("test".into()),
        source: "xyz".into(),
        next_node_id: 0,
    };
    forest.push_error_chunk((0, 3));

    let (has_error, missing, cost) = forest.debug_error_stats();
    assert!(has_error);
    assert_eq!(missing, 0);
    assert_eq!(cost, 1);
}

#[test]
fn test_parse_forest_error_stats_with_missing_terminal() {
    let mut forest = ParseForest {
        roots: vec![],
        nodes: HashMap::new(),
        grammar: Grammar::new("test".into()),
        source: "".into(),
        next_node_id: 0,
    };

    // Manually insert a missing-terminal node
    forest.nodes.insert(
        0,
        ForestNode {
            id: 0,
            symbol: SymbolId(5),
            span: (0, 0),
            alternatives: vec![ForestAlternative { children: vec![] }],
            error_meta: ErrorMeta {
                missing: true,
                is_error: false,
                cost: 1,
            },
        },
    );
    forest.next_node_id = 1;

    let (has_error, missing, cost) = forest.debug_error_stats();
    assert!(!has_error);
    assert_eq!(missing, 1);
    assert_eq!(cost, 1);
}

#[test]
fn test_parse_forest_multiple_error_chunks_accumulate_cost() {
    let mut forest = ParseForest {
        roots: vec![],
        nodes: HashMap::new(),
        grammar: Grammar::new("test".into()),
        source: "abcdef".into(),
        next_node_id: 0,
    };
    forest.push_error_chunk((0, 3));
    forest.push_error_chunk((3, 6));

    let (has_error, missing, cost) = forest.debug_error_stats();
    assert!(has_error);
    assert_eq!(missing, 0);
    assert_eq!(cost, 2);
}

// ═══════════════════════════════════════════════════════════════════════
// 8. Disambiguation — to_single_tree
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_to_single_tree_returns_incomplete_when_no_roots() {
    let forest = ParseForest {
        roots: vec![],
        nodes: HashMap::new(),
        grammar: Grammar::new("test".into()),
        source: "".into(),
        next_node_id: 0,
    };
    let result = forest.to_single_tree();
    assert!(result.is_err());
}

#[test]
fn test_to_single_tree_succeeds_with_complete_root() {
    let start = SymbolId(10);
    let mut g = Grammar::new("test".into());
    g.rule_names.insert(start, "S".into());
    g.rules.insert(
        start,
        vec![Rule {
            lhs: start,
            rhs: vec![],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(0),
        }],
    );

    let root_node = ForestNode {
        id: 0,
        symbol: start,
        span: (0, 5),
        alternatives: vec![ForestAlternative { children: vec![] }],
        error_meta: ErrorMeta::default(),
    };

    let forest = ParseForest {
        roots: vec![root_node],
        nodes: HashMap::new(),
        grammar: g,
        source: "hello".into(),
        next_node_id: 1,
    };

    let tree = forest.to_single_tree().expect("should produce a tree");
    assert_eq!(tree.root.symbol, start);
    assert_eq!(tree.root.span, (0, 5));
    assert!(tree.root.children.is_empty());
}

// ═══════════════════════════════════════════════════════════════════════
// 9. Build LR1 automaton & sanity check
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_build_lr1_automaton_s_to_a_passes_sanity_check() {
    let g = grammar_s_to_a();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let table = build_lr1_automaton(&g, &ff).unwrap();
    sanity_check_tables(&table).expect("sanity check should pass for S→a");
}

#[test]
fn test_build_lr1_automaton_s_to_ab_has_correct_rule() {
    let g = grammar_s_to_ab();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let table = build_lr1_automaton(&g, &ff).unwrap();

    // The grammar has one user rule: S → a b (rhs_len=2)
    let s = sym_id(&g, "S");
    let user_rule = table
        .rules
        .iter()
        .find(|r| r.lhs == s)
        .expect("should have a rule with LHS = S");
    assert_eq!(user_rule.rhs_len, 2);
}

// ═══════════════════════════════════════════════════════════════════════
// 10. ParseTable.rule() accessor
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_parse_table_rule_accessor() {
    let rules = vec![
        ParseRule {
            lhs: SymbolId(10),
            rhs_len: 3,
        },
        ParseRule {
            lhs: SymbolId(11),
            rhs_len: 1,
        },
    ];
    let table = build_table(
        vec![vec![vec![]; 3]],
        vec![vec![NO_GOTO; 3]],
        rules,
        SymbolId(2),
        SymbolId(0),
        2,
    );

    let (lhs, len) = table.rule(RuleId(0));
    assert_eq!(lhs, SymbolId(10));
    assert_eq!(len, 3);

    let (lhs, len) = table.rule(RuleId(1));
    assert_eq!(lhs, SymbolId(11));
    assert_eq!(len, 1);
}

// ═══════════════════════════════════════════════════════════════════════
// 11. ParseTable eof/start accessors
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_parse_table_eof_and_start_accessors() {
    let table = build_table(
        vec![vec![vec![]; 4]],
        vec![vec![NO_GOTO; 4]],
        vec![],
        SymbolId(3),
        SymbolId(0),
        2,
    );
    assert_eq!(table.eof(), SymbolId(0));
    assert_eq!(table.start_symbol(), SymbolId(3));
}

// ═══════════════════════════════════════════════════════════════════════
// 12. Valid symbols mask
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_valid_symbols_reflects_nonempty_cells() {
    let g = grammar_s_to_a();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let table = build_lr1_automaton(&g, &ff).unwrap();

    let mask = table.valid_symbols(table.initial_state);
    // At least one terminal should have a valid action in the initial state
    assert!(
        mask.iter().any(|&v| v),
        "initial state should have at least one valid terminal action"
    );
}

#[test]
fn test_valid_symbols_empty_for_out_of_range_state() {
    let g = grammar_s_to_a();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let table = build_lr1_automaton(&g, &ff).unwrap();

    let bogus = StateId(table.state_count as u16 + 50);
    let mask = table.valid_symbols(bogus);
    assert!(mask.iter().all(|&v| !v));
}

// ═══════════════════════════════════════════════════════════════════════
// 13. SymbolMetadata construction
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_symbol_metadata_fields() {
    let meta = SymbolMetadata {
        name: "identifier".into(),
        is_visible: true,
        is_named: true,
        is_supertype: false,
        is_terminal: true,
        is_extra: false,
        is_fragile: false,
        symbol_id: SymbolId(5),
    };
    assert_eq!(meta.name, "identifier");
    assert!(meta.is_visible);
    assert!(meta.is_named);
    assert!(!meta.is_supertype);
    assert!(meta.is_terminal);
    assert!(!meta.is_extra);
    assert!(!meta.is_fragile);
    assert_eq!(meta.symbol_id, SymbolId(5));
}

// ═══════════════════════════════════════════════════════════════════════
// 14. ERROR_SYMBOL sentinel
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_error_symbol_is_u16_max() {
    assert_eq!(ERROR_SYMBOL, SymbolId(u16::MAX));
}

#[test]
fn test_error_symbol_differs_from_any_grammar_symbol() {
    let g = grammar_s_to_a();
    for &id in g.tokens.keys() {
        assert_ne!(id, ERROR_SYMBOL);
    }
    for &id in g.rule_names.keys() {
        assert_ne!(id, ERROR_SYMBOL);
    }
}

// ═══════════════════════════════════════════════════════════════════════
// 15. Handcrafted table with conflict cell
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_handcrafted_table_with_shift_reduce_conflict() {
    // 3 symbols: 0=eof, 1=tok, 2=NT_S
    // 2 states: state 0 shifts on tok, state 1 has conflict on eof
    let rules = vec![ParseRule {
        lhs: SymbolId(2),
        rhs_len: 1,
    }];
    let actions = vec![
        // state 0: shift tok→state1, nothing on eof
        vec![vec![], vec![Action::Shift(StateId(1))], vec![]],
        // state 1: accept AND reduce on eof (conflict cell)
        vec![
            vec![Action::Accept, Action::Reduce(RuleId(0))],
            vec![],
            vec![],
        ],
    ];
    let gotos = vec![
        vec![NO_GOTO, NO_GOTO, StateId(1)],
        vec![NO_GOTO, NO_GOTO, NO_GOTO],
    ];

    let table = build_table(actions, gotos, rules, SymbolId(2), SymbolId(0), 2);

    // Verify shift in state 0
    let acts0 = table.actions(StateId(0), SymbolId(1));
    assert_eq!(acts0.len(), 1);
    assert!(matches!(acts0[0], Action::Shift(StateId(1))));

    // Verify conflict cell in state 1
    let acts1 = table.actions(StateId(1), SymbolId(0));
    assert_eq!(acts1.len(), 2);
    assert!(acts1.iter().any(|a| matches!(a, Action::Accept)));
    assert!(acts1.iter().any(|a| matches!(a, Action::Reduce(RuleId(0)))));
}

// ═══════════════════════════════════════════════════════════════════════
// 16. Lex mode accessor
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_lex_mode_returns_default_for_out_of_range() {
    let g = grammar_s_to_a();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let table = build_lr1_automaton(&g, &ff).unwrap();

    let mode = table.lex_mode(StateId(table.state_count as u16 + 10));
    assert_eq!(mode.lex_state, 0);
    assert_eq!(mode.external_lex_state, 0);
}

// ═══════════════════════════════════════════════════════════════════════
// 17. Extras (is_extra)
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_is_extra_false_when_no_extras() {
    let g = grammar_s_to_a();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let table = build_lr1_automaton(&g, &ff).unwrap();

    let a = sym_id(&g, "a");
    assert!(!table.is_extra(a));
}

// ═══════════════════════════════════════════════════════════════════════
// 18. Multi-rule grammar table
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_multi_rule_grammar_has_multiple_rules() {
    // S → a | b
    let mut g = Grammar::new("multi".into());
    let a = SymbolId(1);
    let b = SymbolId(2);
    let s = SymbolId(10);
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
    g.rules.insert(
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
                rhs: vec![Symbol::Terminal(b)],
                precedence: None,
                associativity: None,
                fields: vec![],
                production_id: ProductionId(1),
            },
        ],
    );

    let ff = FirstFollowSets::compute(&g).unwrap();
    let table = build_lr1_automaton(&g, &ff).unwrap();
    sanity_check_tables(&table).expect("sanity check should pass");

    // Both terminals should have shift actions from the initial state
    let acts_a = table.actions(table.initial_state, a);
    let acts_b = table.actions(table.initial_state, b);
    assert!(acts_a.iter().any(|a| matches!(a, Action::Shift(_))));
    assert!(acts_b.iter().any(|a| matches!(a, Action::Shift(_))));
}
