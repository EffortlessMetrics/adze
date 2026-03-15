#![allow(clippy::needless_range_loop)]

//! Property-based tests for field name generation in adze-tool.
//!
//! Uses proptest to validate invariants of field names produced during grammar
//! generation:
//!   - Field names appear as FIELD nodes in grammar JSON
//!   - Field names are ordered deterministically
//!   - Field names appear in NODE_TYPES
//!   - No duplicate field names within a rule
//!   - Field name derived from struct field name
//!   - Field name generation is deterministic
//!   - Grammars with no fields produce no FIELD nodes

use adze_ir::{FieldId, Grammar, ProductionId, Rule, Symbol, SymbolId, Token, TokenPattern};
use adze_tablegen::NodeTypesGenerator;
use proptest::prelude::*;
use serde_json::Value;
use std::collections::HashSet;
use std::fs;
use tempfile::TempDir;

// ===========================================================================
// Helpers
// ===========================================================================

/// Write Rust source to a temp file and extract grammars via the public API.
fn extract(src: &str) -> Vec<Value> {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("lib.rs");
    fs::write(&path, src).unwrap();
    adze_tool::generate_grammars(&path).unwrap()
}

/// Extract exactly one grammar.
fn extract_one(src: &str) -> Value {
    let gs = extract(src);
    assert_eq!(
        gs.len(),
        1,
        "expected exactly one grammar, got {}",
        gs.len()
    );
    gs.into_iter().next().unwrap()
}

/// Recursively collect all FIELD node names from a JSON grammar rule tree.
fn collect_field_names(value: &Value) -> Vec<String> {
    let mut names = Vec::new();
    collect_field_names_inner(value, &mut names);
    names
}

fn collect_field_names_inner(value: &Value, names: &mut Vec<String>) {
    match value {
        Value::Object(obj) => {
            if obj.get("type").and_then(|t| t.as_str()) == Some("FIELD")
                && let Some(name) = obj.get("name").and_then(|n| n.as_str())
            {
                names.push(name.to_string());
            }
            for v in obj.values() {
                collect_field_names_inner(v, names);
            }
        }
        Value::Array(arr) => {
            for v in arr {
                collect_field_names_inner(v, names);
            }
        }
        _ => {}
    }
}

/// Collect all FIELD node names from all rules in a grammar JSON.
fn all_field_names_in_grammar(grammar: &Value) -> Vec<String> {
    let mut names = Vec::new();
    if let Some(rules) = grammar.get("rules").and_then(|r| r.as_object()) {
        for (_rule_name, rule_body) in rules {
            collect_field_names_inner(rule_body, &mut names);
        }
    }
    names
}

/// Collect FIELD names from a specific rule.
fn field_names_in_rule(grammar: &Value, rule_name: &str) -> Vec<String> {
    if let Some(rule) = grammar
        .get("rules")
        .and_then(|r| r.as_object())
        .and_then(|rules| rules.get(rule_name))
    {
        collect_field_names(rule)
    } else {
        vec![]
    }
}

/// Build a minimal struct grammar source.
fn struct_grammar_source(name: &str, type_name: &str, field: &str, pattern: &str) -> String {
    format!(
        r##"
        #[adze::grammar("{name}")]
        mod grammar {{
            #[adze::language]
            pub struct {type_name} {{
                #[adze::leaf(pattern = r"{pattern}")]
                pub {field}: String,
            }}
        }}
        "##,
    )
}

/// Build a grammar with two named struct fields.
fn two_field_grammar_source(
    name: &str,
    type_name: &str,
    f1: &str,
    f2: &str,
    p1: &str,
    p2: &str,
) -> String {
    format!(
        r##"
        #[adze::grammar("{name}")]
        mod grammar {{
            #[adze::language]
            pub struct {type_name} {{
                #[adze::leaf(pattern = r"{p1}")]
                pub {f1}: String,
                #[adze::leaf(pattern = r"{p2}")]
                pub {f2}: String,
            }}
        }}
        "##,
    )
}

/// Build an enum grammar source with one variant.
fn _enum_grammar_source(name: &str, type_name: &str, pattern: &str) -> String {
    format!(
        r##"
        #[adze::grammar("{name}")]
        mod grammar {{
            #[adze::language]
            pub enum {type_name} {{
                Leaf(
                    #[adze::leaf(pattern = r"{pattern}")]
                    String
                ),
            }}
        }}
        "##,
    )
}

/// Build a grammar with three named struct fields.
fn three_field_grammar_source(name: &str, type_name: &str) -> String {
    format!(
        r##"
        #[adze::grammar("{name}")]
        mod grammar {{
            #[adze::language]
            pub struct {type_name} {{
                #[adze::leaf(pattern = r"[a-z]+")]
                pub alpha: String,
                #[adze::leaf(pattern = r"\d+")]
                pub beta: String,
                #[adze::leaf(pattern = r"[A-Z]+")]
                pub gamma: String,
            }}
        }}
        "##,
    )
}

/// Build an IR grammar with N fields.
fn ir_grammar_with_fields(name: &str, field_names: &[String]) -> Grammar {
    let mut grammar = Grammar::new(name.to_string());

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

    let mut field_ids = Vec::new();
    for (i, fname) in field_names.iter().enumerate() {
        let fid = FieldId(i as u16);
        grammar.fields.insert(fid, fname.clone());
        field_ids.push(fid);
    }

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

/// Build an IR grammar with no fields.
fn ir_grammar_no_fields(name: &str, n_rules: usize) -> Grammar {
    let mut grammar = Grammar::new(name.to_string());

    let tok_id = SymbolId(0);
    grammar.tokens.insert(
        tok_id,
        Token {
            name: "NUM".to_string(),
            pattern: TokenPattern::Regex(r"\d+".to_string()),
            fragile: false,
        },
    );

    for i in 0..n_rules {
        let sid = SymbolId((i as u16) + 10);
        grammar.rule_names.insert(sid, format!("rule{}", i));
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

/// Parse NODE_TYPES JSON string into a Vec of entries.
fn parse_node_types(json: &str) -> Vec<Value> {
    let v: Value = serde_json::from_str(json).expect("NODE_TYPES must be valid JSON");
    v.as_array()
        .expect("NODE_TYPES must be a JSON array")
        .clone()
}

// ===========================================================================
// Strategies
// ===========================================================================

/// A valid grammar name.
fn grammar_name_strategy() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9_]{0,12}".prop_filter("must not be empty", |s| !s.is_empty())
}

/// A valid Rust type name (PascalCase).
fn type_name_strategy() -> impl Strategy<Value = String> {
    "[A-Z][a-z]{1,8}".prop_filter("must not be empty", |s| !s.is_empty())
}

/// A valid Rust field name (snake_case, not a keyword).
fn field_name_strategy() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9_]{0,8}".prop_filter("avoid keywords", |s| {
        !matches!(
            s.as_str(),
            "type"
                | "fn"
                | "let"
                | "mut"
                | "ref"
                | "pub"
                | "mod"
                | "use"
                | "self"
                | "super"
                | "crate"
                | "struct"
                | "enum"
                | "impl"
                | "trait"
                | "where"
                | "for"
                | "loop"
                | "while"
                | "if"
                | "else"
                | "match"
                | "return"
                | "break"
                | "continue"
                | "as"
                | "in"
                | "move"
                | "box"
                | "dyn"
                | "async"
                | "await"
                | "try"
                | "yield"
                | "macro"
                | "const"
                | "static"
                | "unsafe"
                | "extern"
                | "do"
                | "abstract"
                | "become"
                | "final"
                | "override"
                | "priv"
                | "typeof"
                | "unsized"
                | "virtual"
        )
    })
}

/// A safe regex pattern for embedding in source.
fn safe_pattern_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        Just(r"[a-z]+".to_string()),
        Just(r"\d+".to_string()),
        Just(r"[a-zA-Z_][a-zA-Z0-9_]*".to_string()),
        Just(r"[0-9]+".to_string()),
        Just(r"[a-f0-9]+".to_string()),
    ]
}

/// A field name for IR-level grammars (simple lowercase identifiers).
fn ir_field_name_strategy() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9]{0,8}".prop_filter("must not be empty", |s| !s.is_empty())
}

// ===========================================================================
// 1. Field names in grammar JSON
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    /// A struct with a named field produces a FIELD node in the grammar JSON.
    #[test]
    fn struct_field_produces_field_node(
        name in grammar_name_strategy(),
        type_name in type_name_strategy(),
        field in field_name_strategy(),
        pattern in safe_pattern_strategy(),
    ) {
        let src = struct_grammar_source(&name, &type_name, &field, &pattern);
        let grammar = extract_one(&src);
        let all_fields = all_field_names_in_grammar(&grammar);
        prop_assert!(
            all_fields.contains(&field),
            "Field '{}' not found in grammar JSON. Fields found: {:?}",
            field,
            all_fields
        );
    }

    /// Two distinct struct fields produce two FIELD nodes.
    #[test]
    fn two_fields_both_present(
        name in grammar_name_strategy(),
        type_name in type_name_strategy(),
        f1 in field_name_strategy(),
        f2 in field_name_strategy(),
        p1 in safe_pattern_strategy(),
        p2 in safe_pattern_strategy(),
    ) {
        let f2_safe = if f2 == f1 { format!("{}_b", f2) } else { f2 };
        let src = two_field_grammar_source(&name, &type_name, &f1, &f2_safe, &p1, &p2);
        let grammar = extract_one(&src);
        let all_fields = all_field_names_in_grammar(&grammar);
        prop_assert!(all_fields.contains(&f1), "Missing field '{}'", f1);
        prop_assert!(all_fields.contains(&f2_safe), "Missing field '{}'", f2_safe);
    }

    /// FIELD nodes always have a "name" and "content" property.
    #[test]
    fn field_nodes_have_name_and_content(
        name in grammar_name_strategy(),
        type_name in type_name_strategy(),
        field in field_name_strategy(),
        pattern in safe_pattern_strategy(),
    ) {
        let src = struct_grammar_source(&name, &type_name, &field, &pattern);
        let grammar = extract_one(&src);
        check_field_nodes_structure(&grammar);
    }

    /// An enum variant with a named field produces a FIELD node.
    #[test]
    fn enum_named_field_produces_field_node(
        name in grammar_name_strategy(),
        type_name in type_name_strategy(),
        field in field_name_strategy(),
    ) {
        let src = format!(
            r##"
            #[adze::grammar("{name}")]
            mod grammar {{
                #[adze::language]
                pub enum {type_name} {{
                    Variant {{
                        #[adze::leaf(pattern = r"[a-z]+")]
                        {field}: String,
                    }}
                }}
            }}
            "##,
        );
        let grammar = extract_one(&src);
        let all_fields = all_field_names_in_grammar(&grammar);
        prop_assert!(
            all_fields.contains(&field),
            "Enum field '{}' not found. Fields: {:?}",
            field,
            all_fields
        );
    }
}

/// Verify all FIELD nodes have required structure.
fn check_field_nodes_structure(value: &Value) {
    match value {
        Value::Object(obj) => {
            if obj.get("type").and_then(|t| t.as_str()) == Some("FIELD") {
                assert!(
                    obj.get("name").is_some(),
                    "FIELD node missing 'name': {:?}",
                    obj
                );
                assert!(
                    obj.get("content").is_some(),
                    "FIELD node missing 'content': {:?}",
                    obj
                );
            }
            for v in obj.values() {
                check_field_nodes_structure(v);
            }
        }
        Value::Array(arr) => {
            for v in arr {
                check_field_nodes_structure(v);
            }
        }
        _ => {}
    }
}

// ===========================================================================
// 2. Field names ordering
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    /// Fields in a multi-field struct appear in declaration order within the rule.
    #[test]
    fn field_order_matches_declaration_order(
        name in grammar_name_strategy(),
        type_name in type_name_strategy(),
    ) {
        let src = three_field_grammar_source(&name, &type_name);
        let grammar = extract_one(&src);
        let fields = field_names_in_rule(&grammar, &type_name);
        // alpha, beta, gamma should appear in that order
        let alpha_pos = fields.iter().position(|f| f == "alpha");
        let beta_pos = fields.iter().position(|f| f == "beta");
        let gamma_pos = fields.iter().position(|f| f == "gamma");
        prop_assert!(alpha_pos.is_some(), "alpha not found");
        prop_assert!(beta_pos.is_some(), "beta not found");
        prop_assert!(gamma_pos.is_some(), "gamma not found");
        prop_assert!(alpha_pos < beta_pos, "alpha should come before beta");
        prop_assert!(beta_pos < gamma_pos, "beta should come before gamma");
    }

    /// Two-field struct: first declared field appears first in FIELD nodes.
    #[test]
    fn two_field_ordering(
        name in grammar_name_strategy(),
        type_name in type_name_strategy(),
        f1 in field_name_strategy(),
        f2 in field_name_strategy(),
    ) {
        let f2_safe = if f2 == f1 { format!("{}_b", f2) } else { f2 };
        let src = two_field_grammar_source(
            &name, &type_name, &f1, &f2_safe, r"[a-z]+", r"\d+",
        );
        let grammar = extract_one(&src);
        let fields = field_names_in_rule(&grammar, &type_name);
        let pos1 = fields.iter().position(|f| f == &f1);
        let pos2 = fields.iter().position(|f| f == &f2_safe);
        prop_assert!(pos1.is_some(), "first field not found");
        prop_assert!(pos2.is_some(), "second field not found");
        prop_assert!(pos1 < pos2, "first field should precede second");
    }

    /// Fields in IR grammar are ordered by FieldId insertion order.
    #[test]
    fn ir_field_ordering_matches_insertion(
        name in grammar_name_strategy(),
    ) {
        let fnames = vec!["zulu".to_string(), "alpha".to_string(), "mike".to_string()];
        let grammar = ir_grammar_with_fields(&name, &fnames);
        let field_list: Vec<&String> = grammar.fields.values().collect();
        prop_assert_eq!(field_list[0], "zulu");
        prop_assert_eq!(field_list[1], "alpha");
        prop_assert_eq!(field_list[2], "mike");
    }
}

// ===========================================================================
// 3. Field names in NODE_TYPES
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    /// Fields registered in IR grammar appear in NODE_TYPES output.
    #[test]
    fn ir_fields_in_node_types(
        name in grammar_name_strategy(),
        f1 in ir_field_name_strategy(),
        f2 in ir_field_name_strategy(),
    ) {
        let f2_safe = if f2 == f1 { format!("{}b", f2) } else { f2 };
        let fnames = vec![f1.clone(), f2_safe.clone()];
        let grammar = ir_grammar_with_fields(&name, &fnames);
        let generator = NodeTypesGenerator::new(&grammar);
        let result = generator.generate().unwrap();
        let entries = parse_node_types(&result);
        let expr_entry = entries
            .iter()
            .find(|e| e.get("type").and_then(|t| t.as_str()) == Some("expr"));
        prop_assert!(expr_entry.is_some(), "expr entry not found in NODE_TYPES");
        let fields_obj = expr_entry.unwrap().get("fields");
        prop_assert!(fields_obj.is_some(), "expr missing 'fields' key");
        let fields_map = fields_obj.unwrap().as_object().unwrap();
        prop_assert!(fields_map.contains_key(&f1), "Field '{}' missing", f1);
        prop_assert!(fields_map.contains_key(&f2_safe), "Field '{}' missing", f2_safe);
    }

    /// Each field entry in NODE_TYPES has `types`, `required`, and `multiple`.
    #[test]
    fn node_types_field_entries_have_schema(
        name in grammar_name_strategy(),
        field in ir_field_name_strategy(),
    ) {
        let grammar = ir_grammar_with_fields(&name, std::slice::from_ref(&field));
        let generator = NodeTypesGenerator::new(&grammar);
        let result = generator.generate().unwrap();
        let entries = parse_node_types(&result);
        let expr = entries
            .iter()
            .find(|e| e.get("type").and_then(|t| t.as_str()) == Some("expr"))
            .unwrap();
        let info = expr["fields"][&field].as_object().unwrap();
        prop_assert!(info.contains_key("types"), "Missing 'types'");
        prop_assert!(info.contains_key("required"), "Missing 'required'");
        prop_assert!(info.contains_key("multiple"), "Missing 'multiple'");
    }

    /// Field types entries in NODE_TYPES each have `type` and `named`.
    #[test]
    fn node_types_field_type_refs(
        name in grammar_name_strategy(),
        field in ir_field_name_strategy(),
    ) {
        let grammar = ir_grammar_with_fields(&name, std::slice::from_ref(&field));
        let generator = NodeTypesGenerator::new(&grammar);
        let result = generator.generate().unwrap();
        let entries = parse_node_types(&result);
        let expr = entries
            .iter()
            .find(|e| e.get("type").and_then(|t| t.as_str()) == Some("expr"))
            .unwrap();
        let types_arr = expr["fields"][&field]["types"].as_array().unwrap();
        for tref in types_arr {
            prop_assert!(tref.get("type").is_some(), "Type ref missing 'type'");
            prop_assert!(tref.get("named").is_some(), "Type ref missing 'named'");
        }
    }
}

// ===========================================================================
// 4. No duplicate field names
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    /// A single struct rule never produces duplicate FIELD names.
    #[test]
    fn no_duplicate_field_names_struct(
        name in grammar_name_strategy(),
        type_name in type_name_strategy(),
        f1 in field_name_strategy(),
        f2 in field_name_strategy(),
    ) {
        let f2_safe = if f2 == f1 { format!("{}_b", f2) } else { f2 };
        let src = two_field_grammar_source(
            &name, &type_name, &f1, &f2_safe, r"[a-z]+", r"\d+",
        );
        let grammar = extract_one(&src);
        let fields = field_names_in_rule(&grammar, &type_name);
        let unique: HashSet<&String> = fields.iter().collect();
        prop_assert_eq!(
            unique.len(),
            fields.len(),
            "Duplicate field names detected: {:?}",
            fields
        );
    }

    /// In the IR grammar, distinct FieldIds map to distinct names.
    #[test]
    fn ir_grammar_no_duplicate_fields(
        name in grammar_name_strategy(),
        f1 in ir_field_name_strategy(),
        f2 in ir_field_name_strategy(),
        f3 in ir_field_name_strategy(),
    ) {
        // Make all names unique
        let mut fnames = vec![f1];
        if !fnames.contains(&f2) { fnames.push(f2); } else { fnames.push(format!("{}x", fnames.last().unwrap())); }
        if !fnames.contains(&f3) { fnames.push(f3); } else { fnames.push(format!("{}y", fnames.last().unwrap())); }

        let grammar = ir_grammar_with_fields(&name, &fnames);
        let seen: HashSet<&String> = grammar.fields.values().collect();
        prop_assert_eq!(
            seen.len(),
            grammar.fields.len(),
            "IR grammar has duplicate field names"
        );
    }

    /// NODE_TYPES fields map has no duplicate keys (JSON guarantees this, but verify).
    #[test]
    fn node_types_no_duplicate_field_keys(
        name in grammar_name_strategy(),
    ) {
        let fnames = vec!["left".to_string(), "right".to_string(), "value".to_string()];
        let grammar = ir_grammar_with_fields(&name, &fnames);
        let generator = NodeTypesGenerator::new(&grammar);
        let result = generator.generate().unwrap();
        let entries = parse_node_types(&result);
        let expr = entries
            .iter()
            .find(|e| e.get("type").and_then(|t| t.as_str()) == Some("expr"))
            .unwrap();
        let fields_obj = expr["fields"].as_object().unwrap();
        prop_assert_eq!(fields_obj.len(), 3, "Expected 3 unique field keys");
    }
}

// ===========================================================================
// 5. Field name from struct field name
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    /// The field name in grammar JSON matches the Rust struct field identifier.
    #[test]
    fn field_name_matches_rust_ident(
        name in grammar_name_strategy(),
        type_name in type_name_strategy(),
        field in field_name_strategy(),
        pattern in safe_pattern_strategy(),
    ) {
        let src = struct_grammar_source(&name, &type_name, &field, &pattern);
        let grammar = extract_one(&src);
        let fields = field_names_in_rule(&grammar, &type_name);
        prop_assert!(
            fields.contains(&field),
            "Rule '{}' should have field '{}', found: {:?}",
            type_name,
            field,
            fields
        );
    }

    /// A struct field with #[adze::field("custom")] uses the custom name.
    #[test]
    fn custom_field_name_via_attribute(
        name in grammar_name_strategy(),
        type_name in type_name_strategy(),
    ) {
        let src = format!(
            r##"
            #[adze::grammar("{name}")]
            mod grammar {{
                #[adze::language]
                pub struct {type_name} {{
                    #[adze::field("custom_name")]
                    #[adze::leaf(pattern = r"[a-z]+")]
                    pub original: String,
                }}
            }}
            "##,
        );
        let grammar = extract_one(&src);
        let fields = all_field_names_in_grammar(&grammar);
        prop_assert!(
            fields.contains(&"custom_name".to_string()),
            "Custom field name 'custom_name' not found. Fields: {:?}",
            fields
        );
    }

    /// Unnamed enum tuple fields get a deterministic generated name (in variant rule).
    #[test]
    fn unnamed_field_gets_generated_name(
        name in grammar_name_strategy(),
        type_name in type_name_strategy(),
    ) {
        // Enum variants with unnamed fields that are NOT inlined produce FIELD nodes
        // in a separate variant rule. Inlined single-leaf variants may not.
        // For multi-field unnamed variants, a variant rule with generated names exists.
        let src = format!(
            r##"
            #[adze::grammar("{name}")]
            mod grammar {{
                #[adze::language]
                pub enum {type_name} {{
                    Pair(
                        #[adze::leaf(pattern = r"[a-z]+")]
                        String,
                        #[adze::leaf(pattern = r"\d+")]
                        String,
                    ),
                }}
            }}
            "##,
        );
        let grammar = extract_one(&src);
        let all_fields = all_field_names_in_grammar(&grammar);
        // Multi-field unnamed tuple variant should produce generated field names
        prop_assert!(
            !all_fields.is_empty(),
            "Expected generated field names for unnamed tuple fields"
        );
    }
}

// ===========================================================================
// 6. Field names determinism
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    /// Generating the same grammar twice yields identical field names.
    #[test]
    fn deterministic_field_names_struct(
        name in grammar_name_strategy(),
        type_name in type_name_strategy(),
        field in field_name_strategy(),
        pattern in safe_pattern_strategy(),
    ) {
        let src = struct_grammar_source(&name, &type_name, &field, &pattern);
        let g1 = extract_one(&src);
        let g2 = extract_one(&src);
        let f1 = all_field_names_in_grammar(&g1);
        let f2 = all_field_names_in_grammar(&g2);
        prop_assert_eq!(&f1, &f2, "Field names not deterministic");
    }

    /// Determinism for multi-field struct.
    #[test]
    fn deterministic_field_names_multi_field(
        name in grammar_name_strategy(),
        type_name in type_name_strategy(),
    ) {
        let src = three_field_grammar_source(&name, &type_name);
        let g1 = extract_one(&src);
        let g2 = extract_one(&src);
        let f1 = all_field_names_in_grammar(&g1);
        let f2 = all_field_names_in_grammar(&g2);
        prop_assert_eq!(&f1, &f2);
    }

    /// Determinism for enum with named variant fields.
    #[test]
    fn deterministic_field_names_enum(
        name in grammar_name_strategy(),
        type_name in type_name_strategy(),
        field in field_name_strategy(),
    ) {
        let src = format!(
            r##"
            #[adze::grammar("{name}")]
            mod grammar {{
                #[adze::language]
                pub enum {type_name} {{
                    Variant {{
                        #[adze::leaf(pattern = r"[a-z]+")]
                        {field}: String,
                    }}
                }}
            }}
            "##,
        );
        let g1 = extract_one(&src);
        let g2 = extract_one(&src);
        let f1 = all_field_names_in_grammar(&g1);
        let f2 = all_field_names_in_grammar(&g2);
        prop_assert_eq!(&f1, &f2, "Enum field names not deterministic");
    }

    /// IR-level NODE_TYPES generation is deterministic for field names (compare parsed).
    #[test]
    fn deterministic_node_types_fields(
        name in grammar_name_strategy(),
    ) {
        let fnames = vec!["alpha".to_string(), "bravo".to_string()];
        let grammar = ir_grammar_with_fields(&name, &fnames);
        let generator = NodeTypesGenerator::new(&grammar);
        let r1 = generator.generate().unwrap();
        let r2 = generator.generate().unwrap();
        let v1: Value = serde_json::from_str(&r1).unwrap();
        let v2: Value = serde_json::from_str(&r2).unwrap();
        prop_assert_eq!(&v1, &v2, "NODE_TYPES not deterministic (as parsed JSON)");
    }

    /// Serialized grammar JSON byte-identical across runs.
    #[test]
    fn deterministic_serialized_fields(
        name in grammar_name_strategy(),
        type_name in type_name_strategy(),
        field in field_name_strategy(),
    ) {
        let src = struct_grammar_source(&name, &type_name, &field, r"[a-z]+");
        let s1 = serde_json::to_string(&extract_one(&src)).unwrap();
        let s2 = serde_json::to_string(&extract_one(&src)).unwrap();
        prop_assert_eq!(&s1, &s2, "Serialized form not byte-identical");
    }
}

// ===========================================================================
// 7. Grammar with no fields
// ===========================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    /// An IR grammar with no fields produces no field entries in NODE_TYPES.
    #[test]
    fn no_fields_grammar_has_no_node_types_fields(
        name in grammar_name_strategy(),
        n in 1..=3usize,
    ) {
        let grammar = ir_grammar_no_fields(&name, n);
        let generator = NodeTypesGenerator::new(&grammar);
        let result = generator.generate().unwrap();
        let entries = parse_node_types(&result);
        for entry in &entries {
            prop_assert!(
                entry.get("fields").is_none(),
                "Rule without fields should not have 'fields': {:?}",
                entry
            );
        }
    }

    /// An empty IR grammar field map produces an empty field count.
    #[test]
    fn empty_field_map(name in grammar_name_strategy()) {
        let grammar = ir_grammar_no_fields(&name, 1);
        prop_assert!(grammar.fields.is_empty(), "Expected no fields");
    }

    /// Adding then removing a field yields an empty field map.
    #[test]
    fn field_removal_produces_empty(name in grammar_name_strategy()) {
        let mut grammar = Grammar::new(name);
        grammar.fields.insert(FieldId(0), "temp".to_string());
        grammar.fields.shift_remove(&FieldId(0));
        prop_assert!(grammar.fields.is_empty());
    }
}

// ===========================================================================
// Non-proptest deterministic tests for edge cases
// ===========================================================================

/// Three fixed fields appear in the correct order.
#[test]
fn fixed_three_field_order() {
    let src = three_field_grammar_source("test", "Root");
    let grammar = extract_one(&src);
    let fields = field_names_in_rule(&grammar, "Root");
    let alpha_pos = fields.iter().position(|f| f == "alpha").unwrap();
    let beta_pos = fields.iter().position(|f| f == "beta").unwrap();
    let gamma_pos = fields.iter().position(|f| f == "gamma").unwrap();
    assert!(alpha_pos < beta_pos);
    assert!(beta_pos < gamma_pos);
}

/// A struct with a single field has exactly one FIELD node.
#[test]
fn single_field_count() {
    let src = struct_grammar_source("test", "Root", "value", r"[a-z]+");
    let grammar = extract_one(&src);
    let fields = field_names_in_rule(&grammar, "Root");
    assert_eq!(
        fields.len(),
        1,
        "Expected exactly 1 field, got {:?}",
        fields
    );
    assert_eq!(fields[0], "value");
}

/// Two fields both present in grammar JSON.
#[test]
fn fixed_two_fields_present() {
    let src = two_field_grammar_source("test", "Pair", "left", "right", r"[a-z]+", r"\d+");
    let grammar = extract_one(&src);
    let fields = field_names_in_rule(&grammar, "Pair");
    assert!(fields.contains(&"left".to_string()));
    assert!(fields.contains(&"right".to_string()));
}

/// An enum grammar with unit variants produces valid JSON.
#[test]
fn enum_unit_variant_no_crash() {
    let src = r##"
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub enum Token {
                Alpha(
                    #[adze::leaf(pattern = r"[a-z]+")]
                    String
                ),
            }
        }
    "##;
    let grammar = extract_one(src);
    let _ = serde_json::to_string_pretty(&grammar).unwrap();
}

/// Custom field name attribute overrides the Rust field name.
#[test]
fn custom_field_name_present() {
    let src = r##"
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::field("my_field")]
                #[adze::leaf(pattern = r"[a-z]+")]
                pub original_name: String,
            }
        }
    "##;
    let grammar = extract_one(src);
    let fields = all_field_names_in_grammar(&grammar);
    assert!(
        fields.contains(&"my_field".to_string()),
        "Expected 'my_field', got {:?}",
        fields
    );
    // The original Rust name should NOT appear
    assert!(
        !fields.contains(&"original_name".to_string()),
        "Original name should be overridden"
    );
}

/// IR grammar with zero field names: fields map is empty.
#[test]
fn ir_empty_fields() {
    let grammar = ir_grammar_no_fields("test", 2);
    assert!(grammar.fields.is_empty());
}
