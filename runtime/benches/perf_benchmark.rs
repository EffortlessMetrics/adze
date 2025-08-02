// Performance benchmarks for rust-sitter optimizations
// Compares standard vs SIMD lexing and single vs parallel parsing

use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use rust_sitter::lexer::GrammarLexer;
use rust_sitter::parallel_parser::{ParallelConfig, ParallelParser};
use rust_sitter::parser_v3::Parser;
use rust_sitter::simd_lexer::SimdLexer;
use rust_sitter_glr_core::{Action, ParseTable};
use rust_sitter_ir::{Grammar, Rule, SymbolId, TokenPattern};
use std::time::Duration;

/// Create a test grammar for benchmarking
fn create_benchmark_grammar() -> (Grammar, ParseTable) {
    let mut grammar = Grammar::new("benchmark".to_string());

    // Add common token patterns
    let id_symbol = SymbolId(1);
    let number_symbol = SymbolId(2);
    let whitespace_symbol = SymbolId(3);
    let keyword_symbol = SymbolId(4);
    let operator_symbol = SymbolId(5);

    grammar.tokens.insert(
        id_symbol,
        rust_sitter_ir::Token {
            name: "identifier".to_string(),
            pattern: TokenPattern::Regex(r"[a-zA-Z_][a-zA-Z0-9_]*".to_string()),
            fragile: false,
        },
    );

    grammar.tokens.insert(
        number_symbol,
        rust_sitter_ir::Token {
            name: "number".to_string(),
            pattern: TokenPattern::Regex(r"\d+".to_string()),
            fragile: false,
        },
    );

    grammar.tokens.insert(
        whitespace_symbol,
        rust_sitter_ir::Token {
            name: "whitespace".to_string(),
            pattern: TokenPattern::Regex(r"\s+".to_string()),
            fragile: false,
        },
    );

    grammar.tokens.insert(
        keyword_symbol,
        rust_sitter_ir::Token {
            name: "function".to_string(),
            pattern: TokenPattern::String("function".to_string()),
            fragile: false,
        },
    );

    grammar.tokens.insert(
        operator_symbol,
        rust_sitter_ir::Token {
            name: "plus".to_string(),
            pattern: TokenPattern::String("+".to_string()),
            fragile: false,
        },
    );

    // Create a simple parse table
    let mut action_table = vec![vec![Action::Error; 6]; 10];
    action_table[0][id_symbol.0 as usize] = Action::Shift(rust_sitter_ir::StateId(1));
    action_table[0][number_symbol.0 as usize] = Action::Shift(rust_sitter_ir::StateId(2));
    action_table[0][keyword_symbol.0 as usize] = Action::Shift(rust_sitter_ir::StateId(3));

    let parse_table = ParseTable {
        action_table,
        goto_table: vec![],
        symbol_metadata: vec![],
        state_count: 10,
        symbol_count: 6,
    };

    (grammar, parse_table)
}

/// Generate test input of specified size
fn generate_test_input(size_kb: usize) -> String {
    let mut input = String::new();
    let line =
        "function calculate_value_12345(param_x, param_y) { return param_x + param_y + 42; }\n";

    let lines_needed = (size_kb * 1024) / line.len();
    for _ in 0..lines_needed {
        input.push_str(line);
    }

    input
}

/// Benchmark standard lexer
fn bench_standard_lexer(c: &mut Criterion) {
    let (grammar, _) = create_benchmark_grammar();
    let patterns: Vec<_> = grammar
        .tokens
        .iter()
        .map(|(&id, token)| (id, token.pattern.clone()))
        .collect();

    let mut group = c.benchmark_group("lexer/standard");
    group.measurement_time(Duration::from_secs(10));

    for size_kb in [1, 10, 100, 1000].iter() {
        let input = generate_test_input(*size_kb);
        let input_bytes = input.as_bytes();

        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}KB", size_kb)),
            &input_bytes,
            |b, input| {
                let lexer = GrammarLexer::new(&patterns);
                b.iter(|| {
                    let mut pos = 0;
                    let mut token_count = 0;
                    while pos < input.len() {
                        if let Ok(token) = lexer.next_token(input, pos) {
                            pos = token.end;
                            token_count += 1;
                        } else {
                            pos += 1;
                        }
                    }
                    black_box(token_count)
                });
            },
        );
    }

    group.finish();
}

/// Benchmark SIMD lexer
fn bench_simd_lexer(c: &mut Criterion) {
    let (grammar, _) = create_benchmark_grammar();
    let patterns: Vec<_> = grammar
        .tokens
        .iter()
        .map(|(&id, token)| (id, token.pattern.clone()))
        .collect();

    let mut group = c.benchmark_group("lexer/simd");
    group.measurement_time(Duration::from_secs(10));

    for size_kb in [1, 10, 100, 1000].iter() {
        let input = generate_test_input(*size_kb);
        let input_bytes = input.as_bytes();

        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}KB", size_kb)),
            &input_bytes,
            |b, input| {
                let lexer = SimdLexer::new(&patterns);
                b.iter(|| {
                    let mut pos = 0;
                    let mut token_count = 0;
                    while pos < input.len() {
                        if let Some(token) = lexer.scan(input, pos) {
                            pos = token.end;
                            token_count += 1;
                        } else {
                            pos += 1;
                        }
                    }
                    black_box(token_count)
                });
            },
        );
    }

    group.finish();
}

/// Benchmark single-threaded parser
fn bench_single_parser(c: &mut Criterion) {
    let (grammar, parse_table) = create_benchmark_grammar();

    let mut group = c.benchmark_group("parser/single");
    group.measurement_time(Duration::from_secs(10));

    for size_kb in [1, 10, 100].iter() {
        let input = generate_test_input(*size_kb);

        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}KB", size_kb)),
            &input,
            |b, input| {
                b.iter(|| {
                    let mut parser = Parser::new(grammar.clone(), parse_table.clone());
                    let result = parser.parse(input);
                    black_box(result)
                });
            },
        );
    }

    group.finish();
}

/// Benchmark parallel parser
fn bench_parallel_parser(c: &mut Criterion) {
    let (grammar, parse_table) = create_benchmark_grammar();

    let mut group = c.benchmark_group("parser/parallel");
    group.measurement_time(Duration::from_secs(10));

    for size_kb in [100, 500, 1000].iter() {
        let input = generate_test_input(*size_kb);

        let config = ParallelConfig {
            min_file_size: 50_000, // 50KB minimum
            chunk_size: 50_000,    // 50KB chunks
            num_threads: 0,        // Use all cores
            enable_caching: true,
        };

        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}KB", size_kb)),
            &input,
            |b, input| {
                let parser =
                    ParallelParser::new(grammar.clone(), parse_table.clone(), config.clone());
                b.iter(|| {
                    let result = parser.parse(input);
                    black_box(result)
                });
            },
        );
    }

    group.finish();
}

/// Compare lexer performance
fn bench_lexer_comparison(c: &mut Criterion) {
    let (grammar, _) = create_benchmark_grammar();
    let patterns: Vec<_> = grammar
        .tokens
        .iter()
        .map(|(&id, token)| (id, token.pattern.clone()))
        .collect();

    let mut group = c.benchmark_group("lexer_comparison");
    group.measurement_time(Duration::from_secs(10));

    let input = generate_test_input(100); // 100KB input
    let input_bytes = input.as_bytes();

    group.bench_function("standard", |b| {
        let lexer = GrammarLexer::new(&patterns);
        b.iter(|| {
            let mut pos = 0;
            let mut token_count = 0;
            while pos < input_bytes.len() {
                if let Ok(token) = lexer.next_token(input_bytes, pos) {
                    pos = token.end;
                    token_count += 1;
                } else {
                    pos += 1;
                }
            }
            black_box(token_count)
        });
    });

    group.bench_function("simd", |b| {
        let lexer = SimdLexer::new(&patterns);
        b.iter(|| {
            let mut pos = 0;
            let mut token_count = 0;
            while pos < input_bytes.len() {
                if let Some(token) = lexer.scan(input_bytes, pos) {
                    pos = token.end;
                    token_count += 1;
                } else {
                    pos += 1;
                }
            }
            black_box(token_count)
        });
    });

    group.finish();
}

/// Compare parser performance
fn bench_parser_comparison(c: &mut Criterion) {
    let (grammar, parse_table) = create_benchmark_grammar();

    let mut group = c.benchmark_group("parser_comparison");
    group.measurement_time(Duration::from_secs(10));

    let input = generate_test_input(500); // 500KB input

    group.bench_function("single_threaded", |b| {
        b.iter(|| {
            let mut parser = Parser::new(grammar.clone(), parse_table.clone());
            let result = parser.parse(&input);
            black_box(result)
        });
    });

    group.bench_function("parallel", |b| {
        let config = ParallelConfig {
            min_file_size: 50_000,
            chunk_size: 50_000,
            num_threads: 0,
            enable_caching: true,
        };
        let parser = ParallelParser::new(grammar.clone(), parse_table.clone(), config);
        b.iter(|| {
            let result = parser.parse(&input);
            black_box(result)
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_standard_lexer,
    bench_simd_lexer,
    bench_single_parser,
    bench_parallel_parser,
    bench_lexer_comparison,
    bench_parser_comparison
);

criterion_main!(benches);
