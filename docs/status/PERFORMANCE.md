# Performance Guide

This guide covers parse performance characteristics, optimization strategies,
benchmarking, and known limitations of adze's parsing infrastructure.

## Parse Performance Characteristics

### Time Complexity by Grammar Type

| Grammar Type | Complexity | Typical Latency | Notes |
|---|---|---|---|
| Deterministic LR(1) | O(n) | ~46 µs for 50-op expressions | No conflicts; single parse path |
| GLR with few conflicts | O(n) amortized | ~224 µs for 200-op expressions | Fork/merge overhead is constant per conflict site |
| GLR with pervasive ambiguity | O(n³) worst case | Varies widely | Every token triggers forking; avoid if possible |

The arithmetic grammar benchmarks illustrate **indicative scaling on the
arithmetic fixture set** (not a universal SLA for all grammars/hardware):

- **Small** (~100 LOC, ~88 expressions): 10–100 µs
- **Medium** (~2,000 LOC, ~1,914 expressions): 200 µs – 2 ms
- **Large** (~10,000 LOC, ~9,606 expressions): 1–10 ms

These numbers come from `cargo bench -p adze-benchmarks --bench glr_performance_real`
using valid arithmetic expression fixtures.

### Memory Usage Patterns

- **Parse tables** are generated at build time and embedded as static data. Table
  size scales with the number of states × symbols in the grammar. The Python grammar
  (273 symbols, 57 fields) produces tables that fit comfortably in L2 cache.
- **Parse trees** use arena-style allocation. Node count and tree depth are the
  primary drivers of runtime memory. Enable arena allocators for parsing-heavy
  workloads:
  ```toml
  [dependencies]
  adze = { version = "0.8", features = ["glr"] }
  ```
- **GLR forests** maintain multiple parse paths simultaneously. Each fork doubles
  the working set until paths merge or are pruned.

### Impact of Grammar Size on Parse Time

Grammar size affects two separate phases:

1. **Build time** – Table generation (FIRST/FOLLOW sets, LR(1) automaton construction,
   table compression) is the bottleneck. Larger grammars with more states take
   significantly longer to compile. The `adze-tablegen` compression benchmarks
   (`tablegen/benches/compression.rs`) measure this directly.
2. **Runtime** – Parse table lookups are O(1) per token regardless of grammar size.
   However, larger grammars increase cache pressure from bigger parse tables.

## Optimization Tips

### Use the Pure-Rust Backend for WASM

The default `tree-sitter-c2rust` backend compiles to pure Rust, making it
WASM-compatible without a C toolchain:

```toml
[dependencies]
adze = { version = "0.8" }  # c2rust backend is the default
```

For native builds where you want the standard C runtime:

```toml
[dependencies]
adze = { version = "0.8", features = ["tree-sitter-standard"] }
```

The pure-Rust backend avoids FFI overhead in WASM and produces smaller binaries
for `wasm32-unknown-unknown` targets.

### Design Grammars for Performance

**Reduce ambiguity.** Every ambiguity in the grammar creates a fork point in the
GLR parser. Use precedence and associativity annotations to resolve conflicts
statically:

```rust
#[adze::prec_left(1)]
fn addition(left: Expr, _op: Plus, right: Expr) -> Expr { /* ... */ }

#[adze::prec_left(2)]
fn multiplication(left: Expr, _op: Star, right: Expr) -> Expr { /* ... */ }
```

**Prefer simpler token patterns.** String literals are faster to match than complex
regexes. Replace `r"[+]"` with `"+"` where possible.

**Flatten unnecessary nesting.** Deeply nested grammar rules increase tree depth
and allocation count. If intermediate nodes carry no semantic meaning, consider
inlining them.

### Build Configuration for Production

Always use release mode with LTO for production parsing:

```toml
[profile.release]
lto = true
codegen-units = 1
panic = "abort"
```

### Check for Conflicts

Use `ADZE_EMIT_ARTIFACTS=true` to inspect the generated grammar and identify
conflicts:

```bash
ADZE_EMIT_ARTIFACTS=true cargo build 2>&1
# Generated files appear in target/debug/build/<crate>-<hash>/out/
```

Review the grammar JSON to see which states have multiple actions (indicating
GLR forking at runtime).

## Benchmarking

### Running Benchmarks

adze uses [Criterion](https://bheisler.github.io/criterion.rs/book/) for
micro-benchmarks. Configuration lives in `.config/cargo-criterion.toml`.

```bash
# Run all benchmarks in the workspace
cargo bench

# Run only the release-gated parser benchmarks
cargo xtask bench

# Run and save a named baseline
cargo xtask bench --save-baseline v0.8.0

# Quick dev-loop benchmarks (glr-core only)
./scripts/bench-quick.sh

# Run benchmarks for a specific crate
cargo bench -p adze-glr-core
cargo bench -p adze-tablegen
cargo bench -p adze-benchmarks
```

### Benchmark Inventory (Truth-in-labeling)

Legend for **Class**: real parser workload, tablegen workload, compression/decode
workload, GLR forest/fork workload, placeholder/mock, utility microbenchmark.

| Path | Class | Notes |
|---|---|---|
| `benchmarks/benches/glr_hot.rs` | real parser workload | Parses valid arithmetic fixtures with generated parser. |
| `benchmarks/benches/glr_performance.rs` | real parser workload | Same fixture family as `glr_hot`, size sweep small→large. |
| `benchmarks/benches/glr_performance_real.rs` | real parser workload + utility microbenchmark | Main group is real parse throughput; `utility_*` benches are non-parse helpers. |
| `benchmarks/benches/core_baselines.rs` | tablegen workload | IR normalize, FIRST/FOLLOW, LR(1) automaton build, compression. |
| `benchmarks/benches/arena_vs_box_allocation.rs` | utility microbenchmark | Allocation strategy benchmark, not parsing end-to-end. |
| `benchmarks/benches/stack_optimization.rs` | GLR forest/fork workload | Stack fork/memory-pool mechanics; synthetic but parser-adjacent. |
| `benchmarks/benches/optimization_bench.rs` | utility microbenchmark | Pool/arena simulation workload, not parser correctness throughput. |
| `benchmarks/benches/incremental_bench.rs` | GLR forest/fork workload | Incremental edit-path behaviors over arithmetic token stream. |
| `benchmarks/benches/parse_bench.rs` | placeholder/mock | Explicit placeholder during parser API migration (`placeholder_no_parser_work`). |
| `runtime/benches/glr_parser_bench.rs` | GLR forest/fork workload | Ambiguous grammar parse path/fork pressure. |
| `runtime/benches/runtime_parse_serialize_bench.rs` | real parser workload + utility microbenchmark | Real parse plus serialization/traversal costs. |
| `runtime/benches/parser_benchmark.rs` | real parser workload | Macro-defined arithmetic grammar parser calls. |
| `runtime/benches/simple_bench.rs` | utility microbenchmark | Lexer-focused throughput on synthetic snippets. |
| `runtime/benches/parser_bench.rs` | placeholder/mock | Gated unstable API migration bench; not active in default runs. |
| `runtime/benches/perf_benchmark.rs` | placeholder/mock | Legacy scaffold with removed parser/lexer paths in comments. |
| `runtime/benches/pure_rust_bench.rs` | placeholder/mock | Temporary API-migration stub executable. |
| `runtime/benches/incremental_benchmark.rs` | GLR forest/fork workload | Incremental parser edit replay; feature-gated unstable bench. |
| `runtime/benches/incremental_parsing.rs` | GLR forest/fork workload | Incremental token edit cases; unstable bench. |
| `runtime/benches/incremental_simple.rs` | GLR forest/fork workload | Simplified incremental edit workload; unstable bench. |
| `glr-core/benches/automaton.rs` | tablegen workload | FIRST/FOLLOW and LR(1) automaton construction. |
| `glr-core/benches/perf_snapshot.rs` | utility microbenchmark | EOF-only driver hot-path snapshot (`micro_eof_only_parse`). |
| `tablegen/benches/compression.rs` | compression/decode workload | Parse-table compression across grammar shapes/sizes. |

### Still-missing Real Benchmark Categories

- Real-world language grammar parse fixtures (Python/JavaScript/Go) beyond arithmetic.
- Compression **decode**/lookup runtime benchmarks (current suite is compression-build heavy).
- End-to-end incremental parsing benchmark with validated token edit mapping from real source edits.

### Interpreting Results

Criterion reports three values per benchmark:

```
arithmetic_parsing/parse/medium
                        time:   [215.3 µs 224.1 µs 233.8 µs]
                        change: [-2.1% +0.5% +3.2%] (p = 0.72 > 0.05)
                        No change in performance detected.
```

- **Left bound / center / right bound** – the 95% confidence interval for the
  mean execution time.
- **change** – comparison against the last saved baseline (if any).
- **p-value** – statistical significance. Values > 0.05 mean the change is within
  noise.

To compare against a saved baseline:

```bash
cargo xtask compare-baseline v0.8.0 --threshold 5
```

This fails if any benchmark regresses by more than the threshold percentage.

### Adding New Benchmarks

1. **Generate fixtures** if your benchmark needs input data:
   ```bash
   cargo xtask generate-fixtures --force
   cargo xtask validate-fixtures
   ```

2. **Add a benchmark file** in the appropriate crate's `benches/` directory:
   ```rust
   use criterion::{black_box, criterion_group, criterion_main, Criterion};

   fn bench_my_feature(c: &mut Criterion) {
       let input = "your test input";
       c.bench_function("my_feature", |b| {
           b.iter(|| {
               black_box(do_something(input));
           });
       });
   }

   criterion_group!(benches, bench_my_feature);
   criterion_main!(benches);
   ```

3. **Register the benchmark** in the crate's `Cargo.toml`:
   ```toml
   [[bench]]
   name = "my_benchmark"
   harness = false
   ```

4. **Run and save a baseline** so future changes can be compared:
   ```bash
   cargo bench --bench my_benchmark
   ```

### Profiling

For CPU and memory profiling:

```bash
# CPU profile with flamegraph
cargo install flamegraph
cargo flamegraph --bench glr_performance_real

# CPU/memory profiles via xtask
cargo xtask profile cpu arithmetic large
cargo xtask profile memory arithmetic medium
```

Enable runtime performance logging for forest-to-tree conversion:

```bash
ADZE_LOG_PERFORMANCE=true cargo run
# Output: 🚀 Forest->Tree conversion: 1247 nodes, depth 23, took 2.1ms
```

## Known Performance Limitations

### GLR Worst-Case Exponential Behavior

Grammars with pervasive ambiguity (e.g., every token triggers multiple valid
parse paths) cause the GLR parser to fork exponentially. The theoretical worst
case is O(n³) for highly ambiguous grammars. In practice:

- **Most programming languages** hit a small, bounded number of conflict sites
  and run in near-linear time.
- **Pathological grammars** (e.g., `S → S S | a`) can exhibit cubic or worse
  behavior on long inputs.

**Mitigation:** Resolve ambiguities with `#[adze::prec_left]` /
`#[adze::prec_right]` annotations. Use `ADZE_EMIT_ARTIFACTS=true` to audit
conflict counts.

### Large Grammar Table Generation Time

Table generation (FIRST/FOLLOW, LR(1) automaton, compression) dominates build
time for large grammars. The Python grammar with 273 symbols takes noticeably
longer to compile than small arithmetic grammars.

**Mitigation:** Table generation is a build-time cost only. Once generated, the
tables are embedded as static data and impose no runtime penalty. Use
`cargo build --release` to speed up the generation itself.

### Incremental Parsing (Disabled)

The GLR incremental parsing path (`runtime/src/glr_incremental.rs`) is
currently **disabled** and falls back to fresh parsing. The infrastructure exists
but has known architectural issues:

- Error tracking uses hardcoded `is_error: false` in subtree creation
- Root kind determination diverges between forest symbols and parse results
- Token-level vs grammar-level parsing produces inconsistent trees

The conservative fallback ensures correctness at the cost of not reusing
subtrees from previous parses. See `glr_incremental.rs:281-297` for details.

### Fork/Merge Overhead

GLR fork and merge operations have fixed overhead per conflict site. For
grammars with many conflicts, this overhead can dominate even on moderate-sized
inputs. The `runtime/benches/glr_parser_bench.rs` benchmark measures this
directly with an intentionally ambiguous expression grammar.
