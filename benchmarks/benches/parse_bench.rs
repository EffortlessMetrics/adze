// Benchmarks comparing pure-Rust parser with C Tree-sitter

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rust_sitter_runtime::pure_parser::Parser as PureParser;
use tree_sitter::{Language as CLanguage, Parser as CParser};

// Simple test grammar for benchmarking
const SIMPLE_GRAMMAR_SOURCE: &str = r#"
    (1 + 2) * 3 + 4 * (5 + 6)
    7 - 8 / 9 + 10 * 11
    (12 + 13) * (14 - 15) / 16
    17 + 18 - 19 * 20 / 21
    ((22 + 23) * 24) - (25 / (26 + 27))
"#;

// Larger source for stress testing
fn generate_large_source(size: usize) -> String {
    let mut source = String::new();
    for i in 0..size {
        source.push_str(&format!(
            "({} + {}) * {} - {} / {}\n",
            i,
            i + 1,
            i + 2,
            i + 3,
            i + 4
        ));
    }
    source
}

fn bench_pure_rust_parser(c: &mut Criterion) {
    let mut group = c.benchmark_group("pure_rust_parser");

    // Small input benchmark
    group.bench_function("small_input", |b| {
        let mut parser = PureParser::new();
        // Note: In real usage, we'd set a proper language here

        b.iter(|| {
            let _result = parser.parse_string(black_box(SIMPLE_GRAMMAR_SOURCE));
        });
    });

    // Medium input benchmark
    let medium_source = generate_large_source(100);
    group.bench_function("medium_input", |b| {
        let mut parser = PureParser::new();

        b.iter(|| {
            let _result = parser.parse_string(black_box(&medium_source));
        });
    });

    // Large input benchmark
    let large_source = generate_large_source(1000);
    group.bench_function("large_input", |b| {
        let mut parser = PureParser::new();

        b.iter(|| {
            let _result = parser.parse_string(black_box(&large_source));
        });
    });

    group.finish();
}

fn bench_tree_sitter_c_parser(c: &mut Criterion) {
    let mut group = c.benchmark_group("tree_sitter_c_parser");

    // Note: In a real benchmark, we'd use an actual Tree-sitter language
    // For now, we'll create a placeholder that shows the structure

    group.bench_function("small_input", |b| {
        let mut parser = CParser::new();
        // parser.set_language(language).unwrap();

        b.iter(|| {
            // let _tree = parser.parse(black_box(SIMPLE_GRAMMAR_SOURCE), None);
            black_box(SIMPLE_GRAMMAR_SOURCE);
        });
    });

    let medium_source = generate_large_source(100);
    group.bench_function("medium_input", |b| {
        let mut parser = CParser::new();

        b.iter(|| {
            // let _tree = parser.parse(black_box(&medium_source), None);
            black_box(&medium_source);
        });
    });

    let large_source = generate_large_source(1000);
    group.bench_function("large_input", |b| {
        let mut parser = CParser::new();

        b.iter(|| {
            // let _tree = parser.parse(black_box(&large_source), None);
            black_box(&large_source);
        });
    });

    group.finish();
}

fn bench_lexer_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("lexer");

    // Benchmark just the lexing phase
    group.bench_function("tokenize_small", |b| {
        let source = SIMPLE_GRAMMAR_SOURCE.as_bytes();

        b.iter(|| {
            let mut position = 0;
            let mut tokens = Vec::new();

            while position < source.len() {
                // Simple lexer logic
                if source[position].is_ascii_whitespace() {
                    let mut len = 1;
                    while position + len < source.len()
                        && source[position + len].is_ascii_whitespace()
                    {
                        len += 1;
                    }
                    tokens.push((1u16, len));
                    position += len;
                } else if source[position].is_ascii_digit() {
                    let mut len = 1;
                    while position + len < source.len() && source[position + len].is_ascii_digit() {
                        len += 1;
                    }
                    tokens.push((2u16, len));
                    position += len;
                } else {
                    tokens.push((source[position] as u16, 1));
                    position += 1;
                }
            }

            black_box(tokens);
        });
    });

    group.finish();
}

fn bench_table_lookup(c: &mut Criterion) {
    let mut group = c.benchmark_group("table_lookup");

    // Benchmark parse table lookups
    group.bench_function("action_lookup", |b| {
        // Simulate compressed action table
        let table: Vec<u16> = (0..1000).map(|i| i as u16).collect();
        let state = 42u16;
        let symbol = 7u16;

        b.iter(|| {
            // Simulate table lookup
            let offset = (state as usize * 2) % table.len();
            let entry = table.get(offset).copied().unwrap_or(0);
            let action = if entry == 0xFFFF {
                "accept"
            } else if entry == 0xFFFE {
                "error"
            } else if entry & 0x8000 != 0 {
                "reduce"
            } else {
                "shift"
            };
            black_box(action);
        });
    });

    group.bench_function("goto_lookup", |b| {
        // Simulate goto table
        let table: Vec<u16> = (0..1000).map(|i| (i % 100) as u16).collect();
        let state = 42u16;
        let symbol = 7u16;

        b.iter(|| {
            let offset = ((state as usize) * 10 + (symbol as usize)) % table.len();
            let next_state = table.get(offset).copied().unwrap_or(0);
            black_box(next_state);
        });
    });

    group.finish();
}

fn bench_error_recovery(c: &mut Criterion) {
    let mut group = c.benchmark_group("error_recovery");

    // Benchmark error recovery strategies
    let error_source = r#"
        (1 + 2 * 3  // Missing closing paren
        4 + + 5     // Double operator
        (6 * (7 + ) // Missing operand
        8 9 10      // Missing operators
        + 11 * 12   // Leading operator
    "#;

    group.bench_function("skip_token", |b| {
        let mut parser = PureParser::new();

        b.iter(|| {
            let _result = parser.parse_string(black_box(error_source));
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_pure_rust_parser,
    bench_tree_sitter_c_parser,
    bench_lexer_performance,
    bench_table_lookup,
    bench_error_recovery
);
criterion_main!(benches);
