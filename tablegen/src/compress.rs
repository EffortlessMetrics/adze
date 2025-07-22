use rust_sitter_glr_core::ParseTable;

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