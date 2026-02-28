// GLR (Generalized LR) parsing support
// This module implements fork/merge handling for ambiguous grammars

use crate::parser_v3::{ParseNode, ParserState};
use adze_glr_core::Action;
use adze_ir::{StateId, SymbolId};
use anyhow::Result;
use std::collections::{HashMap, VecDeque};

/// A GLR parser stack that can represent multiple parse paths
#[derive(Debug, Clone)]
pub struct GLRStack {
    /// Active parse stacks (each represents a different parse path)
    pub stacks: Vec<ParseStack>,
    /// Merge points where stacks converge
    pub merge_points: HashMap<(StateId, usize), Vec<usize>>, // (state, position) -> stack indices
}

/// A single parse stack in the GLR forest
#[derive(Debug, Clone)]
pub struct ParseStack {
    /// Stack ID for tracking
    pub id: usize,
    /// Parser state stack
    pub state_stack: Vec<ParserState>,
    /// Parse node stack
    pub node_stack: Vec<ParseNode>,
    /// Current position in input
    pub position: usize,
    /// Whether this stack is still active
    pub active: bool,
}

/// Result of a GLR fork operation
#[derive(Debug)]
pub struct ForkResult {
    /// New stacks created by the fork
    pub new_stacks: Vec<ParseStack>,
    /// Indices of stacks that should be deactivated
    pub deactivate: Vec<usize>,
}

impl GLRStack {
    /// Create a new GLR stack with an initial parse stack
    pub fn new(initial_state: StateId) -> Self {
        let initial_stack = ParseStack {
            id: 0,
            state_stack: vec![ParserState {
                state: initial_state,
                symbol: None,
                position: 0,
            }],
            node_stack: Vec::new(),
            position: 0,
            active: true,
        };

        Self {
            stacks: vec![initial_stack],
            merge_points: HashMap::new(),
        }
    }

    /// Handle a fork action by creating new parse stacks
    pub fn fork(&mut self, stack_idx: usize, actions: &[Action]) -> Result<ForkResult> {
        if stack_idx >= self.stacks.len() {
            anyhow::bail!("Invalid stack index for fork");
        }

        let source_stack = &self.stacks[stack_idx];
        if !source_stack.active {
            anyhow::bail!("Cannot fork from inactive stack");
        }

        let mut new_stacks = Vec::new();
        let next_id = self.stacks.len();

        // Create a new stack for each action
        for (i, _action) in actions.iter().enumerate() {
            let mut new_stack = source_stack.clone();
            new_stack.id = next_id + i;
            new_stacks.push(new_stack);
        }

        Ok(ForkResult {
            new_stacks,
            deactivate: vec![stack_idx], // Deactivate the original stack
        })
    }

    /// Check if stacks can be merged at the current state
    pub fn check_merge(&mut self, state: StateId, position: usize) -> Vec<Vec<usize>> {
        let _key = (state, position);
        let mut stacks_at_state = Vec::new();

        // Find all active stacks at this state and position
        for (idx, stack) in self.stacks.iter().enumerate() {
            if stack.active
                && stack.position == position
                && stack.state_stack.last().map(|s| s.state) == Some(state)
            {
                stacks_at_state.push(idx);
            }
        }

        // Group stacks that can be merged
        let mut merge_groups = Vec::new();
        if stacks_at_state.len() > 1 {
            // For now, merge all stacks at the same state
            // In a more sophisticated implementation, we'd check compatibility
            merge_groups.push(stacks_at_state);
        }

        merge_groups
    }

    /// Merge multiple stacks into one
    pub fn merge(&mut self, stack_indices: &[usize]) -> Result<usize> {
        if stack_indices.len() < 2 {
            anyhow::bail!("Need at least 2 stacks to merge");
        }

        // Use the first stack as the base
        let base_idx = stack_indices[0];

        // Create ambiguity nodes for different parse trees
        let mut ambiguous_nodes = Vec::new();
        for &idx in stack_indices {
            if let Some(node) = self.stacks[idx].node_stack.last() {
                ambiguous_nodes.push(node.clone());
            }
        }

        // Create an ambiguity node if parse trees differ
        if ambiguous_nodes.len() > 1 && !Self::nodes_equal(&ambiguous_nodes) {
            let ambiguity_node = ParseNode {
                symbol: SymbolId(0xFFFF), // Special ambiguity marker
                children: ambiguous_nodes,
                start_byte: self.stacks[base_idx].position,
                end_byte: self.stacks[base_idx].position,
                field_name: Some("ambiguous".to_string()),
            };

            // Replace top node with ambiguity node
            if let Some(stack) = self.stacks.get_mut(base_idx) {
                stack.node_stack.pop();
                stack.node_stack.push(ambiguity_node);
            }
        }

        // Deactivate all but the base stack
        for &idx in &stack_indices[1..] {
            if let Some(stack) = self.stacks.get_mut(idx) {
                stack.active = false;
            }
        }

        Ok(base_idx)
    }

    /// Check if nodes are structurally equal
    fn nodes_equal(nodes: &[ParseNode]) -> bool {
        if nodes.is_empty() {
            return true;
        }

        let first = &nodes[0];
        nodes[1..]
            .iter()
            .all(|node| node.symbol == first.symbol && node.children.len() == first.children.len())
    }

    /// Get all active stacks
    pub fn active_stacks(&self) -> Vec<&ParseStack> {
        self.stacks.iter().filter(|s| s.active).collect()
    }

    /// Get mutable references to active stacks
    pub fn active_stacks_mut(&mut self) -> Vec<&mut ParseStack> {
        self.stacks.iter_mut().filter(|s| s.active).collect()
    }
}

/// GLR parser coordinator that manages multiple parse stacks
pub struct GLRParser {
    /// The GLR stack forest
    pub stack: GLRStack,
    /// Queue of stacks to process
    pub work_queue: VecDeque<usize>,
}

impl GLRParser {
    /// Create a new GLR parser
    pub fn new(initial_state: StateId) -> Self {
        Self {
            stack: GLRStack::new(initial_state),
            work_queue: VecDeque::from(vec![0]),
        }
    }

    /// Process all active stacks for the current token
    pub fn process_token(
        &mut self,
        token_symbol: SymbolId,
        get_action: impl Fn(StateId, SymbolId) -> Action,
    ) -> Result<()> {
        let mut new_work = Vec::new();

        while let Some(stack_idx) = self.work_queue.pop_front() {
            let Some(stack) = self.stack.stacks.get(stack_idx) else {
                continue;
            };
            if !stack.active {
                continue;
            }

            let current_state = stack
                .state_stack
                .last()
                .ok_or_else(|| anyhow::anyhow!("Empty state stack"))?
                .state;

            let action = get_action(current_state, token_symbol);

            match action {
                Action::Fork(ref actions) => {
                    // Handle fork by creating new stacks
                    let fork_result = self.stack.fork(stack_idx, actions)?;

                    // Add new stacks to the GLR forest
                    for new_stack in fork_result.new_stacks {
                        let new_idx = self.stack.stacks.len();
                        self.stack.stacks.push(new_stack);
                        new_work.push(new_idx);
                    }

                    // Deactivate forked stack
                    for idx in fork_result.deactivate {
                        if let Some(stack) = self.stack.stacks.get_mut(idx) {
                            stack.active = false;
                        }
                    }
                }
                _ => {
                    // Non-fork actions are handled by the regular parser
                    new_work.push(stack_idx);
                }
            }
        }

        // Check for merge opportunities
        let mut merge_data = Vec::new();
        for stack in self.stack.active_stacks() {
            if let Some(state) = stack.state_stack.last() {
                merge_data.push((state.state, stack.position));
            }
        }

        for (state, position) in merge_data {
            let merge_groups = self.stack.check_merge(state, position);

            for group in merge_groups {
                if group.len() > 1 {
                    self.stack.merge(&group)?;
                }
            }
        }

        // Add remaining work to queue
        self.work_queue.extend(new_work);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use adze_ir::RuleId;

    #[test]
    fn test_glr_fork() {
        let mut glr = GLRStack::new(StateId(0));

        // Create a fork with 3 actions
        let actions = vec![
            Action::Shift(StateId(1)),
            Action::Shift(StateId(2)),
            Action::Reduce(RuleId(1)),
        ];

        let fork_result = glr.fork(0, &actions).unwrap();
        assert_eq!(fork_result.new_stacks.len(), 3);
        assert_eq!(fork_result.deactivate, vec![0]);
    }

    #[test]
    fn test_glr_merge() {
        let mut glr = GLRStack::new(StateId(0));

        // Add another stack at the same state
        let mut stack2 = glr.stacks[0].clone();
        stack2.id = 1;
        glr.stacks.push(stack2);

        // Both stacks at state 0, position 0
        let merge_groups = glr.check_merge(StateId(0), 0);
        assert_eq!(merge_groups.len(), 1);
        assert_eq!(merge_groups[0].len(), 2);

        // Merge them
        let merged_idx = glr.merge(&merge_groups[0]).unwrap();
        assert_eq!(merged_idx, 0);
        assert!(!glr.stacks[1].active);
    }
}
