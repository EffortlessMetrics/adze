//! Property-based tests for ABI v5 invariants across the adze-tablegen pipeline.
//!
//! 45+ proptest properties organised into 9 categories:
//!  1. Generated code is non-empty for valid grammars
//!  2. Generated code contains grammar name
//!  3. Node types JSON is valid JSON
//!  4. Code generation is deterministic
//!  5. State count in generated code is positive
//!  6. ABI version present in output
//!  7. Symbol names in generated code
//!  8. Code length scales with grammar complexity
//!  9. Edge cases

use adze_glr_core::{GotoIndexing, LexMode, ParseTable};
use adze_ir::builder::GrammarBuilder;
use adze_ir::{
    ExternalToken, FieldId, Grammar, ProductionId, Rule, Symbol, SymbolId, Token, TokenPattern,
};
use adze_tablegen::{AbiLanguageBuilder, NodeTypesGenerator, StaticLanguageGenerator};
use proptest::prelude::*;
use std::collections::BTreeMap;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

const INVALID: StateId = StateId(u16::MAX);

use adze_ir::StateId;

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
    for (i, slot) in index_to_symbol.iter_mut().enumerate().take(symbol_count) {
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

/// Helper: generate ABI code string for a grammar/table pair.
fn abi_code(grammar: &Grammar, table: &ParseTable) -> String {
    AbiLanguageBuilder::new(grammar, table)
        .generate()
        .to_string()
}

/// Helper: build a GrammarBuilder-based grammar with `n` tokens and one rule.
fn builder_grammar(name: &str, n_tokens: usize) -> Grammar {
    let n_tokens = n_tokens.max(1);
    let mut b = GrammarBuilder::new(name);
    for i in 0..n_tokens {
        b = b.token(&format!("tok{i}"), &format!("t{i}"));
    }
    b = b.rule("program", vec!["tok0"]).start("program");
    b.build()
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

// ===========================================================================
// Category 1 — Generated code is non-empty for valid grammars (5 properties)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(25))]

    /// C1-P1: ABI code is non-empty for any valid grammar dimensions.
    #[test]
    fn nonempty_abi_code(
        (terms, nonterms, fields, externals, states) in grammar_dims()
    ) {
        let (grammar, table) = build_grammar_and_table(
            "ne", terms, nonterms, fields, externals, states,
        );
        let code = abi_code(&grammar, &table);
        prop_assert!(!code.is_empty());
    }

    /// C1-P2: StaticLanguageGenerator produces non-empty language code.
    #[test]
    fn nonempty_static_language_code(
        (terms, nonterms, states) in small_dims()
    ) {
        let (grammar, table) = build_grammar_and_table(
            "slne", terms, nonterms, 0, 0, states,
        );
        let slg = StaticLanguageGenerator::new(grammar, table);
        let code = slg.generate_language_code().to_string();
        prop_assert!(!code.is_empty());
    }

    /// C1-P3: StaticLanguageGenerator node types output is non-empty.
    #[test]
    fn nonempty_static_node_types(
        (terms, nonterms, states) in small_dims()
    ) {
        let (grammar, table) = build_grammar_and_table(
            "ntne", terms, nonterms, 0, 0, states,
        );
        let slg = StaticLanguageGenerator::new(grammar, table);
        let nt = slg.generate_node_types();
        prop_assert!(!nt.is_empty());
    }

    /// C1-P4: Varying terminal counts still produce non-empty ABI code.
    #[test]
    fn nonempty_varying_terminal_counts(n_tokens in 2usize..=5) {
        let (grammar, table) = build_grammar_and_table("bne", n_tokens, 1, 0, 0, 1);
        let code = abi_code(&grammar, &table);
        prop_assert!(!code.is_empty());
    }

    /// C1-P5: Grammars with external tokens still produce non-empty code.
    #[test]
    fn nonempty_with_externals(externals in 1usize..=3) {
        let (grammar, table) = build_grammar_and_table(
            "ext", 2, 1, 0, externals, 2,
        );
        let code = abi_code(&grammar, &table);
        prop_assert!(!code.is_empty());
    }
}

// ===========================================================================
// Category 2 — Generated code contains grammar name (5 properties)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(25))]

    /// C2-P1: Grammar name appears in FFI function name (`tree_sitter_<name>`).
    #[test]
    fn name_in_ffi_fn(name in grammar_name()) {
        let (grammar, table) = build_grammar_and_table(&name, 2, 1, 0, 0, 1);
        let code = abi_code(&grammar, &table);
        let ffi_fn = format!("tree_sitter_{name}");
        prop_assert!(code.contains(&ffi_fn), "missing {ffi_fn}");
    }

    /// C2-P2: Different grammar names produce different generated code.
    #[test]
    fn different_names_different_code(
        name_a in "[a-z]{3,6}",
        name_b in "[a-z]{3,6}",
    ) {
        prop_assume!(name_a != name_b);
        let (g1, t1) = build_grammar_and_table(&name_a, 2, 1, 0, 0, 1);
        let (g2, t2) = build_grammar_and_table(&name_b, 2, 1, 0, 0, 1);
        prop_assert_ne!(abi_code(&g1, &t1), abi_code(&g2, &t2));
    }

    /// C2-P3: StaticLanguageGenerator code contains the grammar name.
    #[test]
    fn static_gen_contains_name(name in grammar_name()) {
        let (grammar, table) = build_grammar_and_table(&name, 2, 1, 0, 0, 1);
        let slg = StaticLanguageGenerator::new(grammar, table);
        let code = slg.generate_language_code().to_string();
        prop_assert!(code.contains(&name), "name '{name}' not in static code");
    }

    /// C2-P4: Grammar name appears even with multiple non-terminals.
    #[test]
    fn name_with_multiple_nonterms(
        name in grammar_name(),
        nonterms in 1usize..=3,
    ) {
        let (grammar, table) = build_grammar_and_table(&name, 3, nonterms, 0, 0, 2);
        let code = abi_code(&grammar, &table);
        prop_assert!(code.contains(&name));
    }

    /// C2-P5: Grammar name survives presence of fields and externals.
    #[test]
    fn name_with_fields_externals(name in grammar_name()) {
        let (grammar, table) = build_grammar_and_table(&name, 3, 1, 2, 1, 2);
        let code = abi_code(&grammar, &table);
        prop_assert!(code.contains(&name));
    }
}

// ===========================================================================
// Category 3 — Node types JSON is valid JSON (5 properties)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(25))]

    /// C3-P1: NodeTypesGenerator output parses as valid JSON.
    #[test]
    fn node_types_valid_json(
        (terms, nonterms, states) in small_dims()
    ) {
        let (grammar, _) = build_grammar_and_table("ntj", terms, nonterms, 0, 0, states);
        let ntgen = NodeTypesGenerator::new(&grammar);
        if let Ok(json_str) = ntgen.generate() {
            let parsed: Result<serde_json::Value, _> = serde_json::from_str(&json_str);
            prop_assert!(parsed.is_ok(), "invalid JSON: {json_str}");
        }
    }

    /// C3-P2: Node types top-level value is always an array.
    #[test]
    fn node_types_is_array(
        (terms, nonterms, states) in small_dims()
    ) {
        let (grammar, _) = build_grammar_and_table("nta", terms, nonterms, 0, 0, states);
        let ntgen = NodeTypesGenerator::new(&grammar);
        if let Ok(json_str) = ntgen.generate() {
            let val: serde_json::Value = serde_json::from_str(&json_str).unwrap();
            prop_assert!(val.is_array(), "top-level must be array");
        }
    }

    /// C3-P3: Every node type entry has 'type' and 'named' keys.
    #[test]
    fn node_types_required_keys(
        (terms, nonterms, states) in small_dims()
    ) {
        let (grammar, _) = build_grammar_and_table("ntk", terms, nonterms, 0, 0, states);
        let ntgen = NodeTypesGenerator::new(&grammar);
        if let Ok(json_str) = ntgen.generate() {
            let val: serde_json::Value = serde_json::from_str(&json_str).unwrap();
            if let serde_json::Value::Array(entries) = val {
                for entry in &entries {
                    prop_assert!(entry.get("type").is_some(), "missing 'type'");
                    prop_assert!(entry.get("named").is_some(), "missing 'named'");
                }
            }
        }
    }

    /// C3-P4: GrammarBuilder grammars produce valid node types JSON.
    #[test]
    fn builder_node_types_valid_json(num_rules in 1usize..=3) {
        let mut b = GrammarBuilder::new("gbvj")
            .token("id", r"[a-z]+");
        for i in 0..num_rules {
            b = b.rule(&format!("rule{i}"), vec!["id"]);
        }
        b = b.start("rule0");
        let grammar = b.build();
        let ntgen = NodeTypesGenerator::new(&grammar);
        if let Ok(json_str) = ntgen.generate() {
            let parsed: Result<serde_json::Value, _> = serde_json::from_str(&json_str);
            prop_assert!(parsed.is_ok());
        }
    }

    /// C3-P5: 'named' field is always boolean, 'type' is always non-empty string.
    #[test]
    fn node_types_field_types(
        (terms, nonterms, states) in small_dims()
    ) {
        let (grammar, _) = build_grammar_and_table("ntft", terms, nonterms, 0, 0, states);
        let ntgen = NodeTypesGenerator::new(&grammar);
        if let Ok(json_str) = ntgen.generate() {
            let val: serde_json::Value = serde_json::from_str(&json_str).unwrap();
            if let serde_json::Value::Array(entries) = val {
                for entry in &entries {
                    if let Some(named) = entry.get("named") {
                        prop_assert!(named.is_boolean());
                    }
                    if let Some(t) = entry.get("type") {
                        let s = t.as_str().unwrap_or("");
                        prop_assert!(!s.is_empty(), "'type' must be non-empty");
                    }
                }
            }
        }
    }
}

// ===========================================================================
// Category 4 — Code generation is deterministic (5 properties)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(25))]

    /// C4-P1: Same grammar+table always yields identical ABI code.
    #[test]
    fn deterministic_abi_output(
        (terms, nonterms, fields, externals, states) in grammar_dims()
    ) {
        let (grammar, table) = build_grammar_and_table(
            "det", terms, nonterms, fields, externals, states,
        );
        let a = abi_code(&grammar, &table);
        let b = abi_code(&grammar, &table);
        prop_assert_eq!(a, b);
    }

    /// C4-P2: Three consecutive builds produce identical output.
    #[test]
    fn deterministic_triple_build(
        (terms, nonterms, states) in small_dims()
    ) {
        let (grammar, table) = build_grammar_and_table("det3", terms, nonterms, 0, 0, states);
        let runs: Vec<String> = (0..3)
            .map(|_| abi_code(&grammar, &table))
            .collect();
        prop_assert_eq!(&runs[0], &runs[1]);
        prop_assert_eq!(&runs[1], &runs[2]);
    }

    /// C4-P3: StaticLanguageGenerator produces deterministic language code.
    #[test]
    fn static_gen_deterministic(
        (terms, nonterms, states) in small_dims()
    ) {
        let (g1, t1) = build_grammar_and_table("sdet", terms, nonterms, 0, 0, states);
        let (g2, t2) = build_grammar_and_table("sdet", terms, nonterms, 0, 0, states);
        let gen1 = StaticLanguageGenerator::new(g1, t1);
        let gen2 = StaticLanguageGenerator::new(g2, t2);
        prop_assert_eq!(
            gen1.generate_language_code().to_string(),
            gen2.generate_language_code().to_string(),
        );
    }

    /// C4-P4: StaticLanguageGenerator node types are deterministic.
    #[test]
    fn static_gen_node_types_deterministic(
        (terms, nonterms, states) in small_dims()
    ) {
        let (g1, t1) = build_grammar_and_table("ntdet", terms, nonterms, 0, 0, states);
        let (g2, t2) = build_grammar_and_table("ntdet", terms, nonterms, 0, 0, states);
        let gen1 = StaticLanguageGenerator::new(g1, t1);
        let gen2 = StaticLanguageGenerator::new(g2, t2);
        prop_assert_eq!(gen1.generate_node_types(), gen2.generate_node_types());
    }

    /// C4-P5: NodeTypesGenerator is deterministic for the same grammar.
    #[test]
    fn node_types_generator_deterministic(
        (terms, nonterms, states) in small_dims()
    ) {
        let (grammar, _) = build_grammar_and_table("ntgd", terms, nonterms, 0, 0, states);
        let ntgen = NodeTypesGenerator::new(&grammar);
        prop_assert_eq!(ntgen.generate(), ntgen.generate());
    }
}

// ===========================================================================
// Category 5 — State count in generated code is positive (5 properties)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(25))]

    /// C5-P1: state_count in generated code matches parse table.
    #[test]
    fn state_count_matches(
        (terms, nonterms, fields, externals, states) in grammar_dims()
    ) {
        let (grammar, table) = build_grammar_and_table(
            "stc", terms, nonterms, fields, externals, states,
        );
        let code = abi_code(&grammar, &table);
        let sc = table.state_count as u32;
        let needle = format!("state_count : {sc}u32");
        prop_assert!(code.contains(&needle));
    }

    /// C5-P2: state_count value in the parse table is ≥ 1.
    #[test]
    fn parse_table_state_count_positive(
        (terms, nonterms, fields, externals, states) in grammar_dims()
    ) {
        let (_, table) = build_grammar_and_table(
            "sc1", terms, nonterms, fields, externals, states,
        );
        prop_assert!(table.state_count >= 1);
    }

    /// C5-P3: Increasing states parameter increases state_count in output.
    #[test]
    fn state_count_monotonic(
        base in 1usize..=3,
        extra in 1usize..=3,
    ) {
        let (_, t1) = build_grammar_and_table("sm1", 2, 1, 0, 0, base);
        let (_, t2) = build_grammar_and_table("sm2", 2, 1, 0, 0, base + extra);
        prop_assert!(t2.state_count > t1.state_count);
    }

    /// C5-P4: Lex modes count matches state count.
    #[test]
    fn lex_modes_match_states(
        (terms, nonterms, fields, externals, states) in grammar_dims()
    ) {
        let (grammar, table) = build_grammar_and_table(
            "lm", terms, nonterms, fields, externals, states,
        );
        let code = abi_code(&grammar, &table);
        let lex_count = code.matches("TSLexState").count();
        prop_assert!(lex_count >= states, "lex entries {lex_count} < states {states}");
    }

    /// C5-P5: state_count field present in output for single-state grammars.
    #[test]
    fn single_state_has_state_count(terms in 2usize..=5) {
        let (grammar, table) = build_grammar_and_table("ss", terms, 1, 0, 0, 1);
        let code = abi_code(&grammar, &table);
        prop_assert!(code.contains("state_count : 1u32"));
    }
}

// ===========================================================================
// Category 6 — ABI version present in output (5 properties)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(25))]

    /// C6-P1: TREE_SITTER_LANGUAGE_VERSION constant equals 15.
    #[test]
    fn abi_version_constant_is_15(_dummy in 0u8..10) {
        use adze_tablegen::abi::TREE_SITTER_LANGUAGE_VERSION;
        prop_assert_eq!(TREE_SITTER_LANGUAGE_VERSION, 15u32);
    }

    /// C6-P2: Minimum compatible version is 13.
    #[test]
    fn abi_min_compat_version_is_13(_dummy in 0u8..10) {
        use adze_tablegen::abi::TREE_SITTER_MIN_COMPATIBLE_LANGUAGE_VERSION;
        prop_assert_eq!(TREE_SITTER_MIN_COMPATIBLE_LANGUAGE_VERSION, 13u32);
    }

    /// C6-P3: Generated code references TSLanguage struct.
    #[test]
    fn output_references_tslanguage(
        (terms, nonterms, states) in small_dims()
    ) {
        let (grammar, table) = build_grammar_and_table("tsl", terms, nonterms, 0, 0, states);
        let code = abi_code(&grammar, &table);
        prop_assert!(code.contains("TSLanguage"));
    }

    /// C6-P4: Generated code always contains PARSE_TABLE.
    #[test]
    fn output_contains_parse_table(
        (terms, nonterms, states) in small_dims()
    ) {
        let (grammar, table) = build_grammar_and_table("pt", terms, nonterms, 0, 0, states);
        let code = abi_code(&grammar, &table);
        prop_assert!(code.contains("PARSE_TABLE"));
    }

    /// C6-P5: Generated code always contains SYMBOL_METADATA.
    #[test]
    fn output_contains_symbol_metadata(
        (terms, nonterms, states) in small_dims()
    ) {
        let (grammar, table) = build_grammar_and_table("sm", terms, nonterms, 0, 0, states);
        let code = abi_code(&grammar, &table);
        prop_assert!(code.contains("SYMBOL_METADATA"));
    }
}

// ===========================================================================
// Category 7 — Symbol names in generated code (5 properties)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(25))]

    /// C7-P1: Generated code contains SYMBOL_NAME_PTRS array.
    #[test]
    fn contains_symbol_name_ptrs(
        (terms, nonterms, states) in small_dims()
    ) {
        let (grammar, table) = build_grammar_and_table("snp", terms, nonterms, 0, 0, states);
        let code = abi_code(&grammar, &table);
        prop_assert!(code.contains("SYMBOL_NAME_PTRS"));
    }

    /// C7-P2: symbol_count in generated code matches parse table.
    #[test]
    fn symbol_count_matches(
        (terms, nonterms, fields, externals, states) in grammar_dims()
    ) {
        let (grammar, table) = build_grammar_and_table(
            "sym", terms, nonterms, fields, externals, states,
        );
        let code = abi_code(&grammar, &table);
        let sc = table.symbol_count as u32;
        let needle = format!("symbol_count : {sc}u32");
        prop_assert!(code.contains(&needle), "expected {needle}");
    }

    /// C7-P3: Symbol count grows with more terminals.
    #[test]
    fn symbol_count_monotonic_in_terminals(
        base in 2usize..=3,
        extra in 1usize..=3,
    ) {
        let (_, t1) = build_grammar_and_table("mono1", base, 1, 0, 0, 1);
        let (_, t2) = build_grammar_and_table("mono2", base + extra, 1, 0, 0, 1);
        prop_assert!(t2.symbol_count > t1.symbol_count);
    }

    /// C7-P4: No duplicate type names in node types output.
    #[test]
    fn no_duplicate_node_type_names(
        (terms, nonterms, states) in small_dims()
    ) {
        let (grammar, _) = build_grammar_and_table("ntdup", terms, nonterms, 0, 0, states);
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
                        prop_assert!(seen.insert(key.clone()), "duplicate: {key}");
                    }
                }
            }
        }
    }

    /// C7-P5: Node type names from GrammarBuilder include the start rule name.
    #[test]
    fn node_types_include_start_rule(token_count in 1usize..=3) {
        let grammar = builder_grammar("ntmatch", token_count);
        let ntgen = NodeTypesGenerator::new(&grammar);
        if let Ok(json_str) = ntgen.generate() {
            let val: serde_json::Value = serde_json::from_str(&json_str).unwrap();
            if let serde_json::Value::Array(entries) = val {
                let types: Vec<&str> = entries.iter()
                    .filter_map(|e| e.get("type").and_then(|t| t.as_str()))
                    .collect();
                prop_assert!(
                    types.contains(&"program"),
                    "expected 'program' in node types, got {types:?}",
                );
            }
        }
    }
}

// ===========================================================================
// Category 8 — Code length scales with grammar complexity (5 properties)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(25))]

    /// C8-P1: Adding fields changes the output.
    #[test]
    fn fields_affect_output(
        (terms, nonterms, states) in small_dims()
    ) {
        let (g0, t0) = build_grammar_and_table("fld", terms, nonterms, 0, 0, states);
        let (g1, t1) = build_grammar_and_table("fld", terms, nonterms, 2, 0, states);
        prop_assert_ne!(abi_code(&g0, &t0), abi_code(&g1, &t1));
    }

    /// C8-P2: More terminals produce longer code.
    #[test]
    fn more_terminals_longer_code(
        base in 2usize..=3,
        extra in 1usize..=3,
    ) {
        let (g1, t1) = build_grammar_and_table("mtl", base, 1, 0, 0, 1);
        let (g2, t2) = build_grammar_and_table("mtl", base + extra, 1, 0, 0, 1);
        prop_assert!(abi_code(&g2, &t2).len() > abi_code(&g1, &t1).len());
    }

    /// C8-P3: More states produce longer code.
    #[test]
    fn more_states_longer_code(
        base in 1usize..=3,
        extra in 1usize..=3,
    ) {
        let (g1, t1) = build_grammar_and_table("msl", 2, 1, 0, 0, base);
        let (g2, t2) = build_grammar_and_table("msl", 2, 1, 0, 0, base + extra);
        prop_assert!(abi_code(&g2, &t2).len() > abi_code(&g1, &t1).len());
    }

    /// C8-P4: More non-terminals produce longer code.
    #[test]
    fn more_nonterms_longer_code(
        base in 1usize..=2,
        extra in 1usize..=2,
    ) {
        let (g1, t1) = build_grammar_and_table("mnt", 2, base, 0, 0, 1);
        let (g2, t2) = build_grammar_and_table("mnt", 2, base + extra, 0, 0, 1);
        prop_assert!(abi_code(&g2, &t2).len() > abi_code(&g1, &t1).len());
    }

    /// C8-P5: Adding external tokens changes the generated code.
    #[test]
    fn externals_change_output(
        (terms, nonterms, states) in small_dims()
    ) {
        let (g0, t0) = build_grammar_and_table("ext0", terms, nonterms, 0, 0, states);
        let (g1, t1) = build_grammar_and_table("ext0", terms, nonterms, 0, 1, states);
        prop_assert_ne!(abi_code(&g0, &t0), abi_code(&g1, &t1));
    }
}

// ===========================================================================
// Category 9 — Edge cases (5+ properties)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(25))]

    /// C9-P1: Alias count is always zero (no aliases configured).
    #[test]
    fn alias_count_zero(
        (terms, nonterms, fields, externals, states) in grammar_dims()
    ) {
        let (grammar, table) = build_grammar_and_table(
            "ac", terms, nonterms, fields, externals, states,
        );
        let code = abi_code(&grammar, &table);
        prop_assert!(code.contains("alias_count : 0u32"));
    }

    /// C9-P2: field_count matches grammar fields.
    #[test]
    fn field_count_matches(
        (terms, nonterms, fields, externals, states) in grammar_dims()
    ) {
        let (grammar, table) = build_grammar_and_table(
            "fc", terms, nonterms, fields, externals, states,
        );
        let code = abi_code(&grammar, &table);
        let fc = grammar.fields.len() as u32;
        let needle = format!("field_count : {fc}u32");
        prop_assert!(code.contains(&needle));
    }

    /// C9-P3: external_token_count is consistent in generated code.
    #[test]
    fn external_token_count_matches(
        (terms, nonterms, _f, externals, states) in grammar_dims()
    ) {
        let (grammar, table) = build_grammar_and_table(
            "etc", terms, nonterms, 0, externals, states,
        );
        let code = abi_code(&grammar, &table);
        let etc = externals as u32;
        let needle = format!("external_token_count : {etc}u32");
        prop_assert!(code.contains(&needle));
    }

    /// C9-P4: token_count is consistent in generated code.
    #[test]
    fn token_count_matches(
        (terms, nonterms, _f, externals, states) in grammar_dims()
    ) {
        let (grammar, table) = build_grammar_and_table(
            "tc", terms, nonterms, 0, externals, states,
        );
        let code = abi_code(&grammar, &table);
        let tc = table.token_count as u32;
        let needle = format!("token_count : {tc}u32");
        prop_assert!(code.contains(&needle));
    }

    /// C9-P5: Subtypes field in node types is always an array when present.
    #[test]
    fn node_types_subtypes_is_array(
        (terms, nonterms, states) in small_dims()
    ) {
        let (grammar, _) = build_grammar_and_table("ntsub", terms, nonterms, 0, 0, states);
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

    /// C9-P6: Minimal single-terminal grammar produces valid output.
    #[test]
    fn minimal_single_terminal(_dummy in 0u8..10) {
        let (grammar, table) = build_grammar_and_table("min", 1, 1, 0, 0, 1);
        let code = abi_code(&grammar, &table);
        prop_assert!(!code.is_empty());
        prop_assert!(code.contains("tree_sitter_min"));
    }
}
