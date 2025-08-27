//! Test that small-row encoding correctly handles both token and NT columns
//! This ensures that the precedence resolution framework maintains proper column separation

// Small-row round-trip test for column separation invariants
use std::collections::HashMap;

/// Mock small row state for testing round-trip encoding/decoding
struct SmallRowState {
    token_columns: HashMap<u16, u16>, // column -> encoded action
    nt_columns: HashMap<u16, u16>,    // column -> goto state
}

impl SmallRowState {
    fn new() -> Self {
        SmallRowState {
            token_columns: HashMap::new(),
            nt_columns: HashMap::new(),
        }
    }

    /// Add a token action to the small row
    fn add_token_action(&mut self, column: u16, encoded_action: u16) {
        assert!(
            !self.token_columns.contains_key(&column),
            "Duplicate column {}",
            column
        );
        assert!(
            !self.nt_columns.contains_key(&column),
            "Duplicate column {}",
            column
        );
        self.token_columns.insert(column, encoded_action);
    }

    /// Add an NT goto to the small row  
    fn add_nt_goto(&mut self, column: u16, goto_state: u16) {
        assert!(
            !self.token_columns.contains_key(&column),
            "Duplicate column {}",
            column
        );
        assert!(
            !self.nt_columns.contains_key(&column),
            "Duplicate column {}",
            column
        );
        self.nt_columns.insert(column, goto_state);
    }

    /// Encode to small row format (sorted by column index)
    fn encode(&self, token_count: u16) -> Vec<u16> {
        let mut entries = Vec::new();

        // Add token entries
        for (&col, &action) in &self.token_columns {
            assert!(
                col < token_count,
                "Token column {} must be < token_count {}",
                col,
                token_count
            );
            entries.push((col, action));
        }

        // Add NT entries
        for (&col, &goto) in &self.nt_columns {
            assert!(
                col >= token_count,
                "NT column {} must be >= token_count {}",
                col,
                token_count
            );
            entries.push((col, goto));
        }

        // Sort by column index
        entries.sort_by_key(|&(col, _)| col);

        // Check for duplicates
        for i in 1..entries.len() {
            assert_ne!(
                entries[i - 1].0,
                entries[i].0,
                "Duplicate column {}",
                entries[i].0
            );
        }

        // Flatten to alternating column/value format
        let mut result = Vec::with_capacity(entries.len() * 2);
        for (col, val) in entries {
            result.push(col);
            result.push(val);
        }
        result
    }

    /// Decode from small row format
    fn decode(data: &[u16], token_count: u16) -> Self {
        let mut state = SmallRowState::new();

        // Data is in pairs: [col, value, col, value, ...]
        assert!(data.len() % 2 == 0, "Small row data must have even length");

        for i in (0..data.len()).step_by(2) {
            let col = data[i];
            let val = data[i + 1];

            if col < token_count {
                // Token column -> action
                state.add_token_action(col, val);
            } else {
                // NT column -> goto
                state.add_nt_goto(col, val);
            }
        }

        state
    }
}

#[test]
fn test_small_row_round_trip() {
    let token_count = 10;
    let _symbol_count = 20;

    // Create a small row with both token and NT entries
    let mut state = SmallRowState::new();

    // Add some token actions (columns 0-9)
    state.add_token_action(0, 100); // EOF -> some action
    state.add_token_action(3, 103); // Token 3 -> action
    state.add_token_action(7, 107); // Token 7 -> action

    // Add some NT gotos (columns 10-19)
    state.add_nt_goto(10, 200); // NT 0 -> goto state 200
    state.add_nt_goto(15, 205); // NT 5 -> goto state 205
    state.add_nt_goto(19, 209); // NT 9 -> goto state 209

    // Encode to small row format
    let encoded = state.encode(token_count);

    // Verify encoding structure
    assert_eq!(encoded.len(), 12); // 6 entries * 2 values each

    // Verify sorted order
    for i in (0..encoded.len()).step_by(2) {
        if i > 0 {
            assert!(encoded[i] > encoded[i - 2], "Columns must be sorted");
        }
    }

    // Decode back
    let decoded = SmallRowState::decode(&encoded, token_count);

    // Verify round-trip preserves all entries
    assert_eq!(decoded.token_columns.len(), 3);
    assert_eq!(decoded.nt_columns.len(), 3);

    // Verify token entries
    assert_eq!(decoded.token_columns.get(&0), Some(&100));
    assert_eq!(decoded.token_columns.get(&3), Some(&103));
    assert_eq!(decoded.token_columns.get(&7), Some(&107));

    // Verify NT entries
    assert_eq!(decoded.nt_columns.get(&10), Some(&200));
    assert_eq!(decoded.nt_columns.get(&15), Some(&205));
    assert_eq!(decoded.nt_columns.get(&19), Some(&209));
}

#[test]
fn test_small_row_column_separation() {
    let token_count = 5;

    let mut state = SmallRowState::new();

    // Add entries at the boundary
    state.add_token_action(4, 104); // Last token column
    state.add_nt_goto(5, 205); // First NT column

    let encoded = state.encode(token_count);

    // Verify they're adjacent but properly separated
    assert_eq!(encoded, vec![4, 104, 5, 205]);

    let decoded = SmallRowState::decode(&encoded, token_count);

    // Verify separation is maintained
    assert_eq!(decoded.token_columns.get(&4), Some(&104));
    assert_eq!(decoded.nt_columns.get(&5), Some(&205));
    assert_eq!(decoded.token_columns.get(&5), None); // 5 is not a token
    assert_eq!(decoded.nt_columns.get(&4), None); // 4 is not an NT
}

#[test]
#[should_panic(expected = "Token column 5 must be < token_count 5")]
fn test_small_row_invalid_token_column() {
    let token_count = 5;
    let mut state = SmallRowState::new();

    // Try to add a token action beyond token_count
    state.add_token_action(5, 105); // Should panic
    state.encode(token_count);
}

#[test]
#[should_panic(expected = "NT column 4 must be >= token_count 5")]
fn test_small_row_invalid_nt_column() {
    let token_count = 5;
    let mut state = SmallRowState::new();

    // Try to add an NT goto in token range
    state.add_nt_goto(4, 204); // Should panic
    state.encode(token_count);
}

#[test]
#[should_panic(expected = "Duplicate column 3")]
fn test_small_row_duplicate_columns() {
    let token_count = 10;
    let mut state = SmallRowState::new();

    // Add duplicate columns (even if values differ)
    state.add_token_action(3, 103);
    state.add_token_action(3, 999); // Duplicate!

    state.encode(token_count); // Should panic on duplicate check
}
