use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;

use adze_feature_policy_core::ParserFeatureProfile;
use adze_governance_metadata::{GovernanceMetadata, ParserFeatureProfileSnapshot};

fn sample_profile_snapshot() -> ParserFeatureProfileSnapshot {
    ParserFeatureProfileSnapshot::new(true, false, true, false)
}

fn sample_governance_metadata() -> GovernanceMetadata {
    GovernanceMetadata::with_counts("runtime", 5, 10, "runtime:5/10")
}

fn bench_profile_snapshot_creation(c: &mut Criterion) {
    c.bench_function("profile_snapshot_new", |b| {
        b.iter(|| {
            let snap = ParserFeatureProfileSnapshot::new(
                black_box(true),
                black_box(false),
                black_box(true),
                black_box(false),
            );
            black_box(snap)
        });
    });

    c.bench_function("profile_snapshot_from_profile", |b| {
        let profile = ParserFeatureProfile {
            pure_rust: true,
            tree_sitter_standard: false,
            tree_sitter_c2rust: true,
            glr: false,
        };
        b.iter(|| {
            let snap = ParserFeatureProfileSnapshot::from_profile(black_box(profile));
            black_box(snap)
        });
    });

    c.bench_function("profile_snapshot_as_profile", |b| {
        let snap = sample_profile_snapshot();
        b.iter(|| black_box(black_box(snap).as_profile()));
    });
}

fn bench_profile_snapshot_backend_resolution(c: &mut Criterion) {
    let snap = sample_profile_snapshot();

    c.bench_function("profile_snapshot_non_conflict_backend", |b| {
        b.iter(|| black_box(black_box(snap).non_conflict_backend()));
    });

    c.bench_function("profile_snapshot_resolve_non_conflict_backend", |b| {
        b.iter(|| black_box(black_box(snap).resolve_non_conflict_backend()));
    });

    c.bench_function("profile_snapshot_resolve_conflict_backend", |b| {
        b.iter(|| black_box(black_box(snap).resolve_conflict_backend()));
    });
}

fn bench_governance_metadata_creation(c: &mut Criterion) {
    c.bench_function("governance_metadata_default", |b| {
        b.iter(|| black_box(GovernanceMetadata::default()));
    });

    c.bench_function("governance_metadata_with_counts", |b| {
        b.iter(|| {
            let meta = GovernanceMetadata::with_counts(
                black_box("runtime"),
                black_box(5),
                black_box(10),
                black_box("runtime:5/10"),
            );
            black_box(meta)
        });
    });
}

fn bench_governance_metadata_queries(c: &mut Criterion) {
    let meta = sample_governance_metadata();

    c.bench_function("governance_metadata_is_complete_false", |b| {
        b.iter(|| black_box(black_box(&meta).is_complete()));
    });

    let complete_meta = GovernanceMetadata::with_counts("core", 8, 8, "core:8/8");
    c.bench_function("governance_metadata_is_complete_true", |b| {
        b.iter(|| black_box(black_box(&complete_meta).is_complete()));
    });
}

fn bench_profile_snapshot_serde(c: &mut Criterion) {
    let snap = sample_profile_snapshot();

    c.bench_function("profile_snapshot_serialize_json", |b| {
        b.iter(|| {
            let json = serde_json::to_string(black_box(&snap)).unwrap();
            black_box(json)
        });
    });

    let json = serde_json::to_string(&snap).unwrap();
    c.bench_function("profile_snapshot_deserialize_json", |b| {
        b.iter(|| {
            let s: ParserFeatureProfileSnapshot = serde_json::from_str(black_box(&json)).unwrap();
            black_box(s)
        });
    });

    c.bench_function("profile_snapshot_roundtrip", |b| {
        b.iter(|| {
            let json = serde_json::to_string(black_box(&snap)).unwrap();
            let s: ParserFeatureProfileSnapshot = serde_json::from_str(&json).unwrap();
            black_box(s)
        });
    });
}

fn bench_governance_metadata_serde(c: &mut Criterion) {
    let meta = sample_governance_metadata();

    c.bench_function("governance_metadata_serialize_json", |b| {
        b.iter(|| {
            let json = serde_json::to_string(black_box(&meta)).unwrap();
            black_box(json)
        });
    });

    let json = serde_json::to_string(&meta).unwrap();
    c.bench_function("governance_metadata_deserialize_json", |b| {
        b.iter(|| {
            let m: GovernanceMetadata = serde_json::from_str(black_box(&json)).unwrap();
            black_box(m)
        });
    });

    c.bench_function("governance_metadata_roundtrip", |b| {
        b.iter(|| {
            let json = serde_json::to_string(black_box(&meta)).unwrap();
            let m: GovernanceMetadata = serde_json::from_str(&json).unwrap();
            black_box(m)
        });
    });
}

criterion_group!(
    benches,
    bench_profile_snapshot_creation,
    bench_profile_snapshot_backend_resolution,
    bench_governance_metadata_creation,
    bench_governance_metadata_queries,
    bench_profile_snapshot_serde,
    bench_governance_metadata_serde,
);
criterion_main!(benches);
