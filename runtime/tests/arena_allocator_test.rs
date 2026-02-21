#![allow(unexpected_cfgs)]
//! Arena Allocator Tests
//!
//! This test suite implements all behavioral specifications from
//! docs/specs/ARENA_ALLOCATOR_SPEC.md
//!
//! Test-Driven Development: These tests define expected behavior before implementation.

use adze::arena_allocator::{NodeHandle, TreeArena, TreeNode};

// ============================================================================
// Spec 1: Basic Allocation
// ============================================================================

#[test]
fn spec_1_basic_allocation() {
    // Given: A new TreeArena
    let mut arena = TreeArena::new();

    // When: User calls alloc(node)
    let node = TreeNode::leaf(42);
    let handle = arena.alloc(node.clone());

    // Then: Returns valid NodeHandle, get() returns allocated node
    let retrieved = arena.get(handle);
    assert_eq!(retrieved.value(), node.value());
}

#[test]
fn spec_1_basic_allocation_preserves_data() {
    let mut arena = TreeArena::new();

    // Allocate various node types
    let h1 = arena.alloc(TreeNode::leaf(1));
    let h2 = arena.alloc(TreeNode::leaf(2));
    let h3 = arena.alloc(TreeNode::branch(vec![h1, h2]));

    // Verify all data preserved
    assert_eq!(arena.get(h1).value(), 1);
    assert_eq!(arena.get(h2).value(), 2);
    assert!(arena.get(h3).is_branch());
}

// ============================================================================
// Spec 2: Multiple Allocations and Chunk Growth
// ============================================================================

#[test]
fn spec_2_chunk_growth() {
    // Given: TreeArena with capacity N
    let mut arena = TreeArena::with_capacity(2);

    // When: User allocates N+1 nodes
    let h1 = arena.alloc(TreeNode::leaf(1));
    let h2 = arena.alloc(TreeNode::leaf(2));
    assert_eq!(arena.num_chunks(), 1, "Should use first chunk");

    let h3 = arena.alloc(TreeNode::leaf(3));

    // Then: New chunk allocated, all handles valid
    assert_eq!(arena.num_chunks(), 2, "Should have allocated second chunk");

    // All handles remain valid
    assert_eq!(arena.get(h1).value(), 1);
    assert_eq!(arena.get(h2).value(), 2);
    assert_eq!(arena.get(h3).value(), 3);
}

#[test]
fn spec_2_exponential_growth() {
    let mut arena = TreeArena::with_capacity(4);

    // Fill first chunk
    for i in 0..4 {
        arena.alloc(TreeNode::leaf(i));
    }
    assert_eq!(arena.num_chunks(), 1);

    // Trigger second chunk (should be 2x = 8)
    arena.alloc(TreeNode::leaf(999));
    assert_eq!(arena.num_chunks(), 2);

    // Capacity should grow exponentially
    assert!(arena.capacity() > 4, "Capacity should have grown");
}

#[test]
fn spec_2_handles_valid_across_chunks() {
    let mut arena = TreeArena::with_capacity(2);
    let mut handles = Vec::new();

    // Allocate across multiple chunks
    for i in 0..10 {
        handles.push(arena.alloc(TreeNode::leaf(i)));
    }

    // All handles from all chunks should be valid
    for (i, &handle) in handles.iter().enumerate() {
        assert_eq!(arena.get(handle).value(), i as i32);
    }
}

// ============================================================================
// Spec 3: Arena Reset
// ============================================================================

#[test]
fn spec_3_arena_reset() {
    // Given: Arena with N allocated nodes
    let mut arena = TreeArena::new();
    arena.alloc(TreeNode::leaf(1));
    arena.alloc(TreeNode::leaf(2));
    let initial_capacity = arena.capacity();

    // When: User calls reset()
    arena.reset();

    // Then: len() == 0, capacity unchanged, chunks retained
    assert_eq!(arena.len(), 0, "Arena should be empty after reset");
    assert_eq!(
        arena.capacity(),
        initial_capacity,
        "Capacity should be unchanged"
    );
    assert_eq!(arena.num_chunks(), 1, "First chunk should be retained");
}

#[test]
fn spec_3_reset_enables_reuse() {
    let mut arena = TreeArena::new();

    // First allocation session
    for i in 0..100 {
        arena.alloc(TreeNode::leaf(i));
    }
    let capacity_after_first = arena.capacity();

    // Reset
    arena.reset();

    // Second allocation session (same size)
    let mut new_handles = Vec::new();
    for i in 0..100 {
        new_handles.push(arena.alloc(TreeNode::leaf(i + 1000)));
    }

    // No new allocations needed
    assert_eq!(arena.capacity(), capacity_after_first);

    // New data is accessible
    assert_eq!(arena.get(new_handles[0]).value(), 1000);
}

// ============================================================================
// Spec 4: Lifetime Safety (Compile-Time Tests)
// ============================================================================

// Note: These are compile-fail tests that would be in a separate test suite
// with trybuild or similar. Documented here for completeness.

/*
#[test]
fn spec_4_lifetime_safety_arena_dropped() {
    // This should NOT compile
    let tree = {
        let mut arena = TreeArena::new();
        let root = arena.alloc(TreeNode::leaf(1));
        Tree { root, arena: &arena }
    }; // arena dropped here

    // Compilation error: arena does not live long enough
    let _root = tree.root();
}
*/

// ============================================================================
// Spec 5: Performance - Allocation Count
// ============================================================================

#[test]
fn spec_5_allocation_count_bounded() {
    let mut arena = TreeArena::with_capacity(1024);

    // Allocate 10,000 nodes
    for i in 0..10_000 {
        arena.alloc(TreeNode::leaf(i));
    }

    // With chunk size 1024, expect ~10 chunks (log₂-ish growth)
    // Each chunk is one allocation, vs 10,000 with Box
    assert!(
        arena.num_chunks() < 20,
        "Should have < 20 chunks (actual: {})",
        arena.num_chunks()
    );

    // Compare to Box: would be 10,000 allocations
    // Arena: ~10-15 allocations
    // Reduction: >99%
}

// ============================================================================
// Spec 7: Memory Reuse
// ============================================================================

#[test]
fn spec_7_memory_reuse_same_size() {
    let mut arena = TreeArena::new();

    // First parse
    for i in 0..1000 {
        arena.alloc(TreeNode::leaf(i));
    }
    let capacity_after_first = arena.capacity();
    let chunks_after_first = arena.num_chunks();

    arena.reset();

    // Second parse (same size)
    for i in 0..1000 {
        arena.alloc(TreeNode::leaf(i));
    }

    // No new allocations
    assert_eq!(arena.capacity(), capacity_after_first);
    assert_eq!(arena.num_chunks(), chunks_after_first);
}

#[test]
fn spec_7_memory_reuse_grows_if_needed() {
    let mut arena = TreeArena::new();

    // First parse (small)
    for i in 0..100 {
        arena.alloc(TreeNode::leaf(i));
    }
    let chunks_after_first = arena.num_chunks();

    arena.reset();

    // Second parse (larger)
    for i in 0..5000 {
        arena.alloc(TreeNode::leaf(i));
    }

    // Should have allocated more chunks
    assert!(
        arena.num_chunks() > chunks_after_first,
        "Should allocate additional chunks for larger parse"
    );
}

// ============================================================================
// Metrics and Introspection
// ============================================================================

#[test]
fn test_arena_metrics() {
    let mut arena = TreeArena::new();

    // Initially empty
    assert_eq!(arena.len(), 0);
    assert!(arena.is_empty());

    // Add nodes
    arena.alloc(TreeNode::leaf(1));
    arena.alloc(TreeNode::leaf(2));

    assert_eq!(arena.len(), 2);
    assert!(!arena.is_empty());
    assert!(arena.capacity() >= 2);

    // Reset clears length but keeps capacity
    arena.reset();
    assert_eq!(arena.len(), 0);
    assert!(arena.capacity() > 0);
}

#[test]
fn test_memory_usage_tracking() {
    let mut arena = TreeArena::new();

    let initial_memory = arena.memory_usage();
    assert!(initial_memory > 0, "Should have initial chunk allocated");

    // Allocate enough nodes to trigger chunk growth (default chunk size is 1024)
    for i in 0..2000 {
        arena.alloc(TreeNode::leaf(i));
    }

    let after_alloc = arena.memory_usage();
    assert!(after_alloc > initial_memory, "Memory usage should increase");
}

// ============================================================================
// Clear vs Reset
// ============================================================================

#[test]
fn test_clear_vs_reset() {
    let mut arena = TreeArena::with_capacity(10);

    // Allocate many nodes to create multiple chunks
    for i in 0..100 {
        arena.alloc(TreeNode::leaf(i));
    }
    let num_chunks_before = arena.num_chunks();
    assert!(num_chunks_before > 1, "Should have multiple chunks");

    // reset() keeps chunks
    arena.reset();
    assert_eq!(arena.len(), 0);
    assert_eq!(arena.num_chunks(), num_chunks_before);

    // Allocate again
    for i in 0..100 {
        arena.alloc(TreeNode::leaf(i));
    }

    // clear() reduces to single chunk
    arena.clear();
    assert_eq!(arena.len(), 0);
    assert_eq!(arena.num_chunks(), 1, "clear() should reduce to one chunk");
}

// ============================================================================
// Handle Validity (Debug Assertions)
// ============================================================================

#[test]
#[should_panic(expected = "Invalid node handle")]
#[cfg(debug_assertions)]
fn test_invalid_handle_panics() {
    let arena = TreeArena::new();

    // Create invalid handle manually
    let invalid_handle = NodeHandle::new(999, 999);

    // Should panic in debug builds
    let _node = arena.get(invalid_handle);
}

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
fn test_single_node() {
    let mut arena = TreeArena::new();
    let handle = arena.alloc(TreeNode::leaf(42));

    assert_eq!(arena.len(), 1);
    assert_eq!(arena.get(handle).value(), 42);
}

#[test]
fn test_capacity_boundaries() {
    let mut arena = TreeArena::with_capacity(3);

    // Fill exactly to capacity
    let h1 = arena.alloc(TreeNode::leaf(1));
    let _h2 = arena.alloc(TreeNode::leaf(2));
    let _h3 = arena.alloc(TreeNode::leaf(3));

    assert_eq!(arena.num_chunks(), 1);

    // One more triggers new chunk
    let h4 = arena.alloc(TreeNode::leaf(4));
    assert_eq!(arena.num_chunks(), 2);

    // All handles valid
    assert_eq!(arena.get(h1).value(), 1);
    assert_eq!(arena.get(h4).value(), 4);
}

#[test]
fn test_large_allocation() {
    let mut arena = TreeArena::new();

    // Allocate many nodes
    let handles: Vec<_> = (0..10_000)
        .map(|i| arena.alloc(TreeNode::leaf(i)))
        .collect();

    // Spot check
    assert_eq!(arena.get(handles[0]).value(), 0);
    assert_eq!(arena.get(handles[5000]).value(), 5000);
    assert_eq!(arena.get(handles[9999]).value(), 9999);

    assert_eq!(arena.len(), 10_000);
}

// ============================================================================
// Mutable Access
// ============================================================================

#[test]
fn test_mutable_access() {
    let mut arena = TreeArena::new();
    let handle = arena.alloc(TreeNode::leaf(10));

    // Immutable access
    assert_eq!(arena.get(handle).value(), 10);

    // Mutable access
    {
        let mut node_mut = arena.get_mut(handle);
        node_mut.set_value(20);
    }

    // Verify mutation
    assert_eq!(arena.get(handle).value(), 20);
}

// ============================================================================
// Property Tests (if proptest is available)
// ============================================================================

#[allow(unexpected_cfgs)]
#[cfg(feature = "proptest")]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn prop_all_allocated_nodes_retrievable(values in prop::collection::vec(any::<i32>(), 0..1000)) {
            let mut arena = TreeArena::new();
            let handles: Vec<_> = values.iter()
                .map(|&v| arena.alloc(TreeNode::leaf(v)))
                .collect();

            for (&handle, &expected_value) in handles.iter().zip(values.iter()) {
                assert_eq!(arena.get(handle).value(), expected_value);
            }
        }

        #[test]
        fn prop_reset_allows_reallocation(n in 1usize..1000) {
            let mut arena = TreeArena::new();

            // First session
            for i in 0..n {
                arena.alloc(TreeNode::leaf(i as i32));
            }
            let cap1 = arena.capacity();

            arena.reset();

            // Second session
            for i in 0..n {
                arena.alloc(TreeNode::leaf(i as i32));
            }

            // Capacity should not grow if same size
            assert_eq!(arena.capacity(), cap1);
        }
    }
}
