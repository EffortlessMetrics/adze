use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::time::Duration;

#[cfg(feature = "perf-counters")]
use rust_sitter_glr_core::perf;

pub fn bench_parse_small(c: &mut Criterion) {
    let mut g = c.benchmark_group("glr-perf-snapshot");
    
    // Micro-bench stability knobs for tiny workloads
    g.sample_size(60);
    g.measurement_time(Duration::from_millis(600));
    g.warm_up_time(Duration::from_millis(300));

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
        
        g.throughput(Throughput::Elements(TOKENS.len() as u64));

        // Pre-warm to remove first-iteration noise (OnceLock init, JIT-ish cold paths)
        let _ = perf::take(); // clear
        let _ = black_box(driver.parse_tokens(black_box(TOKENS.iter().copied()))); // warm caches
        let _ = perf::take(); // clear again before measuring

        g.bench_with_input(BenchmarkId::new("small-parse", TOKENS.len()), &(), |b, _| {
            b.iter(|| {
                // Scoped measurement: zero → run → take snapshot
                let _ = perf::take(); // clear
                let _res = black_box(driver.parse_tokens(black_box(TOKENS.iter().copied())));
                let _snap = black_box(perf::take());
            })
        });
    }

    #[cfg(not(feature = "perf-counters"))]
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
        
        g.throughput(Throughput::Elements(TOKENS.len() as u64));
        
        g.bench_with_input(BenchmarkId::new("small-parse", TOKENS.len()), &(), |b, _| {
            b.iter(|| {
                // Without perf counters, just run the parser
                let _res = black_box(driver.parse_tokens(black_box(TOKENS.iter().copied())));
            })
        });
    }

    g.finish();
}

criterion_group!(benches, bench_parse_small);
criterion_main!(benches);