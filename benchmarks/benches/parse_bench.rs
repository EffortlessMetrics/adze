// Placeholder benchmark kept to verify Criterion harness wiring while parser APIs stabilize.
// This does NOT measure parser, GLR, or tablegen performance.

use criterion::{criterion_group, criterion_main};

fn placeholder_harness_smoke(c: &mut criterion::Criterion) {
    c.bench_function("placeholder_harness_smoke", |b| b.iter(|| 1 + 1));
}

criterion_group!(benches, placeholder_harness_smoke);
criterion_main!(benches);
