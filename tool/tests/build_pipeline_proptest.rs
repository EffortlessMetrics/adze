//! Property-based tests for the tool crate's build pipeline and grammar conversion.

use adze_tool::grammar_js::{GrammarJs, GrammarJsConverter, Rule};
use proptest::prelude::*;
use serde_json::{Value, json};

// ---------------------------------------------------------------------------
// Strategies
// ---------------------------------------------------------------------------

/// Generate a valid identifier-like name (non-empty, ASCII alpha + underscore start).
fn ident_strategy() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9_]{0,15}".prop_map(|s| s)
}

/// Generate a leaf Rule (String, Pattern, Blank, or Symbol referencing a known name).
fn leaf_rule_strategy() -> impl Strategy<Value = Rule> {
    prop_oneof![
        "[a-zA-Z0-9 +\\-*/]{1,10}".prop_map(|v| Rule::String { value: v }),
        Just(Rule::Pattern {
            value: r"[a-z]+".to_string(),
        }),
        Just(Rule::Blank),
    ]
}

/// Generate a possibly-nested Rule tree up to a bounded depth.
fn rule_strategy() -> impl Strategy<Value = Rule> {
    leaf_rule_strategy().prop_recursive(3, 16, 4, |inner| {
        prop_oneof![
            // Seq with 1..=4 members
            prop::collection::vec(inner.clone(), 1..=4).prop_map(|members| Rule::Seq { members }),
            // Choice with 2..=4 members
            prop::collection::vec(inner.clone(), 2..=4)
                .prop_map(|members| Rule::Choice { members }),
            // Optional
            inner
                .clone()
                .prop_map(|r| Rule::Optional { value: Box::new(r) }),
            // Repeat
            inner.clone().prop_map(|r| Rule::Repeat {
                content: Box::new(r)
            }),
            // Repeat1
            inner.clone().prop_map(|r| Rule::Repeat1 {
                content: Box::new(r)
            }),
            // Token wrapper
            inner.clone().prop_map(|r| Rule::Token {
                content: Box::new(r)
            }),
            // Prec
            (-10i32..10i32, inner.clone()).prop_map(|(v, r)| Rule::Prec {
                value: v,
                content: Box::new(r),
            }),
            // PrecLeft
            (-10i32..10i32, inner.clone()).prop_map(|(v, r)| Rule::PrecLeft {
                value: v,
                content: Box::new(r),
            }),
            // PrecRight
            (-10i32..10i32, inner.clone()).prop_map(|(v, r)| Rule::PrecRight {
                value: v,
                content: Box::new(r),
            }),
            // Field
            (ident_strategy(), inner.clone()).prop_map(|(n, r)| Rule::Field {
                name: n,
                content: Box::new(r),
            }),
        ]
    })
}

/// Build a minimal valid GrammarJs with at least one rule.
fn grammar_js_strategy() -> impl Strategy<Value = GrammarJs> {
    (
        ident_strategy(),
        prop::collection::vec((ident_strategy(), rule_strategy()), 1..=5),
    )
        .prop_map(|(name, rule_pairs)| {
            let mut g = GrammarJs::new(name);
            for (rname, rule) in rule_pairs {
                g.rules.insert(rname, rule);
            }
            g
        })
}

/// Build a grammar JSON Value with a given set of (name, rule) pairs.
fn grammar_json_strategy() -> impl Strategy<Value = Value> {
    (
        ident_strategy(),
        prop::collection::vec(
            (
                ident_strategy(),
                prop_oneof![
                    "[a-zA-Z +]{1,8}".prop_map(|v| json!({"type": "STRING", "value": v})),
                    Just(json!({"type": "PATTERN", "value": "[a-z]+"})),
                    Just(json!({"type": "BLANK"})),
                ],
            ),
            1..=5,
        ),
    )
        .prop_map(|(name, rule_pairs)| {
            let mut rules = serde_json::Map::new();
            for (rname, rule) in rule_pairs {
                rules.insert(rname, rule);
            }
            json!({
                "name": name,
                "rules": rules,
            })
        })
}

// ---------------------------------------------------------------------------
// 1. Rule serde roundtrip
// ---------------------------------------------------------------------------
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn rule_serde_roundtrip(rule in rule_strategy()) {
        let serialized = serde_json::to_string(&rule).unwrap();
        let deserialized: Rule = serde_json::from_str(&serialized).unwrap();
        let reserialized = serde_json::to_string(&deserialized).unwrap();
        prop_assert_eq!(serialized, reserialized);
    }
}

// ---------------------------------------------------------------------------
// 2. GrammarJs serde roundtrip
// ---------------------------------------------------------------------------
proptest! {
    #![proptest_config(ProptestConfig::with_cases(60))]

    #[test]
    fn grammar_js_serde_roundtrip(g in grammar_js_strategy()) {
        let s = serde_json::to_string(&g).unwrap();
        let g2: GrammarJs = serde_json::from_str(&s).unwrap();
        prop_assert_eq!(g.name, g2.name);
        prop_assert_eq!(g.rules.len(), g2.rules.len());
    }
}

// ---------------------------------------------------------------------------
// 3. from_json always succeeds for well-formed input
// ---------------------------------------------------------------------------
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn from_json_accepts_valid_grammar(json_val in grammar_json_strategy()) {
        let result = adze_tool::grammar_js::from_json(&json_val);
        prop_assert!(result.is_ok(), "from_json failed: {:?}", result.err());
        let g = result.unwrap();
        // name should match
        prop_assert_eq!(
            g.name,
            json_val["name"].as_str().unwrap()
        );
    }
}

// ---------------------------------------------------------------------------
// 4. from_json rejects missing name
// ---------------------------------------------------------------------------
proptest! {
    #[test]
    fn from_json_rejects_missing_name(rules_count in 0usize..3) {
        let mut rules = serde_json::Map::new();
        for i in 0..rules_count {
            rules.insert(
                format!("r{}", i),
                json!({"type": "BLANK"}),
            );
        }
        let val = json!({"rules": rules});
        prop_assert!(adze_tool::grammar_js::from_json(&val).is_err());
    }
}

// ---------------------------------------------------------------------------
// 5. from_json rejects non-object input
// ---------------------------------------------------------------------------
proptest! {
    #[test]
    fn from_json_rejects_non_object(s in ".*") {
        let val = Value::String(s);
        prop_assert!(adze_tool::grammar_js::from_json(&val).is_err());
    }
}

// ---------------------------------------------------------------------------
// 6. GrammarJs validate passes for self-contained rules
// ---------------------------------------------------------------------------
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn validate_passes_self_contained(g in grammar_js_strategy()) {
        // Generated grammars have no Symbol refs → validation should succeed
        let result = g.validate();
        prop_assert!(result.is_ok(), "validate failed: {:?}", result.err());
    }
}

// ---------------------------------------------------------------------------
// 7. GrammarJs validate rejects dangling symbol references
// ---------------------------------------------------------------------------
proptest! {
    #[test]
    fn validate_rejects_dangling_symbol(name in ident_strategy()) {
        let mut g = GrammarJs::new("test".to_string());
        g.rules.insert(
            "start".to_string(),
            Rule::Symbol { name: format!("{}_nonexistent", name) },
        );
        prop_assert!(g.validate().is_err());
    }
}

// ---------------------------------------------------------------------------
// 8. GrammarJs validate rejects invalid word token
// ---------------------------------------------------------------------------
proptest! {
    #[test]
    fn validate_rejects_invalid_word(word in ident_strategy()) {
        let mut g = GrammarJs::new("test".to_string());
        g.rules.insert("start".to_string(), Rule::Blank);
        g.word = Some(format!("{}_missing", word));
        prop_assert!(g.validate().is_err());
    }
}

// ---------------------------------------------------------------------------
// 9. GrammarJs validate rejects invalid inline references
// ---------------------------------------------------------------------------
proptest! {
    #[test]
    fn validate_rejects_invalid_inline(inline_name in ident_strategy()) {
        let mut g = GrammarJs::new("test".to_string());
        g.rules.insert("start".to_string(), Rule::Blank);
        g.inline.push(format!("{}_nope", inline_name));
        prop_assert!(g.validate().is_err());
    }
}

// ---------------------------------------------------------------------------
// 10. GrammarJs validate rejects invalid conflict references
// ---------------------------------------------------------------------------
proptest! {
    #[test]
    fn validate_rejects_invalid_conflict(conflict_name in ident_strategy()) {
        let mut g = GrammarJs::new("test".to_string());
        g.rules.insert("start".to_string(), Rule::Blank);
        g.conflicts.push(vec![format!("{}_bad", conflict_name)]);
        prop_assert!(g.validate().is_err());
    }
}

// ---------------------------------------------------------------------------
// 11. HelperFunctions: commaSep produces Optional(Seq(...))
// ---------------------------------------------------------------------------
proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    #[test]
    fn helper_comma_sep_structure(rule in leaf_rule_strategy()) {
        use adze_tool::grammar_js::helpers::HelperFunctions;
        let result = HelperFunctions::evaluate_helper("commaSep", vec![rule]).unwrap();
        match &result {
            Rule::Optional { value } => {
                match value.as_ref() {
                    Rule::Seq { members } => {
                        prop_assert_eq!(members.len(), 2);
                        // Second element must be Repeat
                        match &members[1] {
                            Rule::Repeat { .. } => {}
                            other => prop_assert!(false, "Expected Repeat, got {:?}", other),
                        }
                    }
                    other => prop_assert!(false, "Expected Seq, got {:?}", other),
                }
            }
            other => prop_assert!(false, "Expected Optional, got {:?}", other),
        }
    }
}

// ---------------------------------------------------------------------------
// 12. HelperFunctions: commaSep1 produces Seq (non-optional)
// ---------------------------------------------------------------------------
proptest! {
    #![proptest_config(ProptestConfig::with_cases(80))]

    #[test]
    fn helper_comma_sep1_structure(rule in leaf_rule_strategy()) {
        use adze_tool::grammar_js::helpers::HelperFunctions;
        let result = HelperFunctions::evaluate_helper("commaSep1", vec![rule]).unwrap();
        match &result {
            Rule::Seq { members } => {
                prop_assert_eq!(members.len(), 2);
                match &members[1] {
                    Rule::Repeat { .. } => {}
                    other => prop_assert!(false, "Expected Repeat, got {:?}", other),
                }
            }
            other => prop_assert!(false, "Expected Seq, got {:?}", other),
        }
    }
}

// ---------------------------------------------------------------------------
// 13. HelperFunctions: parens/brackets/braces wrap with correct delimiters
// ---------------------------------------------------------------------------
proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn helper_delimiters(rule in leaf_rule_strategy()) {
        use adze_tool::grammar_js::helpers::HelperFunctions;

        for (helper, open, close) in [
            ("parens", "(", ")"),
            ("brackets", "[", "]"),
            ("braces", "{", "}"),
        ] {
            let result = HelperFunctions::evaluate_helper(helper, vec![rule.clone()]).unwrap();
            match &result {
                Rule::Seq { members } => {
                    prop_assert_eq!(members.len(), 3, "Expected 3 members for {}", helper);
                    match &members[0] {
                        Rule::String { value } => prop_assert_eq!(value, open),
                        other => prop_assert!(false, "Expected open String, got {:?}", other),
                    }
                    match &members[2] {
                        Rule::String { value } => prop_assert_eq!(value, close),
                        other => prop_assert!(false, "Expected close String, got {:?}", other),
                    }
                }
                other => prop_assert!(false, "Expected Seq for {}, got {:?}", helper, other),
            }
        }
    }
}

// ---------------------------------------------------------------------------
// 14. HelperFunctions: wrong arity is rejected
// ---------------------------------------------------------------------------
proptest! {
    #[test]
    fn helper_wrong_arity_rejected(extra_count in 2usize..5) {
        use adze_tool::grammar_js::helpers::HelperFunctions;
        let args: Vec<Rule> = (0..extra_count).map(|_| Rule::Blank).collect();

        // commaSep / commaSep1 expect exactly 1 arg
        prop_assert!(HelperFunctions::evaluate_helper("commaSep", args.clone()).is_err());
        prop_assert!(HelperFunctions::evaluate_helper("commaSep1", args.clone()).is_err());
        prop_assert!(HelperFunctions::evaluate_helper("parens", args.clone()).is_err());
        prop_assert!(HelperFunctions::evaluate_helper("brackets", args.clone()).is_err());
        prop_assert!(HelperFunctions::evaluate_helper("braces", args).is_err());
    }
}

// ---------------------------------------------------------------------------
// 15. HelperFunctions: is_helper_function consistency
// ---------------------------------------------------------------------------
proptest! {
    #[test]
    fn is_helper_recognizes_known_names(
        name in prop::sample::select(vec![
            "commaSep", "commaSep1", "sep", "sep1",
            "parens", "brackets", "braces",
        ])
    ) {
        use adze_tool::grammar_js::helpers::HelperFunctions;
        prop_assert!(HelperFunctions::is_helper_function(name));
    }

    #[test]
    fn is_helper_rejects_unknown_names(name in "[a-z]{1,8}") {
        use adze_tool::grammar_js::helpers::HelperFunctions;
        let known = [
            "commaSep", "commaSep1", "sep", "sep1", "sepBy", "sepBy1",
            "list", "list1", "delimited", "parens", "brackets", "braces",
        ];
        if !known.contains(&name.as_str()) {
            prop_assert!(!HelperFunctions::is_helper_function(&name));
        }
    }
}

// ---------------------------------------------------------------------------
// 16. Converter preserves grammar name
// ---------------------------------------------------------------------------
proptest! {
    #![proptest_config(ProptestConfig::with_cases(40))]

    #[test]
    fn converter_preserves_name(g in grammar_js_strategy()) {
        let name = g.name.clone();
        let converter = GrammarJsConverter::new(g);
        let ir = converter.convert();
        // Conversion may or may not succeed depending on rule content,
        // but if it does the name should be preserved.
        if let Ok(grammar) = ir {
            prop_assert_eq!(grammar.name, name);
        }
    }
}

// ---------------------------------------------------------------------------
// 17. Converter: token deduplication
// ---------------------------------------------------------------------------
proptest! {
    #![proptest_config(ProptestConfig::with_cases(40))]

    #[test]
    fn converter_deduplicates_string_tokens(val in "[a-z]{1,5}") {
        // Build a grammar that uses the same string literal in two rules
        let mut g = GrammarJs::new("dedup_test".to_string());
        g.rules.insert(
            "rule_a".to_string(),
            Rule::String { value: val.clone() },
        );
        g.rules.insert(
            "rule_b".to_string(),
            Rule::String { value: val },
        );
        let converter = GrammarJsConverter::new(g);
        if let Ok(grammar) = converter.convert() {
            // Count how many tokens have a String pattern with our value
            let string_tokens: Vec<_> = grammar.tokens.values()
                .filter(|t| matches!(&t.pattern, adze_ir::TokenPattern::String(_)))
                .collect();
            // Each unique string value should appear at most once
            let mut seen = std::collections::HashSet::new();
            for t in &string_tokens {
                if let adze_ir::TokenPattern::String(s) = &t.pattern {
                    prop_assert!(seen.insert(s.clone()), "Duplicate string token: {}", s);
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// 18. JSON rule parse roundtrip through serde
// ---------------------------------------------------------------------------
proptest! {
    #![proptest_config(ProptestConfig::with_cases(150))]

    #[test]
    fn json_rule_parse_roundtrip(rule in rule_strategy()) {
        // Serialize Rule to JSON Value, then deserialize back
        let val: Value = serde_json::to_value(&rule).unwrap();
        let back: Rule = serde_json::from_value(val.clone()).unwrap();
        let val2: Value = serde_json::to_value(&back).unwrap();
        prop_assert_eq!(val, val2);
    }
}

// ---------------------------------------------------------------------------
// 19. from_json preserves extras count
// ---------------------------------------------------------------------------
proptest! {
    #![proptest_config(ProptestConfig::with_cases(40))]

    #[test]
    fn from_json_preserves_extras(extras_count in 0usize..4) {
        let mut extras = Vec::new();
        for _ in 0..extras_count {
            extras.push(json!({"type": "PATTERN", "value": r"\s"}));
        }
        let val = json!({
            "name": "extras_test",
            "rules": {
                "start": {"type": "BLANK"}
            },
            "extras": extras,
        });
        let g = adze_tool::grammar_js::from_json(&val).unwrap();
        prop_assert_eq!(g.extras.len(), extras_count);
    }
}

// ---------------------------------------------------------------------------
// 20. from_json preserves externals
// ---------------------------------------------------------------------------
proptest! {
    #![proptest_config(ProptestConfig::with_cases(30))]

    #[test]
    fn from_json_preserves_externals(names in prop::collection::vec(ident_strategy(), 0..=3)) {
        let externals: Vec<Value> = names.iter().enumerate()
            .map(|(i, n)| json!({"name": n, "symbol": format!("external_{}", i)}))
            .collect();
        let val = json!({
            "name": "ext_test",
            "rules": { "start": {"type": "BLANK"} },
            "externals": externals,
        });
        let g = adze_tool::grammar_js::from_json(&val).unwrap();
        prop_assert_eq!(g.externals.len(), names.len());
    }
}

// ---------------------------------------------------------------------------
// 21. GrammarJs::new produces empty collections
// ---------------------------------------------------------------------------
proptest! {
    #[test]
    fn grammar_js_new_is_empty(name in ident_strategy()) {
        let g = GrammarJs::new(name.clone());
        prop_assert_eq!(g.name, name);
        prop_assert!(g.word.is_none());
        prop_assert!(g.rules.is_empty());
        prop_assert!(g.extras.is_empty());
        prop_assert!(g.externals.is_empty());
        prop_assert!(g.inline.is_empty());
        prop_assert!(g.conflicts.is_empty());
        prop_assert!(g.supertypes.is_empty());
        prop_assert!(g.precedences.is_empty());
    }
}
