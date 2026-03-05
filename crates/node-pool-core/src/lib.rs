//! Thread-safe object pool for `Arc`-managed parser nodes.
#![forbid(unsafe_op_in_unsafe_fn)]
#![deny(missing_docs)]
#![cfg_attr(feature = "strict_api", deny(unreachable_pub))]
#![cfg_attr(not(feature = "strict_api"), warn(unreachable_pub))]
#![cfg_attr(feature = "strict_docs", deny(missing_docs))]
#![cfg_attr(not(feature = "strict_docs"), allow(missing_docs))]

use std::collections::VecDeque;
use std::sync::Arc;
use std::sync::Mutex;

/// Default pool capacity.
const DEFAULT_POOL_CAPACITY: usize = 256;

/// A thread-safe object pool using `Arc` for shared ownership.
pub struct NodePool<T> {
    /// Queue of available nodes.
    queue: Mutex<VecDeque<Arc<T>>>,
    /// Maximum capacity of the pool.
    capacity: usize,
    /// Statistics.
    stats: PoolStats,
}

/// Pool statistics for monitoring and tuning.
#[derive(Default, Debug)]
pub struct PoolStats {
    /// Total allocations from pool.
    pub gets: std::sync::atomic::AtomicU64,
    /// Total returns to pool.
    pub puts: std::sync::atomic::AtomicU64,
    /// Times pool was empty (had to allocate new).
    pub misses: std::sync::atomic::AtomicU64,
    /// Times pool was full (had to drop returned item).
    pub drops: std::sync::atomic::AtomicU64,
}

impl<T> NodePool<T> {
    /// Create a new pool with default capacity.
    #[must_use]
    pub fn new() -> Self {
        Self::with_capacity(DEFAULT_POOL_CAPACITY)
    }

    /// Create a new pool with specified capacity.
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            queue: Mutex::new(VecDeque::with_capacity(capacity)),
            capacity,
            stats: PoolStats::default(),
        }
    }

    /// Get a node from the pool or create a new one.
    pub fn get_or<F>(&self, factory: F) -> Arc<T>
    where
        F: FnOnce() -> T,
    {
        self.stats
            .gets
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        let mut queue = self.queue.lock().unwrap_or_else(|err| err.into_inner());
        if let Some(node) = queue.pop_front() {
            node
        } else {
            self.stats
                .misses
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            Arc::new(factory())
        }
    }

    /// Get a node from the pool or use default.
    pub fn get_or_default(&self) -> Arc<T>
    where
        T: Default,
    {
        self.get_or(T::default)
    }

    /// Return a node to the pool for reuse.
    pub fn put(&self, node: Arc<T>) {
        if Arc::strong_count(&node) == 1 {
            self.stats
                .puts
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);

            let mut queue = self.queue.lock().unwrap_or_else(|err| err.into_inner());
            if queue.len() < self.capacity {
                queue.push_back(node);
            } else {
                self.stats
                    .drops
                    .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            }
        }
    }

    /// Clear all nodes from the pool.
    pub fn clear(&self) {
        let mut queue = self.queue.lock().unwrap_or_else(|err| err.into_inner());
        queue.clear();
    }

    /// Get current pool size.
    #[must_use]
    pub fn size(&self) -> usize {
        let queue = self.queue.lock().unwrap_or_else(|err| err.into_inner());
        queue.len()
    }

    /// Get pool statistics.
    #[must_use]
    pub fn stats(&self) -> PoolStatsSnapshot {
        PoolStatsSnapshot {
            gets: self.stats.gets.load(std::sync::atomic::Ordering::Relaxed),
            puts: self.stats.puts.load(std::sync::atomic::Ordering::Relaxed),
            misses: self.stats.misses.load(std::sync::atomic::Ordering::Relaxed),
            drops: self.stats.drops.load(std::sync::atomic::Ordering::Relaxed),
            current_size: self.size(),
            capacity: self.capacity,
        }
    }
}

impl<T> Default for NodePool<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Snapshot of pool statistics.
#[derive(Debug, Clone)]
pub struct PoolStatsSnapshot {
    /// Total number of objects requested from the pool.
    pub gets: u64,
    /// Total number of objects returned to the pool.
    pub puts: u64,
    /// Number of requests that were not satisfied from the pool.
    pub misses: u64,
    /// Number of objects dropped instead of pooled.
    pub drops: u64,
    /// Current number of objects stored in the pool.
    pub current_size: usize,
    /// Maximum capacity of the pool.
    pub capacity: usize,
}

impl PoolStatsSnapshot {
    /// Calculate hit rate (percentage of gets that were satisfied from pool).
    #[must_use]
    pub fn hit_rate(&self) -> f64 {
        if self.gets == 0 {
            0.0
        } else {
            ((self.gets - self.misses) as f64 / self.gets as f64) * 100.0
        }
    }

    /// Calculate reuse rate (percentage of puts that were kept in pool).
    #[must_use]
    pub fn reuse_rate(&self) -> f64 {
        if self.puts == 0 {
            0.0
        } else {
            ((self.puts - self.drops) as f64 / self.puts as f64) * 100.0
        }
    }
}

impl std::fmt::Display for PoolStatsSnapshot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Pool Stats: {} gets, {} puts, {:.1}% hit rate, {:.1}% reuse rate, {}/{} capacity",
            self.gets,
            self.puts,
            self.hit_rate(),
            self.reuse_rate(),
            self.current_size,
            self.capacity
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default, Debug)]
    struct TestNode {
        #[allow(dead_code)]
        value: u32,
    }

    #[test]
    fn pool_basic() {
        let pool = NodePool::<TestNode>::with_capacity(2);

        let node1 = pool.get_or_default();
        assert_eq!(pool.size(), 0);

        pool.put(node1);
        assert_eq!(pool.size(), 1);

        let _node2 = pool.get_or_default();
        assert_eq!(pool.size(), 0);

        let stats = pool.stats();
        assert_eq!(stats.gets, 2);
        assert_eq!(stats.puts, 1);
        assert_eq!(stats.misses, 1);
    }

    #[test]
    fn pool_capacity() {
        let pool = NodePool::<TestNode>::with_capacity(2);

        let node1 = Arc::new(TestNode { value: 1 });
        let node2 = Arc::new(TestNode { value: 2 });
        let node3 = Arc::new(TestNode { value: 3 });

        pool.put(node1);
        pool.put(node2);
        pool.put(node3);

        assert_eq!(pool.size(), 2);

        let stats = pool.stats();
        assert_eq!(stats.drops, 1);
    }

    #[test]
    fn pool_shared_references_not_pooled() {
        let pool = NodePool::<TestNode>::new();

        let node = Arc::new(TestNode { value: 42 });
        let node_clone = node.clone();

        pool.put(node);
        assert_eq!(pool.size(), 0);

        drop(node_clone);
    }
}
