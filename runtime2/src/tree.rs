//! Tree representation for parsed syntax trees

use crate::{node::Node, Language};
use std::fmt;

/// Errors that can occur during tree editing operations
#[cfg(feature = "incremental")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EditError {
    /// Invalid byte range in edit operation
    InvalidRange {
        /// Start byte position
        start: usize,
        /// End byte position (old_end or new_end)
        old_end: usize,
    },
    /// Arithmetic overflow during position calculation
    ArithmeticOverflow,
    /// Arithmetic underflow during position calculation  
    ArithmeticUnderflow,
}

#[cfg(feature = "incremental")]
impl fmt::Display for EditError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EditError::InvalidRange { start, old_end } => {
                write!(f, "Invalid edit range: start={}, end={}", start, old_end)
            }
            EditError::ArithmeticOverflow => {
                write!(f, "Arithmetic overflow during tree edit operation")
            }
            EditError::ArithmeticUnderflow => {
                write!(f, "Arithmetic underflow during tree edit operation")
            }
        }
    }
}

#[cfg(feature = "incremental")]
impl std::error::Error for EditError {}

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
    pub(crate) root: TreeNode,
    /// Language used to parse this tree
    pub(crate) language: Option<Language>,
    /// Source text (optional, for convenience)
    #[allow(dead_code)]
    pub(crate) source: Option<Vec<u8>>,
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
    /// Whether this node has been affected by an edit
    #[cfg(feature = "incremental")]
    dirty: bool,
}

impl TreeNode {
    /// Create a new tree node with children
    #[allow(dead_code)]
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
    #[allow(dead_code)]
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
                #[cfg(feature = "incremental")]
                dirty: false,
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

    /// Apply an edit to the tree (for incremental parsing) - Enhanced with comprehensive error handling
    #[cfg(feature = "incremental")]
    pub fn edit(&mut self, edit: &crate::InputEdit) -> Result<(), EditError> {
        // Validate edit parameters upfront
        if edit.old_end_byte < edit.start_byte {
            return Err(EditError::InvalidRange {
                start: edit.start_byte,
                old_end: edit.old_end_byte,
            });
        }
        if edit.new_end_byte < edit.start_byte {
            return Err(EditError::InvalidRange {
                start: edit.start_byte,
                old_end: edit.new_end_byte,
            });
        }

        fn safe_shift(node: &mut TreeNode, delta: isize) -> Result<(), EditError> {
            // Use checked arithmetic to prevent overflow
            if delta >= 0 {
                let positive_delta = delta as usize;
                node.start_byte = node
                    .start_byte
                    .checked_add(positive_delta)
                    .ok_or(EditError::ArithmeticOverflow)?;
                node.end_byte = node
                    .end_byte
                    .checked_add(positive_delta)
                    .ok_or(EditError::ArithmeticOverflow)?;
            } else {
                let negative_delta = (-delta) as usize;
                node.start_byte = node
                    .start_byte
                    .checked_sub(negative_delta)
                    .ok_or(EditError::ArithmeticUnderflow)?;
                node.end_byte = node
                    .end_byte
                    .checked_sub(negative_delta)
                    .ok_or(EditError::ArithmeticUnderflow)?;
            }

            for child in node.children.iter_mut() {
                safe_shift(child, delta)?;
            }
            Ok(())
        }

        fn safe_apply(node: &mut TreeNode, edit: &crate::InputEdit) -> Result<(), EditError> {
            // Calculate delta with overflow protection
            let delta = if edit.new_end_byte >= edit.old_end_byte {
                (edit.new_end_byte - edit.old_end_byte) as isize
            } else {
                -((edit.old_end_byte - edit.new_end_byte) as isize)
            };

            if node.end_byte <= edit.start_byte {
                // This node is completely before the edit; recurse to children in case
                // they are after the edit (shouldn't happen if invariants hold).
                for child in node.children.iter_mut() {
                    safe_apply(child, edit)?;
                }
                return Ok(());
            }

            if node.start_byte >= edit.old_end_byte {
                // Node is completely after the edit range; shift it forward/backward.
                safe_shift(node, delta)?;
                return Ok(());
            }

            // Node intersects edit. Mark dirty and adjust bounds.
            #[cfg(feature = "incremental")]
            {
                node.dirty = true;
            }
            if node.start_byte > edit.start_byte {
                node.start_byte = edit.start_byte;
            }
            if node.end_byte >= edit.old_end_byte {
                // Use safe arithmetic for end_byte adjustment
                if delta >= 0 {
                    node.end_byte = node
                        .end_byte
                        .checked_add(delta as usize)
                        .ok_or(EditError::ArithmeticOverflow)?;
                } else {
                    node.end_byte = node
                        .end_byte
                        .checked_sub((-delta) as usize)
                        .ok_or(EditError::ArithmeticUnderflow)?;
                }
            } else {
                node.end_byte = edit.new_end_byte;
            }

            for child in node.children.iter_mut() {
                safe_apply(child, edit)?;
            }
            Ok(())
        }

        safe_apply(&mut self.root, edit)?;
        self.last_edit = Some(*edit);
        Ok(())
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

    use crate::Point;

    #[cfg(feature = "incremental")]
    use super::EditError;

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
        tree.edit(&edit).expect("Edit should succeed");

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

    #[cfg(feature = "incremental")]
    #[test]
    fn edit_handles_edge_cases_safely() {
        let mut tree = sample_tree();

        // Test 1: Invalid range (old_end < start)
        let invalid_edit = crate::InputEdit {
            start_byte: 5,
            old_end_byte: 3, // Invalid: less than start
            new_end_byte: 8,
            start_position: Point::new(0, 5),
            old_end_position: Point::new(0, 3),
            new_end_position: Point::new(0, 8),
        };
        assert!(matches!(
            tree.edit(&invalid_edit),
            Err(EditError::InvalidRange { .. })
        ));

        // Test 2: Large deletion (negative delta)
        let large_deletion = crate::InputEdit {
            start_byte: 1,
            old_end_byte: 10,
            new_end_byte: 1, // Deletes 9 characters
            start_position: Point::new(0, 1),
            old_end_position: Point::new(0, 10),
            new_end_position: Point::new(0, 1),
        };
        tree.edit(&large_deletion)
            .expect("Large deletion should succeed");

        // Test 3: Zero-length edit (insertion)
        let mut tree2 = sample_tree();
        let insertion = crate::InputEdit {
            start_byte: 3,
            old_end_byte: 3, // Zero-length edit
            new_end_byte: 8, // Insert 5 characters
            start_position: Point::new(0, 3),
            old_end_position: Point::new(0, 3),
            new_end_position: Point::new(0, 8),
        };
        tree2.edit(&insertion).expect("Insertion should succeed");
        assert_eq!(tree2.root.end_byte, 10); // Original 5 + 5 inserted
    }

    #[cfg(feature = "incremental")]
    #[test]
    fn edit_validates_input_ranges() {
        let mut tree = sample_tree();

        // Test invalid range where old_end < start
        let invalid_edit = crate::InputEdit {
            start_byte: 10,
            old_end_byte: 5, // Invalid: less than start
            new_end_byte: 8,
            start_position: Point::new(0, 10),
            old_end_position: Point::new(0, 5),
            new_end_position: Point::new(0, 8),
        };

        let result = tree.edit(&invalid_edit);
        assert!(matches!(
            result,
            Err(EditError::InvalidRange {
                start: 10,
                old_end: 5
            })
        ));

        // Test invalid range where new_end < start
        let invalid_edit2 = crate::InputEdit {
            start_byte: 10,
            old_end_byte: 15,
            new_end_byte: 5, // Invalid: less than start
            start_position: Point::new(0, 10),
            old_end_position: Point::new(0, 15),
            new_end_position: Point::new(0, 5),
        };

        let result2 = tree.edit(&invalid_edit2);
        assert!(matches!(
            result2,
            Err(EditError::InvalidRange {
                start: 10,
                old_end: 5
            })
        ));
    }

    #[cfg(feature = "incremental")]
    #[test]
    fn edit_underflow_protection() {
        let mut tree = Tree::new(TreeNode::new_with_children(
            0,
            10,
            50,
            vec![TreeNode::new_with_children(1, 20, 30, vec![])],
        ));

        // Large deletion that would cause underflow in naive implementation
        let underflow_edit = crate::InputEdit {
            start_byte: 5,
            old_end_byte: 60, // Delete more than what exists
            new_end_byte: 6,  // Replace with small content
            start_position: Point::new(0, 5),
            old_end_position: Point::new(0, 60),
            new_end_position: Point::new(0, 6),
        };

        // The edit should either succeed safely or return underflow error
        let result = tree.edit(&underflow_edit);
        if let Err(EditError::ArithmeticUnderflow) = result {
            // Expected case - underflow was detected and prevented
        } else {
            // If it succeeds, verify bounds are reasonable
            result.expect("Edit should either succeed or return underflow error");
            assert!(tree.root.start_byte <= tree.root.end_byte);
        }
    }

    #[cfg(feature = "incremental")]
    #[test]
    fn edit_recursive_safety() {
        // Test deep tree to ensure recursive operations are bounded
        let mut deep_tree = TreeNode::new_with_children(0, 0, 100, vec![]);

        // Create a deep nested structure
        let mut current = &mut deep_tree;
        for i in 1..50 {
            let child =
                TreeNode::new_with_children(i, (i * 2) as usize, (i * 2 + 10) as usize, vec![]);
            current.children.push(child);
            current = &mut current.children[0];
        }

        let mut tree = Tree::new(deep_tree);

        // Apply edit that affects the entire tree
        let edit = crate::InputEdit {
            start_byte: 0,
            old_end_byte: 5,
            new_end_byte: 10,
            start_position: Point::new(0, 0),
            old_end_position: Point::new(0, 5),
            new_end_position: Point::new(0, 10),
        };

        // This should complete without stack overflow
        tree.edit(&edit).expect("Deep tree edit should succeed");
        assert!(tree.root.dirty);
    }
}
