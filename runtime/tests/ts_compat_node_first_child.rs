//! Tree-sitter compatibility node byte child lookup canaries.

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
fn node_first_child_for_byte_returns_matching_direct_child() {
    let source = "1-2";
    let tree = parse_arithmetic(source);
    let root = tree.root_node();
    let expression = root
        .first_child_for_byte(0)
        .expect("root should expose expression child");

    assert_eq!(expression.kind(), "expression");
    assert_eq!(expression.text(source.as_bytes()), source);

    let root_byte_one = root
        .first_child_for_byte(1)
        .expect("root lookup should stay on direct expression child");
    assert_eq!(root_byte_one.kind(), "expression");
    assert_eq!(root_byte_one.text(source.as_bytes()), source);

    let left = expression
        .first_child_for_byte(0)
        .expect("expression should expose left child");
    let operator = expression
        .first_child_for_byte(1)
        .expect("expression should expose operator child");
    let right = expression
        .first_child_for_byte(2)
        .expect("expression should expose right child");

    assert_eq!(left.kind(), "expression");
    assert_eq!(left.text(source.as_bytes()), "1");
    assert_eq!(operator.kind(), "-");
    assert_eq!(operator.text(source.as_bytes()), "-");
    assert_eq!(right.kind(), "expression");
    assert_eq!(right.text(source.as_bytes()), "2");

    assert!(expression.first_child_for_byte(3).is_none());
    assert!(root.first_child_for_byte(99).is_none());
}

#[test]
fn node_first_named_child_for_byte_skips_anonymous_children() {
    let source = "1-2";
    let tree = parse_arithmetic(source);
    let expression = tree
        .root_node()
        .child(0)
        .expect("root should expose expression child");

    let left = expression
        .first_named_child_for_byte(0)
        .expect("expression should expose left named child");
    let right = expression
        .first_named_child_for_byte(1)
        .expect("expression should skip anonymous operator child");
    let right_from_start = expression
        .first_named_child_for_byte(2)
        .expect("expression should expose right named child at its start byte");

    assert_eq!(left.kind(), "expression");
    assert_eq!(left.text(source.as_bytes()), "1");
    assert_eq!(right.kind(), "expression");
    assert_eq!(right.text(source.as_bytes()), "2");
    assert_eq!(right_from_start.kind(), "expression");
    assert_eq!(right_from_start.text(source.as_bytes()), "2");
    assert!(
        !expression
            .first_child_for_byte(1)
            .expect("operator child should exist")
            .is_named()
    );

    assert!(expression.first_named_child_for_byte(3).is_none());
}
