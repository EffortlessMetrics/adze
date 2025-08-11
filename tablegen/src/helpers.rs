use rust_sitter_glr_core::{Action, ParseTable};
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
            eprintln!(
                "Warning: Token {:?} not found in symbol_to_index map",
                token_id
            );
        }
    }

    // Sort and deduplicate for stable output
    token_indices.sort_unstable();
    token_indices.dedup();

    token_indices
}

/// Check if state 0 has an Accept or Reduce action in the EOF column.
/// This is used to detect nullable start symbols in GLR grammars.
///
/// Returns true if state 0 can accept or reduce on EOF, indicating
/// that the start symbol can be empty (nullable).
pub(crate) fn eof_accepts_or_reduces(parse_table: &ParseTable) -> bool {
    // Get EOF column index
    let eof_idx = match parse_table.symbol_to_index.get(&SymbolId(0)) {
        Some(&idx) => idx,
        None => return false, // No EOF column means no nullable start
    };

    // Check state 0 (initial state)
    if parse_table.action_table.is_empty() {
        return false;
    }

    let state0 = &parse_table.action_table[0];

    // Check if EOF column exists and has Accept or Reduce actions
    state0.get(eof_idx).map_or(false, |cell| {
        cell.iter().any(|action| matches!(action, Action::Accept | Action::Reduce(_)))
    })
}
