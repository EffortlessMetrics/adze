//! Typed CST arithmetic spike canaries.

#![cfg(all(test, feature = "pure-rust", feature = "ts-compat"))]

use adze::{
    adze_ir::RuleId,
    document::{AdzeDocument, AdzeNode, NodeId},
    parser_v4::Parser as CoreParser,
    ts_compat::Language,
};
use std::{ops::Range, sync::Arc};

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

fn parse_fielded_document(source: &str) -> AdzeDocument {
    let lang = fielded_arithmetic_language();
    let mut parser = CoreParser::new(lang.grammar.clone(), lang.table.clone(), lang.name.clone());
    parser
        .parse_document(source)
        .expect("document parse should succeed")
}

trait SyntaxNode<'doc>: Copy {
    fn document(&self) -> &'doc AdzeDocument;
    fn node_id(&self) -> NodeId;

    fn node(&self) -> AdzeNode<'doc> {
        self.document()
            .tree()
            .node(self.node_id())
            .expect("typed CST handle should point at a document node")
    }

    fn byte_range(&self) -> Range<usize> {
        self.node().byte_range()
    }

    fn text(&self) -> &'doc str {
        self.document()
            .source_slice(self.byte_range())
            .expect("typed CST node range should slice document source")
    }

    fn is_error(&self) -> bool {
        self.node().is_error()
    }

    fn is_missing(&self) -> bool {
        self.node().is_missing()
    }

    fn has_error(&self) -> bool {
        self.node().has_error()
    }
}

#[derive(Clone, Copy, Debug)]
struct SourceFile<'doc> {
    document: &'doc AdzeDocument,
    id: NodeId,
}

impl<'doc> SourceFile<'doc> {
    fn cast(document: &'doc AdzeDocument, id: NodeId) -> Option<Self> {
        node_has_kind(document, id, "source_file").then_some(Self { document, id })
    }

    fn expression(&self) -> Option<Expression<'doc>> {
        Expression::cast(self.document, self.node().child(0)?.node_id())
    }
}

impl<'doc> SyntaxNode<'doc> for SourceFile<'doc> {
    fn document(&self) -> &'doc AdzeDocument {
        self.document
    }

    fn node_id(&self) -> NodeId {
        self.id
    }
}

#[derive(Clone, Copy, Debug)]
struct Expression<'doc> {
    document: &'doc AdzeDocument,
    id: NodeId,
}

impl<'doc> Expression<'doc> {
    fn cast(document: &'doc AdzeDocument, id: NodeId) -> Option<Self> {
        node_has_kind(document, id, "expression").then_some(Self { document, id })
    }

    fn left(&self) -> Option<Expression<'doc>> {
        let edge = self.node().edge_by_field_name("left")?;
        Expression::cast(self.document, edge.child_id())
    }

    fn operator(&self) -> Option<MinusToken<'doc>> {
        let edge = self.node().edge_by_field_name("operator")?;
        MinusToken::cast(self.document, edge.child_id())
    }

    fn right(&self) -> Option<Expression<'doc>> {
        let edge = self.node().edge_by_field_name("right")?;
        Expression::cast(self.document, edge.child_id())
    }

    fn number(&self) -> Option<NumberToken<'doc>> {
        NumberToken::cast(self.document, self.node().child(0)?.node_id())
    }
}

impl<'doc> SyntaxNode<'doc> for Expression<'doc> {
    fn document(&self) -> &'doc AdzeDocument {
        self.document
    }

    fn node_id(&self) -> NodeId {
        self.id
    }
}

#[derive(Clone, Copy, Debug)]
struct MinusToken<'doc> {
    document: &'doc AdzeDocument,
    id: NodeId,
}

impl<'doc> MinusToken<'doc> {
    fn cast(document: &'doc AdzeDocument, id: NodeId) -> Option<Self> {
        node_has_kind(document, id, "-").then_some(Self { document, id })
    }
}

impl<'doc> SyntaxNode<'doc> for MinusToken<'doc> {
    fn document(&self) -> &'doc AdzeDocument {
        self.document
    }

    fn node_id(&self) -> NodeId {
        self.id
    }
}

#[derive(Clone, Copy, Debug)]
struct NumberToken<'doc> {
    document: &'doc AdzeDocument,
    id: NodeId,
}

impl<'doc> NumberToken<'doc> {
    fn cast(document: &'doc AdzeDocument, id: NodeId) -> Option<Self> {
        node_has_kind(document, id, "number").then_some(Self { document, id })
    }
}

impl<'doc> SyntaxNode<'doc> for NumberToken<'doc> {
    fn document(&self) -> &'doc AdzeDocument {
        self.document
    }

    fn node_id(&self) -> NodeId {
        self.id
    }
}

fn node_has_kind(document: &AdzeDocument, id: NodeId, expected: &str) -> bool {
    document.tree().node(id).and_then(|node| node.kind_name()) == Some(expected)
}

#[test]
fn typed_cst_wrappers_project_from_document_node_ids_and_edges() {
    let source = "1-2";
    let document = parse_fielded_document(source);
    let tree = document.tree();
    let root = tree.root();
    let expression_node = root.child(0).expect("root should expose expression child");

    let syntax = SourceFile::cast(&document, tree.root_id()).expect("root should cast");
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
    assert_eq!(syntax.text(), source);
    assert_eq!(expression.node_id(), expression_node.node_id());
    assert_eq!(expression.text(), source);

    assert_eq!(
        left.node_id(),
        expression
            .node()
            .edge_by_field_name("left")
            .expect("left edge should exist")
            .child_id()
    );
    assert_eq!(
        operator.node_id(),
        expression
            .node()
            .edge_by_field_name("operator")
            .expect("operator edge should exist")
            .child_id()
    );
    assert_eq!(
        right.node_id(),
        expression
            .node()
            .edge_by_field_name("right")
            .expect("right edge should exist")
            .child_id()
    );

    assert_eq!(left.text(), "1");
    assert_eq!(left.byte_range(), 0..1);
    assert_eq!(
        left.number().expect("left should contain number").text(),
        "1"
    );
    assert_eq!(operator.text(), "-");
    assert_eq!(operator.byte_range(), 1..2);
    assert_eq!(right.text(), "2");
    assert_eq!(right.byte_range(), 2..3);
    assert_eq!(
        right.number().expect("right should contain number").text(),
        "2"
    );

    assert!(SourceFile::cast(&document, expression.node_id()).is_none());
    assert!(MinusToken::cast(&document, left.node_id()).is_none());
    assert!(expression.node().edge_by_field_name("missing").is_none());
}

#[test]
fn typed_cst_wrappers_treat_recovered_mismatches_as_fallible_casts() {
    let document = parse_fielded_document("1-@");
    let root = document.tree().root();

    assert!(!document.diagnostics().is_empty());
    assert!(root.has_error());

    let maybe_syntax = SourceFile::cast(&document, root.node_id());
    if let Some(syntax) = maybe_syntax {
        assert_eq!(syntax.is_error(), syntax.node().is_error());
        assert_eq!(syntax.is_missing(), syntax.node().is_missing());
        assert_eq!(syntax.has_error(), syntax.node().has_error());

        if let Some(expression) = syntax.expression() {
            assert_eq!(expression.is_error(), expression.node().is_error());
            assert_eq!(expression.is_missing(), expression.node().is_missing());
            assert_eq!(expression.has_error(), expression.node().has_error());
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
