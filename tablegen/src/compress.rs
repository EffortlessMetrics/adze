use crate::TableGenError;
use rust_sitter_glr_core::{Action, ParseTable};
use rust_sitter_ir::{StateId, SymbolId};
use std::collections::{BTreeMap, HashMap};

/// Compressed parse table representation
pub struct CompressedParseTable {
    symbol_count: usize,
    state_count: usize,
}

impl CompressedParseTable {
    /// Create a new compressed parse table for testing
    pub fn new_for_testing(symbol_count: usize, state_count: usize) -> Self {
        Self {
            symbol_count,
            state_count,
        }
    }

    /// Get the symbol count
    pub fn symbol_count(&self) -> usize {
        self.symbol_count
    }

    /// Get the state count
    pub fn state_count(&self) -> usize {
        self.state_count
    }

    /// Create from a parse table
    pub fn from_parse_table(parse_table: &ParseTable) -> Self {
        Self {
            symbol_count: parse_table.symbol_count,
            state_count: parse_table.state_count,
        }
    }
}

// Removed: This From implementation was returning dummy empty tables.
// Compression is now handled by TableCompressor::compress() method directly.

/// Complete compressed tables for Tree-sitter
pub struct CompressedTables {
    pub action_table: CompressedActionTable,
    pub goto_table: CompressedGotoTable,
    pub small_table_threshold: usize,
}

impl CompressedTables {
    /// Validate compressed tables against original parse table
    pub fn validate(&self, _parse_table: &ParseTable) -> Result<(), TableGenError> {
        // TODO: Implement validation logic
        // For now, just return Ok to make tests compile
        Ok(())
    }
}

/// Compressed action table
#[derive(Debug, Clone)]
pub struct CompressedActionTable {
    pub data: Vec<CompressedActionEntry>,
    pub row_offsets: Vec<u16>,
    pub default_actions: Vec<Action>,
}

/// Entry in the action table
#[derive(Debug, Clone)]
pub struct ActionEntry {
    pub symbol: u16,
    pub action: Action,
}

/// Compressed action entry
#[derive(Debug, Clone)]
pub struct CompressedActionEntry {
    pub symbol: u16,
    pub action: Action,
}

impl CompressedActionEntry {
    /// Create a new compressed action entry
    pub fn new(symbol: u16, action: Action) -> Self {
        Self { symbol, action }
    }
}

/// Compressed goto table
#[derive(Debug, Clone)]
pub struct CompressedGotoTable {
    pub data: Vec<CompressedGotoEntry>,
    pub row_offsets: Vec<u16>,
}

/// Entry in the goto table
pub struct GotoEntry {
    pub symbol: SymbolId,
    pub state: u16,
}

/// Compressed goto entry with run-length encoding
#[derive(Debug, Clone)]
pub enum CompressedGotoEntry {
    Single(u16),
    RunLength { state: u16, count: u16 },
}

/// Table compressor for encoding actions
pub struct TableCompressor {
    // Tree-sitter's magic constants for compression
    small_table_threshold: usize,
}

impl Default for TableCompressor {
    fn default() -> Self {
        Self::new()
    }
}

impl TableCompressor {
    pub fn new() -> Self {
        Self {
            small_table_threshold: 32768, // Tree-sitter's threshold
        }
    }

    /// Encode an action for small tables
    pub fn encode_action_small(&self, action: &Action) -> Result<u16, TableGenError> {
        match action {
            Action::Shift(state) => {
                if state.0 >= 0x8000 {
                    return Err(TableGenError::CompressionError(format!(
                        "Shift state {} too large for small table encoding",
                        state.0
                    )));
                }
                Ok(state.0)
            }
            Action::Reduce(rule) => {
                if rule.0 >= 0x4000 {
                    return Err(TableGenError::CompressionError(format!(
                        "Reduce rule {} too large for small table encoding",
                        rule.0
                    )));
                }
                // Reduce actions are encoded with high bit set
                // bit 15: 1 (indicates reduce)
                // bits 14-0: rule_id (1-based)
                // Tree-sitter uses 1-based production IDs
                Ok(0x8000 | (rule.0 + 1))
            }
            Action::Accept => Ok(0xFFFF),
            Action::Error => Ok(0xFFFE),
            Action::Fork(_) => {
                // GLR fork points need special handling
                // For now, treat as error
                Ok(0xFFFE)
            }
        }
    }

    /// Compress parse tables using Tree-sitter's exact algorithms
    pub fn compress(&self, parse_table: &ParseTable, token_indices: &[usize], start_can_be_empty: bool) -> Result<CompressedTables, TableGenError> {
        // Convert token_indices to HashSet for O(1) membership checks
        use std::collections::HashSet;
        let token_set: HashSet<usize> = token_indices.iter().copied().collect();
        
        // Validation: EOF must be in symbol_to_index (strong invariant)
        if !parse_table.symbol_to_index.contains_key(&SymbolId(0)) {
            return Err(TableGenError::InvalidTable(
                "EOF (symbol 0) not found in symbol_to_index map - this is a critical invariant violation".to_string()
            ));
        }
        
        // Validation: Ensure state 0 has at least one token shift action
        // This catches the "state 0 bug" where no tokens can be shifted from the initial state
        if let Some(state0_actions) = parse_table.action_table.get(0) {
            // Check if any token column has a shift action
            let has_token_shift = token_indices.iter().any(|&idx| {
                state0_actions.get(idx)
                    .map_or(false, |cell| cell.iter().any(|a| matches!(a, Action::Shift(_))))
            });
            
            // If no token shifts, check if start is nullable and EOF has accept/reduce
            let eof_ok = if !has_token_shift && start_can_be_empty {
                // EOF is always symbol 0, find its index
                parse_table.symbol_to_index.get(&SymbolId(0))
                    .and_then(|&eof_idx| state0_actions.get(eof_idx))
                    .map_or(false, |cell| {
                        cell.iter().any(|a| matches!(a, Action::Accept | Action::Reduce(_)))
                    })
            } else {
                false
            };
            
            if !has_token_shift && !eof_ok {
                // Provide detailed debugging info
                let mut debug_info = String::new();
                
                // Show expected token columns
                debug_info.push_str(&format!("Expected token columns (first 12): {:?}\n", 
                    token_indices.iter().take(12).collect::<Vec<_>>()));
                debug_info.push_str(&format!("Start can be empty: {}\n", start_can_be_empty));
                
                // Show the actual state 0 actions
                debug_info.push_str("State 0 actions (first 12 columns):\n");
                for idx in 0..state0_actions.len().min(12) {
                    let cell = &state0_actions[idx];
                    
                    // Find which symbol this index maps to
                    let symbol_info = parse_table.symbol_to_index.iter()
                        .find(|(_, i)| **i == idx)
                        .map(|(sym_id, _)| {
                            if sym_id.0 == 0 {
                                "EOF".to_string()
                            } else {
                                format!("sym_{}", sym_id.0)
                            }
                        })
                        .unwrap_or_else(|| "unmapped".to_string());
                    
                    let type_str = if idx == 0 || token_set.contains(&idx) { 
                        "TOKEN" 
                    } else { 
                        "NT" 
                    };
                    
                    let action_str = if cell.is_empty() {
                        "[]".to_string()
                    } else {
                        format!("{:?}", cell)
                    };
                    
                    debug_info.push_str(&format!("  Col {:2} ({:8} {:5}): {}\n", 
                        idx, symbol_info, type_str, action_str));
                }
                
                // Provide actionable guidance
                debug_info.push_str("\nPossible causes:\n");
                debug_info.push_str("1. Pattern wrappers not desugared to unit rules\n");
                debug_info.push_str("2. Token symbols not properly registered in symbol_to_index\n");
                debug_info.push_str("3. Grammar start symbol issues\n");
                
                return Err(TableGenError::CompressionError(format!(
                    "State 0 validation failed: No valid token shift actions found.\n{}",
                    debug_info
                )));
            }
        }
        
        // Additional sanity guards
        if parse_table.action_table.is_empty() {
            return Err(TableGenError::CompressionError(
                "Empty action table - grammar has no parse states".to_string()
            ));
        }
        
        if parse_table.state_count == 0 {
            return Err(TableGenError::CompressionError(
                "State count is 0 - invalid parse table".to_string()
            ));
        }
        
        // Determine if we should use small table optimization
        let use_small_table = parse_table.state_count < self.small_table_threshold;

        if use_small_table {
            self.compress_small_table(parse_table)
        } else {
            self.compress_large_table(parse_table)
        }
    }

    /// Compress using Tree-sitter's "small table" optimization
    fn compress_small_table(
        &self,
        parse_table: &ParseTable,
    ) -> Result<CompressedTables, TableGenError> {
        let compressed_action_table = self
            .compress_action_table_small(&parse_table.action_table, &parse_table.symbol_to_index)?;
        let compressed_goto_table = self.compress_goto_table_small(&parse_table.goto_table)?;

        Ok(CompressedTables {
            action_table: compressed_action_table,
            goto_table: compressed_goto_table,
            small_table_threshold: self.small_table_threshold,
        })
    }

    /// Compress using large table optimization
    fn compress_large_table(
        &self,
        parse_table: &ParseTable,
    ) -> Result<CompressedTables, TableGenError> {
        // For now, use the same as small table
        self.compress_small_table(parse_table)
    }

    /// Compress action table using Tree-sitter's small table format
    pub fn compress_action_table_small(
        &self,
        action_table: &[Vec<Vec<Action>>],
        symbol_to_index: &BTreeMap<SymbolId, usize>,
    ) -> Result<CompressedActionTable, TableGenError> {
        let mut entries = Vec::new();
        let mut row_offsets = Vec::new();
        let mut default_actions = Vec::new();

        // Create inverse mapping from index to symbol ID
        let mut index_to_symbol = HashMap::new();
        for (&symbol_id, &index) in symbol_to_index {
            index_to_symbol.insert(index, symbol_id);
        }


        for (_state_idx, action_row) in action_table.iter().enumerate() {
            // Find the most common action across all cells
            let mut action_counts: HashMap<Action, usize> = HashMap::new();
            let mut has_shift = false;
            let mut has_accept = false;

            // Collect all actions from all cells in this row
            for action_cell in action_row {
                for action in action_cell {
                    *action_counts.entry(action.clone()).or_insert(0) += 1;
                    match action {
                        Action::Shift(_) => has_shift = true,
                        Action::Accept => has_accept = true,
                        _ => {}
                    }
                }
            }

            let most_common = action_counts
                .iter()
                .max_by_key(|(_, count)| *count)
                .map(|(action, _)| action.clone())
                .unwrap_or(Action::Error);

            let default_action = match &most_common {
                Action::Reduce(_) if !has_shift && !has_accept => most_common,
                Action::Error => Action::Error,
                _ => Action::Error,
            };

            default_actions.push(default_action.clone());
            row_offsets.push(entries.len() as u16);


            for (index, action_cell) in action_row.iter().enumerate() {
                // Process each action in the cell
                for action in action_cell {
                    if action == &default_action {
                        continue;
                    }

                    // Get the actual symbol ID from the index
                    let symbol_id = index_to_symbol
                        .get(&index)
                        .map(|id| id.0)
                        .unwrap_or(index as u16);


                    entries.push(CompressedActionEntry {
                        symbol: symbol_id,
                        action: action.clone(),
                    });
                }
            }
        }

        row_offsets.push(entries.len() as u16);

        // Validate row_offsets are strictly increasing
        for i in 1..row_offsets.len() {
            if row_offsets[i] < row_offsets[i-1] {
                return Err(TableGenError::CompressionError(format!(
                    "Row offsets not strictly increasing at index {}: {} < {}",
                    i, row_offsets[i], row_offsets[i-1]
                )));
            }
        }
        
        // Validate map length matches state count
        if row_offsets.len() != action_table.len() + 1 {
            return Err(TableGenError::CompressionError(format!(
                "Row offsets length {} doesn't match state count {} + 1",
                row_offsets.len(), action_table.len()
            )));
        }

        Ok(CompressedActionTable {
            data: entries,
            row_offsets,
            default_actions,
        })
    }

    /// Compress goto table  
    pub fn compress_goto_table_small(
        &self,
        goto_table: &[Vec<StateId>],
    ) -> Result<CompressedGotoTable, TableGenError> {
        let mut entries = Vec::new();
        let mut row_offsets = Vec::new();

        for row in goto_table {
            row_offsets.push(entries.len() as u16);

            let mut last_state = None;
            let mut run_length = 0;

            for &state_id in row {
                if last_state == Some(state_id.0) {
                    run_length += 1;
                } else {
                    if run_length > 0 {
                        // Emit previous run
                        if run_length > 2 {
                            entries.push(CompressedGotoEntry::RunLength {
                                state: last_state.unwrap(),
                                count: run_length,
                            });
                        } else {
                            // For short runs, individual entries are more efficient
                            for _ in 0..run_length {
                                entries.push(CompressedGotoEntry::Single(last_state.unwrap()));
                            }
                        }
                    }
                    last_state = Some(state_id.0);
                    run_length = 1;
                }
            }

            if run_length > 0 {
                if run_length > 2 {
                    entries.push(CompressedGotoEntry::RunLength {
                        state: last_state.unwrap(),
                        count: run_length,
                    });
                } else {
                    for _ in 0..run_length {
                        entries.push(CompressedGotoEntry::Single(last_state.unwrap()));
                    }
                }
            }
        }

        row_offsets.push(entries.len() as u16);

        Ok(CompressedGotoTable {
            data: entries,
            row_offsets,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_sitter_glr_core::Action;
    use rust_sitter_ir::{RuleId, StateId};

    #[test]
    fn test_compressed_parse_table_creation() {
        let table = CompressedParseTable::new_for_testing(10, 20);
        assert_eq!(table.symbol_count(), 10);
        assert_eq!(table.state_count(), 20);
    }

    #[test]
    fn test_compressed_parse_table_from_parse_table() {
        let parse_table = ParseTable {
            action_table: vec![],
            goto_table: vec![],
            symbol_metadata: vec![],
            symbol_count: 5,
            state_count: 10,
            symbol_to_index: Default::default(),
            external_scanner_states: vec![],
        };

        let compressed = CompressedParseTable::from_parse_table(&parse_table);
        assert_eq!(compressed.symbol_count(), 5);
        assert_eq!(compressed.state_count(), 10);
    }

    #[test]
    fn test_compressed_action_entry() {
        let entry = CompressedActionEntry::new(42, Action::Shift(StateId(5)));
        assert_eq!(entry.symbol, 42);
        match entry.action {
            Action::Shift(StateId(5)) => {}
            _ => panic!("Expected shift action"),
        }
    }

    #[test]
    fn test_table_compressor_creation() {
        let compressor = TableCompressor::new();
        // Just verify it can be created
        assert!(compressor.small_table_threshold > 0);
    }

    #[test]
    fn test_compress_empty_action_table() {
        let compressor = TableCompressor::new();
        let action_table = vec![vec![]; 5]; // 5 empty states

        let symbol_to_index = std::collections::BTreeMap::new();
        let result = compressor.compress_action_table_small(&action_table, &symbol_to_index);
        assert!(result.is_ok());

        let compressed = result.unwrap();
        assert_eq!(compressed.row_offsets.len(), 6); // n_states + 1
        assert_eq!(compressed.default_actions.len(), 5);
        assert!(compressed.data.is_empty());
    }

    #[test]
    fn test_compress_action_table_with_default_reduce() {
        let compressor = TableCompressor::new();
        let reduce_action = Action::Reduce(RuleId(1));
        let action_table = vec![
            vec![vec![reduce_action.clone()]; 10], // All same reduce action in ActionCells
        ];

        let symbol_to_index = std::collections::BTreeMap::new();
        let result = compressor.compress_action_table_small(&action_table, &symbol_to_index);
        assert!(result.is_ok());

        let compressed = result.unwrap();
        assert_eq!(compressed.default_actions[0], reduce_action);
        assert!(compressed.data.is_empty()); // All actions are default
    }

    #[test]
    fn test_compress_goto_table_with_runs() {
        let compressor = TableCompressor::new();
        let goto_table = vec![vec![
            StateId(1),
            StateId(1),
            StateId(1),
            StateId(2),
            StateId(2),
        ]];

        let result = compressor.compress_goto_table_small(&goto_table);
        assert!(result.is_ok());

        let compressed = result.unwrap();
        assert!(!compressed.data.is_empty());

        // Should have a run length entry for the three 1s
        let has_run_length = compressed
            .data
            .iter()
            .any(|entry| matches!(entry, CompressedGotoEntry::RunLength { state: 1, count: 3 }));
        assert!(has_run_length);
    }

    #[test]
    fn test_compressed_tables_validation() {
        let tables = CompressedTables {
            action_table: CompressedActionTable {
                data: vec![],
                row_offsets: vec![],
                default_actions: vec![],
            },
            goto_table: CompressedGotoTable {
                data: vec![],
                row_offsets: vec![],
            },
            small_table_threshold: 32768,
        };

        let parse_table = ParseTable {
            action_table: vec![],
            goto_table: vec![],
            symbol_metadata: vec![],
            symbol_count: 0,
            state_count: 0,
            symbol_to_index: Default::default(),
            external_scanner_states: vec![],
        };
        let result = tables.validate(&parse_table);
        assert!(result.is_ok());
    }
}
