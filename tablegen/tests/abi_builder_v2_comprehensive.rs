//! Comprehensive v2 tests for `AbiLanguageBuilder` covering construction,
//! generation output, grammar sizes, precedence, determinism, keyword
//! checks, `StaticLanguageGenerator` comparison, and complex grammars.
//!
//! Target: 50+ tests exercising the public API of `adze_tablegen`.

use adze_glr_core::{FirstFollowSets, GotoIndexing, LexMode, ParseTable, build_lr1_automaton};
use adze_ir::builder::GrammarBuilder;
use adze_ir::{
    Associativity, ExternalToken, FieldId, Grammar, ProductionId, Rule, Symbol, SymbolId, Token,
    TokenPattern,
};
use adze_tablegen::{AbiLanguageBuilder, StaticLanguageGenerator};
use std::collections::BTreeMap;

// ===========================================================================
// Helpers
// ===========================================================================

const INVALID: adze_glr_core::StateId = adze_glr_core::StateId(u16::MAX);

/// Build a grammar + parse table pair using a manual layout convention.
fn build_pair(
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

    let eof_idx = 1 + num_terms + num_externals;
    let symbol_count = eof_idx + 1 + num_nonterms;

    let actions = vec![vec![vec![]; symbol_count]; num_states];
    let gotos = vec![vec![INVALID; symbol_count]; num_states];

    let eof_symbol = SymbolId(eof_idx as u16);
    let start_symbol = SymbolId((eof_idx + 1) as u16);

    let mut symbol_to_index = BTreeMap::new();
    let mut index_to_symbol = vec![SymbolId(0); symbol_count];
    for i in 0..symbol_count {
        let sym = SymbolId(i as u16);
        symbol_to_index.insert(sym, i);
        index_to_symbol[i] = sym;
    }

    let mut grammar = Grammar::new(name.to_string());

    let first_term = SymbolId(1);
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

    let mut nonterminal_to_index = BTreeMap::new();
    nonterminal_to_index.insert(start_symbol, start_symbol.0 as usize);

    let table = ParseTable {
        action_table: actions,
        goto_table: gotos,
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
        initial_state: adze_glr_core::StateId(0),
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

/// Minimal single-token, single-rule pair.
fn minimal() -> (Grammar, ParseTable) {
    build_pair("minimal", 1, 1, 0, 0, 2)
}

/// Generate code string from grammar+table.
fn gen_code(grammar: &Grammar, table: &ParseTable) -> String {
    AbiLanguageBuilder::new(grammar, table)
        .generate()
        .to_string()
}

/// Full pipeline: GrammarBuilder → normalize → FIRST/FOLLOW → LR(1) → ABI code.
fn full_pipeline(g: Grammar) -> (String, ParseTable) {
    let mut grammar = g;
    let ff =
        FirstFollowSets::compute_normalized(&mut grammar).expect("FIRST/FOLLOW computation failed");
    let table = build_lr1_automaton(&grammar, &ff).expect("LR(1) build failed");
    let code = AbiLanguageBuilder::new(&grammar, &table)
        .generate()
        .to_string();
    (code, table)
}

// ===========================================================================
// 1. AbiLanguageBuilder construction
// ===========================================================================

#[test]
fn construct_from_minimal_grammar() {
    let (g, t) = minimal();
    let _builder = AbiLanguageBuilder::new(&g, &t);
}

#[test]
fn construct_from_large_grammar() {
    let (g, t) = build_pair("lg", 30, 10, 5, 2, 20);
    let _builder = AbiLanguageBuilder::new(&g, &t);
}

#[test]
fn construct_with_zero_fields() {
    let (g, t) = build_pair("nf", 2, 1, 0, 0, 2);
    let _builder = AbiLanguageBuilder::new(&g, &t);
}

#[test]
fn construct_with_externals() {
    let (g, t) = build_pair("ext", 1, 1, 0, 3, 2);
    let _builder = AbiLanguageBuilder::new(&g, &t);
}

#[test]
fn construct_multiple_times_same_input() {
    let (g, t) = minimal();
    let _b1 = AbiLanguageBuilder::new(&g, &t);
    let _b2 = AbiLanguageBuilder::new(&g, &t);
}

// ===========================================================================
// 2. generate() returns non-empty TokenStream
// ===========================================================================

#[test]
fn generate_returns_nonempty_token_stream() {
    let (g, t) = minimal();
    let ts = AbiLanguageBuilder::new(&g, &t).generate();
    assert!(!ts.is_empty(), "TokenStream must not be empty");
}

#[test]
fn generate_nonempty_for_two_tokens() {
    let (g, t) = build_pair("two", 2, 1, 0, 0, 3);
    let ts = AbiLanguageBuilder::new(&g, &t).generate();
    assert!(!ts.is_empty());
}

#[test]
fn generate_nonempty_with_fields() {
    let (g, t) = build_pair("wf", 1, 1, 3, 0, 2);
    let ts = AbiLanguageBuilder::new(&g, &t).generate();
    assert!(!ts.is_empty());
}

#[test]
fn generate_nonempty_with_externals() {
    let (g, t) = build_pair("we", 1, 1, 0, 2, 2);
    let ts = AbiLanguageBuilder::new(&g, &t).generate();
    assert!(!ts.is_empty());
}

// ===========================================================================
// 3. TokenStream to_string is non-empty
// ===========================================================================

#[test]
fn to_string_nonempty_minimal() {
    let (g, t) = minimal();
    let code = gen_code(&g, &t);
    assert!(!code.is_empty());
}

#[test]
fn to_string_nonempty_medium() {
    let (g, t) = build_pair("med", 5, 3, 2, 0, 6);
    let code = gen_code(&g, &t);
    assert!(!code.is_empty());
}

#[test]
fn to_string_has_substantial_length() {
    let (g, t) = minimal();
    let code = gen_code(&g, &t);
    assert!(
        code.len() > 100,
        "generated code should be substantial, got {} bytes",
        code.len()
    );
}

// ===========================================================================
// 4. Various grammar sizes
// ===========================================================================

#[test]
fn size_one_token_one_rule() {
    let (g, t) = build_pair("s1", 1, 1, 0, 0, 2);
    let code = gen_code(&g, &t);
    assert!(code.contains("LANGUAGE"));
}

#[test]
fn size_five_tokens_two_rules() {
    let (g, t) = build_pair("s5", 5, 2, 0, 0, 4);
    let code = gen_code(&g, &t);
    assert!(code.contains("LANGUAGE"));
}

#[test]
fn size_ten_tokens_five_rules() {
    let (g, t) = build_pair("s10", 10, 5, 0, 0, 8);
    let code = gen_code(&g, &t);
    assert!(code.contains("LANGUAGE"));
}

#[test]
fn size_fifty_tokens() {
    let (g, t) = build_pair("s50", 50, 5, 3, 0, 10);
    let code = gen_code(&g, &t);
    assert!(code.contains("LANGUAGE"));
}

#[test]
fn size_twenty_nonterminals() {
    let (g, t) = build_pair("s20nt", 5, 20, 0, 0, 15);
    let code = gen_code(&g, &t);
    assert!(code.contains("LANGUAGE"));
}

#[test]
fn size_hundred_states() {
    let (g, t) = build_pair("s100st", 5, 3, 0, 0, 100);
    let code = gen_code(&g, &t);
    assert!(code.contains("state_count : 100u32"));
}

#[test]
fn size_many_fields() {
    let (g, t) = build_pair("smf", 3, 2, 15, 0, 5);
    let code = gen_code(&g, &t);
    assert!(code.contains("field_count : 15u32"));
}

#[test]
fn size_many_externals() {
    let (g, t) = build_pair("sme", 3, 2, 0, 5, 5);
    let code = gen_code(&g, &t);
    assert!(code.contains("external_token_count : 5u32"));
}

// ===========================================================================
// 5. Grammar with precedence
// ===========================================================================

#[test]
fn precedence_left_generates() {
    let grammar = GrammarBuilder::new("prec_l")
        .token("num", "[0-9]+")
        .token("plus", "+")
        .rule("expr", vec!["num"])
        .rule_with_precedence("expr", vec!["expr", "plus", "expr"], 1, Associativity::Left)
        .start("expr")
        .build();
    let (code, _) = full_pipeline(grammar);
    assert!(code.contains("LANGUAGE"));
}

#[test]
fn precedence_right_generates() {
    let grammar = GrammarBuilder::new("prec_r")
        .token("x", "x")
        .token("hat", "^")
        .rule("expr", vec!["x"])
        .rule_with_precedence("expr", vec!["expr", "hat", "expr"], 1, Associativity::Right)
        .start("expr")
        .build();
    let (code, _) = full_pipeline(grammar);
    assert!(code.contains("LANGUAGE"));
}

#[test]
fn precedence_multiple_levels() {
    let grammar = GrammarBuilder::new("prec_m")
        .token("num", "[0-9]+")
        .token("plus", "+")
        .token("star", "*")
        .rule("expr", vec!["num"])
        .rule_with_precedence("expr", vec!["expr", "plus", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "star", "expr"], 2, Associativity::Left)
        .start("expr")
        .build();
    let (code, _) = full_pipeline(grammar);
    assert!(code.contains("LANGUAGE"));
}

#[test]
fn precedence_code_contains_tree_sitter_fn() {
    let grammar = GrammarBuilder::new("prec_fn")
        .token("a", "a")
        .token("op", "+")
        .rule("e", vec!["a"])
        .rule_with_precedence("e", vec!["e", "op", "e"], 1, Associativity::Left)
        .start("e")
        .build();
    let (code, _) = full_pipeline(grammar);
    assert!(code.contains("tree_sitter_prec_fn"));
}

// ===========================================================================
// 6. Multiple builds
// ===========================================================================

#[test]
fn generate_twice_same_builder() {
    let (g, t) = minimal();
    let builder = AbiLanguageBuilder::new(&g, &t);
    let code1 = builder.generate().to_string();
    let code2 = builder.generate().to_string();
    assert_eq!(code1, code2);
}

#[test]
fn generate_two_builders_same_input() {
    let (g, t) = minimal();
    let b1 = AbiLanguageBuilder::new(&g, &t);
    let b2 = AbiLanguageBuilder::new(&g, &t);
    assert_eq!(b1.generate().to_string(), b2.generate().to_string());
}

#[test]
fn generate_after_different_grammars() {
    let (g1, t1) = build_pair("first", 2, 1, 0, 0, 3);
    let (g2, t2) = build_pair("second", 3, 2, 1, 0, 4);
    let c1 = gen_code(&g1, &t1);
    let c2 = gen_code(&g2, &t2);
    assert!(!c1.is_empty());
    assert!(!c2.is_empty());
    assert_ne!(c1, c2);
}

// ===========================================================================
// 7. Deterministic output
// ===========================================================================

#[test]
fn deterministic_same_params() {
    let (g1, t1) = build_pair("det", 3, 2, 1, 0, 5);
    let (g2, t2) = build_pair("det", 3, 2, 1, 0, 5);
    assert_eq!(gen_code(&g1, &t1), gen_code(&g2, &t2));
}

#[test]
fn deterministic_full_pipeline() {
    let make = || {
        GrammarBuilder::new("det_fp")
            .token("a", "a")
            .token("b", "b")
            .rule("start", vec!["a", "b"])
            .start("start")
            .build()
    };
    let (c1, _) = full_pipeline(make());
    let (c2, _) = full_pipeline(make());
    assert_eq!(c1, c2);
}

#[test]
fn deterministic_multiple_runs() {
    let (g, t) = build_pair("det_mr", 4, 2, 2, 1, 6);
    let results: Vec<String> = (0..5).map(|_| gen_code(&g, &t)).collect();
    for r in &results[1..] {
        assert_eq!(&results[0], r);
    }
}

#[test]
fn different_name_different_output() {
    let (g1, t1) = build_pair("aaa", 2, 1, 0, 0, 3);
    let (g2, t2) = build_pair("bbb", 2, 1, 0, 0, 3);
    assert_ne!(gen_code(&g1, &t1), gen_code(&g2, &t2));
}

// ===========================================================================
// 8. ABI output contains expected keywords
// ===========================================================================

#[test]
fn output_contains_language() {
    let (g, t) = minimal();
    assert!(gen_code(&g, &t).contains("LANGUAGE"));
}

#[test]
fn output_contains_ts_language() {
    let (g, t) = minimal();
    assert!(gen_code(&g, &t).contains("TSLanguage"));
}

#[test]
fn output_contains_tree_sitter_version() {
    let (g, t) = minimal();
    assert!(gen_code(&g, &t).contains("TREE_SITTER_LANGUAGE_VERSION"));
}

#[test]
fn output_contains_symbol_names() {
    let (g, t) = minimal();
    assert!(gen_code(&g, &t).contains("SYMBOL_NAME"));
}

#[test]
fn output_contains_parse_actions() {
    let (g, t) = minimal();
    assert!(gen_code(&g, &t).contains("PARSE_ACTIONS"));
}

#[test]
fn output_contains_lex_modes() {
    let (g, t) = minimal();
    assert!(gen_code(&g, &t).contains("LEX_MODES"));
}

#[test]
fn output_contains_symbol_metadata() {
    let (g, t) = minimal();
    assert!(gen_code(&g, &t).contains("SYMBOL_METADATA"));
}

#[test]
fn output_contains_ffi_function_name() {
    let (g, t) = build_pair("myfoo", 1, 1, 0, 0, 2);
    assert!(gen_code(&g, &t).contains("tree_sitter_myfoo"));
}

#[test]
fn output_contains_symbol_count() {
    let (g, t) = build_pair("sc", 3, 2, 0, 0, 4);
    let code = gen_code(&g, &t);
    let expected = format!("symbol_count : {}u32", t.symbol_count);
    assert!(code.contains(&expected));
}

#[test]
fn output_contains_state_count() {
    let (g, t) = build_pair("stc", 1, 1, 0, 0, 7);
    assert!(gen_code(&g, &t).contains("state_count : 7u32"));
}

#[test]
fn output_contains_field_count() {
    let (g, t) = build_pair("fc", 1, 1, 4, 0, 2);
    assert!(gen_code(&g, &t).contains("field_count : 4u32"));
}

#[test]
fn output_contains_token_count() {
    let (g, t) = build_pair("tc", 3, 1, 0, 0, 2);
    let code = gen_code(&g, &t);
    let expected = format!("token_count : {}u32", t.token_count);
    assert!(code.contains(&expected));
}

#[test]
fn output_contains_production_id_count() {
    let (g, t) = build_pair("pid", 2, 3, 0, 0, 4);
    assert!(gen_code(&g, &t).contains("production_id_count : 3u32"));
}

#[test]
fn output_contains_primary_state_ids() {
    let (g, t) = minimal();
    assert!(gen_code(&g, &t).contains("PRIMARY_STATE_IDS"));
}

#[test]
fn output_contains_eof_symbol_zero() {
    let (g, t) = minimal();
    assert!(gen_code(&g, &t).contains("eof_symbol : 0"));
}

// ===========================================================================
// 9. StaticLanguageGenerator vs AbiLanguageBuilder comparison
// ===========================================================================

#[test]
fn static_gen_returns_nonempty() {
    let (g, t) = minimal();
    let gen_out = StaticLanguageGenerator::new(g, t);
    let ts = gen_out.generate_language_code();
    assert!(!ts.is_empty());
}

#[test]
fn static_gen_to_string_nonempty() {
    let (g, t) = minimal();
    let gen_out = StaticLanguageGenerator::new(g, t);
    let code = gen_out.generate_language_code().to_string();
    assert!(!code.is_empty());
}

#[test]
fn both_generators_produce_output() {
    let (g, t) = build_pair("both", 2, 1, 0, 0, 3);
    let abi_code = gen_code(&g, &t);
    let static_code = StaticLanguageGenerator::new(g.clone(), t.clone())
        .generate_language_code()
        .to_string();
    assert!(!abi_code.is_empty());
    assert!(!static_code.is_empty());
}

#[test]
fn static_gen_deterministic() {
    let make = || {
        let (g, t) = build_pair("sdet", 2, 1, 0, 0, 3);
        StaticLanguageGenerator::new(g, t)
            .generate_language_code()
            .to_string()
    };
    assert_eq!(make(), make());
}

#[test]
fn static_gen_contains_language_keyword() {
    let (g, t) = build_pair("slk", 2, 1, 0, 0, 3);
    let code = StaticLanguageGenerator::new(g, t)
        .generate_language_code()
        .to_string();
    let has_lang =
        code.contains("Language") || code.contains("LANGUAGE") || code.contains("language");
    assert!(
        has_lang,
        "StaticLanguageGenerator output must reference language"
    );
}

#[test]
fn static_gen_with_fields() {
    let (g, t) = build_pair("sf", 1, 1, 3, 0, 2);
    let code = StaticLanguageGenerator::new(g, t)
        .generate_language_code()
        .to_string();
    assert!(!code.is_empty());
}

// ===========================================================================
// 10. Complex grammars
// ===========================================================================

#[test]
fn complex_chain_rules() {
    let grammar = GrammarBuilder::new("chain")
        .token("x", "x")
        .rule("C", vec!["x"])
        .rule("B", vec!["C"])
        .rule("A", vec!["B"])
        .rule("start", vec!["A"])
        .start("start")
        .build();
    let (code, _) = full_pipeline(grammar);
    assert!(code.contains("LANGUAGE"));
}

#[test]
fn complex_self_recursive() {
    let grammar = GrammarBuilder::new("rec")
        .token("a", "a")
        .rule("list", vec!["a"])
        .rule("list", vec!["list", "a"])
        .start("list")
        .build();
    let (code, _) = full_pipeline(grammar);
    assert!(code.contains("LANGUAGE"));
}

#[test]
fn complex_nested_rules() {
    let grammar = GrammarBuilder::new("nested")
        .token("x", "x")
        .token("y", "y")
        .rule("inner", vec!["x"])
        .rule("outer", vec!["inner", "y"])
        .rule("start", vec!["outer"])
        .start("start")
        .build();
    let (code, _) = full_pipeline(grammar);
    assert!(code.contains("tree_sitter_nested"));
}

#[test]
fn complex_alternatives() {
    let grammar = GrammarBuilder::new("alts")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("start", vec!["a"])
        .rule("start", vec!["b"])
        .rule("start", vec!["c"])
        .start("start")
        .build();
    let (code, _) = full_pipeline(grammar);
    assert!(code.contains("LANGUAGE"));
}

#[test]
fn complex_parenthesized_expression() {
    let grammar = GrammarBuilder::new("paren")
        .token("a", "a")
        .token("lp", "(")
        .token("rp", ")")
        .rule("atom", vec!["a"])
        .rule("atom", vec!["lp", "expr", "rp"])
        .rule("expr", vec!["atom"])
        .start("expr")
        .build();
    let (code, _) = full_pipeline(grammar);
    assert!(code.contains("LANGUAGE"));
}

#[test]
fn complex_multiple_nonterminals() {
    let grammar = GrammarBuilder::new("multi_nt")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("X", vec!["a"])
        .rule("Y", vec!["b"])
        .rule("Z", vec!["c"])
        .rule("start", vec!["X", "Y", "Z"])
        .start("start")
        .build();
    let (code, _) = full_pipeline(grammar);
    assert!(code.contains("LANGUAGE"));
}

#[test]
fn complex_epsilon_rule() {
    let grammar = GrammarBuilder::new("eps")
        .token("a", "a")
        .rule("start", vec![])
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let (code, _) = full_pipeline(grammar);
    assert!(code.contains("LANGUAGE"));
}

// ===========================================================================
// 11. Roundtrip consistency
// ===========================================================================

#[test]
fn roundtrip_state_count_consistent() {
    let grammar = GrammarBuilder::new("rt_sc")
        .token("t", "t")
        .rule("start", vec!["t"])
        .start("start")
        .build();
    let (code, table) = full_pipeline(grammar);
    let expected = format!("state_count : {}u32", table.state_count);
    assert!(code.contains(&expected));
}

#[test]
fn roundtrip_symbol_count_consistent() {
    let grammar = GrammarBuilder::new("rt_sym")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a", "b"])
        .start("start")
        .build();
    let (code, table) = full_pipeline(grammar);
    let expected = format!("symbol_count : {}u32", table.symbol_count);
    assert!(code.contains(&expected));
}

#[test]
fn roundtrip_token_count_consistent() {
    let grammar = GrammarBuilder::new("rt_tc")
        .token("x", "x")
        .rule("start", vec!["x"])
        .start("start")
        .build();
    let (code, table) = full_pipeline(grammar);
    let expected = format!("token_count : {}u32", table.token_count);
    assert!(code.contains(&expected));
}

// ===========================================================================
// 12. Edge cases and additional coverage
// ===========================================================================

#[test]
fn single_state_grammar() {
    let (g, t) = build_pair("one_st", 1, 1, 0, 0, 1);
    let code = gen_code(&g, &t);
    assert!(code.contains("state_count : 1u32"));
}

#[test]
fn zero_fields_produces_zero_count() {
    let (g, t) = build_pair("zf", 1, 1, 0, 0, 2);
    let code = gen_code(&g, &t);
    assert!(code.contains("field_count : 0u32"));
}

#[test]
fn zero_externals_produces_zero_count() {
    let (g, t) = build_pair("ze", 1, 1, 0, 0, 2);
    let code = gen_code(&g, &t);
    assert!(code.contains("external_token_count : 0u32"));
}

#[test]
fn external_token_count_matches() {
    let (g, t) = build_pair("etc", 1, 1, 0, 4, 2);
    let code = gen_code(&g, &t);
    assert!(code.contains("external_token_count : 4u32"));
}

#[test]
fn alias_count_zero_simple() {
    let (g, t) = minimal();
    let code = gen_code(&g, &t);
    assert!(code.contains("alias_count : 0u32"));
}

#[test]
fn keyword_capture_token_default_zero() {
    let (g, t) = minimal();
    let code = gen_code(&g, &t);
    assert!(code.contains("keyword_capture_token : 0"));
}

#[test]
fn public_symbol_map_emitted() {
    let (g, t) = minimal();
    assert!(gen_code(&g, &t).contains("PUBLIC_SYMBOL_MAP"));
}

#[test]
fn production_id_map_emitted() {
    let (g, t) = minimal();
    assert!(gen_code(&g, &t).contains("PRODUCTION_ID_MAP"));
}

#[test]
fn lexer_fn_generated() {
    let (g, t) = minimal();
    assert!(gen_code(&g, &t).contains("lexer_fn"));
}
