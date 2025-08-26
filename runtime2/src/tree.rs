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
        // TODO: Implement tree editing
        // 1. Update byte offsets in affected nodes
        // 2. Mark dirty regions for re-parsing
        // 3. Maintain tree structure invariants
        let _ = edit;
    }

    /// Get a copy of this tree
    pub fn clone(&self) -> Self {
        // TODO: Implement proper cloning
        Self::new_stub()
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

/// Internal cursor stack entry
#[derive(Clone, Copy)]
struct CursorEntry<'tree> {
    /// Node at this level
    node: &'tree TreeNode,
    /// Index of this node within its parent's children
    index: usize,
}

/// Tree cursor for efficient tree traversal
pub struct TreeCursor<'tree> {
    /// Stack of nodes from root to current position
    stack: Vec<CursorEntry<'tree>>,
}

impl<'tree> TreeCursor<'tree> {
    /// Create a new cursor at the root
    pub fn new(tree: &'tree Tree) -> Self {
        Self {
            stack: vec![CursorEntry {
                node: &tree.root,
                index: 0,
            }],
        }
    }

    /// Move to the first child
    pub fn goto_first_child(&mut self) -> bool {
        if let Some(entry) = self.stack.last() {
            if let Some(child) = entry.node.children.first() {
                self.stack.push(CursorEntry {
                    node: child,
                    index: 0,
                });
                return true;
            }
        }
        false
    }

    /// Move to the next sibling
    pub fn goto_next_sibling(&mut self) -> bool {
        let len = self.stack.len();
        if len < 2 {
            return false;
        }

        // Split the stack to borrow parent immutably and current mutably
        let (parent_slice, current_slice) = self.stack.split_at_mut(len - 1);
        let parent = parent_slice.last().unwrap();
        let current = &mut current_slice[0];
        let next_index = current.index + 1;
        if next_index < parent.node.children.len() {
            current.node = &parent.node.children[next_index];
            current.index = next_index;
            true
        } else {
            false
        }
    }

    /// Move to the parent
    pub fn goto_parent(&mut self) -> bool {
        if self.stack.len() > 1 {
            self.stack.pop();
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build_test_tree() -> Tree {
        let child1 = TreeNode::new_with_children(
            1,
            0,
            0,
            vec![TreeNode::new_with_children(3, 0, 0, vec![])],
        );
        let child2 = TreeNode::new_with_children(2, 0, 0, vec![]);
        let root = TreeNode::new_with_children(0, 0, 0, vec![child1, child2]);
        Tree::new(root)
    }

    #[test]
    fn cursor_traversal() {
        let tree = build_test_tree();
        let mut cursor = TreeCursor::new(&tree);

        // Start at root
        assert_eq!(cursor.stack.last().unwrap().node.symbol, 0);

        // Traverse to first child
        assert!(cursor.goto_first_child());
        assert_eq!(cursor.stack.last().unwrap().node.symbol, 1);

        // Traverse to grandchild
        assert!(cursor.goto_first_child());
        assert_eq!(cursor.stack.last().unwrap().node.symbol, 3);

        // No sibling for grandchild
        assert!(!cursor.goto_next_sibling());

        // Back to first child
        assert!(cursor.goto_parent());
        assert_eq!(cursor.stack.last().unwrap().node.symbol, 1);

        // Move to second child of root
        assert!(cursor.goto_next_sibling());
        assert_eq!(cursor.stack.last().unwrap().node.symbol, 2);

        // Back to root
        assert!(cursor.goto_parent());
        assert_eq!(cursor.stack.last().unwrap().node.symbol, 0);

        // Root has no parent
        assert!(!cursor.goto_parent());
    }
}
