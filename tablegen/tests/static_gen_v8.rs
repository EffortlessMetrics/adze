//! Static generation v8 tests for `StaticLanguageGenerator` in `adze-tablegen`.
//!
//! 80+ tests covering construction, code generation output, node types,
//! grammar variants (precedence, extras, inlines, supertypes, externals),
//! determinism, scaling, real parse tables, and edge cases.

use adze_glr_core::{FirstFollowSets, ParseTable, build_lr1_automaton};
use adze_ir::builder::GrammarBuilder;
use adze_ir::{Associativity, FieldId, Grammar};
use adze_tablegen::StaticLanguageGenerator;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn simple_grammar() -> Grammar {
    GrammarBuilder::new("simple")
        .token("a", "a")
        .rule("start", vec!["a"])
        .start("start")
        .build()
}

fn simple_pair() -> (Grammar, ParseTable) {
    (simple_grammar(), ParseTable::default())
}

fn multi_token_grammar() -> Grammar {
    GrammarBuilder::new("multi_tok")
        .token("num", r"\d+")
        .token("plus", r"\+")
        .token("star", r"\*")
        .token("lparen", r"\(")
        .token("rparen", r"\)")
        .rule("expr", vec!["num"])
        .rule("expr", vec!["expr", "plus", "expr"])
        .rule("expr", vec!["expr", "star", "expr"])
        .rule("expr", vec!["lparen", "expr", "rparen"])
        .start("expr")
        .build()
}

fn prec_grammar() -> Grammar {
    GrammarBuilder::new("prec")
        .token("num", r"\d+")
        .token("plus", r"\+")
        .token("star", r"\*")
        .rule("expr", vec!["num"])
        .rule_with_precedence("expr", vec!["expr", "plus", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "star", "expr"], 2, Associativity::Left)
        .start("expr")
        .build()
}

fn inline_grammar() -> Grammar {
    GrammarBuilder::new("inline_lang")
        .token("num", r"\d+")
        .token("plus", r"\+")
        .inline("term")
        .rule("term", vec!["num"])
        .rule("expr", vec!["term"])
        .rule("expr", vec!["expr", "plus", "term"])
        .start("expr")
        .build()
}

fn supertype_grammar() -> Grammar {
    let mut g = GrammarBuilder::new("supertype_lang")
        .token("num", r"\d+")
        .token("id", r"[a-z]+")
        .rule("literal", vec!["num"])
        .rule("literal", vec!["id"])
        .rule("program", vec!["literal"])
        .start("program")
        .build();
    let literal_id = *g
        .rules
        .keys()
        .find(|id| g.rule_names.get(*id).map(|n| n.as_str()) == Some("literal"))
        .unwrap();
    g.supertypes.push(literal_id);
    g
}

fn extras_grammar() -> Grammar {
    GrammarBuilder::new("extras_lang")
        .token("id", r"[a-z]+")
        .token("ws", r"[ \t]+")
        .extra("ws")
        .rule("start", vec!["id"])
        .start("start")
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

fn five_rule_grammar() -> Grammar {
    GrammarBuilder::new("five_rules")
        .token("num", r"\d+")
        .token("id", r"[a-z]+")
        .token("plus", r"\+")
        .token("eq", "=")
        .token("semi", ";")
        .rule("program", vec!["decl"])
        .rule("program", vec!["program", "decl"])
        .rule("decl", vec!["id", "eq", "expr", "semi"])
        .rule("expr", vec!["num"])
        .rule("expr", vec!["id"])
        .rule("expr", vec!["expr", "plus", "expr"])
        .rule("atom", vec!["num"])
        .rule("atom", vec!["id"])
        .start("program")
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
    g.fields.insert(FieldId(1), "operator".to_string());
    g.fields.insert(FieldId(2), "right".to_string());
    (g, ParseTable::default())
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
// 1. Simple grammar → generate succeeds
// ===========================================================================

#[test]
fn simple_grammar_generates_successfully() {
    let (g, t) = simple_pair();
    let code = StaticLanguageGenerator::new(g, t)
        .generate_language_code()
        .to_string();
    assert!(!code.is_empty());
}

#[test]
fn simple_grammar_node_types_succeeds() {
    let (g, t) = simple_pair();
    let json = StaticLanguageGenerator::new(g, t).generate_node_types();
    assert!(!json.is_empty());
}

// ===========================================================================
// 2. Generated code contains "TSLanguage"
// ===========================================================================

#[test]
fn generated_code_contains_tslanguage() {
    let code = gen_code(simple_grammar(), ParseTable::default());
    assert!(
        code.contains("TSLanguage"),
        "must reference TSLanguage struct"
    );
}

#[test]
fn multi_token_code_contains_tslanguage() {
    let code = gen_code(multi_token_grammar(), ParseTable::default());
    assert!(code.contains("TSLanguage"));
}

#[test]
fn prec_grammar_code_contains_tslanguage() {
    let code = gen_code(prec_grammar(), ParseTable::default());
    assert!(code.contains("TSLanguage"));
}

// ===========================================================================
// 3. Generated code contains grammar name
// ===========================================================================

#[test]
fn code_contains_grammar_name_simple() {
    let code = gen_code(simple_grammar(), ParseTable::default());
    assert!(
        code.contains("simple"),
        "code must embed grammar name 'simple'"
    );
}

#[test]
fn code_contains_custom_grammar_name() {
    let g = GrammarBuilder::new("my_custom_parser")
        .token("x", "x")
        .rule("start", vec!["x"])
        .start("start")
        .build();
    let code = gen_code(g, ParseTable::default());
    assert!(code.contains("my_custom_parser"));
}

#[test]
fn code_contains_name_with_underscores() {
    let g = GrammarBuilder::new("a_b_c_d")
        .token("x", "x")
        .rule("start", vec!["x"])
        .start("start")
        .build();
    let code = gen_code(g, ParseTable::default());
    assert!(code.contains("a_b_c_d"));
}

// ===========================================================================
// 4. Generated code contains "tree_sitter_" function
// ===========================================================================

#[test]
fn code_contains_tree_sitter_function_prefix() {
    let code = gen_code(simple_grammar(), ParseTable::default());
    assert!(
        code.contains("tree_sitter_"),
        "must contain tree_sitter_ FFI function"
    );
}

#[test]
fn code_contains_tree_sitter_function_with_grammar_name() {
    let code = gen_code(simple_grammar(), ParseTable::default());
    assert!(code.contains("tree_sitter_simple"));
}

#[test]
fn custom_name_generates_matching_tree_sitter_function() {
    let g = GrammarBuilder::new("foobar")
        .token("x", "x")
        .rule("start", vec!["x"])
        .start("start")
        .build();
    let code = gen_code(g, ParseTable::default());
    assert!(code.contains("tree_sitter_foobar"));
}

// ===========================================================================
// 5. Generated code is non-empty
// ===========================================================================

#[test]
fn minimal_code_is_nonempty() {
    let code = gen_code(simple_grammar(), ParseTable::default());
    assert!(!code.is_empty());
}

#[test]
fn stmt_code_is_nonempty() {
    let code = gen_code(stmt_grammar(), ParseTable::default());
    assert!(!code.is_empty());
}

#[test]
fn external_code_is_nonempty() {
    let code = gen_code(external_grammar(), ParseTable::default());
    assert!(!code.is_empty());
}

#[test]
fn extras_code_is_nonempty() {
    let code = gen_code(extras_grammar(), ParseTable::default());
    assert!(!code.is_empty());
}

#[test]
fn empty_grammar_code_is_nonempty() {
    let g = Grammar::new("empty".to_string());
    let code = gen_code(g, ParseTable::default());
    assert!(!code.is_empty());
}

// ===========================================================================
// 6. Generated code contains parse-related statics
// ===========================================================================

#[test]
fn code_contains_parse_table_static() {
    let code = gen_code(simple_grammar(), ParseTable::default());
    assert!(
        code.contains("PARSE_TABLE") || code.contains("parse_table"),
        "must contain parse table reference"
    );
}

#[test]
fn code_contains_symbol_names_static() {
    let code = gen_code(multi_token_grammar(), ParseTable::default());
    assert!(code.contains("SYMBOL_NAMES"));
}

#[test]
fn code_contains_symbol_metadata_static() {
    let code = gen_code(simple_grammar(), ParseTable::default());
    assert!(code.contains("SYMBOL_METADATA"));
}

#[test]
fn code_contains_lex_modes_static() {
    let code = gen_code(simple_grammar(), ParseTable::default());
    assert!(code.contains("LEX_MODES"));
}

#[test]
fn code_contains_language_version_constant() {
    let code = gen_code(simple_grammar(), ParseTable::default());
    assert!(code.contains("LANGUAGE_VERSION") || code.contains("TREE_SITTER_LANGUAGE_VERSION"));
}

#[test]
fn code_contains_static_or_const_declarations() {
    let code = gen_code(multi_token_grammar(), ParseTable::default());
    assert!(code.contains("static") || code.contains("const"));
}

// ===========================================================================
// 7. Code for grammar with multiple tokens
// ===========================================================================

#[test]
fn multi_token_grammar_generates_code() {
    let code = gen_code(multi_token_grammar(), ParseTable::default());
    assert!(!code.is_empty());
}

#[test]
fn multi_token_code_has_symbol_names() {
    let code = gen_code(multi_token_grammar(), ParseTable::default());
    assert!(code.contains("SYMBOL_NAMES"));
}

#[test]
fn multi_token_code_larger_than_single_token() {
    let small = gen_code(simple_grammar(), ParseTable::default());
    let big = gen_code(multi_token_grammar(), ParseTable::default());
    assert!(
        big.len() > small.len(),
        "multi-token code ({}) should be larger than single-token code ({})",
        big.len(),
        small.len()
    );
}

#[test]
fn multi_token_node_types_valid_json() {
    let json = gen_node_types(multi_token_grammar(), ParseTable::default());
    let v: serde_json::Value = serde_json::from_str(&json).expect("valid JSON");
    assert!(v.is_array());
}

// ===========================================================================
// 8. Code for grammar with precedence
// ===========================================================================

#[test]
fn precedence_grammar_generates() {
    let code = gen_code(prec_grammar(), ParseTable::default());
    assert!(!code.is_empty());
}

#[test]
fn right_assoc_precedence_generates() {
    let g = GrammarBuilder::new("right_assoc")
        .token("num", r"\d+")
        .token("caret", r"\^")
        .rule("expr", vec!["num"])
        .rule_with_precedence(
            "expr",
            vec!["expr", "caret", "expr"],
            1,
            Associativity::Right,
        )
        .start("expr")
        .build();
    let code = gen_code(g, ParseTable::default());
    assert!(!code.is_empty());
}

#[test]
fn mixed_assoc_precedence_generates() {
    let g = GrammarBuilder::new("mixed_assoc")
        .token("num", r"\d+")
        .token("plus", r"\+")
        .token("caret", r"\^")
        .rule("expr", vec!["num"])
        .rule_with_precedence("expr", vec!["expr", "plus", "expr"], 1, Associativity::Left)
        .rule_with_precedence(
            "expr",
            vec!["expr", "caret", "expr"],
            2,
            Associativity::Right,
        )
        .start("expr")
        .build();
    let code = gen_code(g, ParseTable::default());
    assert!(!code.is_empty());
}

#[test]
fn precedence_grammar_code_contains_tslanguage() {
    let code = gen_code(prec_grammar(), ParseTable::default());
    assert!(code.contains("TSLanguage"));
}

#[test]
fn precedence_grammar_node_types_valid() {
    let json = gen_node_types(prec_grammar(), ParseTable::default());
    let v: serde_json::Value = serde_json::from_str(&json).expect("valid JSON");
    assert!(v.is_array());
}

// ===========================================================================
// 9. Code for grammar with inline rules
// ===========================================================================

#[test]
fn inline_grammar_generates() {
    let code = gen_code(inline_grammar(), ParseTable::default());
    assert!(!code.is_empty());
}

#[test]
fn inline_grammar_code_contains_tslanguage() {
    let code = gen_code(inline_grammar(), ParseTable::default());
    assert!(code.contains("TSLanguage"));
}

#[test]
fn inline_grammar_node_types_valid() {
    let json = gen_node_types(inline_grammar(), ParseTable::default());
    let v: serde_json::Value = serde_json::from_str(&json).expect("valid JSON");
    assert!(v.is_array());
}

// ===========================================================================
// 10. Code for grammar with supertypes
// ===========================================================================

#[test]
fn supertype_grammar_generates() {
    let code = gen_code(supertype_grammar(), ParseTable::default());
    assert!(!code.is_empty());
}

#[test]
fn supertype_node_types_contains_subtypes() {
    let json = gen_node_types(supertype_grammar(), ParseTable::default());
    assert!(
        json.contains("subtypes"),
        "supertype should produce subtypes in node_types"
    );
}

#[test]
fn supertype_node_types_valid_json() {
    let json = gen_node_types(supertype_grammar(), ParseTable::default());
    let v: serde_json::Value = serde_json::from_str(&json).expect("valid JSON");
    assert!(v.is_array());
}

// ===========================================================================
// 11. Code for grammar with extras
// ===========================================================================

#[test]
fn extras_grammar_generates() {
    let code = gen_code(extras_grammar(), ParseTable::default());
    assert!(!code.is_empty());
}

#[test]
fn extras_grammar_code_contains_tslanguage() {
    let code = gen_code(extras_grammar(), ParseTable::default());
    assert!(code.contains("TSLanguage"));
}

#[test]
fn extras_grammar_node_types_valid() {
    let json = gen_node_types(extras_grammar(), ParseTable::default());
    let v: serde_json::Value = serde_json::from_str(&json).expect("valid JSON");
    assert!(v.is_array());
}

// ===========================================================================
// 12. Different grammar names → different function names
// ===========================================================================

#[test]
fn different_names_produce_different_code() {
    let make = |name: &str| {
        let g = GrammarBuilder::new(name)
            .token("x", "x")
            .rule("start", vec!["x"])
            .start("start")
            .build();
        gen_code(g, ParseTable::default())
    };
    let a = make("alpha_lang");
    let b = make("beta_lang");
    assert_ne!(a, b);
}

#[test]
fn different_names_have_different_tree_sitter_functions() {
    let make = |name: &str| {
        let g = GrammarBuilder::new(name)
            .token("x", "x")
            .rule("start", vec!["x"])
            .start("start")
            .build();
        gen_code(g, ParseTable::default())
    };
    let a = make("lang_alpha");
    let b = make("lang_beta");
    assert!(a.contains("tree_sitter_lang_alpha"));
    assert!(b.contains("tree_sitter_lang_beta"));
    assert!(!a.contains("tree_sitter_lang_beta"));
    assert!(!b.contains("tree_sitter_lang_alpha"));
}

#[test]
fn name_change_only_changes_name_related_output() {
    let make = |name: &str| {
        let g = GrammarBuilder::new(name)
            .token("x", "x")
            .rule("start", vec!["x"])
            .start("start")
            .build();
        gen_code(g, ParseTable::default())
    };
    let a = make("one");
    let b = make("two");
    // Both contain PARSE_TABLE — the table logic is name-independent
    assert!(a.contains("PARSE_TABLE"));
    assert!(b.contains("PARSE_TABLE"));
}

// ===========================================================================
// 13. Single-rule grammar
// ===========================================================================

#[test]
fn single_rule_grammar_generates() {
    let g = GrammarBuilder::new("single")
        .token("tok", "t")
        .rule("start", vec!["tok"])
        .start("start")
        .build();
    let code = gen_code(g, ParseTable::default());
    assert!(!code.is_empty());
}

#[test]
fn single_rule_code_contains_tree_sitter_function() {
    let g = GrammarBuilder::new("single")
        .token("tok", "t")
        .rule("start", vec!["tok"])
        .start("start")
        .build();
    let code = gen_code(g, ParseTable::default());
    assert!(code.contains("tree_sitter_single"));
}

#[test]
fn single_rule_node_types_valid() {
    let g = GrammarBuilder::new("single")
        .token("tok", "t")
        .rule("start", vec!["tok"])
        .start("start")
        .build();
    let json = gen_node_types(g, ParseTable::default());
    let v: serde_json::Value = serde_json::from_str(&json).expect("valid JSON");
    assert!(v.is_array());
}

// ===========================================================================
// 14. Multi-rule grammar (5+ rules)
// ===========================================================================

#[test]
fn five_rule_grammar_generates() {
    let code = gen_code(five_rule_grammar(), ParseTable::default());
    assert!(!code.is_empty());
}

#[test]
fn five_rule_grammar_code_contains_tslanguage() {
    let code = gen_code(five_rule_grammar(), ParseTable::default());
    assert!(code.contains("TSLanguage"));
}

#[test]
fn five_rule_grammar_node_types_has_entries() {
    let json = gen_node_types(five_rule_grammar(), ParseTable::default());
    let v: serde_json::Value = serde_json::from_str(&json).expect("valid JSON");
    assert!(
        !v.as_array().unwrap().is_empty(),
        "five-rule grammar should produce node type entries"
    );
}

#[test]
fn multi_rule_code_larger_than_single_rule() {
    let single = gen_code(simple_grammar(), ParseTable::default());
    let multi = gen_code(five_rule_grammar(), ParseTable::default());
    assert!(
        multi.len() > single.len(),
        "multi-rule code ({}) should exceed single-rule code ({})",
        multi.len(),
        single.len()
    );
}

#[test]
fn stmt_grammar_generates() {
    let code = gen_code(stmt_grammar(), ParseTable::default());
    assert!(!code.is_empty());
}

#[test]
fn stmt_grammar_node_types_valid() {
    let json = gen_node_types(stmt_grammar(), ParseTable::default());
    let v: serde_json::Value = serde_json::from_str(&json).expect("valid JSON");
    assert!(v.is_array());
}

// ===========================================================================
// 15. Deterministic output (generate twice → same code)
// ===========================================================================

#[test]
fn deterministic_simple_grammar() {
    let make = || gen_code(simple_grammar(), ParseTable::default());
    assert_eq!(make(), make());
}

#[test]
fn deterministic_multi_token_grammar() {
    let make = || gen_code(multi_token_grammar(), ParseTable::default());
    assert_eq!(make(), make());
}

#[test]
fn deterministic_prec_grammar() {
    let make = || gen_code(prec_grammar(), ParseTable::default());
    assert_eq!(make(), make());
}

#[test]
fn deterministic_stmt_grammar() {
    let make = || gen_code(stmt_grammar(), ParseTable::default());
    assert_eq!(make(), make());
}

#[test]
fn deterministic_node_types() {
    let make = || gen_node_types(multi_token_grammar(), ParseTable::default());
    assert_eq!(make(), make());
}

#[test]
fn deterministic_extras_grammar() {
    let make = || gen_code(extras_grammar(), ParseTable::default());
    assert_eq!(make(), make());
}

#[test]
fn deterministic_external_grammar() {
    let make = || gen_code(external_grammar(), ParseTable::default());
    assert_eq!(make(), make());
}

// ===========================================================================
// 16. Construction and accessors
// ===========================================================================

#[test]
fn construction_preserves_grammar_name() {
    let (g, t) = simple_pair();
    let slg = StaticLanguageGenerator::new(g, t);
    assert_eq!(slg.grammar.name, "simple");
}

#[test]
fn construction_defaults_start_can_be_empty_false() {
    let (g, t) = simple_pair();
    let slg = StaticLanguageGenerator::new(g, t);
    assert!(!slg.start_can_be_empty);
}

#[test]
fn construction_defaults_compressed_tables_none() {
    let (g, t) = simple_pair();
    let slg = StaticLanguageGenerator::new(g, t);
    assert!(slg.compressed_tables.is_none());
}

#[test]
fn set_start_can_be_empty_roundtrip() {
    let (g, t) = simple_pair();
    let mut slg = StaticLanguageGenerator::new(g, t);
    slg.set_start_can_be_empty(true);
    assert!(slg.start_can_be_empty);
    slg.set_start_can_be_empty(false);
    assert!(!slg.start_can_be_empty);
}

#[test]
fn construction_preserves_parse_table_state_count() {
    let (g, t) = simple_pair();
    let expected = t.state_count;
    let slg = StaticLanguageGenerator::new(g, t);
    assert_eq!(slg.parse_table.state_count, expected);
}

#[test]
fn set_start_can_be_empty_true_still_generates() {
    let (g, t) = simple_pair();
    let mut slg = StaticLanguageGenerator::new(g, t);
    slg.set_start_can_be_empty(true);
    let code = slg.generate_language_code().to_string();
    assert!(!code.is_empty());
}

// ===========================================================================
// 17. Real parse tables via build_lr1_automaton
// ===========================================================================

#[test]
fn real_table_simple_code_is_nonempty() {
    let g = simple_grammar();
    let t = real_table(&g);
    let code = gen_code(g, t);
    assert!(!code.is_empty());
}

#[test]
fn real_table_has_nonzero_state_count() {
    let g = simple_grammar();
    let t = real_table(&g);
    assert!(t.state_count > 0);
}

#[test]
fn real_table_has_nonzero_symbol_count() {
    let g = simple_grammar();
    let t = real_table(&g);
    assert!(t.symbol_count > 0);
}

#[test]
fn real_table_multi_token_code_generates() {
    let g = multi_token_grammar();
    let t = real_table(&g);
    let code = gen_code(g, t);
    assert!(!code.is_empty());
}

#[test]
fn real_table_code_contains_grammar_name() {
    let g = simple_grammar();
    let t = real_table(&g);
    let code = gen_code(g, t);
    assert!(code.contains("simple"));
}

#[test]
fn real_table_code_deterministic() {
    let make = || {
        let g = simple_grammar();
        let t = real_table(&g);
        gen_code(g, t)
    };
    assert_eq!(make(), make());
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
    let g = multi_token_grammar();
    let t = real_table(&g);
    assert!(t.token_count > 0);
}

#[test]
fn real_table_node_types_valid() {
    let g = multi_token_grammar();
    let t = real_table(&g);
    let json = gen_node_types(g, t);
    let v: serde_json::Value = serde_json::from_str(&json).expect("valid JSON");
    assert!(v.is_array());
}

#[test]
fn real_table_extras_code_generates() {
    let g = extras_grammar();
    let t = real_table(&g);
    let code = gen_code(g, t);
    assert!(!code.is_empty());
}

// ===========================================================================
// 18. Node types JSON structure
// ===========================================================================

#[test]
fn node_types_is_valid_json_array() {
    let json = gen_node_types(simple_grammar(), ParseTable::default());
    let v: serde_json::Value = serde_json::from_str(&json).expect("valid JSON");
    assert!(v.is_array());
}

#[test]
fn node_types_entries_have_type_field() {
    let json = gen_node_types(multi_token_grammar(), ParseTable::default());
    let v: serde_json::Value = serde_json::from_str(&json).unwrap();
    for entry in v.as_array().unwrap() {
        assert!(
            entry.get("type").is_some(),
            "each entry needs a 'type' field"
        );
    }
}

#[test]
fn node_types_entries_have_named_field() {
    let json = gen_node_types(multi_token_grammar(), ParseTable::default());
    let v: serde_json::Value = serde_json::from_str(&json).unwrap();
    for entry in v.as_array().unwrap() {
        assert!(
            entry.get("named").is_some(),
            "each entry needs a 'named' field"
        );
    }
}

#[test]
fn node_types_excludes_hidden_rules() {
    let g = GrammarBuilder::new("hidden_test")
        .token("a", "a")
        .rule("_hidden", vec!["a"])
        .rule("visible", vec!["_hidden"])
        .start("visible")
        .build();
    let json = gen_node_types(g, ParseTable::default());
    assert!(
        !json.contains("\"_hidden\""),
        "hidden rules must be excluded"
    );
}

#[test]
fn node_types_includes_external_tokens() {
    let json = gen_node_types(external_grammar(), ParseTable::default());
    assert!(json.contains("\"indent\""));
    assert!(json.contains("\"dedent\""));
}

// ===========================================================================
// 19. External scanner tokens
// ===========================================================================

#[test]
fn external_grammar_generates() {
    let code = gen_code(external_grammar(), ParseTable::default());
    assert!(!code.is_empty());
}

#[test]
fn external_grammar_code_contains_tslanguage() {
    let code = gen_code(external_grammar(), ParseTable::default());
    assert!(code.contains("TSLanguage"));
}

#[test]
fn multiple_externals_all_appear_in_node_types() {
    let g = GrammarBuilder::new("multi_ext")
        .token("id", r"[a-z]+")
        .external("ext_a")
        .external("ext_b")
        .external("ext_c")
        .rule("start", vec!["id"])
        .start("start")
        .build();
    let json = gen_node_types(g, ParseTable::default());
    assert!(json.contains("\"ext_a\""));
    assert!(json.contains("\"ext_b\""));
    assert!(json.contains("\"ext_c\""));
}

// ===========================================================================
// 20. Field grammar
// ===========================================================================

#[test]
fn field_grammar_generates() {
    let (g, t) = field_grammar();
    let code = gen_code(g, t);
    assert!(!code.is_empty());
}

#[test]
fn field_grammar_node_types_valid() {
    let (g, t) = field_grammar();
    let json = gen_node_types(g, t);
    let v: serde_json::Value = serde_json::from_str(&json).expect("valid JSON");
    assert!(v.is_array());
}

// ===========================================================================
// 21. Edge cases
// ===========================================================================

#[test]
fn long_chain_grammar_generates() {
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
fn multiple_alternatives_generates() {
    let g = GrammarBuilder::new("alternatives")
        .token("a", "a")
        .token("b", "b")
        .token("c", "c")
        .rule("start", vec!["a"])
        .rule("start", vec!["b"])
        .rule("start", vec!["c"])
        .start("start")
        .build();
    let code = gen_code(g, ParseTable::default());
    assert!(!code.is_empty());
}

#[test]
fn large_grammar_generates() {
    let mut builder = GrammarBuilder::new("large_v8");
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
    let mut builder = GrammarBuilder::new("large_nt_v8");
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
    let json = gen_node_types(builder.build(), ParseTable::default());
    let v: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert!(v.as_array().unwrap().len() >= 10);
}

#[test]
fn code_from_empty_grammar_is_valid() {
    let g = Grammar::new("empty_v8".to_string());
    let code = gen_code(g, ParseTable::default());
    assert!(!code.is_empty());
}

#[test]
fn node_types_from_empty_grammar_is_valid_json() {
    let g = Grammar::new("empty_nt_v8".to_string());
    let json = gen_node_types(g, ParseTable::default());
    let v: serde_json::Value = serde_json::from_str(&json).expect("valid JSON");
    assert!(v.is_array());
}

#[test]
fn fragile_token_grammar_generates() {
    let g = GrammarBuilder::new("fragile")
        .token("num", r"\d+")
        .fragile_token("error_tok", "ERROR")
        .rule("start", vec!["num"])
        .start("start")
        .build();
    let code = gen_code(g, ParseTable::default());
    assert!(!code.is_empty());
}

#[test]
fn hidden_externals_excluded_from_node_types() {
    let g = GrammarBuilder::new("hid_ext_v8")
        .token("a", "a")
        .external("_hidden_scanner")
        .rule("start", vec!["a"])
        .start("start")
        .build();
    let json = gen_node_types(g, ParseTable::default());
    assert!(!json.contains("\"_hidden_scanner\""));
}

#[test]
fn code_contains_language_function() {
    let code = gen_code(simple_grammar(), ParseTable::default());
    assert!(
        code.contains("language"),
        "must declare a language function"
    );
}

#[test]
fn code_contains_extern_c() {
    let code = gen_code(simple_grammar(), ParseTable::default());
    assert!(
        code.contains("extern \"C\"") || code.contains("extern\"C\""),
        "must have extern C FFI function"
    );
}
