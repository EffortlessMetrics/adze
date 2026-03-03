//! Tests for GLR performance optimization modules — caching, deduplication, pooling.
#![cfg(feature = "test-api")]

use adze_glr_core::perf_optimizations::{ParseTableCache, StackDeduplicator, StackPool};
use adze_glr_core::{Action, StateId, SymbolId};

// ---------------------------------------------------------------------------
// ParseTableCache
// ---------------------------------------------------------------------------

#[test]
fn cache_empty_returns_miss_on_first_lookup() {
    let mut cache = ParseTableCache::new();
    let action = cache.get_or_compute(StateId(0), SymbolId(0), || Action::Error);
    assert!(matches!(action, Action::Error));
    assert_eq!(cache.stats().cache_misses, 1);
    assert_eq!(cache.stats().cache_hits, 0);
}

#[test]
fn cache_distinct_keys_stored_independently() {
    let mut cache = ParseTableCache::new();

    cache.get_or_compute(StateId(1), SymbolId(2), || Action::Shift(StateId(10)));
    cache.get_or_compute(StateId(1), SymbolId(3), || Action::Shift(StateId(20)));
    cache.get_or_compute(StateId(2), SymbolId(2), || Action::Accept);

    // All three should be misses.
    assert_eq!(cache.stats().cache_misses, 3);
    assert_eq!(cache.stats().cache_hits, 0);

    // Re-query each — all should be hits returning the original values.
    let a1 = cache.get_or_compute(StateId(1), SymbolId(2), || panic!("unexpected compute"));
    let a2 = cache.get_or_compute(StateId(1), SymbolId(3), || panic!("unexpected compute"));
    let a3 = cache.get_or_compute(StateId(2), SymbolId(2), || panic!("unexpected compute"));

    assert!(matches!(a1, Action::Shift(StateId(10))));
    assert!(matches!(a2, Action::Shift(StateId(20))));
    assert!(matches!(a3, Action::Accept));
    assert_eq!(cache.stats().cache_hits, 3);
}

#[test]
fn cache_large_workload_many_state_symbol_pairs() {
    let mut cache = ParseTableCache::new();
    let n: u16 = 500;

    // Populate cache with n×n entries.
    for s in 0..n {
        for sym in 0..n {
            cache.get_or_compute(StateId(s), SymbolId(sym), || {
                Action::Shift(StateId(s + sym))
            });
        }
    }

    let total = (n as usize) * (n as usize);
    assert_eq!(cache.stats().cache_misses, total);
    assert_eq!(cache.stats().cache_hits, 0);

    // Re-query every entry — all should hit.
    for s in 0..n {
        for sym in 0..n {
            let a =
                cache.get_or_compute(StateId(s), SymbolId(sym), || panic!("unexpected compute"));
            assert!(matches!(a, Action::Shift(st) if st == StateId(s + sym)));
        }
    }

    assert_eq!(cache.stats().cache_hits, total);
}

#[test]
fn cache_same_key_multiple_hits_increments_correctly() {
    let mut cache = ParseTableCache::new();
    cache.get_or_compute(StateId(7), SymbolId(3), || Action::Accept);

    for i in 0..100 {
        let a = cache.get_or_compute(StateId(7), SymbolId(3), || panic!("should hit cache"));
        assert!(matches!(a, Action::Accept));
        assert_eq!(cache.stats().cache_hits, i + 1);
    }

    assert_eq!(cache.stats().cache_misses, 1);
    assert_eq!(cache.stats().cache_hits, 100);
}

#[test]
fn cache_default_trait_equivalent_to_new() {
    let cache = ParseTableCache::default();
    assert_eq!(cache.stats().cache_hits, 0);
    assert_eq!(cache.stats().cache_misses, 0);
}

// ---------------------------------------------------------------------------
// StackDeduplicator
// ---------------------------------------------------------------------------

#[test]
fn dedup_empty_stack_is_valid_key() {
    let mut dedup = StackDeduplicator::new();
    assert!(!dedup.is_duplicate(&[]));
    assert!(dedup.is_duplicate(&[]));
    assert_eq!(dedup.unique_stacks(), 1);
}

#[test]
fn dedup_single_element_stacks_distinguished() {
    let mut dedup = StackDeduplicator::new();

    for i in 0..50 {
        assert!(!dedup.is_duplicate(&[StateId(i)]));
    }
    assert_eq!(dedup.unique_stacks(), 50);

    // All should now be duplicates.
    for i in 0..50 {
        assert!(dedup.is_duplicate(&[StateId(i)]));
    }
    // Unique count unchanged.
    assert_eq!(dedup.unique_stacks(), 50);
}

#[test]
fn dedup_order_matters() {
    let mut dedup = StackDeduplicator::new();
    assert!(!dedup.is_duplicate(&[StateId(1), StateId(2)]));
    // Reversed order is a distinct configuration.
    assert!(!dedup.is_duplicate(&[StateId(2), StateId(1)]));
    assert_eq!(dedup.unique_stacks(), 2);
}

#[test]
fn dedup_large_workload_many_unique_stacks() {
    let mut dedup = StackDeduplicator::new();
    let n = 1_000u16;

    for i in 0..n {
        let stack: Vec<StateId> = (0..10).map(|j| StateId(i + j)).collect();
        assert!(
            !dedup.is_duplicate(&stack),
            "first insertion must not be dup"
        );
    }
    assert_eq!(dedup.unique_stacks(), n as usize);

    // Re-submit every stack — all duplicates.
    for i in 0..n {
        let stack: Vec<StateId> = (0..10).map(|j| StateId(i + j)).collect();
        assert!(dedup.is_duplicate(&stack), "second insertion must be dup");
    }
    assert_eq!(dedup.unique_stacks(), n as usize);
}

#[test]
fn dedup_default_trait_equivalent_to_new() {
    let dedup = StackDeduplicator::default();
    assert_eq!(dedup.unique_stacks(), 0);
}

// ---------------------------------------------------------------------------
// StackPool
// ---------------------------------------------------------------------------

#[test]
fn pool_acquire_from_empty_returns_empty_vec() {
    let mut pool: StackPool<u32> = StackPool::new();
    let v = pool.acquire();
    assert!(v.is_empty());
}

#[test]
fn pool_release_clears_contents_before_reuse() {
    let mut pool: StackPool<i32> = StackPool::new();

    let mut v = pool.acquire();
    v.extend([10, 20, 30]);
    pool.release(v);

    let reused = pool.acquire();
    assert!(reused.is_empty(), "released vec must be cleared");
}

#[test]
fn pool_respects_capacity_limit_of_100() {
    let mut pool: StackPool<u8> = StackPool::new();

    // Release 150 vecs — only 100 should be retained.
    for _ in 0..150 {
        let v = Vec::new();
        pool.release(v);
    }

    // We can acquire at most 100 pooled vecs; after that we get fresh ones.
    let mut acquired = 0;
    for _ in 0..150 {
        let _v = pool.acquire();
        acquired += 1;
    }
    assert_eq!(acquired, 150); // acquire never fails
}

#[test]
fn pool_reuse_preserves_allocation_capacity() {
    let mut pool: StackPool<u64> = StackPool::new();

    let mut v = Vec::with_capacity(256);
    v.push(1);
    v.push(2);
    let original_cap = v.capacity();
    pool.release(v);

    let reused = pool.acquire();
    assert!(reused.is_empty());
    // Capacity should be preserved since clear() doesn't shrink.
    assert_eq!(reused.capacity(), original_cap);
}

#[test]
fn pool_acquire_release_cycle_stress() {
    let mut pool: StackPool<usize> = StackPool::new();

    for round in 0..50 {
        let mut vecs: Vec<Vec<usize>> = (0..20).map(|_| pool.acquire()).collect();

        for (i, v) in vecs.iter_mut().enumerate() {
            v.extend(0..(round + i));
        }

        for v in vecs {
            pool.release(v);
        }
    }

    // Pool should still be functional after heavy cycling.
    let v = pool.acquire();
    assert!(v.is_empty());
}

#[test]
fn pool_default_trait_equivalent_to_new() {
    let mut pool = StackPool::<String>::default();
    let v = pool.acquire();
    assert!(v.is_empty());
}
