#![allow(clippy::needless_range_loop)]

//! Property-based tests for `ErrorMeta` in adze-glr-core.

use adze_glr_core::parse_forest::{ERROR_SYMBOL, ErrorMeta, ForestAlternative, ForestNode};
use adze_ir::SymbolId;
use proptest::prelude::*;

// ─── Strategies ──────────────────────────────────────────────────────

fn arb_error_meta() -> impl Strategy<Value = ErrorMeta> {
    (any::<bool>(), any::<bool>(), any::<u32>()).prop_map(|(missing, is_error, cost)| ErrorMeta {
        missing,
        is_error,
        cost,
    })
}

fn arb_symbol_id() -> impl Strategy<Value = SymbolId> {
    (0u16..=u16::MAX).prop_map(SymbolId)
}

// ─── Creation ────────────────────────────────────────────────────────

proptest! {
    #[test]
    fn creation_preserves_fields(missing: bool, is_error: bool, cost: u32) {
        let meta = ErrorMeta { missing, is_error, cost };
        prop_assert_eq!(meta.missing, missing);
        prop_assert_eq!(meta.is_error, is_error);
        prop_assert_eq!(meta.cost, cost);
    }
}

#[test]
fn creation_all_boolean_combos() {
    for missing in [false, true] {
        for is_error in [false, true] {
            let meta = ErrorMeta {
                missing,
                is_error,
                cost: 42,
            };
            assert_eq!(meta.missing, missing);
            assert_eq!(meta.is_error, is_error);
            assert_eq!(meta.cost, 42);
        }
    }
}

#[test]
fn creation_false_false() {
    let m = ErrorMeta {
        missing: false,
        is_error: false,
        cost: 0,
    };
    assert!(!m.missing);
    assert!(!m.is_error);
}

#[test]
fn creation_true_true() {
    let m = ErrorMeta {
        missing: true,
        is_error: true,
        cost: 99,
    };
    assert!(m.missing);
    assert!(m.is_error);
    assert_eq!(m.cost, 99);
}

// ─── Copy semantics ─────────────────────────────────────────────────

proptest! {
    #[test]
    fn copy_produces_independent_value(meta in arb_error_meta()) {
        let copy = meta;
        prop_assert_eq!(copy.missing, meta.missing);
        prop_assert_eq!(copy.is_error, meta.is_error);
        prop_assert_eq!(copy.cost, meta.cost);
    }

    #[test]
    fn copy_into_vec_preserves_all_fields(meta in arb_error_meta()) {
        let v = vec![meta; 3];
        for item in &v {
            prop_assert_eq!(item.missing, meta.missing);
            prop_assert_eq!(item.is_error, meta.is_error);
            prop_assert_eq!(item.cost, meta.cost);
        }
    }
}

#[test]
fn copy_is_bitwise_identical() {
    let orig = ErrorMeta {
        missing: true,
        is_error: false,
        cost: 1000,
    };
    let copied = orig;
    // After copy, both are usable (Copy trait).
    assert_eq!(orig.cost, copied.cost);
    assert_eq!(orig.missing, copied.missing);
    assert_eq!(orig.is_error, copied.is_error);
}

// ─── Default ─────────────────────────────────────────────────────────

#[test]
fn default_is_no_error() {
    let meta = ErrorMeta::default();
    assert!(!meta.missing);
    assert!(!meta.is_error);
    assert_eq!(meta.cost, 0);
}

#[test]
fn default_matches_manual_zero() {
    let def = ErrorMeta::default();
    let manual = ErrorMeta {
        missing: false,
        is_error: false,
        cost: 0,
    };
    assert_eq!(def.missing, manual.missing);
    assert_eq!(def.is_error, manual.is_error);
    assert_eq!(def.cost, manual.cost);
}

// ─── Debug ───────────────────────────────────────────────────────────

proptest! {
    #[test]
    fn debug_contains_field_names(meta in arb_error_meta()) {
        let dbg = format!("{:?}", meta);
        prop_assert!(dbg.contains("missing"));
        prop_assert!(dbg.contains("is_error"));
        prop_assert!(dbg.contains("cost"));
    }

    #[test]
    fn debug_contains_field_values(missing: bool, is_error: bool, cost: u32) {
        let meta = ErrorMeta { missing, is_error, cost };
        let dbg = format!("{:?}", meta);
        let missing_str = missing.to_string();
        let is_error_str = is_error.to_string();
        let cost_str = cost.to_string();
        prop_assert!(dbg.contains(&missing_str));
        prop_assert!(dbg.contains(&is_error_str));
        prop_assert!(dbg.contains(&cost_str));
    }
}

#[test]
fn debug_default_output() {
    let dbg = format!("{:?}", ErrorMeta::default());
    assert!(dbg.starts_with("ErrorMeta"));
}

// ─── Cost ranges ─────────────────────────────────────────────────────

proptest! {
    #[test]
    fn cost_roundtrips_any_u32(cost: u32) {
        let meta = ErrorMeta { missing: false, is_error: false, cost };
        prop_assert_eq!(meta.cost, cost);
    }

    #[test]
    fn cost_ordering_preserved(a: u32, b: u32) {
        let ma = ErrorMeta { missing: false, is_error: false, cost: a };
        let mb = ErrorMeta { missing: false, is_error: false, cost: b };
        prop_assert_eq!(ma.cost < mb.cost, a < b);
        prop_assert_eq!(ma.cost == mb.cost, a == b);
    }
}

// ─── Zero / max cost ─────────────────────────────────────────────────

#[test]
fn zero_cost() {
    let meta = ErrorMeta {
        missing: false,
        is_error: false,
        cost: 0,
    };
    assert_eq!(meta.cost, 0);
}

#[test]
fn max_cost() {
    let meta = ErrorMeta {
        missing: true,
        is_error: true,
        cost: u32::MAX,
    };
    assert_eq!(meta.cost, u32::MAX);
}

#[test]
fn cost_boundary_values() {
    for cost in [0, 1, u32::MAX - 1, u32::MAX] {
        let meta = ErrorMeta {
            missing: false,
            is_error: false,
            cost,
        };
        assert_eq!(meta.cost, cost);
    }
}

// ─── Comparison ──────────────────────────────────────────────────────

proptest! {
    #[test]
    fn field_wise_equality(meta in arb_error_meta()) {
        let other = ErrorMeta {
            missing: meta.missing,
            is_error: meta.is_error,
            cost: meta.cost,
        };
        prop_assert_eq!(meta.missing, other.missing);
        prop_assert_eq!(meta.is_error, other.is_error);
        prop_assert_eq!(meta.cost, other.cost);
    }

    #[test]
    fn different_cost_differs(cost_a: u32, cost_b: u32) {
        let a = ErrorMeta { missing: false, is_error: false, cost: cost_a };
        let b = ErrorMeta { missing: false, is_error: false, cost: cost_b };
        prop_assert_eq!(a.cost == b.cost, cost_a == cost_b);
    }
}

#[test]
fn comparison_different_flags() {
    let a = ErrorMeta {
        missing: true,
        is_error: false,
        cost: 0,
    };
    let b = ErrorMeta {
        missing: false,
        is_error: true,
        cost: 0,
    };
    assert_ne!(a.missing, b.missing);
    assert_ne!(a.is_error, b.is_error);
}

#[test]
fn comparison_same_flags_different_cost() {
    let a = ErrorMeta {
        missing: true,
        is_error: true,
        cost: 5,
    };
    let b = ErrorMeta {
        missing: true,
        is_error: true,
        cost: 10,
    };
    assert_eq!(a.missing, b.missing);
    assert_eq!(a.is_error, b.is_error);
    assert_ne!(a.cost, b.cost);
}

// ─── In forest nodes ─────────────────────────────────────────────────

proptest! {
    #[test]
    fn forest_node_carries_error_meta(meta in arb_error_meta(), sym in arb_symbol_id()) {
        let node = ForestNode {
            id: 0,
            symbol: sym,
            span: (0, 10),
            alternatives: vec![ForestAlternative { children: vec![] }],
            error_meta: meta,
        };
        prop_assert_eq!(node.error_meta.missing, meta.missing);
        prop_assert_eq!(node.error_meta.is_error, meta.is_error);
        prop_assert_eq!(node.error_meta.cost, meta.cost);
    }
}

#[test]
fn forest_node_error_symbol_with_meta() {
    let meta = ErrorMeta {
        missing: false,
        is_error: true,
        cost: 1,
    };
    let node = ForestNode {
        id: 42,
        symbol: ERROR_SYMBOL,
        span: (5, 8),
        alternatives: vec![],
        error_meta: meta,
    };
    assert_eq!(node.symbol, ERROR_SYMBOL);
    assert!(node.error_meta.is_error);
    assert_eq!(node.error_meta.cost, 1);
}

#[test]
fn forest_node_default_meta_is_clean() {
    let node = ForestNode {
        id: 0,
        symbol: SymbolId(1),
        span: (0, 0),
        alternatives: vec![],
        error_meta: ErrorMeta::default(),
    };
    assert!(!node.error_meta.missing);
    assert!(!node.error_meta.is_error);
    assert_eq!(node.error_meta.cost, 0);
}

#[test]
fn forest_node_missing_terminal_meta() {
    let meta = ErrorMeta {
        missing: true,
        is_error: false,
        cost: 3,
    };
    let node = ForestNode {
        id: 7,
        symbol: SymbolId(10),
        span: (4, 4),
        alternatives: vec![ForestAlternative { children: vec![] }],
        error_meta: meta,
    };
    assert!(node.error_meta.missing);
    assert!(!node.error_meta.is_error);
    assert_eq!(node.error_meta.cost, 3);
}

#[test]
fn multiple_forest_nodes_independent_meta() {
    let meta_a = ErrorMeta {
        missing: true,
        is_error: false,
        cost: 2,
    };
    let meta_b = ErrorMeta {
        missing: false,
        is_error: true,
        cost: 5,
    };
    let a = ForestNode {
        id: 0,
        symbol: SymbolId(1),
        span: (0, 3),
        alternatives: vec![],
        error_meta: meta_a,
    };
    let b = ForestNode {
        id: 1,
        symbol: SymbolId(2),
        span: (3, 6),
        alternatives: vec![],
        error_meta: meta_b,
    };
    assert!(a.error_meta.missing);
    assert!(!b.error_meta.missing);
    assert!(!a.error_meta.is_error);
    assert!(b.error_meta.is_error);
    assert_ne!(a.error_meta.cost, b.error_meta.cost);
}

// ─── Clone ───────────────────────────────────────────────────────────

proptest! {
    #[test]
    fn clone_matches_original(meta in arb_error_meta()) {
        let cloned = meta;
        prop_assert_eq!(cloned.missing, meta.missing);
        prop_assert_eq!(cloned.is_error, meta.is_error);
        prop_assert_eq!(cloned.cost, meta.cost);
    }
}
