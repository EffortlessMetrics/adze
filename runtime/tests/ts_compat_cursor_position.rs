//! Tree-sitter compatibility cursor byte/point child lookup canaries.

#![cfg(all(test, feature = "ts-compat", feature = "pure-rust"))]

use adze::ts_compat::{Parser, Point, Tree};

fn point(row: u32, column: u32) -> Point {
    Point { row, column }
}

fn parse_arithmetic(source: &str) -> Tree {
    let mut parser = Parser::new();
    parser
        .set_language(adze_example::ts_langs::arithmetic())
        .expect("Failed to set language");
    parser.parse(source, None).expect("Parse failed")
}

#[test]
fn cursor_goto_first_child_for_byte_moves_to_matching_direct_child() {
    let source = "1-2";
    let tree = parse_arithmetic(source);
    let mut cursor = tree.root_node().walk();

    assert_eq!(cursor.goto_first_child_for_byte(0), Some(0));
    assert_eq!(cursor.node().kind(), "expression");
    assert_eq!(cursor.node().text(source.as_bytes()), source);

    assert_eq!(cursor.goto_first_child_for_byte(0), Some(0));
    assert_eq!(cursor.node().kind(), "expression");
    assert_eq!(cursor.node().text(source.as_bytes()), "1");
    assert!(cursor.goto_parent());

    assert_eq!(cursor.goto_first_child_for_byte(1), Some(1));
    assert_eq!(cursor.node().kind(), "-");
    assert_eq!(cursor.node().text(source.as_bytes()), "-");
    assert!(cursor.goto_parent());

    assert_eq!(cursor.goto_first_child_for_byte(2), Some(2));
    assert_eq!(cursor.node().kind(), "expression");
    assert_eq!(cursor.node().text(source.as_bytes()), "2");
}

#[test]
fn cursor_goto_first_child_for_point_moves_to_matching_direct_child() {
    let source = "1-2";
    let tree = parse_arithmetic(source);
    let mut cursor = tree.root_node().walk();

    assert_eq!(cursor.goto_first_child_for_point(point(0, 0)), Some(0));
    assert_eq!(cursor.node().kind(), "expression");
    assert_eq!(cursor.node().text(source.as_bytes()), source);

    assert_eq!(cursor.goto_first_child_for_point(point(0, 0)), Some(0));
    assert_eq!(cursor.node().kind(), "expression");
    assert_eq!(cursor.node().text(source.as_bytes()), "1");
    assert!(cursor.goto_parent());

    assert_eq!(cursor.goto_first_child_for_point(point(0, 1)), Some(1));
    assert_eq!(cursor.node().kind(), "-");
    assert_eq!(cursor.node().text(source.as_bytes()), "-");
    assert!(cursor.goto_parent());

    assert_eq!(cursor.goto_first_child_for_point(point(0, 2)), Some(2));
    assert_eq!(cursor.node().kind(), "expression");
    assert_eq!(cursor.node().text(source.as_bytes()), "2");
}

#[test]
fn cursor_position_lookup_returns_none_and_preserves_cursor_when_no_child_matches() {
    let source = "1-2";
    let tree = parse_arithmetic(source);
    let mut cursor = tree.root_node().walk();

    assert_eq!(cursor.goto_first_child_for_byte(0), Some(0));
    assert_eq!(cursor.node().text(source.as_bytes()), source);
    assert_eq!(cursor.goto_first_child_for_byte(3), None);
    assert_eq!(cursor.node().kind(), "expression");
    assert_eq!(cursor.node().text(source.as_bytes()), source);

    assert_eq!(cursor.goto_first_child_for_point(point(0, 3)), None);
    assert_eq!(cursor.node().kind(), "expression");
    assert_eq!(cursor.node().text(source.as_bytes()), source);

    assert!(cursor.goto_parent());
    assert_eq!(cursor.node().kind(), "source_file");
    assert_eq!(cursor.goto_first_child_for_byte(99), None);
    assert_eq!(cursor.node().kind(), "source_file");
    assert_eq!(cursor.goto_first_child_for_point(point(1, 0)), None);
    assert_eq!(cursor.node().kind(), "source_file");
}
