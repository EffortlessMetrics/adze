//! Core bounded parallel map utilities built on deterministic partition planning.

#![forbid(unsafe_op_in_unsafe_fn)]
#![deny(missing_docs)]
#![cfg_attr(feature = "strict_api", deny(unreachable_pub))]
#![cfg_attr(not(feature = "strict_api"), warn(unreachable_pub))]
#![cfg_attr(feature = "strict_docs", deny(missing_docs))]
#![cfg_attr(not(feature = "strict_docs"), allow(missing_docs))]

/// Re-exported bounded parallel map and partition planning utilities.
pub use adze_concurrency_bounded_map_core::{
    ParallelPartitionPlan, bounded_parallel_map, normalized_concurrency,
};
