use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;

use adze_concurrency_caps_core::{
    ConcurrencyCaps, normalized_concurrency, parse_positive_usize_or_default,
};

fn bench_caps_from_lookup(c: &mut Criterion) {
    c.bench_function("caps_from_lookup_defaults", |b| {
        b.iter(|| {
            let caps = ConcurrencyCaps::from_lookup(|_| None);
            black_box(caps)
        });
    });

    c.bench_function("caps_from_lookup_with_values", |b| {
        b.iter(|| {
            let caps = ConcurrencyCaps::from_lookup(|name| match name {
                "RAYON_NUM_THREADS" => Some("8".to_string()),
                "TOKIO_WORKER_THREADS" => Some("4".to_string()),
                _ => None,
            });
            black_box(caps)
        });
    });
}

fn bench_caps_default(c: &mut Criterion) {
    c.bench_function("caps_default", |b| {
        b.iter(|| black_box(ConcurrencyCaps::default()));
    });
}

fn bench_normalized_concurrency(c: &mut Criterion) {
    c.bench_function("normalized_concurrency_zero", |b| {
        b.iter(|| black_box(normalized_concurrency(black_box(0))));
    });

    c.bench_function("normalized_concurrency_nonzero", |b| {
        b.iter(|| black_box(normalized_concurrency(black_box(8))));
    });
}

fn bench_parse_positive_usize(c: &mut Criterion) {
    c.bench_function("parse_positive_usize_valid", |b| {
        b.iter(|| black_box(parse_positive_usize_or_default(black_box(Some("16")), 4)));
    });

    c.bench_function("parse_positive_usize_none", |b| {
        b.iter(|| black_box(parse_positive_usize_or_default(black_box(None), 4)));
    });

    c.bench_function("parse_positive_usize_invalid", |b| {
        b.iter(|| black_box(parse_positive_usize_or_default(black_box(Some("abc")), 4)));
    });
}

criterion_group!(
    benches,
    bench_caps_from_lookup,
    bench_caps_default,
    bench_normalized_concurrency,
    bench_parse_positive_usize,
);
criterion_main!(benches);
