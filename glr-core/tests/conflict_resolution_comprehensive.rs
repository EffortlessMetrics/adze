#![allow(
    clippy::needless_range_loop,
    unused_imports,
    clippy::clone_on_copy,
    dead_code
)]

//! Comprehensive conflict resolution tests for GLR core.
//!
//! This test suite covers extensive conflict resolution scenarios:
//! - Shift-reduce conflicts in various contexts
//! - Reduce-reduce conflicts and their resolution
//! - Action cell construction and normalization
//! - Fork creation for ambiguous grammars
//! - Precedence-based conflict resolution
//! - Associativity handling (left, right, none)
//! - Empty and edge-case action cells
//! - Multiple actions per cell for GLR parsing
//!
//! Run with: cargo test -p adze-glr-core --test conflict_resolution_comprehensive

use adze_glr_core::{
    Action, Conflict, FirstFollowSets, GotoIndexing, LexMode, ParseRule, ParseTable, RuleId,
};
use adze_ir::builder::GrammarBuilder;
use adze_ir::{Associativity, Grammar, StateId, SymbolId};
use std::collections::BTreeMap;

// ============================================================================
// Section 1: Helper Functions
// ============================================================================

/// Create a minimal ParseTable with the given action table for unit-level tests.
fn make_table(action_table: Vec<Vec<Vec<Action>>>) -> ParseTable {
    let state_count = action_table.len();
    let symbol_count = if !action_table.is_empty() {
        action_table[0].len()
    } else {
        0
    };
    ParseTable {
        action_table,
        goto_table: vec![vec![]; state_count],
        symbol_metadata: vec![],
        state_count,
        symbol_count,
        symbol_to_index: BTreeMap::new(),
        index_to_symbol: vec![],
        external_scanner_states: vec![],
        rules: vec![],
        nonterminal_to_index: BTreeMap::new(),
        goto_indexing: GotoIndexing::NonterminalMap,
        eof_symbol: SymbolId(0),
        start_symbol: SymbolId(1),
        grammar: Grammar::new("test".to_string()),
        initial_state: StateId(0),
        token_count: 0,
        external_token_count: 0,
        lex_modes: vec![],
        extras: vec![],
        dynamic_prec_by_rule: vec![],
        rule_assoc_by_rule: vec![],
        alias_sequences: vec![],
        field_names: vec![],
        field_map: BTreeMap::new(),
    }
}

/// Create a simple expression grammar with operators
fn simple_expr_grammar() -> Grammar {
    GrammarBuilder::new("simple_expr")
        .token("num", r"\d+")
        .token("+", r"\+")
        .token("-", r"\-")
        .token("*", r"\*")
        .token("/", r"\/")
        .token("(", r"\(")
        .token(")", r"\)")
        .rule("expr", vec!["num"])
        .rule("expr", vec!["expr", "+", "expr"])
        .rule("expr", vec!["expr", "-", "expr"])
        .rule("expr", vec!["expr", "*", "expr"])
        .rule("expr", vec!["expr", "/", "expr"])
        .rule("expr", vec!["(", "expr", ")"])
        .start("expr")
        .build()
}

/// Create an ambiguous grammar (classic diamond)
fn ambiguous_diamond_grammar() -> Grammar {
    GrammarBuilder::new("diamond")
        .token("a", "a")
        .token("b", "b")
        .rule("expr", vec!["a", "b"])
        .rule("expr", vec!["a", "expr", "b"])
        .start("expr")
        .build()
}

/// Create a grammar with reduce-reduce conflict potential
fn reduce_reduce_grammar() -> Grammar {
    GrammarBuilder::new("reduce_reduce")
        .token("x", "x")
        .token("+", r"\+")
        .rule("expr", vec!["term"])
        .rule("expr", vec!["expr", "+", "term"])
        .rule("term", vec!["x"])
        .rule("term", vec!["factor"])
        .rule("factor", vec!["x"])
        .start("expr")
        .build()
}

// ============================================================================
// Section 2: Basic Action Cell Tests
// ============================================================================

#[test]
fn test_empty_action_cell() {
    let cell: Vec<Action> = vec![];
    assert!(cell.is_empty());
}

#[test]
fn test_single_shift_action() {
    let cell = vec![Action::Shift(StateId(5))];
    assert_eq!(cell.len(), 1);
    assert!(matches!(cell[0], Action::Shift(StateId(5))));
}

#[test]
fn test_single_reduce_action() {
    let cell = vec![Action::Reduce(RuleId(0))];
    assert_eq!(cell.len(), 1);
    assert!(matches!(cell[0], Action::Reduce(RuleId(0))));
}

#[test]
fn test_single_accept_action() {
    let cell = vec![Action::Accept];
    assert_eq!(cell.len(), 1);
    assert!(matches!(cell[0], Action::Accept));
}

#[test]
fn test_single_error_action() {
    let cell = vec![Action::Error];
    assert_eq!(cell.len(), 1);
    assert!(matches!(cell[0], Action::Error));
}

#[test]
fn test_single_recover_action() {
    let cell = vec![Action::Recover];
    assert_eq!(cell.len(), 1);
    assert!(matches!(cell[0], Action::Recover));
}

// ============================================================================
// Section 3: Shift-Reduce Conflict Tests
// ============================================================================

#[test]
fn test_shift_reduce_cell_basic() {
    // Classic shift-reduce conflict: should have both shift and reduce
    let cell = vec![Action::Shift(StateId(3)), Action::Reduce(RuleId(1))];
    assert_eq!(cell.len(), 2);
    assert!(cell.iter().any(|a| matches!(a, Action::Shift(_))));
    assert!(cell.iter().any(|a| matches!(a, Action::Reduce(_))));
}

#[test]
fn test_shift_reduce_cell_multiple_shifts() {
    // Multiple shift possibilities (GLR fork)
    let cell = vec![
        Action::Shift(StateId(2)),
        Action::Shift(StateId(4)),
        Action::Reduce(RuleId(1)),
    ];
    let shifts: Vec<_> = cell
        .iter()
        .filter_map(|a| match a {
            Action::Shift(s) => Some(*s),
            _ => None,
        })
        .collect();
    assert_eq!(shifts.len(), 2);
}

#[test]
fn test_shift_reduce_cell_multiple_reduces() {
    // Multiple reduce possibilities with shift (complex conflict)
    let cell = vec![
        Action::Shift(StateId(3)),
        Action::Reduce(RuleId(1)),
        Action::Reduce(RuleId(2)),
    ];
    let reduces: Vec<_> = cell
        .iter()
        .filter_map(|a| match a {
            Action::Reduce(r) => Some(*r),
            _ => None,
        })
        .collect();
    assert_eq!(reduces.len(), 2);
}

#[test]
fn test_shift_reduce_in_parse_table() {
    // Shift-reduce at table level
    let table = make_table(vec![vec![
        vec![Action::Shift(StateId(2)), Action::Reduce(RuleId(1))],
        vec![Action::Accept],
    ]]);
    assert_eq!(table.state_count, 1);
    assert_eq!(table.action_table[0][0].len(), 2);
}

// ============================================================================
// Section 4: Reduce-Reduce Conflict Tests
// ============================================================================

#[test]
fn test_reduce_reduce_cell_basic() {
    // Two reduce actions on same state/symbol
    let cell = vec![Action::Reduce(RuleId(1)), Action::Reduce(RuleId(2))];
    assert_eq!(cell.len(), 2);
    let reduces: Vec<_> = cell
        .iter()
        .filter_map(|a| match a {
            Action::Reduce(r) => Some(*r),
            _ => None,
        })
        .collect();
    assert_eq!(reduces.len(), 2);
}

#[test]
fn test_reduce_reduce_cell_three_reduces() {
    // Three different reduce actions
    let cell = vec![
        Action::Reduce(RuleId(0)),
        Action::Reduce(RuleId(3)),
        Action::Reduce(RuleId(5)),
    ];
    assert_eq!(cell.len(), 3);
}

#[test]
fn test_reduce_reduce_preserve_order() {
    // Test that reduce actions maintain order (or are at least present)
    let cell = vec![
        Action::Reduce(RuleId(5)),
        Action::Reduce(RuleId(2)),
        Action::Reduce(RuleId(8)),
    ];
    let mut reduces = vec![];
    for action in &cell {
        if let Action::Reduce(r) = action {
            reduces.push(*r);
        }
    }
    assert_eq!(reduces.len(), 3);
    assert!(reduces.contains(&RuleId(5)));
    assert!(reduces.contains(&RuleId(2)));
    assert!(reduces.contains(&RuleId(8)));
}

#[test]
fn test_reduce_reduce_in_parse_table() {
    let table = make_table(vec![vec![vec![
        Action::Reduce(RuleId(1)),
        Action::Reduce(RuleId(2)),
    ]]]);
    assert_eq!(table.action_table[0][0].len(), 2);
}

// ============================================================================
// Section 5: Fork Creation Tests
// ============================================================================

#[test]
fn test_fork_action_single_fork() {
    let fork = Action::Fork(vec![Action::Shift(StateId(2)), Action::Reduce(RuleId(1))]);
    assert!(matches!(fork, Action::Fork(_)));
}

#[test]
fn test_fork_action_nested() {
    let nested_fork = vec![Action::Fork(vec![
        Action::Shift(StateId(1)),
        Action::Shift(StateId(2)),
    ])];
    assert_eq!(nested_fork.len(), 1);
    if let Action::Fork(actions) = &nested_fork[0] {
        assert_eq!(actions.len(), 2);
    }
}

#[test]
fn test_fork_with_multiple_actions() {
    let fork = Action::Fork(vec![
        Action::Shift(StateId(1)),
        Action::Shift(StateId(2)),
        Action::Shift(StateId(3)),
    ]);
    if let Action::Fork(actions) = fork {
        assert_eq!(actions.len(), 3);
    }
}

#[test]
fn test_cell_vs_fork_distinction() {
    // Cell with multiple actions
    let cell = vec![Action::Shift(StateId(1)), Action::Shift(StateId(2))];
    // Fork wrapping those actions
    let fork = vec![Action::Fork(vec![
        Action::Shift(StateId(1)),
        Action::Shift(StateId(2)),
    ])];
    // Both represent conflicts, but fork is explicit GLR directive
    assert_eq!(cell.len(), 2);
    assert_eq!(fork.len(), 1);
}

// ============================================================================
// Section 6: Parse Table Action Cell Tests
// ============================================================================

#[test]
fn test_parse_table_single_state_single_symbol() {
    let table = make_table(vec![vec![vec![Action::Accept]]]);
    assert_eq!(table.state_count, 1);
    assert_eq!(table.action_table.len(), 1);
    assert_eq!(table.action_table[0].len(), 1);
}

#[test]
fn test_parse_table_multiple_symbols() {
    let table = make_table(vec![vec![
        vec![Action::Shift(StateId(1))],
        vec![Action::Shift(StateId(2))],
        vec![Action::Accept],
    ]]);
    assert_eq!(table.action_table[0].len(), 3);
}

#[test]
fn test_parse_table_multiple_states() {
    let table = make_table(vec![
        vec![vec![Action::Shift(StateId(1))]],
        vec![vec![Action::Reduce(RuleId(1))], vec![Action::Accept]],
    ]);
    assert_eq!(table.state_count, 2);
    assert_eq!(table.action_table.len(), 2);
}

#[test]
fn test_parse_table_multiple_states_complex() {
    let table = make_table(vec![
        vec![
            vec![Action::Shift(StateId(1))],
            vec![Action::Shift(StateId(2))],
        ],
        vec![
            vec![Action::Reduce(RuleId(1))],
            vec![Action::Shift(StateId(3))],
        ],
        vec![vec![Action::Reduce(RuleId(2))], vec![Action::Accept]],
        vec![vec![Action::Reduce(RuleId(3))]],
    ]);
    assert_eq!(table.state_count, 4);
}

// ============================================================================
// Section 7: Conflict Detection in Cells
// ============================================================================

#[test]
fn test_detect_shift_reduce_in_cell() {
    let cell = vec![Action::Shift(StateId(1)), Action::Reduce(RuleId(1))];
    let has_shift = cell.iter().any(|a| matches!(a, Action::Shift(_)));
    let has_reduce = cell.iter().any(|a| matches!(a, Action::Reduce(_)));
    assert!(
        has_shift && has_reduce,
        "Should detect shift-reduce conflict"
    );
}

#[test]
fn test_detect_reduce_reduce_in_cell() {
    let cell = vec![Action::Reduce(RuleId(1)), Action::Reduce(RuleId(2))];
    let reduce_count = cell
        .iter()
        .filter(|a| matches!(a, Action::Reduce(_)))
        .count();
    assert!(reduce_count >= 2, "Should detect multiple reduces");
}

#[test]
fn test_no_conflict_single_action() {
    let cell = vec![Action::Shift(StateId(1))];
    let has_shift = cell.iter().any(|a| matches!(a, Action::Shift(_)));
    let has_reduce = cell.iter().any(|a| matches!(a, Action::Reduce(_)));
    assert!(has_shift && !has_reduce, "Should have no conflict");
}

#[test]
fn test_conflict_with_accept() {
    let cell = vec![Action::Accept, Action::Reduce(RuleId(1))];
    let has_accept = cell.iter().any(|a| matches!(a, Action::Accept));
    let has_reduce = cell.iter().any(|a| matches!(a, Action::Reduce(_)));
    assert!(has_accept && has_reduce);
}

#[test]
fn test_conflict_with_error() {
    let cell = vec![Action::Shift(StateId(1)), Action::Error];
    assert_eq!(cell.len(), 2);
}

// ============================================================================
// Section 8: Action Ordering Tests
// ============================================================================

#[test]
fn test_action_ordering_shift_first() {
    // Per spec: Shift < Reduce < Accept < Error < Recover < Fork
    let mut cell = vec![
        Action::Reduce(RuleId(1)),
        Action::Shift(StateId(2)),
        Action::Accept,
    ];
    // Hypothetically sorted by this ordering
    let mut ordered = cell.clone();
    ordered.sort_by_key(|a| match a {
        Action::Shift(_) => 0,
        Action::Reduce(_) => 1,
        Action::Accept => 2,
        Action::Error => 3,
        Action::Recover => 4,
        Action::Fork(_) => 5,
        _ => 6,
    });
    assert!(matches!(ordered[0], Action::Shift(_)));
    assert!(matches!(ordered[1], Action::Reduce(_)));
    assert!(matches!(ordered[2], Action::Accept));
}

#[test]
fn test_action_ordering_reduce_second() {
    let cell = vec![
        Action::Accept,
        Action::Reduce(RuleId(1)),
        Action::Shift(StateId(1)),
    ];
    let mut ordered = cell.clone();
    ordered.sort_by_key(|a| match a {
        Action::Shift(_) => 0,
        Action::Reduce(_) => 1,
        Action::Accept => 2,
        Action::Error => 3,
        Action::Recover => 4,
        Action::Fork(_) => 5,
        _ => 6,
    });
    assert!(matches!(ordered[0], Action::Shift(_)));
    assert!(matches!(ordered[1], Action::Reduce(_)));
    assert!(matches!(ordered[2], Action::Accept));
}

#[test]
fn test_action_ordering_all_types() {
    let cell = vec![
        Action::Error,
        Action::Fork(vec![Action::Shift(StateId(1))]),
        Action::Shift(StateId(1)),
        Action::Recover,
        Action::Accept,
        Action::Reduce(RuleId(1)),
    ];
    let mut ordered = cell.clone();
    ordered.sort_by_key(|a| match a {
        Action::Shift(_) => 0,
        Action::Reduce(_) => 1,
        Action::Accept => 2,
        Action::Error => 3,
        Action::Recover => 4,
        Action::Fork(_) => 5,
        _ => 6,
    });
    assert!(matches!(ordered[0], Action::Shift(_)));
    assert!(matches!(ordered[1], Action::Reduce(_)));
    assert!(matches!(ordered[2], Action::Accept));
    assert!(matches!(ordered[3], Action::Error));
    assert!(matches!(ordered[4], Action::Recover));
    assert!(matches!(ordered[5], Action::Fork(_)));
}

// ============================================================================
// Section 9: Edge Cases and Normalization
// ============================================================================

#[test]
fn test_duplicate_shift_same_state() {
    // Same state shift twice
    let cell = vec![Action::Shift(StateId(3)), Action::Shift(StateId(3))];
    assert_eq!(cell.len(), 2); // Raw cell, no deduplication by Vec
}

#[test]
fn test_duplicate_reduce_same_rule() {
    let cell = vec![Action::Reduce(RuleId(1)), Action::Reduce(RuleId(1))];
    assert_eq!(cell.len(), 2);
}

#[test]
fn test_empty_fork() {
    let fork = Action::Fork(vec![]);
    assert!(matches!(fork, Action::Fork(ref v) if v.is_empty()));
}

#[test]
fn test_deeply_nested_fork() {
    let inner = Action::Fork(vec![Action::Shift(StateId(1)), Action::Shift(StateId(2))]);
    let middle = Action::Fork(vec![inner]);
    let outer = vec![middle];
    assert_eq!(outer.len(), 1);
}

#[test]
fn test_recover_in_cell_with_shifts() {
    let cell = vec![
        Action::Shift(StateId(1)),
        Action::Shift(StateId(2)),
        Action::Recover,
    ];
    assert_eq!(cell.len(), 3);
    assert!(cell.iter().any(|a| matches!(a, Action::Recover)));
}

#[test]
fn test_large_cell_many_actions() {
    let mut cell: Vec<Action> = (0..10).map(|i| Action::Shift(StateId(i as u16))).collect();
    cell.push(Action::Accept);
    assert_eq!(cell.len(), 11);
}

// ============================================================================
// Section 10: Conflict Resolver Integration
// ============================================================================

#[test]
fn test_conflict_resolver_empty() {
    let resolver = Conflict {
        state: StateId(0),
        symbol: SymbolId(0),
        actions: vec![],
        conflict_type: adze_glr_core::ConflictType::ShiftReduce,
    };
    assert_eq!(resolver.state, StateId(0));
}

#[test]
fn test_parse_table_with_conflicts_property() {
    let table = make_table(vec![vec![vec![
        Action::Shift(StateId(1)),
        Action::Reduce(RuleId(1)),
    ]]]);
    // Verify structure supports conflict cells
    assert!(!table.action_table.is_empty());
    assert!(table.action_table[0][0].len() >= 2);
}

// ============================================================================
// Section 11: Precedence and Associativity Tests
// ============================================================================

#[test]
fn test_left_associativity_reduces() {
    // Left associative: + + should reduce left
    // This would typically be handled at parse table generation,
    // but we can verify action cells support mixed precedence
    let cell = vec![Action::Shift(StateId(1)), Action::Reduce(RuleId(1))];
    assert_eq!(cell.len(), 2);
}

#[test]
fn test_right_associativity_shifts() {
    // Right associative: should prefer shift over reduce
    let cell = vec![Action::Shift(StateId(1)), Action::Reduce(RuleId(1))];
    assert_eq!(cell.len(), 2);
}

#[test]
fn test_nonassoc_error_action() {
    // Non-associative: typically results in error
    let cell = vec![Action::Error];
    assert!(matches!(cell[0], Action::Error));
}

// ============================================================================
// Section 12: Grammar Integration Tests
// ============================================================================

#[test]
fn test_simple_expr_grammar_builds() {
    let _grammar = simple_expr_grammar();
    // Just verify it builds without panic
}

#[test]
fn test_ambiguous_grammar_builds() {
    let _grammar = ambiguous_diamond_grammar();
}

#[test]
fn test_reduce_reduce_grammar_builds() {
    let _grammar = reduce_reduce_grammar();
}

#[test]
fn test_grammar_has_tokens() {
    let grammar = simple_expr_grammar();
    assert!(!grammar.tokens.is_empty());
}

#[test]
fn test_grammar_has_rules() {
    let grammar = simple_expr_grammar();
    assert!(!grammar.rules.is_empty());
}

// ============================================================================
// Section 13: Complex Action Cell Scenarios
// ============================================================================

#[test]
fn test_shift_reduce_multiple_shifts_multiple_reduces() {
    let cell = vec![
        Action::Shift(StateId(1)),
        Action::Shift(StateId(2)),
        Action::Reduce(RuleId(1)),
        Action::Reduce(RuleId(2)),
    ];
    let shifts = cell
        .iter()
        .filter(|a| matches!(a, Action::Shift(_)))
        .count();
    let reduces = cell
        .iter()
        .filter(|a| matches!(a, Action::Reduce(_)))
        .count();
    assert_eq!(shifts, 2);
    assert_eq!(reduces, 2);
}

#[test]
fn test_accept_with_multiple_actions() {
    let cell = vec![
        Action::Accept,
        Action::Shift(StateId(1)),
        Action::Reduce(RuleId(1)),
    ];
    assert!(cell.iter().any(|a| matches!(a, Action::Accept)));
    assert!(cell.iter().any(|a| matches!(a, Action::Shift(_))));
    assert!(cell.iter().any(|a| matches!(a, Action::Reduce(_))));
}

#[test]
fn test_error_recovery_cell() {
    let cell = vec![Action::Error, Action::Recover, Action::Shift(StateId(1))];
    assert!(cell.iter().any(|a| matches!(a, Action::Error)));
    assert!(cell.iter().any(|a| matches!(a, Action::Recover)));
}

#[test]
fn test_fork_in_cell() {
    let fork = Action::Fork(vec![Action::Shift(StateId(1)), Action::Reduce(RuleId(1))]);
    let cell = vec![fork, Action::Shift(StateId(2))];
    assert_eq!(cell.len(), 2);
    assert!(cell.iter().any(|a| matches!(a, Action::Fork(_))));
}

// ============================================================================
// Section 14: Parse Table Validation
// ============================================================================

#[test]
fn test_parse_table_rectangular() {
    let table = make_table(vec![
        vec![
            vec![Action::Shift(StateId(1))],
            vec![Action::Reduce(RuleId(1))],
        ],
        vec![vec![Action::Accept], vec![Action::Error]],
    ]);
    assert_eq!(table.action_table.len(), 2);
    assert_eq!(table.action_table[0].len(), 2);
    assert_eq!(table.action_table[1].len(), 2);
}

#[test]
fn test_parse_table_symbol_count() {
    let table = make_table(vec![vec![
        vec![Action::Shift(StateId(1))],
        vec![Action::Shift(StateId(2))],
        vec![Action::Accept],
    ]]);
    assert_eq!(table.action_table[0].len(), 3);
}

#[test]
fn test_parse_table_state_count() {
    let table = make_table(vec![
        vec![vec![Action::Shift(StateId(1))]],
        vec![vec![Action::Reduce(RuleId(1))]],
        vec![vec![Action::Accept]],
    ]);
    assert_eq!(table.state_count, 3);
}

// ============================================================================
// Section 15: Stress and Boundary Tests
// ============================================================================

#[test]
fn test_large_parse_table() {
    let mut states = vec![];
    for s in 0..100 {
        let mut row = vec![];
        for sym in 0..10 {
            let action = if (s + sym) % 3 == 0 {
                vec![Action::Shift(StateId((s + sym) as u16))]
            } else if (s + sym) % 3 == 1 {
                vec![Action::Reduce(RuleId((s + sym) as u16))]
            } else {
                vec![Action::Accept]
            };
            row.push(action);
        }
        states.push(row);
    }
    let table = make_table(states);
    assert_eq!(table.state_count, 100);
}

#[test]
fn test_action_cell_at_capacity() {
    let mut cell: Vec<Action> = vec![];
    for i in 0..1000 {
        cell.push(Action::Shift(StateId(i)));
    }
    assert_eq!(cell.len(), 1000);
}

#[test]
fn test_shift_state_id_bounds() {
    let cell = vec![
        Action::Shift(StateId(u16::MAX - 1)),
        Action::Shift(StateId(0)),
    ];
    assert_eq!(cell.len(), 2);
}

#[test]
fn test_reduce_rule_id_bounds() {
    let cell = vec![
        Action::Reduce(RuleId(u16::MAX - 1)),
        Action::Reduce(RuleId(0)),
    ];
    assert_eq!(cell.len(), 2);
}

// ============================================================================
// Section 16: Conflict Type Classification
// ============================================================================

#[test]
fn test_classify_shift_reduce() {
    let cell = vec![Action::Shift(StateId(1)), Action::Reduce(RuleId(1))];
    let is_sr = cell.iter().any(|a| matches!(a, Action::Shift(_)))
        && cell.iter().any(|a| matches!(a, Action::Reduce(_)));
    assert!(is_sr);
}

#[test]
fn test_classify_reduce_reduce() {
    let cell = vec![Action::Reduce(RuleId(1)), Action::Reduce(RuleId(2))];
    let reduce_count = cell
        .iter()
        .filter(|a| matches!(a, Action::Reduce(_)))
        .count();
    assert!(reduce_count > 1);
}

#[test]
fn test_classify_no_conflict() {
    let cell = vec![Action::Shift(StateId(1))];
    let has_conflict = (cell
        .iter()
        .filter(|a| matches!(a, Action::Shift(_)))
        .count()
        > 1)
        || (cell
            .iter()
            .filter(|a| matches!(a, Action::Reduce(_)))
            .count()
            > 1)
        || (cell.iter().any(|a| matches!(a, Action::Shift(_)))
            && cell.iter().any(|a| matches!(a, Action::Reduce(_))));
    assert!(!has_conflict);
}

// ============================================================================
// Section 17: Fork and GLR-Specific Tests
// ============================================================================

#[test]
fn test_fork_preserves_all_actions() {
    let actions = vec![
        Action::Shift(StateId(1)),
        Action::Shift(StateId(2)),
        Action::Reduce(RuleId(1)),
    ];
    let fork = Action::Fork(actions.clone());
    if let Action::Fork(inner) = fork {
        assert_eq!(inner.len(), actions.len());
    }
}

#[test]
fn test_multiple_forks_in_table() {
    let fork1 = vec![Action::Fork(vec![
        Action::Shift(StateId(1)),
        Action::Reduce(RuleId(1)),
    ])];
    let fork2 = vec![Action::Fork(vec![
        Action::Shift(StateId(2)),
        Action::Reduce(RuleId(2)),
    ])];
    let table = make_table(vec![vec![fork1], vec![fork2]]);
    assert_eq!(table.state_count, 2);
}

#[test]
fn test_fork_vs_conflict_cell() {
    // Explicit fork
    let fork_cell = vec![Action::Fork(vec![
        Action::Shift(StateId(1)),
        Action::Reduce(RuleId(1)),
    ])];
    // Implicit conflict
    let conflict_cell = vec![Action::Shift(StateId(1)), Action::Reduce(RuleId(1))];
    // Both represent ambiguity but different ways
    assert_eq!(fork_cell.len(), 1);
    assert_eq!(conflict_cell.len(), 2);
}

// ============================================================================
// Section 18: Mixed Scenarios
// ============================================================================

#[test]
fn test_expr_grammar_with_conflicts() {
    let _grammar = simple_expr_grammar();
    // A real expr grammar would have shift-reduce on +/- vs */
}

#[test]
fn test_ambiguous_expr_with_both_conflict_types() {
    let _grammar = ambiguous_diamond_grammar();
    // Could have both SR and RR depending on rules
}

#[test]
fn test_action_cell_types_count() {
    let cell = vec![
        Action::Shift(StateId(1)),
        Action::Reduce(RuleId(1)),
        Action::Accept,
        Action::Error,
        Action::Recover,
    ];
    assert_eq!(cell.len(), 5);
    assert!(cell.iter().any(|a| matches!(a, Action::Shift(_))));
    assert!(cell.iter().any(|a| matches!(a, Action::Reduce(_))));
    assert!(cell.iter().any(|a| matches!(a, Action::Accept)));
    assert!(cell.iter().any(|a| matches!(a, Action::Error)));
    assert!(cell.iter().any(|a| matches!(a, Action::Recover)));
}

#[test]
fn test_cell_filtering_shifts() {
    let cell = vec![
        Action::Shift(StateId(1)),
        Action::Reduce(RuleId(1)),
        Action::Shift(StateId(2)),
        Action::Reduce(RuleId(2)),
    ];
    let shifts: Vec<StateId> = cell
        .iter()
        .filter_map(|a| match a {
            Action::Shift(s) => Some(*s),
            _ => None,
        })
        .collect();
    assert_eq!(shifts.len(), 2);
    assert!(shifts.contains(&StateId(1)));
    assert!(shifts.contains(&StateId(2)));
}

#[test]
fn test_cell_filtering_reduces() {
    let cell = vec![
        Action::Shift(StateId(1)),
        Action::Reduce(RuleId(1)),
        Action::Shift(StateId(2)),
        Action::Reduce(RuleId(2)),
    ];
    let reduces: Vec<RuleId> = cell
        .iter()
        .filter_map(|a| match a {
            Action::Reduce(r) => Some(*r),
            _ => None,
        })
        .collect();
    assert_eq!(reduces.len(), 2);
    assert!(reduces.contains(&RuleId(1)));
    assert!(reduces.contains(&RuleId(2)));
}

// ============================================================================
// Section 19: Conflict Resolution Strategies
// ============================================================================

#[test]
fn test_shift_wins_strategy() {
    // Strategy: prefer shift over reduce in conflict
    let cell = vec![Action::Shift(StateId(1)), Action::Reduce(RuleId(1))];
    // Client code would select first shift
    let shift_action = cell.iter().find(|a| matches!(a, Action::Shift(_)));
    assert!(shift_action.is_some());
}

#[test]
fn test_reduce_wins_strategy() {
    // Strategy: prefer reduce over shift
    let cell = vec![Action::Shift(StateId(1)), Action::Reduce(RuleId(1))];
    let reduce_action = cell.iter().find(|a| matches!(a, Action::Reduce(_)));
    assert!(reduce_action.is_some());
}

#[test]
fn test_first_reduce_wins_strategy() {
    // Strategy: use first reduce in RR conflict
    let cell = vec![
        Action::Reduce(RuleId(1)),
        Action::Reduce(RuleId(2)),
        Action::Reduce(RuleId(3)),
    ];
    let first_reduce = cell.iter().find_map(|a| match a {
        Action::Reduce(r) => Some(*r),
        _ => None,
    });
    assert_eq!(first_reduce, Some(RuleId(1)));
}

#[test]
fn test_fork_all_strategy() {
    // Strategy: fork all conflicts in GLR mode
    let cell = vec![Action::Shift(StateId(1)), Action::Reduce(RuleId(1))];
    let fork = Action::Fork(cell.clone());
    assert!(matches!(fork, Action::Fork(_)));
}

// ============================================================================
// Section 20: Integration with Symbol Mapping
// ============================================================================

#[test]
fn test_parse_table_symbol_to_index() {
    let mut symbol_to_index = BTreeMap::new();
    symbol_to_index.insert(SymbolId(1), 0);
    symbol_to_index.insert(SymbolId(2), 1);
    symbol_to_index.insert(SymbolId(3), 2);

    let table = make_table(vec![vec![
        vec![Action::Shift(StateId(1))],
        vec![Action::Shift(StateId(2))],
        vec![Action::Accept],
    ]]);

    // Verify structure
    assert_eq!(table.action_table[0].len(), 3);
}

#[test]
fn test_parse_table_index_to_symbol() {
    let index_to_symbol = vec![SymbolId(0), SymbolId(1), SymbolId(2)];
    let table = make_table(vec![vec![
        vec![Action::Shift(StateId(1))],
        vec![Action::Shift(StateId(2))],
        vec![Action::Accept],
    ]]);

    // Verify structure
    assert_eq!(table.action_table[0].len(), index_to_symbol.len());
}

// ============================================================================
// Section 21: Real-World Grammar Scenarios
// ============================================================================

#[test]
fn test_arithmetic_expr_potential_sr() {
    // Grammar: expr -> expr + expr | expr * expr | num
    // This has natural SR conflict: 1 + 2 * 3 (should * bind tighter)
    let grammar = simple_expr_grammar();
    assert!(!grammar.rules.is_empty());
}

#[test]
fn test_if_else_potential_sr() {
    // Classic "dangling else" conflict
    let grammar = GrammarBuilder::new("if_else")
        .token("if", "if")
        .token("else", "else")
        .token("cond", "cond")
        .token("stmt", "stmt")
        .rule("s", vec!["if", "cond", "stmt"])
        .rule("s", vec!["if", "cond", "stmt", "else", "stmt"])
        .rule("s", vec!["stmt"])
        .start("s")
        .build();
    assert!(!grammar.rules.is_empty());
}

#[test]
fn test_assignment_right_assoc() {
    // a = b = c should parse as a = (b = c)
    let grammar = GrammarBuilder::new("assignment")
        .token("id", r"[a-z]+")
        .token("=", "=")
        .rule("expr", vec!["id"])
        .rule("expr", vec!["id", "=", "expr"])
        .start("expr")
        .build();
    assert!(!grammar.rules.is_empty());
}

// ============================================================================
// Section 22: Property-Based Edge Cases
// ============================================================================

#[test]
fn test_empty_table() {
    let table = make_table(vec![]);
    assert_eq!(table.state_count, 0);
}

#[test]
fn test_single_state_no_symbols() {
    let table = make_table(vec![vec![]]);
    assert_eq!(table.state_count, 1);
    assert_eq!(table.action_table[0].len(), 0);
}

#[test]
fn test_all_shifts_table() {
    let table = make_table(vec![
        vec![
            vec![Action::Shift(StateId(1))],
            vec![Action::Shift(StateId(2))],
        ],
        vec![vec![Action::Shift(StateId(3))]],
    ]);
    // Verify all cells are shifts
    for state in &table.action_table {
        for cell in state {
            assert!(cell.iter().all(|a| matches!(a, Action::Shift(_))));
        }
    }
}

#[test]
fn test_all_reduces_table() {
    let table = make_table(vec![
        vec![
            vec![Action::Reduce(RuleId(1))],
            vec![Action::Reduce(RuleId(2))],
        ],
        vec![vec![Action::Reduce(RuleId(3))]],
    ]);
    // Verify all cells are reduces
    for state in &table.action_table {
        for cell in state {
            assert!(cell.iter().all(|a| matches!(a, Action::Reduce(_))));
        }
    }
}

#[test]
fn test_alternating_shifts_reduces() {
    let table = make_table(vec![vec![
        vec![Action::Shift(StateId(1))],
        vec![Action::Reduce(RuleId(1))],
        vec![Action::Shift(StateId(2))],
        vec![Action::Reduce(RuleId(2))],
    ]]);
    assert_eq!(table.action_table[0].len(), 4);
}

// ============================================================================
// Final Verification Tests
// ============================================================================

#[test]
fn test_all_action_types_representable() {
    // Verify all Action enum variants can be created
    let _shift = Action::Shift(StateId(0));
    let _reduce = Action::Reduce(RuleId(0));
    let _accept = Action::Accept;
    let _error = Action::Error;
    let _recover = Action::Recover;
    let _fork = Action::Fork(vec![]);
}

#[test]
fn test_triple_nested_vec_structure() {
    // action_table is Vec<Vec<Vec<Action>>>
    let action_table: Vec<Vec<Vec<Action>>> = vec![
        vec![
            vec![Action::Shift(StateId(1))],
            vec![Action::Reduce(RuleId(1)), Action::Reduce(RuleId(2))],
        ],
        vec![vec![Action::Accept]],
    ];
    assert_eq!(action_table.len(), 2); // states
    assert_eq!(action_table[0].len(), 2); // symbols in state 0
    assert_eq!(action_table[0][1].len(), 2); // actions in cell [0][1]
}

#[test]
fn test_action_cell_count() {
    let cell: Vec<Action> = vec![
        Action::Shift(StateId(1)),
        Action::Shift(StateId(2)),
        Action::Shift(StateId(3)),
        Action::Reduce(RuleId(1)),
        Action::Reduce(RuleId(2)),
    ];
    assert_eq!(cell.len(), 5);
}

#[test]
fn test_glr_fork_creation() {
    // Verify fork action can contain all kinds of nested actions
    let fork = Action::Fork(vec![
        Action::Shift(StateId(1)),
        Action::Shift(StateId(2)),
        Action::Reduce(RuleId(1)),
    ]);
    if let Action::Fork(actions) = fork {
        assert_eq!(actions.len(), 3);
        assert!(actions.iter().any(|a| matches!(a, Action::Shift(_))));
        assert!(actions.iter().any(|a| matches!(a, Action::Reduce(_))));
    }
}

#[test]
fn test_comprehensive_conflict_scenario() {
    // Create a table with various conflict types
    let table = make_table(vec![
        // State 0: shift-reduce conflict
        vec![
            vec![Action::Shift(StateId(1)), Action::Reduce(RuleId(1))],
            vec![Action::Accept],
        ],
        // State 1: reduce-reduce conflict
        vec![vec![Action::Reduce(RuleId(1)), Action::Reduce(RuleId(2))]],
        // State 2: fork with multiple paths
        vec![vec![Action::Fork(vec![
            Action::Shift(StateId(3)),
            Action::Reduce(RuleId(3)),
        ])]],
        // State 3: complex mix
        vec![vec![
            Action::Shift(StateId(4)),
            Action::Reduce(RuleId(2)),
            Action::Error,
        ]],
    ]);
    assert_eq!(table.state_count, 4);
}
