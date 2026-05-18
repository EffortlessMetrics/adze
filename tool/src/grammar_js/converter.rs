//! Converter from Grammar.js to Adze IR

use super::{GrammarJs, Rule as JsRule};
use adze_ir::{FieldId, SymbolId};
use indexmap::IndexMap;
use indexmap::IndexMap as OrderedMap;
use std::collections::HashMap;

mod choice_rules;
mod conversion;
mod extras;
mod field_rules;
mod fields;
mod productions;
mod rule_body;
mod sequences;
mod symbols;
mod tokens;

#[cfg(test)]
mod tests;

#[cfg(not(debug_assertions))]
macro_rules! eprintln {
    ($($arg:tt)*) => {};
}

#[cfg(debug_assertions)]
macro_rules! eprintln {
    ($($arg:tt)*) => {
        if std::env::var("RUST_LOG")
            .ok()
            .unwrap_or_default()
            .contains("debug")
        {
            std::eprintln!($($arg)*);
        }
    };
}

/// Converts a Grammar.js structure to Adze IR
pub struct GrammarJsConverter {
    grammar_js: GrammarJs,
    symbol_names: OrderedMap<String, SymbolId>,
    token_symbols: HashMap<SymbolId, SymbolId>, // Maps token-backed rule symbols to their token IDs
    next_symbol_id: usize,
    next_production_id: usize,
    next_field_id: usize,
    fields: IndexMap<FieldId, String>,
}

impl GrammarJsConverter {
    pub fn new(grammar_js: GrammarJs) -> Self {
        Self {
            grammar_js,
            symbol_names: OrderedMap::new(),
            token_symbols: HashMap::new(),
            next_symbol_id: 1, // Start at 1 to reserve SymbolId(0) for EOF
            next_production_id: 0,
            next_field_id: 0,
            fields: IndexMap::new(),
        }
    }
}
