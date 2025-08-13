// Compatibility layer to make pure-Rust types work with existing Extract trait
use crate::pure_incremental::Tree as PureTree;
use crate::pure_parser::{ParsedNode, Parser as PureParser};
use std::ffi::CStr;

// Type aliases for compatibility
#[allow(dead_code)]
pub type Node<'a> = NodeCompat<'a>;
#[allow(dead_code)]
pub type Parser = ParserCompat;
#[allow(dead_code)]
pub type Language = &'static crate::pure_parser::TSLanguage;

/// Compatibility wrapper for ParsedNode to work with Extract trait
pub struct NodeCompat<'a> {
    pub inner: &'a ParsedNode,
    source: &'a [u8],
}

#[allow(dead_code)]
impl<'a> NodeCompat<'a> {
    pub fn new(node: &'a ParsedNode, source: &'a [u8]) -> Self {
        NodeCompat {
            inner: node,
            source,
        }
    }

    pub fn kind(&self) -> &str {
        self.inner.kind()
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
        self.inner.is_error
            || self
                .inner
                .children
                .iter()
                .any(|c| Self::new(c, self.source).has_error())
    }

    pub fn is_named(&self) -> bool {
        self.inner.is_named
    }

    pub fn child(&self, index: usize) -> Option<NodeCompat<'a>> {
        self.inner
            .child(index)
            .map(|c| NodeCompat::new(c, self.source))
    }

    pub fn children<'b>(&'b self) -> impl Iterator<Item = NodeCompat<'a>> + 'b {
        self.inner
            .children()
            .iter()
            .map(move |c| NodeCompat::new(c, self.source))
    }

    pub fn walk(&self) -> TreeCursor<'a> {
        TreeCursor::new(self)
    }

    pub fn field_name_for_child(&self, index: usize) -> Option<&str> {
        let language = self.inner.language?;
        let production_id = self.inner.production_id;

        if production_id == 0 {
            return None;
        }

        unsafe {
            let language = &*language;
            if (production_id as u32) >= language.production_id_count {
                return None;
            }
            let slice_start = *language.field_map_slices.add(production_id as usize) as usize;
            let slice_end = *language
                .field_map_slices
                .add(production_id as usize + 1) as usize;

            for i in (slice_start..slice_end).step_by(2) {
                let field_id = *language.field_map_entries.add(i) as usize;
                let child_index = *language.field_map_entries.add(i + 1) as usize;

                if child_index == index {
                    if field_id < language.field_count as usize {
                        let field_names = std::slice::from_raw_parts(
                            language.field_names,
                            language.field_count as usize,
                        );
                        let name_ptr = field_names[field_id];
                        if !name_ptr.is_null() {
                            let c_str = CStr::from_ptr(name_ptr as *const i8);
                            return c_str.to_str().ok();
                        }
                    }
                    break;
                }
            }
        }

        None
    }

    pub fn reset(&mut self, node: Node<'a>) {
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
pub struct TreeCursor<'a> {
    stack: Vec<(NodeCompat<'a>, usize)>, // (parent, child_index)
    node: NodeCompat<'a>,
}

#[allow(dead_code)]
impl<'a> TreeCursor<'a> {
    pub fn new(node: &NodeCompat<'a>) -> Self {
        TreeCursor {
            stack: vec![],
            node: node.clone(),
        }
    }
    pub fn node(&self) -> NodeCompat<'a> {
        self.node.clone()
    }

    pub fn goto_first_child(&mut self) -> bool {
        if let Some(first_child) = self.node.child(0) {
            self.stack.push((self.node.clone(), 0));
            self.node = first_child;
            true
        } else {
            false
        }
    }

    pub fn goto_next_sibling(&mut self) -> bool {
        if let Some((parent, child_index)) = self.stack.last_mut() {
            let next_child_index = *child_index + 1;
            if let Some(next_sibling) = parent.child(next_child_index) {
                *child_index = next_child_index;
                self.node = next_sibling;
                true
            } else {
                false
            }
        } else {
            false // no parent
        }
    }

    pub fn goto_parent(&mut self) -> bool {
        if let Some((parent, _)) = self.stack.pop() {
            self.node = parent;
            true
        } else {
            false
        }
    }

    pub fn field_name(&self) -> Option<&str> {
        if let Some((parent, child_index)) = self.stack.last() {
            parent.field_name_for_child(*child_index)
        } else {
            None
        }
    }

    pub fn reset(&mut self, node: Node<'a>) {
        self.stack.clear();
        self.node = node;
    }
}

/// Parser wrapper for compatibility
pub struct ParserCompat {
    #[allow(dead_code)]
    inner: PureParser,
}

#[allow(dead_code)]
impl ParserCompat {
    pub fn new() -> Self {
        ParserCompat {
            inner: PureParser::new(),
        }
    }

    pub fn set_language(
        &mut self,
        language: &'static crate::pure_parser::TSLanguage,
    ) -> Result<(), String> {
        let _ = self.inner.set_language(language);
        Ok(())
    }

    pub fn parse(&mut self, source: &str, old_tree: Option<&PureTree>) -> Option<PureTree> {
        let result = if let Some(tree) = old_tree {
            self.inner.parse_string_with_tree(source, Some(tree))
        } else {
            self.inner.parse_string(source)
        };

        result.root.map(|root| {
            // Create tree with source
            PureTree::new(root, self.inner.language().unwrap(), source.as_bytes())
        })
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
