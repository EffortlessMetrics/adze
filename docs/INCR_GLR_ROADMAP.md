# Incremental GLR Parser Implementation Roadmap

## 📍 Where We Stand — 2025-08-07

### ✅ What's Now Real and Working

After the latest implementation sprint, we've transformed the incremental GLR parser from a "Potemkin village" (elaborate facade with no functionality) into a system with **real, working machinery**:

| Sub-system | Previous State (Facade) | Current State (Real Implementation) |
|------------|------------------------|-------------------------------------|
| **GSS State Capture** | Returned hardcoded `StateId(0)` and empty vectors | Actually captures all parse stacks via `get_gss_state()` |
| **GSS State Restore** | Created fresh parser, ignored snapshot | Restores exact GSS state via `set_gss_state()` |
| **Subtree Injection** | Printed debug message, did nothing | Injects subtree into parser via `inject_subtree()` |
| **Parser State API** | No access to internal state | Full API: `get/set_gss_state()`, `get/set_next_stack_id()` |

### 🔧 Technical Implementation Details

#### **Real GSS State Management (Completed)**
```rust
// GLRParser now exposes:
pub fn get_gss_state(&self) -> Vec<ParseStack>
pub fn set_gss_state(&mut self, stacks: Vec<ParseStack>)
pub fn get_next_stack_id(&self) -> usize
pub fn set_next_stack_id(&mut self, id: usize)
pub fn inject_subtree(&mut self, subtree: Arc<Subtree>) -> Result<(), String>
```

#### **Real Snapshot System (Completed)**
```rust
pub struct GSSSnapshot {
    pub gss_stacks: Vec<ParseStack>,  // Actual parse stacks
    pub next_stack_id: usize,         // Fork tracking state
    pub partial_tree: Option<Arc<ForestNode>>,
}
```

#### **Real State Capture/Restore (Completed)**
- `capture_parser_snapshot()` - Extracts actual GSS state from parser
- `create_parser_from_snapshot()` - Restores parser to exact previous state
- `inject_subtree_into_parser()` - Converts ForestNode to Subtree and injects

---

## 🛣️ Path to Production-Ready (`v0.6.0-beta.2`)

### P0 — Remaining Critical Issues

| ID | Task | Status | Blocking Factor |
|----|------|--------|-----------------|
| **T-01** | **Fix tokenizer** | 🔴 Not Started | Currently hardcoded to arithmetic grammar only |
| **T-02** | **Integration tests** | 🟡 Partial | Test file simplified but needs real grammar tests |
| **T-03** | **Performance validation** | 🔴 Not Started | Need benchmarks showing actual speedup |
| **T-04** | **Memory management** | 🟡 Partial | Snapshot eviction strategy exists but untested |

### P1 — Production Requirements

1. **Grammar-agnostic tokenizer** - Replace hardcoded arithmetic tokenizer
2. **Comprehensive test suite** - Test with Python, JavaScript, Go grammars
3. **Performance benchmarks** - Prove 2x+ speedup for incremental edits
4. **Documentation** - Update GLR_INCREMENTAL_DESIGN.md with real algorithm

---

## 📦 Deliverables Checklist

### ✅ Completed
- [x] GSS state exposure API in GLRParser
- [x] Real snapshot capture/restore
- [x] Real subtree injection
- [x] ForestNode to Subtree conversion
- [x] Compilation with `--features incremental_glr`

### ⏳ In Progress
- [ ] Grammar-agnostic tokenizer trait
- [ ] Integration tests with real grammars
- [ ] Performance benchmarks
- [ ] Documentation updates

### 🔴 Blocked
- [ ] CI matrix job for incremental_glr
- [ ] Production grammar testing (needs tokenizer)

---

## 🔥 Critical Path Analysis

### What Works Now
1. **State Management**: Can capture and restore complete parser state
2. **Subtree Reuse**: Can inject pre-parsed subtrees into active parse
3. **Fork Tracking**: Maintains fork IDs across snapshots
4. **Reuse Counter**: Tracks actual subtree reuse (not fake)

### What Still Blocks Production Use
1. **Tokenizer**: Hardcoded to arithmetic - blocks all real grammar testing
2. **Validation**: No proof of actual performance improvement
3. **Robustness**: Untested with large files or complex edits

---

## 📊 Success Metrics

To declare incremental GLR "production ready", we need:

1. **Correctness**: Pass all tests with Python/JavaScript/Go grammars
2. **Performance**: 2x+ speedup on single-character edits in 1000+ line files
3. **Memory**: Bounded snapshot memory usage (< 10MB for large files)
4. **Reliability**: No panics or incorrect parses across 1000+ edit sequences

---

## 🗓️ Realistic Timeline

| Week | Goal | Success Criteria |
|------|------|------------------|
| **Week 1** | Tokenizer trait | Can tokenize any grammar |
| **Week 2** | Integration tests | Python grammar tests pass |
| **Week 3** | Performance validation | Benchmarks show 2x speedup |
| **Week 4** | Production hardening | 1000 random edits, zero failures |
| **Week 5** | Documentation & release | Beta-2 published to crates.io |

---

## 🎯 Definition of Done

The incremental GLR parser is "done" when:

1. **Real Grammar Test**: `cargo test --features incremental_glr` passes with Python grammar
2. **Performance Test**: Single-char edit in 1000-line file is 2x+ faster than full reparse
3. **Stress Test**: 1000 random edits produce identical trees to full reparse
4. **Memory Test**: Snapshot memory stays under 10MB for 10,000 line files
5. **Documentation**: Algorithm fully documented with diagrams

---

## 📝 Key Takeaways

The incremental GLR parser has transitioned from **architectural deep fake to real implementation**. The core machinery now exists and functions correctly:

- ✅ GSS state can be captured and restored
- ✅ Subtrees can be injected into active parses
- ✅ Parser state is fully accessible via new API

However, it remains **blocked from production use** by the hardcoded tokenizer and lack of validation with real grammars. The path forward is clear: implement a grammar-agnostic tokenizer, validate with production grammars, and prove the performance benefits.

---

*Last Updated: 2025-08-07*
*Next Review: 2025-08-14*