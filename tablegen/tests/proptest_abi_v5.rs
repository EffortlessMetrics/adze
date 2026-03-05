#![allow(clippy::needless_range_loop)]
//! Property-based tests for ABI v5 invariants across the adze-tablegen pipeline.
//!
//! 40+ proptest properties covering:
//! - Generated code determinism (same inputs → identical outputs)
//! - Compression properties (roundtrip, deduplication, sparsity)
//! - Node types consistency (valid JSON, correct structure)
//! - ABI builder invariants (symbol/state/field counts, version)

use adze_glr_core::{Action, GotoIndexing, LexMode, ParseTable};
use adze_ir::builder::GrammarBuilder;
use adze_ir::{
    ExternalToken, FieldId, Grammar, ProductionId, Rule, RuleId, StateId, Symbol, SymbolId, Token,
    TokenPattern,
};
use adze_tablegen::compression::{
    BitPackedActionTable, compress_action_table, compress_goto_table, decompress_action,
    decompress_goto,
};
use adze_tablegen::{
    AbiLanguageBuilder, CompressedParseTable, NodeTypesGenerator, StaticLanguageGenerator,
};
use proptest::prelude::*;
use std::collections::BTreeMap;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

const INVALID: StateId = StateId(u16::MAX);

/// Build a minimal ParseTable from dimensions.
fn make_table(states: usize, terms: usize, nonterms: usize, externals: usize) -> ParseTable {
    let states = states.max(1);
    let nonterms = nonterms.max(1);
    let eof_idx = 1 + terms + externals;
    let symbol_count = eof_idx + 1 + nonterms;

    let actions = vec![vec![vec![]; symbol_count]; states];
    let gotos = vec![vec![INVALID; symbol_count]; states];

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
        initial_state: StateId(0),
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

    let mut table = make_table(num_states, num_terms, num_nonterms, num_externals);
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

    let first_nt_idx = 1 + num_terms + num_externals + 1;
    let first_term = SymbolId(1);

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

    for i in 0..num_fields {
        grammar
            .fields
            .insert(FieldId(i as u16), format!("field_{i}"));
    }

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

fn small_dims() -> impl Strategy<Value = (usize, usize, usize)> {
    (2usize..=4, 1usize..=2, 1usize..=4)
}

fn grammar_name() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9_]{1,8}"
}

fn action_strategy() -> impl Strategy<Value = Action> {
    prop_oneof![
        3 => Just(Action::Error),
        2 => (1u16..100).prop_map(|s| Action::Shift(StateId(s))),
        2 => (0u16..50).prop_map(|r| Action::Reduce(RuleId(r))),
        1 => Just(Action::Accept),
    ]
}

fn action_cell_strategy() -> impl Strategy<Value = Vec<Action>> {
    prop::collection::vec(action_strategy(), 0..=3)
}

fn action_table_strategy(
    max_states: usize,
    max_symbols: usize,
) -> impl Strategy<Value = Vec<Vec<Vec<Action>>>> {
    (1..=max_states, 1..=max_symbols).prop_flat_map(|(states, symbols)| {
        prop::collection::vec(
            prop::collection::vec(action_cell_strategy(), symbols..=symbols),
            states..=states,
        )
    })
}

fn goto_table_strategy(
    max_states: usize,
    max_symbols: usize,
) -> impl Strategy<Value = Vec<Vec<Option<StateId>>>> {
    (1..=max_states, 1..=max_symbols).prop_flat_map(|(states, symbols)| {
        prop::collection::vec(
            prop::collection::vec(
                prop_oneof![
                    3 => Just(None),
                    1 => (0u16..100).prop_map(|s| Some(StateId(s))),
                ],
                symbols..=symbols,
            ),
            states..=states,
        )
    })
}

// ===========================================================================
// 1. Generated code determinism (properties 1–7)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(25))]

    /// P1: Same grammar+table always yields identical ABI code.
    #[test]
    fn deterministic_abi_output(
        (terms, nonterms, fields, externals, states) in grammar_dims()
    ) {
        let (grammar, table) = build_grammar_and_table(
            "det", terms, nonterms, fields, externals, states,
        );
        let a = AbiLanguageBuilder::new(&grammar, &table).generate().to_string();
        let b = AbiLanguageBuilder::new(&grammar, &table).generate().to_string();
        prop_assert_eq!(a, b);
    }

    /// P2: Three consecutive builds produce identical output.
    #[test]
    fn deterministic_triple_build(
        (terms, nonterms, states) in small_dims()
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

    /// P3: Different grammar names produce different generated code.
    #[test]
    fn different_names_different_output(
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

    /// P4: Grammar name appears in FFI function name.
    #[test]
    fn grammar_name_in_ffi(name in grammar_name()) {
        let (grammar, table) = build_grammar_and_table(&name, 2, 1, 0, 0, 1);
        let code = AbiLanguageBuilder::new(&grammar, &table).generate().to_string();
        let ffi_fn = format!("tree_sitter_{name}");
        prop_assert!(code.contains(&ffi_fn), "missing {ffi_fn}");
    }

    /// P5: Adding fields changes the output.
    #[test]
    fn fields_affect_output(
        (terms, nonterms, states) in small_dims()
    ) {
        let (g0, t0) = build_grammar_and_table("fld", terms, nonterms, 0, 0, states);
        let (g1, t1) = build_grammar_and_table("fld", terms, nonterms, 2, 0, states);
        let c0 = AbiLanguageBuilder::new(&g0, &t0).generate().to_string();
        let c1 = AbiLanguageBuilder::new(&g1, &t1).generate().to_string();
        prop_assert_ne!(c0, c1);
    }

    /// P6: StaticLanguageGenerator produces deterministic language code.
    #[test]
    fn static_gen_deterministic(
        (terms, nonterms, states) in small_dims()
    ) {
        let (grammar, table) = build_grammar_and_table(
            "sdet", terms, nonterms, 0, 0, states,
        );
        let gen1 = StaticLanguageGenerator::new(grammar.clone(), table.clone());
        let gen2 = StaticLanguageGenerator::new(grammar, table);
        let a = gen1.generate_language_code().to_string();
        let b = gen2.generate_language_code().to_string();
        prop_assert_eq!(a, b);
    }

    /// P7: StaticLanguageGenerator node types are deterministic.
    #[test]
    fn static_gen_node_types_deterministic(
        (terms, nonterms, states) in small_dims()
    ) {
        let (grammar, table) = build_grammar_and_table(
            "ntdet", terms, nonterms, 0, 0, states,
        );
        let gen1 = StaticLanguageGenerator::new(grammar.clone(), table.clone());
        let gen2 = StaticLanguageGenerator::new(grammar, table);
        let a = gen1.generate_node_types();
        let b = gen2.generate_node_types();
        prop_assert_eq!(a, b);
    }
}

// ===========================================================================
// 2. Compression properties (properties 8–20)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(30))]

    /// P8: Action table compression roundtrips losslessly.
    #[test]
    fn action_compression_roundtrip(table in action_table_strategy(6, 6)) {
        let compressed = compress_action_table(&table);
        for (state, row) in table.iter().enumerate() {
            for (symbol, cell) in row.iter().enumerate() {
                let expected = cell.first().cloned().unwrap_or(Action::Error);
                let actual = decompress_action(&compressed, state, symbol);
                prop_assert_eq!(expected, actual,
                    "mismatch at state={} symbol={}", state, symbol);
            }
        }
    }

    /// P9: Goto table compression roundtrips losslessly.
    #[test]
    fn goto_compression_roundtrip(table in goto_table_strategy(6, 6)) {
        let compressed = compress_goto_table(&table);
        for (state, row) in table.iter().enumerate() {
            for (symbol, expected) in row.iter().enumerate() {
                let actual = decompress_goto(&compressed, state, symbol);
                prop_assert_eq!(*expected, actual,
                    "mismatch at state={} symbol={}", state, symbol);
            }
        }
    }

    /// P10: Identical action rows are deduplicated.
    #[test]
    fn action_dedup_efficiency(
        row in prop::collection::vec(action_cell_strategy(), 3..=6),
        count in 2usize..=5,
    ) {
        let table: Vec<Vec<Vec<Action>>> = vec![row; count];
        let compressed = compress_action_table(&table);
        prop_assert_eq!(compressed.unique_rows.len(), 1,
            "identical rows should deduplicate to 1 unique row");
        prop_assert_eq!(compressed.state_to_row.len(), count);
    }

    /// P11: All state-to-row indices are valid after action compression.
    #[test]
    fn action_state_to_row_valid(table in action_table_strategy(8, 8)) {
        let compressed = compress_action_table(&table);
        let num_unique = compressed.unique_rows.len();
        for &row_idx in &compressed.state_to_row {
            prop_assert!(row_idx < num_unique,
                "row_idx {row_idx} >= unique count {num_unique}");
        }
    }

    /// P12: Goto sparse compression stores only Some entries.
    #[test]
    fn goto_sparse_only_some(table in goto_table_strategy(6, 6)) {
        let compressed = compress_goto_table(&table);
        let some_count: usize = table.iter()
            .flat_map(|row| row.iter())
            .filter(|g| g.is_some())
            .count();
        prop_assert_eq!(compressed.entries.len(), some_count);
    }

    /// P13: Compressing empty action table yields empty unique rows.
    #[test]
    fn empty_action_table_compressed(_dummy in 0u8..5) {
        let table: Vec<Vec<Vec<Action>>> = vec![];
        let compressed = compress_action_table(&table);
        prop_assert!(compressed.unique_rows.is_empty());
        prop_assert!(compressed.state_to_row.is_empty());
    }

    /// P14: Compressing empty goto table yields empty entries.
    #[test]
    fn empty_goto_table_compressed(_dummy in 0u8..5) {
        let table: Vec<Vec<Option<StateId>>> = vec![];
        let compressed = compress_goto_table(&table);
        prop_assert!(compressed.entries.is_empty());
    }

    /// P15: Single-row action table compresses to exactly 1 unique row.
    #[test]
    fn single_row_action(
        row in prop::collection::vec(action_cell_strategy(), 1..=6)
    ) {
        let table = vec![row];
        let compressed = compress_action_table(&table);
        prop_assert_eq!(compressed.unique_rows.len(), 1);
        prop_assert_eq!(compressed.state_to_row.len(), 1);
        prop_assert_eq!(compressed.state_to_row[0], 0);
    }

    /// P16: BitPackedActionTable preserves error cells.
    #[test]
    fn bitpack_preserves_errors(
        states in 1usize..=4,
        symbols in 1usize..=8,
    ) {
        let table: Vec<Vec<Action>> = vec![vec![Action::Error; symbols]; states];
        let packed = BitPackedActionTable::from_table(&table);
        for s in 0..states {
            for sym in 0..symbols {
                prop_assert_eq!(packed.decompress(s, sym), Action::Error);
            }
        }
    }

    /// P17: Unique rows count never exceeds state count.
    #[test]
    fn dedup_rows_leq_states(table in action_table_strategy(10, 6)) {
        let state_count = table.len();
        let compressed = compress_action_table(&table);
        prop_assert!(compressed.unique_rows.len() <= state_count);
    }

    /// P18: Goto entries count never exceeds total cells.
    #[test]
    fn goto_entries_leq_total(table in goto_table_strategy(8, 8)) {
        let total: usize = table.iter().map(|r| r.len()).sum();
        let compressed = compress_goto_table(&table);
        prop_assert!(compressed.entries.len() <= total);
    }

    /// P19: CompressedParseTable from_parse_table preserves dimensions.
    #[test]
    fn compressed_pt_preserves_dims(
        (terms, nonterms, _fields, externals, states) in grammar_dims()
    ) {
        let table = make_table(states, terms, nonterms, externals);
        let cpt = CompressedParseTable::from_parse_table(&table);
        prop_assert_eq!(cpt.symbol_count(), table.symbol_count);
        prop_assert_eq!(cpt.state_count(), table.state_count);
    }

    /// P20: CompressedParseTable test factory roundtrips dimensions.
    #[test]
    fn compressed_pt_test_factory(sym in 3usize..=20, st in 1usize..=15) {
        let cpt = CompressedParseTable::new_for_testing(sym, st);
        prop_assert_eq!(cpt.symbol_count(), sym);
        prop_assert_eq!(cpt.state_count(), st);
    }
}

// ===========================================================================
// 3. Node types consistency (properties 21–30)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(25))]

    /// P21: NodeTypesGenerator output is valid JSON.
    #[test]
    fn node_types_valid_json(
        (terms, nonterms, states) in small_dims()
    ) {
        let (grammar, _table) = build_grammar_and_table(
            "ntj", terms, nonterms, 0, 0, states,
        );
        let ntgen = NodeTypesGenerator::new(&grammar);
        if let Ok(json_str) = ntgen.generate() {
            let parsed: Result<serde_json::Value, _> = serde_json::from_str(&json_str);
            prop_assert!(parsed.is_ok(), "invalid JSON: {json_str}");
        }
    }

    /// P22: Node types top-level value is always an array.
    #[test]
    fn node_types_is_array(
        (terms, nonterms, states) in small_dims()
    ) {
        let (grammar, _table) = build_grammar_and_table(
            "nta", terms, nonterms, 0, 0, states,
        );
        let ntgen = NodeTypesGenerator::new(&grammar);
        if let Ok(json_str) = ntgen.generate() {
            let val: serde_json::Value = serde_json::from_str(&json_str).unwrap();
            prop_assert!(val.is_array(), "top-level must be array");
        }
    }

    /// P23: Every node type entry has 'type' and 'named' keys.
    #[test]
    fn node_types_entries_have_required_keys(
        (terms, nonterms, states) in small_dims()
    ) {
        let (grammar, _table) = build_grammar_and_table(
            "ntk", terms, nonterms, 0, 0, states,
        );
        let ntgen = NodeTypesGenerator::new(&grammar);
        if let Ok(json_str) = ntgen.generate() {
            let val: serde_json::Value = serde_json::from_str(&json_str).unwrap();
            if let serde_json::Value::Array(entries) = val {
                for entry in &entries {
                    prop_assert!(entry.get("type").is_some(), "missing 'type' key");
                    prop_assert!(entry.get("named").is_some(), "missing 'named' key");
                }
            }
        }
    }

    /// P24: NodeTypesGenerator is deterministic for the same grammar.
    #[test]
    fn node_types_deterministic(
        (terms, nonterms, states) in small_dims()
    ) {
        let (grammar, _table) = build_grammar_and_table(
            "ntd", terms, nonterms, 0, 0, states,
        );
        let ntgen = NodeTypesGenerator::new(&grammar);
        let a = ntgen.generate();
        let b = ntgen.generate();
        prop_assert_eq!(a, b);
    }

    /// P25: Node type names from GrammarBuilder match grammar rule names.
    #[test]
    fn node_types_match_grammar_builder(
        token_count in 1usize..=3,
    ) {
        let mut builder = GrammarBuilder::new("ntmatch");
        for i in 0..token_count {
            builder = builder.token(&format!("tok{i}"), &format!("t{i}"));
        }
        builder = builder
            .rule("program", vec!["tok0"])
            .start("program");
        let grammar = builder.build();
        let ntgen = NodeTypesGenerator::new(&grammar);
        if let Ok(json_str) = ntgen.generate() {
            let val: serde_json::Value = serde_json::from_str(&json_str).unwrap();
            if let serde_json::Value::Array(entries) = val {
                let types: Vec<String> = entries.iter()
                    .filter_map(|e| e.get("type").and_then(|t| t.as_str()).map(String::from))
                    .collect();
                prop_assert!(
                    types.iter().any(|t| t == "program"),
                    "expected 'program' in node types, got {:?}", types,
                );
            }
        }
    }

    /// P26: No duplicate type names in node types output.
    #[test]
    fn node_types_no_duplicate_names(
        (terms, nonterms, states) in small_dims()
    ) {
        let (grammar, _table) = build_grammar_and_table(
            "ntdup", terms, nonterms, 0, 0, states,
        );
        let ntgen = NodeTypesGenerator::new(&grammar);
        if let Ok(json_str) = ntgen.generate() {
            let val: serde_json::Value = serde_json::from_str(&json_str).unwrap();
            if let serde_json::Value::Array(entries) = val {
                let mut seen = std::collections::HashSet::new();
                for entry in &entries {
                    if let Some(name) = entry.get("type").and_then(|t| t.as_str()) {
                        let named = entry.get("named")
                            .and_then(|n| n.as_bool())
                            .unwrap_or(false);
                        let key = format!("{name}:{named}");
                        prop_assert!(seen.insert(key.clone()),
                            "duplicate node type: {key}");
                    }
                }
            }
        }
    }

    /// P27: 'named' field is always a boolean in node types entries.
    #[test]
    fn node_types_named_is_bool(
        (terms, nonterms, states) in small_dims()
    ) {
        let (grammar, _table) = build_grammar_and_table(
            "ntbool", terms, nonterms, 0, 0, states,
        );
        let ntgen = NodeTypesGenerator::new(&grammar);
        if let Ok(json_str) = ntgen.generate() {
            let val: serde_json::Value = serde_json::from_str(&json_str).unwrap();
            if let serde_json::Value::Array(entries) = val {
                for entry in &entries {
                    if let Some(named) = entry.get("named") {
                        prop_assert!(named.is_boolean(),
                            "'named' must be boolean, got {named}");
                    }
                }
            }
        }
    }

    /// P28: 'type' field is always a non-empty string.
    #[test]
    fn node_types_type_nonempty_string(
        (terms, nonterms, states) in small_dims()
    ) {
        let (grammar, _table) = build_grammar_and_table(
            "ntstr", terms, nonterms, 0, 0, states,
        );
        let ntgen = NodeTypesGenerator::new(&grammar);
        if let Ok(json_str) = ntgen.generate() {
            let val: serde_json::Value = serde_json::from_str(&json_str).unwrap();
            if let serde_json::Value::Array(entries) = val {
                for entry in &entries {
                    if let Some(type_val) = entry.get("type") {
                        let s = type_val.as_str().unwrap_or("");
                        prop_assert!(!s.is_empty(), "'type' must be non-empty");
                    }
                }
            }
        }
    }

    /// P29: GrammarBuilder-produced grammars yield valid node types JSON.
    #[test]
    fn grammar_builder_node_types_valid(
        num_rules in 1usize..=3,
    ) {
        let mut builder = GrammarBuilder::new("gbvalid")
            .token("id", r"[a-z]+");
        for i in 0..num_rules {
            builder = builder.rule(&format!("rule{i}"), vec!["id"]);
        }
        builder = builder.start("rule0");
        let grammar = builder.build();
        let ntgen = NodeTypesGenerator::new(&grammar);
        if let Ok(json_str) = ntgen.generate() {
            let parsed: Result<serde_json::Value, _> = serde_json::from_str(&json_str);
            prop_assert!(parsed.is_ok(), "GrammarBuilder grammar produced invalid JSON");
        }
    }

    /// P30: Subtypes field is always an array when present.
    #[test]
    fn node_types_subtypes_is_array(
        (terms, nonterms, states) in small_dims()
    ) {
        let (grammar, _table) = build_grammar_and_table(
            "ntsub", terms, nonterms, 0, 0, states,
        );
        let ntgen = NodeTypesGenerator::new(&grammar);
        if let Ok(json_str) = ntgen.generate() {
            let val: serde_json::Value = serde_json::from_str(&json_str).unwrap();
            if let serde_json::Value::Array(entries) = val {
                for entry in &entries {
                    if let Some(subtypes) = entry.get("subtypes") {
                        prop_assert!(subtypes.is_array(),
                            "'subtypes' must be array, got {subtypes}");
                    }
                }
            }
        }
    }
}

// ===========================================================================
// 4. ABI builder invariants (properties 31–43)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(25))]

    /// P31: Symbol count in generated code matches parse table.
    #[test]
    fn abi_symbol_count_matches(
        (terms, nonterms, fields, externals, states) in grammar_dims()
    ) {
        let (grammar, table) = build_grammar_and_table(
            "sym", terms, nonterms, fields, externals, states,
        );
        let code = AbiLanguageBuilder::new(&grammar, &table).generate().to_string();
        let sc = table.symbol_count as u32;
        let needle = format!("symbol_count : {sc}u32");
        prop_assert!(code.contains(&needle), "expected {needle}");
    }

    /// P32: State count in generated code matches parse table.
    #[test]
    fn abi_state_count_matches(
        (terms, nonterms, fields, externals, states) in grammar_dims()
    ) {
        let (grammar, table) = build_grammar_and_table(
            "stc", terms, nonterms, fields, externals, states,
        );
        let code = AbiLanguageBuilder::new(&grammar, &table).generate().to_string();
        let sc = table.state_count as u32;
        let needle = format!("state_count : {sc}u32");
        prop_assert!(code.contains(&needle));
    }

    /// P33: Field count in generated code matches grammar fields.
    #[test]
    fn abi_field_count_matches(
        (terms, nonterms, fields, externals, states) in grammar_dims()
    ) {
        let (grammar, table) = build_grammar_and_table(
            "fc", terms, nonterms, fields, externals, states,
        );
        let code = AbiLanguageBuilder::new(&grammar, &table).generate().to_string();
        let fc = grammar.fields.len() as u32;
        let needle = format!("field_count : {fc}u32");
        prop_assert!(code.contains(&needle));
    }

    /// P34: External token count in generated code is consistent.
    #[test]
    fn abi_external_token_count(
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

    /// P35: Token count in generated code is consistent.
    #[test]
    fn abi_token_count(
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

    /// P36: ABI version constant is always 15.
    #[test]
    fn abi_version_is_15(_dummy in 0u8..10) {
        use adze_tablegen::abi::TREE_SITTER_LANGUAGE_VERSION;
        prop_assert_eq!(TREE_SITTER_LANGUAGE_VERSION, 15u32);
    }

    /// P37: Generated code always references TSLanguage.
    #[test]
    fn abi_contains_tslanguage(
        (terms, nonterms, states) in small_dims()
    ) {
        let (grammar, table) = build_grammar_and_table(
            "tsl", terms, nonterms, 0, 0, states,
        );
        let code = AbiLanguageBuilder::new(&grammar, &table).generate().to_string();
        prop_assert!(code.contains("TSLanguage"));
    }

    /// P38: Generated code always contains PARSE_TABLE.
    #[test]
    fn abi_contains_parse_table(
        (terms, nonterms, states) in small_dims()
    ) {
        let (grammar, table) = build_grammar_and_table(
            "pt", terms, nonterms, 0, 0, states,
        );
        let code = AbiLanguageBuilder::new(&grammar, &table).generate().to_string();
        prop_assert!(code.contains("PARSE_TABLE"));
    }

    /// P39: Generated code always contains SYMBOL_METADATA.
    #[test]
    fn abi_contains_symbol_metadata(
        (terms, nonterms, states) in small_dims()
    ) {
        let (grammar, table) = build_grammar_and_table(
            "sm", terms, nonterms, 0, 0, states,
        );
        let code = AbiLanguageBuilder::new(&grammar, &table).generate().to_string();
        prop_assert!(code.contains("SYMBOL_METADATA"));
    }

    /// P40: Symbol count grows with more terminals.
    #[test]
    fn symbol_count_monotonic_in_terminals(
        base in 2usize..=3,
        extra in 1usize..=3,
    ) {
        let (_, t1) = build_grammar_and_table("mono1", base, 1, 0, 0, 1);
        let (_, t2) = build_grammar_and_table("mono2", base + extra, 1, 0, 0, 1);
        prop_assert!(t2.symbol_count > t1.symbol_count);
    }

    /// P41: Lex modes count matches state count.
    #[test]
    fn lex_modes_match_states(
        (terms, nonterms, fields, externals, states) in grammar_dims()
    ) {
        let (grammar, table) = build_grammar_and_table(
            "lm", terms, nonterms, fields, externals, states,
        );
        let code = AbiLanguageBuilder::new(&grammar, &table).generate().to_string();
        let lex_count = code.matches("TSLexState").count();
        prop_assert!(lex_count >= states,
            "lex entries {lex_count} < states {states}");
    }

    /// P42: Alias count is always zero (no aliases supported yet).
    #[test]
    fn abi_alias_count_zero(
        (terms, nonterms, fields, externals, states) in grammar_dims()
    ) {
        let (grammar, table) = build_grammar_and_table(
            "ac", terms, nonterms, fields, externals, states,
        );
        let code = AbiLanguageBuilder::new(&grammar, &table).generate().to_string();
        prop_assert!(code.contains("alias_count : 0u32"));
    }

    /// P43: Generated code contains SYMBOL_NAME_PTRS array.
    #[test]
    fn abi_contains_symbol_names(
        (terms, nonterms, states) in small_dims()
    ) {
        let (grammar, table) = build_grammar_and_table(
            "snp", terms, nonterms, 0, 0, states,
        );
        let code = AbiLanguageBuilder::new(&grammar, &table).generate().to_string();
        prop_assert!(code.contains("SYMBOL_NAME_PTRS"));
    }
}
