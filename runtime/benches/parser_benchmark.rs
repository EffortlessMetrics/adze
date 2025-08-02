use criterion::{Criterion, black_box, criterion_group, criterion_main};
use rust_sitter::tree_sitter::Parser;

// Simple arithmetic grammar for benchmarking
#[rust_sitter::grammar("benchmark")]
mod grammar {
    #[rust_sitter::language]
    pub enum Expression {
        Number(
            #[rust_sitter::leaf(pattern = r"\d+", transform = |v| v.parse::<i32>().unwrap())] i32,
        ),
        #[rust_sitter::prec_left(1)]
        Add(
            Box<Expression>,
            #[rust_sitter::leaf(text = "+")] (),
            Box<Expression>,
        ),
        #[rust_sitter::prec_left(2)]
        Multiply(
            Box<Expression>,
            #[rust_sitter::leaf(text = "*")] (),
            Box<Expression>,
        ),
        #[rust_sitter::prec(3)]
        Parenthesized(
            #[rust_sitter::leaf(text = "(")] (),
            Box<Expression>,
            #[rust_sitter::leaf(text = ")")] (),
        ),
    }

    #[rust_sitter::extra]
    pub struct Whitespace {
        #[rust_sitter::leaf(pattern = r"\s")]
        _whitespace: (),
    }
}

fn benchmark_simple_expression(c: &mut Criterion) {
    let parser = Parser::<grammar::Expression>::new();

    c.bench_function("parse_simple_expr", |b| {
        b.iter(|| {
            let _tree = parser.parse(black_box("1 + 2 * 3"), None);
        })
    });
}

fn benchmark_complex_expression(c: &mut Criterion) {
    let parser = Parser::<grammar::Expression>::new();
    let input = "1 + 2 * 3 + 4 * (5 + 6) * 7 + 8 * 9 + 10";

    c.bench_function("parse_complex_expr", |b| {
        b.iter(|| {
            let _tree = parser.parse(black_box(input), None);
        })
    });
}

fn benchmark_deeply_nested(c: &mut Criterion) {
    let parser = Parser::<grammar::Expression>::new();
    let input = "((((((((((1 + 2) * 3) + 4) * 5) + 6) * 7) + 8) * 9) + 10) * 11)";

    c.bench_function("parse_deeply_nested", |b| {
        b.iter(|| {
            let _tree = parser.parse(black_box(input), None);
        })
    });
}

fn benchmark_large_expression(c: &mut Criterion) {
    let parser = Parser::<grammar::Expression>::new();
    // Generate a large expression
    let mut input = String::new();
    for i in 0..100 {
        if i > 0 {
            input.push_str(" + ");
        }
        input.push_str(&i.to_string());
        if i % 10 == 0 && i > 0 {
            input.push_str(" * 2");
        }
    }

    c.bench_function("parse_large_expr", |b| {
        b.iter(|| {
            let _tree = parser.parse(black_box(&input), None);
        })
    });
}

criterion_group!(
    benches,
    benchmark_simple_expression,
    benchmark_complex_expression,
    benchmark_deeply_nested,
    benchmark_large_expression
);
criterion_main!(benches);
