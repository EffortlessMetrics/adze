#![cfg(feature = "test-api")]

use adze_glr_core::symbol_comparison::*;
use adze_glr_core::*;
use adze_ir::SymbolId;

// ── Edge cases: identity & boundary IDs ──────────────────────────────

#[test]
fn same_symbol_yields_tie() {
    for id in [0u16, 1, 100, u16::MAX] {
        assert_eq!(
            compare_symbols(SymbolId(id), SymbolId(id)),
            CompareResult::Tie,
            "identical SymbolId({id}) must tie"
        );
    }
}

#[test]
fn zero_symbol_id_wins_over_any() {
    // SymbolId(0) is typically EOF; it should beat every higher ID.
    for other in [1u16, 50, 999, u16::MAX] {
        assert_eq!(
            compare_symbols(SymbolId(0), SymbolId(other)),
            CompareResult::TakeLeft,
        );
        assert_eq!(
            compare_symbols(SymbolId(other), SymbolId(0)),
            CompareResult::TakeRight,
        );
    }
}

#[test]
fn max_symbol_id_loses_to_any() {
    let max = SymbolId(u16::MAX);
    for id in [0u16, 1, u16::MAX - 1] {
        assert_eq!(compare_symbols(max, SymbolId(id)), CompareResult::TakeRight,);
    }
    // But ties with itself.
    assert_eq!(compare_symbols(max, max), CompareResult::Tie);
}

// ── Ordering / priority ─────────────────────────────────────────────

#[test]
fn adjacent_ids_resolve_correctly() {
    // Consecutive IDs: lower wins.
    for base in [0u16, 10, 100, 1000] {
        assert_eq!(
            compare_symbols(SymbolId(base), SymbolId(base + 1)),
            CompareResult::TakeLeft,
        );
        assert_eq!(
            compare_symbols(SymbolId(base + 1), SymbolId(base)),
            CompareResult::TakeRight,
        );
    }
}

#[test]
fn ordering_is_antisymmetric() {
    let pairs = [(1, 5), (0, 999), (42, 43), (100, 200)];
    for (a, b) in pairs {
        let fwd = compare_symbols(SymbolId(a), SymbolId(b));
        let rev = compare_symbols(SymbolId(b), SymbolId(a));
        // If forward is TakeLeft, reverse must be TakeRight, and vice-versa.
        match fwd {
            CompareResult::TakeLeft => assert_eq!(rev, CompareResult::TakeRight),
            CompareResult::TakeRight => assert_eq!(rev, CompareResult::TakeLeft),
            CompareResult::Tie => assert_eq!(rev, CompareResult::Tie),
            _ => panic!("compare_symbols should only return TakeLeft/TakeRight/Tie"),
        }
    }
}

#[test]
fn ordering_is_transitive() {
    let a = SymbolId(5);
    let b = SymbolId(10);
    let c = SymbolId(20);

    assert_eq!(compare_symbols(a, b), CompareResult::TakeLeft);
    assert_eq!(compare_symbols(b, c), CompareResult::TakeLeft);
    // Transitivity: a < b < c ⇒ a < c
    assert_eq!(compare_symbols(a, c), CompareResult::TakeLeft);
}

// ── Terminals vs non-terminals (convention: terminals < non-terminals) ──

#[test]
fn terminal_range_beats_nonterminal_range() {
    // Tree-sitter convention: terminals occupy low IDs, non-terminals higher.
    let terminals = [SymbolId(1), SymbolId(2), SymbolId(3)];
    let nonterminals = [SymbolId(100), SymbolId(200), SymbolId(300)];

    for &t in &terminals {
        for &nt in &nonterminals {
            assert_eq!(
                compare_symbols(t, nt),
                CompareResult::TakeLeft,
                "terminal {t:?} should win over non-terminal {nt:?}",
            );
        }
    }
}

// ── Symbol comparison as tie-breaker (compare_versions_with_symbols) ──

#[test]
fn version_tie_resolved_by_symbol() {
    let v = VersionInfo::new();

    // Equal versions → symbol comparison decides.
    assert_eq!(
        compare_versions_with_symbols(&v, &v, SymbolId(3), SymbolId(7)),
        CompareResult::TakeLeft,
    );
    assert_eq!(
        compare_versions_with_symbols(&v, &v, SymbolId(7), SymbolId(3)),
        CompareResult::TakeRight,
    );
    assert_eq!(
        compare_versions_with_symbols(&v, &v, SymbolId(5), SymbolId(5)),
        CompareResult::Tie,
    );
}

#[test]
fn version_difference_overrides_symbol() {
    let mut better = VersionInfo::new();
    better.add_dynamic_prec(10);
    let worse = VersionInfo::new();

    // Version wins even when the symbol comparison would go the other way.
    assert_eq!(
        compare_versions_with_symbols(&better, &worse, SymbolId(999), SymbolId(1)),
        CompareResult::TakeLeft,
    );
    assert_eq!(
        compare_versions_with_symbols(&worse, &better, SymbolId(1), SymbolId(999)),
        CompareResult::TakeRight,
    );
}

#[test]
fn error_version_overrides_symbol() {
    let clean = VersionInfo::new();
    let mut errored = VersionInfo::new();
    errored.enter_error();

    // Error path loses regardless of symbol order.
    assert_eq!(
        compare_versions_with_symbols(&errored, &clean, SymbolId(0), SymbolId(9999)),
        CompareResult::TakeRight,
    );
    assert_eq!(
        compare_versions_with_symbols(&clean, &errored, SymbolId(9999), SymbolId(0)),
        CompareResult::TakeLeft,
    );
}

// ── Large sets of symbols ───────────────────────────────────────────

#[test]
fn large_symbol_set_sorted_by_compare() {
    // Build 500 symbol IDs, shuffle-ish via stride, then sort using compare_symbols.
    let mut ids: Vec<SymbolId> = (0u16..500).map(|i| SymbolId(i * 7 % 500)).collect();

    ids.sort_by(|a, b| match compare_symbols(*a, *b) {
        CompareResult::TakeLeft => std::cmp::Ordering::Less,
        CompareResult::TakeRight => std::cmp::Ordering::Greater,
        _ => std::cmp::Ordering::Equal,
    });

    // After sorting, IDs must be in non-decreasing order.
    for window in ids.windows(2) {
        assert!(window[0].0 <= window[1].0, "sorted order violated");
    }
}

#[test]
fn bulk_pairwise_no_panics() {
    // Ensure compare_symbols never panics across a wide range.
    let ids: Vec<SymbolId> = (0u16..200).map(SymbolId).collect();
    for &a in &ids {
        for &b in &ids {
            let _ = compare_symbols(a, b);
        }
    }
}
