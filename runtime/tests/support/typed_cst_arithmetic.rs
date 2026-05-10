use adze::{
    adze_ir::RuleId,
    document::{AdzeDocument, NodeId, SyntaxNode},
    parser_v4::Parser as CoreParser,
    ts_compat::Language,
};
use std::sync::Arc;

pub fn parse_fielded_document(source: &str) -> AdzeDocument {
    let lang = fielded_arithmetic_language();
    let mut parser = CoreParser::new(lang.grammar.clone(), lang.table.clone(), lang.name.clone());
    parser
        .parse_document(source)
        .expect("document parse should succeed")
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

/// Generated-style typed CST wrappers for the arithmetic fixture.
///
/// This module is test support, not a public generated API. It keeps the
/// wrapper shape separate from canary assertions so future generator work has
/// a concrete target to replace.
pub mod syntax {
    use super::*;

    pub fn source_file(document: &AdzeDocument) -> Option<SourceFile<'_>> {
        SourceFile::cast(document, document.tree().root_id())
    }

    #[derive(Clone, Copy, Debug)]
    pub struct SourceFile<'doc> {
        document: &'doc AdzeDocument,
        id: NodeId,
    }

    impl<'doc> SourceFile<'doc> {
        pub fn cast(document: &'doc AdzeDocument, id: NodeId) -> Option<Self> {
            node_has_kind(document, id, "source_file").then_some(Self { document, id })
        }

        pub fn expression(&self) -> Option<Expression<'doc>> {
            Expression::cast(self.document, self.child(0)?.node_id())
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
    pub struct Expression<'doc> {
        document: &'doc AdzeDocument,
        id: NodeId,
    }

    impl<'doc> Expression<'doc> {
        pub fn cast(document: &'doc AdzeDocument, id: NodeId) -> Option<Self> {
            node_has_kind(document, id, "expression").then_some(Self { document, id })
        }

        pub fn left(&self) -> Option<Expression<'doc>> {
            let edge = self.edge_by_field_name("left")?;
            Expression::cast(self.document, edge.child_id())
        }

        pub fn operator(&self) -> Option<MinusToken<'doc>> {
            let edge = self.edge_by_field_name("operator")?;
            MinusToken::cast(self.document, edge.child_id())
        }

        pub fn right(&self) -> Option<Expression<'doc>> {
            let edge = self.edge_by_field_name("right")?;
            Expression::cast(self.document, edge.child_id())
        }

        pub fn number(&self) -> Option<NumberToken<'doc>> {
            NumberToken::cast(self.document, self.child(0)?.node_id())
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
    pub struct MinusToken<'doc> {
        document: &'doc AdzeDocument,
        id: NodeId,
    }

    impl<'doc> MinusToken<'doc> {
        pub fn cast(document: &'doc AdzeDocument, id: NodeId) -> Option<Self> {
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
    pub struct NumberToken<'doc> {
        document: &'doc AdzeDocument,
        id: NodeId,
    }

    impl<'doc> NumberToken<'doc> {
        pub fn cast(document: &'doc AdzeDocument, id: NodeId) -> Option<Self> {
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
}
