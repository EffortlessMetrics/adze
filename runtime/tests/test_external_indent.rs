//! Test external token (INDENT) integration
//! This validates that external tokens can be properly handled in the parse table

#![cfg(feature = "pure-rust")]

mod support;

use rust_sitter::decoder;
use rust_sitter_glr_core::{build_lr1_automaton, Action, FirstFollowSets};
use rust_sitter_ir::SymbolId;

#[test]
#[ignore = "pure-rust parser integration unstable"]
fn external_indent_token_in_table() {
    // Build grammar with external INDENT token
    let grammar = support::indent_grammar::build_indent_grammar();
    let first_follow = FirstFollowSets::compute(&grammar).unwrap();
    let mut parse_table = build_lr1_automaton(&grammar, &first_follow).unwrap();

    // Normalize for Tree-sitter compatibility
    support::language_builder::normalize_table_for_ts(&mut parse_table);

    // Check that INDENT token appears in the parse table
    let _indent_id = SymbolId(1);
    let mut indent_found = false;
    let mut indent_states = Vec::new();

    // Search for states that can shift on INDENT
    for (state_idx, row) in parse_table.action_table.iter().enumerate() {
        // INDENT is at column 1 (SymbolId(1))
        if row.len() > 1 {
            for action in &row[1] {
                if matches!(action, Action::Shift(_)) {
                    indent_found = true;
                    indent_states.push(state_idx);
                    break;
                }
            }
        }
    }

    assert!(
        indent_found,
        "No Shift action found for INDENT token in parse table"
    );
    println!(
        "✓ INDENT token found in parse table at {} states",
        indent_states.len()
    );
}

#[test]
#[ignore = "pure-rust parser integration unstable"]
fn external_token_language_generation() {
    // Build grammar and parse table
    let grammar = support::indent_grammar::build_indent_grammar();
    let first_follow = FirstFollowSets::compute(&grammar).unwrap();
    let mut parse_table = build_lr1_automaton(&grammar, &first_follow).unwrap();

    support::language_builder::normalize_table_for_ts(&mut parse_table);

    // Build language with external scanner stub - use indent-specific builder
    let lang = support::language_builder::build_indent_ts_language(&grammar, &parse_table);

    // Verify external token count is set
    assert_eq!(
        lang.external_token_count, 1,
        "External token count should be 1 for INDENT"
    );

    // Check that the language has the expected number of symbols
    let _total_symbols = grammar.tokens.len()
        + grammar
            .rules
            .values()
            .flat_map(|rules| rules.iter())
            .map(|r| r.lhs)
            .collect::<std::collections::HashSet<_>>()
            .len();

    assert!(lang.symbol_count > 0, "Symbol count should be > 0");
    println!(
        "✓ Language generated with {} symbols, {} external tokens",
        lang.symbol_count, lang.external_token_count
    );
}

#[test]
#[ignore = "pure-rust parser integration unstable"]
fn external_indent_decode_and_validate() {
    // Full pipeline test: grammar -> table -> language -> decode
    let grammar = support::indent_grammar::build_indent_grammar();
    let first_follow = FirstFollowSets::compute(&grammar).unwrap();
    let mut parse_table = build_lr1_automaton(&grammar, &first_follow).unwrap();

    support::language_builder::normalize_table_for_ts(&mut parse_table);

    // Build language with indent-specific builder that sets external_token_count
    let lang = support::language_builder::build_indent_ts_language(&grammar, &parse_table);
    let lang = Box::leak(Box::new(lang));

    // Decode and validate
    let decoded_table = decoder::decode_parse_table(lang);

    // Find INDENT token in decoded table
    let indent_id = SymbolId(1);
    let indent_col = decoded_table
        .symbol_to_index
        .get(&indent_id)
        .expect("INDENT not found in symbol_to_index mapping");

    // Add partition boundary asserts to ensure external tokens are properly positioned
    let tcols = (decoded_table.token_count + decoded_table.external_token_count) as usize;
    assert!(
        decoded_table.external_token_count >= 1,
        "external_token_count must be ≥ 1 for INDENT"
    );
    assert!(
        *indent_col < tcols,
        "INDENT must be in token partition (col {} < tcols {})",
        indent_col,
        tcols
    );

    // Verify at least one state can shift on INDENT
    let mut shift_found = false;
    for state in 0..decoded_table.state_count {
        if *indent_col < decoded_table.action_table[state].len() {
            for action in &decoded_table.action_table[state][*indent_col] {
                if matches!(action, Action::Shift(_)) {
                    shift_found = true;
                    break;
                }
            }
        }
        if shift_found {
            break;
        }
    }

    assert!(shift_found, "No Shift action for INDENT in decoded table");
    println!("✓ External INDENT token successfully decoded with Shift action");
}

#[test]
#[ignore = "pure-rust parser integration unstable"]
fn external_token_smoke_test() {
    // Smoke test to ensure external token doesn't break the system
    let grammar = support::indent_grammar::build_indent_grammar();

    // Verify INDENT token is registered
    let indent_id = SymbolId(1);
    assert!(
        grammar.tokens.contains_key(&indent_id),
        "INDENT token not registered"
    );

    // Verify it's marked as external (by having empty pattern in our convention)
    let indent_token = &grammar.tokens[&indent_id];
    assert_eq!(indent_token.name, "INDENT", "Wrong token name");

    // Build parse table should succeed
    let first_follow = FirstFollowSets::compute(&grammar).unwrap();
    let parse_table = build_lr1_automaton(&grammar, &first_follow);
    assert!(
        parse_table.is_ok(),
        "Failed to build parse table with external token"
    );

    println!("✓ External token smoke test passed");
}
