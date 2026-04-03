//! Test performance counters functionality

#[cfg(feature = "perf_counters")]
#[test]
fn test_perf_counters_enabled() {
    use adze_glr_core::perf;

    // Reset counters
    perf::reset();

    // Initial state should be zero
    let initial = perf::snapshot();
    assert_eq!(initial.shifts, 0);
    assert_eq!(initial.reductions, 0);
    assert_eq!(initial.forks, 0);
    assert_eq!(initial.merges, 0);

    // Increment counters
    perf::inc_shifts(5);
    perf::inc_reductions(3);
    perf::inc_forks(2);
    perf::inc_merges(1);

    // Verify counts
    let snapshot = perf::snapshot();
    assert_eq!(snapshot.shifts, 5);
    assert_eq!(snapshot.reductions, 3);
    assert_eq!(snapshot.forks, 2);
    assert_eq!(snapshot.merges, 1);

    // Reset and verify
    perf::reset();
    let after_reset = perf::snapshot();
    assert_eq!(after_reset.shifts, 0);
    assert_eq!(after_reset.reductions, 0);
    assert_eq!(after_reset.forks, 0);
    assert_eq!(after_reset.merges, 0);
}

#[cfg(not(feature = "perf_counters"))]
#[test]
fn test_perf_counters_disabled() {
    use adze_glr_core::perf;

    // When disabled, counters should always be zero
    perf::inc_shifts(100);
    perf::inc_reductions(50);
    perf::inc_forks(25);
    perf::inc_merges(10);

    let snapshot = perf::snapshot();
    assert_eq!(snapshot.shifts, 0);
    assert_eq!(snapshot.reductions, 0);
    assert_eq!(snapshot.forks, 0);
    assert_eq!(snapshot.merges, 0);
}
