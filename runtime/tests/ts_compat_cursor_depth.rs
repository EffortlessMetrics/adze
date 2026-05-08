//! Tree-sitter compatibility cursor depth canaries.

#![cfg(all(test, feature = "ts-compat", feature = "pure-rust"))]

use adze::ts_compat::{Parser, Tree};

fn parse_arithmetic(source: &str) -> Tree {
    let mut parser = Parser::new();
    parser
        .set_language(adze_example::ts_langs::arithmetic())
        .expect("Failed to set language");
    parser.parse(source, None).expect("Parse failed")
}

#[test]
fn cursor_depth_tracks_forward_and_reverse_navigation() {
    let source = "1-2";
    let tree = parse_arithmetic(source);
    let mut cursor = tree.root_node().walk();

    assert_eq!(cursor.depth(), 0);
    assert_eq!(cursor.node().kind(), "source_file");

    assert!(cursor.goto_first_child());
    assert_eq!(cursor.depth(), 1);
    assert_eq!(cursor.node().text(source.as_bytes()), source);

    assert!(cursor.goto_first_child());
    assert_eq!(cursor.depth(), 2);
    assert_eq!(cursor.node().text(source.as_bytes()), "1");

    assert!(cursor.goto_next_sibling());
    assert_eq!(cursor.depth(), 2);
    assert_eq!(cursor.node().text(source.as_bytes()), "-");

    assert!(cursor.goto_next_sibling());
    assert_eq!(cursor.depth(), 2);
    assert_eq!(cursor.node().text(source.as_bytes()), "2");

    assert!(cursor.goto_last_child());
    assert_eq!(cursor.depth(), 3);
    assert_eq!(cursor.node().kind(), "number");
    assert_eq!(cursor.node().text(source.as_bytes()), "2");

    assert!(cursor.goto_parent());
    assert_eq!(cursor.depth(), 2);
    assert_eq!(cursor.node().text(source.as_bytes()), "2");

    assert!(cursor.goto_previous_sibling());
    assert_eq!(cursor.depth(), 2);
    assert_eq!(cursor.node().text(source.as_bytes()), "-");

    assert!(cursor.goto_parent());
    assert_eq!(cursor.depth(), 1);
    assert_eq!(cursor.node().text(source.as_bytes()), source);

    assert!(cursor.goto_parent());
    assert_eq!(cursor.depth(), 0);
    assert_eq!(cursor.node().kind(), "source_file");
}

#[test]
fn cursor_depth_is_relative_to_walk_root() {
    let source = "1-2";
    let tree = parse_arithmetic(source);
    let root = tree.root_node();
    let expression = root.child(0).expect("root should have an expression child");
    let mut cursor = expression.walk();

    assert_eq!(cursor.depth(), 0);
    assert_eq!(cursor.node().kind(), "expression");
    assert_eq!(cursor.node().text(source.as_bytes()), source);

    assert!(cursor.goto_first_child());
    assert_eq!(cursor.depth(), 1);
    assert_eq!(cursor.node().text(source.as_bytes()), "1");

    assert!(cursor.goto_first_child());
    assert_eq!(cursor.depth(), 2);
    assert_eq!(cursor.node().kind(), "number");
    assert_eq!(cursor.node().text(source.as_bytes()), "1");
}
