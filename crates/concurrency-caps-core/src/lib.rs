//! Compatibility façade for concurrency cap primitives and contracts.

#![forbid(unsafe_op_in_unsafe_fn)]
#![deny(missing_docs)]
#![cfg_attr(feature = "strict_api", deny(unreachable_pub))]
#![cfg_attr(not(feature = "strict_api"), warn(unreachable_pub))]
#![cfg_attr(feature = "strict_docs", deny(missing_docs))]
#![cfg_attr(not(feature = "strict_docs"), allow(missing_docs))]

pub use adze_concurrency_env_core::{
    ConcurrencyCaps, DEFAULT_RAYON_NUM_THREADS, DEFAULT_TOKIO_WORKER_THREADS,
    RAYON_NUM_THREADS_ENV, TOKIO_WORKER_THREADS_ENV, current_caps, parse_positive_usize_or_default,
};
pub use adze_concurrency_init_core::{
    init_concurrency_caps, init_rayon_global_once, is_already_initialized_error,
};
pub use adze_concurrency_map_core::{
    ParallelPartitionPlan, bounded_parallel_map, normalized_concurrency,
};
