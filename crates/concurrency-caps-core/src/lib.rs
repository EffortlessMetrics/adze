//! Compatibility façade for concurrency cap primitives and contracts.
//!
//! # Examples
//!
//! ```
//! use adze_concurrency_caps_core::{ConcurrencyCaps, parse_positive_usize_or_default};
//!
//! // Build caps from a custom lookup (no env mutation)
//! let caps = ConcurrencyCaps::from_lookup(|_| None);
//! assert_eq!(caps.rayon_threads, 4);   // default
//! assert_eq!(caps.tokio_worker_threads, 2); // default
//!
//! // Parse helper returns default for invalid input
//! assert_eq!(parse_positive_usize_or_default(Some("0"), 8), 8);
//! assert_eq!(parse_positive_usize_or_default(Some("16"), 8), 16);
//! ```

#![forbid(unsafe_op_in_unsafe_fn)]
#![deny(missing_docs)]
#![cfg_attr(feature = "strict_api", deny(unreachable_pub))]
#![cfg_attr(not(feature = "strict_api"), warn(unreachable_pub))]
#![cfg_attr(feature = "strict_docs", deny(missing_docs))]
#![cfg_attr(not(feature = "strict_docs"), allow(missing_docs))]

/// Environment-based concurrency cap configuration and defaults.
pub use adze_concurrency_env_core::{
    ConcurrencyCaps, DEFAULT_RAYON_NUM_THREADS, DEFAULT_TOKIO_WORKER_THREADS,
    RAYON_NUM_THREADS_ENV, TOKIO_WORKER_THREADS_ENV, current_caps, parse_positive_usize_or_default,
};
/// One-time concurrency initialization helpers (rayon global pool, caps bootstrap).
pub use adze_concurrency_init_core::{
    init_concurrency_caps, init_rayon_global_once, is_already_initialized_error,
};
/// Bounded parallel map and partition planning utilities.
pub use adze_concurrency_map_core::{
    ParallelPartitionPlan, bounded_parallel_map, normalized_concurrency,
};
