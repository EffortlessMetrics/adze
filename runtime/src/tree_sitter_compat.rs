// Compatibility layer to make pure-Rust types work with existing Extract trait
use crate::pure_incremental::Tree as PureTree;
use crate::pure_parser::{ParsedNode, Parser as PureParser};

// Type aliases for compatibility
#[allow(dead_code)]
pub(crate) type Node<'a> = NodeCompat<'a>;
#[allow(dead_code)]
pub(crate) type Parser = ParserCompat;
#[allow(dead_code)]
pub(crate) type Language = &'static crate::pure_parser::TSLanguage;

/// Compatibility wrapper for ParsedNode to work with Extract trait
pub(crate) struct NodeCompat<'a> {
    pub inner: &'a ParsedNode,
    source: &'a [u8],
}

#[allow(dead_code)]
impl<'a> NodeCompat<'a> {
    pub(crate) fn new(node: &'a ParsedNode, source: &'a [u8]) -> Self {
        NodeCompat {
            inner: node,
            source,
        }
    }

    pub(crate) fn kind(&self) -> &str {
        // In pure-Rust, we use symbol IDs, but we need to convert to strings
        // This would be populated from the language's symbol_names
        match self.inner.symbol {
            0 => "program",
            1 => "expression",
            2 => "number",
            3 => "identifier",
            _ => "unknown",
        }
    }

    pub(crate) fn start_byte(&self) -> usize {
        self.inner.start_byte
    }

    pub(crate) fn end_byte(&self) -> usize {
        self.inner.end_byte
    }

    pub(crate) fn utf8_text<'b>(&self, source: &'b [u8]) -> Result<&'b str, std::str::Utf8Error> {
        let text = &source[self.inner.start_byte..self.inner.end_byte];
        std::str::from_utf8(text)
    }

    pub(crate) fn is_error(&self) -> bool {
        self.inner.is_error
    }

    pub(crate) fn is_missing(&self) -> bool {
        self.inner.is_missing
    }

    pub(crate) fn has_error(&self) -> bool {
        // Check if this node or any descendant has an error
        self.inner.is_error
            || self
                .inner
                .children
                .iter()
                .any(|c| Self::new(c, self.source).has_error())
    }

    pub(crate) fn is_named(&self) -> bool {
        self.inner.is_named
    }

    pub(crate) fn child(&self, index: usize) -> Option<NodeCompat<'a>> {
        self.inner
            .child(index)
            .map(|c| NodeCompat::new(c, self.source))
    }

    pub(crate) fn children<'b>(&'b self) -> impl Iterator<Item = NodeCompat<'a>> + 'b {
        self.inner
            .children()
            .iter()
            .map(move |c| NodeCompat::new(c, self.source))
    }

    pub(crate) fn walk(&self) -> TreeCursor<'a> {
        TreeCursor {
            node: self.clone(),
            index: 0,
        }
    }

    pub(crate) fn field_name_for_child(&self, _index: usize) -> Option<&str> {
        // In pure-Rust, field names would come from the language definition
        // For now, return None
        None
    }

    pub(crate) fn reset(&mut self, node: Node<'a>) {
        *self = node;
    }
}

impl<'a> Clone for NodeCompat<'a> {
    fn clone(&self) -> Self {
        NodeCompat {
            inner: self.inner,
            source: self.source,
        }
    }
}

/// Tree cursor for traversing the parse tree
#[allow(dead_code)]
pub(crate) struct TreeCursor<'a> {
    node: NodeCompat<'a>,
    index: usize,
}

#[allow(dead_code)]
impl<'a> TreeCursor<'a> {
    pub(crate) fn node(&self) -> NodeCompat<'a> {
        self.node.clone()
    }

    pub(crate) fn goto_first_child(&mut self) -> bool {
        if let Some(first_child) = self.node.child(0) {
            self.node = first_child;
            self.index = 0;
            true
        } else {
            false
        }
    }

    pub(crate) fn goto_next_sibling(&mut self) -> bool {
        // This is simplified - in reality we'd need to track parent nodes
        false
    }

    pub(crate) fn field_name(&self) -> Option<&str> {
        None
    }

    pub(crate) fn reset(&mut self, node: Node<'a>) {
        self.node = node;
        self.index = 0;
    }
}

/// Parser wrapper for compatibility
pub(crate) struct ParserCompat {
    #[allow(dead_code)]
    inner: PureParser,
}

#[allow(dead_code)]
impl ParserCompat {
    pub(crate) fn new() -> Self {
        ParserCompat {
            inner: PureParser::new(),
        }
    }

    pub(crate) fn set_language(
        &mut self,
        language: &'static crate::pure_parser::TSLanguage,
    ) -> Result<(), String> {
        let _ = self.inner.set_language(language);
        Ok(())
    }

    pub(crate) fn parse(&mut self, source: &str, old_tree: Option<&PureTree>) -> Option<PureTree> {
        let result = if let Some(tree) = old_tree {
            self.inner.parse_string_with_tree(source, Some(tree))
        } else {
            self.inner.parse_string(source)
        };

        result.root.and_then(|root| {
            self.inner
                .language()
                .map(|language| PureTree::new(root, language, source.as_bytes()))
        })
    }

    pub(crate) fn set_timeout_micros(&mut self, timeout: u64) {
        self.inner.set_timeout_micros(timeout);
    }
}

impl Default for ParserCompat {
    fn default() -> Self {
        Self::new()
    }
}
