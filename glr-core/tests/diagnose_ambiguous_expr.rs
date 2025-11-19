/// Diagnostic test to understand why ambiguous_expr shows 0 conflicts
///
/// This test loads the actual grammar IR from the example build and runs
/// glr-core table generation to see what's happening.

use rust_sitter_glr_core::conflict_inspection::*;
use rust_sitter_glr_core::{FirstFollowSets, build_lr1_automaton};
use std::fs;
use std::path::PathBuf;

#[test]
fn diagnose_ambiguous_expr_grammar() {
    // Find the generated grammar IR
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let workspace_root = PathBuf::from(manifest_dir).parent().unwrap().to_path_buf();

    let target_dir = workspace_root.join("target/debug/build");

    // Find the grammar IR file
    let grammar_ir_path = find_file_recursive(&target_dir, "grammar_ambiguous_expr/grammar.ir.json")
        .expect("Could not find grammar.ir.json - run `cargo build -p rust-sitter-example --features pure-rust` first");

    eprintln!("Loading grammar IR from: {}", grammar_ir_path.display());

    // Load the grammar IR
    let ir_json = fs::read_to_string(&grammar_ir_path)
        .expect("Failed to read grammar IR");

    let mut grammar: rust_sitter_ir::Grammar = serde_json::from_str(&ir_json)
        .expect("Failed to parse grammar IR");

    eprintln!("\n=== Grammar IR ===");
    eprintln!("Name: {}", grammar.name);
    eprintln!("Rules: {} non-terminals", grammar.rules.len());
    eprintln!("Tokens: {} terminals", grammar.tokens.len());

    // Print the key rules (Expr)
    for (symbol_id, rules) in &grammar.rules {
        eprintln!("\nSymbol {:?}:", symbol_id);
        for rule in rules {
            eprintln!("  LHS: {:?}, RHS: {:?}, prec: {:?}, assoc: {:?}",
                rule.lhs, rule.rhs, rule.precedence, rule.associativity);
        }
    }

    // Run table generation
    eprintln!("\n=== Building LR(1) Automaton ===");

    let first_follow = FirstFollowSets::compute_normalized(&mut grammar)
        .expect("FIRST/FOLLOW computation failed");

    let parse_table = build_lr1_automaton(&grammar, &first_follow)
        .expect("LR(1) automaton construction failed");

    eprintln!("States: {}", parse_table.state_count);
    eprintln!("Action table size: {}", parse_table.action_table.len());

    // Inspect conflicts
    eprintln!("\n=== Conflict Inspection ===");
    let summary = count_conflicts(&parse_table);

    eprintln!("Shift/Reduce conflicts: {}", summary.shift_reduce);
    eprintln!("Reduce/Reduce conflicts: {}", summary.reduce_reduce);
    eprintln!("States with conflicts: {:?}", summary.states_with_conflicts);

    if summary.conflict_details.is_empty() {
        eprintln!("\n⚠️  NO CONFLICTS FOUND!");
        eprintln!("This is unexpected for an ambiguous grammar.");

        // Debug: Print action table structure for each state
        eprintln!("\n=== Action Table Structure ===");
        for (state_idx, state_actions) in parse_table.action_table.iter().enumerate() {
            let multi_action_cells: Vec<_> = state_actions.iter()
                .enumerate()
                .filter(|(_, cell)| cell.len() > 1)
                .collect();

            if !multi_action_cells.is_empty() {
                eprintln!("State {}: {} multi-action cells", state_idx, multi_action_cells.len());
                for (symbol_idx, cell) in multi_action_cells {
                    eprintln!("  Symbol {}: {} actions - {:?}", symbol_idx, cell.len(), cell);
                }
            }
        }
    } else {
        eprintln!("\n✅ Conflicts detected:");
        for conflict in &summary.conflict_details {
            eprintln!("  State: {}, Symbol: {}, Type: {:?}, Actions: {}",
                conflict.state.0, conflict.symbol_name, conflict.conflict_type, conflict.actions.len());
        }
    }

    // This is a diagnostic test - we're investigating why conflicts aren't detected
    // If this fails, it confirms the issue is in glr-core table generation
    assert!(
        summary.shift_reduce >= 1 || summary.reduce_reduce >= 1,
        "Expected conflicts in ambiguous_expr grammar, but found none. \
         This indicates glr-core table generation is not creating multi-action cells."
    );
}

/// Recursively search for a file in a directory
fn find_file_recursive(dir: &PathBuf, target: &str) -> Option<PathBuf> {
    if !dir.is_dir() {
        return None;
    }

    for entry in fs::read_dir(dir).ok()? {
        let entry = entry.ok()?;
        let path = entry.path();

        if path.is_dir() {
            if let Some(found) = find_file_recursive(&path, target) {
                return Some(found);
            }
        } else if path.to_str()?.ends_with(target) {
            return Some(path);
        }
    }

    None
}
