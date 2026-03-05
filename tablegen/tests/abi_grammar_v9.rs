//! Comprehensive tests for `AbiLanguageBuilder` with diverse grammar patterns.
//!
//! 80+ tests across 20 categories — each grammar uses the "ag_v9_" prefix.
//!
//! Categories:
//!   single_token_*   — single token, single rule
//!   two_token_*      — two tokens
//!   three_token_*    — three tokens
//!   keyword_*        — keyword grammar
//!   number_*         — number grammar
//!   binexpr_*        — binary expression
//!   unary_*          — unary expression
//!   ifthen_*         — if-then grammar
//!   assign_*         — assignment grammar
//!   funcall_*        — function call grammar
//!   list_*           — list grammar
//!   block_*          — block grammar
//!   ws_extra_*       — whitespace extra
//!   comment_*        — comment token
//!   prec_*           — precedence
//!   left_assoc_*     — left associativity
//!   right_assoc_*    — right associativity
//!   inline_*         — inline rules
//!   extern_*         — external scanner
//!   determinism_*    — ABI output deterministic

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

// --- grammar factories ---

fn single_token_grammar() -> Grammar {
    GrammarBuilder::new("ag_v9_single")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build()
}

fn two_token_grammar() -> Grammar {
    GrammarBuilder::new("ag_v9_two")
        .token("x", "x")
        .token("y", "y")
        .rule("start", vec!["x", "y"])
        .start("start")
        .build()
}

fn three_token_grammar() -> Grammar {
    GrammarBuilder::new("ag_v9_three")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("start", vec!["a", "b", "c"])
        .start("start")
        .build()
}

fn keyword_grammar() -> Grammar {
    GrammarBuilder::new("ag_v9_keyword")
        .token("kw_let", "let")
        .token("kw_in", "in")
        .token("ident", "[a-z]+")
        .rule("start", vec!["kw_let", "ident", "kw_in", "ident"])
        .start("start")
        .build()
}

fn number_grammar() -> Grammar {
    GrammarBuilder::new("ag_v9_number")
        .token("num", "[0-9]+")
        .rule("start", vec!["num"])
        .start("start")
        .build()
}

fn binexpr_grammar() -> Grammar {
    GrammarBuilder::new("ag_v9_binexpr")
        .token("num", r"\d+")
        .token("plus", r"\+")
        .token("star", r"\*")
        .rule_with_precedence("expr", vec!["expr", "plus", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "star", "expr"], 2, Associativity::Left)
        .rule("expr", vec!["num"])
        .start("expr")
        .build()
}

fn unary_grammar() -> Grammar {
    GrammarBuilder::new("ag_v9_unary")
        .token("num", r"\d+")
        .token("minus", r"\-")
        .rule("expr", vec!["minus", "expr"])
        .rule("expr", vec!["num"])
        .start("expr")
        .build()
}

fn ifthen_grammar() -> Grammar {
    GrammarBuilder::new("ag_v9_ifthen")
        .token("kw_if", "if")
        .token("kw_then", "then")
        .token("cond", "true")
        .token("body", "ok")
        .rule("start", vec!["kw_if", "cond", "kw_then", "body"])
        .start("start")
        .build()
}

fn assign_grammar() -> Grammar {
    GrammarBuilder::new("ag_v9_assign")
        .token("ident", "[a-z]+")
        .token("eq", "=")
        .token("num", "[0-9]+")
        .rule("start", vec!["ident", "eq", "num"])
        .start("start")
        .build()
}

fn funcall_grammar() -> Grammar {
    GrammarBuilder::new("ag_v9_funcall")
        .token("ident", "[a-z]+")
        .token("lparen", r"\(")
        .token("rparen", r"\)")
        .token("arg", "[0-9]+")
        .rule("start", vec!["ident", "lparen", "arg", "rparen"])
        .start("start")
        .build()
}

fn list_grammar() -> Grammar {
    GrammarBuilder::new("ag_v9_list")
        .token("lbracket", r"\[")
        .token("rbracket", r"\]")
        .token("item", "[a-z]+")
        .token("comma", ",")
        .rule("start", vec!["lbracket", "items", "rbracket"])
        .rule("items", vec!["item"])
        .rule("items", vec!["items", "comma", "item"])
        .start("start")
        .build()
}

fn block_grammar() -> Grammar {
    GrammarBuilder::new("ag_v9_block")
        .token("lbrace", r"\{")
        .token("rbrace", r"\}")
        .token("stmt", "[a-z]+")
        .token("semi", ";")
        .rule("start", vec!["lbrace", "body", "rbrace"])
        .rule("body", vec!["stmt", "semi"])
        .rule("body", vec!["body", "stmt", "semi"])
        .start("start")
        .build()
}

fn ws_extra_grammar() -> Grammar {
    GrammarBuilder::new("ag_v9_wsextra")
        .token("word", "[a-z]+")
        .token("ws", r"\s+")
        .extra("ws")
        .rule("start", vec!["word"])
        .start("start")
        .build()
}

fn comment_grammar() -> Grammar {
    GrammarBuilder::new("ag_v9_comment")
        .token("word", "[a-z]+")
        .token("comment", "//[^\n]*")
        .extra("comment")
        .rule("start", vec!["word"])
        .start("start")
        .build()
}

fn prec_grammar() -> Grammar {
    GrammarBuilder::new("ag_v9_prec")
        .token("num", r"\d+")
        .token("plus", r"\+")
        .token("star", r"\*")
        .token("minus", r"\-")
        .rule_with_precedence("expr", vec!["expr", "plus", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "star", "expr"], 2, Associativity::Left)
        .rule_with_precedence(
            "expr",
            vec!["expr", "minus", "expr"],
            1,
            Associativity::Left,
        )
        .rule("expr", vec!["num"])
        .start("expr")
        .build()
}

fn left_assoc_grammar() -> Grammar {
    GrammarBuilder::new("ag_v9_lassoc")
        .token("num", r"\d+")
        .token("minus", r"\-")
        .rule_with_precedence(
            "expr",
            vec!["expr", "minus", "expr"],
            1,
            Associativity::Left,
        )
        .rule("expr", vec!["num"])
        .start("expr")
        .build()
}

fn right_assoc_grammar() -> Grammar {
    GrammarBuilder::new("ag_v9_rassoc")
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
    GrammarBuilder::new("ag_v9_inline")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["helper"])
        .rule("helper", vec!["a", "b"])
        .inline("helper")
        .start("start")
        .build()
}

fn extern_grammar() -> Grammar {
    GrammarBuilder::new("ag_v9_extern")
        .token("word", "[a-z]+")
        .external("indent")
        .rule("start", vec!["word"])
        .start("start")
        .build()
}

// ============================================================================
// 1. single_token_* — single token, single rule (4 tests)
// ============================================================================

#[test]
fn single_token_nonempty() {
    let code = generate(&single_token_grammar());
    assert!(!code.is_empty());
}

#[test]
fn single_token_contains_grammar_name() {
    let code = generate(&single_token_grammar());
    assert!(code.contains("ag_v9_single"), "must contain grammar name");
}

#[test]
fn single_token_contains_tree_sitter_fn() {
    let code = generate(&single_token_grammar());
    assert!(code.contains("tree_sitter_ag_v9_single"));
}

#[test]
fn single_token_contains_tslanguage() {
    let code = generate(&single_token_grammar());
    assert!(code.contains("TSLanguage"));
}

// ============================================================================
// 2. two_token_* — two tokens (4 tests)
// ============================================================================

#[test]
fn two_token_nonempty() {
    let code = generate(&two_token_grammar());
    assert!(!code.is_empty());
}

#[test]
fn two_token_contains_grammar_name() {
    let code = generate(&two_token_grammar());
    assert!(code.contains("ag_v9_two"));
}

#[test]
fn two_token_more_symbols_than_single() {
    let pt_one = build_table(&single_token_grammar());
    let pt_two = build_table(&two_token_grammar());
    assert!(
        pt_two.symbol_count > pt_one.symbol_count,
        "two-token grammar should have more symbols"
    );
}

#[test]
fn two_token_differs_from_single() {
    let code_one = generate(&single_token_grammar());
    let code_two = generate(&two_token_grammar());
    assert_ne!(code_one, code_two);
}

// ============================================================================
// 3. three_token_* — three tokens (4 tests)
// ============================================================================

#[test]
fn three_token_nonempty() {
    let code = generate(&three_token_grammar());
    assert!(!code.is_empty());
}

#[test]
fn three_token_contains_grammar_name() {
    let code = generate(&three_token_grammar());
    assert!(code.contains("ag_v9_three"));
}

#[test]
fn three_token_more_symbols_than_two() {
    let pt_two = build_table(&two_token_grammar());
    let pt_three = build_table(&three_token_grammar());
    assert!(
        pt_three.symbol_count > pt_two.symbol_count,
        "three-token grammar should have more symbols"
    );
}

#[test]
fn three_token_differs_from_two() {
    let code_two = generate(&two_token_grammar());
    let code_three = generate(&three_token_grammar());
    assert_ne!(code_two, code_three);
}

// ============================================================================
// 4. keyword_* — keyword grammar (4 tests)
// ============================================================================

#[test]
fn keyword_nonempty() {
    let code = generate(&keyword_grammar());
    assert!(!code.is_empty());
}

#[test]
fn keyword_contains_grammar_name() {
    let code = generate(&keyword_grammar());
    assert!(code.contains("ag_v9_keyword"));
}

#[test]
fn keyword_contains_tree_sitter_fn() {
    let code = generate(&keyword_grammar());
    assert!(code.contains("tree_sitter_ag_v9_keyword"));
}

#[test]
fn keyword_differs_from_single() {
    let code_kw = generate(&keyword_grammar());
    let code_single = generate(&single_token_grammar());
    assert_ne!(code_kw, code_single);
}

// ============================================================================
// 5. number_* — number grammar (4 tests)
// ============================================================================

#[test]
fn number_nonempty() {
    let code = generate(&number_grammar());
    assert!(!code.is_empty());
}

#[test]
fn number_contains_grammar_name() {
    let code = generate(&number_grammar());
    assert!(code.contains("ag_v9_number"));
}

#[test]
fn number_contains_tree_sitter_fn() {
    let code = generate(&number_grammar());
    assert!(code.contains("tree_sitter_ag_v9_number"));
}

#[test]
fn number_differs_from_keyword() {
    let code_num = generate(&number_grammar());
    let code_kw = generate(&keyword_grammar());
    assert_ne!(code_num, code_kw);
}

// ============================================================================
// 6. binexpr_* — binary expression (4 tests)
// ============================================================================

#[test]
fn binexpr_nonempty() {
    let code = generate(&binexpr_grammar());
    assert!(!code.is_empty());
}

#[test]
fn binexpr_contains_grammar_name() {
    let code = generate(&binexpr_grammar());
    assert!(code.contains("ag_v9_binexpr"));
}

#[test]
fn binexpr_contains_tree_sitter_fn() {
    let code = generate(&binexpr_grammar());
    assert!(code.contains("tree_sitter_ag_v9_binexpr"));
}

#[test]
fn binexpr_more_complex_than_single() {
    let code_bin = generate(&binexpr_grammar());
    let code_single = generate(&single_token_grammar());
    assert!(
        code_bin.len() > code_single.len(),
        "binary expr grammar should produce more code"
    );
}

// ============================================================================
// 7. unary_* — unary expression (4 tests)
// ============================================================================

#[test]
fn unary_nonempty() {
    let code = generate(&unary_grammar());
    assert!(!code.is_empty());
}

#[test]
fn unary_contains_grammar_name() {
    let code = generate(&unary_grammar());
    assert!(code.contains("ag_v9_unary"));
}

#[test]
fn unary_contains_tree_sitter_fn() {
    let code = generate(&unary_grammar());
    assert!(code.contains("tree_sitter_ag_v9_unary"));
}

#[test]
fn unary_differs_from_binexpr() {
    let code_un = generate(&unary_grammar());
    let code_bin = generate(&binexpr_grammar());
    assert_ne!(code_un, code_bin);
}

// ============================================================================
// 8. ifthen_* — if-then grammar (4 tests)
// ============================================================================

#[test]
fn ifthen_nonempty() {
    let code = generate(&ifthen_grammar());
    assert!(!code.is_empty());
}

#[test]
fn ifthen_contains_grammar_name() {
    let code = generate(&ifthen_grammar());
    assert!(code.contains("ag_v9_ifthen"));
}

#[test]
fn ifthen_contains_tree_sitter_fn() {
    let code = generate(&ifthen_grammar());
    assert!(code.contains("tree_sitter_ag_v9_ifthen"));
}

#[test]
fn ifthen_differs_from_assign() {
    let code_if = generate(&ifthen_grammar());
    let code_asgn = generate(&assign_grammar());
    assert_ne!(code_if, code_asgn);
}

// ============================================================================
// 9. assign_* — assignment grammar (4 tests)
// ============================================================================

#[test]
fn assign_nonempty() {
    let code = generate(&assign_grammar());
    assert!(!code.is_empty());
}

#[test]
fn assign_contains_grammar_name() {
    let code = generate(&assign_grammar());
    assert!(code.contains("ag_v9_assign"));
}

#[test]
fn assign_contains_tree_sitter_fn() {
    let code = generate(&assign_grammar());
    assert!(code.contains("tree_sitter_ag_v9_assign"));
}

#[test]
fn assign_differs_from_funcall() {
    let code_asgn = generate(&assign_grammar());
    let code_fn = generate(&funcall_grammar());
    assert_ne!(code_asgn, code_fn);
}

// ============================================================================
// 10. funcall_* — function call grammar (4 tests)
// ============================================================================

#[test]
fn funcall_nonempty() {
    let code = generate(&funcall_grammar());
    assert!(!code.is_empty());
}

#[test]
fn funcall_contains_grammar_name() {
    let code = generate(&funcall_grammar());
    assert!(code.contains("ag_v9_funcall"));
}

#[test]
fn funcall_contains_tree_sitter_fn() {
    let code = generate(&funcall_grammar());
    assert!(code.contains("tree_sitter_ag_v9_funcall"));
}

#[test]
fn funcall_differs_from_list() {
    let code_fn = generate(&funcall_grammar());
    let code_list = generate(&list_grammar());
    assert_ne!(code_fn, code_list);
}

// ============================================================================
// 11. list_* — list grammar (4 tests)
// ============================================================================

#[test]
fn list_nonempty() {
    let code = generate(&list_grammar());
    assert!(!code.is_empty());
}

#[test]
fn list_contains_grammar_name() {
    let code = generate(&list_grammar());
    assert!(code.contains("ag_v9_list"));
}

#[test]
fn list_contains_tree_sitter_fn() {
    let code = generate(&list_grammar());
    assert!(code.contains("tree_sitter_ag_v9_list"));
}

#[test]
fn list_differs_from_block() {
    let code_list = generate(&list_grammar());
    let code_block = generate(&block_grammar());
    assert_ne!(code_list, code_block);
}

// ============================================================================
// 12. block_* — block grammar (4 tests)
// ============================================================================

#[test]
fn block_nonempty() {
    let code = generate(&block_grammar());
    assert!(!code.is_empty());
}

#[test]
fn block_contains_grammar_name() {
    let code = generate(&block_grammar());
    assert!(code.contains("ag_v9_block"));
}

#[test]
fn block_contains_tree_sitter_fn() {
    let code = generate(&block_grammar());
    assert!(code.contains("tree_sitter_ag_v9_block"));
}

#[test]
fn block_more_complex_than_single() {
    let code_block = generate(&block_grammar());
    let code_single = generate(&single_token_grammar());
    assert!(
        code_block.len() > code_single.len(),
        "block grammar should produce more code"
    );
}

// ============================================================================
// 13. ws_extra_* — whitespace extra (4 tests)
// ============================================================================

#[test]
fn ws_extra_nonempty() {
    let code = generate(&ws_extra_grammar());
    assert!(!code.is_empty());
}

#[test]
fn ws_extra_contains_grammar_name() {
    let code = generate(&ws_extra_grammar());
    assert!(code.contains("ag_v9_wsextra"));
}

#[test]
fn ws_extra_grammar_has_extras_set() {
    let g = ws_extra_grammar();
    assert!(!g.extras.is_empty(), "extras should be set");
}

#[test]
fn ws_extra_differs_from_comment() {
    let code_ws = generate(&ws_extra_grammar());
    let code_cmt = generate(&comment_grammar());
    assert_ne!(code_ws, code_cmt);
}

// ============================================================================
// 14. comment_* — comment token (4 tests)
// ============================================================================

#[test]
fn comment_nonempty() {
    let code = generate(&comment_grammar());
    assert!(!code.is_empty());
}

#[test]
fn comment_contains_grammar_name() {
    let code = generate(&comment_grammar());
    assert!(code.contains("ag_v9_comment"));
}

#[test]
fn comment_grammar_has_extras_set() {
    let g = comment_grammar();
    assert!(!g.extras.is_empty(), "extras should be set for comment");
}

#[test]
fn comment_differs_from_single() {
    let code_cmt = generate(&comment_grammar());
    let code_single = generate(&single_token_grammar());
    assert_ne!(code_cmt, code_single);
}

// ============================================================================
// 15. prec_* — precedence (4 tests)
// ============================================================================

#[test]
fn prec_nonempty() {
    let code = generate(&prec_grammar());
    assert!(!code.is_empty());
}

#[test]
fn prec_contains_grammar_name() {
    let code = generate(&prec_grammar());
    assert!(code.contains("ag_v9_prec"));
}

#[test]
fn prec_contains_tree_sitter_fn() {
    let code = generate(&prec_grammar());
    assert!(code.contains("tree_sitter_ag_v9_prec"));
}

#[test]
fn prec_differs_from_binexpr() {
    let code_prec = generate(&prec_grammar());
    let code_bin = generate(&binexpr_grammar());
    assert_ne!(code_prec, code_bin);
}

// ============================================================================
// 16. left_assoc_* — left associativity (4 tests)
// ============================================================================

#[test]
fn left_assoc_nonempty() {
    let code = generate(&left_assoc_grammar());
    assert!(!code.is_empty());
}

#[test]
fn left_assoc_contains_grammar_name() {
    let code = generate(&left_assoc_grammar());
    assert!(code.contains("ag_v9_lassoc"));
}

#[test]
fn left_assoc_contains_tree_sitter_fn() {
    let code = generate(&left_assoc_grammar());
    assert!(code.contains("tree_sitter_ag_v9_lassoc"));
}

#[test]
fn left_assoc_differs_from_right_assoc() {
    let code_left = generate(&left_assoc_grammar());
    let code_right = generate(&right_assoc_grammar());
    assert_ne!(code_left, code_right);
}

// ============================================================================
// 17. right_assoc_* — right associativity (4 tests)
// ============================================================================

#[test]
fn right_assoc_nonempty() {
    let code = generate(&right_assoc_grammar());
    assert!(!code.is_empty());
}

#[test]
fn right_assoc_contains_grammar_name() {
    let code = generate(&right_assoc_grammar());
    assert!(code.contains("ag_v9_rassoc"));
}

#[test]
fn right_assoc_contains_tree_sitter_fn() {
    let code = generate(&right_assoc_grammar());
    assert!(code.contains("tree_sitter_ag_v9_rassoc"));
}

#[test]
fn right_assoc_differs_from_prec() {
    let code_right = generate(&right_assoc_grammar());
    let code_prec = generate(&prec_grammar());
    assert_ne!(code_right, code_prec);
}

// ============================================================================
// 18. inline_* — inline rules (4 tests)
// ============================================================================

#[test]
fn inline_nonempty() {
    let code = generate(&inline_grammar());
    assert!(!code.is_empty());
}

#[test]
fn inline_contains_grammar_name() {
    let code = generate(&inline_grammar());
    assert!(code.contains("ag_v9_inline"));
}

#[test]
fn inline_grammar_has_inline_set() {
    let g = inline_grammar();
    assert!(!g.inline_rules.is_empty(), "inline rules should be set");
}

#[test]
fn inline_differs_from_single() {
    let code_inl = generate(&inline_grammar());
    let code_single = generate(&single_token_grammar());
    assert_ne!(code_inl, code_single);
}

// ============================================================================
// 19. extern_* — external scanner (4 tests)
// ============================================================================

#[test]
fn extern_nonempty() {
    let code = generate(&extern_grammar());
    assert!(!code.is_empty());
}

#[test]
fn extern_contains_grammar_name() {
    let code = generate(&extern_grammar());
    assert!(code.contains("ag_v9_extern"));
}

#[test]
fn extern_grammar_has_externals() {
    let g = extern_grammar();
    assert!(!g.externals.is_empty(), "externals should be set");
}

#[test]
fn extern_differs_from_single() {
    let code_ext = generate(&extern_grammar());
    let code_single = generate(&single_token_grammar());
    assert_ne!(code_ext, code_single);
}

// ============================================================================
// 20. determinism_* — ABI output deterministic (4 tests)
// ============================================================================

#[test]
fn determinism_single_token() {
    let g = single_token_grammar();
    let code1 = generate(&g);
    let code2 = generate(&g);
    assert_eq!(code1, code2, "generation must be deterministic");
}

#[test]
fn determinism_binexpr() {
    let g = binexpr_grammar();
    let code1 = generate(&g);
    let code2 = generate(&g);
    assert_eq!(code1, code2, "generation must be deterministic");
}

#[test]
fn determinism_list() {
    let g = list_grammar();
    let code1 = generate(&g);
    let code2 = generate(&g);
    assert_eq!(code1, code2, "generation must be deterministic");
}

#[test]
fn determinism_block() {
    let g = block_grammar();
    let code1 = generate(&g);
    let code2 = generate(&g);
    assert_eq!(code1, code2, "generation must be deterministic");
}

// ============================================================================
// Extra cross-cutting tests (to reach 80+)
// ============================================================================

#[test]
fn cross_all_grammars_nonempty() {
    let grammars: Vec<Grammar> = vec![
        single_token_grammar(),
        two_token_grammar(),
        three_token_grammar(),
        keyword_grammar(),
        number_grammar(),
        binexpr_grammar(),
        unary_grammar(),
        ifthen_grammar(),
        assign_grammar(),
        funcall_grammar(),
        list_grammar(),
        block_grammar(),
        ws_extra_grammar(),
        comment_grammar(),
        prec_grammar(),
        left_assoc_grammar(),
        right_assoc_grammar(),
        inline_grammar(),
        extern_grammar(),
    ];
    for g in &grammars {
        let code = generate(g);
        assert!(
            !code.is_empty(),
            "grammar '{}' produced empty output",
            g.name
        );
    }
}

#[test]
fn cross_all_grammars_contain_tslanguage() {
    let grammars: Vec<Grammar> = vec![
        single_token_grammar(),
        two_token_grammar(),
        three_token_grammar(),
        keyword_grammar(),
        number_grammar(),
        binexpr_grammar(),
        unary_grammar(),
        ifthen_grammar(),
        assign_grammar(),
        funcall_grammar(),
        list_grammar(),
        block_grammar(),
        ws_extra_grammar(),
        comment_grammar(),
        prec_grammar(),
        left_assoc_grammar(),
        right_assoc_grammar(),
        inline_grammar(),
        extern_grammar(),
    ];
    for g in &grammars {
        let code = generate(g);
        assert!(
            code.contains("TSLanguage"),
            "grammar '{}' missing TSLanguage",
            g.name
        );
    }
}

#[test]
fn cross_all_grammars_code_len_above_100() {
    let grammars: Vec<Grammar> = vec![
        single_token_grammar(),
        two_token_grammar(),
        three_token_grammar(),
        keyword_grammar(),
        number_grammar(),
        binexpr_grammar(),
        unary_grammar(),
        ifthen_grammar(),
        assign_grammar(),
        funcall_grammar(),
        list_grammar(),
        block_grammar(),
        ws_extra_grammar(),
        comment_grammar(),
        prec_grammar(),
        left_assoc_grammar(),
        right_assoc_grammar(),
        inline_grammar(),
        extern_grammar(),
    ];
    for g in &grammars {
        let code = generate(g);
        assert!(
            code.len() > 100,
            "grammar '{}' code too short: {} bytes",
            g.name,
            code.len()
        );
    }
}

#[test]
fn cross_all_grammars_contain_static_or_const() {
    let grammars: Vec<Grammar> = vec![
        single_token_grammar(),
        two_token_grammar(),
        three_token_grammar(),
        keyword_grammar(),
        number_grammar(),
        binexpr_grammar(),
        unary_grammar(),
        ifthen_grammar(),
        assign_grammar(),
        funcall_grammar(),
        list_grammar(),
        block_grammar(),
        ws_extra_grammar(),
        comment_grammar(),
        prec_grammar(),
        left_assoc_grammar(),
        right_assoc_grammar(),
        inline_grammar(),
        extern_grammar(),
    ];
    for g in &grammars {
        let code = generate(g);
        let has_static = code.contains("static");
        let has_const = code.contains("const");
        assert!(
            has_static || has_const,
            "grammar '{}' must contain 'static' or 'const'",
            g.name
        );
    }
}

#[test]
fn cross_all_grammars_no_unknown_symbols() {
    let grammars: Vec<Grammar> = vec![
        single_token_grammar(),
        two_token_grammar(),
        three_token_grammar(),
        keyword_grammar(),
        number_grammar(),
        binexpr_grammar(),
        unary_grammar(),
        ifthen_grammar(),
        assign_grammar(),
        funcall_grammar(),
        list_grammar(),
        block_grammar(),
        ws_extra_grammar(),
        comment_grammar(),
        prec_grammar(),
        left_assoc_grammar(),
        right_assoc_grammar(),
        inline_grammar(),
        extern_grammar(),
    ];
    for g in &grammars {
        let code = generate(g);
        assert!(
            !code.contains("???"),
            "grammar '{}' has unknown symbol markers",
            g.name
        );
    }
}

#[test]
fn cross_all_unique_names_produce_unique_code() {
    let grammars: Vec<Grammar> = vec![
        single_token_grammar(),
        two_token_grammar(),
        three_token_grammar(),
        keyword_grammar(),
        number_grammar(),
        binexpr_grammar(),
        unary_grammar(),
        ifthen_grammar(),
        assign_grammar(),
        funcall_grammar(),
        list_grammar(),
        block_grammar(),
        ws_extra_grammar(),
        comment_grammar(),
        prec_grammar(),
        left_assoc_grammar(),
        right_assoc_grammar(),
        inline_grammar(),
        extern_grammar(),
    ];
    let codes: Vec<String> = grammars.iter().map(generate).collect();
    for i in 0..codes.len() {
        for j in (i + 1)..codes.len() {
            assert_ne!(
                codes[i], codes[j],
                "grammar '{}' and '{}' should produce different code",
                grammars[i].name, grammars[j].name
            );
        }
    }
}

#[test]
fn cross_parse_table_states_at_least_two() {
    let grammars: Vec<Grammar> = vec![
        single_token_grammar(),
        two_token_grammar(),
        three_token_grammar(),
        keyword_grammar(),
        number_grammar(),
        unary_grammar(),
        ifthen_grammar(),
        assign_grammar(),
        funcall_grammar(),
        list_grammar(),
        block_grammar(),
        ws_extra_grammar(),
        comment_grammar(),
        left_assoc_grammar(),
        right_assoc_grammar(),
        inline_grammar(),
        extern_grammar(),
    ];
    for g in &grammars {
        let pt = build_table(g);
        assert!(
            pt.state_count >= 2,
            "grammar '{}' should have at least 2 states, got {}",
            g.name,
            pt.state_count
        );
    }
}

#[test]
fn cross_builder_construct_no_panic() {
    let grammars: Vec<Grammar> = vec![
        single_token_grammar(),
        two_token_grammar(),
        three_token_grammar(),
        keyword_grammar(),
        number_grammar(),
        binexpr_grammar(),
        unary_grammar(),
        ifthen_grammar(),
        assign_grammar(),
        funcall_grammar(),
        list_grammar(),
        block_grammar(),
        ws_extra_grammar(),
        comment_grammar(),
        prec_grammar(),
        left_assoc_grammar(),
        right_assoc_grammar(),
        inline_grammar(),
        extern_grammar(),
    ];
    for g in &grammars {
        let pt = build_table(g);
        let _builder = AbiLanguageBuilder::new(g, &pt);
    }
}

#[test]
fn cross_determinism_all_grammars() {
    let grammars: Vec<Grammar> = vec![
        single_token_grammar(),
        two_token_grammar(),
        three_token_grammar(),
        keyword_grammar(),
        number_grammar(),
        binexpr_grammar(),
        unary_grammar(),
        ifthen_grammar(),
        assign_grammar(),
        funcall_grammar(),
        list_grammar(),
        block_grammar(),
        ws_extra_grammar(),
        comment_grammar(),
        prec_grammar(),
        left_assoc_grammar(),
        right_assoc_grammar(),
        inline_grammar(),
        extern_grammar(),
    ];
    for g in &grammars {
        let code1 = generate(g);
        let code2 = generate(g);
        assert_eq!(code1, code2, "grammar '{}' is not deterministic", g.name);
    }
}

#[test]
fn cross_combined_extras_and_precedence() {
    let g = GrammarBuilder::new("ag_v9_combined")
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
    assert!(code.contains("tree_sitter_ag_v9_combined"));
}

#[test]
fn cross_nullable_rule() {
    let g = GrammarBuilder::new("ag_v9_nullable")
        .token("a", "a")
        .rule("start", vec!["opt"])
        .rule("opt", vec!["a"])
        .rule("opt", vec![])
        .start("start")
        .build();
    let code = generate(&g);
    assert!(!code.is_empty());
    assert!(code.contains("ag_v9_nullable"));
}

#[test]
fn cross_left_recursive() {
    let g = GrammarBuilder::new("ag_v9_leftrec")
        .token("a", "a")
        .rule("start", vec!["a"])
        .rule("start", vec!["start", "a"])
        .start("start")
        .build();
    let code = generate(&g);
    assert!(!code.is_empty());
    assert!(code.contains("ag_v9_leftrec"));
}

#[test]
fn cross_right_recursive() {
    let g = GrammarBuilder::new("ag_v9_rightrec")
        .token("a", "a")
        .rule("start", vec!["a"])
        .rule("start", vec!["a", "start"])
        .start("start")
        .build();
    let code = generate(&g);
    assert!(!code.is_empty());
    assert!(code.contains("ag_v9_rightrec"));
}

#[test]
fn cross_deep_chain() {
    let g = GrammarBuilder::new("ag_v9_deep")
        .token("z", "z")
        .rule("start", vec!["l1"])
        .rule("l1", vec!["l2"])
        .rule("l2", vec!["l3"])
        .rule("l3", vec!["z"])
        .start("start")
        .build();
    let code = generate(&g);
    assert!(!code.is_empty());
    assert!(code.contains("ag_v9_deep"));
}

#[test]
fn cross_wide_alternatives() {
    let mut gb = GrammarBuilder::new("ag_v9_wide");
    for i in 0..10u8 {
        let name = format!("t{i}");
        let pat = format!("{}", (b'a' + i) as char);
        gb = gb.token(&name, &pat);
        gb = gb.rule("start", vec![&name]);
    }
    let g = gb.start("start").build();
    let code = generate(&g);
    assert!(!code.is_empty());
    assert!(code.contains("ag_v9_wide"));
}

#[test]
fn cross_multiple_nonterminals() {
    let g = GrammarBuilder::new("ag_v9_multi_nt")
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
    assert!(code.contains("tree_sitter_ag_v9_multi_nt"));
}

#[test]
fn cross_default_table_no_panic() {
    let g = Grammar::new("ag_v9_empty".to_string());
    let pt = ParseTable::default();
    let code = AbiLanguageBuilder::new(&g, &pt).generate().to_string();
    assert!(!code.is_empty());
}
