//! Test helper utilities for creating stub languages and parse tables.
//!
//! This module provides common functionality for tests and examples that need
//! to create minimal Language instances for testing purposes.

use crate::{Language, Token, language::SymbolMetadata};

#[cfg(feature = "glr-core")]
fn empty_parse_table() -> &'static rust_sitter_glr_core::ParseTable {
    use rust_sitter_glr_core::{GotoIndexing, ParseTable};
    use rust_sitter_ir::{Grammar, StateId, SymbolId};
    use std::collections::BTreeMap;

    Box::leak(Box::new(ParseTable {
        action_table: vec![],
        goto_table: vec![],
        symbol_metadata: vec![],
        state_count: 0,
        symbol_count: 0,
        symbol_to_index: BTreeMap::new(),
        index_to_symbol: vec![],
        external_scanner_states: vec![],
        rules: vec![],
        nonterminal_to_index: BTreeMap::new(),
        goto_indexing: GotoIndexing::NonterminalMap,
        eof_symbol: SymbolId(0),
        start_symbol: SymbolId(0),
        grammar: Grammar::new("stub".to_string()),
        initial_state: StateId(0),
        token_count: 0,
        external_token_count: 0,
        lex_modes: vec![],
        extras: vec![],
        dynamic_prec_by_rule: vec![],
        rule_assoc_by_rule: vec![],
        alias_sequences: vec![],
        field_names: vec![],
        field_map: BTreeMap::new(),
    }))
}

#[cfg(not(feature = "glr-core"))]
fn empty_parse_table() -> crate::language::ParseTable {
    crate::language::ParseTable {
        state_count: 0,
        action_table: vec![],
        small_parse_table: None,
        small_parse_table_map: None,
    }
}

/// Create a minimal stub language for testing purposes.
///
/// This creates a Language with:
/// - Empty parse tables (will not actually parse successfully)
/// - Single placeholder symbol with metadata
/// - Empty field names
/// - Optional tokenizer (GLR mode only) or static tokens
pub fn stub_language() -> Language {
    let table = empty_parse_table();
    let builder = Language::builder()
        .parse_table(table)
        .symbol_names(vec!["placeholder".into()])
        .symbol_metadata(vec![SymbolMetadata {
            is_terminal: true,
            is_visible: true,
            is_supertype: false,
        }])
        .field_names(vec![]);

    #[cfg(feature = "glr-core")]
    let builder = builder.tokenizer(|_| Box::new(std::iter::empty()));

    builder.build().unwrap()
}

/// Create a stub language with pre-defined tokens (for GLR mode).
///
/// In non-GLR mode, tokens are ignored since there's no tokenizer field.
#[cfg(feature = "glr-core")]
pub fn stub_language_with_tokens(tokens: Vec<Token>) -> Language {
    Language::builder()
        .parse_table(empty_parse_table())
        .symbol_names(vec!["placeholder".into()])
        .symbol_metadata(vec![SymbolMetadata {
            is_terminal: true,
            is_visible: true,
            is_supertype: false,
        }])
        .field_names(vec![])
        .tokenizer(move |_| Box::new(tokens.clone().into_iter()))
        .build()
        .unwrap()
}

/// For non-GLR builds, tokens parameter is ignored
#[cfg(not(feature = "glr-core"))]
pub fn stub_language_with_tokens(_tokens: Vec<Token>) -> Language {
    stub_language()
}

/// Create a test language with more symbols for complex testing
pub fn multi_symbol_test_language(symbol_count: usize) -> Language {
    let table = empty_parse_table();
    let builder = Language::builder()
        .parse_table(table)
        .symbol_names((0..symbol_count).map(|i| format!("symbol_{}", i)).collect())
        .symbol_metadata(vec![
            SymbolMetadata {
                is_terminal: true,
                is_visible: true,
                is_supertype: false,
            };
            symbol_count
        ])
        .field_names(vec![]);

    #[cfg(feature = "glr-core")]
    let builder = builder.tokenizer(|_| Box::new(std::iter::empty()));

    builder.build().unwrap()
}
