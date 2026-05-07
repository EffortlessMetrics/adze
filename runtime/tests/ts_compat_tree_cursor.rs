//! Tree-sitter compatibility cursor traversal canaries.

#![cfg(all(test, feature = "ts-compat", feature = "pure-rust"))]

use adze::{adze_ir::RuleId, ts_compat::Parser};
use std::sync::Arc;

#[test]
fn tree_cursor_walks_generated_arithmetic_tree() {
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
    let mut cursor = tree.root_node().walk();

    assert_eq!(cursor.node().kind(), "source_file");
    assert!(!cursor.goto_next_sibling());

    assert!(cursor.goto_first_child());
    assert_eq!(cursor.node().kind(), "expression");
    assert_eq!(cursor.field_name(), None);

    assert!(cursor.goto_first_child());
    assert_eq!(cursor.field_name(), Some("left"));
    assert_eq!(cursor.node().text(source.as_bytes()), "1");
    assert!(cursor.goto_first_child());
    assert_eq!(cursor.node().text(source.as_bytes()), "1");
    assert!(!cursor.goto_first_child());
    assert_eq!(cursor.node().text(source.as_bytes()), "1");
    assert!(cursor.goto_parent());

    assert!(cursor.goto_next_sibling());
    assert_eq!(cursor.field_name(), Some("operator"));
    assert_eq!(cursor.node().text(source.as_bytes()), "-");

    assert!(cursor.goto_next_sibling());
    assert_eq!(cursor.field_name(), Some("right"));
    assert_eq!(cursor.node().text(source.as_bytes()), "2");
    assert!(!cursor.goto_next_sibling());
    assert_eq!(cursor.node().text(source.as_bytes()), "2");

    assert!(cursor.goto_parent());
    assert_eq!(cursor.node().kind(), "expression");
    assert_eq!(cursor.node().text(source.as_bytes()), source);

    assert!(cursor.goto_parent());
    assert_eq!(cursor.node().kind(), "source_file");
    assert!(!cursor.goto_parent());
}
