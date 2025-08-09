# Rust-Sitter v0.6.0 Tracking Issues

This document tracks known issues and planned improvements for the rust-sitter project.

## High Priority Issues

### 1. Implement `Parser::reparse` for Incremental Parsing
**Owner:** TBD  
**Milestone:** v0.6.1  
**Status:** Not Started

The incremental parsing tests expect a `reparse` method that doesn't exist yet. This is critical for the incremental parsing feature.

**Tasks:**
- [ ] Implement `Parser::reparse` in `runtime/src/parser_v4.rs`
- [ ] Add unit tests for reparse functionality
- [ ] Update incremental integration tests to use the new method
- [ ] Document the API and usage patterns

### 2. Fix Benchmark Compilation Issues
**Owner:** TBD  
**Milestone:** v0.6.0  
**Status:** In Progress

Several benchmarks are currently failing to compile or run.

**Tasks:**
- [ ] Fix compilation errors in benchmark suite
- [ ] Ensure all benchmarks run in CI
- [ ] Add performance regression detection

### 3. Documentation Debt
**Owner:** TBD  
**Milestone:** v0.6.1  
**Status:** Not Started

Several modules have `missing_docs` warnings, particularly in `glr-core`.

**Tasks:**
- [ ] Document all public APIs in `glr-core`
- [ ] Document all public APIs in `tablegen`
- [ ] Document all public APIs in `common`
- [ ] Update module-level documentation

### 4. MSRV Workspace Inheritance
**Owner:** TBD  
**Milestone:** v0.6.0  
**Status:** Not Started

Ensure consistent Rust version requirements across all crates.

**Tasks:**
- [ ] Add `rust-version.workspace = true` to all internal crates
- [ ] Set explicit MSRV for published crates
- [ ] Update CI to test against MSRV

## Medium Priority Issues

### 5. External Scanner FFI Integration
**Owner:** TBD  
**Milestone:** v0.7.0  
**Status:** Partially Complete

Complete the FFI integration for external scanners in pure-Rust mode.

**Tasks:**
- [ ] Fix remaining FFI signature issues
- [ ] Add comprehensive tests for external scanners
- [ ] Document external scanner API

### 6. GLR Runtime Performance
**Owner:** TBD  
**Milestone:** v0.7.0  
**Status:** Research Phase

The GLR fork/merge logic needs performance optimization for large files.

**Tasks:**
- [ ] Profile GLR parser on large files
- [ ] Implement stack sharing optimizations
- [ ] Add performance benchmarks for GLR mode

## Low Priority / Future Work

### 7. Grammar-Aware Root Selection
**Owner:** TBD  
**Milestone:** Future  
**Status:** Not Started

Implement more sophisticated root selection for incremental parsing.

### 8. Configurable Reuse Thresholds
**Owner:** TBD  
**Milestone:** Future  
**Status:** Not Started

Allow users to configure when incremental reuse is applied.

### 9. Advanced Parsing Strategies
**Owner:** TBD  
**Milestone:** Future  
**Status:** Research

Explore hybrid LR/GLR parsing and action-replay caching.

## How to Use This Document

1. **For Contributors:** Pick an issue without an owner and comment on the tracking issue
2. **For Maintainers:** Update status and assignments regularly
3. **For Users:** Watch this document for updates on features you need

## Issue Creation Template

When creating a GitHub issue for any of these items, use this template:

```markdown
### Task: [Task Name from Above]

**Tracking:** Part of #[tracking issue number]
**Priority:** [High/Medium/Low]
**Milestone:** [version]

#### Description
[Detailed description of the work needed]

#### Acceptance Criteria
- [ ] [Specific measurable outcome]
- [ ] [Another specific outcome]

#### Technical Notes
[Any relevant technical context or constraints]
```

## Release Blocking Issues

The following issues MUST be resolved before v0.6.0 release:
- None currently (v0.6.0 can ship as-is)

The following issues SHOULD be resolved before v0.6.1:
- Implement `Parser::reparse` (#1)
- Fix benchmark compilation (#2)