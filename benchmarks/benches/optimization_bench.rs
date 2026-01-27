use criterion::{Criterion, black_box, criterion_group, criterion_main};
use rust_sitter::stack_pool::StackPool;
use rust_sitter::arena_allocator::{TreeArena, TreeNode};

fn benchmark_stack_pool(c: &mut Criterion) {
    let mut group = c.benchmark_group("stack_pool");

    // Benchmark pooled vs non-pooled stack operations
    group.bench_function("without_pool", |b| {
        b.iter(|| {
            let mut stacks = Vec::new();
            for i in 0..100 {
                let mut stack = Vec::with_capacity(256);
                for j in 0..50 {
                    stack.push(i * 100 + j);
                }
                stacks.push(stack);
            }
            black_box(stacks)
        });
    });

    group.bench_function("with_pool", |b| {
        let pool = StackPool::new(64);
        b.iter(|| {
            let mut stacks = Vec::new();
            for i in 0..100 {
                let mut stack = pool.acquire();
                for j in 0..50 {
                    stack.push(i * 100 + j);
                }
                stacks.push(stack);
            }
            // Return stacks to pool
            for stack in stacks {
                pool.release(stack);
            }
        });
    });

    // Benchmark fork operations with pool
    group.bench_function("fork_with_pool", |b| {
        let pool = StackPool::new(128);
        let source = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];

        b.iter(|| {
            let mut forks = Vec::new();
            for _ in 0..50 {
                let fork = pool.clone_stack(&source);
                forks.push(fork);
            }
            // Return to pool
            for fork in forks {
                pool.release(fork);
            }
        });
    });

    group.finish();
}

fn benchmark_combined_optimizations(c: &mut Criterion) {
    let mut group = c.benchmark_group("combined_optimizations");

    // Simulate a parsing workload with both optimizations
    group.bench_function("parse_simulation", |b| {
        // Use iter_with_setup to provide fresh pool and arena for each iteration
        // This ensures the arena doesn't grow indefinitely
        b.iter_with_setup(
            || (StackPool::new(32), TreeArena::with_capacity(512)),
            |(pool, mut arena)| {
                // Simulate parsing with forks
                let mut stacks = Vec::new();
                let mut nodes = Vec::new();

                // Initial stack
                let mut stack = pool.acquire();
                stack.push(0);

                // Simulate 100 parsing steps
                for step in 0..100 {
                    // Occasionally fork (simulate ambiguity)
                    if step % 10 == 0 && stacks.len() < 10 {
                        let fork = pool.clone_stack(&stack);
                        stacks.push(fork);
                    }

                    // Allocate parse nodes
                    let node = arena.alloc(TreeNode::leaf(step as i32));
                    nodes.push(node);

                    // Update stack
                    stack.push(step);

                    // Occasionally reduce (pop from stack)
                    if step % 5 == 0 && stack.len() > 1 {
                        stack.pop();
                    }
                }

                // Clean up
                pool.release(stack);
                for s in stacks {
                    pool.release(s);
                }

                black_box(nodes)
            },
        );
    });

    group.finish();
}

fn benchmark_memory_patterns(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_patterns");

    // Test different allocation patterns
    group.bench_function("small_frequent", |b| {
        b.iter(|| {
            let mut vecs = Vec::new();
            for _ in 0..10000 {
                vecs.push(vec![0u8; 8]);
            }
            black_box(vecs)
        });
    });

    group.bench_function("large_infrequent", |b| {
        b.iter(|| {
            let mut vecs = Vec::new();
            for _ in 0..10 {
                vecs.push(vec![0u8; 8192]);
            }
            black_box(vecs)
        });
    });

    group.bench_function("mixed_sizes", |b| {
        b.iter(|| {
            let mut vecs = Vec::new();
            for i in 0..1000 {
                let size = if i % 10 == 0 { 1024 } else { 16 };
                vecs.push(vec![0u8; size]);
            }
            black_box(vecs)
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    benchmark_stack_pool,
    benchmark_combined_optimizations,
    benchmark_memory_patterns
);
criterion_main!(benches);
