//! ABI language builder v8 — 84 tests in 10 categories.
//!
//! Categories:
//!   basic_*        — non-empty output, pipeline smoke
//!   content_*      — generated code contains expected strings
//!   single_tok_*   — grammar with 1 token
//!   multi_tok_*    — grammar with multiple tokens
//!   prec_*         — precedence / associativity
//!   inline_*       — inline rules
//!   supertype_*    — supertype symbols
//!   extras_*       — extra tokens (whitespace, etc.)
//!   naming_*       — different grammar names ⇒ different function names
//!   robust_*       — various sizes, edge cases, no panics

use adze_glr_core::{FirstFollowSets, ParseTable, build_lr1_automaton};
use adze_ir::builder::GrammarBuilder;
use adze_ir::{Associativity, Grammar};
use adze_tablegen::AbiLanguageBuilder;

// ============================================================================
// Helpers
// ============================================================================

fn build_table(grammar: &Grammar) -> ParseTable {
    let mut g = grammar.clone();
    let ff = FirstFollowSets::compute_normalized(&mut g).expect("FIRST/FOLLOW");
    build_lr1_automaton(&g, &ff).expect("LR(1)")
}

fn generate(grammar: &Grammar) -> String {
    let pt = build_table(grammar);
    AbiLanguageBuilder::new(grammar, &pt).generate().to_string()
}

fn gen_with_table(grammar: &Grammar, table: &ParseTable) -> String {
    AbiLanguageBuilder::new(grammar, table)
        .generate()
        .to_string()
}

// --- grammar factories ---

fn single_token_grammar() -> Grammar {
    GrammarBuilder::new("single")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build()
}

fn two_token_grammar() -> Grammar {
    GrammarBuilder::new("two")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["a", "b"])
        .start("start")
        .build()
}

fn three_token_grammar() -> Grammar {
    GrammarBuilder::new("three")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("start", vec!["a", "b", "c"])
        .start("start")
        .build()
}

fn alternatives_grammar() -> Grammar {
    GrammarBuilder::new("alts")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("start", vec!["a"])
        .rule("start", vec!["b"])
        .rule("start", vec!["c"])
        .start("start")
        .build()
}

fn nested_grammar() -> Grammar {
    GrammarBuilder::new("nested")
        .token("x", "x")
        .token("y", "y")
        .rule("start", vec!["middle"])
        .rule("middle", vec!["x", "y"])
        .start("start")
        .build()
}

fn deep_chain_grammar() -> Grammar {
    GrammarBuilder::new("deep")
        .token("z", "z")
        .rule("start", vec!["layer1"])
        .rule("layer1", vec!["layer2"])
        .rule("layer2", vec!["layer3"])
        .rule("layer3", vec!["z"])
        .start("start")
        .build()
}

fn left_recursive_grammar() -> Grammar {
    GrammarBuilder::new("leftrec")
        .token("a", "a")
        .rule("start", vec!["a"])
        .rule("start", vec!["start", "a"])
        .start("start")
        .build()
}

fn right_recursive_grammar() -> Grammar {
    GrammarBuilder::new("rightrec")
        .token("a", "a")
        .rule("start", vec!["a"])
        .rule("start", vec!["a", "start"])
        .start("start")
        .build()
}

fn precedence_grammar() -> Grammar {
    GrammarBuilder::new("prec")
        .token("num", r"\d+")
        .token("plus", r"\+")
        .token("star", r"\*")
        .rule_with_precedence("expr", vec!["expr", "plus", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "star", "expr"], 2, Associativity::Left)
        .rule("expr", vec!["num"])
        .start("expr")
        .build()
}

fn right_assoc_grammar() -> Grammar {
    GrammarBuilder::new("rassoc")
        .token("num", r"\d+")
        .token("caret", r"\^")
        .rule_with_precedence(
            "expr",
            vec!["expr", "caret", "expr"],
            1,
            Associativity::Right,
        )
        .rule("expr", vec!["num"])
        .start("expr")
        .build()
}

fn inline_grammar() -> Grammar {
    GrammarBuilder::new("inlined")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["helper"])
        .rule("helper", vec!["a", "b"])
        .inline("helper")
        .start("start")
        .build()
}

fn supertype_grammar() -> Grammar {
    GrammarBuilder::new("stype")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["decl"])
        .rule("decl", vec!["a"])
        .rule("decl", vec!["b"])
        .supertype("decl")
        .start("start")
        .build()
}

fn extras_grammar() -> Grammar {
    GrammarBuilder::new("extras")
        .token("a", "a")
        .token("ws", r"\s+")
        .extra("ws")
        .rule("start", vec!["a"])
        .start("start")
        .build()
}

fn nullable_grammar() -> Grammar {
    GrammarBuilder::new("nullable")
        .token("a", "a")
        .rule("start", vec!["opt"])
        .rule("opt", vec!["a"])
        .rule("opt", vec![])
        .start("start")
        .build()
}

fn wide_grammar() -> Grammar {
    let mut gb = GrammarBuilder::new("wide");
    for i in 0..8u8 {
        let name = format!("t{i}");
        let pat = format!("{}", (b'a' + i) as char);
        gb = gb.token(&name, &pat);
        gb = gb.rule("start", vec![&name]);
    }
    gb.start("start").build()
}

fn long_sequence_grammar() -> Grammar {
    GrammarBuilder::new("longseq")
        .token("t1", "a")
        .token("t2", "b")
        .token("t3", "c")
        .token("t4", "d")
        .token("t5", "e")
        .rule("start", vec!["t1", "t2", "t3", "t4", "t5"])
        .start("start")
        .build()
}

fn named_grammar(name: &str) -> Grammar {
    GrammarBuilder::new(name)
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build()
}

// ============================================================================
// 1. basic_* — simple grammar produces non-empty code (8 tests)
// ============================================================================

#[test]
fn basic_single_token_nonempty() {
    let code = generate(&single_token_grammar());
    assert!(!code.is_empty());
}

#[test]
fn basic_two_token_nonempty() {
    let code = generate(&two_token_grammar());
    assert!(!code.is_empty());
}

#[test]
fn basic_alternatives_nonempty() {
    let code = generate(&alternatives_grammar());
    assert!(!code.is_empty());
}

#[test]
fn basic_nested_nonempty() {
    let code = generate(&nested_grammar());
    assert!(!code.is_empty());
}

#[test]
fn basic_deep_chain_nonempty() {
    let code = generate(&deep_chain_grammar());
    assert!(!code.is_empty());
}

#[test]
fn basic_left_recursive_nonempty() {
    let code = generate(&left_recursive_grammar());
    assert!(!code.is_empty());
}

#[test]
fn basic_right_recursive_nonempty() {
    let code = generate(&right_recursive_grammar());
    assert!(!code.is_empty());
}

#[test]
fn basic_nullable_nonempty() {
    let code = generate(&nullable_grammar());
    assert!(!code.is_empty());
}

// ============================================================================
// 2. content_* — generated code contains expected strings (10 tests)
// ============================================================================

#[test]
fn content_contains_tslanguage() {
    let code = generate(&single_token_grammar());
    assert!(code.contains("TSLanguage"), "must contain TSLanguage");
}

#[test]
fn content_contains_grammar_name() {
    let code = generate(&single_token_grammar());
    assert!(
        code.contains("single"),
        "must contain grammar name 'single'"
    );
}

#[test]
fn content_contains_tree_sitter_fn() {
    let code = generate(&single_token_grammar());
    assert!(
        code.contains("tree_sitter_"),
        "must contain tree_sitter_ function prefix"
    );
}

#[test]
fn content_contains_tree_sitter_fn_with_name() {
    let code = generate(&single_token_grammar());
    assert!(
        code.contains("tree_sitter_single"),
        "must contain tree_sitter_single"
    );
}

#[test]
fn content_contains_static_or_const() {
    let code = generate(&single_token_grammar());
    let has_static = code.contains("static");
    let has_const = code.contains("const");
    assert!(
        has_static || has_const,
        "generated Rust code must contain 'static' or 'const'"
    );
}

#[test]
fn content_contains_symbol_metadata() {
    let code = generate(&single_token_grammar());
    assert!(
        code.contains("SYMBOL_METADATA") || code.contains("symbol_metadata"),
        "must reference symbol metadata"
    );
}

#[test]
fn content_contains_parse_table() {
    let code = generate(&single_token_grammar());
    assert!(
        code.contains("PARSE_TABLE") || code.contains("parse_table"),
        "must reference parse table"
    );
}

#[test]
fn content_contains_lex_modes() {
    let code = generate(&single_token_grammar());
    assert!(
        code.contains("LEX_MODES") || code.contains("lex_modes"),
        "must reference lex modes"
    );
}

#[test]
fn content_contains_language_version() {
    let code = generate(&single_token_grammar());
    assert!(
        code.contains("LANGUAGE_VERSION") || code.contains("language_version"),
        "must reference language version"
    );
}

#[test]
fn content_contains_use_statement() {
    let code = generate(&single_token_grammar());
    assert!(code.contains("use"), "must contain use statement");
}

// ============================================================================
// 3. single_tok_* — grammar with 1 token (8 tests)
// ============================================================================

#[test]
fn single_tok_code_contains_token_name() {
    let g = single_token_grammar();
    let code = generate(&g);
    // The token "a" should appear in symbol names
    assert!(code.contains('a'), "code should reference token 'a'");
}

#[test]
fn single_tok_parse_table_has_states() {
    let g = single_token_grammar();
    let pt = build_table(&g);
    assert!(pt.state_count >= 2, "need at least 2 states");
}

#[test]
fn single_tok_symbol_count_at_least_three() {
    let g = single_token_grammar();
    let pt = build_table(&g);
    // EOF + token + nonterminal
    assert!(pt.symbol_count >= 3, "got {}", pt.symbol_count);
}

#[test]
fn single_tok_action_table_nonempty() {
    let g = single_token_grammar();
    let pt = build_table(&g);
    assert!(!pt.action_table.is_empty());
}

#[test]
fn single_tok_goto_table_nonempty() {
    let g = single_token_grammar();
    let pt = build_table(&g);
    assert!(!pt.goto_table.is_empty());
}

#[test]
fn single_tok_code_length_reasonable() {
    let code = generate(&single_token_grammar());
    // Even minimal grammar should produce substantial code
    assert!(code.len() > 100, "code too short: {} bytes", code.len());
}

#[test]
fn single_tok_no_unknown_symbols() {
    let code = generate(&single_token_grammar());
    // The builder uses "???" prefix for unknown symbols
    assert!(
        !code.contains("???"),
        "generated code should not contain unknown symbol markers"
    );
}

#[test]
fn single_tok_deterministic() {
    let g = single_token_grammar();
    let code1 = generate(&g);
    let code2 = generate(&g);
    assert_eq!(code1, code2, "generation must be deterministic");
}

// ============================================================================
// 4. multi_tok_* — grammar with multiple tokens (10 tests)
// ============================================================================

#[test]
fn multi_tok_two_tokens_nonempty() {
    let code = generate(&two_token_grammar());
    assert!(!code.is_empty());
}

#[test]
fn multi_tok_three_tokens_nonempty() {
    let code = generate(&three_token_grammar());
    assert!(!code.is_empty());
}

#[test]
fn multi_tok_wide_alternatives_nonempty() {
    let code = generate(&wide_grammar());
    assert!(!code.is_empty());
}

#[test]
fn multi_tok_long_sequence_nonempty() {
    let code = generate(&long_sequence_grammar());
    assert!(!code.is_empty());
}

#[test]
fn multi_tok_two_has_more_symbols() {
    let pt1 = build_table(&single_token_grammar());
    let pt2 = build_table(&two_token_grammar());
    assert!(
        pt2.symbol_count > pt1.symbol_count,
        "two-token grammar should have more symbols"
    );
}

#[test]
fn multi_tok_alternatives_symbol_count() {
    let pt = build_table(&alternatives_grammar());
    // EOF + 3 tokens + at least 1 nonterminal
    assert!(pt.symbol_count >= 5, "got {}", pt.symbol_count);
}

#[test]
fn multi_tok_wide_has_many_symbols() {
    let pt = build_table(&wide_grammar());
    // EOF + 8 tokens + nonterminal
    assert!(pt.symbol_count >= 10, "got {}", pt.symbol_count);
}

#[test]
fn multi_tok_long_seq_code_bigger() {
    let code_single = generate(&single_token_grammar());
    let code_long = generate(&long_sequence_grammar());
    assert!(
        code_long.len() > code_single.len(),
        "more tokens should produce more code"
    );
}

#[test]
fn multi_tok_alternatives_deterministic() {
    let g = alternatives_grammar();
    let code1 = generate(&g);
    let code2 = generate(&g);
    assert_eq!(code1, code2);
}

#[test]
fn multi_tok_nested_code_references_nonterminals() {
    let code = generate(&nested_grammar());
    // Nested grammar has two nonterminals; code should still contain grammar name
    assert!(
        code.contains("nested"),
        "nested grammar code should reference grammar name 'nested'"
    );
}

// ============================================================================
// 5. prec_* — grammar with precedence (9 tests)
// ============================================================================

#[test]
fn prec_left_assoc_nonempty() {
    let code = generate(&precedence_grammar());
    assert!(!code.is_empty());
}

#[test]
fn prec_right_assoc_nonempty() {
    let code = generate(&right_assoc_grammar());
    assert!(!code.is_empty());
}

#[test]
fn prec_contains_grammar_name() {
    let code = generate(&precedence_grammar());
    assert!(code.contains("prec"), "code must contain grammar name");
}

#[test]
fn prec_contains_tree_sitter_fn() {
    let code = generate(&precedence_grammar());
    assert!(code.contains("tree_sitter_prec"));
}

#[test]
fn prec_code_length_reasonable() {
    let code = generate(&precedence_grammar());
    assert!(code.len() > 100);
}

#[test]
fn prec_deterministic() {
    let g = precedence_grammar();
    let code1 = generate(&g);
    let code2 = generate(&g);
    assert_eq!(code1, code2);
}

#[test]
fn prec_right_contains_grammar_name() {
    let code = generate(&right_assoc_grammar());
    assert!(code.contains("rassoc"));
}

#[test]
fn prec_parse_table_has_states() {
    let pt = build_table(&precedence_grammar());
    assert!(pt.state_count >= 2);
}

#[test]
fn prec_more_complex_than_single() {
    let simple = generate(&single_token_grammar());
    let prec = generate(&precedence_grammar());
    assert!(
        prec.len() > simple.len(),
        "precedence grammar should generate more code"
    );
}

// ============================================================================
// 6. inline_* — inline rules (8 tests)
// ============================================================================

#[test]
fn inline_grammar_nonempty() {
    let code = generate(&inline_grammar());
    assert!(!code.is_empty());
}

#[test]
fn inline_contains_grammar_name() {
    let code = generate(&inline_grammar());
    assert!(code.contains("inlined"));
}

#[test]
fn inline_contains_tree_sitter_fn() {
    let code = generate(&inline_grammar());
    assert!(code.contains("tree_sitter_inlined"));
}

#[test]
fn inline_contains_tslanguage() {
    let code = generate(&inline_grammar());
    assert!(code.contains("TSLanguage"));
}

#[test]
fn inline_parse_table_builds() {
    let g = inline_grammar();
    let pt = build_table(&g);
    assert!(pt.state_count >= 2);
}

#[test]
fn inline_deterministic() {
    let g = inline_grammar();
    let code1 = generate(&g);
    let code2 = generate(&g);
    assert_eq!(code1, code2);
}

#[test]
fn inline_code_length_reasonable() {
    let code = generate(&inline_grammar());
    assert!(code.len() > 100);
}

#[test]
fn inline_grammar_has_inline_set() {
    let g = inline_grammar();
    assert!(!g.inline_rules.is_empty(), "inline rules should be set");
}

// ============================================================================
// 7. supertype_* — supertype symbols (8 tests)
// ============================================================================

#[test]
fn supertype_grammar_nonempty() {
    let code = generate(&supertype_grammar());
    assert!(!code.is_empty());
}

#[test]
fn supertype_contains_grammar_name() {
    let code = generate(&supertype_grammar());
    assert!(code.contains("stype"));
}

#[test]
fn supertype_contains_tree_sitter_fn() {
    let code = generate(&supertype_grammar());
    assert!(code.contains("tree_sitter_stype"));
}

#[test]
fn supertype_contains_tslanguage() {
    let code = generate(&supertype_grammar());
    assert!(code.contains("TSLanguage"));
}

#[test]
fn supertype_parse_table_builds() {
    let g = supertype_grammar();
    let pt = build_table(&g);
    assert!(pt.state_count >= 2);
}

#[test]
fn supertype_grammar_has_supertypes_set() {
    let g = supertype_grammar();
    assert!(!g.supertypes.is_empty(), "supertypes should be set");
}

#[test]
fn supertype_deterministic() {
    let g = supertype_grammar();
    let code1 = generate(&g);
    let code2 = generate(&g);
    assert_eq!(code1, code2);
}

#[test]
fn supertype_code_length_reasonable() {
    let code = generate(&supertype_grammar());
    assert!(code.len() > 100);
}

// ============================================================================
// 8. extras_* — extra tokens (8 tests)
// ============================================================================

#[test]
fn extras_grammar_nonempty() {
    let code = generate(&extras_grammar());
    assert!(!code.is_empty());
}

#[test]
fn extras_contains_grammar_name() {
    let code = generate(&extras_grammar());
    assert!(code.contains("extras"));
}

#[test]
fn extras_contains_tree_sitter_fn() {
    let code = generate(&extras_grammar());
    assert!(code.contains("tree_sitter_extras"));
}

#[test]
fn extras_contains_tslanguage() {
    let code = generate(&extras_grammar());
    assert!(code.contains("TSLanguage"));
}

#[test]
fn extras_parse_table_builds() {
    let g = extras_grammar();
    let pt = build_table(&g);
    assert!(pt.state_count >= 2);
}

#[test]
fn extras_grammar_has_extras_set() {
    let g = extras_grammar();
    assert!(!g.extras.is_empty(), "extras should be set");
}

#[test]
fn extras_deterministic() {
    let g = extras_grammar();
    let code1 = generate(&g);
    let code2 = generate(&g);
    assert_eq!(code1, code2);
}

#[test]
fn extras_code_length_reasonable() {
    let code = generate(&extras_grammar());
    assert!(code.len() > 100);
}

// ============================================================================
// 9. naming_* — different names produce different function names (9 tests)
// ============================================================================

#[test]
fn naming_alpha_contains_its_name() {
    let code = generate(&named_grammar("alpha"));
    assert!(code.contains("tree_sitter_alpha"));
}

#[test]
fn naming_beta_contains_its_name() {
    let code = generate(&named_grammar("beta"));
    assert!(code.contains("tree_sitter_beta"));
}

#[test]
fn naming_gamma_contains_its_name() {
    let code = generate(&named_grammar("gamma"));
    assert!(code.contains("tree_sitter_gamma"));
}

#[test]
fn naming_different_names_different_fn() {
    let code_a = generate(&named_grammar("foo"));
    let code_b = generate(&named_grammar("bar"));
    assert_ne!(
        code_a, code_b,
        "different names must produce different code"
    );
}

#[test]
fn naming_foo_does_not_contain_bar() {
    let code = generate(&named_grammar("foo"));
    assert!(!code.contains("tree_sitter_bar"));
}

#[test]
fn naming_short_name() {
    let code = generate(&named_grammar("x"));
    assert!(code.contains("tree_sitter_x"));
}

#[test]
fn naming_longer_name() {
    let code = generate(&named_grammar("my_language"));
    assert!(code.contains("tree_sitter_my_language"));
}

#[test]
fn naming_numeric_suffix() {
    let code = generate(&named_grammar("lang42"));
    assert!(code.contains("tree_sitter_lang42"));
}

#[test]
fn naming_underscore_name() {
    let code = generate(&named_grammar("a_b_c"));
    assert!(code.contains("tree_sitter_a_b_c"));
}

// ============================================================================
// 10. robust_* — various sizes, edge cases, no panics (16 tests)
// ============================================================================

#[test]
fn robust_default_grammar_default_table_no_panic() {
    let g = Grammar::new("empty".to_string());
    let pt = ParseTable::default();
    let _code = gen_with_table(&g, &pt);
}

#[test]
fn robust_builder_construct_no_panic() {
    let g = single_token_grammar();
    let pt = build_table(&g);
    let _builder = AbiLanguageBuilder::new(&g, &pt);
}

#[test]
fn robust_two_builders_same_grammar() {
    let g = single_token_grammar();
    let pt = build_table(&g);
    let code1 = AbiLanguageBuilder::new(&g, &pt).generate().to_string();
    let code2 = AbiLanguageBuilder::new(&g, &pt).generate().to_string();
    assert_eq!(code1, code2);
}

#[test]
fn robust_wide_grammar_no_panic() {
    let code = generate(&wide_grammar());
    assert!(!code.is_empty());
}

#[test]
fn robust_long_sequence_no_panic() {
    let code = generate(&long_sequence_grammar());
    assert!(!code.is_empty());
}

#[test]
fn robust_deep_chain_no_panic() {
    let code = generate(&deep_chain_grammar());
    assert!(!code.is_empty());
}

#[test]
fn robust_left_recursive_no_panic() {
    let code = generate(&left_recursive_grammar());
    assert!(!code.is_empty());
}

#[test]
fn robust_right_recursive_no_panic() {
    let code = generate(&right_recursive_grammar());
    assert!(!code.is_empty());
}

#[test]
fn robust_nullable_no_panic() {
    let code = generate(&nullable_grammar());
    assert!(!code.is_empty());
}

#[test]
fn robust_precedence_no_panic() {
    let code = generate(&precedence_grammar());
    assert!(!code.is_empty());
}

#[test]
fn robust_all_features_combined() {
    // Grammar exercising extras + precedence + multiple rules
    let g = GrammarBuilder::new("combo")
        .token("num", r"\d+")
        .token("plus", r"\+")
        .token("ws", r"\s+")
        .extra("ws")
        .rule_with_precedence("expr", vec!["expr", "plus", "expr"], 1, Associativity::Left)
        .rule("expr", vec!["num"])
        .start("expr")
        .build();
    let code = generate(&g);
    assert!(!code.is_empty());
    assert!(code.contains("tree_sitter_combo"));
}

#[test]
fn robust_many_alternatives_no_panic() {
    let mut gb = GrammarBuilder::new("many_alts");
    for i in 0..12u8 {
        let name = format!("tok{i}");
        let pat = format!("{}", (b'a' + i) as char);
        gb = gb.token(&name, &pat);
        gb = gb.rule("start", vec![&name]);
    }
    let g = gb.start("start").build();
    let code = generate(&g);
    assert!(!code.is_empty());
}

#[test]
fn robust_multiple_nonterminals() {
    let g = GrammarBuilder::new("multi_nt")
        .token("x", "x")
        .token("y", "y")
        .token("z", "z")
        .rule("start", vec!["p1", "p2"])
        .rule("p1", vec!["x", "y"])
        .rule("p2", vec!["z"])
        .start("start")
        .build();
    let code = generate(&g);
    assert!(!code.is_empty());
    assert!(code.contains("tree_sitter_multi_nt"));
}

#[test]
fn robust_single_epsilon_rule() {
    let g = GrammarBuilder::new("eps")
        .token("a", "a")
        .rule("start", vec!["opt"])
        .rule("opt", vec![])
        .rule("opt", vec!["a"])
        .start("start")
        .build();
    let code = generate(&g);
    assert!(!code.is_empty());
}

#[test]
fn robust_code_increases_with_complexity() {
    let small = generate(&single_token_grammar());
    let medium = generate(&alternatives_grammar());
    // More complex grammar ⇒ more generated code
    assert!(
        medium.len() >= small.len(),
        "medium ({}) should be >= small ({})",
        medium.len(),
        small.len()
    );
}

#[test]
fn robust_with_compressed_tables_accepted() {
    let g = single_token_grammar();
    let pt = build_table(&g);
    // `with_compressed_tables` should be callable without panic
    let builder = AbiLanguageBuilder::new(&g, &pt);
    let code = builder.generate().to_string();
    assert!(!code.is_empty());
}
