use rust_sitter_glr_core::{ParseTable, Action};
use rust_sitter_ir::{SymbolId, StateId};
use std::collections::HashMap;
use crate::TableGenError;

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

/// Complete compressed tables for Tree-sitter
pub struct CompressedTables {
    pub action_table: CompressedActionTable,
    pub goto_table: CompressedGotoTable,
    pub small_table_threshold: usize,
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

impl TableCompressor {
    pub fn new() -> Self {
        Self {
            small_table_threshold: 32768, // Tree-sitter's threshold
        }
    }
    
    /// Encode an action for small tables
    pub fn encode_action_small(&self, action: &Action) -> Result<u16, String> {
        match action {
            Action::Shift(state) => {
                // Shift actions: state << 2 | 0
                Ok((state.0 as u16) << 2)
            }
            Action::Reduce(rule_id) => {
                // Reduce actions: rule_id << 2 | 1
                Ok(((rule_id.0 as u16) << 2) | 1)
            }
            Action::Accept => {
                // Accept action: special value
                Ok(0xFFFF)
            }
            Action::Error => {
                // Error action: 0
                Ok(0)
            }
            Action::Fork(_) => {
                // Fork actions need special handling
                Err("Fork actions not yet supported in small tables".to_string())
            }
        }
    }
    
    /// Compress parse tables using Tree-sitter's exact algorithms
    pub fn compress(&self, parse_table: &ParseTable) -> Result<CompressedTables, TableGenError> {
        // Determine if we should use small table optimization
        let use_small_table = parse_table.state_count < self.small_table_threshold;
        
        if use_small_table {
            self.compress_small_table(parse_table)
        } else {
            self.compress_large_table(parse_table)
        }
    }
    
    /// Compress using Tree-sitter's "small table" optimization
    fn compress_small_table(&self, parse_table: &ParseTable) -> Result<CompressedTables, TableGenError> {
        let compressed_action_table = self.compress_action_table_small(&parse_table.action_table)?;
        let compressed_goto_table = self.compress_goto_table_small(&parse_table.goto_table)?;
        
        Ok(CompressedTables {
            action_table: compressed_action_table,
            goto_table: compressed_goto_table,
            small_table_threshold: self.small_table_threshold,
        })
    }
    
    /// Compress using large table optimization
    fn compress_large_table(&self, parse_table: &ParseTable) -> Result<CompressedTables, TableGenError> {
        // For now, use the same as small table
        self.compress_small_table(parse_table)
    }
    
    /// Compress action table using Tree-sitter's small table format
    fn compress_action_table_small(&self, action_table: &[Vec<Action>]) -> Result<CompressedActionTable, TableGenError> {
        let mut entries = Vec::new();
        let mut row_offsets = Vec::new();
        let mut default_actions = Vec::new();
        
        for actions in action_table {
            // Find the most common action
            let mut action_counts: HashMap<&Action, usize> = HashMap::new();
            let mut has_shift = false;
            let mut has_accept = false;
            
            for action in actions {
                *action_counts.entry(action).or_insert(0) += 1;
                match action {
                    Action::Shift(_) => has_shift = true,
                    Action::Accept => has_accept = true,
                    _ => {}
                }
            }
            
            let most_common = action_counts
                .iter()
                .max_by_key(|(_, count)| *count)
                .map(|(action, _)| (*action).clone())
                .unwrap_or(Action::Error);
            
            let default_action = match &most_common {
                Action::Reduce(_) if !has_shift && !has_accept => most_common,
                Action::Error => Action::Error,
                _ => Action::Error,
            };
            
            default_actions.push(default_action.clone());
            row_offsets.push(entries.len() as u16);
            
            for (symbol_id, action) in actions.iter().enumerate() {
                if action == &default_action {
                    continue;
                }
                
                entries.push(CompressedActionEntry {
                    symbol: symbol_id as u16,
                    action: action.clone(),
                });
            }
        }
        
        row_offsets.push(entries.len() as u16);
        
        Ok(CompressedActionTable {
            data: entries,
            row_offsets,
            default_actions,
        })
    }
    
    /// Compress goto table  
    fn compress_goto_table_small(&self, goto_table: &[Vec<StateId>]) -> Result<CompressedGotoTable, TableGenError> {
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
                        entries.push(CompressedGotoEntry::RunLength {
                            state: last_state.unwrap(),
                            count: run_length,
                        });
                    }
                    last_state = Some(state_id.0);
                    run_length = 1;
                }
            }
            
            if run_length > 0 {
                if run_length == 1 {
                    entries.push(CompressedGotoEntry::Single(last_state.unwrap()));
                } else {
                    entries.push(CompressedGotoEntry::RunLength {
                        state: last_state.unwrap(),
                        count: run_length,
                    });
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

impl From<CompressedParseTable> for CompressedTables {
    fn from(_compressed: CompressedParseTable) -> Self {
        CompressedTables {
            action_table: CompressedActionTable { 
                data: Vec::new(),
                row_offsets: Vec::new(),
                default_actions: Vec::new(),
            },
            goto_table: CompressedGotoTable { 
                data: Vec::new(),
                row_offsets: Vec::new(),
            },
            small_table_threshold: 256,
        }
    }
}