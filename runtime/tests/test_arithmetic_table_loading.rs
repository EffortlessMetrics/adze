#![cfg(feature = "with-grammars")]
//! Test to diagnose parse table loading for arithmetic grammar
//!
//! This test loads the arithmetic grammar's parse table and inspects
//! its structure to verify that:
//! 1. Tables are loaded correctly
//! 2. Multi-action cells are preserved
//! 3. State 0 has the expected actions

#[cfg(all(feature = "pure-rust", feature = "glr"))]
#[test]
fn inspect_arithmetic_parse_table() {
    // Access the generated arithmetic language
    let lang = &adze_example::arithmetic::generated::LANGUAGE;

    // Decode the parse table
    let parse_table = adze::decoder::decode_parse_table(lang);

    eprintln!("\n=== Arithmetic Grammar Parse Table Inspection ===");
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

    // Check if any state has multi-action cells (GLR conflicts)
    eprintln!("\n--- Multi-Action Cells (GLR Conflicts) ---");
    let mut found_conflicts = false;
    for (state_idx, state) in parse_table.action_table.iter().enumerate() {
        for (symbol_idx, action_cell) in state.iter().enumerate() {
            if action_cell.len() > 1 {
                found_conflicts = true;
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
                for action in action_cell {
                    eprintln!("    {:?}", action);
                }
            }
        }
    }

    if !found_conflicts {
        eprintln!("  No multi-action cells found (no GLR conflicts detected)");
    }

    // Inspect rules
    eprintln!("\n--- Parse Rules ---");
    for (i, rule) in parse_table.rules.iter().enumerate().take(10) {
        eprintln!("  Rule {}: LHS={}, RHS_LEN={}", i, rule.lhs.0, rule.rhs_len);
    }

    // This test always passes - it's just for inspection
    assert!(
        !parse_table.action_table.is_empty(),
        "Parse table should have at least one state"
    );
}
