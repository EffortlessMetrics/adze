//! Tree-sitter compatibility node descendant-count canaries.

#![cfg(all(test, feature = "ts-compat", feature = "pure-rust"))]

use adze::ts_compat::Parser;

#[test]
fn generated_tree_exposes_descendant_counts_including_self() {
    let mut parser = Parser::new();
    parser
        .set_language(adze_example::ts_langs::arithmetic())
        .expect("Failed to set language");

    let tree = parser.parse("1-2", None).expect("Parse failed");
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
    let left_number = left.child(0).expect("left expression should expose number");
    let right_number = right
        .child(0)
        .expect("right expression should expose number");

    assert_eq!(left_number.kind(), "number");
    assert_eq!(right_number.kind(), "number");

    assert_eq!(left_number.descendant_count(), 1);
    assert_eq!(right_number.descendant_count(), 1);
    assert_eq!(operator.descendant_count(), 1);
    assert_eq!(left.descendant_count(), 2);
    assert_eq!(right.descendant_count(), 2);
    assert_eq!(expression.descendant_count(), 6);
    assert_eq!(root.descendant_count(), 7);
}
