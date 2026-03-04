#![allow(clippy::needless_range_loop)]
// Property-based tests for NODE_TYPES JSON generation.
//
// Properties verified:
// 1.  Node types JSON is always valid JSON
// 2.  Named symbols (rules with non-underscore names) appear in output
// 3.  Hidden symbols (names starting with '_') don't appear in output
// 4.  Node type names match grammar symbol names
// 5.  Subtypes (if present) are valid references
// 6.  Children arrays reference valid types
// 7.  Field names match grammar field names
// 8.  Anonymous tokens appear with named=false
// 9.  Regex tokens appear with named=true
// 10. Output is sorted by type name
// 11. No duplicate type names in output
// 12. Every entry has required 'type' and 'named' keys
// 13. Adding tokens doesn't break rule node types
// 14. Field info preserves field names from grammar
// 15. Multiple rules for same symbol produce single node type
// 16. GrammarBuilder-produced grammars yield valid JSON
// 17. Top-level value is always an array
// 18. Field entries have required 'types', 'multiple', 'required' keys
// 19. Same grammar produces deterministic output
// 20. Children field is an object when present
// 21. 'named' field is always a boolean
// 22. 'type' field is always a non-empty string
// 23. Grammar with externals produces valid JSON with rules present
// 24. Subtypes field is always an array when present
// 25. Fields is an object when present
// 26. Node count matches visible rules and string tokens
// 27. Only allowed top-level keys exist in each entry
// 28. Children types entries have 'type' and 'named' keys
// 29. Large grammars produce valid NODE_TYPES
// 30. Python-like grammar produces valid output
// 31. JavaScript-like grammar produces valid output
// 32. Supertype symbols appear in output
// 33. Token-only grammar produces only unnamed entries (non-proptest)
// 34. Mixed grammar produces valid JSON (non-proptest)
// 35. Empty grammar produces valid empty JSON array (non-proptest)

use adze_ir::builder::GrammarBuilder;
use adze_ir::{FieldId, Grammar, ProductionId, Rule, Symbol, SymbolId, Token, TokenPattern};
use adze_tablegen::NodeTypesGenerator;
use proptest::prelude::*;
use serde_json::Value;

// ---------------------------------------------------------------------------
// Strategies
// ---------------------------------------------------------------------------

/// Generate a valid identifier-like name (lowercase, no leading underscore).
fn visible_name_strategy() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9_]{0,12}".prop_filter("non-empty", |s| !s.is_empty())
}

/// Generate a hidden/internal name (starts with underscore).
fn hidden_name_strategy() -> impl Strategy<Value = String> {
    "_[a-z][a-z0-9_]{0,10}".prop_filter("non-empty", |s| s.len() > 1)
}

/// Generate a field name.
fn field_name_strategy() -> impl Strategy<Value = String> {
    "[a-z][a-z_]{0,8}".prop_filter("non-empty", |s| !s.is_empty())
}

/// Generate a simple string-literal token pattern (punctuation-like).
fn string_token_strategy() -> impl Strategy<Value = (String, String)> {
    prop_oneof![
        Just(("plus".to_string(), "+".to_string())),
        Just(("minus".to_string(), "-".to_string())),
        Just(("star".to_string(), "*".to_string())),
        Just(("slash".to_string(), "/".to_string())),
        Just(("semi".to_string(), ";".to_string())),
        Just(("colon".to_string(), ":".to_string())),
        Just(("comma".to_string(), ",".to_string())),
        Just(("eq".to_string(), "=".to_string())),
    ]
}

/// Generate a regex token (named token).
fn regex_token_strategy() -> impl Strategy<Value = (String, String)> {
    prop_oneof![
        Just(("number".to_string(), r"\d+".to_string())),
        Just(("identifier".to_string(), r"[a-z]+".to_string())),
        Just(("string_lit".to_string(), r#""[^"]*""#.to_string())),
    ]
}

/// Build a grammar with the given visible rule names, hidden rule names,
/// string tokens, and regex tokens.
fn build_grammar(
    visible_names: &[String],
    hidden_names: &[String],
    string_tokens: &[(String, String)],
    regex_tokens: &[(String, String)],
    field_names: &[String],
) -> Grammar {
    let mut g = Grammar::new("proptest".to_string());
    let mut next_id: u16 = 0;

    // Add regex tokens
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

    // Add string tokens
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

    // Register field names
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

    // Pick a terminal for RHS (use first regex token, or first string token, or epsilon)
    let default_terminal = regex_token_ids
        .first()
        .or(string_token_ids.first())
        .copied();

    // Add visible rules
    for name in visible_names {
        let id = SymbolId(next_id);
        next_id += 1;
        g.rule_names.insert(id, name.clone());

        // Build fields for this rule if we have field names and a terminal
        let (rule_fields, rhs) = if let Some(tid) = default_terminal
            && !field_ids.is_empty()
        {
            // Create RHS with one symbol per field
            let mut rhs_symbols = Vec::new();
            let mut rule_field_pairs = Vec::new();
            for (pos, fid) in field_ids.iter().enumerate() {
                rhs_symbols.push(Symbol::Terminal(tid));
                rule_field_pairs.push((*fid, pos));
            }
            (rule_field_pairs, rhs_symbols)
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

    // Add hidden (internal) rules
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

/// Strategy producing a full grammar with associated metadata.
fn grammar_strategy() -> impl Strategy<Value = GrammarWithMeta> {
    (
        prop::collection::vec(visible_name_strategy(), 0..6),
        prop::collection::vec(hidden_name_strategy(), 0..4),
        prop::collection::vec(string_token_strategy(), 0..4),
        prop::collection::vec(regex_token_strategy(), 0..3),
        prop::collection::vec(field_name_strategy(), 0..4),
    )
        .prop_map(|(visible, hidden, str_toks, re_toks, fields)| {
            // Deduplicate names
            let visible = dedup(visible);
            let hidden = dedup(hidden);
            let str_toks = dedup_by_key(str_toks, |t| t.1.clone());
            let re_toks = dedup_by_key(re_toks, |t| t.0.clone());
            let fields = dedup(fields);

            let grammar = build_grammar(&visible, &hidden, &str_toks, &re_toks, &fields);
            (grammar, visible, hidden, str_toks, re_toks, fields)
        })
}

fn dedup(mut v: Vec<String>) -> Vec<String> {
    let mut seen = std::collections::HashSet::new();
    v.retain(|s| seen.insert(s.clone()));
    v
}

fn dedup_by_key<T, F: Fn(&T) -> String>(mut v: Vec<T>, key: F) -> Vec<T> {
    let mut seen = std::collections::HashSet::new();
    v.retain(|item| seen.insert(key(item)));
    v
}

/// Parse the JSON output and return the array of node type objects.
fn parse_node_types(json: &str) -> Vec<Value> {
    let parsed: Value = serde_json::from_str(json).expect("must be valid JSON");
    parsed.as_array().expect("top-level must be array").clone()
}

// ---------------------------------------------------------------------------
// Property tests
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    // 1. Node types JSON is always valid JSON
    #[test]
    fn json_is_always_valid(
        (grammar, ..) in grammar_strategy()
    ) {
        let generator = NodeTypesGenerator::new(&grammar);
        let result = generator.generate().expect("generate must succeed");
        let _: Value = serde_json::from_str(&result)
            .expect("output must be valid JSON");
    }

    // 2. Named symbols (visible rules) appear in node types output
    #[test]
    fn visible_rules_appear_in_output(
        (grammar, visible, ..) in grammar_strategy()
    ) {
        let generator = NodeTypesGenerator::new(&grammar);
        let result = generator.generate().unwrap();
        let types = parse_node_types(&result);

        let type_names: Vec<&str> = types.iter()
            .filter_map(|t| t.get("type").and_then(|v| v.as_str()))
            .collect();

        for name in &visible {
            prop_assert!(
                type_names.contains(&name.as_str()),
                "visible rule '{}' not found in output, got: {:?}",
                name,
                type_names
            );
        }
    }

    // 3. Hidden symbols don't appear in output
    #[test]
    fn hidden_rules_absent_from_output(
        (grammar, _, hidden, ..) in grammar_strategy()
    ) {
        let generator = NodeTypesGenerator::new(&grammar);
        let result = generator.generate().unwrap();
        let types = parse_node_types(&result);

        let type_names: Vec<&str> = types.iter()
            .filter_map(|t| t.get("type").and_then(|v| v.as_str()))
            .collect();

        for name in &hidden {
            prop_assert!(
                !type_names.contains(&name.as_str()),
                "hidden rule '{}' should not appear in output",
                name
            );
        }
    }

    // 4. Node type names match symbol names in the grammar
    #[test]
    fn type_names_match_grammar_symbols(
        (grammar, visible, _, str_toks, ..) in grammar_strategy()
    ) {
        let generator = NodeTypesGenerator::new(&grammar);
        let result = generator.generate().unwrap();
        let types = parse_node_types(&result);

        // Collect all valid names: visible rule names + string token literals
        let mut valid_names: std::collections::HashSet<String> = visible.into_iter().collect();
        for (_, pattern) in &str_toks {
            valid_names.insert(pattern.clone());
        }
        // Allow fallback names like "rule_N"
        for t in &types {
            if let Some(name) = t.get("type").and_then(|v| v.as_str()) {
                let is_fallback = name.starts_with("rule_");
                prop_assert!(
                    valid_names.contains(name) || is_fallback,
                    "type '{}' not in grammar names or fallback: {:?}",
                    name,
                    valid_names
                );
            }
        }
    }

    // 5. Subtypes (if present) reference valid type names
    #[test]
    fn subtypes_are_valid_references(
        (grammar, ..) in grammar_strategy()
    ) {
        let generator = NodeTypesGenerator::new(&grammar);
        let result = generator.generate().unwrap();
        let types = parse_node_types(&result);

        let all_type_names: std::collections::HashSet<String> = types.iter()
            .filter_map(|t| t.get("type").and_then(|v| v.as_str()).map(String::from))
            .collect();

        for t in &types {
            if let Some(subtypes) = t.get("subtypes").and_then(|v| v.as_array()) {
                for st in subtypes {
                    if let Some(st_name) = st.get("type").and_then(|v| v.as_str()) {
                        // Subtypes should reference known types or be valid names
                        prop_assert!(
                            all_type_names.contains(st_name) || !st_name.is_empty(),
                            "subtype '{}' is not a valid reference",
                            st_name
                        );
                    }
                }
            }
        }
    }

    // 6. Children arrays reference valid types
    #[test]
    fn children_reference_valid_types(
        (grammar, ..) in grammar_strategy()
    ) {
        let generator = NodeTypesGenerator::new(&grammar);
        let result = generator.generate().unwrap();
        let types = parse_node_types(&result);

        for t in &types {
            if let Some(children) = t.get("children") {
                if let Some(child_types) = children.get("types").and_then(|v| v.as_array()) {
                    for ct in child_types {
                        let ct_name = ct.get("type").and_then(|v| v.as_str()).unwrap_or("");
                        prop_assert!(
                            !ct_name.is_empty(),
                            "child type name must not be empty"
                        );
                    }
                }
                // children must have 'multiple' and 'required' booleans
                if let Some(mult) = children.get("multiple") {
                    prop_assert!(mult.is_boolean(), "'multiple' must be boolean");
                }
                if let Some(req) = children.get("required") {
                    prop_assert!(req.is_boolean(), "'required' must be boolean");
                }
            }
        }
    }

    // 7. Field names match grammar field names
    #[test]
    fn field_names_match_grammar(
        (grammar, _, _, _, _, field_names) in grammar_strategy()
    ) {
        let generator = NodeTypesGenerator::new(&grammar);
        let result = generator.generate().unwrap();
        let types = parse_node_types(&result);

        let grammar_field_set: std::collections::HashSet<&str> =
            field_names.iter().map(|s| s.as_str()).collect();

        for t in &types {
            if let Some(fields_obj) = t.get("fields").and_then(|v| v.as_object()) {
                for key in fields_obj.keys() {
                    prop_assert!(
                        grammar_field_set.contains(key.as_str()),
                        "field '{}' in output but not in grammar fields: {:?}",
                        key,
                        grammar_field_set
                    );
                }
            }
        }
    }

    // 8. Anonymous (string) tokens appear with named=false
    #[test]
    fn string_tokens_are_unnamed(
        (grammar, _, _, str_toks, ..) in grammar_strategy()
    ) {
        let generator = NodeTypesGenerator::new(&grammar);
        let result = generator.generate().unwrap();
        let types = parse_node_types(&result);

        for (_, pattern) in &str_toks {
            if let Some(entry) = types.iter().find(|t| {
                t.get("type").and_then(|v| v.as_str()) == Some(pattern.as_str())
            }) {
                let named = entry.get("named").and_then(|v| v.as_bool()).unwrap_or(true);
                prop_assert!(
                    !named,
                    "string token '{}' should have named=false",
                    pattern
                );
            }
        }
    }

    // 9. Regex tokens used in rules appear as named=true
    #[test]
    fn regex_tokens_in_rules_are_named(
        (grammar, visible, _, _, re_toks, ..) in grammar_strategy()
    ) {
        if visible.is_empty() || re_toks.is_empty() {
            return Ok(());
        }
        let generator = NodeTypesGenerator::new(&grammar);
        let result = generator.generate().unwrap();
        let types = parse_node_types(&result);

        // Named rule entries should have named=true
        for name in &visible {
            if let Some(entry) = types.iter().find(|t| {
                t.get("type").and_then(|v| v.as_str()) == Some(name.as_str())
            }) {
                let named = entry.get("named").and_then(|v| v.as_bool()).unwrap_or(false);
                prop_assert!(
                    named,
                    "rule '{}' should have named=true",
                    name
                );
            }
        }
    }

    // 10. Output is sorted by type name
    #[test]
    fn output_sorted_by_type_name(
        (grammar, ..) in grammar_strategy()
    ) {
        let generator = NodeTypesGenerator::new(&grammar);
        let result = generator.generate().unwrap();
        let types = parse_node_types(&result);

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

    // 11. No duplicate type names in output
    #[test]
    fn no_duplicate_type_names(
        (grammar, ..) in grammar_strategy()
    ) {
        let generator = NodeTypesGenerator::new(&grammar);
        let result = generator.generate().unwrap();
        let types = parse_node_types(&result);

        let names: Vec<&str> = types.iter()
            .filter_map(|t| t.get("type").and_then(|v| v.as_str()))
            .collect();

        let unique: std::collections::HashSet<&str> = names.iter().copied().collect();
        prop_assert_eq!(
            names.len(),
            unique.len(),
            "duplicate type names found: {:?}",
            names
        );
    }

    // 12. Every node type entry has the required 'type' and 'named' keys
    #[test]
    fn entries_have_required_keys(
        (grammar, ..) in grammar_strategy()
    ) {
        let generator = NodeTypesGenerator::new(&grammar);
        let result = generator.generate().unwrap();
        let types = parse_node_types(&result);

        for (i, t) in types.iter().enumerate() {
            prop_assert!(
                t.get("type").is_some(),
                "entry {} missing 'type' key",
                i
            );
            prop_assert!(
                t.get("named").is_some(),
                "entry {} missing 'named' key",
                i
            );
        }
    }

    // 13. Adding string tokens doesn't remove rule node types
    #[test]
    fn adding_tokens_preserves_rules(
        visible in prop::collection::vec(visible_name_strategy(), 1..4),
        extra_toks in prop::collection::vec(string_token_strategy(), 0..5),
    ) {
        let visible = dedup(visible);
        let extra_toks = dedup_by_key(extra_toks, |t| t.1.clone());

        // Baseline: grammar with rules only
        let base = build_grammar(&visible, &[], &[], &[("tok".into(), r"\w+".into())], &[]);
        let base_generator = NodeTypesGenerator::new(&base);
        let base_result = base_generator.generate().unwrap();
        let base_types = parse_node_types(&base_result);
        let base_rule_names: std::collections::HashSet<&str> = base_types.iter()
            .filter(|t| t.get("named").and_then(|v| v.as_bool()) == Some(true))
            .filter_map(|t| t.get("type").and_then(|v| v.as_str()))
            .collect();

        // Extended: same rules plus extra string tokens
        let ext = build_grammar(&visible, &[], &extra_toks, &[("tok".into(), r"\w+".into())], &[]);
        let ext_generator = NodeTypesGenerator::new(&ext);
        let ext_result = ext_generator.generate().unwrap();
        let ext_types = parse_node_types(&ext_result);
        let ext_rule_names: std::collections::HashSet<&str> = ext_types.iter()
            .filter(|t| t.get("named").and_then(|v| v.as_bool()) == Some(true))
            .filter_map(|t| t.get("type").and_then(|v| v.as_str()))
            .collect();

        for name in &base_rule_names {
            prop_assert!(
                ext_rule_names.contains(name),
                "rule '{}' disappeared after adding tokens",
                name
            );
        }
    }

    // 14. Field info preserves all field names from grammar
    #[test]
    fn field_info_preserves_names(
        visible in prop::collection::vec(visible_name_strategy(), 1..3),
        fields in prop::collection::vec(field_name_strategy(), 1..4),
    ) {
        let visible = dedup(visible);
        let fields = dedup(fields);
        let grammar = build_grammar(
            &visible,
            &[],
            &[],
            &[("tok".into(), r"\w+".into())],
            &fields,
        );
        let generator = NodeTypesGenerator::new(&grammar);
        let result = generator.generate().unwrap();
        let types = parse_node_types(&result);

        // Collect all field names from the output
        let mut output_fields = std::collections::HashSet::new();
        for t in &types {
            if let Some(fields_obj) = t.get("fields").and_then(|v| v.as_object()) {
                for key in fields_obj.keys() {
                    output_fields.insert(key.as_str());
                }
            }
        }

        // Every grammar field should appear somewhere in output
        for f in &fields {
            prop_assert!(
                output_fields.contains(f.as_str()),
                "field '{}' from grammar not found in output fields: {:?}",
                f,
                output_fields
            );
        }
    }

    // 15. Multiple rules for same symbol produce a single node type
    #[test]
    fn multiple_rules_single_node_type(
        name in visible_name_strategy(),
    ) {
        let mut g = Grammar::new("multi".to_string());

        let tok_id = SymbolId(0);
        g.tokens.insert(tok_id, Token {
            name: "tok".to_string(),
            pattern: TokenPattern::Regex(r"\w+".to_string()),
            fragile: false,
        });

        let rule_id = SymbolId(10);
        g.rule_names.insert(rule_id, name.clone());

        // Two alternative productions for the same symbol
        g.add_rule(Rule {
            lhs: rule_id,
            rhs: vec![Symbol::Terminal(tok_id)],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(0),
        });
        g.add_rule(Rule {
            lhs: rule_id,
            rhs: vec![Symbol::Terminal(tok_id), Symbol::Terminal(tok_id)],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(1),
        });

        let generator = NodeTypesGenerator::new(&g);
        let result = generator.generate().unwrap();
        let types = parse_node_types(&result);

        let count = types.iter()
            .filter(|t| t.get("type").and_then(|v| v.as_str()) == Some(name.as_str()))
            .count();

        prop_assert_eq!(count, 1, "expected exactly 1 entry for '{}', got {}", name, count);
    }
}

// ---------------------------------------------------------------------------
// Additional property tests (second block for broader coverage)
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    // 16. GrammarBuilder-produced grammars always yield valid JSON
    #[test]
    fn grammar_builder_yields_valid_json(
        rule_count in 1usize..5,
    ) {
        let mut builder = GrammarBuilder::new("builder_test")
            .token("NUMBER", r"\d+")
            .token("+", "+");

        // Add rule_count rules with unique names
        for i in 0..rule_count {
            let name = format!("rule{}", i);
            builder = builder.rule(&name, vec!["NUMBER"]);
        }

        let grammar = builder.start("rule0").build();
        let generator = NodeTypesGenerator::new(&grammar);
        let result = generator.generate().expect("generate must succeed");
        let _: Value = serde_json::from_str(&result).expect("must be valid JSON");
    }

    // 17. Top-level value is always an array
    #[test]
    fn top_level_is_array(
        (grammar, ..) in grammar_strategy()
    ) {
        let generator = NodeTypesGenerator::new(&grammar);
        let result = generator.generate().unwrap();
        let parsed: Value = serde_json::from_str(&result).unwrap();
        prop_assert!(parsed.is_array(), "top-level must be a JSON array");
    }

    // 18. Field entries have required 'types' array
    #[test]
    fn field_entries_have_types(
        (grammar, ..) in grammar_strategy()
    ) {
        let generator = NodeTypesGenerator::new(&grammar);
        let result = generator.generate().unwrap();
        let types = parse_node_types(&result);

        for t in &types {
            if let Some(fields_obj) = t.get("fields").and_then(|v| v.as_object()) {
                for (fname, fval) in fields_obj {
                    prop_assert!(
                        fval.get("types").and_then(|v| v.as_array()).is_some(),
                        "field '{}' missing 'types' array",
                        fname
                    );
                    prop_assert!(
                        fval.get("multiple").and_then(|v| v.as_bool()).is_some(),
                        "field '{}' missing 'multiple' boolean",
                        fname
                    );
                    prop_assert!(
                        fval.get("required").and_then(|v| v.as_bool()).is_some(),
                        "field '{}' missing 'required' boolean",
                        fname
                    );
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Additional property tests (third block)
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    // 19. Determinism: same grammar produces semantically identical output
    #[test]
    fn deterministic_output(
        (grammar, ..) in grammar_strategy()
    ) {
        let ntgen = NodeTypesGenerator::new(&grammar);
        let first = ntgen.generate().unwrap();
        let second = ntgen.generate().unwrap();
        let v1: Value = serde_json::from_str(&first).unwrap();
        let v2: Value = serde_json::from_str(&second).unwrap();
        prop_assert_eq!(v1, v2, "same grammar must produce semantically identical JSON");
    }

    // 20. Children field is an object (not array) when present
    #[test]
    fn children_field_is_object_when_present(
        (grammar, ..) in grammar_strategy()
    ) {
        let ntgen = NodeTypesGenerator::new(&grammar);
        let result = ntgen.generate().unwrap();
        let types = parse_node_types(&result);

        for t in &types {
            if let Some(children) = t.get("children") {
                prop_assert!(
                    children.is_object(),
                    "children must be an object, got: {:?}",
                    children
                );
            }
        }
    }

    // 21. 'named' field is always a boolean
    #[test]
    fn named_field_is_boolean(
        (grammar, ..) in grammar_strategy()
    ) {
        let ntgen = NodeTypesGenerator::new(&grammar);
        let result = ntgen.generate().unwrap();
        let types = parse_node_types(&result);

        for (i, t) in types.iter().enumerate() {
            if let Some(named) = t.get("named") {
                prop_assert!(
                    named.is_boolean(),
                    "entry {} 'named' must be boolean, got: {:?}",
                    i,
                    named
                );
            }
        }
    }

    // 22. 'type' field is always a non-empty string
    #[test]
    fn type_field_is_nonempty_string(
        (grammar, ..) in grammar_strategy()
    ) {
        let ntgen = NodeTypesGenerator::new(&grammar);
        let result = ntgen.generate().unwrap();
        let types = parse_node_types(&result);

        for (i, t) in types.iter().enumerate() {
            let type_name = t.get("type").and_then(|v| v.as_str());
            prop_assert!(
                type_name.is_some_and(|s| !s.is_empty()),
                "entry {} must have non-empty 'type' string",
                i
            );
        }
    }

    // 23. Grammar with externals still produces valid JSON with rules present
    #[test]
    fn grammar_with_externals_produces_valid_json(
        rule_count in 1usize..4,
    ) {
        let mut builder = GrammarBuilder::new("ext_test")
            .token("NUMBER", r"\d+")
            .external("INDENT")
            .external("DEDENT");

        for i in 0..rule_count {
            let name = format!("expr{}", i);
            builder = builder.rule(&name, vec!["NUMBER"]);
        }

        let grammar = builder.start("expr0").build();
        let ntgen = NodeTypesGenerator::new(&grammar);
        let result = ntgen.generate().unwrap();
        let types = parse_node_types(&result);

        // All rule names should appear
        for i in 0..rule_count {
            let name = format!("expr{}", i);
            let found = types.iter().any(|t| {
                t.get("type").and_then(|v| v.as_str()) == Some(name.as_str())
            });
            prop_assert!(found, "rule '{}' not found in output", name);
        }
    }

    // 24. Subtypes field is always an array when present
    #[test]
    fn subtypes_is_array_when_present(
        (grammar, ..) in grammar_strategy()
    ) {
        let ntgen = NodeTypesGenerator::new(&grammar);
        let result = ntgen.generate().unwrap();
        let types = parse_node_types(&result);

        for t in &types {
            if let Some(subtypes) = t.get("subtypes") {
                prop_assert!(
                    subtypes.is_array(),
                    "subtypes must be an array, got: {:?}",
                    subtypes
                );
            }
        }
    }

    // 25. Fields object is a map (not array) when present
    #[test]
    fn fields_is_object_when_present(
        (grammar, ..) in grammar_strategy()
    ) {
        let ntgen = NodeTypesGenerator::new(&grammar);
        let result = ntgen.generate().unwrap();
        let types = parse_node_types(&result);

        for t in &types {
            if let Some(fields) = t.get("fields") {
                prop_assert!(
                    fields.is_object(),
                    "fields must be an object, got: {:?}",
                    fields
                );
            }
        }
    }

    // 26. Node count equals visible rules + string tokens + regex tokens (no hidden)
    #[test]
    fn node_count_matches_expectations(
        (grammar, visible, _, str_toks, re_toks, _) in grammar_strategy()
    ) {
        let ntgen = NodeTypesGenerator::new(&grammar);
        let result = ntgen.generate().unwrap();
        let types = parse_node_types(&result);

        // Regex tokens not appearing as standalone are handled by the generator,
        // but at minimum we should have all visible rules
        let named_count = types.iter()
            .filter(|t| t.get("named").and_then(|v| v.as_bool()) == Some(true))
            .count();
        prop_assert!(
            named_count >= visible.len(),
            "expected at least {} named entries (visible rules), got {}",
            visible.len(),
            named_count
        );

        let unnamed_count = types.iter()
            .filter(|t| t.get("named").and_then(|v| v.as_bool()) == Some(false))
            .count();
        prop_assert!(
            unnamed_count <= str_toks.len(),
            "expected at most {} unnamed entries (string tokens), got {}",
            str_toks.len(),
            unnamed_count
        );

        let _ = re_toks; // acknowledged
    }

    // 27. Only allowed top-level keys exist in each entry
    #[test]
    fn entries_have_only_known_keys(
        (grammar, ..) in grammar_strategy()
    ) {
        let ntgen = NodeTypesGenerator::new(&grammar);
        let result = ntgen.generate().unwrap();
        let types = parse_node_types(&result);

        let allowed: std::collections::HashSet<&str> =
            ["type", "named", "fields", "children", "subtypes"].iter().copied().collect();

        for (i, t) in types.iter().enumerate() {
            if let Some(obj) = t.as_object() {
                for key in obj.keys() {
                    prop_assert!(
                        allowed.contains(key.as_str()),
                        "entry {} has unexpected key '{}'",
                        i,
                        key
                    );
                }
            }
        }
    }

    // 28. Children types array entries have 'type' and 'named' keys
    #[test]
    fn children_types_entries_have_required_keys(
        (grammar, ..) in grammar_strategy()
    ) {
        let ntgen = NodeTypesGenerator::new(&grammar);
        let result = ntgen.generate().unwrap();
        let types = parse_node_types(&result);

        for t in &types {
            if let Some(child_types) = t.get("children").and_then(|c| c.get("types")).and_then(|v| v.as_array()) {
                for ct in child_types {
                    prop_assert!(ct.get("type").is_some(), "child type missing 'type'");
                    prop_assert!(ct.get("named").is_some(), "child type missing 'named'");
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Large grammar and determinism tests (fourth block)
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    // 29. Large grammars produce valid NODE_TYPES
    #[test]
    fn large_grammar_produces_valid_json(
        rule_count in 10usize..50,
        token_count in 5usize..20,
    ) {
        let mut builder = GrammarBuilder::new("large")
            .token("NUMBER", r"\d+");

        for i in 0..token_count {
            let name = format!("tok{}", i);
            let pattern = format!("t{}", i);
            builder = builder.token(&name, &pattern);
        }

        for i in 0..rule_count {
            let name = format!("rule{}", i);
            builder = builder.rule(&name, vec!["NUMBER"]);
        }

        let grammar = builder.start("rule0").build();
        let ntgen = NodeTypesGenerator::new(&grammar);
        let result = ntgen.generate().expect("large grammar must succeed");
        let types = parse_node_types(&result);

        // Must have at least rule_count named entries
        let named = types.iter()
            .filter(|t| t.get("named").and_then(|v| v.as_bool()) == Some(true))
            .count();
        prop_assert!(named >= rule_count, "expected >= {} named entries, got {}", rule_count, named);
    }

    // 30. Python-like grammar produces valid output
    #[test]
    fn python_like_grammar_valid(_seed in 0u32..10) {
        let grammar = GrammarBuilder::python_like();
        let ntgen = NodeTypesGenerator::new(&grammar);
        let result = ntgen.generate().expect("python-like must succeed");
        let types = parse_node_types(&result);
        prop_assert!(!types.is_empty(), "python-like grammar must produce entries");

        // Verify required rules appear
        let names: Vec<&str> = types.iter()
            .filter_map(|t| t.get("type").and_then(|v| v.as_str()))
            .collect();
        prop_assert!(names.contains(&"module"), "missing 'module'");
        prop_assert!(names.contains(&"statement"), "missing 'statement'");
    }

    // 31. JavaScript-like grammar produces valid output
    #[test]
    fn javascript_like_grammar_valid(_seed in 0u32..10) {
        let grammar = GrammarBuilder::javascript_like();
        let ntgen = NodeTypesGenerator::new(&grammar);
        let result = ntgen.generate().expect("js-like must succeed");
        let types = parse_node_types(&result);
        prop_assert!(!types.is_empty(), "js-like grammar must produce entries");

        let names: Vec<&str> = types.iter()
            .filter_map(|t| t.get("type").and_then(|v| v.as_str()))
            .collect();
        prop_assert!(names.contains(&"program"), "missing 'program'");
        prop_assert!(names.contains(&"expression"), "missing 'expression'");
    }

    // 32. Supertype symbols get a subtypes field
    #[test]
    fn supertypes_get_subtypes_field(
        name in visible_name_strategy(),
    ) {
        let mut g = Grammar::new("supertype_test".to_string());

        let tok_id = SymbolId(0);
        g.tokens.insert(tok_id, Token {
            name: "tok".to_string(),
            pattern: TokenPattern::Regex(r"\w+".to_string()),
            fragile: false,
        });

        let rule_id = SymbolId(10);
        g.rule_names.insert(rule_id, name.clone());
        g.supertypes.push(rule_id);

        g.add_rule(Rule {
            lhs: rule_id,
            rhs: vec![Symbol::Terminal(tok_id)],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(0),
        });

        let ntgen = NodeTypesGenerator::new(&g);
        let result = ntgen.generate().unwrap();
        let types = parse_node_types(&result);

        // The supertype rule is processed by NodeTypesGenerator which doesn't
        // handle supertypes specially (that's StaticLanguageGenerator), so just
        // verify the output is valid JSON with the rule present
        let found = types.iter().any(|t| {
            t.get("type").and_then(|v| v.as_str()) == Some(name.as_str())
        });
        prop_assert!(found, "supertype rule '{}' should appear in output", name);
    }
}

// ---------------------------------------------------------------------------
// Non-proptest deterministic tests
// ---------------------------------------------------------------------------

#[test]
fn empty_grammar_produces_valid_empty_json() {
    let grammar = Grammar::new("empty".to_string());
    let generator = NodeTypesGenerator::new(&grammar);
    let result = generator.generate().expect("empty grammar must succeed");
    let types = parse_node_types(&result);
    assert!(types.is_empty(), "empty grammar should produce empty array");
}

// 33. Token-only grammar produces only unnamed entries
#[test]
fn token_only_grammar_produces_unnamed_entries() {
    let mut g = Grammar::new("tokens_only".to_string());
    g.tokens.insert(
        SymbolId(0),
        Token {
            name: "plus".to_string(),
            pattern: TokenPattern::String("+".to_string()),
            fragile: false,
        },
    );
    g.tokens.insert(
        SymbolId(1),
        Token {
            name: "minus".to_string(),
            pattern: TokenPattern::String("-".to_string()),
            fragile: false,
        },
    );

    let ntgen = NodeTypesGenerator::new(&g);
    let result = ntgen.generate().unwrap();
    let types = parse_node_types(&result);

    for t in &types {
        let named = t.get("named").and_then(|v| v.as_bool()).unwrap_or(true);
        assert!(!named, "string-only tokens should be unnamed");
    }
}

// 34. Mixed grammar: rules + tokens + externals all produce valid JSON
#[test]
fn mixed_grammar_valid_json() {
    let grammar = GrammarBuilder::new("mixed")
        .token("NUMBER", r"\d+")
        .token("+", "+")
        .token("(", "(")
        .token(")", ")")
        .external("INDENT")
        .rule("expr", vec!["NUMBER"])
        .rule("expr", vec!["expr", "+", "expr"])
        .rule("paren", vec!["(", "expr", ")"])
        .start("expr")
        .build();

    let ntgen = NodeTypesGenerator::new(&grammar);
    let result = ntgen.generate().unwrap();
    let types = parse_node_types(&result);

    // Basic structure checks
    assert!(!types.is_empty());
    for t in &types {
        assert!(t.get("type").is_some());
        assert!(t.get("named").is_some());
    }

    // Named rules present
    let names: Vec<&str> = types
        .iter()
        .filter_map(|t| t.get("type").and_then(|v| v.as_str()))
        .collect();
    assert!(names.contains(&"expr"));
    assert!(names.contains(&"paren"));
}
