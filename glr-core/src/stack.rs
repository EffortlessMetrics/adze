/// Persistent stack implementation for GLR parser
///
/// This module provides a memory-efficient stack structure that shares
/// common tails between forked stacks, reducing memory allocation and
/// copy overhead during GLR parsing.
use std::sync::Arc;

/// Small vector optimization size for stack heads
const SMALL_VEC_SIZE: usize = 8;

/// A persistent stack node with shared tail
#[derive(Clone, Debug)]
pub struct StackNode {
    /// Parser state
    pub state: u16,
    /// Optional symbol that led to this state
    pub symbol: Option<u16>,
    /// Small head for recent pushes (avoids allocation for small stacks)
    pub head: Vec<u16>,
    /// Shared tail with other stacks
    pub tail: Option<Arc<StackNode>>,
}

impl StackNode {
    /// Create a new empty stack
    pub fn new() -> Self {
        Self {
            state: 0,
            symbol: None,
            head: Vec::with_capacity(SMALL_VEC_SIZE),
            tail: None,
        }
    }

    /// Create a stack with an initial state
    pub fn with_state(state: u16) -> Self {
        Self {
            state,
            symbol: None,
            head: Vec::with_capacity(SMALL_VEC_SIZE),
            tail: None,
        }
    }

    /// Push a new state onto the stack
    pub fn push(&mut self, state: u16, symbol: Option<u16>) {
        if self.head.len() >= SMALL_VEC_SIZE {
            // Spill to a new node with shared tail
            let old_node = Self {
                state: self.state,
                symbol: self.symbol,
                head: std::mem::replace(&mut self.head, Vec::with_capacity(SMALL_VEC_SIZE)),
                tail: self.tail.take(),
            };
            self.tail = Some(Arc::new(old_node));
        }

        self.head.push(state);
        if let Some(sym) = symbol {
            self.head.push(sym);
        }
    }

    /// Pop a state from the stack
    pub fn pop(&mut self) -> Option<(u16, Option<u16>)> {
        if !self.head.is_empty() {
            let symbol = if self.head.len() >= 2 {
                Some(self.head.pop().unwrap())
            } else {
                None
            };
            let state = self.head.pop().unwrap();
            return Some((state, symbol));
        }

        // Need to restore from tail
        if let Some(tail) = self.tail.take() {
            match Arc::try_unwrap(tail) {
                Ok(node) => {
                    // We own the tail, can move its contents
                    self.state = node.state;
                    self.symbol = node.symbol;
                    self.head = node.head;
                    self.tail = node.tail;
                    self.pop()
                }
                Err(arc) => {
                    // Tail is shared, need to clone
                    let node = (*arc).clone();
                    self.state = node.state;
                    self.symbol = node.symbol;
                    self.head = node.head;
                    self.tail = node.tail;
                    self.pop()
                }
            }
        } else {
            None
        }
    }

    /// Get the current top state without popping
    pub fn top(&self) -> Option<u16> {
        self.head.last().copied().or(Some(self.state))
    }

    /// Get the depth of the stack
    pub fn depth(&self) -> usize {
        let mut depth = self.head.len() + 1; // +1 for self.state
        let mut tail = &self.tail;

        while let Some(node) = tail {
            depth += node.head.len() + 1;
            tail = &node.tail;
        }

        depth
    }

    /// Check if the stack is empty
    pub fn is_empty(&self) -> bool {
        self.head.is_empty() && self.tail.is_none() && self.state == 0
    }

    /// Convert to a vector for debugging
    pub fn to_vec(&self) -> Vec<u16> {
        let mut result = Vec::with_capacity(self.depth());

        // Add current node
        if self.state != 0 {
            result.push(self.state);
        }
        result.extend(&self.head);

        // Add tail nodes
        let mut tail = &self.tail;
        while let Some(node) = tail {
            if node.state != 0 {
                result.push(node.state);
            }
            result.extend(&node.head);
            tail = &node.tail;
        }

        result.reverse();
        result
    }

    /// Fork this stack (cheap due to structural sharing)
    pub fn fork(&self) -> Self {
        self.clone()
    }

    /// Check if two stacks can be merged (have compatible suffixes)
    pub fn can_merge_with(&self, other: &Self) -> bool {
        // For simplicity, check if they have the same depth and top state
        // In a real implementation, we'd check more of the suffix
        self.depth() == other.depth() && self.top() == other.top()
    }
}

impl Default for StackNode {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_push_pop() {
        let mut stack = StackNode::new();

        stack.push(1, None);
        stack.push(2, Some(100));
        stack.push(3, None);

        assert_eq!(stack.pop(), Some((3, None)));
        assert_eq!(stack.pop(), Some((2, Some(100))));
        assert_eq!(stack.pop(), Some((1, None)));
        assert_eq!(stack.pop(), None);
    }

    #[test]
    fn test_fork_sharing() {
        let mut stack1 = StackNode::with_state(1);
        stack1.push(2, None);
        stack1.push(3, None);

        let stack2 = stack1.fork();

        // Both stacks should have the same content
        assert_eq!(stack1.to_vec(), vec![1, 2, 3]);
        assert_eq!(stack2.to_vec(), vec![1, 2, 3]);

        // Modifying one shouldn't affect the other
        stack1.push(4, None);
        assert_eq!(stack1.to_vec(), vec![1, 2, 3, 4]);
        assert_eq!(stack2.to_vec(), vec![1, 2, 3]);
    }

    #[test]
    fn test_spill_to_tail() {
        let mut stack = StackNode::new();

        // Push enough to trigger spill
        for i in 0..20 {
            stack.push(i, None);
        }

        // Should still work correctly
        assert_eq!(stack.depth(), 20);

        // Pop everything back
        for i in (0..20).rev() {
            assert_eq!(stack.pop(), Some((i, None)));
        }

        assert!(stack.is_empty());
    }
}
