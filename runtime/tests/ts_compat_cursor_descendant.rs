//! Tree-sitter compatibility cursor descendant-index canaries.

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
fn cursor_descendant_index_tracks_preorder_from_tree_root() {
    let source = "1-2";
    let tree = parse_arithmetic(source);
    let mut cursor = tree.root_node().walk();

    assert_eq!(cursor.descendant_index(), 0);
    assert_eq!(cursor.node().kind(), "source_file");

    assert!(cursor.goto_first_child());
    assert_eq!(cursor.descendant_index(), 1);
    assert_eq!(cursor.node().text(source.as_bytes()), source);

    assert!(cursor.goto_first_child());
    assert_eq!(cursor.descendant_index(), 2);
    assert_eq!(cursor.node().text(source.as_bytes()), "1");

    assert!(cursor.goto_first_child());
    assert_eq!(cursor.descendant_index(), 3);
    assert_eq!(cursor.node().kind(), "number");
    assert_eq!(cursor.node().text(source.as_bytes()), "1");

    assert!(cursor.goto_parent());
    assert!(cursor.goto_next_sibling());
    assert_eq!(cursor.descendant_index(), 4);
    assert_eq!(cursor.node().kind(), "-");
    assert_eq!(cursor.node().text(source.as_bytes()), "-");

    assert!(cursor.goto_next_sibling());
    assert_eq!(cursor.descendant_index(), 5);
    assert_eq!(cursor.node().text(source.as_bytes()), "2");

    assert!(cursor.goto_first_child());
    assert_eq!(cursor.descendant_index(), 6);
    assert_eq!(cursor.node().kind(), "number");
    assert_eq!(cursor.node().text(source.as_bytes()), "2");
}

#[test]
fn cursor_goto_descendant_moves_by_preorder_index() {
    let source = "1-2";
    let tree = parse_arithmetic(source);
    let mut cursor = tree.root_node().walk();

    cursor.goto_descendant(4);
    assert_eq!(cursor.descendant_index(), 4);
    assert_eq!(cursor.depth(), 2);
    assert_eq!(cursor.node().kind(), "-");
    assert_eq!(cursor.node().text(source.as_bytes()), "-");

    cursor.goto_descendant(6);
    assert_eq!(cursor.descendant_index(), 6);
    assert_eq!(cursor.depth(), 3);
    assert_eq!(cursor.node().kind(), "number");
    assert_eq!(cursor.node().text(source.as_bytes()), "2");

    cursor.goto_descendant(0);
    assert_eq!(cursor.descendant_index(), 0);
    assert_eq!(cursor.depth(), 0);
    assert_eq!(cursor.node().kind(), "source_file");
    assert_eq!(cursor.node().text(source.as_bytes()), source);

    cursor.goto_descendant(99);
    assert_eq!(cursor.descendant_index(), 0);
    assert_eq!(cursor.node().kind(), "source_file");
}

#[test]
fn cursor_descendant_indexes_are_relative_to_walk_root() {
    let source = "1-2";
    let tree = parse_arithmetic(source);
    let root = tree.root_node();
    let expression = root.child(0).expect("root should have expression child");
    let mut cursor = expression.walk();

    assert_eq!(cursor.descendant_index(), 0);
    assert_eq!(cursor.node().text(source.as_bytes()), source);

    cursor.goto_descendant(3);
    assert_eq!(cursor.descendant_index(), 3);
    assert_eq!(cursor.depth(), 1);
    assert_eq!(cursor.node().kind(), "-");
    assert_eq!(cursor.node().text(source.as_bytes()), "-");

    cursor.goto_descendant(5);
    assert_eq!(cursor.descendant_index(), 5);
    assert_eq!(cursor.depth(), 2);
    assert_eq!(cursor.node().kind(), "number");
    assert_eq!(cursor.node().text(source.as_bytes()), "2");
}
