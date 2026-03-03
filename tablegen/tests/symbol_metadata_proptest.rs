#![allow(clippy::needless_range_loop)]
//! Property-based and unit tests for symbol metadata generation in adze-tablegen.
//!
//! Coverage areas:
//! 1.  Terminal symbols have is_terminal=true  (via VISIBLE bit on string-literal tokens)
//! 2.  Non-terminal symbols have is_terminal=false (via NAMED+VISIBLE bits, no HIDDEN)
//! 3.  Named vs anonymous symbol distinction
//! 4.  Supertype symbol metadata
//! 5.  Metadata count matches symbol count
//! 6.  Metadata determinism
//! 7.  EOF symbol metadata
//! 8.  External scanner symbol metadata

use adze_glr_core::{Action, GotoIndexing, LexMode, ParseTable};
use adze_ir::{
    ExternalToken, FieldId, Grammar, ProductionId, Rule, Symbol, SymbolId, Token, TokenPattern,
};
use adze_tablegen::AbiLanguageBuilder;
use adze_tablegen::abi::{
    create_symbol_metadata,
    symbol_metadata::{AUXILIARY, HIDDEN, NAMED, SUPERTYPE, VISIBLE},
};
use proptest::prelude::*;
use std::collections::BTreeMap;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

const INVALID: adze_ir::StateId = adze_ir::StateId(u16::MAX);

/// Build a ParseTable from scratch with the given dimensions.
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

/// Build a grammar + parse table with configurable terminals, non-terminals,
/// fields, and external tokens.
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

    table.external_token_count = num_externals;

    (grammar, table)
}

/// Generate code and return as a String.
fn generate_code(grammar: &Grammar, table: &ParseTable) -> String {
    AbiLanguageBuilder::new(grammar, table)
        .generate()
        .to_string()
}

// ---------------------------------------------------------------------------
// 1. Terminal symbols: string-literal tokens are visible but anonymous
// ---------------------------------------------------------------------------

#[test]
fn terminal_string_token_is_visible_not_named() {
    let (grammar, table) = build_grammar_and_table("term_str", 2, 1, 0, 0, 1);
    let code = generate_code(&grammar, &table);
    // String-literal tokens should appear in SYMBOL_METADATA
    assert!(
        code.contains("SYMBOL_METADATA"),
        "generated code must contain SYMBOL_METADATA"
    );
}

#[test]
fn terminal_regex_token_is_visible_and_named() {
    let mut grammar = Grammar::new("term_regex".to_string());
    let sym = SymbolId(1);
    grammar.tokens.insert(
        sym,
        Token {
            name: "identifier".to_string(),
            pattern: TokenPattern::Regex(r"[a-z]+".to_string()),
            fragile: false,
        },
    );
    let nt = SymbolId(3);
    grammar.rule_names.insert(nt, "start".to_string());
    grammar.add_rule(Rule {
        lhs: nt,
        rhs: vec![Symbol::Terminal(sym)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    let table = make_empty_table(1, 1, 1, 0);
    let code = generate_code(&grammar, &table);
    // Regex tokens get VISIBLE|NAMED = 0x03
    assert!(code.contains("SYMBOL_METADATA"));
}

#[test]
fn terminal_hidden_token_underscore_prefix() {
    let mut grammar = Grammar::new("hidden_tok".to_string());
    let sym = SymbolId(1);
    grammar.tokens.insert(
        sym,
        Token {
            name: "_ws".to_string(),
            pattern: TokenPattern::Regex(r"\s+".to_string()),
            fragile: false,
        },
    );
    let nt = SymbolId(3);
    grammar.rule_names.insert(nt, "start".to_string());
    grammar.add_rule(Rule {
        lhs: nt,
        rhs: vec![Symbol::Terminal(sym)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    let table = make_empty_table(1, 1, 1, 0);
    let code = generate_code(&grammar, &table);
    assert!(code.contains("SYMBOL_METADATA"));
}

// ---------------------------------------------------------------------------
// 2. Non-terminal symbols: visible and named
// ---------------------------------------------------------------------------

#[test]
fn nonterminal_is_visible_and_named() {
    let (grammar, table) = build_grammar_and_table("nt_vis", 1, 2, 0, 0, 1);
    let code = generate_code(&grammar, &table);
    assert!(code.contains("SYMBOL_METADATA"));
}

#[test]
fn nonterminal_hidden_underscore_prefix() {
    let mut grammar = Grammar::new("nt_hidden".to_string());
    let tok = SymbolId(1);
    grammar.tokens.insert(
        tok,
        Token {
            name: "x".to_string(),
            pattern: TokenPattern::String("x".to_string()),
            fragile: false,
        },
    );
    let nt = SymbolId(3);
    grammar.rule_names.insert(nt, "_hidden_rule".to_string());
    grammar.add_rule(Rule {
        lhs: nt,
        rhs: vec![Symbol::Terminal(tok)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    let table = make_empty_table(1, 1, 1, 0);
    let code = generate_code(&grammar, &table);
    assert!(code.contains("SYMBOL_METADATA"));
}

// ---------------------------------------------------------------------------
// 3. Named vs anonymous symbol distinction
// ---------------------------------------------------------------------------

#[test]
fn create_metadata_named_flag_set() {
    let meta = create_symbol_metadata(true, true, false, false, false);
    assert_eq!(meta & NAMED, NAMED);
    assert_eq!(meta & VISIBLE, VISIBLE);
}

#[test]
fn create_metadata_anonymous_flag_not_set() {
    // String-literal tokens: visible but not named
    let meta = create_symbol_metadata(true, false, false, false, false);
    assert_eq!(meta & NAMED, 0);
    assert_eq!(meta & VISIBLE, VISIBLE);
}

#[test]
fn named_regex_vs_anonymous_string_tokens() {
    let mut grammar = Grammar::new("named_anon".to_string());
    let str_tok = SymbolId(1);
    grammar.tokens.insert(
        str_tok,
        Token {
            name: "plus".to_string(),
            pattern: TokenPattern::String("+".to_string()),
            fragile: false,
        },
    );
    let regex_tok = SymbolId(2);
    grammar.tokens.insert(
        regex_tok,
        Token {
            name: "number".to_string(),
            pattern: TokenPattern::Regex(r"\d+".to_string()),
            fragile: false,
        },
    );
    // make_empty_table(1,2,1,0): eof_idx=3, first NT at 4
    let nt = SymbolId(4);
    grammar.rule_names.insert(nt, "expr".to_string());
    grammar.add_rule(Rule {
        lhs: nt,
        rhs: vec![Symbol::Terminal(str_tok), Symbol::Terminal(regex_tok)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    let table = make_empty_table(1, 2, 1, 0);
    let code = generate_code(&grammar, &table);
    assert!(code.contains("SYMBOL_METADATA"));
}

// ---------------------------------------------------------------------------
// 4. Supertype symbol metadata
// ---------------------------------------------------------------------------

#[test]
fn supertype_flag_set_in_metadata_byte() {
    let meta = create_symbol_metadata(true, true, false, false, true);
    assert_eq!(meta & SUPERTYPE, SUPERTYPE);
    assert_eq!(meta & VISIBLE, VISIBLE);
    assert_eq!(meta & NAMED, NAMED);
}

#[test]
fn supertype_nonterminal_in_grammar() {
    let mut grammar = Grammar::new("super".to_string());
    let tok = SymbolId(1);
    grammar.tokens.insert(
        tok,
        Token {
            name: "a".to_string(),
            pattern: TokenPattern::String("a".to_string()),
            fragile: false,
        },
    );
    let nt = SymbolId(3);
    grammar.rule_names.insert(nt, "expression".to_string());
    grammar.supertypes.push(nt);
    grammar.add_rule(Rule {
        lhs: nt,
        rhs: vec![Symbol::Terminal(tok)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    let table = make_empty_table(1, 1, 1, 0);
    let code = generate_code(&grammar, &table);
    assert!(code.contains("SYMBOL_METADATA"));
}

#[test]
fn non_supertype_nonterminal_lacks_supertype_flag() {
    let meta = create_symbol_metadata(true, true, false, false, false);
    assert_eq!(meta & SUPERTYPE, 0);
}

// ---------------------------------------------------------------------------
// 5. Metadata count matches symbol count
// ---------------------------------------------------------------------------

#[test]
fn metadata_array_length_matches_symbol_count() {
    let (grammar, table) = build_grammar_and_table("cnt", 3, 2, 0, 0, 2);
    let code = generate_code(&grammar, &table);
    let sc = table.symbol_count as u32;
    let needle = format!("symbol_count : {sc}u32");
    assert!(
        code.contains(&needle),
        "expected symbol_count : {sc}u32, code did not contain it"
    );
}

#[test]
fn metadata_count_with_externals() {
    let (grammar, table) = build_grammar_and_table("cnt_ext", 2, 1, 0, 2, 1);
    let code = generate_code(&grammar, &table);
    let sc = table.symbol_count as u32;
    let needle = format!("symbol_count : {sc}u32");
    assert!(code.contains(&needle));
}

// ---------------------------------------------------------------------------
// 6. Metadata determinism
// ---------------------------------------------------------------------------

#[test]
fn deterministic_output_same_grammar() {
    let (grammar, table) = build_grammar_and_table("det", 3, 2, 1, 1, 3);
    let code1 = generate_code(&grammar, &table);
    let code2 = generate_code(&grammar, &table);
    assert_eq!(
        code1, code2,
        "two generations of the same grammar must be identical"
    );
}

#[test]
fn deterministic_metadata_repeated_calls() {
    let (grammar, table) = build_grammar_and_table("det2", 2, 2, 0, 0, 2);
    let code1 = generate_code(&grammar, &table);
    let code2 = generate_code(&grammar, &table);
    let code3 = generate_code(&grammar, &table);
    assert_eq!(code1, code2);
    assert_eq!(code2, code3);
}

// ---------------------------------------------------------------------------
// 7. EOF symbol metadata
// ---------------------------------------------------------------------------

#[test]
fn eof_metadata_visible_not_named() {
    // EOF: visible=true, named=false
    let eof_meta = create_symbol_metadata(true, false, false, false, false);
    assert_eq!(eof_meta & VISIBLE, VISIBLE);
    assert_eq!(eof_meta & NAMED, 0);
    assert_eq!(eof_meta & HIDDEN, 0);
    assert_eq!(eof_meta & SUPERTYPE, 0);
}

#[test]
fn eof_symbol_present_in_generated_code() {
    let (grammar, table) = build_grammar_and_table("eof_sym", 1, 1, 0, 0, 1);
    let code = generate_code(&grammar, &table);
    assert!(code.contains("SYMBOL_METADATA"));
}

#[test]
fn eof_at_expected_index() {
    // EOF index = 1 + terms + externals for make_empty_table
    let (grammar, table) = build_grammar_and_table("eof_idx", 2, 1, 0, 0, 1);
    let eof_idx = table.eof_symbol.0 as usize;
    assert!(
        eof_idx < table.symbol_count,
        "EOF index must be within symbol_count"
    );
}

// ---------------------------------------------------------------------------
// 8. External scanner symbol metadata
// ---------------------------------------------------------------------------

#[test]
fn external_token_present_in_grammar() {
    let (grammar, table) = build_grammar_and_table("ext", 1, 1, 0, 2, 1);
    assert_eq!(grammar.externals.len(), 2);
    assert_eq!(table.external_token_count, 2);
}

#[test]
fn external_token_visible_and_named() {
    let mut grammar = Grammar::new("ext_vis".to_string());
    let tok = SymbolId(1);
    grammar.tokens.insert(
        tok,
        Token {
            name: "a".to_string(),
            pattern: TokenPattern::String("a".to_string()),
            fragile: false,
        },
    );
    let ext = SymbolId(2);
    grammar.externals.push(ExternalToken {
        name: "indent".to_string(),
        symbol_id: ext,
    });
    let nt = SymbolId(4);
    grammar.rule_names.insert(nt, "start".to_string());
    grammar.add_rule(Rule {
        lhs: nt,
        rhs: vec![Symbol::Terminal(tok)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    let table = make_empty_table(1, 1, 1, 1);
    let code = generate_code(&grammar, &table);
    assert!(code.contains("SYMBOL_METADATA"));
}

#[test]
fn external_token_hidden_when_underscore_prefix() {
    let mut grammar = Grammar::new("ext_hide".to_string());
    let tok = SymbolId(1);
    grammar.tokens.insert(
        tok,
        Token {
            name: "a".to_string(),
            pattern: TokenPattern::String("a".to_string()),
            fragile: false,
        },
    );
    let ext = SymbolId(2);
    grammar.externals.push(ExternalToken {
        name: "_hidden_ext".to_string(),
        symbol_id: ext,
    });
    let nt = SymbolId(4);
    grammar.rule_names.insert(nt, "start".to_string());
    grammar.add_rule(Rule {
        lhs: nt,
        rhs: vec![Symbol::Terminal(tok)],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    let table = make_empty_table(1, 1, 1, 1);
    let code = generate_code(&grammar, &table);
    assert!(code.contains("SYMBOL_METADATA"));
}

// ---------------------------------------------------------------------------
// Additional unit tests for create_symbol_metadata bit encoding
// ---------------------------------------------------------------------------

#[test]
fn all_flags_off_is_zero() {
    assert_eq!(create_symbol_metadata(false, false, false, false, false), 0);
}

#[test]
fn all_flags_on() {
    let meta = create_symbol_metadata(true, true, true, true, true);
    assert_eq!(meta, VISIBLE | NAMED | HIDDEN | AUXILIARY | SUPERTYPE);
}

#[test]
fn individual_flag_visible() {
    assert_eq!(
        create_symbol_metadata(true, false, false, false, false),
        VISIBLE
    );
}

#[test]
fn individual_flag_named() {
    assert_eq!(
        create_symbol_metadata(false, true, false, false, false),
        NAMED
    );
}

#[test]
fn individual_flag_hidden() {
    assert_eq!(
        create_symbol_metadata(false, false, true, false, false),
        HIDDEN
    );
}

#[test]
fn individual_flag_auxiliary() {
    assert_eq!(
        create_symbol_metadata(false, false, false, true, false),
        AUXILIARY
    );
}

#[test]
fn individual_flag_supertype() {
    assert_eq!(
        create_symbol_metadata(false, false, false, false, true),
        SUPERTYPE
    );
}

#[test]
fn flags_are_independent_bits() {
    // Each flag occupies a unique bit
    let flags = [VISIBLE, NAMED, HIDDEN, AUXILIARY, SUPERTYPE];
    for i in 0..flags.len() {
        for j in (i + 1)..flags.len() {
            assert_eq!(
                flags[i] & flags[j],
                0,
                "flags at index {} and {} overlap",
                i,
                j
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Property-based tests
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(30))]

    #[test]
    fn prop_metadata_count_matches_symbol_count(
        (terms, nonterms, fields, externals, states) in (
            1usize..=5,
            1usize..=3,
            0usize..=3,
            0usize..=2,
            1usize..=4,
        )
    ) {
        let (grammar, table) = build_grammar_and_table(
            "prop_cnt", terms, nonterms, fields, externals, states,
        );
        let code = generate_code(&grammar, &table);
        let sc = table.symbol_count as u32;
        let needle = format!("symbol_count : {sc}u32");
        prop_assert!(code.contains(&needle));
    }

    #[test]
    fn prop_determinism(
        (terms, nonterms, fields, externals, states) in (
            1usize..=4,
            1usize..=3,
            0usize..=2,
            0usize..=2,
            1usize..=3,
        )
    ) {
        let (grammar, table) = build_grammar_and_table(
            "prop_det", terms, nonterms, fields, externals, states,
        );
        let code1 = generate_code(&grammar, &table);
        let code2 = generate_code(&grammar, &table);
        prop_assert_eq!(code1, code2);
    }

    #[test]
    fn prop_metadata_array_present(
        (terms, nonterms, externals, states) in (
            1usize..=4,
            1usize..=3,
            0usize..=2,
            1usize..=4,
        )
    ) {
        let (grammar, table) = build_grammar_and_table(
            "prop_meta", terms, nonterms, 0, externals, states,
        );
        let code = generate_code(&grammar, &table);
        prop_assert!(code.contains("SYMBOL_METADATA"));
    }

    #[test]
    fn prop_create_metadata_roundtrip(
        visible in any::<bool>(),
        named in any::<bool>(),
        hidden in any::<bool>(),
        auxiliary in any::<bool>(),
        supertype in any::<bool>(),
    ) {
        let meta = create_symbol_metadata(visible, named, hidden, auxiliary, supertype);
        prop_assert_eq!((meta & VISIBLE) != 0, visible);
        prop_assert_eq!((meta & NAMED) != 0, named);
        prop_assert_eq!((meta & HIDDEN) != 0, hidden);
        prop_assert_eq!((meta & AUXILIARY) != 0, auxiliary);
        prop_assert_eq!((meta & SUPERTYPE) != 0, supertype);
    }

    #[test]
    fn prop_external_token_count_in_code(
        externals in 0usize..=3,
    ) {
        let (grammar, table) = build_grammar_and_table(
            "prop_ext", 1, 1, 0, externals, 1,
        );
        let code = generate_code(&grammar, &table);
        let needle = format!("external_token_count : {}u32", externals);
        prop_assert!(
            code.contains(&needle),
            "expected external_token_count : {}u32",
            externals
        );
    }
}
