# ADR-0010: Policy-as-Code Architecture

**Status**: Accepted
**Date**: 2025-11-20
**Deciders**: rust-sitter core team
**Context**: [POLICY_AS_CODE_CONTRACT.md](../specs/POLICY_AS_CODE_CONTRACT.md)

---

## Executive Summary

This ADR documents the architectural decisions for implementing **Policy-as-Code v1** in rust-sitter - an automated system for enforcing quality, security, and governance standards throughout the development lifecycle.

**Core Decision**: Implement layered policy enforcement using pre-commit hooks (local), verification scripts (self-service), and CI workflows (automated gates) to eliminate manual quality checks while maintaining development velocity.

**Key Benefits**:
- 🚀 Faster feedback (issues caught locally in <5 seconds)
- 🛡️ Higher quality (zero tolerance for warnings, vulnerabilities)
- 😊 Better developer experience (clear remediation guidance)
- 📉 Reduced review burden (automated enforcement frees reviewers)

---

## Context and Problem Statement

### The Quality Problem

**Current State** (Manual Quality Checks):
```
Developer writes code
    ↓
Commits to branch
    ↓
Pushes to GitHub
    ↓
CI runs (5-10 min)
    ↓
Reviewer finds issues (formatting, linting, security)
    ↓
Developer fixes issues
    ↓
REPEAT 2-3 times per PR
```

**Pain Points**:
- ⏱️ **Slow feedback**: Issues found after push (5-10 min CI + review time)
- 🔄 **Rework cycles**: 2-3 round-trips per PR on average
- 😤 **Frustration**: Trivial issues block substantive review
- 🐛 **Slippage**: Issues occasionally merge to main
- 📈 **Quality debt**: Inconsistent standards accumulate

### Why Manual Reviews Don't Scale

1. **Human Error**: Reviewers miss formatting, linting issues
2. **Inconsistency**: Different reviewers enforce different standards
3. **Time Waste**: 30-40% of review time on mechanical checks
4. **Late Discovery**: Security issues found after merge
5. **Context Switching**: Developer has moved on when feedback arrives

### Requirements for Policy Automation

**Must Have**:
- ✅ Fast local feedback (<5 seconds)
- ✅ Zero false positives (well-tuned thresholds)
- ✅ Clear remediation (actionable error messages)
- ✅ CI enforcement (automated gates)
- ✅ Override mechanism (for valid exceptions)

**Should Have**:
- Performance regression detection (>5% threshold)
- Security vulnerability scanning (CVE detection)
- License compliance checking (approved licenses)
- Test connectivity safeguards (no silent failures)

**Could Have** (deferred to v1.1+):
- Code coverage enforcement
- Advanced static analysis (beyond clippy)
- Automated dependency updates
- Policy configuration (YAML/TOML)

---

## Decision Drivers

### Technical Drivers

1. **Infrastructure Foundation**: Nix environment provides reproducible tooling
2. **CI Maturity**: GitHub Actions workflows already established
3. **Rust Ecosystem Tools**: cargo-audit, cargo-deny, clippy, rustfmt
4. **Test Connectivity**: Existing safeguards against test disconnection

### Strategic Drivers

1. **Quality Bar**: Production-ready GLR parser demands high quality
2. **Contributor Experience**: Lower barriers for new contributors
3. **Velocity**: Eliminate rework cycles, speed up PR merging
4. **Compliance**: Security/license policies for regulated environments

### Constraints

1. **Development Velocity**: Can't significantly slow down commits/pushes
2. **False Positives**: Zero tolerance (breaks trust in automation)
3. **Escape Hatches**: Must support valid exceptions
4. **Tooling Maturity**: Rely on proven tools (no custom analyzers)

---

## Considered Options

### Option 1: Pre-commit Hooks Only (Local-Only)

**Approach**: All policy enforcement via pre-commit hooks.

**Pros**:
- ✅ Fast feedback (local)
- ✅ No CI overhead
- ✅ Developer autonomy

**Cons**:
- ❌ Easily bypassed (`git commit --no-verify`)
- ❌ No enforcement on direct pushes to main
- ❌ Inconsistent setup across contributors
- ❌ Security scans too slow for pre-commit

**Verdict**: **REJECTED** - Insufficient enforcement

---

### Option 2: CI Gates Only (Centralized)

**Approach**: All policy enforcement in CI workflows.

**Pros**:
- ✅ Consistent enforcement
- ✅ Cannot be bypassed
- ✅ Centralized configuration

**Cons**:
- ❌ Slow feedback (5-10 min CI time)
- ❌ High rework cost (push → wait → fix → repeat)
- ❌ Poor developer experience
- ❌ CI resource waste (obvious failures)

**Verdict**: **REJECTED** - Too slow

---

### Option 3: Layered Enforcement (Hybrid) ✅ SELECTED

**Approach**: Multi-layer defense with fast local checks + CI safety net.

**Architecture**:
```
Layer 1: Pre-commit Hooks (Fast, Local)
  - Formatting (cargo fmt)
  - Basic linting (cargo clippy)
  - Test connectivity (no .rs.disabled)
  - Commit message validation
  ↓ (if hook bypassed or failed to install)
Layer 2: Verification Scripts (Self-Service)
  - check-quality.sh (run before push)
  - check-security.sh (run before push)
  - pre-push.sh (comprehensive validation)
  ↓ (if scripts not run)
Layer 3: CI Policy Workflow (Safety Net)
  - Quality gates (formatting, linting, tests, docs)
  - Security scanning (audit, deny, secrets)
  - Performance gates (regression detection)
  - Test connectivity (non-zero counts)
```

**Pros**:
- ✅ Fast feedback (Layer 1: <5s)
- ✅ Self-service validation (Layer 2: <30s)
- ✅ Guaranteed enforcement (Layer 3: always runs)
- ✅ Graceful degradation (layers redundant)
- ✅ Developer-friendly (fix locally first)

**Cons**:
- ⚠️ More complex setup (3 layers to maintain)
- ⚠️ Redundant checks (same checks in multiple layers)

**Mitigation**:
- Nix shell automates hook installation
- Scripts reuse same commands as hooks
- CI workflow identical to local scripts

**Verdict**: **ACCEPTED** - Best balance of speed and enforcement

---

## Decision Outcome

### Chosen Solution: Layered Policy Enforcement

**Core Architecture**:

1. **Layer 1: Pre-commit Hooks** (Fast Local Checks)
   - Framework: `pre-commit` (Python-based, widely adopted)
   - Installation: Automated in Nix shell `shellHook`
   - Hooks: formatting, linting, test connectivity, commit message
   - Execution: <5 seconds typical, blocks commit on failure
   - Bypass: Allowed (`--no-verify`) but caught in Layer 3

2. **Layer 2: Verification Scripts** (Self-Service)
   - Scripts: `check-quality.sh`, `check-security.sh`, `pre-push.sh`
   - Purpose: Comprehensive pre-push validation
   - Execution: <30 seconds typical, optional but recommended
   - Integration: Can be used as git pre-push hook

3. **Layer 3: CI Policy Workflow** (Automated Safety Net)
   - Workflow: `.github/workflows/policy.yml`
   - Jobs: quality gates, security scanning, performance gates, test connectivity
   - Execution: 5-10 minutes, blocks PR merge on failure
   - Enforcement: Cannot be bypassed, required for merge

### Policy Categories

**Quality Policies**:
- **Formatting**: Zero tolerance (`cargo fmt --check`)
- **Linting**: Zero warnings (`cargo clippy -D warnings`)
- **Tests**: 100% pass rate (`cargo test`)
- **Documentation**: Zero doc warnings (`RUSTDOCFLAGS=-D warnings`)
- **Test Connectivity**: No `.rs.disabled` files, non-zero test counts

**Security Policies**:
- **Vulnerabilities**: No known CVEs (`cargo audit`)
- **License Compliance**: Approved licenses only (`cargo deny`)
- **Secret Detection**: No credentials in commits (TruffleHog)
- **SBOM**: Software Bill of Materials generated

**Performance Policies**:
- **Regression Detection**: <5% slowdown threshold (criterion)
- **Benchmark Stability**: Consistent results across runs

### Tool Selection

| Category | Tool | Rationale |
|----------|------|-----------|
| **Pre-commit Framework** | `pre-commit` | Industry standard, easy config, Git integration |
| **Formatting** | `cargo fmt` (rustfmt) | Official Rust formatter, zero config |
| **Linting** | `cargo clippy` | Official Rust linter, catches common bugs |
| **Vulnerability Scanning** | `cargo audit` | RustSec advisory database, Rust-native |
| **License Compliance** | `cargo deny` | Comprehensive, supports multiple policies |
| **Secret Detection** | `TruffleHog` | High accuracy, low false positives |
| **Benchmarking** | `criterion` | Statistical rigor, regression detection |

**Decision Rationale**:
- All tools Rust-native or widely adopted
- Minimal false positives (well-tuned defaults)
- Fast execution (suitable for pre-commit)
- Active maintenance and community support

---

## Detailed Design

### Pre-commit Hook Configuration

**File**: `.pre-commit-config.yaml`

```yaml
repos:
  - repo: local
    hooks:
      # Formatting (fast: ~1s)
      - id: cargo-fmt
        name: Cargo Format Check
        entry: cargo fmt --all -- --check
        language: system
        types: [rust]
        pass_filenames: false

      # Linting (medium: ~3s for changed files)
      - id: cargo-clippy
        name: Cargo Clippy
        entry: cargo clippy --all-targets -- -D warnings
        language: system
        types: [rust]
        pass_filenames: false

      # Test connectivity (fast: <1s)
      - id: test-connectivity
        name: Test Connectivity Check
        entry: ./scripts/check-test-connectivity.sh
        language: system
        pass_filenames: false

      # Commit message (fast: <1s)
      - id: commit-msg
        name: Commit Message Validation
        entry: ./scripts/validate-commit-msg.sh
        language: system
        stages: [commit-msg]

      # Large files (fast: <1s)
      - id: large-files
        name: Prevent Large Files
        entry: ./scripts/check-large-files.sh
        language: system
```

**Performance Budget**:
- Total execution time: <5 seconds (typical)
- Breakdown: fmt (1s) + clippy (3s) + connectivity (<1s)
- Bypass available if needed (`--no-verify`)

**Installation**:
```bash
# Nix shell hook (automatic)
if [ -f .pre-commit-config.yaml ]; then
    if ! pre-commit --version >/dev/null 2>&1; then
        echo "Installing pre-commit..."
        pip install --user pre-commit
    fi
    pre-commit install --install-hooks
    echo "✅ Pre-commit hooks installed"
fi
```

---

### CI Policy Workflow

**File**: `.github/workflows/policy.yml`

```yaml
name: Policy Enforcement

on:
  pull_request:
  push:
    branches: [main, master]

jobs:
  # Quality gates (required for merge)
  quality-gates:
    name: Quality Gates
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Nix
        uses: cachix/install-nix-action@v27
        with:
          nix_path: nixpkgs=channel:nixos-unstable

      - name: Check Formatting
        run: nix develop --command cargo fmt --all -- --check

      - name: Check Clippy (Zero Warnings)
        run: nix develop --command cargo clippy --all-targets -- -D warnings

      - name: Check Tests (100% Pass Rate)
        run: nix develop --command cargo test --workspace -- --test-threads=2

      - name: Check Documentation (Zero Warnings)
        run: nix develop --command cargo doc --no-deps --all-features
        env:
          RUSTDOCFLAGS: -D warnings

  # Security scanning (required for merge)
  security-scanning:
    name: Security Scanning
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Cargo Audit (Vulnerabilities)
        uses: actions-rs/audit-check@v1
        with:
          token: ${{ secrets.GITHUB_TOKEN }}

      - name: Cargo Deny (Licenses & Bans)
        run: |
          cargo install cargo-deny
          cargo deny check

  # Performance gates (informational on main, blocking on PR)
  performance-gates:
    name: Performance Regression Detection
    runs-on: ubuntu-latest
    if: github.event_name == 'pull_request'
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0  # Need base branch for comparison

      - name: Install Nix
        uses: cachix/install-nix-action@v27

      - name: Benchmark PR Branch
        run: |
          nix develop --command cargo bench --workspace -- --save-baseline pr

      - name: Checkout Base Branch
        run: git checkout ${{ github.base_ref }}

      - name: Benchmark Base Branch
        run: |
          nix develop --command cargo bench --workspace -- --save-baseline base

      - name: Compare Performance (5% Threshold)
        run: |
          git checkout ${{ github.head_ref }}
          ./scripts/check-perf-regression.sh base pr 5
          # Exits 1 if any benchmark regresses >5%

  # Test connectivity (hard failure)
  test-connectivity:
    name: Test Connectivity Safeguards
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Check for .rs.disabled files
        run: |
          if find . -name "*.rs.disabled" | grep -q .; then
            echo "❌ Found .rs.disabled files (tests disconnected)"
            find . -name "*.rs.disabled"
            exit 1
          fi

      - name: Install Nix
        uses: cachix/install-nix-action@v27

      - name: Verify Non-Zero Test Counts
        run: nix develop --command ./scripts/check-test-connectivity.sh
```

**Job Dependencies**:
- All jobs run in parallel (no blocking dependencies)
- All jobs required for PR merge (branch protection)
- Failure in any job blocks the entire workflow

**Performance Optimization**:
- Nix caching reduces setup time (2-3 min → <1 min)
- Clippy incremental compilation (only changed files)
- Test parallelism capped (`--test-threads=2`)

---

### Verification Scripts

**1. Quality Check Script** (`scripts/check-quality.sh`):

```bash
#!/usr/bin/env bash
set -euo pipefail

echo "🔍 Checking Quality..."
echo ""

FAILED=0

# Formatting
echo -n "✅ Formatting (cargo fmt): "
if cargo fmt --all -- --check >/dev/null 2>&1; then
    echo "PASS"
else
    echo "FAIL"
    echo "  Run: cargo fmt"
    FAILED=1
fi

# Clippy
echo -n "✅ Clippy (cargo clippy): "
CLIPPY_OUTPUT=$(cargo clippy --all-targets -- -D warnings 2>&1)
if echo "$CLIPPY_OUTPUT" | grep -q "0 warnings"; then
    echo "PASS"
else
    echo "FAIL"
    echo "$CLIPPY_OUTPUT" | grep "warning:" | head -5
    echo "  Run: cargo clippy --fix"
    FAILED=1
fi

# Tests
echo -n "✅ Tests (cargo test): "
if cargo test --workspace -- --test-threads=2 >/tmp/test.log 2>&1; then
    TEST_COUNT=$(grep -oP '\d+(?= passed)' /tmp/test.log | head -1)
    echo "PASS ($TEST_COUNT tests)"
else
    echo "FAIL"
    tail -20 /tmp/test.log
    FAILED=1
fi

# Documentation
echo -n "✅ Documentation (cargo doc): "
if RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --all-features >/dev/null 2>&1; then
    echo "PASS"
else
    echo "FAIL"
    echo "  Fix documentation warnings"
    FAILED=1
fi

# Test connectivity
echo -n "✅ Test Connectivity: "
if ./scripts/check-test-connectivity.sh >/dev/null 2>&1; then
    echo "PASS"
else
    echo "FAIL"
    echo "  Some tests may be disconnected"
    FAILED=1
fi

echo ""
if [ $FAILED -eq 0 ]; then
    echo "🎉 All quality checks passed!"
    exit 0
else
    echo "❌ Quality checks failed."
    exit 1
fi
```

**Performance**: <30 seconds typical (incremental builds)

**2. Security Check Script** (`scripts/check-security.sh`):

```bash
#!/usr/bin/env bash
set -euo pipefail

echo "🔒 Checking Security..."
echo ""

FAILED=0

# Cargo audit
echo -n "🔍 Vulnerability Scan (cargo audit): "
if cargo audit >/dev/null 2>&1; then
    echo "PASS"
else
    echo "FAIL"
    cargo audit
    FAILED=1
fi

# Cargo deny
echo -n "📜 License Compliance (cargo deny): "
if cargo deny check >/dev/null 2>&1; then
    echo "PASS"
else
    echo "FAIL"
    cargo deny check
    FAILED=1
fi

# Secret detection (local)
echo -n "🔐 Secret Detection: "
if git diff --cached | grep -qiE "(api[_-]?key|password|secret|token|private[_-]?key)"; then
    echo "WARNING"
    echo "  Possible secret in staged changes"
    echo "  Review carefully before committing"
else
    echo "PASS"
fi

echo ""
if [ $FAILED -eq 0 ]; then
    echo "🎉 All security checks passed!"
    exit 0
else
    echo "❌ Security checks failed."
    exit 1
fi
```

**Performance**: <10 seconds (audit database cached)

**3. Pre-push Validation** (`scripts/pre-push.sh`):

```bash
#!/usr/bin/env bash
set -euo pipefail

echo "🚀 Pre-Push Validation..."
echo ""

# Run quality checks
if ! ./scripts/check-quality.sh; then
    echo ""
    echo "❌ Quality checks failed. Fix before pushing."
    exit 1
fi

# Run security checks
if ! ./scripts/check-security.sh; then
    echo ""
    echo "❌ Security checks failed. Fix before pushing."
    exit 1
fi

# Check branch name
BRANCH=$(git rev-parse --abbrev-ref HEAD)
if [[ "$BRANCH" == "main" ]] || [[ "$BRANCH" == "master" ]]; then
    echo ""
    echo "⚠️  WARNING: Pushing directly to $BRANCH"
    echo "Consider using a feature branch instead."
    read -p "Continue? (y/N) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
fi

echo ""
echo "✅ Pre-push validation passed!"
```

**Usage**: Can be installed as git pre-push hook (optional)

---

### Security Policy Configuration

**1. Cargo Audit** (`audit.toml`):

```toml
[advisories]
db-path = "~/.cargo/advisory-db"
db-urls = ["https://github.com/rustsec/advisory-db"]

# Deny any vulnerability
vulnerability = "deny"

# Warn on unmaintained crates
unmaintained = "warn"

# Warn on unsound code
unsound = "warn"

# Deny yanked crates
yanked = "deny"

[bans]
# Warn on multiple versions of same crate
multiple-versions = "warn"

# Deny wildcard dependencies
wildcards = "deny"
```

**2. Cargo Deny** (`deny.toml`):

```toml
[advisories]
# Same as cargo-audit (redundant for safety)
vulnerability = "deny"
unmaintained = "warn"
notice = "warn"
ignore = []  # Add CVE IDs to ignore (with justification)

[licenses]
# Deny code without license
unlicensed = "deny"

# Allow only permissive licenses
allow = [
    "MIT",
    "Apache-2.0",
    "BSD-3-Clause",
    "ISC",
    "Zlib",
]

# Deny copyleft licenses
deny = [
    "GPL-2.0",
    "GPL-3.0",
    "AGPL-3.0",
]

copyleft = "deny"

[bans]
# Warn on multiple versions (not deny - sometimes unavoidable)
multiple-versions = "warn"

# Explicit bans (e.g., known problematic crates)
deny = []

[sources]
# Only allow crates.io and known git repos
unknown-registry = "deny"
unknown-git = "deny"

# Allow specific git sources (e.g., forked dependencies)
allow-git = []
```

**Override Process**:
1. Add CVE to `advisories.ignore` with comment explaining justification
2. Add crate to `bans.deny` if problematic
3. Document in `SECURITY.md`

---

### Commit Message Validation

**Script**: `scripts/validate-commit-msg.sh`

```bash
#!/usr/bin/env bash
# Validates conventional commit format

COMMIT_MSG_FILE=$1
COMMIT_MSG=$(cat "$COMMIT_MSG_FILE")

# Conventional Commits pattern
PATTERN="^(feat|fix|docs|style|refactor|perf|test|build|ci|chore|revert)(\(.+\))?: .{1,72}"

if echo "$COMMIT_MSG" | grep -qE "$PATTERN"; then
    exit 0
else
    echo "❌ Commit message does not follow Conventional Commits format"
    echo ""
    echo "Expected format:"
    echo "  <type>(<scope>): <subject>"
    echo ""
    echo "Types: feat, fix, docs, style, refactor, perf, test, build, ci, chore, revert"
    echo ""
    echo "Examples:"
    echo "  feat(parser): add incremental parsing support"
    echo "  fix(glr): resolve shift/reduce conflict in state 0"
    echo "  docs: update README with Nix installation"
    echo ""
    exit 1
fi
```

**Enforcement**: Pre-commit hook (commit-msg stage)

---

### Performance Regression Detection

**Script**: `scripts/check-perf-regression.sh`

```bash
#!/usr/bin/env bash
# Compare two criterion baselines and fail if regression >threshold

BASELINE=$1
CANDIDATE=$2
THRESHOLD=${3:-5}  # Default 5%

echo "📊 Comparing Performance: $CANDIDATE vs $BASELINE (threshold: $THRESHOLD%)"
echo ""

# Criterion stores baselines in target/criterion/<benchmark>/base/
CRITERION_DIR="target/criterion"

REGRESSIONS=0

for BENCH_DIR in "$CRITERION_DIR"/*; do
    BENCH_NAME=$(basename "$BENCH_DIR")

    BASE_FILE="$BENCH_DIR/$BASELINE/estimates.json"
    CAND_FILE="$BENCH_DIR/$CANDIDATE/estimates.json"

    if [ ! -f "$BASE_FILE" ] || [ ! -f "$CAND_FILE" ]; then
        continue
    fi

    # Extract mean time (nanoseconds)
    BASE_MEAN=$(jq '.mean.point_estimate' "$BASE_FILE")
    CAND_MEAN=$(jq '.mean.point_estimate' "$CAND_FILE")

    # Calculate percentage change
    CHANGE=$(echo "scale=2; (($CAND_MEAN - $BASE_MEAN) / $BASE_MEAN) * 100" | bc)

    if (( $(echo "$CHANGE > $THRESHOLD" | bc -l) )); then
        echo "❌ $BENCH_NAME: REGRESSION +${CHANGE}%"
        REGRESSIONS=$((REGRESSIONS + 1))
    elif (( $(echo "$CHANGE < -$THRESHOLD" | bc -l) )); then
        echo "✅ $BENCH_NAME: IMPROVEMENT ${CHANGE}%"
    else
        echo "➡️  $BENCH_NAME: NEUTRAL ${CHANGE}%"
    fi
done

echo ""
if [ $REGRESSIONS -gt 0 ]; then
    echo "❌ Performance regressions detected: $REGRESSIONS benchmarks"
    exit 1
else
    echo "✅ No performance regressions"
    exit 0
fi
```

**Usage**: Called from CI performance gates job

---

## Trade-offs and Consequences

### Positive Consequences

1. **Faster Feedback Loop**:
   - Before: Push → CI (5-10 min) → Review → Rework
   - After: Pre-commit (<5s) → Fix immediately → Push clean code
   - Impact: 80-90% of issues caught before push

2. **Higher Code Quality**:
   - Zero tolerance for warnings (clippy, rustdoc)
   - Zero known vulnerabilities (cargo audit/deny)
   - Consistent formatting (cargo fmt)
   - Impact: Measurable quality improvement

3. **Better Developer Experience**:
   - Clear error messages with remediation steps
   - Self-service validation (scripts)
   - Fast local checks (no waiting for CI)
   - Impact: Reduced frustration, faster iteration

4. **Reduced Review Burden**:
   - Reviewers focus on design/logic, not formatting
   - Automated checks eliminate mechanical review
   - Security/license compliance automated
   - Impact: 30-40% reduction in review time

5. **Compliance Readiness**:
   - SBOM generation (supply chain security)
   - License compliance (legal requirements)
   - Audit trail (policy enforcement logs)
   - Impact: Suitable for regulated environments

### Negative Consequences

1. **Initial Setup Complexity**:
   - Three layers to configure and maintain
   - More dependencies (pre-commit, cargo-audit, cargo-deny)
   - Learning curve for contributors
   - Mitigation: Nix automates setup, comprehensive docs

2. **Potential False Positives**:
   - Clippy warnings sometimes overzealous
   - License scanning may flag acceptable licenses
   - Performance thresholds may need tuning
   - Mitigation: Override process, well-tuned defaults

3. **Performance Overhead**:
   - Pre-commit adds <5s to commit time
   - CI policy workflow adds ~2 min to PR checks
   - Benchmark comparison adds ~5 min
   - Mitigation: Acceptable trade-off for quality

4. **Bypass Risk**:
   - Pre-commit can be skipped (`--no-verify`)
   - Developers may disable hooks
   - Scripts optional (not enforced)
   - Mitigation: CI Layer 3 catches all bypasses

### Risks and Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| **False positives block work** | Medium | High | Override process, tuned thresholds |
| **Slow pre-commit (<5s goal)** | Low | Medium | Optimize hooks, allow bypass for WIP |
| **Security tool false negatives** | Low | High | Multiple tools (audit + deny), manual review |
| **Performance regression false alarms** | Medium | Low | Statistical rigor (criterion), 5% threshold |
| **Hook installation failures** | Low | Medium | Nix shell automates, fallback to manual |
| **Developer resistance** | Medium | Medium | Clear communication, training, gradual rollout |

---

## Alternatives Considered (Detailed)

### Alternative 1: GitHub Apps (Renovate, Dependabot)

**Approach**: Use third-party GitHub Apps for policy enforcement.

**Tools**:
- Renovate: Automated dependency updates
- Dependabot: Security alerts, version updates
- CodeFactor: Code quality analysis
- SonarCloud: Advanced static analysis

**Pros**:
- ✅ Managed service (no maintenance)
- ✅ Rich dashboards and reporting
- ✅ Automatic PR creation for updates

**Cons**:
- ❌ External dependency (vendor lock-in)
- ❌ Limited customization
- ❌ May require paid plans for private repos
- ❌ Data leaves GitHub (privacy concerns)
- ❌ No local enforcement (CI-only)

**Decision**: **DEFERRED** - Use for dependency updates (v1.1+), not core policy enforcement

---

### Alternative 2: Custom Static Analysis (Linters)

**Approach**: Build custom Rust linters using syn/quote.

**Example**: Custom lint for "no unwrap() in production code"

**Pros**:
- ✅ Fully customizable rules
- ✅ Project-specific checks
- ✅ Better error messages (context-aware)

**Cons**:
- ❌ Significant development effort
- ❌ Maintenance burden (keep up with Rust changes)
- ❌ Reinventing the wheel (clippy already excellent)
- ❌ Risk of bugs in linter itself

**Decision**: **REJECTED** - Clippy sufficient for v1, defer custom lints to v1.1+

---

### Alternative 3: Git Server-Side Hooks

**Approach**: Enforce policies via server-side hooks (GitHub Enterprise, GitLab).

**Pros**:
- ✅ Cannot be bypassed (enforced on server)
- ✅ Consistent enforcement
- ✅ No client-side setup

**Cons**:
- ❌ Requires GitHub Enterprise (not available on github.com)
- ❌ Slow feedback (server-side processing)
- ❌ No local validation (developer experience worse)
- ❌ Limited error reporting (pre-receive hook constraints)

**Decision**: **REJECTED** - GitHub Enterprise not available, CI workflows sufficient

---

### Alternative 4: Trunk-Based Development (No Branches)

**Approach**: All commits to main, CI gates only.

**Pros**:
- ✅ Simplified workflow (no PRs, branches)
- ✅ Continuous integration (literally)

**Cons**:
- ❌ Main branch unstable (broken commits)
- ❌ No review process (quality risk)
- ❌ Rollback complexity (revert commits)
- ❌ Incompatible with policy enforcement goals

**Decision**: **REJECTED** - PR workflow with policy gates superior for quality

---

## Evolution Strategy

### Phase 1: Policy v1.0 (Week 2)

**Scope**: Core policy infrastructure
- Pre-commit hooks (formatting, linting, test connectivity)
- Verification scripts (quality, security, pre-push)
- CI policy workflow (quality gates, security scanning)
- Basic documentation (POLICIES.md, ADR-0010)

**Success Criteria**: All ACs met, policies enforced on all PRs

---

### Phase 2: Policy v1.1 (Q1 2026, post-incremental parsing)

**Enhancements**:
- **Code Coverage**: Enforce minimum coverage % (e.g., 80%)
- **Advanced Static Analysis**: Beyond clippy (e.g., MIRI for unsafe code)
- **Dependency Updates**: Automated PRs (Dependabot/Renovate)
- **Policy Configuration**: YAML/TOML for tuning thresholds
- **Performance Tracking**: Historical benchmark trends

**Rationale**: Defer complexity until core policies proven

---

### Phase 3: Policy v2.0 (Future)

**Advanced Features**:
- **Custom Linters**: Project-specific rules
- **Fuzz Testing**: Continuous fuzzing (OSS-Fuzz)
- **SAST/DAST**: Advanced security scanning
- **Supply Chain Security**: Cosign signing, SLSA attestation
- **Policy as Code (Rego)**: Open Policy Agent integration

**Rationale**: Only if proven need (avoid over-engineering)

---

## Monitoring and Observability

### Metrics to Track

**Pre-commit Hook Metrics**:
- Hook installation success rate (target: >95%)
- Average execution time (target: <5s)
- Bypass rate (commits with `--no-verify`, target: <10%)
- Failure rate by hook type (identify problematic hooks)

**CI Policy Metrics**:
- Policy workflow pass rate (target: 100% on main)
- Average execution time (target: <10 min)
- Failure distribution (which policies fail most)
- False positive rate (target: <5%)

**Security Metrics**:
- Known vulnerabilities detected (trend: decreasing)
- License violations detected (trend: 0)
- Secrets detected (trend: 0)
- Time to remediation (target: <24h)

**Developer Experience Metrics**:
- Rework cycles per PR (target: <1.5, down from ~2.5)
- Time to merge (target: <24h)
- Developer satisfaction (survey: >4/5)

### Monitoring Implementation

**GitHub Actions Metrics**:
```yaml
# In .github/workflows/policy.yml
- name: Report Metrics
  if: always()
  run: |
    echo "::notice::Policy execution time: ${{ steps.policy.outputs.duration }}"
    echo "::notice::Policies checked: ${{ steps.policy.outputs.policy_count }}"
    echo "::notice::Failures: ${{ steps.policy.outputs.failure_count }}"
```

**Pre-commit Metrics** (collected locally, aggregated in CI):
```bash
# In .pre-commit-config.yaml hooks
- id: metrics-collector
  entry: ./scripts/collect-pre-commit-metrics.sh
  language: system
  always_run: true
  pass_filenames: false
```

---

## Documentation Requirements

### Documentation Deliverables

1. **POLICIES.md**: High-level policy overview (what, why, how)
2. **docs/guides/POLICY_ENFORCEMENT.md**: Detailed implementation guide
3. **docs/adr/ADR-0010-POLICY-AS-CODE.md**: This architecture decision record
4. **docs/guides/LOCAL_QUALITY_CHECKS.md**: Using verification scripts
5. **CONTRIBUTING.md**: Updated with policy section
6. **SECURITY.md**: Security policy and disclosure process
7. **.github/ISSUE_TEMPLATE/policy-override.md**: Exception request template

### Documentation Standards

- **Clarity**: Clear explanation of each policy
- **Actionability**: Step-by-step remediation guidance
- **Examples**: Real-world examples of violations and fixes
- **Searchability**: Keywords for common error messages

---

## Success Criteria

Policy-as-Code v1 is **successful** if:

1. **Enforcement**:
   - ✅ 100% of PRs pass policy checks before merge
   - ✅ Zero policy violations on main branch
   - ✅ No bypassed security vulnerabilities

2. **Performance**:
   - ✅ Pre-commit execution <5 seconds (typical)
   - ✅ CI policy workflow <10 minutes (typical)
   - ✅ False positive rate <5%

3. **Developer Experience**:
   - ✅ Developer satisfaction >4/5 (survey)
   - ✅ Rework cycles per PR <1.5 (down from ~2.5)
   - ✅ Time to merge <24h (policy not bottleneck)

4. **Quality Impact**:
   - ✅ Zero clippy warnings on main
   - ✅ Zero known CVEs in dependencies
   - ✅ 100% test pass rate
   - ✅ No disconnected tests

5. **Adoption**:
   - ✅ >90% contributors use pre-commit hooks
   - ✅ >80% contributors use verification scripts
   - ✅ Zero manual quality reviews required

---

## Conclusion

**Decision**: Implement **Layered Policy Enforcement** (pre-commit + scripts + CI) for Policy-as-Code v1.

**Rationale**:
- ✅ Fast local feedback (<5s) via pre-commit hooks
- ✅ Self-service validation via verification scripts
- ✅ Guaranteed enforcement via CI gates
- ✅ Balances developer experience and quality standards
- ✅ Leverages existing Nix/CI infrastructure
- ✅ Uses proven, mature tools (cargo-audit, cargo-deny, clippy)

**Expected Impact**:
- 80-90% of issues caught locally (before push)
- 30-40% reduction in review time (eliminate mechanical review)
- Zero tolerance for warnings, vulnerabilities, test failures
- Production-ready quality standards for rust-sitter

**Next Steps**:
1. Implement pre-commit hooks (Day 1)
2. Implement verification scripts (Day 2)
3. Implement CI policy workflow (Day 3)
4. Add security scanning (Day 4)
5. Add performance gates (Day 5)
6. Write documentation (Day 5)
7. Test with intentional violations (Day 5)
8. Rollout to team (Week 3)

---

**Status**: Accepted
**Last Updated**: 2025-11-20
**Next Review**: After Phase 1B implementation (Week 3)
**Owner**: rust-sitter core team

---

END OF ADR
