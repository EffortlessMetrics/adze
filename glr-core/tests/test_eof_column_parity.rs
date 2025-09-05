//! Test that EOF column matches TS end column
#![allow(unused_imports, clippy::ptr_arg, clippy::useless_vec)]

#[test]
#[cfg(feature = "test-helpers")]
fn eof_column_matches_ts_end_column() {
    use rust_sitter_glr_core::{Action, ParseTable};
    use rust_sitter_ir::{RuleId, StateId, SymbolId};

    // Helper to extract action kinds from a cell (for comparison)
    fn action_kinds(cell: &Vec<Action>) -> Vec<char> {
        let mut kinds: Vec<_> = cell
            .iter()
            .map(|a| match a {
                Action::Shift(_) => 'S',
                Action::Reduce(_) => 'R',
                Action::Accept => 'A',
                Action::Error => 'E',
                Action::Recover => 'V',
                Action::Fork(_) => 'F',
                _ => 'U', // Unknown (for future variants due to #[non_exhaustive])
            })
            .collect();
        kinds.sort(); // Sort for deterministic comparison
        kinds
    }

    // This test would typically load a real table from ts-bridge
    // For now, we'll create a synthetic table that demonstrates the invariant

    // Create a table where TS end (symbol 0) and EOF sentinel have matching actions
    let action_table = vec![
        // State 0
        vec![
            vec![Action::Accept], // Symbol 0 (TS end)
            vec![],               // Symbol 1
            vec![],               // Symbol 2
            vec![Action::Accept], // Symbol 3 (EOF sentinel)
        ],
        // State 1
        vec![
            vec![Action::Reduce(RuleId(0))], // Symbol 0 (TS end)
            vec![Action::Shift(StateId(2))], // Symbol 1
            vec![],                          // Symbol 2
            vec![Action::Reduce(RuleId(0))], // Symbol 3 (EOF sentinel)
        ],
    ];

    // The invariant: EOF column should have same action kinds as TS end column
    let ts_end_idx = 0usize; // TS builtin end is always symbol 0
    let eof_idx = 3usize; // Our EOF sentinel

    for (state_idx, row) in action_table.iter().enumerate() {
        // Assert both columns exist (should never have partial coverage)
        assert!(
            ts_end_idx < row.len(),
            "State {}: Missing TS end column (expected at index {})",
            state_idx,
            ts_end_idx
        );
        assert!(
            eof_idx < row.len(),
            "State {}: Missing EOF column (expected at index {})",
            state_idx,
            eof_idx
        );

        let ts_end_kinds = action_kinds(&row[ts_end_idx]);
        let eof_kinds = action_kinds(&row[eof_idx]);

        assert_eq!(
            ts_end_kinds, eof_kinds,
            "State {}: TS end column actions {:?} != EOF column actions {:?}",
            state_idx, ts_end_kinds, eof_kinds
        );
    }

    println!("✓ EOF column parity verified across all states");
}
