use criterion::{Criterion, black_box, criterion_group, criterion_main};

use adze_parsetable_metadata::{
    FeatureFlags, GenerationInfo, GrammarInfo, ParsetableMetadata, TableStatistics,
};

fn sample_metadata() -> ParsetableMetadata {
    ParsetableMetadata {
        schema_version: "1.0".to_string(),
        grammar: GrammarInfo {
            name: "python".to_string(),
            version: "3.12.0".to_string(),
            language: "python".to_string(),
        },
        generation: GenerationInfo {
            timestamp: "2025-01-15T12:00:00Z".to_string(),
            tool_version: "0.1.0".to_string(),
            rust_version: "1.92.0".to_string(),
            host_triple: "x86_64-unknown-linux-gnu".to_string(),
        },
        statistics: TableStatistics {
            state_count: 1500,
            symbol_count: 273,
            rule_count: 450,
            conflict_count: 12,
            multi_action_cells: 38,
        },
        features: FeatureFlags {
            glr_enabled: true,
            external_scanner: true,
            incremental: false,
        },
        feature_profile: None,
        governance: None,
    }
}

fn bench_serialize(c: &mut Criterion) {
    let metadata = sample_metadata();

    c.bench_function("metadata_serialize_json", |b| {
        b.iter(|| {
            let json = serde_json::to_string(black_box(&metadata)).unwrap();
            black_box(json)
        });
    });

    c.bench_function("metadata_serialize_json_pretty", |b| {
        b.iter(|| {
            let json = serde_json::to_string_pretty(black_box(&metadata)).unwrap();
            black_box(json)
        });
    });

    c.bench_function("metadata_serialize_to_vec", |b| {
        b.iter(|| {
            let bytes = serde_json::to_vec(black_box(&metadata)).unwrap();
            black_box(bytes)
        });
    });
}

fn bench_deserialize(c: &mut Criterion) {
    let metadata = sample_metadata();
    let json_str = serde_json::to_string(&metadata).unwrap();
    let json_bytes = serde_json::to_vec(&metadata).unwrap();

    c.bench_function("metadata_deserialize_parse_json", |b| {
        b.iter(|| {
            let m = ParsetableMetadata::parse_json(black_box(&json_str)).unwrap();
            black_box(m)
        });
    });

    c.bench_function("metadata_deserialize_from_bytes", |b| {
        b.iter(|| {
            let m = ParsetableMetadata::from_bytes(black_box(&json_bytes)).unwrap();
            black_box(m)
        });
    });
}

fn bench_roundtrip(c: &mut Criterion) {
    let metadata = sample_metadata();

    c.bench_function("metadata_roundtrip", |b| {
        b.iter(|| {
            let json = serde_json::to_string(black_box(&metadata)).unwrap();
            let m = ParsetableMetadata::parse_json(&json).unwrap();
            black_box(m)
        });
    });
}

criterion_group!(benches, bench_serialize, bench_deserialize, bench_roundtrip);
criterion_main!(benches);
