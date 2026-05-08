//! Tree-sitter compatibility S-expression canaries.

#![cfg(all(test, feature = "ts-compat", feature = "pure-rust"))]

use adze::{adze_ir::RuleId, ts_compat::Parser};
use std::sync::Arc;

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
    let mut lang = (*adze_example::ts_langs::arithmetic()).clone();
    lang.table.field_names = vec![
        "left".to_string(),
        "operator".to_string(),
        "right".to_string(),
    ];
    lang.table.field_map.insert((RuleId(2), 0), 0);
    lang.table.field_map.insert((RuleId(2), 1), 1);
    lang.table.field_map.insert((RuleId(2), 2), 2);

    let mut parser = Parser::new();
    parser
        .set_language(Arc::new(lang))
        .expect("Failed to set language");

    let tree = parser.parse("1-2", None).expect("Parse failed");

    assert_eq!(
        tree.root_node().to_sexp(),
        "(source_file (expression left: (expression) right: (expression)))"
    );
}
