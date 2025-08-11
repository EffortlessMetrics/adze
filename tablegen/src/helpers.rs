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
///
/// # Examples
///
/// ```ignore
/// use rust_sitter_ir::Grammar;
/// use rust_sitter_glr_core::ParseTable;
/// use rust_sitter_tablegen::collect_token_indices;
/// 
/// let grammar = Grammar::new("my_grammar".to_string());
/// // assume parse_table is built from grammar processing
/// let token_indices = collect_token_indices(&grammar, &parse_table);
/// // token_indices will include the EOF column and all grammar tokens
/// // EOF column is always included (but not necessarily at index 0)
/// ```
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

/// Returns `true` if state 0 has an **Accept** or **Reduce** action in the EOF column.
///
/// This is a convenience for deriving `start_can_be_empty` from a `ParseTable`.
/// It only inspects the EOF cell of state 0.
///
/// This is used to detect nullable start symbols in GLR grammars.
/// Returns true if state 0 can accept or reduce on EOF, indicating
/// that the start symbol can be empty (nullable).
///
/// # Examples
///
/// ```ignore
/// use rust_sitter_glr_core::ParseTable;
/// use rust_sitter_tablegen::eof_accepts_or_reduces;
/// 
/// // assume parse_table is built from grammar processing
/// let start_can_be_empty = eof_accepts_or_reduces(&parse_table);
/// if start_can_be_empty {
///     println!("Grammar has a nullable start symbol");
/// }
/// ```
pub fn eof_accepts_or_reduces(parse_table: &ParseTable) -> bool {
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
        cell.iter()
            .any(|action| matches!(action, Action::Accept | Action::Reduce(_)))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_sitter_glr_core::ParseTable;
    use rust_sitter_ir::{Grammar, SymbolId};
    use std::collections::BTreeMap;

    #[test]
    fn test_collect_token_indices() {
        let grammar = Grammar::default();
        let mut parse_table = ParseTable {
            action_table: vec![],
            goto_table: vec![],
            symbol_metadata: vec![],
            state_count: 0,
            symbol_count: 5,
            symbol_to_index: BTreeMap::new(),
            external_scanner_states: vec![],
        };

        // Add some symbols to the mapping
        parse_table.symbol_to_index.insert(SymbolId(0), 0); // EOF - always present
        parse_table.symbol_to_index.insert(SymbolId(1), 3); // Some token at column 3
        parse_table.symbol_to_index.insert(SymbolId(2), 1); // Another token at column 1
        parse_table.symbol_to_index.insert(SymbolId(3), 3); // Duplicate column (should be deduped)
        parse_table.symbol_to_index.insert(SymbolId(4), 2); // Token at column 2

        let indices = collect_token_indices(&grammar, &parse_table);

        // Should be sorted, deduped, and always contain EOF (0)
        assert_eq!(indices, vec![0, 1, 2, 3]);

        // Verify EOF is always included
        assert!(
            indices.contains(&0),
            "Token indices must always include EOF column (0)"
        );

        // Verify sorted
        let mut sorted = indices.clone();
        sorted.sort_unstable();
        assert_eq!(indices, sorted, "Token indices must be sorted");

        // Verify deduped
        let mut deduped = indices.clone();
        deduped.dedup();
        assert_eq!(indices, deduped, "Token indices must be deduplicated");
    }

    #[test]
    fn test_collect_token_indices_empty() {
        let grammar = Grammar::default();
        let mut parse_table = ParseTable {
            action_table: vec![],
            goto_table: vec![],
            symbol_metadata: vec![],
            state_count: 0,
            symbol_count: 1,
            symbol_to_index: BTreeMap::new(),
            external_scanner_states: vec![],
        };

        // Only EOF in the mapping
        parse_table.symbol_to_index.insert(SymbolId(0), 0);

        let indices = collect_token_indices(&grammar, &parse_table);

        // Should still contain EOF
        assert_eq!(indices, vec![0]);
    }
}
