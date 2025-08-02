// GLR parser performance optimizations

use crate::{Action, StateId, SymbolId};
use std::collections::HashMap;

/// Performance statistics for GLR parsing
#[derive(Debug, Default)]
pub struct PerfStats {
    pub total_tokens: usize,
    pub total_stacks: usize,
    pub max_stacks: usize,
    pub stack_merges: usize,
    pub cache_hits: usize,
    pub cache_misses: usize,
}

/// Cache for parse table lookups
pub struct ParseTableCache {
    cache: HashMap<(StateId, SymbolId), Action>,
    stats: PerfStats,
}

impl ParseTableCache {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
            stats: PerfStats::default(),
        }
    }

    pub fn get_or_compute<F>(&mut self, state: StateId, symbol: SymbolId, compute: F) -> Action
    where
        F: FnOnce() -> Action,
    {
        let key = (state, symbol);
        if let Some(action) = self.cache.get(&key) {
            self.stats.cache_hits += 1;
            action.clone()
        } else {
            self.stats.cache_misses += 1;
            let action = compute();
            self.cache.insert(key, action.clone());
            action
        }
    }

    pub fn stats(&self) -> &PerfStats {
        &self.stats
    }
}

/// Stack deduplication for GLR parsing
pub struct StackDeduplicator {
    seen_states: HashMap<Vec<StateId>, usize>,
}

impl StackDeduplicator {
    pub fn new() -> Self {
        Self {
            seen_states: HashMap::new(),
        }
    }

    /// Check if a stack configuration has been seen before
    pub fn is_duplicate(&mut self, states: &[StateId]) -> bool {
        if let Some(count) = self.seen_states.get_mut(states) {
            *count += 1;
            true
        } else {
            self.seen_states.insert(states.to_vec(), 1);
            false
        }
    }

    pub fn unique_stacks(&self) -> usize {
        self.seen_states.len()
    }
}

/// Memory pool for stack allocations
pub struct StackPool<T> {
    pool: Vec<Vec<T>>,
}

impl<T> StackPool<T> {
    pub fn new() -> Self {
        Self { pool: Vec::new() }
    }

    pub fn acquire(&mut self) -> Vec<T> {
        self.pool.pop().unwrap_or_else(Vec::new)
    }

    pub fn release(&mut self, mut vec: Vec<T>) {
        vec.clear();
        if self.pool.len() < 100 {
            // Keep pool size reasonable
            self.pool.push(vec);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_table_cache() {
        let mut cache = ParseTableCache::new();

        let action1 = cache.get_or_compute(StateId(1), SymbolId(2), || Action::Shift(StateId(3)));
        assert_eq!(cache.stats().cache_misses, 1);
        assert_eq!(cache.stats().cache_hits, 0);

        let action2 = cache.get_or_compute(StateId(1), SymbolId(2), || panic!("Should use cache"));
        assert_eq!(cache.stats().cache_hits, 1);
        assert!(matches!(action1, Action::Shift(StateId(3))));
        assert!(matches!(action2, Action::Shift(StateId(3))));
    }

    #[test]
    fn test_stack_deduplicator() {
        let mut dedup = StackDeduplicator::new();

        let stack1 = vec![StateId(1), StateId(2), StateId(3)];
        let stack2 = vec![StateId(1), StateId(2), StateId(3)];
        let stack3 = vec![StateId(1), StateId(2), StateId(4)];

        assert!(!dedup.is_duplicate(&stack1));
        assert!(dedup.is_duplicate(&stack2));
        assert!(!dedup.is_duplicate(&stack3));

        assert_eq!(dedup.unique_stacks(), 2);
    }

    #[test]
    fn test_stack_pool() {
        let mut pool: StackPool<i32> = StackPool::new();

        let vec1 = pool.acquire();
        assert!(vec1.is_empty());

        let mut vec2 = pool.acquire();
        vec2.extend(&[1, 2, 3]);

        pool.release(vec2);

        let vec3 = pool.acquire();
        assert!(vec3.is_empty()); // Should be cleared
    }
}
