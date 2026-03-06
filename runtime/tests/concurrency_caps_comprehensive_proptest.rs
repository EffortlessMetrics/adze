//! Comprehensive property-based and unit tests for the concurrency caps module.
//!
//! Exercises `init_concurrency_caps`, `bounded_parallel_map`,
//! `ParallelPartitionPlan`, `ConcurrencyCaps`, `normalized_concurrency`,
//! `parse_positive_usize_or_default`, `is_already_initialized_error`,
//! and related helpers via the `adze::concurrency_caps` re-export façade.

#![cfg(not(miri))]

use adze::concurrency_caps::{
    ConcurrencyCaps, DEFAULT_RAYON_NUM_THREADS, DEFAULT_TOKIO_WORKER_THREADS,
    ParallelPartitionPlan, RAYON_NUM_THREADS_ENV, TOKIO_WORKER_THREADS_ENV, bounded_parallel_map,
    init_concurrency_caps, init_rayon_global_once, is_already_initialized_error,
    normalized_concurrency, parse_positive_usize_or_default,
};
use proptest::prelude::*;
use std::collections::HashSet;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Barrier};

// =========================================================================
// 1. init_concurrency_caps initialization
// =========================================================================

#[test]
fn init_concurrency_caps_is_idempotent() {
    init_concurrency_caps();
    init_concurrency_caps();
    init_concurrency_caps();
}

#[test]
fn init_rayon_global_once_is_idempotent() {
    assert!(init_rayon_global_once(2).is_ok());
    assert!(init_rayon_global_once(16).is_ok());
}

#[test]
fn init_rayon_global_once_with_zero_normalizes() {
    assert!(init_rayon_global_once(0).is_ok());
}

#[test]
fn init_rayon_global_once_result_is_stable() {
    let first = init_rayon_global_once(1);
    let second = init_rayon_global_once(64);
    assert_eq!(first, second);
}

#[test]
fn init_rayon_global_once_with_large_count_succeeds() {
    assert!(init_rayon_global_once(1024).is_ok());
}

// =========================================================================
// 2. bounded_parallel_map with various inputs
// =========================================================================

#[test]
fn bounded_parallel_map_empty_vec() {
    let result: Vec<i32> = bounded_parallel_map(vec![], 4, |x: i32| x + 1);
    assert!(result.is_empty());
}

#[test]
fn bounded_parallel_map_single_item() {
    let result = bounded_parallel_map(vec![42], 4, |x| x * 2);
    assert_eq!(result, vec![84]);
}

#[test]
fn bounded_parallel_map_preserves_length() {
    let input: Vec<i32> = (0..100).collect();
    let result = bounded_parallel_map(input.clone(), 4, |x| x * 2);
    assert_eq!(result.len(), input.len());
}

#[test]
fn bounded_parallel_map_preserves_values() {
    let input: Vec<i32> = (0..50).collect();
    let mut result = bounded_parallel_map(input, 4, |x| x * 3);
    result.sort_unstable();
    let expected: Vec<i32> = (0..50).map(|x| x * 3).collect();
    assert_eq!(result, expected);
}

#[test]
fn bounded_parallel_map_zero_concurrency() {
    let mut result = bounded_parallel_map(vec![1, 2, 3], 0, |x| x * 10);
    result.sort_unstable();
    assert_eq!(result, vec![10, 20, 30]);
}

#[test]
fn bounded_parallel_map_concurrency_one() {
    let mut result = bounded_parallel_map(vec![5, 3, 1, 4, 2], 1, |x| x * x);
    result.sort_unstable();
    assert_eq!(result, vec![1, 4, 9, 16, 25]);
}

#[test]
fn bounded_parallel_map_concurrency_exceeds_items() {
    let mut result = bounded_parallel_map(vec![1, 2, 3], 100, |x| x + 100);
    result.sort_unstable();
    assert_eq!(result, vec![101, 102, 103]);
}

#[test]
fn bounded_parallel_map_large_batch() {
    let input: Vec<i32> = (0..10_000).collect();
    let result = bounded_parallel_map(input, 8, |x| x + 1);
    assert_eq!(result.len(), 10_000);
    let mut sorted = result;
    sorted.sort_unstable();
    let expected: Vec<i32> = (1..=10_000).collect();
    assert_eq!(sorted, expected);
}

#[test]
fn bounded_parallel_map_string_transform() {
    let input = vec!["hello", "world", "foo"];
    let mut result = bounded_parallel_map(input, 2, |s| s.to_uppercase());
    result.sort();
    assert_eq!(result, vec!["FOO", "HELLO", "WORLD"]);
}

#[test]
fn bounded_parallel_map_type_change() {
    let input: Vec<i32> = vec![1, 2, 3, 4, 5];
    let mut result = bounded_parallel_map(input, 2, |x| format!("n{x}"));
    result.sort();
    assert_eq!(result, vec!["n1", "n2", "n3", "n4", "n5"]);
}

#[test]
fn bounded_parallel_map_shared_atomic_counter() {
    let counter = Arc::new(AtomicUsize::new(0));
    let input: Vec<usize> = (0..100).collect();
    let c = Arc::clone(&counter);
    let result = bounded_parallel_map(input, 4, move |x| {
        c.fetch_add(1, Ordering::Relaxed);
        x
    });
    assert_eq!(result.len(), 100);
    assert_eq!(counter.load(Ordering::Relaxed), 100);
}

#[test]
fn bounded_parallel_map_all_same_value() {
    let input = vec![7; 50];
    let result = bounded_parallel_map(input, 4, |x| x * 2);
    assert!(result.iter().all(|&v| v == 14));
    assert_eq!(result.len(), 50);
}

#[test]
fn bounded_parallel_map_identity() {
    let input: Vec<i32> = (0..20).collect();
    let mut result = bounded_parallel_map(input.clone(), 4, |x| x);
    result.sort_unstable();
    assert_eq!(result, input);
}

// =========================================================================
// 3. Thread pool configuration (ConcurrencyCaps)
// =========================================================================

#[test]
fn caps_default_values() {
    let caps = ConcurrencyCaps::default();
    assert_eq!(caps.rayon_threads, DEFAULT_RAYON_NUM_THREADS);
    assert_eq!(caps.tokio_worker_threads, DEFAULT_TOKIO_WORKER_THREADS);
}

#[test]
fn caps_default_constants_are_positive() {
    #[allow(clippy::assertions_on_constants)]
    {
        assert!(DEFAULT_RAYON_NUM_THREADS >= 1);
        assert!(DEFAULT_TOKIO_WORKER_THREADS >= 1);
    }
}

#[test]
fn caps_default_constants_specific_values() {
    assert_eq!(DEFAULT_RAYON_NUM_THREADS, 4);
    assert_eq!(DEFAULT_TOKIO_WORKER_THREADS, 2);
}

#[test]
fn caps_from_lookup_returns_defaults_when_none() {
    let caps = ConcurrencyCaps::from_lookup(|_| None);
    assert_eq!(caps, ConcurrencyCaps::default());
}

#[test]
fn caps_from_lookup_parses_valid_values() {
    let caps = ConcurrencyCaps::from_lookup(|name| match name {
        "RAYON_NUM_THREADS" => Some("16".to_string()),
        "TOKIO_WORKER_THREADS" => Some("8".to_string()),
        _ => None,
    });
    assert_eq!(caps.rayon_threads, 16);
    assert_eq!(caps.tokio_worker_threads, 8);
}

#[test]
fn caps_from_lookup_falls_back_on_zero() {
    let caps = ConcurrencyCaps::from_lookup(|_| Some("0".to_string()));
    assert_eq!(caps, ConcurrencyCaps::default());
}

#[test]
fn caps_from_lookup_falls_back_on_invalid() {
    let caps = ConcurrencyCaps::from_lookup(|_| Some("not_a_number".to_string()));
    assert_eq!(caps, ConcurrencyCaps::default());
}

#[test]
fn caps_from_lookup_partial_override() {
    let caps = ConcurrencyCaps::from_lookup(|name| match name {
        "RAYON_NUM_THREADS" => Some("32".to_string()),
        _ => None,
    });
    assert_eq!(caps.rayon_threads, 32);
    assert_eq!(caps.tokio_worker_threads, DEFAULT_TOKIO_WORKER_THREADS);
}

#[test]
fn caps_clone_and_eq() {
    let caps = ConcurrencyCaps::from_lookup(|_| Some("3".to_string()));
    let cloned = caps;
    assert_eq!(caps, cloned);
}

#[test]
fn caps_debug_format_contains_fields() {
    let caps = ConcurrencyCaps::default();
    let dbg = format!("{caps:?}");
    assert!(dbg.contains("rayon_threads"));
    assert!(dbg.contains("tokio_worker_threads"));
}

// =========================================================================
// 4. Environment variable handling
// =========================================================================

#[test]
fn env_var_constants_match_expected_names() {
    assert_eq!(RAYON_NUM_THREADS_ENV, "RAYON_NUM_THREADS");
    assert_eq!(TOKIO_WORKER_THREADS_ENV, "TOKIO_WORKER_THREADS");
}

#[test]
fn parse_positive_usize_or_default_none_returns_default() {
    assert_eq!(parse_positive_usize_or_default(None, 7), 7);
}

#[test]
fn parse_positive_usize_or_default_zero_returns_default() {
    assert_eq!(parse_positive_usize_or_default(Some("0"), 5), 5);
}

#[test]
fn parse_positive_usize_or_default_valid_value() {
    assert_eq!(parse_positive_usize_or_default(Some("42"), 1), 42);
}

#[test]
fn parse_positive_usize_or_default_empty_string() {
    assert_eq!(parse_positive_usize_or_default(Some(""), 9), 9);
}

#[test]
fn parse_positive_usize_or_default_invalid_string() {
    assert_eq!(parse_positive_usize_or_default(Some("nope"), 3), 3);
}

#[test]
fn parse_positive_usize_or_default_trimmed_whitespace() {
    assert_eq!(parse_positive_usize_or_default(Some("  10  "), 1), 10);
}

#[test]
fn parse_positive_usize_or_default_negative_string() {
    assert_eq!(parse_positive_usize_or_default(Some("-1"), 5), 5);
}

#[test]
fn parse_positive_usize_or_default_one() {
    assert_eq!(parse_positive_usize_or_default(Some("1"), 99), 1);
}

#[test]
fn parse_positive_usize_or_default_large_value() {
    assert_eq!(
        parse_positive_usize_or_default(Some("1000000"), 1),
        1_000_000
    );
}

// =========================================================================
// 5. Edge cases — normalized_concurrency
// =========================================================================

#[test]
fn normalized_concurrency_zero_becomes_one() {
    assert_eq!(normalized_concurrency(0), 1);
}

#[test]
fn normalized_concurrency_one_stays_one() {
    assert_eq!(normalized_concurrency(1), 1);
}

#[test]
fn normalized_concurrency_preserves_positive() {
    assert_eq!(normalized_concurrency(4), 4);
    assert_eq!(normalized_concurrency(128), 128);
}

#[test]
fn normalized_concurrency_large_value() {
    assert_eq!(normalized_concurrency(usize::MAX), usize::MAX);
}

// =========================================================================
// 6. is_already_initialized_error classifier
// =========================================================================

#[test]
fn is_already_initialized_detects_canonical_message() {
    assert!(is_already_initialized_error(
        "The global thread pool has already been initialized"
    ));
}

#[test]
fn is_already_initialized_case_insensitive() {
    assert!(is_already_initialized_error(
        "The GlObAl thread pool has AlReAdY been initialized"
    ));
}

#[test]
fn is_already_initialized_requires_both_tokens() {
    assert!(!is_already_initialized_error("thread pool already done"));
    assert!(!is_already_initialized_error("global pool not set"));
}

#[test]
fn is_already_initialized_empty_string() {
    assert!(!is_already_initialized_error(""));
}

#[test]
fn is_already_initialized_unrelated_message() {
    assert!(!is_already_initialized_error("something went wrong"));
}

#[test]
fn is_already_initialized_reversed_order() {
    assert!(is_already_initialized_error(
        "already set on the global pool"
    ));
}

#[test]
fn is_already_initialized_adjacent_tokens() {
    assert!(is_already_initialized_error("globalalready"));
}

// =========================================================================
// 7. ParallelPartitionPlan
// =========================================================================

#[test]
fn partition_plan_empty_input() {
    let plan = ParallelPartitionPlan::for_item_count(0, 4);
    assert!(plan.chunk_size >= 1);
    assert!(plan.use_direct_parallel_iter);
}

#[test]
fn partition_plan_zero_concurrency_normalizes() {
    let plan = ParallelPartitionPlan::for_item_count(10, 0);
    assert_eq!(plan.concurrency, 1);
    assert!(plan.chunk_size >= 1);
}

#[test]
fn partition_plan_single_item() {
    let plan = ParallelPartitionPlan::for_item_count(1, 4);
    assert!(plan.use_direct_parallel_iter);
    assert!(plan.chunk_size >= 1);
}

#[test]
fn partition_plan_large_work_uses_chunking() {
    let plan = ParallelPartitionPlan::for_item_count(257, 4);
    assert_eq!(plan.concurrency, 4);
    assert_eq!(plan.chunk_size, 65); // ceil(257/4) = 65
    assert!(!plan.use_direct_parallel_iter);
}

#[test]
fn partition_plan_items_equal_threshold() {
    // threshold = concurrency * 2 = 8
    let plan = ParallelPartitionPlan::for_item_count(8, 4);
    assert!(plan.use_direct_parallel_iter);
}

#[test]
fn partition_plan_items_just_above_threshold() {
    // threshold = 4 * 2 = 8; 9 items should use chunking
    let plan = ParallelPartitionPlan::for_item_count(9, 4);
    assert!(!plan.use_direct_parallel_iter);
}

#[test]
fn partition_plan_concurrency_one() {
    let plan = ParallelPartitionPlan::for_item_count(100, 1);
    assert_eq!(plan.concurrency, 1);
    // threshold = 1*2 = 2; 100 > 2, so chunking
    assert!(!plan.use_direct_parallel_iter);
    assert_eq!(plan.chunk_size, 100);
}

#[test]
fn partition_plan_debug_and_clone() {
    let plan = ParallelPartitionPlan::for_item_count(50, 4);
    let cloned = plan;
    assert_eq!(plan, cloned);
    let dbg = format!("{plan:?}");
    assert!(dbg.contains("concurrency"));
}

// =========================================================================
// 8. Proptest — parse_positive_usize_or_default
// =========================================================================

proptest! {
    #[test]
    fn proptest_parse_none_always_returns_default(default in 1..1000usize) {
        let result = parse_positive_usize_or_default(None, default);
        prop_assert_eq!(result, default);
    }

    #[test]
    fn proptest_parse_valid_positive_returns_value(val in 1..100_000usize, default in 1..100usize) {
        let s = val.to_string();
        let result = parse_positive_usize_or_default(Some(&s), default);
        prop_assert_eq!(result, val);
    }

    #[test]
    fn proptest_parse_zero_returns_default(default in 1..1000usize) {
        let result = parse_positive_usize_or_default(Some("0"), default);
        prop_assert_eq!(result, default);
    }
}

// =========================================================================
// 9. Proptest — normalized_concurrency
// =========================================================================

proptest! {
    #[test]
    fn proptest_normalized_concurrency_never_zero(val in 0..10_000usize) {
        let result = normalized_concurrency(val);
        prop_assert!(result >= 1);
        if val > 0 {
            prop_assert_eq!(result, val);
        }
    }
}

// =========================================================================
// 10. Proptest — bounded_parallel_map
// =========================================================================

proptest! {
    #[test]
    fn proptest_bounded_parallel_map_preserves_length(
        len in 0..500usize,
        concurrency in 0..32usize,
    ) {
        let input: Vec<i32> = (0..len as i32).collect();
        let result = bounded_parallel_map(input, concurrency, |x| x + 1);
        prop_assert_eq!(result.len(), len);
    }

    #[test]
    fn proptest_bounded_parallel_map_values_correct(
        len in 0..200usize,
        concurrency in 1..16usize,
    ) {
        let input: Vec<i32> = (0..len as i32).collect();
        let mut result = bounded_parallel_map(input, concurrency, |x| x * 2);
        result.sort_unstable();
        let expected: Vec<i32> = (0..len as i32).map(|x| x * 2).collect();
        prop_assert_eq!(result, expected);
    }

    #[test]
    fn proptest_bounded_parallel_map_no_duplicates(
        len in 1..300usize,
        concurrency in 1..16usize,
    ) {
        let input: Vec<usize> = (0..len).collect();
        let result = bounded_parallel_map(input, concurrency, |x| x);
        let unique: HashSet<usize> = result.iter().copied().collect();
        prop_assert_eq!(unique.len(), len);
    }
}

// =========================================================================
// 11. Proptest — ParallelPartitionPlan
// =========================================================================

proptest! {
    #[test]
    fn proptest_partition_plan_chunk_size_always_positive(
        items in 0..10_000usize,
        concurrency in 0..64usize,
    ) {
        let plan = ParallelPartitionPlan::for_item_count(items, concurrency);
        prop_assert!(plan.chunk_size >= 1);
        prop_assert!(plan.concurrency >= 1);
    }

    #[test]
    fn proptest_partition_plan_concurrency_normalized(concurrency in 0..100usize) {
        let plan = ParallelPartitionPlan::for_item_count(50, concurrency);
        prop_assert!(plan.concurrency >= 1);
        if concurrency > 0 {
            prop_assert_eq!(plan.concurrency, concurrency);
        }
    }

    #[test]
    fn proptest_partition_plan_direct_iter_for_small_workloads(
        concurrency in 1..32usize,
    ) {
        // Items <= concurrency * 2 should use direct parallel iteration
        let threshold = concurrency.saturating_mul(2);
        let plan = ParallelPartitionPlan::for_item_count(threshold, concurrency);
        prop_assert!(plan.use_direct_parallel_iter);
    }
}

// =========================================================================
// 12. Proptest — ConcurrencyCaps::from_lookup
// =========================================================================

proptest! {
    #[test]
    fn proptest_caps_from_lookup_valid_values(
        rayon in 1..256usize,
        tokio in 1..256usize,
    ) {
        let caps = ConcurrencyCaps::from_lookup(|name| match name {
            "RAYON_NUM_THREADS" => Some(rayon.to_string()),
            "TOKIO_WORKER_THREADS" => Some(tokio.to_string()),
            _ => None,
        });
        prop_assert_eq!(caps.rayon_threads, rayon);
        prop_assert_eq!(caps.tokio_worker_threads, tokio);
    }

    #[test]
    fn proptest_caps_from_lookup_zero_falls_back(
        default_rayon in Just(DEFAULT_RAYON_NUM_THREADS),
        default_tokio in Just(DEFAULT_TOKIO_WORKER_THREADS),
    ) {
        let caps = ConcurrencyCaps::from_lookup(|_| Some("0".to_string()));
        prop_assert_eq!(caps.rayon_threads, default_rayon);
        prop_assert_eq!(caps.tokio_worker_threads, default_tokio);
    }
}

// =========================================================================
// 13. Concurrent usage of bounded_parallel_map
// =========================================================================

#[test]
fn bounded_parallel_map_concurrent_invocations() {
    let barrier = Arc::new(Barrier::new(4));
    let handles: Vec<_> = (0..4)
        .map(|t| {
            let b = Arc::clone(&barrier);
            std::thread::spawn(move || {
                b.wait();
                let input: Vec<i32> = (0..100).map(|i| i + t * 100).collect();
                let mut result = bounded_parallel_map(input, 4, |x| x * 2);
                result.sort_unstable();
                result
            })
        })
        .collect();

    for (t, handle) in handles.into_iter().enumerate() {
        let result = handle.join().expect("thread panicked");
        assert_eq!(result.len(), 100);
        let expected: Vec<i32> = (0..100).map(|i| (i + t as i32 * 100) * 2).collect();
        assert_eq!(result, expected);
    }
}

#[test]
fn init_concurrency_caps_from_multiple_threads() {
    let barrier = Arc::new(Barrier::new(4));
    let handles: Vec<_> = (0..4)
        .map(|_| {
            let b = Arc::clone(&barrier);
            std::thread::spawn(move || {
                b.wait();
                init_concurrency_caps();
            })
        })
        .collect();

    for handle in handles {
        handle.join().expect("thread panicked");
    }
}
