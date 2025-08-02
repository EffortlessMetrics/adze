// Parse tree visitor API for the pure-Rust Tree-sitter implementation
// This module provides flexible visitor patterns for traversing and analyzing parse trees

#[cfg(not(feature = "pure-rust"))]
use crate::tree_sitter::Node;
#[cfg(feature = "pure-rust")]
use crate::pure_parser::ParsedNode as Node;

use std::collections::VecDeque;

/// Visitor trait for traversing parse trees
pub trait Visitor {
    /// Called when entering a node
    fn enter_node(&mut self, _node: &Node) -> VisitorAction {
        VisitorAction::Continue
    }
    
    /// Called when leaving a node
    fn leave_node(&mut self, _node: &Node) {
        // Default: do nothing
    }
    
    /// Called for leaf nodes (tokens)
    fn visit_leaf(&mut self, _node: &Node, _text: &str) {
        // Default: do nothing
    }
    
    /// Called for error nodes
    fn visit_error(&mut self, _node: &Node) {
        // Default: do nothing
    }
}

/// Action to take after visiting a node
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VisitorAction {
    /// Continue traversing children
    Continue,
    /// Skip children of this node
    SkipChildren,
    /// Stop traversal entirely
    Stop,
}

/// Depth-first tree walker
pub struct TreeWalker<'a> {
    source: &'a [u8],
}

impl<'a> TreeWalker<'a> {
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
            let text = self.get_node_text(node);
            visitor.visit_leaf(&node, &text);
        } else {
            for child in node.children() {
                self.walk_node(child, visitor);
            }
        }
        
        // Leave the node
        visitor.leave_node(&node);
    }
}

/// Breadth-first tree walker
pub struct BreadthFirstWalker<'a> {
    source: &'a [u8],
}

impl<'a> BreadthFirstWalker<'a> {
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
                let text = &self.source[node.start_byte()..node.end_byte()];
                if let Ok(text_str) = std::str::from_utf8(text) {
                    visitor.visit_leaf(&node, text_str);
                }
            } else {
                for child in node.children() {
                    queue.push_back(child);
                }
            }
        }
    }
}

/// Visitor that collects statistics about the parse tree
#[derive(Debug, Default)]
pub struct StatsVisitor {
    pub total_nodes: usize,
    pub leaf_nodes: usize,
    pub error_nodes: usize,
    pub max_depth: usize,
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

/// Visitor that searches for specific node types
pub struct SearchVisitor<F> {
    predicate: F,
    pub matches: Vec<(usize, usize, String)>, // (start, end, kind)
}

impl<F> SearchVisitor<F>
where
    F: Fn(&Node) -> bool,
{
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
            self.matches.push((
                node.start_byte(),
                node.end_byte(),
                node.kind().to_string(),
            ));
        }
        VisitorAction::Continue
    }
}

/// Visitor that pretty-prints the tree structure
pub struct PrettyPrintVisitor {
    indent: usize,
    output: String,
}

impl PrettyPrintVisitor {
    pub fn new() -> Self {
        Self {
            indent: 0,
            output: String::new(),
        }
    }
    
    pub fn output(&self) -> &str {
        &self.output
    }
}

impl Visitor for PrettyPrintVisitor {
    fn enter_node(&mut self, node: &Node) -> VisitorAction {
        let indent_str = "  ".repeat(self.indent);
        self.output.push_str(&format!("{}{}", indent_str, node.kind()));
        
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
        self.output.push_str(&format!("{}ERROR: {}\n", indent_str, node.kind()));
    }
}

/// Transform visitor that can modify nodes during traversal
pub trait TransformVisitor {
    type Output;
    
    /// Transform a node
    fn transform_node(&mut self, node: &Node, children: Vec<Self::Output>) -> Self::Output;
    
    /// Transform a leaf node
    fn transform_leaf(&mut self, node: &Node, text: &str) -> Self::Output;
    
    /// Transform an error node
    fn transform_error(&mut self, node: &Node) -> Self::Output;
}

/// Walker that applies transformations
pub struct TransformWalker<'a> {
    source: &'a [u8],
}

impl<'a> TransformWalker<'a> {
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
            return visitor.transform_error(&node);
        }
        
        if node.child_count() == 0 {
            let text = &self.source[node.start_byte()..node.end_byte()];
            let text_str = std::str::from_utf8(text).unwrap_or("");
            return visitor.transform_leaf(&node, text_str);
        }
        
        let mut children = Vec::new();
        for child in node.children() {
            children.push(self.transform_node(child, visitor));
        }
        
        visitor.transform_node(&node, children)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    // Mock Node for testing
    #[derive(Debug, Clone, Copy)]
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
            field_name: None,
            language: None,
        }
    }
    
    #[derive(Default)]
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