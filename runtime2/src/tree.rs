//! Tree representation for parsed syntax trees

use crate::{node::Node, Language};
use std::fmt;

/// A parsed syntax tree
#[derive(Clone)]
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
    /// Whether this node was affected by an edit
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
        fn shift_subtree(node: &mut TreeNode, delta: isize) {
            node.start_byte = (node.start_byte as isize + delta).max(0) as usize;
            node.end_byte = (node.end_byte as isize + delta).max(0) as usize;
            for child in &mut node.children {
                shift_subtree(child, delta);
            }
        }

        fn apply_edit(node: &mut TreeNode, edit: &crate::InputEdit, delta: isize) -> bool {
            // If the node ends before the edit, nothing to do
            if node.end_byte <= edit.start_byte {
                return false;
            }

            // If the node starts after the old end, shift the whole subtree
            if node.start_byte >= edit.old_end_byte {
                shift_subtree(node, delta);
                return false;
            }

            // Otherwise, the edit touches this node
            let mut dirty = true;

            if node.start_byte >= edit.start_byte {
                node.start_byte = (node.start_byte as isize + delta).max(0) as usize;
            }
            if node.end_byte >= edit.old_end_byte {
                node.end_byte = (node.end_byte as isize + delta).max(0) as usize;
            }

            for child in &mut node.children {
                if apply_edit(child, edit, delta) {
                    dirty = true;
                }
            }

            node.dirty = dirty;
            dirty
        }

        let delta = edit.new_end_byte as isize - edit.old_end_byte as isize;
        apply_edit(&mut self.root, edit, delta);
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
pub struct TreeCursor<'tree> {
    stack: Vec<(&'tree TreeNode, usize)>,
    language: Option<&'tree Language>,
}

impl<'tree> TreeCursor<'tree> {
    /// Create a new cursor at the root
    pub fn new(tree: &'tree Tree) -> Self {
        Self {
            stack: vec![(&tree.root, 0)],
            language: tree.language.as_ref(),
        }
    }

    /// Get the current node
    pub fn node(&self) -> Node<'tree> {
        Node::new(self.stack.last().unwrap().0, self.language)
    }

    /// Move to the first child
    pub fn goto_first_child(&mut self) -> bool {
        let node = self.stack.last().unwrap().0;
        if node.children.is_empty() {
            false
        } else {
            let child = &node.children[0];
            self.stack.push((child, 0));
            true
        }
    }

    /// Move to the next sibling
    pub fn goto_next_sibling(&mut self) -> bool {
        if self.stack.len() < 2 {
            return false;
        }
        let (_, idx) = *self.stack.last().unwrap();
        let parent = self.stack[self.stack.len() - 2].0;
        if idx + 1 < parent.children.len() {
            let next = &parent.children[idx + 1];
            self.stack.pop();
            self.stack.push((next, idx + 1));
            true
        } else {
            false
        }
    }

    /// Move to the parent
    pub fn goto_parent(&mut self) -> bool {
        if self.stack.len() <= 1 {
            false
        } else {
            self.stack.pop();
            true
        }
    }
}
