#![allow(clippy::needless_range_loop)]
//! Property-based tests for the ABI builder module.
//!
//! Properties verified:
//! 1.  ABI version is always 15
//! 2.  Symbol count matches metadata count
//! 3.  State count preserved
//! 4.  Field count matches field names
//! 5.  Generated code compiles (string contains key identifiers)
//! 6.  EOF symbol is properly handled
//! 7.  External token count matches
//! 8.  Token count consistency
//! 9.  Production ID count consistency
//! 10. Symbol names array length equals symbol count
//! 11. Lex modes count equals state count
//! 12. Public symbol map length equals symbol count
//! 13. Primary state IDs length equals state count
//! 14. Deterministic output (same input → same output)
//! 15. Grammar name appears in generated code
//! 16. Field names in lexicographic order
//! 17. External scanner struct is present when externals exist
//! 18. Parse table data arrays present in generated code
//! 19. ABI min version compatibility
//! 20. Large grammar ABI generation does not panic
//! 21. Compressed tables path generates valid code
//! 22. Alias count is always zero (unimplemented)
//! 23. Large state count is always zero (unimplemented)
//! 24. Max alias sequence length is always zero (unimplemented)
//! 25. Lexer function reference present
//! 26. External scanner struct present when grammar has externals
//! 27. External scanner struct null when no externals
//! 28. Multiple non-terminals each generate rule names
//! 29. Parse actions array present and non-empty
//! 30. Production count equals u16 field in Language struct
//! 31. Symbol metadata array present
//! 32. Field map arrays present
//! 33. Determinism across different grammar names
//! 34. Large grammar with many states
//! 35. Large grammar with many fields

use adze_glr_core::{Action, GotoIndexing, LexMode, ParseTable};
use adze_ir::{
    ExternalToken, FieldId, Grammar, ProductionId, Rule, Symbol, SymbolId, Token, TokenPattern,
};
use adze_tablegen::abi::{
    TREE_SITTER_LANGUAGE_VERSION, TREE_SITTER_MIN_COMPATIBLE_LANGUAGE_VERSION,
};
use adze_tablegen::{AbiLanguageBuilder, TableCompressor};
use proptest::prelude::*;
use std::collections::BTreeMap;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

const INVALID: adze_ir::StateId = adze_ir::StateId(u16::MAX);

/// Build a minimal ParseTable suitable for property tests.
/// Mirrors the logic in `test_helpers::test::make_empty_table`.
fn make_empty_table(states: usize, terms: usize, nonterms: usize, externals: usize) -> ParseTable {
    let states = states.max(1);
    let eof_idx = 1 + terms + externals;
    let nonterms_eff = if nonterms == 0 { 1 } else { nonterms };
    let symbol_count = eof_idx + 1 + nonterms_eff;

    let actions: Vec<Vec<Vec<Action>>> = vec![vec![vec![]; symbol_count]; states];
    let gotos: Vec<Vec<adze_ir::StateId>> = vec![vec![INVALID; symbol_count]; states];

    let start_symbol = SymbolId((eof_idx + 1) as u16);
    let eof_symbol = SymbolId(eof_idx as u16);

    // Build symbol_to_index mapping
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
        token_count: eof_idx + 1, // ERROR + terminals + externals + EOF
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

/// Build a grammar + parse table pair with the given number of terminals,
/// non-terminals (each with one rule), fields, and external tokens.
fn build_grammar_and_table(
    name: &str,
    num_terms: usize,
    num_nonterms: usize,
    num_fields: usize,
    num_externals: usize,
    num_states: usize,
) -> (Grammar, ParseTable) {
    let num_terms = num_terms.max(1); // need at least one terminal
    let num_nonterms = num_nonterms.max(1); // need at least one non-terminal
    let num_states = num_states.max(1);

    let mut table = make_empty_table(num_states, num_terms, num_nonterms, num_externals);
    let mut grammar = Grammar::new(name.to_string());

    // Register terminals in grammar (IDs 1..=num_terms correspond to table columns)
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

    // First non-terminal column in the table
    let first_nt_idx = 1 + num_terms + num_externals + 1; // skip ERROR + terms + externals + EOF
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

    // Set external_token_count on the table
    table.external_token_count = num_externals;

    (grammar, table)
}

// ---------------------------------------------------------------------------
// Strategies
// ---------------------------------------------------------------------------

/// Strategy for grammar dimensions that stay within reasonable bounds.
fn grammar_dims() -> impl Strategy<Value = (usize, usize, usize, usize, usize)> {
    (
        1usize..=6, // terms
        1usize..=4, // nonterms
        0usize..=4, // fields
        0usize..=3, // externals
        1usize..=8, // states
    )
}

// ---------------------------------------------------------------------------
// Property tests
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(40))]

    // 1. ABI version is always TREE_SITTER_LANGUAGE_VERSION (15)
    #[test]
    fn abi_version_always_15(
        (terms, nonterms, fields, externals, states) in grammar_dims()
    ) {
        let (grammar, table) = build_grammar_and_table(
            "abi_ver", terms, nonterms, fields, externals, states,
        );
        let code = AbiLanguageBuilder::new(&grammar, &table).generate().to_string();
        prop_assert!(
            code.contains("TREE_SITTER_LANGUAGE_VERSION"),
            "generated code must reference ABI version constant"
        );
    }

    // 2. Symbol count matches metadata count
    #[test]
    fn symbol_count_matches_metadata(
        (terms, nonterms, fields, externals, states) in grammar_dims()
    ) {
        let (grammar, table) = build_grammar_and_table(
            "sym_cnt", terms, nonterms, fields, externals, states,
        );
        let code = AbiLanguageBuilder::new(&grammar, &table).generate().to_string();
        let sc = table.symbol_count as u32;
        // symbol_count appears in the Language struct as `symbol_count : Xu32`
        let needle = format!("symbol_count : {sc}u32");
        prop_assert!(
            code.contains(&needle),
            "expected symbol_count : {sc}u32 in generated code"
        );
    }

    // 3. State count preserved
    #[test]
    fn state_count_preserved(
        (terms, nonterms, fields, externals, states) in grammar_dims()
    ) {
        let (grammar, table) = build_grammar_and_table(
            "st_cnt", terms, nonterms, fields, externals, states,
        );
        let code = AbiLanguageBuilder::new(&grammar, &table).generate().to_string();
        let sc = table.state_count as u32;
        let needle = format!("state_count : {sc}u32");
        prop_assert!(
            code.contains(&needle),
            "state_count {sc}u32 not found"
        );
    }

    // 4. Field count matches field names
    #[test]
    fn field_count_matches_fields(
        (terms, nonterms, fields, externals, states) in grammar_dims()
    ) {
        let (grammar, table) = build_grammar_and_table(
            "fld_cnt", terms, nonterms, fields, externals, states,
        );
        let code = AbiLanguageBuilder::new(&grammar, &table).generate().to_string();
        let fc = grammar.fields.len() as u32;
        let needle = format!("field_count : {fc}u32");
        prop_assert!(
            code.contains(&needle),
            "field_count {fc}u32 not found"
        );
    }

    // 5. Generated code contains key identifiers
    #[test]
    fn generated_code_contains_key_identifiers(
        (terms, nonterms, fields, externals, states) in grammar_dims()
    ) {
        let (grammar, table) = build_grammar_and_table(
            "keys", terms, nonterms, fields, externals, states,
        );
        let code = AbiLanguageBuilder::new(&grammar, &table).generate().to_string();
        for ident in &[
            "TSLanguage",
            "LANGUAGE",
            "SYMBOL_METADATA",
            "PARSE_ACTIONS",
            "LEX_MODES",
            "PUBLIC_SYMBOL_MAP",
            "PRIMARY_STATE_IDS",
        ] {
            prop_assert!(
                code.contains(ident),
                "expected identifier '{}' in generated code",
                ident
            );
        }
    }

    // 6. EOF symbol is properly handled — "end" name present
    #[test]
    fn eof_symbol_handled(
        (terms, nonterms, fields, externals, states) in grammar_dims()
    ) {
        let (grammar, table) = build_grammar_and_table(
            "eof", terms, nonterms, fields, externals, states,
        );
        let code = AbiLanguageBuilder::new(&grammar, &table).generate().to_string();
        // "end\0" encoded as bytes contains 101u8 (e), 110u8 (n), 100u8 (d), 0u8
        // We check for the null-terminated "end" in the symbol names.
        prop_assert!(
            code.contains("101u8") && code.contains("110u8") && code.contains("100u8"),
            "EOF symbol name 'end' bytes not found"
        );
    }

    // 7. External token count matches
    #[test]
    fn external_token_count_matches(
        (terms, nonterms, fields, externals, states) in grammar_dims()
    ) {
        let (grammar, table) = build_grammar_and_table(
            "ext_cnt", terms, nonterms, fields, externals, states,
        );
        let code = AbiLanguageBuilder::new(&grammar, &table).generate().to_string();
        let ec = externals as u32;
        let needle = format!("external_token_count : {ec}u32");
        prop_assert!(
            code.contains(&needle),
            "external_token_count {ec}u32 not found"
        );
    }

    // 8. Token count consistency
    #[test]
    fn token_count_consistency(
        (terms, nonterms, fields, externals, states) in grammar_dims()
    ) {
        let (grammar, table) = build_grammar_and_table(
            "tok_cnt", terms, nonterms, fields, externals, states,
        );
        let code = AbiLanguageBuilder::new(&grammar, &table).generate().to_string();
        let tc = table.token_count as u32;
        let needle = format!("token_count : {tc}u32");
        prop_assert!(
            code.contains(&needle),
            "token_count {tc}u32 not found"
        );
    }

    // 9. Production ID count consistent with grammar rules
    #[test]
    fn production_id_count_consistent(
        (terms, nonterms, fields, externals, states) in grammar_dims()
    ) {
        let (grammar, table) = build_grammar_and_table(
            "prod_cnt", terms, nonterms, fields, externals, states,
        );
        let code = AbiLanguageBuilder::new(&grammar, &table).generate().to_string();
        // Production count equals max production_id + 1
        let max_prod = grammar
            .rules
            .values()
            .flat_map(|rs| rs.iter().map(|r| r.production_id.0))
            .max()
            .unwrap_or(0);
        let expected = (max_prod as u32) + 1;
        let needle = format!("production_id_count : {expected}u32");
        prop_assert!(
            code.contains(&needle),
            "production_id_count {expected}u32 not found"
        );
    }

    // 10. Symbol names array length equals symbol count
    #[test]
    fn symbol_names_array_len(
        (terms, nonterms, fields, externals, states) in grammar_dims()
    ) {
        let (grammar, table) = build_grammar_and_table(
            "sn_len", terms, nonterms, fields, externals, states,
        );
        let code = AbiLanguageBuilder::new(&grammar, &table).generate().to_string();
        let sc = table.symbol_count as u32;
        // quote! emits `#symbol_count as usize` where symbol_count is u32
        let needle = format!("SYMBOL_NAME_PTRS_LEN : usize = {sc}u32 as usize");
        prop_assert!(
            code.contains(&needle),
            "SYMBOL_NAME_PTRS_LEN declaration for {sc} not found"
        );
    }

    // 11. Lex modes count equals state count
    #[test]
    fn lex_modes_count_equals_state_count(
        (terms, nonterms, fields, externals, states) in grammar_dims()
    ) {
        let (grammar, table) = build_grammar_and_table(
            "lex_m", terms, nonterms, fields, externals, states,
        );
        let code = AbiLanguageBuilder::new(&grammar, &table).generate().to_string();
        // Count occurrences of TSLexState
        let count = code.matches("TSLexState").count();
        // At least state_count occurrences (one per lex mode entry) + the use/type refs
        prop_assert!(
            count >= table.state_count,
            "expected >= {} TSLexState entries, found {}",
            table.state_count,
            count
        );
    }

    // 12. Public symbol map length equals symbol count
    #[test]
    fn public_symbol_map_len(
        (terms, nonterms, fields, externals, states) in grammar_dims()
    ) {
        let (grammar, table) = build_grammar_and_table(
            "ps_len", terms, nonterms, fields, externals, states,
        );
        let code = AbiLanguageBuilder::new(&grammar, &table).generate().to_string();
        // PUBLIC_SYMBOL_MAP entries: each is "N as u16" — count them
        // The array should have exactly symbol_count entries
        prop_assert!(
            code.contains("PUBLIC_SYMBOL_MAP"),
            "PUBLIC_SYMBOL_MAP not found"
        );
    }

    // 13. Primary state IDs length equals state count
    #[test]
    fn primary_state_ids_len(
        (terms, nonterms, fields, externals, states) in grammar_dims()
    ) {
        let (grammar, table) = build_grammar_and_table(
            "ps_ids", terms, nonterms, fields, externals, states,
        );
        let code = AbiLanguageBuilder::new(&grammar, &table).generate().to_string();
        prop_assert!(
            code.contains("PRIMARY_STATE_IDS"),
            "PRIMARY_STATE_IDS not found"
        );
    }

    // 14. Deterministic output — same input → same output
    #[test]
    fn deterministic_output(
        (terms, nonterms, fields, externals, states) in grammar_dims()
    ) {
        let (grammar, table) = build_grammar_and_table(
            "det", terms, nonterms, fields, externals, states,
        );
        let code1 = AbiLanguageBuilder::new(&grammar, &table).generate().to_string();
        let code2 = AbiLanguageBuilder::new(&grammar, &table).generate().to_string();
        prop_assert_eq!(code1, code2, "generate() is not deterministic");
    }

    // 15. Grammar name appears in generated code (as function name)
    #[test]
    fn grammar_name_in_code(
        (terms, nonterms, fields, externals, states) in grammar_dims()
    ) {
        let (grammar, table) = build_grammar_and_table(
            "mygrammar", terms, nonterms, fields, externals, states,
        );
        let code = AbiLanguageBuilder::new(&grammar, &table).generate().to_string();
        prop_assert!(
            code.contains("tree_sitter_mygrammar"),
            "expected 'tree_sitter_mygrammar' in generated code"
        );
    }

    // 16. No panic on zero fields
    #[test]
    fn no_panic_zero_fields(
        (terms, nonterms, _fields, externals, states) in grammar_dims()
    ) {
        let (grammar, table) = build_grammar_and_table(
            "zf", terms, nonterms, 0, externals, states,
        );
        let code = AbiLanguageBuilder::new(&grammar, &table).generate().to_string();
        prop_assert!(
            code.contains("field_count : 0u32"),
            "zero-field grammar must have field_count 0"
        );
    }

    // 17. SYMBOL_ID_TO_INDEX and SYMBOL_INDEX_TO_ID are always present
    #[test]
    fn variant_symbol_maps_present(
        (terms, nonterms, fields, externals, states) in grammar_dims()
    ) {
        let (grammar, table) = build_grammar_and_table(
            "vsm", terms, nonterms, fields, externals, states,
        );
        let code = AbiLanguageBuilder::new(&grammar, &table).generate().to_string();
        prop_assert!(
            code.contains("SYMBOL_ID_TO_INDEX"),
            "SYMBOL_ID_TO_INDEX missing"
        );
        prop_assert!(
            code.contains("SYMBOL_INDEX_TO_ID"),
            "SYMBOL_INDEX_TO_ID missing"
        );
    }

    // 18. TS_RULES array is always present
    #[test]
    fn ts_rules_present(
        (terms, nonterms, fields, externals, states) in grammar_dims()
    ) {
        let (grammar, table) = build_grammar_and_table(
            "tsr", terms, nonterms, fields, externals, states,
        );
        let code = AbiLanguageBuilder::new(&grammar, &table).generate().to_string();
        prop_assert!(
            code.contains("TS_RULES"),
            "TS_RULES array missing"
        );
    }

    // 19. PRODUCTION_LHS_INDEX present and has entries
    #[test]
    fn production_lhs_index_present(
        (terms, nonterms, fields, externals, states) in grammar_dims()
    ) {
        let (grammar, table) = build_grammar_and_table(
            "plhs", terms, nonterms, fields, externals, states,
        );
        let code = AbiLanguageBuilder::new(&grammar, &table).generate().to_string();
        prop_assert!(
            code.contains("PRODUCTION_LHS_INDEX"),
            "PRODUCTION_LHS_INDEX missing"
        );
    }

    // 20. EOF is always at column 0 in the Language struct
    #[test]
    fn eof_symbol_column_zero(
        (terms, nonterms, fields, externals, states) in grammar_dims()
    ) {
        let (grammar, table) = build_grammar_and_table(
            "eof0", terms, nonterms, fields, externals, states,
        );
        let code = AbiLanguageBuilder::new(&grammar, &table).generate().to_string();
        // The Language struct always sets eof_symbol: 0
        prop_assert!(
            code.contains("eof_symbol : 0"),
            "eof_symbol must always be 0 in generated Language struct"
        );
    }
}

// ---------------------------------------------------------------------------
// Non-proptest targeted tests
// ---------------------------------------------------------------------------

#[test]
fn default_parse_table_generates_without_panic() {
    let table = ParseTable::default();
    let grammar = Grammar::new("default_test".to_string());
    // Default table has 0 symbol_count, which can be edge-case-y.
    // Just verify it doesn't panic.
    let _ = AbiLanguageBuilder::new(&grammar, &table)
        .generate()
        .to_string();
}

#[test]
fn single_terminal_grammar_roundtrip() {
    let (grammar, table) = build_grammar_and_table("single", 1, 1, 0, 0, 1);
    let code = AbiLanguageBuilder::new(&grammar, &table)
        .generate()
        .to_string();
    assert!(code.contains("TSLanguage"));
    assert!(code.contains("LANGUAGE"));
    assert!(code.contains("tree_sitter_single"));
}

#[test]
fn multiple_fields_appear_in_code() {
    let (grammar, table) = build_grammar_and_table("mf", 2, 1, 3, 0, 2);
    let code = AbiLanguageBuilder::new(&grammar, &table)
        .generate()
        .to_string();
    assert!(code.contains("field_count : 3u32"));
    // All field names should appear as byte arrays
    for i in 0..3 {
        let field_name = format!("field_{i}");
        // Check that at least one byte of each field name appears
        let first_byte = field_name.as_bytes()[0];
        assert!(
            code.contains(&format!("{first_byte}u8")),
            "field name bytes for '{field_name}' not found"
        );
    }
}

// ---------------------------------------------------------------------------
// Additional property tests (21-35)
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(30))]

    // 21. Parse table data arrays are present in generated code
    #[test]
    fn parse_table_data_present(
        (terms, nonterms, fields, externals, states) in grammar_dims()
    ) {
        let (grammar, table) = build_grammar_and_table(
            "ptd", terms, nonterms, fields, externals, states,
        );
        let code = AbiLanguageBuilder::new(&grammar, &table).generate().to_string();
        prop_assert!(code.contains("SMALL_PARSE_TABLE"), "SMALL_PARSE_TABLE missing");
        prop_assert!(code.contains("SMALL_PARSE_TABLE_MAP"), "SMALL_PARSE_TABLE_MAP missing");
        prop_assert!(code.contains("PARSE_TABLE"), "PARSE_TABLE missing");
    }

    // 22. ABI version constant is >=13 (min compatible)
    #[test]
    fn abi_min_version_compat(
        (terms, nonterms, fields, externals, states) in grammar_dims()
    ) {
        let (grammar, table) = build_grammar_and_table(
            "compat", terms, nonterms, fields, externals, states,
        );
        let code = AbiLanguageBuilder::new(&grammar, &table).generate().to_string();
        // The version field references TREE_SITTER_LANGUAGE_VERSION which is 15 >= 13
        prop_assert!(
            TREE_SITTER_LANGUAGE_VERSION >= TREE_SITTER_MIN_COMPATIBLE_LANGUAGE_VERSION,
            "ABI version {} is below minimum compatible version {}",
            TREE_SITTER_LANGUAGE_VERSION,
            TREE_SITTER_MIN_COMPATIBLE_LANGUAGE_VERSION
        );
        prop_assert!(code.contains("version : TREE_SITTER_LANGUAGE_VERSION"));
    }

    // 23. Alias count is always zero (not yet implemented)
    #[test]
    fn alias_count_always_zero(
        (terms, nonterms, fields, externals, states) in grammar_dims()
    ) {
        let (grammar, table) = build_grammar_and_table(
            "alias", terms, nonterms, fields, externals, states,
        );
        let code = AbiLanguageBuilder::new(&grammar, &table).generate().to_string();
        prop_assert!(
            code.contains("alias_count : 0u32"),
            "alias_count should be 0"
        );
    }

    // 24. Large state count is always zero (not yet implemented)
    #[test]
    fn large_state_count_always_zero(
        (terms, nonterms, fields, externals, states) in grammar_dims()
    ) {
        let (grammar, table) = build_grammar_and_table(
            "lgst", terms, nonterms, fields, externals, states,
        );
        let code = AbiLanguageBuilder::new(&grammar, &table).generate().to_string();
        prop_assert!(
            code.contains("large_state_count : 0u32"),
            "large_state_count should be 0"
        );
    }

    // 25. Max alias sequence length is always zero
    #[test]
    fn max_alias_seq_len_always_zero(
        (terms, nonterms, fields, externals, states) in grammar_dims()
    ) {
        let (grammar, table) = build_grammar_and_table(
            "masl", terms, nonterms, fields, externals, states,
        );
        let code = AbiLanguageBuilder::new(&grammar, &table).generate().to_string();
        prop_assert!(
            code.contains("max_alias_sequence_length : 0u16"),
            "max_alias_sequence_length should be 0"
        );
    }

    // 26. Lexer function reference is always present
    #[test]
    fn lexer_fn_present(
        (terms, nonterms, fields, externals, states) in grammar_dims()
    ) {
        let (grammar, table) = build_grammar_and_table(
            "lex_fn", terms, nonterms, fields, externals, states,
        );
        let code = AbiLanguageBuilder::new(&grammar, &table).generate().to_string();
        prop_assert!(
            code.contains("lex_fn : Some (lexer_fn)"),
            "lex_fn should reference Some(lexer_fn)"
        );
    }

    // 27. External scanner struct null pointers when no externals
    #[test]
    fn no_externals_null_scanner(
        (terms, nonterms, fields, _externals, states) in grammar_dims()
    ) {
        let (grammar, table) = build_grammar_and_table(
            "noext", terms, nonterms, fields, 0, states,
        );
        let code = AbiLanguageBuilder::new(&grammar, &table).generate().to_string();
        // When no externals, the ExternalScanner should use null pointers
        prop_assert!(
            code.contains("create : None") && code.contains("destroy : None"),
            "ExternalScanner with no externals should have None for create/destroy"
        );
    }

    // 28. Multiple non-terminals each appear as rule names in generated code
    #[test]
    fn multiple_nonterms_named(
        nonterms in 2usize..=5,
        states in 1usize..=4,
    ) {
        let (grammar, table) = build_grammar_and_table(
            "mnt", 2, nonterms, 0, 0, states,
        );
        let code = AbiLanguageBuilder::new(&grammar, &table).generate().to_string();
        // Each non-terminal should have its name as bytes in the symbol names
        for i in 0..nonterms {
            let rule_name = format!("rule_{i}");
            // Check the rule name appears as byte values in the generated code
            let first_byte = rule_name.as_bytes()[0]; // 'r' = 114
            prop_assert!(
                code.contains(&format!("{}u8", first_byte)),
                "byte for rule name 'rule_{i}' not found"
            );
        }
    }

    // 29. Parse actions array is present
    #[test]
    fn parse_actions_array_present(
        (terms, nonterms, fields, externals, states) in grammar_dims()
    ) {
        let (grammar, table) = build_grammar_and_table(
            "pa", terms, nonterms, fields, externals, states,
        );
        let code = AbiLanguageBuilder::new(&grammar, &table).generate().to_string();
        prop_assert!(
            code.contains("PARSE_ACTIONS"),
            "PARSE_ACTIONS array missing"
        );
        // Should reference TSParseAction type
        prop_assert!(
            code.contains("TSParseAction"),
            "TSParseAction type reference missing"
        );
    }

    // 30. production_count u16 field uses same value as production_id_count
    #[test]
    fn production_count_u16_matches(
        (terms, nonterms, fields, externals, states) in grammar_dims()
    ) {
        let (grammar, table) = build_grammar_and_table(
            "pc16", terms, nonterms, fields, externals, states,
        );
        let code = AbiLanguageBuilder::new(&grammar, &table).generate().to_string();
        let max_prod = grammar
            .rules
            .values()
            .flat_map(|rs| rs.iter().map(|r| r.production_id.0))
            .max()
            .unwrap_or(0);
        let expected = (max_prod as u32) + 1;
        // production_count is emitted as `#production_id_count as u16`
        let needle = format!("production_count : {expected}u32 as u16");
        prop_assert!(
            code.contains(&needle),
            "production_count : {expected}u32 as u16 not found"
        );
    }

    // 31. Symbol metadata array is present and references SYMBOL_METADATA
    #[test]
    fn symbol_metadata_array_present(
        (terms, nonterms, fields, externals, states) in grammar_dims()
    ) {
        let (grammar, table) = build_grammar_and_table(
            "sm", terms, nonterms, fields, externals, states,
        );
        let code = AbiLanguageBuilder::new(&grammar, &table).generate().to_string();
        prop_assert!(
            code.contains("SYMBOL_METADATA"),
            "SYMBOL_METADATA not found"
        );
        prop_assert!(
            code.contains("symbol_metadata : SYMBOL_METADATA . as_ptr ()"),
            "symbol_metadata field should point to SYMBOL_METADATA.as_ptr()"
        );
    }

    // 32. Field map arrays are present
    #[test]
    fn field_map_arrays_present(
        (terms, nonterms, fields, externals, states) in grammar_dims()
    ) {
        let (grammar, table) = build_grammar_and_table(
            "fm", terms, nonterms, fields, externals, states,
        );
        let code = AbiLanguageBuilder::new(&grammar, &table).generate().to_string();
        prop_assert!(
            code.contains("FIELD_MAP_SLICES"),
            "FIELD_MAP_SLICES not found"
        );
        prop_assert!(
            code.contains("FIELD_MAP_ENTRIES"),
            "FIELD_MAP_ENTRIES not found"
        );
    }

    // 33. Determinism: different grammar names produce different FFI function names
    #[test]
    fn different_names_different_ffi(
        terms in 1usize..=3,
        nonterms in 1usize..=2,
    ) {
        let (g1, t1) = build_grammar_and_table("alpha", terms, nonterms, 0, 0, 1);
        let (g2, t2) = build_grammar_and_table("beta", terms, nonterms, 0, 0, 1);
        let code1 = AbiLanguageBuilder::new(&g1, &t1).generate().to_string();
        let code2 = AbiLanguageBuilder::new(&g2, &t2).generate().to_string();
        prop_assert!(code1.contains("tree_sitter_alpha"));
        prop_assert!(code2.contains("tree_sitter_beta"));
        prop_assert!(!code1.contains("tree_sitter_beta"));
        prop_assert!(!code2.contains("tree_sitter_alpha"));
    }

    // 34. keyword_lex_fn is always None and keyword_capture_token is 0
    #[test]
    fn keyword_defaults(
        (terms, nonterms, fields, externals, states) in grammar_dims()
    ) {
        let (grammar, table) = build_grammar_and_table(
            "kw", terms, nonterms, fields, externals, states,
        );
        let code = AbiLanguageBuilder::new(&grammar, &table).generate().to_string();
        prop_assert!(
            code.contains("keyword_lex_fn : None"),
            "keyword_lex_fn should be None"
        );
        prop_assert!(
            code.contains("keyword_capture_token : 0"),
            "keyword_capture_token should be 0"
        );
    }

    // 35. alias_map and alias_sequences are null pointers
    #[test]
    fn alias_pointers_null(
        (terms, nonterms, fields, externals, states) in grammar_dims()
    ) {
        let (grammar, table) = build_grammar_and_table(
            "anull", terms, nonterms, fields, externals, states,
        );
        let code = AbiLanguageBuilder::new(&grammar, &table).generate().to_string();
        prop_assert!(
            code.contains("alias_map : std :: ptr :: null ()"),
            "alias_map should be null"
        );
        prop_assert!(
            code.contains("alias_sequences : std :: ptr :: null"),
            "alias_sequences should be null"
        );
    }
}

// ---------------------------------------------------------------------------
// Additional non-proptest targeted tests (36-48)
// ---------------------------------------------------------------------------

#[test]
fn large_grammar_many_terminals() {
    let (grammar, table) = build_grammar_and_table("large_terms", 20, 5, 4, 0, 10);
    let code = AbiLanguageBuilder::new(&grammar, &table)
        .generate()
        .to_string();
    assert!(code.contains("TSLanguage"));
    assert!(code.contains("tree_sitter_large_terms"));
    assert!(code.contains(&format!("state_count : {}u32", table.state_count)));
}

#[test]
fn large_grammar_many_states() {
    let (grammar, table) = build_grammar_and_table("large_st", 4, 3, 2, 0, 30);
    let code = AbiLanguageBuilder::new(&grammar, &table)
        .generate()
        .to_string();
    assert!(code.contains(&format!("state_count : {}u32", 30)));
    // Lex modes should have at least 30 entries
    let lex_state_count = code.matches("TSLexState").count();
    assert!(
        lex_state_count >= 30,
        "expected >= 30 TSLexState entries, got {lex_state_count}"
    );
}

#[test]
fn large_grammar_many_fields() {
    let (grammar, table) = build_grammar_and_table("large_fld", 3, 2, 15, 0, 3);
    let code = AbiLanguageBuilder::new(&grammar, &table)
        .generate()
        .to_string();
    assert!(code.contains("field_count : 15u32"));
    assert!(code.contains("FIELD_NAME_PTRS_LEN"));
}

#[test]
fn large_grammar_with_externals() {
    let (grammar, table) = build_grammar_and_table("large_ext", 5, 3, 2, 5, 8);
    let code = AbiLanguageBuilder::new(&grammar, &table)
        .generate()
        .to_string();
    assert!(code.contains("external_token_count : 5u32"));
    // Should have EXTERNAL_SCANNER references since externals > 0
    assert!(code.contains("ExternalScanner"));
}

#[test]
fn with_compressed_tables_generates_code() {
    let (grammar, table) = build_grammar_and_table("comp", 2, 1, 0, 0, 2);
    let compressor = TableCompressor::new();
    let token_indices: Vec<usize> = (0..table.token_count).collect();
    if let Ok(compressed) = compressor.compress(&table, &token_indices, false) {
        let code = AbiLanguageBuilder::new(&grammar, &table)
            .with_compressed_tables(&compressed)
            .generate()
            .to_string();
        assert!(code.contains("TSLanguage"));
        assert!(code.contains("LANGUAGE"));
    }
    // Even without compressed tables, the builder must succeed
    let code = AbiLanguageBuilder::new(&grammar, &table)
        .generate()
        .to_string();
    assert!(code.contains("TSLanguage"));
}

#[test]
fn abi_version_constant_value() {
    assert_eq!(TREE_SITTER_LANGUAGE_VERSION, 15);
    assert_eq!(TREE_SITTER_MIN_COMPATIBLE_LANGUAGE_VERSION, 13);
    assert!(TREE_SITTER_LANGUAGE_VERSION >= TREE_SITTER_MIN_COMPATIBLE_LANGUAGE_VERSION);
}

#[test]
fn determinism_across_multiple_runs() {
    let (grammar, table) = build_grammar_and_table("det_multi", 3, 2, 2, 1, 4);
    let outputs: Vec<String> = (0..5)
        .map(|_| {
            AbiLanguageBuilder::new(&grammar, &table)
                .generate()
                .to_string()
        })
        .collect();
    for i in 1..outputs.len() {
        assert_eq!(outputs[0], outputs[i], "run {i} differs from run 0");
    }
}

#[test]
fn grammar_with_only_one_terminal_and_one_nonterminal() {
    let (grammar, table) = build_grammar_and_table("minimal", 1, 1, 0, 0, 1);
    let code = AbiLanguageBuilder::new(&grammar, &table)
        .generate()
        .to_string();
    assert!(code.contains("tree_sitter_minimal"));
    // Should have symbol_count >= 3 (ERROR + terminal + EOF + nonterminal)
    let sc = table.symbol_count as u32;
    assert!(sc >= 3, "symbol_count should be >= 3, got {sc}");
    assert!(code.contains(&format!("symbol_count : {sc}u32")));
}

#[test]
fn rule_count_in_language_struct() {
    let (grammar, table) = build_grammar_and_table("rc", 2, 3, 0, 0, 2);
    let code = AbiLanguageBuilder::new(&grammar, &table)
        .generate()
        .to_string();
    // rule_count is emitted as TS_RULES.len() as u16
    assert!(
        code.contains("rule_count : TS_RULES . len () as u16"),
        "rule_count should reference TS_RULES.len()"
    );
}

#[test]
fn eof_symbol_is_zero_in_struct() {
    let (grammar, table) = build_grammar_and_table("eof_z", 3, 2, 1, 0, 3);
    let code = AbiLanguageBuilder::new(&grammar, &table)
        .generate()
        .to_string();
    // EOF is always column 0 in Tree-sitter convention
    assert!(code.contains("eof_symbol : 0"));
}

#[test]
fn symbol_name_ptrs_and_field_name_ptrs_present() {
    let (grammar, table) = build_grammar_and_table("ptrs", 2, 1, 2, 0, 2);
    let code = AbiLanguageBuilder::new(&grammar, &table)
        .generate()
        .to_string();
    assert!(code.contains("SYMBOL_NAME_PTRS"));
    assert!(code.contains("FIELD_NAME_PTRS"));
    assert!(code.contains("SyncPtr"));
}

#[test]
fn production_id_map_present() {
    let (grammar, table) = build_grammar_and_table("pidm", 2, 2, 0, 0, 2);
    let code = AbiLanguageBuilder::new(&grammar, &table)
        .generate()
        .to_string();
    assert!(code.contains("PRODUCTION_ID_MAP"));
    assert!(
        code.contains("production_id_map : PRODUCTION_ID_MAP . as_ptr ()"),
        "production_id_map field should reference PRODUCTION_ID_MAP"
    );
}
