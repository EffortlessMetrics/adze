#![allow(clippy::needless_range_loop)]
#![cfg(feature = "test-api")]

//! Comprehensive tests for LR(1) automaton construction (`build_lr1_automaton`).
//!
//! Covers: simple grammars, multi-rule automata, state count validation,
//! action table correctness, goto table correctness, accept action placement,
//! first/follow set correctness, nullable symbol detection, item set closure,
//! goto transitions, and edge cases.

use adze_glr_core::*;
use adze_ir::Associativity;
use adze_ir::builder::GrammarBuilder;
use std::collections::BTreeSet;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Build parse table from grammar via the standard pipeline.
fn build_table(grammar: &mut adze_ir::Grammar) -> ParseTable {
    let ff = FirstFollowSets::compute_normalized(grammar)
        .expect("FIRST/FOLLOW computation should succeed");
    build_lr1_automaton(grammar, &ff).expect("automaton construction should succeed")
}

/// Count shift actions in action table
fn count_shifts(table: &ParseTable) -> usize {
    table
        .action_table
        .iter()
        .flat_map(|row| row.iter())
        .flat_map(|cell| cell.iter())
        .filter(|action| matches!(action, Action::Shift(_)))
        .count()
}

/// Count reduce actions in action table
fn count_reduces(table: &ParseTable) -> usize {
    table
        .action_table
        .iter()
        .flat_map(|row| row.iter())
        .flat_map(|cell| cell.iter())
        .filter(|action| matches!(action, Action::Reduce(_)))
        .count()
}

/// Count accept actions in action table
fn count_accepts(table: &ParseTable) -> usize {
    table
        .action_table
        .iter()
        .flat_map(|row| row.iter())
        .flat_map(|cell| cell.iter())
        .filter(|action| matches!(action, Action::Accept))
        .count()
}

/// Get actions for a state and symbol using test helpers
fn actions_for(table: &ParseTable, state: usize, sym: SymbolId) -> Vec<Action> {
    let idx = table.symbol_to_index.get(&sym).copied().unwrap_or_else(|| {
        panic!("Symbol {:?} not found in symbol_to_index", sym);
    });
    if state < table.action_table.len() && idx < table.action_table[state].len() {
        table.action_table[state][idx].clone()
    } else {
        vec![]
    }
}

/// Check if state has accept on EOF
fn has_accept_on_eof(table: &ParseTable, state: usize) -> bool {
    actions_for(table, state, table.eof_symbol)
        .iter()
        .any(|a| matches!(a, Action::Accept))
}

/// Get shift destination for symbol
fn shift_destination(table: &ParseTable, state: usize, sym: SymbolId) -> Option<StateId> {
    actions_for(table, state, sym).iter().find_map(|a| match a {
        Action::Shift(s) => Some(*s),
        _ => None,
    })
}

/// Get reduce rule for symbol
fn reduce_rule(table: &ParseTable, state: usize, sym: SymbolId) -> Option<RuleId> {
    actions_for(table, state, sym).iter().find_map(|a| match a {
        Action::Reduce(r) => Some(*r),
        _ => None,
    })
}

// ===========================================================================
// 1. Simple Grammar Automaton Construction (S → a)
// ===========================================================================

#[test]
fn simple_single_terminal_rule_has_states() {
    let mut g = GrammarBuilder::new("simple_term")
        .token("a", "a")
        .rule("S", vec!["a"])
        .start("S")
        .build();
    let table = build_table(&mut g);

    assert!(
        table.state_count >= 2,
        "S→a needs ≥2 states, got {}",
        table.state_count
    );
    assert!(
        table.state_count <= 10,
        "S→a should not have excessive states: {}",
        table.state_count
    );
}

#[test]
fn simple_grammar_has_shifts() {
    let mut g = GrammarBuilder::new("simple_shifts")
        .token("a", "a")
        .rule("S", vec!["a"])
        .start("S")
        .build();
    let table = build_table(&mut g);

    let shifts = count_shifts(&table);
    assert!(shifts >= 1, "S→a should have at least 1 shift action");
}

#[test]
fn simple_grammar_has_reduces() {
    let mut g = GrammarBuilder::new("simple_reduces")
        .token("a", "a")
        .rule("S", vec!["a"])
        .start("S")
        .build();
    let table = build_table(&mut g);

    let reduces = count_reduces(&table);
    assert!(reduces >= 1, "S→a should have at least 1 reduce action");
}

#[test]
fn simple_grammar_has_accept() {
    let mut g = GrammarBuilder::new("simple_accept")
        .token("a", "a")
        .rule("S", vec!["a"])
        .start("S")
        .build();
    let table = build_table(&mut g);

    let accepts = count_accepts(&table);
    assert!(accepts >= 1, "S→a should have at least 1 accept action");
}

#[test]
fn simple_grammar_initial_state_accessible() {
    let mut g = GrammarBuilder::new("simple_init")
        .token("a", "a")
        .rule("S", vec!["a"])
        .start("S")
        .build();
    let table = build_table(&mut g);

    assert!(
        table.action_table.len() > 0,
        "action table must contain initial state"
    );
}

// ===========================================================================
// 2. Multi-Rule Automaton (Arithmetic Expressions)
// ===========================================================================

#[test]
fn arithmetic_expr_has_multiple_rules() {
    let mut g = GrammarBuilder::new("arith")
        .token("number", "number")
        .token("+", "\\+")
        .token("*", "\\*")
        .rule("E", vec!["T"])
        .rule("E", vec!["E", "+", "T"])
        .rule("T", vec!["F"])
        .rule("T", vec!["T", "*", "F"])
        .rule("F", vec!["number"])
        .start("E")
        .build();
    let table = build_table(&mut g);

    assert!(
        table.rules.len() >= 5,
        "arithmetic grammar should have ≥5 rules"
    );
}

#[test]
fn arithmetic_expr_has_sufficient_states() {
    let mut g = GrammarBuilder::new("arith_states")
        .token("number", "number")
        .token("+", "\\+")
        .token("*", "\\*")
        .rule("E", vec!["T"])
        .rule("E", vec!["E", "+", "T"])
        .rule("T", vec!["F"])
        .rule("T", vec!["T", "*", "F"])
        .rule("F", vec!["number"])
        .start("E")
        .build();
    let table = build_table(&mut g);

    // Arithmetic expressions typically need more states than simple grammars
    assert!(
        table.state_count >= 5,
        "arithmetic grammar needs ≥5 states, got {}",
        table.state_count
    );
}

#[test]
fn arithmetic_expr_has_shifts_and_reduces() {
    let mut g = GrammarBuilder::new("arith_sr")
        .token("number", "number")
        .token("+", "\\+")
        .rule("E", vec!["number"])
        .rule("E", vec!["E", "+", "E"])
        .start("E")
        .build();
    let table = build_table(&mut g);

    let shifts = count_shifts(&table);
    let reduces = count_reduces(&table);
    assert!(
        shifts >= 1 && reduces >= 1,
        "arithmetic needs both shifts and reduces"
    );
}

#[test]
fn multi_rule_goto_table_populated() {
    let mut g = GrammarBuilder::new("multi_goto")
        .token("a", "a")
        .token("b", "b")
        .rule("S", vec!["A", "B"])
        .rule("A", vec!["a"])
        .rule("B", vec!["b"])
        .start("S")
        .build();
    let table = build_table(&mut g);

    assert!(
        table
            .goto_table
            .iter()
            .any(|row| row.iter().any(|&s| s.0 != 0)),
        "goto table should have non-zero entries"
    );
}

// ===========================================================================
// 3. State Count Validation for Known Grammars
// ===========================================================================

#[test]
fn two_terminal_rule_state_count() {
    let mut g = GrammarBuilder::new("two_term")
        .token("a", "a")
        .token("b", "b")
        .rule("S", vec!["a", "b"])
        .start("S")
        .build();
    let table = build_table(&mut g);

    // S→a b: needs at least 3 shift+goto states (init, after-a, after-ab)
    assert!(
        table.state_count >= 3 && table.state_count <= 8,
        "S→a b should have 3-8 states, got {}",
        table.state_count
    );
}

#[test]
fn alternation_state_count() {
    let mut g = GrammarBuilder::new("alt_states")
        .token("a", "a")
        .token("b", "b")
        .rule("S", vec!["a"])
        .rule("S", vec!["b"])
        .start("S")
        .build();
    let table = build_table(&mut g);

    // S→a | b: needs at least 2 shift+goto states
    assert!(
        table.state_count >= 2 && table.state_count <= 6,
        "S→a|b should have 2-6 states, got {}",
        table.state_count
    );
}

#[test]
fn nested_nonterminal_state_count() {
    let mut g = GrammarBuilder::new("nested_nt")
        .token("c", "c")
        .rule("B", vec!["c"])
        .rule("A", vec!["B"])
        .rule("S", vec!["A"])
        .start("S")
        .build();
    let table = build_table(&mut g);

    // S→A, A→B, B→c requires multiple states for closures
    assert!(
        table.state_count >= 4,
        "nested nonterminals need ≥4 states, got {}",
        table.state_count
    );
}

// ===========================================================================
// 4. Action Table Correctness
// ===========================================================================

#[test]
fn action_table_shift_on_correct_terminal() {
    let mut g = GrammarBuilder::new("shift_correct")
        .token("a", "a")
        .rule("S", vec!["a"])
        .start("S")
        .build();
    let table = build_table(&mut g);

    let a_sym = *g
        .tokens
        .iter()
        .find(|(_, tok)| tok.name == "a")
        .map(|(id, _)| id)
        .expect("token 'a' exists");

    // State 0 should have shift on 'a'
    let actions = actions_for(&table, 0, a_sym);
    assert!(
        actions.iter().any(|a| matches!(a, Action::Shift(_))),
        "state 0 should shift on 'a'"
    );
}

#[test]
fn action_table_reduce_on_correct_lookahead() {
    let mut g = GrammarBuilder::new("reduce_lookahead")
        .token("a", "a")
        .rule("S", vec!["a"])
        .start("S")
        .build();
    let table = build_table(&mut g);

    // Find a state with a reduce action
    let has_reduce = table.action_table.iter().any(|row| {
        row.iter()
            .any(|cell| cell.iter().any(|a| matches!(a, Action::Reduce(_))))
    });
    assert!(has_reduce, "table should have at least one reduce action");
}

#[test]
fn action_table_accept_on_eof() {
    let mut g = GrammarBuilder::new("accept_eof")
        .token("a", "a")
        .rule("S", vec!["a"])
        .start("S")
        .build();
    let table = build_table(&mut g);

    // Some state (likely a final state) should have accept on EOF
    let has_accept = (0..table.state_count).any(|s| has_accept_on_eof(&table, s));
    assert!(
        has_accept,
        "table should have accept action on EOF in some state"
    );
}

#[test]
fn action_table_dimensions_match_states_and_symbols() {
    let mut g = GrammarBuilder::new("dimensions")
        .token("a", "a")
        .rule("S", vec!["a"])
        .start("S")
        .build();
    let table = build_table(&mut g);

    assert_eq!(
        table.action_table.len(),
        table.state_count,
        "action table rows must equal state_count"
    );
}

#[test]
fn action_table_no_negative_shift_states() {
    let mut g = GrammarBuilder::new("no_neg_shift")
        .token("a", "a")
        .rule("S", vec!["a"])
        .start("S")
        .build();
    let table = build_table(&mut g);

    for row in &table.action_table {
        for cell in row {
            for action in cell {
                if let Action::Shift(state_id) = action {
                    assert!(
                        state_id.0 < table.state_count as u16,
                        "shift state {} must be < state_count {}",
                        state_id.0,
                        table.state_count
                    );
                }
            }
        }
    }
}

// ===========================================================================
// 5. Goto Table Correctness
// ===========================================================================

#[test]
fn goto_table_dimensions_match_states() {
    let mut g = GrammarBuilder::new("goto_dim")
        .token("a", "a")
        .rule("S", vec!["a"])
        .start("S")
        .build();
    let table = build_table(&mut g);

    assert_eq!(
        table.goto_table.len(),
        table.state_count,
        "goto table rows must equal state_count"
    );
}

#[test]
fn goto_table_nonterminal_indices_valid() {
    let mut g = GrammarBuilder::new("goto_nt_indices")
        .token("a", "a")
        .rule("S", vec!["a"])
        .start("S")
        .build();
    let table = build_table(&mut g);

    for (nt_id, &col_idx) in &table.nonterminal_to_index {
        for row in &table.goto_table {
            if col_idx < row.len() {
                let state_id = row[col_idx];
                assert!(
                    state_id.0 == 0 || state_id.0 < table.state_count as u16,
                    "goto state {} must be < state_count {}",
                    state_id.0,
                    table.state_count
                );
            }
        }
    }
}

#[test]
fn goto_table_populated_for_nonterminals() {
    let mut g = GrammarBuilder::new("goto_populated")
        .token("a", "a")
        .rule("S", vec!["A"])
        .rule("A", vec!["a"])
        .start("S")
        .build();
    let table = build_table(&mut g);

    // Should have at least one nonterminal in nonterminal_to_index
    assert!(
        !table.nonterminal_to_index.is_empty(),
        "nonterminal_to_index should have entries"
    );
}

// ===========================================================================
// 6. Accept Action Placement
// ===========================================================================

#[test]
fn accept_action_present() {
    let mut g = GrammarBuilder::new("accept_present")
        .token("a", "a")
        .rule("S", vec!["a"])
        .start("S")
        .build();
    let table = build_table(&mut g);

    let accepts = count_accepts(&table);
    assert!(accepts >= 1, "table must have at least 1 accept action");
}

#[test]
fn accept_only_on_eof() {
    let mut g = GrammarBuilder::new("accept_eof_only")
        .token("a", "a")
        .rule("S", vec!["a"])
        .start("S")
        .build();
    let table = build_table(&mut g);

    // Check that accept actions only appear in EOF column
    let eof_col = *table.symbol_to_index.get(&table.eof_symbol).unwrap();
    for (state_idx, row) in table.action_table.iter().enumerate() {
        for (col_idx, cell) in row.iter().enumerate() {
            for action in cell {
                if matches!(action, Action::Accept) {
                    assert_eq!(
                        col_idx, eof_col,
                        "accept in state {} col {} should only be in EOF col {}",
                        state_idx, col_idx, eof_col
                    );
                }
            }
        }
    }
}

#[test]
fn accept_in_final_state() {
    let mut g = GrammarBuilder::new("accept_final")
        .token("a", "a")
        .rule("S", vec!["a"])
        .start("S")
        .build();
    let table = build_table(&mut g);

    // Accept should appear in at least one state (typically a final state)
    let accept_states: Vec<_> = (0..table.state_count)
        .filter(|&s| has_accept_on_eof(&table, s))
        .collect();
    assert!(
        !accept_states.is_empty(),
        "some state must have accept on EOF"
    );
}

// ===========================================================================
// 7. First/Follow Set Correctness
// ===========================================================================

#[test]
fn first_set_for_terminal_symbol() {
    let mut g = GrammarBuilder::new("first_term")
        .token("a", "a")
        .rule("S", vec!["a"])
        .start("S")
        .build();
    let ff = FirstFollowSets::compute(&g).expect("FIRST/FOLLOW computation");

    let a_sym = *g
        .tokens
        .iter()
        .find(|(_, tok)| tok.name == "a")
        .map(|(id, _)| id)
        .expect("token 'a' exists");

    // Just check that FIRST set computation succeeds for terminals
    let first_a = ff.first(a_sym);
    assert!(
        first_a.is_some(),
        "FIRST set should be computed for terminals"
    );
}

#[test]
fn first_set_for_nonterminal() {
    let mut g = GrammarBuilder::new("first_nt")
        .token("a", "a")
        .rule("S", vec!["a"])
        .start("S")
        .build();
    let ff = FirstFollowSets::compute(&g).expect("FIRST/FOLLOW computation");

    let s_sym = g.start_symbol().expect("start symbol exists");
    let first_s = ff.first(s_sym).expect("FIRST(S) should exist");

    let a_sym = *g
        .tokens
        .iter()
        .find(|(_, tok)| tok.name == "a")
        .map(|(id, _)| id)
        .expect("token 'a' exists");

    assert!(
        first_s.contains(a_sym.0 as usize),
        "FIRST(S) should contain a from S→a"
    );
}

#[test]
fn nullable_symbol_detection() {
    let mut g = GrammarBuilder::new("nullable")
        .token("a", "a")
        .rule("A", vec![]) // epsilon
        .rule("S", vec!["A"])
        .start("S")
        .build();
    let ff = FirstFollowSets::compute(&g).expect("FIRST/FOLLOW computation");

    // Just verify that FF computation succeeds and detects some nullable symbols
    let nullable_count = g.rules.keys().filter(|&sym| ff.is_nullable(*sym)).count();

    assert!(
        nullable_count > 0,
        "grammar with epsilon rule should have nullable symbols"
    );
}

#[test]
fn non_nullable_symbol() {
    let mut g = GrammarBuilder::new("non_nullable")
        .token("a", "a")
        .rule("S", vec!["a"])
        .start("S")
        .build();
    let ff = FirstFollowSets::compute(&g).expect("FIRST/FOLLOW computation");

    let a_sym = *g
        .tokens
        .iter()
        .find(|(_, tok)| tok.name == "a")
        .map(|(id, _)| id)
        .expect("token 'a' exists");

    assert!(
        !ff.is_nullable(a_sym),
        "terminal 'a' should not be nullable"
    );
}

// ===========================================================================
// 8. Item Set Closure Computation
// ===========================================================================

#[test]
fn item_set_closure_adds_items() {
    let mut g = GrammarBuilder::new("closure_add")
        .token("a", "a")
        .rule("S", vec!["A"])
        .rule("A", vec!["a"])
        .start("S")
        .build();
    let ff = FirstFollowSets::compute(&g).expect("FIRST/FOLLOW computation");
    let col = ItemSetCollection::build_canonical_collection(&mut g, &ff);

    // Initial state (state 0) should have closure items
    assert!(
        col.sets[0].items.len() >= 2,
        "closure should include S→•A and A→•a"
    );
}

#[test]
fn item_set_closure_for_chain() {
    let mut g = GrammarBuilder::new("closure_chain")
        .token("c", "c")
        .rule("C", vec!["c"])
        .rule("B", vec!["C"])
        .rule("A", vec!["B"])
        .rule("S", vec!["A"])
        .start("S")
        .build();
    let ff = FirstFollowSets::compute(&g).expect("FIRST/FOLLOW computation");
    let col = ItemSetCollection::build_canonical_collection(&mut g, &ff);

    // Initial closure should have items for S, A, B, C
    assert!(
        col.sets[0].items.len() >= 4,
        "deep closure should have ≥4 items"
    );
}

// ===========================================================================
// 9. Goto Transitions Between States
// ===========================================================================

#[test]
fn goto_transition_exists() {
    let mut g = GrammarBuilder::new("goto_exists")
        .token("a", "a")
        .rule("S", vec!["a"])
        .start("S")
        .build();
    let table = build_table(&mut g);

    let a_sym = *g
        .tokens
        .iter()
        .find(|(_, tok)| tok.name == "a")
        .map(|(id, _)| id)
        .expect("token 'a' exists");

    // From state 0, shifting 'a' should lead to another state
    let shift_dest = shift_destination(&table, 0, a_sym);
    assert!(
        shift_dest.is_some() && shift_dest.unwrap().0 != 0,
        "state 0 should shift to a different state on 'a'"
    );
}

#[test]
fn goto_nonterminal_transition() {
    let mut g = GrammarBuilder::new("goto_nt")
        .token("a", "a")
        .rule("S", vec!["A"])
        .rule("A", vec!["a"])
        .start("S")
        .build();
    let mut g_clone = g.clone();
    let ff = FirstFollowSets::compute(&mut g_clone).expect("FIRST/FOLLOW computation");
    let col = ItemSetCollection::build_canonical_collection(&mut g, &ff);

    // Should have goto transitions on nonterminals in the grammar
    let nt_gotos: Vec<_> = col
        .goto_table
        .iter()
        .filter(|((_, sym), _)| *col.symbol_is_terminal.get(sym).unwrap_or(&false) == false)
        .collect();

    assert!(
        !nt_gotos.is_empty(),
        "should have at least one nonterminal goto transition"
    );
}

#[test]
fn all_goto_targets_valid_states() {
    let mut g = GrammarBuilder::new("goto_valid")
        .token("a", "a")
        .token("b", "b")
        .rule("S", vec!["A", "B"])
        .rule("A", vec!["a"])
        .rule("B", vec!["b"])
        .start("S")
        .build();
    let ff = FirstFollowSets::compute(&g).expect("FIRST/FOLLOW computation");
    let col = ItemSetCollection::build_canonical_collection(&mut g, &ff);

    for (_, &target) in &col.goto_table {
        assert!(
            target.0 < col.sets.len() as u16,
            "goto target {} must be < {}: size",
            target.0,
            col.sets.len()
        );
    }
}

// ===========================================================================
// 10. Edge Cases: Epsilon-only Rules
// ===========================================================================

#[test]
fn epsilon_rule_handled() {
    let mut g = GrammarBuilder::new("epsilon_rule")
        .token("a", "a")
        .rule("A", vec![]) // epsilon
        .rule("S", vec!["A", "a"])
        .start("S")
        .build();
    let table = build_table(&mut g);

    assert!(
        table.state_count >= 1,
        "epsilon rules should still build table"
    );
}

#[test]
fn epsilon_nullable_propagates() {
    let mut g = GrammarBuilder::new("epsilon_nullable")
        .token("a", "a")
        .rule("B", vec![]) // epsilon
        .rule("A", vec!["B"])
        .rule("S", vec!["A"])
        .start("S")
        .build();
    let ff = FirstFollowSets::compute(&g).expect("FIRST/FOLLOW computation");

    // Just verify that epsilon rules propagate nullable status to multiple symbols
    let nullable_count = g.rules.keys().filter(|&sym| ff.is_nullable(*sym)).count();

    // With B→ε, A→B, S→A, we should have at least B and A as nullable (possibly S too)
    assert!(
        nullable_count >= 2,
        "epsilon should propagate to multiple symbols, got {} nullable",
        nullable_count
    );
}

// ===========================================================================
// 11. Edge Cases: Single-Token Language
// ===========================================================================

#[test]
fn single_token_language() {
    let mut g = GrammarBuilder::new("single_token")
        .token("x", "x")
        .rule("S", vec!["x"])
        .start("S")
        .build();
    let table = build_table(&mut g);

    let x_sym = *g
        .tokens
        .iter()
        .find(|(_, tok)| tok.name == "x")
        .map(|(id, _)| id)
        .expect("token x exists");

    // Should shift on x from state 0
    let shift = shift_destination(&table, 0, x_sym);
    assert!(shift.is_some(), "should shift on x from state 0");

    // Some state should have accept on EOF (not necessarily the shift destination)
    let has_any_accept = (0..table.state_count).any(|s| has_accept_on_eof(&table, s));
    assert!(has_any_accept, "table should have accept action on EOF");
}

// ===========================================================================
// 12. Edge Cases: Deeply Nested Rules
// ===========================================================================

#[test]
fn deeply_nested_nonterminals() {
    let mut g = GrammarBuilder::new("deep_nest")
        .token("x", "x")
        .rule("D", vec!["x"])
        .rule("C", vec!["D"])
        .rule("B", vec!["C"])
        .rule("A", vec!["B"])
        .rule("S", vec!["A"])
        .start("S")
        .build();
    let table = build_table(&mut g);

    // Deep nesting should still produce a valid parse table
    assert!(table.state_count >= 5, "deep nesting needs multiple states");
    assert!(
        count_shifts(&table) >= 1 && count_reduces(&table) >= 1,
        "deep nesting needs shifts and reduces"
    );
}

// ===========================================================================
// 13. Additional Comprehensive Tests
// ===========================================================================

#[test]
fn action_table_eof_symbol_mapped() {
    let mut g = GrammarBuilder::new("eof_mapped")
        .token("a", "a")
        .rule("S", vec!["a"])
        .start("S")
        .build();
    let table = build_table(&mut g);

    let eof_idx = table.symbol_to_index.get(&table.eof_symbol);
    assert!(eof_idx.is_some(), "EOF symbol must be in symbol_to_index");
}

#[test]
fn symbol_count_matches_index_map() {
    let mut g = GrammarBuilder::new("symbol_count")
        .token("a", "a")
        .token("b", "b")
        .rule("S", vec!["a", "b"])
        .start("S")
        .build();
    let table = build_table(&mut g);

    assert_eq!(
        table.symbol_to_index.len(),
        table.index_to_symbol.len(),
        "symbol_to_index and index_to_symbol must have same size"
    );
}

#[test]
fn index_to_symbol_bijection() {
    let mut g = GrammarBuilder::new("index_bijection")
        .token("a", "a")
        .rule("S", vec!["a"])
        .start("S")
        .build();
    let table = build_table(&mut g);

    for (sym, &idx) in &table.symbol_to_index {
        assert_eq!(
            table.index_to_symbol[idx], *sym,
            "index_to_symbol should be inverse of symbol_to_index"
        );
    }
}

#[test]
fn parse_table_rules_correspond_to_grammar() {
    let mut g = GrammarBuilder::new("rules_correspond")
        .token("a", "a")
        .rule("S", vec!["a"])
        .start("S")
        .build();
    let table = build_table(&mut g);

    // Count rules in grammar
    let grammar_rule_count = g.all_rules().count();
    // Parse table has rules as well
    assert!(
        table.rules.len() >= grammar_rule_count,
        "parse table rules should be >= grammar rules"
    );
}

#[test]
fn shift_reduce_consistency() {
    let mut g = GrammarBuilder::new("shift_reduce_consistent")
        .token("a", "a")
        .token("b", "b")
        .rule("S", vec!["a", "b"])
        .start("S")
        .build();
    let table = build_table(&mut g);

    // For each shift destination, verify it's a valid state
    for row in &table.action_table {
        for cell in row {
            for action in cell {
                match action {
                    Action::Shift(StateId(state_id)) => {
                        assert!(
                            *state_id < table.state_count as u16,
                            "shift to invalid state {}: >= {}",
                            state_id,
                            table.state_count
                        );
                    }
                    Action::Reduce(RuleId(rule_id)) => {
                        assert!(
                            *rule_id < table.rules.len() as u16,
                            "reduce by invalid rule {}: >= {}",
                            rule_id,
                            table.rules.len()
                        );
                    }
                    _ => {}
                }
            }
        }
    }
}

#[test]
fn left_recursive_grammar_builds() {
    let mut g = GrammarBuilder::new("left_recursive")
        .token("item", "item")
        .rule("L", vec!["item"])
        .rule("L", vec!["L", "item"])
        .start("L")
        .build();
    let table = build_table(&mut g);

    assert!(
        table.state_count >= 3,
        "left-recursive grammar should build with ≥3 states"
    );
}

#[test]
fn right_recursive_grammar_builds() {
    let mut g = GrammarBuilder::new("right_recursive")
        .token("item", "item")
        .rule("R", vec!["item"])
        .rule("R", vec!["item", "R"])
        .start("R")
        .build();
    let table = build_table(&mut g);

    assert!(
        table.state_count >= 3,
        "right-recursive grammar should build with ≥3 states"
    );
}

#[test]
fn initial_state_in_bounds() {
    let mut g = GrammarBuilder::new("init_bounds")
        .token("a", "a")
        .rule("S", vec!["a"])
        .start("S")
        .build();
    let table = build_table(&mut g);

    assert!(
        table.initial_state.0 < table.state_count as u16,
        "initial_state must be < state_count"
    );
}

#[test]
fn start_symbol_set() {
    let mut g = GrammarBuilder::new("start_symbol_set")
        .token("a", "a")
        .rule("S", vec!["a"])
        .start("S")
        .build();
    let table = build_table(&mut g);

    assert_eq!(
        table.start_symbol,
        g.start_symbol().expect("start symbol"),
        "parse table start symbol should match grammar"
    );
}

#[test]
fn multiple_token_types() {
    let mut g = GrammarBuilder::new("multi_token")
        .token("num", "[0-9]+")
        .token("id", "[a-zA-Z_][a-zA-Z0-9_]*")
        .token("+", "\\+")
        .rule("S", vec!["num", "id", "+"])
        .start("S")
        .build();
    let table = build_table(&mut g);

    let num_sym = *g
        .tokens
        .iter()
        .find(|(_, tok)| tok.name == "num")
        .map(|(id, _)| id)
        .expect("num token");
    let id_sym = *g
        .tokens
        .iter()
        .find(|(_, tok)| tok.name == "id")
        .map(|(id, _)| id)
        .expect("id token");
    let plus_sym = *g
        .tokens
        .iter()
        .find(|(_, tok)| tok.name == "+")
        .map(|(id, _)| id)
        .expect("+ token");

    // All should be in symbol_to_index
    assert!(table.symbol_to_index.contains_key(&num_sym));
    assert!(table.symbol_to_index.contains_key(&id_sym));
    assert!(table.symbol_to_index.contains_key(&plus_sym));
}

#[test]
fn complex_multi_rule_grammar() {
    let mut g = GrammarBuilder::new("complex_multi")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("S", vec!["A", "B"])
        .rule("S", vec!["C"])
        .rule("A", vec!["a"])
        .rule("B", vec!["b", "B"])
        .rule("B", vec!["b"])
        .rule("C", vec!["c"])
        .start("S")
        .build();
    let table = build_table(&mut g);

    // Multiple rules should produce a reasonable state machine
    assert!(table.state_count >= 5, "complex grammar needs >= 5 states");
    assert!(!table.rules.is_empty(), "rules must be populated");
    assert!(table.symbol_count > 0, "symbol_count must be > 0");
}

#[test]
fn grammar_with_common_prefix() {
    let mut g = GrammarBuilder::new("common_prefix")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("S", vec!["a", "b"])
        .rule("S", vec!["a", "c"])
        .start("S")
        .build();
    let table = build_table(&mut g);

    let a_sym = *g
        .tokens
        .iter()
        .find(|(_, tok)| tok.name == "a")
        .map(|(id, _)| id)
        .expect("a");

    // From state 0, should shift on 'a' to merge point
    let shift = shift_destination(&table, 0, a_sym);
    assert!(shift.is_some(), "should shift on 'a' from state 0");
}

#[test]
fn action_table_no_undefined_symbols() {
    let mut g = GrammarBuilder::new("no_undefined")
        .token("a", "a")
        .rule("S", vec!["a"])
        .start("S")
        .build();
    let table = build_table(&mut g);

    for (idx, sym) in table.index_to_symbol.iter().enumerate() {
        assert_eq!(
            table.symbol_to_index.get(sym),
            Some(&idx),
            "every symbol in index_to_symbol must be in symbol_to_index"
        );
    }
}

#[test]
fn parse_table_serializable_structure() {
    let mut g = GrammarBuilder::new("serial")
        .token("a", "a")
        .rule("S", vec!["a"])
        .start("S")
        .build();
    let table = build_table(&mut g);

    // Verify table can be traversed without panics
    assert!(table.state_count > 0);
    assert!(table.symbol_count > 0);
    assert_eq!(table.action_table.len(), table.state_count);
}
