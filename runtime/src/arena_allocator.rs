use std::cell::{Cell, RefCell};
use std::mem;
use std::ptr;

/// Arena allocator for efficient allocation of parse tree nodes
/// This reduces allocation overhead and improves cache locality
pub struct Arena<T> {
    /// Current chunk being allocated from
    current_chunk: RefCell<Vec<T>>,
    /// Completed chunks that are full
    chunks: RefCell<Vec<Vec<T>>>,
    /// Current allocation position in the current chunk
    position: Cell<usize>,
    /// Size of each chunk
    chunk_size: usize,
    /// Statistics
    stats: RefCell<ArenaStats>,
}

#[derive(Debug, Default, Clone)]
pub struct ArenaStats {
    pub total_allocations: usize,
    pub total_chunks: usize,
    pub bytes_allocated: usize,
    pub bytes_wasted: usize,
}

impl<T: Clone> Arena<T> {
    /// Create a new arena with the specified chunk size
    pub fn new(chunk_size: usize) -> Self {
        Arena {
            current_chunk: RefCell::new(Vec::with_capacity(chunk_size)),
            chunks: RefCell::new(Vec::new()),
            position: Cell::new(0),
            chunk_size,
            stats: RefCell::new(ArenaStats::default()),
        }
    }
    
    /// Allocate space for one item in the arena
    pub fn alloc(&self, value: T) -> ArenaRef<T> {
        let pos = self.position.get();
        let mut chunk = self.current_chunk.borrow_mut();
        
        if pos >= self.chunk_size {
            // Current chunk is full, create a new one
            let old_chunk = mem::replace(&mut *chunk, Vec::with_capacity(self.chunk_size));
            self.chunks.borrow_mut().push(old_chunk);
            self.position.set(0);
            
            let mut stats = self.stats.borrow_mut();
            stats.total_chunks += 1;
            stats.bytes_wasted += (self.chunk_size - pos) * mem::size_of::<T>();
        }
        
        chunk.push(value.clone());
        self.position.set(pos + 1);
        
        let mut stats = self.stats.borrow_mut();
        stats.total_allocations += 1;
        stats.bytes_allocated += mem::size_of::<T>();
        
        // Return a reference to the allocated item
        ArenaRef {
            value,
        }
    }
    
    /// Allocate space for multiple items at once
    pub fn alloc_slice(&self, values: &[T]) -> Vec<ArenaRef<T>> {
        values.iter().map(|v| self.alloc(v.clone())).collect()
    }
    
    /// Clear the arena, releasing all allocations
    pub fn clear(&self) {
        self.current_chunk.borrow_mut().clear();
        self.chunks.borrow_mut().clear();
        self.position.set(0);
        *self.stats.borrow_mut() = ArenaStats::default();
    }
    
    /// Get arena statistics
    pub fn stats(&self) -> ArenaStats {
        self.stats.borrow().clone()
    }
}

/// Reference to an arena-allocated value
pub struct ArenaRef<T> {
    value: T,
}

impl<T> ArenaRef<T> {
    pub fn get(&self) -> &T {
        &self.value
    }
}

impl<T> std::ops::Deref for ArenaRef<T> {
    type Target = T;
    
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

/// Type-erased arena for heterogeneous allocations
pub struct TypedArena {
    /// Byte buffer for allocations
    buffer: RefCell<Vec<u8>>,
    /// Current position in buffer
    position: Cell<usize>,
    /// Capacity of current buffer
    capacity: Cell<usize>,
    /// All allocated buffers
    buffers: RefCell<Vec<Vec<u8>>>,
    /// Statistics
    stats: RefCell<ArenaStats>,
}

impl TypedArena {
    pub fn new(initial_capacity: usize) -> Self {
        TypedArena {
            buffer: RefCell::new(Vec::with_capacity(initial_capacity)),
            position: Cell::new(0),
            capacity: Cell::new(initial_capacity),
            buffers: RefCell::new(Vec::new()),
            stats: RefCell::new(ArenaStats::default()),
        }
    }
    
    /// Allocate space for a value of type T
    pub unsafe fn alloc<T>(&self, value: T) -> *mut T {
        let size = mem::size_of::<T>();
        let align = mem::align_of::<T>();
        
        // Align the position
        let pos = self.position.get();
        let aligned_pos = (pos + align - 1) & !(align - 1);
        
        if aligned_pos + size > self.capacity.get() {
            // Need a new buffer
            self.grow(size.max(self.capacity.get()));
        }
        
        let mut buffer = self.buffer.borrow_mut();
        buffer.resize(aligned_pos + size, 0);
        
        let ptr = unsafe {
            let ptr = buffer.as_mut_ptr().add(aligned_pos) as *mut T;
            ptr::write(ptr, value);
            ptr
        };
        
        self.position.set(aligned_pos + size);
        
        let mut stats = self.stats.borrow_mut();
        stats.total_allocations += 1;
        stats.bytes_allocated += size;
        
        ptr
    }
    
    fn grow(&self, min_size: usize) {
        let new_capacity = (self.capacity.get() * 2).max(min_size);
        let old_buffer = mem::replace(
            &mut *self.buffer.borrow_mut(),
            Vec::with_capacity(new_capacity)
        );
        
        if !old_buffer.is_empty() {
            self.buffers.borrow_mut().push(old_buffer);
        }
        
        self.position.set(0);
        self.capacity.set(new_capacity);
        
        let mut stats = self.stats.borrow_mut();
        stats.total_chunks += 1;
    }
    
    pub fn stats(&self) -> ArenaStats {
        self.stats.borrow().clone()
    }
    
    pub fn clear(&self) {
        self.buffer.borrow_mut().clear();
        self.buffers.borrow_mut().clear();
        self.position.set(0);
        *self.stats.borrow_mut() = ArenaStats::default();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_basic_arena() {
        let arena = Arena::new(10);
        
        let refs: Vec<_> = (0..25).map(|i| arena.alloc(i)).collect();
        
        assert_eq!(*refs[0], 0);
        assert_eq!(*refs[24], 24);
        
        let stats = arena.stats();
        assert_eq!(stats.total_allocations, 25);
        assert!(stats.total_chunks >= 2); // Should have allocated at least 2 chunks
    }
    
    #[test]
    fn test_arena_slice() {
        let arena = Arena::new(10);
        let values = vec![1, 2, 3, 4, 5];
        
        let refs = arena.alloc_slice(&values);
        
        for (i, r) in refs.iter().enumerate() {
            assert_eq!(**r, values[i]);
        }
    }
    
    #[test]
    fn test_typed_arena() {
        let arena = TypedArena::new(1024);
        
        unsafe {
            let i32_ptr = arena.alloc(42i32);
            let str_ptr = arena.alloc(String::from("hello"));
            let vec_ptr = arena.alloc(vec![1, 2, 3]);
            
            assert_eq!(*i32_ptr, 42);
            assert_eq!(&*str_ptr, "hello");
            assert_eq!(*vec_ptr, vec![1, 2, 3]);
        }
        
        let stats = arena.stats();
        assert_eq!(stats.total_allocations, 3);
    }
    
    #[test]
    fn test_arena_clear() {
        let arena = Arena::new(10);
        
        for i in 0..20 {
            arena.alloc(i);
        }
        
        let stats_before = arena.stats();
        assert_eq!(stats_before.total_allocations, 20);
        
        arena.clear();
        
        let stats_after = arena.stats();
        assert_eq!(stats_after.total_allocations, 0);
    }
}