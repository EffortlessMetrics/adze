//! Diagnostic test to validate glr-core conflict generation
//!
//! This test directly validates that glr-core generates multi-action cells
//! for ambiguous grammars, BEFORE any TSLanguage encoding happens.
//!
//! This is the critical missing validation from Phase 2.

use rust_sitter_glr_core::conflict_inspection::*;
use rust_sitter_glr_core::{build_lr1_automaton, FirstFollowSets};
use rust_sitter_ir::builder::GrammarBuilder;
use rust_sitter_ir::Grammar;

/// Build a minimal ambiguous expression grammar programmatically
///
/// Grammar:
///   expr → expr + expr    (NO precedence, NO associativity)
///   expr → NUMBER
///
/// This should generate at least one shift/reduce conflict:
/// After parsing "1 + 2" on lookahead "+":
///   - Shift: Continue to form "(1 + 2) + 3"
///   - Reduce: Complete "1 + 2", then form "1 + (2 + 3)"
fn build_ambiguous_expr_grammar() -> Grammar {
    GrammarBuilder::new("ambiguous_expr_test")
        .token("NUMBER", r"\d+")
        .token("+", "+")
        .rule("expr", vec!["expr", "+", "expr"]) // NO precedence!
        .rule("expr", vec!["NUMBER"])
        .start("expr")
        .build()
}

/// Build a simple unambiguous grammar for comparison
///
/// Grammar:
///   expr → NUMBER
///
/// This should generate NO conflicts.
fn build_simple_unambiguous_grammar() -> Grammar {
    GrammarBuilder::new("simple_unambiguous_test")
        .token("NUMBER", r"\d+")
        .rule("expr", vec!["NUMBER"])
        .start("expr")
        .build()
}

#[test]
fn test_glr_core_generates_conflicts_for_ambiguous_grammar() {
    eprintln!("\n=== GLR-CORE CONFLICT GENERATION TEST ===");
    eprintln!("Objective: Validate glr-core creates multi-action cells for ambiguous grammars");
    eprintln!("Context: Phase 2 found 0 conflicts through TSLanguage decode");
    eprintln!("Question: Does glr-core generate conflicts BEFORE encoding?\n");

    let mut grammar = build_ambiguous_expr_grammar();

    eprintln!("Grammar: {}", grammar.name);
    eprintln!("Rules: {} non-terminals", grammar.rules.len());
    eprintln!("Tokens: {} terminals", grammar.tokens.len());

    // Print grammar structure
    eprintln!("\nGrammar Structure:");
    for (symbol_id, productions) in &grammar.rules {
        eprintln!("  Symbol {:?}:", symbol_id);
        for prod in productions {
            eprintln!(
                "    {:?} → {:?} (prec: {:?}, assoc: {:?})",
                prod.lhs, prod.rhs, prod.precedence, prod.associativity
            );
        }
    }

    eprintln!("\n--- Running glr-core Pipeline ---");

    // Step 1: Compute FIRST/FOLLOW sets
    eprintln!("Step 1: Computing FIRST/FOLLOW sets...");
    let first_follow = FirstFollowSets::compute_normalized(&mut grammar)
        .expect("FIRST/FOLLOW computation failed");
    eprintln!("  ✓ FIRST/FOLLOW computed");

    // Step 2: Build LR(1) automaton (this is where conflicts should appear)
    eprintln!("Step 2: Building LR(1) automaton...");
    let parse_table = build_lr1_automaton(&grammar, &first_follow)
        .expect("LR(1) automaton construction failed");

    eprintln!("  ✓ ParseTable generated");
    eprintln!("  States: {}", parse_table.state_count);
    eprintln!("  Action table dimensions: {} states", parse_table.action_table.len());

    // Step 3: Inspect for multi-action cells DIRECTLY
    eprintln!("\n--- Direct Action Table Inspection ---");
    let mut total_multi_action_cells = 0;
    let mut states_with_conflicts = vec![];

    for (state_idx, state_actions) in parse_table.action_table.iter().enumerate() {
        let multi_action_cells: Vec<_> = state_actions
            .iter()
            .enumerate()
            .filter(|(_, cell)| cell.len() > 1)
            .collect();

        if !multi_action_cells.is_empty() {
            total_multi_action_cells += multi_action_cells.len();
            states_with_conflicts.push(state_idx);

            eprintln!("State {}: {} multi-action cells", state_idx, multi_action_cells.len());
            for (symbol_idx, cell) in &multi_action_cells {
                eprintln!("  Symbol {}: {} actions", symbol_idx, cell.len());
                for action in cell.iter() {
                    eprintln!("    - {:?}", action);
                }
            }
        }
    }

    eprintln!("\nDirect Inspection Results:");
    eprintln!("  Total multi-action cells: {}", total_multi_action_cells);
    eprintln!("  States with conflicts: {:?}", states_with_conflicts);

    // Step 4: Run conflict inspection API
    eprintln!("\n--- Conflict Inspection API ---");
    let summary = count_conflicts(&parse_table);

    let total_conflicts = summary.shift_reduce + summary.reduce_reduce;

    eprintln!("Conflict Summary:");
    eprintln!("  Shift/Reduce conflicts: {}", summary.shift_reduce);
    eprintln!("  Reduce/Reduce conflicts: {}", summary.reduce_reduce);
    eprintln!("  Total conflicts: {}", total_conflicts);
    eprintln!("  States with conflicts: {:?}", summary.states_with_conflicts);

    if !summary.conflict_details.is_empty() {
        eprintln!("\nConflict Details:");
        for conflict in &summary.conflict_details {
            eprintln!(
                "  State {}, Symbol '{}': {:?} ({} actions)",
                conflict.state.0,
                conflict.symbol_name,
                conflict.conflict_type,
                conflict.actions.len()
            );
        }
    }

    // Validation
    eprintln!("\n=== VALIDATION ===");

    if total_multi_action_cells == 0 && total_conflicts == 0 {
        eprintln!("❌ FAILURE: No conflicts detected by either method");
        eprintln!("\nConclusion: The issue is in glr-core table generation");
        eprintln!("Root cause: build_lr1_automaton() is NOT creating multi-action cells");
        eprintln!("Impact: GLR conflicts are being resolved during table generation");
        eprintln!("Next step: Investigate conflict resolution in build_lr1_automaton()");

        panic!(
            "glr-core did not generate conflicts for ambiguous grammar. \
             Expected at least 1 shift/reduce conflict for 'expr → expr + expr' rule."
        );
    } else if total_multi_action_cells > 0 && total_conflicts == 0 {
        eprintln!("⚠️  PARTIAL: Multi-action cells exist but conflict_inspection doesn't detect them");
        eprintln!("Conclusion: conflict_inspection API has a bug");

        panic!(
            "Multi-action cells detected ({}) but conflict_inspection reports 0 conflicts. \
             Bug in conflict_inspection API.",
            total_multi_action_cells
        );
    } else {
        eprintln!("✅ SUCCESS: Conflicts detected in glr-core output");
        eprintln!("  Multi-action cells: {}", total_multi_action_cells);
        eprintln!("  Conflicts reported: {}", total_conflicts);
        eprintln!("\nConclusion: glr-core generates conflicts correctly");
        eprintln!("Impact: Phase 2 issue is confirmed to be in encode/decode pipeline");
        eprintln!("Next step: Implement pure-Rust GLR runtime (bypass TSLanguage)");

        assert!(
            summary.shift_reduce >= 1,
            "Expected at least 1 shift/reduce conflict"
        );
    }
}

#[test]
fn test_glr_core_no_conflicts_for_unambiguous_grammar() {
    eprintln!("\n=== UNAMBIGUOUS GRAMMAR BASELINE TEST ===");

    let mut grammar = build_simple_unambiguous_grammar();

    eprintln!("Grammar: {}", grammar.name);

    let first_follow = FirstFollowSets::compute_normalized(&mut grammar)
        .expect("FIRST/FOLLOW computation failed");

    let parse_table = build_lr1_automaton(&grammar, &first_follow)
        .expect("LR(1) automaton construction failed");

    let summary = count_conflicts(&parse_table);

    let total_conflicts = summary.shift_reduce + summary.reduce_reduce;
    eprintln!("Conflicts: {}", total_conflicts);

    assert_eq!(
        total_conflicts,
        0,
        "Unambiguous grammar should have no conflicts"
    );

    eprintln!("✅ Baseline validated: Unambiguous grammar has 0 conflicts");
}
