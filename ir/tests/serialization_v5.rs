//! Serialization v5 tests — Grammar JSON serialization/deserialization.
//!
//! 55+ tests covering roundtrip, structure, normalization, optimization,
//! large grammars, edge cases, determinism, and field preservation.

use adze_ir::builder::GrammarBuilder;
use adze_ir::{Associativity, Grammar};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Serialize to JSON string and deserialize back.
fn json_roundtrip(g: &Grammar) -> Grammar {
    let json = serde_json::to_string(g).expect("serialize");
    serde_json::from_str(&json).expect("deserialize")
}

/// Build a minimal grammar with one token and one rule.
fn minimal_grammar() -> Grammar {
    GrammarBuilder::new("minimal")
        .token("x", "x")
        .rule("start", vec!["x"])
        .start("start")
        .build()
}

/// Build an arithmetic grammar with operators and precedence.
fn arith_grammar() -> Grammar {
    GrammarBuilder::new("arith")
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

/// Build a grammar with extras, externals, and precedence declarations.
fn decorated_grammar() -> Grammar {
    GrammarBuilder::new("decorated")
        .token("id", r"[a-z]+")
        .token("ws", r"\s+")
        .token("semi", ";")
        .rule("program", vec!["stmt"])
        .rule("stmt", vec!["id", "semi"])
        .start("program")
        .extra("ws")
        .external("indent")
        .external("dedent")
        .precedence(1, Associativity::Left, vec!["id"])
        .build()
}

// ===========================================================================
// Category 1: JSON roundtrip — serialize → deserialize → compare
// ===========================================================================

#[test]
fn test_roundtrip_minimal_grammar_name() {
    let g = minimal_grammar();
    let g2 = json_roundtrip(&g);
    assert_eq!(g.name, g2.name);
}

#[test]
fn test_roundtrip_minimal_rules_count() {
    let g = minimal_grammar();
    let g2 = json_roundtrip(&g);
    assert_eq!(g.rules.len(), g2.rules.len());
}

#[test]
fn test_roundtrip_minimal_tokens_count() {
    let g = minimal_grammar();
    let g2 = json_roundtrip(&g);
    assert_eq!(g.tokens.len(), g2.tokens.len());
}

#[test]
fn test_roundtrip_arith_grammar_equality() {
    let g = arith_grammar();
    let g2 = json_roundtrip(&g);
    assert_eq!(g, g2);
}

#[test]
fn test_roundtrip_decorated_grammar_equality() {
    let g = decorated_grammar();
    let g2 = json_roundtrip(&g);
    assert_eq!(g, g2);
}

#[test]
fn test_roundtrip_preserves_rule_names() {
    let g = arith_grammar();
    let g2 = json_roundtrip(&g);
    assert_eq!(g.rule_names, g2.rule_names);
}

#[test]
fn test_roundtrip_preserves_token_names() {
    let g = arith_grammar();
    let g2 = json_roundtrip(&g);
    for (id, tok) in &g.tokens {
        let tok2 = &g2.tokens[id];
        assert_eq!(tok.name, tok2.name);
    }
}

#[test]
fn test_roundtrip_preserves_token_patterns() {
    let g = arith_grammar();
    let g2 = json_roundtrip(&g);
    for (id, tok) in &g.tokens {
        let tok2 = &g2.tokens[id];
        assert_eq!(tok.pattern, tok2.pattern);
    }
}

#[test]
fn test_roundtrip_pretty_json() {
    let g = minimal_grammar();
    let pretty = serde_json::to_string_pretty(&g).expect("pretty serialize");
    let g2: Grammar = serde_json::from_str(&pretty).expect("deserialize pretty");
    assert_eq!(g, g2);
}

#[test]
fn test_roundtrip_via_value() {
    let g = arith_grammar();
    let val = serde_json::to_value(&g).expect("to_value");
    let g2: Grammar = serde_json::from_value(val).expect("from_value");
    assert_eq!(g, g2);
}

// ===========================================================================
// Category 2: JSON structure — specific fields present in output
// ===========================================================================

#[test]
fn test_json_has_name_field() {
    let g = minimal_grammar();
    let val = serde_json::to_value(&g).expect("serialize");
    assert!(val.get("name").is_some());
}

#[test]
fn test_json_name_is_string() {
    let g = minimal_grammar();
    let val = serde_json::to_value(&g).expect("serialize");
    assert!(val["name"].is_string());
}

#[test]
fn test_json_has_rules_field() {
    let g = minimal_grammar();
    let val = serde_json::to_value(&g).expect("serialize");
    assert!(val.get("rules").is_some());
}

#[test]
fn test_json_rules_is_object() {
    let g = minimal_grammar();
    let val = serde_json::to_value(&g).expect("serialize");
    assert!(val["rules"].is_object());
}

#[test]
fn test_json_has_tokens_field() {
    let g = minimal_grammar();
    let val = serde_json::to_value(&g).expect("serialize");
    assert!(val.get("tokens").is_some());
}

#[test]
fn test_json_tokens_is_object() {
    let g = minimal_grammar();
    let val = serde_json::to_value(&g).expect("serialize");
    assert!(val["tokens"].is_object());
}

#[test]
fn test_json_has_precedences_field() {
    let g = minimal_grammar();
    let val = serde_json::to_value(&g).expect("serialize");
    assert!(val.get("precedences").is_some());
}

#[test]
fn test_json_has_extras_field() {
    let g = minimal_grammar();
    let val = serde_json::to_value(&g).expect("serialize");
    assert!(val.get("extras").is_some());
}

#[test]
fn test_json_has_externals_field() {
    let g = minimal_grammar();
    let val = serde_json::to_value(&g).expect("serialize");
    assert!(val.get("externals").is_some());
}

#[test]
fn test_json_has_conflicts_field() {
    let g = minimal_grammar();
    let val = serde_json::to_value(&g).expect("serialize");
    assert!(val.get("conflicts").is_some());
}

#[test]
fn test_json_rules_count_matches() {
    let g = arith_grammar();
    let val = serde_json::to_value(&g).expect("serialize");
    let rules_obj = val["rules"].as_object().expect("rules object");
    assert_eq!(rules_obj.len(), g.rules.len());
}

#[test]
fn test_json_tokens_count_matches() {
    let g = arith_grammar();
    let val = serde_json::to_value(&g).expect("serialize");
    let tokens_obj = val["tokens"].as_object().expect("tokens object");
    assert_eq!(tokens_obj.len(), g.tokens.len());
}

// ===========================================================================
// Category 3: Normalized grammar serialization roundtrip
// ===========================================================================

#[test]
fn test_normalized_grammar_roundtrip_equality() {
    let mut g = arith_grammar();
    let _aux = g.normalize();
    let g2 = json_roundtrip(&g);
    assert_eq!(g, g2);
}

#[test]
fn test_normalized_grammar_name_preserved() {
    let mut g = minimal_grammar();
    let _aux = g.normalize();
    let g2 = json_roundtrip(&g);
    assert_eq!(g.name, g2.name);
}

#[test]
fn test_normalized_grammar_rules_preserved() {
    let mut g = arith_grammar();
    let _aux = g.normalize();
    let g2 = json_roundtrip(&g);
    assert_eq!(g.rules.len(), g2.rules.len());
}

#[test]
fn test_normalized_grammar_tokens_preserved() {
    let mut g = arith_grammar();
    let _aux = g.normalize();
    let g2 = json_roundtrip(&g);
    assert_eq!(g.tokens.len(), g2.tokens.len());
}

// ===========================================================================
// Category 4: Optimized grammar serialization roundtrip
// ===========================================================================

#[test]
fn test_optimized_grammar_roundtrip_equality() {
    let mut g = arith_grammar();
    g.optimize();
    let g2 = json_roundtrip(&g);
    assert_eq!(g, g2);
}

#[test]
fn test_optimized_grammar_name_preserved() {
    let mut g = minimal_grammar();
    g.optimize();
    let g2 = json_roundtrip(&g);
    assert_eq!(g.name, g2.name);
}

#[test]
fn test_optimized_normalized_roundtrip() {
    let mut g = arith_grammar();
    g.optimize();
    let _aux = g.normalize();
    let g2 = json_roundtrip(&g);
    assert_eq!(g, g2);
}

#[test]
fn test_optimizer_struct_roundtrip() {
    use adze_ir::GrammarOptimizer;
    let mut g = arith_grammar();
    let mut opt = GrammarOptimizer::new();
    let _stats = opt.optimize(&mut g);
    let g2 = json_roundtrip(&g);
    assert_eq!(g, g2);
}

// ===========================================================================
// Category 5: Large grammars (10+ rules, 20+ tokens)
// ===========================================================================

fn large_grammar() -> Grammar {
    let mut b = GrammarBuilder::new("large");
    // 25 tokens
    for i in 0..25 {
        b = b.token(&format!("tok_{i}"), &format!("t{i}"));
    }
    // 12 rules — each references a different token
    for i in 0..12 {
        b = b.rule("program", vec![&format!("tok_{i}")]);
    }
    b.start("program").build()
}

#[test]
fn test_large_grammar_roundtrip_equality() {
    let g = large_grammar();
    let g2 = json_roundtrip(&g);
    assert_eq!(g, g2);
}

#[test]
fn test_large_grammar_token_count() {
    let g = large_grammar();
    assert!(g.tokens.len() >= 25);
    let g2 = json_roundtrip(&g);
    assert_eq!(g.tokens.len(), g2.tokens.len());
}

#[test]
fn test_large_grammar_rule_count() {
    let g = large_grammar();
    // rules are grouped by LHS, so the count is number of distinct LHS symbols
    let total: usize = g.rules.values().map(|v| v.len()).sum();
    assert!(total >= 12);
    let g2 = json_roundtrip(&g);
    let total2: usize = g2.rules.values().map(|v| v.len()).sum();
    assert_eq!(total, total2);
}

#[test]
fn test_large_grammar_json_is_valid() {
    let g = large_grammar();
    let json = serde_json::to_string(&g).expect("serialize");
    let val: serde_json::Value = serde_json::from_str(&json).expect("parse json");
    assert!(val.is_object());
}

#[test]
fn test_large_grammar_all_tokens_survive() {
    let g = large_grammar();
    let g2 = json_roundtrip(&g);
    for (id, tok) in &g.tokens {
        let tok2 = &g2.tokens[id];
        assert_eq!(tok.name, tok2.name);
        assert_eq!(tok.pattern, tok2.pattern);
    }
}

#[test]
fn test_large_grammar_with_many_rules() {
    let mut b = GrammarBuilder::new("many_rules");
    b = b.token("a", "a").token("b", "b").token("c", "c");
    for i in 0..15 {
        let name = format!("rule_{i}");
        // Leak the string so we get a &'static str for the vec
        let name_ref: &'static str = Box::leak(name.into_boxed_str());
        b = b.rule(name_ref, vec!["a", "b"]);
    }
    b = b.start("rule_0");
    let g = b.build();
    let g2 = json_roundtrip(&g);
    assert_eq!(g, g2);
}

// ===========================================================================
// Category 6: Edge cases
// ===========================================================================

#[test]
fn test_empty_grammar_roundtrip() {
    let g = Grammar::new("empty".into());
    let g2 = json_roundtrip(&g);
    assert_eq!(g.name, g2.name);
    assert!(g2.rules.is_empty());
    assert!(g2.tokens.is_empty());
}

#[test]
fn test_empty_grammar_json_structure() {
    let g = Grammar::new("empty".into());
    let val = serde_json::to_value(&g).expect("serialize");
    assert_eq!(val["name"].as_str().unwrap(), "empty");
    assert!(val["rules"].as_object().unwrap().is_empty());
    assert!(val["tokens"].as_object().unwrap().is_empty());
}

#[test]
fn test_single_token_grammar_roundtrip() {
    let g = GrammarBuilder::new("single_token")
        .token("tok", "hello")
        .rule("start", vec!["tok"])
        .start("start")
        .build();
    let g2 = json_roundtrip(&g);
    assert_eq!(g, g2);
}

#[test]
fn test_unicode_grammar_name() {
    let g = GrammarBuilder::new("grammaire_française")
        .token("mot", r"[a-zàâéèêëîïôùûüç]+")
        .rule("phrase", vec!["mot"])
        .start("phrase")
        .build();
    let g2 = json_roundtrip(&g);
    assert_eq!(g2.name, "grammaire_française");
}

#[test]
fn test_unicode_token_name() {
    let g = GrammarBuilder::new("unicode_tok")
        .token("数字", r"\d+")
        .rule("式", vec!["数字"])
        .start("式")
        .build();
    let g2 = json_roundtrip(&g);
    assert_eq!(g, g2);
}

#[test]
fn test_empty_string_name() {
    let g = Grammar::new(String::new());
    let g2 = json_roundtrip(&g);
    assert!(g2.name.is_empty());
}

#[test]
fn test_special_chars_in_token_pattern() {
    let g = GrammarBuilder::new("special")
        .token("regex_tok", r#"["\\/\n\t\r]+"#)
        .rule("start", vec!["regex_tok"])
        .start("start")
        .build();
    let g2 = json_roundtrip(&g);
    assert_eq!(g, g2);
}

#[test]
fn test_long_grammar_name() {
    let long_name = "a".repeat(1000);
    let g = Grammar::new(long_name.clone());
    let g2 = json_roundtrip(&g);
    assert_eq!(g2.name, long_name);
}

#[test]
fn test_grammar_default_roundtrip() {
    let g = Grammar::default();
    let g2 = json_roundtrip(&g);
    assert_eq!(g, g2);
}

// ===========================================================================
// Category 7: Determinism — same grammar → same JSON output
// ===========================================================================

#[test]
fn test_deterministic_json_output_minimal() {
    let g = minimal_grammar();
    let json1 = serde_json::to_string(&g).expect("serialize 1");
    let json2 = serde_json::to_string(&g).expect("serialize 2");
    assert_eq!(json1, json2);
}

#[test]
fn test_deterministic_json_output_arith() {
    let g = arith_grammar();
    let json1 = serde_json::to_string(&g).expect("serialize 1");
    let json2 = serde_json::to_string(&g).expect("serialize 2");
    assert_eq!(json1, json2);
}

#[test]
fn test_deterministic_json_output_decorated() {
    let g = decorated_grammar();
    let json1 = serde_json::to_string(&g).expect("serialize 1");
    let json2 = serde_json::to_string(&g).expect("serialize 2");
    assert_eq!(json1, json2);
}

#[test]
fn test_deterministic_json_output_large() {
    let g = large_grammar();
    let json1 = serde_json::to_string(&g).expect("serialize 1");
    let json2 = serde_json::to_string(&g).expect("serialize 2");
    assert_eq!(json1, json2);
}

#[test]
fn test_deterministic_pretty_json() {
    let g = arith_grammar();
    let p1 = serde_json::to_string_pretty(&g).expect("pretty 1");
    let p2 = serde_json::to_string_pretty(&g).expect("pretty 2");
    assert_eq!(p1, p2);
}

#[test]
fn test_deterministic_after_roundtrip() {
    let g = arith_grammar();
    let json1 = serde_json::to_string(&g).expect("first");
    let g2: Grammar = serde_json::from_str(&json1).expect("deser");
    let json2 = serde_json::to_string(&g2).expect("second");
    assert_eq!(json1, json2);
}

// ===========================================================================
// Category 8: Field preservation — extras, externals, precedences survive
// ===========================================================================

#[test]
fn test_extras_survive_roundtrip() {
    let g = decorated_grammar();
    assert!(!g.extras.is_empty());
    let g2 = json_roundtrip(&g);
    assert_eq!(g.extras.len(), g2.extras.len());
    for (a, b) in g.extras.iter().zip(g2.extras.iter()) {
        assert_eq!(*a, *b);
    }
}

#[test]
fn test_externals_survive_roundtrip() {
    let g = decorated_grammar();
    assert!(!g.externals.is_empty());
    let g2 = json_roundtrip(&g);
    assert_eq!(g.externals.len(), g2.externals.len());
    for (a, b) in g.externals.iter().zip(g2.externals.iter()) {
        assert_eq!(a.name, b.name);
        assert_eq!(a.symbol_id, b.symbol_id);
    }
}

#[test]
fn test_precedences_survive_roundtrip() {
    let g = decorated_grammar();
    assert!(!g.precedences.is_empty());
    let g2 = json_roundtrip(&g);
    assert_eq!(g.precedences.len(), g2.precedences.len());
    for (a, b) in g.precedences.iter().zip(g2.precedences.iter()) {
        assert_eq!(a.level, b.level);
        assert_eq!(a.associativity, b.associativity);
        assert_eq!(a.symbols, b.symbols);
    }
}

#[test]
fn test_precedence_with_associativity_roundtrip() {
    let g = GrammarBuilder::new("prec_assoc")
        .token("num", r"\d+")
        .token("plus", "+")
        .token("minus", "-")
        .token("star", "*")
        .rule("expr", vec!["num"])
        .rule_with_precedence("expr", vec!["expr", "plus", "expr"], 1, Associativity::Left)
        .rule_with_precedence(
            "expr",
            vec!["expr", "minus", "expr"],
            1,
            Associativity::Left,
        )
        .rule_with_precedence("expr", vec!["expr", "star", "expr"], 2, Associativity::Left)
        .start("expr")
        .build();
    let g2 = json_roundtrip(&g);
    assert_eq!(g, g2);
}

#[test]
fn test_right_associativity_preserved() {
    let g = GrammarBuilder::new("right_assoc")
        .token("num", r"\d+")
        .token("caret", "^")
        .rule("expr", vec!["num"])
        .rule_with_precedence(
            "expr",
            vec!["expr", "caret", "expr"],
            3,
            Associativity::Right,
        )
        .start("expr")
        .build();
    let g2 = json_roundtrip(&g);
    assert_eq!(g, g2);
}

#[test]
fn test_multiple_extras_survive() {
    let g = GrammarBuilder::new("multi_extra")
        .token("id", r"[a-z]+")
        .token("ws", r"\s+")
        .token("comment", r"//[^\n]*")
        .rule("start", vec!["id"])
        .start("start")
        .extra("ws")
        .extra("comment")
        .build();
    let g2 = json_roundtrip(&g);
    assert_eq!(g.extras.len(), g2.extras.len());
    assert_eq!(g, g2);
}

#[test]
fn test_multiple_externals_survive() {
    let g = GrammarBuilder::new("multi_ext")
        .token("id", r"[a-z]+")
        .rule("block", vec!["id"])
        .start("block")
        .external("indent")
        .external("dedent")
        .external("newline")
        .build();
    let g2 = json_roundtrip(&g);
    assert_eq!(g.externals.len(), g2.externals.len());
    assert_eq!(g, g2);
}

#[test]
fn test_inline_rules_survive_roundtrip() {
    let g = GrammarBuilder::new("inline_test")
        .token("a", "a")
        .token("b", "b")
        .rule("start", vec!["helper"])
        .rule("helper", vec!["a", "b"])
        .start("start")
        .inline("helper")
        .build();
    let g2 = json_roundtrip(&g);
    assert_eq!(g.inline_rules.len(), g2.inline_rules.len());
    assert_eq!(g, g2);
}

#[test]
fn test_supertypes_survive_roundtrip() {
    let g = GrammarBuilder::new("supertype_test")
        .token("id", r"[a-z]+")
        .token("num", r"\d+")
        .rule("expression", vec!["id"])
        .rule("expression", vec!["num"])
        .start("expression")
        .supertype("expression")
        .build();
    let g2 = json_roundtrip(&g);
    assert_eq!(g.supertypes.len(), g2.supertypes.len());
    assert_eq!(g, g2);
}

#[test]
fn test_conflicts_survive_roundtrip() {
    let g = decorated_grammar();
    let g2 = json_roundtrip(&g);
    assert_eq!(g.conflicts.len(), g2.conflicts.len());
}

#[test]
fn test_fields_survive_roundtrip() {
    let g = arith_grammar();
    let g2 = json_roundtrip(&g);
    assert_eq!(g.fields, g2.fields);
}

#[test]
fn test_production_ids_survive_roundtrip() {
    let g = arith_grammar();
    let g2 = json_roundtrip(&g);
    assert_eq!(g.production_ids, g2.production_ids);
}

#[test]
fn test_alias_sequences_survive_roundtrip() {
    let g = arith_grammar();
    let g2 = json_roundtrip(&g);
    assert_eq!(g.alias_sequences, g2.alias_sequences);
}

#[test]
fn test_max_alias_sequence_length_survives() {
    let g = arith_grammar();
    let g2 = json_roundtrip(&g);
    assert_eq!(g.max_alias_sequence_length, g2.max_alias_sequence_length);
}

#[test]
fn test_symbol_registry_survives_roundtrip() {
    let g = arith_grammar();
    let g2 = json_roundtrip(&g);
    assert_eq!(g.symbol_registry, g2.symbol_registry);
}
