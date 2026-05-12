use crate::GLRError;
use adze_ir::*;
use std::collections::{BTreeMap, BTreeSet};

pub(super) struct SymbolPartitions {
    pub(super) nonterminal_symbols: BTreeSet<SymbolId>,
    pub(super) external_symbols: BTreeSet<SymbolId>,
    pub(super) rhs_terminals: BTreeSet<SymbolId>,
    pub(super) max_symbol: u16,
    pub(super) eof_symbol: SymbolId,
}

impl SymbolPartitions {
    pub(super) fn collect(grammar: &Grammar) -> Result<Self, GLRError> {
        let nonterminal_symbols: BTreeSet<SymbolId> = grammar.rules.keys().copied().collect();
        let external_symbols: BTreeSet<SymbolId> =
            grammar.externals.iter().map(|e| e.symbol_id).collect();
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
            .chain(nonterminal_symbols.iter())
            .chain(external_symbols.iter())
            .chain(rhs_terminals.iter())
            .map(|s| s.0)
            .max()
            .unwrap_or(0);
        let eof_symbol = SymbolId(max_symbol.checked_add(1).ok_or_else(|| {
            GLRError::StateMachine(
                "EOF symbol would overflow u16: grammar has too many symbols".into(),
            )
        })?);

        Ok(Self {
            nonterminal_symbols,
            external_symbols,
            rhs_terminals,
            max_symbol,
            eof_symbol,
        })
    }
}

pub(super) struct SymbolIndex {
    pub(super) symbol_to_index: BTreeMap<SymbolId, usize>,
    pub(super) internal_tokens: Vec<SymbolId>,
    pub(super) ext_tokens: Vec<SymbolId>,
}

pub(super) fn build_symbol_index(
    grammar: &Grammar,
    partitions: &SymbolPartitions,
) -> Result<SymbolIndex, GLRError> {
    let mut symbol_to_index = BTreeMap::new();
    symbol_to_index.insert(partitions.eof_symbol, 0);

    let mut internal_terminals: BTreeSet<SymbolId> = grammar.tokens.keys().copied().collect();
    internal_terminals.extend(partitions.rhs_terminals.iter().copied());
    internal_terminals.remove(&partitions.eof_symbol);
    for id in &partitions.external_symbols {
        internal_terminals.remove(id);
    }
    for id in &partitions.nonterminal_symbols {
        internal_terminals.remove(id);
    }

    let mut internal_tokens: Vec<SymbolId> = internal_terminals.into_iter().collect();
    internal_tokens.sort_by_key(|s| s.0);
    for &id in &internal_tokens {
        if !symbol_to_index.contains_key(&id) {
            let idx = symbol_to_index.len();
            symbol_to_index.insert(id, idx);
        }
    }

    let mut ext_tokens: Vec<SymbolId> = partitions.external_symbols.iter().copied().collect();
    ext_tokens.sort_by_key(|s| s.0);
    for &id in &ext_tokens {
        if !symbol_to_index.contains_key(&id) {
            let idx = symbol_to_index.len();
            symbol_to_index.insert(id, idx);
        }
    }

    let mut non_terminals: Vec<SymbolId> = partitions.nonterminal_symbols.iter().copied().collect();
    non_terminals.sort_by_key(|s| s.0);
    for id in non_terminals {
        if !symbol_to_index.contains_key(&id) {
            let idx = symbol_to_index.len();
            symbol_to_index.insert(id, idx);
        }
    }

    let mut other_symbols: Vec<SymbolId> = grammar
        .rule_names
        .keys()
        .cloned()
        .filter(|id| !symbol_to_index.contains_key(id))
        .collect();
    other_symbols.sort_by_key(|s| s.0);
    if !other_symbols.is_empty() {
        return Err(GLRError::StateMachine(format!(
            "Unexpected symbols outside terminal/nonterminal partitions: {:?}",
            other_symbols
        )));
    }

    Ok(SymbolIndex {
        symbol_to_index,
        internal_tokens,
        ext_tokens,
    })
}

pub(super) fn build_reverse_symbol_index(
    symbol_to_index: &BTreeMap<SymbolId, usize>,
) -> Vec<SymbolId> {
    let mut index_to_symbol = vec![SymbolId(u16::MAX); symbol_to_index.len()];
    for (sym, &idx) in symbol_to_index {
        index_to_symbol[idx] = *sym;
    }
    index_to_symbol
}

pub(super) fn build_nonterminal_to_index(
    symbol_to_index: &BTreeMap<SymbolId, usize>,
    nonterminal_symbols: &BTreeSet<SymbolId>,
) -> BTreeMap<SymbolId, usize> {
    let mut nonterminal_to_index = BTreeMap::new();
    for (&symbol_id, &idx) in symbol_to_index {
        if nonterminal_symbols.contains(&symbol_id) {
            nonterminal_to_index.insert(symbol_id, idx);
        }
    }
    nonterminal_to_index
}
