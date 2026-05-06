//! Benchmarks for parser-backend-core hot-path functions.
//!
//! Measures performance of backend selection used in parser.

use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;

use adze_parser_backend_core::ParserBackend;

fn bench_backend_is_glr(c: &mut Criterion) {
    c.bench_function("backend_is_glr", |b| {
        b.iter(|| {
            black_box(black_box(ParserBackend::GLR).is_glr());
            black_box(black_box(ParserBackend::PureRust).is_glr());
            black_box(black_box(ParserBackend::TreeSitter).is_glr());
        });
    });
}

fn bench_backend_is_pure_rust(c: &mut Criterion) {
    c.bench_function("backend_is_pure_rust", |b| {
        b.iter(|| {
            black_box(black_box(ParserBackend::GLR).is_pure_rust());
            black_box(black_box(ParserBackend::PureRust).is_pure_rust());
            black_box(black_box(ParserBackend::TreeSitter).is_pure_rust());
        });
    });
}

fn bench_backend_name(c: &mut Criterion) {
    c.bench_function("backend_name", |b| {
        b.iter(|| {
            black_box(black_box(ParserBackend::GLR).name());
            black_box(black_box(ParserBackend::PureRust).name());
            black_box(black_box(ParserBackend::TreeSitter).name());
        });
    });
}

fn bench_backend_display(c: &mut Criterion) {
    c.bench_function("backend_display", |b| {
        b.iter(|| {
            black_box(format!("{}", black_box(ParserBackend::GLR)));
            black_box(format!("{}", black_box(ParserBackend::PureRust)));
            black_box(format!("{}", black_box(ParserBackend::TreeSitter)));
        });
    });
}

fn bench_backend_clone(c: &mut Criterion) {
    c.bench_function("backend_clone", |b| {
        let backend = ParserBackend::GLR;
        b.iter(|| black_box(black_box(backend)));
    });
}

fn bench_backend_eq(c: &mut Criterion) {
    c.bench_function("backend_eq", |b| {
        let a = ParserBackend::GLR;
        let d = ParserBackend::GLR;
        let e = ParserBackend::PureRust;
        b.iter(|| {
            black_box(black_box(a) == black_box(d));
            black_box(black_box(a) == black_box(e));
        });
    });
}

fn bench_backend_debug(c: &mut Criterion) {
    c.bench_function("backend_debug", |b| {
        b.iter(|| {
            black_box(format!("{:?}", black_box(ParserBackend::GLR)));
            black_box(format!("{:?}", black_box(ParserBackend::PureRust)));
            black_box(format!("{:?}", black_box(ParserBackend::TreeSitter)));
        });
    });
}

fn bench_backend_all_variants(c: &mut Criterion) {
    c.bench_function("backend_all_variants_check", |b| {
        let backends = [
            ParserBackend::TreeSitter,
            ParserBackend::PureRust,
            ParserBackend::GLR,
        ];
        b.iter(|| {
            for backend in &backends {
                black_box(black_box(*backend).is_glr());
                black_box(black_box(*backend).is_pure_rust());
                black_box(black_box(*backend).name());
            }
        });
    });
}

criterion_group!(
    benches,
    bench_backend_is_glr,
    bench_backend_is_pure_rust,
    bench_backend_name,
    bench_backend_display,
    bench_backend_clone,
    bench_backend_eq,
    bench_backend_debug,
    bench_backend_all_variants,
);

criterion_main!(benches);
