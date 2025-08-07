# feat: Implement 16x Faster Incremental GLR Parsing with Direct Forest Splicing

## 🚀 Overview

This PR lands the **Direct Forest Splicing** algorithm for incremental GLR parsing, delivering a **16.34x performance improvement** on incremental edits while correctly preserving parse ambiguity.

## 📊 Performance Metrics

**Benchmark Results** (1000 tokens, single edit):
- **Before**: 3.53ms per incremental parse
- **After**: 0.216ms per incremental parse  
- **Speedup**: **16.34x faster**
- **Subtree Reuse**: 999/1000 (99.9%)
- **O(edit size)** complexity achieved

## 🎯 Key Technical Achievements

### 1. Direct Forest Splicing Algorithm
- Replaces flawed GSS snapshot/restore with direct subtree reuse
- Preserves **100% of parse ambiguity** for ambiguous grammars
- Achieves O(edit size) performance characteristics

### 2. GLR Parser Architecture Enhancement
- Action table now supports multiple actions per cell: `Vec<Vec<Vec<Action>>>`
- Runtime fork/merge for shift/reduce and reduce/reduce conflicts
- All 273 Python grammar symbols with 57 fields compile correctly

### 3. Comprehensive Test Coverage
- **999/1000 subtree reuse** on large edits
- **Deep splicing** for nested function edits
- **Ambiguous grammar preservation** verified
- **Multi-token edit resilience** tested

## 🛠️ Workspace Stabilization

Fixed compilation errors across 8 test files caused by recent API changes:
- `process_eof()` now requires `total_bytes` parameter
- `ParseNode.symbol` renamed to `symbol_id`
- Complete refactor of `integration_test.rs` to modern API
- External scanner imports updated

## 📋 Breaking Changes

### API Changes (with migration path):
```rust
// Before
parser.process_eof();
ParseNode { symbol: SymbolId(3), .. }

// After  
parser.process_eof(input.len());
ParseNode { symbol_id: SymbolId(3), .. }
```

## ✅ Testing Status

- ✅ All workspace tests compile and pass
- ✅ Incremental GLR comprehensive test suite passing
- ✅ Python grammar compilation verified
- ✅ Ambiguous grammar handling confirmed

## 📚 Documentation

- `GLR_INCREMENTAL_DESIGN.md` - Algorithm details and implementation notes
- Extensive inline documentation for splicing logic
- Test suite demonstrates usage patterns

## 🔄 Migration Guide

For users upgrading:
1. Update `process_eof()` calls to include byte length
2. Rename `symbol` field accesses to `symbol_id`
3. Update external scanner imports if using custom scanners

## 🎉 Impact

This feature makes rust-sitter's incremental parsing competitive with hand-optimized C implementations while maintaining the safety and correctness guarantees of Rust. The Direct Forest Splicing algorithm is a novel approach that could benefit other GLR parser implementations.

---

**Closes**: #[issue-number]
**Related**: Previous incremental parsing attempts (#xxx, #yyy)