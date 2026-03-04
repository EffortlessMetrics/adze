#![allow(clippy::needless_range_loop)]

//! Property-based tests for language generation in adze-tablegen.
//!
//! Tests properties of `StaticLanguageGenerator` and `NodeTypesGenerator`
//! using `adze_ir::builder::GrammarBuilder` to construct valid grammars,
//! `adze_glr_core` to build parse tables, and then verifying invariants
//! of the generated code and node types JSON.

use adze_glr_core::{FirstFollowSets, ParseTable, build_lr1_automaton};
use adze_ir::Grammar;
use adze_ir::builder::GrammarBuilder;
use adze_tablegen::StaticLanguageGenerator;
use adze_tablegen::node_types::NodeTypesGenerator;
use proptest::prelude::*;

// ---------------------------------------------------------------------------
// Strategies
// ---------------------------------------------------------------------------

/// Grammar names: [a-z]{2,8}
fn grammar_name_strategy() -> impl Strategy<Value = String> {
    "[a-z]{2,8}"
}

/// Token names: [a-z]{1,5} — must be unique, so we generate a base and append index
fn token_name_strategy() -> impl Strategy<Value = String> {
    "[a-z]{1,5}"
}

/// Number of tokens: 1..5
fn token_count_strategy() -> impl Strategy<Value = usize> {
    1usize..=5
}

/// Number of rules: 1..3
fn rule_count_strategy() -> impl Strategy<Value = usize> {
    1usize..=3
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Build a simple valid grammar with the given name, number of tokens, and number of rules.
/// Each rule is `root -> tokN` for a different token.
fn build_grammar(name: &str, num_tokens: usize, num_rules: usize) -> Grammar {
    let num_tokens = num_tokens.max(1);
    let num_rules = num_rules.max(1);
    let mut builder = GrammarBuilder::new(name);

    // Add tokens
    for i in 0..num_tokens {
        builder = builder.token(&format!("tok{i}"), &format!("pat{i}"));
    }

    // Add rules: each alternative for 'root' references a different token
    for i in 0..num_rules {
        let tok_idx = i % num_tokens;
        let tok_name = format!("tok{tok_idx}");
        builder = builder.rule("root", vec![Box::leak(tok_name.into_boxed_str())]);
    }

    builder = builder.start("root");
    builder.build()
}

/// Build a grammar and its parse table via the full pipeline.
/// Returns (grammar, parse_table) or panics if pipeline fails.
fn build_grammar_and_table(
    name: &str,
    num_tokens: usize,
    num_rules: usize,
) -> (Grammar, ParseTable) {
    let grammar = build_grammar(name, num_tokens, num_rules);
    let ff = FirstFollowSets::compute(&grammar).expect("FIRST/FOLLOW should succeed");
    let table = build_lr1_automaton(&grammar, &ff).expect("LR(1) automaton should succeed");
    (grammar, table)
}

/// Build a more complex grammar with chained non-terminals.
fn build_complex_grammar(name: &str, depth: usize) -> Grammar {
    let depth = depth.max(1).min(5);
    let mut builder = GrammarBuilder::new(name);
    builder = builder.token("leaf", "leaf");

    // chain: rule0 -> rule1 -> ... -> leaf
    for i in 0..depth {
        let lhs = format!("rule{i}");
        if i + 1 < depth {
            let rhs = format!("rule{}", i + 1);
            builder = builder.rule(
                Box::leak(lhs.into_boxed_str()),
                vec![Box::leak(rhs.into_boxed_str())],
            );
        } else {
            builder = builder.rule(Box::leak(lhs.into_boxed_str()), vec!["leaf"]);
        }
    }

    builder = builder.start("rule0");
    builder.build()
}

/// Build a complex grammar with its parse table.
fn build_complex_grammar_and_table(name: &str, depth: usize) -> (Grammar, ParseTable) {
    let grammar = build_complex_grammar(name, depth);
    let ff = FirstFollowSets::compute(&grammar).expect("FIRST/FOLLOW should succeed");
    let table = build_lr1_automaton(&grammar, &ff).expect("LR(1) automaton should succeed");
    (grammar, table)
}

// ---------------------------------------------------------------------------
// proptest! macro tests (properties 1-8)
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    // 1. Any valid grammar → StaticLanguageGenerator produces non-empty code
    #[test]
    fn prop_static_gen_produces_nonempty_code(
        name in grammar_name_strategy(),
        ntok in token_count_strategy(),
        nrule in rule_count_strategy(),
    ) {
        let (grammar, table) = build_grammar_and_table(&name, ntok, nrule);
        let lang_gen = StaticLanguageGenerator::new(grammar, table);
        let code = lang_gen.generate_language_code();
        prop_assert!(!code.is_empty(), "StaticLanguageGenerator must produce non-empty code");
    }

    // 2. Any valid grammar → NodeTypesGenerator produces valid JSON array
    #[test]
    fn prop_node_types_produces_valid_json_array(
        name in grammar_name_strategy(),
        ntok in token_count_strategy(),
        nrule in rule_count_strategy(),
    ) {
        let grammar = build_grammar(&name, ntok, nrule);
        let ntg = NodeTypesGenerator::new(&grammar);
        let json_str = ntg.generate().expect("NodeTypesGenerator should succeed");
        let parsed: serde_json::Value = serde_json::from_str(&json_str)
            .expect("NodeTypes output must be valid JSON");
        prop_assert!(parsed.is_array(), "NodeTypes JSON must be an array");
    }

    // 3. Code generation is deterministic (same grammar → same code)
    #[test]
    fn prop_code_gen_is_deterministic(
        name in grammar_name_strategy(),
        ntok in token_count_strategy(),
        nrule in rule_count_strategy(),
    ) {
        let (g1, t1) = build_grammar_and_table(&name, ntok, nrule);
        let (g2, t2) = build_grammar_and_table(&name, ntok, nrule);
        let code1 = StaticLanguageGenerator::new(g1, t1).generate_language_code().to_string();
        let code2 = StaticLanguageGenerator::new(g2, t2).generate_language_code().to_string();
        prop_assert_eq!(code1, code2, "same grammar must produce identical code");
    }

    // 4. Node types JSON always has entries with "type" and "named" fields
    #[test]
    fn prop_node_types_entries_have_type_and_named(
        name in grammar_name_strategy(),
        ntok in token_count_strategy(),
        nrule in rule_count_strategy(),
    ) {
        let grammar = build_grammar(&name, ntok, nrule);
        let ntg = NodeTypesGenerator::new(&grammar);
        let json_str = ntg.generate().expect("generate should succeed");
        let arr: Vec<serde_json::Value> = serde_json::from_str(&json_str)
            .expect("must be valid JSON array");
        for entry in &arr {
            prop_assert!(entry.get("type").is_some(), "each entry must have 'type' field");
            prop_assert!(entry.get("named").is_some(), "each entry must have 'named' field");
        }
    }

    // 5. Generated code length increases with grammar complexity
    #[test]
    fn prop_code_length_increases_with_complexity(
        name in grammar_name_strategy(),
    ) {
        let (g_small, t_small) = build_complex_grammar_and_table(&name, 1);
        let (g_large, t_large) = build_complex_grammar_and_table(&name, 4);
        let code_small = StaticLanguageGenerator::new(g_small, t_small)
            .generate_language_code().to_string();
        let code_large = StaticLanguageGenerator::new(g_large, t_large)
            .generate_language_code().to_string();
        prop_assert!(
            code_large.len() >= code_small.len(),
            "larger grammar should produce >= length code: small={} large={}",
            code_small.len(), code_large.len()
        );
    }

    // 6. Grammar name is preserved in generated artifacts
    #[test]
    fn prop_grammar_name_preserved_in_code(
        name in grammar_name_strategy(),
    ) {
        let (grammar, table) = build_grammar_and_table(&name, 1, 1);
        let lang_gen = StaticLanguageGenerator::new(grammar, table);
        let code_str = lang_gen.generate_language_code().to_string();
        prop_assert!(
            code_str.contains(&name),
            "generated code must contain grammar name '{}'", name
        );
    }

    // 7. Token count doesn't affect node types (only non-terminals matter)
    #[test]
    fn prop_extra_tokens_dont_affect_node_types(
        name in grammar_name_strategy(),
    ) {
        // Both grammars have same rules, just different token counts
        let g1 = build_grammar(&name, 1, 1);
        let g2 = build_grammar(&name, 5, 1);

        let json1 = NodeTypesGenerator::new(&g1).generate().expect("gen1");
        let json2 = NodeTypesGenerator::new(&g2).generate().expect("gen2");

        let arr1: Vec<serde_json::Value> = serde_json::from_str(&json1).unwrap();
        let arr2: Vec<serde_json::Value> = serde_json::from_str(&json2).unwrap();

        // Count named (non-terminal) entries
        let named1 = arr1.iter().filter(|e| e["named"] == true).count();
        let named2 = arr2.iter().filter(|e| e["named"] == true).count();
        prop_assert_eq!(named1, named2, "named node count should be same regardless of token count");
    }

    // 8. Multiple generators from same grammar produce identical output
    #[test]
    fn prop_multiple_generators_identical(
        name in grammar_name_strategy(),
        ntok in token_count_strategy(),
    ) {
        let (g1, t1) = build_grammar_and_table(&name, ntok, 1);
        let (g2, t2) = build_grammar_and_table(&name, ntok, 1);
        let code_a = StaticLanguageGenerator::new(g1.clone(), t1.clone())
            .generate_language_code().to_string();
        let code_b = StaticLanguageGenerator::new(g1, t1)
            .generate_language_code().to_string();
        let code_c = StaticLanguageGenerator::new(g2, t2)
            .generate_language_code().to_string();
        prop_assert_eq!(&code_a, &code_b, "same generator inputs must match");
        prop_assert_eq!(&code_a, &code_c, "rebuilt grammar must match");
    }

    // 9. NodeTypes determinism
    #[test]
    fn prop_node_types_deterministic(
        name in grammar_name_strategy(),
        ntok in token_count_strategy(),
    ) {
        let g1 = build_grammar(&name, ntok, 1);
        let g2 = build_grammar(&name, ntok, 1);
        let j1 = NodeTypesGenerator::new(&g1).generate().unwrap();
        let j2 = NodeTypesGenerator::new(&g2).generate().unwrap();
        prop_assert_eq!(j1, j2, "NodeTypes must be deterministic");
    }

    // 10. Code contains LANGUAGE keyword
    #[test]
    fn prop_code_contains_language(
        name in grammar_name_strategy(),
        ntok in token_count_strategy(),
    ) {
        let (grammar, table) = build_grammar_and_table(&name, ntok, 1);
        let code = StaticLanguageGenerator::new(grammar, table)
            .generate_language_code().to_string();
        prop_assert!(code.contains("LANGUAGE"), "must contain LANGUAGE");
    }

    // 11. Code contains SYMBOL_METADATA
    #[test]
    fn prop_code_contains_symbol_metadata(
        name in grammar_name_strategy(),
    ) {
        let (grammar, table) = build_grammar_and_table(&name, 2, 1);
        let code = StaticLanguageGenerator::new(grammar, table)
            .generate_language_code().to_string();
        prop_assert!(code.contains("SYMBOL_METADATA"), "must contain SYMBOL_METADATA");
    }

    // 12. Code contains PARSE_TABLE or PARSE_ACTIONS
    #[test]
    fn prop_code_contains_parse_info(
        name in grammar_name_strategy(),
    ) {
        let (grammar, table) = build_grammar_and_table(&name, 2, 1);
        let code = StaticLanguageGenerator::new(grammar, table)
            .generate_language_code().to_string();
        prop_assert!(
            code.contains("PARSE_TABLE") || code.contains("PARSE_ACTIONS"),
            "must contain PARSE_TABLE or PARSE_ACTIONS"
        );
    }

    // 13. Generated code contains tree_sitter_{name} function
    #[test]
    fn prop_code_contains_language_function(
        name in grammar_name_strategy(),
    ) {
        let (grammar, table) = build_grammar_and_table(&name, 1, 1);
        let code = StaticLanguageGenerator::new(grammar, table)
            .generate_language_code().to_string();
        let fn_name = format!("tree_sitter_{name}");
        prop_assert!(
            code.contains(&fn_name),
            "must contain function '{fn_name}'"
        );
    }

    // 14. start_can_be_empty doesn't crash
    #[test]
    fn prop_start_can_be_empty_no_crash(
        name in grammar_name_strategy(),
        empty in proptest::bool::ANY,
    ) {
        let (grammar, table) = build_grammar_and_table(&name, 1, 1);
        let mut lang_gen = StaticLanguageGenerator::new(grammar, table);
        lang_gen.set_start_can_be_empty(empty);
        let code = lang_gen.generate_language_code();
        prop_assert!(!code.is_empty());
    }

    // 15. Different grammar names produce different code
    #[test]
    fn prop_different_names_produce_different_code(
        name1 in "[a-z]{3,6}",
        name2 in "[a-z]{3,6}",
    ) {
        prop_assume!(name1 != name2);
        let (g1, t1) = build_grammar_and_table(&name1, 1, 1);
        let (g2, t2) = build_grammar_and_table(&name2, 1, 1);
        let code1 = StaticLanguageGenerator::new(g1, t1).generate_language_code().to_string();
        let code2 = StaticLanguageGenerator::new(g2, t2).generate_language_code().to_string();
        prop_assert_ne!(code1, code2, "different names should produce different code");
    }

    // 16. Node types JSON has at least one entry for non-trivial grammar
    #[test]
    fn prop_node_types_nonempty(
        name in grammar_name_strategy(),
        ntok in token_count_strategy(),
    ) {
        let grammar = build_grammar(&name, ntok, 1);
        let json_str = NodeTypesGenerator::new(&grammar).generate().unwrap();
        let arr: Vec<serde_json::Value> = serde_json::from_str(&json_str).unwrap();
        prop_assert!(!arr.is_empty(), "node types must have at least one entry");
    }

    // 17. Code contains SYMBOL_NAMES or symbol_names
    #[test]
    fn prop_code_contains_symbol_names(
        name in grammar_name_strategy(),
    ) {
        let (grammar, table) = build_grammar_and_table(&name, 2, 1);
        let code = StaticLanguageGenerator::new(grammar, table)
            .generate_language_code().to_string();
        let lower = code.to_lowercase();
        prop_assert!(
            lower.contains("symbol_name") || lower.contains("symbol_names"),
            "must contain symbol names reference"
        );
    }

    // 18. Parse table has at least one state
    #[test]
    fn prop_parse_table_has_states(
        name in grammar_name_strategy(),
        ntok in token_count_strategy(),
    ) {
        let (_grammar, table) = build_grammar_and_table(&name, ntok, 1);
        prop_assert!(table.state_count > 0, "parse table must have at least one state");
    }

    // 19. NodeTypesGenerator on complex grammar
    #[test]
    fn prop_complex_grammar_node_types(
        name in grammar_name_strategy(),
        depth in 1usize..=4,
    ) {
        let grammar = build_complex_grammar(&name, depth);
        let json_str = NodeTypesGenerator::new(&grammar).generate().unwrap();
        let arr: Vec<serde_json::Value> = serde_json::from_str(&json_str).unwrap();
        // Each depth level adds a non-terminal rule
        let named_count = arr.iter().filter(|e| e["named"] == true).count();
        prop_assert!(named_count >= 1, "complex grammar must have at least 1 named node type, got {named_count}");
    }

    // 20. StaticLanguageGenerator preserves grammar name in struct
    #[test]
    fn prop_generator_preserves_grammar_name(
        name in grammar_name_strategy(),
    ) {
        let (grammar, table) = build_grammar_and_table(&name, 1, 1);
        let lang_gen = StaticLanguageGenerator::new(grammar, table);
        prop_assert_eq!(&lang_gen.grammar.name, &name);
    }
}

// ---------------------------------------------------------------------------
// Unit tests (#[test]) — properties 21-45
// ---------------------------------------------------------------------------

#[test]
fn test_minimal_grammar_produces_code() {
    let (grammar, table) = build_grammar_and_table("min", 1, 1);
    let code = StaticLanguageGenerator::new(grammar, table)
        .generate_language_code()
        .to_string();
    assert!(!code.is_empty());
}

#[test]
fn test_minimal_grammar_node_types() {
    let grammar = build_grammar("min", 1, 1);
    let json_str = NodeTypesGenerator::new(&grammar).generate().unwrap();
    let arr: Vec<serde_json::Value> = serde_json::from_str(&json_str).unwrap();
    assert!(!arr.is_empty());
}

#[test]
fn test_node_types_type_field_is_string() {
    let grammar = build_grammar("types", 2, 2);
    let json_str = NodeTypesGenerator::new(&grammar).generate().unwrap();
    let arr: Vec<serde_json::Value> = serde_json::from_str(&json_str).unwrap();
    for entry in &arr {
        assert!(entry["type"].is_string(), "type field must be a string");
    }
}

#[test]
fn test_node_types_named_field_is_bool() {
    let grammar = build_grammar("namedbool", 2, 2);
    let json_str = NodeTypesGenerator::new(&grammar).generate().unwrap();
    let arr: Vec<serde_json::Value> = serde_json::from_str(&json_str).unwrap();
    for entry in &arr {
        assert!(entry["named"].is_boolean(), "named field must be a boolean");
    }
}

#[test]
fn test_determinism_across_five_runs() {
    let mut codes = Vec::new();
    for _ in 0..5 {
        let (grammar, table) = build_grammar_and_table("detfive", 2, 2);
        let code = StaticLanguageGenerator::new(grammar, table)
            .generate_language_code()
            .to_string();
        codes.push(code);
    }
    for i in 1..codes.len() {
        assert_eq!(codes[0], codes[i], "run {i} differs from run 0");
    }
}

#[test]
fn test_node_types_determinism_across_five_runs() {
    let mut jsons = Vec::new();
    for _ in 0..5 {
        let grammar = build_grammar("detntfive", 3, 2);
        let json_str = NodeTypesGenerator::new(&grammar).generate().unwrap();
        jsons.push(json_str);
    }
    for i in 1..jsons.len() {
        assert_eq!(jsons[0], jsons[i], "node types run {i} differs from run 0");
    }
}

#[test]
fn test_grammar_name_in_generated_code() {
    let (grammar, table) = build_grammar_and_table("mygrammar", 1, 1);
    let code = StaticLanguageGenerator::new(grammar, table)
        .generate_language_code()
        .to_string();
    assert!(code.contains("mygrammar"), "code must contain grammar name");
}

#[test]
fn test_tree_sitter_function_name() {
    let (grammar, table) = build_grammar_and_table("testlang", 1, 1);
    let code = StaticLanguageGenerator::new(grammar, table)
        .generate_language_code()
        .to_string();
    assert!(
        code.contains("tree_sitter_testlang"),
        "code must contain tree_sitter_testlang"
    );
}

#[test]
fn test_compressed_tables_none_by_default() {
    let (grammar, table) = build_grammar_and_table("nocomp", 1, 1);
    let lang_gen = StaticLanguageGenerator::new(grammar, table);
    assert!(lang_gen.compressed_tables.is_none());
}

#[test]
fn test_start_can_be_empty_default_false() {
    let (grammar, table) = build_grammar_and_table("emptydef", 1, 1);
    let lang_gen = StaticLanguageGenerator::new(grammar, table);
    assert!(!lang_gen.start_can_be_empty);
}

#[test]
fn test_set_start_can_be_empty() {
    let (grammar, table) = build_grammar_and_table("emptymod", 1, 1);
    let mut lang_gen = StaticLanguageGenerator::new(grammar, table);
    lang_gen.set_start_can_be_empty(true);
    assert!(lang_gen.start_can_be_empty);
}

#[test]
fn test_single_token_grammar() {
    let (grammar, table) = build_grammar_and_table("single", 1, 1);
    let code = StaticLanguageGenerator::new(grammar, table)
        .generate_language_code()
        .to_string();
    assert!(!code.is_empty());
    assert!(code.contains("LANGUAGE"));
}

#[test]
fn test_five_token_grammar() {
    let (grammar, table) = build_grammar_and_table("five", 5, 3);
    let code = StaticLanguageGenerator::new(grammar, table)
        .generate_language_code()
        .to_string();
    assert!(!code.is_empty());
}

#[test]
fn test_node_types_root_is_named() {
    let grammar = build_grammar("rootnamed", 1, 1);
    let json_str = NodeTypesGenerator::new(&grammar).generate().unwrap();
    let arr: Vec<serde_json::Value> = serde_json::from_str(&json_str).unwrap();
    // At least one entry should be named (the "root" non-terminal)
    let has_named = arr.iter().any(|e| e["named"] == true);
    assert!(has_named, "should have at least one named entry for 'root'");
}

#[test]
fn test_parse_table_state_count_positive() {
    let (_grammar, table) = build_grammar_and_table("stcount", 2, 1);
    assert!(table.state_count > 0);
}

#[test]
fn test_parse_table_symbol_count_positive() {
    let (_grammar, table) = build_grammar_and_table("symcount", 2, 1);
    assert!(table.symbol_count > 0);
}

#[test]
fn test_first_follow_computes_for_simple_grammar() {
    let grammar = build_grammar("fftest", 2, 1);
    let ff = FirstFollowSets::compute(&grammar);
    assert!(ff.is_ok(), "FIRST/FOLLOW should succeed on simple grammar");
}

#[test]
fn test_build_lr1_automaton_succeeds() {
    let grammar = build_grammar("lr1test", 2, 1);
    let ff = FirstFollowSets::compute(&grammar).unwrap();
    let result = build_lr1_automaton(&grammar, &ff);
    assert!(result.is_ok(), "build_lr1_automaton should succeed");
}

#[test]
fn test_complex_grammar_depth_1() {
    let (grammar, table) = build_complex_grammar_and_table("cd1", 1);
    let code = StaticLanguageGenerator::new(grammar, table)
        .generate_language_code()
        .to_string();
    assert!(!code.is_empty());
}

#[test]
fn test_complex_grammar_depth_3() {
    let (grammar, table) = build_complex_grammar_and_table("cd3", 3);
    let code = StaticLanguageGenerator::new(grammar, table)
        .generate_language_code()
        .to_string();
    assert!(!code.is_empty());
}

#[test]
fn test_complex_grammar_depth_5() {
    let (grammar, table) = build_complex_grammar_and_table("cd5", 5);
    let code = StaticLanguageGenerator::new(grammar, table)
        .generate_language_code()
        .to_string();
    assert!(!code.is_empty());
}

#[test]
fn test_node_types_json_is_array_not_object() {
    let grammar = build_grammar("arrcheck", 2, 1);
    let json_str = NodeTypesGenerator::new(&grammar).generate().unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    assert!(parsed.is_array(), "must be JSON array, not object");
}

#[test]
fn test_code_generation_not_empty_string() {
    let (grammar, table) = build_grammar_and_table("notempty", 1, 1);
    let code = StaticLanguageGenerator::new(grammar, table)
        .generate_language_code()
        .to_string();
    assert!(
        !code.trim().is_empty(),
        "code string must not be empty or whitespace"
    );
}

#[test]
fn test_generator_grammar_field_accessible() {
    let (grammar, table) = build_grammar_and_table("accessible", 1, 1);
    let lang_gen = StaticLanguageGenerator::new(grammar, table);
    // Verify the grammar and parse_table are accessible
    assert_eq!(lang_gen.grammar.name, "accessible");
    assert!(lang_gen.parse_table.state_count > 0);
}

#[test]
fn test_node_types_no_duplicate_type_names() {
    let grammar = build_grammar("nodup", 3, 2);
    let json_str = NodeTypesGenerator::new(&grammar).generate().unwrap();
    let arr: Vec<serde_json::Value> = serde_json::from_str(&json_str).unwrap();
    let mut seen = std::collections::HashSet::new();
    for entry in &arr {
        let type_name = entry["type"].as_str().unwrap();
        let named = entry["named"].as_bool().unwrap();
        let key = (type_name.to_string(), named);
        assert!(seen.insert(key.clone()), "duplicate entry: {key:?}");
    }
}

#[test]
fn test_multiple_rules_same_lhs() {
    // Grammar with 3 alternatives for 'root'
    let (grammar, table) = build_grammar_and_table("multirule", 3, 3);
    let code = StaticLanguageGenerator::new(grammar, table)
        .generate_language_code()
        .to_string();
    assert!(!code.is_empty());
}

#[test]
fn test_code_generation_with_default_table() {
    // Using default (empty) parse table should still produce code
    let grammar = build_grammar("deftable", 1, 1);
    let lang_gen = StaticLanguageGenerator::new(grammar, ParseTable::default());
    let code = lang_gen.generate_language_code().to_string();
    assert!(!code.is_empty());
}

#[test]
fn test_node_types_consistent_structure() {
    // Every entry should be an object (not a primitive)
    let grammar = build_grammar("structure", 2, 1);
    let json_str = NodeTypesGenerator::new(&grammar).generate().unwrap();
    let arr: Vec<serde_json::Value> = serde_json::from_str(&json_str).unwrap();
    for entry in &arr {
        assert!(entry.is_object(), "each entry must be a JSON object");
    }
}
