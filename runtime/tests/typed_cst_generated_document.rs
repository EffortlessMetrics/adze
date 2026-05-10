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
