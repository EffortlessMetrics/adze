use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use std::time::Duration;

#[cfg(feature = "perf-counters")]
use rust_sitter_glr_core::perf;
use glr_test_support::test_utilities::make_minimal_table;
use rust_sitter_glr_core::Driver;
use rust_sitter_ir::SymbolId;

pub fn bench_parse_small(c: &mut Criterion) {
    let mut g = c.benchmark_group("glr-perf-snapshot");

    // Quick/stable knobs: default to "quick" for dev loops; unset for longer runs.
    if std::env::var_os("BENCH_QUICK").is_some() {
        g.sample_size(60);
        g.measurement_time(Duration::from_millis(600));
        g.warm_up_time(Duration::from_millis(300));
    } else {
        // Criterion defaults are already conservative; set explicitly if you want.
        g.sample_size(100);
        g.measurement_time(Duration::from_secs(5));
        g.warm_up_time(Duration::from_secs(3));
    }

    // Minimal table/driver for this microbench.
    let table = make_minimal_table(
        vec![vec![vec![], vec![], vec![]]], // ERROR, terminal, EOF
        vec![vec![], vec![], vec![]],
        vec![],
        SymbolId(3), // start symbol
        SymbolId(2), // EOF symbol
        0,
    );
    let mut driver = Driver::new(&table);

    // One EOF token. Pass as iterator of values (no alloc per iter).
    const TOKENS: &[(u32, u32, u32)] = &[(2, 0, 0)];
    g.throughput(Throughput::Elements(TOKENS.len() as u64));

    #[cfg(feature = "perf-counters")]
    {
        // Pre‑warm to remove first‑iteration noise.
        let _ = perf::take(); // clear
        let _ = black_box(driver.parse_tokens(black_box(TOKENS.iter().copied()))); // warm caches
        let _ = perf::take(); // clear again before measuring
    }

    // Time only the hot path.
    g.bench_function("small-parse", |b| {
        b.iter(|| {
            #[cfg(feature = "perf-counters")]
            let _ = perf::take(); // zero

            let _res = black_box(driver.parse_tokens(black_box(TOKENS.iter().copied())));

            #[cfg(feature = "perf-counters")]
            let _snap = black_box(perf::take()); // snapshot to keep used
        })
    });

    g.finish();
}

criterion_group!(benches, bench_parse_small);
criterion_main!(benches);