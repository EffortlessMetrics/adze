//! Property-based tests for code generation in adze-tablegen (v6).
//!
//! 10 categories (60+ proptest properties):
//!  1. prop_node_types_*         — NodeTypesGenerator basics (6)
//!  2. prop_node_types_json_*    — JSON structure / content (7)
//!  3. prop_abi_*                — AbiLanguageBuilder output (7)
//!  4. prop_static_gen_*         — StaticLanguageGenerator output (7)
//!  5. prop_determinism_*        — deterministic output (6)
//!  6. prop_name_sensitivity_*   — different names → different output (4)
//!  7. prop_scaling_*            — output scales with grammar size (4)
//!  8. prop_precedence_*         — grammars with precedence (6)
//!  9. prop_associativity_*      — grammars with associativity (6)
//! 10. prop_roundtrip_*          — JSON roundtrip / re-parse (7)

use adze_glr_core::{FirstFollowSets, ParseTable, build_lr1_automaton};
use adze_ir::builder::GrammarBuilder;
use adze_ir::{Associativity, Grammar};
use adze_tablegen::{AbiLanguageBuilder, NodeTypesGenerator, StaticLanguageGenerator};
use proptest::prelude::*;
use serde_json::Value;

// ---------------------------------------------------------------------------
// Strategies
// ---------------------------------------------------------------------------

/// Grammar name: lowercase ASCII, 1–12 chars, prefixed with `pcg_v6_`.
fn grammar_name(index: u32) -> String {
    format!("pcg_v6_{index}")
}

/// Strategy producing a unique index for grammar names.
fn name_index() -> impl Strategy<Value = u32> {
    0u32..10_000
}

/// Token count 1..8.
fn token_count() -> impl Strategy<Value = usize> {
    1usize..=8
}

/// Small token count for pairing tests.
fn small_token_count() -> impl Strategy<Value = usize> {
    1usize..=4
}

/// Precedence level.
fn prec_level() -> impl Strategy<Value = i16> {
    -5i16..=5
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Build a grammar with `n` visible tokens and a single rule referencing the first.
fn grammar_with_n_tokens(name: &str, n: usize) -> Grammar {
    let count = n.max(1);
    let mut builder = GrammarBuilder::new(name);
    for i in 0..count {
        builder = builder.token(&format!("tok{i}"), &format!("t{i}"));
    }
    builder = builder.rule("root", vec!["tok0"]).start("root");
    builder.build()
}

/// Full LR(1) pipeline for a grammar builder.
fn pipeline(builder: GrammarBuilder) -> (Grammar, ParseTable) {
    let mut g = builder.build();
    let ff = FirstFollowSets::compute_normalized(&mut g).expect("FIRST/FOLLOW");
    let pt = build_lr1_automaton(&g, &ff).expect("LR(1)");
    (g, pt)
}

/// Simple one-token grammar through the pipeline.
fn simple_pipeline(name: &str) -> (Grammar, ParseTable) {
    pipeline(
        GrammarBuilder::new(name)
            .token("a", "a")
            .rule("s", vec!["a"])
            .start("s"),
    )
}

/// Two-alternative grammar.
fn two_alt_pipeline(name: &str) -> (Grammar, ParseTable) {
    pipeline(
        GrammarBuilder::new(name)
            .token("a", "a")
            .token("b", "b")
            .rule("s", vec!["a"])
            .rule("s", vec!["b"])
            .start("s"),
    )
}

/// Chain grammar: s -> a b.
fn chain_pipeline(name: &str) -> (Grammar, ParseTable) {
    pipeline(
        GrammarBuilder::new(name)
            .token("a", "a")
            .token("b", "b")
            .rule("s", vec!["a", "b"])
            .start("s"),
    )
}

/// Multi-nonterminal grammar: inner -> a, s -> inner b.
fn multi_nt_pipeline(name: &str) -> (Grammar, ParseTable) {
    pipeline(
        GrammarBuilder::new(name)
            .token("a", "a")
            .token("b", "b")
            .rule("inner", vec!["a"])
            .rule("s", vec!["inner", "b"])
            .start("s"),
    )
}

/// Left-recursive grammar: s -> a | s a.
fn recursive_pipeline(name: &str) -> (Grammar, ParseTable) {
    pipeline(
        GrammarBuilder::new(name)
            .token("a", "a")
            .rule("s", vec!["a"])
            .rule("s", vec!["s", "a"])
            .start("s"),
    )
}

/// Build a grammar with precedence on the first rule.
fn prec_pipeline(name: &str, prec: i16, assoc: Associativity) -> (Grammar, ParseTable) {
    pipeline(
        GrammarBuilder::new(name)
            .token("a", "a")
            .token("op", "o")
            .rule_with_precedence("s", vec!["a"], prec, assoc)
            .rule("s", vec!["s", "op", "s"])
            .start("s"),
    )
}

/// Build a grammar through the pipeline with `n` tokens.
fn n_token_pipeline(name: &str, n: usize) -> (Grammar, ParseTable) {
    let count = n.max(1);
    let mut builder = GrammarBuilder::new(name);
    for i in 0..count {
        builder = builder.token(&format!("tok{i}"), &format!("t{i}"));
    }
    builder = builder.rule("root", vec!["tok0"]).start("root");
    pipeline(builder)
}

// ===========================================================================
// 1. NodeTypesGenerator basics (6)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig { cases: 40, .. ProptestConfig::default() })]

    #[test]
    fn prop_node_types_succeeds_n_tokens(n in token_count(), idx in name_index()) {
        let name = grammar_name(idx);
        let g = grammar_with_n_tokens(&name, n);
        let result = NodeTypesGenerator::new(&g).generate();
        prop_assert!(result.is_ok(), "NodeTypesGenerator must succeed: {:?}", result.err());
    }

    #[test]
    fn prop_node_types_output_valid_json(idx in name_index()) {
        let name = grammar_name(idx);
        let g = grammar_with_n_tokens(&name, 3);
        let json_str = NodeTypesGenerator::new(&g).generate().unwrap();
        let parsed: Result<Value, _> = serde_json::from_str(&json_str);
        prop_assert!(parsed.is_ok(), "NodeTypes output must be valid JSON: {:?}", parsed.err());
    }

    #[test]
    fn prop_node_types_output_non_empty(idx in name_index()) {
        let name = grammar_name(idx);
        let g = grammar_with_n_tokens(&name, 2);
        let json_str = NodeTypesGenerator::new(&g).generate().unwrap();
        prop_assert!(!json_str.is_empty(), "NodeTypes output must be non-empty");
    }

    #[test]
    fn prop_node_types_single_token_ok(idx in name_index()) {
        let name = grammar_name(idx);
        let g = grammar_with_n_tokens(&name, 1);
        let result = NodeTypesGenerator::new(&g).generate();
        prop_assert!(result.is_ok(), "single-token NodeTypes must succeed");
    }

    #[test]
    fn prop_node_types_max_tokens_ok(idx in name_index()) {
        let name = grammar_name(idx);
        let g = grammar_with_n_tokens(&name, 8);
        let result = NodeTypesGenerator::new(&g).generate();
        prop_assert!(result.is_ok(), "8-token NodeTypes must succeed");
    }

    #[test]
    fn prop_node_types_two_alt_ok(idx in name_index()) {
        let name = grammar_name(idx);
        let (g, _pt) = two_alt_pipeline(&name);
        let result = NodeTypesGenerator::new(&g).generate();
        prop_assert!(result.is_ok(), "two-alt NodeTypes must succeed");
    }
}

// ===========================================================================
// 2. NodeTypes JSON structure / content (7)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig { cases: 40, .. ProptestConfig::default() })]

    #[test]
    fn prop_node_types_json_is_array(idx in name_index()) {
        let name = grammar_name(idx);
        let g = grammar_with_n_tokens(&name, 3);
        let json_str = NodeTypesGenerator::new(&g).generate().unwrap();
        let val: Value = serde_json::from_str(&json_str).unwrap();
        prop_assert!(val.is_array(), "NodeTypes must be a JSON array");
    }

    #[test]
    fn prop_node_types_json_entries_have_type(idx in name_index()) {
        let name = grammar_name(idx);
        let g = grammar_with_n_tokens(&name, 2);
        let json_str = NodeTypesGenerator::new(&g).generate().unwrap();
        let val: Value = serde_json::from_str(&json_str).unwrap();
        if let Some(arr) = val.as_array() {
            for entry in arr {
                prop_assert!(entry.get("type").is_some(), "each entry must have 'type'");
            }
        }
    }

    #[test]
    fn prop_node_types_json_entries_have_named(idx in name_index()) {
        let name = grammar_name(idx);
        let g = grammar_with_n_tokens(&name, 2);
        let json_str = NodeTypesGenerator::new(&g).generate().unwrap();
        let val: Value = serde_json::from_str(&json_str).unwrap();
        if let Some(arr) = val.as_array() {
            for entry in arr {
                prop_assert!(entry.get("named").is_some(), "each entry must have 'named'");
            }
        }
    }

    #[test]
    fn prop_node_types_json_type_is_string(idx in name_index()) {
        let name = grammar_name(idx);
        let g = grammar_with_n_tokens(&name, 4);
        let json_str = NodeTypesGenerator::new(&g).generate().unwrap();
        let val: Value = serde_json::from_str(&json_str).unwrap();
        if let Some(arr) = val.as_array() {
            for entry in arr {
                if let Some(ty) = entry.get("type") {
                    prop_assert!(ty.is_string(), "'type' must be a string");
                }
            }
        }
    }

    #[test]
    fn prop_node_types_json_named_is_bool(idx in name_index()) {
        let name = grammar_name(idx);
        let g = grammar_with_n_tokens(&name, 4);
        let json_str = NodeTypesGenerator::new(&g).generate().unwrap();
        let val: Value = serde_json::from_str(&json_str).unwrap();
        if let Some(arr) = val.as_array() {
            for entry in arr {
                if let Some(named) = entry.get("named") {
                    prop_assert!(named.is_boolean(), "'named' must be a bool");
                }
            }
        }
    }

    #[test]
    fn prop_node_types_json_parseable_serde(idx in name_index()) {
        let name = grammar_name(idx);
        let g = grammar_with_n_tokens(&name, 5);
        let json_str = NodeTypesGenerator::new(&g).generate().unwrap();
        let parsed: Result<Vec<Value>, _> = serde_json::from_str(&json_str);
        prop_assert!(parsed.is_ok(), "NodeTypes must parse as Vec<Value>");
    }

    #[test]
    fn prop_node_types_json_non_empty_array(idx in name_index()) {
        let name = grammar_name(idx);
        let (g, _pt) = simple_pipeline(&name);
        let json_str = NodeTypesGenerator::new(&g).generate().unwrap();
        let val: Value = serde_json::from_str(&json_str).unwrap();
        let arr = val.as_array().unwrap();
        prop_assert!(!arr.is_empty(), "NodeTypes array must not be empty");
    }
}

// ===========================================================================
// 3. AbiLanguageBuilder output (7)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig { cases: 40, .. ProptestConfig::default() })]

    #[test]
    fn prop_abi_non_empty(idx in name_index()) {
        let name = grammar_name(idx);
        let (g, pt) = simple_pipeline(&name);
        let code = AbiLanguageBuilder::new(&g, &pt).generate();
        prop_assert!(!code.is_empty(), "ABI output must be non-empty");
    }

    #[test]
    fn prop_abi_contains_grammar_name(idx in name_index()) {
        let name = grammar_name(idx);
        let (g, pt) = simple_pipeline(&name);
        let code = AbiLanguageBuilder::new(&g, &pt).generate().to_string();
        prop_assert!(
            code.contains(&name),
            "ABI output must contain grammar name '{name}'"
        );
    }

    #[test]
    fn prop_abi_contains_symbol_count(idx in name_index()) {
        let name = grammar_name(idx);
        let (g, pt) = simple_pipeline(&name);
        let code = AbiLanguageBuilder::new(&g, &pt).generate().to_string();
        prop_assert!(code.contains("symbol_count"), "ABI must contain symbol_count");
    }

    #[test]
    fn prop_abi_contains_state_count(idx in name_index()) {
        let name = grammar_name(idx);
        let (g, pt) = simple_pipeline(&name);
        let code = AbiLanguageBuilder::new(&g, &pt).generate().to_string();
        prop_assert!(code.contains("state_count"), "ABI must contain state_count");
    }

    #[test]
    fn prop_abi_contains_token_count(idx in name_index()) {
        let name = grammar_name(idx);
        let (g, pt) = simple_pipeline(&name);
        let code = AbiLanguageBuilder::new(&g, &pt).generate().to_string();
        prop_assert!(code.contains("token_count"), "ABI must contain token_count");
    }

    #[test]
    fn prop_abi_chain_grammar_nonempty(idx in name_index()) {
        let name = grammar_name(idx);
        let (g, pt) = chain_pipeline(&name);
        let code = AbiLanguageBuilder::new(&g, &pt).generate();
        prop_assert!(!code.is_empty(), "chain ABI must be non-empty");
    }

    #[test]
    fn prop_abi_multi_nt_nonempty(idx in name_index()) {
        let name = grammar_name(idx);
        let (g, pt) = multi_nt_pipeline(&name);
        let code = AbiLanguageBuilder::new(&g, &pt).generate();
        prop_assert!(!code.is_empty(), "multi-NT ABI must be non-empty");
    }
}

// ===========================================================================
// 4. StaticLanguageGenerator output (7)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig { cases: 40, .. ProptestConfig::default() })]

    #[test]
    fn prop_static_gen_non_empty(idx in name_index()) {
        let name = grammar_name(idx);
        let (g, pt) = simple_pipeline(&name);
        let code = StaticLanguageGenerator::new(g, pt).generate_language_code();
        prop_assert!(!code.is_empty(), "StaticLanguageGenerator must produce non-empty output");
    }

    #[test]
    fn prop_static_gen_contains_fn_or_const(idx in name_index()) {
        let name = grammar_name(idx);
        let (g, pt) = simple_pipeline(&name);
        let code = StaticLanguageGenerator::new(g, pt)
            .generate_language_code()
            .to_string();
        let has_fn = code.contains("fn ");
        let has_const = code.contains("const ");
        prop_assert!(
            has_fn || has_const,
            "StaticLanguageGenerator output must contain 'fn' or 'const'"
        );
    }

    #[test]
    fn prop_static_gen_contains_parse_table(idx in name_index()) {
        let name = grammar_name(idx);
        let (g, pt) = simple_pipeline(&name);
        let code = StaticLanguageGenerator::new(g, pt)
            .generate_language_code()
            .to_string();
        let has_table = code.contains("PARSE_TABLE") || code.contains("parse_table");
        prop_assert!(has_table, "code must reference parse table");
    }

    #[test]
    fn prop_static_gen_contains_symbol_names(idx in name_index()) {
        let name = grammar_name(idx);
        let (g, pt) = simple_pipeline(&name);
        let code = StaticLanguageGenerator::new(g, pt)
            .generate_language_code()
            .to_string();
        let has = code.contains("SYMBOL_NAMES") || code.contains("symbol_names");
        prop_assert!(has, "code must reference symbol names");
    }

    #[test]
    fn prop_static_gen_node_types_valid_json(idx in name_index()) {
        let name = grammar_name(idx);
        let (g, pt) = simple_pipeline(&name);
        let json_str = StaticLanguageGenerator::new(g, pt).generate_node_types();
        let parsed: Result<Value, _> = serde_json::from_str(&json_str);
        prop_assert!(parsed.is_ok(), "generate_node_types must produce valid JSON");
    }

    #[test]
    fn prop_static_gen_chain_nonempty(idx in name_index()) {
        let name = grammar_name(idx);
        let (g, pt) = chain_pipeline(&name);
        let code = StaticLanguageGenerator::new(g, pt).generate_language_code();
        prop_assert!(!code.is_empty(), "chain grammar StaticLangGen must be non-empty");
    }

    #[test]
    fn prop_static_gen_recursive_nonempty(idx in name_index()) {
        let name = grammar_name(idx);
        let (g, pt) = recursive_pipeline(&name);
        let code = StaticLanguageGenerator::new(g, pt).generate_language_code();
        prop_assert!(!code.is_empty(), "recursive grammar StaticLangGen must be non-empty");
    }
}

// ===========================================================================
// 5. Determinism (6)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig { cases: 40, .. ProptestConfig::default() })]

    #[test]
    fn prop_determinism_node_types(idx in name_index()) {
        let name = grammar_name(idx);
        let (g, _pt) = simple_pipeline(&name);
        let j1 = NodeTypesGenerator::new(&g).generate().unwrap();
        let j2 = NodeTypesGenerator::new(&g).generate().unwrap();
        prop_assert_eq!(j1, j2, "NodeTypesGenerator must be deterministic");
    }

    #[test]
    fn prop_determinism_abi(idx in name_index()) {
        let name = grammar_name(idx);
        let (g, pt) = simple_pipeline(&name);
        let c1 = AbiLanguageBuilder::new(&g, &pt).generate().to_string();
        let c2 = AbiLanguageBuilder::new(&g, &pt).generate().to_string();
        prop_assert_eq!(c1, c2, "AbiLanguageBuilder must be deterministic");
    }

    #[test]
    fn prop_determinism_static_gen(idx in name_index()) {
        let name = grammar_name(idx);
        let (g, pt) = simple_pipeline(&name);
        let c1 = StaticLanguageGenerator::new(g.clone(), pt.clone())
            .generate_language_code()
            .to_string();
        let c2 = StaticLanguageGenerator::new(g, pt)
            .generate_language_code()
            .to_string();
        prop_assert_eq!(c1, c2, "StaticLanguageGenerator must be deterministic");
    }

    #[test]
    fn prop_determinism_static_node_types(idx in name_index()) {
        let name = grammar_name(idx);
        let (g, pt) = simple_pipeline(&name);
        let slg = StaticLanguageGenerator::new(g, pt);
        let j1 = slg.generate_node_types();
        let j2 = slg.generate_node_types();
        prop_assert_eq!(j1, j2, "generate_node_types must be deterministic");
    }

    #[test]
    fn prop_determinism_chain_abi(idx in name_index()) {
        let name = grammar_name(idx);
        let (g, pt) = chain_pipeline(&name);
        let c1 = AbiLanguageBuilder::new(&g, &pt).generate().to_string();
        let c2 = AbiLanguageBuilder::new(&g, &pt).generate().to_string();
        prop_assert_eq!(c1, c2, "chain ABI must be deterministic");
    }

    #[test]
    fn prop_determinism_two_alt_node_types(idx in name_index()) {
        let name = grammar_name(idx);
        let (g, _pt) = two_alt_pipeline(&name);
        let j1 = NodeTypesGenerator::new(&g).generate().unwrap();
        let j2 = NodeTypesGenerator::new(&g).generate().unwrap();
        prop_assert_eq!(j1, j2, "two-alt NodeTypes must be deterministic");
    }
}

// ===========================================================================
// 6. Name sensitivity — different names → different output (4)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig { cases: 30, .. ProptestConfig::default() })]

    #[test]
    fn prop_name_sensitivity_abi(idx1 in name_index(), idx2 in name_index()) {
        prop_assume!(idx1 != idx2);
        let n1 = grammar_name(idx1);
        let n2 = grammar_name(idx2);
        let (g1, pt1) = simple_pipeline(&n1);
        let (g2, pt2) = simple_pipeline(&n2);
        let c1 = AbiLanguageBuilder::new(&g1, &pt1).generate().to_string();
        let c2 = AbiLanguageBuilder::new(&g2, &pt2).generate().to_string();
        prop_assert_ne!(c1, c2, "different names must yield different ABI");
    }

    #[test]
    fn prop_name_sensitivity_static_gen(idx1 in name_index(), idx2 in name_index()) {
        prop_assume!(idx1 != idx2);
        let n1 = grammar_name(idx1);
        let n2 = grammar_name(idx2);
        let (g1, pt1) = simple_pipeline(&n1);
        let (g2, pt2) = simple_pipeline(&n2);
        let c1 = StaticLanguageGenerator::new(g1, pt1)
            .generate_language_code()
            .to_string();
        let c2 = StaticLanguageGenerator::new(g2, pt2)
            .generate_language_code()
            .to_string();
        prop_assert_ne!(c1, c2, "different names must yield different static code");
    }

    #[test]
    fn prop_name_sensitivity_node_types_generator(idx1 in name_index(), idx2 in name_index()) {
        prop_assume!(idx1 != idx2);
        let n1 = grammar_name(idx1);
        let n2 = grammar_name(idx2);
        let g1 = grammar_with_n_tokens(&n1, 3);
        let g2 = grammar_with_n_tokens(&n2, 3);
        let j1 = NodeTypesGenerator::new(&g1).generate().unwrap();
        let j2 = NodeTypesGenerator::new(&g2).generate().unwrap();
        // NodeTypesGenerator may or may not embed grammar name in output;
        // at minimum, grammar IR differs so output _could_ differ.
        // We just verify both succeed independently.
        prop_assert!(!j1.is_empty());
        prop_assert!(!j2.is_empty());
    }

    #[test]
    fn prop_name_sensitivity_abi_contains_each_name(idx1 in name_index(), idx2 in name_index()) {
        prop_assume!(idx1 != idx2);
        let n1 = grammar_name(idx1);
        let n2 = grammar_name(idx2);
        let (g1, pt1) = simple_pipeline(&n1);
        let (g2, pt2) = simple_pipeline(&n2);
        let c1 = AbiLanguageBuilder::new(&g1, &pt1).generate().to_string();
        let c2 = AbiLanguageBuilder::new(&g2, &pt2).generate().to_string();
        prop_assert!(c1.contains(&n1), "ABI must contain own grammar name");
        prop_assert!(c2.contains(&n2), "ABI must contain own grammar name");
    }
}

// ===========================================================================
// 7. Scaling — output scales with grammar size (4)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig { cases: 30, .. ProptestConfig::default() })]

    #[test]
    fn prop_scaling_node_types_length(n in small_token_count(), idx in name_index()) {
        let name_small = grammar_name(idx);
        let name_large = grammar_name(idx + 10_000);
        let g_small = grammar_with_n_tokens(&name_small, n);
        let g_large = grammar_with_n_tokens(&name_large, n + 4);
        let j_small = NodeTypesGenerator::new(&g_small).generate().unwrap();
        let j_large = NodeTypesGenerator::new(&g_large).generate().unwrap();
        prop_assert!(
            j_large.len() >= j_small.len(),
            "more tokens should produce longer NodeTypes ({} vs {})",
            j_large.len(),
            j_small.len()
        );
    }

    #[test]
    fn prop_scaling_abi_vs_state_count(idx in name_index()) {
        let n1 = grammar_name(idx);
        let n2 = grammar_name(idx + 10_000);
        let (g1, pt1) = simple_pipeline(&n1);
        let (g2, pt2) = chain_pipeline(&n2);
        let c1 = AbiLanguageBuilder::new(&g1, &pt1).generate().to_string();
        let c2 = AbiLanguageBuilder::new(&g2, &pt2).generate().to_string();
        // chain grammar has at least as many states
        if pt2.state_count >= pt1.state_count {
            prop_assert!(
                c2.len() >= c1.len(),
                "more states should yield >= ABI size ({} vs {})",
                c2.len(),
                c1.len()
            );
        }
    }

    #[test]
    fn prop_scaling_static_gen_multi_gt_single(idx in name_index()) {
        let n1 = grammar_name(idx);
        let n2 = grammar_name(idx + 10_000);
        let (g1, pt1) = simple_pipeline(&n1);
        let (g2, pt2) = multi_nt_pipeline(&n2);
        let len1 = StaticLanguageGenerator::new(g1, pt1)
            .generate_language_code()
            .to_string()
            .len();
        let len2 = StaticLanguageGenerator::new(g2, pt2)
            .generate_language_code()
            .to_string()
            .len();
        prop_assert!(
            len2 > len1,
            "multi-NT grammar must produce more code ({} vs {})",
            len2,
            len1
        );
    }

    #[test]
    fn prop_scaling_n_tokens_monotonic(idx in name_index()) {
        let n1 = grammar_name(idx);
        let n2 = grammar_name(idx + 10_000);
        let (g1, pt1) = n_token_pipeline(&n1, 2);
        let (g2, pt2) = n_token_pipeline(&n2, 6);
        let len1 = StaticLanguageGenerator::new(g1, pt1)
            .generate_language_code()
            .to_string()
            .len();
        let len2 = StaticLanguageGenerator::new(g2, pt2)
            .generate_language_code()
            .to_string()
            .len();
        prop_assert!(
            len2 >= len1,
            "more tokens should produce >= code ({} vs {})",
            len2,
            len1
        );
    }
}

// ===========================================================================
// 8. Grammars with precedence (6)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig { cases: 30, .. ProptestConfig::default() })]

    #[test]
    fn prop_precedence_node_types_ok(prec in prec_level(), idx in name_index()) {
        let name = grammar_name(idx);
        let (g, _pt) = prec_pipeline(&name, prec, Associativity::Left);
        let result = NodeTypesGenerator::new(&g).generate();
        prop_assert!(result.is_ok(), "prec grammar NodeTypes must succeed");
    }

    #[test]
    fn prop_precedence_abi_ok(prec in prec_level(), idx in name_index()) {
        let name = grammar_name(idx);
        let (g, pt) = prec_pipeline(&name, prec, Associativity::Left);
        let code = AbiLanguageBuilder::new(&g, &pt).generate();
        prop_assert!(!code.is_empty(), "prec grammar ABI must be non-empty");
    }

    #[test]
    fn prop_precedence_static_gen_ok(prec in prec_level(), idx in name_index()) {
        let name = grammar_name(idx);
        let (g, pt) = prec_pipeline(&name, prec, Associativity::Left);
        let code = StaticLanguageGenerator::new(g, pt).generate_language_code();
        prop_assert!(!code.is_empty(), "prec grammar static gen must be non-empty");
    }

    #[test]
    fn prop_precedence_node_types_valid_json(prec in prec_level(), idx in name_index()) {
        let name = grammar_name(idx);
        let (g, _pt) = prec_pipeline(&name, prec, Associativity::Right);
        let json_str = NodeTypesGenerator::new(&g).generate().unwrap();
        let parsed: Result<Value, _> = serde_json::from_str(&json_str);
        prop_assert!(parsed.is_ok(), "prec grammar NodeTypes must be valid JSON");
    }

    #[test]
    fn prop_precedence_abi_contains_name(prec in prec_level(), idx in name_index()) {
        let name = grammar_name(idx);
        let (g, pt) = prec_pipeline(&name, prec, Associativity::None);
        let code = AbiLanguageBuilder::new(&g, &pt).generate().to_string();
        prop_assert!(code.contains(&name), "prec ABI must contain grammar name");
    }

    #[test]
    fn prop_precedence_deterministic(prec in prec_level(), idx in name_index()) {
        let name = grammar_name(idx);
        let (g, pt) = prec_pipeline(&name, prec, Associativity::Left);
        let c1 = AbiLanguageBuilder::new(&g, &pt).generate().to_string();
        let c2 = AbiLanguageBuilder::new(&g, &pt).generate().to_string();
        prop_assert_eq!(c1, c2, "prec grammar ABI must be deterministic");
    }
}

// ===========================================================================
// 9. Grammars with associativity (6)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig { cases: 30, .. ProptestConfig::default() })]

    #[test]
    fn prop_assoc_left_all_generators_ok(idx in name_index()) {
        let name = grammar_name(idx);
        let (g, pt) = prec_pipeline(&name, 1, Associativity::Left);
        prop_assert!(NodeTypesGenerator::new(&g).generate().is_ok());
        prop_assert!(!AbiLanguageBuilder::new(&g, &pt).generate().is_empty());
        let code = StaticLanguageGenerator::new(g, pt).generate_language_code();
        prop_assert!(!code.is_empty());
    }

    #[test]
    fn prop_assoc_right_all_generators_ok(idx in name_index()) {
        let name = grammar_name(idx);
        let (g, pt) = prec_pipeline(&name, 2, Associativity::Right);
        prop_assert!(NodeTypesGenerator::new(&g).generate().is_ok());
        prop_assert!(!AbiLanguageBuilder::new(&g, &pt).generate().is_empty());
        let code = StaticLanguageGenerator::new(g, pt).generate_language_code();
        prop_assert!(!code.is_empty());
    }

    #[test]
    fn prop_assoc_none_all_generators_ok(idx in name_index()) {
        let name = grammar_name(idx);
        let (g, pt) = prec_pipeline(&name, 0, Associativity::None);
        prop_assert!(NodeTypesGenerator::new(&g).generate().is_ok());
        prop_assert!(!AbiLanguageBuilder::new(&g, &pt).generate().is_empty());
        let code = StaticLanguageGenerator::new(g, pt).generate_language_code();
        prop_assert!(!code.is_empty());
    }

    #[test]
    fn prop_assoc_left_valid_json(idx in name_index()) {
        let name = grammar_name(idx);
        let (g, _pt) = prec_pipeline(&name, 3, Associativity::Left);
        let json_str = NodeTypesGenerator::new(&g).generate().unwrap();
        let parsed: Result<Value, _> = serde_json::from_str(&json_str);
        prop_assert!(parsed.is_ok(), "left-assoc NodeTypes must be valid JSON");
    }

    #[test]
    fn prop_assoc_right_deterministic(idx in name_index()) {
        let name = grammar_name(idx);
        let (g, pt) = prec_pipeline(&name, -2, Associativity::Right);
        let c1 = StaticLanguageGenerator::new(g.clone(), pt.clone())
            .generate_language_code()
            .to_string();
        let c2 = StaticLanguageGenerator::new(g, pt)
            .generate_language_code()
            .to_string();
        prop_assert_eq!(c1, c2, "right-assoc static gen must be deterministic");
    }

    #[test]
    fn prop_assoc_none_abi_contains_name(idx in name_index()) {
        let name = grammar_name(idx);
        let (g, pt) = prec_pipeline(&name, 0, Associativity::None);
        let code = AbiLanguageBuilder::new(&g, &pt).generate().to_string();
        prop_assert!(code.contains(&name), "non-assoc ABI must contain grammar name");
    }
}

// ===========================================================================
// 10. JSON roundtrip / re-parse (7)
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig { cases: 40, .. ProptestConfig::default() })]

    #[test]
    fn prop_roundtrip_node_types_json(idx in name_index()) {
        let name = grammar_name(idx);
        let (g, _pt) = simple_pipeline(&name);
        let json_str = NodeTypesGenerator::new(&g).generate().unwrap();
        let val: Value = serde_json::from_str(&json_str).unwrap();
        let re = serde_json::to_string_pretty(&val).unwrap();
        let val2: Value = serde_json::from_str(&re).unwrap();
        prop_assert_eq!(val, val2, "NodeTypes JSON must roundtrip");
    }

    #[test]
    fn prop_roundtrip_static_node_types_json(idx in name_index()) {
        let name = grammar_name(idx);
        let (g, pt) = simple_pipeline(&name);
        let json_str = StaticLanguageGenerator::new(g, pt).generate_node_types();
        let val: Value = serde_json::from_str(&json_str).unwrap();
        let re = serde_json::to_string_pretty(&val).unwrap();
        let val2: Value = serde_json::from_str(&re).unwrap();
        prop_assert_eq!(val, val2, "static gen node types JSON must roundtrip");
    }

    #[test]
    fn prop_roundtrip_chain_node_types(idx in name_index()) {
        let name = grammar_name(idx);
        let (g, _pt) = chain_pipeline(&name);
        let json_str = NodeTypesGenerator::new(&g).generate().unwrap();
        let val: Value = serde_json::from_str(&json_str).unwrap();
        let re = serde_json::to_string(&val).unwrap();
        let val2: Value = serde_json::from_str(&re).unwrap();
        prop_assert_eq!(val, val2, "chain NodeTypes must roundtrip");
    }

    #[test]
    fn prop_roundtrip_two_alt_node_types(idx in name_index()) {
        let name = grammar_name(idx);
        let (g, _pt) = two_alt_pipeline(&name);
        let json_str = NodeTypesGenerator::new(&g).generate().unwrap();
        let val: Value = serde_json::from_str(&json_str).unwrap();
        let re = serde_json::to_string_pretty(&val).unwrap();
        let val2: Value = serde_json::from_str(&re).unwrap();
        prop_assert_eq!(val, val2, "two-alt NodeTypes must roundtrip");
    }

    #[test]
    fn prop_roundtrip_prec_node_types(idx in name_index()) {
        let name = grammar_name(idx);
        let (g, _pt) = prec_pipeline(&name, 3, Associativity::Left);
        let json_str = NodeTypesGenerator::new(&g).generate().unwrap();
        let val: Value = serde_json::from_str(&json_str).unwrap();
        let re = serde_json::to_string(&val).unwrap();
        let val2: Value = serde_json::from_str(&re).unwrap();
        prop_assert_eq!(val, val2, "prec NodeTypes must roundtrip");
    }

    #[test]
    fn prop_roundtrip_recursive_node_types(idx in name_index()) {
        let name = grammar_name(idx);
        let (g, _pt) = recursive_pipeline(&name);
        let json_str = NodeTypesGenerator::new(&g).generate().unwrap();
        let val: Value = serde_json::from_str(&json_str).unwrap();
        let re = serde_json::to_string_pretty(&val).unwrap();
        let val2: Value = serde_json::from_str(&re).unwrap();
        prop_assert_eq!(val, val2, "recursive NodeTypes must roundtrip");
    }

    #[test]
    fn prop_roundtrip_multi_nt_node_types(idx in name_index()) {
        let name = grammar_name(idx);
        let (g, _pt) = multi_nt_pipeline(&name);
        let json_str = NodeTypesGenerator::new(&g).generate().unwrap();
        let val: Value = serde_json::from_str(&json_str).unwrap();
        let re = serde_json::to_string_pretty(&val).unwrap();
        let val2: Value = serde_json::from_str(&re).unwrap();
        prop_assert_eq!(val, val2, "multi-NT NodeTypes must roundtrip");
    }
}
