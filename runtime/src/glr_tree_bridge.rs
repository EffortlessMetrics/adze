// Bridge between GLR Subtree and Tree-sitter Node/Tree structures
// This module provides conversion and compatibility layer

use crate::subtree::Subtree;
use rust_sitter_ir::Grammar;
use std::sync::Arc;
use std::collections::HashMap;

// Re-export the tree-sitter types based on feature
#[cfg(all(feature = "tree-sitter-standard", not(feature = "tree-sitter-c2rust")))]
use tree_sitter_runtime_standard as tree_sitter;

#[cfg(all(feature = "tree-sitter-c2rust", not(feature = "tree-sitter-standard")))]
use tree_sitter_runtime_c2rust as tree_sitter;

// Provide a default for when no features are enabled
#[cfg(not(any(feature = "tree-sitter-standard", feature = "tree-sitter-c2rust")))]
mod tree_sitter {
    #[allow(dead_code)]
    pub struct Node;
    #[allow(dead_code)]
    pub struct Tree;
}

/// A Tree-sitter compatible tree structure built from GLR Subtree
pub struct GLRTree {
    /// Root subtree from GLR parser
    root: Arc<Subtree>,
    /// Source text
    source: Vec<u8>,
    /// Grammar for symbol information
    grammar: Grammar,
    /// Map from Subtree pointer to node ID
    node_map: HashMap<usize, usize>,
    /// Next node ID
    next_node_id: usize,
}

impl GLRTree {
    /// Create a new GLR tree from a subtree
    pub fn new(root: Arc<Subtree>, source: Vec<u8>, grammar: Grammar) -> Self {
        let mut tree = Self {
            root,
            source,
            grammar,
            node_map: HashMap::new(),
            next_node_id: 0,
        };
        
        // Build node map
        tree.build_node_map(&tree.root.clone());
        tree
    }
    
    /// Build a map from subtree pointers to node IDs
    fn build_node_map(&mut self, subtree: &Arc<Subtree>) {
        let ptr = Arc::as_ptr(subtree) as usize;
        if !self.node_map.contains_key(&ptr) {
            self.node_map.insert(ptr, self.next_node_id);
            self.next_node_id += 1;
            
            for child in &subtree.children {
                self.build_node_map(child);
            }
        }
    }
    
    /// Get root node
    pub fn root_node(&self) -> GLRNode {
        GLRNode {
            subtree: self.root.clone(),
            tree: self,
        }
    }
    
    /// Get the language (grammar)
    pub fn language(&self) -> &Grammar {
        &self.grammar
    }
    
    /// Get source text
    pub fn text(&self) -> &[u8] {
        &self.source
    }
}

/// A node in the GLR tree that provides Tree-sitter-like API
pub struct GLRNode<'tree> {
    subtree: Arc<Subtree>,
    tree: &'tree GLRTree,
}

impl<'tree> GLRNode<'tree> {
    /// Get the node's type (symbol name)
    pub fn kind(&self) -> &str {
        // Look up symbol name from grammar
        if let Some(name) = self.tree.grammar.rule_names.get(&self.subtree.node.symbol_id) {
            name
        } else if let Some(token) = self.tree.grammar.tokens.get(&self.subtree.node.symbol_id) {
            &token.name
        } else {
            "unknown"
        }
    }
    
    /// Get the node's symbol ID
    pub fn symbol(&self) -> u16 {
        self.subtree.node.symbol_id.0
    }
    
    /// Get start byte
    pub fn start_byte(&self) -> usize {
        self.subtree.node.byte_range.start
    }
    
    /// Get end byte
    pub fn end_byte(&self) -> usize {
        self.subtree.node.byte_range.end
    }
    
    /// Get byte range
    pub fn byte_range(&self) -> std::ops::Range<usize> {
        self.subtree.node.byte_range.clone()
    }
    
    /// Check if this node is an error
    pub fn is_error(&self) -> bool {
        self.subtree.node.is_error
    }
    
    /// Check if this node has errors (including descendants)
    pub fn has_error(&self) -> bool {
        if self.is_error() {
            return true;
        }
        
        self.subtree.children.iter().any(|child| {
            GLRNode {
                subtree: child.clone(),
                tree: self.tree,
            }.has_error()
        })
    }
    
    /// Get child count
    pub fn child_count(&self) -> usize {
        self.subtree.children.len()
    }
    
    /// Get child at index
    pub fn child(&self, index: usize) -> Option<GLRNode<'tree>> {
        self.subtree.children.get(index).map(|child| GLRNode {
            subtree: child.clone(),
            tree: self.tree,
        })
    }
    
    /// Get all children
    pub fn children(&self) -> impl Iterator<Item = GLRNode<'tree>> {
        let tree = self.tree;
        self.subtree.children.iter().map(move |child| GLRNode {
            subtree: child.clone(),
            tree,
        })
    }
    
    /// Get text for this node
    pub fn utf8_text<'a>(&self, source: &'a [u8]) -> Result<&'a str, std::str::Utf8Error> {
        let range = self.byte_range();
        std::str::from_utf8(&source[range])
    }
    
    /// Get parent node (not implemented - would require parent tracking)
    pub fn parent(&self) -> Option<GLRNode<'tree>> {
        // Tree-sitter nodes track parent pointers, but our Subtrees don't
        // This would require a different tree structure or parent map
        None
    }
    
    /// Create a tree cursor starting at this node
    pub fn walk(&self) -> GLRTreeCursor<'tree> {
        GLRTreeCursor::new(self.clone())
    }
    
    /// Get node ID (for comparison)
    pub fn id(&self) -> usize {
        let ptr = Arc::as_ptr(&self.subtree) as usize;
        *self.tree.node_map.get(&ptr).unwrap_or(&0)
    }
}

impl<'tree> Clone for GLRNode<'tree> {
    fn clone(&self) -> Self {
        Self {
            subtree: self.subtree.clone(),
            tree: self.tree,
        }
    }
}

impl<'tree> PartialEq for GLRNode<'tree> {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.subtree, &other.subtree)
    }
}

impl<'tree> std::fmt::Debug for GLRNode<'tree> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GLRNode")
            .field("kind", &self.kind())
            .field("symbol", &self.symbol())
            .field("range", &self.byte_range())
            .field("children", &self.child_count())
            .finish()
    }
}

/// Tree cursor for traversing GLR trees
pub struct GLRTreeCursor<'tree> {
    /// Stack of (node, child_index) for traversal
    stack: Vec<(GLRNode<'tree>, usize)>,
}

impl<'tree> GLRTreeCursor<'tree> {
    /// Create a new cursor at the given node
    pub fn new(node: GLRNode<'tree>) -> Self {
        Self {
            stack: vec![(node, 0)],
        }
    }
    
    /// Get current node
    pub fn node(&self) -> GLRNode<'tree> {
        self.stack.last().unwrap().0.clone()
    }
    
    /// Go to first child
    pub fn goto_first_child(&mut self) -> bool {
        if let Some((current, _)) = self.stack.last() {
            if current.child_count() > 0 {
                if let Some(child) = current.child(0) {
                    self.stack.push((child, 0));
                    return true;
                }
            }
        }
        false
    }
    
    /// Go to next sibling
    pub fn goto_next_sibling(&mut self) -> bool {
        if self.stack.len() <= 1 {
            return false;
        }
        
        if let Some((_, index)) = self.stack.pop() {
            if let Some((parent, _)) = self.stack.last() {
                let next_index = index + 1;
                if next_index < parent.child_count() {
                    if let Some(sibling) = parent.child(next_index) {
                        self.stack.push((sibling, next_index));
                        return true;
                    }
                }
            }
        }
        false
    }
    
    /// Go to parent
    pub fn goto_parent(&mut self) -> bool {
        if self.stack.len() > 1 {
            self.stack.pop();
            true
        } else {
            false
        }
    }
    
    /// Reset cursor to a node
    pub fn reset(&mut self, node: GLRNode<'tree>) {
        self.stack.clear();
        self.stack.push((node, 0));
    }
    
    /// Get field name of current node (not implemented - would require field tracking)
    pub fn field_name(&self) -> Option<&str> {
        // This would require tracking field information during parsing
        None
    }
}

/// Convert a GLR Subtree to a Tree-sitter compatible tree
pub fn subtree_to_tree(
    subtree: Arc<Subtree>,
    source: Vec<u8>,
    grammar: Grammar,
) -> GLRTree {
    GLRTree::new(subtree, source, grammar)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_glr_node_api() {
        // Create a simple subtree
        let root = Arc::new(Subtree::new(
            SubtreeNode {
                symbol_id: SymbolId(1),
                is_error: false,
                byte_range: 0..10,
            },
            vec![
                Arc::new(Subtree::new(
                    SubtreeNode {
                        symbol_id: SymbolId(2),
                        is_error: false,
                        byte_range: 0..5,
                    },
                    vec![],
                )),
                Arc::new(Subtree::new(
                    SubtreeNode {
                        symbol_id: SymbolId(3),
                        is_error: false,
                        byte_range: 5..10,
                    },
                    vec![],
                )),
            ],
        ));
        
        let source = b"hello world".to_vec();
        let mut grammar = Grammar::new("test".to_string());
        grammar.rule_names.insert(SymbolId(1), "root".to_string());
        grammar.rule_names.insert(SymbolId(2), "left".to_string());
        grammar.rule_names.insert(SymbolId(3), "right".to_string());
        
        let tree = GLRTree::new(root, source, grammar);
        let root_node = tree.root_node();
        
        // Test node API
        assert_eq!(root_node.kind(), "root");
        assert_eq!(root_node.symbol(), 1);
        assert_eq!(root_node.start_byte(), 0);
        assert_eq!(root_node.end_byte(), 10);
        assert!(!root_node.is_error());
        assert_eq!(root_node.child_count(), 2);
        
        // Test children
        let child1 = root_node.child(0).unwrap();
        assert_eq!(child1.kind(), "left");
        assert_eq!(child1.byte_range(), 0..5);
        
        let child2 = root_node.child(1).unwrap();
        assert_eq!(child2.kind(), "right");
        assert_eq!(child2.byte_range(), 5..10);
    }
    
    #[test]
    fn test_tree_cursor() {
        let root = Arc::new(Subtree::new(
            SubtreeNode {
                symbol_id: SymbolId(1),
                is_error: false,
                byte_range: 0..20,
            },
            vec![
                Arc::new(Subtree::new(
                    SubtreeNode {
                        symbol_id: SymbolId(2),
                        is_error: false,
                        byte_range: 0..10,
                    },
                    vec![
                        Arc::new(Subtree::new(
                            SubtreeNode {
                                symbol_id: SymbolId(4),
                                is_error: false,
                                byte_range: 0..5,
                            },
                            vec![],
                        )),
                    ],
                )),
                Arc::new(Subtree::new(
                    SubtreeNode {
                        symbol_id: SymbolId(3),
                        is_error: false,
                        byte_range: 10..20,
                    },
                    vec![],
                )),
            ],
        ));
        
        let tree = GLRTree::new(root, vec![], Grammar::new("test".to_string()));
        let mut cursor = tree.root_node().walk();
        
        // Test cursor navigation
        assert_eq!(cursor.node().symbol(), 1);
        
        // Go to first child
        assert!(cursor.goto_first_child());
        assert_eq!(cursor.node().symbol(), 2);
        
        // Go to grandchild
        assert!(cursor.goto_first_child());
        assert_eq!(cursor.node().symbol(), 4);
        
        // Can't go deeper
        assert!(!cursor.goto_first_child());
        
        // Go back to parent
        assert!(cursor.goto_parent());
        assert_eq!(cursor.node().symbol(), 2);
        
        // Go to sibling
        assert!(cursor.goto_next_sibling());
        assert_eq!(cursor.node().symbol(), 3);
        
        // No more siblings
        assert!(!cursor.goto_next_sibling());
    }
}