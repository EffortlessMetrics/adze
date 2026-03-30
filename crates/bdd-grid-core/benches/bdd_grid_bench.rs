//! Benchmarks for bdd-grid-core hot-path functions.
//!
//! Measures performance of scenario lookup and progress reporting used in BDD framework.

use criterion::{Criterion, black_box, criterion_group, criterion_main};

use adze_bdd_grid_core::{
    BddPhase, BddScenario, BddScenarioStatus, GLR_CONFLICT_PRESERVATION_GRID, bdd_progress,
    bdd_progress_report,
};

fn bench_bdd_progress_core(c: &mut Criterion) {
    c.bench_function("bdd_progress_core_phase", |b| {
        b.iter(|| {
            black_box(bdd_progress(
                black_box(BddPhase::Core),
                black_box(GLR_CONFLICT_PRESERVATION_GRID),
            ))
        });
    });
}

fn bench_bdd_progress_runtime(c: &mut Criterion) {
    c.bench_function("bdd_progress_runtime_phase", |b| {
        b.iter(|| {
            black_box(bdd_progress(
                black_box(BddPhase::Runtime),
                black_box(GLR_CONFLICT_PRESERVATION_GRID),
            ))
        });
    });
}

fn bench_bdd_progress_report_core(c: &mut Criterion) {
    c.bench_function("bdd_progress_report_core", |b| {
        b.iter(|| {
            black_box(bdd_progress_report(
                black_box(BddPhase::Core),
                black_box(GLR_CONFLICT_PRESERVATION_GRID),
                black_box("Core Phase"),
            ))
        });
    });
}

fn bench_bdd_progress_report_runtime(c: &mut Criterion) {
    c.bench_function("bdd_progress_report_runtime", |b| {
        b.iter(|| {
            black_box(bdd_progress_report(
                black_box(BddPhase::Runtime),
                black_box(GLR_CONFLICT_PRESERVATION_GRID),
                black_box("Runtime Phase"),
            ))
        });
    });
}

fn bench_scenario_status_implemented(c: &mut Criterion) {
    c.bench_function("scenario_status_implemented", |b| {
        let status = BddScenarioStatus::Implemented;
        b.iter(|| {
            black_box(black_box(status).implemented());
            black_box(black_box(status).icon());
            black_box(black_box(status).label());
            black_box(black_box(status).detail());
        });
    });
}

fn bench_scenario_status_deferred(c: &mut Criterion) {
    c.bench_function("scenario_status_deferred", |b| {
        let status = BddScenarioStatus::Deferred { reason: "pending" };
        b.iter(|| {
            black_box(black_box(status).implemented());
            black_box(black_box(status).icon());
            black_box(black_box(status).label());
            black_box(black_box(status).detail());
        });
    });
}

fn bench_scenario_display(c: &mut Criterion) {
    c.bench_function("scenario_display", |b| {
        let scenario = &GLR_CONFLICT_PRESERVATION_GRID[0];
        b.iter(|| black_box(format!("{}", black_box(scenario))));
    });
}

fn bench_scenario_status_lookup(c: &mut Criterion) {
    c.bench_function("scenario_status_lookup", |b| {
        let scenario = &GLR_CONFLICT_PRESERVATION_GRID[0];
        b.iter(|| black_box(black_box(*scenario).status(black_box(BddPhase::Core))));
    });
}

fn bench_bdd_progress_empty_scenarios(c: &mut Criterion) {
    c.bench_function("bdd_progress_empty_scenarios", |b| {
        let empty: &[BddScenario] = &[];
        b.iter(|| black_box(bdd_progress(black_box(BddPhase::Core), black_box(empty))));
    });
}

fn bench_bdd_progress_single_scenario(c: &mut Criterion) {
    c.bench_function("bdd_progress_single_scenario", |b| {
        let single = [BddScenario {
            id: 1,
            title: "Test scenario",
            reference: "REF-1",
            core_status: BddScenarioStatus::Implemented,
            runtime_status: BddScenarioStatus::Deferred { reason: "todo" },
        }];
        b.iter(|| black_box(bdd_progress(black_box(BddPhase::Core), black_box(&single))));
    });
}

fn bench_bdd_phase_display(c: &mut Criterion) {
    c.bench_function("bdd_phase_display", |b| {
        b.iter(|| {
            black_box(format!("{}", black_box(BddPhase::Core)));
            black_box(format!("{}", black_box(BddPhase::Runtime)));
        });
    });
}

criterion_group!(
    benches,
    bench_bdd_progress_core,
    bench_bdd_progress_runtime,
    bench_bdd_progress_report_core,
    bench_bdd_progress_report_runtime,
    bench_scenario_status_implemented,
    bench_scenario_status_deferred,
    bench_scenario_display,
    bench_scenario_status_lookup,
    bench_bdd_progress_empty_scenarios,
    bench_bdd_progress_single_scenario,
    bench_bdd_phase_display,
);

criterion_main!(benches);
