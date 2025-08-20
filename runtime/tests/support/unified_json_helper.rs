#![cfg(feature = "pure-rust")]

// Minimal "tablegen → unified parser" glue for tests.
// This keeps the pure-Rust golden tests compileable, and lets you flip them on
// once you define a real grammar + table.

use std::collections::BTreeMap;

use rust_sitter_glr_core::{ParseTable, ActionCell, SymbolMetadata, ParseRule, LexMode};
use rust_sitter_ir::{Grammar, SymbolId, StateId, RuleId, Token, TokenPattern};
use rust_sitter_tablegen::LanguageBuilder;
use rust_sitter::pure_parser::TSLanguage;

/// Return a language when the pipeline is wired. Until then, fail fast.
/// This preserves the type so tests compile, but avoids UB if someone
/// runs `--ignored` prematurely.
pub fn unified_json_language() -> &'static TSLanguage {
    panic!(
        "pure-rust golden language not wired yet: replace this with \
         LanguageBuilder output before removing #[ignore] on pure-rust goldens"
    )
}

// Keep the scaffolding functions below as reference for when we wire up the real implementation

/// Very small JSON-like token set (expand as you port real rules).
#[allow(dead_code)]
fn build_min_json_grammar() -> Grammar {
    let mut g = Grammar::new("json_min".to_string());

    // Tokens. Add/rename to match your IR expectations.
    // These are enough to keep the scaffold compiling; semantics come later.
    g.tokens.insert(SymbolId(1), Token {
        name: "{".to_string(),
        pattern: TokenPattern::String("{".to_string()),
        fragile: false,
    });
    g.tokens.insert(SymbolId(2), Token {
        name: "}".to_string(),
        pattern: TokenPattern::String("}".to_string()),
        fragile: false,
    });
    g.tokens.insert(SymbolId(3), Token {
        name: ":".to_string(),
        pattern: TokenPattern::String(":".to_string()),
        fragile: false,
    });
    g.tokens.insert(SymbolId(4), Token {
        name: ",".to_string(),
        pattern: TokenPattern::String(",".to_string()),
        fragile: false,
    });
    g.tokens.insert(SymbolId(5), Token {
        name: "string".to_string(),
        pattern: TokenPattern::Regex(r#""([^"\\]|\\.)*""#.to_string()),
        fragile: false,
    });
    g.tokens.insert(SymbolId(6), Token {
        name: "number".to_string(),
        pattern: TokenPattern::Regex(r#"-?(0|[1-9]\d*)(\.\d+)?([eE][+-]?\d+)?"#.to_string()),
        fragile: false,
    });

    // TODO: Add nonterminals + rules via g.rules[...] when you port a real grammar.
    // Keep this stub minimal to compile; LanguageBuilder may accept empty rules
    // until you wire a real table (the test remains #[ignore] meanwhile).

    g
}

/// A stub parse table that satisfies struct shape and keeps tests compiling.
/// Replace with a real table (actions/gotos/metadata) when you flip tests on.
#[allow(dead_code)]
fn make_minimal_parse_table(grammar: Grammar) -> ParseTable {
    ParseTable {
        // ActionCell model: Vec<Vec<ActionCell>> (state × symbol)
        action_table: vec![],
        goto_table: vec![],
        symbol_metadata: vec![],
        state_count: 0,
        symbol_count: 0,
        symbol_to_index: BTreeMap::new(),
        index_to_symbol: vec![],
        external_scanner_states: vec![],
        rules: vec![], // Fill with real rules when ready
        nonterminal_to_index: BTreeMap::new(),
        eof_symbol: SymbolId(0),
        start_symbol: SymbolId(1),
        grammar,
        initial_state: StateId(0),
        token_count: 0,
        external_token_count: 0,
        lex_modes: vec![],
        extras: vec![],
        dynamic_prec_by_rule: vec![],
        alias_sequences: vec![],
        field_names: vec![],
        field_map: BTreeMap::new(),
    }
}