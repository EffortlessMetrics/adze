//! Tree-sitter compatibility node error-state guardrail canaries.

#![cfg(all(test, feature = "ts-compat", feature = "pure-rust"))]

use adze::ts_compat::{Node, Parser};

fn assert_node_error_free(node: Node<'_>) {
    assert!(
        !node.is_error(),
        "{} should not be reported as an error node",
        node.kind()
    );
    assert!(
        !node.is_missing(),
        "{} should not be reported as a missing node",
        node.kind()
    );
    assert!(
        !node.has_error(),
        "{} should not report descendant errors",
        node.kind()
    );

    for index in 0..node.child_count() {
        let child = node.child(index).expect("child index should be valid");
        assert_node_error_free(child);
    }
}

#[test]
fn generated_tree_reports_error_metadata_when_parser_recovers() {
    let mut parser = Parser::new();
    parser
        .set_language(adze_example::ts_langs::arithmetic())
        .expect("Failed to set language");

    let source = "1-@";
    let tree = parser
        .parse(source, None)
        .expect("parser should return an inspectable error tree");
    let root = tree.root_node();

    assert!(
        tree.error_count() > 0,
        "parser recovery errors should be preserved on the compatibility tree"
    );
    assert!(tree.has_errors());
    assert!(
        !root.is_error(),
        "partial roots with parser recoveries should report has_error, not is_error"
    );
    assert!(root.has_error());
    assert!(!root.is_missing());
    assert!(root.start_byte() <= root.end_byte());
    assert!(root.end_byte() <= source.len());
}

#[test]
fn generated_tree_reports_error_free_node_metadata_for_valid_input() {
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

    assert_eq!(tree.error_count(), 0);
    assert!(!tree.has_errors());

    assert_node_error_free(root);
    assert_node_error_free(expression);
    assert_node_error_free(left);
    assert_node_error_free(operator);
    assert_node_error_free(right);
}
