use criterion::{black_box, criterion_group, criterion_main, Criterion};

#[cfg(feature = "perf-counters")]
use rust_sitter_glr_core::perf;

pub fn bench_parse_small(c: &mut Criterion) {
    let mut g = c.benchmark_group("glr-perf-snapshot");

    #[cfg(feature = "perf-counters")]
    {
        use glr_test_support::test_utilities::make_minimal_table;
        use rust_sitter_glr_core::Driver;

        use rust_sitter_ir::SymbolId;
        
        // Create a minimal table for benchmarking
        let table = make_minimal_table(
            vec![vec![vec![]]],  // Minimal action table
            vec![vec![]],        // Minimal goto table
            vec![],              // No rules for this simple test
            SymbolId(1),         // Start symbol
            SymbolId(2),         // EOF symbol
            0,                   // No external tokens
        );
        let mut driver = Driver::new(&table);

        // One EOF token. Pass as an iterator of values (no alloc each iter).
        const TOKENS: &[(u32, u32, u32)] = &[(2, 0, 0)];

        g.bench_function("small-parse", |b| {
            b.iter(|| {
                // Scoped measurement: zero → run → take snapshot
                let _ = perf::take(); // clear
                let iter = TOKENS.iter().copied(); // -> impl IntoIterator<Item=(u32,u32,u32)>
                let _ = driver.parse_tokens(black_box(iter));
                let c = perf::take();
                // Keep the counters "used" to avoid being optimized away
                black_box((c.shifts, c.reductions, c.forks, c.merges));
            })
        });
    }

    #[cfg(not(feature = "perf-counters"))]
    g.bench_function("skipped-no-perf-counters", |b| b.iter(|| {}));

    g.finish();
}

criterion_group!(benches, bench_parse_small);
criterion_main!(benches);