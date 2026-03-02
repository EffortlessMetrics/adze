//! Core reusable `Vec` pool for stack-like parser workloads.

#![forbid(unsafe_op_in_unsafe_fn)]
#![deny(missing_docs)]
#![cfg_attr(feature = "strict_api", deny(unreachable_pub))]
#![cfg_attr(not(feature = "strict_api"), warn(unreachable_pub))]
#![cfg_attr(feature = "strict_docs", deny(missing_docs))]
#![cfg_attr(not(feature = "strict_docs"), allow(missing_docs))]

/// A simple reusable `Vec<T>` pool with a bounded cache size.
#[derive(Debug)]
pub struct StackPool<T> {
    pool: Vec<Vec<T>>,
    max_cached: usize,
}

impl<T> StackPool<T> {
    /// Create a pool using the default cache size of `100` vectors.
    #[must_use]
    pub fn new() -> Self {
        Self::with_max_cached(100)
    }

    /// Create a pool with an explicit max number of cached vectors.
    #[must_use]
    pub fn with_max_cached(max_cached: usize) -> Self {
        Self {
            pool: Vec::new(),
            max_cached,
        }
    }

    /// Acquire a cleared vector from the pool.
    #[must_use]
    pub fn acquire(&mut self) -> Vec<T> {
        self.pool.pop().unwrap_or_default()
    }

    /// Return a vector to the pool; it will be cleared and optionally cached.
    pub fn release(&mut self, mut vec: Vec<T>) {
        vec.clear();
        if self.pool.len() < self.max_cached {
            self.pool.push(vec);
        }
    }

    /// Number of currently cached vectors.
    #[must_use]
    pub fn cached_len(&self) -> usize {
        self.pool.len()
    }
}

impl<T> Default for StackPool<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn acquire_returns_empty_vec() {
        let mut pool: StackPool<i32> = StackPool::new();
        assert!(pool.acquire().is_empty());
    }

    #[test]
    fn release_clears_vec_before_reuse() {
        let mut pool = StackPool::new();
        pool.release(vec![1, 2, 3]);

        let reused = pool.acquire();
        assert!(reused.is_empty());
    }

    #[test]
    fn respects_cache_limit() {
        let mut pool = StackPool::with_max_cached(1);
        pool.release(vec![1]);
        pool.release(vec![2]);
        assert_eq!(pool.cached_len(), 1);
    }
}
