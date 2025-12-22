//! Persistent stack implementation for GLR parser
//!
//! This module provides a memory-efficient stack structure that shares
//! common tails between forked stacks, reducing memory allocation and
//! copy overhead during GLR parsing.
//!
//! # Invariants
//!
//! - `head` stores pairs `[state, symbol_or_NO_SYM]`
//! - Head vector always has even length
//! - `top()` returns the last state in the last pair (unless `head` is empty, then returns `state`)
//! - `depth()` counts states only, not symbols

use std::sync::Arc;

/// Small vector optimization: number of state/symbol pairs before spilling
const SMALL_VEC_PAIR_CAP: usize = 4;
/// Total entry capacity (pairs * 2)
const ENTRY_CAP: usize = SMALL_VEC_PAIR_CAP * 2;

/// Sentinel value for "no symbol" in head pairs.
/// Symbol IDs are guaranteed to be less than u16::MAX.
const NO_SYM: u16 = u16::MAX;

/// Reserved base state value.
/// State 0 is reserved as the "empty base" state for stack initialization.
/// All valid grammar states must be > 0.
const BASE_EMPTY_STATE: u16 = 0;

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
    // Private helper functions for pair operations
    #[inline]
    fn push_pair(head: &mut Vec<u16>, state: u16, sym: Option<u16>) {
        debug_assert!(
            head.len().is_multiple_of(2),
            "head must contain pairs before push"
        );
        debug_assert!(sym != Some(NO_SYM), "symbol id must be < u16::MAX");
        head.push(state);
        head.push(sym.unwrap_or(NO_SYM));
        debug_assert!(
            head.len().is_multiple_of(2),
            "head must contain pairs after push"
        );
    }

    #[inline]
    fn pop_pair(head: &mut Vec<u16>) -> Option<(u16, Option<u16>)> {
        debug_assert!(head.len().is_multiple_of(2), "head must contain pairs");
        if head.len() < 2 {
            return None;
        }
        let sym = head.pop().unwrap();
        let state = head.pop().unwrap();
        Some((state, (sym != NO_SYM).then_some(sym)))
    }

    /// Create a new empty stack
    pub fn new() -> Self {
        Self {
            state: BASE_EMPTY_STATE,
            symbol: None,
            head: Vec::with_capacity(ENTRY_CAP),
            tail: None,
        }
    }

    /// Create a stack with an initial state
    pub fn with_state(state: u16) -> Self {
        Self {
            state,
            symbol: None,
            head: Vec::with_capacity(ENTRY_CAP),
            tail: None,
        }
    }

    /// Push a new state onto the stack
    pub fn push(&mut self, state: u16, symbol: Option<u16>) {
        // Check if we need to spill (need room for 1 more pair = 2 entries)
        if self.head.len() + 2 > ENTRY_CAP {
            // Spill to a new node with shared tail
            let old_node = Self {
                state: self.state,
                symbol: self.symbol,
                head: std::mem::replace(&mut self.head, Vec::with_capacity(ENTRY_CAP)),
                tail: self.tail.take(),
            };
            self.tail = Some(Arc::new(old_node));
        }

        // Always push as a pair using helper
        Self::push_pair(&mut self.head, state, symbol);

        #[cfg(debug_assertions)]
        self.assert_well_formed();
    }

    /// Pop a state from the stack
    pub fn pop(&mut self) -> Option<(u16, Option<u16>)> {
        if let Some(pair) = Self::pop_pair(&mut self.head) {
            #[cfg(debug_assertions)]
            self.assert_well_formed();
            return Some(pair);
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
        } else if self.state != 0 {
            // Return the initial state
            let state = self.state;
            let symbol = self.symbol;
            self.state = 0;
            self.symbol = None;

            #[cfg(debug_assertions)]
            self.assert_well_formed();

            Some((state, symbol))
        } else {
            None
        }
    }

    /// Get the current top state without popping
    #[inline]
    pub fn top(&self) -> Option<u16> {
        // Iterative to avoid deep recursion on long tails
        let mut node: Option<&StackNode> = Some(self);
        while let Some(n) = node {
            debug_assert!(n.head.len() % 2 == 0, "head must contain pairs");
            if n.head.len() >= 2 {
                return Some(n.head[n.head.len() - 2]);
            }
            if n.state != 0 {
                return Some(n.state);
            }
            node = n.tail.as_deref();
        }
        None
    }

    /// Get the last state without popping
    #[inline]
    pub fn last(&self) -> Option<u16> {
        self.top()
    }

    /// Get the depth of the stack (number of states pushed)
    pub fn depth(&self) -> usize {
        let mut count = 0;
        let mut cur: Option<&StackNode> = Some(self);
        while let Some(n) = cur {
            if n.state != 0 {
                count += 1;
            }
            debug_assert!(n.head.len() % 2 == 0, "head must contain pairs");
            count += n.head.len() / 2;
            cur = n.tail.as_deref();
        }
        count
    }

    /// Check if the stack is empty
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.head.is_empty() && self.tail.is_none() && self.state == 0
    }

    /// Convert to a vector for debugging (returns only states, not symbols)
    pub fn to_vec(&self) -> Vec<u16> {
        // O(n) walk: accumulate tail→head states in order
        let mut out = Vec::with_capacity(self.depth());

        // Walk tails first to build chain from root to head
        let mut chain: Vec<&StackNode> = Vec::new();
        let mut cur: Option<&StackNode> = Some(self);
        while let Some(n) = cur {
            chain.push(n);
            cur = n.tail.as_deref();
        }

        // Process nodes in reverse order (root to head)
        for n in chain.iter().rev() {
            if n.state != 0 {
                out.push(n.state);
            }
            debug_assert!(n.head.len() % 2 == 0, "head must contain pairs");
            // Extract states from pairs (at even indices)
            for i in (0..n.head.len()).step_by(2) {
                out.push(n.head[i]);
            }
        }
        out
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

    /// Assert that the stack structure is well-formed (for debugging)
    #[inline]
    #[cfg_attr(not(any(test, debug_assertions)), doc(hidden))]
    pub fn assert_well_formed(&self) {
        let mut cur: Option<&StackNode> = Some(self);
        while let Some(n) = cur {
            debug_assert!(n.head.len() % 2 == 0, "head must contain pairs");
            cur = n.tail.as_deref();
        }
    }

    /// Test-only constructor for creating a StackNode with raw fields
    #[cfg(any(test, feature = "test-api"))]
    pub fn from_raw(state: u16, head: Vec<u16>, tail: Option<Arc<StackNode>>) -> Self {
        let s = Self {
            state,
            symbol: None,
            head,
            tail,
        };
        s.assert_well_formed();
        s
    }
}

impl Default for StackNode {
    fn default() -> Self {
        Self::new()
    }
}

/// Test helpers module (only available in tests or with test-api feature)
#[cfg(any(test, feature = "test-api"))]
pub mod test_helpers {
    use super::*;

    /// Minimal trait the engine uses. Implemented by the old Vec-based stack and the new persistent one.
    pub trait GlrStack: Clone {
        /// Push a parser state onto the stack.
        fn push(&mut self, state: u16);
        /// Remove and return the top parser state, if any.
        fn pop(&mut self) -> Option<u16>;
        /// View the top parser state without removing it.
        fn peek(&self) -> Option<u16>;
        /// Current number of states stored in the stack.
        fn len(&self) -> usize;
        /// Returns `true` when the stack contains no states.
        fn is_empty(&self) -> bool {
            self.len() == 0
        }
    }

    // Implement GlrStack for Vec<u16> for backwards compatibility
    impl GlrStack for Vec<u16> {
        fn push(&mut self, state: u16) {
            Vec::push(self, state)
        }
        fn pop(&mut self) -> Option<u16> {
            Vec::pop(self)
        }
        fn peek(&self) -> Option<u16> {
            self.last().copied()
        }
        fn len(&self) -> usize {
            Vec::len(self)
        }
    }

    // Implement GlrStack for StackNode
    impl GlrStack for StackNode {
        fn push(&mut self, state: u16) {
            StackNode::push(self, state, None)
        }
        fn pop(&mut self) -> Option<u16> {
            StackNode::pop(self).map(|(state, _)| state)
        }
        fn peek(&self) -> Option<u16> {
            self.top()
        }
        fn len(&self) -> usize {
            self.depth()
        }
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

    #[test]
    fn top_ignores_symbol_and_reads_state() {
        // Guard test for NO_SYM semantics
        let mut s = StackNode::new();

        // Push state 7 with symbol 11
        s.push(7, Some(11));

        // top() should return the state, not the symbol
        assert_eq!(s.top(), Some(7));

        // Verify the stack is well-formed
        s.assert_well_formed();

        // Also test with NO_SYM (None)
        s.push(9, None);
        assert_eq!(s.top(), Some(9));
        s.assert_well_formed();
    }
}
