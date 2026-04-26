// Placeholder benchmark while parser API migration is in progress.
//
// IMPORTANT: This does NOT exercise parser behavior. It exists only to keep
// the benchmark target compiling until real parser benches are reintroduced.

use criterion::{criterion_group, criterion_main};

fn placeholder_api_migration_smoke(c: &mut criterion::Criterion) {
    c.bench_function("placeholder_no_parser_work", |b| b.iter(|| 1 + 1));
}

criterion_group!(benches, placeholder_api_migration_smoke);
criterion_main!(benches);
