# Rust-Sitter v0.6.0 Tracking Issues

This document tracks known issues and planned improvements for the rust-sitter project.

## High Priority Issues

### 1. Implement `Parser::reparse` for Incremental Parsing
**Owner:** TBD  
**Milestone:** v0.6.1  
**Status:** Not Started

**Acceptance Criteria:**
- [ ] Implement `reparse` method in GLR parser
- [ ] All incremental tests pass without `#[ignore]` attributes
- [ ] Performance benchmarks show expected gains
- [ ] Property tests confirm equivalence with fresh parse

**Related Files:**
- `runtime/src/glr_incremental.rs`
- `runtime/tests/incremental_*_test.rs`

---

### 2. Restore Benchmark Suite
**Owner:** TBD  
**Milestone:** v0.6.1  
**Status:** Not Started

**Acceptance Criteria:**
- [ ] All benchmarks compile and run successfully
- [ ] CI runs benchmarks on PRs with performance comparison
- [ ] No regression > 10% from baseline

**Related Files:**
- `runtime/benches/`
- `.github/workflows/benchmarks.yml`

---

### 3. Fix Missing Documentation Warnings
**Owner:** TBD  
**Milestone:** v0.6.2  
**Status:** Not Started

**Acceptance Criteria:**
- [ ] Zero `missing_docs` warnings in public crates
- [ ] Internal crates have `#[allow(missing_docs)]` where appropriate
- [ ] All public APIs have comprehensive documentation

**Affected Crates:**
- `rust-sitter-glr-core` (123 warnings)
- `rust-sitter-tablegen` (multiple warnings)
- `rust-sitter-common` (11 warnings)

---

## Medium Priority Issues

### 4. Grammar-Aware Root Selection
**Owner:** TBD  
**Milestone:** v0.7.0  
**Status:** Not Started

**Description:** Improve splicing algorithm to use grammar knowledge for more robust root selection.

---

### 5. Configurable Reuse Thresholds
**Owner:** TBD  
**Milestone:** v0.7.0  
**Status:** Not Started

**Description:** Allow configuration of when to apply subtree reuse vs fresh parsing.

---

### 6. Deterministic Codegen Verification
**Owner:** TBD  
**Milestone:** v0.6.2  
**Status:** In Progress

**Acceptance Criteria:**
- [ ] CI checks determinism on Linux
- [ ] CI checks determinism on macOS
- [ ] Zero non-deterministic build failures

---

## Low Priority / Future Work

### 7. Hybrid LR/GLR Parser Mode
**Owner:** TBD  
**Milestone:** Future  
**Status:** Research

**Description:** Explore switching between LR and GLR modes based on grammar analysis.

---

### 8. Action-Replay Caching
**Owner:** TBD  
**Milestone:** Future  
**Status:** Research

**Description:** Cache and replay parse actions for common patterns.

---

## Issue Template

When creating new tracking issues, use this template:

```markdown
### N. [Issue Title]
**Owner:** [GitHub username or TBD]  
**Milestone:** [version number]  
**Status:** [Not Started | In Progress | Blocked | Complete]

**Description:** [Brief description of the issue]

**Acceptance Criteria:**
- [ ] [Specific, measurable criterion]
- [ ] [Another criterion]

**Related Files:**
- [file path]
- [file path]

**Notes:**
[Any additional context, blockers, or dependencies]
```

## Status Definitions

- **Not Started**: Issue has not been worked on
- **In Progress**: Active development is happening
- **Blocked**: Work is blocked by external dependencies
- **Complete**: All acceptance criteria met and merged to main