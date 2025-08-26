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
    /// Whether this node has been affected by an edit
    #[cfg(feature = "incremental")]
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
            #[cfg(feature = "incremental")]
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
                #[cfg(feature = "incremental")]
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
        fn shift(node: &mut TreeNode, delta: isize) {
            node.start_byte = (node.start_byte as isize + delta) as usize;
            node.end_byte = (node.end_byte as isize + delta) as usize;
            for child in node.children.iter_mut() {
                shift(child, delta);
            }
        }

        fn apply(node: &mut TreeNode, edit: &crate::InputEdit) {
            let delta = edit.new_end_byte as isize - edit.old_end_byte as isize;

            if node.end_byte <= edit.start_byte {
                // This node is completely before the edit; recurse to children in case
                // they are after the edit (shouldn't happen if invariants hold).
                for child in node.children.iter_mut() {
                    apply(child, edit);
                }
                return;
            }

            if node.start_byte >= edit.old_end_byte {
                // Node is completely after the edit range; shift it forward/backward.
                shift(node, delta);
                return;
            }

            // Node intersects edit. Mark dirty and adjust bounds.
            node.dirty = true;
            if node.start_byte > edit.start_byte {
                node.start_byte = edit.start_byte;
            }
            if node.end_byte >= edit.old_end_byte {
                node.end_byte = (node.end_byte as isize + delta) as usize;
            } else {
                node.end_byte = edit.new_end_byte;
            }

            for child in node.children.iter_mut() {
                apply(child, edit);
            }
        }

        apply(&mut self.root, edit);
    }

    /// Get a copy of this tree
    pub fn clone(&self) -> Self {
        fn clone_node(node: &TreeNode) -> TreeNode {
            TreeNode {
                symbol: node.symbol,
                start_byte: node.start_byte,
                end_byte: node.end_byte,
                children: node.children.iter().map(clone_node).collect(),
                field_id: node.field_id,
                #[cfg(feature = "incremental")]
                dirty: node.dirty,
            }
        }

        Self {
            root: clone_node(&self.root),
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
pub struct TreeCursor {
    // TODO: Implement cursor for efficient traversal
}

impl TreeCursor {
    /// Create a new cursor at the root
    pub fn new(tree: &Tree) -> Self {
        let _ = tree;
        Self {}
    }

    /// Move to the first child
    pub fn goto_first_child(&mut self) -> bool {
        false
    }

    /// Move to the next sibling
    pub fn goto_next_sibling(&mut self) -> bool {
        false
    }

    /// Move to the parent
    pub fn goto_parent(&mut self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Point;

    fn sample_tree() -> Tree {
        let child1 = TreeNode::new_with_children(1, 0, 2, vec![]);
        let child2 = TreeNode::new_with_children(2, 2, 5, vec![]);
        let root = TreeNode::new_with_children(0, 0, 5, vec![child1, child2]);
        Tree::new(root)
    }

    #[test]
    fn clone_deep_copies_tree() {
        let tree = sample_tree();
        let mut cloned = tree.clone();
        // Modify the clone's first child; original should remain unchanged
        cloned.root.children[0].start_byte = 10;
        assert_eq!(tree.root.children[0].start_byte, 0);
    }

    #[cfg(feature = "incremental")]
    #[test]
    fn edit_updates_ranges_and_marks_dirty() {
        let mut tree = sample_tree();
        let edit = crate::InputEdit {
            start_byte: 2,
            old_end_byte: 4,
            new_end_byte: 6,
            start_position: Point::new(0, 2),
            old_end_position: Point::new(0, 4),
            new_end_position: Point::new(0, 6),
        };
        tree.edit(&edit);

        // Root adjusted by +2 at end and marked dirty
        assert_eq!(tree.root.start_byte, 0);
        assert_eq!(tree.root.end_byte, 7);
        assert!(tree.root.dirty);

        // First child unaffected
        let first = &tree.root.children[0];
        assert_eq!(first.start_byte, 0);
        assert_eq!(first.end_byte, 2);
        assert!(!first.dirty);

        // Second child overlaps edit and is marked dirty/shifted
        let second = &tree.root.children[1];
        assert_eq!(second.start_byte, 2);
        assert_eq!(second.end_byte, 7);
        assert!(second.dirty);
    }
}
