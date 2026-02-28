use adze_example::arithmetic::grammar::parse;
use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};

// Load real arithmetic fixtures to keep benchmark inputs valid
// for the GLR parser used in perf gating.
const ARITH_SMALL: &str = include_str!("../fixtures/arithmetic/small.expr");
const ARITH_MEDIUM: &str = include_str!("../fixtures/arithmetic/medium.expr");
const ARITH_LARGE: &str = include_str!("../fixtures/arithmetic/large.expr");

fn benchmark_glr_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("glr_parsing");

    // Validate fixtures once up front so benches only measure parse work.
    for (label, source) in &[
        ("small", ARITH_SMALL),
        ("medium", ARITH_MEDIUM),
        ("large", ARITH_LARGE),
    ] {
        assert!(
            parse(source).is_ok(),
            "Fixture {} must parse successfully for perf benchmarks",
            label
        );
    }

    // Real parser workload: parse valid arithmetic expressions.
    for (label, source) in &[
        ("small", ARITH_SMALL),
        ("medium", ARITH_MEDIUM),
        ("large", ARITH_LARGE),
    ] {
        group.bench_with_input(BenchmarkId::new("parse", label), source, |b, source| {
            b.iter(|| {
                black_box(parse(source).expect("fixture must parse"));
            });
        });
    }

    group.finish();
}

criterion_group!(benches, benchmark_glr_parsing);
criterion_main!(benches);
