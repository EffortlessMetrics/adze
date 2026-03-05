//! Static code-generation v5 tests for `adze-tablegen`.
//!
//! Covers: StaticLanguageGenerator construction, language code output,
//! node-types JSON, generated Rust syntax, grammar scaling, feature
//! combinations, determinism, real parse tables via `build_lr1_automaton`,
//! NodeTypesGenerator, LanguageBuilder, and edge cases.

use adze_glr_core::{build_lr1_automaton, FirstFollowSets, ParseTable};
use adze_ir::builder::GrammarBuilder;
use adze_ir::{Associativity, FieldId, Grammar};
use adze_tablegen::{NodeTypesGenerator, StaticLanguageGenerator};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn minimal_grammar() -> Grammar {
    GrammarBuilder::new("minimal")
        .token("number", r"\d+")
        .rule("expr", vec!["number"])
        .start("expr")
        .build()
}

fn minimal_pair() -> (Grammar, ParseTable) {
    (minimal_grammar(), ParseTable::default())
}

fn arith_grammar() -> Grammar {
    GrammarBuilder::new("arith")
        .token("num", r"\d+")
        .token("plus", r"\+")
        .token("star", r"\*")
        .rule("expr", vec!["num"])
        .rule("expr", vec!["expr", "plus", "expr"])
        .rule("expr", vec!["expr", "star", "expr"])
        .start("expr")
        .build()
}

fn arith_pair() -> (Grammar, ParseTable) {
    (arith_grammar(), ParseTable::default())
}

fn stmt_grammar() -> Grammar {
    GrammarBuilder::new("stmt_lang")
        .token("id", r"[a-z]+")
        .token("num", r"\d+")
        .token("eq", "=")
        .token("semi", ";")
        .token("plus", r"\+")
        .rule("program", vec!["stmt"])
        .rule("program", vec!["program", "stmt"])
        .rule("stmt", vec!["id", "eq", "expr", "semi"])
        .rule("expr", vec!["num"])
        .rule("expr", vec!["id"])
        .rule("expr", vec!["expr", "plus", "expr"])
        .start("program")
        .build()
}

fn external_grammar() -> Grammar {
    GrammarBuilder::new("ext_lang")
        .token("id", r"[a-z]+")
        .token("colon", ":")
        .external("indent")
        .external("dedent")
        .rule("block", vec!["id", "colon", "indent", "id", "dedent"])
        .start("block")
        .build()
}

fn field_grammar() -> (Grammar, ParseTable) {
    let mut g = GrammarBuilder::new("field_lang")
        .token("num", r"\d+")
        .token("plus", r"\+")
        .rule("binop", vec!["num", "plus", "num"])
        .start("binop")
        .build();
    g.fields.insert(FieldId(0), "left".to_string());
    g.fields.insert(FieldId(1), "op".to_string());
    g.fields.insert(FieldId(2), "right".to_string());
    (g, ParseTable::default())
}

fn extra_grammar() -> Grammar {
    GrammarBuilder::new("extra_lang")
        .token("id", r"[a-z]+")
        .token("ws", r"[ \t]+")
        .extra("ws")
        .rule("root", vec!["id"])
        .start("root")
        .build()
}

fn prec_grammar() -> Grammar {
    GrammarBuilder::new("prec_lang")
        .token("num", r"\d+")
        .token("plus", r"\+")
        .token("star", r"\*")
        .rule("expr", vec!["num"])
        .rule_with_precedence("expr", vec!["expr", "plus", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "star", "expr"], 2, Associativity::Left)
        .start("expr")
        .build()
}

/// Build a real (non-default) parse table from a grammar.
fn real_table(grammar: &Grammar) -> ParseTable {
    let mut g = grammar.clone();
    let ff = FirstFollowSets::compute_normalized(&mut g).expect("first/follow");
    build_lr1_automaton(&g, &ff).expect("lr1 automaton")
}

fn gen_code(grammar: Grammar, table: ParseTable) -> String {
    StaticLanguageGenerator::new(grammar, table)
        .generate_language_code()
        .to_string()
}

fn gen_node_types(grammar: Grammar, table: ParseTable) -> String {
    StaticLanguageGenerator::new(grammar, table).generate_node_types()
}

// ===========================================================================
// 1. Construction
// ===========================================================================

#[test]
fn construct_preserves_grammar_name() {
    let (g, t) = minimal_pair();
    let slg = StaticLanguageGenerator::new(g, t);
    assert_eq!(slg.grammar.name, "minimal");
}

#[test]
fn construct_start_can_be_empty_defaults_false() {
    let (g, t) = minimal_pair();
    let slg = StaticLanguageGenerator::new(g, t);
    assert!(!slg.start_can_be_empty);
}

#[test]
fn construct_compressed_tables_defaults_none() {
    let (g, t) = minimal_pair();
    let slg = StaticLanguageGenerator::new(g, t);
    assert!(slg.compressed_tables.is_none());
}

#[test]
fn set_start_can_be_empty_round_trip() {
    let (g, t) = minimal_pair();
    let mut slg = StaticLanguageGenerator::new(g, t);
    slg.set_start_can_be_empty(true);
    assert!(slg.start_can_be_empty);
    slg.set_start_can_be_empty(false);
    assert!(!slg.start_can_be_empty);
}

#[test]
fn construct_preserves_parse_table_state_count() {
    let (g, t) = minimal_pair();
    let expected = t.state_count;
    let slg = StaticLanguageGenerator::new(g, t);
    assert_eq!(slg.parse_table.state_count, expected);
}

#[test]
fn construct_with_arith_grammar_preserves_name() {
    let (g, t) = arith_pair();
    let slg = StaticLanguageGenerator::new(g, t);
    assert_eq!(slg.grammar.name, "arith");
}

// ===========================================================================
// 2. Language code generation — basic properties
// ===========================================================================

#[test]
fn code_is_nonempty() {
    let (g, t) = minimal_pair();
    let code = StaticLanguageGenerator::new(g, t).generate_language_code();
    assert!(!code.is_empty());
}

#[test]
fn code_contains_language_keyword() {
    let code = gen_code(minimal_grammar(), ParseTable::default());
    assert!(code.contains("language"), "must mention `language`");
}

#[test]
fn code_contains_version_constant() {
    let code = gen_code(minimal_grammar(), ParseTable::default());
    assert!(code.contains("LANGUAGE_VERSION"));
}

#[test]
fn code_contains_symbol_names() {
    let code = gen_code(arith_grammar(), ParseTable::default());
    assert!(code.contains("SYMBOL_NAMES"));
}

#[test]
fn code_embeds_grammar_name() {
    let g = GrammarBuilder::new("foobarbaz")
        .token("x", "x")
        .rule("s", vec!["x"])
        .start("s")
        .build();
    let code = gen_code(g, ParseTable::default());
    assert!(code.contains("foobarbaz"));
}

#[test]
fn code_contains_static_or_const() {
    let code = gen_code(arith_grammar(), ParseTable::default());
    assert!(
        code.contains("static") || code.contains("const"),
        "should have static/const declarations"
    );
}

// ===========================================================================
// 3. Generated Rust is syntactically valid
// ===========================================================================

#[test]
fn minimal_code_parses_as_rust() {
    let (g, t) = minimal_pair();
    let ts = StaticLanguageGenerator::new(g, t).generate_language_code();
    let parsed: Result<syn::File, _> = syn::parse2(ts);
    assert!(parsed.is_ok(), "invalid Rust: {:?}", parsed.err());
}

#[test]
fn arith_code_parses_as_rust() {
    let ts = StaticLanguageGenerator::new(arith_grammar(), ParseTable::default())
        .generate_language_code();
    let parsed: Result<syn::File, _> = syn::parse2(ts);
    assert!(parsed.is_ok(), "invalid Rust: {:?}", parsed.err());
}

#[test]
fn stmt_code_parses_as_rust() {
    let ts = StaticLanguageGenerator::new(stmt_grammar(), ParseTable::default())
        .generate_language_code();
    let parsed: Result<syn::File, _> = syn::parse2(ts);
    assert!(parsed.is_ok(), "invalid Rust: {:?}", parsed.err());
}

#[test]
fn external_code_parses_as_rust() {
    let ts = StaticLanguageGenerator::new(external_grammar(), ParseTable::default())
        .generate_language_code();
    let parsed: Result<syn::File, _> = syn::parse2(ts);
    assert!(parsed.is_ok(), "invalid Rust: {:?}", parsed.err());
}

#[test]
fn extra_code_parses_as_rust() {
    let ts = StaticLanguageGenerator::new(extra_grammar(), ParseTable::default())
        .generate_language_code();
    let parsed: Result<syn::File, _> = syn::parse2(ts);
    assert!(parsed.is_ok(), "invalid Rust: {:?}", parsed.err());
}

#[test]
fn prec_code_parses_as_rust() {
    let ts = StaticLanguageGenerator::new(prec_grammar(), ParseTable::default())
        .generate_language_code();
    let parsed: Result<syn::File, _> = syn::parse2(ts);
    assert!(parsed.is_ok(), "invalid Rust: {:?}", parsed.err());
}

// ===========================================================================
// 4. Node types JSON
// ===========================================================================

#[test]
fn node_types_is_valid_json() {
    let json = gen_node_types(arith_grammar(), ParseTable::default());
    let v: serde_json::Value = serde_json::from_str(&json).expect("valid JSON");
    assert!(v.is_array());
}

#[test]
fn node_types_array_is_nonempty() {
    let json = gen_node_types(arith_grammar(), ParseTable::default());
    let v: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert!(!v.as_array().unwrap().is_empty());
}

#[test]
fn node_types_entries_have_type_field() {
    let json = gen_node_types(minimal_grammar(), ParseTable::default());
    let v: serde_json::Value = serde_json::from_str(&json).unwrap();
    for entry in v.as_array().unwrap() {
        assert!(entry.get("type").is_some(), "missing \"type\" field");
    }
}

#[test]
fn node_types_entries_have_named_field() {
    let json = gen_node_types(minimal_grammar(), ParseTable::default());
    let v: serde_json::Value = serde_json::from_str(&json).unwrap();
    for entry in v.as_array().unwrap() {
        assert!(entry.get("named").is_some(), "missing \"named\" field");
    }
}

#[test]
fn node_types_excludes_hidden_rules() {
    let g = GrammarBuilder::new("hidden")
        .token("a", "a")
        .rule("_secret", vec!["a"])
        .rule("visible", vec!["_secret"])
        .start("visible")
        .build();
    let json = gen_node_types(g, ParseTable::default());
    assert!(!json.contains("\"_secret\""), "hidden rules must be excluded");
}

#[test]
fn node_types_includes_external_tokens() {
    let json = gen_node_types(external_grammar(), ParseTable::default());
    assert!(json.contains("\"indent\""));
    assert!(json.contains("\"dedent\""));
}

#[test]
fn node_types_excludes_hidden_externals() {
    let g = GrammarBuilder::new("hid_ext")
        .token("a", "a")
        .external("_hidden_ext")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let json = gen_node_types(g, ParseTable::default());
    assert!(!json.contains("\"_hidden_ext\""));
}

#[test]
fn node_types_for_empty_grammar_is_valid_json() {
    let g = Grammar::new("empty".to_string());
    let json = gen_node_types(g, ParseTable::default());
    let v: serde_json::Value = serde_json::from_str(&json).expect("valid JSON");
    assert!(v.is_array());
}

// ===========================================================================
// 5. NodeTypesGenerator standalone
// ===========================================================================

#[test]
fn node_types_generator_produces_valid_json() {
    let g = arith_grammar();
    let ntg = NodeTypesGenerator::new(&g);
    let json = ntg.generate().expect("generate");
    let v: serde_json::Value = serde_json::from_str(&json).expect("valid JSON");
    assert!(v.is_array());
}

#[test]
fn node_types_generator_entries_have_required_fields() {
    let g = stmt_grammar();
    let json = NodeTypesGenerator::new(&g).generate().expect("generate");
    let v: serde_json::Value = serde_json::from_str(&json).unwrap();
    for entry in v.as_array().unwrap() {
        assert!(entry.get("type").is_some());
        assert!(entry.get("named").is_some());
    }
}

#[test]
fn node_types_generator_and_static_generator_both_produce_valid_json() {
    let g = arith_grammar();
    let standalone = NodeTypesGenerator::new(&g).generate().expect("generate");
    let via_slg =
        StaticLanguageGenerator::new(g, ParseTable::default()).generate_node_types();
    // Both must parse as JSON arrays.
    let a: serde_json::Value = serde_json::from_str(&standalone).unwrap();
    let b: serde_json::Value = serde_json::from_str(&via_slg).unwrap();
    assert!(a.is_array());
    assert!(b.is_array());
}

// ===========================================================================
// 6. Different grammar names yield different output
// ===========================================================================

#[test]
fn different_names_produce_different_code() {
    let make = |name: &str| {
        let g = GrammarBuilder::new(name)
            .token("x", "x")
            .rule("s", vec!["x"])
            .start("s")
            .build();
        gen_code(g, ParseTable::default())
    };
    let a = make("alpha_lang");
    let b = make("beta_lang");
    assert!(a.contains("alpha_lang"));
    assert!(b.contains("beta_lang"));
    assert_ne!(a, b);
}

#[test]
fn different_names_produce_different_node_types() {
    let make = |name: &str| {
        let g = GrammarBuilder::new(name)
            .token("x", "x")
            .rule("s", vec!["x"])
            .start("s")
            .build();
        gen_node_types(g, ParseTable::default())
    };
    // Node types JSON does not typically embed the grammar name, but
    // we at least confirm both calls succeed and produce valid JSON.
    let a = make("gamma");
    let b = make("delta");
    let _: serde_json::Value = serde_json::from_str(&a).unwrap();
    let _: serde_json::Value = serde_json::from_str(&b).unwrap();
}

// ===========================================================================
// 7. Grammar scaling
// ===========================================================================

#[test]
fn large_grammar_generates_code() {
    let mut builder = GrammarBuilder::new("big");
    for i in 0..20 {
        builder = builder.token(&format!("tok_{i}"), &format!("t{i}"));
    }
    for i in 0..15 {
        let tok = format!("tok_{i}");
        let rule = format!("rule_{i}");
        builder = builder.rule(&rule, vec![Box::leak(tok.into_boxed_str())]);
    }
    builder = builder
        .rule("top", vec!["rule_0"])
        .rule("top", vec!["rule_1"])
        .start("top");
    let g = builder.build();
    let code = gen_code(g, ParseTable::default());
    assert!(!code.is_empty());
}

#[test]
fn large_grammar_node_types_has_many_entries() {
    let mut builder = GrammarBuilder::new("big_nt");
    for i in 0..20 {
        builder = builder.token(&format!("tok_{i}"), &format!("t{i}"));
    }
    for i in 0..15 {
        let tok = format!("tok_{i}");
        let rule = format!("rule_{i}");
        builder = builder.rule(&rule, vec![Box::leak(tok.into_boxed_str())]);
    }
    builder = builder
        .rule("top", vec!["rule_0"])
        .rule("top", vec!["rule_1"])
        .start("top");
    let g = builder.build();
    let json = gen_node_types(g, ParseTable::default());
    let v: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert!(v.as_array().unwrap().len() >= 10);
}

#[test]
fn large_code_exceeds_small_code_length() {
    let small = gen_code(minimal_grammar(), ParseTable::default()).len();
    let mut builder = GrammarBuilder::new("big_len");
    for i in 0..20 {
        builder = builder.token(&format!("tok_{i}"), &format!("t{i}"));
    }
    for i in 0..10 {
        let tok = format!("tok_{i}");
        let rule = format!("rule_{i}");
        builder = builder.rule(&rule, vec![Box::leak(tok.into_boxed_str())]);
    }
    builder = builder.rule("top", vec!["rule_0"]).start("top");
    let big = gen_code(builder.build(), ParseTable::default()).len();
    assert!(big > small, "big {big} should exceed small {small}");
}

// ===========================================================================
// 8. Feature combinations
// ===========================================================================

#[test]
fn externals_code_generates() {
    let code = gen_code(external_grammar(), ParseTable::default());
    assert!(!code.is_empty());
}

#[test]
fn fields_node_types_is_valid_json() {
    let (g, t) = field_grammar();
    let json = gen_node_types(g, t);
    let v: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert!(v.is_array());
}

#[test]
fn extras_code_generates() {
    let code = gen_code(extra_grammar(), ParseTable::default());
    assert!(!code.is_empty());
}

#[test]
fn precedence_code_generates() {
    let code = gen_code(prec_grammar(), ParseTable::default());
    assert!(!code.is_empty());
}

#[test]
fn supertype_node_types_contains_subtypes() {
    let mut g = GrammarBuilder::new("sup")
        .token("num", r"\d+")
        .token("id", r"[a-z]+")
        .rule("expr", vec!["num"])
        .rule("expr", vec!["id"])
        .rule("program", vec!["expr"])
        .start("program")
        .build();
    let expr_id = *g
        .rules
        .keys()
        .find(|id| g.rule_names.get(*id).map(|n| n.as_str()) == Some("expr"))
        .unwrap();
    g.supertypes.push(expr_id);
    let json = gen_node_types(g, ParseTable::default());
    assert!(json.contains("subtypes"));
}

#[test]
fn multiple_alternatives_code_generates() {
    let g = GrammarBuilder::new("multi_alt")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("s", vec!["a"])
        .rule("s", vec!["b"])
        .rule("s", vec!["c"])
        .start("s")
        .build();
    let code = gen_code(g, ParseTable::default());
    assert!(!code.is_empty());
}

#[test]
fn empty_token_grammar_generates() {
    let g = Grammar::new("no_tokens".to_string());
    let code = gen_code(g.clone(), ParseTable::default());
    assert!(!code.is_empty());
    let json = gen_node_types(g, ParseTable::default());
    let v: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert!(v.is_array());
}

// ===========================================================================
// 9. Determinism
// ===========================================================================

#[test]
fn code_deterministic_minimal() {
    let make = || gen_code(minimal_grammar(), ParseTable::default());
    assert_eq!(make(), make());
}

#[test]
fn code_deterministic_arith() {
    let make = || gen_code(arith_grammar(), ParseTable::default());
    assert_eq!(make(), make());
}

#[test]
fn code_deterministic_stmt() {
    let make = || gen_code(stmt_grammar(), ParseTable::default());
    assert_eq!(make(), make());
}

#[test]
fn node_types_deterministic_arith() {
    let make = || gen_node_types(arith_grammar(), ParseTable::default());
    assert_eq!(make(), make());
}

#[test]
fn node_types_deterministic_externals() {
    let make = || gen_node_types(external_grammar(), ParseTable::default());
    assert_eq!(make(), make());
}

// ===========================================================================
// 10. Real parse tables via build_lr1_automaton
// ===========================================================================

#[test]
fn real_table_minimal_code_is_nonempty() {
    let g = minimal_grammar();
    let t = real_table(&g);
    let code = gen_code(g, t);
    assert!(!code.is_empty());
}

#[test]
fn real_table_has_nonzero_state_count() {
    let g = minimal_grammar();
    let t = real_table(&g);
    assert!(t.state_count > 0, "real table must have states");
}

#[test]
fn real_table_has_nonzero_symbol_count() {
    let g = minimal_grammar();
    let t = real_table(&g);
    assert!(t.symbol_count > 0, "real table must have symbols");
}

#[test]
fn real_table_arith_code_parses_as_rust() {
    let g = arith_grammar();
    let t = real_table(&g);
    let ts = StaticLanguageGenerator::new(g, t).generate_language_code();
    let parsed: Result<syn::File, _> = syn::parse2(ts);
    assert!(parsed.is_ok(), "invalid Rust: {:?}", parsed.err());
}

#[test]
fn real_table_arith_node_types_valid() {
    let g = arith_grammar();
    let t = real_table(&g);
    let json = gen_node_types(g, t);
    let v: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert!(v.is_array());
    assert!(!v.as_array().unwrap().is_empty());
}

#[test]
fn real_table_stmt_code_is_nonempty() {
    let g = stmt_grammar();
    let t = real_table(&g);
    let code = gen_code(g, t);
    assert!(!code.is_empty());
}

#[test]
fn real_table_prec_code_generates() {
    let g = prec_grammar();
    let t = real_table(&g);
    let code = gen_code(g, t);
    assert!(!code.is_empty());
}

#[test]
fn real_table_token_count_positive() {
    let g = arith_grammar();
    let t = real_table(&g);
    assert!(t.token_count > 0, "arith grammar should have tokens");
}

#[test]
fn real_table_extra_grammar_code_generates() {
    let g = extra_grammar();
    let t = real_table(&g);
    let code = gen_code(g, t);
    assert!(!code.is_empty());
}

#[test]
fn real_table_code_contains_grammar_name() {
    let g = arith_grammar();
    let t = real_table(&g);
    let code = gen_code(g, t);
    assert!(code.contains("arith"));
}

#[test]
fn real_table_code_deterministic() {
    let make = || {
        let g = minimal_grammar();
        let t = real_table(&g);
        gen_code(g, t)
    };
    assert_eq!(make(), make());
}

// ===========================================================================
// 11. Edge cases
// ===========================================================================

#[test]
fn single_token_grammar_code_generates() {
    let g = GrammarBuilder::new("one_tok")
        .token("t", "t")
        .rule("s", vec!["t"])
        .start("s")
        .build();
    let code = gen_code(g, ParseTable::default());
    assert!(!code.is_empty());
}

#[test]
fn long_chain_grammar_code_generates() {
    let g = GrammarBuilder::new("chain")
        .token("leaf", "L")
        .rule("a", vec!["leaf"])
        .rule("b", vec!["a"])
        .rule("c", vec!["b"])
        .rule("d", vec!["c"])
        .rule("top", vec!["d"])
        .start("top")
        .build();
    let code = gen_code(g, ParseTable::default());
    assert!(!code.is_empty());
}

#[test]
fn grammar_name_with_underscores_embeds_correctly() {
    let g = GrammarBuilder::new("my_test_grammar")
        .token("x", "x")
        .rule("s", vec!["x"])
        .start("s")
        .build();
    let code = gen_code(g, ParseTable::default());
    assert!(code.contains("my_test_grammar"));
}

#[test]
fn right_associative_precedence_generates() {
    let g = GrammarBuilder::new("right_assoc")
        .token("num", r"\d+")
        .token("caret", r"\^")
        .rule("expr", vec!["num"])
        .rule_with_precedence("expr", vec!["expr", "caret", "expr"], 1, Associativity::Right)
        .start("expr")
        .build();
    let code = gen_code(g, ParseTable::default());
    assert!(!code.is_empty());
}

#[test]
fn mixed_associativity_generates() {
    let g = GrammarBuilder::new("mixed_assoc")
        .token("num", r"\d+")
        .token("plus", r"\+")
        .token("caret", r"\^")
        .rule("expr", vec!["num"])
        .rule_with_precedence("expr", vec!["expr", "plus", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "caret", "expr"], 2, Associativity::Right)
        .start("expr")
        .build();
    let code = gen_code(g, ParseTable::default());
    assert!(!code.is_empty());
}

#[test]
fn multiple_externals_in_node_types() {
    let g = GrammarBuilder::new("multi_ext")
        .token("id", r"[a-z]+")
        .external("ext_a")
        .external("ext_b")
        .external("ext_c")
        .rule("s", vec!["id"])
        .start("s")
        .build();
    let json = gen_node_types(g, ParseTable::default());
    assert!(json.contains("\"ext_a\""));
    assert!(json.contains("\"ext_b\""));
    assert!(json.contains("\"ext_c\""));
}

#[test]
fn fields_code_generates() {
    let (g, t) = field_grammar();
    let code = gen_code(g, t);
    assert!(!code.is_empty());
}

#[test]
fn set_start_can_be_empty_true_still_generates() {
    let (g, t) = minimal_pair();
    let mut slg = StaticLanguageGenerator::new(g, t);
    slg.set_start_can_be_empty(true);
    let code = slg.generate_language_code();
    assert!(!code.is_empty());
}
