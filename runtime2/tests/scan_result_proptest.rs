#![allow(clippy::needless_range_loop)]

use proptest::prelude::*;

use adze_runtime::ScanResult;

// ---------------------------------------------------------------------------
// Strategies
// ---------------------------------------------------------------------------

fn arb_scan_result() -> impl Strategy<Value = ScanResult> {
    (any::<u32>(), any::<usize>()).prop_map(|(token_type, bytes_consumed)| ScanResult {
        token_type,
        bytes_consumed,
    })
}

fn arb_small_bytes() -> impl Strategy<Value = ScanResult> {
    (any::<u32>(), 0..1024usize).prop_map(|(token_type, bytes_consumed)| ScanResult {
        token_type,
        bytes_consumed,
    })
}

// ---------------------------------------------------------------------------
// 1 – Construction and field access
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn fields_roundtrip(token_type in any::<u32>(), bytes_consumed in any::<usize>()) {
        let r = ScanResult { token_type, bytes_consumed };
        prop_assert_eq!(r.token_type, token_type);
        prop_assert_eq!(r.bytes_consumed, bytes_consumed);
    }

    #[test]
    fn zero_bytes_consumed(token_type in any::<u32>()) {
        let r = ScanResult { token_type, bytes_consumed: 0 };
        prop_assert_eq!(r.bytes_consumed, 0);
        prop_assert_eq!(r.token_type, token_type);
    }

    #[test]
    fn zero_token_type(bytes_consumed in any::<usize>()) {
        let r = ScanResult { token_type: 0, bytes_consumed };
        prop_assert_eq!(r.token_type, 0);
        prop_assert_eq!(r.bytes_consumed, bytes_consumed);
    }
}

// ---------------------------------------------------------------------------
// 2 – Copy semantics
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn copy_preserves_all_fields(r in arb_scan_result()) {
        let r2 = r;
        let r3 = r;
        prop_assert_eq!(r, r2);
        prop_assert_eq!(r, r3);
    }

    #[test]
    fn copy_is_independent(token_type in any::<u32>(), bytes_consumed in any::<usize>()) {
        let r1 = ScanResult { token_type, bytes_consumed };
        let mut r2 = r1;
        r2.token_type = token_type.wrapping_add(1);
        prop_assert_eq!(r1.token_type, token_type);
        prop_assert_ne!(r1.token_type, r2.token_type);
    }
}

// ---------------------------------------------------------------------------
// 3 – Clone semantics
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn clone_equals_original(r in arb_scan_result()) {
        let cloned = r;
        prop_assert_eq!(r, cloned);
    }

    #[test]
    fn clone_is_independent(r in arb_scan_result()) {
        let mut cloned = r;
        cloned.bytes_consumed = r.bytes_consumed.wrapping_add(1);
        prop_assert_ne!(r.bytes_consumed, cloned.bytes_consumed);
    }

    #[test]
    fn clone_fields_match(token_type in any::<u32>(), bytes_consumed in any::<usize>()) {
        let r = ScanResult { token_type, bytes_consumed };
        let c = r;
        prop_assert_eq!(c.token_type, token_type);
        prop_assert_eq!(c.bytes_consumed, bytes_consumed);
    }
}

// ---------------------------------------------------------------------------
// 4 – PartialEq / Eq
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn eq_reflexive(r in arb_scan_result()) {
        prop_assert_eq!(r, r);
    }

    #[test]
    fn eq_symmetric(a in arb_scan_result(), b in arb_scan_result()) {
        prop_assert_eq!(a == b, b == a);
    }

    #[test]
    fn eq_by_all_fields(token_type in any::<u32>(), bytes_consumed in any::<usize>()) {
        let a = ScanResult { token_type, bytes_consumed };
        let b = ScanResult { token_type, bytes_consumed };
        prop_assert_eq!(a, b);
    }

    #[test]
    fn ne_when_token_type_differs(
        tt_a in any::<u32>(),
        tt_b in any::<u32>(),
        bytes_consumed in any::<usize>(),
    ) {
        let a = ScanResult { token_type: tt_a, bytes_consumed };
        let b = ScanResult { token_type: tt_b, bytes_consumed };
        prop_assert_eq!(a == b, tt_a == tt_b);
    }

    #[test]
    fn ne_when_bytes_consumed_differs(
        token_type in any::<u32>(),
        bc_a in any::<usize>(),
        bc_b in any::<usize>(),
    ) {
        let a = ScanResult { token_type, bytes_consumed: bc_a };
        let b = ScanResult { token_type, bytes_consumed: bc_b };
        prop_assert_eq!(a == b, bc_a == bc_b);
    }
}

// ---------------------------------------------------------------------------
// 5 – Debug formatting
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn debug_contains_struct_name(r in arb_scan_result()) {
        let dbg = format!("{:?}", r);
        prop_assert!(dbg.contains("ScanResult"));
    }

    #[test]
    fn debug_contains_all_field_values(r in arb_scan_result()) {
        let dbg = format!("{:?}", r);
        prop_assert!(dbg.contains(&r.token_type.to_string()));
        prop_assert!(dbg.contains(&r.bytes_consumed.to_string()));
    }

    #[test]
    fn debug_is_nonempty(r in arb_scan_result()) {
        let dbg = format!("{:?}", r);
        prop_assert!(!dbg.is_empty());
    }
}

// ---------------------------------------------------------------------------
// 6 – Various lengths
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn small_bytes_consumed(r in arb_small_bytes()) {
        prop_assert!(r.bytes_consumed < 1024);
    }

    #[test]
    fn large_bytes_consumed(token_type in any::<u32>()) {
        let r = ScanResult { token_type, bytes_consumed: usize::MAX };
        prop_assert_eq!(r.bytes_consumed, usize::MAX);
    }

    #[test]
    fn power_of_two_lengths(
        token_type in any::<u32>(),
        exp in 0..20u32,
    ) {
        let len = 1usize << exp;
        let r = ScanResult { token_type, bytes_consumed: len };
        prop_assert_eq!(r.bytes_consumed, len);
        prop_assert!(r.bytes_consumed.is_power_of_two());
    }

    #[test]
    fn consecutive_lengths(
        token_type in any::<u32>(),
        base in 0..10_000usize,
    ) {
        let r1 = ScanResult { token_type, bytes_consumed: base };
        let r2 = ScanResult { token_type, bytes_consumed: base + 1 };
        prop_assert_ne!(r1, r2);
        prop_assert_eq!(r2.bytes_consumed - r1.bytes_consumed, 1);
    }
}

// ---------------------------------------------------------------------------
// 7 – Boundary values
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn max_values(_ in 0..1u8) {
        let r = ScanResult { token_type: u32::MAX, bytes_consumed: usize::MAX };
        let r2 = r;
        prop_assert_eq!(r, r2);
    }

    #[test]
    fn min_values(_ in 0..1u8) {
        let r = ScanResult { token_type: 0, bytes_consumed: 0 };
        let r2 = r;
        prop_assert_eq!(r, r2);
    }

    #[test]
    fn boundary_field_values(
        token_type in prop_oneof![Just(0u32), Just(u32::MAX), any::<u32>()],
        bytes_consumed in prop_oneof![Just(0usize), Just(usize::MAX), any::<usize>()],
    ) {
        let r = ScanResult { token_type, bytes_consumed };
        let cloned = r;
        prop_assert_eq!(r, cloned);
        let _ = format!("{:?}", r);
    }
}

// ---------------------------------------------------------------------------
// 8 – Collection behaviour
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn results_in_vec(count in 1..100usize, r in arb_scan_result()) {
        let v: Vec<ScanResult> = vec![r; count];
        prop_assert_eq!(v.len(), count);
        for i in 0..v.len() {
            prop_assert_eq!(v[i], r);
        }
    }

    #[test]
    fn vec_dedup_identical(r in arb_scan_result(), count in 2..50usize) {
        let mut v: Vec<ScanResult> = vec![r; count];
        v.dedup();
        prop_assert_eq!(v.len(), 1);
        prop_assert_eq!(v[0], r);
    }

    #[test]
    fn sort_by_bytes_consumed(lengths in prop::collection::vec(any::<usize>(), 1..50)) {
        let mut results: Vec<ScanResult> = lengths
            .iter()
            .map(|&bc| ScanResult { token_type: 0, bytes_consumed: bc })
            .collect();
        results.sort_by_key(|r| r.bytes_consumed);
        for i in 1..results.len() {
            prop_assert!(results[i - 1].bytes_consumed <= results[i].bytes_consumed);
        }
    }
}

// ---------------------------------------------------------------------------
// 9 – Option<ScanResult> patterns
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn some_preserves_value(r in arb_scan_result()) {
        let opt: Option<ScanResult> = Some(r);
        prop_assert!(opt.is_some());
        prop_assert_eq!(opt, Some(r));
    }

    #[test]
    fn none_is_distinct(_ in 0..1u8) {
        let opt: Option<ScanResult> = None;
        prop_assert!(opt.is_none());
    }
}

// ---------------------------------------------------------------------------
// 10 – Memory layout
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn size_is_expected(_ in 0..1u8) {
        // u32 (4 bytes) + usize (8 bytes on 64-bit) + padding
        let size = std::mem::size_of::<ScanResult>();
        prop_assert!(size >= 4 + std::mem::size_of::<usize>());
        prop_assert!(size <= 24); // generous upper bound
    }

    #[test]
    fn alignment_at_least_4(_ in 0..1u8) {
        prop_assert!(std::mem::align_of::<ScanResult>() >= 4);
    }
}
