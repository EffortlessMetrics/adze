// Compatibility layer to provide a minimal `tree_sitter`-shaped API
// for pure-Rust parsing in environments where no `tree-sitter*` backend
// feature is enabled.
#![allow(dead_code)]
#![allow(unreachable_pub)]
#![allow(clippy::redundant_closure)]

use std::ffi::CStr;

use crate::pure_incremental::Tree as PureTree;
use crate::pure_parser::{ParsedNode, Parser as PureParser, TSLanguage};

pub type Language = &'static TSLanguage;
pub use crate::pure_parser::Point;

pub const LANGUAGE_VERSION: u32 = crate::pure_parser::TREE_SITTER_LANGUAGE_VERSION;
pub const MIN_COMPATIBLE_LANGUAGE_VERSION: u32 =
    crate::pure_parser::TREE_SITTER_MIN_COMPATIBLE_LANGUAGE_VERSION;

/// Compatibility wrapper for a parsed node.
#[derive(Copy, Clone)]
#[allow(clippy::missing_inline_in_public_items)]
pub struct Node {
    ptr: *const ParsedNode,
}

impl Node {
    #[inline]
    fn new(inner: &ParsedNode) -> Self {
        Self {
            ptr: inner as *const _,
        }
    }

    #[inline]
    fn as_ref(&self) -> &ParsedNode {
        // SAFETY: `Node` values are only constructed from references that are
        // valid for the lifetime of the underlying parse tree. This module
        // purposefully mirrors the tree-sitter API shape used by this crate.
        unsafe { &*self.ptr }
    }

    pub fn kind(&self) -> &str {
        self.as_ref().kind()
    }

    pub fn child_count(&self) -> usize {
        self.as_ref().child_count()
    }

    pub fn child(&self, index: usize) -> Option<Node> {
        self.as_ref().child(index).map(Node::new)
    }

    pub fn is_error(&self) -> bool {
        self.as_ref().is_error()
    }

    pub fn is_missing(&self) -> bool {
        self.as_ref().is_missing()
    }

    pub fn has_error(&self) -> bool {
        self.as_ref().has_error()
    }

    pub fn is_named(&self) -> bool {
        self.as_ref().is_named()
    }

    pub fn utf8_text<'a>(&self, source: &'a [u8]) -> Result<&'a str, std::str::Utf8Error> {
        self.as_ref().utf8_text(source)
    }

    pub fn start_byte(&self) -> usize {
        self.as_ref().start_byte()
    }

    pub fn end_byte(&self) -> usize {
        self.as_ref().end_byte()
    }

    pub fn start_position(&self) -> Point {
        self.as_ref().start_point()
    }

    pub fn end_position(&self) -> Point {
        self.as_ref().end_point()
    }

    /// Returns `None` — standalone nodes have no parent cursor context.
    pub fn field_name(&self) -> Option<&str> {
        None
    }

    pub fn walk(&self) -> TreeCursor {
        TreeCursor::new(*self)
    }

    pub fn children<'a>(&'a self, cursor: &'a mut TreeCursor) -> Children<'a> {
        cursor.reset(*self);
        let index = cursor.index;
        let children = self.as_ref().children();
        Children {
            cursor,
            children,
            index,
        }
    }
}

/// Compatibility tree cursor.
#[derive(Copy, Clone)]
pub struct TreeCursor {
    parent: Option<Node>,
    index: usize,
}

impl TreeCursor {
    #[inline]
    fn new(node: Node) -> Self {
        Self {
            parent: Some(node),
            index: 0,
        }
    }

    #[inline]
    fn reset(&mut self, node: Node) {
        self.parent = Some(node);
        self.index = 0;
    }

    pub fn node(&self) -> Node {
        let parent = self.parent.unwrap();
        if let Some(child) = parent.as_ref().child(self.index) {
            Node::new(child)
        } else {
            parent
        }
    }

    pub fn goto_first_child(&mut self) -> bool {
        self.index = 0;
        match self.parent {
            Some(parent) => parent.child_count() > 0,
            None => false,
        }
    }

    pub fn goto_next_sibling(&mut self) -> bool {
        let Some(parent) = self.parent else {
            return false;
        };

        if self.index + 1 < parent.child_count() {
            self.index += 1;
            true
        } else {
            false
        }
    }

    pub fn field_name(&self) -> Option<&str> {
        let parent = self.parent?;
        let children = parent.as_ref().children();
        let child = children.get(self.index)?;
        child.field_id.and_then(|field_id| {
            let language = child.language?;
            let language = unsafe { &*language };

            if usize::from(field_id) >= language.field_count as usize {
                return None;
            }

            let field_names = unsafe {
                std::slice::from_raw_parts(language.field_names, language.field_count as usize)
            };

            let field_ptr = *field_names.get(field_id as usize)?;
            if field_ptr.is_null() {
                return None;
            }

            unsafe { CStr::from_ptr(field_ptr as *const i8).to_str().ok() }
        })
    }
}

pub struct Children<'a> {
    cursor: &'a mut TreeCursor,
    children: &'a [ParsedNode],
    index: usize,
}

impl<'a> Iterator for Children<'a> {
    type Item = Node;

    fn next(&mut self) -> Option<Self::Item> {
        let child = self.children.get(self.index).map(Node::new);
        if self.index < self.children.len() {
            self.index += 1;
            self.cursor.index = self.index;
        }
        child
    }
}

/// Compatibility wrapper for parser object.
pub struct Parser {
    inner: PureParser,
}

impl Parser {
    pub fn new() -> Self {
        Self {
            inner: PureParser::new(),
        }
    }

    pub fn set_language(&mut self, language: Language) -> Result<(), String> {
        self.inner.set_language(language)
    }

    pub fn parse(&mut self, source: &str, old_tree: Option<&Tree>) -> Option<Tree> {
        let result = self
            .inner
            .parse_string_with_tree(source, old_tree.map(|tree| &tree.inner));

        let root = result.root?;
        let language = self.inner.language()?;

        Some(Tree {
            inner: PureTree::new(root, language, source.as_bytes()),
        })
    }

    pub fn set_timeout_micros(&mut self, timeout: u64) {
        self.inner.set_timeout_micros(timeout);
    }
}

impl Default for Parser {
    fn default() -> Self {
        Self::new()
    }
}

/// Compatibility wrapper for a parsed tree.
#[derive(Clone)]
pub struct Tree {
    inner: PureTree,
}

impl Tree {
    pub fn root_node(&self) -> Node {
        Node::new(self.inner.root_node())
    }
}
