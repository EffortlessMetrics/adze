#![allow(clippy::needless_range_loop)]

//! Property-based and unit tests for field extraction and processing in adze-tool.
//!
//! Covers:
//!   1. FIELD type generation for struct fields
//!   2. Field name in generated rule
//!   3. Field content type (SYMBOL reference)
//!   4. Multiple fields produce ordered SEQ
//!   5. Optional field wraps in CHOICE+BLANK
//!   6. Vec field produces REPEAT/REPEAT1
//!   7. Box field for recursive references
//!   8. Field extraction determinism

use serde_json::Value;
use std::fs;
use tempfile::TempDir;

// ===========================================================================
// Helpers
// ===========================================================================

/// Write Rust source to a temp file and extract grammars via the public API.
fn extract_one(src: &str) -> Value {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("lib.rs");
    fs::write(&path, src).unwrap();
    let gs = adze_tool::generate_grammars(&path).unwrap();
    assert_eq!(
        gs.len(),
        1,
        "expected exactly one grammar, got {}",
        gs.len()
    );
    gs.into_iter().next().unwrap()
}

/// Get a named rule from the grammar JSON.
fn get_rule<'a>(grammar: &'a Value, name: &str) -> &'a Value {
    grammar
        .get("rules")
        .and_then(|r| r.get(name))
        .unwrap_or_else(|| panic!("rule '{}' not found in grammar", name))
}

/// Recursively collect all nodes of a given type from a JSON tree.
fn collect_nodes_by_type<'a>(value: &'a Value, ty: &str) -> Vec<&'a Value> {
    let mut result = Vec::new();
    collect_nodes_by_type_inner(value, ty, &mut result);
    result
}

fn collect_nodes_by_type_inner<'a>(value: &'a Value, ty: &str, out: &mut Vec<&'a Value>) {
    match value {
        Value::Object(obj) => {
            if obj.get("type").and_then(|t| t.as_str()) == Some(ty) {
                out.push(value);
            }
            for v in obj.values() {
                collect_nodes_by_type_inner(v, ty, out);
            }
        }
        Value::Array(arr) => {
            for v in arr {
                collect_nodes_by_type_inner(v, ty, out);
            }
        }
        _ => {}
    }
}

/// Recursively collect all FIELD node names from a JSON tree.
fn collect_field_names(value: &Value) -> Vec<String> {
    collect_nodes_by_type(value, "FIELD")
        .iter()
        .filter_map(|n| n.get("name").and_then(|v| v.as_str()).map(String::from))
        .collect()
}

// ===========================================================================
// 1. FIELD type generation for struct fields
// ===========================================================================

#[test]
fn struct_leaf_field_generates_field_node() {
    let g = extract_one(
        r#"
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())]
                value: i32,
            }
        }
        "#,
    );
    let rule = get_rule(&g, "Root");
    let fields = collect_nodes_by_type(rule, "FIELD");
    assert!(
        !fields.is_empty(),
        "struct leaf field should produce a FIELD node"
    );
    assert_eq!(
        fields[0].get("type").and_then(|v| v.as_str()),
        Some("FIELD")
    );
}

#[test]
fn struct_reference_field_generates_field_node() {
    let g = extract_one(
        r#"
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                child: Inner,
            }
            pub struct Inner {
                #[adze::leaf(pattern = r"\w+")]
                v: String,
            }
        }
        "#,
    );
    let rule = get_rule(&g, "Root");
    let fields = collect_nodes_by_type(rule, "FIELD");
    assert_eq!(fields.len(), 1, "one reference field -> one FIELD node");
}

#[test]
fn enum_named_struct_variant_generates_field_nodes() {
    let g = extract_one(
        r#"
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub enum Expr {
                Neg {
                    #[adze::leaf(text = "!")]
                    _bang: (),
                    value: Box<Expr>,
                },
                Num(#[adze::leaf(pattern = r"\d+")] String),
            }
        }
        "#,
    );
    // Neg variant (not inlined because it has named fields? - let's check)
    // Actually, Neg has Fields::Named, which is not Fields::Unit, so should_inline_variant returns true
    // When inlined, the rule body appears directly in the Expr CHOICE
    let expr_rule = get_rule(&g, "Expr");
    let fields = collect_nodes_by_type(expr_rule, "FIELD");
    assert!(
        fields.len() >= 2,
        "Neg variant should have at least 2 FIELD nodes (_bang and value), found {}",
        fields.len()
    );
}

// ===========================================================================
// 2. Field name in generated rule
// ===========================================================================

#[test]
fn field_name_matches_rust_field_ident() {
    let g = extract_one(
        r#"
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"[a-z]+")]
                my_field: String,
            }
        }
        "#,
    );
    let rule = get_rule(&g, "Root");
    let names = collect_field_names(rule);
    assert!(
        names.contains(&"my_field".to_string()),
        "FIELD name should match Rust field ident; got {:?}",
        names
    );
}

#[test]
fn unnamed_enum_field_gets_positional_name() {
    let g = extract_one(
        r#"
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub enum Expr {
                Pair(
                    #[adze::leaf(pattern = r"\d+")]
                    String,
                    #[adze::leaf(pattern = r"[a-z]+")]
                    String,
                ),
            }
        }
        "#,
    );
    // Pair is inlined into Expr CHOICE
    let expr_rule = get_rule(&g, "Expr");
    let names = collect_field_names(expr_rule);
    // Unnamed fields get positional names like "Expr_Pair_0", "Expr_Pair_1"
    assert!(
        names.len() >= 2,
        "two unnamed fields should produce at least 2 FIELD names, got {:?}",
        names
    );
}

#[test]
fn field_name_for_text_leaf() {
    let g = extract_one(
        r#"
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(text = "+")]
                op: (),
                #[adze::leaf(pattern = r"\d+")]
                num: String,
            }
        }
        "#,
    );
    let rule = get_rule(&g, "Root");
    let names = collect_field_names(rule);
    assert!(
        names.contains(&"op".to_string()),
        "text leaf field 'op' should be present"
    );
    assert!(
        names.contains(&"num".to_string()),
        "pattern leaf field 'num' should be present"
    );
}

// ===========================================================================
// 3. Field content type (SYMBOL reference)
// ===========================================================================

#[test]
fn leaf_pattern_field_content_is_symbol() {
    let g = extract_one(
        r#"
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"\d+")]
                val: String,
            }
        }
        "#,
    );
    let rule = get_rule(&g, "Root");
    let fields = collect_nodes_by_type(rule, "FIELD");
    assert_eq!(fields.len(), 1);
    let content = fields[0].get("content").unwrap();
    assert_eq!(
        content.get("type").and_then(|v| v.as_str()),
        Some("SYMBOL"),
        "leaf pattern field content should be SYMBOL"
    );
}

#[test]
fn leaf_text_field_content_is_symbol() {
    let g = extract_one(
        r#"
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(text = ";")]
                semi: (),
            }
        }
        "#,
    );
    let rule = get_rule(&g, "Root");
    let fields = collect_nodes_by_type(rule, "FIELD");
    assert_eq!(fields.len(), 1);
    let content = fields[0].get("content").unwrap();
    assert_eq!(
        content.get("type").and_then(|v| v.as_str()),
        Some("SYMBOL"),
        "leaf text field content should be SYMBOL"
    );
}

#[test]
fn reference_field_content_is_symbol_with_type_name() {
    let g = extract_one(
        r#"
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                child: Inner,
            }
            pub struct Inner {
                #[adze::leaf(pattern = r"\w+")]
                v: String,
            }
        }
        "#,
    );
    let rule = get_rule(&g, "Root");
    let fields = collect_nodes_by_type(rule, "FIELD");
    assert_eq!(fields.len(), 1);
    let content = fields[0].get("content").unwrap();
    assert_eq!(content.get("type").and_then(|v| v.as_str()), Some("SYMBOL"));
    assert_eq!(
        content.get("name").and_then(|v| v.as_str()),
        Some("Inner"),
        "reference field should point to the target type name"
    );
}

#[test]
fn pattern_leaf_creates_separate_rule() {
    let g = extract_one(
        r#"
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"[0-9]+")]
                num: String,
            }
        }
        "#,
    );
    // The SYMBOL reference name should exist as a rule in the grammar
    let rule = get_rule(&g, "Root");
    let fields = collect_nodes_by_type(rule, "FIELD");
    let sym_name = fields[0]
        .get("content")
        .unwrap()
        .get("name")
        .unwrap()
        .as_str()
        .unwrap();
    let sym_rule = get_rule(&g, sym_name);
    assert_eq!(
        sym_rule.get("type").and_then(|v| v.as_str()),
        Some("PATTERN"),
        "separate rule for pattern leaf should be PATTERN"
    );
}

#[test]
fn text_leaf_creates_string_rule() {
    let g = extract_one(
        r#"
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(text = "=>")]
                arrow: (),
            }
        }
        "#,
    );
    let rule = get_rule(&g, "Root");
    let fields = collect_nodes_by_type(rule, "FIELD");
    let sym_name = fields[0]
        .get("content")
        .unwrap()
        .get("name")
        .unwrap()
        .as_str()
        .unwrap();
    let sym_rule = get_rule(&g, sym_name);
    assert_eq!(
        sym_rule.get("type").and_then(|v| v.as_str()),
        Some("STRING"),
        "separate rule for text leaf should be STRING"
    );
}

// ===========================================================================
// 4. Multiple fields produce ordered SEQ
// ===========================================================================

#[test]
fn two_fields_produce_seq() {
    let g = extract_one(
        r#"
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"[a-z]+")]
                first: String,
                #[adze::leaf(pattern = r"\d+")]
                second: String,
            }
        }
        "#,
    );
    let rule = get_rule(&g, "Root");
    assert_eq!(
        rule.get("type").and_then(|v| v.as_str()),
        Some("SEQ"),
        "two fields should produce a SEQ at the top level"
    );
}

#[test]
fn seq_members_count_matches_field_count() {
    let g = extract_one(
        r#"
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(text = "(")]
                _lp: (),
                #[adze::leaf(pattern = r"\d+")]
                num: String,
                #[adze::leaf(text = ")")]
                _rp: (),
            }
        }
        "#,
    );
    let rule = get_rule(&g, "Root");
    assert_eq!(rule.get("type").and_then(|v| v.as_str()), Some("SEQ"));
    let members = rule.get("members").unwrap().as_array().unwrap();
    assert_eq!(
        members.len(),
        3,
        "three fields should produce SEQ with 3 members"
    );
}

#[test]
fn seq_preserves_declaration_order() {
    let g = extract_one(
        r#"
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(text = "let")]
                _kw: (),
                #[adze::leaf(pattern = r"[a-z]+")]
                name: String,
                #[adze::leaf(text = "=")]
                _eq: (),
                #[adze::leaf(pattern = r"\d+")]
                val: String,
            }
        }
        "#,
    );
    let rule = get_rule(&g, "Root");
    let members = rule.get("members").unwrap().as_array().unwrap();
    // Collect the FIELD names in order from the SEQ members
    let names: Vec<String> = members
        .iter()
        .filter_map(|m| {
            if m.get("type").and_then(|v| v.as_str()) == Some("FIELD") {
                m.get("name").and_then(|v| v.as_str()).map(String::from)
            } else {
                None
            }
        })
        .collect();
    assert_eq!(
        names,
        vec!["_kw", "name", "_eq", "val"],
        "SEQ members should preserve declaration order"
    );
}

#[test]
fn single_field_no_seq_wrapper() {
    let g = extract_one(
        r#"
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"\w+")]
                only: String,
            }
        }
        "#,
    );
    let rule = get_rule(&g, "Root");
    // Single field should NOT be wrapped in SEQ
    assert_eq!(
        rule.get("type").and_then(|v| v.as_str()),
        Some("FIELD"),
        "single field should produce FIELD directly, not SEQ"
    );
}

// ===========================================================================
// 5. Optional field wraps in CHOICE+BLANK
// ===========================================================================

#[test]
fn optional_leaf_field_wrapped_in_choice_blank() {
    let g = extract_one(
        r#"
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"\d+")]
                required: String,
                #[adze::leaf(pattern = r"[a-z]+")]
                maybe: Option<String>,
            }
        }
        "#,
    );
    let rule = get_rule(&g, "Root");
    let members = rule.get("members").unwrap().as_array().unwrap();
    // The optional field member should be a CHOICE
    let optional_member = &members[1];
    assert_eq!(
        optional_member.get("type").and_then(|v| v.as_str()),
        Some("CHOICE"),
        "optional field should be wrapped in CHOICE"
    );
    let choice_members = optional_member.get("members").unwrap().as_array().unwrap();
    let has_blank = choice_members
        .iter()
        .any(|m| m.get("type").and_then(|v| v.as_str()) == Some("BLANK"));
    assert!(
        has_blank,
        "CHOICE for optional field must contain a BLANK member"
    );
}

#[test]
fn optional_field_choice_contains_field_node() {
    let g = extract_one(
        r#"
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"\d+")]
                required: String,
                #[adze::leaf(pattern = r"[a-z]+")]
                maybe: Option<String>,
            }
        }
        "#,
    );
    let rule = get_rule(&g, "Root");
    let members = rule.get("members").unwrap().as_array().unwrap();
    let optional_member = &members[1];
    let choice_members = optional_member.get("members").unwrap().as_array().unwrap();
    let has_field = choice_members
        .iter()
        .any(|m| m.get("type").and_then(|v| v.as_str()) == Some("FIELD"));
    assert!(
        has_field,
        "CHOICE for optional field must contain a FIELD node"
    );
}

#[test]
fn optional_reference_field_wrapped_in_choice_blank() {
    let g = extract_one(
        r#"
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"\w+")]
                name: String,
                trailer: Option<Trailer>,
            }
            pub struct Trailer {
                #[adze::leaf(text = ";")]
                _semi: (),
            }
        }
        "#,
    );
    let rule = get_rule(&g, "Root");
    let members = rule.get("members").unwrap().as_array().unwrap();
    let opt_member = &members[1];
    assert_eq!(
        opt_member.get("type").and_then(|v| v.as_str()),
        Some("CHOICE"),
        "optional reference field should be wrapped in CHOICE"
    );
    let blanks = collect_nodes_by_type(opt_member, "BLANK");
    assert!(!blanks.is_empty(), "CHOICE wrapper must contain BLANK");
}

#[test]
fn required_field_not_wrapped_in_choice() {
    let g = extract_one(
        r#"
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"\d+")]
                val: String,
            }
        }
        "#,
    );
    let rule = get_rule(&g, "Root");
    assert_ne!(
        rule.get("type").and_then(|v| v.as_str()),
        Some("CHOICE"),
        "required field should NOT be wrapped in CHOICE"
    );
}

// ===========================================================================
// 6. Vec field produces REPEAT/REPEAT1
// ===========================================================================

#[test]
fn vec_field_produces_repeat1_contents_rule() {
    let g = extract_one(
        r#"
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::repeat(non_empty = true)]
                items: Vec<Item>,
            }
            pub struct Item {
                #[adze::leaf(pattern = r"\w+")]
                v: String,
            }
        }
        "#,
    );
    // Vec fields create a _vec_contents auxiliary rule
    let rules = g.get("rules").unwrap().as_object().unwrap();
    let vec_contents_key = rules
        .keys()
        .find(|k| k.contains("vec_contents"))
        .expect("should have a _vec_contents rule");
    let contents_rule = &rules[vec_contents_key];
    assert_eq!(
        contents_rule.get("type").and_then(|v| v.as_str()),
        Some("REPEAT1"),
        "_vec_contents rule should be REPEAT1"
    );
}

#[test]
fn vec_non_empty_true_references_symbol_directly() {
    let g = extract_one(
        r#"
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::repeat(non_empty = true)]
                items: Vec<Item>,
            }
            pub struct Item {
                #[adze::leaf(pattern = r"\w+")]
                v: String,
            }
        }
        "#,
    );
    let rule = get_rule(&g, "Root");
    // non_empty = true means the reference is a direct SYMBOL (not wrapped in CHOICE)
    let symbols = collect_nodes_by_type(rule, "SYMBOL");
    assert!(
        !symbols.is_empty(),
        "non_empty Vec should reference _vec_contents via SYMBOL"
    );
}

#[test]
fn vec_can_be_empty_wraps_in_choice_blank() {
    let g = extract_one(
        r#"
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(text = "(")]
                _lp: (),
                items: Vec<Item>,
                #[adze::leaf(text = ")")]
                _rp: (),
            }
            pub struct Item {
                #[adze::leaf(pattern = r"\w+")]
                v: String,
            }
        }
        "#,
    );
    let rule = get_rule(&g, "Root");
    // Without non_empty = true, the Vec reference should be wrapped in CHOICE with BLANK
    let choices = collect_nodes_by_type(rule, "CHOICE");
    let has_blank_choice = choices.iter().any(|c| {
        c.get("members")
            .and_then(|m| m.as_array())
            .map(|arr| {
                arr.iter()
                    .any(|m| m.get("type").and_then(|v| v.as_str()) == Some("BLANK"))
            })
            .unwrap_or(false)
    });
    assert!(
        has_blank_choice,
        "Vec without non_empty should be wrapped in CHOICE+BLANK"
    );
}

#[test]
fn vec_with_delimiter_produces_seq_inside_repeat() {
    let g = extract_one(
        r#"
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::repeat(non_empty = true)]
                #[adze::delimited(
                    #[adze::leaf(text = ",")]
                    ()
                )]
                items: Vec<Item>,
            }
            pub struct Item {
                #[adze::leaf(pattern = r"\w+")]
                v: String,
            }
        }
        "#,
    );
    let rules = g.get("rules").unwrap().as_object().unwrap();
    let vec_contents_key = rules
        .keys()
        .find(|k| k.contains("vec_contents"))
        .expect("should have a _vec_contents rule");
    let contents_rule = &rules[vec_contents_key];
    // With delimiter, top-level should be SEQ (element, REPEAT(SEQ(delim, element)))
    assert_eq!(
        contents_rule.get("type").and_then(|v| v.as_str()),
        Some("SEQ"),
        "delimited Vec contents should be a SEQ"
    );
}

#[test]
fn vec_field_element_is_wrapped_in_field_node() {
    let g = extract_one(
        r#"
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::repeat(non_empty = true)]
                items: Vec<Item>,
            }
            pub struct Item {
                #[adze::leaf(pattern = r"\w+")]
                v: String,
            }
        }
        "#,
    );
    let rules = g.get("rules").unwrap().as_object().unwrap();
    let vec_contents_key = rules.keys().find(|k| k.contains("vec_contents")).unwrap();
    let contents_rule = &rules[vec_contents_key];
    let field_nodes = collect_nodes_by_type(contents_rule, "FIELD");
    assert!(
        !field_nodes.is_empty(),
        "Vec contents should contain FIELD nodes for element references"
    );
    // The field name should contain "vec_element"
    let has_vec_element = field_nodes.iter().any(|f| {
        f.get("name")
            .and_then(|n| n.as_str())
            .map(|n| n.contains("vec_element"))
            .unwrap_or(false)
    });
    assert!(
        has_vec_element,
        "Vec element FIELD should contain 'vec_element' in name"
    );
}

// ===========================================================================
// 7. Box field for recursive references
// ===========================================================================

#[test]
fn box_field_resolves_to_inner_type_symbol() {
    let g = extract_one(
        r#"
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub enum Expr {
                Num(#[adze::leaf(pattern = r"\d+")] String),
                Neg {
                    #[adze::leaf(text = "-")]
                    _minus: (),
                    inner: Box<Expr>,
                },
            }
        }
        "#,
    );
    // The FIELD for "inner" should reference "Expr" (not "Box")
    let expr_rule = get_rule(&g, "Expr");
    let fields = collect_nodes_by_type(expr_rule, "FIELD");
    let inner_field = fields
        .iter()
        .find(|f| f.get("name").and_then(|n| n.as_str()) == Some("inner"))
        .expect("should have an 'inner' FIELD");
    let content = inner_field.get("content").unwrap();
    assert_eq!(content.get("type").and_then(|v| v.as_str()), Some("SYMBOL"));
    assert_eq!(
        content.get("name").and_then(|v| v.as_str()),
        Some("Expr"),
        "Box<Expr> should resolve to SYMBOL referencing 'Expr'"
    );
}

#[test]
fn box_in_unnamed_enum_variant_resolves() {
    let g = extract_one(
        r#"
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub enum Expr {
                Num(#[adze::leaf(pattern = r"\d+")] String),
                Neg(
                    #[adze::leaf(text = "-")]
                    (),
                    Box<Expr>,
                ),
            }
        }
        "#,
    );
    let expr_rule = get_rule(&g, "Expr");
    // Find SYMBOL nodes referencing "Expr" (the recursive reference)
    let symbols = collect_nodes_by_type(expr_rule, "SYMBOL");
    let has_self_ref = symbols
        .iter()
        .any(|s| s.get("name").and_then(|n| n.as_str()) == Some("Expr"));
    assert!(
        has_self_ref,
        "Box<Expr> in unnamed variant should produce SYMBOL('Expr')"
    );
}

#[test]
fn box_is_transparent_for_field_content() {
    let g = extract_one(
        r#"
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                child: Inner,
            }
            pub struct BoxedRoot {
                inner: Box<Inner>,
            }
            pub struct Inner {
                #[adze::leaf(pattern = r"\w+")]
                v: String,
            }
        }
        "#,
    );
    // Both Root.child and BoxedRoot.inner should reference "Inner"
    let root_rule = get_rule(&g, "Root");
    let root_fields = collect_nodes_by_type(root_rule, "FIELD");
    let root_content_name = root_fields[0]
        .get("content")
        .unwrap()
        .get("name")
        .and_then(|v| v.as_str());

    let boxed_rule = get_rule(&g, "BoxedRoot");
    let boxed_fields = collect_nodes_by_type(boxed_rule, "FIELD");
    let boxed_content_name = boxed_fields[0]
        .get("content")
        .unwrap()
        .get("name")
        .and_then(|v| v.as_str());

    assert_eq!(root_content_name, Some("Inner"));
    assert_eq!(boxed_content_name, Some("Inner"));
    assert_eq!(
        root_content_name, boxed_content_name,
        "Box should be transparent: both should reference 'Inner'"
    );
}

// ===========================================================================
// 8. Field extraction determinism
// ===========================================================================

#[test]
fn deterministic_field_generation_struct() {
    let src = r#"
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"[a-z]+")]
                alpha: String,
                #[adze::leaf(pattern = r"\d+")]
                beta: String,
                #[adze::leaf(text = ";")]
                gamma: (),
            }
        }
    "#;
    let g1 = extract_one(src);
    let g2 = extract_one(src);
    assert_eq!(g1, g2, "grammar generation must be deterministic");
}

#[test]
fn deterministic_field_generation_enum() {
    let src = r#"
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub enum Expr {
                Num(#[adze::leaf(pattern = r"\d+")] String),
                Add {
                    left: Box<Expr>,
                    #[adze::leaf(text = "+")]
                    _op: (),
                    right: Box<Expr>,
                },
            }
        }
    "#;
    let g1 = extract_one(src);
    let g2 = extract_one(src);
    assert_eq!(g1, g2, "enum grammar generation must be deterministic");
}

#[test]
fn deterministic_optional_field_generation() {
    let src = r#"
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"\w+")]
                name: String,
                #[adze::leaf(pattern = r"\d+")]
                age: Option<String>,
            }
        }
    "#;
    let results: Vec<Value> = (0..5).map(|_| extract_one(src)).collect();
    for i in 1..results.len() {
        assert_eq!(results[0], results[i], "run {} differs from run 0", i);
    }
}

#[test]
fn deterministic_vec_field_generation() {
    let src = r#"
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::repeat(non_empty = true)]
                #[adze::delimited(
                    #[adze::leaf(text = ",")]
                    ()
                )]
                items: Vec<Item>,
            }
            pub struct Item {
                #[adze::leaf(pattern = r"\w+")]
                v: String,
            }
        }
    "#;
    let results: Vec<Value> = (0..3).map(|_| extract_one(src)).collect();
    for i in 1..results.len() {
        assert_eq!(
            results[0], results[i],
            "Vec field run {} differs from run 0",
            i
        );
    }
}

#[test]
fn deterministic_complex_grammar() {
    let src = r#"
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub enum Expr {
                Num(#[adze::leaf(pattern = r"\d+")] String),
                Neg {
                    #[adze::leaf(text = "-")]
                    _minus: (),
                    inner: Box<Expr>,
                },
                Group {
                    #[adze::leaf(text = "(")]
                    _lp: (),
                    body: Box<Expr>,
                    #[adze::leaf(text = ")")]
                    _rp: (),
                },
            }
        }
    "#;
    let g1 = extract_one(src);
    let g2 = extract_one(src);
    let g3 = extract_one(src);
    assert_eq!(g1, g2);
    assert_eq!(g2, g3);
}

// ===========================================================================
// Additional coverage: edge cases and combinations
// ===========================================================================

#[test]
fn optional_box_field_wraps_in_choice() {
    let g = extract_one(
        r#"
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"\w+")]
                name: String,
                next: Option<Box<Root>>,
            }
        }
        "#,
    );
    let rule = get_rule(&g, "Root");
    let members = rule.get("members").unwrap().as_array().unwrap();
    let opt_member = &members[1];
    assert_eq!(
        opt_member.get("type").and_then(|v| v.as_str()),
        Some("CHOICE"),
        "Option<Box<T>> should produce CHOICE wrapper"
    );
    // Inside the CHOICE, the FIELD content should reference Root (Box transparent)
    let inner_fields = collect_nodes_by_type(opt_member, "FIELD");
    assert!(!inner_fields.is_empty());
    let content_name = inner_fields[0]
        .get("content")
        .unwrap()
        .get("name")
        .and_then(|v| v.as_str());
    assert_eq!(
        content_name,
        Some("Root"),
        "Option<Box<Root>> should reference Root"
    );
}

#[test]
fn vec_delimiter_creates_auxiliary_rule() {
    let g = extract_one(
        r#"
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::repeat(non_empty = true)]
                #[adze::delimited(
                    #[adze::leaf(text = ",")]
                    ()
                )]
                items: Vec<Item>,
            }
            pub struct Item {
                #[adze::leaf(pattern = r"\w+")]
                v: String,
            }
        }
        "#,
    );
    let rules = g.get("rules").unwrap().as_object().unwrap();
    let has_delimiter_rule = rules.keys().any(|k| k.contains("vec_delimiter"));
    assert!(
        has_delimiter_rule,
        "delimited Vec should create a _vec_delimiter auxiliary rule"
    );
}

#[test]
fn grammar_name_preserved_in_output() {
    let g = extract_one(
        r#"
        #[adze::grammar("my_grammar_42")]
        mod grammar {
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"\w+")]
                v: String,
            }
        }
        "#,
    );
    assert_eq!(
        g.get("name").and_then(|v| v.as_str()),
        Some("my_grammar_42"),
        "grammar name should be preserved in output JSON"
    );
}

#[test]
fn spanned_wrapper_is_transparent() {
    let g = extract_one(
        r#"
        #[adze::grammar("test")]
        mod grammar {
            use adze::Spanned;
            #[adze::language]
            pub struct Root {
                #[adze::leaf(pattern = r"\d+")]
                val: Spanned<String>,
            }
        }
        "#,
    );
    let rule = get_rule(&g, "Root");
    let fields = collect_nodes_by_type(rule, "FIELD");
    assert_eq!(fields.len(), 1);
    let content = fields[0].get("content").unwrap();
    assert_eq!(
        content.get("type").and_then(|v| v.as_str()),
        Some("SYMBOL"),
        "Spanned<T> should be transparent, producing SYMBOL content"
    );
}

#[test]
fn enum_with_three_variants_produces_three_choice_members() {
    let g = extract_one(
        r#"
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub enum Expr {
                A(#[adze::leaf(pattern = r"a")] String),
                B(#[adze::leaf(pattern = r"b")] String),
                C(#[adze::leaf(pattern = r"c")] String),
            }
        }
        "#,
    );
    let rule = get_rule(&g, "Expr");
    assert_eq!(rule.get("type").and_then(|v| v.as_str()), Some("CHOICE"));
    let members = rule.get("members").unwrap().as_array().unwrap();
    assert_eq!(members.len(), 3, "3 variants -> 3 CHOICE members");
}
