//! Tree-sitter compatibility cursor reset canaries.

#![cfg(all(test, feature = "ts-compat", feature = "pure-rust"))]

use adze::{
    adze_ir::RuleId,
    ts_compat::{Parser, Tree},
};
use std::sync::Arc;

fn parse_arithmetic(source: &str) -> Tree {
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
    parser.parse(source, None).expect("Parse failed")
}

#[test]
fn cursor_reset_restarts_at_node_and_clears_parent_state() {
    let source = "1-2";
    let tree = parse_arithmetic(source);
    let root = tree.root_node();
    let expression = root.child(0).expect("root should have expression child");
    let right_expression = expression
        .child(2)
        .expect("expression should have right child");

    let mut cursor = root.walk();
    assert!(cursor.goto_first_child());
    assert!(cursor.goto_first_child());
    assert_eq!(cursor.depth(), 2);
    assert_eq!(cursor.node().text(source.as_bytes()), "1");

    cursor.reset(right_expression);
    assert_eq!(cursor.depth(), 0);
    assert_eq!(cursor.node().kind(), "expression");
    assert_eq!(cursor.field_name(), None);
    assert_eq!(cursor.node().text(source.as_bytes()), "2");
    assert!(!cursor.goto_parent());
    assert!(!cursor.goto_previous_sibling());
    assert_eq!(cursor.depth(), 0);
    assert_eq!(cursor.field_name(), None);
    assert_eq!(cursor.node().text(source.as_bytes()), "2");

    assert!(cursor.goto_first_child());
    assert_eq!(cursor.depth(), 1);
    assert_eq!(cursor.node().kind(), "number");
    assert_eq!(cursor.node().text(source.as_bytes()), "2");
}

#[test]
fn cursor_reset_to_copies_position_and_parent_state() {
    let source = "1-2";
    let tree = parse_arithmetic(source);
    let root = tree.root_node();

    let mut source_cursor = root.walk();
    assert!(source_cursor.goto_first_child());
    assert!(source_cursor.goto_first_child());
    assert!(source_cursor.goto_next_sibling());
    assert!(source_cursor.goto_next_sibling());
    assert_eq!(source_cursor.depth(), 2);
    assert_eq!(source_cursor.field_name(), Some("right"));
    assert_eq!(source_cursor.node().text(source.as_bytes()), "2");

    let mut target_cursor = root.walk();
    assert_eq!(target_cursor.depth(), 0);
    assert_eq!(target_cursor.node().kind(), "source_file");

    target_cursor.reset_to(&source_cursor);
    assert_eq!(target_cursor.depth(), 2);
    assert_eq!(target_cursor.node().kind(), "expression");
    assert_eq!(target_cursor.field_name(), Some("right"));
    assert_eq!(target_cursor.node().text(source.as_bytes()), "2");

    assert!(target_cursor.goto_previous_sibling());
    assert_eq!(target_cursor.depth(), 2);
    assert_eq!(target_cursor.node().kind(), "-");
    assert_eq!(target_cursor.field_name(), Some("operator"));
    assert_eq!(target_cursor.node().text(source.as_bytes()), "-");

    assert_eq!(source_cursor.depth(), 2);
    assert_eq!(source_cursor.field_name(), Some("right"));
    assert_eq!(source_cursor.node().text(source.as_bytes()), "2");

    assert!(target_cursor.goto_parent());
    assert_eq!(target_cursor.depth(), 1);
    assert_eq!(target_cursor.field_name(), None);
    assert_eq!(target_cursor.node().text(source.as_bytes()), source);
}
