use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};

/// Generate test Python code of various sizes
fn generate_python_code(lines: usize) -> String {
    let base = r#"
def process_data(items):
    results = []
    for item in items:
        if item > 0:
            results.append(item * 2)
    return results

class DataHandler:
    def __init__(self):
        self.data = []
    
    def add(self, value):
        self.data.append(value)
    
    def process(self):
        return [x * 2 for x in self.data if x > 0]
"#;

    let mut code = String::new();
    let iterations = lines / 20; // Each base block is ~20 lines

    for i in 0..iterations {
        code.push_str(&base.replace("process_data", &format!("process_data_{}", i)));
        code.push_str(&base.replace("DataHandler", &format!("DataHandler_{}", i)));
    }

    code
}

fn benchmark_glr_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("glr_parsing");

    // Test different file sizes
    for size in &[100, 500, 1000, 5000] {
        let code = generate_python_code(*size);
        let label = format!("{}_lines", size);

        group.bench_with_input(
            BenchmarkId::new("parse_python", &label),
            &code,
            |b, source| {
                b.iter(|| {
                    // ❌ CRITICAL BUG: This is NOT parsing, just character counting!
                    //
                    // PROBLEM: This benchmark claims to measure "parse_python" performance,
                    // but actually measures character iteration speed (~0.1ns per char).
                    // This creates completely false performance claims like "815 MB/sec"
                    // that are 100x faster than real parsers because we're not parsing!
                    //
                    // IMPACT:
                    // - README claims "100x faster than Tree-sitter" based on this
                    // - Users adopt adze expecting this performance
                    // - Documentation shows false "118M tokens/sec" throughput
                    //
                    // TODO (HIGH PRIORITY - Issue #73):
                    // 1. Replace with actual Python parser once lexer is fixed
                    // 2. Remove all performance claims until real benchmarks exist
                    // 3. Add disclaimer that current benchmarks are mocks
                    // 4. Test with real Python grammar and source code
                    //
                    // REAL FIX NEEDED:
                    // ```rust
                    // let mut parser = Parser::new();
                    // parser.set_language(&PYTHON_LANGUAGE).unwrap();
                    // let tree = parser.parse(source, None).unwrap();
                    // black_box(tree)
                    // ```
                    //
                    // For now, simulate parsing workload (MOCK - NOT REAL PARSING)
                    let mut tokens = 0;
                    for char in source.chars() {
                        if char.is_alphanumeric() || char.is_whitespace() {
                            tokens += 1;
                        }
                    }
                    black_box(tokens)
                });
            },
        );
    }

    group.finish();
}

fn benchmark_fork_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("fork_operations");

    // ❌ CRITICAL BUG: These are NOT GLR operations, just Vec::clone()!
    //
    // PROBLEM: This benchmark claims to measure GLR "fork operations" but actually
    // measures simple Vec::clone() performance (~85ns). This has nothing to do with
    // the complex GLR parsing operations it pretends to benchmark.
    //
    // WHAT GLR FORKS ACTUALLY ARE:
    // - Parse state duplication when shift/reduce conflicts occur
    // - Grammar rule application with different precedence
    // - Parse stack management with LR(1) states and lookahead
    // - Symbol table handling and reduction operations
    // - Parse forest construction for ambiguous derivations
    //
    // WHAT THIS ACTUALLY MEASURES:
    // - Memory allocation for small integer vectors
    // - Simple clone operations with no parsing logic
    // - Basic Vec::push() performance
    //
    // TODO (MEDIUM PRIORITY - Issue #75):
    // Replace with real GLR fork benchmarks once parser works:
    // ```rust
    // let parser = GLRParser::new(ambiguous_grammar());
    // let forest = parser.parse("1 + 2 * 3"); // Creates real forks
    // black_box(forest.derivation_count())
    // ```
    //
    // Simulate different fork scenarios (MOCK - NOT REAL GLR)
    group.bench_function("single_fork", |b| {
        b.iter(|| {
            let mut stacks = vec![vec![1, 2, 3]];
            // Simulate fork (FAKE: just Vec::clone, not GLR fork)
            let forked = stacks[0].clone();
            stacks.push(forked);
            black_box(stacks)
        });
    });

    group.bench_function("multiple_forks_10", |b| {
        b.iter(|| {
            let mut stacks = vec![vec![1, 2, 3, 4, 5]];
            // ❌ FAKE GLR: This is just Vec operations, not grammar conflicts
            for _ in 0..10 {
                let forked = stacks[0].clone(); // NOT a GLR parse stack fork
                stacks.push(forked); // NOT adding parse states
            }
            black_box(stacks)
        });
    });

    group.bench_function("deep_stack_fork", |b| {
        let deep_stack: Vec<i32> = (0..1000).collect();
        b.iter(|| {
            let forked = deep_stack.clone();
            black_box(forked)
        });
    });

    group.finish();
}

fn benchmark_memory_allocation(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_allocation");

    // Test different allocation patterns
    group.bench_function("vec_push_small", |b| {
        b.iter(|| {
            let mut v = Vec::new();
            for i in 0..100 {
                v.push(i);
            }
            black_box(v)
        });
    });

    group.bench_function("vec_with_capacity", |b| {
        b.iter(|| {
            let mut v = Vec::with_capacity(100);
            for i in 0..100 {
                v.push(i);
            }
            black_box(v)
        });
    });

    group.bench_function("arena_simulation", |b| {
        b.iter(|| {
            // Simulate arena allocation pattern
            let mut arena = Vec::with_capacity(10000);
            for i in 0..1000 {
                arena.extend_from_slice(&[i; 10]);
            }
            black_box(arena)
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    benchmark_glr_parsing,
    benchmark_fork_operations,
    benchmark_memory_allocation
);
criterion_main!(benches);
