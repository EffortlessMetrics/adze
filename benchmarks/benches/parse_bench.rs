//! Benchmark naming note:
//! This file used to expose only a dummy benchmark, which looked like parser
//! throughput but only measured integer addition. Keep one explicitly named
//! placeholder microbench for Criterion smoke checks, and add a tiny real parse
//! smoke benchmark so callers can distinguish signal from scaffolding.

use adze_example::arithmetic::grammar::parse;
use criterion::{Criterion, black_box, criterion_group, criterion_main};

const ARITH_SMALL: &str = include_str!("../fixtures/arithmetic/small.expr");

fn placeholder_microbench(c: &mut Criterion) {
    c.bench_function("placeholder/addition_only", |b| b.iter(|| black_box(1 + 1)));
}

fn real_parser_smoke(c: &mut Criterion) {
    assert!(
        parse(ARITH_SMALL).is_ok(),
        "small arithmetic fixture must parse in benchmark setup"
    );

    c.bench_function("parser_smoke/arithmetic_small_fixture", |b| {
        b.iter(|| {
            black_box(parse(ARITH_SMALL).expect("fixture must parse"));
        });
    });
}

criterion_group!(benches, placeholder_microbench, real_parser_smoke);
criterion_main!(benches);
