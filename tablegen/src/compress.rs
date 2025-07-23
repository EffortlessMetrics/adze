use rust_sitter_glr_core::{ParseTable, Action};
use rust_sitter_ir::SymbolId;

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
pub struct CompressedActionTable {
    pub data: Vec<ActionEntry>,
}

/// Entry in the action table
pub struct ActionEntry {
    pub symbol: u16,
    pub action: Action,
}

/// Compressed goto table
pub struct CompressedGotoTable {
    pub data: Vec<CompressedGotoEntry>,
}

/// Entry in the goto table
pub struct GotoEntry {
    pub symbol: SymbolId,
    pub state: u16,
}

/// Compressed goto entry with run-length encoding
pub enum CompressedGotoEntry {
    Single(u16),
    RunLength { state: u16, count: u16 },
}

/// Table compressor for encoding actions
pub struct TableCompressor;

impl TableCompressor {
    pub fn new() -> Self {
        Self
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
        }
    }
}

impl From<CompressedParseTable> for CompressedTables {
    fn from(compressed: CompressedParseTable) -> Self {
        CompressedTables {
            action_table: CompressedActionTable { data: Vec::new() },
            goto_table: CompressedGotoTable { data: Vec::new() },
            small_table_threshold: 256,
        }
    }
}