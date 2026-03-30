use criterion::{Criterion, black_box, criterion_group, criterion_main};

use adze_concurrency_map_core::{
    ParallelPartitionPlan, bounded_parallel_map, normalized_concurrency,
};

fn bench_normalized_concurrency(c: &mut Criterion) {
    c.bench_function("normalized_concurrency_zero", |b| {
        b.iter(|| black_box(normalized_concurrency(black_box(0))));
    });

    c.bench_function("normalized_concurrency_one", |b| {
        b.iter(|| black_box(normalized_concurrency(black_box(1))));
    });

    c.bench_function("normalized_concurrency_large", |b| {
        b.iter(|| black_box(normalized_concurrency(black_box(64))));
    });
}

fn bench_partition_plan(c: &mut Criterion) {
    c.bench_function("partition_plan_empty", |b| {
        b.iter(|| black_box(ParallelPartitionPlan::for_item_count(black_box(0), 4)));
    });

    c.bench_function("partition_plan_small", |b| {
        b.iter(|| black_box(ParallelPartitionPlan::for_item_count(black_box(10), 4)));
    });

    c.bench_function("partition_plan_large", |b| {
        b.iter(|| black_box(ParallelPartitionPlan::for_item_count(black_box(10000), 8)));
    });

    c.bench_function("partition_plan_concurrency_exceeds_items", |b| {
        b.iter(|| black_box(ParallelPartitionPlan::for_item_count(black_box(5), 100)));
    });
}

fn bench_bounded_parallel_map(c: &mut Criterion) {
    c.bench_function("bounded_map_empty", |b| {
        b.iter(|| {
            let result: Vec<i32> = bounded_parallel_map(black_box(vec![]), 4, |x: i32| x * 2);
            black_box(result)
        });
    });

    c.bench_function("bounded_map_single", |b| {
        b.iter(|| {
            let result = bounded_parallel_map(black_box(vec![42]), 4, |x: i32| x * 2);
            black_box(result)
        });
    });

    c.bench_function("bounded_map_small_10", |b| {
        let input: Vec<i32> = (0..10).collect();
        b.iter(|| {
            let result = bounded_parallel_map(black_box(input.clone()), 4, |x: i32| x * 2);
            black_box(result)
        });
    });

    c.bench_function("bounded_map_medium_100", |b| {
        let input: Vec<i32> = (0..100).collect();
        b.iter(|| {
            let result = bounded_parallel_map(black_box(input.clone()), 4, |x: i32| x * 2);
            black_box(result)
        });
    });

    c.bench_function("bounded_map_large_1000", |b| {
        let input: Vec<i32> = (0..1000).collect();
        b.iter(|| {
            let result = bounded_parallel_map(black_box(input.clone()), 8, |x: i32| x * 2);
            black_box(result)
        });
    });

    c.bench_function("bounded_map_zero_concurrency", |b| {
        let input: Vec<i32> = (0..100).collect();
        b.iter(|| {
            let result = bounded_parallel_map(black_box(input.clone()), 0, |x: i32| x * 2);
            black_box(result)
        });
    });
}

criterion_group!(
    benches,
    bench_normalized_concurrency,
    bench_partition_plan,
    bench_bounded_parallel_map,
);
criterion_main!(benches);
