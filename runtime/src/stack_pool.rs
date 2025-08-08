use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;

/// A pool of reusable parse stacks to reduce allocation overhead
/// during GLR parsing when forks are created and destroyed frequently.
pub struct StackPool<T: Clone> {
    /// Pool of available stacks ready for reuse
    available: RefCell<VecDeque<Vec<T>>>,
    /// Maximum number of stacks to keep in the pool
    max_pool_size: usize,
    /// Statistics for monitoring pool performance
    stats: RefCell<PoolStats>,
}

#[derive(Debug, Default, Clone)]
pub struct PoolStats {
    pub total_allocations: usize,
    pub reuse_count: usize,
    pub pool_hits: usize,
    pub pool_misses: usize,
    pub max_pool_depth: usize,
}

impl<T: Clone> StackPool<T> {
    /// Create a new stack pool with the specified maximum size
    pub fn new(max_pool_size: usize) -> Self {
        StackPool {
            available: RefCell::new(VecDeque::with_capacity(max_pool_size)),
            max_pool_size,
            stats: RefCell::new(PoolStats::default()),
        }
    }
    
    /// Acquire a stack from the pool, or allocate a new one if pool is empty
    pub fn acquire(&self) -> Vec<T> {
        let mut pool = self.available.borrow_mut();
        let mut stats = self.stats.borrow_mut();
        
        if let Some(mut stack) = pool.pop_front() {
            stack.clear(); // Ensure it's empty for reuse
            stats.pool_hits += 1;
            stats.reuse_count += 1;
            stack
        } else {
            stats.pool_misses += 1;
            stats.total_allocations += 1;
            Vec::with_capacity(256) // Default capacity for parse stacks
        }
    }
    
    /// Acquire a stack with specific initial capacity
    pub fn acquire_with_capacity(&self, capacity: usize) -> Vec<T> {
        let mut pool = self.available.borrow_mut();
        let mut stats = self.stats.borrow_mut();
        
        // Try to find a stack with at least the requested capacity
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
    
    /// Return a stack to the pool for reuse
    pub fn release(&self, mut stack: Vec<T>) {
        let mut pool = self.available.borrow_mut();
        
        // Only keep stacks that aren't too large (to avoid memory bloat)
        if stack.capacity() <= 4096 && pool.len() < self.max_pool_size {
            stack.clear();
            pool.push_back(stack);
            
            let mut stats = self.stats.borrow_mut();
            stats.max_pool_depth = stats.max_pool_depth.max(pool.len());
        }
        // Otherwise, let the stack be dropped and deallocated
    }
    
    /// Clone a stack, potentially using a pooled stack for the destination
    pub fn clone_stack(&self, source: &[T]) -> Vec<T> {
        let mut dest = self.acquire_with_capacity(source.len());
        dest.extend_from_slice(source);
        dest
    }
    
    /// Get current pool statistics
    pub fn stats(&self) -> PoolStats {
        self.stats.borrow().clone()
    }
    
    /// Reset statistics
    pub fn reset_stats(&self) {
        *self.stats.borrow_mut() = PoolStats::default();
    }
    
    /// Clear the pool, releasing all cached stacks
    pub fn clear(&self) {
        self.available.borrow_mut().clear();
    }
}

thread_local! {
    // Thread-local stack pool for single-threaded parsing
    static STACK_POOL: RefCell<Option<Rc<StackPool<u32>>>> = const { RefCell::new(None) };
}

/// Initialize the thread-local stack pool
pub fn init_thread_local_pool(max_size: usize) {
    STACK_POOL.with(|pool| {
        *pool.borrow_mut() = Some(Rc::new(StackPool::new(max_size)));
    });
}

/// Get the thread-local stack pool, initializing if necessary
pub fn get_thread_local_pool() -> Rc<StackPool<u32>> {
    STACK_POOL.with(|pool| {
        let mut pool_ref = pool.borrow_mut();
        if pool_ref.is_none() {
            *pool_ref = Some(Rc::new(StackPool::new(64))); // Default pool size
        }
        pool_ref.as_ref().unwrap().clone()
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_pool_basic_operations() {
        let pool: StackPool<u32> = StackPool::new(10);
        
        // Acquire a stack
        let mut stack1 = pool.acquire();
        assert_eq!(stack1.capacity(), 256);
        stack1.push(1);
        stack1.push(2);
        stack1.push(3);
        
        // Release it back to the pool
        pool.release(stack1);
        
        // Acquire again - should get the same stack back (cleared)
        let stack2 = pool.acquire();
        assert_eq!(stack2.len(), 0);
        assert!(stack2.capacity() >= 3);
        
        let stats = pool.stats();
        assert_eq!(stats.pool_hits, 1);
        assert_eq!(stats.pool_misses, 1);
        assert_eq!(stats.reuse_count, 1);
    }
    
    #[test]
    fn test_pool_capacity_matching() {
        let pool: StackPool<u32> = StackPool::new(10);
        
        // Create stacks with different capacities
        let stack_small = Vec::with_capacity(10);
        let stack_medium = Vec::with_capacity(100);
        let stack_large = Vec::with_capacity(1000);
        
        pool.release(stack_small);
        pool.release(stack_medium);
        pool.release(stack_large);
        
        // Request a stack with specific capacity
        let acquired = pool.acquire_with_capacity(50);
        assert!(acquired.capacity() >= 100); // Should get the medium one
        
        let stats = pool.stats();
        assert_eq!(stats.pool_hits, 1);
    }
    
    #[test]
    fn test_pool_size_limit() {
        let pool: StackPool<u32> = StackPool::new(2);
        
        // Release more stacks than the pool can hold
        for i in 0..5 {
            let mut stack = Vec::new();
            stack.push(i);
            pool.release(stack);
        }
        
        // Pool should only keep 2 stacks
        let stats = pool.stats();
        assert_eq!(stats.max_pool_depth, 2);
    }
    
    #[test]
    fn test_clone_stack() {
        let pool: StackPool<u32> = StackPool::new(10);
        
        let source = vec![1, 2, 3, 4, 5];
        let cloned = pool.clone_stack(&source);
        
        assert_eq!(cloned, source);
        assert!(cloned.capacity() >= source.len());
    }
    
    #[test]
    fn test_thread_local_pool() {
        init_thread_local_pool(10);
        
        let pool = get_thread_local_pool();
        let stack = pool.acquire();
        assert_eq!(stack.capacity(), 256);
        
        pool.release(stack);
        
        let stats = pool.stats();
        assert_eq!(stats.total_allocations, 1);
    }
}