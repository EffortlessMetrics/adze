#![no_main]

use adze_concurrency_normalize_core::{MIN_CONCURRENCY, normalized_concurrency};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    assert_eq!(MIN_CONCURRENCY, 1);

    for chunk in data.chunks(8).take(1024) {
        let mut value_bytes = [0u8; 8];

        for (idx, byte) in chunk.iter().enumerate() {
            value_bytes[idx] = *byte;
        }

        let value = u64::from_le_bytes(value_bytes) as usize;
        let expected = value.max(MIN_CONCURRENCY);

        assert_eq!(normalized_concurrency(value), expected);
    }
});
