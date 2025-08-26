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
    /// Last edit applied to this tree (for incremental parsing)
    #[cfg(feature = "incremental")]
    last_edit: Option<crate::InputEdit>,
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
            #[cfg(feature = "incremental")]
            last_edit: None,
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
            #[cfg(feature = "incremental")]
            last_edit: None,
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
        let delta = edit.new_end_byte as isize - edit.old_end_byte as isize;

        fn apply_edit(node: &mut TreeNode, edit: &crate::InputEdit, delta: isize) {
            // If the node ends before the edit start, it's unaffected.
            if node.end_byte <= edit.start_byte {
                return;
            }

            // If the node starts after the old edit end, shift it by the delta.
            if node.start_byte >= edit.old_end_byte {
                node.start_byte = (node.start_byte as isize + delta) as usize;
                node.end_byte = (node.end_byte as isize + delta) as usize;
            } else {
                // The node intersects the edit; adjust its range to encompass the change.
                if node.start_byte > edit.start_byte {
                    node.start_byte = edit.start_byte;
                }

                if node.end_byte >= edit.old_end_byte {
                    node.end_byte = (node.end_byte as isize + delta) as usize;
                } else if node.end_byte > edit.start_byte {
                    node.end_byte = edit.start_byte;
                }
            }

            for child in &mut node.children {
                apply_edit(child, edit, delta);
            }
        }

        apply_edit(&mut self.root, edit, delta);

        // Record the last edit so incremental parsing can reparse this region.
        self.last_edit = Some(*edit);
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
