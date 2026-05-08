//! Tree-sitter compatibility point-range descendant canaries.

#![cfg(all(test, feature = "ts-compat", feature = "pure-rust"))]

use adze::ts_compat::{Parser, Point};

fn point(row: u32, column: u32) -> Point {
    Point { row, column }
}

#[test]
fn descendant_for_point_range_returns_smallest_covering_node() {
    let mut parser = Parser::new();
    parser
        .set_language(adze_example::ts_langs::arithmetic())
        .expect("Failed to set language");

    let source = "1-2";
    let tree = parser.parse(source, None).expect("Parse failed");
    let root = tree.root_node();

    let left_number = root
        .descendant_for_point_range(point(0, 0), point(0, 1))
        .expect("left number point range should resolve");
    assert_eq!(left_number.kind(), "number");
    assert_eq!(left_number.text(source.as_bytes()), "1");

    let operator = root
        .descendant_for_point_range(point(0, 1), point(0, 2))
        .expect("operator point range should resolve");
    assert_eq!(operator.kind(), "-");
    assert_eq!(operator.text(source.as_bytes()), "-");

    let binary_expression = root
        .descendant_for_point_range(point(0, 0), point(0, 2))
        .expect("range spanning left and operator should resolve to expression");
    assert_eq!(binary_expression.kind(), "expression");
    assert_eq!(binary_expression.text(source.as_bytes()), source);
}

#[test]
fn named_descendant_for_point_range_skips_anonymous_nodes() {
    let mut parser = Parser::new();
    parser
        .set_language(adze_example::ts_langs::arithmetic())
        .expect("Failed to set language");

    let source = "1-2";
    let tree = parser.parse(source, None).expect("Parse failed");
    let root = tree.root_node();

    let left_expression = root
        .named_descendant_for_point_range(point(0, 0), point(0, 1))
        .expect("left number point range should resolve to named expression");
    assert_eq!(left_expression.kind(), "expression");
    assert_eq!(left_expression.text(source.as_bytes()), "1");

    let operator_parent = root
        .named_descendant_for_point_range(point(0, 1), point(0, 2))
        .expect("anonymous operator point range should resolve to nearest named parent");
    assert_eq!(operator_parent.kind(), "expression");
    assert_eq!(operator_parent.text(source.as_bytes()), source);

    let binary_expression = root
        .named_descendant_for_point_range(point(0, 0), point(0, 3))
        .expect("full expression point range should resolve to expression");
    assert_eq!(binary_expression.kind(), "expression");
    assert_eq!(binary_expression.text(source.as_bytes()), source);
}

#[test]
fn descendant_for_point_range_rejects_invalid_ranges() {
    let mut parser = Parser::new();
    parser
        .set_language(adze_example::ts_langs::arithmetic())
        .expect("Failed to set language");

    let tree = parser.parse("1-2", None).expect("Parse failed");
    let root = tree.root_node();
    let left_expression = root
        .descendant_for_point_range(point(0, 0), point(0, 1))
        .expect("left point range should resolve");

    assert!(
        root.descendant_for_point_range(point(0, 3), point(0, 4))
            .is_none()
    );
    assert!(
        root.descendant_for_point_range(point(1, 0), point(1, 1))
            .is_none()
    );
    assert!(
        root.descendant_for_point_range(point(0, 2), point(0, 1))
            .is_none()
    );
    assert!(
        root.named_descendant_for_point_range(point(0, 3), point(0, 4))
            .is_none()
    );
    assert!(
        root.named_descendant_for_point_range(point(1, 0), point(1, 1))
            .is_none()
    );
    assert!(
        root.named_descendant_for_point_range(point(0, 2), point(0, 1))
            .is_none()
    );
    assert!(
        left_expression
            .descendant_for_point_range(point(0, 1), point(0, 2))
            .is_none()
    );
    assert!(
        left_expression
            .named_descendant_for_point_range(point(0, 1), point(0, 2))
            .is_none()
    );
}
