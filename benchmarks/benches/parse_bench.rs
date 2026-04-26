//! Placeholder benchmark while parser API stabilization is in progress.
//!
//! This does **not** measure parser performance. Keep this bench only so CI and
//! local `cargo bench -p adze-benchmarks --no-run` continue to validate bench
//! wiring until a real parser workload replaces it.

use criterion::{criterion_group, criterion_main};

fn dummy_bench(c: &mut criterion::Criterion) {
    c.bench_function("placeholder_no_parser_workload", |b| b.iter(|| 1 + 1));
}

criterion_group!(benches, dummy_bench);
criterion_main!(benches);
