use criterion::{Criterion, criterion_group, criterion_main};

use adze_linecol_core::LineCol;
use std::hint::black_box;

fn make_ascii_input(size: usize) -> Vec<u8> {
    let line = b"hello world\n";
    line.iter().copied().cycle().take(size).collect()
}

fn make_multibyte_utf8_input(size: usize) -> Vec<u8> {
    let line = "café résumé naïve 日本語\n".as_bytes();
    line.iter().copied().cycle().take(size).collect()
}

fn bench_at_position(c: &mut Criterion) {
    let small = make_ascii_input(10);
    let medium = make_ascii_input(1024);
    let large = make_ascii_input(1_000_000);
    let multibyte = make_multibyte_utf8_input(1024);

    c.bench_function("at_position_small_10B", |b| {
        b.iter(|| LineCol::at_position(black_box(&small), small.len()));
    });
    c.bench_function("at_position_medium_1KB", |b| {
        b.iter(|| LineCol::at_position(black_box(&medium), medium.len()));
    });
    c.bench_function("at_position_large_1MB", |b| {
        b.iter(|| LineCol::at_position(black_box(&large), large.len()));
    });
    c.bench_function("at_position_multibyte_1KB", |b| {
        b.iter(|| LineCol::at_position(black_box(&multibyte), multibyte.len()));
    });
}

fn bench_process_byte(c: &mut Criterion) {
    let input = make_ascii_input(1024);

    c.bench_function("process_byte_stream_1KB", |b| {
        b.iter(|| {
            let mut tracker = LineCol::new();
            for i in 0..input.len() {
                let next = input.get(i + 1).copied();
                tracker.process_byte(black_box(input[i]), next, i);
            }
            black_box(tracker)
        });
    });
}

fn bench_column(c: &mut Criterion) {
    let tracker = LineCol::at_position(b"hello\nworld\nfoo\nbar\n", 18);

    c.bench_function("column_computation", |b| {
        b.iter(|| black_box(tracker).column(black_box(18)));
    });
}

criterion_group!(benches, bench_at_position, bench_process_byte, bench_column);
criterion_main!(benches);
