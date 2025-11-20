//! BDD Scenario Tests: GLR Conflict Preservation
//!
//! This test suite validates that shift/reduce conflicts are properly preserved
//! with precedence ordering instead of being eliminated during table generation.
//!
//! Reference: docs/plans/BDD_GLR_CONFLICT_PRESERVATION.md

use rust_sitter_glr_core::{FirstFollowSets, build_lr1_automaton, Action, ParseTable};
use rust_sitter_ir::{Grammar, ProductionId, Rule, Symbol, SymbolId, Token, TokenPattern};

/// Helper: Create the dangling-else grammar for conflict testing
fn create_dangling_else_grammar() -> Grammar {
    let mut grammar = Grammar::new("if_then_else".to_string());

    // Terminals
    let if_id = SymbolId(1);
    let then_id = SymbolId(2);
    let else_id = SymbolId(3);
    let expr_id = SymbolId(4);
    let stmt_id = SymbolId(5);

    grammar.tokens.insert(
        if_id,
        Token {
            name: "if".to_string(),
            pattern: TokenPattern::String("if".to_string()),
            fragile: false,
        },
    );

    grammar.tokens.insert(
        then_id,
        Token {
            name: "then".to_string(),
            pattern: TokenPattern::String("then".to_string()),
            fragile: false,
        },
    );

    grammar.tokens.insert(
        else_id,
        Token {
            name: "else".to_string(),
            pattern: TokenPattern::String("else".to_string()),
            fragile: false,
        },
    );

    grammar.tokens.insert(
        expr_id,
        Token {
            name: "expr".to_string(),
            pattern: TokenPattern::String("expr".to_string()),
            fragile: false,
        },
    );

    grammar.tokens.insert(
        stmt_id,
        Token {
            name: "stmt".to_string(),
            pattern: TokenPattern::String("stmt".to_string()),
            fragile: false,
        },
    );

    // Non-terminal S
    let s_id = SymbolId(10);
    grammar.rule_names.insert(s_id, "S".to_string());

    // Rules creating the dangling else problem
    grammar.rules.insert(
        s_id,
        vec![
            // S → if expr then S
            Rule {
                lhs: s_id,
                rhs: vec![
                    Symbol::Terminal(if_id),
                    Symbol::Terminal(expr_id),
                    Symbol::Terminal(then_id),
                    Symbol::NonTerminal(s_id),
                ],
                precedence: None,
                associativity: None,
                production_id: ProductionId(0),
                fields: vec![],
            },
            // S → if expr then S else S
            Rule {
                lhs: s_id,
                rhs: vec![
                    Symbol::Terminal(if_id),
                    Symbol::Terminal(expr_id),
                    Symbol::Terminal(then_id),
                    Symbol::NonTerminal(s_id),
                    Symbol::Terminal(else_id),
                    Symbol::NonTerminal(s_id),
                ],
                precedence: None,
                associativity: None,
                production_id: ProductionId(1),
                fields: vec![],
            },
            // S → stmt
            Rule {
                lhs: s_id,
                rhs: vec![Symbol::Terminal(stmt_id)],
                precedence: None,
                associativity: None,
                production_id: ProductionId(2),
                fields: vec![],
            },
        ],
    );

    let _ = grammar.get_or_build_registry();
    grammar
}

/// Helper: Analyze parse table for conflicts
struct ConflictAnalysis {
    total_conflicts: usize,
    shift_reduce_conflicts: usize,
    reduce_reduce_conflicts: usize,
    conflict_details: Vec<(usize, usize, Vec<Action>)>, // (state, symbol, actions)
}

fn analyze_conflicts(parse_table: &ParseTable) -> ConflictAnalysis {
    let mut analysis = ConflictAnalysis {
        total_conflicts: 0,
        shift_reduce_conflicts: 0,
        reduce_reduce_conflicts: 0,
        conflict_details: vec![],
    };

    for state in 0..parse_table.state_count {
        for sym in 0..parse_table.symbol_count {
            let actions = &parse_table.action_table[state][sym];
            if actions.len() > 1 {
                analysis.total_conflicts += 1;

                // Classify conflict type
                let has_shift = actions.iter().any(|a| matches!(a, Action::Shift(_)));
                let has_reduce = actions.iter().any(|a| matches!(a, Action::Reduce(_)));

                if has_shift && has_reduce {
                    analysis.shift_reduce_conflicts += 1;
                } else if !has_shift && has_reduce {
                    analysis.reduce_reduce_conflicts += 1;
                }

                analysis.conflict_details.push((state, sym, actions.clone()));
            }
        }
    }

    analysis
}

//
// ============================================================================
// Scenario 1: Detect Shift/Reduce Conflicts in Ambiguous Grammars
// ============================================================================
//

#[test]
fn scenario_1_detect_shift_reduce_conflicts() {
    // GIVEN a grammar with inherent shift/reduce ambiguity (dangling else)
    let grammar = create_dangling_else_grammar();

    // WHEN the LR(1) automaton is constructed
    let first_follow = FirstFollowSets::compute(&grammar).expect("FIRST/FOLLOW computation failed");
    let parse_table = build_lr1_automaton(&grammar, &first_follow).expect("LR(1) automaton build failed");

    // THEN shift/reduce conflicts are detected in the parse table
    let analysis = analyze_conflicts(&parse_table);

    println!("\n=== Scenario 1: Conflict Detection ===");
    println!("Total conflicts: {}", analysis.total_conflicts);
    println!("Shift/reduce conflicts: {}", analysis.shift_reduce_conflicts);
    println!("Reduce/reduce conflicts: {}", analysis.reduce_reduce_conflicts);

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
    let grammar = create_dangling_else_grammar();
    let first_follow = FirstFollowSets::compute(&grammar).expect("FIRST/FOLLOW failed");
    let parse_table = build_lr1_automaton(&grammar, &first_follow).expect("LR(1) build failed");

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
// Scenario 2-5: Precedence Ordering Validation
// ============================================================================
//

#[test]
fn scenario_2_5_precedence_ordering() {
    // This is a placeholder for precedence ordering tests.
    // These scenarios require grammars with explicit precedence/associativity.
    //
    // TODO: Implement scenarios 2-5 when precedence is added to dangling-else grammar:
    // - Scenario 2: PreferShift → [shift, reduce]
    // - Scenario 3: PreferReduce → [reduce, shift]
    // - Scenario 4: No precedence → Fork
    // - Scenario 5: Non-associative → Fork (error)

    println!("\n=== Scenarios 2-5: Precedence Ordering ===");
    println!("⏳ Deferred: Requires grammar with explicit precedence annotations");
    println!("   See: BDD_GLR_CONFLICT_PRESERVATION.md for full spec");
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
    println!("\n=== BDD GLR Conflict Preservation Test Summary ===");
    println!();
    println!("✅ Scenario 1: Conflict detection - IMPLEMENTED");
    println!("⏳ Scenario 2: PreferShift ordering - DEFERRED");
    println!("⏳ Scenario 3: PreferReduce ordering - DEFERRED");
    println!("⏳ Scenario 4: Fork for no precedence - DEFERRED");
    println!("⏳ Scenario 5: Fork for non-associative - DEFERRED");
    println!("✅ Scenario 6: Multi-action cell generation - IMPLEMENTED");
    println!("⏳ Scenario 7: GLR runtime fork/merge - DEFERRED (runtime2 integration)");
    println!("⏳ Scenario 8: Precedence affects tree selection - DEFERRED (runtime2 integration)");
    println!();
    println!("Phase 1 (glr-core unit tests): 2/8 scenarios complete");
    println!("Next: Implement scenarios 2-5 with precedence-annotated grammars");
    println!("Next: Implement scenarios 7-8 in runtime2 end-to-end tests");
}
