# Curiosity #001: PERFORMANCE_GUIDE.md.ISSUES_DOCUMENTED

**Discovered:** 2026-03-13
**Status:** Historical Record
**Category:** Documentation Transparency

---

## Summary

The file [`PERFORMANCE_GUIDE.md.ISSUES_DOCUMENTED`](../../../PERFORMANCE_GUIDE.md.ISSUES_DOCUMENTED) is a transparency document that catalogs critical issues discovered during a comprehensive codebase audit. It reveals that early performance metrics were based on mock implementations rather than real parsing operations.

---

## What the File Contains

The document is a detailed "mea culpa" that catalogs four interconnected issues:

### Issue #73: Mock Benchmarks
```rust
// What the benchmark claimed to measure:
for char in source.chars() {
    if char.is_alphanumeric() || char.is_whitespace() {
        tokens += 1;  // Just character counting!
    }
}
```
The benchmark claimed "815 MB/sec throughput" and "118M tokens/sec" but was actually measuring character iteration (~0.1ns/char), not parsing.

### Issue #74: Incomplete Lexer
```rust
// ALL grammars with transforms hit this broken path:
eprintln!("Warning: Custom lexer function provided but type conversion not yet implemented safely");
self.parse(input) // Falls back to broken parsing!
```
The lexer couldn't execute transform functions, meaning no real parsing worked for grammars with transforms.

### Issue #75: Fake GLR Benchmarks
```rust
// Claims to measure "GLR fork operations":
let forked = stacks[0].clone();  // Just Vec::clone (~85ns)
stacks.push(forked);            // Just Vec::push
```
GLR benchmarks measured `Vec::clone()` instead of actual GLR parsing operations (parse state duplication, grammar rule application, symbol table handling).

### Issue #76: False Documentation
Performance tables in documentation claimed:
- Rust: 10KB in 0.5ms, 2M tokens/sec
- JavaScript: 50KB in 2ms, 1.8M tokens/sec
- Python: 100KB in 3ms, 2.2M tokens/sec

These numbers were completely fictional.

---

## Why the File Exists

### The Context

During the transition from the original rust-sitter project to Adze, the team was building a pure-Rust GLR implementation. Development pressure led to:

1. **Placeholder benchmarks** being created to establish the testing infrastructure
2. **Mock implementations** that would "pass" tests while real implementations were being developed
3. **Documentation written optimistically** about what the system *would* do

### The Discovery

A comprehensive audit (documented in [`docs/archive/CRITICAL_ISSUES_SUMMARY.md`](../../archive/CRITICAL_ISSUES_SUMMARY.md)) revealed that:

- The impressive performance claims were based on mocks
- Real parsing didn't work for any grammar with transform functions
- The "Potemkin villages" were actually the entire performance and parsing story

### The Response

Rather than silently fixing the issues, the project chose transparency:

1. **Created `.ISSUES_DOCUMENTED` files** to catalog what was wrong
2. **Kept the files in the repository** as historical records
3. **Fixed the underlying implementations** (current benchmarks now use real parsing)

---

## The Story Behind It

### Timeline

| Phase | What Happened |
|-------|---------------|
| **Pre-Audit** | Mock benchmarks and optimistic documentation created during rapid development |
| **Discovery** | Audit revealed disconnect between claims and reality |
| **Documentation** | `.ISSUES_DOCUMENTED` files created to transparently catalog issues |
| **Remediation** | Real implementations completed, benchmarks fixed |
| **Present** | File remains as historical record and lesson |

### The Human Element

This pattern is common in software projects under pressure:

1. **Infrastructure First**: Build the test/benchmark infrastructure
2. **Placeholders**: Use mocks to validate infrastructure works
3. **Optimism**: Document what the system *should* do
4. **Drift**: Placeholders become "good enough" and reality diverges from docs
5. **Reckoning**: Audit discovers the gap
6. **Remediation**: Fix the code, update the docs

The Adze project chose **transparency over concealment** — keeping the `.ISSUES_DOCUMENTED` files as a reminder and lesson.

---

## Lessons Learned

### 1. Mocks Should Be Obvious

```rust
// BAD: Mock that looks real
fn parse_python(source: &str) -> usize {
    source.chars().filter(|c| c.is_alphanumeric()).count()
}

// GOOD: Mock that's clearly a placeholder
fn parse_python(source: &str) -> usize {
    #[cfg(not(feature = "real-parser"))]
    {
        compile_error!("Real parser not yet implemented - remove this mock to enable");
    }
    // ...
}
```

### 2. Documentation Should Match Reality

If the implementation doesn't exist, the documentation should say:
```markdown
⚠️ **Performance metrics are not yet available.**
Current benchmarks measure mock implementations, not real parsing.
```

### 3. Transparency Builds Trust

By keeping the `.ISSUES_DOCUMENTED` files visible, the project:
- Acknowledges past mistakes
- Demonstrates commitment to honesty
- Provides a learning opportunity for others

### 4. Audits Are Essential

Regular comprehensive audits can catch drift between:
- Documentation and implementation
- Benchmarks and actual work being measured
- Claims and reality

---

## Current State

The issues documented in this file have been addressed:

- **Benchmarks**: Now use real parsing with actual GLR operations
- **Lexer**: Transform functions are properly executed
- **Documentation**: Performance claims updated to reflect reality

The [`benchmarks/benches/glr_performance.rs`](../../../benchmarks/benches/glr_performance.rs) file now contains legitimate benchmarks:

```rust
fn benchmark_glr_parsing(c: &mut Criterion) {
    // Validate fixtures once up front so benches only measure parse work.
    for (label, source) in &[...] {
        assert!(
            parse(source).is_ok(),
            "Fixture {} must parse successfully for perf benchmarks",
            label
        );
    }
    // Real parser workload: parse valid arithmetic expressions.
    // ...
}
```

---

## Related Files

- [`PERFORMANCE_GUIDE.md.ISSUES_DOCUMENTED`](../../../PERFORMANCE_GUIDE.md.ISSUES_DOCUMENTED) - The original transparency document
- [`docs/archive/CRITICAL_ISSUES_SUMMARY.md`](../../archive/CRITICAL_ISSUES_SUMMARY.md) - Comprehensive audit findings
- [`docs/archive/DOCUMENTATION_CLEANUP_2025-11-15.md`](../../archive/DOCUMENTATION_CLEANUP_2025-11-15.md) - Documentation reorganization notes
- [`benchmarks/benches/glr_performance.rs`](../../../benchmarks/benches/glr_performance.rs) - Current (real) benchmarks

---

## Takeaway

> *"The file remains not as an accusation, but as a reminder: in software development, the gap between what we claim and what we build can grow silently. Transparency documents like this one serve as both historical record and guardrail against future drift."*

---

**Documented by:** AI Agent Investigation
**Date:** 2026-03-13
**Series:** Adze Curiosities
