# ADR 022: Telemetry and Performance Monitoring Strategy

**Status**: Accepted
**Date**: 2026-03-13
**Authors**: adze maintainers
**Related**: ADR-001 (Pure-Rust GLR Implementation), ADR-013 (GSS Implementation Strategy), ADR-012 (Performance Baseline Management)

## Context

GLR parsing introduces complexity that makes performance debugging challenging:

1. **Non-deterministic execution**: Multiple parse paths fork and merge, making it difficult to predict resource usage
2. **Memory pressure**: Graph-structured stacks can grow rapidly with ambiguous grammars
3. **Production debugging**: Users need visibility into parser behavior without attaching debuggers
4. **Performance regression detection**: Benchmark-only approaches may miss subtle degradation in parse complexity

The original implementation lacked systematic observability, requiring manual instrumentation for each debugging session.

### Problem Statement

Without telemetry, diagnosing issues like excessive forking, memory bloat, or cache inefficiency requires:
- Adding temporary logging statements
- Rebuilding and redeploying
- Removing instrumentation after diagnosis

This cycle is time-consuming and impractical for production environments.

## Decision

We implement a **layered telemetry strategy** with three complementary systems:

### Layer 1: Feature-Gated Atomic Telemetry

The primary telemetry system in [`glr-core/src/telemetry.rs`](../../glr-core/src/telemetry.rs) uses atomic counters for thread-safe, low-overhead tracking:

```rust
#[cfg(feature = "glr_telemetry")]
pub struct Telemetry {
    pub forks: AtomicU64,      // Parse path forks
    pub merges: AtomicU64,     // Stack merge operations
    pub reduces: AtomicU64,    // Reduce operations
    pub shifts: AtomicU64,     // Shift operations
    pub max_stacks: AtomicU64, // Peak concurrent stacks
    pub total_stacks: AtomicU64, // Cumulative stack count
}
```

**Key characteristics**:
- **Zero cost when disabled**: No-op implementations compile away entirely
- **Lock-free**: Uses `AtomicU64` with `Ordering::Relaxed` for minimal synchronization
- **Inline-optimized**: All increment methods marked `#[inline(always)]`

### Layer 2: Performance Statistics

The [`PerfStats`](../../glr-core/src/perf_optimizations.rs) struct tracks optimization effectiveness:

```rust
pub struct PerfStats {
    pub total_tokens: usize,
    pub total_stacks: usize,
    pub max_stacks: usize,
    pub stack_merges: usize,
    pub cache_hits: usize,
    pub cache_misses: usize,
}
```

This enables cache hit-rate analysis and merge efficiency measurement.

### Layer 3: GSS-Specific Statistics

The [`GSSStats`](../../glr-core/src/gss.rs) struct tracks graph-structured stack behavior:

```rust
pub struct GSSStats {
    pub total_nodes_created: usize,
    pub max_active_heads: usize,
    pub total_forks: usize,
    pub total_merges: usize,
    pub shared_segments: usize,
}
```

This provides insight into structural sharing efficiency and stack head proliferation.

### Metrics Summary

| Metric | Source | Purpose |
|--------|--------|---------|
| Forks | Telemetry, GSSStats | Detect excessive ambiguity |
| Merges | Telemetry, GSSStats | Measure stack consolidation |
| Cache hits/misses | PerfStats | Optimize table caching |
| Max active heads | GSSStats | Identify peak memory usage |
| Shared segments | GSSStats | Validate structural sharing |
| Shifts/Reduces | Telemetry | Basic parser operations |

### Usage Pattern

```rust
// Enable telemetry at compile time
// Cargo.toml: adze-glr-core = { features = ["glr_telemetry"] }

let telemetry = Telemetry::new();
// ... parsing happens ...
let stats = telemetry.stats();
println!("{}", stats); // "GLR Stats: 42 forks, 38 merges, ..."
```

## Consequences

### Positive

- **Production debugging**: Telemetry can be enabled in production builds without significant overhead
- **Zero cost abstraction**: No runtime cost when feature is disabled (default)
- **Thread-safe**: Atomic operations allow concurrent access from parallel parsers
- **Comprehensive coverage**: Three layers cover high-level operations through low-level optimizations
- **Display formatting**: Built-in `Display` implementation provides human-readable output

### Negative

- **Atomic overhead**: When enabled, atomic operations have ~2-5ns overhead per increment
- **Memory usage**: Telemetry struct adds 48 bytes per parser instance (6 × AtomicU64)
- **Incomplete picture**: No timing information (would require heavier instrumentation)
- **Manual opt-in**: Users must explicitly enable the feature and call `stats()`

### Neutral

- The `glr_telemetry` feature is **opt-in** and disabled by default
- Statistics are point-in-time snapshots; historical tracking requires external tooling
- The `ADZE_LOG_PERFORMANCE` environment variable can optionally control runtime logging

## Implementation Notes

### Enabling Telemetry

```toml
# Cargo.toml
[dependencies]
adze-glr-core = { version = "0.1", features = ["glr_telemetry"] }
```

### Environment Variable

The `ADZE_LOG_PERFORMANCE=true` environment variable enables additional performance logging at runtime when the feature is compiled in.

## Related

- Related ADRs: [ADR-001](001-pure-rust-glr-implementation.md), [ADR-013](013-gss-implementation-strategy.md), [ADR-012](012-performance-baseline-management.md)
- Implementation: [`glr-core/src/telemetry.rs`](../../glr-core/src/telemetry.rs)
- Performance utilities: [`glr-core/src/perf_optimizations.rs`](../../glr-core/src/perf_optimizations.rs)
- GSS statistics: [`glr-core/src/gss.rs`](../../glr-core/src/gss.rs)
