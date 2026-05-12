use crate::{GLRError, Grammar, Symbol, SymbolId};
use std::collections::{BTreeMap, BTreeSet};

pub(crate) struct SymbolPartitions {
    pub(crate) nonterminals: BTreeSet<SymbolId>,
    pub(crate) externals: BTreeSet<SymbolId>,
    pub(crate) rhs_terminals: BTreeSet<SymbolId>,
    pub(crate) max_symbol: u16,
}

pub(crate) struct SymbolColumns {
    pub(crate) symbol_to_index: BTreeMap<SymbolId, usize>,
    pub(crate) internal_tokens: Vec<SymbolId>,
    pub(crate) external_tokens: Vec<SymbolId>,
}

pub(crate) fn collect_partitions(grammar: &Grammar) -> SymbolPartitions {
    let nonterminals: BTreeSet<SymbolId> = grammar.rules.keys().copied().collect();
    let externals: BTreeSet<SymbolId> = grammar.externals.iter().map(|e| e.symbol_id).collect();
    let mut rhs_terminals: BTreeSet<SymbolId> = BTreeSet::new();
    for rule in grammar.all_rules() {
        for sym in &rule.rhs {
            if let Symbol::Terminal(id) = sym {
                rhs_terminals.insert(*id);
            }
        }
    }

    let max_symbol = grammar
        .tokens
        .keys()
        .chain(grammar.rule_names.keys())
        .chain(nonterminals.iter())
        .chain(externals.iter())
        .chain(rhs_terminals.iter())
        .map(|s| s.0)
        .max()
        .unwrap_or(0);

    SymbolPartitions {
        nonterminals,
        externals,
        rhs_terminals,
        max_symbol,
    }
}

pub(crate) fn eof_symbol(partitions: &SymbolPartitions) -> Result<SymbolId, GLRError> {
    Ok(SymbolId(partitions.max_symbol.checked_add(1).ok_or_else(
        || {
            GLRError::StateMachine(
                "EOF symbol would overflow u16: grammar has too many symbols".into(),
            )
        },
    )?))
}

pub(crate) fn assign_columns(
    grammar: &Grammar,
    partitions: &SymbolPartitions,
    eof_symbol: SymbolId,
) -> Result<SymbolColumns, GLRError> {
    let mut symbol_to_index = BTreeMap::new();
    symbol_to_index.insert(eof_symbol, 0);

    let mut internal_terminals: BTreeSet<SymbolId> = grammar.tokens.keys().copied().collect();
    internal_terminals.extend(partitions.rhs_terminals.iter().copied());
    internal_terminals.remove(&eof_symbol);
    for id in &partitions.externals {
        internal_terminals.remove(id);
    }
    for id in &partitions.nonterminals {
        internal_terminals.remove(id);
    }

    let mut internal_tokens: Vec<SymbolId> = internal_terminals.into_iter().collect();
    internal_tokens.sort_by_key(|s| s.0);
    append_symbols(&mut symbol_to_index, internal_tokens.iter().copied());

    let mut external_tokens: Vec<SymbolId> = partitions.externals.iter().copied().collect();
    external_tokens.sort_by_key(|s| s.0);
    append_symbols(&mut symbol_to_index, external_tokens.iter().copied());

    let mut non_terminals: Vec<SymbolId> = partitions.nonterminals.iter().copied().collect();
    non_terminals.sort_by_key(|s| s.0);
    append_symbols(&mut symbol_to_index, non_terminals);

    let mut other_symbols: Vec<SymbolId> = grammar
        .rule_names
        .keys()
        .copied()
        .filter(|id| !symbol_to_index.contains_key(id))
        .collect();
    other_symbols.sort_by_key(|s| s.0);
    if !other_symbols.is_empty() {
        return Err(GLRError::StateMachine(format!(
            "Unexpected symbols outside terminal/nonterminal partitions: {:?}",
            other_symbols
        )));
    }

    Ok(SymbolColumns {
        symbol_to_index,
        internal_tokens,
        external_tokens,
    })
}

fn append_symbols(
    symbol_to_index: &mut BTreeMap<SymbolId, usize>,
    symbols: impl IntoIterator<Item = SymbolId>,
) {
    for id in symbols {
        if !symbol_to_index.contains_key(&id) {
            let idx = symbol_to_index.len();
            symbol_to_index.insert(id, idx);
        }
    }
}
