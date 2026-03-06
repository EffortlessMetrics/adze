#![allow(clippy::needless_range_loop)]

//! Property-based tests for NODE_TYPES generation in adze-tool.
//!
//! Uses proptest to validate invariants of `NodeTypesGenerator`:
//!   - NODE_TYPES output is always valid JSON
//!   - Every named rule appears in NODE_TYPES
//!   - Each entry has `type` and `named` fields
//!   - Children types are listed correctly
//!   - Output is deterministic across multiple runs
//!   - Simple and complex grammars produce correct NODE_TYPES

use adze_ir::{FieldId, Grammar, ProductionId, Rule, Symbol, SymbolId, Token, TokenPattern};
use adze_tablegen::NodeTypesGenerator;
use proptest::prelude::*;
use serde_json::Value;

// ===========================================================================
// Strategies
// ===========================================================================

/// A valid grammar name (lowercase + digits + underscores, starting with a letter).
fn grammar_name_strategy() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9_]{0,12}".prop_filter("must not be empty", |s| !s.is_empty())
}

/// A simple token name (uppercase letters + underscores).
fn _token_name_strategy() -> impl Strategy<Value = String> {
    "[A-Z][A-Z0-9_]{0,8}".prop_filter("must not be empty", |s| !s.is_empty())
}

/// A rule name (lowercase letters, no underscore prefix so it's not internal).
fn _rule_name_strategy() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9]{0,8}".prop_filter("must not be empty", |s| !s.is_empty())
}

/// A simple regex pattern for tokens.
fn _token_pattern_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        Just(r"[a-z]+".to_string()),
        Just(r"\d+".to_string()),
        Just(r"[a-zA-Z_]+".to_string()),
        Just(r"\w+".to_string()),
    ]
}

/// Number of tokens/rules to generate.
fn count_strategy() -> impl Strategy<Value = usize> {
    1..=5usize
}

// ===========================================================================
// Helpers
// ===========================================================================

/// Build a grammar with N named rules, each referencing a single terminal.
fn grammar_with_named_rules(name: &str, rule_names: &[String]) -> Grammar {
    let mut grammar = Grammar::new(name.to_string());

    // One shared terminal
    let tok_id = SymbolId(0);
    grammar.tokens.insert(
        tok_id,
        Token {
            name: "NUMBER".to_string(),
            pattern: TokenPattern::Regex(r"\d+".to_string()),
            fragile: false,
        },
    );

    for (i, rn) in rule_names.iter().enumerate() {
        let sid = SymbolId((i as u16) + 10);
        grammar.rule_names.insert(sid, rn.clone());
        grammar.add_rule(Rule {
            lhs: sid,
            rhs: vec![Symbol::Terminal(tok_id)],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(i as u16),
        });
    }
    grammar
}

/// Build a grammar with N string-literal tokens (unnamed nodes).
fn grammar_with_string_tokens(name: &str, literals: &[String]) -> Grammar {
    let mut grammar = Grammar::new(name.to_string());
    for (i, lit) in literals.iter().enumerate() {
        let id = SymbolId(i as u16);
        grammar.tokens.insert(
            id,
            Token {
                name: format!("tok_{}", i),
                pattern: TokenPattern::String(lit.clone()),
                fragile: false,
            },
        );
    }
    grammar
}

/// Build a grammar with both named rules and string tokens.
fn grammar_with_rules_and_tokens(name: &str, n_rules: usize, n_string_tokens: usize) -> Grammar {
    let mut grammar = Grammar::new(name.to_string());

    // String tokens start at id 0
    for i in 0..n_string_tokens {
        let id = SymbolId(i as u16);
        grammar.tokens.insert(
            id,
            Token {
                name: format!("op_{}", i),
                pattern: TokenPattern::String(format!("op{}", i)),
                fragile: false,
            },
        );
    }

    // A regex token for rules to reference
    let regex_tok = SymbolId((n_string_tokens) as u16);
    grammar.tokens.insert(
        regex_tok,
        Token {
            name: "IDENT".to_string(),
            pattern: TokenPattern::Regex(r"[a-z]+".to_string()),
            fragile: false,
        },
    );

    // Named rules start after tokens
    let base = (n_string_tokens + 1) as u16;
    for i in 0..n_rules {
        let sid = SymbolId(base + i as u16);
        grammar.rule_names.insert(sid, format!("rule{}", i));
        grammar.add_rule(Rule {
            lhs: sid,
            rhs: vec![Symbol::Terminal(regex_tok)],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(i as u16),
        });
    }
    grammar
}

/// Build a grammar with fields on a rule.
fn grammar_with_fields(name: &str, field_names: &[String]) -> Grammar {
    let mut grammar = Grammar::new(name.to_string());

    // Create one token per field
    let mut tok_ids = Vec::new();
    for (i, _) in field_names.iter().enumerate() {
        let tid = SymbolId(i as u16);
        grammar.tokens.insert(
            tid,
            Token {
                name: format!("T{}", i),
                pattern: TokenPattern::Regex(format!("[a-z]{{{}}}", i + 1)),
                fragile: false,
            },
        );
        tok_ids.push(tid);
    }

    // Register field names
    let mut field_ids = Vec::new();
    for (i, fname) in field_names.iter().enumerate() {
        let fid = FieldId(i as u16);
        grammar.fields.insert(fid, fname.clone());
        field_ids.push(fid);
    }

    // One rule with fields mapping to tokens
    let rule_id = SymbolId(100);
    grammar.rule_names.insert(rule_id, "expr".to_string());
    let rhs: Vec<Symbol> = tok_ids.iter().map(|id| Symbol::Terminal(*id)).collect();
    let fields: Vec<(FieldId, usize)> = field_ids
        .iter()
        .enumerate()
        .map(|(i, fid)| (*fid, i))
        .collect();
    grammar.add_rule(Rule {
        lhs: rule_id,
        rhs,
        precedence: None,
        associativity: None,
        fields,
        production_id: ProductionId(0),
    });
    grammar
}

/// Build a chain grammar: rule_N -> rule_{N-1} -> ... -> rule_0 -> terminal.
fn grammar_with_chain(name: &str, depth: usize) -> Grammar {
    let mut grammar = Grammar::new(name.to_string());
    let tok = SymbolId(0);
    grammar.tokens.insert(
        tok,
        Token {
            name: "LEAF".to_string(),
            pattern: TokenPattern::Regex(r"[a-z]+".to_string()),
            fragile: false,
        },
    );
    for i in 0..depth {
        let rule_id = SymbolId((i as u16) + 10);
        grammar.rule_names.insert(rule_id, format!("chain{}", i));
        let rhs = if i == 0 {
            vec![Symbol::Terminal(tok)]
        } else {
            vec![Symbol::NonTerminal(SymbolId((i as u16) + 10 - 1))]
        };
        grammar.add_rule(Rule {
            lhs: rule_id,
            rhs,
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(i as u16),
        });
    }
    grammar
}

/// Parse NODE_TYPES JSON and return the array of entries.
fn parse_node_types(json: &str) -> Vec<Value> {
    let v: Value = serde_json::from_str(json).expect("NODE_TYPES must be valid JSON");
    v.as_array()
        .expect("NODE_TYPES must be a JSON array")
        .clone()
}

/// Extract the set of type names from a NODE_TYPES array.
fn type_names(entries: &[Value]) -> Vec<String> {
    entries
        .iter()
        .filter_map(|e| e.get("type").and_then(|t| t.as_str()).map(String::from))
        .collect()
}

/// Extract only named entries from NODE_TYPES.
fn named_entries(entries: &[Value]) -> Vec<&Value> {
    entries
        .iter()
        .filter(|e| e.get("named").and_then(|n| n.as_bool()) == Some(true))
        .collect()
}

// ===========================================================================
// Tests
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(30))]

    // ---- 1. NODE_TYPES is valid JSON ----

    /// NODE_TYPES output is always valid JSON regardless of grammar name.
    #[test]
    fn node_types_is_valid_json(name in grammar_name_strategy()) {
        let grammar = Grammar::new(name);
        let generator = NodeTypesGenerator::new(&grammar);
        let result = generator.generate().unwrap();
        let parsed: Result<Value, _> = serde_json::from_str(&result);
        prop_assert!(parsed.is_ok(), "Invalid JSON: {}", result);
    }

    /// NODE_TYPES for a grammar with tokens is valid JSON.
    #[test]
    fn node_types_with_tokens_is_valid_json(
        name in grammar_name_strategy(),
        n in count_strategy(),
    ) {
        let lits: Vec<String> = (0..n).map(|i| format!("lit{}", i)).collect();
        let grammar = grammar_with_string_tokens(&name, &lits);
        let generator = NodeTypesGenerator::new(&grammar);
        let result = generator.generate().unwrap();
        prop_assert!(serde_json::from_str::<Value>(&result).is_ok());
    }

    /// NODE_TYPES for a grammar with named rules is valid JSON.
    #[test]
    fn node_types_with_named_rules_is_valid_json(
        name in grammar_name_strategy(),
        n in count_strategy(),
    ) {
        let rules: Vec<String> = (0..n).map(|i| format!("rule{}", i)).collect();
        let grammar = grammar_with_named_rules(&name, &rules);
        let generator = NodeTypesGenerator::new(&grammar);
        let result = generator.generate().unwrap();
        prop_assert!(serde_json::from_str::<Value>(&result).is_ok());
    }

    // ---- 2. Every named type appears in NODE_TYPES ----

    /// Every non-internal rule name appears in NODE_TYPES.
    #[test]
    fn named_rules_appear_in_node_types(
        name in grammar_name_strategy(),
        n in 1..=4usize,
    ) {
        let rules: Vec<String> = (0..n).map(|i| format!("myrule{}", i)).collect();
        let grammar = grammar_with_named_rules(&name, &rules);
        let generator = NodeTypesGenerator::new(&grammar);
        let result = generator.generate().unwrap();
        let entries = parse_node_types(&result);
        let names = type_names(&entries);
        for rn in &rules {
            prop_assert!(
                names.contains(rn),
                "Rule '{}' missing from NODE_TYPES: {:?}",
                rn,
                names
            );
        }
    }

    /// Rules starting with '_' (internal) do NOT appear as named entries.
    #[test]
    fn internal_rules_excluded(name in grammar_name_strategy()) {
        let mut grammar = Grammar::new(name);
        let tok = SymbolId(0);
        grammar.tokens.insert(tok, Token {
            name: "NUM".to_string(),
            pattern: TokenPattern::Regex(r"\d+".to_string()),
            fragile: false,
        });
        let sid = SymbolId(10);
        grammar.rule_names.insert(sid, "_internal".to_string());
        grammar.add_rule(Rule {
            lhs: sid,
            rhs: vec![Symbol::Terminal(tok)],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(0),
        });
        let generator = NodeTypesGenerator::new(&grammar);
        let result = generator.generate().unwrap();
        let entries = parse_node_types(&result);
        let named = named_entries(&entries);
        for e in &named {
            let tname = e.get("type").unwrap().as_str().unwrap();
            prop_assert!(
                !tname.starts_with('_'),
                "Internal rule '{}' should not appear as named",
                tname
            );
        }
    }

    // ---- 3. NODE_TYPES entries have `type` and `named` fields ----

    /// Every entry in NODE_TYPES has both `type` and `named` keys.
    #[test]
    fn entries_have_type_and_named(
        name in grammar_name_strategy(),
        n in 1..=4usize,
    ) {
        let grammar = grammar_with_rules_and_tokens(&name, n, n);
        let generator = NodeTypesGenerator::new(&grammar);
        let result = generator.generate().unwrap();
        let entries = parse_node_types(&result);
        for (i, entry) in entries.iter().enumerate() {
            prop_assert!(
                entry.get("type").is_some(),
                "Entry {} missing 'type' field: {:?}",
                i,
                entry
            );
            prop_assert!(
                entry.get("named").is_some(),
                "Entry {} missing 'named' field: {:?}",
                i,
                entry
            );
        }
    }

    /// Named entries have `named: true`, unnamed (string literal tokens) have `named: false`.
    #[test]
    fn named_flag_correctness(name in grammar_name_strategy()) {
        let grammar = grammar_with_rules_and_tokens(&name, 2, 2);
        let generator = NodeTypesGenerator::new(&grammar);
        let result = generator.generate().unwrap();
        let entries = parse_node_types(&result);
        for entry in &entries {
            let named = entry.get("named").unwrap().as_bool().unwrap();
            let type_name = entry.get("type").unwrap().as_str().unwrap();
            // String-literal tokens (unnamed) start with "op" in our helper
            if type_name.starts_with("op") && !type_name.starts_with("op_") {
                prop_assert!(!named, "'{}' should be unnamed", type_name);
            }
        }
    }

    // ---- 4. Children types listed correctly ----

    /// Fields registered in the grammar appear in the NODE_TYPES entry.
    #[test]
    fn fields_appear_in_node_types(name in grammar_name_strategy()) {
        let field_names = vec!["left".to_string(), "right".to_string()];
        let grammar = grammar_with_fields(&name, &field_names);
        let generator = NodeTypesGenerator::new(&grammar);
        let result = generator.generate().unwrap();
        let entries = parse_node_types(&result);
        let expr_entry = entries.iter().find(|e| {
            e.get("type").and_then(|t| t.as_str()) == Some("expr")
        });
        prop_assert!(expr_entry.is_some(), "expr entry not found");
        let fields_obj = expr_entry.unwrap().get("fields");
        prop_assert!(fields_obj.is_some(), "expr entry missing fields");
        let fields_map = fields_obj.unwrap().as_object().unwrap();
        for fname in &field_names {
            prop_assert!(
                fields_map.contains_key(fname),
                "Field '{}' missing from expr entry",
                fname
            );
        }
    }

    /// Field info contains `types`, `required`, and `multiple` keys.
    #[test]
    fn field_info_has_required_keys(name in grammar_name_strategy()) {
        let field_names = vec!["operand".to_string()];
        let grammar = grammar_with_fields(&name, &field_names);
        let generator = NodeTypesGenerator::new(&grammar);
        let result = generator.generate().unwrap();
        let entries = parse_node_types(&result);
        let expr_entry = entries.iter().find(|e| {
            e.get("type").and_then(|t| t.as_str()) == Some("expr")
        }).unwrap();
        let fields_map = expr_entry.get("fields").unwrap().as_object().unwrap();
        let field_info = fields_map.get("operand").unwrap();
        prop_assert!(field_info.get("types").is_some(), "Missing 'types'");
        prop_assert!(field_info.get("required").is_some(), "Missing 'required'");
        prop_assert!(field_info.get("multiple").is_some(), "Missing 'multiple'");
    }

    /// Field types entries each have `type` and `named` keys.
    #[test]
    fn field_type_entries_have_type_and_named(name in grammar_name_strategy()) {
        let field_names = vec!["value".to_string()];
        let grammar = grammar_with_fields(&name, &field_names);
        let generator = NodeTypesGenerator::new(&grammar);
        let result = generator.generate().unwrap();
        let entries = parse_node_types(&result);
        let expr_entry = entries.iter().find(|e| {
            e.get("type").and_then(|t| t.as_str()) == Some("expr")
        }).unwrap();
        let types_arr = expr_entry["fields"]["value"]["types"].as_array().unwrap();
        for tref in types_arr {
            prop_assert!(tref.get("type").is_some(), "Type ref missing 'type'");
            prop_assert!(tref.get("named").is_some(), "Type ref missing 'named'");
        }
    }

    // ---- 5. NODE_TYPES determinism ----

    /// Generating NODE_TYPES twice from the same grammar yields identical output.
    #[test]
    fn deterministic_output(name in grammar_name_strategy(), n in 1..=3usize) {
        let rules: Vec<String> = (0..n).map(|i| format!("drule{}", i)).collect();
        let grammar = grammar_with_named_rules(&name, &rules);
        let generator = NodeTypesGenerator::new(&grammar);
        let first = generator.generate().unwrap();
        let second = generator.generate().unwrap();
        prop_assert_eq!(&first, &second, "NODE_TYPES output is not deterministic");
    }

    /// Output is deterministic when grammar has both rules and tokens.
    #[test]
    fn deterministic_with_mixed_content(
        name in grammar_name_strategy(),
        nr in 1..=3usize,
        nt in 1..=3usize,
    ) {
        let grammar = grammar_with_rules_and_tokens(&name, nr, nt);
        let generator = NodeTypesGenerator::new(&grammar);
        let a = generator.generate().unwrap();
        let b = generator.generate().unwrap();
        prop_assert_eq!(&a, &b);
    }

    /// NODE_TYPES entries are sorted by type name.
    #[test]
    fn entries_sorted_by_type_name(
        name in grammar_name_strategy(),
        n in 1..=4usize,
    ) {
        let grammar = grammar_with_rules_and_tokens(&name, n, n);
        let generator = NodeTypesGenerator::new(&grammar);
        let result = generator.generate().unwrap();
        let entries = parse_node_types(&result);
        let names = type_names(&entries);
        let mut sorted = names.clone();
        sorted.sort();
        prop_assert_eq!(names, sorted, "NODE_TYPES entries are not sorted");
    }

    // ---- 6. Simple grammar NODE_TYPES ----

    /// Empty grammar produces an empty array.
    #[test]
    fn empty_grammar_produces_empty_array(name in grammar_name_strategy()) {
        let grammar = Grammar::new(name);
        let generator = NodeTypesGenerator::new(&grammar);
        let result = generator.generate().unwrap();
        let entries = parse_node_types(&result);
        prop_assert!(entries.is_empty(), "Expected empty array, got {:?}", entries);
    }

    /// Grammar with only regex tokens produces no unnamed entries (regex = named).
    #[test]
    fn regex_tokens_are_named(name in grammar_name_strategy()) {
        let mut grammar = Grammar::new(name);
        let id = SymbolId(1);
        grammar.tokens.insert(id, Token {
            name: "IDENT".to_string(),
            pattern: TokenPattern::Regex(r"[a-z]+".to_string()),
            fragile: false,
        });
        let generator = NodeTypesGenerator::new(&grammar);
        let result = generator.generate().unwrap();
        let entries = parse_node_types(&result);
        // Regex tokens are named so they don't appear as unnamed in tokens section
        // (the generator only adds string-literal tokens as unnamed)
        let unnamed: Vec<_> = entries.iter().filter(|e| {
            e.get("named").and_then(|n| n.as_bool()) == Some(false)
        }).collect();
        prop_assert!(unnamed.is_empty(), "Regex tokens should not produce unnamed entries");
    }

    /// Grammar with one rule and one token produces at least one named entry.
    #[test]
    fn single_rule_produces_named_entry(name in grammar_name_strategy()) {
        let rules = vec!["expr".to_string()];
        let grammar = grammar_with_named_rules(&name, &rules);
        let generator = NodeTypesGenerator::new(&grammar);
        let result = generator.generate().unwrap();
        let entries = parse_node_types(&result);
        let named = named_entries(&entries);
        prop_assert!(!named.is_empty(), "Expected at least one named entry");
    }

    /// A string-literal token produces an unnamed entry with `named: false`.
    #[test]
    fn string_token_is_unnamed(name in grammar_name_strategy()) {
        let grammar = grammar_with_string_tokens(&name, &["plus".to_string()]);
        let generator = NodeTypesGenerator::new(&grammar);
        let result = generator.generate().unwrap();
        let entries = parse_node_types(&result);
        let plus_entry = entries.iter().find(|e| {
            e.get("type").and_then(|t| t.as_str()) == Some("plus")
        });
        prop_assert!(plus_entry.is_some(), "String token 'plus' not found");
        prop_assert_eq!(
            plus_entry.unwrap().get("named").unwrap().as_bool().unwrap(),
            false,
            "String token should be unnamed"
        );
    }

    // ---- 7. Complex grammar NODE_TYPES ----

    /// Chain grammar produces an entry for each depth level.
    #[test]
    fn chain_grammar_entries(
        name in grammar_name_strategy(),
        depth in 1..=4usize,
    ) {
        let grammar = grammar_with_chain(&name, depth);
        let generator = NodeTypesGenerator::new(&grammar);
        let result = generator.generate().unwrap();
        let entries = parse_node_types(&result);
        let names = type_names(&entries);
        for i in 0..depth {
            let expected = format!("chain{}", i);
            prop_assert!(
                names.contains(&expected),
                "Chain rule '{}' missing, got {:?}",
                expected,
                names
            );
        }
    }

    /// A grammar with mixed rules and tokens has correct total entry count.
    #[test]
    fn mixed_grammar_entry_count(
        name in grammar_name_strategy(),
        nr in 1..=3usize,
        nt in 1..=3usize,
    ) {
        let grammar = grammar_with_rules_and_tokens(&name, nr, nt);
        let generator = NodeTypesGenerator::new(&grammar);
        let result = generator.generate().unwrap();
        let entries = parse_node_types(&result);
        let named_count = entries.iter().filter(|e| {
            e.get("named").and_then(|n| n.as_bool()) == Some(true)
        }).count();
        let unnamed_count = entries.iter().filter(|e| {
            e.get("named").and_then(|n| n.as_bool()) == Some(false)
        }).count();
        // nr named rules + nt unnamed string tokens
        prop_assert_eq!(named_count, nr, "Named entry count mismatch");
        prop_assert_eq!(unnamed_count, nt, "Unnamed entry count mismatch");
    }

    /// NODE_TYPES for grammar with multiple fields has all fields present.
    #[test]
    fn multiple_fields_all_present(name in grammar_name_strategy()) {
        let fnames = vec![
            "left".to_string(),
            "operator".to_string(),
            "right".to_string(),
        ];
        let grammar = grammar_with_fields(&name, &fnames);
        let generator = NodeTypesGenerator::new(&grammar);
        let result = generator.generate().unwrap();
        let entries = parse_node_types(&result);
        let expr = entries.iter().find(|e| {
            e.get("type").and_then(|t| t.as_str()) == Some("expr")
        }).unwrap();
        let fields_obj = expr.get("fields").unwrap().as_object().unwrap();
        prop_assert_eq!(fields_obj.len(), fnames.len());
    }

    /// Rules with no fields have no `fields` key (or it is null).
    #[test]
    fn rules_without_fields_have_no_fields_key(
        name in grammar_name_strategy(),
        n in 1..=3usize,
    ) {
        let rules: Vec<String> = (0..n).map(|i| format!("simple{}", i)).collect();
        let grammar = grammar_with_named_rules(&name, &rules);
        let generator = NodeTypesGenerator::new(&grammar);
        let result = generator.generate().unwrap();
        let entries = parse_node_types(&result);
        for entry in &entries {
            if entry.get("named").and_then(|n| n.as_bool()) == Some(true) {
                // For rules with no fields, `fields` should be absent
                prop_assert!(
                    entry.get("fields").is_none(),
                    "Rule without fields should not have 'fields' key: {:?}",
                    entry
                );
            }
        }
    }

    /// Unnamed tokens have no fields, children, or subtypes.
    #[test]
    fn unnamed_tokens_have_no_extra_keys(name in grammar_name_strategy()) {
        let lits = vec!["plus".to_string(), "minus".to_string()];
        let grammar = grammar_with_string_tokens(&name, &lits);
        let generator = NodeTypesGenerator::new(&grammar);
        let result = generator.generate().unwrap();
        let entries = parse_node_types(&result);
        for entry in &entries {
            if entry.get("named").and_then(|n| n.as_bool()) == Some(false) {
                prop_assert!(
                    entry.get("fields").is_none(),
                    "Unnamed token should not have 'fields': {:?}",
                    entry
                );
                prop_assert!(
                    entry.get("children").is_none(),
                    "Unnamed token should not have 'children': {:?}",
                    entry
                );
                prop_assert!(
                    entry.get("subtypes").is_none(),
                    "Unnamed token should not have 'subtypes': {:?}",
                    entry
                );
            }
        }
    }
}

// ===========================================================================
// Non-proptest (deterministic) tests for specific invariants
// ===========================================================================

/// NODE_TYPES output is a JSON array (not object, not scalar).
#[test]
fn node_types_is_json_array() {
    let grammar = Grammar::new("test".to_string());
    let generator = NodeTypesGenerator::new(&grammar);
    let result = generator.generate().unwrap();
    let v: Value = serde_json::from_str(&result).unwrap();
    assert!(v.is_array(), "NODE_TYPES must be a JSON array");
}

/// Type names in entries are non-empty strings.
#[test]
fn type_names_are_nonempty() {
    let rules = vec!["alpha".to_string(), "beta".to_string()];
    let grammar = grammar_with_named_rules("test", &rules);
    let generator = NodeTypesGenerator::new(&grammar);
    let result = generator.generate().unwrap();
    let entries = parse_node_types(&result);
    for entry in &entries {
        let t = entry.get("type").unwrap().as_str().unwrap();
        assert!(!t.is_empty(), "Type name must not be empty");
    }
}

/// `named` field is always a boolean.
#[test]
fn named_field_is_boolean() {
    let grammar = grammar_with_rules_and_tokens("test", 2, 2);
    let generator = NodeTypesGenerator::new(&grammar);
    let result = generator.generate().unwrap();
    let entries = parse_node_types(&result);
    for entry in &entries {
        assert!(
            entry.get("named").unwrap().is_boolean(),
            "named field must be boolean"
        );
    }
}

/// Duplicate rule names produce only one entry each (deduplication via symbol IDs).
#[test]
fn no_duplicate_type_entries() {
    let grammar = grammar_with_rules_and_tokens("dedup", 3, 2);
    let generator = NodeTypesGenerator::new(&grammar);
    let result = generator.generate().unwrap();
    let entries = parse_node_types(&result);
    let names = type_names(&entries);
    let mut seen = std::collections::HashSet::new();
    for n in &names {
        assert!(seen.insert(n.clone()), "Duplicate type name: {}", n);
    }
}

/// A grammar with only string tokens produces only unnamed entries.
#[test]
fn only_string_tokens_means_only_unnamed() {
    let grammar = grammar_with_string_tokens("toks", &["a".to_string(), "b".to_string()]);
    let generator = NodeTypesGenerator::new(&grammar);
    let result = generator.generate().unwrap();
    let entries = parse_node_types(&result);
    for entry in &entries {
        assert!(
            !entry.get("named").unwrap().as_bool().unwrap(),
            "String-only grammar should have only unnamed entries"
        );
    }
}

/// A grammar with optional symbol in RHS still generates valid NODE_TYPES.
#[test]
fn optional_symbol_in_rhs() {
    let mut grammar = Grammar::new("opt".to_string());
    let tok = SymbolId(0);
    grammar.tokens.insert(
        tok,
        Token {
            name: "NUM".to_string(),
            pattern: TokenPattern::Regex(r"\d+".to_string()),
            fragile: false,
        },
    );
    let rule_id = SymbolId(10);
    grammar.rule_names.insert(rule_id, "maybe".to_string());
    grammar.add_rule(Rule {
        lhs: rule_id,
        rhs: vec![Symbol::Optional(Box::new(Symbol::Terminal(tok)))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    let generator = NodeTypesGenerator::new(&grammar);
    let result = generator.generate().unwrap();
    let entries = parse_node_types(&result);
    assert!(
        entries
            .iter()
            .any(|e| { e.get("type").and_then(|t| t.as_str()) == Some("maybe") })
    );
}

/// A grammar with repeat symbol in RHS generates valid NODE_TYPES.
#[test]
fn repeat_symbol_in_rhs() {
    let mut grammar = Grammar::new("rep".to_string());
    let tok = SymbolId(0);
    grammar.tokens.insert(
        tok,
        Token {
            name: "WORD".to_string(),
            pattern: TokenPattern::Regex(r"[a-z]+".to_string()),
            fragile: false,
        },
    );
    let rule_id = SymbolId(10);
    grammar.rule_names.insert(rule_id, "many".to_string());
    grammar.add_rule(Rule {
        lhs: rule_id,
        rhs: vec![Symbol::Repeat(Box::new(Symbol::Terminal(tok)))],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    let generator = NodeTypesGenerator::new(&grammar);
    let result = generator.generate().unwrap();
    assert!(serde_json::from_str::<Value>(&result).is_ok());
    let entries = parse_node_types(&result);
    assert!(
        entries
            .iter()
            .any(|e| { e.get("type").and_then(|t| t.as_str()) == Some("many") })
    );
}

/// A grammar with choice symbol generates valid output.
#[test]
fn choice_symbol_in_rhs() {
    let mut grammar = Grammar::new("choice".to_string());
    let tok_a = SymbolId(0);
    let tok_b = SymbolId(1);
    grammar.tokens.insert(
        tok_a,
        Token {
            name: "A".to_string(),
            pattern: TokenPattern::Regex(r"[a-z]+".to_string()),
            fragile: false,
        },
    );
    grammar.tokens.insert(
        tok_b,
        Token {
            name: "B".to_string(),
            pattern: TokenPattern::Regex(r"\d+".to_string()),
            fragile: false,
        },
    );
    let rule_id = SymbolId(10);
    grammar.rule_names.insert(rule_id, "either".to_string());
    grammar.add_rule(Rule {
        lhs: rule_id,
        rhs: vec![Symbol::Choice(vec![
            Symbol::Terminal(tok_a),
            Symbol::Terminal(tok_b),
        ])],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    let generator = NodeTypesGenerator::new(&grammar);
    let result = generator.generate().unwrap();
    assert!(serde_json::from_str::<Value>(&result).is_ok());
}

/// A grammar with sequence symbol generates valid output.
#[test]
fn sequence_symbol_in_rhs() {
    let mut grammar = Grammar::new("seq".to_string());
    let tok = SymbolId(0);
    grammar.tokens.insert(
        tok,
        Token {
            name: "X".to_string(),
            pattern: TokenPattern::Regex(r"\w+".to_string()),
            fragile: false,
        },
    );
    let rule_id = SymbolId(10);
    grammar.rule_names.insert(rule_id, "pair".to_string());
    grammar.add_rule(Rule {
        lhs: rule_id,
        rhs: vec![Symbol::Sequence(vec![
            Symbol::Terminal(tok),
            Symbol::Terminal(tok),
        ])],
        precedence: None,
        associativity: None,
        fields: vec![],
        production_id: ProductionId(0),
    });
    let generator = NodeTypesGenerator::new(&grammar);
    let result = generator.generate().unwrap();
    assert!(serde_json::from_str::<Value>(&result).is_ok());
    let entries = parse_node_types(&result);
    assert!(
        entries
            .iter()
            .any(|e| { e.get("type").and_then(|t| t.as_str()) == Some("pair") })
    );
}
