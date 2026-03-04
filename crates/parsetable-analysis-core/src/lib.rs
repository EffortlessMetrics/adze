//! Shared parse-table analysis helpers used during generation and compression.

#![forbid(unsafe_op_in_unsafe_fn)]
#![deny(missing_docs)]
#![cfg_attr(feature = "strict_api", deny(unreachable_pub))]
#![cfg_attr(not(feature = "strict_api"), warn(unreachable_pub))]
#![cfg_attr(feature = "strict_docs", deny(missing_docs))]
#![cfg_attr(not(feature = "strict_docs"), allow(missing_docs))]

use adze_glr_core::{Action, ParseTable};
use adze_ir::Grammar;

/// Collect all token column indices from a parse table.
///
/// Returns a sorted, deduplicated list of column indices for all tokens.
/// The EOF token is always included when its symbol index exists in the table.
#[must_use]
pub fn collect_token_indices(grammar: &Grammar, parse_table: &ParseTable) -> Vec<usize> {
    let mut token_indices = Vec::new();

    if let Some(&eof_idx) = parse_table.symbol_to_index.get(&parse_table.eof_symbol) {
        token_indices.push(eof_idx);
    }

    for token_id in grammar.tokens.keys() {
        if let Some(&idx) = parse_table.symbol_to_index.get(token_id) {
            token_indices.push(idx);
        }
    }

    token_indices.sort_unstable();
    token_indices.dedup();
    token_indices
}

/// Returns `true` if state 0 has an `Accept` or `Reduce` action in the EOF column.
///
/// This is a convenience for deriving whether the start symbol can be empty from a parse table.
#[must_use]
pub fn eof_accepts_or_reduces(parse_table: &ParseTable) -> bool {
    let eof_idx = match parse_table.symbol_to_index.get(&parse_table.eof_symbol) {
        Some(&idx) => idx,
        None => return false,
    };

    let Some(state0) = parse_table.action_table.first() else {
        return false;
    };

    state0.get(eof_idx).is_some_and(|cell| {
        cell.iter()
            .any(|action| matches!(action, Action::Accept | Action::Reduce(_)))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use adze_glr_core::{ActionCell, RuleId, StateId};
    use adze_ir::{SymbolId, Token, TokenPattern};

    #[test]
    fn collect_token_indices_includes_eof_sorted_deduped() {
        let mut grammar = Grammar::default();
        grammar.tokens.insert(
            SymbolId(1),
            Token {
                name: "a".to_string(),
                pattern: TokenPattern::String("a".to_string()),
                fragile: false,
            },
        );
        grammar.tokens.insert(
            SymbolId(2),
            Token {
                name: "b".to_string(),
                pattern: TokenPattern::String("b".to_string()),
                fragile: false,
            },
        );

        let mut parse_table = ParseTable::default();
        parse_table.eof_symbol = SymbolId(9);
        parse_table.symbol_to_index.insert(SymbolId(9), 5);
        parse_table.symbol_to_index.insert(SymbolId(1), 3);
        parse_table.symbol_to_index.insert(SymbolId(2), 3);

        assert_eq!(collect_token_indices(&grammar, &parse_table), vec![3, 5]);
    }

    #[test]
    fn eof_accepts_or_reduces_true_for_accept_or_reduce() {
        let mut parse_table = ParseTable::default();
        parse_table.eof_symbol = SymbolId(7);
        parse_table.symbol_to_index.insert(SymbolId(7), 1);

        let mut state = vec![ActionCell::default(), ActionCell::default()];
        state[1] = vec![Action::Accept];
        parse_table.action_table = vec![state.clone()];
        assert!(eof_accepts_or_reduces(&parse_table));

        state[1] = vec![Action::Reduce(RuleId(0))];
        parse_table.action_table = vec![state.clone()];
        assert!(eof_accepts_or_reduces(&parse_table));

        state[1] = vec![Action::Shift(StateId(2))];
        parse_table.action_table = vec![state];
        assert!(!eof_accepts_or_reduces(&parse_table));
    }
}
