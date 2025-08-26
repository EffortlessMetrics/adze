//! Tree representation for parsed syntax trees

use crate::{node::Node, Language};
use std::fmt;

/// A parsed syntax tree
pub struct Tree {
    /// Root node of the tree
    root: TreeNode,
    /// Language used to parse this tree
    language: Option<Language>,
    /// Source text (optional, for convenience)
    #[allow(dead_code)]
    source: Option<Vec<u8>>,
}

/// Internal tree node representation
#[derive(Clone)]
#[allow(dead_code)]
pub(crate) struct TreeNode {
    /// Symbol type
    symbol: u32,
    /// Byte range in source
    start_byte: usize,
    end_byte: usize,
    /// Children nodes
    children: Vec<TreeNode>,
    /// Field ID if this node has a field name
    field_id: Option<u16>,
    /// Whether this node was touched by the last edit
    #[allow(dead_code)]
    dirty: bool,
}

impl TreeNode {
    /// Create a new tree node with children
    pub(crate) fn new_with_children(
        symbol: u32,
        start_byte: usize,
        end_byte: usize,
        children: Vec<TreeNode>,
    ) -> Self {
        Self {
            symbol,
            start_byte,
            end_byte,
            children,
            field_id: None,
            dirty: false,
        }
    }
}

impl Tree {
    /// Create a new tree from a root node
    pub(crate) fn new(root: TreeNode) -> Self {
        Self {
            root,
            language: None,
            source: None,
        }
    }

    /// Get the root node's kind
    pub fn root_kind(&self) -> u32 {
        self.root.symbol
    }

    /// Create a stub tree for testing
    pub fn new_stub() -> Self {
        Self {
            root: TreeNode {
                symbol: 0,
                start_byte: 0,
                end_byte: 0,
                children: vec![],
                field_id: None,
                dirty: false,
            },
            language: None,
            source: None,
        }
    }

    /// Get the root node of the tree
    pub fn root_node(&self) -> Node {
        Node::new(&self.root, self.language.as_ref())
    }

    /// Get the language used to parse this tree
    pub fn language(&self) -> Option<&Language> {
        self.language.as_ref()
    }

    /// Apply an edit to the tree (for incremental parsing)
    #[cfg(feature = "incremental")]
    pub fn edit(&mut self, edit: &crate::InputEdit) {
        fn apply_edit(node: &mut TreeNode, edit: &crate::InputEdit, delta: isize) {
            // Update children first so we process leaves before parents
            for child in &mut node.children {
                apply_edit(child, edit, delta);
            }

            // Determine how this node relates to the edit
            if node.end_byte <= edit.start_byte {
                // Node occurs before the edit range – nothing to do
                return;
            }

            if node.start_byte >= edit.old_end_byte {
                // Node occurs completely after the edit range – shift by delta
                let start = node.start_byte as isize + delta;
                let end = node.end_byte as isize + delta;
                node.start_byte = start.max(0) as usize;
                node.end_byte = end.max(0) as usize;
                return;
            }

            // Node intersects the edited region – mark dirty and expand to
            // cover the new text. Children have already been shifted/marked.
            node.start_byte = node.start_byte.min(edit.start_byte);
            let new_end = node.end_byte as isize + delta;
            node.end_byte = edit.new_end_byte.max(new_end.max(0) as usize);
            node.dirty = true;
        }

        let delta = edit.new_end_byte as isize - edit.old_end_byte as isize;
        apply_edit(&mut self.root, edit, delta);
    }

    /// Get a copy of this tree
    pub fn clone(&self) -> Self {
        Self {
            root: self.root.clone(),
            language: self.language.clone(),
            source: self.source.clone(),
        }
    }

    /// Walk the tree with a callback
    #[allow(dead_code)]
    pub(crate) fn walk<F>(&self, mut callback: F)
    where
        F: FnMut(&TreeNode),
    {
        walk_tree_node(&self.root, &mut callback);
    }
}

#[allow(dead_code)]
fn walk_tree_node<F>(node: &TreeNode, callback: &mut F)
where
    F: FnMut(&TreeNode),
{
    callback(node);
    for child in &node.children {
        walk_tree_node(child, callback);
    }
}

impl fmt::Debug for Tree {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Tree {{ root: {:?} }}", self.root_node())
    }
}

/// Tree cursor for efficient tree traversal
pub struct TreeCursor<'a> {
    /// Stack of nodes representing the path from the root to the current node
    stack: Vec<CursorEntry<'a>>,
    /// Language reference for creating `Node`s
    language: Option<&'a Language>,
}

struct CursorEntry<'a> {
    node: &'a TreeNode,
    /// Index of this node in its parent's child list
    index: usize,
}

impl<'a> TreeCursor<'a> {
    /// Create a new cursor at the root
    pub fn new(tree: &'a Tree) -> Self {
        Self {
            stack: vec![CursorEntry {
                node: &tree.root,
                index: 0,
            }],
            language: tree.language.as_ref(),
        }
    }

    /// Get the node currently pointed to by the cursor
    #[allow(dead_code)]
    pub fn node(&self) -> Node<'a> {
        Node::new(self.stack.last().unwrap().node, self.language)
    }

    /// Move to the first child
    pub fn goto_first_child(&mut self) -> bool {
        let current = self.stack.last().unwrap().node;
        if current.children.is_empty() {
            return false;
        }
        self.stack.push(CursorEntry {
            node: &current.children[0],
            index: 0,
        });
        true
    }

    /// Move to the next sibling
    pub fn goto_next_sibling(&mut self) -> bool {
        if self.stack.len() < 2 {
            return false;
        }
        let parent = self.stack[self.stack.len() - 2].node;
        let next_index = self.stack.last().unwrap().index + 1;
        if next_index >= parent.children.len() {
            return false;
        }
        let top = self.stack.last_mut().unwrap();
        top.node = &parent.children[next_index];
        top.index = next_index;
        true
    }

    /// Move to the parent
    pub fn goto_parent(&mut self) -> bool {
        if self.stack.len() <= 1 {
            return false;
        }
        self.stack.pop();
        true
    }
}
