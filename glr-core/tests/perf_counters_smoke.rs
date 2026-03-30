#![cfg(feature = "perf_counters")]

use adze_glr_core::perf;

#[test]
fn counters_api_smoke_test() {
    // Just verify the perf counter API works
    let initial = perf::take();

    // Manually increment counters
    perf::inc_shifts(10);
    perf::inc_reductions(5);
    perf::inc_forks(2);
    perf::inc_merges(1);

    let after = perf::take();

    // Verify counters moved
    assert_eq!(after.shifts - initial.shifts, 10);
    assert_eq!(after.reductions - initial.reductions, 5);
    assert_eq!(after.forks - initial.forks, 2);
    assert_eq!(after.merges - initial.merges, 1);

    // Verify take() truly resets
    let reset = perf::take();
    assert_eq!(reset.shifts, 0, "take() should reset shifts to 0");
    assert_eq!(reset.reductions, 0, "take() should reset reductions to 0");
    assert_eq!(reset.forks, 0, "take() should reset forks to 0");
    assert_eq!(reset.merges, 0, "take() should reset merges to 0");
}
