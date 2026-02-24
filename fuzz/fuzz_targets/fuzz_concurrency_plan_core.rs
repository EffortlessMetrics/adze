#![no_main]

use adze_concurrency_plan_core::{
    DIRECT_PARALLEL_THRESHOLD_MULTIPLIER, ParallelPartitionPlan, normalized_concurrency,
};
use libfuzzer_sys::fuzz_target;

fn model_plan(item_count: usize, requested_concurrency: usize) -> (usize, usize, bool) {
    let concurrency = requested_concurrency.max(1);
    let chunk_size = if item_count == 0 {
        1
    } else {
        item_count.div_ceil(concurrency)
    };
    let use_direct_parallel_iter =
        item_count <= concurrency.saturating_mul(DIRECT_PARALLEL_THRESHOLD_MULTIPLIER);

    (concurrency, chunk_size, use_direct_parallel_iter)
}

fuzz_target!(|data: &[u8]| {
    if data.is_empty() {
        return;
    }

    for chunk in data.chunks(16).take(256) {
        let mut item_count_bytes = [0u8; 8];
        let mut concurrency_bytes = [0u8; 8];

        for (idx, byte) in chunk.iter().take(8).enumerate() {
            item_count_bytes[idx] = *byte;
        }
        for (idx, byte) in chunk.iter().skip(8).take(8).enumerate() {
            concurrency_bytes[idx] = *byte;
        }

        let item_count = (u64::from_le_bytes(item_count_bytes) as usize) % 4097;
        let requested_concurrency = (u64::from_le_bytes(concurrency_bytes) as usize) % 1025;

        let plan = ParallelPartitionPlan::for_item_count(item_count, requested_concurrency);
        let (expected_concurrency, expected_chunk_size, expected_direct) =
            model_plan(item_count, requested_concurrency);

        assert_eq!(
            normalized_concurrency(requested_concurrency),
            expected_concurrency
        );
        assert_eq!(plan.concurrency, expected_concurrency);
        assert_eq!(plan.chunk_size, expected_chunk_size);
        assert_eq!(plan.use_direct_parallel_iter, expected_direct);
        assert!(plan.chunk_size >= 1);
    }
});
