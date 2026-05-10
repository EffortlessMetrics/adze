//! Native parse document alpha.
//!
//! This module is the first implementation slice of the `AdzeDocument`
//! contract. It intentionally exposes only the current parser tree, source
//! text, basic diagnostics, and metadata. Richer projections such as typed CST,
//! typed AST provenance, and GLR forest summaries are future slices.

use crate::parser_v4::ParseNode;
use adze_glr_core::{ParseTable, SymbolMetadata as TableSymbolMetadata};
use adze_ir::{Grammar, SymbolId};
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
    node_index: Vec<NodeIndex>,
    language: LanguageMetadata,
    diagnostics: Vec<ParseDiagnostic>,
    metadata: ParseMetadata,
}

impl AdzeDocument {
    pub(crate) fn from_parse_result(
        source: &str,
        root: ParseNode,
        error_count: usize,
        language_name: &str,
        grammar: &Grammar,
        parse_table: &ParseTable,
    ) -> Self {
        let diagnostics = build_diagnostics(&root, error_count, source.len());
        let node_index = build_node_index(&root);
        Self {
            source: source.to_string(),
            root,
            node_index,
            language: LanguageMetadata::from_runtime(language_name, grammar, parse_table),
            diagnostics,
            metadata: ParseMetadata { error_count },
        }
    }

    /// Return the generic native CST view.
    pub fn tree(&self) -> AdzeTree<'_> {
        AdzeTree { document: self }
    }

    /// Return language metadata recorded for this document.
    pub fn language(&self) -> &LanguageMetadata {
        &self.language
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

    /// Return a UTF-8 source slice for a byte range.
    ///
    /// Returns `None` if the range is outside the document source or does not
    /// align to UTF-8 character boundaries.
    pub fn source_slice(&self, range: Range<usize>) -> Option<&str> {
        self.source.get(range)
    }

    pub(crate) fn root_parse_node(&self) -> &ParseNode {
        &self.root
    }

    fn node_by_id(&self, node_id: NodeId) -> Option<&ParseNode> {
        let index = self.node_index.get(node_id.as_usize())?;
        let mut node = &self.root;

        for &child_index in &index.path {
            node = node.children.get(child_index)?;
        }

        Some(node)
    }

    fn child_id(&self, node_id: NodeId, child_index: usize) -> Option<NodeId> {
        self.node_index
            .get(node_id.as_usize())?
            .child_ids
            .get(child_index)
            .copied()
    }

    fn parent_id(&self, node_id: NodeId) -> Option<NodeId> {
        self.node_index.get(node_id.as_usize())?.parent_id
    }
}

/// Stable node identifier within one [`AdzeDocument`].
///
/// Node IDs are assigned in preorder over the selected parse tree. They are
/// stable for the lifetime of a document but are not meaningful across parses.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct NodeId(usize);

impl NodeId {
    /// Construct a node id from its raw preorder index.
    pub fn new(index: usize) -> Self {
        Self(index)
    }

    /// Return this node id as a raw preorder index.
    pub fn as_usize(self) -> usize {
        self.0
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct NodeIndex {
    path: Vec<usize>,
    parent_id: Option<NodeId>,
    child_ids: Vec<NodeId>,
}

/// Native language metadata attached to a parse document.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LanguageMetadata {
    name: String,
    symbols: Vec<NodeKind>,
}

impl LanguageMetadata {
    pub(crate) fn from_runtime(
        language_name: &str,
        grammar: &Grammar,
        parse_table: &ParseTable,
    ) -> Self {
        let mut symbols = Vec::new();

        for metadata in &parse_table.symbol_metadata {
            insert_symbol(&mut symbols, NodeKind::from_table_metadata(metadata));
        }

        for (symbol_id, name) in &grammar.rule_names {
            if !symbols.iter().any(|symbol| symbol.symbol_id == *symbol_id) {
                let is_terminal = grammar.tokens.contains_key(symbol_id);
                insert_symbol(
                    &mut symbols,
                    NodeKind {
                        symbol_id: *symbol_id,
                        name: name.clone(),
                        is_visible: !name.starts_with('_'),
                        is_named: !is_terminal,
                        is_supertype: grammar.supertypes.contains(symbol_id),
                        is_terminal,
                        is_extra: grammar.extras.contains(symbol_id),
                    },
                );
            }
        }

        for (symbol_id, token) in &grammar.tokens {
            if !symbols.iter().any(|symbol| symbol.symbol_id == *symbol_id) {
                insert_symbol(
                    &mut symbols,
                    NodeKind {
                        symbol_id: *symbol_id,
                        name: token.name.clone(),
                        is_visible: !token.name.starts_with('_'),
                        is_named: false,
                        is_supertype: grammar.supertypes.contains(symbol_id),
                        is_terminal: true,
                        is_extra: grammar.extras.contains(symbol_id),
                    },
                );
            }
        }

        symbols.sort_by_key(|symbol| symbol.symbol_id.0);

        Self {
            name: language_name.to_string(),
            symbols,
        }
    }

    /// Return the language name used to create this document.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Return all known node kinds for this language.
    pub fn symbols(&self) -> &[NodeKind] {
        &self.symbols
    }

    /// Return metadata for a symbol id.
    pub fn symbol(&self, symbol_id: SymbolId) -> Option<&NodeKind> {
        self.symbols
            .iter()
            .find(|symbol| symbol.symbol_id == symbol_id)
    }

    /// Return the display name for a symbol id.
    pub fn symbol_name(&self, symbol_id: SymbolId) -> Option<&str> {
        self.symbol(symbol_id).map(NodeKind::name)
    }
}

/// Native metadata for one grammar symbol.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NodeKind {
    symbol_id: SymbolId,
    name: String,
    is_visible: bool,
    is_named: bool,
    is_supertype: bool,
    is_terminal: bool,
    is_extra: bool,
}

impl NodeKind {
    fn from_table_metadata(metadata: &TableSymbolMetadata) -> Self {
        Self {
            symbol_id: metadata.symbol_id,
            name: metadata.name.clone(),
            is_visible: metadata.is_visible,
            is_named: metadata.is_named,
            is_supertype: metadata.is_supertype,
            is_terminal: metadata.is_terminal,
            is_extra: metadata.is_extra,
        }
    }

    /// Return the symbol id for this node kind.
    pub fn symbol_id(&self) -> SymbolId {
        self.symbol_id
    }

    /// Return the display name for this node kind.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Return whether this node kind is visible in syntax output.
    pub fn is_visible(&self) -> bool {
        self.is_visible
    }

    /// Return whether this node kind is named.
    pub fn is_named(&self) -> bool {
        self.is_named
    }

    /// Return whether this node kind is a supertype.
    pub fn is_supertype(&self) -> bool {
        self.is_supertype
    }

    /// Return whether this node kind is a terminal token.
    pub fn is_terminal(&self) -> bool {
        self.is_terminal
    }

    /// Return whether this node kind is extra syntax such as trivia.
    pub fn is_extra(&self) -> bool {
        self.is_extra
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
    /// Return language metadata for this tree.
    pub fn language(&self) -> &'doc LanguageMetadata {
        self.document.language()
    }

    /// Return the root node id.
    pub fn root_id(&self) -> NodeId {
        NodeId::new(0)
    }

    /// Return the number of indexed nodes in this tree.
    pub fn node_count(&self) -> usize {
        self.document.node_index.len()
    }

    /// Return a node by document-local id.
    pub fn node(&self, node_id: NodeId) -> Option<AdzeNode<'doc>> {
        self.document.node_by_id(node_id).map(|node| AdzeNode {
            document: self.document,
            node,
            id: node_id,
        })
    }

    /// Return the root node.
    pub fn root(&self) -> AdzeNode<'doc> {
        AdzeNode {
            document: self.document,
            node: &self.document.root,
            id: self.root_id(),
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
    id: NodeId,
}

impl<'doc> AdzeNode<'doc> {
    /// Return this node's document-local id.
    pub fn node_id(&self) -> NodeId {
        self.id
    }

    /// Return this node's parent id, if it is not the root.
    pub fn parent_id(&self) -> Option<NodeId> {
        self.document.parent_id(self.id)
    }

    /// Return this node's parent, if it is not the root.
    pub fn parent(&self) -> Option<AdzeNode<'doc>> {
        self.parent_id()
            .and_then(|parent_id| self.document.tree().node(parent_id))
    }

    /// Return metadata for this node's kind, when known.
    pub fn kind(&self) -> Option<&'doc NodeKind> {
        self.document.language.symbol(self.symbol_id())
    }

    /// Return this node's display kind name, when known.
    pub fn kind_name(&self) -> Option<&'doc str> {
        self.kind().map(NodeKind::name)
    }

    /// Return this node's grammar symbol name, ignoring aliases.
    ///
    /// Current native parse nodes do not carry alias-specific identity, so the
    /// grammar name matches the visible kind name when metadata is available.
    pub fn grammar_name(&self) -> Option<&'doc str> {
        self.kind_name()
    }

    /// Return this node's visible kind id.
    ///
    /// Current native parse nodes do not carry alias-specific identity, so the
    /// visible kind id matches the grammar symbol id.
    pub fn kind_id(&self) -> SymbolId {
        self.symbol_id()
    }

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
        self.child_edge(index)?.child()
    }

    /// Return a child edge by index.
    pub fn child_edge(&self, index: usize) -> Option<AdzeEdge<'doc>> {
        let child = self.node.children.get(index)?;
        let child_id = self.document.child_id(self.id, index)?;
        Some(AdzeEdge {
            document: self.document,
            parent_id: self.id,
            child_index: index,
            child_id,
            field_name: child.field_name.as_deref(),
        })
    }

    /// Return direct child edges in source order.
    pub fn child_edges(&self) -> impl Iterator<Item = AdzeEdge<'doc>> + '_ {
        (0..self.child_count()).filter_map(|index| self.child_edge(index))
    }

    /// Return the field name for a child edge by index.
    pub fn field_name_for_child(&self, index: usize) -> Option<&'doc str> {
        self.child_edge(index).and_then(|edge| edge.field_name())
    }

    /// Return the first child edge attached through the given field name.
    pub fn edge_by_field_name(&self, field_name: &str) -> Option<AdzeEdge<'doc>> {
        self.child_edges()
            .find(|edge| edge.field_name() == Some(field_name))
    }

    /// Return the first child attached through the given field name.
    pub fn child_by_field_name(&self, field_name: &str) -> Option<AdzeNode<'doc>> {
        self.edge_by_field_name(field_name)?.child()
    }

    /// Return whether this node is named according to language metadata.
    pub fn is_named(&self) -> bool {
        self.kind().map(NodeKind::is_named).unwrap_or(false)
    }

    /// Return whether this node is visible according to language metadata.
    pub fn is_visible(&self) -> bool {
        self.kind().map(NodeKind::is_visible).unwrap_or(false)
    }

    /// Return whether this node is an extra syntax node according to metadata.
    pub fn is_extra(&self) -> bool {
        self.kind().map(NodeKind::is_extra).unwrap_or(false)
    }

    /// Return whether this node is a terminal token according to metadata.
    pub fn is_terminal(&self) -> bool {
        self.kind().map(NodeKind::is_terminal).unwrap_or(false)
    }

    /// Return whether this node is a supertype according to metadata.
    pub fn is_supertype(&self) -> bool {
        self.kind().map(NodeKind::is_supertype).unwrap_or(false)
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
            || (0..self.child_count()).any(|index| {
                self.child(index)
                    .map(|child| child.has_error())
                    .unwrap_or(false)
            })
    }
}

/// Borrowed parent-to-child CST edge view.
///
/// Field labels belong to edges, not globally to child nodes. `AdzeEdge`
/// makes that relationship explicit for native syntax tooling and future
/// generated typed CST accessors.
#[derive(Clone, Copy, Debug)]
pub struct AdzeEdge<'doc> {
    document: &'doc AdzeDocument,
    parent_id: NodeId,
    child_index: usize,
    child_id: NodeId,
    field_name: Option<&'doc str>,
}

impl<'doc> AdzeEdge<'doc> {
    /// Return the parent node id for this edge.
    pub fn parent_id(&self) -> NodeId {
        self.parent_id
    }

    /// Return this edge's child index within its parent.
    pub fn child_index(&self) -> usize {
        self.child_index
    }

    /// Return the child node id for this edge.
    pub fn child_id(&self) -> NodeId {
        self.child_id
    }

    /// Return the child node for this edge.
    pub fn child(&self) -> Option<AdzeNode<'doc>> {
        self.document.tree().node(self.child_id)
    }

    /// Return the field name attached to this edge, if any.
    pub fn field_name(&self) -> Option<&'doc str> {
        self.field_name
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

fn build_node_index(root: &ParseNode) -> Vec<NodeIndex> {
    let mut index = Vec::new();
    let mut path = Vec::new();
    collect_node_index(root, &mut path, &mut index);
    index
}

fn collect_node_index(
    node: &ParseNode,
    path: &mut Vec<usize>,
    index: &mut Vec<NodeIndex>,
) -> NodeId {
    collect_node_index_with_parent(node, path, index, None)
}

fn collect_node_index_with_parent(
    node: &ParseNode,
    path: &mut Vec<usize>,
    index: &mut Vec<NodeIndex>,
    parent_id: Option<NodeId>,
) -> NodeId {
    let id = NodeId::new(index.len());
    index.push(NodeIndex {
        path: path.clone(),
        parent_id,
        child_ids: Vec::with_capacity(node.children.len()),
    });

    let mut child_ids = Vec::with_capacity(node.children.len());
    for (child_index, child) in node.children.iter().enumerate() {
        path.push(child_index);
        child_ids.push(collect_node_index_with_parent(child, path, index, Some(id)));
        path.pop();
    }
    index[id.as_usize()].child_ids = child_ids;

    id
}

fn insert_symbol(symbols: &mut Vec<NodeKind>, symbol: NodeKind) {
    if let Some(existing) = symbols
        .iter_mut()
        .find(|existing| existing.symbol_id == symbol.symbol_id)
    {
        *existing = symbol;
    } else {
        symbols.push(symbol);
    }
}
