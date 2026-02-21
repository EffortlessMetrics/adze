#[cfg(feature = "perf-counters")]
pub fn measure<F, R>(f: F) -> (adze_glr_core::perf::Counters, R)
where
    F: FnOnce() -> R,
{
    adze_glr_core::perf::take();
    let r = f();
    (adze_glr_core::perf::take(), r)
}

#[cfg(not(feature = "perf-counters"))]
pub fn measure<F, R>(f: F) -> (adze_glr_core::perf::Counters, R)
where
    F: FnOnce() -> R,
{
    let r = f();
    (adze_glr_core::perf::Counters::default(), r)
}
