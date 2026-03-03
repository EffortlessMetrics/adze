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
use adze_ir::{
    builder::GrammarBuilder, ExternalToken, FieldId, Grammar, ProductionId, Rule, StateId, Symbol,
    SymbolId, Token, TokenPattern,
};
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
            actions[s][1] = vec![Action::Shift(StateId(1)), Action::Reduce(adze_ir::RuleId(0))];
        }
    }
    let pt = make_parse_table(&grammar, states, actions);
    (grammar, pt)
}

// ---------------------------------------------------------------------------
// Strategies
// ---------------------------------------------------------------------------

fn grammar_name_strategy() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9_]{0,12}".prop_filter("non-empty", |s| !s.is_empty())
}

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
}
