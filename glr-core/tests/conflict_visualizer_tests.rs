//! Comprehensive tests for the conflict visualizer module.
//!
//! Run with: cargo test -p adze-glr-core --test conflict_visualizer_tests --features test-api

use adze_glr_core::conflict_visualizer::{ConflictVisualizer, generate_dot_graph};
use adze_glr_core::{Action, Conflict, ConflictType, ItemSet, ItemSetCollection, LRItem};
use adze_ir::{
    ExternalToken, Grammar, ProductionId, Rule, RuleId, StateId, Symbol, SymbolId, Token,
    TokenPattern,
};
use std::collections::BTreeSet;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Build a minimal grammar: S → a b | a
fn simple_grammar() -> Grammar {
    let mut g = Grammar::new("simple".to_string());

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
                rhs: vec![Symbol::Terminal(a), Symbol::Terminal(b)],
                precedence: None,
                associativity: None,
                production_id: ProductionId(0),
                fields: vec![],
            },
            Rule {
                lhs: s,
                rhs: vec![Symbol::Terminal(a)],
                precedence: None,
                associativity: None,
                production_id: ProductionId(1),
                fields: vec![],
            },
        ],
    );
    g
}

/// Build an expression grammar: E → E + E | num
fn expr_grammar() -> Grammar {
    let mut g = Grammar::new("expr".to_string());

    let num = SymbolId(1);
    let plus = SymbolId(2);
    let e = SymbolId(10);

    g.tokens.insert(
        num,
        Token {
            name: "num".into(),
            pattern: TokenPattern::Regex(r"\d+".into()),
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
                    Symbol::NonTerminal(e),
                ],
                precedence: None,
                associativity: None,
                production_id: ProductionId(0),
                fields: vec![],
            },
            Rule {
                lhs: e,
                rhs: vec![Symbol::Terminal(num)],
                precedence: None,
                associativity: None,
                production_id: ProductionId(1),
                fields: vec![],
            },
        ],
    );
    g
}

fn shift_reduce_conflict() -> Conflict {
    Conflict {
        state: StateId(3),
        symbol: SymbolId(2),
        actions: vec![Action::Shift(StateId(5)), Action::Reduce(RuleId(0))],
        conflict_type: ConflictType::ShiftReduce,
    }
}

fn reduce_reduce_conflict() -> Conflict {
    Conflict {
        state: StateId(4),
        symbol: SymbolId(1),
        actions: vec![Action::Reduce(RuleId(0)), Action::Reduce(RuleId(1))],
        conflict_type: ConflictType::ReduceReduce,
    }
}

// ---------------------------------------------------------------------------
// ConflictVisualizer::generate_report tests
// ---------------------------------------------------------------------------

#[test]
fn test_report_header_with_no_conflicts() {
    let g = simple_grammar();
    let conflicts: Vec<Conflict> = vec![];
    let vis = ConflictVisualizer::new(&g, &conflicts);
    let report = vis.generate_report();

    assert!(report.contains("=== GLR Conflict Report ==="));
    assert!(report.contains("Total conflicts: 0"));
    assert!(report.contains("Shift/Reduce: 0"));
    assert!(report.contains("Reduce/Reduce: 0"));
}

#[test]
fn test_report_single_shift_reduce() {
    let g = expr_grammar();
    let conflicts = vec![shift_reduce_conflict()];
    let vis = ConflictVisualizer::new(&g, &conflicts);
    let report = vis.generate_report();

    assert!(report.contains("Total conflicts: 1"));
    assert!(report.contains("Shift/Reduce: 1"));
    assert!(report.contains("Reduce/Reduce: 0"));
    assert!(report.contains("Conflict #1"));
    assert!(report.contains("State: 3"));
    assert!(report.contains("Shift to state 5"));
    assert!(report.contains("Reduce by rule 0"));
}

#[test]
fn test_report_single_reduce_reduce() {
    let g = simple_grammar();
    let conflicts = vec![reduce_reduce_conflict()];
    let vis = ConflictVisualizer::new(&g, &conflicts);
    let report = vis.generate_report();

    assert!(report.contains("Total conflicts: 1"));
    assert!(report.contains("Shift/Reduce: 0"));
    assert!(report.contains("Reduce/Reduce: 1"));
    assert!(report.contains("Conflict #1"));
    assert!(report.contains("State: 4"));
}

#[test]
fn test_report_mixed_conflicts() {
    let g = expr_grammar();
    let conflicts = vec![shift_reduce_conflict(), reduce_reduce_conflict()];
    let vis = ConflictVisualizer::new(&g, &conflicts);
    let report = vis.generate_report();

    assert!(report.contains("Total conflicts: 2"));
    assert!(report.contains("Shift/Reduce: 1"));
    assert!(report.contains("Reduce/Reduce: 1"));
    assert!(report.contains("Conflict #1"));
    assert!(report.contains("Conflict #2"));
}

#[test]
fn test_report_shows_symbol_name_for_known_token() {
    let g = expr_grammar();
    // Symbol 2 is "+" in expr_grammar
    let conflicts = vec![shift_reduce_conflict()];
    let vis = ConflictVisualizer::new(&g, &conflicts);
    let report = vis.generate_report();

    assert!(report.contains("+"), "Report should resolve token name");
}

#[test]
fn test_report_shows_fallback_for_unknown_symbol() {
    let g = expr_grammar();
    let conflicts = vec![Conflict {
        state: StateId(1),
        symbol: SymbolId(999),
        actions: vec![Action::Shift(StateId(2))],
        conflict_type: ConflictType::ShiftReduce,
    }];
    let vis = ConflictVisualizer::new(&g, &conflicts);
    let report = vis.generate_report();

    assert!(
        report.contains("symbol_999"),
        "Unknown symbols should get fallback name"
    );
}

#[test]
fn test_report_shows_rule_name_for_nonterminal() {
    let g = expr_grammar();
    // SymbolId(10) is rule "E"
    let conflicts = vec![Conflict {
        state: StateId(0),
        symbol: SymbolId(10),
        actions: vec![Action::Reduce(RuleId(0)), Action::Reduce(RuleId(1))],
        conflict_type: ConflictType::ReduceReduce,
    }];
    let vis = ConflictVisualizer::new(&g, &conflicts);
    let report = vis.generate_report();

    assert!(
        report.contains("rule_10"),
        "Non-terminal symbols should use rule_N naming"
    );
}

#[test]
fn test_report_format_reduce_rule_text() {
    let g = expr_grammar();
    let conflicts = vec![Conflict {
        state: StateId(3),
        symbol: SymbolId(2),
        actions: vec![Action::Reduce(RuleId(0))],
        conflict_type: ConflictType::ShiftReduce,
    }];
    let vis = ConflictVisualizer::new(&g, &conflicts);
    let report = vis.generate_report();

    // Rule 0 is "E → E + E", so the formatted rule should contain those symbols
    assert!(
        report.contains("rule_10 ->"),
        "Reduce should show the formatted rule"
    );
}

#[test]
fn test_report_unknown_rule_fallback() {
    let g = simple_grammar();
    let conflicts = vec![Conflict {
        state: StateId(0),
        symbol: SymbolId(1),
        actions: vec![Action::Reduce(RuleId(999))],
        conflict_type: ConflictType::ReduceReduce,
    }];
    let vis = ConflictVisualizer::new(&g, &conflicts);
    let report = vis.generate_report();

    assert!(
        report.contains("Rule 999"),
        "Unknown rule should get fallback format"
    );
}

#[test]
fn test_report_fork_action() {
    let g = expr_grammar();
    let conflicts = vec![Conflict {
        state: StateId(3),
        symbol: SymbolId(2),
        actions: vec![Action::Fork(vec![
            Action::Shift(StateId(5)),
            Action::Reduce(RuleId(0)),
            Action::Reduce(RuleId(1)),
        ])],
        conflict_type: ConflictType::ShiftReduce,
    }];
    let vis = ConflictVisualizer::new(&g, &conflicts);
    let report = vis.generate_report();

    assert!(
        report.contains("Fork into 3 actions"),
        "Fork action should report the count of sub-actions"
    );
}

#[test]
fn test_report_accept_and_error_actions_handled() {
    let g = simple_grammar();
    let conflicts = vec![Conflict {
        state: StateId(0),
        symbol: SymbolId(1),
        actions: vec![Action::Accept, Action::Error],
        conflict_type: ConflictType::ShiftReduce,
    }];
    let vis = ConflictVisualizer::new(&g, &conflicts);
    // Should not panic — Accept/Error fall into the `_ => {}` branch
    let report = vis.generate_report();
    assert!(report.contains("Conflict #1"));
}

// ---------------------------------------------------------------------------
// ConflictVisualizer with item sets
// ---------------------------------------------------------------------------

#[test]
fn test_report_with_item_sets_shows_conflicting_items() {
    let g = expr_grammar();
    let e = SymbolId(10);
    let plus = SymbolId(2);

    // Build a minimal item set for state 3 that matches the conflict
    let mut items = BTreeSet::new();
    // A reduce item: E → E + E •, lookahead = +
    items.insert(LRItem::new(RuleId(0), 3, plus));
    // A shift item: E → E • + E, lookahead = + (next symbol = +)
    items.insert(LRItem::new(RuleId(0), 1, plus));
    // An unrelated item that should NOT appear (different lookahead)
    items.insert(LRItem::new(RuleId(1), 0, e));

    let item_set = ItemSet {
        items,
        id: StateId(3),
    };

    let collection = ItemSetCollection {
        sets: vec![item_set],
        goto_table: Default::default(),
        symbol_is_terminal: Default::default(),
    };

    let conflicts = vec![shift_reduce_conflict()];
    let vis = ConflictVisualizer::new(&g, &conflicts).with_item_sets(&collection);
    let report = vis.generate_report();

    assert!(
        report.contains("Items in state:"),
        "Report should show items section when item sets are provided"
    );
}

#[test]
fn test_report_without_item_sets_no_items_section() {
    let g = expr_grammar();
    let conflicts = vec![shift_reduce_conflict()];
    let vis = ConflictVisualizer::new(&g, &conflicts);
    let report = vis.generate_report();

    assert!(
        !report.contains("Items in state:"),
        "Report should not show items section without item sets"
    );
}

#[test]
fn test_report_item_set_state_mismatch_no_items() {
    let g = expr_grammar();

    // Item set for state 99 — conflict is in state 3, so no match
    let item_set = ItemSet {
        items: BTreeSet::new(),
        id: StateId(99),
    };
    let collection = ItemSetCollection {
        sets: vec![item_set],
        goto_table: Default::default(),
        symbol_is_terminal: Default::default(),
    };

    let conflicts = vec![shift_reduce_conflict()];
    let vis = ConflictVisualizer::new(&g, &conflicts).with_item_sets(&collection);
    let report = vis.generate_report();

    assert!(
        !report.contains("Items in state:"),
        "No items section when item set state doesn't match conflict state"
    );
}

// ---------------------------------------------------------------------------
// Symbol formatting
// ---------------------------------------------------------------------------

#[test]
fn test_external_symbol_name_resolution() {
    let mut g = Grammar::new("ext_test".to_string());
    let ext_id = SymbolId(50);
    g.externals.push(ExternalToken {
        name: "indent".into(),
        symbol_id: ext_id,
    });

    let conflicts = vec![Conflict {
        state: StateId(0),
        symbol: ext_id,
        actions: vec![Action::Shift(StateId(1)), Action::Reduce(RuleId(0))],
        conflict_type: ConflictType::ShiftReduce,
    }];
    let vis = ConflictVisualizer::new(&g, &conflicts);
    let report = vis.generate_report();

    assert!(
        report.contains("indent"),
        "External symbols should resolve to their name"
    );
}

// ---------------------------------------------------------------------------
// generate_dot_graph tests
// ---------------------------------------------------------------------------

#[test]
fn test_dot_graph_basic_structure() {
    let g = simple_grammar();
    let collection = ItemSetCollection {
        sets: vec![
            ItemSet {
                items: BTreeSet::new(),
                id: StateId(0),
            },
            ItemSet {
                items: {
                    let mut s = BTreeSet::new();
                    s.insert(LRItem::new(RuleId(0), 1, SymbolId(1)));
                    s
                },
                id: StateId(1),
            },
        ],
        goto_table: Default::default(),
        symbol_is_terminal: Default::default(),
    };

    let dot = generate_dot_graph(&collection, &[], &g);

    assert!(dot.contains("digraph parse_automaton {"));
    assert!(dot.contains("rankdir=LR"));
    assert!(dot.contains("node [shape=box]"));
    assert!(dot.contains("state0"));
    assert!(dot.contains("state1"));
    assert!(dot.ends_with("}\n"));
}

#[test]
fn test_dot_graph_conflict_state_colored_red() {
    let g = simple_grammar();
    let collection = ItemSetCollection {
        sets: vec![ItemSet {
            items: BTreeSet::new(),
            id: StateId(3),
        }],
        goto_table: Default::default(),
        symbol_is_terminal: Default::default(),
    };

    let conflict = shift_reduce_conflict(); // state 3
    let dot = generate_dot_graph(&collection, &[conflict], &g);

    assert!(
        dot.contains("color=red"),
        "Conflicting state should be colored red"
    );
}

#[test]
fn test_dot_graph_no_conflict_state_colored_black() {
    let g = simple_grammar();
    let collection = ItemSetCollection {
        sets: vec![ItemSet {
            items: BTreeSet::new(),
            id: StateId(0),
        }],
        goto_table: Default::default(),
        symbol_is_terminal: Default::default(),
    };

    let dot = generate_dot_graph(&collection, &[], &g);

    assert!(
        dot.contains("color=black"),
        "Non-conflicting state should be colored black"
    );
}

#[test]
fn test_dot_graph_transitions() {
    let g = simple_grammar();
    let a = SymbolId(1);

    let mut goto_table = indexmap::IndexMap::new();
    goto_table.insert((StateId(0), a), StateId(1));

    let collection = ItemSetCollection {
        sets: vec![
            ItemSet {
                items: BTreeSet::new(),
                id: StateId(0),
            },
            ItemSet {
                items: BTreeSet::new(),
                id: StateId(1),
            },
        ],
        goto_table,
        symbol_is_terminal: Default::default(),
    };

    let dot = generate_dot_graph(&collection, &[], &g);

    assert!(
        dot.contains("state0 -> state1"),
        "Transition edge should appear in DOT output"
    );
    assert!(
        dot.contains("label=\"a\""),
        "Edge label should be the token name"
    );
}

#[test]
fn test_dot_graph_unknown_symbol_transition() {
    let g = simple_grammar();
    let unknown = SymbolId(999);

    let mut goto_table = indexmap::IndexMap::new();
    goto_table.insert((StateId(0), unknown), StateId(1));

    let collection = ItemSetCollection {
        sets: vec![
            ItemSet {
                items: BTreeSet::new(),
                id: StateId(0),
            },
            ItemSet {
                items: BTreeSet::new(),
                id: StateId(1),
            },
        ],
        goto_table,
        symbol_is_terminal: Default::default(),
    };

    let dot = generate_dot_graph(&collection, &[], &g);

    assert!(
        dot.contains("label=\"?\""),
        "Unknown symbol transitions should use '?' label"
    );
}

#[test]
fn test_dot_graph_item_count_label() {
    let g = simple_grammar();
    let mut items = BTreeSet::new();
    items.insert(LRItem::new(RuleId(0), 0, SymbolId(1)));
    items.insert(LRItem::new(RuleId(1), 0, SymbolId(2)));

    let collection = ItemSetCollection {
        sets: vec![ItemSet {
            items,
            id: StateId(0),
        }],
        goto_table: Default::default(),
        symbol_is_terminal: Default::default(),
    };

    let dot = generate_dot_graph(&collection, &[], &g);

    assert!(
        dot.contains("2 items"),
        "State label should include item count"
    );
}
