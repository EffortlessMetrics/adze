#![allow(clippy::needless_range_loop)]

use adze::concurrency_caps::{
    ConcurrencyCaps, DEFAULT_RAYON_NUM_THREADS, DEFAULT_TOKIO_WORKER_THREADS,
    ParallelPartitionPlan, RAYON_NUM_THREADS_ENV, TOKIO_WORKER_THREADS_ENV, bounded_parallel_map,
    current_caps, init_concurrency_caps, init_rayon_global_once, is_already_initialized_error,
    normalized_concurrency, parse_positive_usize_or_default,
};

// ── ConcurrencyCaps construction ─────────────────────────────────────

#[test]
fn caps_default_matches_documented_constants() {
    let caps = ConcurrencyCaps::default();
    assert_eq!(caps.rayon_threads, DEFAULT_RAYON_NUM_THREADS);
    assert_eq!(caps.tokio_worker_threads, DEFAULT_TOKIO_WORKER_THREADS);
}

#[test]
fn caps_from_lookup_returns_defaults_when_all_none() {
    let caps = ConcurrencyCaps::from_lookup(|_| None);
    assert_eq!(caps, ConcurrencyCaps::default());
}

#[test]
fn caps_from_lookup_parses_custom_values() {
    let caps = ConcurrencyCaps::from_lookup(|name| match name {
        "RAYON_NUM_THREADS" => Some("12".into()),
        "TOKIO_WORKER_THREADS" => Some("6".into()),
        _ => None,
    });
    assert_eq!(caps.rayon_threads, 12);
    assert_eq!(caps.tokio_worker_threads, 6);
}

#[test]
fn caps_from_lookup_falls_back_on_zero_values() {
    let caps = ConcurrencyCaps::from_lookup(|_| Some("0".into()));
    assert_eq!(caps, ConcurrencyCaps::default());
}

#[test]
fn caps_from_lookup_falls_back_on_non_numeric_input() {
    let caps = ConcurrencyCaps::from_lookup(|_| Some("banana".into()));
    assert_eq!(caps, ConcurrencyCaps::default());
}

#[test]
fn caps_from_lookup_handles_mixed_valid_and_invalid() {
    let caps = ConcurrencyCaps::from_lookup(|name| match name {
        "RAYON_NUM_THREADS" => Some("16".into()),
        "TOKIO_WORKER_THREADS" => Some("garbage".into()),
        _ => None,
    });
    assert_eq!(caps.rayon_threads, 16);
    assert_eq!(caps.tokio_worker_threads, DEFAULT_TOKIO_WORKER_THREADS);
}

#[test]
fn caps_clone_and_equality() {
    let caps = ConcurrencyCaps::from_lookup(|_| Some("3".into()));
    let cloned = caps;
    assert_eq!(caps, cloned);
}

#[test]
fn caps_debug_format_contains_field_names() {
    let caps = ConcurrencyCaps::default();
    let dbg = format!("{caps:?}");
    assert!(dbg.contains("rayon_threads"));
    assert!(dbg.contains("tokio_worker_threads"));
}

// ── Environment variable constants ───────────────────────────────────

#[test]
fn env_var_names_match_expected_strings() {
    assert_eq!(RAYON_NUM_THREADS_ENV, "RAYON_NUM_THREADS");
    assert_eq!(TOKIO_WORKER_THREADS_ENV, "TOKIO_WORKER_THREADS");
}

// ── current_caps ─────────────────────────────────────────────────────

#[test]
fn current_caps_returns_positive_values() {
    let caps = current_caps();
    assert!(caps.rayon_threads >= 1);
    assert!(caps.tokio_worker_threads >= 1);
}

// ── parse_positive_usize_or_default ──────────────────────────────────

#[test]
fn parse_returns_default_for_none() {
    assert_eq!(parse_positive_usize_or_default(None, 42), 42);
}

#[test]
fn parse_returns_default_for_zero() {
    assert_eq!(parse_positive_usize_or_default(Some("0"), 5), 5);
}

#[test]
fn parse_returns_default_for_empty_string() {
    assert_eq!(parse_positive_usize_or_default(Some(""), 9), 9);
}

#[test]
fn parse_returns_default_for_non_numeric() {
    assert_eq!(parse_positive_usize_or_default(Some("xyz"), 7), 7);
}

#[test]
fn parse_accepts_valid_positive_value() {
    assert_eq!(parse_positive_usize_or_default(Some("64"), 1), 64);
}

#[test]
fn parse_trims_whitespace() {
    assert_eq!(parse_positive_usize_or_default(Some("  8  "), 1), 8);
}

#[test]
fn parse_returns_default_for_negative_looking_input() {
    assert_eq!(parse_positive_usize_or_default(Some("-1"), 10), 10);
}

// ── normalized_concurrency ───────────────────────────────────────────

#[test]
fn normalized_concurrency_clamps_zero_to_one() {
    assert_eq!(normalized_concurrency(0), 1);
}

#[test]
fn normalized_concurrency_preserves_one() {
    assert_eq!(normalized_concurrency(1), 1);
}

#[test]
fn normalized_concurrency_preserves_large_value() {
    assert_eq!(normalized_concurrency(1024), 1024);
}

// ── init_concurrency_caps ────────────────────────────────────────────

#[test]
fn init_concurrency_caps_is_idempotent() {
    init_concurrency_caps();
    init_concurrency_caps();
}

// ── init_rayon_global_once ───────────────────────────────────────────

#[test]
fn init_rayon_global_once_succeeds() {
    assert!(init_rayon_global_once(2).is_ok());
}

#[test]
fn init_rayon_global_once_is_idempotent_across_thread_counts() {
    let first = init_rayon_global_once(1);
    let second = init_rayon_global_once(64);
    assert_eq!(first, second);
}

#[test]
fn init_rayon_global_once_normalizes_zero_threads() {
    assert!(init_rayon_global_once(0).is_ok());
}

// ── is_already_initialized_error ─────────────────────────────────────

#[test]
fn already_initialized_detects_canonical_message() {
    assert!(is_already_initialized_error(
        "The global thread pool has already been initialized"
    ));
}

#[test]
fn already_initialized_is_case_insensitive() {
    assert!(is_already_initialized_error(
        "The GLOBAL thread pool has ALREADY been initialized"
    ));
}

#[test]
fn already_initialized_rejects_missing_global_keyword() {
    assert!(!is_already_initialized_error(
        "thread pool already initialized"
    ));
}

#[test]
fn already_initialized_rejects_missing_already_keyword() {
    assert!(!is_already_initialized_error("global thread pool failed"));
}

#[test]
fn already_initialized_rejects_empty_string() {
    assert!(!is_already_initialized_error(""));
}

// ── ParallelPartitionPlan ────────────────────────────────────────────

#[test]
fn partition_plan_empty_items_uses_direct_parallel() {
    let plan = ParallelPartitionPlan::for_item_count(0, 4);
    assert!(plan.use_direct_parallel_iter);
    assert!(plan.chunk_size >= 1);
}

#[test]
fn partition_plan_small_workload_uses_direct_parallel() {
    // With concurrency=4, threshold = 4*2 = 8, so 8 items → direct
    let plan = ParallelPartitionPlan::for_item_count(8, 4);
    assert!(plan.use_direct_parallel_iter);
}

#[test]
fn partition_plan_large_workload_uses_chunking() {
    let plan = ParallelPartitionPlan::for_item_count(100, 4);
    assert!(!plan.use_direct_parallel_iter);
    assert_eq!(plan.concurrency, 4);
    assert_eq!(plan.chunk_size, 25);
}

#[test]
fn partition_plan_normalizes_zero_concurrency() {
    let plan = ParallelPartitionPlan::for_item_count(10, 0);
    assert_eq!(plan.concurrency, 1);
}

#[test]
fn partition_plan_chunk_size_rounds_up() {
    let plan = ParallelPartitionPlan::for_item_count(7, 3);
    // 7 / 3 = 2.33 → rounds up to 3
    assert_eq!(plan.chunk_size, 3);
}

// ── bounded_parallel_map ─────────────────────────────────────────────

#[test]
fn bounded_map_empty_input_returns_empty() {
    let result: Vec<i32> = bounded_parallel_map(vec![], 4, |x: i32| x + 1);
    assert!(result.is_empty());
}

#[test]
fn bounded_map_single_element() {
    let result = bounded_parallel_map(vec![42], 4, |x| x * 2);
    assert_eq!(result, vec![84]);
}

#[test]
fn bounded_map_preserves_output_length() {
    let input: Vec<i32> = (0..200).collect();
    let result = bounded_parallel_map(input.clone(), 4, |x| x + 1);
    assert_eq!(result.len(), input.len());
}

#[test]
fn bounded_map_produces_correct_values() {
    let input: Vec<i32> = (0..50).collect();
    let mut result = bounded_parallel_map(input.clone(), 4, |x| x * 3);
    result.sort_unstable();
    let expected: Vec<i32> = input.into_iter().map(|x| x * 3).collect();
    assert_eq!(result, expected);
}

#[test]
fn bounded_map_with_concurrency_one() {
    let mut result = bounded_parallel_map(vec![5, 3, 1, 4, 2], 1, |x| x * x);
    result.sort_unstable();
    assert_eq!(result, vec![1, 4, 9, 16, 25]);
}

#[test]
fn bounded_map_with_zero_concurrency_still_works() {
    let mut result = bounded_parallel_map(vec![10, 20, 30], 0, |x| x / 10);
    result.sort_unstable();
    assert_eq!(result, vec![1, 2, 3]);
}

#[test]
fn bounded_map_concurrency_exceeds_item_count() {
    let mut result = bounded_parallel_map(vec![1, 2], 1000, |x| x + 100);
    result.sort_unstable();
    assert_eq!(result, vec![101, 102]);
}

#[test]
fn bounded_map_with_string_transformation() {
    let input: Vec<&str> = vec!["hello", "world"];
    let mut result = bounded_parallel_map(input, 2, |s: &str| s.to_uppercase());
    result.sort();
    assert_eq!(result, vec!["HELLO", "WORLD"]);
}

#[test]
fn bounded_map_closure_captures_environment() {
    let factor = 7;
    let input: Vec<i32> = (1..=5).collect();
    let mut result = bounded_parallel_map(input, 2, |x| x * factor);
    result.sort_unstable();
    assert_eq!(result, vec![7, 14, 21, 28, 35]);
}
