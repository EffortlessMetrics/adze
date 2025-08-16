#[cfg(feature = "perf-counters")]
pub fn measure<F, R>(f: F) -> (rust_sitter_glr_core::perf::Counters, R)
where
    F: FnOnce() -> R,
{
    rust_sitter_glr_core::perf::take();
    let r = f();
    (rust_sitter_glr_core::perf::take(), r)
}

#[cfg(not(feature = "perf-counters"))]
pub fn measure<F, R>(f: F) -> (rust_sitter_glr_core::perf::Counters, R)
where
    F: FnOnce() -> R,
{
    let r = f();
    (rust_sitter_glr_core::perf::Counters::default(), r)
}