/// Test helper functions for working with ParseTable
#[cfg(test)]
pub mod test {
    use crate::{Action, GotoIndexing, ParseTable};
    use rust_sitter_ir::{StateId, SymbolId};

    /// Get actions for a given state and symbol, using proper index mapping
    pub fn actions_for(table: &ParseTable, state: usize, sym: SymbolId) -> &[Action] {
        let idx = table.symbol_to_index.get(&sym).copied().unwrap_or_else(|| {
            panic!("Symbol {:?} not found in symbol_to_index", sym);
        });
        &table.action_table[state][idx]
    }

    /// Get goto state for a given state and nonterminal, using proper index mapping
    pub fn goto_for(table: &ParseTable, state: usize, lhs: SymbolId) -> Option<StateId> {
        let row = &table.goto_table[state];
        let col = match table.goto_indexing {
            GotoIndexing::NonterminalMap => *table.nonterminal_to_index.get(&lhs)?,
            GotoIndexing::DirectSymbolId => lhs.0 as usize,
        };
        row.get(col).copied().filter(|s| s.0 != 0)
    }

    /// Check if a state has an Accept action on EOF
    pub fn has_accept_on_eof(table: &ParseTable, state: usize) -> bool {
        actions_for(table, state, table.eof_symbol)
            .iter()
            .any(|a| matches!(a, Action::Accept))
    }

    /// Get all shift destinations from a state for a given symbol
    pub fn shift_destinations(table: &ParseTable, state: usize, sym: SymbolId) -> Vec<StateId> {
        actions_for(table, state, sym)
            .iter()
            .filter_map(|a| match a {
                Action::Shift(s) => Some(*s),
                _ => None,
            })
            .collect()
    }

    /// Get all reduce rules from a state for a given symbol
    pub fn reduce_rules(table: &ParseTable, state: usize, sym: SymbolId) -> Vec<crate::RuleId> {
        actions_for(table, state, sym)
            .iter()
            .filter_map(|a| match a {
                Action::Reduce(r) => Some(*r),
                _ => None,
            })
            .collect()
    }
}
