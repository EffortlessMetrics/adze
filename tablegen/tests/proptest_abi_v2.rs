//! Property-based tests for ABI builder properties (v2).
//!
//! Verifies structural invariants of the ABI language generation:
//! - Symbol counts match grammar definitions
//! - State counts match parse table dimensions
//! - Version constant is always ABI v15
//! - Build output is deterministic
//! - Field counts match grammar field registrations
//! - Edge cases for minimal and maximal grammars

use adze_glr_core::{Action, GotoIndexing, LexMode, ParseTable};
use adze_ir::{
    ExternalToken, FieldId, Grammar, ProductionId, Rule, Symbol, SymbolId, Token, TokenPattern,
};
use adze_tablegen::abi::{
    TREE_SITTER_LANGUAGE_VERSION, TREE_SITTER_MIN_COMPATIBLE_LANGUAGE_VERSION,
};
use adze_tablegen::{AbiLanguageBuilder, CompressedParseTable};
use proptest::prelude::*;
use std::collections::BTreeMap;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

const INVALID: adze_ir::StateId = adze_ir::StateId(u16::MAX);

/// Build a minimal ParseTable suitable for property tests.
fn make_test_table(states: usize, terms: usize, nonterms: usize, externals: usize) -> ParseTable {
    let states = states.max(1);
    let eof_idx = 1 + terms + externals;
    let nonterms_eff = nonterms.max(1);
    let symbol_count = eof_idx + 1 + nonterms_eff;

    let actions: Vec<Vec<Vec<Action>>> = vec![vec![vec![]; symbol_count]; states];
    let gotos: Vec<Vec<adze_ir::StateId>> = vec![vec![INVALID; symbol_count]; states];

    let start_symbol = SymbolId((eof_idx + 1) as u16);
    let eof_symbol = SymbolId(eof_idx as u16);

    let mut symbol_to_index = BTreeMap::new();
    let mut index_to_symbol = vec![SymbolId(0); symbol_count];
    for (i, slot) in index_to_symbol.iter_mut().enumerate() {
        let sym = SymbolId(i as u16);
        symbol_to_index.insert(sym, i);
        *slot = sym;
    }

    ParseTable {
        action_table: actions,
        goto_table: gotos,
        symbol_metadata: vec![],
        state_count: states,
        symbol_count,
        symbol_to_index,
        index_to_symbol,
        external_scanner_states: vec![],
        rules: vec![],
        nonterminal_to_index: BTreeMap::new(),
        goto_indexing: GotoIndexing::NonterminalMap,
        eof_symbol,
        start_symbol,
        grammar: Grammar::new("default".to_string()),
        initial_state: adze_ir::StateId(0),
        token_count: eof_idx + 1,
        external_token_count: externals,
        lex_modes: vec![
            LexMode {
                lex_state: 0,
                external_lex_state: 0,
            };
            states
        ],
        extras: vec![],
        dynamic_prec_by_rule: vec![],
        rule_assoc_by_rule: vec![],
        alias_sequences: vec![],
        field_names: vec![],
        field_map: BTreeMap::new(),
    }
}

/// Build a grammar + parse table pair with given dimensions.
fn build_grammar_and_table(
    name: &str,
    num_terms: usize,
    num_nonterms: usize,
    num_fields: usize,
    num_externals: usize,
    num_states: usize,
) -> (Grammar, ParseTable) {
    let num_terms = num_terms.max(1);
    let num_nonterms = num_nonterms.max(1);
    let num_states = num_states.max(1);

    let mut table = make_test_table(num_states, num_terms, num_nonterms, num_externals);
    let mut grammar = Grammar::new(name.to_string());

    // Register terminals (IDs 1..=num_terms)
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

    let first_nt_idx = 1 + num_terms + num_externals + 1;
    let first_term = SymbolId(1);

    // Register non-terminals
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

    // Add fields
    for i in 0..num_fields {
        grammar
            .fields
            .insert(FieldId(i as u16), format!("field_{i}"));
    }

    // Add external tokens
    for i in 0..num_externals {
        grammar.externals.push(ExternalToken {
            name: format!("ext_{i}"),
            symbol_id: SymbolId((1 + num_terms + i) as u16),
        });
    }

    table.external_token_count = num_externals;

    (grammar, table)
}

// ---------------------------------------------------------------------------
// Strategies
// ---------------------------------------------------------------------------

fn grammar_dims() -> impl Strategy<Value = (usize, usize, usize, usize, usize)> {
    (
        2usize..=5, // terms
        1usize..=3, // nonterms
        0usize..=4, // fields
        0usize..=2, // externals
        1usize..=6, // states
    )
}

fn small_grammar_dims() -> impl Strategy<Value = (usize, usize, usize)> {
    (
        2usize..=4, // terms
        1usize..=2, // nonterms
        1usize..=4, // states
    )
}

fn grammar_name() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9_]{1,8}".prop_map(|s| s)
}

// ===========================================================================
// 1. Symbol count matches grammar (5 proptest)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(30))]

    /// Symbol count in generated code equals parse table symbol_count.
    #[test]
    fn symbol_count_matches_table(
        (terms, nonterms, fields, externals, states) in grammar_dims()
    ) {
        let (grammar, table) = build_grammar_and_table(
            "sc1", terms, nonterms, fields, externals, states,
        );
        let code = AbiLanguageBuilder::new(&grammar, &table).generate().to_string();
        let sc = table.symbol_count as u32;
        let needle = format!("symbol_count : {sc}u32");
        prop_assert!(code.contains(&needle), "expected {needle}");
    }

    /// CompressedParseTable symbol_count matches source table.
    #[test]
    fn compressed_symbol_count_matches(
        (terms, nonterms, _fields, externals, states) in grammar_dims()
    ) {
        let table = make_test_table(states, terms, nonterms, externals);
        let cpt = CompressedParseTable::from_parse_table(&table);
        prop_assert_eq!(cpt.symbol_count(), table.symbol_count);
    }

    /// Symbol count grows with more terminals.
    #[test]
    fn symbol_count_grows_with_terminals(base_terms in 2usize..=3, extra in 1usize..=3) {
        let (_, t1) = build_grammar_and_table("grow1", base_terms, 1, 0, 0, 1);
        let (_, t2) = build_grammar_and_table("grow2", base_terms + extra, 1, 0, 0, 1);
        prop_assert!(t2.symbol_count > t1.symbol_count);
    }

    /// Symbol count grows with more non-terminals.
    #[test]
    fn symbol_count_grows_with_nonterminals(base_nt in 1usize..=2, extra in 1usize..=2) {
        let (_, t1) = build_grammar_and_table("gnt1", 2, base_nt, 0, 0, 1);
        let (_, t2) = build_grammar_and_table("gnt2", 2, base_nt + extra, 0, 0, 1);
        prop_assert!(t2.symbol_count > t1.symbol_count);
    }

    /// Symbol count includes ERROR symbol (column 0).
    #[test]
    fn symbol_count_includes_error(
        (terms, nonterms, _fields, externals, states) in grammar_dims()
    ) {
        let (grammar, table) = build_grammar_and_table(
            "err", terms, nonterms, 0, externals, states,
        );
        // Minimum: ERROR(0) + terms + externals + EOF + nonterms
        let minimum = 1 + terms + externals + 1 + nonterms;
        prop_assert!(
            table.symbol_count >= minimum,
            "symbol_count {} < minimum {minimum}", table.symbol_count,
        );
        let code = AbiLanguageBuilder::new(&grammar, &table).generate().to_string();
        let needle = format!("symbol_count : {}u32", table.symbol_count);
        prop_assert!(code.contains(&needle));
    }
}

// ===========================================================================
// 2. State count matches table (5 proptest)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(30))]

    /// State count in generated code equals table state_count.
    #[test]
    fn state_count_in_generated_code(
        (terms, nonterms, fields, externals, states) in grammar_dims()
    ) {
        let (grammar, table) = build_grammar_and_table(
            "stc1", terms, nonterms, fields, externals, states,
        );
        let code = AbiLanguageBuilder::new(&grammar, &table).generate().to_string();
        let sc = table.state_count as u32;
        let needle = format!("state_count : {sc}u32");
        prop_assert!(code.contains(&needle));
    }

    /// CompressedParseTable state_count matches source table.
    #[test]
    fn compressed_state_count_matches(
        (terms, nonterms, _fields, externals, states) in grammar_dims()
    ) {
        let table = make_test_table(states, terms, nonterms, externals);
        let cpt = CompressedParseTable::from_parse_table(&table);
        prop_assert_eq!(cpt.state_count(), table.state_count);
    }

    /// State count is at least 1.
    #[test]
    fn state_count_at_least_one(
        (terms, nonterms, _fields, _externals, states) in grammar_dims()
    ) {
        let table = make_test_table(states, terms, nonterms, 0);
        prop_assert!(table.state_count >= 1);
    }

    /// Lex modes count matches state count.
    #[test]
    fn lex_modes_match_state_count(
        (terms, nonterms, fields, externals, states) in grammar_dims()
    ) {
        let (grammar, table) = build_grammar_and_table(
            "lmsc", terms, nonterms, fields, externals, states,
        );
        let code = AbiLanguageBuilder::new(&grammar, &table).generate().to_string();
        let lex_entries = code.matches("TSLexState").count();
        // At least state_count entries (array def + struct uses)
        prop_assert!(lex_entries >= states, "lex entries {lex_entries} < states {states}");
    }

    /// Primary state IDs appear in generated code for any state count.
    #[test]
    fn primary_state_ids_present(
        (terms, nonterms, states) in small_grammar_dims()
    ) {
        let (grammar, table) = build_grammar_and_table(
            "psi", terms, nonterms, 0, 0, states,
        );
        let code = AbiLanguageBuilder::new(&grammar, &table).generate().to_string();
        prop_assert!(code.contains("PRIMARY_STATE_IDS"));
    }
}

// ===========================================================================
// 3. Version is constant (5 proptest)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(30))]

    /// Generated code references TREE_SITTER_LANGUAGE_VERSION.
    #[test]
    fn version_constant_referenced(
        (terms, nonterms, fields, externals, states) in grammar_dims()
    ) {
        let (grammar, table) = build_grammar_and_table(
            "ver1", terms, nonterms, fields, externals, states,
        );
        let code = AbiLanguageBuilder::new(&grammar, &table).generate().to_string();
        prop_assert!(code.contains("TREE_SITTER_LANGUAGE_VERSION"));
    }

    /// ABI version constant is always 15.
    #[test]
    fn abi_version_is_15(_dummy in 0u8..10) {
        prop_assert_eq!(TREE_SITTER_LANGUAGE_VERSION, 15u32);
    }

    /// Min compatible version is always 13.
    #[test]
    fn min_compat_version_is_13(_dummy in 0u8..10) {
        prop_assert_eq!(TREE_SITTER_MIN_COMPATIBLE_LANGUAGE_VERSION, 13u32);
    }

    /// Version >= min compatible version.
    #[test]
    fn version_gte_min_compat(_dummy in 0u8..10) {
        prop_assert!(TREE_SITTER_LANGUAGE_VERSION >= TREE_SITTER_MIN_COMPATIBLE_LANGUAGE_VERSION);
    }

    /// Generated code does not contain outdated ABI version numbers.
    #[test]
    fn no_outdated_version(
        (terms, nonterms, states) in small_grammar_dims()
    ) {
        let (grammar, table) = build_grammar_and_table(
            "vold", terms, nonterms, 0, 0, states,
        );
        let code = AbiLanguageBuilder::new(&grammar, &table).generate().to_string();
        // Should not hardcode version : 14u32 or version : 12u32
        prop_assert!(!code.contains("version : 14u32"));
        prop_assert!(!code.contains("version : 12u32"));
    }
}

// ===========================================================================
// 4. Deterministic build (5 proptest)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    /// Same grammar+table always produces identical code.
    #[test]
    fn deterministic_same_input(
        (terms, nonterms, fields, externals, states) in grammar_dims()
    ) {
        let (grammar, table) = build_grammar_and_table(
            "det1", terms, nonterms, fields, externals, states,
        );
        let a = AbiLanguageBuilder::new(&grammar, &table).generate().to_string();
        let b = AbiLanguageBuilder::new(&grammar, &table).generate().to_string();
        prop_assert_eq!(a, b);
    }

    /// Three consecutive builds produce identical output.
    #[test]
    fn deterministic_triple(
        (terms, nonterms, states) in small_grammar_dims()
    ) {
        let (grammar, table) = build_grammar_and_table(
            "det3", terms, nonterms, 0, 0, states,
        );
        let runs: Vec<String> = (0..3)
            .map(|_| AbiLanguageBuilder::new(&grammar, &table).generate().to_string())
            .collect();
        prop_assert_eq!(&runs[0], &runs[1]);
        prop_assert_eq!(&runs[1], &runs[2]);
    }

    /// Different grammar names produce different code.
    #[test]
    fn different_names_different_code(
        name_a in "[a-z]{3,6}",
        name_b in "[a-z]{3,6}",
    ) {
        prop_assume!(name_a != name_b);
        let (g1, t1) = build_grammar_and_table(&name_a, 2, 1, 0, 0, 1);
        let (g2, t2) = build_grammar_and_table(&name_b, 2, 1, 0, 0, 1);
        let c1 = AbiLanguageBuilder::new(&g1, &t1).generate().to_string();
        let c2 = AbiLanguageBuilder::new(&g2, &t2).generate().to_string();
        prop_assert_ne!(c1, c2);
    }

    /// Grammar name appears in tree_sitter function name.
    #[test]
    fn name_in_ffi_function(name in grammar_name()) {
        let (grammar, table) = build_grammar_and_table(&name, 2, 1, 0, 0, 1);
        let code = AbiLanguageBuilder::new(&grammar, &table).generate().to_string();
        let fn_name = format!("tree_sitter_{name}");
        prop_assert!(code.contains(&fn_name), "missing {fn_name}");
    }

    /// Adding a field changes the output.
    #[test]
    fn fields_change_output(
        (terms, nonterms, states) in small_grammar_dims()
    ) {
        let (g0, t0) = build_grammar_and_table("fchg", terms, nonterms, 0, 0, states);
        let (g1, t1) = build_grammar_and_table("fchg", terms, nonterms, 2, 0, states);
        let c0 = AbiLanguageBuilder::new(&g0, &t0).generate().to_string();
        let c1 = AbiLanguageBuilder::new(&g1, &t1).generate().to_string();
        prop_assert_ne!(c0, c1);
    }
}

// ===========================================================================
// 5. Regular ABI tests (10 tests)
// ===========================================================================

#[test]
fn abi_contains_tslanguage_struct() {
    let (grammar, table) = build_grammar_and_table("basic", 2, 1, 0, 0, 2);
    let code = AbiLanguageBuilder::new(&grammar, &table)
        .generate()
        .to_string();
    assert!(code.contains("TSLanguage"));
}

#[test]
fn abi_contains_language_static() {
    let (grammar, table) = build_grammar_and_table("lang_st", 3, 1, 0, 0, 2);
    let code = AbiLanguageBuilder::new(&grammar, &table)
        .generate()
        .to_string();
    assert!(code.contains("LANGUAGE"));
}

#[test]
fn abi_symbol_names_array_present() {
    let (grammar, table) = build_grammar_and_table("sn", 2, 1, 0, 0, 1);
    let code = AbiLanguageBuilder::new(&grammar, &table)
        .generate()
        .to_string();
    assert!(code.contains("SYMBOL_NAME_PTRS"));
}

#[test]
fn abi_parse_table_data_present() {
    let (grammar, table) = build_grammar_and_table("pt", 2, 1, 0, 0, 2);
    let code = AbiLanguageBuilder::new(&grammar, &table)
        .generate()
        .to_string();
    assert!(code.contains("PARSE_TABLE"));
}

#[test]
fn abi_symbol_metadata_present() {
    let (grammar, table) = build_grammar_and_table("sm", 3, 2, 0, 0, 2);
    let code = AbiLanguageBuilder::new(&grammar, &table)
        .generate()
        .to_string();
    assert!(code.contains("SYMBOL_METADATA"));
}

#[test]
fn abi_public_symbol_map_present() {
    let (grammar, table) = build_grammar_and_table("psm", 2, 1, 0, 0, 1);
    let code = AbiLanguageBuilder::new(&grammar, &table)
        .generate()
        .to_string();
    assert!(code.contains("PUBLIC_SYMBOL_MAP"));
}

#[test]
fn abi_production_id_map_present() {
    let (grammar, table) = build_grammar_and_table("pid", 2, 2, 0, 0, 2);
    let code = AbiLanguageBuilder::new(&grammar, &table)
        .generate()
        .to_string();
    assert!(code.contains("PRODUCTION_ID_MAP"));
}

#[test]
fn abi_parse_actions_present() {
    let (grammar, table) = build_grammar_and_table("pa", 2, 1, 0, 0, 2);
    let code = AbiLanguageBuilder::new(&grammar, &table)
        .generate()
        .to_string();
    assert!(code.contains("PARSE_ACTIONS"));
}

#[test]
fn abi_lex_fn_is_some() {
    let (grammar, table) = build_grammar_and_table("lf", 2, 1, 0, 0, 1);
    let code = AbiLanguageBuilder::new(&grammar, &table)
        .generate()
        .to_string();
    assert!(code.contains("lex_fn : Some"));
}

#[test]
fn abi_keyword_lex_fn_is_none() {
    let (grammar, table) = build_grammar_and_table("klf", 2, 1, 0, 0, 1);
    let code = AbiLanguageBuilder::new(&grammar, &table)
        .generate()
        .to_string();
    assert!(code.contains("keyword_lex_fn : None"));
}

// ===========================================================================
// 6. ABI with various grammar sizes (5 tests)
// ===========================================================================

#[test]
fn abi_two_terminals_one_rule() {
    let (grammar, table) = build_grammar_and_table("sz2", 2, 1, 0, 0, 2);
    let code = AbiLanguageBuilder::new(&grammar, &table)
        .generate()
        .to_string();
    assert!(code.contains("tree_sitter_sz2"));
    let sc = table.symbol_count as u32;
    assert!(code.contains(&format!("symbol_count : {sc}u32")));
}

#[test]
fn abi_five_terminals_three_rules() {
    let (grammar, table) = build_grammar_and_table("sz5", 5, 3, 0, 0, 4);
    let code = AbiLanguageBuilder::new(&grammar, &table)
        .generate()
        .to_string();
    assert!(code.contains("tree_sitter_sz5"));
    assert!(code.contains(&format!("state_count : {}u32", table.state_count)));
}

#[test]
fn abi_with_fields_and_externals() {
    let (grammar, table) = build_grammar_and_table("szfe", 3, 2, 3, 2, 5);
    let code = AbiLanguageBuilder::new(&grammar, &table)
        .generate()
        .to_string();
    assert!(code.contains("field_count : 3u32"));
    assert!(code.contains("external_token_count : 2u32"));
}

#[test]
fn abi_many_states() {
    let (grammar, table) = build_grammar_and_table("szms", 3, 2, 0, 0, 20);
    let code = AbiLanguageBuilder::new(&grammar, &table)
        .generate()
        .to_string();
    assert!(code.contains("state_count : 20u32"));
    let lex_count = code.matches("TSLexState").count();
    assert!(
        lex_count >= 20,
        "expected >= 20 TSLexState, got {lex_count}"
    );
}

#[test]
fn abi_many_fields() {
    let (grammar, table) = build_grammar_and_table("szmf", 2, 1, 10, 0, 2);
    let code = AbiLanguageBuilder::new(&grammar, &table)
        .generate()
        .to_string();
    assert!(code.contains("field_count : 10u32"));
    assert!(code.contains("FIELD_NAME_PTRS"));
}

// ===========================================================================
// 7. Field count matches (5 proptest)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(30))]

    /// Field count in generated code equals grammar fields length.
    #[test]
    fn field_count_matches_grammar(
        (terms, nonterms, fields, externals, states) in grammar_dims()
    ) {
        let (grammar, table) = build_grammar_and_table(
            "fc1", terms, nonterms, fields, externals, states,
        );
        let code = AbiLanguageBuilder::new(&grammar, &table).generate().to_string();
        let fc = grammar.fields.len() as u32;
        let needle = format!("field_count : {fc}u32");
        prop_assert!(code.contains(&needle));
    }

    /// Zero fields produces field_count : 0u32.
    #[test]
    fn zero_fields(
        (terms, nonterms, states) in small_grammar_dims()
    ) {
        let (grammar, table) = build_grammar_and_table(
            "fc0", terms, nonterms, 0, 0, states,
        );
        let code = AbiLanguageBuilder::new(&grammar, &table).generate().to_string();
        prop_assert!(code.contains("field_count : 0u32"));
    }

    /// Adding fields increases field_count.
    #[test]
    fn more_fields_more_count(base in 0usize..=2, extra in 1usize..=3) {
        let (g1, t1) = build_grammar_and_table("mf1", 2, 1, base, 0, 1);
        let (g2, t2) = build_grammar_and_table("mf2", 2, 1, base + extra, 0, 1);
        let c1 = AbiLanguageBuilder::new(&g1, &t1).generate().to_string();
        let c2 = AbiLanguageBuilder::new(&g2, &t2).generate().to_string();
        let fc1 = base as u32;
        let fc2 = (base + extra) as u32;
        let n1 = format!("field_count : {fc1}u32");
        let n2 = format!("field_count : {fc2}u32");
        prop_assert!(c1.contains(&n1));
        prop_assert!(c2.contains(&n2));
    }

    /// Field names array present when fields exist.
    #[test]
    fn field_names_present_when_fields_exist(fields in 1usize..=4) {
        let (grammar, table) = build_grammar_and_table("fnp", 2, 1, fields, 0, 1);
        let code = AbiLanguageBuilder::new(&grammar, &table).generate().to_string();
        prop_assert!(code.contains("FIELD_NAME_PTRS"));
    }

    /// CompressedParseTable test factory preserves counts.
    #[test]
    fn compressed_test_factory(sym in 3usize..=20, st in 1usize..=15) {
        let cpt = CompressedParseTable::new_for_testing(sym, st);
        prop_assert_eq!(cpt.symbol_count(), sym);
        prop_assert_eq!(cpt.state_count(), st);
    }
}

// ===========================================================================
// 8. Edge cases (10 tests)
// ===========================================================================

#[test]
fn edge_single_terminal_single_nonterminal() {
    let (grammar, table) = build_grammar_and_table("e1", 1, 1, 0, 0, 1);
    let code = AbiLanguageBuilder::new(&grammar, &table)
        .generate()
        .to_string();
    assert!(code.contains("TSLanguage"));
    assert!(code.contains("tree_sitter_e1"));
}

#[test]
fn edge_single_state() {
    let (grammar, table) = build_grammar_and_table("e2", 2, 1, 0, 0, 1);
    let code = AbiLanguageBuilder::new(&grammar, &table)
        .generate()
        .to_string();
    assert!(code.contains("state_count : 1u32"));
}

#[test]
fn edge_no_externals() {
    let (grammar, table) = build_grammar_and_table("e3", 2, 1, 0, 0, 2);
    let code = AbiLanguageBuilder::new(&grammar, &table)
        .generate()
        .to_string();
    assert!(code.contains("external_token_count : 0u32"));
}

#[test]
fn edge_no_fields() {
    let (grammar, table) = build_grammar_and_table("e4", 3, 2, 0, 0, 3);
    let code = AbiLanguageBuilder::new(&grammar, &table)
        .generate()
        .to_string();
    assert!(code.contains("field_count : 0u32"));
}

#[test]
fn edge_alias_count_always_zero() {
    let (grammar, table) = build_grammar_and_table("e5", 2, 1, 2, 1, 2);
    let code = AbiLanguageBuilder::new(&grammar, &table)
        .generate()
        .to_string();
    assert!(code.contains("alias_count : 0u32"));
}

#[test]
fn edge_large_state_count_always_zero() {
    let (grammar, table) = build_grammar_and_table("e6", 2, 1, 0, 0, 2);
    let code = AbiLanguageBuilder::new(&grammar, &table)
        .generate()
        .to_string();
    assert!(code.contains("large_state_count : 0u32"));
}

#[test]
fn edge_eof_symbol_is_zero() {
    let (grammar, table) = build_grammar_and_table("e7", 3, 2, 1, 0, 3);
    let code = AbiLanguageBuilder::new(&grammar, &table)
        .generate()
        .to_string();
    assert!(code.contains("eof_symbol : 0"));
}

#[test]
fn edge_max_alias_seq_length_zero() {
    let (grammar, table) = build_grammar_and_table("e8", 2, 1, 0, 0, 1);
    let code = AbiLanguageBuilder::new(&grammar, &table)
        .generate()
        .to_string();
    assert!(code.contains("max_alias_sequence_length : 0u16"));
}

#[test]
fn edge_external_scanner_present_with_externals() {
    let (grammar, table) = build_grammar_and_table("e9", 2, 1, 0, 2, 2);
    let code = AbiLanguageBuilder::new(&grammar, &table)
        .generate()
        .to_string();
    assert!(code.contains("ExternalScanner"));
}

#[test]
fn edge_alias_pointers_null() {
    let (grammar, table) = build_grammar_and_table("e10", 2, 1, 0, 0, 1);
    let code = AbiLanguageBuilder::new(&grammar, &table)
        .generate()
        .to_string();
    assert!(code.contains("alias_map : std :: ptr :: null ()"));
    assert!(code.contains("alias_sequences : std :: ptr :: null"));
}

// ===========================================================================
// Bonus: additional property tests to exceed 50 total
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    /// Token count in generated code is consistent.
    #[test]
    fn token_count_consistent(
        (terms, nonterms, _f, externals, states) in grammar_dims()
    ) {
        let (grammar, table) = build_grammar_and_table(
            "tc", terms, nonterms, 0, externals, states,
        );
        let code = AbiLanguageBuilder::new(&grammar, &table).generate().to_string();
        let tc = table.token_count as u32;
        let needle = format!("token_count : {tc}u32");
        prop_assert!(code.contains(&needle));
    }

    /// External token count in generated code is consistent.
    #[test]
    fn external_token_count_consistent(
        (terms, nonterms, _f, externals, states) in grammar_dims()
    ) {
        let (grammar, table) = build_grammar_and_table(
            "etc", terms, nonterms, 0, externals, states,
        );
        let code = AbiLanguageBuilder::new(&grammar, &table).generate().to_string();
        let etc = externals as u32;
        let needle = format!("external_token_count : {etc}u32");
        prop_assert!(code.contains(&needle));
    }

    /// Production LHS index present in output.
    #[test]
    fn production_lhs_index_present(
        (terms, nonterms, states) in small_grammar_dims()
    ) {
        let (grammar, table) = build_grammar_and_table(
            "plhs", terms, nonterms, 0, 0, states,
        );
        let code = AbiLanguageBuilder::new(&grammar, &table).generate().to_string();
        prop_assert!(code.contains("PRODUCTION_LHS_INDEX"));
    }

    /// Rule count reference present in generated code.
    #[test]
    fn rule_count_reference_present(
        (terms, nonterms, states) in small_grammar_dims()
    ) {
        let (grammar, table) = build_grammar_and_table(
            "rcr", terms, nonterms, 0, 0, states,
        );
        let code = AbiLanguageBuilder::new(&grammar, &table).generate().to_string();
        prop_assert!(code.contains("rule_count"));
    }

    /// SyncPtr wrapper used for thread-safe statics.
    #[test]
    fn sync_ptr_used(
        (terms, nonterms, states) in small_grammar_dims()
    ) {
        let (grammar, table) = build_grammar_and_table(
            "sp", terms, nonterms, 0, 0, states,
        );
        let code = AbiLanguageBuilder::new(&grammar, &table).generate().to_string();
        prop_assert!(code.contains("SyncPtr"));
    }
}
