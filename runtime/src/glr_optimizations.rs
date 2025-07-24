// Performance optimizations for the GLR parser
use crate::glr_parser::{GLRParser, ParseStack};
use crate::subtree::Subtree;
use rust_sitter_glr_core::{StateId, SymbolId, Action};
use std::sync::Arc;
use std::collections::{HashMap, VecDeque};

/// Stack merging optimization for GLR parser
pub struct StackMerger {
    /// Map from (state, position) to list of stack indices
    merge_candidates: HashMap<(StateId, usize), Vec<usize>>,
}

impl StackMerger {
    pub fn new() -> Self {
        Self {
            merge_candidates: HashMap::new(),
        }
    }

    /// Find stacks that can be merged
    pub fn find_mergeable_stacks(&mut self, stacks: &[ParseStack]) -> Vec<(usize, usize)> {
        self.merge_candidates.clear();
        let mut merges = Vec::new();

        // Group stacks by state and position
        for (idx, stack) in stacks.iter().enumerate() {
            let key = (stack.current_state(), stack.nodes.len());
            self.merge_candidates
                .entry(key)
                .or_insert_with(Vec::new)
                .push(idx);
        }

        // Find actual merges
        for candidates in self.merge_candidates.values() {
            if candidates.len() > 1 {
                // Merge all stacks with same state/position
                for i in 1..candidates.len() {
                    merges.push((candidates[0], candidates[i]));
                }
            }
        }

        merges
    }

    /// Merge two stacks with the same state
    pub fn merge_stacks(stack1: &ParseStack, stack2: &ParseStack) -> ParseStack {
        // For now, keep the stack with better version info
        if stack1.version.is_better_than(&stack2.version) {
            stack1.clone()
        } else {
            stack2.clone()
        }
    }
}

/// Action cache to avoid repeated lookups
pub struct ActionCache {
    cache: HashMap<(StateId, SymbolId), Action>,
    hits: usize,
    misses: usize,
}

impl ActionCache {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
            hits: 0,
            misses: 0,
        }
    }

    /// Get an action from cache or compute it
    pub fn get_or_compute<F>(&mut self, state: StateId, symbol: SymbolId, compute: F) -> Action
    where
        F: FnOnce() -> Action,
    {
        let key = (state, symbol);
        
        if let Some(&action) = self.cache.get(&key) {
            self.hits += 1;
            action
        } else {
            self.misses += 1;
            let action = compute();
            self.cache.insert(key, action.clone());
            action
        }
    }

    /// Get cache statistics
    pub fn stats(&self) -> (usize, usize) {
        (self.hits, self.misses)
    }

    /// Clear the cache
    pub fn clear(&mut self) {
        self.cache.clear();
        self.hits = 0;
        self.misses = 0;
    }
}

/// Subtree cache for memoization
pub struct SubtreeCache {
    /// Cache keyed by (symbol, start_pos, end_pos)
    cache: HashMap<(SymbolId, usize, usize), Arc<Subtree>>,
}

impl SubtreeCache {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }

    /// Try to get a cached subtree
    pub fn get(&self, symbol: SymbolId, start: usize, end: usize) -> Option<Arc<Subtree>> {
        self.cache.get(&(symbol, start, end)).cloned()
    }

    /// Insert a subtree into the cache
    pub fn insert(&mut self, subtree: Arc<Subtree>) {
        let key = (
            subtree.node.symbol_id,
            subtree.node.byte_range.start,
            subtree.node.byte_range.end,
        );
        self.cache.insert(key, subtree);
    }

    /// Clear old entries to prevent unbounded growth
    pub fn trim(&mut self, max_size: usize) {
        if self.cache.len() > max_size {
            // Simple strategy: remove half the entries
            let to_remove = self.cache.len() / 2;
            let keys: Vec<_> = self.cache.keys().take(to_remove).cloned().collect();
            for key in keys {
                self.cache.remove(&key);
            }
        }
    }
}

/// Stack pruning to limit exponential growth
pub struct StackPruner {
    /// Maximum number of stacks to keep
    max_stacks: usize,
    /// Pruning statistics
    total_pruned: usize,
}

impl StackPruner {
    pub fn new(max_stacks: usize) -> Self {
        Self {
            max_stacks,
            total_pruned: 0,
        }
    }

    /// Prune stacks keeping only the best ones
    pub fn prune_stacks(&mut self, stacks: &mut Vec<ParseStack>) {
        if stacks.len() <= self.max_stacks {
            return;
        }

        // Sort by version info (best first)
        stacks.sort_by(|a, b| {
            use rust_sitter_glr_core::CompareResult;
            match a.version.compare(&b.version) {
                CompareResult::Better => std::cmp::Ordering::Less,
                CompareResult::Worse => std::cmp::Ordering::Greater,
                CompareResult::Equal => std::cmp::Ordering::Equal,
            }
        });

        // Keep only the best stacks
        let pruned = stacks.len() - self.max_stacks;
        stacks.truncate(self.max_stacks);
        self.total_pruned += pruned;
    }

    /// Get pruning statistics
    pub fn stats(&self) -> usize {
        self.total_pruned
    }
}

/// Batch processing optimization
pub struct BatchProcessor {
    /// Queue of pending tokens
    token_queue: VecDeque<(SymbolId, String, usize)>,
    /// Batch size for processing
    batch_size: usize,
}

impl BatchProcessor {
    pub fn new(batch_size: usize) -> Self {
        Self {
            token_queue: VecDeque::new(),
            batch_size,
        }
    }

    /// Add a token to the batch
    pub fn add_token(&mut self, token: SymbolId, text: String, offset: usize) {
        self.token_queue.push_back((token, text, offset));
    }

    /// Process a batch of tokens
    pub fn process_batch(&mut self, parser: &mut GLRParser) -> usize {
        let mut processed = 0;
        
        while !self.token_queue.is_empty() && processed < self.batch_size {
            if let Some((token, text, offset)) = self.token_queue.pop_front() {
                parser.process_token(token, &text, offset);
                processed += 1;
            }
        }
        
        processed
    }

    /// Check if batch is ready
    pub fn is_batch_ready(&self) -> bool {
        self.token_queue.len() >= self.batch_size
    }

    /// Flush remaining tokens
    pub fn flush(&mut self, parser: &mut GLRParser) {
        while let Some((token, text, offset)) = self.token_queue.pop_front() {
            parser.process_token(token, &text, offset);
        }
    }
}

/// Memory pool for stack allocation
pub struct StackPool {
    /// Pool of reusable stacks
    pool: Vec<ParseStack>,
    /// Maximum pool size
    max_size: usize,
}

impl StackPool {
    pub fn new(max_size: usize) -> Self {
        Self {
            pool: Vec::with_capacity(max_size),
            max_size,
        }
    }

    /// Get a stack from the pool or create new
    pub fn acquire(&mut self) -> ParseStack {
        self.pool.pop().unwrap_or_else(|| ParseStack::new(0))
    }

    /// Return a stack to the pool
    pub fn release(&mut self, mut stack: ParseStack) {
        if self.pool.len() < self.max_size {
            stack.clear();
            self.pool.push(stack);
        }
    }

    /// Get pool statistics
    pub fn stats(&self) -> (usize, usize) {
        (self.pool.len(), self.max_size)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::glr_parser::ParseStack;
    use rust_sitter_glr_core::{StateId, VersionInfo};

    #[test]
    fn test_stack_merger() {
        let mut merger = StackMerger::new();
        
        // Create stacks with same state
        let mut stack1 = ParseStack::new(0);
        stack1.states.push(StateId(5));
        
        let mut stack2 = ParseStack::new(1);
        stack2.states.push(StateId(5));
        
        let mut stack3 = ParseStack::new(2);
        stack3.states.push(StateId(7)); // Different state
        
        let stacks = vec![stack1, stack2, stack3];
        let merges = merger.find_mergeable_stacks(&stacks);
        
        assert_eq!(merges.len(), 1);
        assert_eq!(merges[0], (0, 1));
    }

    #[test]
    fn test_action_cache() {
        let mut cache = ActionCache::new();
        
        let action1 = cache.get_or_compute(StateId(1), SymbolId(2), || Action::Shift(StateId(3)));
        let action2 = cache.get_or_compute(StateId(1), SymbolId(2), || Action::Error);
        
        assert_eq!(action1, Action::Shift(StateId(3)));
        assert_eq!(action2, Action::Shift(StateId(3))); // From cache
        
        let (hits, misses) = cache.stats();
        assert_eq!(hits, 1);
        assert_eq!(misses, 1);
    }

    #[test]
    fn test_stack_pruner() {
        let mut pruner = StackPruner::new(2);
        
        let mut stacks = vec![
            ParseStack::new(0),
            ParseStack::new(1),
            ParseStack::new(2),
        ];
        
        // Set different error costs
        stacks[0].version.error_cost = 2;
        stacks[1].version.error_cost = 1;
        stacks[2].version.error_cost = 3;
        
        pruner.prune_stacks(&mut stacks);
        
        assert_eq!(stacks.len(), 2);
        assert_eq!(stacks[0].version.error_cost, 1); // Best
        assert_eq!(stacks[1].version.error_cost, 2); // Second best
        assert_eq!(pruner.stats(), 1); // One pruned
    }

    #[test]
    fn test_batch_processor() {
        let mut processor = BatchProcessor::new(3);
        
        processor.add_token(SymbolId(1), "a".to_string(), 0);
        processor.add_token(SymbolId(2), "b".to_string(), 1);
        
        assert!(!processor.is_batch_ready());
        
        processor.add_token(SymbolId(3), "c".to_string(), 2);
        assert!(processor.is_batch_ready());
    }

    #[test]
    fn test_stack_pool() {
        let mut pool = StackPool::new(5);
        
        let stack1 = pool.acquire();
        let stack2 = pool.acquire();
        
        assert_eq!(pool.stats().0, 0); // Pool empty
        
        pool.release(stack1);
        pool.release(stack2);
        
        assert_eq!(pool.stats().0, 2); // Two returned
        
        let _stack3 = pool.acquire();
        assert_eq!(pool.stats().0, 1); // One reused
    }
}