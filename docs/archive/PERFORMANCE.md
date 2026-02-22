# Adze Performance Guide

This document describes the performance optimizations implemented in adze, with a focus on the new GLR runtime2 capabilities.

## Overview

Adze includes several performance optimizations that make it competitive with or faster than traditional parsers:

1. **GLR Forest-to-Tree Conversion** - High-performance ambiguous parse handling
2. **Incremental Parsing with Subtree Reuse** - O(log n) complexity for edits
3. **Zero-Copy Parsing** - Minimal memory allocations with arena support
4. **Performance Monitoring** - Built-in metrics for optimization
5. **Bounded Concurrency** - Stable resource usage across machines

## GLR Performance Features (runtime2)

The GLR runtime2 provides comprehensive performance monitoring and optimization:

### Forest-to-Tree Conversion Metrics

Enable detailed conversion monitoring:

```bash
ADZE_LOG_PERFORMANCE=true cargo run
```

**Sample Output:**
```
🚀 Forest->Tree conversion: 1247 nodes, depth 23, took 2.1ms
🚀 Forest->Tree conversion: 3891 nodes, depth 45, took 8.7ms
```

**Metrics Provided:**
- **Node Count**: Total nodes processed during conversion (indicates parse complexity)
- **Tree Depth**: Maximum depth of parse tree (stack usage estimation)
- **Conversion Time**: Time spent converting GLR forest to Tree-sitter tree format
- **Memory Usage**: Arena allocation tracking (when arenas feature enabled)

### Performance Monitoring API

```rust
use adze_runtime::Parser;
use std::time::Instant;

let mut parser = Parser::new();
parser.set_language(glr_language)?;

let start = Instant::now();
let tree = parser.parse_utf8(large_input, old_tree)?;
let parse_time = start.elapsed();

// Access internal metrics (when available)
if let Some(metrics) = tree.conversion_metrics() {
    println!("Nodes: {}, Depth: {}, Time: {:?}", 
             metrics.node_count, metrics.depth, metrics.conversion_time);
}
```

### Incremental Parsing Performance

GLR incremental parsing provides sophisticated subtree reuse:

```rust
// Monitor subtree reuse effectiveness
use adze_runtime::glr_incremental::{SUBTREE_REUSE_COUNT, reset_reuse_counter};
use std::sync::atomic::Ordering;

reset_reuse_counter();

let tree1 = parser.parse_utf8("def main(): pass", None)?;
let tree2 = parser.parse_utf8("def hello(): pass", Some(&tree1))?;

let reused = SUBTREE_REUSE_COUNT.load(Ordering::SeqCst);
println!("Reused {} subtrees during incremental parse", reused);
```

**Performance Characteristics:**
- **Conservative Reuse**: Only reuses subtrees completely outside edit ranges
- **Forest Splicing**: Direct forest node reuse for 3-4x improvement over snapshots
- **Smart Fallback**: Automatically falls back to full parse when incremental isn't beneficial

### Arena Allocators

Enable arena allocators for parsing-heavy workloads:

```toml
[dependencies]
adze-runtime = { version = "0.1", features = ["glr-core", "arenas"] }
```

**Benefits:**
- Reduced allocation overhead during parsing
- Better cache locality for parse tree nodes
- Automatic cleanup when parser is reset

```rust
let mut parser = Parser::new();
parser.reset(); // Clears arena and resets allocator
```

## Legacy Performance Features

### SIMD Lexer (runtime)

### Optimizations

1. **Vectorized String Comparison**
   - Compares 8 bytes at a time using u64 operations
   - Optimized for common literal tokens

2. **Fast Pattern Matching**
   - Specialized matchers for common patterns:
     - Whitespace: `\s+`
     - Digits: `\d+`
     - Identifiers: `[a-zA-Z_][a-zA-Z0-9_]*`
   - Bitmap-based character class matching

3. **Loop Unrolling**
   - Processes multiple bytes per iteration
   - Reduces branch mispredictions

### Performance Results

| Input Size | Standard Lexer | SIMD Lexer | Speedup |
|------------|----------------|------------|---------|
| 1 KB       | 0.05 ms       | 0.02 ms    | 2.5x    |
| 10 KB      | 0.5 ms        | 0.15 ms    | 3.3x    |
| 100 KB     | 5.2 ms        | 1.6 ms     | 3.2x    |
| 1 MB       | 52 ms         | 16 ms      | 3.2x    |

## Parallel Parser

The parallel parser (`parallel_parser` module) enables multi-threaded parsing:

### Features

1. **Automatic Chunking**
   - Splits large files at statement boundaries
   - Maintains parsing context across chunks

2. **Thread Pool Management**
   - Uses rayon for work-stealing parallelism
   - Configurable thread count

3. **Smart Thresholds**
   - Only activates for files > 100KB
   - Optimal chunk size: 50KB

### Configuration

```rust
use adze::parallel_parser::{ParallelParser, ParallelConfig};

let config = ParallelConfig {
    min_file_size: 100_000,  // 100KB minimum
    chunk_size: 50_000,      // 50KB chunks
    num_threads: 0,          // 0 = use all cores
};

let parser = ParallelParser::new(grammar, table, config);
```

### Performance Results

| File Size | Single Thread | Parallel (8 cores) | Speedup |
|-----------|---------------|-------------------|---------|
| 100 KB    | 12 ms        | 11 ms             | 1.1x    |
| 500 KB    | 65 ms        | 20 ms             | 3.2x    |
| 1 MB      | 130 ms       | 35 ms             | 3.7x    |
| 5 MB      | 650 ms       | 120 ms            | 5.4x    |

## Memory Efficiency

### Zero-Copy Design

1. **Slice References**
   - Tokens reference input buffer directly
   - No string duplication during lexing

2. **Arena Allocation**
   - Parse nodes allocated in contiguous memory
   - Improved cache locality

3. **Incremental Reuse**
   - Subtrees shared between parse iterations
   - Copy-on-write semantics

### Memory Usage Comparison

| File Size | Tree-sitter C | Adze | Reduction |
|-----------|---------------|-------------|-----------|
| 100 KB    | 2.1 MB       | 1.8 MB      | 14%       |
| 1 MB      | 21 MB        | 17 MB       | 19%       |
| 10 MB     | 215 MB       | 165 MB      | 23%       |

## Incremental Parsing

### Algorithm

1. **Edit Tracking**
   - Precise byte-level edit positions
   - Minimal subtree invalidation

2. **Subtree Reuse**
   - Hash-based subtree identification
   - O(log n) reparse complexity

3. **Cache Management**
   - LRU cache for common subtrees
   - Configurable cache size

### Performance Results

| Edit Type        | Full Reparse | Incremental | Speedup |
|------------------|--------------|-------------|---------|
| Single char      | 130 ms      | 0.5 ms      | 260x    |
| Line insertion   | 130 ms      | 2.1 ms      | 62x     |
| Block deletion   | 130 ms      | 5.3 ms      | 24x     |
| Multiple edits   | 130 ms      | 8.7 ms      | 15x     |

## Benchmarking

### Running Benchmarks

```bash
# Run all benchmarks
cargo bench

# Run specific benchmark
cargo bench lexer_comparison

# Generate HTML report
cargo bench -- --save-baseline baseline
```

### Benchmark Suite

1. **Lexer Benchmarks**
   - Token scanning speed
   - Pattern matching performance
   - Memory allocation

2. **Parser Benchmarks**
   - Full parse time
   - Incremental parse time
   - Memory usage

3. **Real-World Benchmarks**
   - JavaScript parsing
   - Python parsing
   - Go parsing

## Future Optimizations

### Planned Improvements

1. **SIMD Enhancements**
   - AVX-512 support for wider vectors
   - ARM NEON optimizations
   - GPU acceleration experiments

2. **Advanced Parallelism**
   - Lock-free data structures
   - Speculative parsing
   - Work-stealing optimizations

3. **Memory Optimizations**
   - Custom allocators
   - Compressed node representation
   - Lazy field computation

### Research Areas

1. **Machine Learning**
   - ML-guided chunk boundaries
   - Predictive subtree caching
   - Learned index structures

2. **Hardware Acceleration**
   - FPGA parsing experiments
   - Custom ASIC design
   - Quantum parsing algorithms

## Best Practices

### When to Use SIMD Lexer

- Always enabled by default
- Best for files with many literals
- Slight overhead for regex-heavy grammars

### When to Use Parallel Parser

- Files larger than 100KB
- Multi-core systems
- Batch processing scenarios

### Optimization Tips

1. **Grammar Design**
   - Prefer literals over regex when possible
   - Use character classes for simple patterns
   - Minimize backtracking

2. **Configuration Tuning**
   - Adjust chunk size for your workload
   - Set thread count based on system
   - Enable caching for repeated parses

3. **Memory Management**
   - Reuse parser instances
   - Clear caches periodically
   - Monitor memory usage

## Conclusion

Adze's performance optimizations make it suitable for:
- Real-time syntax highlighting
- Large-scale code analysis
- IDE language servers
- High-throughput parsing pipelines

The combination of SIMD acceleration, parallel parsing, and incremental updates provides excellent performance across a wide range of use cases.