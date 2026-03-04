//! Parse-table analysis helpers used by table compression and language generation.

#![forbid(unsafe_op_in_unsafe_fn)]

use adze_glr_core::{Action, ParseTable};
use adze_ir::Grammar;

#[cfg(not(debug_assertions))]
macro_rules! debug_trace {
    ($($arg:tt)*) => {};
}

#[cfg(debug_assertions)]
macro_rules! debug_trace {
    ($($arg:tt)*) => {
        if std::env::var("RUST_LOG")
            .ok()
            .unwrap_or_default()
            .contains("debug")
        {
            eprintln!($($arg)*);
        }
    };
}

/// Collect all token column indices from a parse table.
///
/// Returns a sorted, deduplicated list of column indices for all tokens.
/// The EOF column is always included when present in `symbol_to_index`.
#[must_use]
pub fn collect_token_indices(grammar: &Grammar, parse_table: &ParseTable) -> Vec<usize> {
    let mut token_indices = Vec::new();

    if let Some(&eof_idx) = parse_table.symbol_to_index.get(&parse_table.eof_symbol) {
        token_indices.push(eof_idx);
    } else {
        debug_trace!(
            "Warning: EOF (symbol {}) not found in symbol_to_index map",
            parse_table.eof_symbol.0
        );
    }

    for token_id in grammar.tokens.keys() {
        if let Some(&idx) = parse_table.symbol_to_index.get(token_id) {
            token_indices.push(idx);
        } else {
            debug_trace!(
                "Warning: Token {:?} not found in symbol_to_index map",
                token_id
            );
        }
    }

    token_indices.sort_unstable();
    token_indices.dedup();
    token_indices
}

/// Returns `true` if state 0 has an `Accept` or `Reduce` action in the EOF column.
#[must_use]
pub fn eof_accepts_or_reduces(parse_table: &ParseTable) -> bool {
    let eof_idx = match parse_table.symbol_to_index.get(&parse_table.eof_symbol) {
        Some(&idx) => idx,
        None => return false,
    };

    if parse_table.action_table.is_empty() {
        return false;
    }

    parse_table.action_table[0]
        .get(eof_idx)
        .is_some_and(|cell| {
            cell.iter()
                .any(|action| matches!(action, Action::Accept | Action::Reduce(_)))
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use adze_glr_core::ActionCell;
    use adze_ir::{RuleId, SymbolId, Token, TokenPattern};

    #[test]
    fn collect_token_indices_sorts_dedups_and_includes_eof() {
        let mut grammar = Grammar::default();
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

        let mut parse_table = ParseTable::default();
        parse_table.eof_symbol = SymbolId(99);
        parse_table.symbol_to_index.insert(SymbolId(99), 4);
        parse_table.symbol_to_index.insert(SymbolId(1), 3);
        parse_table.symbol_to_index.insert(SymbolId(2), 1);
        parse_table.symbol_to_index.insert(SymbolId(3), 3);

        assert_eq!(collect_token_indices(&grammar, &parse_table), vec![1, 3, 4]);
    }

    #[test]
    fn eof_accepts_or_reduces_checks_state_0_eof_cell() {
        let mut parse_table = ParseTable::default();
        parse_table.eof_symbol = SymbolId(10);
        parse_table.symbol_to_index.insert(SymbolId(10), 1);
        parse_table.action_table = vec![vec![ActionCell::new(), vec![Action::Reduce(RuleId(0))]]];

        assert!(eof_accepts_or_reduces(&parse_table));

        parse_table.action_table[0][1] = vec![Action::Shift(adze_ir::StateId(1))];
        assert!(!eof_accepts_or_reduces(&parse_table));
    }
}
