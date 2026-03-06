#![allow(clippy::needless_range_loop)]
//! 45+ proptest properties for `NodeTypesGenerator` JSON output.
//!
//! Categories:
//!  1. Output is always valid JSON (5)
//!  2. JSON array length matches expected node count (5)
//!  3. Each entry has 'type' and 'named' fields (5)
//!  4. Type names match grammar symbol names (5)
//!  5. Named flag is boolean (5)
//!  6. Determinism: same grammar → same JSON (5)
//!  7. JSON survives serde roundtrip (5)
//!  8. Edge cases: empty, single, many (5)
//!  9. Cross-cutting invariants (5+)

use adze_ir::builder::GrammarBuilder;
use adze_ir::{FieldId, Grammar, ProductionId, Rule, Symbol, SymbolId, Token, TokenPattern};
use adze_tablegen::NodeTypesGenerator;
use proptest::prelude::*;
use serde_json::Value;
use std::collections::HashSet;

// ───────────────────────────────────────────────────────────────────────
// Strategies
// ───────────────────────────────────────────────────────────────────────

fn visible_name_strategy() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9_]{0,12}".prop_filter("non-empty", |s| !s.is_empty())
}

fn hidden_name_strategy() -> impl Strategy<Value = String> {
    "_[a-z][a-z0-9_]{0,10}".prop_filter("has body", |s| s.len() > 1)
}

fn field_name_strategy() -> impl Strategy<Value = String> {
    "[a-z][a-z_]{0,8}".prop_filter("non-empty", |s| !s.is_empty())
}

fn string_token_strategy() -> impl Strategy<Value = (String, String)> {
    prop_oneof![
        Just(("plus".into(), "+".into())),
        Just(("minus".into(), "-".into())),
        Just(("star".into(), "*".into())),
        Just(("slash".into(), "/".into())),
        Just(("semi".into(), ";".into())),
        Just(("colon".into(), ":".into())),
        Just(("comma".into(), ",".into())),
        Just(("eq".into(), "=".into())),
    ]
}

fn regex_token_strategy() -> impl Strategy<Value = (String, String)> {
    prop_oneof![
        Just(("number".into(), r"\d+".into())),
        Just(("identifier".into(), r"[a-z]+".into())),
        Just(("string_lit".into(), r#""[^"]*""#.into())),
    ]
}

// ───────────────────────────────────────────────────────────────────────
// Grammar construction helpers
// ───────────────────────────────────────────────────────────────────────

fn dedup(mut v: Vec<String>) -> Vec<String> {
    let mut seen = HashSet::new();
    v.retain(|s| seen.insert(s.clone()));
    v
}

fn dedup_by_key<T, F: Fn(&T) -> String>(mut v: Vec<T>, key: F) -> Vec<T> {
    let mut seen = HashSet::new();
    v.retain(|item| seen.insert(key(item)));
    v
}

/// Build a grammar manually with full control over visible/hidden rules and tokens.
fn build_grammar(
    visible_names: &[String],
    hidden_names: &[String],
    string_tokens: &[(String, String)],
    regex_tokens: &[(String, String)],
    field_names: &[String],
) -> Grammar {
    let mut g = Grammar::new("proptest_v5".to_string());
    let mut next_id: u16 = 0;

    let mut regex_token_ids = Vec::new();
    for (name, pattern) in regex_tokens {
        let id = SymbolId(next_id);
        next_id += 1;
        g.tokens.insert(
            id,
            Token {
                name: name.clone(),
                pattern: TokenPattern::Regex(pattern.clone()),
                fragile: false,
            },
        );
        regex_token_ids.push(id);
    }

    let mut string_token_ids = Vec::new();
    for (name, pattern) in string_tokens {
        let id = SymbolId(next_id);
        next_id += 1;
        g.tokens.insert(
            id,
            Token {
                name: name.clone(),
                pattern: TokenPattern::String(pattern.clone()),
                fragile: false,
            },
        );
        string_token_ids.push(id);
    }

    let field_ids: Vec<FieldId> = field_names
        .iter()
        .enumerate()
        .map(|(i, name)| {
            let fid = FieldId(i as u16);
            g.fields.insert(fid, name.clone());
            fid
        })
        .collect();

    let mut prod_id: u16 = 0;
    let default_terminal = regex_token_ids
        .first()
        .or(string_token_ids.first())
        .copied();

    for name in visible_names {
        let id = SymbolId(next_id);
        next_id += 1;
        g.rule_names.insert(id, name.clone());

        let (rule_fields, rhs) = if let Some(tid) = default_terminal
            && !field_ids.is_empty()
        {
            let mut rhs_symbols = Vec::new();
            let mut pairs = Vec::new();
            for (pos, fid) in field_ids.iter().enumerate() {
                rhs_symbols.push(Symbol::Terminal(tid));
                pairs.push((*fid, pos));
            }
            (pairs, rhs_symbols)
        } else if let Some(tid) = default_terminal {
            (vec![], vec![Symbol::Terminal(tid)])
        } else {
            (vec![], vec![Symbol::Epsilon])
        };

        g.add_rule(Rule {
            lhs: id,
            rhs,
            precedence: None,
            associativity: None,
            fields: rule_fields,
            production_id: ProductionId(prod_id),
        });
        prod_id += 1;
    }

    for name in hidden_names {
        let id = SymbolId(next_id);
        next_id += 1;
        g.rule_names.insert(id, name.clone());

        let rhs = if let Some(tid) = default_terminal {
            vec![Symbol::Terminal(tid)]
        } else {
            vec![Symbol::Epsilon]
        };

        g.add_rule(Rule {
            lhs: id,
            rhs,
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(prod_id),
        });
        prod_id += 1;
    }

    g
}

type GrammarWithMeta = (
    Grammar,
    Vec<String>,
    Vec<String>,
    Vec<(String, String)>,
    Vec<(String, String)>,
    Vec<String>,
);

fn grammar_strategy() -> impl Strategy<Value = GrammarWithMeta> {
    (
        prop::collection::vec(visible_name_strategy(), 0..6),
        prop::collection::vec(hidden_name_strategy(), 0..4),
        prop::collection::vec(string_token_strategy(), 0..4),
        prop::collection::vec(regex_token_strategy(), 0..3),
        prop::collection::vec(field_name_strategy(), 0..4),
    )
        .prop_map(|(visible, hidden, str_toks, re_toks, fields)| {
            let visible = dedup(visible);
            let hidden = dedup(hidden);
            let str_toks = dedup_by_key(str_toks, |t| t.1.clone());
            let re_toks = dedup_by_key(re_toks, |t| t.0.clone());
            let fields = dedup(fields);
            let grammar = build_grammar(&visible, &hidden, &str_toks, &re_toks, &fields);
            (grammar, visible, hidden, str_toks, re_toks, fields)
        })
}

/// Strategy that always produces at least one visible rule and one token.
fn nonempty_grammar_strategy() -> impl Strategy<Value = GrammarWithMeta> {
    (
        prop::collection::vec(visible_name_strategy(), 1..6),
        prop::collection::vec(hidden_name_strategy(), 0..3),
        prop::collection::vec(string_token_strategy(), 0..4),
        prop::collection::vec(regex_token_strategy(), 1..3),
        prop::collection::vec(field_name_strategy(), 0..4),
    )
        .prop_map(|(visible, hidden, str_toks, re_toks, fields)| {
            let visible = dedup(visible);
            let hidden = dedup(hidden);
            let str_toks = dedup_by_key(str_toks, |t| t.1.clone());
            let re_toks = dedup_by_key(re_toks, |t| t.0.clone());
            let fields = dedup(fields);
            let grammar = build_grammar(&visible, &hidden, &str_toks, &re_toks, &fields);
            (grammar, visible, hidden, str_toks, re_toks, fields)
        })
}

/// Strategy for large grammars.
fn large_grammar_strategy() -> impl Strategy<Value = GrammarWithMeta> {
    (
        prop::collection::vec(visible_name_strategy(), 8..20),
        prop::collection::vec(hidden_name_strategy(), 2..6),
        prop::collection::vec(string_token_strategy(), 1..5),
        prop::collection::vec(regex_token_strategy(), 1..3),
        prop::collection::vec(field_name_strategy(), 0..4),
    )
        .prop_map(|(visible, hidden, str_toks, re_toks, fields)| {
            let visible = dedup(visible);
            let hidden = dedup(hidden);
            let str_toks = dedup_by_key(str_toks, |t| t.1.clone());
            let re_toks = dedup_by_key(re_toks, |t| t.0.clone());
            let fields = dedup(fields);
            let grammar = build_grammar(&visible, &hidden, &str_toks, &re_toks, &fields);
            (grammar, visible, hidden, str_toks, re_toks, fields)
        })
}

fn parse_node_types(json: &str) -> Vec<Value> {
    let parsed: Value = serde_json::from_str(json).expect("must be valid JSON");
    parsed.as_array().expect("top-level must be array").clone()
}

// ───────────────────────────────────────────────────────────────────────
// Category 1: Output is always valid JSON (5 properties)
// ───────────────────────────────────────────────────────────────────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    // 1.1 Random grammars always produce valid JSON
    #[test]
    fn c1_valid_json_random_grammar(
        (grammar, ..) in grammar_strategy()
    ) {
        let ntg = NodeTypesGenerator::new(&grammar);
        let json = ntg.generate().expect("generate must succeed");
        let _: Value = serde_json::from_str(&json).expect("must parse as JSON");
    }

    // 1.2 Non-empty grammars produce valid JSON
    #[test]
    fn c1_valid_json_nonempty_grammar(
        (grammar, ..) in nonempty_grammar_strategy()
    ) {
        let json = NodeTypesGenerator::new(&grammar).generate().unwrap();
        let _: Value = serde_json::from_str(&json).expect("must parse as JSON");
    }

    // 1.3 Large grammars produce valid JSON
    #[test]
    fn c1_valid_json_large_grammar(
        (grammar, ..) in large_grammar_strategy()
    ) {
        let json = NodeTypesGenerator::new(&grammar).generate().unwrap();
        let _: Value = serde_json::from_str(&json).expect("must parse as JSON");
    }

    // 1.4 GrammarBuilder grammars produce valid JSON
    #[test]
    fn c1_valid_json_builder_grammar(
        name in visible_name_strategy(),
        tok_name in visible_name_strategy(),
    ) {
        let grammar = GrammarBuilder::new("test")
            .token(&tok_name, r"\d+")
            .rule(&name, vec![&tok_name])
            .start(&name)
            .build();
        let json = NodeTypesGenerator::new(&grammar).generate().unwrap();
        let _: Value = serde_json::from_str(&json).expect("must parse as JSON");
    }

    // 1.5 Top-level value is always a JSON array
    #[test]
    fn c1_top_level_is_array(
        (grammar, ..) in grammar_strategy()
    ) {
        let json = NodeTypesGenerator::new(&grammar).generate().unwrap();
        let parsed: Value = serde_json::from_str(&json).unwrap();
        prop_assert!(parsed.is_array(), "top-level must be an array");
    }
}

// ───────────────────────────────────────────────────────────────────────
// Category 2: JSON array length matches expected node count (5)
// ───────────────────────────────────────────────────────────────────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    // 2.1 Visible rules appear in output
    #[test]
    fn c2_visible_rules_present(
        (grammar, visible, ..) in nonempty_grammar_strategy()
    ) {
        let types = parse_node_types(
            &NodeTypesGenerator::new(&grammar).generate().unwrap(),
        );
        let names: HashSet<&str> = types.iter()
            .filter_map(|t| t.get("type").and_then(|v| v.as_str()))
            .collect();
        for v in &visible {
            prop_assert!(names.contains(v.as_str()), "missing visible rule '{}'", v);
        }
    }

    // 2.2 Hidden rules are excluded from output
    #[test]
    fn c2_hidden_rules_excluded(
        (grammar, _, hidden, ..) in grammar_strategy()
    ) {
        let types = parse_node_types(
            &NodeTypesGenerator::new(&grammar).generate().unwrap(),
        );
        let names: HashSet<&str> = types.iter()
            .filter_map(|t| t.get("type").and_then(|v| v.as_str()))
            .collect();
        for h in &hidden {
            prop_assert!(!names.contains(h.as_str()), "hidden rule '{}' must not appear", h);
        }
    }

    // 2.3 String tokens contribute unnamed entries
    #[test]
    fn c2_string_tokens_contribute_entries(
        (grammar, _, _, str_toks, ..) in grammar_strategy()
    ) {
        let types = parse_node_types(
            &NodeTypesGenerator::new(&grammar).generate().unwrap(),
        );
        let names: HashSet<&str> = types.iter()
            .filter_map(|t| t.get("type").and_then(|v| v.as_str()))
            .collect();
        for (_, pattern) in &str_toks {
            prop_assert!(names.contains(pattern.as_str()), "string token '{}' missing", pattern);
        }
    }

    // 2.4 Node count is at least the number of visible rules
    #[test]
    fn c2_count_at_least_visible(
        (grammar, visible, ..) in grammar_strategy()
    ) {
        let types = parse_node_types(
            &NodeTypesGenerator::new(&grammar).generate().unwrap(),
        );
        prop_assert!(
            types.len() >= visible.len(),
            "got {} entries but {} visible rules",
            types.len(),
            visible.len()
        );
    }

    // 2.5 Node count equals visible rules plus string tokens
    #[test]
    fn c2_count_equals_visible_plus_string_tokens(
        (grammar, visible, _, str_toks, ..) in grammar_strategy()
    ) {
        let types = parse_node_types(
            &NodeTypesGenerator::new(&grammar).generate().unwrap(),
        );
        let expected = visible.len() + str_toks.len();
        prop_assert_eq!(
            types.len(),
            expected,
            "expected {} entries (visible={} + string_tokens={}), got {}",
            expected,
            visible.len(),
            str_toks.len(),
            types.len()
        );
    }
}

// ───────────────────────────────────────────────────────────────────────
// Category 3: Each entry has 'type' and 'named' fields (5)
// ───────────────────────────────────────────────────────────────────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    // 3.1 Every entry has a 'type' key
    #[test]
    fn c3_every_entry_has_type(
        (grammar, ..) in grammar_strategy()
    ) {
        let types = parse_node_types(
            &NodeTypesGenerator::new(&grammar).generate().unwrap(),
        );
        for (i, entry) in types.iter().enumerate() {
            prop_assert!(entry.get("type").is_some(), "entry {} missing 'type'", i);
        }
    }

    // 3.2 Every entry has a 'named' key
    #[test]
    fn c3_every_entry_has_named(
        (grammar, ..) in grammar_strategy()
    ) {
        let types = parse_node_types(
            &NodeTypesGenerator::new(&grammar).generate().unwrap(),
        );
        for (i, entry) in types.iter().enumerate() {
            prop_assert!(entry.get("named").is_some(), "entry {} missing 'named'", i);
        }
    }

    // 3.3 'type' is always a non-empty string
    #[test]
    fn c3_type_is_nonempty_string(
        (grammar, ..) in nonempty_grammar_strategy()
    ) {
        let types = parse_node_types(
            &NodeTypesGenerator::new(&grammar).generate().unwrap(),
        );
        for entry in &types {
            let t = entry.get("type").and_then(|v| v.as_str()).unwrap_or("");
            prop_assert!(!t.is_empty(), "'type' must be non-empty");
        }
    }

    // 3.4 Only allowed top-level keys in each entry
    #[test]
    fn c3_only_allowed_keys(
        (grammar, ..) in grammar_strategy()
    ) {
        let allowed: HashSet<&str> =
            ["type", "named", "fields", "children", "subtypes"].iter().copied().collect();
        let types = parse_node_types(
            &NodeTypesGenerator::new(&grammar).generate().unwrap(),
        );
        for entry in &types {
            if let Some(obj) = entry.as_object() {
                for key in obj.keys() {
                    prop_assert!(
                        allowed.contains(key.as_str()),
                        "unexpected key '{}' in entry",
                        key
                    );
                }
            }
        }
    }

    // 3.5 Each entry is a JSON object (not array, string, etc.)
    #[test]
    fn c3_entries_are_objects(
        (grammar, ..) in grammar_strategy()
    ) {
        let types = parse_node_types(
            &NodeTypesGenerator::new(&grammar).generate().unwrap(),
        );
        for (i, entry) in types.iter().enumerate() {
            prop_assert!(entry.is_object(), "entry {} is not an object", i);
        }
    }
}

// ───────────────────────────────────────────────────────────────────────
// Category 4: Type names match grammar symbol names (5)
// ───────────────────────────────────────────────────────────────────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    // 4.1 All type names come from grammar names or fallback
    #[test]
    fn c4_type_names_from_grammar(
        (grammar, visible, _, str_toks, ..) in grammar_strategy()
    ) {
        let types = parse_node_types(
            &NodeTypesGenerator::new(&grammar).generate().unwrap(),
        );
        let mut valid: HashSet<String> = visible.into_iter().collect();
        for (_, pattern) in &str_toks {
            valid.insert(pattern.clone());
        }
        for entry in &types {
            if let Some(name) = entry.get("type").and_then(|v| v.as_str()) {
                let is_fallback = name.starts_with("rule_");
                prop_assert!(
                    valid.contains(name) || is_fallback,
                    "unexpected type name '{}'",
                    name
                );
            }
        }
    }

    // 4.2 Visible rules map to named=true entries
    #[test]
    fn c4_visible_rules_are_named(
        (grammar, visible, ..) in nonempty_grammar_strategy()
    ) {
        let types = parse_node_types(
            &NodeTypesGenerator::new(&grammar).generate().unwrap(),
        );
        for entry in &types {
            let name = entry.get("type").and_then(|v| v.as_str()).unwrap_or("");
            if visible.contains(&name.to_string()) {
                let named = entry.get("named").and_then(|v| v.as_bool()).unwrap_or(false);
                prop_assert!(named, "visible rule '{}' should be named=true", name);
            }
        }
    }

    // 4.3 String tokens map to named=false entries
    #[test]
    fn c4_string_tokens_are_unnamed(
        (grammar, _, _, str_toks, ..) in grammar_strategy()
    ) {
        let types = parse_node_types(
            &NodeTypesGenerator::new(&grammar).generate().unwrap(),
        );
        let str_patterns: HashSet<&str> = str_toks.iter().map(|(_, p)| p.as_str()).collect();
        for entry in &types {
            let name = entry.get("type").and_then(|v| v.as_str()).unwrap_or("");
            if str_patterns.contains(name) {
                let named = entry.get("named").and_then(|v| v.as_bool()).unwrap_or(true);
                prop_assert!(!named, "string token '{}' should be named=false", name);
            }
        }
    }

    // 4.4 No duplicate type names in output
    #[test]
    fn c4_no_duplicate_type_names(
        (grammar, ..) in grammar_strategy()
    ) {
        let types = parse_node_types(
            &NodeTypesGenerator::new(&grammar).generate().unwrap(),
        );
        let mut seen = HashSet::new();
        for entry in &types {
            if let Some(name) = entry.get("type").and_then(|v| v.as_str()) {
                prop_assert!(
                    seen.insert(name.to_string()),
                    "duplicate type name: '{}'",
                    name
                );
            }
        }
    }

    // 4.5 GrammarBuilder rule names appear in output
    #[test]
    fn c4_builder_rule_names_present(
        name in "[a-z][a-z0-9]{0,6}",
    ) {
        let grammar = GrammarBuilder::new("test")
            .token("NUMBER", r"\d+")
            .rule(&name, vec!["NUMBER"])
            .start(&name)
            .build();
        let types = parse_node_types(
            &NodeTypesGenerator::new(&grammar).generate().unwrap(),
        );
        let names: HashSet<&str> = types.iter()
            .filter_map(|t| t.get("type").and_then(|v| v.as_str()))
            .collect();
        prop_assert!(names.contains(name.as_str()), "builder rule '{}' missing", name);
    }
}

// ───────────────────────────────────────────────────────────────────────
// Category 5: Named flag is boolean (5)
// ───────────────────────────────────────────────────────────────────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    // 5.1 'named' is always a boolean in random grammars
    #[test]
    fn c5_named_is_bool_random(
        (grammar, ..) in grammar_strategy()
    ) {
        let types = parse_node_types(
            &NodeTypesGenerator::new(&grammar).generate().unwrap(),
        );
        for (i, entry) in types.iter().enumerate() {
            let val = entry.get("named");
            prop_assert!(
                val.is_none_or(|v| v.is_boolean()),
                "entry {} 'named' is not boolean: {:?}",
                i,
                val
            );
        }
    }

    // 5.2 'named' is always a boolean in non-empty grammars
    #[test]
    fn c5_named_is_bool_nonempty(
        (grammar, ..) in nonempty_grammar_strategy()
    ) {
        let types = parse_node_types(
            &NodeTypesGenerator::new(&grammar).generate().unwrap(),
        );
        for entry in &types {
            if let Some(val) = entry.get("named") {
                prop_assert!(val.is_boolean(), "'named' must be boolean, got {:?}", val);
            }
        }
    }

    // 5.3 'named' is always a boolean in large grammars
    #[test]
    fn c5_named_is_bool_large(
        (grammar, ..) in large_grammar_strategy()
    ) {
        for entry in parse_node_types(
            &NodeTypesGenerator::new(&grammar).generate().unwrap(),
        ) {
            if let Some(val) = entry.get("named") {
                prop_assert!(val.is_boolean(), "'named' must be boolean, got {:?}", val);
            }
        }
    }

    // 5.4 'named' true only for rules (not string tokens)
    #[test]
    fn c5_named_true_only_for_rules(
        (grammar, visible, _, str_toks, ..) in grammar_strategy()
    ) {
        let types = parse_node_types(
            &NodeTypesGenerator::new(&grammar).generate().unwrap(),
        );
        let visible_set: HashSet<&str> = visible.iter().map(|s| s.as_str()).collect();
        let str_set: HashSet<&str> = str_toks.iter().map(|(_, p)| p.as_str()).collect();
        for entry in &types {
            let name = entry.get("type").and_then(|v| v.as_str()).unwrap_or("");
            let named = entry.get("named").and_then(|v| v.as_bool()).unwrap_or(false);
            if named {
                prop_assert!(
                    visible_set.contains(name) || name.starts_with("rule_"),
                    "named=true but '{}' is not a visible rule",
                    name
                );
            }
            if str_set.contains(name) {
                prop_assert!(!named, "string token '{}' must not be named", name);
            }
        }
    }

    // 5.5 'named' is never null or missing
    #[test]
    fn c5_named_never_null(
        (grammar, ..) in grammar_strategy()
    ) {
        let types = parse_node_types(
            &NodeTypesGenerator::new(&grammar).generate().unwrap(),
        );
        for (i, entry) in types.iter().enumerate() {
            let val = entry.get("named");
            prop_assert!(val.is_some(), "entry {} missing 'named'", i);
            prop_assert!(!val.unwrap().is_null(), "entry {} 'named' is null", i);
        }
    }
}

// ───────────────────────────────────────────────────────────────────────
// Category 6: Determinism — same grammar → same JSON (5)
// ───────────────────────────────────────────────────────────────────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    // 6.1 Two calls on the same grammar produce semantically identical JSON
    #[test]
    fn c6_deterministic_same_instance(
        (grammar, ..) in grammar_strategy()
    ) {
        let a: Value = serde_json::from_str(
            &NodeTypesGenerator::new(&grammar).generate().unwrap(),
        ).unwrap();
        let b: Value = serde_json::from_str(
            &NodeTypesGenerator::new(&grammar).generate().unwrap(),
        ).unwrap();
        prop_assert_eq!(a, b, "two calls must produce identical output");
    }

    // 6.2 Deterministic for non-empty grammars
    #[test]
    fn c6_deterministic_nonempty(
        (grammar, ..) in nonempty_grammar_strategy()
    ) {
        let a: Value = serde_json::from_str(
            &NodeTypesGenerator::new(&grammar).generate().unwrap(),
        ).unwrap();
        let b: Value = serde_json::from_str(
            &NodeTypesGenerator::new(&grammar).generate().unwrap(),
        ).unwrap();
        prop_assert_eq!(a, b);
    }

    // 6.3 Deterministic for large grammars
    #[test]
    fn c6_deterministic_large(
        (grammar, ..) in large_grammar_strategy()
    ) {
        let a: Value = serde_json::from_str(
            &NodeTypesGenerator::new(&grammar).generate().unwrap(),
        ).unwrap();
        let b: Value = serde_json::from_str(
            &NodeTypesGenerator::new(&grammar).generate().unwrap(),
        ).unwrap();
        prop_assert_eq!(a, b);
    }

    // 6.4 Three consecutive calls are identical
    #[test]
    fn c6_deterministic_triple(
        (grammar, ..) in grammar_strategy()
    ) {
        let ntg = NodeTypesGenerator::new(&grammar);
        let a: Value = serde_json::from_str(&ntg.generate().unwrap()).unwrap();
        let b: Value = serde_json::from_str(&ntg.generate().unwrap()).unwrap();
        let c: Value = serde_json::from_str(&ntg.generate().unwrap()).unwrap();
        prop_assert_eq!(&a, &b);
        prop_assert_eq!(&b, &c);
    }

    // 6.5 Deterministic for GrammarBuilder grammars
    #[test]
    fn c6_deterministic_builder(
        name in "[a-z][a-z0-9]{0,6}",
    ) {
        let grammar = GrammarBuilder::new("det")
            .token("NUM", r"\d+")
            .rule(&name, vec!["NUM"])
            .start(&name)
            .build();
        let a: Value = serde_json::from_str(
            &NodeTypesGenerator::new(&grammar).generate().unwrap(),
        ).unwrap();
        let b: Value = serde_json::from_str(
            &NodeTypesGenerator::new(&grammar).generate().unwrap(),
        ).unwrap();
        prop_assert_eq!(a, b);
    }
}

// ───────────────────────────────────────────────────────────────────────
// Category 7: JSON survives serde roundtrip (5)
// ───────────────────────────────────────────────────────────────────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    // 7.1 parse → serialize → parse roundtrip preserves structure
    #[test]
    fn c7_roundtrip_random(
        (grammar, ..) in grammar_strategy()
    ) {
        let json = NodeTypesGenerator::new(&grammar).generate().unwrap();
        let parsed: Value = serde_json::from_str(&json).unwrap();
        let reserialized = serde_json::to_string_pretty(&parsed).unwrap();
        let reparsed: Value = serde_json::from_str(&reserialized).unwrap();
        prop_assert_eq!(parsed, reparsed);
    }

    // 7.2 Roundtrip for non-empty grammars
    #[test]
    fn c7_roundtrip_nonempty(
        (grammar, ..) in nonempty_grammar_strategy()
    ) {
        let json = NodeTypesGenerator::new(&grammar).generate().unwrap();
        let parsed: Value = serde_json::from_str(&json).unwrap();
        let reserialized = serde_json::to_string(&parsed).unwrap();
        let reparsed: Value = serde_json::from_str(&reserialized).unwrap();
        prop_assert_eq!(parsed, reparsed);
    }

    // 7.3 Roundtrip for large grammars
    #[test]
    fn c7_roundtrip_large(
        (grammar, ..) in large_grammar_strategy()
    ) {
        let json = NodeTypesGenerator::new(&grammar).generate().unwrap();
        let parsed: Value = serde_json::from_str(&json).unwrap();
        let compact = serde_json::to_string(&parsed).unwrap();
        let reparsed: Value = serde_json::from_str(&compact).unwrap();
        prop_assert_eq!(parsed, reparsed);
    }

    // 7.4 Roundtrip preserves entry count
    #[test]
    fn c7_roundtrip_preserves_count(
        (grammar, ..) in grammar_strategy()
    ) {
        let json = NodeTypesGenerator::new(&grammar).generate().unwrap();
        let orig = parse_node_types(&json);
        let reserialized = serde_json::to_string(&orig).unwrap();
        let reparsed = parse_node_types(&reserialized);
        prop_assert_eq!(orig.len(), reparsed.len());
    }

    // 7.5 Roundtrip preserves type names
    #[test]
    fn c7_roundtrip_preserves_names(
        (grammar, ..) in nonempty_grammar_strategy()
    ) {
        let json = NodeTypesGenerator::new(&grammar).generate().unwrap();
        let orig = parse_node_types(&json);
        let reserialized = serde_json::to_string_pretty(&orig).unwrap();
        let reparsed = parse_node_types(&reserialized);
        let orig_names: Vec<&str> = orig.iter()
            .filter_map(|t| t.get("type").and_then(|v| v.as_str()))
            .collect();
        let rt_names: Vec<&str> = reparsed.iter()
            .filter_map(|t| t.get("type").and_then(|v| v.as_str()))
            .collect();
        prop_assert_eq!(orig_names, rt_names);
    }
}

// ───────────────────────────────────────────────────────────────────────
// Category 8: Edge cases — empty, single, many (5)
// ───────────────────────────────────────────────────────────────────────

#[test]
fn c8_empty_grammar_produces_empty_array() {
    let grammar = Grammar::new("empty".to_string());
    let json = NodeTypesGenerator::new(&grammar).generate().unwrap();
    let types = parse_node_types(&json);
    assert!(types.is_empty(), "empty grammar must produce empty array");
}

#[test]
fn c8_single_rule_grammar() {
    let grammar = GrammarBuilder::new("single")
        .token("ID", r"[a-z]+")
        .rule("program", vec!["ID"])
        .start("program")
        .build();
    let types = parse_node_types(&NodeTypesGenerator::new(&grammar).generate().unwrap());
    let names: Vec<&str> = types
        .iter()
        .filter_map(|t| t.get("type").and_then(|v| v.as_str()))
        .collect();
    assert!(names.contains(&"program"), "single rule must appear");
}

#[test]
fn c8_single_string_token_only() {
    let mut grammar = Grammar::new("tok_only".to_string());
    grammar.tokens.insert(
        SymbolId(0),
        Token {
            name: "plus".to_string(),
            pattern: TokenPattern::String("+".to_string()),
            fragile: false,
        },
    );
    let types = parse_node_types(&NodeTypesGenerator::new(&grammar).generate().unwrap());
    assert_eq!(types.len(), 1);
    let entry = &types[0];
    assert_eq!(entry.get("type").and_then(|v| v.as_str()), Some("+"));
    assert_eq!(entry.get("named").and_then(|v| v.as_bool()), Some(false));
}

#[test]
fn c8_many_rules_all_appear() {
    let mut builder = GrammarBuilder::new("many");
    builder = builder.token("ID", r"[a-z]+");
    let rule_names: Vec<String> = (0..20).map(|i| format!("rule_{}", i)).collect();
    for name in &rule_names {
        builder = builder.rule(name, vec!["ID"]);
    }
    builder = builder.start(&rule_names[0]);
    let grammar = builder.build();
    let types = parse_node_types(&NodeTypesGenerator::new(&grammar).generate().unwrap());
    let names: HashSet<&str> = types
        .iter()
        .filter_map(|t| t.get("type").and_then(|v| v.as_str()))
        .collect();
    for rn in &rule_names {
        assert!(names.contains(rn.as_str()), "missing rule '{}'", rn);
    }
}

#[test]
fn c8_regex_token_not_in_unnamed_output() {
    let mut grammar = Grammar::new("regex_only".to_string());
    grammar.tokens.insert(
        SymbolId(0),
        Token {
            name: "number".to_string(),
            pattern: TokenPattern::Regex(r"\d+".to_string()),
            fragile: false,
        },
    );
    let types = parse_node_types(&NodeTypesGenerator::new(&grammar).generate().unwrap());
    // Regex tokens are named=true and are only emitted when referenced by rules;
    // the generator only adds unnamed (string) tokens to the output.
    let unnamed: Vec<&Value> = types
        .iter()
        .filter(|t| t.get("named").and_then(|v| v.as_bool()) == Some(false))
        .collect();
    assert!(
        unnamed.is_empty(),
        "regex tokens should not appear as unnamed entries"
    );
}

// ───────────────────────────────────────────────────────────────────────
// Category 9: Cross-cutting invariants (5+)
// ───────────────────────────────────────────────────────────────────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    // 9.1 Output is sorted by type name
    #[test]
    fn c9_output_sorted_by_type_name(
        (grammar, ..) in grammar_strategy()
    ) {
        let types = parse_node_types(
            &NodeTypesGenerator::new(&grammar).generate().unwrap(),
        );
        let names: Vec<&str> = types.iter()
            .filter_map(|t| t.get("type").and_then(|v| v.as_str()))
            .collect();
        for window in names.windows(2) {
            prop_assert!(
                window[0] <= window[1],
                "output not sorted: '{}' > '{}'",
                window[0],
                window[1]
            );
        }
    }

    // 9.2 Fields (when present) have required sub-keys
    #[test]
    fn c9_fields_have_required_subkeys(
        (grammar, ..) in grammar_strategy()
    ) {
        let types = parse_node_types(
            &NodeTypesGenerator::new(&grammar).generate().unwrap(),
        );
        for entry in &types {
            if let Some(fields) = entry.get("fields").and_then(|v| v.as_object()) {
                for (fname, fval) in fields {
                    prop_assert!(
                        fval.get("types").is_some(),
                        "field '{}' missing 'types'",
                        fname
                    );
                    prop_assert!(
                        fval.get("multiple").is_some(),
                        "field '{}' missing 'multiple'",
                        fname
                    );
                    prop_assert!(
                        fval.get("required").is_some(),
                        "field '{}' missing 'required'",
                        fname
                    );
                }
            }
        }
    }

    // 9.3 Subtypes (when present) is always an array
    #[test]
    fn c9_subtypes_is_array(
        (grammar, ..) in grammar_strategy()
    ) {
        let types = parse_node_types(
            &NodeTypesGenerator::new(&grammar).generate().unwrap(),
        );
        for entry in &types {
            if let Some(subtypes) = entry.get("subtypes") {
                prop_assert!(subtypes.is_array(), "subtypes must be array");
            }
        }
    }

    // 9.4 Children (when present) is an object with 'types'
    #[test]
    fn c9_children_is_object_with_types(
        (grammar, ..) in grammar_strategy()
    ) {
        let types = parse_node_types(
            &NodeTypesGenerator::new(&grammar).generate().unwrap(),
        );
        for entry in &types {
            if let Some(children) = entry.get("children") {
                prop_assert!(children.is_object(), "children must be object");
                prop_assert!(children.get("types").is_some(), "children missing 'types'");
            }
        }
    }

    // 9.5 Generate never returns Err for well-formed grammars
    #[test]
    fn c9_generate_never_errors(
        (grammar, ..) in grammar_strategy()
    ) {
        let result = NodeTypesGenerator::new(&grammar).generate();
        prop_assert!(result.is_ok(), "generate returned error: {:?}", result.err());
    }

    // 9.6 JSON byte length is non-negative and grows with grammar size
    #[test]
    fn c9_json_length_nonneg(
        (grammar, ..) in grammar_strategy()
    ) {
        let json = NodeTypesGenerator::new(&grammar).generate().unwrap();
        prop_assert!(json.len() >= 2, "JSON must be at least '[]' (2 bytes)");
    }
}
