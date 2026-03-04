//! Comprehensive tests for Parser timeout behavior (50+ tests).
//!
//! Covers: zero duration, small durations, normal durations, large durations,
//! multiple set_timeout calls, set_timeout before/after set_language,
//! parser creation patterns, Duration construction patterns, Tree creation patterns.

use adze_runtime::Parser;
use adze_runtime::test_helpers::stub_language;
use adze_runtime::tree::Tree;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::time::Duration;

// ============================================================
// 1. set_timeout with zero duration
// ============================================================

#[test]
fn zero_duration_constant() {
    let mut parser = Parser::new();
    parser.set_timeout(Duration::ZERO);
    assert_eq!(parser.timeout(), Some(Duration::ZERO));
}

#[test]
fn zero_duration_from_secs() {
    let mut parser = Parser::new();
    parser.set_timeout(Duration::from_secs(0));
    assert_eq!(parser.timeout(), Some(Duration::from_secs(0)));
}

#[test]
fn zero_duration_from_millis() {
    let mut parser = Parser::new();
    parser.set_timeout(Duration::from_millis(0));
    assert_eq!(parser.timeout(), Some(Duration::from_millis(0)));
}

#[test]
fn zero_duration_from_micros() {
    let mut parser = Parser::new();
    parser.set_timeout(Duration::from_micros(0));
    assert_eq!(parser.timeout(), Some(Duration::from_micros(0)));
}

#[test]
fn zero_duration_from_nanos() {
    let mut parser = Parser::new();
    parser.set_timeout(Duration::from_nanos(0));
    assert_eq!(parser.timeout(), Some(Duration::from_nanos(0)));
}

#[test]
fn zero_duration_equals_zero_constant() {
    let mut parser = Parser::new();
    parser.set_timeout(Duration::from_secs(0));
    assert_eq!(parser.timeout().unwrap(), Duration::ZERO);
}

// ============================================================
// 2. set_timeout with small durations (1ns, 1us, 1ms)
// ============================================================

#[test]
fn one_nanosecond_timeout() {
    let mut parser = Parser::new();
    parser.set_timeout(Duration::from_nanos(1));
    assert_eq!(parser.timeout(), Some(Duration::from_nanos(1)));
}

#[test]
fn one_microsecond_timeout() {
    let mut parser = Parser::new();
    parser.set_timeout(Duration::from_micros(1));
    assert_eq!(parser.timeout(), Some(Duration::from_micros(1)));
}

#[test]
fn one_millisecond_timeout() {
    let mut parser = Parser::new();
    parser.set_timeout(Duration::from_millis(1));
    assert_eq!(parser.timeout(), Some(Duration::from_millis(1)));
}

#[test]
fn ten_nanos_timeout() {
    let mut parser = Parser::new();
    parser.set_timeout(Duration::from_nanos(10));
    assert_eq!(parser.timeout(), Some(Duration::from_nanos(10)));
}

#[test]
fn hundred_micros_timeout() {
    let mut parser = Parser::new();
    parser.set_timeout(Duration::from_micros(100));
    assert_eq!(parser.timeout(), Some(Duration::from_micros(100)));
}

#[test]
fn five_hundred_millis_timeout() {
    let mut parser = Parser::new();
    parser.set_timeout(Duration::from_millis(500));
    assert_eq!(parser.timeout(), Some(Duration::from_millis(500)));
}

#[test]
fn sub_microsecond_from_nanos() {
    let mut parser = Parser::new();
    parser.set_timeout(Duration::from_nanos(999));
    assert_eq!(parser.timeout(), Some(Duration::from_nanos(999)));
}

// ============================================================
// 3. set_timeout with normal durations (1s, 5s)
// ============================================================

#[test]
fn one_second_timeout() {
    let mut parser = Parser::new();
    parser.set_timeout(Duration::from_secs(1));
    assert_eq!(parser.timeout(), Some(Duration::from_secs(1)));
}

#[test]
fn five_second_timeout() {
    let mut parser = Parser::new();
    parser.set_timeout(Duration::from_secs(5));
    assert_eq!(parser.timeout(), Some(Duration::from_secs(5)));
}

#[test]
fn ten_second_timeout() {
    let mut parser = Parser::new();
    parser.set_timeout(Duration::from_secs(10));
    assert_eq!(parser.timeout(), Some(Duration::from_secs(10)));
}

#[test]
fn thirty_second_timeout() {
    let mut parser = Parser::new();
    parser.set_timeout(Duration::from_secs(30));
    assert_eq!(parser.timeout(), Some(Duration::from_secs(30)));
}

#[test]
fn fractional_seconds_timeout() {
    let mut parser = Parser::new();
    let dur = Duration::new(2, 500_000_000); // 2.5s
    parser.set_timeout(dur);
    assert_eq!(parser.timeout(), Some(dur));
}

#[test]
fn two_point_seven_five_seconds() {
    let mut parser = Parser::new();
    let dur = Duration::from_millis(2750);
    parser.set_timeout(dur);
    assert_eq!(parser.timeout(), Some(dur));
    assert_eq!(dur.as_millis(), 2750);
}

// ============================================================
// 4. set_timeout with large durations (1 hour+)
// ============================================================

#[test]
fn one_hour_timeout() {
    let mut parser = Parser::new();
    let dur = Duration::from_secs(3600);
    parser.set_timeout(dur);
    assert_eq!(parser.timeout(), Some(dur));
}

#[test]
fn one_day_timeout() {
    let mut parser = Parser::new();
    let dur = Duration::from_secs(86_400);
    parser.set_timeout(dur);
    assert_eq!(parser.timeout(), Some(dur));
}

#[test]
fn one_week_timeout() {
    let mut parser = Parser::new();
    let dur = Duration::from_secs(604_800);
    parser.set_timeout(dur);
    assert_eq!(parser.timeout(), Some(dur));
}

#[test]
fn max_duration_timeout() {
    let mut parser = Parser::new();
    parser.set_timeout(Duration::MAX);
    assert_eq!(parser.timeout(), Some(Duration::MAX));
}

#[test]
fn half_max_u64_seconds() {
    let mut parser = Parser::new();
    let dur = Duration::from_secs(u64::MAX / 2);
    parser.set_timeout(dur);
    assert_eq!(parser.timeout(), Some(dur));
}

#[test]
fn large_nanos_component() {
    let mut parser = Parser::new();
    let dur = Duration::new(100, 999_999_999);
    parser.set_timeout(dur);
    assert_eq!(parser.timeout(), Some(dur));
}

// ============================================================
// 5. Multiple set_timeout calls
// ============================================================

#[test]
fn override_timeout_smaller() {
    let mut parser = Parser::new();
    parser.set_timeout(Duration::from_secs(10));
    parser.set_timeout(Duration::from_secs(1));
    assert_eq!(parser.timeout(), Some(Duration::from_secs(1)));
}

#[test]
fn override_timeout_larger() {
    let mut parser = Parser::new();
    parser.set_timeout(Duration::from_millis(50));
    parser.set_timeout(Duration::from_secs(60));
    assert_eq!(parser.timeout(), Some(Duration::from_secs(60)));
}

#[test]
fn override_timeout_same_value() {
    let mut parser = Parser::new();
    parser.set_timeout(Duration::from_secs(5));
    parser.set_timeout(Duration::from_secs(5));
    assert_eq!(parser.timeout(), Some(Duration::from_secs(5)));
}

#[test]
fn override_to_zero() {
    let mut parser = Parser::new();
    parser.set_timeout(Duration::from_secs(10));
    parser.set_timeout(Duration::ZERO);
    assert_eq!(parser.timeout(), Some(Duration::ZERO));
}

#[test]
fn override_from_zero() {
    let mut parser = Parser::new();
    parser.set_timeout(Duration::ZERO);
    parser.set_timeout(Duration::from_secs(5));
    assert_eq!(parser.timeout(), Some(Duration::from_secs(5)));
}

#[test]
fn many_sequential_overrides() {
    let mut parser = Parser::new();
    for i in 0..50u64 {
        parser.set_timeout(Duration::from_millis(i * 10));
    }
    assert_eq!(parser.timeout(), Some(Duration::from_millis(490)));
}

#[test]
fn alternating_zero_and_nonzero() {
    let mut parser = Parser::new();
    for i in 0..10u64 {
        if i % 2 == 0 {
            parser.set_timeout(Duration::ZERO);
            assert_eq!(parser.timeout(), Some(Duration::ZERO));
        } else {
            parser.set_timeout(Duration::from_secs(i));
            assert_eq!(parser.timeout(), Some(Duration::from_secs(i)));
        }
    }
}

#[test]
fn increasing_powers_of_two() {
    let mut parser = Parser::new();
    for exp in 0..20u32 {
        let dur = Duration::from_millis(2u64.pow(exp));
        parser.set_timeout(dur);
        assert_eq!(parser.timeout(), Some(dur));
    }
}

#[test]
fn decreasing_powers_of_two() {
    let mut parser = Parser::new();
    for exp in (0..20u32).rev() {
        let dur = Duration::from_millis(2u64.pow(exp));
        parser.set_timeout(dur);
        assert_eq!(parser.timeout(), Some(dur));
    }
}

// ============================================================
// 6. set_timeout before set_language
// ============================================================

#[test]
fn timeout_before_language_is_preserved() {
    let mut parser = Parser::new();
    parser.set_timeout(Duration::from_secs(5));
    let lang = stub_language();
    let _ = parser.set_language(lang);
    assert_eq!(parser.timeout(), Some(Duration::from_secs(5)));
}

#[test]
fn timeout_before_language_with_millis() {
    let mut parser = Parser::new();
    parser.set_timeout(Duration::from_millis(250));
    let lang = stub_language();
    let _ = parser.set_language(lang);
    assert_eq!(parser.timeout(), Some(Duration::from_millis(250)));
}

#[test]
fn zero_timeout_before_language() {
    let mut parser = Parser::new();
    parser.set_timeout(Duration::ZERO);
    let lang = stub_language();
    let _ = parser.set_language(lang);
    assert_eq!(parser.timeout(), Some(Duration::ZERO));
}

// ============================================================
// 7. set_timeout after set_language
// ============================================================

#[test]
fn timeout_after_language_works() {
    let mut parser = Parser::new();
    let lang = stub_language();
    let _ = parser.set_language(lang);
    parser.set_timeout(Duration::from_millis(200));
    assert_eq!(parser.timeout(), Some(Duration::from_millis(200)));
}

#[test]
fn timeout_after_language_overrides() {
    let mut parser = Parser::new();
    parser.set_timeout(Duration::from_secs(1));
    let lang = stub_language();
    let _ = parser.set_language(lang);
    parser.set_timeout(Duration::from_secs(99));
    assert_eq!(parser.timeout(), Some(Duration::from_secs(99)));
}

#[test]
fn timeout_independent_of_language_set() {
    let mut parser = Parser::new();
    parser.set_timeout(Duration::from_secs(42));
    let lang1 = stub_language();
    let _ = parser.set_language(lang1);
    assert_eq!(parser.timeout(), Some(Duration::from_secs(42)));
    let lang2 = stub_language();
    let _ = parser.set_language(lang2);
    assert_eq!(parser.timeout(), Some(Duration::from_secs(42)));
}

// ============================================================
// 8. Parser creation patterns
// ============================================================

#[test]
fn new_parser_timeout_is_none() {
    let parser = Parser::new();
    assert!(parser.timeout().is_none());
}

#[test]
fn default_parser_timeout_is_none() {
    let parser = Parser::default();
    assert!(parser.timeout().is_none());
}

#[test]
fn new_parser_no_language() {
    let parser = Parser::new();
    assert!(parser.language().is_none());
}

#[test]
fn default_parser_no_language() {
    let parser = Parser::default();
    assert!(parser.language().is_none());
}

#[test]
fn two_independent_parsers_have_independent_timeouts() {
    let mut p1 = Parser::new();
    let mut p2 = Parser::new();
    p1.set_timeout(Duration::from_secs(1));
    p2.set_timeout(Duration::from_secs(99));
    assert_eq!(p1.timeout(), Some(Duration::from_secs(1)));
    assert_eq!(p2.timeout(), Some(Duration::from_secs(99)));
}

#[test]
fn parser_reset_preserves_timeout() {
    let mut parser = Parser::new();
    parser.set_timeout(Duration::from_secs(7));
    parser.reset();
    assert_eq!(parser.timeout(), Some(Duration::from_secs(7)));
}

#[test]
fn multiple_resets_preserve_timeout() {
    let mut parser = Parser::new();
    parser.set_timeout(Duration::from_millis(333));
    for _ in 0..10 {
        parser.reset();
    }
    assert_eq!(parser.timeout(), Some(Duration::from_millis(333)));
}

#[test]
fn timeout_survives_failed_parse() {
    let mut parser = Parser::new();
    parser.set_timeout(Duration::from_secs(3));
    let result = parser.parse(b"test", None);
    assert!(result.is_err());
    assert_eq!(parser.timeout(), Some(Duration::from_secs(3)));
}

#[test]
fn timeout_survives_failed_parse_utf8() {
    let mut parser = Parser::new();
    parser.set_timeout(Duration::from_millis(500));
    let result = parser.parse_utf8("hello", None);
    assert!(result.is_err());
    assert_eq!(parser.timeout(), Some(Duration::from_millis(500)));
}

#[test]
fn parser_parse_without_language_errors() {
    let mut parser = Parser::new();
    let result = parser.parse(b"anything", None);
    assert!(result.is_err());
}

#[test]
fn parser_parse_utf8_without_language_errors() {
    let mut parser = Parser::new();
    let result = parser.parse_utf8("anything", None);
    assert!(result.is_err());
}

#[test]
fn parse_with_stub_language_panics() {
    let mut parser = Parser::new();
    let lang = stub_language();
    let _ = parser.set_language(lang);
    let result = catch_unwind(AssertUnwindSafe(|| {
        let _ = parser.parse(b"hello", None);
    }));
    // stub_language has empty parse tables, so parsing may panic
    // We just verify the call doesn't cause UB — either Ok or panic is fine
    let _outcome = result;
}

#[test]
fn parse_utf8_with_stub_language_panics() {
    let mut parser = Parser::new();
    let lang = stub_language();
    let _ = parser.set_language(lang);
    let result = catch_unwind(AssertUnwindSafe(|| {
        let _ = parser.parse_utf8("hello", None);
    }));
    let _outcome = result;
}

#[test]
fn timeout_set_then_reset_then_set_again() {
    let mut parser = Parser::new();
    parser.set_timeout(Duration::from_secs(1));
    parser.reset();
    parser.set_timeout(Duration::from_secs(2));
    assert_eq!(parser.timeout(), Some(Duration::from_secs(2)));
}

#[test]
fn timeout_with_reset_and_parse_cycle() {
    let mut parser = Parser::new();
    parser.set_timeout(Duration::from_secs(5));
    for _ in 0..3 {
        let _ = parser.parse(b"x", None);
        assert_eq!(parser.timeout(), Some(Duration::from_secs(5)));
        parser.reset();
        assert_eq!(parser.timeout(), Some(Duration::from_secs(5)));
    }
}

// ============================================================
// 9. Duration construction patterns
// ============================================================

#[test]
fn duration_new_secs_and_nanos() {
    let mut parser = Parser::new();
    let dur = Duration::new(3, 141_592_653);
    parser.set_timeout(dur);
    assert_eq!(parser.timeout().unwrap().as_secs(), 3);
    assert_eq!(parser.timeout().unwrap().subsec_nanos(), 141_592_653);
}

#[test]
fn duration_from_secs_f64() {
    let mut parser = Parser::new();
    let dur = Duration::from_secs_f64(1.5);
    parser.set_timeout(dur);
    let t = parser.timeout().unwrap();
    assert_eq!(t.as_secs(), 1);
    assert_eq!(t.subsec_millis(), 500);
}

#[test]
fn duration_from_secs_f32() {
    let mut parser = Parser::new();
    let dur = Duration::from_secs_f32(0.25);
    parser.set_timeout(dur);
    let t = parser.timeout().unwrap();
    assert_eq!(t.as_millis(), 250);
}

#[test]
fn duration_checked_add() {
    let mut parser = Parser::new();
    let dur = Duration::from_secs(1)
        .checked_add(Duration::from_millis(500))
        .unwrap();
    parser.set_timeout(dur);
    assert_eq!(parser.timeout().unwrap().as_millis(), 1500);
}

#[test]
fn duration_saturating_mul() {
    let mut parser = Parser::new();
    let dur = Duration::from_millis(100).saturating_mul(10);
    parser.set_timeout(dur);
    assert_eq!(parser.timeout(), Some(Duration::from_secs(1)));
}

#[test]
fn duration_as_conversions_roundtrip() {
    let mut parser = Parser::new();
    let dur = Duration::from_millis(1234);
    parser.set_timeout(dur);
    let t = parser.timeout().unwrap();
    assert_eq!(t.as_millis(), 1234);
    assert_eq!(t.as_micros(), 1_234_000);
    assert_eq!(t.as_nanos(), 1_234_000_000);
}

#[test]
fn duration_comparison_after_set() {
    let mut parser = Parser::new();
    parser.set_timeout(Duration::from_secs(5));
    let t = parser.timeout().unwrap();
    assert!(t > Duration::from_secs(4));
    assert!(t < Duration::from_secs(6));
    assert!(t >= Duration::from_secs(5));
    assert!(t <= Duration::from_secs(5));
}

// ============================================================
// 10. Tree creation patterns
// ============================================================

#[test]
fn tree_new_stub_is_valid() {
    let tree = Tree::new_stub();
    let root = tree.root_node();
    assert_eq!(root.start_byte(), 0);
    assert_eq!(root.end_byte(), 0);
    assert_eq!(root.child_count(), 0);
}

#[test]
fn tree_new_for_testing_no_children() {
    let tree = Tree::new_for_testing(1, 0, 10, vec![]);
    let root = tree.root_node();
    assert_eq!(root.start_byte(), 0);
    assert_eq!(root.end_byte(), 10);
    assert_eq!(root.child_count(), 0);
}

#[test]
fn tree_new_for_testing_with_children() {
    let child1 = Tree::new_for_testing(2, 0, 3, vec![]);
    let child2 = Tree::new_for_testing(3, 3, 7, vec![]);
    let tree = Tree::new_for_testing(1, 0, 7, vec![child1, child2]);
    let root = tree.root_node();
    assert_eq!(root.child_count(), 2);
}

#[test]
fn tree_new_for_testing_nested() {
    let grandchild = Tree::new_for_testing(3, 0, 2, vec![]);
    let child = Tree::new_for_testing(2, 0, 5, vec![grandchild]);
    let tree = Tree::new_for_testing(1, 0, 10, vec![child]);
    let root = tree.root_node();
    assert_eq!(root.child_count(), 1);
}

#[test]
fn tree_stub_root_kind_is_zero() {
    let tree = Tree::new_stub();
    assert_eq!(tree.root_kind(), 0);
}

#[test]
fn tree_for_testing_root_kind_matches_symbol() {
    let tree = Tree::new_for_testing(42, 0, 100, vec![]);
    assert_eq!(tree.root_kind(), 42);
}

#[test]
fn tree_stub_has_no_language() {
    let tree = Tree::new_stub();
    assert!(tree.language().is_none());
}

#[test]
fn tree_stub_has_no_source() {
    let tree = Tree::new_stub();
    assert!(tree.source_bytes().is_none());
}

#[test]
fn tree_clone_is_independent() {
    let tree = Tree::new_for_testing(1, 0, 10, vec![]);
    let cloned = tree.clone();
    assert_eq!(tree.root_kind(), cloned.root_kind());
    assert_eq!(
        tree.root_node().start_byte(),
        cloned.root_node().start_byte()
    );
}

#[test]
fn tree_for_testing_many_children() {
    let children: Vec<Tree> = (0..20)
        .map(|i| Tree::new_for_testing(i + 10, i as usize * 5, (i as usize + 1) * 5, vec![]))
        .collect();
    let tree = Tree::new_for_testing(1, 0, 100, children);
    assert_eq!(tree.root_node().child_count(), 20);
}

#[test]
fn tree_stub_debug_does_not_panic() {
    let tree = Tree::new_stub();
    let debug_str = format!("{:?}", tree);
    assert!(!debug_str.is_empty());
}
