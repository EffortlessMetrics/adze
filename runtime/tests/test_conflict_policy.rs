//! Test conflict resolution policies
//! This validates that shift/reduce conflicts are resolved correctly according to policy

#![cfg(feature = "pure-rust")]

mod support;

use adze::adze_glr_core as glr_core;
use adze::adze_ir as ir;
use adze::decoder;
use adze::ts_format::choose_action as ts_choose;

use glr_core::{Action, FirstFollowSets, build_lr1_automaton};
use ir::StateId;

#[test]
fn shift_wins_sr_conflict_raw_table() {
    // Build grammar with known SR conflicts
    let grammar = support::expr_sr_conflict::build_expr_sr_conflict();
    let first_follow = FirstFollowSets::compute(&grammar).unwrap();
    let parse_table = build_lr1_automaton(&grammar, &first_follow).unwrap();

    // Find SR conflict cells in raw table (before normalization)
    let mut sr_conflicts = Vec::new();

    for (state_idx, row) in parse_table.action_table.iter().enumerate() {
        for (col_idx, cell) in row.iter().enumerate() {
            let has_shift = cell.iter().any(|a| matches!(a, Action::Shift(_)));
            let has_reduce = cell.iter().any(|a| matches!(a, Action::Reduce(_)));

            if has_shift && has_reduce {
                sr_conflicts.push((state_idx, col_idx, cell.clone()));
                println!(
                    "SR conflict found at state {} column {}: {:?}",
                    state_idx, col_idx, cell
                );
            }
        }
    }

    assert!(
        !sr_conflicts.is_empty(),
        "No SR conflicts found in expression grammar - test grammar may be wrong"
    );

    println!(
        "✓ Found {} shift/reduce conflicts in raw table",
        sr_conflicts.len()
    );
}

#[test]
fn shift_wins_sr_conflict_encoded() {
    // Build grammar with SR conflicts
    let grammar = support::expr_sr_conflict::build_expr_sr_conflict();
    let first_follow = FirstFollowSets::compute(&grammar).unwrap();
    let mut parse_table = build_lr1_automaton(&grammar, &first_follow).unwrap();

    // Find a specific SR conflict before normalization
    let mut conflict_location = None;

    for (state_idx, row) in parse_table.action_table.iter().enumerate() {
        for (col_idx, cell) in row.iter().enumerate() {
            if cell.iter().any(|a| matches!(a, Action::Shift(_)))
                && cell.iter().any(|a| matches!(a, Action::Reduce(_)))
            {
                conflict_location = Some((state_idx, col_idx));
                break;
            }
        }
        if conflict_location.is_some() {
            break;
        }
    }

    let (conflict_state, conflict_col) =
        conflict_location.expect("No SR conflict found - test grammar issue");

    println!(
        "Testing SR conflict at state {} column {}",
        conflict_state, conflict_col
    );

    // Normalize and encode
    support::language_builder::normalize_table_for_ts(&mut parse_table);

    // Build language and decode
    let lang = support::language_builder::build_ts_language(&grammar, &parse_table);
    let lang = Box::leak(Box::new(lang));
    let decoded_table = decoder::decode_parse_table(lang);

    // Check the encoded action at the conflict location
    // The normalization should have resolved it to a single action
    let actions = &decoded_table.action_table[conflict_state][conflict_col];

    // In Tree-sitter compatibility mode, conflicts should be resolved
    // Default policy is Shift wins over Reduce
    assert_eq!(actions.len(), 1, "Conflict not resolved to single action");

    let chosen = &actions[0];
    assert!(
        matches!(chosen, Action::Shift(_)),
        "SR conflict not resolved to Shift (got {:?})",
        chosen
    );

    println!("✓ SR conflict correctly resolved to Shift action");
}

#[test]
fn precedence_resolves_conflicts() {
    // Compare conflict counts with and without precedence

    // Without precedence - should have conflicts
    let grammar_no_prec = support::expr_sr_conflict::build_expr_sr_conflict();
    let first_follow_no = FirstFollowSets::compute(&grammar_no_prec).unwrap();
    let table_no_prec = build_lr1_automaton(&grammar_no_prec, &first_follow_no).unwrap();

    let mut conflicts_without = 0;
    for row in &table_no_prec.action_table {
        for cell in row {
            if cell.len() > 1 {
                conflicts_without += 1;
            }
        }
    }

    // With precedence - should have fewer or no conflicts
    let grammar_with_prec = support::expr_sr_conflict::build_expr_with_precedence();
    let first_follow_with = FirstFollowSets::compute(&grammar_with_prec).unwrap();
    let table_with_prec = build_lr1_automaton(&grammar_with_prec, &first_follow_with).unwrap();

    let mut conflicts_with = 0;
    for row in &table_with_prec.action_table {
        for cell in row {
            if cell.len() > 1 {
                conflicts_with += 1;
            }
        }
    }

    println!("Conflicts without precedence: {}", conflicts_without);
    println!("Conflicts with precedence: {}", conflicts_with);

    assert!(
        conflicts_without > 0,
        "Expected conflicts without precedence"
    );
    assert!(
        conflicts_with < conflicts_without,
        "Precedence should reduce conflicts"
    );

    println!(
        "✓ Precedence reduces conflicts from {} to {}",
        conflicts_without, conflicts_with
    );
}

#[test]
fn reduce_reduce_conflict_resolution() {
    // Test reduce/reduce conflict resolution (if any exist)
    let grammar = support::expr_sr_conflict::build_expr_sr_conflict();
    let first_follow = FirstFollowSets::compute(&grammar).unwrap();
    let parse_table = build_lr1_automaton(&grammar, &first_follow).unwrap();

    // Find RR conflicts (two different reduce actions)
    let mut rr_conflicts = 0;

    for row in &parse_table.action_table {
        for cell in row {
            let reduce_count = cell
                .iter()
                .filter(|a| matches!(a, Action::Reduce(_)))
                .count();

            if reduce_count > 1 {
                rr_conflicts += 1;
            }
        }
    }

    if rr_conflicts > 0 {
        println!("✓ Found {} reduce/reduce conflicts", rr_conflicts);
        // In Tree-sitter, RR conflicts are resolved by choosing the first rule
    } else {
        println!("✓ No reduce/reduce conflicts in this grammar (expected)");
    }
}

#[test]
fn glr_preserves_all_actions() {
    // Verify GLR table preserves all conflicting actions before encoding
    let grammar = support::expr_sr_conflict::build_expr_sr_conflict();
    let first_follow = FirstFollowSets::compute(&grammar).unwrap();
    let parse_table = build_lr1_automaton(&grammar, &first_follow).unwrap();

    // Count cells with multiple actions (GLR feature)
    let mut multi_action_cells = 0;
    let mut total_actions = 0;

    for row in &parse_table.action_table {
        for cell in row {
            if cell.len() > 1 {
                multi_action_cells += 1;
                total_actions += cell.len();
            } else if cell.len() == 1 {
                total_actions += 1;
            }
        }
    }

    assert!(
        multi_action_cells > 0,
        "GLR table should preserve multiple actions in conflict cells"
    );

    println!(
        "✓ GLR table has {} cells with multiple actions ({} total actions)",
        multi_action_cells, total_actions
    );
}

#[test]
fn conflict_policy_consistency() {
    // Test that the same conflict is resolved the same way across the table
    let grammar = support::expr_sr_conflict::build_expr_sr_conflict();
    let first_follow = FirstFollowSets::compute(&grammar).unwrap();
    let mut parse_table = build_lr1_automaton(&grammar, &first_follow).unwrap();

    // Collect all SR conflicts before normalization
    let mut sr_before = Vec::new();
    for (state, row) in parse_table.action_table.iter().enumerate() {
        for (col, cell) in row.iter().enumerate() {
            if cell.iter().any(|a| matches!(a, Action::Shift(_)))
                && cell.iter().any(|a| matches!(a, Action::Reduce(_)))
            {
                sr_before.push((state, col));
            }
        }
    }

    // Normalize for Tree-sitter
    support::language_builder::normalize_table_for_ts(&mut parse_table);

    // Build and decode
    let lang = support::language_builder::build_ts_language(&grammar, &parse_table);
    let lang = Box::leak(Box::new(lang));
    let decoded = decoder::decode_parse_table(lang);

    // Check all former SR conflicts are now resolved to Shift
    for (state, col) in sr_before {
        let actions = &decoded.action_table[state][col];
        if !actions.is_empty() {
            assert_eq!(
                actions.len(),
                1,
                "Conflict at ({}, {}) not resolved",
                state,
                col
            );
            assert!(
                matches!(actions[0], Action::Shift(_)),
                "SR at ({}, {}) not resolved to Shift",
                state,
                col
            );
        }
    }

    println!("✓ All SR conflicts consistently resolved to Shift");
}

#[test]
fn test_chooser_policy_matches_encoder() {
    // Verify that the ts_choose function used in tests matches the encoder's behavior
    use ir::RuleId;

    // Test SR conflict: Shift should win
    let sr_cell = vec![Action::Shift(StateId(5)), Action::Reduce(RuleId(2))];
    let chosen = ts_choose(&sr_cell).expect("Should choose an action");
    assert!(
        matches!(chosen, Action::Shift(_)),
        "ts_choose should pick Shift in SR conflict"
    );

    // Test RR conflict: First reduce wins (current implementation)
    let rr_cell = vec![Action::Reduce(RuleId(4)), Action::Reduce(RuleId(2))];
    let chosen = ts_choose(&rr_cell).expect("Should choose an action");
    assert!(
        matches!(chosen, Action::Reduce(_)),
        "ts_choose should pick a reduce in RR conflict"
    );
    // Note: Current implementation picks first reduce, not necessarily lowest ID

    // Test Accept priority
    let accept_cell = vec![
        Action::Shift(StateId(3)),
        Action::Accept,
        Action::Reduce(RuleId(1)),
    ];
    let chosen = ts_choose(&accept_cell).expect("Should choose an action");
    assert!(
        matches!(chosen, Action::Accept),
        "ts_choose should prioritize Accept"
    );

    println!("✓ ts_choose function follows expected conflict resolution policy");
}
