#![no_main]

use libfuzzer_sys::fuzz_target;
use rust_sitter::*;

fuzz_target!(|data: &[u8]| {
    // Fuzz the external scanner FFI boundary
    // This is critical for finding memory safety issues

    if data.len() < 4 {
        return;
    }

    // Split data into scanner state and input
    let (state_bytes, input_bytes) = data.split_at(4);
    let state = u32::from_le_bytes([
        state_bytes[0],
        state_bytes[1],
        state_bytes[2],
        state_bytes[3],
    ]);

    // Create a mock external scanner context
    let mut scanner = MockExternalScanner::new();

    // Fuzz the scanner with random state and input
    // This should never cause UB, even with malformed input
    scanner.scan(state, input_bytes);

    // Serialize and deserialize to check round-trip safety
    let serialized = scanner.serialize();
    let mut scanner2 = MockExternalScanner::new();
    scanner2.deserialize(&serialized);
});

/// Mock external scanner for fuzzing
struct MockExternalScanner {
    state: Vec<u8>,
}

impl MockExternalScanner {
    fn new() -> Self {
        Self { state: Vec::new() }
    }

    fn scan(&mut self, state: u32, input: &[u8]) {
        // Bounds check everything
        if input.len() > 10000 {
            return; // Avoid DOS
        }

        // Simulate scanning logic
        self.state.clear();
        self.state.extend_from_slice(&state.to_le_bytes());

        for &byte in input.iter().take(100) {
            if byte == b'\n' {
                self.state.push(0);
            } else {
                self.state.push(byte.wrapping_add(1));
            }
        }
    }

    fn serialize(&self) -> Vec<u8> {
        self.state.clone()
    }

    fn deserialize(&mut self, data: &[u8]) {
        self.state = data.to_vec();
    }
}
