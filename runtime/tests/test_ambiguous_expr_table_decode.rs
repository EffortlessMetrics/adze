#![cfg(feature = "with-grammars")]
//! Deep decode of ambiguous_expr parse table to understand why no conflicts are found

#[cfg(all(feature = "pure-rust", feature = "glr"))]
use adze_glr_core::Action;

#[cfg(all(feature = "pure-rust", feature = "glr"))]
#[test]
fn decode_ambiguous_expr_table_deep() {
    // Access the generated ambiguous_expr language
    let lang = &adze_example::ambiguous_expr::generated::LANGUAGE;

    // Decode the parse table
    let parse_table = adze::decoder::decode_parse_table(lang);

    eprintln!("\n=== DEEP TABLE DECODE FOR AMBIGUOUS_EXPR ===\n");
    eprintln!("Total states: {}", parse_table.action_table.len());
    eprintln!("Total symbols: {}", parse_table.symbol_count);
    eprintln!("Total rules: {}\n", parse_table.rules.len());

    // Print all rules first
    eprintln!("--- ALL PARSE RULES ---");
    for (i, rule) in parse_table.rules.iter().enumerate() {
        let lhs_name = if (rule.lhs.0 as usize) < parse_table.symbol_metadata.len() {
            &parse_table.symbol_metadata[rule.lhs.0 as usize].name
        } else {
            "UNKNOWN"
        };
        eprintln!("  Rule {}: {} → <{} symbols>", i, lhs_name, rule.rhs_len);
    }

    // Print complete action table
    eprintln!("\n--- COMPLETE ACTION TABLE ---");
    for (state_idx, state) in parse_table.action_table.iter().enumerate() {
        eprintln!("\nState {}:", state_idx);

        for (symbol_idx, action_cell) in state.iter().enumerate() {
            if !action_cell.is_empty() {
                let symbol_name = if symbol_idx < parse_table.symbol_metadata.len() {
                    &parse_table.symbol_metadata[symbol_idx].name
                } else {
                    "UNKNOWN"
                };

                eprintln!("  On symbol {} ({}):", symbol_idx, symbol_name);

                for (action_idx, action) in action_cell.iter().enumerate() {
                    match action {
                        Action::Shift(target_state) => {
                            eprintln!("    Action {}: SHIFT to state {}", action_idx, target_state);
                        }
                        Action::Reduce(rule_id) => {
                            let rule = &parse_table.rules[rule_id.0 as usize];
                            let lhs_name =
                                if (rule.lhs.0 as usize) < parse_table.symbol_metadata.len() {
                                    &parse_table.symbol_metadata[rule.lhs.0 as usize].name
                                } else {
                                    "UNKNOWN"
                                };
                            eprintln!(
                                "    Action {}: REDUCE via rule {} ({} → <{} symbols>)",
                                action_idx, rule_id.0, lhs_name, rule.rhs_len
                            );
                        }
                        Action::Accept => {
                            eprintln!("    Action {}: ACCEPT", action_idx);
                        }
                        _ => {
                            eprintln!("    Action {}: OTHER ({:?})", action_idx, action);
                        }
                    }
                }

                // Highlight multi-action cells
                if action_cell.len() > 1 {
                    eprintln!(
                        "    ⚠️  CONFLICT: {} actions in this cell!",
                        action_cell.len()
                    );
                }
            }
        }
    }

    // Analyze for expected conflicts
    eprintln!("\n--- CONFLICT ANALYSIS ---");
    eprintln!("Looking for expected shift/reduce conflicts...");
    eprintln!("Expected: After parsing 'Expr Op Expr', on operator lookahead:");
    eprintln!("  - SHIFT: Continue reading (form left-associative)");
    eprintln!("  - REDUCE: Complete current expr (form right-associative)");
    eprintln!();

    let mut found_any_conflict = false;
    for (state_idx, state) in parse_table.action_table.iter().enumerate() {
        for (symbol_idx, action_cell) in state.iter().enumerate() {
            if action_cell.len() > 1 {
                found_any_conflict = true;
                let symbol_name = if symbol_idx < parse_table.symbol_metadata.len() {
                    &parse_table.symbol_metadata[symbol_idx].name
                } else {
                    "UNKNOWN"
                };
                eprintln!(
                    "✓ Found conflict in state {} on symbol {}",
                    state_idx, symbol_name
                );
            }
        }
    }

    if !found_any_conflict {
        eprintln!("❌ NO CONFLICTS FOUND");
        eprintln!("\nThis means:");
        eprintln!("  1. The grammar was constructed without ambiguity");
        eprintln!("  2. LR(1) lookahead successfully disambiguated all states");
        eprintln!("  3. OR: Conflicts were eliminated during table generation");
        eprintln!("\nNext steps:");
        eprintln!("  - Check grammar IR to see what productions were created");
        eprintln!("  - Verify LR(1) item sets during automaton construction");
        eprintln!("  - Examine if precedence is being applied during grammar normalization");
    } else {
        eprintln!("✅ CONFLICTS DETECTED - GLR preservation is working!");
    }
}
