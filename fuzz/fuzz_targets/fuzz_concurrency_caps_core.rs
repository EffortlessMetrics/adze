#![no_main]

use adze_concurrency_caps_contract_core::{bounded_parallel_map, normalized_concurrency};
use libfuzzer_sys::fuzz_target;

fn model_transform(value: i32) -> i32 {
    value.wrapping_mul(17).wrapping_add(3)
}

fuzz_target!(|data: &[u8]| {
    if data.is_empty() {
        return;
    }

    let concurrency = usize::from(data[0]);
    let mut values = Vec::with_capacity(data.len() / 4);

    for chunk in data[1..].chunks_exact(4).take(1024) {
        let mut bytes = [0u8; 4];
        bytes.copy_from_slice(chunk);
        values.push(i32::from_le_bytes(bytes));
    }

    let mut got = bounded_parallel_map(values.clone(), concurrency, model_transform);
    let mut expected: Vec<i32> = values.into_iter().map(model_transform).collect();

    got.sort_unstable();
    expected.sort_unstable();

    assert_eq!(got, expected);
    assert_eq!(normalized_concurrency(concurrency), concurrency.max(1));
});
