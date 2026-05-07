//! Tree-sitter compatibility child traversal canaries.

#![cfg(all(test, feature = "ts-compat", feature = "pure-rust"))]

use adze::ts_compat::Parser;

#[test]
fn generated_tree_exposes_nested_children() {
    let mut parser = Parser::new();
    let lang = adze_example::ts_langs::arithmetic();

    parser.set_language(lang).expect("Failed to set language");

    let source = "1-2";
    let tree = parser.parse(source, None).expect("Parse failed");
    let root = tree.root_node();

    assert_eq!(root.kind(), "source_file");
    assert_eq!(root.child_count(), 1);

    let expression = root.child(0).expect("root should expose expression child");
    assert_eq!(expression.kind(), "expression");
    assert_eq!(expression.child_count(), 3);
    assert_eq!(expression.text(source.as_bytes()), source);

    let left = expression
        .child(0)
        .expect("expression should expose left child");
    let operator = expression
        .child(1)
        .expect("expression should expose operator child");
    let right = expression
        .child(2)
        .expect("expression should expose right child");

    assert_eq!(left.kind(), "expression");
    assert_eq!(left.text(source.as_bytes()), "1");
    assert_eq!(operator.kind(), "-");
    assert_eq!(operator.text(source.as_bytes()), "-");
    assert_eq!(right.kind(), "expression");
    assert_eq!(right.text(source.as_bytes()), "2");
    assert!(expression.child(3).is_none());
}
