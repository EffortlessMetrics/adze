#![no_main]

use adze_concurrency_init_rayon_core::init_rayon_global_once;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    for chunk in data.chunks(8).take(1024) {
        let mut value_bytes = [0u8; 8];

        for (idx, byte) in chunk.iter().enumerate() {
            value_bytes[idx] = *byte;
        }

        let thread_count = u64::from_le_bytes(value_bytes) as usize;
        assert!(init_rayon_global_once(thread_count).is_ok());
    }
});
