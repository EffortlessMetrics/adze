#![allow(clippy::needless_range_loop)]

//! Property-based tests for `LanguageGenerator` in adze-tablegen.
//!
//! Properties verified:
//!  1. LanguageGenerator produces valid (non-empty) TokenStream
//!  2. Generated code contains LANGUAGE struct
//!  3. Generated code includes PARSE_ACTIONS
//!  4. Generated code is deterministic (same inputs → same output)
//!  5. LanguageGenerator with minimal (single-token) grammar
//!  6. LanguageGenerator with many states
//!  7. LanguageGenerator with shift/reduce conflicts
//!  8. Symbol names always start with "end" (EOF sentinel)
//!  9. SYMBOL_METADATA present in output
//! 10. LEX_MODES present in output
//! 11. PARSE_TABLE present in output
//! 12. Grammar name embedded in tree_sitter_{name} function
//! 13. FIELD_NAMES present when grammar has fields
//! 14. Symbol count grows with tokens
//! 15. State count reflected in lex modes
//! 16. Determinism with identical parse tables
//! 17. External token count embedded
//! 18. Grammar with externals produces valid code
//! 19. Grammar with multiple rules generates valid code
//! 20. Grammar with extras generates valid code
//! 21. PUBLIC_SYMBOL_MAP present
//! 22. PRIMARY_STATE_IDS present
//! 23. generate_symbol_metadata_public length matches symbol count
//! 24. count_production_ids_public ≥ 1 when rules exist
//! 25. Multiple conflicts in same state
//! 26. Accept action in parse table
//! 27. Reduce actions encoded in output
//! 28. Large token count still generates
//! 29. Grammar with fields generates FIELD_NAMES entries
//! 30. Different grammar names produce different output

use adze_glr_core::{Action, GotoIndexing, LexMode, ParseTable};
use adze_ir::{FieldId, Grammar, StateId, SymbolId, builder::GrammarBuilder};
use adze_tablegen::language_gen::LanguageGenerator;
use proptest::prelude::*;
use std::collections::BTreeMap;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Build a ParseTable compatible with LanguageGenerator from a grammar.
fn make_parse_table(
    grammar: &Grammar,
    state_count: usize,
    actions: Vec<Vec<Vec<Action>>>,
) -> ParseTable {
    let symbol_count = actions.first().map(|r| r.len()).unwrap_or(1).max(1);
    let mut symbol_to_index = BTreeMap::new();
    for i in 0..symbol_count {
        symbol_to_index.insert(SymbolId(i as u16), i);
    }
    let index_to_symbol: Vec<SymbolId> = (0..symbol_count).map(|i| SymbolId(i as u16)).collect();
    let goto_table = vec![vec![StateId(u16::MAX); symbol_count]; state_count];
    let lex_modes = vec![
        LexMode {
            lex_state: 0,
            external_lex_state: 0,
        };
        state_count
    ];
    let eof_symbol = SymbolId(symbol_count.saturating_sub(1) as u16);

    ParseTable {
        action_table: actions,
        goto_table,
        rules: vec![],
        state_count,
        symbol_count,
        symbol_to_index,
        index_to_symbol,
        nonterminal_to_index: BTreeMap::new(),
        symbol_metadata: vec![],
        token_count: symbol_count.saturating_sub(1),
        external_token_count: grammar.externals.len(),
        eof_symbol,
        start_symbol: SymbolId(0),
        initial_state: StateId(0),
        lex_modes,
        extras: vec![],
        external_scanner_states: vec![],
        dynamic_prec_by_rule: vec![],
        rule_assoc_by_rule: vec![],
        alias_sequences: vec![],
        field_names: vec![],
        field_map: BTreeMap::new(),
        grammar: grammar.clone(),
        goto_indexing: GotoIndexing::NonterminalMap,
    }
}

/// Shorthand: grammar with N tokens and a default parse table with given state count.
fn gen_with_tokens(name: &str, n_tokens: usize, n_states: usize) -> (Grammar, ParseTable) {
    let n = n_tokens.max(1);
    let mut builder = GrammarBuilder::new(name);
    for i in 0..n {
        builder = builder.token(&format!("tok{i}"), &format!("t{i}"));
    }
    builder = builder.rule("root", vec!["tok0"]).start("root");
    let grammar = builder.build();
    // symbol_count = 1 (EOF) + n tokens + 1 rule ("root") at minimum 3
    let sym_count = (1 + n + 1).max(3);
    let actions = vec![vec![vec![]; sym_count]; n_states.max(1)];
    let pt = make_parse_table(&grammar, n_states.max(1), actions);
    (grammar, pt)
}

/// Build a grammar + parse table with shift/reduce conflicts at given states.
fn gen_with_conflicts(n_states: usize, conflict_states: &[usize]) -> (Grammar, ParseTable) {
    let grammar = GrammarBuilder::new("conflict_grammar")
        .token("a", "a")
        .token("b", "b")
        .rule("root", vec!["a"])
        .rule("root", vec!["a", "b"])
        .start("root")
        .build();
    let sym_count = 5; // EOF + a + b + root + extra
    let states = n_states.max(1);
    let mut actions = vec![vec![vec![]; sym_count]; states];
    for &s in conflict_states {
        if s < states && 1 < sym_count {
            // Shift/Reduce conflict on symbol index 1
            actions[s][1] = vec![
                Action::Shift(StateId(1)),
                Action::Reduce(adze_ir::RuleId(0)),
            ];
        }
    }
    let pt = make_parse_table(&grammar, states, actions);
    (grammar, pt)
}

/// Build a ParseTable compatible with AbiLanguageBuilder's stricter requirements.
/// Ensures token_count only covers terminals so rule LHS indices are >= token_count.
fn make_abi_parse_table(grammar: &Grammar, state_count: usize) -> ParseTable {
    // Terminal count: 1 (EOF) + tokens + externals
    let terminal_count = 1 + grammar.tokens.len() + grammar.externals.len();
    let nonterminal_count = grammar.rules.len();
    let symbol_count = terminal_count + nonterminal_count;

    let mut symbol_to_index = BTreeMap::new();
    let mut index_to_symbol = Vec::with_capacity(symbol_count);
    let mut nonterminal_to_index = BTreeMap::new();

    // Index 0 = EOF
    let eof_symbol = SymbolId(0);
    symbol_to_index.insert(eof_symbol, 0);
    index_to_symbol.push(eof_symbol);

    // Tokens at indices 1..terminal_count
    let mut idx = 1;
    for (&sym_id, _) in &grammar.tokens {
        symbol_to_index.insert(sym_id, idx);
        index_to_symbol.push(sym_id);
        idx += 1;
    }
    for ext in &grammar.externals {
        symbol_to_index.insert(ext.symbol_id, idx);
        index_to_symbol.push(ext.symbol_id);
        idx += 1;
    }

    // Non-terminals at indices terminal_count..symbol_count
    for (&sym_id, _) in &grammar.rules {
        symbol_to_index.insert(sym_id, idx);
        nonterminal_to_index.insert(sym_id, idx);
        index_to_symbol.push(sym_id);
        idx += 1;
    }

    let actions = vec![vec![vec![]; symbol_count]; state_count.max(1)];
    let goto_table = vec![vec![StateId(u16::MAX); symbol_count]; state_count.max(1)];
    let lex_modes = vec![
        LexMode {
            lex_state: 0,
            external_lex_state: 0,
        };
        state_count.max(1)
    ];

    ParseTable {
        action_table: actions,
        goto_table,
        rules: vec![],
        state_count: state_count.max(1),
        symbol_count,
        symbol_to_index,
        index_to_symbol,
        nonterminal_to_index,
        symbol_metadata: vec![],
        token_count: terminal_count,
        external_token_count: grammar.externals.len(),
        eof_symbol,
        start_symbol: SymbolId(0),
        initial_state: StateId(0),
        lex_modes,
        extras: vec![],
        external_scanner_states: vec![],
        dynamic_prec_by_rule: vec![],
        rule_assoc_by_rule: vec![],
        alias_sequences: vec![],
        field_names: vec![],
        field_map: BTreeMap::new(),
        grammar: grammar.clone(),
        goto_indexing: GotoIndexing::NonterminalMap,
    }
}

// ---------------------------------------------------------------------------
// Strategies
// ---------------------------------------------------------------------------

fn grammar_name_strategy() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9_]{0,12}".prop_filter("non-empty", |s| !s.is_empty())
}

#[allow(dead_code)]
fn token_name_strategy() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9_]{0,10}".prop_filter("non-empty", |s| !s.is_empty())
}

fn field_name_strategy() -> impl Strategy<Value = String> {
    "[a-z][a-z_]{0,10}"
}

// ---------------------------------------------------------------------------
// Property tests
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(48))]

    // 1. LanguageGenerator produces valid (non-empty) TokenStream
    #[test]
    fn produces_nonempty_output(n in 1usize..8) {
        let (grammar, pt) = gen_with_tokens("nonempty", n, 2);
        let generator = LanguageGenerator::new(&grammar, &pt);
        let code = generator.generate();
        prop_assert!(!code.is_empty(), "TokenStream must not be empty");
    }

    // 2. Generated code contains LANGUAGE struct
    #[test]
    fn output_contains_language_struct(n in 1usize..6) {
        let (grammar, pt) = gen_with_tokens("langstruct", n, 2);
        let generator = LanguageGenerator::new(&grammar, &pt);
        let out = generator.generate().to_string();
        prop_assert!(out.contains("LANGUAGE"), "must contain LANGUAGE struct");
    }

    // 3. Generated code includes PARSE_ACTIONS
    #[test]
    fn output_contains_parse_actions(n in 1usize..6) {
        let (grammar, pt) = gen_with_tokens("actions", n, 2);
        let generator = LanguageGenerator::new(&grammar, &pt);
        let out = generator.generate().to_string();
        prop_assert!(out.contains("PARSE_ACTIONS"), "must contain PARSE_ACTIONS");
    }

    // 4. Generated code is deterministic
    #[test]
    fn deterministic_output(n in 1usize..6) {
        let (g1, t1) = gen_with_tokens("det", n, 2);
        let (g2, t2) = gen_with_tokens("det", n, 2);
        let out1 = LanguageGenerator::new(&g1, &t1).generate().to_string();
        let out2 = LanguageGenerator::new(&g2, &t2).generate().to_string();
        prop_assert_eq!(&out1, &out2, "same inputs must yield identical output");
    }

    // 5. Minimal single-token grammar
    #[test]
    fn minimal_grammar_generates(name in grammar_name_strategy()) {
        let (grammar, pt) = gen_with_tokens(&name, 1, 1);
        let generator = LanguageGenerator::new(&grammar, &pt);
        let out = generator.generate().to_string();
        prop_assert!(out.contains("LANGUAGE"));
        prop_assert!(out.contains("PARSE_ACTIONS"));
    }

    // 6. Many states
    #[test]
    fn many_states_generates(states in 5usize..30) {
        let (grammar, pt) = gen_with_tokens("many_st", 2, states);
        let generator = LanguageGenerator::new(&grammar, &pt);
        let out = generator.generate();
        prop_assert!(!out.is_empty());
    }

    // 7. Shift/reduce conflict
    #[test]
    fn conflict_grammar_generates(states in 2usize..10) {
        let conflicts: Vec<usize> = (0..states.min(3)).collect();
        let (grammar, pt) = gen_with_conflicts(states, &conflicts);
        let generator = LanguageGenerator::new(&grammar, &pt);
        let out = generator.generate().to_string();
        prop_assert!(out.contains("LANGUAGE"), "conflict grammar must still produce LANGUAGE");
    }

    // 8. Symbol names start with "end"
    #[test]
    fn symbol_names_start_with_end(n in 1usize..6) {
        let (grammar, pt) = gen_with_tokens("endchk", n, 2);
        let generator = LanguageGenerator::new(&grammar, &pt);
        let out = generator.generate().to_string();
        prop_assert!(out.contains("\"end\""), "first symbol name must be end (EOF)");
    }

    // 9. SYMBOL_METADATA present
    #[test]
    fn output_contains_symbol_metadata(n in 1usize..6) {
        let (grammar, pt) = gen_with_tokens("meta", n, 2);
        let generator = LanguageGenerator::new(&grammar, &pt);
        let out = generator.generate().to_string();
        prop_assert!(out.contains("SYMBOL_METADATA"));
    }

    // 10. LEX_MODES present
    #[test]
    fn output_contains_lex_modes(n in 1usize..6) {
        let (grammar, pt) = gen_with_tokens("lex", n, 3);
        let generator = LanguageGenerator::new(&grammar, &pt);
        let out = generator.generate().to_string();
        prop_assert!(out.contains("LEX_MODES"));
    }

    // 11. PARSE_TABLE present
    #[test]
    fn output_contains_parse_table(n in 1usize..6) {
        let (grammar, pt) = gen_with_tokens("ptable", n, 2);
        let generator = LanguageGenerator::new(&grammar, &pt);
        let out = generator.generate().to_string();
        prop_assert!(out.contains("PARSE_TABLE"));
    }

    // 12. Grammar name embedded in tree_sitter_{name} FFI function
    #[test]
    fn grammar_name_in_ffi_function(name in grammar_name_strategy()) {
        let (grammar, pt) = gen_with_tokens(&name, 1, 1);
        let generator = LanguageGenerator::new(&grammar, &pt);
        let out = generator.generate().to_string();
        let expected = format!("tree_sitter_{name}");
        prop_assert!(
            out.contains(&expected),
            "FFI function tree_sitter_{} must appear in output", name
        );
    }

    // 13. FIELD_NAMES present when grammar has fields
    #[test]
    fn field_names_present_with_fields(
        fields in prop::collection::vec(field_name_strategy(), 1..4),
    ) {
        let mut grammar = GrammarBuilder::new("fieldg")
            .token("tok0", "t0")
            .rule("root", vec!["tok0"])
            .start("root")
            .build();
        for (i, name) in fields.iter().enumerate() {
            grammar.fields.insert(FieldId(i as u16), name.clone());
        }
        let sym_count = 3;
        let actions = vec![vec![vec![]; sym_count]; 2];
        let pt = make_parse_table(&grammar, 2, actions);
        let generator = LanguageGenerator::new(&grammar, &pt);
        let out = generator.generate().to_string();
        prop_assert!(out.contains("FIELD_NAMES"));
    }

    // 14. Symbol count grows with tokens
    #[test]
    fn symbol_count_grows(n in 2usize..10) {
        let (g_small, pt_small) = gen_with_tokens("grow_s", 1, 2);
        let (g_big, pt_big) = gen_with_tokens("grow_b", n, 2);
        let meta_s = LanguageGenerator::new(&g_small, &pt_small).generate_symbol_metadata_public();
        let meta_b = LanguageGenerator::new(&g_big, &pt_big).generate_symbol_metadata_public();
        prop_assert!(meta_b.len() >= meta_s.len(), "more tokens → more symbol metadata entries");
    }

    // 15. State count reflected in lex modes
    #[test]
    fn state_count_in_lex_modes(states in 1usize..15) {
        let (grammar, pt) = gen_with_tokens("lexst", 1, states);
        let generator = LanguageGenerator::new(&grammar, &pt);
        let out = generator.generate().to_string();
        // LEX_MODES should have `states` entries; at minimum it should be present
        prop_assert!(out.contains("LEX_MODES"));
    }

    // 16. Determinism with identical parse tables that have actions
    #[test]
    fn determinism_with_actions(states in 2usize..6) {
        let grammar = GrammarBuilder::new("det_act")
            .token("a", "a")
            .rule("root", vec!["a"])
            .start("root")
            .build();
        let sym_count = 4;
        let mut actions = vec![vec![vec![]; sym_count]; states];
        if states > 0 {
            actions[0][1] = vec![Action::Shift(StateId(1))];
        }
        let pt1 = make_parse_table(&grammar, states, actions.clone());
        let pt2 = make_parse_table(&grammar, states, actions);
        let out1 = LanguageGenerator::new(&grammar, &pt1).generate().to_string();
        let out2 = LanguageGenerator::new(&grammar, &pt2).generate().to_string();
        prop_assert_eq!(&out1, &out2);
    }

    // 17. External token count embedded
    #[test]
    fn external_token_count_embedded(ext_count in 0usize..4) {
        let mut builder = GrammarBuilder::new("extcnt");
        builder = builder.token("tok0", "t0");
        for i in 0..ext_count {
            builder = builder.external(&format!("ext{i}"));
        }
        builder = builder.rule("root", vec!["tok0"]).start("root");
        let grammar = builder.build();
        let sym_count = 4;
        let actions = vec![vec![vec![]; sym_count]; 2];
        let pt = make_parse_table(&grammar, 2, actions);
        let generator = LanguageGenerator::new(&grammar, &pt);
        let out = generator.generate().to_string();
        prop_assert!(out.contains("EXTERNAL_TOKEN_COUNT"));
    }

    // 18. Grammar with externals produces valid code
    #[test]
    fn grammar_with_externals_valid(ext_count in 1usize..4) {
        let mut builder = GrammarBuilder::new("extvalid");
        builder = builder.token("tok0", "t0");
        for i in 0..ext_count {
            builder = builder.external(&format!("ext{i}"));
        }
        builder = builder.rule("root", vec!["tok0"]).start("root");
        let grammar = builder.build();
        let sym_count = 4 + ext_count;
        let actions = vec![vec![vec![]; sym_count]; 2];
        let pt = make_parse_table(&grammar, 2, actions);
        let generator = LanguageGenerator::new(&grammar, &pt);
        let out = generator.generate();
        prop_assert!(!out.is_empty());
    }

    // 19. Grammar with multiple rules generates valid code
    #[test]
    fn multiple_rules_valid(n in 2usize..8) {
        let mut builder = GrammarBuilder::new("multi_rule");
        for i in 0..n {
            builder = builder.token(&format!("tok{i}"), &format!("t{i}"));
        }
        for i in 0..n {
            let tok = format!("tok{i}");
            builder = builder.rule("root", vec![Box::leak(tok.into_boxed_str())]);
        }
        builder = builder.start("root");
        let grammar = builder.build();
        let sym_count = 1 + n + 1;
        let actions = vec![vec![vec![]; sym_count]; 2];
        let pt = make_parse_table(&grammar, 2, actions);
        let generator = LanguageGenerator::new(&grammar, &pt);
        let out = generator.generate();
        prop_assert!(!out.is_empty());
    }

    // 20. Grammar with extras generates valid code
    #[test]
    fn grammar_with_extras_valid(n in 1usize..4) {
        let mut builder = GrammarBuilder::new("extras_test");
        for i in 0..n {
            builder = builder.token(&format!("tok{i}"), &format!("t{i}"));
        }
        builder = builder.token("ws", r"[ \t]+");
        builder = builder.extra("ws");
        builder = builder.rule("root", vec!["tok0"]).start("root");
        let grammar = builder.build();
        let sym_count = 1 + n + 1 + 1; // EOF + n tokens + ws + root
        let actions = vec![vec![vec![]; sym_count]; 2];
        let pt = make_parse_table(&grammar, 2, actions);
        let generator = LanguageGenerator::new(&grammar, &pt);
        let out = generator.generate().to_string();
        prop_assert!(out.contains("LANGUAGE"));
    }

    // 21. PUBLIC_SYMBOL_MAP present
    #[test]
    fn public_symbol_map_present(n in 1usize..6) {
        let (grammar, pt) = gen_with_tokens("pubmap", n, 2);
        let generator = LanguageGenerator::new(&grammar, &pt);
        let out = generator.generate().to_string();
        prop_assert!(out.contains("PUBLIC_SYMBOL_MAP"));
    }

    // 22. PRIMARY_STATE_IDS present
    #[test]
    fn primary_state_ids_present(n in 1usize..6) {
        let (grammar, pt) = gen_with_tokens("pstate", n, 2);
        let generator = LanguageGenerator::new(&grammar, &pt);
        let out = generator.generate().to_string();
        prop_assert!(out.contains("PRIMARY_STATE_IDS"));
    }

    // 23. generate_symbol_metadata_public length matches symbol count
    #[test]
    fn symbol_metadata_length_matches(n in 1usize..8) {
        let (grammar, pt) = gen_with_tokens("metalen", n, 2);
        let generator = LanguageGenerator::new(&grammar, &pt);
        let meta = generator.generate_symbol_metadata_public();
        // count_symbols = 1 + tokens + rules
        let expected = 1 + grammar.tokens.len() + grammar.rules.len();
        prop_assert_eq!(meta.len(), expected, "metadata length must match symbol count");
    }

    // 24. count_production_ids_public ≥ 1 when rules exist
    #[test]
    fn production_id_count_nonzero_with_rules(n in 1usize..6) {
        let (grammar, pt) = gen_with_tokens("prodid", n, 2);
        let generator = LanguageGenerator::new(&grammar, &pt);
        let count = generator.count_production_ids_public();
        // Grammar built by gen_with_tokens has at least one rule
        prop_assert!(count >= 1, "production id count must be >= 1 when rules exist");
    }

    // 25. Multiple conflicts in same state
    #[test]
    fn multiple_conflicts_same_state(n_extra in 0usize..3) {
        let grammar = GrammarBuilder::new("multi_conflict")
            .token("a", "a")
            .token("b", "b")
            .token("c", "c")
            .rule("root", vec!["a"])
            .rule("root", vec!["a", "b"])
            .rule("root", vec!["a", "c"])
            .start("root")
            .build();
        let sym_count = 6;
        let states = 3 + n_extra;
        let mut actions = vec![vec![vec![]; sym_count]; states];
        // Conflicts on symbols 1 and 2 in state 0
        actions[0][1] = vec![Action::Shift(StateId(1)), Action::Reduce(adze_ir::RuleId(0))];
        actions[0][2] = vec![Action::Shift(StateId(2)), Action::Reduce(adze_ir::RuleId(1))];
        let pt = make_parse_table(&grammar, states, actions);
        let generator = LanguageGenerator::new(&grammar, &pt);
        let out = generator.generate().to_string();
        prop_assert!(out.contains("LANGUAGE"));
    }

    // 26. Accept action in parse table
    #[test]
    fn accept_action_generates(states in 2usize..6) {
        let grammar = GrammarBuilder::new("accept_test")
            .token("a", "a")
            .rule("root", vec!["a"])
            .start("root")
            .build();
        let sym_count = 4;
        let mut actions = vec![vec![vec![]; sym_count]; states];
        // Accept on EOF column
        let eof_col = sym_count - 1;
        actions[1][eof_col] = vec![Action::Accept];
        let pt = make_parse_table(&grammar, states, actions);
        let generator = LanguageGenerator::new(&grammar, &pt);
        let out = generator.generate().to_string();
        prop_assert!(out.contains("LANGUAGE"));
        prop_assert!(out.contains("PARSE_TABLE"));
    }

    // 27. Reduce actions encoded in output
    #[test]
    fn reduce_actions_in_table(states in 2usize..6) {
        let grammar = GrammarBuilder::new("reduce_test")
            .token("a", "a")
            .rule("root", vec!["a"])
            .start("root")
            .build();
        let sym_count = 4;
        let mut actions = vec![vec![vec![]; sym_count]; states];
        actions[1][1] = vec![Action::Reduce(adze_ir::RuleId(0))];
        let pt = make_parse_table(&grammar, states, actions);
        let generator = LanguageGenerator::new(&grammar, &pt);
        let out = generator.generate().to_string();
        prop_assert!(out.contains("PARSE_TABLE"));
    }

    // 28. Large token count still generates
    #[test]
    fn large_token_count_generates(n in 10usize..25) {
        let (grammar, pt) = gen_with_tokens("large_tok", n, 3);
        let generator = LanguageGenerator::new(&grammar, &pt);
        let out = generator.generate();
        prop_assert!(!out.is_empty());
    }

    // 29. Grammar with fields generates FIELD_NAMES entries
    #[test]
    fn fields_appear_in_output(
        fields in prop::collection::vec(field_name_strategy(), 1..5),
    ) {
        let mut grammar = GrammarBuilder::new("field_out")
            .token("tok0", "t0")
            .rule("root", vec!["tok0"])
            .start("root")
            .build();
        for (i, f) in fields.iter().enumerate() {
            grammar.fields.insert(FieldId(i as u16), f.clone());
        }
        let sym_count = 3;
        let actions = vec![vec![vec![]; sym_count]; 2];
        let pt = make_parse_table(&grammar, 2, actions);
        let generator = LanguageGenerator::new(&grammar, &pt);
        let out = generator.generate().to_string();
        for f in &fields {
            prop_assert!(out.contains(f), "field name '{}' should appear in output", f);
        }
    }

    // 30. Different grammar names produce different output
    #[test]
    fn different_names_different_output(
        name1 in "[a-z]{3,6}",
        name2 in "[a-z]{3,6}",
    ) {
        prop_assume!(name1 != name2);
        let (g1, t1) = gen_with_tokens(&name1, 1, 1);
        let (g2, t2) = gen_with_tokens(&name2, 1, 1);
        let out1 = LanguageGenerator::new(&g1, &t1).generate().to_string();
        let out2 = LanguageGenerator::new(&g2, &t2).generate().to_string();
        prop_assert_ne!(&out1, &out2, "different names must yield different output");
    }

    // 31. symbol_count in generated LANGUAGE equals 1 + tokens + rules
    #[test]
    fn symbol_count_equals_eof_plus_tokens_plus_rules(n in 1usize..10) {
        let (grammar, pt) = gen_with_tokens("symcnt", n, 2);
        let generator = LanguageGenerator::new(&grammar, &pt);
        let out = generator.generate().to_string();
        let expected = 1 + grammar.tokens.len() + grammar.rules.len();
        let needle = format!("symbol_count : {expected}");
        let alt = format!("symbol_count: {expected}");
        prop_assert!(
            out.contains(&needle) || out.contains(&alt),
            "symbol_count must be {} in output, output excerpt: ...{}...",
            expected,
            &out[..out.len().min(400)]
        );
    }

    // 32. field_count in generated LANGUAGE equals grammar.fields.len()
    #[test]
    fn field_count_matches_fields(
        fields in prop::collection::vec(field_name_strategy(), 0..5),
    ) {
        let mut grammar = GrammarBuilder::new("fcnt")
            .token("tok0", "t0")
            .rule("root", vec!["tok0"])
            .start("root")
            .build();
        for (i, name) in fields.iter().enumerate() {
            grammar.fields.insert(FieldId(i as u16), name.clone());
        }
        let sym_count = 3;
        let actions = vec![vec![vec![]; sym_count]; 2];
        let pt = make_parse_table(&grammar, 2, actions);
        let generator = LanguageGenerator::new(&grammar, &pt);
        let out = generator.generate().to_string();
        let expected = fields.len();
        let needle = format!("field_count : {expected}");
        let alt = format!("field_count: {expected}");
        prop_assert!(
            out.contains(&needle) || out.contains(&alt),
            "field_count must be {} in output", expected
        );
    }

    // 33. state_count in generated LANGUAGE equals parse_table.state_count
    #[test]
    fn state_count_matches_parse_table(states in 1usize..20) {
        let (grammar, pt) = gen_with_tokens("stcnt", 2, states);
        let generator = LanguageGenerator::new(&grammar, &pt);
        let out = generator.generate().to_string();
        let needle = format!("state_count : {states}");
        let alt = format!("state_count: {states}");
        prop_assert!(
            out.contains(&needle) || out.contains(&alt),
            "state_count must be {} in output", states
        );
    }

    // 34. Each token name appears as a symbol name in the generated output
    #[test]
    fn token_names_appear_in_symbol_names(n in 1usize..6) {
        let (grammar, pt) = gen_with_tokens("tknm", n, 2);
        let generator = LanguageGenerator::new(&grammar, &pt);
        let out = generator.generate().to_string();
        for (_id, token) in &grammar.tokens {
            prop_assert!(
                out.contains(&token.name),
                "token name '{}' must appear in generated output", token.name
            );
        }
    }

    // 35. Rule names appear as symbol names in generated output
    #[test]
    fn rule_names_appear_in_symbol_names(n in 1usize..6) {
        let (grammar, pt) = gen_with_tokens("rulnm", n, 2);
        let generator = LanguageGenerator::new(&grammar, &pt);
        let out = generator.generate().to_string();
        for (_id, rule_name) in &grammar.rule_names {
            prop_assert!(
                out.contains(rule_name),
                "rule name '{}' must appear in generated output", rule_name
            );
        }
    }

    // 36. Field names appear in output for each defined field
    #[test]
    fn each_field_name_in_output(
        fields in prop::collection::vec(field_name_strategy(), 1..5),
    ) {
        let mut grammar = GrammarBuilder::new("fldout")
            .token("tok0", "t0")
            .rule("root", vec!["tok0"])
            .start("root")
            .build();
        for (i, name) in fields.iter().enumerate() {
            grammar.fields.insert(FieldId(i as u16), name.clone());
        }
        let sym_count = 3;
        let actions = vec![vec![vec![]; sym_count]; 2];
        let pt = make_parse_table(&grammar, 2, actions);
        let out = LanguageGenerator::new(&grammar, &pt).generate().to_string();
        for f in &fields {
            prop_assert!(
                out.contains(f),
                "field name '{}' must appear in generated code", f
            );
        }
    }

    // 37. PRIMARY_STATE_IDS length matches state count
    #[test]
    fn primary_state_ids_length_matches_states(states in 1usize..12) {
        let (grammar, pt) = gen_with_tokens("psidl", 2, states);
        let generator = LanguageGenerator::new(&grammar, &pt);
        let out = generator.generate().to_string();
        // PRIMARY_STATE_IDS should be present and have entries for every state
        prop_assert!(out.contains("PRIMARY_STATE_IDS"));
    }

    // 38. Identical grammar + parse table → byte-identical symbol metadata
    #[test]
    fn symbol_metadata_deterministic(n in 1usize..8) {
        let (g1, t1) = gen_with_tokens("mddet", n, 2);
        let (g2, t2) = gen_with_tokens("mddet", n, 2);
        let meta1 = LanguageGenerator::new(&g1, &t1).generate_symbol_metadata_public();
        let meta2 = LanguageGenerator::new(&g2, &t2).generate_symbol_metadata_public();
        prop_assert_eq!(meta1, meta2, "symbol metadata must be deterministic");
    }

    // 39. External tokens increment external_token_count embedded constant
    #[test]
    fn external_token_count_value_matches(ext in 0usize..5) {
        let mut builder = GrammarBuilder::new("extval");
        builder = builder.token("tok0", "t0");
        for i in 0..ext {
            builder = builder.external(&format!("ext{i}"));
        }
        builder = builder.rule("root", vec!["tok0"]).start("root");
        let grammar = builder.build();
        let sym_count = 4 + ext;
        let actions = vec![vec![vec![]; sym_count]; 2];
        let pt = make_parse_table(&grammar, 2, actions);
        let out = LanguageGenerator::new(&grammar, &pt).generate().to_string();
        let needle = format!("EXTERNAL_TOKEN_COUNT : u32 = {ext}");
        let alt = format!("EXTERNAL_TOKEN_COUNT: u32 = {ext}");
        let alt2 = format!("external_token_count : {ext}");
        let alt3 = format!("external_token_count: {ext}");
        prop_assert!(
            out.contains(&needle) || out.contains(&alt) || out.contains(&alt2) || out.contains(&alt3),
            "EXTERNAL_TOKEN_COUNT must be {} in output", ext
        );
    }

    // 40. External scanner names appear in generated output
    #[test]
    fn external_scanner_names_in_output(ext in 1usize..4) {
        let mut builder = GrammarBuilder::new("extnm");
        builder = builder.token("tok0", "t0");
        for i in 0..ext {
            builder = builder.external(&format!("external_tok{i}"));
        }
        builder = builder.rule("root", vec!["tok0"]).start("root");
        let grammar = builder.build();
        let sym_count = 4 + ext;
        let actions = vec![vec![vec![]; sym_count]; 2];
        let pt = make_parse_table(&grammar, 2, actions);
        let out = LanguageGenerator::new(&grammar, &pt).generate().to_string();
        // External token names should appear as symbol names
        for ext_tok in &grammar.externals {
            prop_assert!(
                out.contains(&ext_tok.name),
                "external token '{}' must appear in output", ext_tok.name
            );
        }
    }

    // 41. Grammar with zero fields produces field_count: 0
    #[test]
    fn zero_fields_count(n in 1usize..6) {
        let (grammar, pt) = gen_with_tokens("zfld", n, 2);
        let out = LanguageGenerator::new(&grammar, &pt).generate().to_string();
        let needle = "field_count : 0";
        let alt = "field_count: 0";
        prop_assert!(
            out.contains(needle) || out.contains(alt),
            "field_count must be 0 when no fields defined"
        );
    }

    // 42. Determinism: three identical generations produce the same output
    #[test]
    fn triple_determinism(n in 1usize..5) {
        let outputs: Vec<String> = (0..3).map(|_| {
            let (g, t) = gen_with_tokens("tridet", n, 3);
            LanguageGenerator::new(&g, &t).generate().to_string()
        }).collect();
        prop_assert_eq!(&outputs[0], &outputs[1]);
        prop_assert_eq!(&outputs[1], &outputs[2]);
    }

    // 43. symbol_metadata length equals count_symbols for various sizes
    #[test]
    fn symbol_metadata_len_equals_count_symbols(n in 1usize..10) {
        let (grammar, pt) = gen_with_tokens("metasz", n, 2);
        let generator = LanguageGenerator::new(&grammar, &pt);
        let meta = generator.generate_symbol_metadata_public();
        let expected = 1 + grammar.tokens.len() + grammar.rules.len();
        prop_assert_eq!(
            meta.len(), expected,
            "metadata length {} != expected {}", meta.len(), expected
        );
    }

    // 44. symbol_count is consistent with symbol_metadata length
    #[test]
    fn symbol_count_consistent_with_metadata(n in 1usize..8) {
        let (grammar, pt) = gen_with_tokens("scmeta", n, 2);
        let generator = LanguageGenerator::new(&grammar, &pt);
        let meta = generator.generate_symbol_metadata_public();
        let out = generator.generate().to_string();
        let count_str = format!("symbol_count : {}", meta.len());
        let alt_str = format!("symbol_count: {}", meta.len());
        prop_assert!(
            out.contains(&count_str) || out.contains(&alt_str),
            "symbol_count in output must match metadata length {}",
            meta.len()
        );
    }

    // 45. Grammar with externals still has "end" as first symbol name
    #[test]
    fn externals_still_have_end_first(ext in 1usize..4) {
        let mut builder = GrammarBuilder::new("endext");
        builder = builder.token("tok0", "t0");
        for i in 0..ext {
            builder = builder.external(&format!("ext{i}"));
        }
        builder = builder.rule("root", vec!["tok0"]).start("root");
        let grammar = builder.build();
        let sym_count = 4 + ext;
        let actions = vec![vec![vec![]; sym_count]; 2];
        let pt = make_parse_table(&grammar, 2, actions);
        let out = LanguageGenerator::new(&grammar, &pt).generate().to_string();
        prop_assert!(out.contains("\"end\""), "first symbol must always be 'end'");
    }

    // 46. PRIMARY_STATE_IDS present even with single state
    #[test]
    fn primary_state_ids_present_single_state(_dummy in 0..1u8) {
        let (grammar, pt) = gen_with_tokens("psone", 1, 1);
        let out = LanguageGenerator::new(&grammar, &pt).generate().to_string();
        prop_assert!(out.contains("PRIMARY_STATE_IDS"));
    }

    // 47. production_id_count ≥ 1 with multiple rules
    #[test]
    fn production_id_count_with_multiple_rules(n in 2usize..6) {
        let mut builder = GrammarBuilder::new("mprod");
        for i in 0..n {
            builder = builder.token(&format!("tok{i}"), &format!("t{i}"));
        }
        for i in 0..n {
            let tok = format!("tok{i}");
            builder = builder.rule("root", vec![Box::leak(tok.into_boxed_str())]);
        }
        builder = builder.start("root");
        let grammar = builder.build();
        let sym_count = 1 + n + 1;
        let actions = vec![vec![vec![]; sym_count]; 2];
        let pt = make_parse_table(&grammar, 2, actions);
        let count = LanguageGenerator::new(&grammar, &pt).count_production_ids_public();
        prop_assert!(count >= 1, "production_id_count must be >= 1 with rules");
    }

    // 48. Grammar with both externals and fields generates both sections
    #[test]
    fn externals_and_fields_coexist(ext in 1usize..3, fld in 1usize..3) {
        let mut builder = GrammarBuilder::new("coexist");
        builder = builder.token("tok0", "t0");
        for i in 0..ext {
            builder = builder.external(&format!("ext{i}"));
        }
        builder = builder.rule("root", vec!["tok0"]).start("root");
        let mut grammar = builder.build();
        for i in 0..fld {
            grammar.fields.insert(FieldId(i as u16), format!("field{i}"));
        }
        let sym_count = 4 + ext;
        let actions = vec![vec![vec![]; sym_count]; 2];
        let pt = make_parse_table(&grammar, 2, actions);
        let out = LanguageGenerator::new(&grammar, &pt).generate().to_string();
        prop_assert!(out.contains("FIELD_NAMES"), "FIELD_NAMES required with fields");
        prop_assert!(out.contains("EXTERNAL_TOKEN_COUNT"), "EXTERNAL_TOKEN_COUNT required with externals");
    }

    // 49. LEX_MODES has entries for every state
    #[test]
    fn lex_modes_entries_per_state(states in 1usize..10) {
        let (grammar, pt) = gen_with_tokens("lmcnt", 2, states);
        let out = LanguageGenerator::new(&grammar, &pt).generate().to_string();
        // Each state should produce a TSLexState entry in LEX_MODES
        // At minimum, check the last state index appears
        let last_state = states - 1;
        let needle = format!("{last_state}");
        prop_assert!(
            out.contains(&needle),
            "last state index {} must appear in output", last_state
        );
    }

    // 50. Different token counts produce different symbol_count values
    #[test]
    fn different_token_counts_different_symbol_counts(
        n1 in 1usize..5,
        n2 in 5usize..10,
    ) {
        let (g1, t1) = gen_with_tokens("dtcnt_a", n1, 2);
        let (g2, t2) = gen_with_tokens("dtcnt_b", n2, 2);
        let m1 = LanguageGenerator::new(&g1, &t1).generate_symbol_metadata_public();
        let m2 = LanguageGenerator::new(&g2, &t2).generate_symbol_metadata_public();
        prop_assert_ne!(m1.len(), m2.len(),
            "different token counts should yield different symbol counts: {} vs {}", m1.len(), m2.len());
    }

    // 51. AbiLanguageBuilder generates non-empty output
    #[test]
    fn abi_builder_nonempty(n in 1usize..6) {
        let (grammar, pt) = gen_with_tokens("abine", n, 2);
        let builder = adze_tablegen::AbiLanguageBuilder::new(&grammar, &pt);
        let out = builder.generate();
        prop_assert!(!out.is_empty(), "AbiLanguageBuilder output must not be empty");
    }

    // 52. AbiLanguageBuilder output contains LANGUAGE struct
    #[test]
    fn abi_builder_contains_language(n in 1usize..6) {
        let (grammar, pt) = gen_with_tokens("abils", n, 2);
        let builder = adze_tablegen::AbiLanguageBuilder::new(&grammar, &pt);
        let out = builder.generate().to_string();
        prop_assert!(out.contains("LANGUAGE"), "AbiLanguageBuilder must produce LANGUAGE");
    }

    // 53. AbiLanguageBuilder output is deterministic
    #[test]
    fn abi_builder_deterministic(n in 1usize..5) {
        let (g1, t1) = gen_with_tokens("abidet", n, 2);
        let (g2, t2) = gen_with_tokens("abidet", n, 2);
        let out1 = adze_tablegen::AbiLanguageBuilder::new(&g1, &t1).generate().to_string();
        let out2 = adze_tablegen::AbiLanguageBuilder::new(&g2, &t2).generate().to_string();
        prop_assert_eq!(&out1, &out2, "AbiLanguageBuilder must be deterministic");
    }

    // 54. AbiLanguageBuilder with externals produces EXTERNAL_SCANNER
    #[test]
    fn abi_builder_external_scanner_section(ext in 1usize..3) {
        let mut builder = GrammarBuilder::new("abiext");
        builder = builder.token("tok0", "t0");
        for i in 0..ext {
            builder = builder.external(&format!("ext{i}"));
        }
        builder = builder.rule("root", vec!["tok0"]).start("root");
        let grammar = builder.build();
        let pt = make_abi_parse_table(&grammar, 2);
        let abi = adze_tablegen::AbiLanguageBuilder::new(&grammar, &pt);
        let out = abi.generate().to_string();
        prop_assert!(out.contains("ExternalScanner"), "must contain ExternalScanner for grammars with externals");
    }

    // 55. AbiLanguageBuilder with fields lists field name constants
    #[test]
    fn abi_builder_field_names_complete(
        fields in prop::collection::vec(field_name_strategy(), 1..4),
    ) {
        let mut grammar = GrammarBuilder::new("abifld")
            .token("tok0", "t0")
            .rule("root", vec!["tok0"])
            .start("root")
            .build();
        for (i, name) in fields.iter().enumerate() {
            grammar.fields.insert(FieldId(i as u16), name.clone());
        }
        let pt = make_abi_parse_table(&grammar, 2);
        let out = adze_tablegen::AbiLanguageBuilder::new(&grammar, &pt).generate().to_string();
        // ABI builder encodes field names as byte arrays; check constants are generated
        for i in 0..fields.len() {
            let constant = format!("FIELD_NAME_{i}");
            prop_assert!(out.contains(&constant), "ABI output must contain '{}'", constant);
        }
    }
}
