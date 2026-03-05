//! Tests validating the generated Rust code from `StaticLanguageGenerator`.
//!
//! 80+ tests covering: non-empty output, const/static presence, UTF-8 validity,
//! array syntax, numeric literals, determinism, grammar differentiation, code
//! growth, compactness, state/symbol data, grammar name embedding, precedence,
//! extras, externals, inlines, alternatives, chain rules, recursion, formatting.

use adze_glr_core::{FirstFollowSets, ParseTable, build_lr1_automaton};
use adze_ir::Associativity;
use adze_ir::builder::GrammarBuilder;
use adze_tablegen::StaticLanguageGenerator;

// ---------------------------------------------------------------------------
// Helpers — each grammar name is prefixed "sc_v9_" for uniqueness
// ---------------------------------------------------------------------------

/// Build a grammar, normalize, compute FIRST/FOLLOW, build LR(1), generate code string.
fn gen_code(
    name: &str,
    tokens: &[(&str, &str)],
    rules: &[(&str, Vec<&str>)],
    start: &str,
) -> String {
    let mut b = GrammarBuilder::new(name);
    for &(n, p) in tokens {
        b = b.token(n, p);
    }
    for (lhs, rhs) in rules {
        b = b.rule(lhs, rhs.clone());
    }
    b = b.start(start);
    let mut g = b.build();
    g.normalize();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let pt = build_lr1_automaton(&g, &ff).unwrap();
    let slg = StaticLanguageGenerator::new(g, pt);
    slg.generate_language_code().to_string()
}

/// Minimal single-token grammar.
fn minimal_code() -> String {
    gen_code("sc_v9_min", &[("a", "a")], &[("s", vec!["a"])], "s")
}

/// Two-alternative grammar.
fn two_alt_code() -> String {
    gen_code(
        "sc_v9_two",
        &[("a", "a"), ("b", "b")],
        &[("s", vec!["a"]), ("s", vec!["b"])],
        "s",
    )
}

/// Medium grammar with multiple non-terminals and tokens.
fn medium_code() -> String {
    gen_code(
        "sc_v9_med",
        &[
            ("n", r"\d+"),
            ("id", r"[a-z]+"),
            ("plus", "+"),
            ("semi", ";"),
        ],
        &[
            ("expr", vec!["n"]),
            ("expr", vec!["id"]),
            ("stmt", vec!["expr", "semi"]),
            ("prog", vec!["stmt"]),
            ("prog", vec!["prog", "stmt"]),
        ],
        "prog",
    )
}

/// Large grammar (many tokens and rules).
fn large_code() -> String {
    let tokens: Vec<(&str, &str)> = vec![
        ("t0", "a"),
        ("t1", "b"),
        ("t2", "c"),
        ("t3", "d"),
        ("t4", "e"),
        ("t5", "f"),
        ("t6", "g"),
        ("t7", "h"),
        ("t8", "i"),
        ("t9", "j"),
    ];
    let rules: Vec<(&str, Vec<&str>)> = vec![
        ("r0", vec!["t0"]),
        ("r1", vec!["t1"]),
        ("r2", vec!["t2"]),
        ("r3", vec!["t3"]),
        ("r4", vec!["t4"]),
        ("r5", vec!["t5"]),
        ("r6", vec!["t6"]),
        ("r7", vec!["t7"]),
        ("top", vec!["r0"]),
        ("top", vec!["r1"]),
        ("top", vec!["r2"]),
        ("top", vec!["r3"]),
    ];
    gen_code("sc_v9_large", &tokens, &rules, "top")
}

// =========================================================================
// 1. Generated code is non-empty
// =========================================================================

#[test]
fn t01_minimal_code_nonempty() {
    assert!(!minimal_code().is_empty());
}

#[test]
fn t01b_two_alt_code_nonempty() {
    assert!(!two_alt_code().is_empty());
}

#[test]
fn t01c_medium_code_nonempty() {
    assert!(!medium_code().is_empty());
}

#[test]
fn t01d_large_code_nonempty() {
    assert!(!large_code().is_empty());
}

// =========================================================================
// 2. Generated code contains "const" or "static"
// =========================================================================

#[test]
fn t02_minimal_has_const_or_static() {
    let code = minimal_code();
    assert!(
        code.contains("const") || code.contains("static"),
        "generated code must contain const or static declarations"
    );
}

#[test]
fn t02b_medium_has_const_or_static() {
    let code = medium_code();
    assert!(code.contains("const") || code.contains("static"));
}

// =========================================================================
// 3. Generated code is valid UTF-8
// =========================================================================

#[test]
fn t03_minimal_is_valid_utf8() {
    let code = minimal_code();
    // code is already a String (valid UTF-8), verify round-trip
    let bytes = code.as_bytes();
    assert!(std::str::from_utf8(bytes).is_ok());
}

#[test]
fn t03b_large_is_valid_utf8() {
    let code = large_code();
    assert!(std::str::from_utf8(code.as_bytes()).is_ok());
}

// =========================================================================
// 4. Generated code contains array syntax (brackets)
// =========================================================================

#[test]
fn t04_minimal_has_brackets() {
    let code = minimal_code();
    assert!(code.contains('[') && code.contains(']'));
}

#[test]
fn t04b_medium_has_brackets() {
    let code = medium_code();
    assert!(code.contains('[') && code.contains(']'));
}

// =========================================================================
// 5. Generated code contains numeric literals
// =========================================================================

#[test]
fn t05_minimal_has_numeric_literal() {
    let code = minimal_code();
    assert!(
        code.chars().any(|c| c.is_ascii_digit()),
        "generated code should contain numeric literals"
    );
}

#[test]
fn t05b_large_has_numeric_literal() {
    let code = large_code();
    assert!(code.chars().any(|c| c.is_ascii_digit()));
}

// =========================================================================
// 6. Generated code is deterministic
// =========================================================================

#[test]
fn t06_minimal_deterministic() {
    let a = minimal_code();
    let b = minimal_code();
    assert_eq!(a, b);
}

#[test]
fn t06b_two_alt_deterministic() {
    let a = two_alt_code();
    let b = two_alt_code();
    assert_eq!(a, b);
}

#[test]
fn t06c_medium_deterministic() {
    let a = medium_code();
    let b = medium_code();
    assert_eq!(a, b);
}

#[test]
fn t06d_large_deterministic() {
    let a = large_code();
    let b = large_code();
    assert_eq!(a, b);
}

// =========================================================================
// 7. Different grammars produce different code
// =========================================================================

#[test]
fn t07_different_names_differ() {
    let a = gen_code("sc_v9_alpha", &[("x", "x")], &[("s", vec!["x"])], "s");
    let b = gen_code("sc_v9_beta", &[("x", "x")], &[("s", vec!["x"])], "s");
    assert_ne!(a, b);
}

#[test]
fn t07b_different_rules_differ() {
    let a = gen_code(
        "sc_v9_ra",
        &[("x", "x"), ("y", "y")],
        &[("s", vec!["x"])],
        "s",
    );
    let b = gen_code(
        "sc_v9_rb",
        &[("x", "x"), ("y", "y")],
        &[("s", vec!["y"])],
        "s",
    );
    assert_ne!(a, b);
}

#[test]
fn t07c_different_token_counts_differ() {
    let a = gen_code("sc_v9_t1", &[("x", "x")], &[("s", vec!["x"])], "s");
    let b = gen_code(
        "sc_v9_t2",
        &[("x", "x"), ("y", "y")],
        &[("s", vec!["x"]), ("s", vec!["y"])],
        "s",
    );
    assert_ne!(a, b);
}

// =========================================================================
// 8. Code length grows with grammar size
// =========================================================================

#[test]
fn t08_large_longer_than_minimal() {
    assert!(large_code().len() > minimal_code().len());
}

#[test]
fn t08b_medium_longer_than_minimal() {
    assert!(medium_code().len() > minimal_code().len());
}

#[test]
fn t08c_large_longer_than_medium() {
    assert!(large_code().len() > medium_code().len());
}

// =========================================================================
// 9. Minimal grammar produces compact code
// =========================================================================

#[test]
fn t09_minimal_compact() {
    let min_len = minimal_code().len();
    let large_len = large_code().len();
    assert!(
        min_len < large_len,
        "minimal grammar ({min_len}) should be shorter than large grammar ({large_len})"
    );
}

#[test]
fn t09b_minimal_under_threshold() {
    // Minimal grammar shouldn't explode in size
    let code = minimal_code();
    assert!(
        code.len() < 100_000,
        "minimal grammar code length {} is unexpectedly large",
        code.len()
    );
}

// =========================================================================
// 10. Code contains state count data
// =========================================================================

#[test]
fn t10_has_state_count_keyword() {
    let code = medium_code();
    // The generator embeds state-related data in arrays or constants
    assert!(
        code.contains("STATE") || code.contains("state") || code.contains("lex_state"),
        "generated code should reference state information"
    );
}

#[test]
fn t10b_large_has_state_data() {
    let code = large_code();
    assert!(code.contains("STATE") || code.contains("state") || code.contains("lex_state"),);
}

// =========================================================================
// 11. Code contains symbol data
// =========================================================================

#[test]
fn t11_has_symbol_keyword() {
    let code = medium_code();
    assert!(
        code.contains("SYMBOL") || code.contains("symbol"),
        "generated code should reference symbol data"
    );
}

#[test]
fn t11b_has_symbol_names() {
    let code = medium_code();
    assert!(
        code.contains("SYMBOL_NAMES") || code.contains("symbol_names"),
        "generated code should contain symbol names"
    );
}

#[test]
fn t11c_minimal_has_symbol_data() {
    let code = minimal_code();
    assert!(code.contains("SYMBOL") || code.contains("symbol"));
}

// =========================================================================
// 12. Grammar name in code
// =========================================================================

#[test]
fn t12_grammar_name_embedded() {
    let code = minimal_code();
    assert!(
        code.contains("sc_v9_min"),
        "generated code should contain the grammar name"
    );
}

#[test]
fn t12b_medium_name_embedded() {
    let code = medium_code();
    assert!(code.contains("sc_v9_med"));
}

#[test]
fn t12c_large_name_embedded() {
    let code = large_code();
    assert!(code.contains("sc_v9_large"));
}

#[test]
fn t12d_custom_name_embedded() {
    let code = gen_code("sc_v9_custom_name", &[("z", "z")], &[("s", vec!["z"])], "s");
    assert!(code.contains("sc_v9_custom_name"));
}

// =========================================================================
// 13. Code with precedence grammar
// =========================================================================

fn precedence_code() -> String {
    let mut b = GrammarBuilder::new("sc_v9_prec")
        .token("n", r"\d+")
        .token("plus", "+")
        .token("star", "*")
        .rule("expr", vec!["n"]);
    b = b.rule_with_precedence("expr", vec!["expr", "plus", "expr"], 1, Associativity::Left);
    b = b.rule_with_precedence("expr", vec!["expr", "star", "expr"], 2, Associativity::Left);
    b = b.start("expr");
    let mut g = b.build();
    g.normalize();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let pt = build_lr1_automaton(&g, &ff).unwrap();
    StaticLanguageGenerator::new(g, pt)
        .generate_language_code()
        .to_string()
}

#[test]
fn t13_precedence_nonempty() {
    assert!(!precedence_code().is_empty());
}

#[test]
fn t13b_precedence_has_brackets() {
    assert!(precedence_code().contains('['));
}

#[test]
fn t13c_precedence_has_const_or_static() {
    let code = precedence_code();
    assert!(code.contains("const") || code.contains("static"));
}

#[test]
fn t13d_precedence_deterministic() {
    assert_eq!(precedence_code(), precedence_code());
}

// =========================================================================
// 14. Code with extras grammar
// =========================================================================

fn extras_code() -> String {
    let b = GrammarBuilder::new("sc_v9_extras")
        .token("id", r"[a-z]+")
        .token("ws", r"[ \t]+")
        .extra("ws")
        .rule("s", vec!["id"])
        .start("s");
    let mut g = b.build();
    g.normalize();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let pt = build_lr1_automaton(&g, &ff).unwrap();
    StaticLanguageGenerator::new(g, pt)
        .generate_language_code()
        .to_string()
}

#[test]
fn t14_extras_nonempty() {
    assert!(!extras_code().is_empty());
}

#[test]
fn t14b_extras_has_name() {
    assert!(extras_code().contains("sc_v9_extras"));
}

#[test]
fn t14c_extras_deterministic() {
    assert_eq!(extras_code(), extras_code());
}

#[test]
fn t14d_extras_has_brackets() {
    assert!(extras_code().contains('['));
}

// =========================================================================
// 15. Code with externals grammar
// =========================================================================

fn externals_code() -> String {
    let b = GrammarBuilder::new("sc_v9_ext")
        .token("id", r"[a-z]+")
        .external("indent")
        .external("dedent")
        .rule("block", vec!["indent", "id", "dedent"])
        .start("block");
    let mut g = b.build();
    g.normalize();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let pt = build_lr1_automaton(&g, &ff).unwrap();
    StaticLanguageGenerator::new(g, pt)
        .generate_language_code()
        .to_string()
}

#[test]
fn t15_externals_nonempty() {
    assert!(!externals_code().is_empty());
}

#[test]
fn t15b_externals_has_name() {
    assert!(externals_code().contains("sc_v9_ext"));
}

#[test]
fn t15c_externals_deterministic() {
    assert_eq!(externals_code(), externals_code());
}

#[test]
fn t15d_externals_has_const_or_static() {
    let code = externals_code();
    assert!(code.contains("const") || code.contains("static"));
}

// =========================================================================
// 16. Code with inline grammar
// =========================================================================

fn inline_code() -> String {
    let b = GrammarBuilder::new("sc_v9_inl")
        .token("a", "a")
        .token("b", "b")
        .rule("helper", vec!["a"])
        .inline("helper")
        .rule("s", vec!["helper", "b"])
        .start("s");
    let mut g = b.build();
    g.normalize();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let pt = build_lr1_automaton(&g, &ff).unwrap();
    StaticLanguageGenerator::new(g, pt)
        .generate_language_code()
        .to_string()
}

#[test]
fn t16_inline_nonempty() {
    assert!(!inline_code().is_empty());
}

#[test]
fn t16b_inline_has_name() {
    assert!(inline_code().contains("sc_v9_inl"));
}

#[test]
fn t16c_inline_deterministic() {
    assert_eq!(inline_code(), inline_code());
}

#[test]
fn t16d_inline_has_numeric() {
    assert!(inline_code().chars().any(|c| c.is_ascii_digit()));
}

// =========================================================================
// 17. Code with alternatives
// =========================================================================

#[test]
fn t17_three_alternatives() {
    let code = gen_code(
        "sc_v9_alt3",
        &[("a", "a"), ("b", "b"), ("c", "c")],
        &[("s", vec!["a"]), ("s", vec!["b"]), ("s", vec!["c"])],
        "s",
    );
    assert!(!code.is_empty());
}

#[test]
fn t17b_five_alternatives() {
    let code = gen_code(
        "sc_v9_alt5",
        &[("a", "a"), ("b", "b"), ("c", "c"), ("d", "d"), ("e", "e")],
        &[
            ("s", vec!["a"]),
            ("s", vec!["b"]),
            ("s", vec!["c"]),
            ("s", vec!["d"]),
            ("s", vec!["e"]),
        ],
        "s",
    );
    assert!(!code.is_empty());
}

#[test]
fn t17c_alternatives_differ_from_single() {
    let single = gen_code("sc_v9_s1", &[("a", "a")], &[("s", vec!["a"])], "s");
    let multi = gen_code(
        "sc_v9_m1",
        &[("a", "a"), ("b", "b")],
        &[("s", vec!["a"]), ("s", vec!["b"])],
        "s",
    );
    assert_ne!(single, multi);
}

#[test]
fn t17d_alternatives_have_symbol_data() {
    let code = gen_code(
        "sc_v9_alt_sym",
        &[("a", "a"), ("b", "b"), ("c", "c")],
        &[("s", vec!["a"]), ("s", vec!["b"]), ("s", vec!["c"])],
        "s",
    );
    assert!(code.contains("SYMBOL") || code.contains("symbol"));
}

// =========================================================================
// 18. Code with chain rules
// =========================================================================

#[test]
fn t18_chain_two_levels() {
    let code = gen_code(
        "sc_v9_ch2",
        &[("x", "x")],
        &[("inner", vec!["x"]), ("s", vec!["inner"])],
        "s",
    );
    assert!(!code.is_empty());
}

#[test]
fn t18b_chain_three_levels() {
    let code = gen_code(
        "sc_v9_ch3",
        &[("x", "x")],
        &[
            ("deep", vec!["x"]),
            ("mid", vec!["deep"]),
            ("s", vec!["mid"]),
        ],
        "s",
    );
    assert!(!code.is_empty());
}

#[test]
fn t18c_chain_has_brackets() {
    let code = gen_code(
        "sc_v9_chb",
        &[("x", "x")],
        &[("inner", vec!["x"]), ("s", vec!["inner"])],
        "s",
    );
    assert!(code.contains('['));
}

#[test]
fn t18d_deeper_chain_is_longer() {
    let shallow = gen_code("sc_v9_shw", &[("x", "x")], &[("s", vec!["x"])], "s");
    let deep = gen_code(
        "sc_v9_dp",
        &[("x", "x")],
        &[("d", vec!["x"]), ("m", vec!["d"]), ("s", vec!["m"])],
        "s",
    );
    assert!(deep.len() > shallow.len());
}

// =========================================================================
// 19. Code with recursion
// =========================================================================

fn recursive_code() -> String {
    gen_code(
        "sc_v9_rec",
        &[("a", "a"), ("lp", "("), ("rp", ")")],
        &[("s", vec!["a"]), ("s", vec!["lp", "s", "rp"])],
        "s",
    )
}

#[test]
fn t19_recursive_nonempty() {
    assert!(!recursive_code().is_empty());
}

#[test]
fn t19b_recursive_has_name() {
    assert!(recursive_code().contains("sc_v9_rec"));
}

#[test]
fn t19c_recursive_deterministic() {
    assert_eq!(recursive_code(), recursive_code());
}

#[test]
fn t19d_recursive_has_const_or_static() {
    let code = recursive_code();
    assert!(code.contains("const") || code.contains("static"));
}

// =========================================================================
// 20. Code formatting consistency
// =========================================================================

#[test]
fn t20_no_null_bytes() {
    assert!(!minimal_code().contains('\0'));
}

#[test]
fn t20b_no_null_bytes_large() {
    assert!(!large_code().contains('\0'));
}

#[test]
fn t20c_semicolons_present() {
    let code = minimal_code();
    assert!(code.contains(';'), "Rust code should contain semicolons");
}

#[test]
fn t20d_braces_balanced() {
    let code = minimal_code();
    let opens = code.chars().filter(|&c| c == '{').count();
    let closes = code.chars().filter(|&c| c == '}').count();
    assert_eq!(opens, closes, "braces should be balanced");
}

// =========================================================================
// Additional coverage tests (21–25)
// =========================================================================

#[test]
fn t21_sequence_rule() {
    let code = gen_code(
        "sc_v9_seq",
        &[("a", "a"), ("b", "b"), ("c", "c")],
        &[("s", vec!["a", "b", "c"])],
        "s",
    );
    assert!(!code.is_empty());
    assert!(code.contains('['));
}

#[test]
fn t21b_sequence_has_name() {
    let code = gen_code(
        "sc_v9_seqn",
        &[("a", "a"), ("b", "b")],
        &[("s", vec!["a", "b"])],
        "s",
    );
    assert!(code.contains("sc_v9_seqn"));
}

#[test]
fn t22_diamond_pattern() {
    let code = gen_code(
        "sc_v9_dia",
        &[("a", "a")],
        &[
            ("left", vec!["a"]),
            ("right", vec!["a"]),
            ("s", vec!["left"]),
            ("s", vec!["right"]),
        ],
        "s",
    );
    assert!(!code.is_empty());
}

#[test]
fn t22b_diamond_has_symbol_data() {
    let code = gen_code(
        "sc_v9_dia2",
        &[("a", "a")],
        &[
            ("left", vec!["a"]),
            ("right", vec!["a"]),
            ("s", vec!["left"]),
            ("s", vec!["right"]),
        ],
        "s",
    );
    assert!(code.contains("SYMBOL") || code.contains("symbol"));
}

#[test]
fn t23_right_assoc_precedence() {
    let mut b = GrammarBuilder::new("sc_v9_rprec")
        .token("n", r"\d+")
        .token("pow", "^")
        .rule("expr", vec!["n"]);
    b = b.rule_with_precedence("expr", vec!["expr", "pow", "expr"], 1, Associativity::Right);
    b = b.start("expr");
    let mut g = b.build();
    g.normalize();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let pt = build_lr1_automaton(&g, &ff).unwrap();
    let code = StaticLanguageGenerator::new(g, pt)
        .generate_language_code()
        .to_string();
    assert!(!code.is_empty());
    assert!(code.contains("sc_v9_rprec"));
}

#[test]
fn t23b_none_assoc_precedence() {
    let mut b = GrammarBuilder::new("sc_v9_nprec")
        .token("n", r"\d+")
        .token("eq", "=")
        .rule("expr", vec!["n"]);
    b = b.rule_with_precedence("expr", vec!["expr", "eq", "expr"], 1, Associativity::None);
    b = b.start("expr");
    let mut g = b.build();
    g.normalize();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let pt = build_lr1_automaton(&g, &ff).unwrap();
    let code = StaticLanguageGenerator::new(g, pt)
        .generate_language_code()
        .to_string();
    assert!(!code.is_empty());
}

#[test]
fn t24_multiple_extras() {
    let b = GrammarBuilder::new("sc_v9_mext")
        .token("id", r"[a-z]+")
        .token("ws", r"[ \t]+")
        .token("nl", r"\n")
        .extra("ws")
        .extra("nl")
        .rule("s", vec!["id"])
        .start("s");
    let mut g = b.build();
    g.normalize();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let pt = build_lr1_automaton(&g, &ff).unwrap();
    let code = StaticLanguageGenerator::new(g, pt)
        .generate_language_code()
        .to_string();
    assert!(!code.is_empty());
    assert!(code.contains("sc_v9_mext"));
}

#[test]
fn t24b_extras_differs_from_no_extras() {
    let with_extras = extras_code();
    let without = gen_code(
        "sc_v9_noext",
        &[("id", r"[a-z]+"), ("ws", r"[ \t]+")],
        &[("s", vec!["id"])],
        "s",
    );
    // Different names guarantee different code; sizes may also differ
    assert_ne!(with_extras, without);
}

#[test]
fn t25_start_can_be_empty_flag() {
    let mut b = GrammarBuilder::new("sc_v9_empty");
    b = b.token("a", "a");
    b = b.rule("s", vec!["a"]);
    b = b.start("s");
    let mut g = b.build();
    g.normalize();
    let ff = FirstFollowSets::compute(&g).unwrap();
    let pt = build_lr1_automaton(&g, &ff).unwrap();
    let mut slg = StaticLanguageGenerator::new(g, pt);
    let code_default = slg.generate_language_code().to_string();
    slg.set_start_can_be_empty(true);
    let code_empty = slg.generate_language_code().to_string();
    // Both should produce valid, non-empty code
    assert!(!code_default.is_empty());
    assert!(!code_empty.is_empty());
}

#[test]
fn t25b_parse_table_default_generates() {
    let g = GrammarBuilder::new("sc_v9_def")
        .token("x", "x")
        .rule("s", vec!["x"])
        .start("s")
        .build();
    let slg = StaticLanguageGenerator::new(g, ParseTable::default());
    let code = slg.generate_language_code().to_string();
    assert!(!code.is_empty());
}

// =========================================================================
// Additional tests (26–30) for more coverage
// =========================================================================

#[test]
fn t26_multiple_nonterminals_code() {
    let code = gen_code(
        "sc_v9_mnt",
        &[("a", "a"), ("b", "b"), ("c", "c")],
        &[
            ("x", vec!["a"]),
            ("y", vec!["b"]),
            ("z", vec!["c"]),
            ("s", vec!["x", "y", "z"]),
        ],
        "s",
    );
    assert!(!code.is_empty());
    assert!(code.contains("sc_v9_mnt"));
}

#[test]
fn t26b_multi_nt_longer_than_single() {
    let single = gen_code("sc_v9_snt", &[("a", "a")], &[("s", vec!["a"])], "s");
    let multi = gen_code(
        "sc_v9_mnt2",
        &[("a", "a"), ("b", "b")],
        &[("x", vec!["a"]), ("s", vec!["x", "b"])],
        "s",
    );
    assert!(multi.len() > single.len());
}

#[test]
fn t27_generated_code_contains_fn_keyword() {
    let code = minimal_code();
    assert!(
        code.contains("fn"),
        "generated Rust code should contain fn keyword"
    );
}

#[test]
fn t27b_generated_code_contains_language_ref() {
    let code = medium_code();
    assert!(
        code.contains("language") || code.contains("Language") || code.contains("LANGUAGE"),
        "generated code should reference language"
    );
}

#[test]
fn t28_all_helpers_produce_different_code() {
    let m = minimal_code();
    let t = two_alt_code();
    let med = medium_code();
    let lg = large_code();
    assert_ne!(m, t);
    assert_ne!(m, med);
    assert_ne!(m, lg);
    assert_ne!(t, med);
    assert_ne!(t, lg);
    assert_ne!(med, lg);
}

#[test]
fn t29_parentheses_in_code() {
    let code = minimal_code();
    assert!(code.contains('(') && code.contains(')'));
}

#[test]
fn t29b_code_has_colons() {
    // Type annotations use colons
    let code = minimal_code();
    assert!(code.contains(':'));
}

#[test]
fn t30_recursive_longer_than_flat() {
    let flat = gen_code("sc_v9_flat", &[("a", "a")], &[("s", vec!["a"])], "s");
    let rec = recursive_code();
    assert!(rec.len() > flat.len());
}

#[test]
fn t30b_precedence_longer_than_flat() {
    let flat = gen_code("sc_v9_fl2", &[("n", r"\d+")], &[("s", vec!["n"])], "s");
    let prec = precedence_code();
    assert!(prec.len() > flat.len());
}
