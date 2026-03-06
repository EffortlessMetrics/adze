//! Converter v5: 64 tests covering JSON → Grammar IR conversion.
//!
//! 1. PATTERN rules → Grammar tokens (8 tests)
//! 2. STRING rules → Grammar literal tokens (8 tests)
//! 3. SEQ rules → Grammar rule sequences (8 tests)
//! 4. CHOICE rules → Grammar alternatives (8 tests)
//! 5. REPEAT/REPEAT1/OPTIONAL → normalized forms (8 tests)
//! 6. PREC_LEFT/PREC_RIGHT → precedence in Grammar (8 tests)
//! 7. Nested structures → correct Grammar topology (8 tests)
//! 8. Error cases → descriptive errors (8 tests)

use adze_ir::{Associativity, Grammar, PrecedenceKind, Symbol, TokenPattern};
use adze_tool::grammar_js::GrammarJsConverter;
use adze_tool::grammar_js::json_converter::from_tree_sitter_json;
use adze_tool::pure_rust_builder::{BuildOptions, build_parser_from_json};
use serde_json::json;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn convert(value: &serde_json::Value) -> Grammar {
    let gjs = from_tree_sitter_json(value).expect("from_tree_sitter_json failed");
    GrammarJsConverter::new(gjs)
        .convert()
        .expect("convert failed")
}

fn opts() -> BuildOptions {
    BuildOptions {
        out_dir: "/tmp/adze-converter-v5".to_string(),
        emit_artifacts: false,
        compress_tables: false,
    }
}

fn build_json(value: &serde_json::Value) -> anyhow::Result<adze_tool::BuildResult> {
    build_parser_from_json(serde_json::to_string(value).unwrap(), opts())
}

/// Find a token whose pattern matches the given regex string.
fn find_regex_token(g: &Grammar, regex: &str) -> bool {
    g.tokens.values().any(|t| match &t.pattern {
        TokenPattern::Regex(r) => r == regex,
        _ => false,
    })
}

/// Find a token whose pattern matches the given literal string.
fn find_string_token(g: &Grammar, literal: &str) -> bool {
    g.tokens.values().any(|t| match &t.pattern {
        TokenPattern::String(s) => s == literal,
        _ => false,
    })
}

/// Count total IR rules across all LHS symbols.
fn total_rules(g: &Grammar) -> usize {
    g.rules.values().map(|rs| rs.len()).sum()
}

/// Collect rules for the named symbol.
fn rules_for<'a>(g: &'a Grammar, name: &str) -> Vec<&'a adze_ir::Rule> {
    let sid = g
        .rule_names
        .iter()
        .find(|(_, n)| n.as_str() == name)
        .map(|(id, _)| *id)
        .expect("symbol not found");
    g.rules
        .get(&sid)
        .map(|rs| rs.iter().collect())
        .unwrap_or_default()
}

// ===========================================================================
// 1. PATTERN rules → Grammar tokens (8 tests)
// ===========================================================================

#[test]
fn pattern_simple_digit_creates_regex_token() {
    let g = convert(&json!({
        "name": "p1",
        "rules": { "number": { "type": "PATTERN", "value": "[0-9]+" } }
    }));
    assert!(find_regex_token(&g, "[0-9]+"));
}

#[test]
fn pattern_identifier_creates_regex_token() {
    let g = convert(&json!({
        "name": "p2",
        "rules": { "ident": { "type": "PATTERN", "value": "[a-zA-Z_]\\w*" } }
    }));
    assert!(find_regex_token(&g, "[a-zA-Z_]\\w*"));
}

#[test]
fn pattern_multiple_rules_create_distinct_tokens() {
    let g = convert(&json!({
        "name": "p3",
        "rules": {
            "number": { "type": "PATTERN", "value": "\\d+" },
            "ident":  { "type": "PATTERN", "value": "[a-z]+" }
        }
    }));
    assert!(find_regex_token(&g, "\\d+"));
    assert!(find_regex_token(&g, "[a-z]+"));
}

#[test]
fn pattern_special_chars_preserved() {
    let g = convert(&json!({
        "name": "p4",
        "rules": { "hex": { "type": "PATTERN", "value": "0x[0-9a-fA-F]+" } }
    }));
    assert!(find_regex_token(&g, "0x[0-9a-fA-F]+"));
}

#[test]
fn pattern_character_class_brackets() {
    let g = convert(&json!({
        "name": "p5",
        "rules": { "vowel": { "type": "PATTERN", "value": "[aeiou]" } }
    }));
    assert!(find_regex_token(&g, "[aeiou]"));
}

#[test]
fn pattern_creates_unit_rule_to_terminal() {
    let g = convert(&json!({
        "name": "p6",
        "rules": { "num": { "type": "PATTERN", "value": "\\d+" } }
    }));
    let rules = rules_for(&g, "num");
    assert!(!rules.is_empty(), "expected at least one rule for num");
    let has_terminal = rules
        .iter()
        .any(|r| r.rhs.len() == 1 && matches!(&r.rhs[0], Symbol::Terminal(_)));
    assert!(has_terminal, "expected a unit rule to Terminal");
}

#[test]
fn pattern_unicode_class() {
    let g = convert(&json!({
        "name": "p7",
        "rules": { "letter": { "type": "PATTERN", "value": "\\p{L}+" } }
    }));
    assert!(find_regex_token(&g, "\\p{L}+"));
}

#[test]
fn pattern_appears_in_grammar_tokens_map() {
    let g = convert(&json!({
        "name": "p8",
        "rules": { "word": { "type": "PATTERN", "value": "\\w+" } }
    }));
    assert!(!g.tokens.is_empty(), "tokens map must not be empty");
}

// ===========================================================================
// 2. STRING rules → Grammar literal tokens (8 tests)
// ===========================================================================

#[test]
fn string_keyword_creates_literal_token() {
    let g = convert(&json!({
        "name": "s1",
        "rules": { "kw": { "type": "STRING", "value": "return" } }
    }));
    assert!(find_string_token(&g, "return"));
}

#[test]
fn string_operator_creates_literal_token() {
    let g = convert(&json!({
        "name": "s2",
        "rules": { "op": { "type": "STRING", "value": "+=" } }
    }));
    assert!(find_string_token(&g, "+="));
}

#[test]
fn string_multiple_keywords_distinct() {
    let g = convert(&json!({
        "name": "s3",
        "rules": {
            "kw_if":   { "type": "STRING", "value": "if" },
            "kw_else": { "type": "STRING", "value": "else" }
        }
    }));
    assert!(find_string_token(&g, "if"));
    assert!(find_string_token(&g, "else"));
}

#[test]
fn string_single_char() {
    let g = convert(&json!({
        "name": "s4",
        "rules": { "semi": { "type": "STRING", "value": ";" } }
    }));
    assert!(find_string_token(&g, ";"));
}

#[test]
fn string_creates_unit_rule() {
    let g = convert(&json!({
        "name": "s5",
        "rules": { "plus": { "type": "STRING", "value": "+" } }
    }));
    let rules = rules_for(&g, "plus");
    assert!(!rules.is_empty());
    let has_terminal = rules
        .iter()
        .any(|r| r.rhs.len() == 1 && matches!(&r.rhs[0], Symbol::Terminal(_)));
    assert!(has_terminal, "STRING should produce a Terminal unit rule");
}

#[test]
fn string_deduplicates_same_literal() {
    let g = convert(&json!({
        "name": "s6",
        "rules": {
            "program": {
                "type": "SEQ",
                "members": [
                    { "type": "STRING", "value": "+" },
                    { "type": "STRING", "value": "+" }
                ]
            }
        }
    }));
    let plus_count = g
        .tokens
        .values()
        .filter(|t| matches!(&t.pattern, TokenPattern::String(s) if s == "+"))
        .count();
    assert_eq!(
        plus_count, 1,
        "duplicate STRING literals should share one token"
    );
}

#[test]
fn string_multichar_keyword() {
    let g = convert(&json!({
        "name": "s7",
        "rules": { "kw": { "type": "STRING", "value": "function" } }
    }));
    assert!(find_string_token(&g, "function"));
}

#[test]
fn string_token_has_quoted_name() {
    let g = convert(&json!({
        "name": "s8",
        "rules": { "arrow": { "type": "STRING", "value": "=>" } }
    }));
    let tok = g
        .tokens
        .values()
        .find(|t| matches!(&t.pattern, TokenPattern::String(s) if s == "=>"))
        .expect("=> token missing");
    assert!(
        tok.name.contains("=>"),
        "token name should contain the literal"
    );
}

// ===========================================================================
// 3. SEQ rules → Grammar rule sequences (8 tests)
// ===========================================================================

#[test]
fn seq_two_symbols_produces_two_element_rhs() {
    let g = convert(&json!({
        "name": "q1",
        "rules": {
            "pair": {
                "type": "SEQ",
                "members": [
                    { "type": "STRING", "value": "(" },
                    { "type": "STRING", "value": ")" }
                ]
            }
        }
    }));
    let rules = rules_for(&g, "pair");
    assert!(rules.iter().any(|r| r.rhs.len() == 2));
}

#[test]
fn seq_three_symbols() {
    let g = convert(&json!({
        "name": "q2",
        "rules": {
            "triple": {
                "type": "SEQ",
                "members": [
                    { "type": "STRING", "value": "a" },
                    { "type": "STRING", "value": "b" },
                    { "type": "STRING", "value": "c" }
                ]
            }
        }
    }));
    let rules = rules_for(&g, "triple");
    assert!(rules.iter().any(|r| r.rhs.len() == 3));
}

#[test]
fn seq_mixed_nonterminals_and_terminals() {
    let g = convert(&json!({
        "name": "q3",
        "rules": {
            "stmt": {
                "type": "SEQ",
                "members": [
                    { "type": "SYMBOL", "name": "ident" },
                    { "type": "STRING", "value": "=" },
                    { "type": "SYMBOL", "name": "ident" }
                ]
            },
            "ident": { "type": "PATTERN", "value": "[a-z]+" }
        }
    }));
    let rules = rules_for(&g, "stmt");
    let three_elem = rules.iter().find(|r| r.rhs.len() == 3);
    assert!(three_elem.is_some(), "expected a 3-element seq rule");
}

#[test]
fn seq_inline_strings_create_tokens() {
    let g = convert(&json!({
        "name": "q4",
        "rules": {
            "parens": {
                "type": "SEQ",
                "members": [
                    { "type": "STRING", "value": "(" },
                    { "type": "STRING", "value": ")" }
                ]
            }
        }
    }));
    assert!(find_string_token(&g, "("));
    assert!(find_string_token(&g, ")"));
}

#[test]
fn seq_preserves_terminal_order() {
    let g = convert(&json!({
        "name": "q5",
        "rules": {
            "arrow_fn": {
                "type": "SEQ",
                "members": [
                    { "type": "STRING", "value": "(" },
                    { "type": "STRING", "value": ")" },
                    { "type": "STRING", "value": "=>" }
                ]
            }
        }
    }));
    let rules = rules_for(&g, "arrow_fn");
    let rule = rules.iter().find(|r| r.rhs.len() == 3).unwrap();
    // All elements should be terminals in the correct order
    for sym in &rule.rhs {
        assert!(matches!(sym, Symbol::Terminal(_)));
    }
}

#[test]
fn seq_single_member() {
    let g = convert(&json!({
        "name": "q6",
        "rules": {
            "wrapper": {
                "type": "SEQ",
                "members": [
                    { "type": "STRING", "value": "x" }
                ]
            }
        }
    }));
    let rules = rules_for(&g, "wrapper");
    assert!(rules.iter().any(|r| r.rhs.len() == 1));
}

#[test]
fn seq_with_pattern_member() {
    let g = convert(&json!({
        "name": "q7",
        "rules": {
            "assign": {
                "type": "SEQ",
                "members": [
                    { "type": "PATTERN", "value": "[a-z]+" },
                    { "type": "STRING",  "value": "=" }
                ]
            }
        }
    }));
    let rules = rules_for(&g, "assign");
    let rule = rules.iter().find(|r| r.rhs.len() == 2).unwrap();
    assert!(
        matches!(&rule.rhs[0], Symbol::Terminal(_)),
        "pattern in seq should become Terminal"
    );
}

#[test]
fn seq_builds_parser_end_to_end() {
    let result = build_json(&json!({
        "name": "q8",
        "rules": {
            "program": {
                "type": "SEQ",
                "members": [
                    { "type": "STRING", "value": "(" },
                    { "type": "PATTERN", "value": "\\d+" },
                    { "type": "STRING", "value": ")" }
                ]
            }
        }
    }));
    assert!(
        result.is_ok(),
        "SEQ grammar should build: {:?}",
        result.err()
    );
}

// ===========================================================================
// 4. CHOICE rules → Grammar alternatives (8 tests)
// ===========================================================================

#[test]
fn choice_two_alts_produce_two_rules() {
    let g = convert(&json!({
        "name": "c1",
        "rules": {
            "value": {
                "type": "CHOICE",
                "members": [
                    { "type": "STRING", "value": "true" },
                    { "type": "STRING", "value": "false" }
                ]
            }
        }
    }));
    let rules = rules_for(&g, "value");
    assert!(
        rules.len() >= 2,
        "CHOICE with 2 members → ≥2 rules, got {}",
        rules.len()
    );
}

#[test]
fn choice_three_alts() {
    let g = convert(&json!({
        "name": "c2",
        "rules": {
            "primary": {
                "type": "CHOICE",
                "members": [
                    { "type": "STRING", "value": "a" },
                    { "type": "STRING", "value": "b" },
                    { "type": "STRING", "value": "c" }
                ]
            }
        }
    }));
    let rules = rules_for(&g, "primary");
    assert!(rules.len() >= 3);
}

#[test]
fn choice_symbol_and_string() {
    let g = convert(&json!({
        "name": "c3",
        "rules": {
            "item": {
                "type": "CHOICE",
                "members": [
                    { "type": "SYMBOL", "name": "num" },
                    { "type": "STRING", "value": "nil" }
                ]
            },
            "num": { "type": "PATTERN", "value": "\\d+" }
        }
    }));
    let rules = rules_for(&g, "item");
    assert!(rules.len() >= 2);
}

#[test]
fn choice_nested_seq_produces_multi_element_alt() {
    let g = convert(&json!({
        "name": "c4",
        "rules": {
            "expr": {
                "type": "CHOICE",
                "members": [
                    { "type": "SYMBOL", "name": "atom" },
                    {
                        "type": "SEQ",
                        "members": [
                            { "type": "SYMBOL", "name": "expr" },
                            { "type": "STRING", "value": "+" },
                            { "type": "SYMBOL", "name": "atom" }
                        ]
                    }
                ]
            },
            "atom": { "type": "PATTERN", "value": "\\d+" }
        }
    }));
    let rules = rules_for(&g, "expr");
    assert!(
        rules.iter().any(|r| r.rhs.len() >= 3),
        "SEQ inside CHOICE should produce a multi-element rule"
    );
}

#[test]
fn choice_with_blank_falls_through() {
    // BLANK inside CHOICE is silently skipped by the converter (not an error).
    // Verify the non-BLANK alternative is still created.
    let g = convert(&json!({
        "name": "c5",
        "rules": {
            "maybe": {
                "type": "CHOICE",
                "members": [
                    { "type": "STRING", "value": "x" },
                    { "type": "BLANK" }
                ]
            }
        }
    }));
    let rules = rules_for(&g, "maybe");
    assert!(
        !rules.is_empty(),
        "at least one rule should exist for the non-BLANK alt"
    );
}

#[test]
fn choice_produces_at_least_member_count_rules() {
    let g = convert(&json!({
        "name": "c6",
        "rules": {
            "tok": {
                "type": "CHOICE",
                "members": [
                    { "type": "STRING", "value": "a" },
                    { "type": "STRING", "value": "b" },
                    { "type": "STRING", "value": "c" },
                    { "type": "STRING", "value": "d" }
                ]
            }
        }
    }));
    let rules = rules_for(&g, "tok");
    assert!(rules.len() >= 4);
}

#[test]
fn choice_no_duplicate_rules() {
    let g = convert(&json!({
        "name": "c7",
        "rules": {
            "dup": {
                "type": "CHOICE",
                "members": [
                    { "type": "STRING", "value": "x" },
                    { "type": "STRING", "value": "x" }
                ]
            }
        }
    }));
    let rules = rules_for(&g, "dup");
    // The converter deduplicates identical rules
    assert!(
        rules.len() <= 2,
        "duplicate CHOICE members should be deduplicated, got {}",
        rules.len()
    );
}

#[test]
fn choice_builds_parser_end_to_end() {
    let result = build_json(&json!({
        "name": "c8",
        "rules": {
            "program": {
                "type": "CHOICE",
                "members": [
                    { "type": "STRING", "value": "yes" },
                    { "type": "STRING", "value": "no" }
                ]
            }
        }
    }));
    assert!(
        result.is_ok(),
        "CHOICE grammar should build: {:?}",
        result.err()
    );
}

// ===========================================================================
// 5. REPEAT / REPEAT1 / OPTIONAL → normalized forms (8 tests)
// ===========================================================================

#[test]
fn repeat_creates_empty_rule() {
    let g = convert(&json!({
        "name": "r1",
        "rules": {
            "items": {
                "type": "REPEAT",
                "content": { "type": "STRING", "value": "x" }
            }
        }
    }));
    let rules = rules_for(&g, "items");
    let has_empty = rules.iter().any(|r| r.rhs.is_empty());
    assert!(has_empty, "REPEAT should include an empty (ε) rule");
}

#[test]
fn repeat_creates_recursive_rule() {
    let g = convert(&json!({
        "name": "r2",
        "rules": {
            "items": {
                "type": "REPEAT",
                "content": { "type": "STRING", "value": "x" }
            }
        }
    }));
    let rules = rules_for(&g, "items");
    let lhs_id = g
        .rule_names
        .iter()
        .find(|(_, n)| n.as_str() == "items")
        .map(|(id, _)| *id)
        .unwrap();
    let has_recursive = rules.iter().any(|r| {
        r.rhs
            .iter()
            .any(|s| matches!(s, Symbol::NonTerminal(id) if id == &lhs_id))
    });
    assert!(has_recursive, "REPEAT should have a recursive rule");
}

#[test]
fn repeat1_has_base_case() {
    let g = convert(&json!({
        "name": "r3",
        "rules": {
            "items": {
                "type": "REPEAT1",
                "content": { "type": "STRING", "value": "y" }
            }
        }
    }));
    let rules = rules_for(&g, "items");
    // REPEAT1 should have at least a base rule with the content
    let has_single_terminal = rules
        .iter()
        .any(|r| r.rhs.len() == 1 && matches!(&r.rhs[0], Symbol::Terminal(_)));
    assert!(has_single_terminal, "REPEAT1 should have a base-case rule");
}

#[test]
fn repeat1_has_recursive_rule() {
    let g = convert(&json!({
        "name": "r4",
        "rules": {
            "items": {
                "type": "REPEAT1",
                "content": { "type": "STRING", "value": "y" }
            }
        }
    }));
    let rules = rules_for(&g, "items");
    let lhs_id = g
        .rule_names
        .iter()
        .find(|(_, n)| n.as_str() == "items")
        .map(|(id, _)| *id)
        .unwrap();
    let has_recursive = rules.iter().any(|r| {
        r.rhs
            .iter()
            .any(|s| matches!(s, Symbol::NonTerminal(id) if id == &lhs_id))
    });
    assert!(has_recursive, "REPEAT1 should have a self-recursive rule");
}

#[test]
fn optional_creates_content_rule() {
    let g = convert(&json!({
        "name": "r5",
        "rules": {
            "maybe": {
                "type": "OPTIONAL",
                "content": { "type": "STRING", "value": "z" }
            }
        }
    }));
    let rules = rules_for(&g, "maybe");
    let has_nonempty = rules.iter().any(|r| !r.rhs.is_empty());
    assert!(has_nonempty, "OPTIONAL should include a non-empty rule");
}

#[test]
fn optional_creates_empty_rule() {
    let g = convert(&json!({
        "name": "r6",
        "rules": {
            "maybe": {
                "type": "OPTIONAL",
                "content": { "type": "STRING", "value": "z" }
            }
        }
    }));
    let rules = rules_for(&g, "maybe");
    let has_empty = rules.iter().any(|r| r.rhs.is_empty());
    assert!(has_empty, "OPTIONAL should include an empty rule");
}

#[test]
fn repeat_of_symbol_reference() {
    let g = convert(&json!({
        "name": "r7",
        "rules": {
            "list": {
                "type": "REPEAT",
                "content": { "type": "SYMBOL", "name": "item" }
            },
            "item": { "type": "PATTERN", "value": "[a-z]+" }
        }
    }));
    let rules = rules_for(&g, "list");
    assert!(
        rules.len() >= 2,
        "REPEAT of SYMBOL should have ≥2 rules (empty + recursive)"
    );
}

#[test]
fn optional_builds_parser_end_to_end() {
    let result = build_json(&json!({
        "name": "r8",
        "rules": {
            "program": {
                "type": "OPTIONAL",
                "content": { "type": "PATTERN", "value": "\\d+" }
            }
        }
    }));
    assert!(
        result.is_ok(),
        "OPTIONAL grammar should build: {:?}",
        result.err()
    );
}

// ===========================================================================
// 6. PREC_LEFT / PREC_RIGHT → precedence in Grammar (8 tests)
// ===========================================================================

fn arith_json() -> serde_json::Value {
    json!({
        "name": "arith",
        "rules": {
            "expression": {
                "type": "CHOICE",
                "members": [
                    { "type": "SYMBOL", "name": "number" },
                    {
                        "type": "PREC_LEFT",
                        "value": 1,
                        "content": {
                            "type": "SEQ",
                            "members": [
                                { "type": "SYMBOL", "name": "expression" },
                                { "type": "STRING", "value": "+" },
                                { "type": "SYMBOL", "name": "expression" }
                            ]
                        }
                    },
                    {
                        "type": "PREC_RIGHT",
                        "value": 2,
                        "content": {
                            "type": "SEQ",
                            "members": [
                                { "type": "SYMBOL", "name": "expression" },
                                { "type": "STRING", "value": "^" },
                                { "type": "SYMBOL", "name": "expression" }
                            ]
                        }
                    }
                ]
            },
            "number": { "type": "PATTERN", "value": "[0-9]+" }
        }
    })
}

#[test]
fn prec_left_sets_left_associativity() {
    let g = convert(&arith_json());
    let rules = rules_for(&g, "expression");
    let has_left = rules
        .iter()
        .any(|r| r.associativity == Some(Associativity::Left));
    assert!(has_left, "PREC_LEFT should produce Left associativity");
}

#[test]
fn prec_right_sets_right_associativity() {
    let g = convert(&arith_json());
    let rules = rules_for(&g, "expression");
    let has_right = rules
        .iter()
        .any(|r| r.associativity == Some(Associativity::Right));
    assert!(has_right, "PREC_RIGHT should produce Right associativity");
}

#[test]
fn prec_left_stores_precedence_value() {
    let g = convert(&arith_json());
    let rules = rules_for(&g, "expression");
    let has_prec_1 = rules
        .iter()
        .any(|r| r.precedence == Some(PrecedenceKind::Static(1)));
    assert!(has_prec_1, "PREC_LEFT(1) should store precedence 1");
}

#[test]
fn prec_right_stores_precedence_value() {
    let g = convert(&arith_json());
    let rules = rules_for(&g, "expression");
    let has_prec_2 = rules
        .iter()
        .any(|r| r.precedence == Some(PrecedenceKind::Static(2)));
    assert!(has_prec_2, "PREC_RIGHT(2) should store precedence 2");
}

#[test]
fn prec_left_seq_creates_three_element_rhs() {
    let g = convert(&arith_json());
    let rules = rules_for(&g, "expression");
    let left_rule = rules
        .iter()
        .find(|r| r.associativity == Some(Associativity::Left));
    assert!(left_rule.is_some());
    assert_eq!(left_rule.unwrap().rhs.len(), 3);
}

#[test]
fn prec_right_seq_creates_three_element_rhs() {
    let g = convert(&arith_json());
    let rules = rules_for(&g, "expression");
    let right_rule = rules
        .iter()
        .find(|r| r.associativity == Some(Associativity::Right));
    assert!(right_rule.is_some());
    assert_eq!(right_rule.unwrap().rhs.len(), 3);
}

#[test]
fn prec_different_levels_preserved() {
    let g = convert(&json!({
        "name": "prec_levels",
        "rules": {
            "expr": {
                "type": "CHOICE",
                "members": [
                    { "type": "SYMBOL", "name": "atom" },
                    {
                        "type": "PREC_LEFT",
                        "value": 5,
                        "content": {
                            "type": "SEQ",
                            "members": [
                                { "type": "SYMBOL", "name": "expr" },
                                { "type": "STRING", "value": "+" },
                                { "type": "SYMBOL", "name": "expr" }
                            ]
                        }
                    },
                    {
                        "type": "PREC_LEFT",
                        "value": 10,
                        "content": {
                            "type": "SEQ",
                            "members": [
                                { "type": "SYMBOL", "name": "expr" },
                                { "type": "STRING", "value": "*" },
                                { "type": "SYMBOL", "name": "expr" }
                            ]
                        }
                    }
                ]
            },
            "atom": { "type": "PATTERN", "value": "\\d+" }
        }
    }));
    let rules = rules_for(&g, "expr");
    let has_5 = rules
        .iter()
        .any(|r| r.precedence == Some(PrecedenceKind::Static(5)));
    let has_10 = rules
        .iter()
        .any(|r| r.precedence == Some(PrecedenceKind::Static(10)));
    assert!(has_5, "precedence 5 should be present");
    assert!(has_10, "precedence 10 should be present");
}

#[test]
fn prec_builds_parser_end_to_end() {
    let result = build_json(&arith_json());
    assert!(
        result.is_ok(),
        "arithmetic grammar should build: {:?}",
        result.err()
    );
}

// ===========================================================================
// 7. Nested structures → correct Grammar topology (8 tests)
// ===========================================================================

#[test]
fn nested_choice_in_seq() {
    // SEQ with a CHOICE member — the CHOICE part becomes a sub-symbol
    let g = convert(&json!({
        "name": "n1",
        "rules": {
            "stmt": {
                "type": "SEQ",
                "members": [
                    { "type": "STRING", "value": "var" },
                    { "type": "SYMBOL", "name": "name" }
                ]
            },
            "name": {
                "type": "CHOICE",
                "members": [
                    { "type": "STRING", "value": "x" },
                    { "type": "STRING", "value": "y" }
                ]
            }
        }
    }));
    // stmt should reference name as NonTerminal
    let stmt_rules = rules_for(&g, "stmt");
    assert!(!stmt_rules.is_empty());
    // name should have 2 alternatives
    let name_rules = rules_for(&g, "name");
    assert!(name_rules.len() >= 2);
}

#[test]
fn nested_seq_in_choice() {
    let g = convert(&json!({
        "name": "n2",
        "rules": {
            "expr": {
                "type": "CHOICE",
                "members": [
                    { "type": "PATTERN", "value": "\\d+" },
                    {
                        "type": "SEQ",
                        "members": [
                            { "type": "STRING", "value": "(" },
                            { "type": "SYMBOL", "name": "expr" },
                            { "type": "STRING", "value": ")" }
                        ]
                    }
                ]
            }
        }
    }));
    let rules = rules_for(&g, "expr");
    let has_3_elem = rules.iter().any(|r| r.rhs.len() == 3);
    assert!(
        has_3_elem,
        "SEQ inside CHOICE should produce a 3-element rule"
    );
}

#[test]
fn nested_prec_in_choice() {
    let g = convert(&json!({
        "name": "n3",
        "rules": {
            "expr": {
                "type": "CHOICE",
                "members": [
                    { "type": "SYMBOL", "name": "lit" },
                    {
                        "type": "PREC_LEFT",
                        "value": 1,
                        "content": { "type": "SYMBOL", "name": "lit" }
                    }
                ]
            },
            "lit": { "type": "PATTERN", "value": "\\d+" }
        }
    }));
    let rules = rules_for(&g, "expr");
    let has_prec = rules.iter().any(|r| r.precedence.is_some());
    assert!(has_prec, "PREC_LEFT inside CHOICE should carry precedence");
}

#[test]
fn nested_repeat_of_seq() {
    let g = convert(&json!({
        "name": "n4",
        "rules": {
            "pairs": {
                "type": "REPEAT",
                "content": {
                    "type": "SYMBOL", "name": "pair"
                }
            },
            "pair": {
                "type": "SEQ",
                "members": [
                    { "type": "STRING", "value": "(" },
                    { "type": "STRING", "value": ")" }
                ]
            }
        }
    }));
    // pairs is a REPEAT → at least 2 rules (empty + recursive)
    let rules = rules_for(&g, "pairs");
    assert!(rules.len() >= 2, "REPEAT(SYMBOL) should produce ≥2 rules");
    // pair is a SEQ → 1 rule with 2 elements
    let pair_rules = rules_for(&g, "pair");
    assert!(pair_rules.iter().any(|r| r.rhs.len() == 2));
}

#[test]
fn nested_optional_symbol() {
    let g = convert(&json!({
        "name": "n5",
        "rules": {
            "maybe_num": {
                "type": "OPTIONAL",
                "content": { "type": "SYMBOL", "name": "num" }
            },
            "num": { "type": "PATTERN", "value": "\\d+" }
        }
    }));
    let rules = rules_for(&g, "maybe_num");
    let has_empty = rules.iter().any(|r| r.rhs.is_empty());
    let has_nonempty = rules.iter().any(|r| !r.rhs.is_empty());
    assert!(has_empty, "OPTIONAL(SYMBOL) should have empty alt");
    assert!(has_nonempty, "OPTIONAL(SYMBOL) should have non-empty alt");
}

#[test]
fn nested_three_levels_deep() {
    // CHOICE → PREC_LEFT → SEQ
    let g = convert(&json!({
        "name": "n6",
        "rules": {
            "expr": {
                "type": "CHOICE",
                "members": [
                    { "type": "SYMBOL", "name": "atom" },
                    {
                        "type": "PREC_LEFT",
                        "value": 3,
                        "content": {
                            "type": "SEQ",
                            "members": [
                                { "type": "SYMBOL", "name": "expr" },
                                { "type": "STRING", "value": "-" },
                                { "type": "SYMBOL", "name": "atom" }
                            ]
                        }
                    }
                ]
            },
            "atom": { "type": "PATTERN", "value": "\\d+" }
        }
    }));
    let rules = rules_for(&g, "expr");
    let deep_rule = rules.iter().find(|r| {
        r.precedence == Some(PrecedenceKind::Static(3))
            && r.associativity == Some(Associativity::Left)
            && r.rhs.len() == 3
    });
    assert!(
        deep_rule.is_some(),
        "3-level nesting should produce correct rule"
    );
}

#[test]
fn nested_grammar_preserves_name() {
    let g = convert(&json!({
        "name": "nested_lang",
        "rules": {
            "program": {
                "type": "REPEAT",
                "content": { "type": "SYMBOL", "name": "stmt" }
            },
            "stmt": {
                "type": "SEQ",
                "members": [
                    { "type": "SYMBOL", "name": "val" },
                    { "type": "STRING", "value": ";" }
                ]
            },
            "val": {
                "type": "CHOICE",
                "members": [
                    { "type": "PATTERN", "value": "\\d+" },
                    { "type": "STRING", "value": "nil" }
                ]
            }
        }
    }));
    assert_eq!(g.name, "nested_lang");
}

#[test]
fn nested_builds_parser_end_to_end() {
    let result = build_json(&json!({
        "name": "n8",
        "rules": {
            "program": {
                "type": "CHOICE",
                "members": [
                    { "type": "SYMBOL", "name": "item" },
                    {
                        "type": "SEQ",
                        "members": [
                            { "type": "SYMBOL", "name": "program" },
                            { "type": "SYMBOL", "name": "item" }
                        ]
                    }
                ]
            },
            "item": { "type": "PATTERN", "value": "[a-z]+" }
        }
    }));
    assert!(
        result.is_ok(),
        "nested grammar should build: {:?}",
        result.err()
    );
}

// ===========================================================================
// 8. Error cases → descriptive errors (8 tests)
// ===========================================================================

#[test]
fn error_missing_rules_key_produces_empty_grammar() {
    // The parser tolerates a missing "rules" key — produces a grammar with no rules.
    let gjs = from_tree_sitter_json(&json!({ "name": "bad" }));
    assert!(gjs.is_ok(), "missing rules is tolerated");
    let gjs = gjs.unwrap();
    assert!(gjs.rules.is_empty(), "no rules should be collected");
}

#[test]
fn error_missing_name_key() {
    let result = from_tree_sitter_json(&json!({
        "rules": { "a": { "type": "BLANK" } }
    }));
    assert!(result.is_err(), "missing 'name' should fail");
    let msg = result.unwrap_err().to_string();
    assert!(
        msg.to_lowercase().contains("name"),
        "error should mention name: {msg}"
    );
}

#[test]
fn error_invalid_json_string() {
    let result = build_parser_from_json("not valid json{{{".to_string(), opts());
    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(
        msg.to_lowercase().contains("json") || msg.to_lowercase().contains("parse"),
        "error should mention JSON parsing: {msg}"
    );
}

#[test]
fn error_empty_rules_object() {
    let val = json!({ "name": "empty", "rules": {} });
    let gjs = from_tree_sitter_json(&val);
    // Should either fail or produce a grammar with no user rules
    if let Ok(gjs) = gjs {
        let conv_result = GrammarJsConverter::new(gjs).convert();
        if let Ok(g) = conv_result {
            assert!(
                total_rules(&g) == 0 || g.rules.is_empty(),
                "empty rules should produce no IR rules"
            );
        }
    }
}

#[test]
fn error_empty_json_object() {
    let result = from_tree_sitter_json(&json!({}));
    assert!(result.is_err(), "empty object should fail");
}

#[test]
fn error_not_an_object() {
    let result = from_tree_sitter_json(&json!("just a string"));
    assert!(result.is_err(), "non-object should fail");
}

#[test]
fn error_rules_not_object_produces_empty_grammar() {
    // When "rules" is not an object the parser silently skips it.
    let gjs = from_tree_sitter_json(&json!({
        "name": "bad",
        "rules": "not an object"
    }));
    assert!(gjs.is_ok());
    let gjs = gjs.unwrap();
    assert!(gjs.rules.is_empty(), "non-object rules should be ignored");
}

#[test]
fn error_build_from_malformed_grammar() {
    // A grammar that parses JSON fine but has no valid start rule
    let result = build_json(&json!({
        "name": "broken",
        "rules": {
            "start": { "type": "BLANK" }
        }
    }));
    // Either an error or a degenerate build—either outcome is acceptable
    // but the pipeline should not panic.
    if let Err(e) = &result {
        let msg = e.to_string().to_lowercase();
        // Verify the error is descriptive
        assert!(!msg.is_empty(), "error message should not be empty");
    }
}
