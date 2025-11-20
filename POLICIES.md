# Policy Enforcement

This document describes rust-sitter's automated quality, security, and performance policies.

## Table of Contents

- [Overview](#overview)
- [Policy Categories](#policy-categories)
- [Enforcement Layers](#enforcement-layers)
- [Working with Policies](#working-with-policies)
- [Policy Reference](#policy-reference)
- [Troubleshooting](#troubleshooting)
- [Policy Overrides](#policy-overrides)

## Overview

**rust-sitter uses automated policy enforcement** to maintain high quality, security, and performance standards. Policies are enforced at three layers:

1. **Pre-commit Hooks** (Layer 1): Fast local checks (<5s) before commit
2. **Verification Scripts** (Layer 2): Self-service validation (<60s) before push
3. **CI Workflows** (Layer 3): Required checks that cannot be bypassed

This layered approach provides:
- **Fast feedback**: Catch issues locally before pushing
- **Self-service**: Developers can validate changes independently
- **Safety net**: CI catches anything that slips through

> **Design**: See [ADR-0010: Policy-as-Code](docs/adr/ADR-0010-POLICY-AS-CODE.md) for architectural decisions and rationale.

## Policy Categories

### 1. Quality Policies

**Goal**: Zero warnings, 100% test pass rate

| Policy | Tool | Enforcement | Rationale |
|--------|------|-------------|-----------|
| Code formatting | `cargo fmt` | All layers | Consistent style, no bikeshedding |
| Zero clippy warnings | `cargo clippy` | All layers | Catch bugs early, maintain quality |
| 100% test pass rate | `cargo test` | All layers | Prevent regressions |
| Documentation | `cargo doc` | CI only | API docs stay up-to-date |
| Test connectivity | Custom check | All layers | Tests remain connected (no `.rs.disabled`) |

**Performance Targets**:
- Pre-commit: <5 seconds (fmt + clippy on changed files)
- Verification script: <30 seconds (full workspace)
- CI: <30 minutes (full suite including docs)

### 2. Security Policies

**Goal**: Zero vulnerabilities, compliant licenses

| Policy | Tool | Enforcement | Rationale |
|--------|------|-------------|-----------|
| Vulnerability scanning | `cargo audit` | CI + local | No known CVEs in dependencies |
| License compliance | `cargo deny` | CI + local | Only approved licenses (MIT, Apache-2.0, BSD, ISC) |
| Secret detection | Multiple tools | CI + pre-commit | Prevent credential leaks |
| Dependency health | `cargo audit` | CI + local | No unmaintained/yanked crates |

**Secret Detection Methods**:
1. **Pattern matching**: API keys, tokens, passwords, AWS/Stripe/GitHub credentials
2. **Entropy analysis**: High entropy strings (potential encoded secrets)
3. **File path analysis**: Sensitive file patterns (`.pem`, `.key`, credentials)
4. **Git history scanning**: TruffleHog for historical secrets

**Performance Targets**:
- Pre-commit: <1 second (pattern scan on staged changes)
- Verification script: <10 seconds (cargo audit + deny)
- CI: <15 minutes (full secret detection suite)

### 3. Performance Policies

**Goal**: No regressions >5% without justification

| Policy | Tool | Enforcement | Rationale |
|--------|------|-------------|-----------|
| Benchmark comparison | `cargo bench` | CI (PR only) | Catch performance regressions |
| Regression threshold | Custom script | CI (PR only) | 5% slowdown requires investigation |

**Performance Targets**:
- CI: <45 minutes (benchmark PR + base, compare)
- Only runs on pull requests (not on push to main)

### 4. Architectural Policies

**Goal**: Maintain design integrity

| Policy | Tool | Enforcement | Rationale |
|--------|------|-------------|-----------|
| MSRV compliance | `rust-toolchain.toml` | CI | Rust 1.89.0+ (Rust 2024 Edition) |
| Platform support | `cargo deny` | CI | Linux, macOS, Windows, WASM |
| Feature flags | `cargo test` | CI | All feature combinations work |

## Enforcement Layers

### Layer 1: Pre-commit Hooks

**Purpose**: Fast local validation before commit

**Setup**: Automatic via Nix development shell
```bash
nix develop  # Pre-commit hooks auto-installed
```

**Manual installation** (if not using Nix):
```bash
pip install pre-commit
pre-commit install
pre-commit install --hook-type commit-msg
```

**What runs**:
- Code formatting check (`cargo fmt --check`)
- Clippy linting (`cargo clippy -- -D warnings`)
- Test connectivity check (no `.rs.disabled` files)
- Large file detection (>1MB warning)
- Commit message validation (Conventional Commits)

**Bypassing** (for work-in-progress):
```bash
git commit --no-verify  # Skip pre-commit hooks
```

> ⚠️ **Warning**: Bypassing pre-commit hooks means your commit will fail CI. Only use for WIP commits on feature branches.

### Layer 2: Verification Scripts

**Purpose**: Self-service validation before push

**Usage**:
```bash
# Check quality (formatting, clippy, tests, docs)
./scripts/check-quality.sh

# Check security (vulnerabilities, licenses, secrets)
./scripts/check-security.sh

# Check everything (quality + security)
./scripts/pre-push.sh
```

**Color-coded output**:
- 🟢 **GREEN PASS**: Check succeeded
- 🔴 **RED FAIL**: Check failed (fix required)
- 🟡 **YELLOW SKIP**: Check skipped (tool not installed)
- 🟡 **YELLOW WARNING**: Potential issue (review recommended)

**Install optional tools** (for complete scanning):
```bash
cargo install cargo-audit cargo-deny
```

### Layer 3: CI Workflows

**Purpose**: Safety net that cannot be bypassed

**Workflows**:

1. **Policy Enforcement** (`.github/workflows/policy.yml`)
   - Quality gates (fmt, clippy, tests, docs)
   - Security scanning (audit, deny)
   - Test connectivity (no `.rs.disabled`, non-zero test counts)
   - Performance gates (benchmark comparison, PR only)
   - **Status**: Required for PR merge

2. **Secret Detection** (`.github/workflows/secrets.yml`)
   - TruffleHog (git history)
   - Pattern scanning (API keys, tokens)
   - Entropy analysis (high entropy strings)
   - File analysis (sensitive paths)
   - **Status**: Required for PR merge

**CI Performance**:
- Policy workflow: ~30 minutes (parallel jobs)
- Secret detection: ~10 minutes (parallel jobs)
- Total: ~40 minutes for full policy enforcement

**Branch Protection**:
- `main` and `master` branches require:
  - ✅ Policy Enforcement workflow passing
  - ✅ Secret Detection workflow passing
  - ✅ At least 1 approving review

## Working with Policies

### Development Workflow

1. **Start development**: Enter Nix shell
   ```bash
   nix develop
   # Pre-commit hooks auto-installed
   ```

2. **Make changes**: Write code, tests, docs

3. **Local validation**: Pre-commit hooks run automatically
   ```bash
   git add .
   git commit -m "feat: Add new feature"
   # Hooks run: fmt, clippy, test connectivity
   ```

4. **Pre-push validation**: Run verification scripts
   ```bash
   ./scripts/pre-push.sh
   # Runs: check-quality.sh + check-security.sh
   ```

5. **Push changes**: CI validates everything
   ```bash
   git push origin feature-branch
   # CI runs: policy.yml + secrets.yml
   ```

6. **Create PR**: Branch protection requires CI to pass

### Common Scenarios

#### Scenario 1: Formatting Failure

**Pre-commit output**:
```
❌ Cargo Format Check failed
  Run: cargo fmt
```

**Fix**:
```bash
cargo fmt
git add .
git commit
```

#### Scenario 2: Clippy Warnings

**Pre-commit output**:
```
❌ Cargo Clippy failed
warning: unused variable: `x`
  Run: cargo clippy --fix
```

**Fix**:
```bash
cargo clippy --fix --allow-dirty
git add .
git commit
```

#### Scenario 3: Test Failure

**Verification script output**:
```
❌ Tests (cargo test): FAIL
  Last 20 lines of output:
  ...
```

**Fix**:
```bash
# Run tests locally to debug
cargo test

# Fix the failing test
# Re-run verification
./scripts/check-quality.sh
```

#### Scenario 4: Vulnerability Detected

**CI output**:
```
❌ Cargo Audit (Vulnerabilities)
Crate:     tokio
Version:   1.28.0
Warning:   memory safety issue
Advisory:  RUSTSEC-2023-0001
```

**Fix**:
```bash
# Update the vulnerable dependency
cargo update -p tokio

# Verify fix
cargo audit

# Commit and push
git add Cargo.lock
git commit -m "chore: Update tokio to fix RUSTSEC-2023-0001"
git push
```

#### Scenario 5: Secret Detected

**CI output**:
```
❌ Pattern Scan failed
Found potential secret: api_key = "sk_live_..."
```

**Fix**:
```bash
# Remove the secret from the code
# If already committed, remove from git history:
git filter-branch --force --index-filter \
  "git rm --cached --ignore-unmatch path/to/file" \
  --prune-empty --tag-name-filter cat -- --all

# Rotate the credential (if real secret)
# Add to .gitignore to prevent future commits
echo "config/secrets.yml" >> .gitignore

# Commit and push
git add .gitignore
git commit -m "fix: Remove secret and update .gitignore"
git push --force
```

#### Scenario 6: Work-in-Progress (WIP) Commits

**Scenario**: Need to commit incomplete work

**Bypass pre-commit** (feature branch only):
```bash
git commit --no-verify -m "wip: Incomplete feature"
```

**Before pushing** (CI will fail otherwise):
```bash
# Complete the work
# Run full validation
./scripts/pre-push.sh

# Amend the commit or add fix commits
git commit --amend
git push
```

> ⚠️ **Warning**: Never bypass policies on `main` or `master` branches.

### Performance Regression Workflow

**When**: PR introduces performance-sensitive changes

**Process**:
1. CI automatically benchmarks PR branch vs base branch
2. Script compares results with 5% threshold
3. If regression >5%:
   - **Option A**: Fix the performance issue
   - **Option B**: Request policy override (see below) with justification

**Manual benchmarking**:
```bash
# Benchmark current changes
cargo bench --workspace -- --save-baseline pr

# Checkout base branch
git checkout main
cargo bench --workspace -- --save-baseline base

# Compare (requires script in Phase III)
./scripts/check-perf-regression.sh base pr 5
```

## Policy Reference

### Configuration Files

| File | Purpose | Location |
|------|---------|----------|
| `.pre-commit-config.yaml` | Pre-commit hook configuration | Root |
| `audit.toml` | cargo-audit configuration | Root |
| `deny.toml` | cargo-deny configuration | Root |
| `.github/workflows/policy.yml` | CI policy enforcement | `.github/workflows/` |
| `.github/workflows/secrets.yml` | Secret detection workflow | `.github/workflows/` |

### Verification Scripts

| Script | Purpose | Performance Target |
|--------|---------|-------------------|
| `scripts/check-quality.sh` | Quality validation | <30s |
| `scripts/check-security.sh` | Security scanning | <10s |
| `scripts/pre-push.sh` | Combined validation | <60s |

### Environment Variables

| Variable | Default | Purpose |
|----------|---------|---------|
| `RUST_BACKTRACE` | `1` | Enable backtraces for debugging |
| `RUST_TEST_THREADS` | `2` | Cap test concurrency (stability) |
| `RAYON_NUM_THREADS` | `4` | Rayon thread pool limit |
| `TOKIO_WORKER_THREADS` | `2` | Tokio async runtime limit |
| `RUSTDOCFLAGS` | `-D warnings` | Fail on documentation warnings |

### Exit Codes

| Exit Code | Meaning | Action |
|-----------|---------|--------|
| `0` | All checks passed | Proceed |
| `1` | Policy violation | Fix and retry |
| `2` | User cancelled | Cancelled by user input |

## Troubleshooting

### Pre-commit Hooks Not Running

**Symptom**: Commits succeed without running hooks

**Diagnosis**:
```bash
ls -la .git/hooks/pre-commit
# Should exist and be executable
```

**Fix**:
```bash
pre-commit install
pre-commit install --hook-type commit-msg
```

**Alternative** (Nix):
```bash
nix develop  # Re-enter shell to reinstall hooks
```

### Verification Script Not Found

**Symptom**: `./scripts/check-quality.sh: command not found`

**Diagnosis**:
```bash
ls -la scripts/check-quality.sh
# Should exist and be executable
```

**Fix**:
```bash
chmod +x scripts/check-quality.sh
chmod +x scripts/check-security.sh
chmod +x scripts/pre-push.sh
```

### Cargo Tools Not Installed

**Symptom**: `SKIP (cargo-audit not installed)`

**Fix**:
```bash
# Install required tools
cargo install cargo-audit cargo-deny

# Re-run verification
./scripts/check-security.sh
```

**Alternative** (Nix):
```bash
nix develop  # Tools auto-installed in Nix shell
```

### CI Failing but Local Checks Pass

**Possible causes**:
1. **Stale branch**: Rebase on latest main
   ```bash
   git fetch origin
   git rebase origin/main
   git push --force-with-lease
   ```

2. **Environment difference**: Use Nix shell
   ```bash
   nix develop --command ./scripts/pre-push.sh
   ```

3. **Concurrency issues**: Tests may be flaky
   ```bash
   # Run with capped concurrency
   RUST_TEST_THREADS=1 cargo test
   ```

4. **Dependency cache**: Clear cache and rebuild
   ```bash
   cargo clean
   cargo build
   ./scripts/check-quality.sh
   ```

### Secret Detection False Positive

**Symptom**: CI flags a string that isn't a real secret

**Fix**:
1. **Add to exception list** (if safe):
   - For pattern scan: Add to `.secretsignore` (future feature)
   - For TruffleHog: Add to `.trufflehog-ignore.yml` (future feature)

2. **Request policy override** (see below)

## Policy Overrides

**When**: Legitimate need to bypass a policy

**Process**:
1. Open issue using `.github/ISSUE_TEMPLATE/policy-override.md`
2. Provide:
   - Policy being overridden
   - Justification with technical rationale
   - Alternative mitigations
   - Expected duration (temporary) or permanence
3. Get approval from maintainer
4. Implement override:
   - For `cargo-audit`: Add to `audit.toml` ignore list with comment
   - For `cargo-deny`: Add to `deny.toml` exceptions with comment
   - For secrets: Add to ignore file with comment
   - For clippy: Use `#[allow(clippy::lint_name)]` with comment
   - For performance: Document in PR description

**Example** (cargo-audit override):
```toml
# audit.toml
[advisories]
ignore = [
    "RUSTSEC-2023-0001",  # tokio: memory safety issue
    # Override approved: Issue #123
    # Justification: Not exploitable in our use case (no untrusted input)
    # Mitigation: Input validation at API boundary
    # Expected fix: tokio 1.29.0 (ETA: 2025-12-01)
]
```

**Override Review**:
- Temporary overrides: Reviewed quarterly
- Permanent overrides: Reviewed annually
- All overrides tracked in policy override issues

## Related Documentation

- [ADR-0010: Policy-as-Code Architecture](docs/adr/ADR-0010-POLICY-AS-CODE.md) - Design decisions and rationale
- [Policy Enforcement Implementation Guide](docs/guides/POLICY_ENFORCEMENT.md) - Detailed technical implementation
- [Contributing Guidelines](CONTRIBUTING.md) - Contribution workflow with policies
- [BDD Planning Document](docs/plans/BDD_POLICY_ENFORCEMENT.md) - BDD scenarios and acceptance criteria
- [Phase 1B Contract](docs/specs/POLICY_AS_CODE_CONTRACT.md) - Implementation contract and specifications

## Feedback and Improvements

**Questions or issues?**
- Open an issue: https://github.com/EffortlessMetrics/rust-sitter/issues
- Tag: `policy-as-code`, `ci`, `security`

**Suggestions for improvement?**
- Policy too strict? Open a discussion with use case and alternative
- Policy too lenient? Open an issue with security/quality concern
- New policy needed? Propose via RFC process

---

**Last Updated**: 2025-11-20
**Version**: 1.0.0 (Phase 1B complete)
**Maintained by**: rust-sitter maintainers
