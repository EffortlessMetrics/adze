// Ensure only one backend is enabled
#[cfg(all(feature = "pure-rust", feature = "c-backend"))]
compile_error!("Enable exactly one backend: 'pure-rust' OR 'c-backend'.");

// Re-export modules that contain grammars
pub mod ambiguous;
pub mod arithmetic;
pub mod external_word_example;
pub mod optionals;
pub mod performance_test;
pub mod repetitions;
pub mod test_precedence;
pub mod test_whitespace;
pub mod words;

// Tree-sitter compatibility language helpers
#[cfg(all(feature = "ts-compat", feature = "pure-rust"))]
pub mod ts_langs {
    use rust_sitter::ts_compat::Language;
    use std::sync::Arc;

    /// Get the arithmetic language for ts_compat API
    pub fn arithmetic() -> Arc<Language> {
        // For now, create a minimal language with empty tables
        // TODO: Implement proper loading from generated LANGUAGE and SMALL_PARSE_TABLE
        // This will require exposing proper conversion methods from tablegen

        // Create a minimal Grammar
        let mut grammar = rust_sitter::rust_sitter_ir::Grammar::default();
        grammar.name = "arithmetic".to_string();

        // Create a minimal ParseTable
        let table = rust_sitter::rust_sitter_glr_core::ParseTable {
            action_table: vec![vec![]], // At least one state
            goto_table: vec![vec![]],   // At least one state
            symbol_metadata: vec![],
            state_count: 1,
            symbol_count: 3, // minimal: EOF, error, expression
            symbol_to_index: Default::default(),
            index_to_symbol: vec![],
            external_scanner_states: vec![],
            rules: vec![],
            nonterminal_to_index: Default::default(),
            eof_symbol: rust_sitter::rust_sitter_ir::SymbolId(0),
            start_symbol: rust_sitter::rust_sitter_ir::SymbolId(2),
            grammar: grammar.clone(),
            initial_state: rust_sitter::rust_sitter_ir::StateId(0),
            token_count: 1,
            external_token_count: 0,
            lex_modes: vec![],
            extras: vec![],
            dynamic_prec_by_rule: vec![],
            alias_sequences: vec![],
            field_names: vec![],
            field_map: Default::default(),
        };

        Arc::new(Language::new("arithmetic", grammar, table))
    }
}
