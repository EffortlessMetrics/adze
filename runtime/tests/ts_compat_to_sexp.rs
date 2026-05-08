//! Tree-sitter compatibility S-expression canaries.

#![cfg(all(test, feature = "ts-compat", feature = "pure-rust"))]

use adze::{
    adze_ir::RuleId,
    ts_compat::{Language, Parser},
};
use std::sync::Arc;

fn arithmetic_with_fields() -> Language {
    let mut lang = (*adze_example::ts_langs::arithmetic()).clone();
    lang.table.field_names = vec![
        "left".to_string(),
        "operator".to_string(),
        "right".to_string(),
    ];
    lang.table.field_map.insert((RuleId(2), 0), 0);
    lang.table.field_map.insert((RuleId(2), 1), 1);
    lang.table.field_map.insert((RuleId(2), 2), 2);
    lang
}

#[test]
fn to_sexp_serializes_generated_arithmetic_tree() {
    let mut parser = Parser::new();
    parser
        .set_language(adze_example::ts_langs::arithmetic())
        .expect("Failed to set language");

    let tree = parser.parse("1-2", None).expect("Parse failed");

    assert_eq!(
        tree.root_node().to_sexp(),
        "(source_file (expression (expression) (expression)))"
    );
}

#[test]
fn to_sexp_matches_root_and_subtree_contract_for_generated_arithmetic() {
    let mut parser = Parser::new();
    parser
        .set_language(adze_example::ts_langs::arithmetic())
        .expect("Failed to set language");

    let tree = parser.parse("1-2", None).expect("Parse failed");
    let root = tree.root_node();
    let expression = root.child(0).expect("root should expose expression child");

    assert_eq!(
        root.to_sexp(),
        "(source_file (expression (expression) (expression)))"
    );
    assert_eq!(
        expression.to_sexp(),
        "(expression (expression) (expression))"
    );
}

#[test]
fn to_sexp_omits_anonymous_operator_children() {
    let mut parser = Parser::new();
    parser
        .set_language(adze_example::ts_langs::arithmetic())
        .expect("Failed to set language");

    let tree = parser.parse("1-2", None).expect("Parse failed");
    let expression = tree
        .root_node()
        .child(0)
        .expect("root should expose expression child");

    assert_eq!(expression.child_count(), 3);
    assert_eq!(expression.named_child_count(), 2);
    assert_eq!(
        expression.to_sexp(),
        "(expression (expression) (expression))"
    );
    assert!(!expression.to_sexp().contains("-"));
}

#[test]
fn to_sexp_includes_field_labels_for_named_children() {
    let mut parser = Parser::new();
    parser
        .set_language(Arc::new(arithmetic_with_fields()))
        .expect("Failed to set language");

    let tree = parser.parse("1-2", None).expect("Parse failed");

    assert_eq!(
        tree.root_node().to_sexp(),
        "(source_file (expression left: (expression) right: (expression)))"
    );
}

#[test]
fn to_sexp_field_labels_round_trip_through_language_field_ids() {
    let mut parser = Parser::new();
    parser
        .set_language(Arc::new(arithmetic_with_fields()))
        .expect("Failed to set language");

    let source = "1-2";
    let tree = parser.parse(source, None).expect("Parse failed");
    let expression = tree
        .root_node()
        .child(0)
        .expect("root should expose expression child");
    let left_id = tree
        .language()
        .field_id_for_name("left")
        .expect("left should have a public field id");
    let right_id = tree
        .language()
        .field_id_for_name("right")
        .expect("right should have a public field id");

    assert_eq!(
        tree.language().field_name_for_id(left_id.get()),
        Some("left")
    );
    assert_eq!(
        expression
            .child_by_field_id(left_id.get())
            .expect("left field id should resolve")
            .text(source.as_bytes()),
        "1"
    );
    assert_eq!(
        expression
            .child_by_field_id(right_id.get())
            .expect("right field id should resolve")
            .text(source.as_bytes()),
        "2"
    );
    assert_eq!(
        expression.to_sexp(),
        "(expression left: (expression) right: (expression))"
    );
}

#[test]
fn to_sexp_remains_named_only_when_anonymous_child_has_field_id() {
    let mut parser = Parser::new();
    parser
        .set_language(Arc::new(arithmetic_with_fields()))
        .expect("Failed to set language");

    let tree = parser.parse("1-2", None).expect("Parse failed");
    let expression = tree
        .root_node()
        .child(0)
        .expect("root should expose expression child");
    let operator = expression
        .child(1)
        .expect("expression should expose operator child");

    assert!(!operator.is_named());
    assert_eq!(expression.field_name_for_child(1), Some("operator"));
    assert_eq!(expression.field_id_for_child(1).map(|id| id.get()), Some(2));
    assert_eq!(
        expression.to_sexp(),
        "(expression left: (expression) right: (expression))"
    );
    assert!(!expression.to_sexp().contains("operator:"));
    assert!(!expression.to_sexp().contains("-"));
}
