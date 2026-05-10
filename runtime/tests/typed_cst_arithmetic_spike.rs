//! Typed CST arithmetic spike canaries.

#![cfg(all(test, feature = "pure-rust", feature = "ts-compat"))]

#[path = "support/typed_cst_arithmetic.rs"]
mod typed_cst_arithmetic;

use adze::document::SyntaxNode;
use typed_cst_arithmetic::{parse_fielded_document, syntax};

#[test]
fn typed_cst_scaffold_projects_from_document_node_ids_and_edges() {
    let source = "1-2";
    let document = parse_fielded_document(source);
    let tree = document.tree();
    let root = tree.root();
    let expression_node = root.child(0).expect("root should expose expression child");

    let syntax = syntax::source_file(&document).expect("root should cast");
    let expression = syntax
        .expression()
        .expect("source file should expose expression");
    let left = expression
        .left()
        .expect("expression should expose left field");
    let operator = expression
        .operator()
        .expect("expression should expose operator field");
    let right = expression
        .right()
        .expect("expression should expose right field");

    assert_eq!(syntax.node_id(), root.node_id());
    assert_eq!(syntax.text(), Some(source));
    assert_eq!(expression.node_id(), expression_node.node_id());
    assert_eq!(expression.text(), Some(source));

    assert_eq!(
        left.node_id(),
        expression
            .edge_by_field_name("left")
            .expect("left edge should exist")
            .child_id()
    );
    assert_eq!(
        operator.node_id(),
        expression
            .edge_by_field_name("operator")
            .expect("operator edge should exist")
            .child_id()
    );
    assert_eq!(
        right.node_id(),
        expression
            .edge_by_field_name("right")
            .expect("right edge should exist")
            .child_id()
    );

    assert_eq!(left.text(), Some("1"));
    assert_eq!(left.byte_range(), Some(0..1));
    assert_eq!(
        left.number().expect("left should contain number").text(),
        Some("1")
    );
    assert_eq!(operator.text(), Some("-"));
    assert_eq!(operator.byte_range(), Some(1..2));
    assert_eq!(right.text(), Some("2"));
    assert_eq!(right.byte_range(), Some(2..3));
    assert_eq!(
        right.number().expect("right should contain number").text(),
        Some("2")
    );

    assert!(syntax::SourceFile::cast(&document, expression.node_id()).is_none());
    assert!(syntax::MinusToken::cast(&document, left.node_id()).is_none());
    assert!(expression.edge_by_field_name("missing").is_none());
}

#[test]
fn typed_cst_wrappers_treat_recovered_mismatches_as_fallible_casts() {
    let document = parse_fielded_document("1-@");
    let root = document.tree().root();

    assert!(!document.diagnostics().is_empty());
    assert!(root.has_error());

    let maybe_syntax = syntax::source_file(&document);
    if let Some(syntax) = maybe_syntax {
        let syntax_node = syntax.node().expect("source-file handle should resolve");
        assert_eq!(syntax.is_error(), syntax_node.is_error());
        assert_eq!(syntax.is_missing(), syntax_node.is_missing());
        assert_eq!(syntax.has_error(), syntax_node.has_error());

        if let Some(expression) = syntax.expression() {
            let expression_node = expression.node().expect("expression handle should resolve");
            assert_eq!(expression.is_error(), expression_node.is_error());
            assert_eq!(expression.is_missing(), expression_node.is_missing());
            assert_eq!(expression.has_error(), expression_node.has_error());
            let _ = expression.left();
            let _ = expression.operator();
            let _ = expression.right();
        }
    } else {
        assert!(
            root.is_error() || root.kind_name() != Some("source_file"),
            "failed source-file casts should reflect a true syntax-kind mismatch"
        );
    }
}
