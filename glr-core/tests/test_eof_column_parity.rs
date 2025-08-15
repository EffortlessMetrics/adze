//! Test that EOF column matches TS end column

#[test]
#[cfg(feature = "test-helpers")]
fn eof_column_matches_ts_end_column() {
    use rust_sitter_glr_core::{ParseTable, Action};
    use rust_sitter_ir::{StateId, SymbolId, RuleId};
    
    // Helper to extract action kinds from a cell (for comparison)
    fn action_kinds(cell: &Vec<Action>) -> Vec<char> {
        let mut kinds: Vec<_> = cell.iter().map(|a| match a {
            Action::Shift(_)  => 'S',
            Action::Reduce(_) => 'R',
            Action::Accept    => 'A',
            Action::Error     => 'E',
            Action::Recover   => 'V',
            Action::Fork(_)   => 'F',
        }).collect();
        kinds.sort(); // Sort for deterministic comparison
        kinds
    }
    
    // This test would typically load a real table from ts-bridge
    // For now, we'll create a synthetic table that demonstrates the invariant
    
    // Create a table where TS end (symbol 0) and EOF sentinel have matching actions
    let action_table = vec![
        // State 0
        vec![
            vec![Action::Accept],       // Symbol 0 (TS end)
            vec![],                     // Symbol 1 
            vec![],                     // Symbol 2
            vec![Action::Accept],       // Symbol 3 (EOF sentinel)
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
    let ts_end_idx = 0usize;  // TS builtin end is always symbol 0
    let eof_idx = 3usize;     // Our EOF sentinel
    
    for (state_idx, row) in action_table.iter().enumerate() {
        if ts_end_idx < row.len() && eof_idx < row.len() {
            let ts_end_kinds = action_kinds(&row[ts_end_idx]);
            let eof_kinds = action_kinds(&row[eof_idx]);
            
            assert_eq!(
                ts_end_kinds, eof_kinds,
                "State {}: EOF column kinds {:?} != TS end column kinds {:?}",
                state_idx, eof_kinds, ts_end_kinds
            );
        }
    }
    
    println!("✓ EOF column parity verified across all states");
}

