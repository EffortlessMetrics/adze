//! Core utilities for runtime concurrency caps and bounded parallel work.

#![forbid(unsafe_op_in_unsafe_fn)]
#![deny(missing_docs)]
#![cfg_attr(feature = "strict_api", deny(unreachable_pub))]
#![cfg_attr(not(feature = "strict_api"), warn(unreachable_pub))]
#![cfg_attr(feature = "strict_docs", deny(missing_docs))]
#![cfg_attr(not(feature = "strict_docs"), allow(missing_docs))]

use std::env;
use std::sync::OnceLock;

pub use adze_concurrency_plan_core::{ParallelPartitionPlan, normalized_concurrency};

const RAYON_NUM_THREADS_ENV: &str = "RAYON_NUM_THREADS";
const TOKIO_WORKER_THREADS_ENV: &str = "TOKIO_WORKER_THREADS";

/// Default thread count used for Rayon when `RAYON_NUM_THREADS` is unset/invalid.
pub const DEFAULT_RAYON_NUM_THREADS: usize = 4;

/// Default worker count used for Tokio when `TOKIO_WORKER_THREADS` is unset/invalid.
pub const DEFAULT_TOKIO_WORKER_THREADS: usize = 2;

/// Snapshot of active concurrency cap values.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConcurrencyCaps {
    /// Rayon global thread-pool thread count.
    pub rayon_threads: usize,
    /// Tokio worker thread count.
    pub tokio_worker_threads: usize,
}

impl ConcurrencyCaps {
    /// Read concurrency caps from process environment with stable defaults.
    #[must_use]
    pub fn from_env() -> Self {
        Self {
            rayon_threads: parse_env_usize(RAYON_NUM_THREADS_ENV, DEFAULT_RAYON_NUM_THREADS),
            tokio_worker_threads: parse_env_usize(
                TOKIO_WORKER_THREADS_ENV,
                DEFAULT_TOKIO_WORKER_THREADS,
            ),
        }
    }
}

impl Default for ConcurrencyCaps {
    fn default() -> Self {
        Self {
            rayon_threads: DEFAULT_RAYON_NUM_THREADS,
            tokio_worker_threads: DEFAULT_TOKIO_WORKER_THREADS,
        }
    }
}

/// Return the current caps resolved from environment values.
#[must_use]
pub fn current_caps() -> ConcurrencyCaps {
    ConcurrencyCaps::from_env()
}

/// Initialize Rayon global thread-pool caps once for the process.
///
/// Calling this function multiple times is safe and idempotent.
pub fn init_concurrency_caps() {
    let caps = current_caps();

    let init_result = RAYON_INIT_RESULT.get_or_init(|| init_rayon_global(caps.rayon_threads));
    if let Err(message) = init_result {
        panic!("failed to initialize rayon global thread pool: {message}");
    }

    eprintln!(
        "Concurrency caps initialized: {RAYON_NUM_THREADS_ENV}={}, {TOKIO_WORKER_THREADS_ENV}={}",
        caps.rayon_threads, caps.tokio_worker_threads
    );
}

/// Run a bounded parallel map operation.
///
/// This keeps work partitioned by `concurrency`, while preserving all outputs.
pub fn bounded_parallel_map<T, R, F>(items: Vec<T>, concurrency: usize, f: F) -> Vec<R>
where
    T: Send,
    R: Send,
    F: Fn(T) -> R + Send + Sync,
{
    use rayon::prelude::*;

    let plan = ParallelPartitionPlan::for_item_count(items.len(), concurrency);

    if items.is_empty() {
        return Vec::new();
    }

    if plan.use_direct_parallel_iter {
        return items.into_par_iter().map(f).collect();
    }

    items
        .into_par_iter()
        .chunks(plan.chunk_size)
        .flat_map(|chunk| chunk.into_iter().map(&f).collect::<Vec<_>>())
        .collect()
}

static RAYON_INIT_RESULT: OnceLock<Result<(), String>> = OnceLock::new();

fn parse_env_usize(name: &str, default: usize) -> usize {
    env::var(name)
        .ok()
        .and_then(|value| value.trim().parse::<usize>().ok())
        .filter(|value| *value > 0)
        .unwrap_or(default)
}

fn init_rayon_global(num_threads: usize) -> Result<(), String> {
    match rayon::ThreadPoolBuilder::new()
        .num_threads(num_threads)
        .build_global()
    {
        Ok(()) => Ok(()),
        Err(error) => {
            let message = error.to_string();
            if is_already_initialized_error(&message) {
                Ok(())
            } else {
                Err(message)
            }
        }
    }
}

fn is_already_initialized_error(message: &str) -> bool {
    let message = message.to_ascii_lowercase();
    message.contains("global") && message.contains("already")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalized_concurrency_is_never_zero() {
        assert_eq!(normalized_concurrency(0), 1);
        assert_eq!(normalized_concurrency(1), 1);
        assert_eq!(normalized_concurrency(8), 8);
    }

    #[test]
    fn bounded_parallel_map_handles_zero_concurrency() {
        let mut result = bounded_parallel_map((0..64).collect::<Vec<_>>(), 0, |x| x * 2);
        result.sort_unstable();

        let expected: Vec<i32> = (0..64).map(|x| x * 2).collect();
        assert_eq!(result, expected);
    }

    #[test]
    fn init_is_idempotent() {
        init_concurrency_caps();
        init_concurrency_caps();
    }
}
