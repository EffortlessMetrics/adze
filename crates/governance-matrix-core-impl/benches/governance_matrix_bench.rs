//! Benchmarks for governance-matrix-core-impl hot-path functions.
//!
//! Measures performance of matrix operations used in governance.

use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;

use adze_governance_matrix_core_impl::{
    BddGovernanceMatrix, BddPhase, GLR_CONFLICT_PRESERVATION_GRID, ParserFeatureProfile,
    bdd_governance_snapshot, bdd_progress_report, bdd_progress_report_with_profile,
    bdd_progress_status_line, describe_backend_for_conflicts,
};

fn bench_bdd_governance_snapshot(c: &mut Criterion) {
    c.bench_function("bdd_governance_snapshot", |b| {
        let profile = ParserFeatureProfile::current();
        b.iter(|| {
            black_box(bdd_governance_snapshot(
                black_box(BddPhase::Core),
                black_box(GLR_CONFLICT_PRESERVATION_GRID),
                black_box(profile),
            ))
        });
    });
}

fn bench_matrix_snapshot(c: &mut Criterion) {
    c.bench_function("matrix_snapshot", |b| {
        let profile = ParserFeatureProfile::current();
        let matrix = BddGovernanceMatrix::standard(profile);
        b.iter(|| black_box(black_box(&matrix).snapshot()));
    });
}

fn bench_matrix_is_fully_implemented(c: &mut Criterion) {
    c.bench_function("matrix_is_fully_implemented", |b| {
        let profile = ParserFeatureProfile::current();
        let matrix = BddGovernanceMatrix::standard(profile);
        b.iter(|| black_box(black_box(&matrix).is_fully_implemented()));
    });
}

fn bench_matrix_report(c: &mut Criterion) {
    c.bench_function("matrix_report", |b| {
        let profile = ParserFeatureProfile::current();
        let matrix = BddGovernanceMatrix::standard(profile);
        b.iter(|| black_box(black_box(&matrix).report("Benchmark Report")));
    });
}

fn bench_matrix_status_line(c: &mut Criterion) {
    c.bench_function("matrix_status_line", |b| {
        let profile = ParserFeatureProfile::current();
        let matrix = BddGovernanceMatrix::standard(profile);
        b.iter(|| black_box(black_box(&matrix).status_line()));
    });
}

fn bench_bdd_progress_report(c: &mut Criterion) {
    c.bench_function("bdd_progress_report", |b| {
        b.iter(|| {
            black_box(bdd_progress_report(
                black_box(BddPhase::Core),
                black_box(GLR_CONFLICT_PRESERVATION_GRID),
                black_box("Benchmark"),
            ))
        });
    });
}

fn bench_bdd_progress_report_with_profile(c: &mut Criterion) {
    c.bench_function("bdd_progress_report_with_profile", |b| {
        let profile = ParserFeatureProfile::current();
        b.iter(|| {
            black_box(bdd_progress_report_with_profile(
                black_box(BddPhase::Core),
                black_box(GLR_CONFLICT_PRESERVATION_GRID),
                black_box("Benchmark"),
                black_box(profile),
            ))
        });
    });
}

fn bench_bdd_progress_status_line(c: &mut Criterion) {
    c.bench_function("bdd_progress_status_line", |b| {
        let profile = ParserFeatureProfile::current();
        b.iter(|| {
            black_box(bdd_progress_status_line(
                black_box(BddPhase::Core),
                black_box(GLR_CONFLICT_PRESERVATION_GRID),
                black_box(profile),
            ))
        });
    });
}

fn bench_describe_backend_for_conflicts(c: &mut Criterion) {
    c.bench_function("describe_backend_for_conflicts", |b| {
        let profile = ParserFeatureProfile::current();
        b.iter(|| black_box(describe_backend_for_conflicts(black_box(profile))));
    });
}

fn bench_matrix_new_vs_standard(c: &mut Criterion) {
    c.bench_function("matrix_new_constructor", |b| {
        let profile = ParserFeatureProfile::current();
        b.iter(|| {
            black_box(BddGovernanceMatrix::new(
                black_box(BddPhase::Core),
                black_box(profile),
                black_box(GLR_CONFLICT_PRESERVATION_GRID),
            ))
        });
    });
}

criterion_group!(
    benches,
    bench_bdd_governance_snapshot,
    bench_matrix_snapshot,
    bench_matrix_is_fully_implemented,
    bench_matrix_report,
    bench_matrix_status_line,
    bench_bdd_progress_report,
    bench_bdd_progress_report_with_profile,
    bench_bdd_progress_status_line,
    bench_describe_backend_for_conflicts,
    bench_matrix_new_vs_standard,
);

criterion_main!(benches);
