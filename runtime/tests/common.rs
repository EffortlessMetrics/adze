// Common test utilities to reduce boilerplate
use rust_sitter_ir::Grammar;
use rust_sitter_glr_core::{FirstFollowSets, build_lr1_automaton, ParseTable};

/// Build a parse table from a grammar - centralizes the construction logic
pub fn build_table(grammar: &Grammar) -> ParseTable {
    let ff = FirstFollowSets::compute(grammar);
    build_lr1_automaton(grammar, &ff).expect("Failed to build automaton")
}

/// Build parse table and wrap in Result for tests that need error handling
pub fn build_table_result(grammar: &Grammar) -> anyhow::Result<ParseTable> {
    let ff = FirstFollowSets::compute(grammar);
    Ok(build_lr1_automaton(grammar, &ff)?)
}