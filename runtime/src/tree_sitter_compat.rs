// Compatibility layer to make pure-Rust types work with existing Extract trait
use crate::pure_parser::{ParsedNode, Parser as PureParser};
use crate::pure_incremental::{Tree as PureTree};

/// Compatibility wrapper for ParsedNode to work with Extract trait
pub struct NodeCompat<'a> {
    pub inner: &'a ParsedNode,
    source: &'a [u8],
}

impl<'a> NodeCompat<'a> {
    pub fn new(node: &'a ParsedNode, source: &'a [u8]) -> Self {
        NodeCompat { inner: node, source }
    }
    
    pub fn kind(&self) -> &str {
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
    
    pub fn start_byte(&self) -> usize {
        self.inner.start_byte
    }
    
    pub fn end_byte(&self) -> usize {
        self.inner.end_byte
    }
    
    pub fn utf8_text<'b>(&self, source: &'b [u8]) -> Result<&'b str, std::str::Utf8Error> {
        let text = &source[self.inner.start_byte..self.inner.end_byte];
        std::str::from_utf8(text)
    }
    
    pub fn is_error(&self) -> bool {
        self.inner.is_error
    }
    
    pub fn is_missing(&self) -> bool {
        self.inner.is_missing
    }
    
    pub fn has_error(&self) -> bool {
        // Check if this node or any descendant has an error
        self.inner.is_error || self.inner.children.iter().any(|c| Self::new(c, self.source).has_error())
    }
    
    pub fn is_named(&self) -> bool {
        self.inner.is_named
    }
    
    pub fn child(&self, index: usize) -> Option<NodeCompat<'a>> {
        self.inner.children.get(index).map(|c| NodeCompat::new(c, self.source))
    }
    
    pub fn children<'b>(&'b self) -> impl Iterator<Item = NodeCompat<'a>> + 'b {
        self.inner.children.iter().map(move |c| NodeCompat::new(c, self.source))
    }
    
    pub fn walk(&self) -> TreeCursor<'a> {
        TreeCursor {
            node: self.clone(),
            index: 0,
        }
    }
    
    pub fn field_name_for_child(&self, index: usize) -> Option<&str> {
        // In pure-Rust, field names would come from the language definition
        // For now, return None
        None
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
pub struct TreeCursor<'a> {
    node: NodeCompat<'a>,
    index: usize,
}

impl<'a> TreeCursor<'a> {
    pub fn node(&self) -> NodeCompat<'a> {
        self.node.clone()
    }
    
    pub fn goto_first_child(&mut self) -> bool {
        if !self.node.inner.children.is_empty() {
            self.node = NodeCompat::new(&self.node.inner.children[0], self.node.source);
            self.index = 0;
            true
        } else {
            false
        }
    }
    
    pub fn goto_next_sibling(&mut self) -> bool {
        // This is simplified - in reality we'd need to track parent nodes
        false
    }
    
    pub fn field_name(&self) -> Option<&str> {
        None
    }
}

/// Parser wrapper for compatibility
pub struct ParserCompat {
    inner: PureParser,
}

impl ParserCompat {
    pub fn new() -> Self {
        ParserCompat {
            inner: PureParser::new(),
        }
    }
    
    pub fn set_language(&mut self, language: &crate::pure_parser::TSLanguage) -> Result<(), String> {
        self.inner.set_language(language)
    }
    
    pub fn parse(&mut self, source: &str, old_tree: Option<&PureTree>) -> Option<PureTree> {
        let result = if let Some(tree) = old_tree {
            self.inner.parse_string_with_tree(source, Some(tree))
        } else {
            self.inner.parse_string(source)
        };
        
        result.root.map(|root| PureTree::new(root, self.inner.language().unwrap(), source.as_bytes()))
    }
    
    pub fn set_timeout_micros(&mut self, timeout: u64) {
        self.inner.set_timeout_micros(timeout);
    }
}

impl Default for ParserCompat {
    fn default() -> Self {
        Self::new()
    }
}