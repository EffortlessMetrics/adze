//! Native parse document alpha.
//!
//! This module is the first implementation slice of the `AdzeDocument`
//! contract. It intentionally exposes only the current parser tree, source
//! text, basic diagnostics, and metadata. Richer projections such as typed CST,
//! typed AST provenance, and GLR forest summaries are future slices.

use crate::parser_v4::ParseNode;
use adze_ir::SymbolId;
use std::ops::Range;

/// A native parse-product document.
///
/// `AdzeDocument` owns the source text and the parser's selected concrete
/// syntax tree. Additional views should project from this document instead of
/// reparsing or constructing a separate parse truth.
#[derive(Clone, Debug)]
pub struct AdzeDocument {
    source: String,
    root: ParseNode,
    diagnostics: Vec<ParseDiagnostic>,
    metadata: ParseMetadata,
}

impl AdzeDocument {
    pub(crate) fn from_parse_result(source: &str, root: ParseNode, error_count: usize) -> Self {
        let diagnostics = build_diagnostics(&root, error_count, source.len());
        Self {
            source: source.to_string(),
            root,
            diagnostics,
            metadata: ParseMetadata { error_count },
        }
    }

    /// Return the generic native CST view.
    pub fn tree(&self) -> AdzeTree<'_> {
        AdzeTree { document: self }
    }

    /// Return structured diagnostics recorded for this parse.
    pub fn diagnostics(&self) -> &[ParseDiagnostic] {
        &self.diagnostics
    }

    /// Return parse metadata recorded for this document.
    pub fn metadata(&self) -> &ParseMetadata {
        &self.metadata
    }

    /// Return the original source text.
    pub fn source_text(&self) -> &str {
        &self.source
    }

    /// Return the original source bytes.
    pub fn source_bytes(&self) -> &[u8] {
        self.source.as_bytes()
    }

    pub(crate) fn root_parse_node(&self) -> &ParseNode {
        &self.root
    }
}

/// Basic parse metadata for a native document.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParseMetadata {
    /// Number of parser recovery/error events recorded for this parse.
    pub error_count: usize,
}

/// A structured parse diagnostic attached to a native document.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParseDiagnostic {
    /// Byte offset where the diagnostic begins.
    pub start_byte: usize,
    /// Byte offset where the diagnostic ends.
    pub end_byte: usize,
    /// Human-readable diagnostic summary.
    pub message: String,
}

/// Borrowed generic CST view for an [`AdzeDocument`].
#[derive(Clone, Copy, Debug)]
pub struct AdzeTree<'doc> {
    document: &'doc AdzeDocument,
}

impl<'doc> AdzeTree<'doc> {
    /// Return the root node.
    pub fn root(&self) -> AdzeNode<'doc> {
        AdzeNode {
            document: self.document,
            node: &self.document.root,
        }
    }

    /// Return whether this tree has parser errors.
    pub fn has_errors(&self) -> bool {
        self.document.metadata.error_count > 0
    }

    /// Return the number of parser recovery/error events.
    pub fn error_count(&self) -> usize {
        self.document.metadata.error_count
    }
}

/// Borrowed generic CST node view.
#[derive(Clone, Copy, Debug)]
pub struct AdzeNode<'doc> {
    document: &'doc AdzeDocument,
    node: &'doc ParseNode,
}

impl<'doc> AdzeNode<'doc> {
    /// Return the node's grammar symbol id.
    pub fn symbol_id(&self) -> SymbolId {
        self.node.symbol_id
    }

    /// Return the start byte for this node.
    pub fn start_byte(&self) -> usize {
        self.node.start_byte
    }

    /// Return the end byte for this node.
    pub fn end_byte(&self) -> usize {
        self.node.end_byte
    }

    /// Return the byte range for this node.
    pub fn byte_range(&self) -> Range<usize> {
        self.start_byte()..self.end_byte()
    }

    /// Return this node's source text if the byte range is valid UTF-8.
    pub fn utf8_text(&self) -> Result<&'doc str, std::str::Utf8Error> {
        let slice = self
            .document
            .source_bytes()
            .get(self.byte_range())
            .unwrap_or(&[]);
        std::str::from_utf8(slice)
    }

    /// Return the field name attached to this node's parent edge, if any.
    pub fn field_name(&self) -> Option<&'doc str> {
        self.node.field_name.as_deref()
    }

    /// Return the number of direct children.
    pub fn child_count(&self) -> usize {
        self.node.children.len()
    }

    /// Return a child by index.
    pub fn child(&self, index: usize) -> Option<AdzeNode<'doc>> {
        self.node.children.get(index).map(|child| AdzeNode {
            document: self.document,
            node: child,
        })
    }

    /// Return the field name for a child edge by index.
    pub fn field_name_for_child(&self, index: usize) -> Option<&'doc str> {
        self.node
            .children
            .get(index)
            .and_then(|child| child.field_name.as_deref())
    }

    /// Return whether this node is a local synthetic error node.
    pub fn is_error(&self) -> bool {
        self.node.symbol.0 == 0 && self.node.children.is_empty()
    }

    /// Return whether this node is a zero-width synthetic missing node.
    pub fn is_missing(&self) -> bool {
        self.start_byte() == self.end_byte() && self.is_error()
    }

    /// Return whether this node or its descendants carry error state.
    pub fn has_error(&self) -> bool {
        self.is_error()
            || (std::ptr::eq(self.node, &self.document.root)
                && self.document.metadata.error_count > 0)
            || self.node.children.iter().any(|child| {
                AdzeNode {
                    document: self.document,
                    node: child,
                }
                .has_error()
            })
    }
}

fn build_diagnostics(
    root: &ParseNode,
    error_count: usize,
    source_len: usize,
) -> Vec<ParseDiagnostic> {
    if error_count == 0 {
        return Vec::new();
    }

    let span = first_error_span(root).unwrap_or_else(|| root.start_byte..root.end_byte);
    let start_byte = span.start.min(source_len);
    let end_byte = span.end.min(source_len).max(start_byte);

    vec![ParseDiagnostic {
        start_byte,
        end_byte,
        message: format!("parser recorded {error_count} recovery/error event(s)"),
    }]
}

fn first_error_span(node: &ParseNode) -> Option<Range<usize>> {
    if node.symbol.0 == 0 && node.children.is_empty() {
        return Some(node.start_byte..node.end_byte);
    }

    node.children.iter().find_map(first_error_span)
}
