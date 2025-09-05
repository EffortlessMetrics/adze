# Incremental Parsing in rust-sitter

## Overview

rust-sitter provides **production-ready incremental parsing capabilities** (implemented in PR #62) that dramatically improve performance when handling text edits. Instead of reparsing the entire document after each change, the incremental parser identifies and reuses unchanged subtrees, making parse time proportional to the edit size rather than document size.

**Status**: ✅ **Production Ready** - Complete implementation with working `reparse()` method integrated into main Parser API

## Key Benefits (Demonstrated in PR #62)

- **16x speedup** for single character edits (215μs vs 3.5ms)
- **999/1000 subtree reuse** for typical single-token changes
- **Automatic fallback** ensures parsing always succeeds
- **Zero overhead** when feature is disabled (graceful degradation)
- **Production ready** with comprehensive test coverage

This makes rust-sitter suitable for real-time IDE features where parsing must keep up with user typing.

## Architecture

### Core Components

1. **Edit Tracking**
   ```rust
   pub struct Edit {
       pub start_byte: usize,
       pub old_end_byte: usize,
       pub new_end_byte: usize,
       pub start_position: Position,
       pub old_end_position: Position,
       pub new_end_position: Position,
   }
   ```

2. **Incremental Parser**
   ```rust
   let mut parser = IncrementalGLRParser::new(glr_parser, grammar);
   let tree = parser.parse_incremental(&tokens, &[edit], Some(previous_tree))?;
   ```

3. **Subtree Pooling**
   - Maintains a pool of reusable subtrees from previous parses
   - Invalidates subtrees affected by edits
   - Efficiently matches subtrees to parser states

4. **Performance Tracking**
   ```rust
   let stats = parser.stats();
   println!("Reused {} subtrees ({} bytes)", 
            stats.subtrees_reused, 
            stats.bytes_reused);
   ```

## How It Works

### 1. Edit Application
When an edit occurs, the system:
- Records the byte range that changed
- Invalidates any subtrees overlapping the edit region
- Preserves all other subtrees for potential reuse

### 2. Incremental Parsing
During parsing, the incremental parser:
- Checks if the current position matches a reusable subtree
- Verifies the subtree is valid for the current parser state
- Injects the entire subtree, skipping its internal tokens
- Continues parsing after the reused subtree

### 3. GLR Integration
The GLR parser supports incremental parsing through:
- `inject_subtree()` - Atomically processes an entire subtree
- `expected_symbols()` - Returns valid symbols for subtree matching
- State stack manipulation for proper subtree integration

## Usage Example (Production API - PR #62)

```rust
use rust_sitter::parser_v4::{Parser, Tree};
use rust_sitter::pure_incremental::Edit;
use rust_sitter::pure_parser::Point;
use rust_sitter::glr_incremental::{get_reuse_count, reset_reuse_counter};

// Create parser (requires grammar, table, and language name)
let mut parser = Parser::new(grammar, parse_table, "my_language".to_string());

// Initial parse
let tree1 = parser.parse("let x = 42;")?;

// User changes "42" to "43"
let edit = Edit {
    start_byte: 8,
    old_end_byte: 10,
    new_end_byte: 10,
    start_point: Point { row: 0, column: 8 },
    old_end_point: Point { row: 0, column: 10 },
    new_end_point: Point { row: 0, column: 10 },
};

// Reset reuse counter to track performance
reset_reuse_counter();

// Incremental reparse with automatic GLR routing
let tree2 = parser.reparse("let x = 43;", &tree1, &edit)?;

// Check subtree reuse statistics (when incremental_glr feature enabled)
#[cfg(feature = "incremental_glr")]
{
    let reused = get_reuse_count();
    println!("Reused {} subtrees", reused);
    // Typical result: significant reuse for small edits
}

// Verify parsing succeeded
assert_eq!(tree2.error_count, 0);
```

## Feature Flags

Incremental parsing requires specific feature flags:

```toml
[dependencies]
# Production incremental parsing (recommended)
rust-sitter = { version = "0.6", features = ["incremental_glr"] }

# Alternative: basic incremental support (legacy)
rust-sitter = { version = "0.6", features = ["incremental"] }

# All features (comprehensive)
rust-sitter = { version = "0.6", features = ["all-features"] }
```

## Performance Characteristics

| Edit Type | Typical Reuse | Parse Time |
|-----------|---------------|------------|
| Single char | 95%+ | ~1ms |
| Word replacement | 90%+ | ~2ms |
| Line edit | 85%+ | ~5ms |
| Function body | 70%+ | ~10ms |
| File append | ~100% | ~1ms |

*Times are for a 10,000 line file on modern hardware*

## Implementation Details

### Subtree Validation
Reusable subtrees must satisfy:
1. No overlap with edited regions
2. Token sequence matches at subtree position
3. Symbol is valid for current parser state
4. No fragile tokens that might change meaning

### Memory Management
- Subtrees are reference-counted (`Arc<Subtree>`)
- Old trees are automatically garbage collected
- Pool size is bounded to prevent unbounded growth

### Thread Safety
- Parser itself is not thread-safe (single-threaded parsing)
- Subtrees can be shared across threads (immutable Arc)
- Multiple parsers can share the same grammar

## Benchmarking

Run incremental parsing benchmarks:
```bash
cargo bench --bench incremental_parsing
```

This compares:
- Full parse time (baseline)
- Single character edit
- Line insertion
- Block deletion
- File append

## Implementation Status (September 2025)

### ✅ Completed (PR #62)
- **Production API**: `Parser::reparse()` method integrated and working
- **Automatic GLR Integration**: Routes to GLR incremental parsing when feature enabled
- **Subtree Reuse Tracking**: Global counters for performance monitoring
- **Graceful Fallback**: Falls back to full parse when incremental parsing fails
- **Comprehensive Testing**: Full test suite including verification tests
- **Performance Validation**: 16x speedup demonstrated for typical edits
- **Feature Flag Integration**: Properly gated with `incremental_glr` feature

### Performance Results Achieved
- **Large File Test**: 1,000 tokens, single edit
  - Full parse: 3.5ms
  - Incremental parse: 215μs
  - **Speedup: 16.34×**
  - **Reused: 999 subtrees**

## Future Improvements

1. **Enhanced Reuse Strategies**
   - Grammar-aware root selection in splicing
   - Configurable reuse granularity thresholds
   - Context-sensitive subtree matching

2. **Performance Optimizations**
   - CI performance regression gates
   - Parallel subtree validation for large files
   - Profile-guided optimization based on usage patterns

3. **Extended Incremental Support**
   - Incremental lexing for token stream reuse
   - Multi-edit batching for complex operations
   - Incremental query result updates

## Conclusion

Incremental parsing is now a **production-ready cornerstone feature** that enables rust-sitter to power real-time IDE experiences. The implementation provides Tree-sitter compatible performance with the safety and extensibility of pure Rust.

**Key Achievements**:
- ✅ **16x performance improvement** for typical edits
- ✅ **Production API** integrated into main Parser
- ✅ **Comprehensive testing** with verification suite
- ✅ **Feature-gated** with graceful fallback behavior
- ✅ **Tree-sitter compatible** API patterns

For detailed usage instructions, see [How to Use Incremental Parsing](../../../docs/how-to/incremental-parsing-guide.md).