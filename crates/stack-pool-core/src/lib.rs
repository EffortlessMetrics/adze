//! Small stack pool used by the parser to amortize allocations.
#![forbid(unsafe_op_in_unsafe_fn)]
#![deny(missing_docs)]
#![cfg_attr(feature = "strict_api", deny(unreachable_pub))]
#![cfg_attr(not(feature = "strict_api"), warn(unreachable_pub))]
#![cfg_attr(feature = "strict_docs", deny(missing_docs))]
#![cfg_attr(not(feature = "strict_docs"), allow(missing_docs))]

use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;

/// A pool of reusable stacks to reduce allocation overhead.
///
/// # Examples
///
/// ```
/// use adze_stack_pool_core::StackPool;
///
/// let pool: StackPool<u32> = StackPool::new(4);
/// let mut stack = pool.acquire();
/// stack.push(42);
/// pool.release(stack);
///
/// let reused = pool.acquire();
/// assert!(reused.is_empty()); // cleared on reuse
/// assert_eq!(pool.stats().reuse_count, 1);
/// ```
pub struct StackPool<T: Clone> {
    /// Pool of available stacks ready for reuse.
    available: RefCell<VecDeque<Vec<T>>>,
    /// Maximum number of stacks to keep in the pool.
    max_pool_size: usize,
    /// Statistics for monitoring pool performance.
    stats: RefCell<PoolStats>,
}

/// Statistics for stack pool usage.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PoolStats {
    /// Total number of stacks allocated.
    pub total_allocations: usize,
    /// Number of times a pooled stack was reused.
    pub reuse_count: usize,
    /// Number of direct pool hits.
    pub pool_hits: usize,
    /// Number of misses requiring new allocation.
    pub pool_misses: usize,
    /// Maximum observed pool depth.
    pub max_pool_depth: usize,
}

impl<T: Clone> std::fmt::Debug for StackPool<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StackPool")
            .field("available", &self.available.borrow().len())
            .field("max_pool_size", &self.max_pool_size)
            .field("stats", &*self.stats.borrow())
            .finish()
    }
}

impl<T: Clone> StackPool<T> {
    /// Create a new stack pool with the specified maximum size.
    ///
    /// # Examples
    ///
    /// ```
    /// use adze_stack_pool_core::StackPool;
    ///
    /// let pool: StackPool<i32> = StackPool::new(8);
    /// assert_eq!(pool.stats().total_allocations, 0);
    /// ```
    #[must_use]
    pub fn new(max_pool_size: usize) -> Self {
        StackPool {
            available: RefCell::new(VecDeque::with_capacity(max_pool_size)),
            max_pool_size,
            stats: RefCell::new(PoolStats::default()),
        }
    }

    /// Acquire a stack from the pool, or allocate a new one if pool is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use adze_stack_pool_core::StackPool;
    ///
    /// let pool: StackPool<u32> = StackPool::new(4);
    /// let stack = pool.acquire();
    /// assert_eq!(stack.capacity(), 256);
    /// assert_eq!(pool.stats().pool_misses, 1);
    /// ```
    #[must_use]
    pub fn acquire(&self) -> Vec<T> {
        let mut pool = self.available.borrow_mut();
        let mut stats = self.stats.borrow_mut();

        if let Some(mut stack) = pool.pop_front() {
            stack.clear();
            stats.pool_hits += 1;
            stats.reuse_count += 1;
            stack
        } else {
            stats.pool_misses += 1;
            stats.total_allocations += 1;
            Vec::with_capacity(256)
        }
    }

    /// Acquire a stack with at least the requested capacity.
    #[must_use]
    pub fn acquire_with_capacity(&self, capacity: usize) -> Vec<T> {
        let mut pool = self.available.borrow_mut();
        let mut stats = self.stats.borrow_mut();

        if let Some(pos) = pool.iter().position(|s| s.capacity() >= capacity) {
            let mut stack = pool.remove(pos).unwrap();
            stack.clear();
            stats.pool_hits += 1;
            stats.reuse_count += 1;
            stack
        } else {
            stats.pool_misses += 1;
            stats.total_allocations += 1;
            Vec::with_capacity(capacity)
        }
    }

    /// Return a stack to the pool for reuse.
    ///
    /// # Examples
    ///
    /// ```
    /// use adze_stack_pool_core::StackPool;
    ///
    /// let pool: StackPool<u32> = StackPool::new(4);
    /// let stack = pool.acquire();
    /// pool.release(stack);
    /// assert_eq!(pool.stats().max_pool_depth, 1);
    /// ```
    pub fn release(&self, mut stack: Vec<T>) {
        let mut pool = self.available.borrow_mut();

        if stack.capacity() <= 4096 && pool.len() < self.max_pool_size {
            stack.clear();
            pool.push_back(stack);

            let mut stats = self.stats.borrow_mut();
            stats.max_pool_depth = stats.max_pool_depth.max(pool.len());
        }
    }

    /// Clone a stack, potentially using a pooled stack for the destination.
    ///
    /// # Examples
    ///
    /// ```
    /// use adze_stack_pool_core::StackPool;
    ///
    /// let pool: StackPool<u32> = StackPool::new(4);
    /// let original = vec![1, 2, 3];
    /// let cloned = pool.clone_stack(&original);
    /// assert_eq!(cloned, vec![1, 2, 3]);
    /// ```
    #[must_use]
    pub fn clone_stack(&self, source: &[T]) -> Vec<T> {
        let mut dest = self.acquire_with_capacity(source.len());
        dest.extend_from_slice(source);
        dest
    }

    /// Get current pool statistics.
    ///
    /// # Examples
    ///
    /// ```
    /// use adze_stack_pool_core::StackPool;
    ///
    /// let pool: StackPool<u32> = StackPool::new(4);
    /// let _ = pool.acquire();
    /// let stats = pool.stats();
    /// assert_eq!(stats.total_allocations, 1);
    /// assert_eq!(stats.pool_misses, 1);
    /// ```
    #[must_use]
    pub fn stats(&self) -> PoolStats {
        *self.stats.borrow()
    }

    /// Reset statistics.
    pub fn reset_stats(&self) {
        *self.stats.borrow_mut() = PoolStats::default();
    }

    /// Clear the pool, releasing all cached stacks.
    pub fn clear(&self) {
        self.available.borrow_mut().clear();
    }
}

thread_local! {
    /// Thread-local stack pool for single-threaded parsing.
    static STACK_POOL: RefCell<Option<Rc<StackPool<u32>>>> = const { RefCell::new(None) };
}

/// Initialize the thread-local stack pool.
pub fn init_thread_local_pool(max_size: usize) {
    STACK_POOL.with(|pool| {
        *pool.borrow_mut() = Some(Rc::new(StackPool::new(max_size)));
    });
}

/// Get the thread-local stack pool, initializing if necessary.
#[must_use]
pub fn get_thread_local_pool() -> Rc<StackPool<u32>> {
    STACK_POOL.with(|pool| {
        let mut pool_ref = pool.borrow_mut();
        if pool_ref.is_none() {
            *pool_ref = Some(Rc::new(StackPool::new(64)));
        }

        pool_ref.as_ref().unwrap().clone()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pool_tracks_reuse_via_release_and_reacquire() {
        let pool: StackPool<u32> = StackPool::new(2);

        let mut stack = pool.acquire();
        stack.push(1);
        pool.release(stack);

        let reused = pool.acquire();
        assert!(reused.is_empty());
        assert_eq!(pool.stats().pool_hits, 1);
        assert_eq!(pool.stats().reuse_count, 1);
    }

    #[test]
    fn acquires_with_capacity_can_reuse_matching_or_larger_stack() {
        let pool: StackPool<u32> = StackPool::new(2);

        let stack_small = Vec::with_capacity(16);
        let stack_medium = Vec::with_capacity(128);

        pool.release(stack_small);
        pool.release(stack_medium);

        let acquired = pool.acquire_with_capacity(64);
        assert!(acquired.capacity() >= 128);

        let stats = pool.stats();
        assert_eq!(stats.pool_hits, 1);
    }

    #[test]
    fn pool_ignores_oversized_stacks() {
        let pool: StackPool<u32> = StackPool::new(1);

        let oversized = vec![0u32; 4097];
        pool.release(oversized);

        assert_eq!(pool.stats().max_pool_depth, 0);
    }

    #[test]
    fn thread_local_pool_defaults_and_reuses() {
        init_thread_local_pool(3);

        let pool = get_thread_local_pool();
        let stack = pool.acquire();
        assert_eq!(stack.capacity(), 256);

        pool.release(stack);

        let stats = pool.stats();
        assert_eq!(stats.total_allocations, 1);
    }

    #[test]
    fn clone_stack_copies_contents() {
        let pool: StackPool<u32> = StackPool::new(4);

        let original = vec![1, 2, 3, 4];
        let cloned = pool.clone_stack(&original);

        assert_eq!(cloned, original);
    }

    // --- Mutation-catching tests ---

    #[test]
    fn release_accepts_stack_at_capacity_boundary() {
        let pool: StackPool<u32> = StackPool::new(2);
        let stack: Vec<u32> = Vec::with_capacity(4096);
        pool.release(stack);
        assert_eq!(pool.stats().max_pool_depth, 1);
    }

    #[test]
    fn release_rejects_stack_just_over_capacity_boundary() {
        let pool: StackPool<u32> = StackPool::new(2);
        let stack: Vec<u32> = Vec::with_capacity(4097);
        pool.release(stack);
        assert_eq!(pool.stats().max_pool_depth, 0);
    }

    #[test]
    fn pool_full_rejects_additional_release() {
        let pool: StackPool<u32> = StackPool::new(1);
        pool.release(Vec::with_capacity(8));
        pool.release(Vec::with_capacity(8));
        assert_eq!(pool.stats().max_pool_depth, 1);
    }

    #[test]
    fn reset_stats_zeroes_all_fields() {
        let pool: StackPool<u32> = StackPool::new(4);
        let s = pool.acquire();
        pool.release(s);
        let _ = pool.acquire();

        pool.reset_stats();
        let stats = pool.stats();
        assert_eq!(stats.total_allocations, 0);
        assert_eq!(stats.reuse_count, 0);
        assert_eq!(stats.pool_hits, 0);
        assert_eq!(stats.pool_misses, 0);
        assert_eq!(stats.max_pool_depth, 0);
    }

    #[test]
    fn acquire_from_empty_pool_always_misses() {
        let pool: StackPool<u32> = StackPool::new(4);
        let _ = pool.acquire();
        assert_eq!(pool.stats().pool_misses, 1);
        assert_eq!(pool.stats().pool_hits, 0);
        assert_eq!(pool.stats().total_allocations, 1);
    }

    #[test]
    fn acquire_with_capacity_from_empty_pool_misses() {
        let pool: StackPool<u32> = StackPool::new(4);
        let s = pool.acquire_with_capacity(64);
        assert!(s.capacity() >= 64);
        assert_eq!(pool.stats().pool_misses, 1);
        assert_eq!(pool.stats().pool_hits, 0);
    }

    #[test]
    fn clear_empties_the_pool() {
        let pool: StackPool<u32> = StackPool::new(4);
        let s1 = pool.acquire();
        let s2 = pool.acquire();
        pool.release(s1);
        pool.release(s2);
        pool.clear();
        pool.reset_stats();

        let _ = pool.acquire();
        assert_eq!(pool.stats().pool_hits, 0);
        assert_eq!(pool.stats().pool_misses, 1);
    }

    #[test]
    fn default_acquire_capacity_is_256() {
        let pool: StackPool<u32> = StackPool::new(4);
        let s = pool.acquire();
        assert_eq!(s.capacity(), 256);
    }
}
