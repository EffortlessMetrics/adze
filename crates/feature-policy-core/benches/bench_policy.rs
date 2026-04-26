use criterion::{Criterion, criterion_group, criterion_main};

use adze_feature_policy_core::{ParserBackend, ParserFeatureProfile};
use std::hint::black_box;

fn sample_profile() -> ParserFeatureProfile {
    ParserFeatureProfile {
        pure_rust: true,
        tree_sitter_standard: false,
        tree_sitter_c2rust: false,
        glr: false,
    }
}

fn bench_profile_creation(c: &mut Criterion) {
    c.bench_function("profile_current", |b| {
        b.iter(|| black_box(ParserFeatureProfile::current()));
    });

    c.bench_function("profile_construct", |b| {
        b.iter(|| {
            let profile = ParserFeatureProfile {
                pure_rust: black_box(true),
                tree_sitter_standard: black_box(false),
                tree_sitter_c2rust: black_box(false),
                glr: black_box(false),
            };
            black_box(profile)
        });
    });
}

fn bench_profile_predicates(c: &mut Criterion) {
    let profile = sample_profile();

    c.bench_function("profile_has_pure_rust", |b| {
        b.iter(|| black_box(black_box(profile).has_pure_rust()));
    });

    c.bench_function("profile_has_glr", |b| {
        b.iter(|| black_box(black_box(profile).has_glr()));
    });

    c.bench_function("profile_has_tree_sitter", |b| {
        b.iter(|| black_box(black_box(profile).has_tree_sitter()));
    });
}

fn bench_backend_resolution(c: &mut Criterion) {
    let profile_no_conflicts = ParserFeatureProfile {
        pure_rust: true,
        tree_sitter_standard: false,
        tree_sitter_c2rust: false,
        glr: false,
    };

    let profile_glr = ParserFeatureProfile {
        pure_rust: true,
        tree_sitter_standard: false,
        tree_sitter_c2rust: false,
        glr: true,
    };

    let profile_tree_sitter = ParserFeatureProfile {
        pure_rust: false,
        tree_sitter_standard: true,
        tree_sitter_c2rust: false,
        glr: false,
    };

    c.bench_function("profile_resolve_backend_pure_rust", |b| {
        b.iter(|| black_box(black_box(profile_no_conflicts).resolve_backend(false)));
    });

    c.bench_function("profile_resolve_backend_glr", |b| {
        b.iter(|| black_box(black_box(profile_glr).resolve_backend(false)));
    });

    c.bench_function("profile_resolve_backend_tree_sitter", |b| {
        b.iter(|| black_box(black_box(profile_tree_sitter).resolve_backend(false)));
    });

    c.bench_function("profile_resolve_backend_glr_with_conflicts", |b| {
        b.iter(|| black_box(black_box(profile_glr).resolve_backend(true)));
    });

    c.bench_function("profile_resolve_backend_tree_sitter_with_conflicts", |b| {
        b.iter(|| black_box(black_box(profile_tree_sitter).resolve_backend(true)));
    });
}

fn bench_backend_operations(c: &mut Criterion) {
    c.bench_function("backend_name_tree_sitter", |b| {
        b.iter(|| black_box(ParserBackend::TreeSitter.name()));
    });

    c.bench_function("backend_name_pure_rust", |b| {
        b.iter(|| black_box(ParserBackend::PureRust.name()));
    });

    c.bench_function("backend_name_glr", |b| {
        b.iter(|| black_box(ParserBackend::GLR.name()));
    });

    c.bench_function("backend_is_glr", |b| {
        b.iter(|| black_box(ParserBackend::GLR.is_glr()));
    });

    c.bench_function("backend_is_pure_rust", |b| {
        b.iter(|| black_box(ParserBackend::PureRust.is_pure_rust()));
    });
}

fn bench_profile_display(c: &mut Criterion) {
    let profile_pure_rust = ParserFeatureProfile {
        pure_rust: true,
        tree_sitter_standard: false,
        tree_sitter_c2rust: false,
        glr: false,
    };

    let profile_multiple = ParserFeatureProfile {
        pure_rust: true,
        tree_sitter_standard: true,
        tree_sitter_c2rust: false,
        glr: true,
    };

    let profile_none = ParserFeatureProfile {
        pure_rust: false,
        tree_sitter_standard: false,
        tree_sitter_c2rust: false,
        glr: false,
    };

    c.bench_function("profile_display_single", |b| {
        b.iter(|| format!("{}", black_box(profile_pure_rust)));
    });

    c.bench_function("profile_display_multiple", |b| {
        b.iter(|| format!("{}", black_box(profile_multiple)));
    });

    c.bench_function("profile_display_none", |b| {
        b.iter(|| format!("{}", black_box(profile_none)));
    });
}

criterion_group!(
    benches,
    bench_profile_creation,
    bench_profile_predicates,
    bench_backend_resolution,
    bench_backend_operations,
    bench_profile_display,
);
criterion_main!(benches);
