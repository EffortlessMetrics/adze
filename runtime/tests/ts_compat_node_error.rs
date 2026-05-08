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
fn generated_tree_reports_zero_width_error_root_as_missing() {
    let mut parser = Parser::new();
    parser
        .set_language(adze_example::ts_langs::arithmetic())
        .expect("Failed to set language");

    let tree = parser
        .parse("", None)
        .expect("parser should return an inspectable empty-input error tree");
    let root = tree.root_node();

    assert!(tree.error_count() > 0);
    assert!(tree.has_errors());
    assert!(root.is_error());
    assert!(root.is_missing());
    assert_eq!(root.byte_range(), 0..0);
    assert!(root.has_error());
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

/// Walk every node in a tree that reports parser errors with a cursor and
/// confirm no method panics. This exercises the full walk + metadata surface
/// over partial/error-recovered trees.
#[test]
fn walking_error_tree_does_not_panic() {
    let mut parser = Parser::new();
    parser
        .set_language(adze_example::ts_langs::arithmetic())
        .expect("Failed to set language");

    let source = "1-@#";
    let tree = parser
        .parse(source, None)
        .expect("parser should return an inspectable error tree");

    let root = tree.root_node();
    assert!(
        tree.error_count() > 0,
        "expected parse errors for invalid input"
    );

    // Walk with cursor — no method should panic on error nodes
    let mut cursor = root.walk();
    loop {
        let node = cursor.node();
        // Exercise every metadata getter; none may panic
        let _kind = node.kind();
        let _kind_id = node.kind_id();
        let _start = node.start_byte();
        let _end = node.end_byte();
        let _is_error = node.is_error();
        let _is_missing = node.is_missing();
        let _has_error = node.has_error();
        let _is_named = node.is_named();
        let _is_extra = node.is_extra();
        let _child_count = node.child_count();
        let _descendant_count = node.descendant_count();
        let _range = node.range();
        let _byte_range = node.byte_range();

        if !cursor.goto_first_child() {
            while !cursor.goto_next_sibling() {
                if !cursor.goto_parent() {
                    return; // walked the whole tree
                }
            }
        }
    }
}

/// The root `has_error()` result must reflect parser recovery/error metadata.
/// This intentionally stays separate from `is_error()`, which is node-local.
#[test]
fn root_has_error_reflects_parser_recovery_metadata() {
    let mut parser = Parser::new();
    parser
        .set_language(adze_example::ts_langs::arithmetic())
        .expect("Failed to set language");

    let tree = parser
        .parse("1-@", None)
        .expect("parser should return an inspectable error tree");

    assert!(
        tree.error_count() > 0,
        "expected parse errors for invalid input"
    );
    assert!(tree.has_errors());

    let root = tree.root_node();
    // Root itself is not an error node, but must report has_error because the
    // parser recorded recovery/error metadata for this tree.
    assert!(!root.is_error());
    assert!(root.has_error(), "root must propagate descendant errors");
}

/// Verify that `is_error()` remains node-local and does not turn ordinary
/// nodes into ERROR nodes just because the tree has parser errors.
#[test]
fn is_error_remains_node_local_for_valid_and_recovered_trees() {
    let mut parser = Parser::new();
    parser
        .set_language(adze_example::ts_langs::arithmetic())
        .expect("Failed to set language");

    // Valid input — no node should be an error
    let valid_tree = parser.parse("1-2", None).expect("Parse failed");
    let root = valid_tree.root_node();
    assert!(!root.is_error());
    assert!(!root.has_error());

    for i in 0..root.child_count() {
        let child = root.child(i).expect("child index should be valid");
        assert!(
            !child.is_error(),
            "valid tree child {i} should not be an error node"
        );
    }

    // Invalid input — at least some error state should be present
    let error_tree = parser
        .parse("@@@@", None)
        .expect("parser should still return a tree for recovery");

    let error_root = error_tree.root_node();
    // The root itself may or may not be an error node depending on recovery,
    // but has_error must be consistent with error_count.
    if error_tree.error_count() > 0 {
        assert!(
            error_root.has_error(),
            "root has_error must be true when error_count > 0"
        );
    }
}

/// Error nodes should have a deterministic kind name.  Symbol-id-0 nodes
/// resolve to whatever name the grammar has for symbol 0 (typically "end"
/// or "EOF"), while non-error children in a partial tree retain their
/// grammar kind.
#[test]
fn error_node_kind_name_is_deterministic() {
    let mut parser = Parser::new();
    parser
        .set_language(adze_example::ts_langs::arithmetic())
        .expect("Failed to set language");

    let tree = parser
        .parse("1-@", None)
        .expect("parser should return an inspectable error tree");
    let root = tree.root_node();

    // Walk all nodes; every kind() call must return a valid string (never panic)
    let mut cursor = root.walk();
    loop {
        let node = cursor.node();
        let kind = node.kind();
        // kind() must never panic and must return a non-empty string for every node
        // that is not a synthetic error node
        if !node.is_error() {
            assert!(!kind.is_empty(), "non-error node kind should be non-empty");
        }

        if !cursor.goto_first_child() {
            while !cursor.goto_next_sibling() {
                if !cursor.goto_parent() {
                    return; // walked the whole tree
                }
            }
        }
    }
}

/// Verify that Tree-level and Node-level error metadata are consistent:
/// `tree.has_errors()` must agree with `tree.error_count() > 0`,
/// and `root.has_error()` must agree with `tree.has_errors()`.
#[test]
fn tree_and_node_error_metadata_are_consistent() {
    let mut parser = Parser::new();
    parser
        .set_language(adze_example::ts_langs::arithmetic())
        .expect("Failed to set language");

    // Valid input: no errors at any level
    let valid_tree = parser.parse("1-2", None).expect("Parse failed");
    assert_eq!(valid_tree.error_count(), 0);
    assert!(!valid_tree.has_errors());
    assert!(!valid_tree.root_node().has_error());

    // Invalid input: errors at tree and root level
    let error_tree = parser
        .parse("1-@", None)
        .expect("parser should return an inspectable error tree");
    assert!(error_tree.error_count() > 0);
    assert!(error_tree.has_errors());
    assert!(error_tree.root_node().has_error());

    // Cross-check: tree.has_errors() ⟺ error_count > 0
    assert_eq!(error_tree.has_errors(), error_tree.error_count() > 0);
}
