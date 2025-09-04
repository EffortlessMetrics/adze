//! Benchmarks for table compression performance
#![allow(clippy::let_and_return)]

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use rust_sitter_glr_core::{build_lr1_automaton, FirstFollowSets};
use rust_sitter_ir::builder::GrammarBuilder;
use rust_sitter_tablegen::{
    helpers::{collect_token_indices, eof_accepts_or_reduces},
    TableCompressor,
};

/// Benchmark small grammar compression
fn bench_small_grammar(c: &mut Criterion) {
    let grammar = GrammarBuilder::new("small")
        .token("A", "a")
        .token("B", "b")
        .rule("S", vec!["A", "B"])
        .start("S")
        .build();

    let first_follow = FirstFollowSets::compute(&grammar).unwrap();
    let parse_table = build_lr1_automaton(&grammar, &first_follow).unwrap();
    let token_indices = collect_token_indices(&grammar, &parse_table);
    let start_nullable = eof_accepts_or_reduces(&parse_table);

    c.bench_function("compress_small_grammar", |b| {
        b.iter(|| {
            let compressor = TableCompressor::new();
            let compressed = compressor.compress(
                black_box(&parse_table),
                black_box(&token_indices),
                black_box(start_nullable),
            );
            compressed
        })
    });
}

/// Benchmark medium-sized arithmetic grammar compression
fn bench_arithmetic_grammar(c: &mut Criterion) {
    let grammar = GrammarBuilder::new("arithmetic")
        .token("NUMBER", r"\d+")
        .token("+", "+")
        .token("-", "-")
        .token("*", "*")
        .token("/", "/")
        .token("(", "(")
        .token(")", ")")
        .extra("WHITESPACE")
        .token("WHITESPACE", r"[ \t\n]+")
        .rule_with_precedence(
            "expr",
            vec!["expr", "+", "expr"],
            1,
            rust_sitter_ir::Associativity::Left,
        )
        .rule_with_precedence(
            "expr",
            vec!["expr", "-", "expr"],
            1,
            rust_sitter_ir::Associativity::Left,
        )
        .rule_with_precedence(
            "expr",
            vec!["expr", "*", "expr"],
            2,
            rust_sitter_ir::Associativity::Left,
        )
        .rule_with_precedence(
            "expr",
            vec!["expr", "/", "expr"],
            2,
            rust_sitter_ir::Associativity::Left,
        )
        .rule("expr", vec!["(", "expr", ")"])
        .rule("expr", vec!["NUMBER"])
        .start("expr")
        .build();

    let first_follow = FirstFollowSets::compute(&grammar).unwrap();
    let parse_table = build_lr1_automaton(&grammar, &first_follow).unwrap();
    let token_indices = collect_token_indices(&grammar, &parse_table);
    let start_nullable = eof_accepts_or_reduces(&parse_table);

    c.bench_function("compress_arithmetic_grammar", |b| {
        b.iter(|| {
            let compressor = TableCompressor::new();
            let compressed = compressor.compress(
                black_box(&parse_table),
                black_box(&token_indices),
                black_box(start_nullable),
            );
            compressed
        })
    });
}

/// Benchmark Python-like grammar compression (nullable start)
fn bench_python_like_grammar(c: &mut Criterion) {
    let grammar = GrammarBuilder::python_like();
    let first_follow = FirstFollowSets::compute(&grammar).unwrap();
    let parse_table = build_lr1_automaton(&grammar, &first_follow).unwrap();
    let token_indices = collect_token_indices(&grammar, &parse_table);
    let start_nullable = eof_accepts_or_reduces(&parse_table);

    c.bench_function("compress_python_like_grammar", |b| {
        b.iter(|| {
            let compressor = TableCompressor::new();
            let compressed = compressor.compress(
                black_box(&parse_table),
                black_box(&token_indices),
                black_box(start_nullable),
            );
            compressed
        })
    });
}

/// Benchmark JavaScript-like grammar compression (non-nullable start)
fn bench_javascript_like_grammar(c: &mut Criterion) {
    let grammar = GrammarBuilder::javascript_like();
    let first_follow = FirstFollowSets::compute(&grammar).unwrap();
    let parse_table = build_lr1_automaton(&grammar, &first_follow).unwrap();
    let token_indices = collect_token_indices(&grammar, &parse_table);
    let start_nullable = eof_accepts_or_reduces(&parse_table);

    c.bench_function("compress_javascript_like_grammar", |b| {
        b.iter(|| {
            let compressor = TableCompressor::new();
            let compressed = compressor.compress(
                black_box(&parse_table),
                black_box(&token_indices),
                black_box(start_nullable),
            );
            compressed
        })
    });
}

/// Benchmark varying table sizes
fn bench_table_size_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("table_size_scaling");

    for size in [10, 20, 50, 100].iter() {
        // Create a grammar with N rules
        let mut builder = GrammarBuilder::new(&format!("size_{}", size));

        // Add tokens
        for i in 0..*size {
            builder = builder.token(&format!("T{}", i), &format!("t{}", i));
        }

        // Add rules
        builder = builder.rule("start", vec!["rule0"]);
        for i in 0..size - 1 {
            let rule_name = format!("rule{}", i);
            let next_rule = format!("rule{}", i + 1);
            let token = format!("T{}", i);
            builder = builder.rule(&rule_name, vec![&token, &next_rule]);
        }
        builder = builder.rule(
            &format!("rule{}", size - 1),
            vec![&format!("T{}", size - 1)],
        );

        let grammar = builder.start("start").build();
        let first_follow = FirstFollowSets::compute(&grammar).unwrap();
        let parse_table = build_lr1_automaton(&grammar, &first_follow).unwrap();
        let token_indices = collect_token_indices(&grammar, &parse_table);
        let start_nullable = eof_accepts_or_reduces(&parse_table);

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| {
                let compressor = TableCompressor::new();
                let compressed = compressor.compress(
                    black_box(&parse_table),
                    black_box(&token_indices),
                    black_box(start_nullable),
                );
                compressed
            })
        });
    }
    group.finish();
}

/// Benchmark compression with different options
fn bench_compression_options(c: &mut Criterion) {
    let grammar = GrammarBuilder::javascript_like();
    let first_follow = FirstFollowSets::compute(&grammar).unwrap();
    let parse_table = build_lr1_automaton(&grammar, &first_follow).unwrap();
    let token_indices = collect_token_indices(&grammar, &parse_table);
    let start_nullable = eof_accepts_or_reduces(&parse_table);

    let mut group = c.benchmark_group("compression_options");

    // Test with actual EOF optimization (based on grammar)
    group.bench_function("actual_eof_optimization", |b| {
        b.iter(|| {
            let compressor = TableCompressor::new();
            let compressed = compressor.compress(
                black_box(&parse_table),
                black_box(&token_indices),
                black_box(start_nullable), // Use actual nullable state
            );
            compressed
        })
    });

    // Test forcing no EOF optimization (even if grammar allows it)
    group.bench_function("forced_no_eof", |b| {
        b.iter(|| {
            let compressor = TableCompressor::new();
            let compressed = compressor.compress(
                black_box(&parse_table),
                black_box(&token_indices),
                black_box(false), // Force disabled
            );
            compressed
        })
    });

    group.finish();
}

/// Benchmark parse table generation time
fn bench_parse_table_generation(c: &mut Criterion) {
    let grammar = GrammarBuilder::javascript_like();

    c.bench_function("parse_table_generation", |b| {
        b.iter(|| {
            let first_follow = FirstFollowSets::compute(black_box(&grammar));
            let parse_table = build_lr1_automaton(black_box(&grammar), black_box(&first_follow));
            parse_table
        })
    });
}

criterion_group!(
    benches,
    bench_small_grammar,
    bench_arithmetic_grammar,
    bench_python_like_grammar,
    bench_javascript_like_grammar,
    bench_table_size_scaling,
    bench_compression_options,
    bench_parse_table_generation
);
criterion_main!(benches);
