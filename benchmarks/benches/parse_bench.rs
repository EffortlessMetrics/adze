// TODO: Fix benchmarks after pure_parser API stabilization
// This file is temporarily disabled while we stabilize the parser API

use criterion::{criterion_group, criterion_main};

fn dummy_bench(c: &mut criterion::Criterion) {
    c.bench_function("dummy", |b| b.iter(|| 1 + 1));
}

criterion_group!(benches, dummy_bench);
criterion_main!(benches);
