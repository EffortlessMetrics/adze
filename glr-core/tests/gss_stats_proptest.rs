#![allow(clippy::needless_range_loop)]

use adze_glr_core::GSSStats;
use proptest::prelude::*;

fn arb_gss_stats() -> impl Strategy<Value = GSSStats> {
    (
        any::<usize>(),
        any::<usize>(),
        any::<usize>(),
        any::<usize>(),
        any::<usize>(),
    )
        .prop_map(
            |(nodes, heads, forks, merges, segments)| GSSStats {
                total_nodes_created: nodes,
                max_active_heads: heads,
                total_forks: forks,
                total_merges: merges,
                shared_segments: segments,
            },
        )
}

fn arb_small_gss_stats() -> impl Strategy<Value = GSSStats> {
    (0..10_000usize, 0..10_000usize, 0..10_000usize, 0..10_000usize, 0..10_000usize).prop_map(
        |(nodes, heads, forks, merges, segments)| GSSStats {
            total_nodes_created: nodes,
            max_active_heads: heads,
            total_forks: forks,
            total_merges: merges,
            shared_segments: segments,
        },
    )
}

// --- Creation tests ---

proptest! {
    #[test]
    fn test_creation_preserves_total_nodes(val in any::<usize>()) {
        let stats = GSSStats {
            total_nodes_created: val,
            ..Default::default()
        };
        prop_assert_eq!(stats.total_nodes_created, val);
    }

    #[test]
    fn test_creation_preserves_max_active_heads(val in any::<usize>()) {
        let stats = GSSStats {
            max_active_heads: val,
            ..Default::default()
        };
        prop_assert_eq!(stats.max_active_heads, val);
    }

    #[test]
    fn test_creation_preserves_total_forks(val in any::<usize>()) {
        let stats = GSSStats {
            total_forks: val,
            ..Default::default()
        };
        prop_assert_eq!(stats.total_forks, val);
    }

    #[test]
    fn test_creation_preserves_total_merges(val in any::<usize>()) {
        let stats = GSSStats {
            total_merges: val,
            ..Default::default()
        };
        prop_assert_eq!(stats.total_merges, val);
    }

    #[test]
    fn test_creation_preserves_shared_segments(val in any::<usize>()) {
        let stats = GSSStats {
            shared_segments: val,
            ..Default::default()
        };
        prop_assert_eq!(stats.shared_segments, val);
    }

    #[test]
    fn test_creation_all_fields(
        nodes in any::<usize>(),
        heads in any::<usize>(),
        forks in any::<usize>(),
        merges in any::<usize>(),
        segments in any::<usize>(),
    ) {
        let stats = GSSStats {
            total_nodes_created: nodes,
            max_active_heads: heads,
            total_forks: forks,
            total_merges: merges,
            shared_segments: segments,
        };
        prop_assert_eq!(stats.total_nodes_created, nodes);
        prop_assert_eq!(stats.max_active_heads, heads);
        prop_assert_eq!(stats.total_forks, forks);
        prop_assert_eq!(stats.total_merges, merges);
        prop_assert_eq!(stats.shared_segments, segments);
    }
}

// --- Default tests ---

#[test]
fn test_default_all_zero() {
    let stats = GSSStats::default();
    assert_eq!(stats.total_nodes_created, 0);
    assert_eq!(stats.max_active_heads, 0);
    assert_eq!(stats.total_forks, 0);
    assert_eq!(stats.total_merges, 0);
    assert_eq!(stats.shared_segments, 0);
}

#[test]
fn test_default_matches_explicit_zeros() {
    let def = GSSStats::default();
    let explicit = GSSStats {
        total_nodes_created: 0,
        max_active_heads: 0,
        total_forks: 0,
        total_merges: 0,
        shared_segments: 0,
    };
    assert_eq!(
        format!("{:?}", def),
        format!("{:?}", explicit),
    );
}

// --- Debug tests ---

proptest! {
    #[test]
    fn test_debug_contains_struct_name(stats in arb_gss_stats()) {
        let debug = format!("{:?}", stats);
        prop_assert!(debug.contains("GSSStats"));
    }

    #[test]
    fn test_debug_contains_field_names(stats in arb_gss_stats()) {
        let debug = format!("{:?}", stats);
        prop_assert!(debug.contains("total_nodes_created"));
        prop_assert!(debug.contains("max_active_heads"));
        prop_assert!(debug.contains("total_forks"));
        prop_assert!(debug.contains("total_merges"));
        prop_assert!(debug.contains("shared_segments"));
    }

    #[test]
    fn test_debug_contains_field_values(
        nodes in 0..1000usize,
        heads in 0..1000usize,
        forks in 0..1000usize,
        merges in 0..1000usize,
        segments in 0..1000usize,
    ) {
        let stats = GSSStats {
            total_nodes_created: nodes,
            max_active_heads: heads,
            total_forks: forks,
            total_merges: merges,
            shared_segments: segments,
        };
        let debug = format!("{:?}", stats);
        prop_assert!(debug.contains(&nodes.to_string()));
        prop_assert!(debug.contains(&heads.to_string()));
        prop_assert!(debug.contains(&forks.to_string()));
        prop_assert!(debug.contains(&merges.to_string()));
        prop_assert!(debug.contains(&segments.to_string()));
    }

    #[test]
    fn test_debug_is_nonempty(stats in arb_gss_stats()) {
        let debug = format!("{:?}", stats);
        prop_assert!(!debug.is_empty());
    }
}

// --- Field access tests ---

proptest! {
    #[test]
    fn test_field_access_roundtrip(stats in arb_gss_stats()) {
        let copy = GSSStats {
            total_nodes_created: stats.total_nodes_created,
            max_active_heads: stats.max_active_heads,
            total_forks: stats.total_forks,
            total_merges: stats.total_merges,
            shared_segments: stats.shared_segments,
        };
        prop_assert_eq!(copy.total_nodes_created, stats.total_nodes_created);
        prop_assert_eq!(copy.max_active_heads, stats.max_active_heads);
        prop_assert_eq!(copy.total_forks, stats.total_forks);
        prop_assert_eq!(copy.total_merges, stats.total_merges);
        prop_assert_eq!(copy.shared_segments, stats.shared_segments);
    }

    #[test]
    fn test_field_mutation_total_nodes(init in any::<usize>(), new_val in any::<usize>()) {
        let mut stats = GSSStats {
            total_nodes_created: init,
            ..Default::default()
        };
        stats.total_nodes_created = new_val;
        prop_assert_eq!(stats.total_nodes_created, new_val);
    }

    #[test]
    fn test_field_mutation_independence(
        nodes in any::<usize>(),
        heads in any::<usize>(),
    ) {
        let mut stats = GSSStats::default();
        stats.total_nodes_created = nodes;
        stats.max_active_heads = heads;
        prop_assert_eq!(stats.total_nodes_created, nodes);
        prop_assert_eq!(stats.max_active_heads, heads);
        prop_assert_eq!(stats.total_forks, 0);
        prop_assert_eq!(stats.total_merges, 0);
        prop_assert_eq!(stats.shared_segments, 0);
    }
}

// --- Clone-like behavior (manual copy since no Clone derive) ---

proptest! {
    #[test]
    fn test_manual_clone_equals_original(stats in arb_gss_stats()) {
        let cloned = GSSStats {
            total_nodes_created: stats.total_nodes_created,
            max_active_heads: stats.max_active_heads,
            total_forks: stats.total_forks,
            total_merges: stats.total_merges,
            shared_segments: stats.shared_segments,
        };
        prop_assert_eq!(
            format!("{:?}", cloned),
            format!("{:?}", stats),
        );
    }

    #[test]
    fn test_manual_clone_field_equality(stats in arb_gss_stats()) {
        let cloned = GSSStats {
            total_nodes_created: stats.total_nodes_created,
            max_active_heads: stats.max_active_heads,
            total_forks: stats.total_forks,
            total_merges: stats.total_merges,
            shared_segments: stats.shared_segments,
        };
        prop_assert_eq!(cloned.total_nodes_created, stats.total_nodes_created);
        prop_assert_eq!(cloned.max_active_heads, stats.max_active_heads);
        prop_assert_eq!(cloned.total_forks, stats.total_forks);
        prop_assert_eq!(cloned.total_merges, stats.total_merges);
        prop_assert_eq!(cloned.shared_segments, stats.shared_segments);
    }
}

// --- Various field value ranges ---

proptest! {
    #[test]
    fn test_small_values(stats in arb_small_gss_stats()) {
        prop_assert!(stats.total_nodes_created < 10_000);
        prop_assert!(stats.max_active_heads < 10_000);
        prop_assert!(stats.total_forks < 10_000);
        prop_assert!(stats.total_merges < 10_000);
        prop_assert!(stats.shared_segments < 10_000);
    }

    #[test]
    fn test_max_values_accepted(field_idx in 0..5usize) {
        let mut stats = GSSStats::default();
        match field_idx {
            0 => stats.total_nodes_created = usize::MAX,
            1 => stats.max_active_heads = usize::MAX,
            2 => stats.total_forks = usize::MAX,
            3 => stats.total_merges = usize::MAX,
            4 => stats.shared_segments = usize::MAX,
            _ => unreachable!(),
        }
        let debug = format!("{:?}", stats);
        prop_assert!(debug.contains(&usize::MAX.to_string()));
    }

    #[test]
    fn test_power_of_two_values(exp in 0..((usize::BITS - 1) as usize)) {
        let val = 1usize << exp;
        let stats = GSSStats {
            total_nodes_created: val,
            max_active_heads: val,
            total_forks: val,
            total_merges: val,
            shared_segments: val,
        };
        prop_assert_eq!(stats.total_nodes_created, val);
        prop_assert_eq!(stats.max_active_heads, val);
        prop_assert_eq!(stats.total_forks, val);
        prop_assert_eq!(stats.total_merges, val);
        prop_assert_eq!(stats.shared_segments, val);
    }
}

// --- Accumulation pattern tests ---

proptest! {
    #[test]
    fn test_accumulate_nodes_created(increments in prop::collection::vec(1..100usize, 1..20)) {
        let mut stats = GSSStats::default();
        let mut expected = 0usize;
        for inc in &increments {
            stats.total_nodes_created += inc;
            expected += inc;
        }
        prop_assert_eq!(stats.total_nodes_created, expected);
    }

    #[test]
    fn test_accumulate_forks(increments in prop::collection::vec(1..100usize, 1..20)) {
        let mut stats = GSSStats::default();
        let mut expected = 0usize;
        for inc in &increments {
            stats.total_forks += inc;
            expected += inc;
        }
        prop_assert_eq!(stats.total_forks, expected);
    }

    #[test]
    fn test_accumulate_merges(increments in prop::collection::vec(1..100usize, 1..20)) {
        let mut stats = GSSStats::default();
        let mut expected = 0usize;
        for inc in &increments {
            stats.total_merges += inc;
            expected += inc;
        }
        prop_assert_eq!(stats.total_merges, expected);
    }

    #[test]
    fn test_accumulate_max_heads_tracking(
        head_counts in prop::collection::vec(1..1000usize, 1..30),
    ) {
        let mut stats = GSSStats::default();
        let mut running_max = 0usize;
        for &count in &head_counts {
            running_max = running_max.max(count);
            stats.max_active_heads = stats.max_active_heads.max(count);
        }
        prop_assert_eq!(stats.max_active_heads, running_max);
    }

    #[test]
    fn test_accumulate_all_fields_independently(
        n in 1..50usize,
        f in 1..50usize,
        m in 1..50usize,
        s in 1..50usize,
    ) {
        let mut stats = GSSStats::default();
        for _ in 0..n {
            stats.total_nodes_created += 1;
        }
        for _ in 0..f {
            stats.total_forks += 1;
        }
        for _ in 0..m {
            stats.total_merges += 1;
        }
        for _ in 0..s {
            stats.shared_segments += 1;
        }
        prop_assert_eq!(stats.total_nodes_created, n);
        prop_assert_eq!(stats.total_forks, f);
        prop_assert_eq!(stats.total_merges, m);
        prop_assert_eq!(stats.shared_segments, s);
    }

    #[test]
    fn test_accumulate_shared_segments(increments in prop::collection::vec(1..100usize, 1..20)) {
        let mut stats = GSSStats::default();
        let mut expected = 0usize;
        for inc in &increments {
            stats.shared_segments += inc;
            expected += inc;
        }
        prop_assert_eq!(stats.shared_segments, expected);
    }
}

// --- Zero value tests ---

#[test]
fn test_zero_stats_debug_all_zeros() {
    let stats = GSSStats {
        total_nodes_created: 0,
        max_active_heads: 0,
        total_forks: 0,
        total_merges: 0,
        shared_segments: 0,
    };
    let debug = format!("{:?}", stats);
    assert!(debug.contains("total_nodes_created: 0"));
    assert!(debug.contains("max_active_heads: 0"));
    assert!(debug.contains("total_forks: 0"));
    assert!(debug.contains("total_merges: 0"));
    assert!(debug.contains("shared_segments: 0"));
}

proptest! {
    #[test]
    fn test_zero_field_with_others_nonzero(
        nodes in 1..usize::MAX,
        heads in 1..usize::MAX,
        forks in 1..usize::MAX,
        merges in 1..usize::MAX,
    ) {
        let stats = GSSStats {
            total_nodes_created: nodes,
            max_active_heads: heads,
            total_forks: forks,
            total_merges: merges,
            shared_segments: 0,
        };
        prop_assert_eq!(stats.shared_segments, 0);
        prop_assert!(stats.total_nodes_created > 0);
    }

    #[test]
    fn test_zero_then_accumulate(target in 1..1000usize) {
        let mut stats = GSSStats::default();
        prop_assert_eq!(stats.total_nodes_created, 0);
        for _ in 0..target {
            stats.total_nodes_created += 1;
        }
        prop_assert_eq!(stats.total_nodes_created, target);
    }
}
