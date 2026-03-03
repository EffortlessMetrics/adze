#![allow(clippy::needless_range_loop)]

use adze_glr_core::ts_lexer::NextToken;
use proptest::prelude::*;

// ---------------------------------------------------------------------------
// Strategies
// ---------------------------------------------------------------------------

fn any_token() -> impl Strategy<Value = NextToken> {
    (any::<u32>(), any::<u32>(), any::<u32>()).prop_map(|(kind, start, end)| NextToken {
        kind,
        start,
        end,
    })
}

fn valid_span_token() -> impl Strategy<Value = NextToken> {
    (any::<u32>(), 0..=u32::MAX / 2).prop_flat_map(|(kind, start)| {
        (Just(kind), Just(start), start..=u32::MAX).prop_map(|(kind, start, end)| NextToken {
            kind,
            start,
            end,
        })
    })
}

fn zero_length_token() -> impl Strategy<Value = NextToken> {
    (any::<u32>(), any::<u32>()).prop_map(|(kind, pos)| NextToken {
        kind,
        start: pos,
        end: pos,
    })
}

// ---------------------------------------------------------------------------
// 1. Token creation
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn test_token_creation_preserves_fields(kind in any::<u32>(), start in any::<u32>(), end in any::<u32>()) {
        let tok = NextToken { kind, start, end };
        prop_assert_eq!(tok.kind, kind);
        prop_assert_eq!(tok.start, start);
        prop_assert_eq!(tok.end, end);
    }
}

#[test]
fn test_token_creation_all_zeros() {
    let tok = NextToken {
        kind: 0,
        start: 0,
        end: 0,
    };
    assert_eq!(tok.kind, 0);
    assert_eq!(tok.start, 0);
    assert_eq!(tok.end, 0);
}

#[test]
fn test_token_creation_max_values() {
    let tok = NextToken {
        kind: u32::MAX,
        start: u32::MAX,
        end: u32::MAX,
    };
    assert_eq!(tok.kind, u32::MAX);
    assert_eq!(tok.start, u32::MAX);
    assert_eq!(tok.end, u32::MAX);
}

// ---------------------------------------------------------------------------
// 2. Token field access
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn test_field_access_kind(tok in any_token()) {
        let _k: u32 = tok.kind;
        prop_assert!(true);
    }
}

proptest! {
    #[test]
    fn test_field_access_start(tok in any_token()) {
        let _s: u32 = tok.start;
        prop_assert!(true);
    }
}

proptest! {
    #[test]
    fn test_field_access_end(tok in any_token()) {
        let _e: u32 = tok.end;
        prop_assert!(true);
    }
}

proptest! {
    #[test]
    fn test_field_access_span_length(tok in valid_span_token()) {
        let len = tok.end - tok.start;
        prop_assert!(tok.end >= tok.start);
        prop_assert_eq!(len, tok.end - tok.start);
    }
}

// ---------------------------------------------------------------------------
// 3. Token ordering (by position)
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn test_ordering_by_start(a in valid_span_token(), b in valid_span_token()) {
        if a.start < b.start {
            prop_assert!(a.start < b.start);
        } else if a.start > b.start {
            prop_assert!(a.start > b.start);
        } else {
            prop_assert_eq!(a.start, b.start);
        }
    }
}

proptest! {
    #[test]
    fn test_ordering_sort_by_start(tokens in proptest::collection::vec(valid_span_token(), 1..50)) {
        let mut sorted = tokens.clone();
        sorted.sort_by_key(|t| t.start);
        for i in 1..sorted.len() {
            prop_assert!(sorted[i - 1].start <= sorted[i].start);
        }
    }
}

proptest! {
    #[test]
    fn test_ordering_sort_by_end(tokens in proptest::collection::vec(valid_span_token(), 1..50)) {
        let mut sorted = tokens.clone();
        sorted.sort_by_key(|t| t.end);
        for i in 1..sorted.len() {
            prop_assert!(sorted[i - 1].end <= sorted[i].end);
        }
    }
}

proptest! {
    #[test]
    fn test_ordering_sort_by_kind(tokens in proptest::collection::vec(any_token(), 1..50)) {
        let mut sorted = tokens.clone();
        sorted.sort_by_key(|t| t.kind);
        for i in 1..sorted.len() {
            prop_assert!(sorted[i - 1].kind <= sorted[i].kind);
        }
    }
}

proptest! {
    #[test]
    fn test_ordering_composite_sort(tokens in proptest::collection::vec(valid_span_token(), 1..50)) {
        let mut sorted = tokens.clone();
        sorted.sort_by_key(|t| (t.start, t.end, t.kind));
        for i in 1..sorted.len() {
            let a = (sorted[i - 1].start, sorted[i - 1].end, sorted[i - 1].kind);
            let b = (sorted[i].start, sorted[i].end, sorted[i].kind);
            prop_assert!(a <= b);
        }
    }
}

// ---------------------------------------------------------------------------
// 4. Token clone/copy
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn test_clone_equals_original(tok in any_token()) {
        let cloned = tok.clone();
        prop_assert_eq!(tok.kind, cloned.kind);
        prop_assert_eq!(tok.start, cloned.start);
        prop_assert_eq!(tok.end, cloned.end);
    }
}

proptest! {
    #[test]
    fn test_copy_equals_original(tok in any_token()) {
        let copied = tok;
        prop_assert_eq!(tok.kind, copied.kind);
        prop_assert_eq!(tok.start, copied.start);
        prop_assert_eq!(tok.end, copied.end);
    }
}

proptest! {
    #[test]
    fn test_copy_is_independent(kind in any::<u32>(), start in any::<u32>(), end in any::<u32>()) {
        let tok = NextToken { kind, start, end };
        let mut copied = tok;
        copied.kind = kind.wrapping_add(1);
        prop_assert_eq!(tok.kind, kind);
        prop_assert_eq!(copied.kind, kind.wrapping_add(1));
    }
}

// ---------------------------------------------------------------------------
// 5. Token debug display
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn test_debug_contains_struct_name(tok in any_token()) {
        let dbg = format!("{:?}", tok);
        prop_assert!(dbg.contains("NextToken"));
    }
}

proptest! {
    #[test]
    fn test_debug_contains_field_values(kind in 0..1000u32, start in 0..1000u32, end in 0..1000u32) {
        let tok = NextToken { kind, start, end };
        let dbg = format!("{:?}", tok);
        prop_assert!(dbg.contains(&kind.to_string()));
        prop_assert!(dbg.contains(&start.to_string()));
        prop_assert!(dbg.contains(&end.to_string()));
    }
}

proptest! {
    #[test]
    fn test_debug_is_nonempty(tok in any_token()) {
        let dbg = format!("{:?}", tok);
        prop_assert!(!dbg.is_empty());
    }
}

// ---------------------------------------------------------------------------
// 6. Zero-length tokens
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn test_zero_length_token_start_equals_end(tok in zero_length_token()) {
        prop_assert_eq!(tok.start, tok.end);
        prop_assert_eq!(tok.end - tok.start, 0);
    }
}

proptest! {
    #[test]
    fn test_zero_length_token_any_kind(kind in any::<u32>(), pos in any::<u32>()) {
        let tok = NextToken { kind, start: pos, end: pos };
        prop_assert_eq!(tok.start, tok.end);
        prop_assert_eq!(tok.kind, kind);
    }
}

#[test]
fn test_zero_length_token_at_boundaries() {
    for pos in [0u32, 1, u32::MAX / 2, u32::MAX - 1, u32::MAX] {
        let tok = NextToken {
            kind: 0,
            start: pos,
            end: pos,
        };
        assert_eq!(tok.start, pos);
        assert_eq!(tok.end, pos);
    }
}

// ---------------------------------------------------------------------------
// 7. Overlapping tokens
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn test_overlapping_detection(
        start_a in 0..1000u32,
        len_a in 1..100u32,
        start_b in 0..1000u32,
        len_b in 1..100u32,
        kind_a in any::<u32>(),
        kind_b in any::<u32>(),
    ) {
        let a = NextToken { kind: kind_a, start: start_a, end: start_a + len_a };
        let b = NextToken { kind: kind_b, start: start_b, end: start_b + len_b };
        let overlaps = a.start < b.end && b.start < a.end;
        if overlaps {
            prop_assert!(a.start < b.end);
            prop_assert!(b.start < a.end);
        } else {
            prop_assert!(a.end <= b.start || b.end <= a.start);
        }
    }
}

proptest! {
    #[test]
    fn test_adjacent_tokens_not_overlapping(
        pos in 0..u32::MAX / 2,
        len_a in 1..100u32,
        len_b in 1..100u32,
    ) {
        let a = NextToken { kind: 0, start: pos, end: pos + len_a };
        let b = NextToken { kind: 1, start: pos + len_a, end: pos + len_a + len_b };
        prop_assert!(a.end == b.start);
        prop_assert!(!(a.start < b.end && b.start < a.end) || a.end == b.start);
    }
}

proptest! {
    #[test]
    fn test_containing_token(
        outer_start in 0..500u32,
        inner_offset in 1..50u32,
        inner_len in 1..50u32,
        outer_extra in 1..50u32,
    ) {
        let outer = NextToken {
            kind: 0,
            start: outer_start,
            end: outer_start + inner_offset + inner_len + outer_extra,
        };
        let inner = NextToken {
            kind: 1,
            start: outer_start + inner_offset,
            end: outer_start + inner_offset + inner_len,
        };
        prop_assert!(inner.start >= outer.start);
        prop_assert!(inner.end <= outer.end);
    }
}

// ---------------------------------------------------------------------------
// 8. Token sequences
// ---------------------------------------------------------------------------

proptest! {
    #[test]
    fn test_sequence_non_overlapping(
        count in 1..20usize,
        kind in any::<u32>(),
    ) {
        let tokens: Vec<NextToken> = (0..count)
            .map(|i| {
                let start = (i as u32) * 10;
                NextToken { kind, start, end: start + 5 }
            })
            .collect();
        for i in 1..tokens.len() {
            prop_assert!(tokens[i - 1].end <= tokens[i].start);
        }
    }
}

proptest! {
    #[test]
    fn test_sequence_contiguous(
        count in 1..30usize,
        token_len in 1..10u32,
    ) {
        let tokens: Vec<NextToken> = (0..count)
            .map(|i| {
                let start = (i as u32) * token_len;
                NextToken { kind: i as u32, start, end: start + token_len }
            })
            .collect();
        for i in 1..tokens.len() {
            prop_assert_eq!(tokens[i - 1].end, tokens[i].start, "sequence must be contiguous");
        }
    }
}

proptest! {
    #[test]
    fn test_sequence_total_span(
        count in 1..20usize,
        token_len in 1..10u32,
    ) {
        let tokens: Vec<NextToken> = (0..count)
            .map(|i| {
                let start = (i as u32) * token_len;
                NextToken { kind: 0, start, end: start + token_len }
            })
            .collect();
        let total_start = tokens.first().unwrap().start;
        let total_end = tokens.last().unwrap().end;
        prop_assert_eq!(total_end - total_start, (count as u32) * token_len);
    }
}

proptest! {
    #[test]
    fn test_sequence_sorted_is_stable(tokens in proptest::collection::vec(valid_span_token(), 2..30)) {
        let mut by_start = tokens.clone();
        by_start.sort_by_key(|t| (t.start, t.end));
        let mut by_start2 = by_start.clone();
        by_start2.sort_by_key(|t| (t.start, t.end));
        for i in 0..by_start.len() {
            prop_assert_eq!(by_start[i].start, by_start2[i].start);
            prop_assert_eq!(by_start[i].end, by_start2[i].end);
            prop_assert_eq!(by_start[i].kind, by_start2[i].kind);
        }
    }
}

proptest! {
    #[test]
    fn test_sequence_partition_by_kind(tokens in proptest::collection::vec(
        (0..5u32, 0..1000u32, 0..1000u32).prop_map(|(k, s, e)| NextToken { kind: k, start: s, end: e }),
        5..30
    )) {
        let mut groups: std::collections::HashMap<u32, Vec<&NextToken>> = std::collections::HashMap::new();
        for tok in &tokens {
            groups.entry(tok.kind).or_default().push(tok);
        }
        let total: usize = groups.values().map(|v| v.len()).sum();
        prop_assert_eq!(total, tokens.len());
    }
}

proptest! {
    #[test]
    fn test_sequence_reverse_preserves_elements(tokens in proptest::collection::vec(any_token(), 1..30)) {
        let mut reversed = tokens.clone();
        reversed.reverse();
        prop_assert_eq!(reversed.len(), tokens.len());
        for i in 0..tokens.len() {
            prop_assert_eq!(tokens[i].kind, reversed[tokens.len() - 1 - i].kind);
            prop_assert_eq!(tokens[i].start, reversed[tokens.len() - 1 - i].start);
            prop_assert_eq!(tokens[i].end, reversed[tokens.len() - 1 - i].end);
        }
    }
}

proptest! {
    #[test]
    fn test_sequence_filter_by_kind(
        tokens in proptest::collection::vec(
            (0..3u32, 0..1000u32, 0..1000u32).prop_map(|(k, s, e)| NextToken { kind: k, start: s, end: e }),
            5..30
        ),
        target_kind in 0..3u32,
    ) {
        let filtered: Vec<_> = tokens.iter().filter(|t| t.kind == target_kind).collect();
        for t in &filtered {
            prop_assert_eq!(t.kind, target_kind);
        }
        let rest: Vec<_> = tokens.iter().filter(|t| t.kind != target_kind).collect();
        prop_assert_eq!(filtered.len() + rest.len(), tokens.len());
    }
}
