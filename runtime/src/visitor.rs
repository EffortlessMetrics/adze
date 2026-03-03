//! Visitor utilities for traversing parsed syntax trees.
//!
//! This module provides the [`Visitor`](crate::visitor::Visitor) trait for
//! depth-first traversal and several ready-made walkers and visitors:
//!
//! - [`TreeWalker`](crate::visitor::TreeWalker) — depth-first traversal
//! - [`BreadthFirstWalker`](crate::visitor::BreadthFirstWalker) — breadth-first
//!   (level-order) traversal
//! - [`TransformWalker`](crate::visitor::TransformWalker) /
//!   [`TransformVisitor`](crate::visitor::TransformVisitor) — bottom-up tree
//!   transformation
//! - [`StatsVisitor`](crate::visitor::StatsVisitor) — collects node counts and
//!   tree depth
//! - [`SearchVisitor`](crate::visitor::SearchVisitor) — finds nodes matching a
//!   predicate
//! - [`PrettyPrintVisitor`](crate::visitor::PrettyPrintVisitor) — produces an
//!   indented text representation
#![cfg_attr(feature = "strict_docs", allow(missing_docs))]

// Parse tree visitor API for Adze
// This module provides flexible visitor patterns for traversing and analyzing parse trees

#[cfg(feature = "pure-rust")]
use crate::pure_parser::ParsedNode as Node;
#[cfg(not(feature = "pure-rust"))]
use crate::tree_sitter::Node;

use std::collections::VecDeque;

/// Trait for visiting nodes in a parse tree.
///
/// Implement one or more of the provided methods to react to specific node
/// types during traversal. All methods have default no-op implementations so
/// you only need to override the ones you care about.
///
/// # Traversal control
///
/// [`enter_node`](Self::enter_node) returns a [`VisitorAction`] that controls
/// whether traversal continues into children, skips them, or stops entirely.
pub trait Visitor {
    /// Called when a node is first entered during traversal.
    ///
    /// Return [`VisitorAction::Continue`] to visit children,
    /// [`VisitorAction::SkipChildren`] to skip them, or
    /// [`VisitorAction::Stop`] to halt the walk.
    fn enter_node(&mut self, _node: &Node) -> VisitorAction {
        VisitorAction::Continue
    }

    /// Called after all of a node's children have been visited.
    fn leave_node(&mut self, _node: &Node) {
        // Default: do nothing
    }

    /// Called for leaf nodes (nodes with no children). `text` is the source
    /// text spanned by the node.
    fn visit_leaf(&mut self, _node: &Node, _text: &str) {
        // Default: do nothing
    }

    /// Called for nodes that represent parse errors.
    fn visit_error(&mut self, _node: &Node) {
        // Default: do nothing
    }
}

/// Controls how traversal proceeds after visiting a node.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VisitorAction {
    /// Continue traversal into this node's children.
    Continue,
    /// Skip this node's children but continue with its siblings.
    SkipChildren,
    /// Stop the entire traversal immediately.
    Stop,
}

/// Walks a parse tree in depth-first (pre-order) fashion, invoking a
/// [`Visitor`] at each node.
///
/// # Examples
///
/// ```ignore
/// use adze::visitor::{TreeWalker, StatsVisitor};
///
/// let walker = TreeWalker::new(source.as_bytes());
/// let mut stats = StatsVisitor::default();
/// walker.walk(&root_node, &mut stats);
/// println!("Total nodes: {}", stats.total_nodes);
/// ```
pub struct TreeWalker<'a> {
    source: &'a [u8],
}

impl<'a> TreeWalker<'a> {
    /// Creates a new depth-first walker for the given source bytes.
    pub fn new(source: &'a [u8]) -> Self {
        Self { source }
    }

    #[cfg(feature = "pure-rust")]
    fn get_node_text(&self, node: &Node) -> String {
        let start = node.start_byte();
        let end = node.end_byte();
        if start < self.source.len() && end <= self.source.len() && start < end {
            std::str::from_utf8(&self.source[start..end])
                .unwrap_or("")
                .to_string()
        } else {
            String::new()
        }
    }

    /// Walk the tree depth-first with the given visitor
    #[cfg(not(feature = "pure-rust"))]
    pub fn walk<V: Visitor>(&self, root: Node, visitor: &mut V) {
        self.walk_node(root, visitor);
    }

    /// Walk the tree depth-first with the given visitor
    #[cfg(feature = "pure-rust")]
    pub fn walk<V: Visitor>(&self, root: &Node, visitor: &mut V) {
        self.walk_node(root, visitor);
    }

    #[cfg(not(feature = "pure-rust"))]
    fn walk_node<V: Visitor>(&self, node: Node, visitor: &mut V) {
        // Handle special node types
        if node.is_error() {
            visitor.visit_error(&node);
            return;
        }

        // Enter the node
        let action = visitor.enter_node(&node);

        match action {
            VisitorAction::Stop => return,
            VisitorAction::SkipChildren => {
                visitor.leave_node(&node);
                return;
            }
            VisitorAction::Continue => {}
        }

        // Process children or leaf content
        if node.child_count() == 0 {
            if let Ok(text) = node.utf8_text(self.source) {
                visitor.visit_leaf(&node, text);
            }
        } else {
            let mut cursor = node.walk();
            if cursor.goto_first_child() {
                loop {
                    self.walk_node(cursor.node(), visitor);
                    if !cursor.goto_next_sibling() {
                        break;
                    }
                }
            }
        }

        // Leave the node
        visitor.leave_node(&node);
    }

    #[cfg(feature = "pure-rust")]
    fn walk_node<V: Visitor>(&self, node: &Node, visitor: &mut V) {
        // Handle special node types
        if node.is_error() {
            visitor.visit_error(node);
            return;
        }

        // Enter the node
        let action = visitor.enter_node(node);

        match action {
            VisitorAction::Stop => return,
            VisitorAction::SkipChildren => {
                visitor.leave_node(node);
                return;
            }
            VisitorAction::Continue => {}
        }

        // Process children or leaf content
        if node.child_count() == 0 {
            let text = self.get_node_text(node);
            visitor.visit_leaf(node, &text);
        } else {
            for child in node.children() {
                self.walk_node(child, visitor);
            }
        }

        // Leave the node
        visitor.leave_node(node);
    }
}

/// Walks a parse tree in breadth-first (level-order) fashion, invoking a
/// [`Visitor`] at each node.
pub struct BreadthFirstWalker<'a> {
    source: &'a [u8],
}

impl<'a> BreadthFirstWalker<'a> {
    /// Creates a new breadth-first walker for the given source bytes.
    pub fn new(source: &'a [u8]) -> Self {
        Self { source }
    }

    /// Walk the tree breadth-first with the given visitor
    #[cfg(not(feature = "pure-rust"))]
    pub fn walk<V: Visitor>(&self, root: Node, visitor: &mut V) {
        let mut queue = VecDeque::new();
        queue.push_back(root);

        while let Some(node) = queue.pop_front() {
            // Handle special node types
            if node.is_error() {
                visitor.visit_error(&node);
                continue;
            }

            // Visit the node
            let action = visitor.enter_node(&node);

            match action {
                VisitorAction::Stop => return,
                VisitorAction::SkipChildren => continue,
                VisitorAction::Continue => {}
            }

            // Process leaf or queue children
            if node.child_count() == 0 {
                if let Ok(text) = node.utf8_text(self.source) {
                    visitor.visit_leaf(&node, text);
                }
            } else {
                let mut cursor = node.walk();
                if cursor.goto_first_child() {
                    loop {
                        queue.push_back(cursor.node());
                        if !cursor.goto_next_sibling() {
                            break;
                        }
                    }
                }
            }
        }
    }

    /// Walk the tree breadth-first with the given visitor
    #[cfg(feature = "pure-rust")]
    pub fn walk<V: Visitor>(&self, root: &Node, visitor: &mut V) {
        let mut queue = VecDeque::new();
        queue.push_back(root);

        while let Some(node) = queue.pop_front() {
            // Handle special node types
            if node.is_error() {
                visitor.visit_error(node);
                continue;
            }

            // Visit the node
            let action = visitor.enter_node(node);

            match action {
                VisitorAction::Stop => return,
                VisitorAction::SkipChildren => continue,
                VisitorAction::Continue => {}
            }

            // Process leaf or queue children
            if node.child_count() == 0 {
                let text = &self.source[node.start_byte()..node.end_byte()];
                if let Ok(text_str) = std::str::from_utf8(text) {
                    visitor.visit_leaf(node, text_str);
                }
            } else {
                for child in node.children() {
                    queue.push_back(child);
                }
            }
        }
    }
}

/// A [`Visitor`] that collects statistics about the parse tree.
///
/// After a walk, inspect [`total_nodes`](Self::total_nodes),
/// [`leaf_nodes`](Self::leaf_nodes), [`error_nodes`](Self::error_nodes),
/// [`max_depth`](Self::max_depth), and per-kind counts in
/// [`node_counts`](Self::node_counts).
#[derive(Debug, Default)]
pub struct StatsVisitor {
    /// Total number of nodes visited.
    pub total_nodes: usize,
    /// Number of leaf (childless) nodes.
    pub leaf_nodes: usize,
    /// Number of error nodes.
    pub error_nodes: usize,
    /// Maximum depth reached during traversal.
    pub max_depth: usize,
    /// Per-kind node counts.
    pub node_counts: std::collections::HashMap<String, usize>,
    current_depth: usize,
}

impl Visitor for StatsVisitor {
    fn enter_node(&mut self, node: &Node) -> VisitorAction {
        self.total_nodes += 1;
        self.current_depth += 1;
        self.max_depth = self.max_depth.max(self.current_depth);

        let kind = node.kind();
        *self.node_counts.entry(kind.to_string()).or_insert(0) += 1;

        VisitorAction::Continue
    }

    fn leave_node(&mut self, _node: &Node) {
        self.current_depth -= 1;
    }

    fn visit_leaf(&mut self, _node: &Node, _text: &str) {
        self.leaf_nodes += 1;
    }

    fn visit_error(&mut self, _node: &Node) {
        self.error_nodes += 1;
    }
}

/// A [`Visitor`] that records nodes matching a user-supplied predicate.
///
/// After the walk, matching nodes are stored in [`matches`](Self::matches) as
/// `(start_byte, end_byte, kind)` tuples.
pub struct SearchVisitor<F> {
    predicate: F,
    /// Matched nodes as `(start_byte, end_byte, kind)` tuples.
    pub matches: Vec<(usize, usize, String)>,
}

impl<F> SearchVisitor<F>
where
    F: Fn(&Node) -> bool,
{
    /// Creates a new search visitor with the given predicate.
    pub fn new(predicate: F) -> Self {
        Self {
            predicate,
            matches: Vec::new(),
        }
    }
}

impl<F> Visitor for SearchVisitor<F>
where
    F: Fn(&Node) -> bool,
{
    fn enter_node(&mut self, node: &Node) -> VisitorAction {
        if (self.predicate)(node) {
            self.matches
                .push((node.start_byte(), node.end_byte(), node.kind().to_string()));
        }
        VisitorAction::Continue
    }
}

/// A [`Visitor`] that produces an indented, human-readable representation of
/// the parse tree.
///
/// After the walk, call [`output`](Self::output) to retrieve the formatted
/// string.
pub struct PrettyPrintVisitor {
    indent: usize,
    output: String,
}

impl Default for PrettyPrintVisitor {
    fn default() -> Self {
        Self::new()
    }
}

impl PrettyPrintVisitor {
    /// Creates a new pretty-print visitor with no accumulated output.
    pub fn new() -> Self {
        Self {
            indent: 0,
            output: String::new(),
        }
    }

    /// Returns the accumulated pretty-printed output.
    pub fn output(&self) -> &str {
        &self.output
    }
}

impl Visitor for PrettyPrintVisitor {
    fn enter_node(&mut self, node: &Node) -> VisitorAction {
        let indent_str = "  ".repeat(self.indent);
        self.output
            .push_str(&format!("{}{}", indent_str, node.kind()));

        if node.is_named() {
            self.output.push_str(" [named]");
        }

        // Field names not directly accessible on Node

        self.output.push('\n');
        self.indent += 1;

        VisitorAction::Continue
    }

    fn leave_node(&mut self, _node: &Node) {
        self.indent -= 1;
    }

    fn visit_leaf(&mut self, _node: &Node, text: &str) {
        let indent_str = "  ".repeat(self.indent);
        self.output.push_str(&format!("{}\"{}\"", indent_str, text));

        // Field names not directly accessible on Node

        self.output.push('\n');
    }

    fn visit_error(&mut self, node: &Node) {
        let indent_str = "  ".repeat(self.indent);
        self.output
            .push_str(&format!("{}ERROR: {}\n", indent_str, node.kind()));
    }
}

/// Trait for bottom-up tree transformations.
///
/// Unlike [`Visitor`], a `TransformVisitor` produces a value at every node.
/// Children are transformed first and their results are passed to
/// [`transform_node`](Self::transform_node).
pub trait TransformVisitor {
    /// The type produced by each node transformation.
    type Output;

    /// Transforms an interior node given its already-transformed children.
    fn transform_node(&mut self, node: &Node, children: Vec<Self::Output>) -> Self::Output;

    /// Transforms a leaf node (no children).
    fn transform_leaf(&mut self, node: &Node, text: &str) -> Self::Output;

    /// Transforms an error node.
    fn transform_error(&mut self, node: &Node) -> Self::Output;
}

/// Applies a [`TransformVisitor`] to a parse tree in post-order.
pub struct TransformWalker<'a> {
    source: &'a [u8],
}

impl<'a> TransformWalker<'a> {
    /// Creates a new transform walker for the given source bytes.
    pub fn new(source: &'a [u8]) -> Self {
        Self { source }
    }

    #[cfg(not(feature = "pure-rust"))]
    pub fn walk<T: TransformVisitor>(&self, root: Node, visitor: &mut T) -> T::Output {
        self.transform_node(root, visitor)
    }

    #[cfg(feature = "pure-rust")]
    pub fn walk<T: TransformVisitor>(&self, root: &Node, visitor: &mut T) -> T::Output {
        self.transform_node(root, visitor)
    }

    #[cfg(not(feature = "pure-rust"))]
    fn transform_node<T: TransformVisitor>(&self, node: Node, visitor: &mut T) -> T::Output {
        if node.is_error() {
            return visitor.transform_error(&node);
        }

        if node.child_count() == 0 {
            let text = node.utf8_text(self.source).unwrap_or("");
            return visitor.transform_leaf(&node, text);
        }

        let mut children = Vec::new();
        let mut cursor = node.walk();

        if cursor.goto_first_child() {
            loop {
                children.push(self.transform_node(cursor.node(), visitor));
                if !cursor.goto_next_sibling() {
                    break;
                }
            }
        }

        visitor.transform_node(&node, children)
    }

    #[cfg(feature = "pure-rust")]
    fn transform_node<T: TransformVisitor>(&self, node: &Node, visitor: &mut T) -> T::Output {
        if node.is_error() {
            return visitor.transform_error(node);
        }

        if node.child_count() == 0 {
            let text = &self.source[node.start_byte()..node.end_byte()];
            let text_str = std::str::from_utf8(text).unwrap_or("");
            return visitor.transform_leaf(node, text_str);
        }

        let mut children = Vec::new();
        for child in node.children() {
            children.push(self.transform_node(child, visitor));
        }

        visitor.transform_node(node, children)
    }
}

#[cfg(test)]
mod tests {

    // Mock Node for testing
    #[derive(Debug, Clone, Copy)]
    #[allow(dead_code)]
    struct MockNode {
        kind: &'static str,
        is_named: bool,
        is_error: bool,
        child_count: usize,
    }

    // Note: In real tests, we'd use actual Tree-sitter nodes

    // TODO: Re-enable these tests once we have a way to create test Nodes
    // #[test]
    // fn test_stats_visitor() {
    //     let mut visitor = StatsVisitor::default();
    //
    //     // Simulate visiting nodes
    //     visitor.enter_node(&Node::default());
    //     visitor.visit_leaf(&Node::default(), "test");
    //     visitor.leave_node(&Node::default());
    //
    //     assert_eq!(visitor.total_nodes, 1);
    //     assert_eq!(visitor.leaf_nodes, 1);
    //     assert_eq!(visitor.max_depth, 1);
    // }
    //
    // #[test]
    // fn test_pretty_print_visitor() {
    //     let mut visitor = PrettyPrintVisitor::new();
    //
    //     // Simulate visiting nodes
    //     visitor.enter_node(&Node::default());
    //     visitor.visit_leaf(&Node::default(), "hello");
    //     visitor.leave_node(&Node::default());
    //
    //     let output = visitor.output();
    //     assert!(output.contains("hello"));
    // }
}

#[cfg(test)]
mod tests2 {
    use super::*;
    use crate::pure_parser::Point;

    fn create_test_node() -> Node {
        Node {
            symbol: 1,
            children: vec![],
            start_byte: 0,
            end_byte: 10,
            start_point: Point { row: 0, column: 0 },
            end_point: Point { row: 0, column: 10 },
            is_extra: false,
            is_error: false,
            is_missing: false,
            is_named: true,
            field_id: None,
            language: None,
        }
    }

    #[derive(Default)]
    #[allow(dead_code)]
    struct TestVisitor {
        entered_nodes: Vec<String>,
        left_nodes: Vec<String>,
        leaves: Vec<String>,
        errors: Vec<String>,
    }

    impl Visitor for TestVisitor {
        fn enter_node(&mut self, _node: &Node) -> VisitorAction {
            self.entered_nodes.push("node".to_string());
            VisitorAction::Continue
        }

        fn leave_node(&mut self, _node: &Node) {
            self.left_nodes.push("node".to_string());
        }

        fn visit_leaf(&mut self, _node: &Node, text: &str) {
            self.leaves.push(text.to_string());
        }

        fn visit_error(&mut self, _node: &Node) {
            self.errors.push("error".to_string());
        }
    }

    #[test]
    fn test_visitor_action() {
        assert_eq!(VisitorAction::Continue, VisitorAction::Continue);
        assert_ne!(VisitorAction::Continue, VisitorAction::Stop);
        assert_ne!(VisitorAction::SkipChildren, VisitorAction::Stop);
    }

    #[test]
    fn test_tree_walker_creation() {
        let source = b"test source";
        let walker = TreeWalker::new(source);
        assert_eq!(walker.source, source);
    }

    #[test]
    fn test_stop_visitor() {
        struct StopVisitor {
            count: usize,
        }

        impl Visitor for StopVisitor {
            fn enter_node(&mut self, _node: &Node) -> VisitorAction {
                self.count += 1;
                if self.count > 2 {
                    VisitorAction::Stop
                } else {
                    VisitorAction::Continue
                }
            }
        }

        let mut visitor = StopVisitor { count: 0 };
        // Test that stop action is respected
        let node = create_test_node();
        let _ = visitor.enter_node(&node);
        let _ = visitor.enter_node(&node);
        let action = visitor.enter_node(&node);
        assert_eq!(action, VisitorAction::Stop);
    }

    #[test]
    fn test_skip_children_visitor() {
        struct SkipVisitor {
            depth: usize,
        }

        impl Visitor for SkipVisitor {
            fn enter_node(&mut self, _node: &Node) -> VisitorAction {
                self.depth += 1;
                if self.depth > 1 {
                    VisitorAction::SkipChildren
                } else {
                    VisitorAction::Continue
                }
            }
        }

        let mut visitor = SkipVisitor { depth: 0 };
        let node = create_test_node();
        assert_eq!(visitor.enter_node(&node), VisitorAction::Continue);
        assert_eq!(visitor.enter_node(&node), VisitorAction::SkipChildren);
    }

    // #[test]
    // fn test_breadth_first_visitor() {
    //     let source = b"test";
    //     let visitor = BreadthFirstVisitor::new(source);
    //     assert_eq!(visitor.source, source);
    // }

    // #[test]
    // fn test_filter_iterator() {
    //     let filters = vec![
    //         NodeFilter::Kind("function"),
    //         NodeFilter::Field("name"),
    //     ];
    //
    //     let mut iter = FilterIterator::new(vec![].into_iter(), filters);
    //
    //     // Just test that it can be created
    //     assert!(iter.next().is_none());
    // }
}
