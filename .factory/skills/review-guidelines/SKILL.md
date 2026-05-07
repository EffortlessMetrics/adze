# Adze Droid Review Guidelines

This skill provides context for automated code review in the Adze repository using Factory Droid.

## Key Review Rules

### No Naked LGTM
Every Droid review comment must be actionable or explain why no action is needed. Do not post bare approval without supporting evidence.

### Repair-Packet Findings
Findings are structured with:
- **Title**: Short summary of the issue
- **Failure mode**: Why this matters and what breaks
- **Why here**: Context about where and why it appears
- **Fix direction**: Actionable next steps
- **Validation**: How to verify the fix
- **Confidence**: High/Medium/Low confidence level

Example:
```
[P1] Unsafe unwrap in parser path

Failure mode: Panics on malformed input instead of graceful error recovery.
Why here: line 127 in parser.rs assumes token exists; it can be None during sync.
Fix direction: Replace unwrap() with match/ok_or to return ParseError.
Validation: Run `cargo test parse_recovery` to verify error handling.
Confidence: High — code path directly exposed to untrusted input.
```

### Evidence Provenance
Split observations into three categories:

- **Observed**: What was found by static analysis or test run (defects, unused code, unreachable paths)
- **Reported**: What claims PR description or comments make (assumptions about behavior)
- **Not verified**: What cannot be confirmed in this context (runtime behavior under load, production incidents)

Never treat PR-body validation claims as independently verified.

### Clean Review Structure
When no actionable findings exist:

```
No actionable findings emitted.

Inspected surfaces:
- [list what was checked: logic, error paths, thread safety, resource cleanup, etc.]

Checks performed:
- [list checks: type safety, data flow, bounds checking, panic prevention, etc.]

Why no comments:
[explain the coverage and why patterns are sound]

Residual risk:
[note any assumptions, untested paths, or areas outside review scope]

Validation signal:
  Observed: [facts from code/tests]
  Reported: [claims from PR body — treat as claims, not confirmation]
  Not verified: [areas needing operational data]
```

### No Extra @mentions
Do not add `@author` or `@reviewer` tags in Droid review comments unless explicitly calling for expertise in a specific domain.

## Adze-Specific Context

### Core Pipeline Crates (7 — PR gate scope)
- `adze` (runtime) — Extract trait, parsing
- `adze-macro` — Proc-macro attributes only
- `adze-tool` — Build-time code generation
- `adze-common` — Shared grammar expansion
- `adze-ir` — Grammar IR with GLR
- `adze-glr-core` — GLR parser generation, conflict resolution
- `adze-tablegen` — Table compression, FFI generation

### Key Invariants for Code Review
1. **Type-driven design**: Parser shape is defined by Rust types, not YAML. Macros mark; tool generates.
2. **Compile-time vs Build-time**: Macros mark types (compile time); `build.rs` calls tool to generate (build time).
3. **Pure-Rust GLR**: No C code generation for standard parsers. Tree-sitter integration via Rust bindings.
4. **Table compression**: Generated parse tables must match Tree-sitter ABI bit-for-bit.
5. **No panics in grammar paths**: Parser must recover gracefully from all inputs.

### Common Review Patterns
- **Snapshot changes**: `cargo insta review` is the source of truth. Always verify snapshot intent.
- **Unsafe code**: Must be in carefully scoped blocks with invariant documentation.
- **Error recovery**: GLR must not panic on malformed input.
- **Workspace consistency**: MSRV 1.92.0, Rust 2024 edition in all crates.

### Testing Scope
- **Unit tests**: `adze`, `adze-ir`, `adze-glr-core`, `adze-tablegen`
- **Integration tests**: `adze-golden-tests` (Tree-sitter parity)
- **Feature matrix**: `scripts/test-matrix.sh` for feature combinations
- **Mutation testing**: `cargo mutate` for critical paths

## Tone and Approach
- Assume competent maintainers; focus on correctness, not pedagogy.
- Priority: data flow safety, error recovery, invariant preservation.
- Reference CLAUDE.md for architecture, justfile for commands.
- Verify claims with code excerpts, not assumptions.
