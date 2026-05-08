//! Tree-sitter compatibility cursor reverse/end traversal canaries.

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
fn cursor_goto_last_child_moves_to_last_direct_child() {
    let source = "1-2";
    let tree = parse_arithmetic(source);
    let mut cursor = tree.root_node().walk();

    assert!(cursor.goto_last_child());
    assert_eq!(cursor.node().kind(), "expression");
    assert_eq!(cursor.field_name(), None);
    assert_eq!(cursor.node().text(source.as_bytes()), source);

    assert!(cursor.goto_last_child());
    assert_eq!(cursor.node().kind(), "expression");
    assert_eq!(cursor.field_name(), Some("right"));
    assert_eq!(cursor.node().text(source.as_bytes()), "2");

    assert!(cursor.goto_last_child());
    assert_eq!(cursor.node().kind(), "number");
    assert_eq!(cursor.field_name(), None);
    assert_eq!(cursor.node().text(source.as_bytes()), "2");

    assert!(!cursor.goto_last_child());
    assert_eq!(cursor.node().kind(), "number");
    assert_eq!(cursor.node().text(source.as_bytes()), "2");
}

#[test]
fn cursor_goto_previous_sibling_moves_across_direct_siblings() {
    let source = "1-2";
    let tree = parse_arithmetic(source);
    let mut cursor = tree.root_node().walk();

    assert!(cursor.goto_first_child());
    assert_eq!(cursor.node().text(source.as_bytes()), source);

    assert!(cursor.goto_first_child());
    assert_eq!(cursor.node().kind(), "expression");
    assert_eq!(cursor.field_name(), Some("left"));
    assert_eq!(cursor.node().text(source.as_bytes()), "1");
    assert!(!cursor.goto_previous_sibling());
    assert_eq!(cursor.field_name(), Some("left"));
    assert_eq!(cursor.node().text(source.as_bytes()), "1");

    assert!(cursor.goto_next_sibling());
    assert_eq!(cursor.node().kind(), "-");
    assert_eq!(cursor.field_name(), Some("operator"));
    assert_eq!(cursor.node().text(source.as_bytes()), "-");

    assert!(cursor.goto_previous_sibling());
    assert_eq!(cursor.node().kind(), "expression");
    assert_eq!(cursor.field_name(), Some("left"));
    assert_eq!(cursor.node().text(source.as_bytes()), "1");

    assert!(cursor.goto_next_sibling());
    assert!(cursor.goto_next_sibling());
    assert_eq!(cursor.node().kind(), "expression");
    assert_eq!(cursor.field_name(), Some("right"));
    assert_eq!(cursor.node().text(source.as_bytes()), "2");

    assert!(cursor.goto_previous_sibling());
    assert_eq!(cursor.node().kind(), "-");
    assert_eq!(cursor.field_name(), Some("operator"));
    assert_eq!(cursor.node().text(source.as_bytes()), "-");
}

#[test]
fn cursor_reverse_navigation_preserves_cursor_on_miss() {
    let source = "1-2";
    let tree = parse_arithmetic(source);
    let mut cursor = tree.root_node().walk();

    assert!(!cursor.goto_previous_sibling());
    assert_eq!(cursor.node().kind(), "source_file");
    assert_eq!(cursor.node().text(source.as_bytes()), source);

    assert!(cursor.goto_first_child());
    assert_eq!(cursor.node().text(source.as_bytes()), source);
    assert!(!cursor.goto_previous_sibling());
    assert_eq!(cursor.node().kind(), "expression");
    assert_eq!(cursor.node().text(source.as_bytes()), source);

    assert!(cursor.goto_first_child());
    assert_eq!(cursor.node().text(source.as_bytes()), "1");
    assert!(!cursor.goto_previous_sibling());
    assert_eq!(cursor.node().kind(), "expression");
    assert_eq!(cursor.node().text(source.as_bytes()), "1");
}
