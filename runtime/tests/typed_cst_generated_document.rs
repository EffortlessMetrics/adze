//! Generated typed CST document-helper canaries.

#![cfg(all(test, feature = "pure-rust"))]

use adze::document::SyntaxNode;

#[test]
fn generated_parse_document_helper_feeds_generated_syntax_module() {
    let source = "1+2+3";
    let document = adze_example::typed_ast_contract::grammar::parse_document(source)
        .expect("generated parse_document helper should return an AdzeDocument");

    assert_eq!(document.source_text(), source);
    assert_eq!(
        document.metadata().error_count,
        document.tree().error_count()
    );
    assert_eq!(
        document.tree().has_errors(),
        document.metadata().error_count > 0
    );
    assert_eq!(
        document.diagnostics().is_empty(),
        document.metadata().error_count == 0
    );
    assert_eq!(document.metadata().error_count, 0);

    let syntax = adze_example::typed_ast_contract::grammar::syntax::source_file(&document)
        .expect("generated syntax root should cast from document root");

    assert_eq!(syntax.node_id(), document.tree().root_id());
    assert_eq!(syntax.kind_name(), Some("source_file"));
    assert_eq!(syntax.text(), Some(source));
    assert!(
        syntax.child(0).is_some(),
        "source_file wrapper should expose the parsed expression child"
    );
}

#[test]
fn generated_typed_cst_wrappers_cast_generic_document_nodes() {
    let source = "1+2+3";
    let document = adze_example::typed_ast_contract::grammar::parse_document(source)
        .expect("generated parse_document helper should return an AdzeDocument");
    let root = document.tree().root();

    let syntax = adze_example::typed_ast_contract::grammar::syntax::source_file(&document)
        .expect("generated source_file wrapper should cast the generic root");
    assert_same_node(syntax, root);

    let add_node = find_node(root, "Expr_Add", source)
        .expect("generic CST should contain the root addition expression");
    let add = adze_example::typed_ast_contract::grammar::syntax::ExprAdd::cast(
        &document,
        add_node.node_id(),
    )
    .expect("generated Expr_Add wrapper should cast the matching generic node");
    assert_same_node(add, add_node);

    let number_node =
        find_node(root, "/\\d+/", "1").expect("generic CST should contain a number token");
    let number = adze_example::typed_ast_contract::grammar::syntax::DToken::cast(
        &document,
        number_node.node_id(),
    )
    .expect("generated number token wrapper should cast the matching generic node");
    assert_same_node(number, number_node);
}

fn assert_same_node<'doc>(wrapper: impl SyntaxNode<'doc>, node: adze::document::AdzeNode<'doc>) {
    assert_eq!(wrapper.node_id(), node.node_id());
    assert_eq!(wrapper.kind_name(), node.kind_name());
    assert_eq!(wrapper.byte_range(), Some(node.byte_range()));
    assert_eq!(wrapper.text(), node.utf8_text().ok());
}

fn find_node<'doc>(
    node: adze::document::AdzeNode<'doc>,
    kind: &str,
    text: &str,
) -> Option<adze::document::AdzeNode<'doc>> {
    if node.kind_name() == Some(kind) && node.utf8_text().ok() == Some(text) {
        return Some(node);
    }

    for child_index in 0..node.child_count() {
        let child = node.child(child_index)?;
        if let Some(found) = find_node(child, kind, text) {
            return Some(found);
        }
    }

    None
}
