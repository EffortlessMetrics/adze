use criterion::{Criterion, criterion_group, criterion_main};

use adze_stack_pool_core::StackPool;
use std::hint::black_box;

fn bench_sequential_push_pop(c: &mut Criterion) {
    c.bench_function("acquire_release_cycle_100", |b| {
        let pool: StackPool<u32> = StackPool::new(16);
        b.iter(|| {
            for _ in 0..100 {
                let mut stack = pool.acquire();
                stack.push(black_box(42));
                pool.release(stack);
            }
        });
    });
}

fn bench_acquire_from_empty(c: &mut Criterion) {
    c.bench_function("acquire_empty_pool", |b| {
        let pool: StackPool<u32> = StackPool::new(16);
        b.iter(|| {
            let stack = pool.acquire();
            black_box(&stack);
            pool.release(stack);
            pool.clear();
        });
    });
}

fn bench_acquire_with_capacity(c: &mut Criterion) {
    c.bench_function("acquire_with_capacity_hit", |b| {
        let pool: StackPool<u32> = StackPool::new(16);
        // Pre-fill pool with various capacities.
        for cap in [64, 128, 256, 512] {
            pool.release(Vec::with_capacity(cap));
        }
        b.iter(|| {
            let stack = pool.acquire_with_capacity(black_box(100));
            black_box(&stack);
            pool.release(stack);
        });
    });
}

fn bench_clone_stack(c: &mut Criterion) {
    let pool: StackPool<u32> = StackPool::new(16);
    let source: Vec<u32> = (0..256).collect();

    c.bench_function("clone_stack_256_elements", |b| {
        b.iter(|| {
            let cloned = pool.clone_stack(black_box(&source));
            black_box(&cloned);
            pool.release(cloned);
        });
    });
}

fn bench_pool_at_capacity(c: &mut Criterion) {
    c.bench_function("release_at_capacity", |b| {
        let pool: StackPool<u32> = StackPool::new(4);
        // Fill pool to capacity.
        for _ in 0..4 {
            pool.release(Vec::with_capacity(64));
        }
        b.iter(|| {
            // This release should be rejected (pool full).
            let extra: Vec<u32> = Vec::with_capacity(64);
            pool.release(black_box(extra));
        });
    });
}

fn bench_random_access_pattern(c: &mut Criterion) {
    c.bench_function("mixed_acquire_release_50", |b| {
        let pool: StackPool<u32> = StackPool::new(16);
        b.iter(|| {
            let mut held = Vec::new();
            for i in 0..50 {
                if i % 3 == 0 && !held.is_empty() {
                    let stack = held.pop().unwrap();
                    pool.release(stack);
                } else {
                    let mut stack = pool.acquire();
                    stack.push(black_box(i as u32));
                    held.push(stack);
                }
            }
            for stack in held {
                pool.release(stack);
            }
        });
    });
}

criterion_group!(
    benches,
    bench_sequential_push_pop,
    bench_acquire_from_empty,
    bench_acquire_with_capacity,
    bench_clone_stack,
    bench_pool_at_capacity,
    bench_random_access_pattern,
);
criterion_main!(benches);
