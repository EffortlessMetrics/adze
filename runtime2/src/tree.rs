//! Tree representation for parsed syntax trees

use crate::{node::Node, Language};
use std::fmt;

/// A parsed syntax tree.
///
/// Represents the result of parsing source code into a structured syntax tree.
/// Supports tree editing for incremental parsing and deep cloning for analysis.
///
/// # Features
///
/// - **Tree-sitter API compatibility**: Provides familiar `root_node()` interface
/// - **Incremental parsing support**: `edit()` method updates ranges for efficient re-parsing
/// - **Deep cloning**: Full tree duplication for analysis and experimentation
/// - **GLR integration**: Works with both deterministic and ambiguous grammars
///
/// # Examples
///
/// Basic usage:
/// ```no_run
/// # use rust_sitter_runtime::{Parser, Tree};
/// # let mut parser = Parser::new();
/// let tree = parser.parse_utf8("fn main() {}", None)?;
/// let root = tree.root_node();
/// println!("Parsed {} with {} children", root.kind(), root.child_count());
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// Tree cloning for analysis:
/// ```no_run
/// # use rust_sitter_runtime::{Tree, InputEdit, Point};
/// # let tree = Tree::new_stub();
/// // Clone for non-destructive analysis
/// let analysis_tree = tree.clone();
///
/// // Original tree unchanged, copy can be modified
/// assert_eq!(tree.root_node().start_byte(), analysis_tree.root_node().start_byte());
/// ```
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

/// Internal tree node representation.
///
/// Represents a single node in the syntax tree with byte range information
/// and child relationships. Supports deep cloning to enable tree duplication
/// for analysis and experimental transformations.
///
/// # Fields
///
/// - `symbol`: The grammar symbol ID for this node type
/// - `start_byte`/`end_byte`: Byte range in the source text
/// - `children`: Child nodes in source order
/// - `field_id`: Optional field name for structured access
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

    /// Set the language for this tree
    pub(crate) fn set_language(&mut self, language: Language) {
        self.language = Some(language);
    }

    /// Set the source bytes for this tree
    pub(crate) fn set_source(&mut self, source: Vec<u8>) {
        self.source = Some(source);
    }

    /// Get the source bytes of this tree, if available
    pub fn source_bytes(&self) -> Option<&[u8]> {
        self.source.as_deref()
    }

    /// Apply an edit to the tree for incremental parsing.
    ///
    /// Updates all node byte ranges affected by the edit while maintaining tree structure
    /// invariants. This is a foundational operation for incremental parsing that enables
    /// efficient re-parsing of modified text.
    ///
    /// # Algorithm
    ///
    /// The editing algorithm processes each node based on its relationship to the edit:
    /// - **Unaffected nodes** (end before edit start): No changes applied
    /// - **Shifted nodes** (start after edit end): Shifted by the edit delta
    /// - **Intersecting nodes**: Range expanded to encompass the edit region
    ///
    /// # Safety
    ///
    /// - Uses saturating arithmetic to prevent integer overflow during range calculations
    /// - Maintains `start_byte <= end_byte` invariants for all nodes
    /// - Handles edge cases like zero-length edits and large deletions safely
    ///
    /// # Arguments
    ///
    /// * `edit` - The input edit describing the text change with old and new byte ranges
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use rust_sitter_runtime::{Tree, InputEdit, Point};
    /// # let mut tree = Tree::new_stub();
    /// // Insert text: changes "hello" to "hello world"
    /// let edit = InputEdit {
    ///     start_byte: 5,      // After "hello"
    ///     old_end_byte: 5,    // No deletion
    ///     new_end_byte: 11,   // " world" added
    ///     start_position: Point { row: 0, column: 5 },
    ///     old_end_position: Point { row: 0, column: 5 },
    ///     new_end_position: Point { row: 0, column: 11 },
    /// };
    ///
    /// #[cfg(feature = "incremental")]
    /// tree.edit(&edit);
    /// // All nodes after byte 5 are now shifted by +6 bytes
    /// ```
    ///
    /// ```no_run
    /// # use rust_sitter_runtime::{Tree, InputEdit, Point};
    /// # let mut tree = Tree::new_stub();
    /// // Replace text: changes "foo" to "bar"
    /// let edit = InputEdit {
    ///     start_byte: 10,
    ///     old_end_byte: 13,   // "foo" removed (3 bytes)
    ///     new_end_byte: 13,   // "bar" added (3 bytes)
    ///     start_position: Point { row: 1, column: 0 },
    ///     old_end_position: Point { row: 1, column: 3 },
    ///     new_end_position: Point { row: 1, column: 3 },
    /// };
    ///
    /// #[cfg(feature = "incremental")]
    /// tree.edit(&edit);
    /// // Nodes intersecting bytes 10-13 are marked for re-parsing
    /// ```
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
                // Bounds checking to prevent integer overflow
                if delta >= 0 {
                    node.start_byte = node.start_byte.saturating_add(delta as usize);
                    node.end_byte = node.end_byte.saturating_add(delta as usize);
                } else {
                    let abs_delta = (-delta) as usize;
                    node.start_byte = node.start_byte.saturating_sub(abs_delta);
                    node.end_byte = node.end_byte.saturating_sub(abs_delta);
                }
            } else {
                // The node intersects the edit; adjust its range to encompass the change.
                if node.start_byte > edit.start_byte {
                    node.start_byte = edit.start_byte;
                }

                if node.end_byte >= edit.old_end_byte {
                    // Bounds checking for intersecting nodes
                    if delta >= 0 {
                        node.end_byte = node.end_byte.saturating_add(delta as usize);
                    } else {
                        let abs_delta = (-delta) as usize;
                        node.end_byte = node.end_byte.saturating_sub(abs_delta);
                    }
                } else if node.end_byte > edit.start_byte {
                    node.end_byte = edit.start_byte;
                }

                // Ensure valid range invariant: start_byte <= end_byte
                if node.start_byte > node.end_byte {
                    node.end_byte = node.start_byte;
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
