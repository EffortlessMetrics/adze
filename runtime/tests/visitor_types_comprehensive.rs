//! Comprehensive tests for visitor types and VisitorAction enum.

use adze::visitor::{PrettyPrintVisitor, StatsVisitor, VisitorAction};

// ── VisitorAction variants ──

#[test]
fn action_continue() {
    let _ = format!("{:?}", VisitorAction::Continue);
}

#[test]
fn action_skip_children() {
    let _ = format!("{:?}", VisitorAction::SkipChildren);
}

#[test]
fn action_stop() {
    let _ = format!("{:?}", VisitorAction::Stop);
}

#[test]
fn action_debug_continue() {
    assert!(format!("{:?}", VisitorAction::Continue).contains("Continue"));
}

#[test]
fn action_debug_skip() {
    assert!(format!("{:?}", VisitorAction::SkipChildren).contains("SkipChildren"));
}

#[test]
fn action_debug_stop() {
    assert!(format!("{:?}", VisitorAction::Stop).contains("Stop"));
}

// ── VisitorAction eq ──

#[test]
fn action_eq_continue() {
    assert_eq!(VisitorAction::Continue, VisitorAction::Continue);
}

#[test]
fn action_eq_skip() {
    assert_eq!(VisitorAction::SkipChildren, VisitorAction::SkipChildren);
}

#[test]
fn action_eq_stop() {
    assert_eq!(VisitorAction::Stop, VisitorAction::Stop);
}

#[test]
fn action_ne_continue_skip() {
    assert_ne!(VisitorAction::Continue, VisitorAction::SkipChildren);
}

#[test]
fn action_ne_continue_stop() {
    assert_ne!(VisitorAction::Continue, VisitorAction::Stop);
}

#[test]
fn action_ne_skip_stop() {
    assert_ne!(VisitorAction::SkipChildren, VisitorAction::Stop);
}

// ── VisitorAction clone ──

#[test]
fn action_clone_continue() {
    let a = VisitorAction::Continue;
    let b = a.clone();
    assert_eq!(a, b);
}

#[test]
fn action_clone_skip() {
    let a = VisitorAction::SkipChildren;
    let b = a.clone();
    assert_eq!(a, b);
}

#[test]
fn action_clone_stop() {
    let a = VisitorAction::Stop;
    let b = a.clone();
    assert_eq!(a, b);
}

// ── VisitorAction size ──

#[test]
fn action_small_size() {
    assert!(std::mem::size_of::<VisitorAction>() <= 8);
}

// ── VisitorAction traits ──

#[test]
fn action_is_debug() {
    fn check<T: std::fmt::Debug>() {}
    check::<VisitorAction>();
}

#[test]
fn action_is_clone() {
    fn check<T: Clone>() {}
    check::<VisitorAction>();
}

#[test]
fn action_is_partialeq() {
    fn check<T: PartialEq>() {}
    check::<VisitorAction>();
}

// ── StatsVisitor ──

#[test]
fn stats_visitor_default() {
    let _v = StatsVisitor::default();
}

#[test]
fn stats_visitor_debug() {
    let v = StatsVisitor::default();
    let s = format!("{:?}", v);
    assert!(!s.is_empty());
}

#[test]
fn stats_visitor_initial_consistent() {
    let v1 = StatsVisitor::default();
    let v2 = StatsVisitor::default();
    assert_eq!(format!("{:?}", v1), format!("{:?}", v2));
}

#[test]
fn stats_visitor_is_debug() {
    fn check<T: std::fmt::Debug>() {}
    check::<StatsVisitor>();
}

#[test]
fn stats_visitor_is_default() {
    fn check<T: Default>() {}
    check::<StatsVisitor>();
}

#[test]
fn stats_visitor_size() {
    assert!(std::mem::size_of::<StatsVisitor>() > 0);
}

// ── PrettyPrintVisitor ──

#[test]
fn pretty_print_new() {
    let _v = PrettyPrintVisitor::new();
}

#[test]
fn pretty_print_output_empty() {
    let v = PrettyPrintVisitor::new();
    assert!(v.output().is_empty());
}

#[test]
fn pretty_print_multiple_fresh() {
    for _ in 0..10 {
        let v = PrettyPrintVisitor::new();
        assert!(v.output().is_empty());
    }
}

#[test]
fn pretty_print_size() {
    assert!(std::mem::size_of::<PrettyPrintVisitor>() > 0);
}

// ── All three actions in vec ──

#[test]
fn action_vec() {
    let actions = vec![
        VisitorAction::Continue,
        VisitorAction::SkipChildren,
        VisitorAction::Stop,
    ];
    assert_eq!(actions.len(), 3);
    assert_ne!(actions[0], actions[1]);
    assert_ne!(actions[1], actions[2]);
}

// ── Action match patterns ──

#[test]
fn action_match_continue() {
    let a = VisitorAction::Continue;
    match a {
        VisitorAction::Continue => {}
        _ => panic!("wrong variant"),
    }
}

#[test]
fn action_match_skip() {
    let a = VisitorAction::SkipChildren;
    match a {
        VisitorAction::SkipChildren => {}
        _ => panic!("wrong variant"),
    }
}

#[test]
fn action_match_stop() {
    let a = VisitorAction::Stop;
    match a {
        VisitorAction::Stop => {}
        _ => panic!("wrong variant"),
    }
}

// ── Exhaustive matching ──

#[test]
fn action_exhaustive() {
    for a in [
        VisitorAction::Continue,
        VisitorAction::SkipChildren,
        VisitorAction::Stop,
    ] {
        match a {
            VisitorAction::Continue => {}
            VisitorAction::SkipChildren => {}
            VisitorAction::Stop => {}
        }
    }
}

// ── StatsVisitor multiple instances ──

#[test]
fn stats_visitor_multiple() {
    let _v1 = StatsVisitor::default();
    let _v2 = StatsVisitor::default();
    let _v3 = StatsVisitor::default();
}

// ── PrettyPrintVisitor multiple instances ──

#[test]
fn pretty_print_multiple() {
    let _v1 = PrettyPrintVisitor::new();
    let _v2 = PrettyPrintVisitor::new();
    let _v3 = PrettyPrintVisitor::new();
}
