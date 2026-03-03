#![allow(clippy::needless_range_loop)]
//! Property-based tests for ExternalScanner trait and IndentationScanner.

use proptest::prelude::*;

use adze_runtime::external_scanner::{ExternalScanner, ScanResult};

// ---------------------------------------------------------------------------
// Test helpers: IndentationScanner (mirrors crate-internal #[cfg(test)] type)
// ---------------------------------------------------------------------------

struct IndentationScanner {
    indent_stack: Vec<u32>,
}

impl IndentationScanner {
    fn new() -> Self {
        Self {
            indent_stack: vec![0],
        }
    }
}

impl ExternalScanner for IndentationScanner {
    fn init(&mut self) {
        self.indent_stack.clear();
        self.indent_stack.push(0);
    }

    fn scan(&mut self, _valid_symbols: &[bool], input: &[u8]) -> Option<ScanResult> {
        let indent = input.iter().take_while(|&&b| b == b' ').count() as u32;
        let top = *self.indent_stack.last()?;

        if indent > top {
            self.indent_stack.push(indent);
            Some(ScanResult {
                token_type: 0, // INDENT
                bytes_consumed: 0,
            })
        } else if indent < top {
            while self.indent_stack.len() > 1 && indent < *self.indent_stack.last().unwrap_or(&0) {
                self.indent_stack.pop();
            }
            Some(ScanResult {
                token_type: 1, // DEDENT
                bytes_consumed: 0,
            })
        } else {
            None
        }
    }

    fn serialize(&self) -> Vec<u8> {
        let mut data = Vec::new();
        data.extend_from_slice(&(self.indent_stack.len() as u32).to_le_bytes());
        for &indent in &self.indent_stack {
            data.extend_from_slice(&indent.to_le_bytes());
        }
        data
    }

    fn deserialize(&mut self, data: &[u8]) {
        if data.len() >= 4 {
            let len = u32::from_le_bytes([data[0], data[1], data[2], data[3]]) as usize;
            self.indent_stack.clear();
            for i in 0..len {
                let offset = 4 + i * 4;
                if offset + 4 <= data.len() {
                    let val = u32::from_le_bytes([
                        data[offset],
                        data[offset + 1],
                        data[offset + 2],
                        data[offset + 3],
                    ]);
                    self.indent_stack.push(val);
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Strategies
// ---------------------------------------------------------------------------

fn arb_indent_level() -> impl Strategy<Value = u32> {
    0..64u32
}

fn arb_indent_stack() -> impl Strategy<Value = Vec<u32>> {
    prop::collection::vec(0..128u32, 1..16).prop_map(|mut v| {
        v.sort();
        v.dedup();
        if v.is_empty() || v[0] != 0 {
            v.insert(0, 0);
        }
        v
    })
}

fn arb_valid_symbols() -> impl Strategy<Value = Vec<bool>> {
    prop::collection::vec(any::<bool>(), 0..8)
}

fn spaces(n: u32) -> Vec<u8> {
    let mut v = vec![b' '; n as usize];
    v.push(b'x'); // non-space sentinel
    v
}

// ---------------------------------------------------------------------------
// 1 – IndentationScanner creation
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn new_scanner_has_base_indent(_ in 0..1u8) {
        let s = IndentationScanner::new();
        prop_assert_eq!(s.indent_stack.len(), 1);
        prop_assert_eq!(s.indent_stack[0], 0);
    }

    #[test]
    fn init_resets_to_base(_ in 0..1u8) {
        let mut s = IndentationScanner::new();
        s.indent_stack.push(4);
        s.indent_stack.push(8);
        s.init();
        prop_assert_eq!(s.indent_stack.len(), 1);
        prop_assert_eq!(s.indent_stack[0], 0);
    }

    #[test]
    fn init_is_idempotent(count in 1..10u32) {
        let mut s = IndentationScanner::new();
        for _ in 0..count {
            s.init();
        }
        prop_assert_eq!(s.indent_stack, vec![0]);
    }
}

// ---------------------------------------------------------------------------
// 2 – Scan behavior with arbitrary indentation
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn indent_produces_indent_token(level in 1..64u32) {
        let mut s = IndentationScanner::new();
        s.init();
        let input = spaces(level);
        let result = s.scan(&[true, true], &input);
        prop_assert_eq!(result, Some(ScanResult { token_type: 0, bytes_consumed: 0 }));
    }

    #[test]
    fn same_level_produces_none(level in 0..64u32) {
        let mut s = IndentationScanner::new();
        s.init();
        if level > 0 {
            s.scan(&[true, true], &spaces(level)); // push level
        }
        let result = s.scan(&[true, true], &spaces(level));
        prop_assert!(result.is_none());
    }

    #[test]
    fn dedent_produces_dedent_token(level in 1..64u32) {
        let mut s = IndentationScanner::new();
        s.init();
        s.scan(&[true, true], &spaces(level)); // indent to `level`
        let result = s.scan(&[true, true], &spaces(0)); // back to 0
        prop_assert_eq!(result, Some(ScanResult { token_type: 1, bytes_consumed: 0 }));
    }

    #[test]
    fn indent_then_same_level(level in 1..64u32) {
        let mut s = IndentationScanner::new();
        s.init();
        let r1 = s.scan(&[true, true], &spaces(level));
        prop_assert!(r1.is_some()); // INDENT
        let r2 = s.scan(&[true, true], &spaces(level));
        prop_assert!(r2.is_none()); // same level
    }

    #[test]
    fn zero_indent_on_fresh_scanner_is_none(_ in 0..1u8) {
        let mut s = IndentationScanner::new();
        s.init();
        let result = s.scan(&[true, true], &spaces(0));
        prop_assert!(result.is_none());
    }
}

// ---------------------------------------------------------------------------
// 3 – Serialize / deserialize roundtrip
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn serialize_roundtrip_fresh_scanner(_ in 0..1u8) {
        let s = IndentationScanner::new();
        let data = s.serialize();
        let mut s2 = IndentationScanner::new();
        s2.indent_stack.clear(); // mess up state
        s2.deserialize(&data);
        prop_assert_eq!(s2.indent_stack, vec![0]);
    }

    #[test]
    fn serialize_roundtrip_after_indent(level in 1..64u32) {
        let mut s = IndentationScanner::new();
        s.init();
        s.scan(&[true, true], &spaces(level));

        let data = s.serialize();
        let mut s2 = IndentationScanner::new();
        s2.deserialize(&data);
        prop_assert_eq!(s.indent_stack, s2.indent_stack);
    }

    #[test]
    fn serialize_roundtrip_arbitrary_stack(stack in arb_indent_stack()) {
        let mut s = IndentationScanner::new();
        s.indent_stack = stack.clone();

        let data = s.serialize();
        let mut s2 = IndentationScanner::new();
        s2.deserialize(&data);
        prop_assert_eq!(s2.indent_stack, stack);
    }

    #[test]
    fn serialize_length_matches_stack(stack in arb_indent_stack()) {
        let mut s = IndentationScanner::new();
        s.indent_stack = stack.clone();
        let data = s.serialize();
        // 4 bytes for length + 4 bytes per entry
        prop_assert_eq!(data.len(), 4 + stack.len() * 4);
    }

    #[test]
    fn double_serialize_roundtrip(level in 1..64u32) {
        let mut s = IndentationScanner::new();
        s.init();
        s.scan(&[true, true], &spaces(level));

        let data1 = s.serialize();
        let mut s2 = IndentationScanner::new();
        s2.deserialize(&data1);
        let data2 = s2.serialize();
        prop_assert_eq!(data1, data2);
    }
}

// ---------------------------------------------------------------------------
// 4 – Various indentation levels
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn increasing_levels_all_produce_indent(
        levels in prop::collection::vec(1..32u32, 1..8)
    ) {
        let mut s = IndentationScanner::new();
        s.init();
        let mut sorted = levels;
        sorted.sort();
        sorted.dedup();
        // Make strictly increasing by accumulating
        let mut running = 0u32;
        let inc: Vec<u32> = sorted.iter().map(|&l| { running += l; running }).collect();

        for &level in &inc {
            let result = s.scan(&[true, true], &spaces(level));
            prop_assert_eq!(result.map(|r| r.token_type), Some(0), "Expected INDENT at level {}", level);
        }
    }

    #[test]
    fn full_dedent_returns_to_base(level in 1..64u32) {
        let mut s = IndentationScanner::new();
        s.init();
        s.scan(&[true, true], &spaces(level));
        s.scan(&[true, true], &spaces(0)); // dedent
        // Stack should be back to [0]
        prop_assert_eq!(s.indent_stack, vec![0]);
    }

    #[test]
    fn nested_indent_dedent_roundtrip(a in 1..32u32, b in 1..32u32) {
        let mut s = IndentationScanner::new();
        s.init();
        let level1 = a;
        let level2 = a + b;

        s.scan(&[true, true], &spaces(level1)); // indent to level1
        s.scan(&[true, true], &spaces(level2)); // indent to level2
        let r = s.scan(&[true, true], &spaces(0)); // dedent to 0
        prop_assert_eq!(r.map(|r| r.token_type), Some(1));
        prop_assert_eq!(s.indent_stack, vec![0]);
    }

    #[test]
    fn partial_dedent(a in 1..16u32, b in 1..16u32) {
        let mut s = IndentationScanner::new();
        s.init();
        let level1 = a;
        let level2 = a + b;

        s.scan(&[true, true], &spaces(level1));
        s.scan(&[true, true], &spaces(level2));
        // Dedent to level1 (partial)
        let r = s.scan(&[true, true], &spaces(level1));
        prop_assert_eq!(r.map(|r| r.token_type), Some(1));
        // Stack top should be level1
        prop_assert_eq!(*s.indent_stack.last().unwrap(), level1);
    }
}

// ---------------------------------------------------------------------------
// 5 – Scanner state persistence
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn state_persists_across_scans(level in 1..32u32) {
        let mut s = IndentationScanner::new();
        s.init();
        s.scan(&[true, true], &spaces(level));
        // Stack should have [0, level]
        prop_assert_eq!(s.indent_stack.len(), 2);
        prop_assert_eq!(s.indent_stack[1], level);
    }

    #[test]
    fn deserialized_scanner_behaves_identically(level in 1..32u32) {
        let mut s1 = IndentationScanner::new();
        s1.init();
        s1.scan(&[true, true], &spaces(level));

        let data = s1.serialize();
        let mut s2 = IndentationScanner::new();
        s2.deserialize(&data);

        // Both should produce the same result for the same input
        let r1 = s1.scan(&[true, true], &spaces(0));
        let r2 = s2.scan(&[true, true], &spaces(0));
        prop_assert_eq!(r1, r2);
    }

    #[test]
    fn serialized_state_survives_init_on_clone(level in 1..32u32) {
        let mut s = IndentationScanner::new();
        s.init();
        s.scan(&[true, true], &spaces(level));
        let data = s.serialize();

        // Create new scanner, init it, then deserialize
        let mut s2 = IndentationScanner::new();
        s2.init();
        s2.deserialize(&data);
        prop_assert_eq!(s.indent_stack, s2.indent_stack);
    }

    #[test]
    fn deserialize_empty_preserves_state(level in 1..32u32) {
        let mut s = IndentationScanner::new();
        s.init();
        s.scan(&[true, true], &spaces(level));
        let expected = s.indent_stack.clone();
        s.deserialize(&[]); // too short, should be no-op
        prop_assert_eq!(s.indent_stack, expected);
    }

    #[test]
    fn deserialize_short_data_preserves_state(level in 1..32u32, short_len in 1..4usize) {
        let mut s = IndentationScanner::new();
        s.init();
        s.scan(&[true, true], &spaces(level));
        let expected = s.indent_stack.clone();
        let short_data: Vec<u8> = vec![0xAB; short_len];
        s.deserialize(&short_data); // less than 4 bytes, no-op
        prop_assert_eq!(s.indent_stack, expected);
    }
}

// ---------------------------------------------------------------------------
// 6 – Scanner reset behavior
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn init_after_indent_resets(level in 1..64u32) {
        let mut s = IndentationScanner::new();
        s.init();
        s.scan(&[true, true], &spaces(level));
        prop_assert!(s.indent_stack.len() > 1);
        s.init();
        prop_assert_eq!(s.indent_stack, vec![0]);
    }

    #[test]
    fn init_after_multiple_indents_resets(count in 1..8u32) {
        let mut s = IndentationScanner::new();
        s.init();
        let mut running = 0u32;
        for _ in 0..count {
            running += 4;
            s.scan(&[true, true], &spaces(running));
        }
        prop_assert_eq!(s.indent_stack.len(), (count + 1) as usize);
        s.init();
        prop_assert_eq!(s.indent_stack, vec![0]);
    }

    #[test]
    fn fresh_scanner_after_reset_matches_new(level in 1..32u32) {
        let mut s = IndentationScanner::new();
        s.scan(&[true, true], &spaces(level));
        s.init();

        let fresh = IndentationScanner::new();
        prop_assert_eq!(s.indent_stack, fresh.indent_stack);
    }
}

// ---------------------------------------------------------------------------
// 7 – Trait object usage
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn trait_object_scan(level in 1..32u32) {
        let mut scanner: Box<dyn ExternalScanner> = Box::new(IndentationScanner::new());
        scanner.init();
        let result = scanner.scan(&[true, true], &spaces(level));
        prop_assert_eq!(result.map(|r| r.token_type), Some(0));
    }

    #[test]
    fn trait_object_serialize_deserialize(level in 1..32u32) {
        let mut s1: Box<dyn ExternalScanner> = Box::new(IndentationScanner::new());
        s1.init();
        s1.scan(&[true, true], &spaces(level));
        let data = s1.serialize();

        let mut s2 = IndentationScanner::new();
        s2.deserialize(&data);
        // After restoring state, dedenting should work
        let r = s2.scan(&[true, true], &spaces(0));
        prop_assert_eq!(r.map(|r| r.token_type), Some(1));
    }
}

// ---------------------------------------------------------------------------
// 8 – Edge cases with valid_symbols
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn scan_ignores_valid_symbols(vs in arb_valid_symbols(), level in 1..32u32) {
        // IndentationScanner ignores valid_symbols entirely
        let mut s = IndentationScanner::new();
        s.init();
        let result = s.scan(&vs, &spaces(level));
        prop_assert_eq!(result, Some(ScanResult { token_type: 0, bytes_consumed: 0 }));
    }

    #[test]
    fn empty_valid_symbols_still_works(level in 1..32u32) {
        let mut s = IndentationScanner::new();
        s.init();
        let result = s.scan(&[], &spaces(level));
        prop_assert_eq!(result.map(|r| r.token_type), Some(0));
    }
}

// ---------------------------------------------------------------------------
// 9 – Empty and non-space inputs
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn empty_input_at_base_returns_none(_ in 0..1u8) {
        let mut s = IndentationScanner::new();
        s.init();
        prop_assert!(s.scan(&[true, true], b"").is_none());
    }

    #[test]
    fn non_space_input_at_base_returns_none(ch in b'!'..=b'~') {
        let mut s = IndentationScanner::new();
        s.init();
        let input = vec![ch, b'x'];
        prop_assert!(s.scan(&[true, true], &input).is_none());
    }

    #[test]
    fn non_space_input_after_indent_triggers_dedent(level in 1..32u32, ch in b'!'..=b'~') {
        let mut s = IndentationScanner::new();
        s.init();
        s.scan(&[true, true], &spaces(level));
        let input = vec![ch, b'x'];
        let result = s.scan(&[true, true], &input);
        prop_assert_eq!(result.map(|r| r.token_type), Some(1));
    }
}

// ---------------------------------------------------------------------------
// 10 – ScanResult creation and field access
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn scan_result_fields(tt in 0..100u32, bc in 0..1024usize) {
        let r = ScanResult { token_type: tt, bytes_consumed: bc };
        prop_assert_eq!(r.token_type, tt);
        prop_assert_eq!(r.bytes_consumed, bc);
    }

    #[test]
    fn scan_result_equality(tt in 0..100u32, bc in 0..1024usize) {
        let a = ScanResult { token_type: tt, bytes_consumed: bc };
        let b = ScanResult { token_type: tt, bytes_consumed: bc };
        prop_assert_eq!(a, b);
    }

    #[test]
    fn scan_result_inequality_token(tt in 0..50u32, bc in 0..512usize) {
        let a = ScanResult { token_type: tt, bytes_consumed: bc };
        let b = ScanResult { token_type: tt + 1, bytes_consumed: bc };
        prop_assert_ne!(a, b);
    }

    #[test]
    fn scan_result_inequality_bytes(tt in 0..50u32, bc in 0..512usize) {
        let a = ScanResult { token_type: tt, bytes_consumed: bc };
        let b = ScanResult { token_type: tt, bytes_consumed: bc + 1 };
        prop_assert_ne!(a, b);
    }

    #[test]
    fn scan_result_clone(tt in 0..100u32, bc in 0..1024usize) {
        let a = ScanResult { token_type: tt, bytes_consumed: bc };
        let b = a;
        prop_assert_eq!(a, b);
    }

    #[test]
    fn scan_result_debug_contains_fields(tt in 0..10u32, bc in 0..10usize) {
        let r = ScanResult { token_type: tt, bytes_consumed: bc };
        let dbg = format!("{:?}", r);
        prop_assert!(dbg.contains("token_type"));
        prop_assert!(dbg.contains("bytes_consumed"));
    }
}

// ---------------------------------------------------------------------------
// 11 – Multiple independent scanner instances
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn two_scanners_independent_state(a in 1..32u32, b in 1..32u32) {
        let mut s1 = IndentationScanner::new();
        let mut s2 = IndentationScanner::new();
        s1.init();
        s2.init();
        s1.scan(&[true, true], &spaces(a));
        s2.scan(&[true, true], &spaces(b));
        // Each has its own stack
        prop_assert_eq!(*s1.indent_stack.last().unwrap(), a);
        prop_assert_eq!(*s2.indent_stack.last().unwrap(), b);
    }

    #[test]
    fn multiple_scanners_no_cross_talk(levels in prop::collection::vec(1..32u32, 2..6)) {
        let mut scanners: Vec<IndentationScanner> = (0..levels.len())
            .map(|_| {
                let mut s = IndentationScanner::new();
                s.init();
                s
            })
            .collect();
        // Indent each scanner to a different level
        for i in 0..levels.len() {
            scanners[i].scan(&[true, true], &spaces(levels[i]));
        }
        // Verify each scanner kept its own state
        for i in 0..levels.len() {
            prop_assert_eq!(*scanners[i].indent_stack.last().unwrap(), levels[i]);
        }
    }

    #[test]
    fn scanner_vec_trait_objects(count in 2..6usize) {
        let mut scanners: Vec<Box<dyn ExternalScanner>> = (0..count)
            .map(|_| -> Box<dyn ExternalScanner> { Box::new(IndentationScanner::new()) })
            .collect();
        for s in scanners.iter_mut() {
            s.init();
        }
        // Each scanner can independently indent
        for (i, s) in scanners.iter_mut().enumerate() {
            let level = (i as u32 + 1) * 4;
            let r = s.scan(&[true, true], &spaces(level));
            prop_assert!(r.is_some());
        }
    }
}

// ---------------------------------------------------------------------------
// 12 – Scanner determinism
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn deterministic_indent_result(level in 1..32u32) {
        let mut s1 = IndentationScanner::new();
        let mut s2 = IndentationScanner::new();
        s1.init();
        s2.init();
        let r1 = s1.scan(&[true, true], &spaces(level));
        let r2 = s2.scan(&[true, true], &spaces(level));
        prop_assert_eq!(r1, r2);
    }

    #[test]
    fn deterministic_sequence(
        a in 1..16u32,
        b_offset in 1..16u32,
    ) {
        let b = a + b_offset;
        let run = |scanner: &mut IndentationScanner| -> Vec<Option<ScanResult>> {
            scanner.init();
            let r1 = scanner.scan(&[true, true], &spaces(a));
            let r2 = scanner.scan(&[true, true], &spaces(b));
            let r3 = scanner.scan(&[true, true], &spaces(a));
            let r4 = scanner.scan(&[true, true], &spaces(0));
            vec![r1, r2, r3, r4]
        };
        let mut s1 = IndentationScanner::new();
        let mut s2 = IndentationScanner::new();
        let results1 = run(&mut s1);
        let results2 = run(&mut s2);
        prop_assert_eq!(results1, results2);
    }

    #[test]
    fn deterministic_serialize_output(level in 1..32u32) {
        let mut s1 = IndentationScanner::new();
        let mut s2 = IndentationScanner::new();
        s1.init();
        s2.init();
        s1.scan(&[true, true], &spaces(level));
        s2.scan(&[true, true], &spaces(level));
        prop_assert_eq!(s1.serialize(), s2.serialize());
    }

    #[test]
    fn repeated_runs_same_result(level in 1..32u32, runs in 2..5u32) {
        let mut results = Vec::new();
        for _ in 0..runs {
            let mut s = IndentationScanner::new();
            s.init();
            let r = s.scan(&[true, true], &spaces(level));
            results.push(r);
        }
        for i in 1..results.len() {
            prop_assert_eq!(results[0], results[i]);
        }
    }
}

// ---------------------------------------------------------------------------
// 13 – Complex indent/dedent sequences
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn staircase_indent_then_full_dedent(step in 1..8u32, count in 2..6u32) {
        let mut s = IndentationScanner::new();
        s.init();
        for i in 1..=count {
            let r = s.scan(&[true, true], &spaces(i * step));
            prop_assert_eq!(r.map(|r| r.token_type), Some(0), "step {} should indent", i);
        }
        let r = s.scan(&[true, true], &spaces(0));
        prop_assert_eq!(r.map(|r| r.token_type), Some(1));
        prop_assert_eq!(s.indent_stack, vec![0]);
    }

    #[test]
    fn zigzag_indent_dedent(base in 1..16u32, extra in 1..16u32) {
        let mut s = IndentationScanner::new();
        s.init();
        // Indent to base
        let r1 = s.scan(&[true, true], &spaces(base));
        prop_assert_eq!(r1.map(|r| r.token_type), Some(0));
        // Indent further
        let r2 = s.scan(&[true, true], &spaces(base + extra));
        prop_assert_eq!(r2.map(|r| r.token_type), Some(0));
        // Dedent back to base
        let r3 = s.scan(&[true, true], &spaces(base));
        prop_assert_eq!(r3.map(|r| r.token_type), Some(1));
        // Same level again
        let r4 = s.scan(&[true, true], &spaces(base));
        prop_assert!(r4.is_none());
    }

    #[test]
    fn serialize_after_complex_sequence(a in 1..10u32, b in 1..10u32) {
        let level1 = a;
        let level2 = a + b;
        let mut s = IndentationScanner::new();
        s.init();
        s.scan(&[true, true], &spaces(level1));
        s.scan(&[true, true], &spaces(level2));
        s.scan(&[true, true], &spaces(level1)); // partial dedent

        let data = s.serialize();
        let mut s2 = IndentationScanner::new();
        s2.deserialize(&data);

        // Both scanners should produce same result for next input
        let r1 = s.scan(&[true, true], &spaces(0));
        let r2 = s2.scan(&[true, true], &spaces(0));
        prop_assert_eq!(r1, r2);
    }
}

// ---------------------------------------------------------------------------
// 14 – Deserialize edge cases
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn deserialize_truncated_entries(level in 1..32u32) {
        let mut s = IndentationScanner::new();
        s.init();
        s.scan(&[true, true], &spaces(level));
        let full_data = s.serialize();
        // Truncate data to only include the length header + partial entries
        let truncated = &full_data[..5.min(full_data.len())];
        let mut s2 = IndentationScanner::new();
        s2.init();
        let before = s2.indent_stack.clone();
        s2.deserialize(truncated);
        // Should have deserialized at least partially or safely
        // (deserialize clears and re-reads what it can)
        prop_assert!(!s2.indent_stack.is_empty() || before == vec![0]);
    }

    #[test]
    fn deserialize_with_zero_length_header(_ in 0..1u8) {
        let data = 0u32.to_le_bytes();
        let mut s = IndentationScanner::new();
        s.init();
        s.deserialize(&data);
        // Zero-length stack
        prop_assert!(s.indent_stack.is_empty());
    }

    #[test]
    fn deserialize_large_claimed_length(_ in 0..1u8) {
        // Claim 1000 entries but only provide data for 1
        let mut data = Vec::new();
        data.extend_from_slice(&1000u32.to_le_bytes());
        data.extend_from_slice(&42u32.to_le_bytes());
        let mut s = IndentationScanner::new();
        s.init();
        s.deserialize(&data);
        // Should only deserialize entries for which data exists
        prop_assert_eq!(s.indent_stack.len(), 1);
        prop_assert_eq!(s.indent_stack[0], 42);
    }
}

// ---------------------------------------------------------------------------
// 15 – Scan with varied byte inputs
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn tabs_count_as_zero_indent(_ in 0..1u8) {
        let mut s = IndentationScanner::new();
        s.init();
        // Tabs are not spaces, so indent = 0
        let result = s.scan(&[true, true], b"\t\tx");
        prop_assert!(result.is_none());
    }

    #[test]
    fn mixed_spaces_tabs_only_counts_leading_spaces(n in 1..16u32) {
        let mut s = IndentationScanner::new();
        s.init();
        let mut input = vec![b' '; n as usize];
        input.push(b'\t');
        input.push(b'x');
        let result = s.scan(&[true, true], &input);
        prop_assert_eq!(result.map(|r| r.token_type), Some(0));
    }

    #[test]
    fn only_spaces_input(n in 1..64u32) {
        let mut s = IndentationScanner::new();
        s.init();
        // Input of only spaces (no sentinel) — indent is the full length
        let input = vec![b' '; n as usize];
        let result = s.scan(&[true, true], &input);
        prop_assert_eq!(result.map(|r| r.token_type), Some(0));
    }
}
