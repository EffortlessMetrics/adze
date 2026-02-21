use adze_ir::{
    Associativity, Grammar, PrecedenceKind, ProductionId, Rule, Symbol, SymbolId, Token,
    TokenPattern,
};
use criterion::{Criterion, black_box, criterion_group, criterion_main};
#[allow(unused_imports)]
use std::collections::HashMap;

/// Build a simple ambiguous grammar: S -> S S | 'a'
/// This creates exponential ambiguity (Catalan number of parse trees)
fn build_ambiguous_grammar() -> Grammar {
    let mut grammar = Grammar::new("Ambiguous".to_string());

    // Define symbols
    let s_id = SymbolId(0); // Start symbol
    let a_id = SymbolId(1); // Terminal 'a'

    // Mark 'a' as a token
    grammar.tokens.insert(
        a_id,
        Token {
            name: "a".to_string(),
            pattern: TokenPattern::String("a".to_string()),
            fragile: false,
        },
    );

    // Add ambiguous rules: S -> S S | 'a'
    let rules = vec![
        Rule {
            lhs: s_id,
            rhs: vec![Symbol::NonTerminal(s_id), Symbol::NonTerminal(s_id)],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(0),
        },
        Rule {
            lhs: s_id,
            rhs: vec![Symbol::Terminal(a_id)],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(1),
        },
    ];

    grammar.rules.insert(s_id, rules);

    grammar
}

/// Build an expression grammar with precedence: E -> E + E | E * E | num
/// This tests conflict resolution with precedence
fn build_expression_grammar() -> Grammar {
    let mut grammar = Grammar::new("Expression".to_string());

    // Define symbols
    let e_id = SymbolId(0); // Expression
    let plus_id = SymbolId(1); // +
    let mult_id = SymbolId(2); // *
    let num_id = SymbolId(3); // number

    // Mark terminals as tokens
    grammar.tokens.insert(
        plus_id,
        Token {
            name: "plus".to_string(),
            pattern: TokenPattern::String("+".to_string()),
            fragile: false,
        },
    );
    grammar.tokens.insert(
        mult_id,
        Token {
            name: "mult".to_string(),
            pattern: TokenPattern::String("*".to_string()),
            fragile: false,
        },
    );
    grammar.tokens.insert(
        num_id,
        Token {
            name: "num".to_string(),
            pattern: TokenPattern::Regex("\\d+".to_string()),
            fragile: false,
        },
    );

    // Add expression rules with precedence
    let rules = vec![
        Rule {
            lhs: e_id,
            rhs: vec![
                Symbol::NonTerminal(e_id),
                Symbol::Terminal(plus_id),
                Symbol::NonTerminal(e_id),
            ],
            precedence: Some(PrecedenceKind::Static(1)), // Lower precedence
            associativity: Some(Associativity::Left),
            fields: vec![],
            production_id: ProductionId(0),
        },
        Rule {
            lhs: e_id,
            rhs: vec![
                Symbol::NonTerminal(e_id),
                Symbol::Terminal(mult_id),
                Symbol::NonTerminal(e_id),
            ],
            precedence: Some(PrecedenceKind::Static(2)), // Higher precedence
            associativity: Some(Associativity::Left),
            fields: vec![],
            production_id: ProductionId(1),
        },
        Rule {
            lhs: e_id,
            rhs: vec![Symbol::Terminal(num_id)],
            precedence: None,
            associativity: None,
            fields: vec![],
            production_id: ProductionId(2),
        },
    ];

    grammar.rules.insert(e_id, rules);

    grammar
}

/// Generate input string of 'a' tokens
fn generate_input(size: usize) -> Vec<u16> {
    // In real use, we'd tokenize; here just simulate token IDs
    vec![1; size] // All 'a' tokens (SymbolId(1))
}

fn benchmark_ambiguous_grammar(c: &mut Criterion) {
    let mut group = c.benchmark_group("glr_ambiguous");

    // Build the grammar once
    let _grammar = build_ambiguous_grammar();

    // Benchmark different input sizes
    for size in &[5, 10, 15, 20] {
        let _input = generate_input(*size);

        group.bench_function(format!("parse_{}_tokens", size), |b| {
            b.iter(|| {
                // TODO: Actually run GLR parser here once integrated
                // For now, simulate the workload
                let mut fork_count = 0;
                let mut merge_count = 0;

                // Simulate exponential fork behavior
                for i in 0..*size {
                    fork_count += 2_usize.pow(i.min(10) as u32);
                    if i > 0 && i % 2 == 0 {
                        merge_count += fork_count / 4;
                    }
                }

                black_box((fork_count, merge_count))
            });
        });
    }

    group.finish();
}

fn benchmark_expression_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("glr_expression");

    let _grammar = build_expression_grammar();

    // Test expression of form: 1 + 2 * 3 + 4 * 5 ...
    for num_ops in &[10, 50, 100, 500] {
        let label = format!("{}_operations", num_ops);

        group.bench_function(&label, |b| {
            b.iter(|| {
                // TODO: Parse actual expression once integrated
                // Simulate precedence resolution work
                let mut stack_ops = 0;
                let mut reduce_ops = 0;

                for i in 0..*num_ops {
                    stack_ops += 3; // Push, check precedence, maybe reduce
                    if i % 2 == 0 {
                        reduce_ops += 1;
                    }
                }

                black_box((stack_ops, reduce_ops))
            });
        });
    }

    group.finish();
}

fn benchmark_fork_merge_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("glr_fork_merge");

    // Test raw fork performance with different stack depths
    group.bench_function("shallow_fork_10", |b| {
        let stack = vec![1, 2, 3, 4, 5];
        b.iter(|| {
            let mut forks = Vec::with_capacity(10);
            for _ in 0..10 {
                forks.push(stack.clone());
            }
            black_box(forks)
        });
    });

    group.bench_function("deep_fork_10", |b| {
        let stack: Vec<u16> = (0..100).collect();
        b.iter(|| {
            let mut forks = Vec::with_capacity(10);
            for _ in 0..10 {
                forks.push(stack.clone());
            }
            black_box(forks)
        });
    });

    group.bench_function("very_deep_fork_10", |b| {
        let stack: Vec<u16> = (0..1000).collect();
        b.iter(|| {
            let mut forks = Vec::with_capacity(10);
            for _ in 0..10 {
                forks.push(stack.clone());
            }
            black_box(forks)
        });
    });

    // Test merge scenarios
    group.bench_function("merge_compatible_stacks", |b| {
        b.iter(|| {
            let mut stacks = vec![vec![1, 2, 3, 4], vec![1, 2, 3, 5], vec![1, 2, 3, 6]];

            // Simulate checking if stacks can merge (same prefix)
            let can_merge = stacks.windows(2).all(|w| {
                !w[0].is_empty()
                    && !w[1].is_empty()
                    && w[0][..w[0].len() - 1] == w[1][..w[1].len() - 1]
            });

            if can_merge {
                stacks.truncate(1); // Merge into one
            }

            black_box(stacks)
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    benchmark_ambiguous_grammar,
    benchmark_expression_parsing,
    benchmark_fork_merge_operations
);
criterion_main!(benches);
