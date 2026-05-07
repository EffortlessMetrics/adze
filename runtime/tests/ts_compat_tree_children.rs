//! Tree-sitter compatibility child traversal canaries.

#![cfg(all(test, feature = "ts-compat", feature = "pure-rust"))]

use adze::{adze_ir::RuleId, ts_compat::Parser};
use std::sync::Arc;

#[test]
fn generated_tree_exposes_nested_children() {
    let mut parser = Parser::new();
    let lang = adze_example::ts_langs::arithmetic();

    parser.set_language(lang).expect("Failed to set language");

    let source = "1-2";
    let tree = parser.parse(source, None).expect("Parse failed");
    let root = tree.root_node();

    assert_eq!(root.kind(), "source_file");
    assert_eq!(root.child_count(), 1);

    let expression = root.child(0).expect("root should expose expression child");
    assert_eq!(expression.kind(), "expression");
    assert_eq!(expression.child_count(), 3);
    assert_eq!(expression.text(source.as_bytes()), source);

    let left = expression
        .child(0)
        .expect("expression should expose left child");
    let operator = expression
        .child(1)
        .expect("expression should expose operator child");
    let right = expression
        .child(2)
        .expect("expression should expose right child");

    assert_eq!(left.kind(), "expression");
    assert_eq!(left.text(source.as_bytes()), "1");
    assert_eq!(operator.kind(), "-");
    assert_eq!(operator.text(source.as_bytes()), "-");
    assert_eq!(right.kind(), "expression");
    assert_eq!(right.text(source.as_bytes()), "2");
    assert!(expression.child(3).is_none());
}

#[test]
fn generated_tree_exposes_fielded_children() {
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

    let source = "1-2";
    let tree = parser.parse(source, None).expect("Parse failed");
    let expression = tree
        .root_node()
        .child(0)
        .expect("root should expose expression child");

    assert_eq!(expression.field_name(), None);
    assert_eq!(expression.field_name_for_child(0), Some("left"));
    assert_eq!(expression.field_name_for_child(1), Some("operator"));
    assert_eq!(expression.field_name_for_child(2), Some("right"));
    assert_eq!(expression.field_name_for_child(3), None);

    let left = expression
        .child_by_field_name("left")
        .expect("left field should resolve");
    let operator = expression
        .child_by_field_name("operator")
        .expect("operator field should resolve");
    let right = expression
        .child_by_field_name("right")
        .expect("right field should resolve");

    assert_eq!(left.field_name(), Some("left"));
    assert_eq!(left.text(source.as_bytes()), "1");
    assert_eq!(operator.field_name(), Some("operator"));
    assert_eq!(operator.text(source.as_bytes()), "-");
    assert_eq!(right.field_name(), Some("right"));
    assert_eq!(right.text(source.as_bytes()), "2");
    assert!(expression.child_by_field_name("missing").is_none());
}
