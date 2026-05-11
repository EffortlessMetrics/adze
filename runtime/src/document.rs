//! Native parse document alpha.
//!
//! This module is the first implementation slice of the `AdzeDocument`
//! contract. It intentionally exposes only the current parser tree, source
//! text, basic diagnostics, and metadata. Richer projections such as typed CST,
//! typed AST provenance, and GLR forest summaries are future slices.

use crate::parser_v4::ParseNode;
use adze_glr_core::{ParseTable, SymbolMetadata as TableSymbolMetadata};
use adze_ir::{Grammar, SymbolId};
use std::ffi::CStr;
use std::num::NonZeroU16;
use std::ops::Range;

/// Nonzero public field identifier used by native document field metadata.
///
/// This matches Tree-sitter's public field-id convention: ID 0 is the
/// sentinel and real field names start at 1.
pub type FieldId = NonZeroU16;

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
    pure_language: Option<&'static crate::pure_parser::TSLanguage>,
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
        let diagnostics = build_diagnostics(&root, error_count, source);
        let runtime = DocumentRuntime {
            language_name,
            grammar,
            parse_table,
            pure_language: None,
        };
        Self::from_parse_result_with_diagnostics(source, root, error_count, runtime, diagnostics)
    }

    pub(crate) fn from_parse_result_with_diagnostics(
        source: &str,
        root: ParseNode,
        error_count: usize,
        runtime: DocumentRuntime<'_>,
        diagnostics: Vec<ParseDiagnostic>,
    ) -> Self {
        let node_index = build_node_index(&root);
        let mut diagnostics = diagnostics;
        attach_related_nodes(&root, &mut diagnostics);
        Self {
            source: source.to_string(),
            root,
            node_index,
            language: LanguageMetadata::from_runtime(
                runtime.language_name,
                runtime.grammar,
                runtime.parse_table,
            ),
            diagnostics,
            metadata: ParseMetadata { error_count },
            pure_language: runtime.pure_language,
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

    /// Return diagnostics directly related to a document-local node id.
    pub fn diagnostics_for_node(
        &self,
        node_id: NodeId,
    ) -> impl Iterator<Item = &ParseDiagnostic> + '_ {
        self.diagnostics
            .iter()
            .filter(move |diagnostic| diagnostic.related_nodes.contains(&node_id))
    }

    /// Extract a typed AST from this document's selected tree.
    ///
    /// This alpha view is available for generated pure-Rust documents, where
    /// the document retains the generated language metadata needed by
    /// [`Extract`](crate::Extract). Documents with parser diagnostics return
    /// those diagnostics as parse errors instead of extracting from recovered
    /// syntax.
    pub fn ast<T>(&self) -> Result<T, Vec<crate::errors::ParseError>>
    where
        T: crate::Extract<T>,
    {
        if !self.diagnostics.is_empty() {
            return Err(self
                .diagnostics
                .iter()
                .map(ParseDiagnostic::to_parse_error)
                .collect());
        }

        let Some(language) = self.pure_language else {
            return Err(vec![crate::errors::ParseError {
                reason: crate::errors::ParseErrorReason::UnexpectedToken(
                    "typed AST extraction requires generated pure-Rust language metadata"
                        .to_string(),
                ),
                start: 0,
                end: 0,
                expected: Vec::new(),
            }]);
        };

        let parsed_root = document_node_to_parsed_node(&self.root, language, self.source_bytes());
        let non_extra_root_children: Vec<_> = parsed_root
            .children
            .iter()
            .filter(|child| !child.is_extra)
            .collect();
        let extract_node =
            if parsed_root.kind() == "source_file" && non_extra_root_children.len() == 1 {
                non_extra_root_children[0]
            } else {
                &parsed_root
            };

        Ok(<T as crate::Extract<_>>::extract(
            Some(extract_node),
            self.source_bytes(),
            0,
            None,
        ))
    }

    #[cfg(feature = "ts-compat")]
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

pub(crate) struct DocumentRuntime<'a> {
    pub(crate) language_name: &'a str,
    pub(crate) grammar: &'a Grammar,
    pub(crate) parse_table: &'a ParseTable,
    pub(crate) pure_language: Option<&'static crate::pure_parser::TSLanguage>,
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
    fields: Vec<String>,
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
            fields: parse_table.field_names.clone(),
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

    /// Return the number of public fields in this language.
    pub fn field_count(&self) -> usize {
        self.fields.len()
    }

    /// Return all public field names in their zero-based table order.
    ///
    /// Public field IDs are one-based, so `fields()[0]` corresponds to
    /// [`field_name_for_id(1)`](Self::field_name_for_id).
    pub fn fields(&self) -> &[String] {
        &self.fields
    }

    /// Return a field name for a nonzero public field id.
    pub fn field_name_for_id(&self, field_id: u16) -> Option<&str> {
        let index = field_id.checked_sub(1)? as usize;
        self.fields.get(index).map(String::as_str)
    }

    /// Return the nonzero public field id for a field name.
    pub fn field_id_for_name(&self, field_name: impl AsRef<[u8]>) -> Option<FieldId> {
        let field_name = field_name.as_ref();
        self.fields
            .iter()
            .position(|candidate| candidate.as_bytes() == field_name)
            .and_then(|index| {
                let field_id = u16::try_from(index.checked_add(1)?).ok()?;
                FieldId::new(field_id)
            })
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
    /// Zero-based row/column range covered by the diagnostic.
    pub point_range: PointRange,
    /// Human-readable found token or symbol name, when known.
    pub found: Option<String>,
    /// Human-readable expected token or symbol names, when known.
    pub expected: Vec<String>,
    /// Document-local nodes related to this diagnostic.
    pub related_nodes: Vec<NodeId>,
    /// Human-readable diagnostic summary.
    pub message: String,
}

impl ParseDiagnostic {
    /// Return the byte span covered by this diagnostic.
    #[must_use]
    pub fn byte_span(&self) -> Range<usize> {
        self.start_byte..self.end_byte
    }

    /// Return a formatter that includes source location and context.
    #[must_use]
    pub fn display_with_source<'a>(&'a self, source: &'a str) -> ParseDiagnosticWithSource<'a> {
        ParseDiagnosticWithSource {
            diagnostic: self,
            source,
        }
    }

    fn to_parse_error(&self) -> crate::errors::ParseError {
        crate::errors::ParseError {
            reason: crate::errors::ParseErrorReason::UnexpectedToken(self.message.clone()),
            start: self.start_byte,
            end: self.end_byte,
            expected: self.expected.clone(),
        }
    }
}

impl std::fmt::Display for ParseDiagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} at {}:{} (bytes {}..{})",
            self.message,
            self.point_range.start.row + 1,
            self.point_range.start.column + 1,
            self.start_byte,
            self.end_byte
        )
    }
}

/// Display helper returned by [`ParseDiagnostic::display_with_source`].
pub struct ParseDiagnosticWithSource<'a> {
    diagnostic: &'a ParseDiagnostic,
    source: &'a str,
}

impl std::fmt::Display for ParseDiagnosticWithSource<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.diagnostic)?;

        if let Some(line) = source_line(self.source, self.diagnostic.start_byte) {
            let range = self.diagnostic.point_range;
            let marker_width = if range.start.row == range.end.row {
                range.end.column.saturating_sub(range.start.column).max(1)
            } else {
                1
            };
            let marker =
                " ".repeat(range.start.column as usize) + &"^".repeat(marker_width as usize);
            write!(f, "\n{line}\n{marker}")?;
        }

        Ok(())
    }
}

/// A zero-based source point in a native parse document.
///
/// Columns are byte offsets within a row, matching Tree-sitter's point model.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct DocumentPoint {
    /// Zero-based row number.
    pub row: u32,
    /// Zero-based byte column within the row.
    pub column: u32,
}

impl DocumentPoint {
    /// Compute a document point from a byte offset.
    ///
    /// Out-of-range byte offsets are clamped to the end of `source`.
    #[must_use]
    pub fn from_byte_offset(source: &str, byte: usize) -> Self {
        let end = byte.min(source.len());
        let mut row = 0u32;
        let mut column = 0u32;

        for &source_byte in &source.as_bytes()[..end] {
            if source_byte == b'\n' {
                row = row.saturating_add(1);
                column = 0;
            } else {
                column = column.saturating_add(1);
            }
        }

        Self { row, column }
    }
}

/// A zero-based source point range in a native parse document.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct PointRange {
    /// Inclusive start point.
    pub start: DocumentPoint,
    /// Exclusive end point.
    pub end: DocumentPoint,
}

impl PointRange {
    /// Compute a point range from a byte range.
    #[must_use]
    pub fn from_byte_range(source: &str, range: Range<usize>) -> Self {
        Self {
            start: DocumentPoint::from_byte_offset(source, range.start),
            end: DocumentPoint::from_byte_offset(source, range.end),
        }
    }
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

    /// Return this node's zero-based point range.
    pub fn point_range(&self) -> PointRange {
        PointRange::from_byte_range(self.document.source_text(), self.byte_range())
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

    /// Return the public field id attached to this node's parent edge, if any.
    pub fn field_id(&self) -> Option<FieldId> {
        self.field_name()
            .and_then(|field_name| self.document.language().field_id_for_name(field_name))
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
            field_id: child
                .field_name
                .as_deref()
                .and_then(|field_name| self.document.language().field_id_for_name(field_name)),
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

    /// Return the public field id for a child edge by index.
    pub fn field_id_for_child(&self, index: usize) -> Option<FieldId> {
        self.child_edge(index).and_then(|edge| edge.field_id())
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

    /// Return the first child attached through the given public field id.
    pub fn child_by_field_id(&self, field_id: FieldId) -> Option<AdzeNode<'doc>> {
        self.child_edges()
            .find(|edge| edge.field_id() == Some(field_id))?
            .child()
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

    /// Return diagnostics directly related to this node.
    pub fn diagnostics(&self) -> impl Iterator<Item = &'doc ParseDiagnostic> + 'doc {
        let node_id = self.id;
        self.document
            .diagnostics
            .iter()
            .filter(move |diagnostic| diagnostic.related_nodes.contains(&node_id))
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
    field_id: Option<FieldId>,
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

    /// Return the public field id attached to this edge, if any.
    pub fn field_id(&self) -> Option<FieldId> {
        self.field_id
    }
}

/// Common handle contract for generated typed CST wrappers.
///
/// Typed CST wrappers should be cheap views over [`AdzeDocument`] node IDs.
/// The default helpers are fallible so a wrapper can preserve honest behavior
/// when constructed around stale, recovered, or dynamically discovered syntax.
/// Generated wrappers are expected to validate their kind in their own `cast`
/// constructors before implementing typed field accessors.
pub trait SyntaxNode<'doc>: Copy {
    /// Return the document backing this typed CST handle.
    fn document(&self) -> &'doc AdzeDocument;

    /// Return the document-local node id represented by this handle.
    fn node_id(&self) -> NodeId;

    /// Return the generic CST node for this typed handle.
    fn node(&self) -> Option<AdzeNode<'doc>> {
        self.document().tree().node(self.node_id())
    }

    /// Return this handle's display kind name, when the node resolves.
    fn kind_name(&self) -> Option<&'doc str> {
        self.node().and_then(|node| node.kind_name())
    }

    /// Return this handle's byte range, when the node resolves.
    fn byte_range(&self) -> Option<Range<usize>> {
        self.node().map(|node| node.byte_range())
    }

    /// Return this handle's zero-based point range, when the node resolves.
    fn point_range(&self) -> Option<PointRange> {
        self.node().map(|node| node.point_range())
    }

    /// Return this handle's source text, when the range is a valid UTF-8 slice.
    fn text(&self) -> Option<&'doc str> {
        self.byte_range()
            .and_then(|range| self.document().source_slice(range))
    }

    /// Return a child node by index, when the node and child resolve.
    fn child(&self, index: usize) -> Option<AdzeNode<'doc>> {
        self.node()?.child(index)
    }

    /// Return a child edge by index, when the node and edge resolve.
    fn child_edge(&self, index: usize) -> Option<AdzeEdge<'doc>> {
        self.node()?.child_edge(index)
    }

    /// Return a child edge by native field name.
    fn edge_by_field_name(&self, field_name: &str) -> Option<AdzeEdge<'doc>> {
        self.node()?.edge_by_field_name(field_name)
    }

    /// Return a child node by native field name.
    fn child_by_field_name(&self, field_name: &str) -> Option<AdzeNode<'doc>> {
        self.node()?.child_by_field_name(field_name)
    }

    /// Return whether this handle resolves to a node-local synthetic error.
    fn is_error(&self) -> bool {
        self.node().map(|node| node.is_error()).unwrap_or(false)
    }

    /// Return whether this handle resolves to a zero-width missing node.
    fn is_missing(&self) -> bool {
        self.node().map(|node| node.is_missing()).unwrap_or(false)
    }

    /// Return whether this handle resolves to syntax that carries error state.
    fn has_error(&self) -> bool {
        self.node().map(|node| node.has_error()).unwrap_or(false)
    }
}

fn build_diagnostics(root: &ParseNode, error_count: usize, source: &str) -> Vec<ParseDiagnostic> {
    if error_count == 0 {
        return Vec::new();
    }

    let span = first_error_span(root).unwrap_or_else(|| root.start_byte..root.end_byte);
    let start_byte = span.start.min(source.len());
    let end_byte = span.end.min(source.len()).max(start_byte);
    let point_range = PointRange::from_byte_range(source, start_byte..end_byte);

    vec![ParseDiagnostic {
        start_byte,
        end_byte,
        point_range,
        found: None,
        expected: Vec::new(),
        related_nodes: Vec::new(),
        message: format!("parser recorded {error_count} recovery/error event(s)"),
    }]
}

fn attach_related_nodes(root: &ParseNode, diagnostics: &mut [ParseDiagnostic]) {
    for diagnostic in diagnostics {
        diagnostic.related_nodes = related_nodes_for_diagnostic(root, diagnostic);
    }
}

fn related_nodes_for_diagnostic(root: &ParseNode, diagnostic: &ParseDiagnostic) -> Vec<NodeId> {
    let mut related_errors = Vec::new();
    let mut next_id = 0;
    collect_related_error_nodes(root, diagnostic, &mut next_id, &mut related_errors);
    if !related_errors.is_empty() {
        return related_errors;
    }

    let mut best = None;
    let mut next_id = 0;
    collect_smallest_covering_node(root, diagnostic, &mut next_id, &mut best);
    best.map(|(node_id, _)| vec![node_id]).unwrap_or_default()
}

fn collect_related_error_nodes(
    node: &ParseNode,
    diagnostic: &ParseDiagnostic,
    next_id: &mut usize,
    related: &mut Vec<NodeId>,
) {
    let node_id = NodeId::new(*next_id);
    *next_id += 1;

    if is_error_parse_node(node) && node_range_touches_diagnostic(node, diagnostic) {
        related.push(node_id);
    }

    for child in &node.children {
        collect_related_error_nodes(child, diagnostic, next_id, related);
    }
}

fn collect_smallest_covering_node(
    node: &ParseNode,
    diagnostic: &ParseDiagnostic,
    next_id: &mut usize,
    best: &mut Option<(NodeId, usize)>,
) {
    let node_id = NodeId::new(*next_id);
    *next_id += 1;

    if node_covers_diagnostic(node, diagnostic) {
        let width = node.end_byte.saturating_sub(node.start_byte);
        if best
            .map(|(_, best_width)| width < best_width)
            .unwrap_or(true)
        {
            *best = Some((node_id, width));
        }
    }

    for child in &node.children {
        collect_smallest_covering_node(child, diagnostic, next_id, best);
    }
}

fn is_error_parse_node(node: &ParseNode) -> bool {
    node.symbol.0 == 0 && node.children.is_empty()
}

fn node_range_touches_diagnostic(node: &ParseNode, diagnostic: &ParseDiagnostic) -> bool {
    if diagnostic.start_byte == diagnostic.end_byte {
        node.start_byte <= diagnostic.start_byte && diagnostic.start_byte <= node.end_byte
    } else {
        node.start_byte < diagnostic.end_byte && diagnostic.start_byte < node.end_byte
    }
}

fn node_covers_diagnostic(node: &ParseNode, diagnostic: &ParseDiagnostic) -> bool {
    if diagnostic.start_byte == diagnostic.end_byte {
        node.start_byte <= diagnostic.start_byte && diagnostic.start_byte <= node.end_byte
    } else {
        node.start_byte <= diagnostic.start_byte && diagnostic.end_byte <= node.end_byte
    }
}

fn first_error_span(node: &ParseNode) -> Option<Range<usize>> {
    if node.symbol.0 == 0 && node.children.is_empty() {
        return Some(node.start_byte..node.end_byte);
    }

    node.children.iter().find_map(first_error_span)
}

fn source_line(source: &str, byte_offset: usize) -> Option<&str> {
    if source.is_empty() {
        return None;
    }

    let bytes = source.as_bytes();
    let offset = byte_offset.min(bytes.len());
    let mut start = offset;
    while start > 0 && bytes[start - 1] != b'\n' && bytes[start - 1] != b'\r' {
        start -= 1;
    }

    let mut end = offset;
    while end < bytes.len() && bytes[end] != b'\n' && bytes[end] != b'\r' {
        end += 1;
    }

    source.get(start..end)
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

fn document_node_to_parsed_node(
    node: &ParseNode,
    language: &'static crate::pure_parser::TSLanguage,
    source: &[u8],
) -> crate::pure_parser::ParsedNode {
    let symbol = table_symbol_for_public_id(language, node.symbol_id);
    let children = node
        .children
        .iter()
        .map(|child| document_node_to_parsed_node(child, language, source))
        .collect();
    let (is_named, is_extra) = symbol_flags(language, symbol);
    let is_empty_error_node =
        node.symbol_id.0 == 0 && node.children.is_empty() && node.start_byte == node.end_byte;

    crate::pure_parser::ParsedNode {
        symbol,
        children,
        start_byte: node.start_byte,
        end_byte: node.end_byte,
        start_point: byte_to_point(source, node.start_byte),
        end_point: byte_to_point(source, node.end_byte),
        is_extra,
        is_error: symbol_name(language, symbol) == Some("ERROR") || is_empty_error_node,
        is_missing: is_empty_error_node,
        is_named,
        field_id: node
            .field_name
            .as_deref()
            .and_then(|field_name| field_id_for_name(language, field_name)),
        language: Some(language as *const _),
    }
}

fn table_symbol_for_public_id(
    language: &crate::pure_parser::TSLanguage,
    public_symbol: SymbolId,
) -> crate::pure_parser::TSSymbol {
    if !language.public_symbol_map.is_null() {
        // SAFETY: `public_symbol_map` has one entry per generated table symbol.
        let public_symbols = unsafe {
            std::slice::from_raw_parts(language.public_symbol_map, language.symbol_count as usize)
        };
        if let Some(index) = public_symbols
            .iter()
            .position(|candidate| *candidate == public_symbol.0)
        {
            return index as crate::pure_parser::TSSymbol;
        }
    }

    public_symbol.0
}

fn symbol_flags(
    language: &crate::pure_parser::TSLanguage,
    symbol: crate::pure_parser::TSSymbol,
) -> (bool, bool) {
    if language.symbol_metadata.is_null() || u32::from(symbol) >= language.symbol_count {
        return (true, false);
    }

    // SAFETY: `symbol` is bounds-checked above, and `symbol_metadata` has one
    // entry per generated table symbol.
    let metadata = unsafe { *language.symbol_metadata.add(usize::from(symbol)) };
    let is_named = (metadata & 0x02) != 0;
    let is_extra = (metadata & 0x04) != 0;
    (is_named, is_extra)
}

fn field_id_for_name(language: &crate::pure_parser::TSLanguage, field_name: &str) -> Option<u16> {
    if language.field_count == 0 || language.field_names.is_null() {
        return None;
    }

    // SAFETY: `field_names` points to a static array of `field_count` C string
    // pointers generated with the language table.
    let field_names =
        unsafe { std::slice::from_raw_parts(language.field_names, language.field_count as usize) };
    field_names
        .iter()
        .enumerate()
        .find_map(|(index, name_ptr)| {
            c_str_to_str(*name_ptr)
                .filter(|candidate| *candidate == field_name)
                .map(|_| index as u16)
        })
}

fn symbol_name(
    language: &crate::pure_parser::TSLanguage,
    symbol: crate::pure_parser::TSSymbol,
) -> Option<&'static str> {
    if language.symbol_names.is_null() || u32::from(symbol) >= language.symbol_count {
        return None;
    }

    // SAFETY: `symbol` is bounds-checked above, and `symbol_names` has one C
    // string pointer per generated table symbol.
    let symbol_names = unsafe {
        std::slice::from_raw_parts(language.symbol_names, language.symbol_count as usize)
    };
    c_str_to_str(symbol_names[usize::from(symbol)])
}

fn c_str_to_str(ptr: *const u8) -> Option<&'static str> {
    if ptr.is_null() {
        return None;
    }

    // SAFETY: generated language tables store static NUL-terminated strings.
    unsafe { CStr::from_ptr(ptr.cast()).to_str().ok() }
}

fn byte_to_point(source: &[u8], byte: usize) -> crate::pure_parser::Point {
    let point = DocumentPoint::from_byte_offset(std::str::from_utf8(source).unwrap_or(""), byte);
    crate::pure_parser::Point {
        row: point.row,
        column: point.column,
    }
}
