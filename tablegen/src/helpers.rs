use rust_sitter_glr_core::ParseTable;
use rust_sitter_ir::{Grammar, SymbolId};

/// Collect all token column indices from a parse table
/// 
/// Returns a sorted, deduplicated list of column indices for all tokens.
/// **Note:** This function ALWAYS includes the EOF token (symbol 0) in the returned indices.
/// 
/// # Parameters
/// - `grammar`: The grammar containing token definitions
/// - `parse_table`: The parse table with symbol-to-index mappings
/// 
/// # Returns
/// A sorted vector of column indices for all tokens, including EOF
pub fn collect_token_indices(grammar: &Grammar, parse_table: &ParseTable) -> Vec<usize> {
    let mut token_indices = Vec::new();
    
    // EOF is always symbol 0 and should be in the symbol_to_index map
    if let Some(&eof_idx) = parse_table.symbol_to_index.get(&SymbolId(0)) {
        token_indices.push(eof_idx);
    } else {
        eprintln!("Warning: EOF (symbol 0) not found in symbol_to_index map");
    }
    
    // Add all grammar tokens
    for token_id in grammar.tokens.keys() {
        if let Some(&idx) = parse_table.symbol_to_index.get(token_id) {
            token_indices.push(idx);
        } else {
            eprintln!("Warning: Token {:?} not found in symbol_to_index map", token_id);
        }
    }
    
    // Sort and deduplicate for stable output
    token_indices.sort_unstable();
    token_indices.dedup();
    
    token_indices
}