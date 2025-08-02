//! GLR Parser Visualization Module
//! 
//! This module provides tools for visualizing the fork/merge behavior of the GLR parser,
//! which is particularly useful for debugging ambiguous grammars and understanding how
//! the parser explores different parse paths.

use crate::glr_parser::GLRStack;
use crate::subtree::Subtree;
use rust_sitter_glr_core::{Action, StateId};
use rust_sitter_ir::SymbolId;
use std::collections::HashMap;
use std::sync::Arc;
use std::fmt::Write;

/// Represents a snapshot of the GLR parser state at a specific point
#[derive(Clone, Debug)]
pub struct GLRSnapshot {
    pub step: usize,
    pub token: Option<(SymbolId, String)>,
    pub stacks: Vec<StackSnapshot>,
    pub action: String,
}

/// Represents a single stack's state
#[derive(Clone, Debug)]
pub struct StackSnapshot {
    pub id: usize,
    pub states: Vec<StateId>,
    pub symbols: Vec<SymbolId>,
    pub is_active: bool,
}

/// GLR Parser Visualizer
pub struct GLRVisualizer {
    snapshots: Vec<GLRSnapshot>,
    stack_counter: usize,
    stack_ids: HashMap<*const GLRStack, usize>,
}

impl GLRVisualizer {
    pub fn new() -> Self {
        Self {
            snapshots: Vec::new(),
            stack_counter: 0,
            stack_ids: HashMap::new(),
        }
    }

    /// Record a parser state snapshot
    pub fn record_snapshot(
        &mut self,
        step: usize,
        token: Option<(SymbolId, String)>,
        stacks: &[GLRStack],
        action: &str,
    ) {
        let stack_snapshots: Vec<StackSnapshot> = stacks
            .iter()
            .map(|stack| {
                let id = self.get_or_create_stack_id(stack);
                StackSnapshot {
                    id,
                    states: stack.states.clone(),
                    symbols: stack.symbols.clone(),
                    is_active: true,
                }
            })
            .collect();

        self.snapshots.push(GLRSnapshot {
            step,
            token,
            stacks: stack_snapshots,
            action: action.to_string(),
        });
    }

    /// Get or create a unique ID for a stack
    fn get_or_create_stack_id(&mut self, stack: &GLRStack) -> usize {
        let ptr = stack as *const GLRStack;
        if let Some(&id) = self.stack_ids.get(&ptr) {
            id
        } else {
            let id = self.stack_counter;
            self.stack_counter += 1;
            self.stack_ids.insert(ptr, id);
            id
        }
    }

    /// Generate a DOT graph visualization
    pub fn to_dot(&self) -> String {
        let mut dot = String::new();
        writeln!(dot, "digraph GLRParse {{").unwrap();
        writeln!(dot, "  rankdir=TB;").unwrap();
        writeln!(dot, "  node [shape=box];").unwrap();
        writeln!(dot, "  edge [fontsize=10];").unwrap();

        // Create nodes for each step
        for (i, snapshot) in self.snapshots.iter().enumerate() {
            let token_str = match &snapshot.token {
                Some((id, text)) => format!("Token: {} '{}'", id.0, text),
                None => "EOF".to_string(),
            };

            writeln!(
                dot,
                "  step{} [label=\"Step {}\\n{}\\n{}\", style=filled, fillcolor=lightblue];",
                i, snapshot.step, token_str, snapshot.action
            ).unwrap();

            // Create stack nodes
            for stack in &snapshot.stacks {
                let stack_label = format!(
                    "Stack {}\\nStates: {:?}\\nSymbols: {:?}",
                    stack.id,
                    stack.states.iter().map(|s| s.0).collect::<Vec<_>>(),
                    stack.symbols.iter().map(|s| s.0).collect::<Vec<_>>()
                );

                writeln!(
                    dot,
                    "  stack_{}_{} [label=\"{}\", shape=rectangle];",
                    i, stack.id, stack_label
                ).unwrap();

                writeln!(dot, "  step{} -> stack_{}_{};", i, i, stack.id).unwrap();
            }
        }

        // Connect consecutive steps
        for i in 0..self.snapshots.len() - 1 {
            writeln!(
                dot,
                "  step{} -> step{} [style=dashed, color=gray];",
                i,
                i + 1
            ).unwrap();
        }

        writeln!(dot, "}}").unwrap();
        dot
    }

    /// Generate a text-based visualization
    pub fn to_text(&self) -> String {
        let mut output = String::new();
        writeln!(output, "GLR Parser Execution Trace").unwrap();
        writeln!(output, "========================").unwrap();

        for snapshot in &self.snapshots {
            writeln!(output, "\nStep {}: {}", snapshot.step, snapshot.action).unwrap();
            
            if let Some((id, text)) = &snapshot.token {
                writeln!(output, "  Token: {} '{}'", id.0, text).unwrap();
            } else {
                writeln!(output, "  Token: EOF").unwrap();
            }

            writeln!(output, "  Active Stacks: {}", snapshot.stacks.len()).unwrap();
            
            for stack in &snapshot.stacks {
                writeln!(output, "    Stack {}:", stack.id).unwrap();
                writeln!(
                    output,
                    "      States: {:?}",
                    stack.states.iter().map(|s| s.0).collect::<Vec<_>>()
                ).unwrap();
                writeln!(
                    output,
                    "      Symbols: {:?}",
                    stack.symbols.iter().map(|s| s.0).collect::<Vec<_>>()
                ).unwrap();
            }
        }

        output
    }

    /// Generate a visualization of the parse forest
    pub fn visualize_parse_forest(subtree: &Arc<Subtree>) -> String {
        let mut dot = String::new();
        writeln!(dot, "digraph ParseForest {{").unwrap();
        writeln!(dot, "  rankdir=TB;").unwrap();
        writeln!(dot, "  node [shape=box];").unwrap();

        let mut node_counter = 0;
        visualize_subtree_recursive(&mut dot, subtree, &mut node_counter, 0);

        writeln!(dot, "}}").unwrap();
        dot
    }
}

fn visualize_subtree_recursive(
    dot: &mut String,
    subtree: &Arc<Subtree>,
    counter: &mut usize,
    parent_id: usize,
) -> usize {
    let node_id = *counter;
    *counter += 1;

    let label = format!(
        "Symbol: {}\\n[{}-{}]",
        subtree.node.symbol_id.0,
        subtree.node.byte_range.start,
        subtree.node.byte_range.end
    );

    writeln!(
        dot,
        "  node{} [label=\"{}\"];",
        node_id, label
    ).unwrap();

    if parent_id != node_id {
        writeln!(dot, "  node{} -> node{};", parent_id, node_id).unwrap();
    }

    for child in &subtree.children {
        visualize_subtree_recursive(dot, child, counter, node_id);
    }

    node_id
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_visualizer_creation() {
        let visualizer = GLRVisualizer::new();
        assert_eq!(visualizer.snapshots.len(), 0);
    }

    #[test]
    fn test_snapshot_recording() {
        let mut visualizer = GLRVisualizer::new();
        let stack = GLRStack {
            states: vec![StateId(0), StateId(1)],
            symbols: vec![SymbolId(1)],
            nodes: vec![],
        };

        visualizer.record_snapshot(
            1,
            Some((SymbolId(2), "+".to_string())),
            &[stack],
            "Shift",
        );

        assert_eq!(visualizer.snapshots.len(), 1);
        assert_eq!(visualizer.snapshots[0].step, 1);
        assert_eq!(visualizer.snapshots[0].action, "Shift");
    }

    #[test]
    fn test_dot_generation() {
        let mut visualizer = GLRVisualizer::new();
        let stack = GLRStack {
            states: vec![StateId(0)],
            symbols: vec![],
            nodes: vec![],
        };

        visualizer.record_snapshot(
            0,
            Some((SymbolId(1), "1".to_string())),
            &[stack],
            "Initial",
        );

        let dot = visualizer.to_dot();
        assert!(dot.contains("digraph GLRParse"));
        assert!(dot.contains("Step 0"));
        assert!(dot.contains("Token: 1 '1'"));
    }
}