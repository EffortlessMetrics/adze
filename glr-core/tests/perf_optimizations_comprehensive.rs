//! Comprehensive tests for GLR performance optimization utilities.
//!
//! Covers: ParseTableCache, StackDeduplicator, StackPool, PerfStats.

use adze_glr_core::Action;
use adze_glr_core::perf_optimizations::{ParseTableCache, StackDeduplicator, StackPool};
use adze_ir::StateId;

// --- ParseTableCache tests ---

#[test]
fn cache_new_has_zero_stats() {
    let cache = ParseTableCache::new();
    let stats = cache.stats();
    assert_eq!(stats.cache_hits, 0);
    assert_eq!(stats.cache_misses, 0);
}

#[test]
fn cache_first_lookup_is_miss() {
    let mut cache = ParseTableCache::new();
    let _action = cache.get_or_compute(StateId(0), adze_ir::SymbolId(0), || Action::Error);
    assert_eq!(cache.stats().cache_misses, 1);
    assert_eq!(cache.stats().cache_hits, 0);
}

#[test]
fn cache_second_lookup_is_hit() {
    let mut cache = ParseTableCache::new();
    let state = StateId(0);
    let symbol = adze_ir::SymbolId(0);
    cache.get_or_compute(state, symbol, || Action::Error);
    cache.get_or_compute(state, symbol, || Action::Error);
    assert_eq!(cache.stats().cache_hits, 1);
    assert_eq!(cache.stats().cache_misses, 1);
}

#[test]
fn cache_different_keys_are_misses() {
    let mut cache = ParseTableCache::new();
    cache.get_or_compute(StateId(0), adze_ir::SymbolId(0), || Action::Error);
    cache.get_or_compute(StateId(1), adze_ir::SymbolId(0), || Action::Error);
    cache.get_or_compute(StateId(0), adze_ir::SymbolId(1), || Action::Error);
    assert_eq!(cache.stats().cache_misses, 3);
}

#[test]
fn cache_returns_computed_value() {
    let mut cache = ParseTableCache::new();
    let action = cache.get_or_compute(StateId(0), adze_ir::SymbolId(0), || {
        Action::Shift(StateId(42))
    });
    assert!(matches!(action, Action::Shift(StateId(42))));
}

#[test]
fn cache_returns_cached_value() {
    let mut cache = ParseTableCache::new();
    let state = StateId(0);
    let symbol = adze_ir::SymbolId(0);
    cache.get_or_compute(state, symbol, || Action::Shift(StateId(99)));
    // Second call with different closure should still return cached value
    let action = cache.get_or_compute(state, symbol, || Action::Error);
    assert!(matches!(action, Action::Shift(StateId(99))));
}

// --- StackDeduplicator tests ---

#[test]
fn deduplicator_new_has_zero_unique() {
    let dedup = StackDeduplicator::new();
    assert_eq!(dedup.unique_stacks(), 0);
}

#[test]
fn deduplicator_first_is_not_duplicate() {
    let mut dedup = StackDeduplicator::new();
    assert!(!dedup.is_duplicate(&[StateId(0), StateId(1)]));
    assert_eq!(dedup.unique_stacks(), 1);
}

#[test]
fn deduplicator_same_twice_is_duplicate() {
    let mut dedup = StackDeduplicator::new();
    dedup.is_duplicate(&[StateId(0), StateId(1)]);
    assert!(dedup.is_duplicate(&[StateId(0), StateId(1)]));
    assert_eq!(dedup.unique_stacks(), 1);
}

#[test]
fn deduplicator_different_not_duplicate() {
    let mut dedup = StackDeduplicator::new();
    dedup.is_duplicate(&[StateId(0)]);
    assert!(!dedup.is_duplicate(&[StateId(1)]));
    assert_eq!(dedup.unique_stacks(), 2);
}

#[test]
fn deduplicator_empty_stack() {
    let mut dedup = StackDeduplicator::new();
    assert!(!dedup.is_duplicate(&[]));
    assert!(dedup.is_duplicate(&[]));
}

#[test]
fn deduplicator_order_matters() {
    let mut dedup = StackDeduplicator::new();
    dedup.is_duplicate(&[StateId(0), StateId(1)]);
    assert!(!dedup.is_duplicate(&[StateId(1), StateId(0)]));
    assert_eq!(dedup.unique_stacks(), 2);
}

// --- StackPool tests ---

#[test]
fn pool_acquire_returns_empty_vec() {
    let mut pool: StackPool<i32> = StackPool::new();
    let v = pool.acquire();
    assert!(v.is_empty());
}

#[test]
fn pool_release_and_reacquire() {
    let mut pool: StackPool<i32> = StackPool::new();
    let mut v = pool.acquire();
    v.push(1);
    v.push(2);
    pool.release(v);
    let v2 = pool.acquire();
    // Released vec should be cleared
    assert!(v2.is_empty());
}

#[test]
fn pool_multiple_release_and_acquire() {
    let mut pool: StackPool<u32> = StackPool::new();
    for _ in 0..10 {
        let v = pool.acquire();
        pool.release(v);
    }
    // Should not panic
    let v = pool.acquire();
    assert!(v.is_empty());
}

#[test]
fn pool_with_different_types() {
    let mut pool: StackPool<String> = StackPool::new();
    let mut v = pool.acquire();
    v.push("hello".to_string());
    pool.release(v);
    let v2 = pool.acquire();
    assert!(v2.is_empty());
}
