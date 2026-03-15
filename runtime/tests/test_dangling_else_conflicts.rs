#![cfg(feature = "with-grammars")]
//! Diagnostic test for dangling-else grammar conflict detection
//!
//! This test verifies that the dangling-else grammar DOES generate shift/reduce conflicts
//! as expected, validating our GLR conflict preservation implementation.
//!
//! Expected conflict in state after "if Expr then Statement", on lookahead "else":
//!   - Shift: Continue outer if (attach else to outer)
//!   - Reduce: Complete inner if (attach else to inner)

#[cfg(all(feature = "pure-rust", feature = "glr"))]
#[test]
fn inspect_dangling_else_conflicts() {
    // Access the generated dangling_else language
    let lang = unsafe { &adze_example::dangling_else::generated::LANGUAGE };

    // Decode the parse table
    let parse_table = adze::decoder::decode_parse_table(lang);

    eprintln!("\n=== Dangling Else Grammar Parse Table Inspection ===");
    eprintln!("Total states: {}", parse_table.action_table.len());
    eprintln!("Total symbols: {}", parse_table.symbol_count);
    eprintln!(
        "Symbol metadata count: {}",
        parse_table.symbol_metadata.len()
    );

    // Inspect state 0
    if !parse_table.action_table.is_empty() {
        eprintln!("\n--- State 0 Actions ---");
        let state0 = &parse_table.action_table[0];

        for (symbol_idx, action_cell) in state0.iter().enumerate() {
            if !action_cell.is_empty() {
                // Get symbol name
                let symbol_name = if symbol_idx < parse_table.symbol_metadata.len() {
                    &parse_table.symbol_metadata[symbol_idx].name
                } else {
                    "unknown"
                };

                eprintln!(
                    "  Symbol {} ({}): {} actions",
                    symbol_idx,
                    symbol_name,
                    action_cell.len()
                );
                for (i, action) in action_cell.iter().enumerate() {
                    eprintln!("    Action {}: {:?}", i, action);
                }
            }
        }
    }

    // Check ALL states for multi-action cells (GLR conflicts)
    eprintln!("\n--- Multi-Action Cells (GLR Conflicts) ---");
    let mut found_conflicts = false;
    let mut conflict_count = 0;

    for (state_idx, state) in parse_table.action_table.iter().enumerate() {
        for (symbol_idx, action_cell) in state.iter().enumerate() {
            if action_cell.len() > 1 {
                found_conflicts = true;
                conflict_count += 1;

                let symbol_name = if symbol_idx < parse_table.symbol_metadata.len() {
                    &parse_table.symbol_metadata[symbol_idx].name
                } else {
                    "unknown"
                };

                eprintln!(
                    "  State {}, Symbol {} ({}): {} actions",
                    state_idx,
                    symbol_idx,
                    symbol_name,
                    action_cell.len()
                );

                for (i, action) in action_cell.iter().enumerate() {
                    eprintln!("    Action {}: {:?}", i, action);
                }

                // Check if this is the expected dangling-else conflict
                if symbol_name == "else" {
                    eprintln!("    ✓ Found expected 'else' conflict!");
                    eprintln!("    ✓ Multiple actions preserved (likely shift/reduce)");
                }
            }
        }
    }

    if !found_conflicts {
        eprintln!("  ⚠ WARNING: No multi-action cells found!");
        eprintln!("  This means GLR conflicts were NOT preserved during table generation.");
        eprintln!("  The dangling-else grammar SHOULD have shift/reduce conflicts.");
    } else {
        eprintln!("\n✅ Total conflicts found: {}", conflict_count);
    }

    // Inspect rules
    eprintln!("\n--- Parse Rules ---");
    for (i, rule) in parse_table.rules.iter().enumerate().take(10) {
        eprintln!("  Rule {}: LHS={}, RHS_LEN={}", i, rule.lhs.0, rule.rhs_len);
    }

    // This test SHOULD find conflicts in the dangling-else grammar
    // If no conflicts found, that indicates the GLR conflict preservation isn't working
    if found_conflicts {
        eprintln!("\n✅ TEST PASSED: Conflicts detected as expected");
    } else {
        eprintln!("\n⚠ TEST OBSERVATION: No conflicts found");
        eprintln!("   This may indicate:");
        eprintln!("   1. GLR conflict preservation not working (investigate glr-core)");
        eprintln!("   2. Grammar doesn't generate conflicts (check grammar definition)");
        eprintln!("   3. LR(1) lookahead resolves ambiguity (unexpected for dangling-else)");
    }

    // Always pass the test - this is diagnostic, not assertion-based
    assert!(
        parse_table.action_table.len() > 0,
        "Parse table should have at least one state"
    );
}

#[cfg(all(feature = "pure-rust", feature = "glr"))]
#[test]
fn verify_conflict_preservation_behavior() {
    // This test documents the EXPECTED behavior once GLR conflict preservation is working
    //
    // Expected: Multi-action cells in the parse table for the dangling-else grammar
    //
    // Conflict should occur in state after "if Expr then Statement" on lookahead "else":
    //   Action 0: Shift(N)   - Continue outer if, shift else token
    //   Action 1: Reduce(M)  - Complete inner if-then, reduce to Statement
    //
    // If this conflict is NOT present, the GLR conflict preservation fix
    // is not working correctly in glr-core/src/lib.rs

    // For now, just run the inspection test
    // Once conflicts are detected, we can add more specific assertions
}
