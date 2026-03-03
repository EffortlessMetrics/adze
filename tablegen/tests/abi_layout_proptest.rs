#![allow(clippy::needless_range_loop)]
//! Property-based tests for ABI layout generation in adze-tablegen.
//!
//! Tests:
//!  1. ABI version number is correct
//!  2. LANGUAGE struct is generated
//!  3. Parse action arrays in generated code
//!  4. State count matches input
//!  5. Symbol count matches input
//!  6. Generated code contains all required arrays
//!  7. Generated code determinism
//!  8. Token count matches parse table
//!  9. External token count matches grammar
//! 10. Field count matches grammar fields
//! 11. Production ID count derived from rules
//! 12. Symbol name pointers array sized to symbol count
//! 13. Lex modes array present
//! 14. Public symbol map present
//! 15. Primary state IDs present
//! 16. Production LHS index present
//! 17. TS rules array present
//! 18. Field map slices present
//! 19. Field map entries present
//! 20. Variant symbol maps present
//! 21. EOF symbol column zero
//! 22. Grammar name in FFI function
//! 23. Zero fields handled
//! 24. Zero externals handled
//! 25. Single state grammar
//! 26. Many states grammar
//! 27. Large state count in generated code
//! 28. Production count field present
//! 29. Rule count field present
//! 30. Alias count is zero
//! 31. Max alias sequence length is zero

use adze_glr_core::{Action, GotoIndexing, LexMode, ParseTable};
use adze_ir::{
    ExternalToken, FieldId, Grammar, ProductionId, Rule, Symbol, SymbolId, Token, TokenPattern,
};
use adze_tablegen::AbiLanguageBuilder;
use proptest::prelude::*;
use std::collections::BTreeMap;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

const INVALID: adze_ir::StateId = adze_ir::StateId(u16::MAX);

/// Build a minimal ParseTable for property tests.
fn make_empty_table(states: usize, terms: usize, nonterms: usize, externals: usize) -> ParseTable {
    let states = states.max(1);
    let eof_idx = 1 + terms + externals;
    let nonterms_eff = if nonterms == 0 { 1 } else { nonterms };
    let symbol_count = eof_idx + 1 + nonterms_eff;

    let actions: Vec<Vec<Vec<Action>>> = vec![vec![vec![]; symbol_count]; states];
    let gotos: Vec<Vec<adze_ir::StateId>> = vec![vec![INVALID; symbol_count]; states];

    let start_symbol = SymbolId((eof_idx + 1) as u16);
    let eof_symbol = SymbolId(eof_idx as u16);

    let mut symbol_to_index = BTreeMap::new();
    let mut index_to_symbol = vec![SymbolId(0); symbol_count];
    for i in 0..symbol_count {
        let sym = SymbolId(i as u16);
        symbol_to_index.insert(sym, i);
        index_to_symbol[i] = sym;
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
                external_lex_state: 0
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

    let mut table = make_empty_table(num_states, num_terms, num_nonterms, num_externals);
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

    let first_nt_idx = 1 + num_terms + num_externals + 1; // ERROR + terms + externals + EOF
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

/// Generate code string from grammar and table.
fn gen_code(name: &str, terms: usize, nonterms: usize, fields: usize, externals: usize, states: usize) -> String {
    let (grammar, table) = build_grammar_and_table(name, terms, nonterms, fields, externals, states);
    AbiLanguageBuilder::new(&grammar, &table).generate().to_string()
}

// ---------------------------------------------------------------------------
// Strategies
// ---------------------------------------------------------------------------

fn layout_dims() -> impl Strategy<Value = (usize, usize, usize, usize, usize)> {
    (
        1usize..=5, // terms
        1usize..=4, // nonterms
        0usize..=3, // fields
        0usize..=2, // externals
        1usize..=6, // states
    )
}

// ---------------------------------------------------------------------------
// Property tests
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(30))]

    // 1. ABI version number is correct
    #[test]
    fn abi_version_is_correct(
        (terms, nonterms, fields, externals, states) in layout_dims()
    ) {
        let code = gen_code("abi_v", terms, nonterms, fields, externals, states);
        prop_assert!(
            code.contains("TREE_SITTER_LANGUAGE_VERSION"),
            "ABI version constant must appear in generated code"
        );
    }

    // 2. LANGUAGE struct is generated
    #[test]
    fn language_struct_generated(
        (terms, nonterms, fields, externals, states) in layout_dims()
    ) {
        let code = gen_code("lang_s", terms, nonterms, fields, externals, states);
        prop_assert!(
            code.contains("pub static LANGUAGE : TSLanguage"),
            "LANGUAGE static must be generated"
        );
    }

    // 3. Parse action arrays in generated code
    #[test]
    fn parse_actions_array_present(
        (terms, nonterms, fields, externals, states) in layout_dims()
    ) {
        let code = gen_code("pa", terms, nonterms, fields, externals, states);
        prop_assert!(
            code.contains("PARSE_ACTIONS"),
            "PARSE_ACTIONS array must be present"
        );
        prop_assert!(
            code.contains("TSParseAction"),
            "TSParseAction type must appear in parse actions"
        );
    }

    // 4. State count matches input
    #[test]
    fn state_count_matches_input(
        (terms, nonterms, fields, externals, states) in layout_dims()
    ) {
        let (grammar, table) = build_grammar_and_table(
            "sc", terms, nonterms, fields, externals, states,
        );
        let code = AbiLanguageBuilder::new(&grammar, &table).generate().to_string();
        let sc = table.state_count as u32;
        let needle = format!("state_count : {sc}u32");
        prop_assert!(code.contains(&needle), "state_count : {sc}u32 not found");
    }

    // 5. Symbol count matches input
    #[test]
    fn symbol_count_matches_input(
        (terms, nonterms, fields, externals, states) in layout_dims()
    ) {
        let (grammar, table) = build_grammar_and_table(
            "symc", terms, nonterms, fields, externals, states,
        );
        let code = AbiLanguageBuilder::new(&grammar, &table).generate().to_string();
        let sc = table.symbol_count as u32;
        let needle = format!("symbol_count : {sc}u32");
        prop_assert!(code.contains(&needle), "symbol_count : {sc}u32 not found");
    }

    // 6. Generated code contains all required arrays
    #[test]
    fn all_required_arrays_present(
        (terms, nonterms, fields, externals, states) in layout_dims()
    ) {
        let code = gen_code("arr", terms, nonterms, fields, externals, states);
        let required = [
            "SYMBOL_METADATA",
            "PARSE_TABLE",
            "SMALL_PARSE_TABLE",
            "SMALL_PARSE_TABLE_MAP",
            "PARSE_ACTIONS",
            "LEX_MODES",
            "FIELD_MAP_SLICES",
            "FIELD_MAP_ENTRIES",
            "PUBLIC_SYMBOL_MAP",
            "PRIMARY_STATE_IDS",
            "PRODUCTION_ID_MAP",
            "PRODUCTION_LHS_INDEX",
            "TS_RULES",
        ];
        for name in &required {
            prop_assert!(
                code.contains(name),
                "required array '{}' not found in generated code", name
            );
        }
    }

    // 7. Generated code determinism
    #[test]
    fn code_generation_is_deterministic(
        (terms, nonterms, fields, externals, states) in layout_dims()
    ) {
        let (grammar, table) = build_grammar_and_table(
            "det", terms, nonterms, fields, externals, states,
        );
        let code1 = AbiLanguageBuilder::new(&grammar, &table).generate().to_string();
        let code2 = AbiLanguageBuilder::new(&grammar, &table).generate().to_string();
        prop_assert_eq!(code1, code2, "generate() must be deterministic");
    }

    // 8. Token count matches parse table
    #[test]
    fn token_count_matches_parse_table(
        (terms, nonterms, fields, externals, states) in layout_dims()
    ) {
        let (grammar, table) = build_grammar_and_table(
            "tc", terms, nonterms, fields, externals, states,
        );
        let code = AbiLanguageBuilder::new(&grammar, &table).generate().to_string();
        let tc = table.token_count as u32;
        let needle = format!("token_count : {tc}u32");
        prop_assert!(code.contains(&needle), "token_count : {tc}u32 not found");
    }

    // 9. External token count matches grammar
    #[test]
    fn external_token_count_matches_grammar(
        (terms, nonterms, fields, externals, states) in layout_dims()
    ) {
        let (grammar, table) = build_grammar_and_table(
            "etc", terms, nonterms, fields, externals, states,
        );
        let code = AbiLanguageBuilder::new(&grammar, &table).generate().to_string();
        let ec = externals as u32;
        let needle = format!("external_token_count : {ec}u32");
        prop_assert!(code.contains(&needle), "external_token_count : {ec}u32 not found");
    }

    // 10. Field count matches grammar fields
    #[test]
    fn field_count_matches_grammar_fields(
        (terms, nonterms, fields, externals, states) in layout_dims()
    ) {
        let (grammar, table) = build_grammar_and_table(
            "fc", terms, nonterms, fields, externals, states,
        );
        let code = AbiLanguageBuilder::new(&grammar, &table).generate().to_string();
        let fc = grammar.fields.len() as u32;
        let needle = format!("field_count : {fc}u32");
        prop_assert!(code.contains(&needle), "field_count : {fc}u32 not found");
    }

    // 11. Production ID count derived from rules
    #[test]
    fn production_id_count_from_rules(
        (terms, nonterms, fields, externals, states) in layout_dims()
    ) {
        let (grammar, table) = build_grammar_and_table(
            "pic", terms, nonterms, fields, externals, states,
        );
        let code = AbiLanguageBuilder::new(&grammar, &table).generate().to_string();
        let max_prod = grammar
            .rules
            .values()
            .flat_map(|rs| rs.iter().map(|r| r.production_id.0))
            .max()
            .unwrap_or(0);
        let expected = (max_prod as u32) + 1;
        let needle = format!("production_id_count : {expected}u32");
        prop_assert!(code.contains(&needle), "production_id_count : {expected}u32 not found");
    }

    // 12. Symbol name pointers array sized to symbol count
    #[test]
    fn symbol_name_ptrs_sized_to_symbol_count(
        (terms, nonterms, fields, externals, states) in layout_dims()
    ) {
        let (grammar, table) = build_grammar_and_table(
            "snp", terms, nonterms, fields, externals, states,
        );
        let code = AbiLanguageBuilder::new(&grammar, &table).generate().to_string();
        let sc = table.symbol_count as u32;
        let needle = format!("SYMBOL_NAME_PTRS_LEN : usize = {sc}u32 as usize");
        prop_assert!(
            code.contains(&needle),
            "SYMBOL_NAME_PTRS_LEN for {} not found", sc
        );
    }

    // 13. Lex modes array present with at least state_count entries
    #[test]
    fn lex_modes_array_present(
        (terms, nonterms, fields, externals, states) in layout_dims()
    ) {
        let (grammar, table) = build_grammar_and_table(
            "lm", terms, nonterms, fields, externals, states,
        );
        let code = AbiLanguageBuilder::new(&grammar, &table).generate().to_string();
        let count = code.matches("TSLexState").count();
        prop_assert!(
            count >= table.state_count,
            "expected >= {} TSLexState entries, found {}", table.state_count, count
        );
    }

    // 14. Public symbol map present
    #[test]
    fn public_symbol_map_in_language(
        (terms, nonterms, fields, externals, states) in layout_dims()
    ) {
        let code = gen_code("psm", terms, nonterms, fields, externals, states);
        prop_assert!(code.contains("PUBLIC_SYMBOL_MAP"));
        prop_assert!(code.contains("public_symbol_map : PUBLIC_SYMBOL_MAP"));
    }

    // 15. Primary state IDs present
    #[test]
    fn primary_state_ids_in_language(
        (terms, nonterms, fields, externals, states) in layout_dims()
    ) {
        let code = gen_code("psi", terms, nonterms, fields, externals, states);
        prop_assert!(code.contains("PRIMARY_STATE_IDS"));
        prop_assert!(code.contains("primary_state_ids : PRIMARY_STATE_IDS"));
    }

    // 16. Production LHS index present
    #[test]
    fn production_lhs_index_in_language(
        (terms, nonterms, fields, externals, states) in layout_dims()
    ) {
        let code = gen_code("pli", terms, nonterms, fields, externals, states);
        prop_assert!(code.contains("PRODUCTION_LHS_INDEX"));
        prop_assert!(code.contains("production_lhs_index : PRODUCTION_LHS_INDEX"));
    }

    // 17. TS rules array present with rule_count field
    #[test]
    fn ts_rules_array_and_rule_count(
        (terms, nonterms, fields, externals, states) in layout_dims()
    ) {
        let code = gen_code("tsr", terms, nonterms, fields, externals, states);
        prop_assert!(code.contains("TS_RULES"), "TS_RULES array missing");
        prop_assert!(code.contains("rule_count"), "rule_count field missing");
    }

    // 18. Field map slices present
    #[test]
    fn field_map_slices_in_code(
        (terms, nonterms, fields, externals, states) in layout_dims()
    ) {
        let code = gen_code("fms", terms, nonterms, fields, externals, states);
        prop_assert!(code.contains("FIELD_MAP_SLICES"));
        prop_assert!(code.contains("field_map_slices : FIELD_MAP_SLICES"));
    }

    // 19. Field map entries present
    #[test]
    fn field_map_entries_in_code(
        (terms, nonterms, fields, externals, states) in layout_dims()
    ) {
        let code = gen_code("fme", terms, nonterms, fields, externals, states);
        prop_assert!(code.contains("FIELD_MAP_ENTRIES"));
        prop_assert!(code.contains("field_map_entries : FIELD_MAP_ENTRIES"));
    }

    // 20. Variant symbol maps present (SYMBOL_ID_TO_INDEX and SYMBOL_INDEX_TO_ID)
    #[test]
    fn variant_symbol_maps_present(
        (terms, nonterms, fields, externals, states) in layout_dims()
    ) {
        let code = gen_code("vsm", terms, nonterms, fields, externals, states);
        prop_assert!(code.contains("SYMBOL_ID_TO_INDEX"), "SYMBOL_ID_TO_INDEX missing");
        prop_assert!(code.contains("SYMBOL_INDEX_TO_ID"), "SYMBOL_INDEX_TO_ID missing");
    }

    // 21. EOF symbol column zero
    #[test]
    fn eof_symbol_is_column_zero(
        (terms, nonterms, fields, externals, states) in layout_dims()
    ) {
        let code = gen_code("eof0", terms, nonterms, fields, externals, states);
        prop_assert!(
            code.contains("eof_symbol : 0"),
            "eof_symbol must always be 0 in Language struct"
        );
    }

    // 22. Grammar name in FFI function
    #[test]
    fn grammar_name_in_ffi_function(
        (terms, nonterms, fields, externals, states) in layout_dims()
    ) {
        let code = gen_code("myparser", terms, nonterms, fields, externals, states);
        prop_assert!(
            code.contains("tree_sitter_myparser"),
            "FFI function tree_sitter_myparser not found"
        );
    }

    // 23. Zero fields handled correctly
    #[test]
    fn zero_fields_handled(
        (terms, nonterms, _fields, externals, states) in layout_dims()
    ) {
        let code = gen_code("zf", terms, nonterms, 0, externals, states);
        prop_assert!(code.contains("field_count : 0u32"), "zero fields must set field_count : 0u32");
        // Empty field name pointers array
        prop_assert!(
            code.contains("FIELD_NAME_PTRS : [SyncPtr ; 0]"),
            "zero fields should produce empty FIELD_NAME_PTRS"
        );
    }

    // 24. Zero externals handled correctly
    #[test]
    fn zero_externals_handled(
        (terms, nonterms, fields, _externals, states) in layout_dims()
    ) {
        let code = gen_code("ze", terms, nonterms, fields, 0, states);
        prop_assert!(
            code.contains("external_token_count : 0u32"),
            "zero externals must set external_token_count : 0u32"
        );
    }

    // 25. Single state grammar generates valid code
    #[test]
    fn single_state_grammar(
        (terms, nonterms, fields, externals, _states) in layout_dims()
    ) {
        let code = gen_code("ss", terms, nonterms, fields, externals, 1);
        prop_assert!(code.contains("state_count : 1u32"), "single state grammar must have state_count : 1u32");
    }

    // 26. Many states grammar generates valid code
    #[test]
    fn many_states_grammar(
        terms in 1usize..=3,
        nonterms in 1usize..=2,
        states in 4usize..=10,
    ) {
        let code = gen_code("ms", terms, nonterms, 0, 0, states);
        let sc = states as u32;
        let needle = format!("state_count : {sc}u32");
        prop_assert!(code.contains(&needle), "state_count : {sc}u32 not found for many-states");
    }

    // 27. Large state count field matches
    #[test]
    fn large_state_count_in_code(
        (terms, nonterms, fields, externals, states) in layout_dims()
    ) {
        let code = gen_code("lsc", terms, nonterms, fields, externals, states);
        // large_state_count is currently always 0
        prop_assert!(
            code.contains("large_state_count : 0u32"),
            "large_state_count : 0u32 not found"
        );
    }

    // 28. Production count field present
    #[test]
    fn production_count_field_present(
        (terms, nonterms, fields, externals, states) in layout_dims()
    ) {
        let code = gen_code("pc", terms, nonterms, fields, externals, states);
        prop_assert!(
            code.contains("production_count :"),
            "production_count field missing from Language struct"
        );
    }

    // 29. Rule count field present
    #[test]
    fn rule_count_field_present(
        (terms, nonterms, fields, externals, states) in layout_dims()
    ) {
        let code = gen_code("rc", terms, nonterms, fields, externals, states);
        prop_assert!(
            code.contains("rule_count :"),
            "rule_count field missing from Language struct"
        );
    }

    // 30. Alias count is zero
    #[test]
    fn alias_count_is_zero(
        (terms, nonterms, fields, externals, states) in layout_dims()
    ) {
        let code = gen_code("ac", terms, nonterms, fields, externals, states);
        prop_assert!(
            code.contains("alias_count : 0u32"),
            "alias_count must be 0u32 (aliases not yet implemented)"
        );
    }

    // 31. Max alias sequence length is zero
    #[test]
    fn max_alias_sequence_length_is_zero(
        (terms, nonterms, fields, externals, states) in layout_dims()
    ) {
        let code = gen_code("masl", terms, nonterms, fields, externals, states);
        prop_assert!(
            code.contains("max_alias_sequence_length : 0u16"),
            "max_alias_sequence_length must be 0u16"
        );
    }
}
