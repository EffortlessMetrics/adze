#![allow(clippy::needless_range_loop)]

//! Property-based and unit tests for precedence generation in adze-tool.
//!
//! Covers PREC_LEFT, PREC_RIGHT, PREC generation, precedence values,
//! multiple levels, nested rules, plain rules, and determinism.

use serde_json::Value;
use std::fs;
use tempfile::TempDir;

// ===========================================================================
// Helpers
// ===========================================================================

fn extract_one(src: &str) -> Value {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("lib.rs");
    fs::write(&path, src).unwrap();
    let gs = adze_tool::generate_grammars(&path).unwrap();
    assert_eq!(gs.len(), 1, "expected exactly one grammar");
    gs.into_iter().next().unwrap()
}

/// Recursively collect every JSON node whose "type" equals `type_name`.
fn collect_nodes_of_type(val: &Value, type_name: &str) -> Vec<Value> {
    let mut out = Vec::new();
    match val {
        Value::Object(map) => {
            if map.get("type").and_then(|v| v.as_str()) == Some(type_name) {
                out.push(val.clone());
            }
            for v in map.values() {
                out.extend(collect_nodes_of_type(v, type_name));
            }
        }
        Value::Array(arr) => {
            for v in arr {
                out.extend(collect_nodes_of_type(v, type_name));
            }
        }
        _ => {}
    }
    out
}

// ===========================================================================
// 1. PREC_LEFT generation for left-associative operators
// ===========================================================================

#[test]
fn prec_left_basic_subtraction() {
    let g = extract_one(
        r#"
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub enum Expr {
                Num(#[adze::leaf(pattern = r"\d+")] String),
                #[adze::prec_left(1)]
                Sub(
                    Box<Expr>,
                    #[adze::leaf(text = "-")] (),
                    Box<Expr>,
                ),
            }
        }
    "#,
    );
    let nodes = collect_nodes_of_type(&g, "PREC_LEFT");
    assert!(!nodes.is_empty(), "expected at least one PREC_LEFT node");
    assert_eq!(nodes[0]["value"], 1);
}

#[test]
fn prec_left_addition() {
    let g = extract_one(
        r#"
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub enum Expr {
                Num(#[adze::leaf(pattern = r"\d+")] String),
                #[adze::prec_left(2)]
                Add(
                    Box<Expr>,
                    #[adze::leaf(text = "+")] (),
                    Box<Expr>,
                ),
            }
        }
    "#,
    );
    let nodes = collect_nodes_of_type(&g, "PREC_LEFT");
    assert_eq!(nodes.len(), 1);
    assert_eq!(nodes[0]["value"], 2);
}

#[test]
fn prec_left_content_is_seq() {
    let g = extract_one(
        r#"
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub enum Expr {
                Num(#[adze::leaf(pattern = r"\d+")] String),
                #[adze::prec_left(1)]
                Sub(
                    Box<Expr>,
                    #[adze::leaf(text = "-")] (),
                    Box<Expr>,
                ),
            }
        }
    "#,
    );
    let nodes = collect_nodes_of_type(&g, "PREC_LEFT");
    assert_eq!(nodes[0]["content"]["type"], "SEQ");
}

#[test]
fn prec_left_zero_value() {
    let g = extract_one(
        r#"
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub enum Expr {
                Num(#[adze::leaf(pattern = r"\d+")] String),
                #[adze::prec_left(0)]
                Add(
                    Box<Expr>,
                    #[adze::leaf(text = "+")] (),
                    Box<Expr>,
                ),
            }
        }
    "#,
    );
    let nodes = collect_nodes_of_type(&g, "PREC_LEFT");
    assert_eq!(nodes.len(), 1);
    assert_eq!(nodes[0]["value"], 0);
}

// ===========================================================================
// 2. PREC_RIGHT generation for right-associative operators
// ===========================================================================

#[test]
fn prec_right_basic_exponent() {
    let g = extract_one(
        r#"
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub enum Expr {
                Num(#[adze::leaf(pattern = r"\d+")] String),
                #[adze::prec_right(3)]
                Pow(
                    Box<Expr>,
                    #[adze::leaf(text = "^")] (),
                    Box<Expr>,
                ),
            }
        }
    "#,
    );
    let nodes = collect_nodes_of_type(&g, "PREC_RIGHT");
    assert!(!nodes.is_empty(), "expected at least one PREC_RIGHT node");
    assert_eq!(nodes[0]["value"], 3);
}

#[test]
fn prec_right_assignment() {
    let g = extract_one(
        r#"
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub enum Expr {
                Num(#[adze::leaf(pattern = r"\d+")] String),
                #[adze::prec_right(1)]
                Assign(
                    Box<Expr>,
                    #[adze::leaf(text = "=")] (),
                    Box<Expr>,
                ),
            }
        }
    "#,
    );
    let nodes = collect_nodes_of_type(&g, "PREC_RIGHT");
    assert_eq!(nodes.len(), 1);
    assert_eq!(nodes[0]["value"], 1);
}

#[test]
fn prec_right_content_is_seq() {
    let g = extract_one(
        r#"
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub enum Expr {
                Num(#[adze::leaf(pattern = r"\d+")] String),
                #[adze::prec_right(5)]
                Pow(
                    Box<Expr>,
                    #[adze::leaf(text = "^")] (),
                    Box<Expr>,
                ),
            }
        }
    "#,
    );
    let nodes = collect_nodes_of_type(&g, "PREC_RIGHT");
    assert_eq!(nodes[0]["content"]["type"], "SEQ");
}

#[test]
fn prec_right_no_prec_left_leak() {
    let g = extract_one(
        r#"
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub enum Expr {
                Num(#[adze::leaf(pattern = r"\d+")] String),
                #[adze::prec_right(2)]
                Pow(
                    Box<Expr>,
                    #[adze::leaf(text = "^")] (),
                    Box<Expr>,
                ),
            }
        }
    "#,
    );
    let left_nodes = collect_nodes_of_type(&g, "PREC_LEFT");
    assert!(
        left_nodes.is_empty(),
        "PREC_RIGHT should not produce PREC_LEFT nodes"
    );
}

// ===========================================================================
// 3. PREC generation for non-associative precedence
// ===========================================================================

#[test]
fn prec_non_associative() {
    let g = extract_one(
        r#"
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub enum Expr {
                Num(#[adze::leaf(pattern = r"\d+")] String),
                #[adze::prec(4)]
                Cmp(
                    Box<Expr>,
                    #[adze::leaf(text = "==")] (),
                    Box<Expr>,
                ),
            }
        }
    "#,
    );
    let nodes = collect_nodes_of_type(&g, "PREC");
    // Filter out PREC_LEFT / PREC_RIGHT which also match substring
    let non_assoc: Vec<_> = nodes
        .iter()
        .filter(|n| n["type"].as_str() == Some("PREC"))
        .collect();
    assert!(!non_assoc.is_empty(), "expected at least one PREC node");
    assert_eq!(non_assoc[0]["value"], 4);
}

#[test]
fn prec_non_associative_content_wraps_seq() {
    let g = extract_one(
        r#"
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub enum Expr {
                Num(#[adze::leaf(pattern = r"\d+")] String),
                #[adze::prec(2)]
                Cmp(
                    Box<Expr>,
                    #[adze::leaf(text = "<")] (),
                    Box<Expr>,
                ),
            }
        }
    "#,
    );
    let nodes = collect_nodes_of_type(&g, "PREC");
    let non_assoc: Vec<_> = nodes
        .iter()
        .filter(|n| n["type"].as_str() == Some("PREC"))
        .collect();
    assert_eq!(non_assoc[0]["content"]["type"], "SEQ");
}

#[test]
fn prec_does_not_generate_left_or_right() {
    let g = extract_one(
        r#"
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub enum Expr {
                Num(#[adze::leaf(pattern = r"\d+")] String),
                #[adze::prec(7)]
                Neg(
                    Box<Expr>,
                    #[adze::leaf(text = "!")] (),
                    Box<Expr>,
                ),
            }
        }
    "#,
    );
    let left = collect_nodes_of_type(&g, "PREC_LEFT");
    let right = collect_nodes_of_type(&g, "PREC_RIGHT");
    assert!(left.is_empty(), "PREC should not produce PREC_LEFT");
    assert!(right.is_empty(), "PREC should not produce PREC_RIGHT");
}

// ===========================================================================
// 4. Precedence values in generated JSON
// ===========================================================================

#[test]
fn prec_left_value_preserved_in_json() {
    let g = extract_one(
        r#"
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub enum Expr {
                Num(#[adze::leaf(pattern = r"\d+")] String),
                #[adze::prec_left(42)]
                Op(
                    Box<Expr>,
                    #[adze::leaf(text = "+")] (),
                    Box<Expr>,
                ),
            }
        }
    "#,
    );
    let json_str = serde_json::to_string(&g).unwrap();
    assert!(json_str.contains(r#""type":"PREC_LEFT"#));
    assert!(json_str.contains(r#""value":42"#));
}

#[test]
fn prec_right_value_preserved_in_json() {
    let g = extract_one(
        r#"
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub enum Expr {
                Num(#[adze::leaf(pattern = r"\d+")] String),
                #[adze::prec_right(99)]
                Op(
                    Box<Expr>,
                    #[adze::leaf(text = "^")] (),
                    Box<Expr>,
                ),
            }
        }
    "#,
    );
    let json_str = serde_json::to_string(&g).unwrap();
    assert!(json_str.contains(r#""type":"PREC_RIGHT"#));
    assert!(json_str.contains(r#""value":99"#));
}

#[test]
fn prec_value_preserved_in_json() {
    let g = extract_one(
        r#"
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub enum Expr {
                Num(#[adze::leaf(pattern = r"\d+")] String),
                #[adze::prec(10)]
                Op(
                    Box<Expr>,
                    #[adze::leaf(text = "==")] (),
                    Box<Expr>,
                ),
            }
        }
    "#,
    );
    let json_str = serde_json::to_string(&g).unwrap();
    assert!(json_str.contains(r#""type":"PREC"#));
    assert!(json_str.contains(r#""value":10"#));
}

#[test]
fn prec_large_value() {
    let g = extract_one(
        r#"
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub enum Expr {
                Num(#[adze::leaf(pattern = r"\d+")] String),
                #[adze::prec_left(1000)]
                Op(
                    Box<Expr>,
                    #[adze::leaf(text = "+")] (),
                    Box<Expr>,
                ),
            }
        }
    "#,
    );
    let nodes = collect_nodes_of_type(&g, "PREC_LEFT");
    assert_eq!(nodes[0]["value"], 1000);
}

// ===========================================================================
// 5. Multiple precedence levels
// ===========================================================================

#[test]
fn multiple_prec_left_levels() {
    let g = extract_one(
        r#"
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub enum Expr {
                Num(#[adze::leaf(pattern = r"\d+")] String),
                #[adze::prec_left(1)]
                Add(
                    Box<Expr>,
                    #[adze::leaf(text = "+")] (),
                    Box<Expr>,
                ),
                #[adze::prec_left(2)]
                Mul(
                    Box<Expr>,
                    #[adze::leaf(text = "*")] (),
                    Box<Expr>,
                ),
            }
        }
    "#,
    );
    let nodes = collect_nodes_of_type(&g, "PREC_LEFT");
    assert_eq!(nodes.len(), 2, "expected two PREC_LEFT nodes");
    let values: Vec<i64> = nodes.iter().map(|n| n["value"].as_i64().unwrap()).collect();
    assert!(values.contains(&1));
    assert!(values.contains(&2));
}

#[test]
fn mixed_prec_left_and_right() {
    let g = extract_one(
        r#"
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub enum Expr {
                Num(#[adze::leaf(pattern = r"\d+")] String),
                #[adze::prec_left(1)]
                Add(
                    Box<Expr>,
                    #[adze::leaf(text = "+")] (),
                    Box<Expr>,
                ),
                #[adze::prec_right(2)]
                Pow(
                    Box<Expr>,
                    #[adze::leaf(text = "^")] (),
                    Box<Expr>,
                ),
            }
        }
    "#,
    );
    let left = collect_nodes_of_type(&g, "PREC_LEFT");
    let right = collect_nodes_of_type(&g, "PREC_RIGHT");
    assert_eq!(left.len(), 1);
    assert_eq!(right.len(), 1);
    assert_eq!(left[0]["value"], 1);
    assert_eq!(right[0]["value"], 2);
}

#[test]
fn three_different_prec_types() {
    let g = extract_one(
        r#"
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub enum Expr {
                Num(#[adze::leaf(pattern = r"\d+")] String),
                #[adze::prec_left(1)]
                Add(
                    Box<Expr>,
                    #[adze::leaf(text = "+")] (),
                    Box<Expr>,
                ),
                #[adze::prec_right(2)]
                Pow(
                    Box<Expr>,
                    #[adze::leaf(text = "^")] (),
                    Box<Expr>,
                ),
                #[adze::prec(3)]
                Cmp(
                    Box<Expr>,
                    #[adze::leaf(text = "==")] (),
                    Box<Expr>,
                ),
            }
        }
    "#,
    );
    let left = collect_nodes_of_type(&g, "PREC_LEFT");
    let right = collect_nodes_of_type(&g, "PREC_RIGHT");
    let prec: Vec<_> = collect_nodes_of_type(&g, "PREC")
        .into_iter()
        .filter(|n| n["type"].as_str() == Some("PREC"))
        .collect();
    assert_eq!(left.len(), 1);
    assert_eq!(right.len(), 1);
    assert_eq!(prec.len(), 1);
}

#[test]
fn four_levels_ascending() {
    let g = extract_one(
        r#"
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub enum Expr {
                Num(#[adze::leaf(pattern = r"\d+")] String),
                #[adze::prec_left(1)]
                Add(Box<Expr>, #[adze::leaf(text = "+")] (), Box<Expr>),
                #[adze::prec_left(2)]
                Sub(Box<Expr>, #[adze::leaf(text = "-")] (), Box<Expr>),
                #[adze::prec_left(3)]
                Mul(Box<Expr>, #[adze::leaf(text = "*")] (), Box<Expr>),
                #[adze::prec_left(4)]
                Div(Box<Expr>, #[adze::leaf(text = "/")] (), Box<Expr>),
            }
        }
    "#,
    );
    let nodes = collect_nodes_of_type(&g, "PREC_LEFT");
    assert_eq!(nodes.len(), 4);
    let mut values: Vec<i64> = nodes.iter().map(|n| n["value"].as_i64().unwrap()).collect();
    values.sort();
    assert_eq!(values, vec![1, 2, 3, 4]);
}

// ===========================================================================
// 6. Precedence with nested rules
// ===========================================================================

#[test]
fn prec_left_with_fields() {
    let g = extract_one(
        r#"
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub enum Expr {
                Num(#[adze::leaf(pattern = r"\d+")] String),
                #[adze::prec_left(1)]
                Add(
                    Box<Expr>,
                    #[adze::leaf(text = "+")] (),
                    Box<Expr>,
                ),
            }
        }
    "#,
    );
    let nodes = collect_nodes_of_type(&g, "PREC_LEFT");
    let content = &nodes[0]["content"];
    assert_eq!(content["type"], "SEQ");
    let members = content["members"].as_array().unwrap();
    // Should have FIELD nodes wrapping the operands
    let field_count = members
        .iter()
        .filter(|m| m["type"].as_str() == Some("FIELD"))
        .count();
    assert!(
        field_count >= 2,
        "expected at least 2 FIELD nodes in SEQ, got {field_count}"
    );
}

#[test]
fn prec_right_preserves_seq_structure() {
    let g = extract_one(
        r#"
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub enum Expr {
                Num(#[adze::leaf(pattern = r"\d+")] String),
                #[adze::prec_right(5)]
                Pow(
                    Box<Expr>,
                    #[adze::leaf(text = "^")] (),
                    Box<Expr>,
                ),
            }
        }
    "#,
    );
    let nodes = collect_nodes_of_type(&g, "PREC_RIGHT");
    let members = nodes[0]["content"]["members"].as_array().unwrap();
    assert_eq!(members.len(), 3, "binary op SEQ should have 3 members");
}

#[test]
fn prec_with_choice_parent() {
    let g = extract_one(
        r#"
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub enum Expr {
                Num(#[adze::leaf(pattern = r"\d+")] String),
                #[adze::prec_left(1)]
                Add(Box<Expr>, #[adze::leaf(text = "+")] (), Box<Expr>),
                #[adze::prec_left(2)]
                Mul(Box<Expr>, #[adze::leaf(text = "*")] (), Box<Expr>),
            }
        }
    "#,
    );
    // The top-level Expr rule should be a CHOICE
    let expr_rule = &g["rules"]["Expr"];
    assert_eq!(expr_rule["type"], "CHOICE");
}

#[test]
fn prec_nested_in_symbol_references() {
    let g = extract_one(
        r#"
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub enum Expr {
                Num(#[adze::leaf(pattern = r"\d+")] String),
                #[adze::prec_left(1)]
                Add(Box<Expr>, #[adze::leaf(text = "+")] (), Box<Expr>),
            }
        }
    "#,
    );
    // The PREC_LEFT content should reference Expr via SYMBOL nodes
    let nodes = collect_nodes_of_type(&g, "PREC_LEFT");
    let symbols = collect_nodes_of_type(&nodes[0]["content"].clone(), "SYMBOL");
    let expr_refs: Vec<_> = symbols
        .iter()
        .filter(|s| s["name"].as_str() == Some("Expr"))
        .collect();
    assert!(
        expr_refs.len() >= 2,
        "expected at least 2 SYMBOL refs to Expr (left and right operand)"
    );
}

// ===========================================================================
// 7. No precedence case (plain rules)
// ===========================================================================

#[test]
fn no_prec_no_prec_nodes() {
    let g = extract_one(
        r#"
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub enum Expr {
                Num(#[adze::leaf(pattern = r"\d+")] String),
                Add(
                    Box<Expr>,
                    #[adze::leaf(text = "+")] (),
                    Box<Expr>,
                ),
            }
        }
    "#,
    );
    let left = collect_nodes_of_type(&g, "PREC_LEFT");
    let right = collect_nodes_of_type(&g, "PREC_RIGHT");
    let prec: Vec<_> = collect_nodes_of_type(&g, "PREC")
        .into_iter()
        .filter(|n| n["type"].as_str() == Some("PREC"))
        .collect();
    assert!(left.is_empty(), "plain rule should not have PREC_LEFT");
    assert!(right.is_empty(), "plain rule should not have PREC_RIGHT");
    assert!(prec.is_empty(), "plain rule should not have PREC");
}

#[test]
fn plain_struct_no_prec_nodes() {
    let g = extract_one(
        r#"
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub struct Token {
                #[adze::leaf(pattern = r"\d+")]
                value: String,
            }
        }
    "#,
    );
    let left = collect_nodes_of_type(&g, "PREC_LEFT");
    let right = collect_nodes_of_type(&g, "PREC_RIGHT");
    let prec: Vec<_> = collect_nodes_of_type(&g, "PREC")
        .into_iter()
        .filter(|n| n["type"].as_str() == Some("PREC"))
        .collect();
    assert!(left.is_empty());
    assert!(right.is_empty());
    assert!(prec.is_empty());
}

#[test]
fn mixed_prec_and_plain_variants() {
    let g = extract_one(
        r#"
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub enum Expr {
                Num(#[adze::leaf(pattern = r"\d+")] String),
                #[adze::prec_left(1)]
                Add(Box<Expr>, #[adze::leaf(text = "+")] (), Box<Expr>),
                Plain(Box<Expr>, #[adze::leaf(text = "?")] (), Box<Expr>),
            }
        }
    "#,
    );
    let left = collect_nodes_of_type(&g, "PREC_LEFT");
    assert_eq!(
        left.len(),
        1,
        "only the annotated variant should be PREC_LEFT"
    );
}

// ===========================================================================
// 8. Precedence generation determinism
// ===========================================================================

#[test]
fn deterministic_prec_left() {
    let src = r#"
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub enum Expr {
                Num(#[adze::leaf(pattern = r"\d+")] String),
                #[adze::prec_left(1)]
                Add(Box<Expr>, #[adze::leaf(text = "+")] (), Box<Expr>),
            }
        }
    "#;
    let g1 = serde_json::to_string(&extract_one(src)).unwrap();
    let g2 = serde_json::to_string(&extract_one(src)).unwrap();
    assert_eq!(g1, g2, "PREC_LEFT generation should be deterministic");
}

#[test]
fn deterministic_prec_right() {
    let src = r#"
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub enum Expr {
                Num(#[adze::leaf(pattern = r"\d+")] String),
                #[adze::prec_right(3)]
                Pow(Box<Expr>, #[adze::leaf(text = "^")] (), Box<Expr>),
            }
        }
    "#;
    let g1 = serde_json::to_string(&extract_one(src)).unwrap();
    let g2 = serde_json::to_string(&extract_one(src)).unwrap();
    assert_eq!(g1, g2, "PREC_RIGHT generation should be deterministic");
}

#[test]
fn deterministic_mixed_prec_levels() {
    let src = r#"
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub enum Expr {
                Num(#[adze::leaf(pattern = r"\d+")] String),
                #[adze::prec_left(1)]
                Add(Box<Expr>, #[adze::leaf(text = "+")] (), Box<Expr>),
                #[adze::prec_left(2)]
                Mul(Box<Expr>, #[adze::leaf(text = "*")] (), Box<Expr>),
                #[adze::prec_right(3)]
                Pow(Box<Expr>, #[adze::leaf(text = "^")] (), Box<Expr>),
            }
        }
    "#;
    let g1 = serde_json::to_string(&extract_one(src)).unwrap();
    let g2 = serde_json::to_string(&extract_one(src)).unwrap();
    assert_eq!(
        g1, g2,
        "mixed precedence generation should be deterministic"
    );
}

#[test]
fn deterministic_five_runs() {
    let src = r#"
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub enum Expr {
                Num(#[adze::leaf(pattern = r"\d+")] String),
                #[adze::prec_left(1)]
                Add(Box<Expr>, #[adze::leaf(text = "+")] (), Box<Expr>),
                #[adze::prec(5)]
                Cmp(Box<Expr>, #[adze::leaf(text = "==")] (), Box<Expr>),
            }
        }
    "#;
    let baseline = serde_json::to_string(&extract_one(src)).unwrap();
    for i in 0..5 {
        let run = serde_json::to_string(&extract_one(src)).unwrap();
        assert_eq!(baseline, run, "run {i} differed from baseline");
    }
}

// ===========================================================================
// Additional coverage: rule naming, operator inlining
// ===========================================================================

#[test]
fn prec_left_variant_becomes_named_rule() {
    let g = extract_one(
        r#"
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub enum Expr {
                Num(#[adze::leaf(pattern = r"\d+")] String),
                #[adze::prec_left(1)]
                Sub(Box<Expr>, #[adze::leaf(text = "-")] (), Box<Expr>),
            }
        }
    "#,
    );
    let rules = g["rules"].as_object().unwrap();
    let has_sub_rule = rules.keys().any(|k| k.contains("Sub"));
    assert!(
        has_sub_rule,
        "prec_left variant should generate a named rule containing 'Sub'"
    );
}

#[test]
fn prec_right_variant_becomes_named_rule() {
    let g = extract_one(
        r#"
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub enum Expr {
                Num(#[adze::leaf(pattern = r"\d+")] String),
                #[adze::prec_right(2)]
                Pow(Box<Expr>, #[adze::leaf(text = "^")] (), Box<Expr>),
            }
        }
    "#,
    );
    let rules = g["rules"].as_object().unwrap();
    let has_pow_rule = rules.keys().any(|k| k.contains("Pow"));
    assert!(
        has_pow_rule,
        "prec_right variant should generate a named rule containing 'Pow'"
    );
}

#[test]
fn prec_operator_string_inlined() {
    let g = extract_one(
        r#"
        #[adze::grammar("test")]
        mod grammar {
            #[adze::language]
            pub enum Expr {
                Num(#[adze::leaf(pattern = r"\d+")] String),
                #[adze::prec_left(1)]
                Add(Box<Expr>, #[adze::leaf(text = "+")] (), Box<Expr>),
            }
        }
    "#,
    );
    let json_str = serde_json::to_string(&g).unwrap();
    // The operator "+" should appear as a STRING node in the generated JSON
    assert!(
        json_str.contains(r#""type":"STRING","value":"+""#),
        "operator should be inlined as STRING node"
    );
}

#[test]
fn grammar_name_preserved_with_prec() {
    let g = extract_one(
        r#"
        #[adze::grammar("my_grammar")]
        mod grammar {
            #[adze::language]
            pub enum Expr {
                Num(#[adze::leaf(pattern = r"\d+")] String),
                #[adze::prec_left(1)]
                Add(Box<Expr>, #[adze::leaf(text = "+")] (), Box<Expr>),
            }
        }
    "#,
    );
    assert_eq!(g["name"], "my_grammar");
}
