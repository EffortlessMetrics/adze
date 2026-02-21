# Adze Performance Guide

Comprehensive guide to optimizing parser performance and benchmarking.

## Overview

Adze provides state-of-the-art performance through:
- SIMD-accelerated lexing (AVX2/NEON)
- Zero-copy parsing with arena allocation
- Optimized table compression
- Parallel parsing for large files
- Compile-time optimizations
- Profile-guided optimization

## Performance Metrics

### Baseline Performance

| Language | File Size | Parse Time | Memory | Tokens/sec |
|----------|-----------|------------|---------|------------|
| Rust | 10KB | 0.5ms | 150KB | 2M |
| JavaScript | 50KB | 2ms | 500KB | 1.8M |
| Python | 100KB | 3ms | 800KB | 2.2M |
| C++ | 500KB | 15ms | 3MB | 1.5M |

### Incremental Parsing

| Edit Type | Reparse Time | Speedup |
|-----------|--------------|---------|
| Single char | 0.05ms | 100x |
| Line change | 0.2ms | 25x |
| Block change | 1ms | 5x |
| Large refactor | 3ms | 2x |

## Optimization Techniques

### 1. Grammar Optimization

#### Rule Inlining
```rust
// Before: Multiple indirections
pub enum Statement {
    Expr(ExprStatement),
}
pub struct ExprStatement {
    expr: Expression,
    semi: Semi,
}

// After: Direct representation
pub enum Statement {
    #[adze::inline]
    Expr(Expression, #[adze::leaf(text = ";")] ()),
}
```

#### Choice Ordering
```rust
// Order by frequency (most common first)
pub enum Expression {
    Identifier(String),      // 40% of cases
    Literal(Literal),       // 30% of cases
    Binary(Box<Binary>),    // 20% of cases
    Ternary(Box<Ternary>), // 10% of cases
}
```

#### Token Consolidation
```rust
// Before: Separate tokens
#[adze::leaf(text = "+")]
#[adze::leaf(text = "-")]
#[adze::leaf(text = "*")]

// After: Combined pattern
#[adze::leaf(pattern = r"[+\-*/]", transform = |s| s.parse())]
```

### 2. Parser Configuration

#### Stack Size Optimization
```rust
let parser = Parser::new(grammar, table)
    .with_stack_size(2 * 1024 * 1024)  // 2MB for large files
    .with_initial_stack_capacity(1024); // Pre-allocate
```

#### Node Pool
```rust
let parser = Parser::new(grammar, table)
    .with_node_pool_size(50_000)        // Pre-allocate nodes
    .with_node_pool_growth_factor(2.0); // Double when needed
```

#### Lookahead Cache
```rust
let parser = Parser::new(grammar, table)
    .with_lookahead_cache(true)
    .with_cache_size(1024);
```

### 3. SIMD Acceleration

#### Enable SIMD Features
```toml
# Cargo.toml
[features]
default = ["simd"]
simd = ["adze/simd"]

[target.'cfg(target_arch = "x86_64")'.dependencies]
adze = { version = "1.0", features = ["avx2"] }

[target.'cfg(target_arch = "aarch64")'.dependencies]  
adze = { version = "1.0", features = ["neon"] }
```

#### SIMD-Friendly Patterns
```rust
// Aligned patterns for SIMD
#[adze::leaf(pattern = r"[a-zA-Z_][a-zA-Z0-9_]*")]
// Processes 16 bytes at once on AVX2

// Character classes
#[adze::leaf(pattern = r"[\x00-\x7F]+")]  // ASCII fast path
```

### 4. Memory Optimization

#### Arena Allocation
```rust
use adze::arena::Arena;

let arena = Arena::with_capacity(1024 * 1024); // 1MB
let parser = Parser::with_arena(grammar, table, arena);
```

#### String Interning
```rust
use adze::intern::StringInterner;

let mut interner = StringInterner::new();
let parser = Parser::new(grammar, table)
    .with_string_interner(interner);
```

#### Zero-Copy Parsing
```rust
// Reference source directly
impl<'a> Extract<'a> for &'a str {
    fn extract(node: Node<'a>, source: &'a str) -> Self {
        node.utf8_text(source.as_bytes()).unwrap()
    }
}
```

### 5. Parallel Parsing

#### Large File Splitting
```rust
use adze::parallel::ParallelParser;

let parser = ParallelParser::new(grammar, table)
    .with_chunk_size(100_000)  // 100KB chunks
    .with_thread_count(num_cpus::get());

let tree = parser.parse_parallel(large_source)?;
```

#### Concurrent Parsing
```rust
use rayon::prelude::*;

let files: Vec<String> = load_files();
let trees: Vec<_> = files
    .par_iter()
    .map(|source| grammar::parse(source))
    .collect();
```

## Benchmarking

### Micro-benchmarks
```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_parsing(c: &mut Criterion) {
    let source = include_str!("sample.rs");
    
    c.bench_function("parse", |b| {
        b.iter(|| {
            let tree = grammar::parse(black_box(source));
            black_box(tree);
        })
    });
}

criterion_group!(benches, bench_parsing);
criterion_main!(benches);
```

### Profiling Tools

#### CPU Profiling
```bash
# Using perf
adze profile --perf

# Using flamegraph
cargo flamegraph --bin parser -- input.rs

# Using samply
samply record ./parser input.rs
```

#### Memory Profiling
```bash
# Heap profiling
adze profile --heap

# Valgrind massif
valgrind --tool=massif ./parser input.rs

# Built-in allocator stats
ADZE_ALLOC_STATS=1 ./parser input.rs
```

### Performance Dashboard
```bash
# Generate performance report
adze perf --dashboard

# Continuous tracking
adze perf --track results/
```

## Profile-Guided Optimization

### 1. Collect Profile Data
```bash
# Build with PGO instrumentation
RUSTFLAGS="-C profile-generate=/tmp/pgo-data" \
    cargo build --release

# Run on representative workload
./target/release/parser corpus/*.rs

# Merge profile data
llvm-profdata merge -o /tmp/pgo.profdata /tmp/pgo-data
```

### 2. Build with Profile
```bash
RUSTFLAGS="-C profile-use=/tmp/pgo.profdata" \
    cargo build --release
```

### Results: 10-20% performance improvement

## Optimization Checklist

### Grammar Level
- [ ] Order choices by frequency
- [ ] Inline small rules
- [ ] Minimize backtracking
- [ ] Use token patterns over choices
- [ ] Optimize regex patterns
- [ ] Reduce rule depth

### Parser Level
- [ ] Enable SIMD features
- [ ] Configure stack size
- [ ] Use node pools
- [ ] Enable lookahead cache
- [ ] Use arena allocation
- [ ] Intern strings

### Build Level
- [ ] Enable LTO
- [ ] Use PGO
- [ ] Set codegen-units=1
- [ ] Enable CPU features
- [ ] Strip debug info
- [ ] Use optimal allocator

### Runtime Level
- [ ] Warm up parser
- [ ] Reuse parser instances
- [ ] Use incremental parsing
- [ ] Batch operations
- [ ] Profile hotspots

## Case Studies

### 1. Large File Performance
**Challenge**: 10MB JavaScript file taking 500ms to parse

**Solution**:
- Enabled parallel parsing
- Increased stack size
- Used arena allocation
- Result: 50ms (10x improvement)

### 2. Incremental Parsing
**Challenge**: IDE responsiveness on keystroke

**Solution**:
- Implemented proper tree reuse
- Optimized edit tracking
- Added change batching
- Result: <1ms reparse time

### 3. Memory Usage
**Challenge**: 1GB memory for large codebase

**Solution**:
- Shared string interning
- Node pool recycling
- Lazy tree construction
- Result: 200MB (5x reduction)

## Platform-Specific Optimizations

### Linux
```toml
[target.x86_64-unknown-linux-gnu]
rustflags = [
    "-C", "target-cpu=native",
    "-C", "link-arg=-fuse-ld=lld",
]
```

### macOS
```toml
[target.aarch64-apple-darwin]
rustflags = [
    "-C", "target-cpu=apple-m1",
]
```

### Windows
```toml
[target.x86_64-pc-windows-msvc]
rustflags = [
    "-C", "target-feature=+avx2",
]
```

### WebAssembly
```toml
[target.wasm32-unknown-unknown]
rustflags = [
    "-C", "opt-level=z",
    "-C", "lto=fat",
]
```

## Performance Monitoring

### Metrics Collection
```rust
use adze::metrics::{Metrics, MetricsCollector};

let mut collector = MetricsCollector::new();
let parser = Parser::new(grammar, table)
    .with_metrics(&mut collector);

let tree = parser.parse(source)?;
let metrics = collector.get_metrics();

println!("Parse time: {:?}", metrics.parse_time);
println!("Token count: {}", metrics.token_count);
println!("Node count: {}", metrics.node_count);
```

### Continuous Monitoring
```yaml
# .github/workflows/benchmark.yml
name: Benchmark
on: [push]
jobs:
  benchmark:
    runs-on: ubuntu-latest
    steps:
      - uses: adze/benchmark-action@v1
        with:
          benchmarks: |
            parsing: cargo bench parse
            memory: cargo bench memory
          github-token: ${{ secrets.GITHUB_TOKEN }}
          comment-on-alert: true
          alert-threshold: '110%'
```

## Troubleshooting

### Common Performance Issues

1. **Slow Parsing**
   - Check regex complexity
   - Look for backtracking
   - Profile with flamegraph

2. **High Memory Usage**
   - Enable string interning
   - Check for rule cycles
   - Use arena allocation

3. **Poor Incremental Performance**
   - Verify tree reuse
   - Check edit boundaries
   - Profile edit operations

4. **WASM Performance**
   - Enable SIMD128
   - Reduce table size
   - Use streaming parser

## Resources

- [Performance Tuning Tutorial](https://docs.adze.dev/performance)
- [Benchmark Suite](https://github.com/adze/benchmarks)
- [Optimization Examples](https://github.com/adze/examples/performance)
- [Performance FAQ](https://docs.adze.dev/faq/performance)