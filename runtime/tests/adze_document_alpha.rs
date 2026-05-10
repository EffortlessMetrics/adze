//! Native parse document alpha canaries.

#![cfg(all(test, feature = "pure-rust", feature = "ts-compat"))]

use adze::{
    adze_ir::{RuleId, SymbolId},
    document::NodeId,
    parser_v4::Parser as CoreParser,
    ts_compat::{Language, Tree},
};
use std::sync::Arc;

fn symbol_named(lang: &Language, name: &str) -> SymbolId {
    let index = lang
        .table
        .symbol_metadata
        .iter()
        .position(|metadata| metadata.name == name)
        .unwrap_or_else(|| panic!("arithmetic fixture should expose '{name}' symbol metadata"));
    lang.table.symbol_metadata[index].symbol_id
}

fn fielded_arithmetic_language() -> Arc<Language> {
    let mut lang = (*adze_example::ts_langs::arithmetic()).clone();
    lang.table.field_names = vec![
        "left".to_string(),
        "operator".to_string(),
        "right".to_string(),
    ];
    lang.table.field_map.insert((RuleId(2), 0), 0);
    lang.table.field_map.insert((RuleId(2), 1), 1);
    lang.table.field_map.insert((RuleId(2), 2), 2);
    Arc::new(lang)
}

#[test]
fn parse_document_exposes_generic_tree_and_ts_projection_from_same_parse() {
    let lang = fielded_arithmetic_language();
    let mut parser = CoreParser::new(lang.grammar.clone(), lang.table.clone(), lang.name.clone());

    let source = "1-2";
    let document = parser
        .parse_document(source)
        .expect("document parse should succeed");

    assert_eq!(document.source_text(), source);
    assert_eq!(document.source_bytes(), source.as_bytes());
    assert_eq!(document.language().name(), lang.name.as_str());
    assert_eq!(document.metadata().error_count, 0);
    assert!(document.diagnostics().is_empty());

    let tree = document.tree();
    assert_eq!(tree.language().name(), lang.name.as_str());
    assert!(!tree.has_errors());
    assert_eq!(tree.error_count(), 0);
    assert!(
        tree.node_count() >= 5,
        "tree should index root, expression, and fielded arithmetic children"
    );

    let root = tree.root();
    assert_eq!(tree.root_id(), root.node_id());
    assert_eq!(
        tree.node(root.node_id())
            .expect("root id should resolve")
            .kind_name(),
        root.kind_name()
    );
    assert!(tree.node(NodeId::new(tree.node_count())).is_none());
    assert_eq!(root.kind_id(), symbol_named(&lang, "source_file"));
    assert_eq!(root.kind_name(), Some("source_file"));
    assert_eq!(root.grammar_name(), Some("source_file"));
    assert!(root.is_named());
    assert!(root.is_visible());
    assert!(!root.is_terminal());
    assert!(!root.is_extra());
    assert_eq!(root.symbol_id(), symbol_named(&lang, "source_file"));
    assert_eq!(root.child_count(), 1);
    assert_eq!(root.utf8_text().expect("root text should be UTF-8"), source);

    let expression = root.child(0).expect("root should expose expression child");
    assert_eq!(
        tree.node(expression.node_id())
            .expect("expression id should resolve")
            .byte_range(),
        expression.byte_range()
    );
    assert_eq!(expression.kind_name(), Some("expression"));
    assert_eq!(expression.grammar_name(), Some("expression"));
    assert!(expression.is_named());
    assert!(expression.is_visible());
    assert!(!expression.is_terminal());
    assert_eq!(expression.symbol_id(), symbol_named(&lang, "expression"));
    assert_eq!(expression.child_count(), 3);
    assert_eq!(expression.field_name_for_child(0), Some("left"));
    assert_eq!(expression.field_name_for_child(1), Some("operator"));
    assert_eq!(expression.field_name_for_child(2), Some("right"));

    let left = expression.child(0).expect("left child should exist");
    let operator = expression.child(1).expect("operator child should exist");
    let right = expression.child(2).expect("right child should exist");

    assert_ne!(left.node_id(), operator.node_id());
    assert_ne!(operator.node_id(), right.node_id());
    assert_eq!(
        tree.node(left.node_id())
            .expect("left id should resolve")
            .utf8_text()
            .expect("left text should be UTF-8"),
        "1"
    );
    assert_eq!(left.field_name(), Some("left"));
    assert_eq!(left.utf8_text().expect("left text should be UTF-8"), "1");
    assert_eq!(operator.field_name(), Some("operator"));
    assert_eq!(
        operator.utf8_text().expect("operator text should be UTF-8"),
        "-"
    );
    assert_eq!(right.field_name(), Some("right"));
    assert_eq!(right.utf8_text().expect("right text should be UTF-8"), "2");

    let ts_tree = Tree::from_document(Arc::clone(&lang), &document);
    let ts_expression = ts_tree
        .root_node()
        .child(0)
        .expect("Tree-sitter projection should expose expression child");

    assert_eq!(ts_tree.error_count(), document.metadata().error_count);
    assert_eq!(ts_expression.kind(), expression.kind_name().unwrap());
    assert_eq!(
        ts_expression.grammar_name(),
        expression.grammar_name().unwrap()
    );
    assert_eq!(ts_expression.field_name_for_child(0), Some("left"));
    assert_eq!(ts_expression.field_name_for_child(1), Some("operator"));
    assert_eq!(ts_expression.field_name_for_child(2), Some("right"));
    assert_eq!(
        ts_expression
            .child_by_field_name("left")
            .expect("left field should project")
            .text(source.as_bytes()),
        "1"
    );
}

#[test]
fn parse_document_exposes_recovery_metadata_and_diagnostics() {
    let lang = adze_example::ts_langs::arithmetic();
    let mut parser = CoreParser::new(lang.grammar.clone(), lang.table.clone(), lang.name.clone());

    let source = "1-@";
    let document = parser
        .parse_document(source)
        .expect("document parse should return partial parse facts");

    assert!(document.metadata().error_count > 0);
    assert!(document.tree().has_errors());
    assert!(document.tree().root().has_error());
    assert!(!document.diagnostics().is_empty());

    let diagnostic = &document.diagnostics()[0];
    assert!(diagnostic.start_byte <= diagnostic.end_byte);
    assert!(diagnostic.end_byte <= document.source_bytes().len());
    assert!(
        diagnostic.message.contains("parser recorded"),
        "diagnostic should explain the recorded parser recovery count"
    );

    let ts_tree = Tree::from_document(lang, &document);
    assert_eq!(ts_tree.error_count(), document.metadata().error_count);
    assert!(ts_tree.has_errors());
    assert_eq!(
        ts_tree.root_node().has_error(),
        document.tree().root().has_error()
    );
}
