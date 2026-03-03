#![allow(clippy::needless_range_loop)]

use proptest::prelude::*;

use adze_runtime::Token;

// ---------------------------------------------------------------------------
// Strategies
// ---------------------------------------------------------------------------

fn arb_token() -> impl Strategy<Value = Token> {
    (any::<u32>(), any::<u32>(), any::<u32>()).prop_map(|(kind, start, end)| Token {
        kind,
        start,
        end,
    })
}

fn arb_valid_span_token() -> impl Strategy<Value = Token> {
    (any::<u32>(), any::<u32>()).prop_flat_map(|(kind, start)| {
        (Just(kind), Just(start), start..=u32::MAX).prop_map(|(kind, start, end)| Token {
            kind,
            start,
            end,
        })
    })
}

// ---------------------------------------------------------------------------
// 1 – Construction and field access
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn fields_roundtrip(kind in any::<u32>(), start in any::<u32>(), end in any::<u32>()) {
        let t = Token { kind, start, end };
        prop_assert_eq!(t.kind, kind);
        prop_assert_eq!(t.start, start);
        prop_assert_eq!(t.end, end);
    }

    #[test]
    fn zero_length_token(kind in any::<u32>(), pos in any::<u32>()) {
        let t = Token { kind, start: pos, end: pos };
        prop_assert_eq!(t.start, t.end);
    }
}

// ---------------------------------------------------------------------------
// 2 – Copy semantics
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn copy_preserves_all_fields(t in arb_token()) {
        let t2 = t;      // copy
        let t3 = t;      // still valid
        prop_assert_eq!(t, t2);
        prop_assert_eq!(t, t3);
    }

    #[test]
    fn copy_is_independent(kind in any::<u32>(), start in any::<u32>(), end in any::<u32>()) {
        let t1 = Token { kind, start, end };
        let mut t2 = t1;
        t2.kind = kind.wrapping_add(1);
        // original unchanged
        prop_assert_eq!(t1.kind, kind);
        prop_assert_ne!(t1.kind, t2.kind);
    }
}

// ---------------------------------------------------------------------------
// 3 – Clone semantics
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn clone_equals_original(t in arb_token()) {
        let cloned = t.clone();
        prop_assert_eq!(t, cloned);
    }

    #[test]
    fn clone_is_independent(t in arb_token()) {
        let mut cloned = t.clone();
        cloned.start = t.start.wrapping_add(1);
        prop_assert_ne!(t.start, cloned.start);
    }
}

// ---------------------------------------------------------------------------
// 4 – PartialEq / Eq
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn eq_reflexive(t in arb_token()) {
        prop_assert_eq!(t, t);
    }

    #[test]
    fn eq_symmetric(a in arb_token(), b in arb_token()) {
        prop_assert_eq!(a == b, b == a);
    }

    #[test]
    fn eq_by_all_fields(
        kind in any::<u32>(),
        start in any::<u32>(),
        end in any::<u32>(),
    ) {
        let a = Token { kind, start, end };
        let b = Token { kind, start, end };
        prop_assert_eq!(a, b);
    }

    #[test]
    fn ne_when_kind_differs(
        kind_a in any::<u32>(),
        kind_b in any::<u32>(),
        start in any::<u32>(),
        end in any::<u32>(),
    ) {
        let a = Token { kind: kind_a, start, end };
        let b = Token { kind: kind_b, start, end };
        prop_assert_eq!(a == b, kind_a == kind_b);
    }

    #[test]
    fn ne_when_start_differs(
        kind in any::<u32>(),
        start_a in any::<u32>(),
        start_b in any::<u32>(),
        end in any::<u32>(),
    ) {
        let a = Token { kind, start: start_a, end };
        let b = Token { kind, start: start_b, end };
        prop_assert_eq!(a == b, start_a == start_b);
    }

    #[test]
    fn ne_when_end_differs(
        kind in any::<u32>(),
        start in any::<u32>(),
        end_a in any::<u32>(),
        end_b in any::<u32>(),
    ) {
        let a = Token { kind, start, end: end_a };
        let b = Token { kind, start, end: end_b };
        prop_assert_eq!(a == b, end_a == end_b);
    }
}

// ---------------------------------------------------------------------------
// 5 – Debug formatting
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn debug_contains_struct_name(t in arb_token()) {
        let dbg = format!("{:?}", t);
        prop_assert!(dbg.contains("Token"));
    }

    #[test]
    fn debug_contains_all_field_values(t in arb_token()) {
        let dbg = format!("{:?}", t);
        prop_assert!(dbg.contains(&t.kind.to_string()));
        prop_assert!(dbg.contains(&t.start.to_string()));
        prop_assert!(dbg.contains(&t.end.to_string()));
    }

    #[test]
    fn debug_is_nonempty(t in arb_token()) {
        let dbg = format!("{:?}", t);
        prop_assert!(!dbg.is_empty());
    }
}

// ---------------------------------------------------------------------------
// 6 – Boundary values
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn max_values_token(_ in 0..1u8) {
        let t = Token { kind: u32::MAX, start: u32::MAX, end: u32::MAX };
        let t2 = t;
        prop_assert_eq!(t, t2);
    }

    #[test]
    fn min_values_token(_ in 0..1u8) {
        let t = Token { kind: 0, start: 0, end: 0 };
        let t2 = t;
        prop_assert_eq!(t, t2);
    }

    #[test]
    fn boundary_field_values(
        kind in prop_oneof![Just(0u32), Just(u32::MAX), any::<u32>()],
        start in prop_oneof![Just(0u32), Just(u32::MAX), any::<u32>()],
        end in prop_oneof![Just(0u32), Just(u32::MAX), any::<u32>()],
    ) {
        let t = Token { kind, start, end };
        let cloned = t.clone();
        prop_assert_eq!(t, cloned);
        let _ = format!("{:?}", t);
    }
}

// ---------------------------------------------------------------------------
// 7 – Collection behaviour
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn tokens_in_vec(count in 1..100usize, t in arb_token()) {
        let v: Vec<Token> = vec![t; count];
        prop_assert_eq!(v.len(), count);
        for i in 0..v.len() {
            prop_assert_eq!(v[i], t);
        }
    }

    #[test]
    fn vec_dedup_identical(t in arb_token(), count in 2..50usize) {
        let mut v: Vec<Token> = vec![t; count];
        v.dedup();
        prop_assert_eq!(v.len(), 1);
        prop_assert_eq!(v[0], t);
    }

    #[test]
    fn vec_dedup_distinct(
        kind_a in 0..u32::MAX,
        kind_b in 0..u32::MAX,
    ) {
        let a = Token { kind: kind_a, start: 0, end: 1 };
        let b = Token { kind: kind_b, start: 0, end: 1 };
        let mut v = vec![a, b];
        v.dedup();
        if kind_a == kind_b {
            prop_assert_eq!(v.len(), 1);
        } else {
            prop_assert_eq!(v.len(), 2);
        }
    }

    #[test]
    fn sort_by_start(starts in prop::collection::vec(any::<u32>(), 1..50)) {
        let mut tokens: Vec<Token> = starts
            .iter()
            .map(|&s| Token { kind: 0, start: s, end: s.saturating_add(1) })
            .collect();
        tokens.sort_by_key(|t| t.start);
        for i in 1..tokens.len() {
            prop_assert!(tokens[i - 1].start <= tokens[i].start);
        }
    }
}

// ---------------------------------------------------------------------------
// 8 – Span arithmetic properties
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn valid_span_length(t in arb_valid_span_token()) {
        let len = t.end - t.start;
        prop_assert!(t.end >= t.start);
        prop_assert_eq!(len, t.end.wrapping_sub(t.start));
    }

    #[test]
    fn span_contains_start(t in arb_valid_span_token()) {
        if t.start < t.end {
            prop_assert!(t.start >= t.start && t.start < t.end);
        }
    }

    #[test]
    fn adjacent_tokens_no_gap(
        kind in any::<u32>(),
        start in 0..u32::MAX / 2,
        len1 in 1..1000u32,
        len2 in 1..1000u32,
    ) {
        let t1 = Token { kind, start, end: start + len1 };
        let t2 = Token { kind, start: t1.end, end: t1.end + len2 };
        prop_assert_eq!(t1.end, t2.start);
    }
}

// ---------------------------------------------------------------------------
// 9 – Memory layout and size
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn size_is_12_bytes(_ in 0..1u8) {
        prop_assert_eq!(std::mem::size_of::<Token>(), 12);
    }

    #[test]
    fn alignment_is_4(_ in 0..1u8) {
        prop_assert_eq!(std::mem::align_of::<Token>(), 4);
    }
}
