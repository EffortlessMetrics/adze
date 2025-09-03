// Concurrency caps for rust-sitter runtime
// Sets up bounded thread pools for tokio, rayon, and other parallel operations

use once_cell::sync::Lazy;
use std::env;

/// Initialize concurrency caps from environment variables or defaults
/// This should be called once at program startup or in test setup
pub fn init_concurrency_caps() {
    // Initialize rayon global thread pool with caps
    Lazy::force(&RAYON_INIT);

    // Print current settings for debugging
    let rayon_threads = env::var("RAYON_NUM_THREADS").unwrap_or_else(|_| "4".to_string());
    let tokio_workers = env::var("TOKIO_WORKER_THREADS").unwrap_or_else(|_| "2".to_string());

    eprintln!(
        "Concurrency caps initialized: RAYON_NUM_THREADS={}, TOKIO_WORKER_THREADS={}",
        rayon_threads, tokio_workers
    );
}

/// Initialize rayon global thread pool once
static RAYON_INIT: Lazy<()> = Lazy::new(|| {
    let num_threads: usize = env::var("RAYON_NUM_THREADS")
        .unwrap_or_else(|_| "4".to_string())
        .parse()
        .unwrap_or(4);

    rayon::ThreadPoolBuilder::new()
        .num_threads(num_threads)
        .build_global()
        .expect("Failed to initialize rayon global thread pool");
});

/// Helper for bounded parallel iteration
/// Use this instead of rayon's unbounded parallel iteration
pub fn bounded_parallel_map<T, R, F>(items: Vec<T>, concurrency: usize, f: F) -> Vec<R>
where
    T: Send,
    R: Send,
    F: Fn(T) -> R + Send + Sync,
{
    use rayon::prelude::*;

    // If items are few or concurrency is high, use regular parallel iteration
    if items.len() <= concurrency * 2 {
        return items.into_par_iter().map(f).collect();
    }

    // For large collections, process in bounded chunks
    let chunk_size = items.len().div_ceil(concurrency);
    items
        .into_par_iter()
        .chunks(chunk_size)
        .flat_map(|chunk| chunk.into_iter().map(&f).collect::<Vec<_>>())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_concurrency_caps_init() {
        // This should not panic
        init_concurrency_caps();
    }

    #[test]
    fn test_bounded_parallel_map() {
        let items: Vec<i32> = (0..100).collect();
        let results = bounded_parallel_map(items, 4, |x| x * 2);

        assert_eq!(results.len(), 100);
        // Results might not be in order due to parallel processing
        let mut sorted_results = results;
        sorted_results.sort();

        let expected: Vec<i32> = (0..100).map(|x| x * 2).collect();
        assert_eq!(sorted_results, expected);
    }
}
