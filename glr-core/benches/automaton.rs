use adze_glr_core::{FirstFollowSets, build_lr1_automaton};
use adze_ir::builder::GrammarBuilder;
use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};

fn small_nullable() -> adze_ir::Grammar {
    GrammarBuilder::new("small_nullable")
        .token("IDENT", r"[a-zA-Z_][a-zA-Z0-9_]*")
        .rule("module", vec![]) // ε
        .rule("module", vec!["IDENT"])
        .start("module")
        .build()
}

fn small_nonnullable() -> adze_ir::Grammar {
    GrammarBuilder::new("small_nonnullable")
        .token("IDENT", r"[a-zA-Z_][a-zA-Z0-9_]*")
        .rule("program", vec!["IDENT"])
        .start("program")
        .build()
}

fn medium_expression() -> adze_ir::Grammar {
    GrammarBuilder::new("medium_expr")
        .token("NUMBER", r"\d+")
        .token("PLUS", r"\+")
        .token("TIMES", r"\*")
        .token("LPAREN", r"\(")
        .token("RPAREN", r"\)")
        .rule("expr", vec!["term"])
        .rule("expr", vec!["expr", "PLUS", "term"])
        .rule("term", vec!["factor"])
        .rule("term", vec!["term", "TIMES", "factor"])
        .rule("factor", vec!["NUMBER"])
        .rule("factor", vec!["LPAREN", "expr", "RPAREN"])
        .start("expr")
        .build()
}

fn bench_automaton(c: &mut Criterion) {
    let cases = [
        ("nullable", small_nullable()),
        ("nonnullable", small_nonnullable()),
        ("expression", medium_expression()),
    ];

    let mut group = c.benchmark_group("lr1_automaton_build");

    for (name, grammar) in cases {
        group.bench_function(BenchmarkId::new("full_build", name), |b| {
            b.iter(|| {
                let ff = FirstFollowSets::compute(&grammar).expect("compute first/follow sets");
                build_lr1_automaton(&grammar, &ff).expect("build automaton");
            })
        });
    }

    group.finish();

    // Separate benchmark for just FIRST/FOLLOW computation
    let mut ff_group = c.benchmark_group("first_follow");

    for (name, grammar) in [
        ("nullable", small_nullable()),
        ("nonnullable", small_nonnullable()),
        ("expression", medium_expression()),
    ] {
        ff_group.bench_function(BenchmarkId::new("compute", name), |b| {
            b.iter(|| {
                let _ = FirstFollowSets::compute(&grammar);
            })
        });
    }

    ff_group.finish();
}

criterion_group!(benches, bench_automaton);
criterion_main!(benches);
