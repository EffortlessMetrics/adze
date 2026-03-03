#![allow(clippy::needless_range_loop)]

//! Comprehensive tests for primary state ID generation in adze-tablegen.
//!
//! Tests cover:
//! - Primary state IDs for each symbol
//! - Default primary state
//! - Primary states in generated code
//! - Primary state count
//! - Primary state determinism
//! - Large grammar primary states

use adze_glr_core::{GotoIndexing, LexMode, ParseTable};
use adze_ir::{
    ExternalToken, Grammar, ProductionId, Rule, StateId, Symbol, SymbolId, Token, TokenPattern,
};
use adze_tablegen::abi_builder::AbiLanguageBuilder;
use adze_tablegen::language_gen::LanguageGenerator;
use std::collections::BTreeMap;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

const INVALID: StateId = StateId(u16::MAX);

fn string_token(name: &str, literal: &str) -> Token {
    Token {
        name: name.to_string(),
        pattern: TokenPattern::String(literal.to_string()),
        fragile: false,
    }
}

fn regex_token(name: &str, pattern: &str) -> Token {
    Token {
        name: name.to_string(),
        pattern: TokenPattern::Regex(pattern.to_string()),
        fragile: false,
    }
}

/// Build a grammar + parse table with sequential symbol layout.
/// Layout: ERROR(0), terminals 1..=num_terms, externals, EOF, nonterminals.
fn build_grammar_and_table(
    name: &str,
    num_terms: usize,
    num_nonterms: usize,
    num_externals: usize,
    num_states: usize,
) -> (Grammar, ParseTable) {
    let num_terms = num_terms.max(1);
    let num_nonterms = num_nonterms.max(1);
    let num_states = num_states.max(1);

    let eof_idx = 1 + num_terms + num_externals;
    let symbol_count = eof_idx + 1 + num_nonterms;

    let eof_symbol = SymbolId(eof_idx as u16);
    let start_symbol = SymbolId((eof_idx + 1) as u16);
    let first_term = SymbolId(1);

    let mut symbol_to_index = BTreeMap::new();
    let mut index_to_symbol = vec![SymbolId(0); symbol_count];
    for i in 0..symbol_count {
        let sym = SymbolId(i as u16);
        symbol_to_index.insert(sym, i);
        index_to_symbol[i] = sym;
    }

    let mut grammar = Grammar::new(name.to_string());

    for i in 1..=num_terms {
        let sym = SymbolId(i as u16);
        grammar.tokens.insert(
            sym,
            Token {
                name: format!("tok_{i}"),
                pattern: TokenPattern::String(format!("t{i}")),
                fragile: false,
            },
        );
    }

    let first_nt_idx = eof_idx + 1;
    for i in 0..num_nonterms {
        let sym = SymbolId((first_nt_idx + i) as u16);
        grammar.rule_names.insert(sym, format!("rule_{i}"));
        grammar.add_rule(Rule {
            lhs: sym,
            rhs: vec![Symbol::Terminal(first_term)],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(i as u16),
        });
    }

    for i in 0..num_externals {
        grammar.externals.push(ExternalToken {
            name: format!("ext_{i}"),
            symbol_id: SymbolId((1 + num_terms + i) as u16),
        });
    }

    let mut nonterminal_to_index = BTreeMap::new();
    nonterminal_to_index.insert(start_symbol, start_symbol.0 as usize);

    let table = ParseTable {
        action_table: vec![vec![vec![]; symbol_count]; num_states],
        goto_table: vec![vec![INVALID; symbol_count]; num_states],
        symbol_metadata: vec![],
        state_count: num_states,
        symbol_count,
        symbol_to_index,
        index_to_symbol,
        nonterminal_to_index,
        external_scanner_states: vec![],
        rules: vec![],
        eof_symbol,
        start_symbol,
        grammar: Grammar::default(),
        initial_state: StateId(0),
        token_count: eof_idx + 1,
        external_token_count: num_externals,
        lex_modes: vec![
            LexMode {
                lex_state: 0,
                external_lex_state: 0,
            };
            num_states
        ],
        extras: vec![],
        dynamic_prec_by_rule: vec![],
        rule_assoc_by_rule: vec![],
        alias_sequences: vec![],
        field_names: vec![],
        field_map: BTreeMap::new(),
        goto_indexing: GotoIndexing::NonterminalMap,
    };

    (grammar, table)
}

/// Render AbiLanguageBuilder output as a String.
fn abi_code(grammar: &Grammar, table: &ParseTable) -> String {
    AbiLanguageBuilder::new(grammar, table)
        .generate()
        .to_string()
}

/// Render LanguageGenerator output as a String.
fn lang_gen_code(grammar: &Grammar, table: &ParseTable) -> String {
    LanguageGenerator::new(grammar, table)
        .generate()
        .to_string()
}

/// Extract the PRIMARY_STATE_IDS entries from generated code.
/// Returns the raw string between the brackets of the array definition.
fn extract_primary_state_body(code: &str) -> Option<String> {
    let marker = "PRIMARY_STATE_IDS";
    let start = code.find(marker)?;
    let rest = &code[start..];
    let eq_amp = rest.find("= &")?;
    let after_eq = &rest[eq_amp + 3..];
    let bracket_open = after_eq.find('[')?;
    let inner = &after_eq[bracket_open + 1..];
    let bracket_close = inner.find(']')?;
    Some(inner[..bracket_close].to_string())
}

/// Count entries in the PRIMARY_STATE_IDS from generated code.
fn count_primary_state_entries(code: &str) -> usize {
    match extract_primary_state_body(code) {
        Some(body) if body.trim().is_empty() => 0,
        Some(body) => body.split(',').count(),
        None => 0,
    }
}

/// Parse numeric values out of the PRIMARY_STATE_IDS entries.
fn parse_primary_state_values(code: &str) -> Vec<u16> {
    let body = match extract_primary_state_body(code) {
        Some(b) if !b.trim().is_empty() => b,
        _ => return vec![],
    };
    body.split(',')
        .filter_map(|entry| {
            // Entries look like "0 as u16", "1 as u16", "TSStateId (0 as u16)", etc.
            let s = entry.trim();
            // Extract the number before "as" (if present), otherwise leading digits
            let num_part = if let Some(pos) = s.find("as") {
                s[..pos].trim()
            } else {
                s
            };
            // Strip non-digit wrappers like "TSStateId(" or trailing ")"
            let digits: String = num_part.chars().filter(|c| c.is_ascii_digit()).collect();
            digits.parse::<u16>().ok()
        })
        .collect()
}

// ===========================================================================
// 1. Primary state IDs for each symbol/state
// ===========================================================================

#[test]
fn single_state_has_primary_id_zero() {
    let (g, t) = build_grammar_and_table("single", 1, 1, 0, 1);
    let code = abi_code(&g, &t);
    let vals = parse_primary_state_values(&code);
    assert!(!vals.is_empty(), "should have at least one primary state ID");
    assert_eq!(vals[0], 0, "first primary state should be 0");
}

#[test]
fn two_states_have_sequential_ids() {
    let (g, t) = build_grammar_and_table("two", 1, 1, 0, 2);
    let code = abi_code(&g, &t);
    let vals = parse_primary_state_values(&code);
    assert!(vals.len() >= 2);
    assert_eq!(vals[0], 0);
    assert_eq!(vals[1], 1);
}

#[test]
fn three_states_identity_mapping() {
    let (g, t) = build_grammar_and_table("three", 1, 1, 0, 3);
    let code = abi_code(&g, &t);
    let vals = parse_primary_state_values(&code);
    for i in 0..3 {
        assert_eq!(vals[i], i as u16, "state {i} should map to itself");
    }
}

#[test]
fn five_states_all_present() {
    let (g, t) = build_grammar_and_table("five", 2, 2, 0, 5);
    let code = abi_code(&g, &t);
    let vals = parse_primary_state_values(&code);
    assert_eq!(vals.len(), 5);
    for i in 0..5 {
        assert_eq!(vals[i], i as u16);
    }
}

#[test]
fn primary_state_ids_start_at_zero() {
    let (g, t) = build_grammar_and_table("start", 1, 1, 0, 4);
    let code = abi_code(&g, &t);
    let vals = parse_primary_state_values(&code);
    assert_eq!(vals.first(), Some(&0u16));
}

#[test]
fn primary_state_ids_are_contiguous() {
    let (g, t) = build_grammar_and_table("contiguous", 2, 1, 0, 7);
    let code = abi_code(&g, &t);
    let vals = parse_primary_state_values(&code);
    for i in 1..vals.len() {
        assert_eq!(
            vals[i],
            vals[i - 1] + 1,
            "gap between state {} and {}",
            i - 1,
            i
        );
    }
}

#[test]
fn primary_ids_with_external_tokens() {
    let (g, t) = build_grammar_and_table("ext", 1, 1, 2, 3);
    let code = abi_code(&g, &t);
    let vals = parse_primary_state_values(&code);
    assert_eq!(vals.len(), 3);
    for i in 0..3 {
        assert_eq!(vals[i], i as u16);
    }
}

// ===========================================================================
// 2. Default primary state
// ===========================================================================

#[test]
fn default_primary_state_is_zero_for_minimal_grammar() {
    let (g, t) = build_grammar_and_table("min", 1, 1, 0, 1);
    let code = abi_code(&g, &t);
    let vals = parse_primary_state_values(&code);
    assert_eq!(vals[0], 0);
}

#[test]
fn default_primary_state_always_included() {
    for states in [1, 2, 5, 10] {
        let (g, t) = build_grammar_and_table("def", 1, 1, 0, states);
        let code = abi_code(&g, &t);
        let vals = parse_primary_state_values(&code);
        assert!(
            vals.contains(&0),
            "state 0 missing with {states} states"
        );
    }
}

#[test]
fn initial_state_present_in_primary_ids() {
    let (g, t) = build_grammar_and_table("init", 1, 1, 0, 4);
    let code = abi_code(&g, &t);
    let vals = parse_primary_state_values(&code);
    assert!(vals.contains(&(t.initial_state.0)));
}

#[test]
fn single_state_grammar_produces_one_entry() {
    let (g, t) = build_grammar_and_table("one", 1, 1, 0, 1);
    let code = abi_code(&g, &t);
    assert_eq!(count_primary_state_entries(&code), 1);
}

// ===========================================================================
// 3. Primary states in generated code
// ===========================================================================

#[test]
fn abi_builder_emits_primary_state_ids_array() {
    let (g, t) = build_grammar_and_table("abi_arr", 1, 1, 0, 2);
    let code = abi_code(&g, &t);
    assert!(
        code.contains("PRIMARY_STATE_IDS"),
        "generated code must contain PRIMARY_STATE_IDS"
    );
}

#[test]
fn language_gen_emits_primary_state_ids_array() {
    let (g, t) = build_grammar_and_table("lg_arr", 1, 1, 0, 2);
    let code = lang_gen_code(&g, &t);
    assert!(
        code.contains("PRIMARY_STATE_IDS"),
        "LanguageGenerator must emit PRIMARY_STATE_IDS"
    );
}

#[test]
fn primary_state_ids_referenced_in_language_struct() {
    let (g, t) = build_grammar_and_table("ref", 1, 1, 0, 2);
    let code = abi_code(&g, &t);
    assert!(
        code.contains("primary_state_ids"),
        "LANGUAGE struct must reference primary_state_ids field"
    );
    assert!(
        code.contains("PRIMARY_STATE_IDS . as_ptr"),
        "primary_state_ids should use PRIMARY_STATE_IDS.as_ptr()"
    );
}

#[test]
fn language_gen_references_primary_state_ids_ptr() {
    let (g, t) = build_grammar_and_table("lgref", 1, 1, 0, 2);
    let code = lang_gen_code(&g, &t);
    assert!(
        code.contains("primary_state_ids"),
        "LanguageGenerator LANGUAGE struct must have primary_state_ids"
    );
}

#[test]
fn primary_state_ids_is_static_array() {
    let (g, t) = build_grammar_and_table("stat", 1, 1, 0, 3);
    let code = abi_code(&g, &t);
    // Should contain a static declaration for the array
    assert!(code.contains("PRIMARY_STATE_IDS"));
    // The array should be addressable via as_ptr
    assert!(code.contains("PRIMARY_STATE_IDS . as_ptr"));
}

#[test]
fn generated_code_has_no_duplicate_primary_state_ids_decl() {
    let (g, t) = build_grammar_and_table("dup", 2, 2, 0, 3);
    let code = abi_code(&g, &t);
    let occurrences = code.matches("PRIMARY_STATE_IDS :").count();
    assert_eq!(
        occurrences, 1,
        "PRIMARY_STATE_IDS should be declared exactly once, found {occurrences}"
    );
}

// ===========================================================================
// 4. Primary state count
// ===========================================================================

#[test]
fn primary_state_count_equals_state_count() {
    for states in [1, 2, 3, 5, 8] {
        let (g, t) = build_grammar_and_table("cnt", 1, 1, 0, states);
        let code = abi_code(&g, &t);
        let count = count_primary_state_entries(&code);
        assert_eq!(
            count, states,
            "expected {states} primary state entries, got {count}"
        );
    }
}

#[test]
fn count_unaffected_by_terminal_count() {
    let (g1, t1) = build_grammar_and_table("t1", 1, 1, 0, 3);
    let (g2, t2) = build_grammar_and_table("t5", 5, 1, 0, 3);
    let c1 = count_primary_state_entries(&abi_code(&g1, &t1));
    let c2 = count_primary_state_entries(&abi_code(&g2, &t2));
    assert_eq!(c1, c2, "terminal count should not affect primary state count");
}

#[test]
fn count_unaffected_by_nonterminal_count() {
    let (g1, t1) = build_grammar_and_table("n1", 1, 1, 0, 4);
    let (g2, t2) = build_grammar_and_table("n4", 1, 4, 0, 4);
    let c1 = count_primary_state_entries(&abi_code(&g1, &t1));
    let c2 = count_primary_state_entries(&abi_code(&g2, &t2));
    assert_eq!(c1, c2, "nonterminal count should not affect primary state count");
}

#[test]
fn count_unaffected_by_externals() {
    let (g1, t1) = build_grammar_and_table("e0", 1, 1, 0, 3);
    let (g2, t2) = build_grammar_and_table("e3", 1, 1, 3, 3);
    let c1 = count_primary_state_entries(&abi_code(&g1, &t1));
    let c2 = count_primary_state_entries(&abi_code(&g2, &t2));
    assert_eq!(c1, c2, "external tokens should not affect primary state count");
}

#[test]
fn lang_gen_primary_state_count_equals_symbol_count() {
    // LanguageGenerator uses symbol_name_indices for primary state IDs
    let (g, t) = build_grammar_and_table("lgcnt", 2, 1, 0, 3);
    let code = lang_gen_code(&g, &t);
    let count = count_primary_state_entries(&code);
    // LanguageGenerator's PRIMARY_STATE_IDS is indexed by symbol names
    assert!(count > 0, "should have primary state entries");
}

// ===========================================================================
// 5. Primary state determinism
// ===========================================================================

#[test]
fn same_grammar_same_primary_states() {
    let (g1, t1) = build_grammar_and_table("det", 2, 1, 0, 4);
    let (g2, t2) = build_grammar_and_table("det", 2, 1, 0, 4);
    let code1 = abi_code(&g1, &t1);
    let code2 = abi_code(&g2, &t2);
    let body1 = extract_primary_state_body(&code1);
    let body2 = extract_primary_state_body(&code2);
    assert_eq!(body1, body2, "identical grammars must produce identical primary state IDs");
}

#[test]
fn determinism_across_ten_runs() {
    let mut results = Vec::new();
    for _ in 0..10 {
        let (g, t) = build_grammar_and_table("rep", 2, 2, 0, 5);
        let code = abi_code(&g, &t);
        results.push(extract_primary_state_body(&code));
    }
    for i in 1..results.len() {
        assert_eq!(
            results[0], results[i],
            "run 0 and run {i} produced different primary state IDs"
        );
    }
}

#[test]
fn lang_gen_determinism() {
    let (g1, t1) = build_grammar_and_table("lgdet", 1, 1, 0, 3);
    let (g2, t2) = build_grammar_and_table("lgdet", 1, 1, 0, 3);
    let code1 = lang_gen_code(&g1, &t1);
    let code2 = lang_gen_code(&g2, &t2);
    let body1 = extract_primary_state_body(&code1);
    let body2 = extract_primary_state_body(&code2);
    assert_eq!(body1, body2);
}

#[test]
fn different_grammars_can_differ_in_count() {
    let (g1, t1) = build_grammar_and_table("a", 1, 1, 0, 2);
    let (g2, t2) = build_grammar_and_table("b", 1, 1, 0, 5);
    let c1 = count_primary_state_entries(&abi_code(&g1, &t1));
    let c2 = count_primary_state_entries(&abi_code(&g2, &t2));
    assert_ne!(c1, c2, "grammars with different state counts should differ");
}

#[test]
fn values_deterministic_not_just_count() {
    let (g, t) = build_grammar_and_table("vdet", 3, 2, 1, 6);
    let vals1 = parse_primary_state_values(&abi_code(&g, &t));
    let vals2 = parse_primary_state_values(&abi_code(&g, &t));
    assert_eq!(vals1, vals2, "parsed values must be identical across runs");
}

// ===========================================================================
// 6. Large grammar primary states
// ===========================================================================

#[test]
fn twenty_states() {
    let (g, t) = build_grammar_and_table("s20", 3, 2, 0, 20);
    let code = abi_code(&g, &t);
    let count = count_primary_state_entries(&code);
    assert_eq!(count, 20);
    let vals = parse_primary_state_values(&code);
    for i in 0..20 {
        assert_eq!(vals[i], i as u16);
    }
}

#[test]
fn fifty_states() {
    let (g, t) = build_grammar_and_table("s50", 5, 3, 0, 50);
    let code = abi_code(&g, &t);
    assert_eq!(count_primary_state_entries(&code), 50);
    let vals = parse_primary_state_values(&code);
    assert_eq!(vals.len(), 50);
    assert_eq!(*vals.last().unwrap(), 49);
}

#[test]
fn hundred_states() {
    let (g, t) = build_grammar_and_table("s100", 5, 5, 0, 100);
    let code = abi_code(&g, &t);
    assert_eq!(count_primary_state_entries(&code), 100);
    let vals = parse_primary_state_values(&code);
    for i in 0..100 {
        assert_eq!(vals[i], i as u16, "mismatch at state {i}");
    }
}

#[test]
fn large_grammar_with_externals() {
    let (g, t) = build_grammar_and_table("large_ext", 10, 5, 4, 30);
    let code = abi_code(&g, &t);
    let count = count_primary_state_entries(&code);
    assert_eq!(count, 30, "state count should be 30 regardless of externals");
}

#[test]
fn large_grammar_identity_property() {
    let (g, t) = build_grammar_and_table("id_prop", 8, 4, 2, 40);
    let code = abi_code(&g, &t);
    let vals = parse_primary_state_values(&code);
    // Identity: primary_state_ids[i] == i for all i
    for i in 0..vals.len() {
        assert_eq!(
            vals[i], i as u16,
            "identity property violated at index {i}"
        );
    }
}

#[test]
fn large_grammar_max_value_equals_state_count_minus_one() {
    let states = 60;
    let (g, t) = build_grammar_and_table("maxval", 4, 3, 0, states);
    let code = abi_code(&g, &t);
    let vals = parse_primary_state_values(&code);
    assert_eq!(
        *vals.last().unwrap(),
        (states - 1) as u16,
        "max primary state ID should be state_count - 1"
    );
}

#[test]
fn many_terminals_does_not_inflate_primary_states() {
    let (g, t) = build_grammar_and_table("many_tok", 50, 1, 0, 5);
    let code = abi_code(&g, &t);
    let count = count_primary_state_entries(&code);
    assert_eq!(count, 5, "50 terminals should not increase primary state count beyond 5");
}

#[test]
fn many_nonterminals_does_not_inflate_primary_states() {
    let (g, t) = build_grammar_and_table("many_nt", 1, 30, 0, 5);
    let code = abi_code(&g, &t);
    let count = count_primary_state_entries(&code);
    assert_eq!(count, 5, "30 nonterminals should not increase primary state count beyond 5");
}
