use adze_glr_core::{Action, ParseTable};
use adze_ir::Grammar;

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
/// ```
/// use adze_ir::builder::GrammarBuilder;
/// use adze_glr_core::{FirstFollowSets, build_lr1_automaton};
/// use adze_tablegen::collect_token_indices;
///
/// let grammar = GrammarBuilder::new("ex")
///     .token("X", "x")
///     .rule("s", vec!["X"])
///     .start("s")
///     .build();
///
/// let ff = FirstFollowSets::compute(&grammar).unwrap();
/// let pt = build_lr1_automaton(&grammar, &ff).unwrap();
/// let indices = collect_token_indices(&grammar, &pt);
/// // indices always contains the EOF column and every grammar token
/// assert!(!indices.is_empty());
/// ```
#[must_use]
pub fn collect_token_indices(grammar: &Grammar, parse_table: &ParseTable) -> Vec<usize> {
    let mut token_indices = Vec::new();

    // EOF symbol should be in the symbol_to_index map
    // Use parse_table.eof_symbol instead of hardcoded SymbolId(0) since EOF symbol
    // is computed as max_symbol + 1 in build_lr1_automaton
    if let Some(&eof_idx) = parse_table.symbol_to_index.get(&parse_table.eof_symbol) {
        token_indices.push(eof_idx);
    } else {
        eprintln!(
            "Warning: EOF (symbol {}) not found in symbol_to_index map",
            parse_table.eof_symbol.0
        );
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
/// ```
/// use adze_ir::builder::GrammarBuilder;
/// use adze_glr_core::{FirstFollowSets, build_lr1_automaton};
/// use adze_tablegen::eof_accepts_or_reduces;
///
/// let grammar = GrammarBuilder::new("ex")
///     .token("X", "x")
///     .rule("s", vec!["X"])
///     .start("s")
///     .build();
///
/// let ff = FirstFollowSets::compute(&grammar).unwrap();
/// let pt = build_lr1_automaton(&grammar, &ff).unwrap();
/// // A grammar that requires at least one token is not nullable
/// assert!(!eof_accepts_or_reduces(&pt));
/// ```
#[must_use]
pub fn eof_accepts_or_reduces(parse_table: &ParseTable) -> bool {
    // Get EOF column index using the actual eof_symbol from the parse table
    let eof_idx = match parse_table.symbol_to_index.get(&parse_table.eof_symbol) {
        Some(&idx) => idx,
        None => return false, // No EOF column means no nullable start
    };

    // Check state 0 (initial state)
    if parse_table.action_table.is_empty() {
        return false;
    }

    let state0 = &parse_table.action_table[0];

    // Check if EOF column exists and has Accept or Reduce actions
    state0.get(eof_idx).is_some_and(|cell| {
        cell.iter()
            .any(|action| matches!(action, Action::Accept | Action::Reduce(_)))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use adze_ir::{Grammar, SymbolId};

    #[test]
    fn test_collect_token_indices() {
        let mut grammar = Grammar::default();
        let mut parse_table = crate::empty_table!(states: 1, terms: 4, nonterms: 0);
        // empty_table puts EOF at column 5 (1 + 4 terms) with eof_symbol = SymbolId(5)
        let eof_col = 5;
        let eof_symbol = parse_table.eof_symbol; // Use the actual eof_symbol from the table

        // Add tokens to the grammar
        use adze_ir::{Token, TokenPattern};
        grammar.tokens.insert(
            SymbolId(1),
            Token {
                name: "token1".to_string(),
                pattern: TokenPattern::String("a".to_string()),
                fragile: false,
            },
        );
        grammar.tokens.insert(
            SymbolId(2),
            Token {
                name: "token2".to_string(),
                pattern: TokenPattern::String("b".to_string()),
                fragile: false,
            },
        );
        grammar.tokens.insert(
            SymbolId(3),
            Token {
                name: "token3".to_string(),
                pattern: TokenPattern::String("c".to_string()),
                fragile: false,
            },
        );
        grammar.tokens.insert(
            SymbolId(4),
            Token {
                name: "token4".to_string(),
                pattern: TokenPattern::String("d".to_string()),
                fragile: false,
            },
        );

        // Replace the default mapping with our test values
        // Use the actual eof_symbol from the parse table
        parse_table.symbol_to_index.clear();
        parse_table.symbol_to_index.insert(eof_symbol, eof_col); // EOF at its standard position
        parse_table.symbol_to_index.insert(SymbolId(1), 3); // Some token at column 3
        parse_table.symbol_to_index.insert(SymbolId(2), 1); // Another token at column 1
        parse_table.symbol_to_index.insert(SymbolId(3), 3); // Duplicate column (should be deduped)
        parse_table.symbol_count = 7; // Adjusted for new layout
        parse_table.symbol_to_index.insert(SymbolId(4), 2); // Token at column 2

        let indices = collect_token_indices(&grammar, &parse_table);

        // Should be sorted, deduped, and always contain EOF
        assert_eq!(indices, vec![1, 2, 3, eof_col]);

        // Verify EOF is always included
        assert!(
            indices.contains(&eof_col),
            "Token indices must always include EOF column ({})",
            eof_col
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
        let mut parse_table = crate::empty_table!(states: 1, terms: 0, nonterms: 0);
        // empty_table puts EOF at column 1 (1 + 0 terms) with eof_symbol = SymbolId(1)
        let eof_symbol = parse_table.eof_symbol;

        // Replace the default mapping with just EOF using the actual eof_symbol
        parse_table.symbol_to_index.clear();
        parse_table.symbol_to_index.insert(eof_symbol, 0);
        parse_table.symbol_count = 1;

        let indices = collect_token_indices(&grammar, &parse_table);

        // Should still contain EOF
        assert_eq!(indices, vec![0]);
    }
}
