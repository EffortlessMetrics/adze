// Minimal "tablegen → unified parser" glue for tests.
// This keeps the pure-Rust golden tests compileable, and lets you flip them on
// once you define a real grammar + table.

mod json_grammar;
mod language_builder;

use std::collections::BTreeMap;

use rust_sitter::pure_parser::TSLanguage;
use rust_sitter_glr_core::ParseTable;
use rust_sitter_ir::{Grammar, StateId, SymbolId, Token, TokenPattern};

/// Return a language when the pipeline is wired. Until then, fail fast.
/// This preserves the type so tests compile, but avoids UB if someone
/// runs `--ignored` prematurely.
pub fn unified_json_language() -> &'static TSLanguage {
    use rust_sitter_glr_core::{FirstFollowSets, build_lr1_automaton};

    let grammar = json_grammar::build_json_grammar();

    let ff = FirstFollowSets::compute(&grammar);
    let mut table = build_lr1_automaton(&grammar, &ff).expect("build LR(1) automaton");

    // Fail fast if something drifts
    eprintln!(
        "DEBUG: Grammar tokens: {}, Table token_count: {}",
        grammar.tokens.len(),
        table.token_count
    );
    eprintln!(
        "DEBUG: Table state_count: {}, action_table.len(): {}",
        table.state_count,
        table.action_table.len()
    );

    // Check the action table structure
    for (state_idx, state_actions) in table.action_table.iter().enumerate().take(3) {
        eprintln!(
            "DEBUG: State {} has {} symbol entries",
            state_idx,
            state_actions.len()
        );
        if state_actions.is_empty() {
            eprintln!("  WARNING: State {} has empty action row!", state_idx);
        }
    }

    assert_eq!(table.token_count, grammar.tokens.len(), "token_count drift");
    assert!(table.state_count > 0, "no states generated");
    assert!(!table.action_table.is_empty(), "action table is empty");
    // The automaton may add extra symbols (like EOF or augmented start)
    assert!(
        table.index_to_symbol.len() >= grammar.tokens.len(),
        "symbol index too small"
    );

    // Normalize the table to Tree-sitter format before building the language
    language_builder::normalize_table_for_ts(&mut table);

    let lang = language_builder::build_json_ts_language(&grammar, &table);
    Box::leak(Box::new(lang))
}

// Keep the scaffolding functions below as reference for when we wire up the real implementation

/// Build a minimal but real JSON grammar using [`GrammarBuilder`].
///
/// This grammar is intentionally small but contains enough structure to
/// exercise the parser pipeline and demonstrate how a grammar is constructed.
#[allow(dead_code)]
fn build_min_json_grammar() -> Grammar {
    use rust_sitter_ir::builder::GrammarBuilder;

    GrammarBuilder::new("json_min")
        // Terminals
        .token("{", "{")
        .token("}", "}")
        .token(":", ":")
        .token(",", ",")
        .token("string", r#""([^"\\]|\\.)*""#)
        .token("number", r#"-?(0|[1-9]\d*)(\.\d+)?([eE][+-]?\d+)?"#)
        // Nonterminals and rules
        .rule("document", vec!["object"])
        .rule("start", vec!["value"])
        .rule("value", vec!["string"])
        .rule("value", vec!["number"])
        .rule("value", vec!["object"])
        .rule("object", vec!["{", "}"])
        .rule("object", vec!["{", "pairs", "}"])
        .rule("pairs", vec!["pair"])
        .rule("pairs", vec!["pair", ",", "pairs"])
        .rule("pair", vec!["string", ":", "value"])
        .start("document")
        .build()
}

/// Construct a minimal parse table for the grammar using the regular
/// `build_lr1_automaton` pipeline. This produces real action and goto tables
/// along with all necessary metadata so the table can be fed into the language
/// builder or parser directly.
#[allow(dead_code)]
fn make_minimal_parse_table(grammar: Grammar) -> ParseTable {
    use rust_sitter_glr_core::{FirstFollowSets, build_lr1_automaton};

    // Compute the LR(1) parse table for the supplied grammar
    let ff = FirstFollowSets::compute(&grammar);
    let mut table = build_lr1_automaton(&grammar, &ff).expect("build LR(1) automaton");

    // Normalize to Tree-sitter conventions so it can be plugged into the
    // language builder without additional massaging.
    language_builder::normalize_table_for_ts(&mut table);

    table
}
