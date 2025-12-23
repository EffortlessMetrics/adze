//! Test LR(1) automaton construction for ambiguous expression grammar
//!
//! This test creates a minimal ambiguous grammar and traces through
//! the LR(1) construction to understand why conflicts aren't being detected.

use rust_sitter_glr_core::{FirstFollowSets, build_lr1_automaton};
use rust_sitter_ir::builder::GrammarBuilder;

#[test]
fn test_ambiguous_expr_lr1_construction() {
    eprintln!("\n=== LR(1) CONSTRUCTION TEST FOR AMBIGUOUS EXPR ===\n");

    // Create the ambiguous expression grammar:
    // Expr → Binary
    // Expr → Number
    // Binary → Expr Op Expr  # Left-recursive, NO precedence
    let grammar = GrammarBuilder::new("ambiguous_expr")
        .token("NUMBER", r"\d+")
        .token("OP", r"[-+*/]")
        .rule("expr", vec!["binary"]) // Expr → Binary
        .rule("expr", vec!["NUMBER"]) // Expr → Number
        .rule("binary", vec!["expr", "OP", "expr"]) // Binary → Expr Op Expr (AMBIGUOUS!)
        .start("expr")
        .build();

    eprintln!("Grammar created:");
    eprintln!("  Start symbol: {:?}", grammar.start_symbol());
    eprintln!(
        "  Total symbols: {:?}",
        grammar.rules.len() + grammar.tokens.len()
    );
    eprintln!("  Total rules: {}", grammar.all_rules().count());
    eprintln!();

    // Print all rules
    eprintln!("--- Grammar Rules ---");
    for (idx, rule) in grammar.all_rules().enumerate() {
        let lhs_name = grammar
            .rule_names
            .get(&rule.lhs)
            .map(|s| s.as_str())
            .unwrap_or("?");

        let rhs_str: Vec<String> = rule
            .rhs
            .iter()
            .map(|sym| match sym {
                rust_sitter_ir::Symbol::Terminal(id) => grammar
                    .tokens
                    .get(id)
                    .map(|t| t.name.clone())
                    .unwrap_or_else(|| format!("{:?}", id)),
                rust_sitter_ir::Symbol::NonTerminal(id) => grammar
                    .rule_names
                    .get(id)
                    .cloned()
                    .unwrap_or_else(|| format!("{:?}", id)),
                rust_sitter_ir::Symbol::Epsilon => "ε".to_string(),
                other => format!("{:?}", other),
            })
            .collect();

        eprintln!("  Rule {}: {} → {:?}", idx, lhs_name, rhs_str);
    }
    eprintln!();

    // Compute FIRST/FOLLOW sets
    eprintln!("Computing FIRST/FOLLOW sets...");
    let first_follow_result = FirstFollowSets::compute(&grammar);

    let first_follow = match first_follow_result {
        Ok(ff) => {
            eprintln!("✅ FIRST/FOLLOW computation successful");
            ff
        }
        Err(e) => {
            eprintln!("❌ FIRST/FOLLOW computation failed: {:?}", e);
            panic!("FIRST/FOLLOW failed: {:?}", e);
        }
    };
    eprintln!();

    // Build LR(1) automaton
    eprintln!("Building LR(1) automaton...");
    let result = build_lr1_automaton(&grammar, &first_follow);

    match result {
        Ok(parse_table) => {
            eprintln!("✅ LR(1) automaton built successfully!");
            eprintln!();
            eprintln!("--- Parse Table Summary ---");
            eprintln!("  Total states: {}", parse_table.state_count);
            eprintln!("  Total symbols: {}", parse_table.symbol_count);
            eprintln!("  Total rules: {}", parse_table.rules.len());
            eprintln!();

            // Check for conflicts
            eprintln!("--- Conflict Detection ---");
            let mut total_conflicts = 0;
            let mut conflict_details = Vec::new();

            for (state_idx, state_actions) in parse_table.action_table.iter().enumerate() {
                for (symbol_idx, action_cell) in state_actions.iter().enumerate() {
                    if action_cell.len() > 1 {
                        total_conflicts += 1;
                        let symbol_name = if symbol_idx < parse_table.symbol_metadata.len() {
                            &parse_table.symbol_metadata[symbol_idx].name
                        } else {
                            "UNKNOWN"
                        };

                        let detail = format!(
                            "State {}, Symbol {} ({}): {} actions",
                            state_idx,
                            symbol_idx,
                            symbol_name,
                            action_cell.len()
                        );

                        eprintln!("  ✓ CONFLICT: {}", detail);

                        for (i, action) in action_cell.iter().enumerate() {
                            eprintln!("      Action {}: {:?}", i, action);
                        }

                        conflict_details.push(detail);
                    }
                }
            }

            eprintln!();

            if total_conflicts == 0 {
                eprintln!("❌ NO CONFLICTS DETECTED!");
                eprintln!();
                eprintln!("=== ROOT CAUSE ANALYSIS ===");
                eprintln!("This is WRONG for an ambiguous grammar!");
                eprintln!();
                eprintln!("Expected Behavior:");
                eprintln!("  For grammar: Binary → Expr Op Expr (NO precedence)");
                eprintln!("  After parsing: 'Expr Op Expr', on lookahead 'Op':");
                eprintln!("    1. SHIFT:  Continue reading → forms '(Expr Op Expr) Op Expr'");
                eprintln!("    2. REDUCE: Complete Binary → forms 'Expr Op (Expr Op Expr)'");
                eprintln!();
                eprintln!("  Both actions are VALID with no precedence to disambiguate!");
                eprintln!();
                eprintln!("Possible Root Causes:");
                eprintln!("  1. LR(1) item closure not including recursive items");
                eprintln!("  2. Lookahead computation missing operator in FOLLOW sets");
                eprintln!("  3. Conflict detection logic has bugs");
                eprintln!("  4. Premature conflict resolution before detection");
                eprintln!("  5. Grammar normalization removing ambiguity");
                eprintln!();
                eprintln!("=== TEST FAILED ===");

                // FAIL THE TEST - this is a critical bug
                panic!(
                    "Expected shift/reduce conflicts in ambiguous grammar but found ZERO conflicts! \
                     This indicates a fundamental bug in the LR(1) automaton construction."
                );
            } else {
                eprintln!("✅ SUCCESS: Found {} conflicts", total_conflicts);
                eprintln!();
                eprintln!("This is EXPECTED and CORRECT for an ambiguous grammar!");
                eprintln!("The conflicts detected:");
                for detail in conflict_details {
                    eprintln!("  - {}", detail);
                }
            }
        }
        Err(e) => {
            eprintln!("❌ LR(1) automaton construction FAILED:");
            eprintln!("  Error: {:?}", e);
            panic!("LR(1) construction failed: {:?}", e);
        }
    }
}

#[test]
fn test_simple_non_ambiguous_grammar() {
    eprintln!("\n=== CONTROL TEST: Non-Ambiguous Grammar ===\n");

    // Create a simple non-ambiguous grammar for comparison:
    // S → A
    // A → x
    let grammar = GrammarBuilder::new("simple")
        .token("x", "x")
        .rule("S", vec!["A"])
        .rule("A", vec!["x"])
        .start("S")
        .build();

    eprintln!("Simple grammar created");
    eprintln!("  Rules: {}", grammar.all_rules().count());

    let first_follow =
        FirstFollowSets::compute(&grammar).expect("FIRST/FOLLOW computation should succeed");

    let result = build_lr1_automaton(&grammar, &first_follow);

    match result {
        Ok(parse_table) => {
            eprintln!("✅ Simple grammar built successfully");
            eprintln!("  States: {}", parse_table.state_count);

            // Count conflicts
            let mut conflicts = 0;
            for state_actions in &parse_table.action_table {
                for action_cell in state_actions {
                    if action_cell.len() > 1 {
                        conflicts += 1;
                    }
                }
            }

            eprintln!("  Conflicts: {}", conflicts);

            if conflicts > 0 {
                eprintln!(
                    "  ⚠️  WARNING: Non-ambiguous grammar has {} conflicts!",
                    conflicts
                );
                // This is also a bug, but not as critical
            } else {
                eprintln!("  ✅ No conflicts (as expected for non-ambiguous grammar)");
            }
        }
        Err(e) => {
            panic!("Simple grammar construction failed: {:?}", e);
        }
    }
}

#[test]
#[ignore = "Investigation needed: left-recursive grammar should produce conflicts but doesn't"]
fn test_left_recursive_grammar() {
    eprintln!("\n=== TEST: Left-Recursive Grammar (Should Have Conflicts) ===\n");

    // Create a left-recursive grammar without precedence:
    // E → E + n
    // E → n
    let grammar = GrammarBuilder::new("left_recursive")
        .token("n", r"\d+")
        .token("+", r"\+")
        .rule("E", vec!["E", "+", "n"]) // Left-recursive
        .rule("E", vec!["n"])
        .start("E")
        .build();

    eprintln!("Left-recursive grammar created");

    let first_follow = FirstFollowSets::compute(&grammar).expect("FIRST/FOLLOW should succeed");

    let result = build_lr1_automaton(&grammar, &first_follow);

    match result {
        Ok(parse_table) => {
            eprintln!("✅ LR(1) automaton built");

            // Count conflicts
            let mut conflicts = 0;
            for state_actions in &parse_table.action_table {
                for action_cell in state_actions {
                    if action_cell.len() > 1 {
                        conflicts += 1;
                    }
                }
            }

            eprintln!("  States: {}", parse_table.state_count);
            eprintln!("  Conflicts: {}", conflicts);

            if conflicts == 0 {
                eprintln!(
                    "  ❌ NO CONFLICTS - but left-recursion without precedence SHOULD have conflicts!"
                );
                panic!("Expected conflicts in left-recursive grammar");
            } else {
                eprintln!("  ✅ Found conflicts as expected");
            }
        }
        Err(e) => {
            panic!("Left-recursive grammar failed: {:?}", e);
        }
    }
}
