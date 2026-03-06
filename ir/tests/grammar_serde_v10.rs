//! Comprehensive tests for Grammar serialization/deserialization (v10).

use adze_ir::builder::GrammarBuilder;
use adze_ir::{Associativity, Grammar};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn roundtrip(g: &Grammar) -> Grammar {
    let json = serde_json::to_string(g).expect("serialize");
    serde_json::from_str(&json).expect("deserialize")
}

fn make_simple() -> Grammar {
    GrammarBuilder::new("gs_v10_simple")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build()
}

fn make_tokens() -> Grammar {
    GrammarBuilder::new("gs_v10_tokens")
        .token("NUM", r"\d+")
        .token("ID", r"[a-z]+")
        .token("+", "+")
        .rule("s", vec!["NUM"])
        .start("s")
        .build()
}

fn make_rules() -> Grammar {
    GrammarBuilder::new("gs_v10_rules")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a"])
        .rule("s", vec!["b"])
        .rule("s", vec!["a", "b"])
        .start("s")
        .build()
}

fn make_prec() -> Grammar {
    GrammarBuilder::new("gs_v10_prec")
        .token("NUM", r"\d+")
        .token("+", "+")
        .token("*", "*")
        .rule_with_precedence("expr", vec!["expr", "+", "expr"], 1, Associativity::Left)
        .rule_with_precedence("expr", vec!["expr", "*", "expr"], 2, Associativity::Left)
        .rule("expr", vec!["NUM"])
        .start("expr")
        .build()
}

fn make_inline() -> Grammar {
    GrammarBuilder::new("gs_v10_inline")
        .token("a", "a")
        .rule("s", vec!["helper"])
        .rule("helper", vec!["a"])
        .inline("helper")
        .start("s")
        .build()
}

fn make_extras() -> Grammar {
    GrammarBuilder::new("gs_v10_extras")
        .token("a", "a")
        .token("WS", r"[ \t]+")
        .extra("WS")
        .rule("s", vec!["a"])
        .start("s")
        .build()
}

fn make_externals() -> Grammar {
    GrammarBuilder::new("gs_v10_externals")
        .token("a", "a")
        .external("INDENT")
        .external("DEDENT")
        .rule("s", vec!["a"])
        .start("s")
        .build()
}

fn make_supertypes() -> Grammar {
    GrammarBuilder::new("gs_v10_super")
        .token("a", "a")
        .token("b", "b")
        .rule("expr", vec!["a"])
        .rule("expr", vec!["b"])
        .supertype("expr")
        .start("expr")
        .build()
}

fn make_conflicts() -> Grammar {
    GrammarBuilder::new("gs_v10_conflicts")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["a"])
        .rule("s", vec!["b"])
        .start("s")
        .build()
}

fn make_large() -> Grammar {
    let mut b = GrammarBuilder::new("gs_v10_large");
    for i in 0..20 {
        let tok = format!("t{i}");
        b = b.token(&tok, &tok);
    }
    for i in 0..20 {
        let tok = format!("t{i}");
        b = b.rule("s", vec![&tok]);
    }
    b.start("s").build()
}

// ===========================================================================
// 1. Default grammar roundtrip
// ===========================================================================
#[test]
fn test_default_grammar_roundtrip() {
    let g = Grammar::default();
    let g2 = roundtrip(&g);
    assert_eq!(g, g2);
}

// ===========================================================================
// 2. Simple grammar roundtrip
// ===========================================================================
#[test]
fn test_simple_grammar_roundtrip() {
    let g = make_simple();
    let g2 = roundtrip(&g);
    assert_eq!(g, g2);
}

// ===========================================================================
// 3. Grammar with tokens roundtrip
// ===========================================================================
#[test]
fn test_tokens_grammar_roundtrip() {
    let g = make_tokens();
    let g2 = roundtrip(&g);
    assert_eq!(g, g2);
}

// ===========================================================================
// 4. Grammar with rules roundtrip
// ===========================================================================
#[test]
fn test_rules_grammar_roundtrip() {
    let g = make_rules();
    let g2 = roundtrip(&g);
    assert_eq!(g, g2);
}

// ===========================================================================
// 5. Grammar with precedence roundtrip
// ===========================================================================
#[test]
fn test_precedence_grammar_roundtrip() {
    let g = make_prec();
    let g2 = roundtrip(&g);
    assert_eq!(g, g2);
}

// ===========================================================================
// 6. Grammar with inline roundtrip
// ===========================================================================
#[test]
fn test_inline_grammar_roundtrip() {
    let g = make_inline();
    let g2 = roundtrip(&g);
    assert_eq!(g, g2);
}

// ===========================================================================
// 7. Grammar with extras roundtrip
// ===========================================================================
#[test]
fn test_extras_grammar_roundtrip() {
    let g = make_extras();
    let g2 = roundtrip(&g);
    assert_eq!(g, g2);
}

// ===========================================================================
// 8. Grammar with externals roundtrip
// ===========================================================================
#[test]
fn test_externals_grammar_roundtrip() {
    let g = make_externals();
    let g2 = roundtrip(&g);
    assert_eq!(g, g2);
}

// ===========================================================================
// 9. Grammar with supertypes roundtrip
// ===========================================================================
#[test]
fn test_supertypes_grammar_roundtrip() {
    let g = make_supertypes();
    let g2 = roundtrip(&g);
    assert_eq!(g, g2);
}

// ===========================================================================
// 10. Grammar with conflicts roundtrip
// ===========================================================================
#[test]
fn test_conflicts_grammar_roundtrip() {
    let g = make_conflicts();
    let g2 = roundtrip(&g);
    assert_eq!(g, g2);
}

// ===========================================================================
// 11. JSON is valid UTF-8
// ===========================================================================
#[test]
fn test_json_valid_utf8() {
    let g = make_simple();
    let json = serde_json::to_string(&g).unwrap();
    assert!(std::str::from_utf8(json.as_bytes()).is_ok());
}

// ===========================================================================
// 12. JSON contains grammar name
// ===========================================================================
#[test]
fn test_json_contains_name() {
    let g = make_simple();
    let json = serde_json::to_string(&g).unwrap();
    assert!(json.contains("gs_v10_simple"));
}

// ===========================================================================
// 13. JSON is non-empty
// ===========================================================================
#[test]
fn test_json_non_empty() {
    let g = Grammar::default();
    let json = serde_json::to_string(&g).unwrap();
    assert!(!json.is_empty());
}

// ===========================================================================
// 14. Different grammars produce different JSON
// ===========================================================================
#[test]
fn test_different_grammars_different_json() {
    let j1 = serde_json::to_string(&make_simple()).unwrap();
    let j2 = serde_json::to_string(&make_tokens()).unwrap();
    assert_ne!(j1, j2);
}

// ===========================================================================
// 15. Same grammar produces same JSON (deterministic)
// ===========================================================================
#[test]
fn test_deterministic_json() {
    let g = make_prec();
    let j1 = serde_json::to_string(&g).unwrap();
    let j2 = serde_json::to_string(&g).unwrap();
    assert_eq!(j1, j2);
}

// ===========================================================================
// 16. Roundtrip preserves name
// ===========================================================================
#[test]
fn test_roundtrip_preserves_name() {
    let g = make_simple();
    let g2 = roundtrip(&g);
    assert_eq!(g.name, g2.name);
}

// ===========================================================================
// 17. Roundtrip preserves token count
// ===========================================================================
#[test]
fn test_roundtrip_preserves_token_count() {
    let g = make_tokens();
    let g2 = roundtrip(&g);
    assert_eq!(g.tokens.len(), g2.tokens.len());
}

// ===========================================================================
// 18. Roundtrip preserves rule count
// ===========================================================================
#[test]
fn test_roundtrip_preserves_rule_count() {
    let g = make_rules();
    let g2 = roundtrip(&g);
    assert_eq!(g.all_rules().count(), g2.all_rules().count());
}

// ===========================================================================
// 19. Roundtrip preserves start symbol
// ===========================================================================
#[test]
fn test_roundtrip_preserves_start_symbol() {
    let g = make_simple();
    let g2 = roundtrip(&g);
    assert_eq!(g.start_symbol(), g2.start_symbol());
}

// ===========================================================================
// 20. Large grammar (20 rules) roundtrip
// ===========================================================================
#[test]
fn test_large_grammar_roundtrip() {
    let g = make_large();
    let g2 = roundtrip(&g);
    assert_eq!(g, g2);
}

// ===========================================================================
// 21. Default grammar name is empty string
// ===========================================================================
#[test]
fn test_default_name_empty() {
    let g = Grammar::default();
    let g2 = roundtrip(&g);
    assert!(g2.name.is_empty());
}

// ===========================================================================
// 22. Default grammar has zero tokens
// ===========================================================================
#[test]
fn test_default_zero_tokens() {
    let g = Grammar::default();
    let g2 = roundtrip(&g);
    assert!(g2.tokens.is_empty());
}

// ===========================================================================
// 23. Default grammar has zero rules
// ===========================================================================
#[test]
fn test_default_zero_rules() {
    let g = Grammar::default();
    let g2 = roundtrip(&g);
    assert!(g2.rules.is_empty());
}

// ===========================================================================
// 24. Pretty-print roundtrip
// ===========================================================================
#[test]
fn test_pretty_print_roundtrip() {
    let g = make_prec();
    let json = serde_json::to_string_pretty(&g).unwrap();
    let g2: Grammar = serde_json::from_str(&json).unwrap();
    assert_eq!(g, g2);
}

// ===========================================================================
// 25. Pretty JSON contains newlines
// ===========================================================================
#[test]
fn test_pretty_json_has_newlines() {
    let g = make_simple();
    let json = serde_json::to_string_pretty(&g).unwrap();
    assert!(json.contains('\n'));
}

// ===========================================================================
// 26. Compact JSON has no newlines
// ===========================================================================
#[test]
fn test_compact_json_no_newlines() {
    let g = make_simple();
    let json = serde_json::to_string(&g).unwrap();
    assert!(!json.contains('\n'));
}

// ===========================================================================
// 27. Double roundtrip stability
// ===========================================================================
#[test]
fn test_double_roundtrip() {
    let g = make_prec();
    let j1 = serde_json::to_string(&g).unwrap();
    let g2: Grammar = serde_json::from_str(&j1).unwrap();
    let j2 = serde_json::to_string(&g2).unwrap();
    assert_eq!(j1, j2);
}

// ===========================================================================
// 28. Triple roundtrip stability
// ===========================================================================
#[test]
fn test_triple_roundtrip() {
    let g = make_large();
    let j1 = serde_json::to_string(&g).unwrap();
    let g2: Grammar = serde_json::from_str(&j1).unwrap();
    let j2 = serde_json::to_string(&g2).unwrap();
    let g3: Grammar = serde_json::from_str(&j2).unwrap();
    let j3 = serde_json::to_string(&g3).unwrap();
    assert_eq!(j1, j3);
}

// ===========================================================================
// 29. JSON value is object
// ===========================================================================
#[test]
fn test_json_is_object() {
    let g = make_simple();
    let val: serde_json::Value = serde_json::to_value(&g).unwrap();
    assert!(val.is_object());
}

// ===========================================================================
// 30. JSON value has name field
// ===========================================================================
#[test]
fn test_json_value_has_name_field() {
    let g = make_simple();
    let val: serde_json::Value = serde_json::to_value(&g).unwrap();
    assert_eq!(val["name"], "gs_v10_simple");
}

// ===========================================================================
// 31. JSON value has rules field
// ===========================================================================
#[test]
fn test_json_value_has_rules_field() {
    let g = make_simple();
    let val: serde_json::Value = serde_json::to_value(&g).unwrap();
    assert!(val.get("rules").is_some());
}

// ===========================================================================
// 32. JSON value has tokens field
// ===========================================================================
#[test]
fn test_json_value_has_tokens_field() {
    let g = make_simple();
    let val: serde_json::Value = serde_json::to_value(&g).unwrap();
    assert!(val.get("tokens").is_some());
}

// ===========================================================================
// 33. Roundtrip preserves extras count
// ===========================================================================
#[test]
fn test_roundtrip_preserves_extras() {
    let g = make_extras();
    let g2 = roundtrip(&g);
    assert_eq!(g.extras.len(), g2.extras.len());
}

// ===========================================================================
// 34. Roundtrip preserves externals count
// ===========================================================================
#[test]
fn test_roundtrip_preserves_externals() {
    let g = make_externals();
    let g2 = roundtrip(&g);
    assert_eq!(g.externals.len(), g2.externals.len());
}

// ===========================================================================
// 35. Roundtrip preserves supertypes count
// ===========================================================================
#[test]
fn test_roundtrip_preserves_supertypes() {
    let g = make_supertypes();
    let g2 = roundtrip(&g);
    assert_eq!(g.supertypes.len(), g2.supertypes.len());
}

// ===========================================================================
// 36. Roundtrip preserves inline_rules count
// ===========================================================================
#[test]
fn test_roundtrip_preserves_inline_rules() {
    let g = make_inline();
    let g2 = roundtrip(&g);
    assert_eq!(g.inline_rules.len(), g2.inline_rules.len());
}

// ===========================================================================
// 37. Roundtrip preserves precedences count
// ===========================================================================
#[test]
fn test_roundtrip_preserves_precedences() {
    let g = make_prec();
    let g2 = roundtrip(&g);
    assert_eq!(g.precedences.len(), g2.precedences.len());
}

// ===========================================================================
// 38. Roundtrip preserves rule_names count
// ===========================================================================
#[test]
fn test_roundtrip_preserves_rule_names() {
    let g = make_rules();
    let g2 = roundtrip(&g);
    assert_eq!(g.rule_names.len(), g2.rule_names.len());
}

// ===========================================================================
// 39. Roundtrip preserves max_alias_sequence_length
// ===========================================================================
#[test]
fn test_roundtrip_preserves_max_alias_sequence_length() {
    let g = make_simple();
    let g2 = roundtrip(&g);
    assert_eq!(g.max_alias_sequence_length, g2.max_alias_sequence_length);
}

// ===========================================================================
// 40. Roundtrip preserves symbol_registry
// ===========================================================================
#[test]
fn test_roundtrip_preserves_symbol_registry() {
    let g = make_simple();
    let g2 = roundtrip(&g);
    assert_eq!(g.symbol_registry, g2.symbol_registry);
}

// ===========================================================================
// 41. Roundtrip preserves conflicts
// ===========================================================================
#[test]
fn test_roundtrip_preserves_conflicts() {
    let g = make_conflicts();
    let g2 = roundtrip(&g);
    assert_eq!(g.conflicts.len(), g2.conflicts.len());
}

// ===========================================================================
// 42. Roundtrip preserves fields
// ===========================================================================
#[test]
fn test_roundtrip_preserves_fields() {
    let g = make_prec();
    let g2 = roundtrip(&g);
    assert_eq!(g.fields.len(), g2.fields.len());
}

// ===========================================================================
// 43. Roundtrip preserves alias_sequences
// ===========================================================================
#[test]
fn test_roundtrip_preserves_alias_sequences() {
    let g = make_simple();
    let g2 = roundtrip(&g);
    assert_eq!(g.alias_sequences.len(), g2.alias_sequences.len());
}

// ===========================================================================
// 44. Roundtrip preserves production_ids
// ===========================================================================
#[test]
fn test_roundtrip_preserves_production_ids() {
    let g = make_simple();
    let g2 = roundtrip(&g);
    assert_eq!(g.production_ids.len(), g2.production_ids.len());
}

// ===========================================================================
// 45. JSON contains "rules" key
// ===========================================================================
#[test]
fn test_json_contains_rules_key() {
    let g = make_simple();
    let json = serde_json::to_string(&g).unwrap();
    assert!(json.contains("\"rules\""));
}

// ===========================================================================
// 46. JSON contains "tokens" key
// ===========================================================================
#[test]
fn test_json_contains_tokens_key() {
    let g = make_simple();
    let json = serde_json::to_string(&g).unwrap();
    assert!(json.contains("\"tokens\""));
}

// ===========================================================================
// 47. JSON contains "extras" key
// ===========================================================================
#[test]
fn test_json_contains_extras_key() {
    let g = make_simple();
    let json = serde_json::to_string(&g).unwrap();
    assert!(json.contains("\"extras\""));
}

// ===========================================================================
// 48. JSON contains "externals" key
// ===========================================================================
#[test]
fn test_json_contains_externals_key() {
    let g = make_simple();
    let json = serde_json::to_string(&g).unwrap();
    assert!(json.contains("\"externals\""));
}

// ===========================================================================
// 49. JSON contains "precedences" key
// ===========================================================================
#[test]
fn test_json_contains_precedences_key() {
    let g = make_simple();
    let json = serde_json::to_string(&g).unwrap();
    assert!(json.contains("\"precedences\""));
}

// ===========================================================================
// 50. JSON contains "conflicts" key
// ===========================================================================
#[test]
fn test_json_contains_conflicts_key() {
    let g = make_simple();
    let json = serde_json::to_string(&g).unwrap();
    assert!(json.contains("\"conflicts\""));
}

// ===========================================================================
// 51. JSON contains "supertypes" key
// ===========================================================================
#[test]
fn test_json_contains_supertypes_key() {
    let g = make_simple();
    let json = serde_json::to_string(&g).unwrap();
    assert!(json.contains("\"supertypes\""));
}

// ===========================================================================
// 52. JSON contains "inline_rules" key
// ===========================================================================
#[test]
fn test_json_contains_inline_rules_key() {
    let g = make_simple();
    let json = serde_json::to_string(&g).unwrap();
    assert!(json.contains("\"inline_rules\""));
}

// ===========================================================================
// 53. Right associativity roundtrip
// ===========================================================================
#[test]
fn test_right_assoc_roundtrip() {
    let g = GrammarBuilder::new("gs_v10_right")
        .token("a", "a")
        .token("^", "^")
        .rule_with_precedence("s", vec!["s", "^", "s"], 1, Associativity::Right)
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let g2 = roundtrip(&g);
    assert_eq!(g, g2);
}

// ===========================================================================
// 54. None associativity roundtrip
// ===========================================================================
#[test]
fn test_none_assoc_roundtrip() {
    let g = GrammarBuilder::new("gs_v10_none")
        .token("a", "a")
        .token("==", "==")
        .rule_with_precedence("s", vec!["s", "==", "s"], 1, Associativity::None)
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let g2 = roundtrip(&g);
    assert_eq!(g, g2);
}

// ===========================================================================
// 55. Mixed associativity roundtrip
// ===========================================================================
#[test]
fn test_mixed_assoc_roundtrip() {
    let g = GrammarBuilder::new("gs_v10_mixed_assoc")
        .token("a", "a")
        .token("+", "+")
        .token("^", "^")
        .rule_with_precedence("s", vec!["s", "+", "s"], 1, Associativity::Left)
        .rule_with_precedence("s", vec!["s", "^", "s"], 2, Associativity::Right)
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let g2 = roundtrip(&g);
    assert_eq!(g, g2);
}

// ===========================================================================
// 56. Multiple extras roundtrip
// ===========================================================================
#[test]
fn test_multiple_extras_roundtrip() {
    let g = GrammarBuilder::new("gs_v10_multi_extras")
        .token("a", "a")
        .token("WS", r"[ \t]+")
        .token("NL", r"\n")
        .extra("WS")
        .extra("NL")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let g2 = roundtrip(&g);
    assert_eq!(g.extras.len(), g2.extras.len());
    assert_eq!(g, g2);
}

// ===========================================================================
// 57. Multiple externals roundtrip
// ===========================================================================
#[test]
fn test_multiple_externals_roundtrip() {
    let g = GrammarBuilder::new("gs_v10_multi_ext")
        .token("a", "a")
        .external("INDENT")
        .external("DEDENT")
        .external("NEWLINE")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let g2 = roundtrip(&g);
    assert_eq!(g.externals.len(), g2.externals.len());
    assert_eq!(g, g2);
}

// ===========================================================================
// 58. Grammar::new roundtrip
// ===========================================================================
#[test]
fn test_grammar_new_roundtrip() {
    let g = Grammar::new("gs_v10_new".into());
    let g2 = roundtrip(&g);
    assert_eq!(g, g2);
}

// ===========================================================================
// 59. Precedence levels preserved
// ===========================================================================
#[test]
fn test_precedence_levels_preserved() {
    let g = GrammarBuilder::new("gs_v10_prec_levels")
        .token("a", "a")
        .token("+", "+")
        .token("*", "*")
        .precedence(1, Associativity::Left, vec!["+"])
        .precedence(2, Associativity::Left, vec!["*"])
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let g2 = roundtrip(&g);
    assert_eq!(g.precedences.len(), g2.precedences.len());
}

// ===========================================================================
// 60. JSON starts with opening brace
// ===========================================================================
#[test]
fn test_json_starts_with_brace() {
    let g = make_simple();
    let json = serde_json::to_string(&g).unwrap();
    assert!(json.starts_with('{'));
}

// ===========================================================================
// 61. JSON ends with closing brace
// ===========================================================================
#[test]
fn test_json_ends_with_brace() {
    let g = make_simple();
    let json = serde_json::to_string(&g).unwrap();
    assert!(json.ends_with('}'));
}

// ===========================================================================
// 62. JSON is valid JSON (parses to Value)
// ===========================================================================
#[test]
fn test_json_is_valid() {
    let g = make_prec();
    let json = serde_json::to_string(&g).unwrap();
    let val: Result<serde_json::Value, _> = serde_json::from_str(&json);
    assert!(val.is_ok());
}

// ===========================================================================
// 63. Empty tokens grammar roundtrip
// ===========================================================================
#[test]
fn test_no_tokens_roundtrip() {
    let g = Grammar::new("gs_v10_no_tok".into());
    let g2 = roundtrip(&g);
    assert!(g2.tokens.is_empty());
}

// ===========================================================================
// 64. Empty rules grammar roundtrip
// ===========================================================================
#[test]
fn test_no_rules_roundtrip() {
    let g = Grammar::new("gs_v10_no_rules".into());
    let g2 = roundtrip(&g);
    assert!(g2.rules.is_empty());
}

// ===========================================================================
// 65. Serialize default grammar size is small
// ===========================================================================
#[test]
fn test_default_json_small() {
    let g = Grammar::default();
    let json = serde_json::to_string(&g).unwrap();
    assert!(json.len() < 500);
}

// ===========================================================================
// 66. Large grammar JSON is substantial
// ===========================================================================
#[test]
fn test_large_json_substantial() {
    let g = make_large();
    let json = serde_json::to_string(&g).unwrap();
    assert!(json.len() > 100);
}

// ===========================================================================
// 67. Roundtrip equality is symmetric
// ===========================================================================
#[test]
fn test_roundtrip_equality_symmetric() {
    let g1 = make_simple();
    let g2 = roundtrip(&g1);
    assert_eq!(g1, g2);
    assert_eq!(g2, g1);
}

// ===========================================================================
// 68. Two different builder grammars stay different after roundtrip
// ===========================================================================
#[test]
fn test_different_grammars_stay_different() {
    let g1 = roundtrip(&make_simple());
    let g2 = roundtrip(&make_tokens());
    assert_ne!(g1, g2);
}

// ===========================================================================
// 69. Regex token pattern roundtrip
// ===========================================================================
#[test]
fn test_regex_token_roundtrip() {
    let g = GrammarBuilder::new("gs_v10_regex")
        .token("NUM", r"\d+(\.\d+)?")
        .rule("s", vec!["NUM"])
        .start("s")
        .build();
    let g2 = roundtrip(&g);
    assert_eq!(g, g2);
}

// ===========================================================================
// 70. String token pattern roundtrip
// ===========================================================================
#[test]
fn test_string_token_roundtrip() {
    let g = GrammarBuilder::new("gs_v10_str_tok")
        .token("PLUS", "+")
        .token("MINUS", "-")
        .rule("s", vec!["PLUS"])
        .start("s")
        .build();
    let g2 = roundtrip(&g);
    assert_eq!(g, g2);
}

// ===========================================================================
// 71. Many rules same LHS roundtrip
// ===========================================================================
#[test]
fn test_many_alternatives_roundtrip() {
    let mut b = GrammarBuilder::new("gs_v10_many_alts");
    for i in 0..10 {
        let tok = format!("t{i}");
        b = b.token(&tok, &tok);
    }
    for i in 0..10 {
        let tok = format!("t{i}");
        b = b.rule("s", vec![&tok]);
    }
    let g = b.start("s").build();
    let g2 = roundtrip(&g);
    assert_eq!(g.all_rules().count(), g2.all_rules().count());
    assert_eq!(g, g2);
}

// ===========================================================================
// 72. Multiple non-terminals roundtrip
// ===========================================================================
#[test]
fn test_multiple_nonterminals_roundtrip() {
    let g = GrammarBuilder::new("gs_v10_multi_nt")
        .token("a", "a")
        .token("b", "b")
        .rule("s", vec!["x", "y"])
        .rule("x", vec!["a"])
        .rule("y", vec!["b"])
        .start("s")
        .build();
    let g2 = roundtrip(&g);
    assert_eq!(g, g2);
}

// ===========================================================================
// 73. Builder preset python_like roundtrip
// ===========================================================================
#[test]
fn test_python_like_roundtrip() {
    let g = GrammarBuilder::python_like();
    let g2 = roundtrip(&g);
    assert_eq!(g, g2);
}

// ===========================================================================
// 74. Builder preset javascript_like roundtrip
// ===========================================================================
#[test]
fn test_javascript_like_roundtrip() {
    let g = GrammarBuilder::javascript_like();
    let g2 = roundtrip(&g);
    assert_eq!(g, g2);
}

// ===========================================================================
// 75. Normalized grammar roundtrip
// ===========================================================================
#[test]
fn test_normalized_grammar_roundtrip() {
    let mut g = make_prec();
    g.normalize();
    let g2 = roundtrip(&g);
    assert_eq!(g, g2);
}

// ===========================================================================
// 76. Serialization preserves token order
// ===========================================================================
#[test]
fn test_token_order_preserved() {
    let g = GrammarBuilder::new("gs_v10_tok_order")
        .token("alpha", "alpha")
        .token("beta", "beta")
        .token("gamma", "gamma")
        .rule("s", vec!["alpha"])
        .start("s")
        .build();
    let g2 = roundtrip(&g);
    let keys1: Vec<_> = g.tokens.keys().collect();
    let keys2: Vec<_> = g2.tokens.keys().collect();
    assert_eq!(keys1, keys2);
}

// ===========================================================================
// 77. Serialization preserves rule_names order
// ===========================================================================
#[test]
fn test_rule_names_order_preserved() {
    let g = make_rules();
    let g2 = roundtrip(&g);
    let keys1: Vec<_> = g.rule_names.keys().collect();
    let keys2: Vec<_> = g2.rule_names.keys().collect();
    assert_eq!(keys1, keys2);
}

// ===========================================================================
// 78. JSON name field matches grammar name
// ===========================================================================
#[test]
fn test_json_name_matches() {
    let g = GrammarBuilder::new("gs_v10_name_check")
        .token("a", "a")
        .rule("s", vec!["a"])
        .start("s")
        .build();
    let val: serde_json::Value = serde_json::to_value(&g).unwrap();
    assert_eq!(val["name"].as_str().unwrap(), "gs_v10_name_check");
}

// ===========================================================================
// 79. Extras grammar has non-empty extras after roundtrip
// ===========================================================================
#[test]
fn test_extras_non_empty_after_roundtrip() {
    let g = make_extras();
    let g2 = roundtrip(&g);
    assert!(!g2.extras.is_empty());
}

// ===========================================================================
// 80. Externals grammar has non-empty externals after roundtrip
// ===========================================================================
#[test]
fn test_externals_non_empty_after_roundtrip() {
    let g = make_externals();
    let g2 = roundtrip(&g);
    assert!(!g2.externals.is_empty());
}

// ===========================================================================
// 81. Inline grammar has non-empty inline_rules after roundtrip
// ===========================================================================
#[test]
fn test_inline_non_empty_after_roundtrip() {
    let g = make_inline();
    let g2 = roundtrip(&g);
    assert!(!g2.inline_rules.is_empty());
}

// ===========================================================================
// 82. Supertypes grammar has non-empty supertypes after roundtrip
// ===========================================================================
#[test]
fn test_supertypes_non_empty_after_roundtrip() {
    let g = make_supertypes();
    let g2 = roundtrip(&g);
    assert!(!g2.supertypes.is_empty());
}

// ===========================================================================
// 83. Compact and pretty deserialize to same grammar
// ===========================================================================
#[test]
fn test_compact_and_pretty_same_result() {
    let g = make_prec();
    let compact = serde_json::to_string(&g).unwrap();
    let pretty = serde_json::to_string_pretty(&g).unwrap();
    let g_compact: Grammar = serde_json::from_str(&compact).unwrap();
    let g_pretty: Grammar = serde_json::from_str(&pretty).unwrap();
    assert_eq!(g_compact, g_pretty);
}

// ===========================================================================
// 84. Large grammar preserves 20 rules
// ===========================================================================
#[test]
fn test_large_grammar_rule_count() {
    let g = make_large();
    let g2 = roundtrip(&g);
    assert_eq!(g2.all_rules().count(), 20);
}

// ===========================================================================
// 85. Large grammar preserves 20 tokens
// ===========================================================================
#[test]
fn test_large_grammar_token_count() {
    let g = make_large();
    let g2 = roundtrip(&g);
    assert_eq!(g2.tokens.len(), 20);
}

// ===========================================================================
// 86. Deserialized grammar can be re-serialized
// ===========================================================================
#[test]
fn test_deserialized_can_reserialize() {
    let g = make_prec();
    let json = serde_json::to_string(&g).unwrap();
    let g2: Grammar = serde_json::from_str(&json).unwrap();
    let json2 = serde_json::to_string(&g2);
    assert!(json2.is_ok());
}

// ===========================================================================
// 87. JSON "name" field is a string
// ===========================================================================
#[test]
fn test_json_name_is_string() {
    let g = make_simple();
    let val: serde_json::Value = serde_json::to_value(&g).unwrap();
    assert!(val["name"].is_string());
}

// ===========================================================================
// 88. JSON "rules" field is an object
// ===========================================================================
#[test]
fn test_json_rules_is_object() {
    let g = make_simple();
    let val: serde_json::Value = serde_json::to_value(&g).unwrap();
    assert!(val["rules"].is_object());
}

// ===========================================================================
// 89. JSON "tokens" field is an object
// ===========================================================================
#[test]
fn test_json_tokens_is_object() {
    let g = make_simple();
    let val: serde_json::Value = serde_json::to_value(&g).unwrap();
    assert!(val["tokens"].is_object());
}

// ===========================================================================
// 90. JSON "extras" field is an array
// ===========================================================================
#[test]
fn test_json_extras_is_array() {
    let g = make_simple();
    let val: serde_json::Value = serde_json::to_value(&g).unwrap();
    assert!(val["extras"].is_array());
}

// ===========================================================================
// 91. JSON "externals" field is an array
// ===========================================================================
#[test]
fn test_json_externals_is_array() {
    let g = make_simple();
    let val: serde_json::Value = serde_json::to_value(&g).unwrap();
    assert!(val["externals"].is_array());
}

// ===========================================================================
// 92. JSON "supertypes" field is an array
// ===========================================================================
#[test]
fn test_json_supertypes_is_array() {
    let g = make_simple();
    let val: serde_json::Value = serde_json::to_value(&g).unwrap();
    assert!(val["supertypes"].is_array());
}

// ===========================================================================
// 93. JSON "inline_rules" field is an array
// ===========================================================================
#[test]
fn test_json_inline_rules_is_array() {
    let g = make_simple();
    let val: serde_json::Value = serde_json::to_value(&g).unwrap();
    assert!(val["inline_rules"].is_array());
}

// ===========================================================================
// 94. JSON "precedences" field is an array
// ===========================================================================
#[test]
fn test_json_precedences_is_array() {
    let g = make_simple();
    let val: serde_json::Value = serde_json::to_value(&g).unwrap();
    assert!(val["precedences"].is_array());
}

// ===========================================================================
// 95. JSON "conflicts" field is an array
// ===========================================================================
#[test]
fn test_json_conflicts_is_array() {
    let g = make_simple();
    let val: serde_json::Value = serde_json::to_value(&g).unwrap();
    assert!(val["conflicts"].is_array());
}
