# Adze Contributor's Guide

Welcome to the Adze project! This guide provides everything you need to know to contribute effectively to Adze, an AST-first grammar toolchain for Rust that generates Tree-sitter parsers from Rust type annotations using a pure-Rust GLR implementation.

## Table of Contents

1. [Getting Started](#getting-started)
2. [Development Workflow](#development-workflow)
3. [Code Standards](#code-standards)
4. [Testing Requirements](#testing-requirements)
5. [Documentation](#documentation)
6. [Release Process](#release-process)

---

## Getting Started

### Prerequisites

- **Rust**: Version 1.92.0 or later (managed via [`rust-toolchain.toml`](../../rust-toolchain.toml))
- **just**: Command runner for development tasks
- **Git**: For version control

### Development Environment Setup

The toolchain is auto-configured via `rust-toolchain.toml` — no manual Rust installation steps needed beyond having rustup installed.

```bash
# Verify toolchain
rustc --version   # Should be >= 1.92.0
just --version    # Command runner

# System dependency (only needed for ts-bridge)
# apt install libtree-sitter-dev
```

### Install Git Hooks

Adze uses version-controlled Git hooks for pre-commit and pre-push validation:

```bash
.githooks/install.sh
```

This creates symlinks from `.git/hooks/` to the hooks in `.githooks/`. See [Git Hooks](#git-hooks) for details on what these hooks check.

### Building the Project

```bash
# Build all workspace members
cargo build

# Build specific crate
cargo build -p adze

# Build everything using just
just build

# Build with release optimizations
just release
```

### Running Tests

```bash
# Core lib tests (recommended for daily development)
just test

# All tests with 2 threads (standard configuration)
cargo t2

# Test specific crate
cargo test -p adze-ir

# Run specific test by name
cargo test test_normalize_simple
```

### Essential Commands Reference

| Command | Description |
|---------|-------------|
| `just ci-supported` | **PR gate** - Required before submitting |
| `just test` | Core lib tests |
| `just fmt` | Format verification |
| `just clippy` | Linting on core crates |
| `just snap` | Review snapshot changes |
| `just build` | Build everything |

---

## Development Workflow

### Branch Naming Conventions

Use descriptive branch names that indicate the type of work:

| Prefix | Purpose | Example |
|--------|---------|---------|
| `feature/` | New functionality | `feature/glr-incremental` |
| `fix/` | Bug fixes | `fix/symbol-id-allocation` |
| `refactor/` | Code restructuring | `refactor/parse-table-compression` |
| `docs/` | Documentation changes | `docs/api-reference` |
| `test/` | Test improvements | `test/glr-coverage` |
| `chore/` | Maintenance tasks | `chore/update-dependencies` |

### Commit Message Format

Write clear, descriptive commit messages following these guidelines:

```
<type>: <short summary>

<body explaining what and why>
```

**Types**: `feat`, `fix`, `refactor`, `docs`, `test`, `chore`

**Example**:
```
feat: add incremental parsing support for GLR runtime

Implement incremental parsing by tracking tree edits and reusing
parse state from unchanged regions. This improves performance for
IDE scenarios with frequent small edits.
```

### PR Process

1. **Create a feature branch** from `main` or `master`
2. **Make your changes** following code standards
3. **Run the PR gate** before submitting:
   ```bash
   just ci-supported
   ```
4. **Push and create PR** with a clear description
5. **Address review feedback** promptly

#### PR Gate Requirements

The `just ci-supported` command is the **single required check** for branch protection. It runs:

1. `cargo fmt --all -- --check` - Format verification
2. `cargo clippy` on core crates - Linting with `-D warnings`
3. `cargo test` on core crates - All tests
4. Doc tests with serialization feature

#### Core Pipeline Crates (PR Gate Scope)

These 7 crates are checked by `just ci-supported`:

| Crate | Path | Purpose |
|-------|------|---------|
| `adze` | `runtime/` | Main runtime library, `Extract` trait |
| `adze-macro` | `macro/` | Proc-macro attributes |
| `adze-tool` | `tool/` | Build-time code generation |
| `adze-common` | `common/` | Shared grammar expansion |
| `adze-ir` | `ir/` | Grammar IR with GLR support |
| `adze-glr-core` | `glr-core/` | GLR parser generation |
| `adze-tablegen` | `tablegen/` | Table compression, FFI generation |

### Code Review Guidelines

**For Authors**:
- Provide context in PR description
- Link related issues
- Break large changes into logical commits
- Respond to all comments

**For Reviewers**:
- Be constructive and specific
- Focus on correctness, maintainability, and performance
- Approve only when all concerns are addressed
- Use GitHub's suggestion feature for code changes

### Git Hooks

#### pre-commit

Runs before each commit with these checks:

1. **Partial staging detection** - Fails if files have both staged and unstaged changes
2. **Targeted formatting** - Formats only staged Rust files
3. **Clippy analysis** - Full error reporting
4. **GOTO indexing pattern validation**
5. **SymbolId(0) misuse detection**
6. **Test connectivity verification** - No `.rs.disabled` files
7. **Optional quick tests** - If `RUN_QUICK_TESTS=1`

#### pre-push

Runs comprehensive validation before pushing:

1. Disabled test file detection in commits
2. Complete code formatting verification
3. Comprehensive clippy analysis
4. GOTO indexing pattern validation
5. Test connectivity verification
6. Core test suite execution
7. Optional full test suite (if `RUN_FULL_TESTS=1`)
8. Breaking change detection
9. Commit message validation

---

## Code Standards

### Rust Formatting

All code must pass `cargo fmt --all -- --check`:

```bash
# Check formatting
just fmt

# Auto-format all code
cargo fmt --all
```

### Clippy Lints

Clippy is run with `-D warnings` (warnings treated as errors):

```bash
# Run clippy on core crates
just clippy

# Run clippy on specific crate
cargo clippy -p adze-ir -- -D warnings
```

### Workspace Lints

Enforced via `[workspace.lints.rust]` in `Cargo.toml`:

| Lint | Level |
|------|-------|
| `unsafe_op_in_unsafe_fn` | deny |
| `unused_must_use` | deny |
| `missing_docs` | warn |
| `unused_extern_crates` | deny |

### Documentation Comments

All public APIs must have documentation comments:

```rust
/// Assert that a ParseTable respects all invariants.
///
/// # Panics
///
/// Panics if the EOF column index does not equal `token_count + external_token_count`.
pub fn assert_parse_table_invariants(table: &ParseTable) {
    // ...
}
```

**Guidelines**:
- Use `///` for documentation comments (not `//`)
- Include examples where helpful
- Document panics and safety requirements
- Use markdown formatting

### Error Handling Patterns

Adze uses a **layered error handling strategy** with clear ownership boundaries:

#### Library Crates: Use `thiserror`

```rust
#[derive(Debug, thiserror::Error)]
pub enum IrError {
    #[error("invalid symbol: {0}")]
    InvalidSymbol(String),

    #[error("duplicate rule: {0}")]
    DuplicateRule(String),

    #[error("internal error: {0}")]
    Internal(String),
}

pub type Result<T> = std::result::Result<T, IrError>;
```

#### Application Crates: Use `anyhow`

CLI tools and applications use `anyhow` for:
- Error chain accumulation
- Context attachment
- Simplified error propagation

```rust
use anyhow::{Context, Result};

fn process_file(path: &Path) -> Result<()> {
    let content = std::fs::read_to_string(path)
        .context(format!("Failed to read file: {}", path.display()))?;
    // ...
}
```

See [ADR-016: Error Handling Strategy](../adr/016-error-handling-strategy.md) for complete details.

### Dependencies

Always use workspace dependencies in your `Cargo.toml`:

```toml
[dependencies]
serde = { workspace = true }
thiserror = { workspace = true }
```

**Available workspace dependencies**: `tree-sitter`, `serde`, `serde_json`, `proptest`, `insta`, `criterion`, `thiserror`, `syn`, `quote`, `proc-macro2`, `anyhow`, `tempfile`, `indexmap`, `bincode`, `clap`, `rayon`, `rustc-hash`, `smallvec`, `regex`

### Collections

Use appropriate collection types for performance:

| Use Case | Type | Import |
|----------|------|--------|
| Hot paths (parser tables, symbol lookups) | `FxHashMap` | `rustc_hash::FxHashMap` |
| Small stack-allocated collections | `SmallVec` | `smallvec::SmallVec` |
| Insertion order matters | `IndexMap` | `indexmap::IndexMap` |

---

## Testing Requirements

### Test Coverage Expectations

- All new features must have tests
- Bug fixes must include regression tests
- Property-based tests for invariants
- Snapshot tests for output formats

### Test Naming Conventions

Follow the pattern: `test_<what>_<condition>_<expected>`

```rust
#[test]
fn test_normalize_optional_completes() { }

#[test]
fn test_grammar_builder_empty_rhs_produces_epsilon() { }

#[test]
fn test_parse_table_invalid_state_returns_error() { }
```

### Property-Based Testing with Proptest

Use `proptest` for testing invariants with random inputs:

```rust
use proptest::prelude::*;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn test_grammar_roundtrip(
        grammar in small_grammar_strategy()
    ) {
        let json = serde_json::to_string(&grammar).unwrap();
        let parsed: Grammar = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(grammar, parsed);
    }
}
```

**Shared strategies** are available in [`testing/src/strategies.rs`](../../testing/src/strategies.rs).

### Snapshot Testing with Insta

Use `insta` for output verification:

```rust
#[test]
fn normalize_simple_grammar() {
    let mut grammar = Grammar::new("simple".into());
    // ... setup grammar ...
    
    grammar.normalize();
    insta::assert_snapshot!("normalize_simple", render_rules(&grammar));
}
```

**Review snapshots**:
```bash
just snap
# Or:
cargo insta review
```

### BDD Testing

For complex parser scenarios, use **Behavior-Driven Development**:

```gherkin
Feature: GLR Conflict Detection and Preservation

  Scenario: Preserve Conflicts with Precedence Ordering (PreferShift)
    Given a shift/reduce conflict with precedence favoring shift
    When resolve_shift_reduce_conflict() is called
    Then both actions are preserved in order [shift, reduce]
    And the first action (shift) has higher runtime priority
```

See [ADR-007: BDD Framework for Parser Testing](../adr/007-bdd-framework-for-parser-testing.md).

### CI Requirements

The **PR gate** (`just ci-supported`) must pass before merging:

```bash
just ci-supported
```

This verifies:
1. Format compliance
2. Clippy passes with no warnings
3. All core tests pass
4. Doc tests compile

### Test Thread Configuration

Tests are configured for stability:

| Variable | Default | Purpose |
|----------|---------|---------|
| `RUST_TEST_THREADS` | 2 | Test concurrency |
| `RAYON_NUM_THREADS` | 4 | Rayon thread pool |
| `CARGO_BUILD_JOBS` | 4 | Parallel builds |

```bash
# Override thread count
RUST_TEST_THREADS=4 cargo test
```

### Verification Checklist

Before submitting any changes:

```bash
# 1. Format
cargo fmt --all

# 2. Lint
just clippy

# 3. Test
just test

# 4. Full PR gate
just ci-supported

# 5. If snapshots changed
cargo insta review
```

---

## Documentation

### When to Update Documentation

Update documentation when:
- Adding new public APIs
- Changing existing behavior
- Fixing bugs that affect user-facing behavior
- Adding new features or capabilities
- Modifying configuration options

### ADR Process

Architecture Decision Records (ADRs) capture important architectural decisions.

#### Creating a New ADR

1. Copy [`000-template.md`](../adr/000-template.md) to a new file with the next sequential number
2. Fill in the sections:
   - **Status**: Proposed, Accepted, Deprecated, or Superseded
   - **Context**: The issue motivating this decision
   - **Decision**: The change being made
   - **Consequences**: What becomes easier or harder
3. Update [`INDEX.md`](../adr/INDEX.md) with the new ADR
4. Submit for review

#### ADR Structure

```markdown
# ADR-NNN: Title

## Status
[Proposed|Accepted|Deprecated|Superseded]

## Context
What is the issue we are trying to address?

## Decision
What is the change being made?

## Consequences
### Positive
What benefits does this provide?

### Negative
What trade-offs does this introduce?

### Neutral
What other effects does this have?
```

### Code Comments

- Use `///` for documentation comments on public items
- Use `//` for implementation notes
- Document panics, safety requirements, and examples
- Keep comments up-to-date with code changes

---

## Release Process

### Version Numbering

Adze follows [Semantic Versioning](https://semver.org/):

- **MAJOR**: Incompatible API changes
- **MINOR**: Backwards-compatible new features
- **PATCH**: Backwards-compatible bug fixes

### Release Workflow

Releases are triggered via the [`release.yml`](../../.github/workflows/release.yml) workflow:

1. **Validate Release**
   - Load publish crate order
   - Get current version
   - Validate release surface
   - Validate version bump with `cargo-semver-checks`

2. **Version Bump**
   - Update `Cargo.toml` versions
   - Update lockfile

3. **Publish to crates.io**
   - Publish crates in dependency order
   - Dry run available for testing

### Changelog Updates

Before releasing:
1. Update `CHANGELOG.md` with:
   - New features
   - Bug fixes
   - Breaking changes
   - Deprecations
2. Include PR/issue references
3. Credit contributors

### Publication Steps

Releases are performed via GitHub Actions:

1. Go to **Actions** → **Release** workflow
2. Click **Run workflow**
3. Provide inputs:
   - **Version**: e.g., `1.2.3`
   - **Release surface mode**: `fixed` or `auto`
   - **Dry run**: Start with `true` for testing
4. Review the workflow output
5. Re-run with `dry_run: false` to publish

### Publish Order

View the publication order:

```bash
just publish-order
```

Crates are published in dependency order to ensure dependent crates can find their dependencies on crates.io.

---

## Quick Reference

### Essential Commands

| Command | Purpose |
|---------|---------|
| `just ci-supported` | PR gate (required) |
| `just test` | Run core tests |
| `just fmt` | Check formatting |
| `just clippy` | Run linter |
| `just snap` | Review snapshots |
| `just build` | Build everything |
| `just check-msrv` | Verify MSRV consistency |

### Key Files

| File | Purpose |
|------|---------|
| [`AGENTS.md`](../../AGENTS.md) | Project overview and agent instructions |
| [`justfile`](../../justfile) | Development recipes |
| [`rust-toolchain.toml`](../../rust-toolchain.toml) | Toolchain pinning |
| [`docs/testing/TESTING_GUIDE.md`](../testing/TESTING_GUIDE.md) | Comprehensive testing documentation |
| [`docs/adr/INDEX.md`](../adr/INDEX.md) | Architecture decisions index |
| [`docs/status/KNOWN_RED.md`](../status/KNOWN_RED.md) | Intentional CI exclusions |

### Getting Help

- Review existing documentation in `docs/`
- Check [ADR index](../adr/INDEX.md) for architectural context
- Consult [`TESTING_GUIDE.md`](../testing/TESTING_GUIDE.md) for testing questions
- Review [`AGENTS.md`](../../AGENTS.md) for project conventions

---

Thank you for contributing to Adze! Your efforts help make parser development more accessible for the Rust community.
