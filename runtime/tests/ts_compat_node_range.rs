//! Tree-sitter compatibility node range canaries.

#![cfg(all(test, feature = "ts-compat", feature = "pure-rust"))]

use adze::ts_compat::{Parser, Point, Range};

fn assert_range_matches_accessors(node: adze::ts_compat::Node<'_>, expected: Range) {
    let actual = node.range();
    assert_eq!(actual, expected);
    assert_eq!(actual.start_byte, node.start_byte());
    assert_eq!(actual.end_byte, node.end_byte());
    assert_eq!(actual.start_point, node.start_position());
    assert_eq!(actual.end_point, node.end_position());
}

#[test]
fn generated_tree_exposes_node_ranges() {
    let mut parser = Parser::new();
    parser
        .set_language(adze_example::ts_langs::arithmetic())
        .expect("Failed to set language");

    let source = "1-2";
    let tree = parser.parse(source, None).expect("Parse failed");
    let root = tree.root_node();
    let expression = root.child(0).expect("root should expose expression child");
    let left = expression
        .child(0)
        .expect("expression should expose left child");
    let operator = expression
        .child(1)
        .expect("expression should expose operator child");
    let right = expression
        .child(2)
        .expect("expression should expose right child");

    assert_range_matches_accessors(
        root,
        Range {
            start_byte: 0,
            end_byte: 3,
            start_point: Point { row: 0, column: 0 },
            end_point: Point { row: 0, column: 3 },
        },
    );
    assert_range_matches_accessors(
        expression,
        Range {
            start_byte: 0,
            end_byte: 3,
            start_point: Point { row: 0, column: 0 },
            end_point: Point { row: 0, column: 3 },
        },
    );
    assert_range_matches_accessors(
        left,
        Range {
            start_byte: 0,
            end_byte: 1,
            start_point: Point { row: 0, column: 0 },
            end_point: Point { row: 0, column: 1 },
        },
    );
    assert_range_matches_accessors(
        operator,
        Range {
            start_byte: 1,
            end_byte: 2,
            start_point: Point { row: 0, column: 1 },
            end_point: Point { row: 0, column: 2 },
        },
    );
    assert_range_matches_accessors(
        right,
        Range {
            start_byte: 2,
            end_byte: 3,
            start_point: Point { row: 0, column: 2 },
            end_point: Point { row: 0, column: 3 },
        },
    );
}
