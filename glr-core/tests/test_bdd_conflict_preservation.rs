//! BDD Scenario Tests: GLR Conflict Preservation
//!
//! This test suite validates that shift/reduce conflicts are properly preserved
//! with precedence ordering instead of being eliminated during table generation.
//!
//! Reference: docs/plans/BDD_GLR_CONFLICT_PRESERVATION.md

use adze_bdd_scenario_fixtures::{
    BddPhase, analyze_conflicts, bdd_progress_report_for_current_profile, build_lr1_parse_table,
    dangling_else_grammar, no_precedence_grammar, precedence_arithmetic_grammar,
    resolve_shift_reduce_actions,
};
use adze_glr_core::{Action, ParseTable};
use adze_ir::{Associativity, RuleId, StateId, SymbolId};

const PRECEDENCE_PLUS_TOKEN: SymbolId = SymbolId(2);
const PRECEDENCE_STAR_TOKEN: SymbolId = SymbolId(3);

//
// ============================================================================
// Scenario 1: Detect Shift/Reduce Conflicts in Ambiguous Grammars
// ============================================================================
//

#[test]
fn scenario_1_detect_shift_reduce_conflicts() {
    // GIVEN a grammar with inherent shift/reduce ambiguity (dangling else)
    let grammar = dangling_else_grammar();

    // WHEN the LR(1) automaton is constructed
    let parse_table = build_lr1_parse_table(&grammar).expect("LR(1) automaton build failed");

    // THEN shift/reduce conflicts are detected in the parse table
    let analysis = analyze_conflicts(&parse_table);

    println!("\n=== Scenario 1: Conflict Detection ===");
    println!("Total conflicts: {}", analysis.total_conflicts);
    println!(
        "Shift/reduce conflicts: {}",
        analysis.shift_reduce_conflicts
    );
    println!(
        "Reduce/reduce conflicts: {}",
        analysis.reduce_reduce_conflicts
    );

    // AND the conflicts are reported with state and symbol information
    for (state, sym, actions) in &analysis.conflict_details {
        println!("\nConflict in state {}, symbol {}:", state, sym);
        for (i, action) in actions.iter().enumerate() {
            println!("  [{}] {:?}", i, action);
        }
    }

    // THEN we should have detected at least one shift/reduce conflict
    assert!(
        analysis.shift_reduce_conflicts > 0,
        "Expected shift/reduce conflicts in dangling else grammar, found none"
    );

    // AND conflicts should be preserved (multi-action cells)
    assert!(
        analysis.total_conflicts > 0,
        "Expected conflicts to be preserved as multi-action cells"
    );
}

//
// ============================================================================
// Scenario 6: Multi-Action Cells in Generated Parse Tables
// ============================================================================
//

#[test]
fn scenario_6_multi_action_cells_generated() {
    // GIVEN a grammar with preserved conflicts
    let grammar = dangling_else_grammar();
    let parse_table = build_lr1_parse_table(&grammar).expect("LR(1) build failed");

    // WHEN the parse table is inspected
    let analysis = analyze_conflicts(&parse_table);

    println!("\n=== Scenario 6: Multi-Action Cell Generation ===");
    println!("Parse table statistics:");
    println!("  States: {}", parse_table.state_count);
    println!("  Symbols: {}", parse_table.symbol_count);
    println!("  Multi-action cells: {}", analysis.total_conflicts);

    // THEN multi-action cells are created in the action table
    assert!(
        analysis.total_conflicts > 0,
        "Expected multi-action cells in parse table"
    );

    // AND cells contain all preserved actions
    for (_state, _sym, actions) in &analysis.conflict_details {
        assert!(
            actions.len() >= 2,
            "Multi-action cell should have at least 2 actions, found {}",
            actions.len()
        );

        println!("\nMulti-action cell with {} actions:", actions.len());
        for (i, action) in actions.iter().enumerate() {
            println!("  [{}] {:?}", i, action);
        }
    }

    // AND action table preserves conflict information
    // (This validates that glr-core preserves conflicts instead of eliminating them)
    println!("\n✓ Multi-action cells successfully preserved in parse table");
}

//
// ============================================================================
// Scenario 2: Preserve Conflicts with Precedence Ordering (PreferShift)
// ============================================================================
//

#[test]
fn scenario_2_prefer_shift_resolution() {
    // GIVEN a conflict where lookahead token (*) has higher precedence than reduce rule (+)
    let grammar = precedence_arithmetic_grammar(Associativity::Left);

    // WHEN shift/reduce conflict is resolved on lookahead '*'
    let actions = resolve_shift_reduce_actions(&grammar, PRECEDENCE_STAR_TOKEN, RuleId(0));

    // THEN shift action is preferred and reduce is eliminated
    assert_eq!(actions, vec![Action::Shift(StateId(7))]);
}

//
// ============================================================================
// Scenario 3: Preserve Conflicts with Precedence Ordering (PreferReduce)
// ============================================================================
//

#[test]
fn scenario_3_prefer_reduce_resolution() {
    // GIVEN a conflict where reduce rule (*) has higher precedence than lookahead token (+)
    let grammar = precedence_arithmetic_grammar(Associativity::Left);

    // WHEN shift/reduce conflict is resolved on lookahead '+'
    let actions = resolve_shift_reduce_actions(&grammar, PRECEDENCE_PLUS_TOKEN, RuleId(1));

    // THEN reduce action is preferred and shift is eliminated
    assert_eq!(actions, vec![Action::Reduce(RuleId(1))]);
}

//
// ============================================================================
// Scenario 4: Use Fork for No Precedence Information
// ============================================================================
//

#[test]
fn scenario_4_fork_when_no_precedence_information() {
    // GIVEN a conflict with no precedence metadata on token or rule
    let grammar = no_precedence_grammar();

    // WHEN shift/reduce conflict is resolved
    let actions = resolve_shift_reduce_actions(&grammar, SymbolId(1), RuleId(0));

    // THEN resolver keeps both paths via Fork
    assert_eq!(actions.len(), 1);
    match &actions[0] {
        Action::Fork(inner) => {
            assert_eq!(
                inner,
                &vec![Action::Shift(StateId(7)), Action::Reduce(RuleId(0))]
            );
        }
        other => panic!("expected Fork action, got {:?}", other),
    }
}

//
// ============================================================================
// Scenario 5: Use Fork for Non-Associative Conflicts
// ============================================================================
//

#[test]
fn scenario_5_fork_when_non_associative() {
    // GIVEN equal precedence with non-associative rule
    let grammar = precedence_arithmetic_grammar(Associativity::None);

    // WHEN shift/reduce conflict is resolved on '+'
    let actions = resolve_shift_reduce_actions(&grammar, PRECEDENCE_PLUS_TOKEN, RuleId(0));

    // THEN resolver returns Fork to preserve ambiguity/error path
    assert_eq!(actions.len(), 1);
    match &actions[0] {
        Action::Fork(inner) => {
            assert_eq!(
                inner,
                &vec![Action::Shift(StateId(7)), Action::Reduce(RuleId(0))]
            );
        }
        other => panic!("expected Fork action, got {:?}", other),
    }
}

//
// ============================================================================
// Helper: Print ParseTable for Debugging
// ============================================================================
//

#[allow(dead_code)]
fn print_parse_table(parse_table: &ParseTable) {
    println!("\n=== Parse Table Dump ===");
    println!("States: {}", parse_table.state_count);
    println!("Symbols: {}", parse_table.symbol_count);

    println!("\nAction Table:");
    for state in 0..parse_table.state_count {
        for sym in 0..parse_table.symbol_count {
            let actions = &parse_table.action_table[state][sym];
            if !actions.is_empty() {
                println!("  [{}, {}] = {:?}", state, sym, actions);
            }
        }
    }

    println!("\nGoto Table:");
    for state in 0..parse_table.state_count {
        for nt in 0..parse_table.goto_table[state].len() {
            let next_state = &parse_table.goto_table[state][nt];
            if next_state.0 != 0 || state == 0 {
                println!("  [{}, NT{}] → State {}", state, nt, next_state.0);
            }
        }
    }
}

//
// ============================================================================
// BDD Test Summary
// ============================================================================
//

#[test]
fn bdd_test_summary() {
    let status =
        bdd_progress_report_for_current_profile(BddPhase::Core, "Phase 1 (glr-core unit tests)");
    println!("{status}");
}
