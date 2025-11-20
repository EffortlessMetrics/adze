# Policy Enforcement Implementation Guide

**Audience**: Contributors, maintainers, CI administrators

This guide provides detailed technical information about rust-sitter's policy enforcement infrastructure.

## Table of Contents

- [Architecture](#architecture)
- [Layer 1: Pre-commit Hooks](#layer-1-pre-commit-hooks)
- [Layer 2: Verification Scripts](#layer-2-verification-scripts)
- [Layer 3: CI Workflows](#layer-3-ci-workflows)
- [Configuration Files](#configuration-files)
- [Testing Policy Enforcement](#testing-policy-enforcement)
- [Maintenance](#maintenance)
- [Troubleshooting](#troubleshooting)

## Architecture

### Layered Enforcement Model

```
┌─────────────────────────────────────────────────────────────┐
│                    Developer Workflow                        │
├─────────────────────────────────────────────────────────────┤
│                                                               │
│  ┌──────────────────────────────────────────────────────┐  │
│  │ Layer 1: Pre-commit Hooks                            │  │
│  │ - Fast local checks (<5s)                            │  │
│  │ - Immediate feedback                                  │  │
│  │ - Can be bypassed (--no-verify)                       │  │
│  └──────────────────────────────────────────────────────┘  │
│                           │                                   │
│                           ▼                                   │
│  ┌──────────────────────────────────────────────────────┐  │
│  │ Layer 2: Verification Scripts                         │  │
│  │ - Self-service validation (<60s)                      │  │
│  │ - Pre-push recommended                                │  │
│  │ - Developer-initiated                                 │  │
│  └──────────────────────────────────────────────────────┘  │
│                           │                                   │
│                           ▼                                   │
│  ┌──────────────────────────────────────────────────────┐  │
│  │ Layer 3: CI Workflows                                 │  │
│  │ - Safety net (cannot bypass)                          │  │
│  │ - Required for PR merge                               │  │
│  │ - Branch protection enforced                          │  │
│  └──────────────────────────────────────────────────────┘  │
│                                                               │
└─────────────────────────────────────────────────────────────┘
```

### Design Principles

1. **Fail Fast**: Catch issues at the earliest possible stage
2. **Progressive Validation**: Each layer provides more comprehensive checks
3. **Developer Experience**: Fast feedback loops, clear error messages
4. **Zero Trust**: CI validates everything, even if local checks passed
5. **Performance**: Parallel execution, caching, incremental checks

### Technology Stack

| Component | Technology | Purpose |
|-----------|------------|---------|
| Pre-commit framework | Python `pre-commit` | Git hook management |
| Verification scripts | Bash | Self-service validation |
| CI/CD | GitHub Actions | Automated validation |
| Build system | Cargo + Nix | Reproducible builds |
| Quality tools | rustfmt, clippy | Code quality |
| Security tools | cargo-audit, cargo-deny | Vulnerability scanning |
| Secret detection | TruffleHog, pattern matching | Credential leak prevention |

## Layer 1: Pre-commit Hooks

### Overview

Pre-commit hooks run automatically before each commit, providing immediate feedback on basic quality issues.

### Configuration

**File**: `.pre-commit-config.yaml`

```yaml
repos:
  - repo: local
    hooks:
      # Fast checks (formatting, linting)
      - id: cargo-fmt
        name: Cargo Format Check
        entry: cargo fmt --all -- --check
        language: system
        types: [rust]
        pass_filenames: false

      - id: cargo-clippy
        name: Cargo Clippy
        entry: cargo clippy --workspace --all-targets -- -D warnings
        language: system
        types: [rust]
        pass_filenames: false

      # Safeguards (connectivity, large files)
      - id: test-connectivity
        name: Test Connectivity Check
        entry: bash -c 'if find . -name "*.rs.disabled" | grep -q .; then echo "❌ Found .rs.disabled files"; exit 1; fi'
        language: system
        pass_filenames: false

      - id: large-files
        name: Prevent Large Files
        entry: bash -c 'for file in $(git diff --cached --name-only); do if [ -f "$file" ]; then size=$(stat -f%z "$file" 2>/dev/null || stat -c%s "$file" 2>/dev/null); if [ "$size" -gt 1048576 ]; then echo "⚠️  Large file: $file"; exit 1; fi; fi; done'
        language: system
        pass_filenames: false

  # Commit message validation
  - repo: local
    hooks:
      - id: commit-msg-validation
        name: Commit Message Validation
        entry: bash -c 'if ! grep -qE "^(feat|fix|docs|style|refactor|perf|test|build|ci|chore|revert)(\(.+\))?: .{1,72}" "$1"; then echo "❌ Invalid commit message"; exit 1; fi'
        language: system
        stages: [commit-msg]
```

### Installation

**Automatic** (via Nix development shell):
```bash
nix develop
# Pre-commit hooks auto-installed via shellHook
```

**Manual**:
```bash
# Install pre-commit framework
pip install pre-commit

# Install hooks from config
pre-commit install
pre-commit install --hook-type commit-msg
```

### Hook Execution

**Normal commit** (hooks run automatically):
```bash
git add .
git commit -m "feat: Add new feature"

# Output:
# Cargo Format Check.....Passed
# Cargo Clippy............Passed
# Test Connectivity.......Passed
# Prevent Large Files.....Passed
# Commit Message..........Passed
```

**Bypass** (for WIP commits):
```bash
git commit --no-verify -m "wip: Incomplete work"
# Hooks skipped, commit succeeds
```

### Performance

| Hook | Typical Time | Worst Case |
|------|--------------|------------|
| cargo-fmt | 0.5s | 2s |
| cargo-clippy | 2s | 10s (cold cache) |
| test-connectivity | 0.1s | 0.5s |
| large-files | 0.1s | 0.5s |
| commit-msg | <0.1s | <0.1s |
| **Total** | **~3s** | **~13s** |

### Troubleshooting

**Problem**: Hooks not running

```bash
# Check installation
ls -la .git/hooks/pre-commit
# Should be a symlink or executable

# Reinstall
pre-commit install
```

**Problem**: Hooks fail due to tool not found

```bash
# Enter Nix shell (tools auto-available)
nix develop

# Or install tools manually
cargo install rustfmt clippy
```

## Layer 2: Verification Scripts

### Overview

Verification scripts provide comprehensive local validation before pushing changes.

### Scripts

#### 1. `scripts/check-quality.sh`

**Purpose**: Validate code quality (formatting, linting, tests, docs)

**Implementation**:
```bash
#!/usr/bin/env bash
set -euo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

FAILED=0

echo -e "${BLUE}🔍 Checking Quality...${NC}"

# 1. Formatting
echo -n "✅ Formatting: "
if cargo fmt --all -- --check >/dev/null 2>&1; then
    echo -e "${GREEN}PASS${NC}"
else
    echo -e "${RED}FAIL${NC}"
    echo -e "${YELLOW}  Run: cargo fmt${NC}"
    FAILED=1
fi

# 2. Clippy (zero warnings)
echo -n "✅ Clippy: "
CLIPPY_OUTPUT=$(cargo clippy --workspace --all-targets -- -D warnings 2>&1)
if echo "$CLIPPY_OUTPUT" | grep -q "0 warnings emitted"; then
    echo -e "${GREEN}PASS${NC}"
else
    echo -e "${RED}FAIL${NC}"
    echo "$CLIPPY_OUTPUT" | grep "warning:" | head -5
    echo -e "${YELLOW}  Run: cargo clippy --fix${NC}"
    FAILED=1
fi

# 3. Tests (100% pass rate)
echo -n "✅ Tests: "
if cargo test --workspace -- --test-threads=2 >/tmp/test.log 2>&1; then
    TEST_COUNT=$(grep -oP '\d+(?= passed)' /tmp/test.log | head -1 || echo "?")
    echo -e "${GREEN}PASS ($TEST_COUNT tests)${NC}"
else
    echo -e "${RED}FAIL${NC}"
    tail -20 /tmp/test.log
    FAILED=1
fi

# 4. Documentation (zero warnings)
echo -n "✅ Documentation: "
if RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --all-features >/dev/null 2>&1; then
    echo -e "${GREEN}PASS${NC}"
else
    echo -e "${RED}FAIL${NC}"
    FAILED=1
fi

# 5. Test connectivity
echo -n "✅ Test Connectivity: "
if find . -name "*.rs.disabled" | grep -q .; then
    echo -e "${RED}FAIL${NC}"
    find . -name "*.rs.disabled"
    FAILED=1
else
    echo -e "${GREEN}PASS${NC}"
fi

# Summary
echo ""
if [ $FAILED -eq 0 ]; then
    echo -e "${GREEN}🎉 All quality checks passed!${NC}"
    exit 0
else
    echo -e "${RED}❌ Quality checks failed.${NC}"
    exit 1
fi
```

**Performance**: ~30 seconds (full workspace)

#### 2. `scripts/check-security.sh`

**Purpose**: Validate security (vulnerabilities, licenses, secrets)

**Implementation**: Similar structure to check-quality.sh, runs:
- `cargo audit` (vulnerabilities)
- `cargo deny check` (licenses, advisories)
- Pattern matching for secrets in staged changes
- Dependency health check

**Performance**: ~10 seconds

#### 3. `scripts/pre-push.sh`

**Purpose**: Combined validation (quality + security)

**Implementation**:
```bash
#!/usr/bin/env bash
set -euo pipefail

echo -e "${BLUE}🚀 Pre-Push Validation...${NC}"

# Warn if pushing to main/master
BRANCH=$(git rev-parse --abbrev-ref HEAD 2>/dev/null || echo "unknown")
if [[ "$BRANCH" == "main" ]] || [[ "$BRANCH" == "master" ]]; then
    echo -e "${YELLOW}⚠️  WARNING: Pushing directly to $BRANCH${NC}"
    read -p "Continue? (y/N) " -n 1 -r
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 2
    fi
fi

# Run quality checks
echo -e "${BLUE}Step 1/2: Quality Checks${NC}"
if ./scripts/check-quality.sh; then
    echo -e "${GREEN}✅ Quality passed${NC}"
else
    echo -e "${RED}❌ Quality failed${NC}"
    exit 1
fi

# Run security checks
echo -e "${BLUE}Step 2/2: Security Checks${NC}"
if ./scripts/check-security.sh; then
    echo -e "${GREEN}✅ Security passed${NC}"
else
    echo -e "${RED}❌ Security failed${NC}"
    exit 1
fi

echo -e "${GREEN}✅ Pre-push validation passed!${NC}"
```

**Performance**: ~40 seconds (combined)

### Usage

```bash
# Quick quality check
./scripts/check-quality.sh

# Quick security check
./scripts/check-security.sh

# Full validation before push
./scripts/pre-push.sh

# Use with Nix for reproducible environment
nix develop --command ./scripts/pre-push.sh
```

### Setting up as Git Hook

**Optional**: Install pre-push.sh as a git hook

```bash
# Create symlink
ln -s ../../scripts/pre-push.sh .git/hooks/pre-push
chmod +x .git/hooks/pre-push

# Now runs automatically on `git push`
```

## Layer 3: CI Workflows

### Overview

CI workflows provide the safety net that cannot be bypassed, required for PR merge via branch protection.

### Workflows

#### 1. Policy Enforcement (`.github/workflows/policy.yml`)

**Purpose**: Comprehensive policy validation

**Jobs**:

1. **quality-gates** (runs in parallel)
   - Formatting check
   - Clippy (zero warnings)
   - Tests (100% pass rate)
   - Documentation (zero warnings)

2. **security-scanning** (runs in parallel)
   - cargo-audit (vulnerabilities)
   - cargo-deny (licenses, advisories)

3. **test-connectivity** (runs in parallel)
   - Check for `.rs.disabled` files
   - Verify non-zero test counts per crate

4. **performance-gates** (PR only, runs in parallel)
   - Benchmark PR branch
   - Benchmark base branch
   - Compare with 5% threshold

5. **policy-summary** (depends on all above)
   - Aggregate results
   - Fail if any job failed
   - Report summary

**Configuration**:
```yaml
name: Policy Enforcement

on:
  pull_request:
  push:
    branches: [main, master]

concurrency:
  group: ${{ github.workflow }}-${{ github.ref }}
  cancel-in-progress: true

env:
  RUST_BACKTRACE: 1
  RUST_TEST_THREADS: 2
  RAYON_NUM_THREADS: 4

jobs:
  quality-gates:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Install Nix
        uses: cachix/install-nix-action@v27
      - name: Setup Nix cache
        uses: cachix/cachix-action@v15
        with:
          name: rust-sitter
          authToken: '${{ secrets.CACHIX_AUTH_TOKEN }}'
      - name: Check Formatting
        run: nix develop --command cargo fmt --all -- --check
      # ... more steps ...
```

**Performance**: ~30 minutes (parallel execution)

#### 2. Secret Detection (`.github/workflows/secrets.yml`)

**Purpose**: Prevent credential leaks

**Jobs**:

1. **trufflehog** (runs in parallel)
   - Scans git history for secrets
   - Uses verified signatures
   - Entropy-based detection

2. **pattern-scan** (runs in parallel)
   - API keys (AWS, Stripe, GitHub)
   - Tokens (Bearer, OAuth)
   - Passwords, private keys
   - Common secret patterns

3. **entropy-scan** (runs in parallel)
   - High entropy strings (>4.5 Shannon entropy)
   - Base64-encoded secrets
   - Hex-encoded secrets

4. **file-analysis** (runs in parallel)
   - Sensitive file patterns (`.pem`, `.key`)
   - Credentials files
   - Certificates

5. **secrets-summary** (depends on all above)
   - Aggregate results
   - Fail if any job failed
   - Report findings

**Configuration**:
```yaml
name: Secret Detection

on:
  pull_request:
  push:
    branches: [main, master]
  workflow_dispatch:

jobs:
  trufflehog:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0  # Full history
      - uses: trufflesecurity/trufflehog@main
        with:
          path: ./
          base: ${{ github.event.repository.default_branch }}
          head: HEAD
          extra_args: --debug --only-verified

  pattern-scan:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Scan for secret patterns
        run: |
          # ... pattern matching script ...
  # ... more jobs ...
```

**Performance**: ~10 minutes (parallel execution)

### Branch Protection

**Configuration** (GitHub repository settings):

```json
{
  "required_status_checks": {
    "strict": true,
    "contexts": [
      "Policy Enforcement / policy-summary",
      "Secret Detection / secrets-summary"
    ]
  },
  "required_pull_request_reviews": {
    "required_approving_review_count": 1
  },
  "enforce_admins": true,
  "restrictions": null
}
```

**Effect**: Pull requests cannot be merged until:
1. Policy Enforcement workflow passes
2. Secret Detection workflow passes
3. At least 1 approving review

## Configuration Files

### 1. `audit.toml` (cargo-audit)

**Purpose**: Configure vulnerability scanning

**Key sections**:
```toml
[advisories]
# Advisory database
db-path = "~/.cargo/advisory-db"
db-urls = ["https://github.com/rustsec/advisory-db"]

# Severity threshold
severity_threshold = "medium"  # Fail on medium+

# Ignores (with justification)
ignore = [
    # "RUSTSEC-2023-0001",  # Example
    # Justification: ...
    # Mitigation: ...
]

[yanked]
enabled = true  # Fail on yanked crates

[unmaintained]
enabled = true  # Warn on unmaintained

[unsound]
enabled = true  # Fail on unsound
```

### 2. `deny.toml` (cargo-deny)

**Purpose**: Configure license compliance and dependency management

**Key sections**:
```toml
[graph]
targets = [
    { triple = "x86_64-unknown-linux-gnu" },
    { triple = "x86_64-apple-darwin" },
    { triple = "x86_64-pc-windows-msvc" },
    { triple = "wasm32-unknown-unknown" },
]

[licenses]
allow = [
    "MIT",
    "Apache-2.0",
    "Apache-2.0 WITH LLVM-exception",
    "BSD-2-Clause",
    "BSD-3-Clause",
    "ISC",
]

[bans]
multiple-versions = "warn"
deny = [
    # { name = "openssl", reason = "Use rustls" },
]
```

### 3. `.pre-commit-config.yaml`

**Purpose**: Configure pre-commit hooks

(See Layer 1 section for full configuration)

### 4. `.github/workflows/policy.yml`

**Purpose**: CI policy enforcement workflow

(See Layer 3 section for full configuration)

### 5. `.github/workflows/secrets.yml`

**Purpose**: Secret detection workflow

(See Layer 3 section for full configuration)

## Testing Policy Enforcement

### Unit Testing Policies

**Test scenarios**:

1. **Formatting violation**
   ```bash
   # Introduce formatting issue
   echo "pub fn test(){}" >> src/lib.rs

   # Test pre-commit
   git add src/lib.rs
   git commit -m "test: Formatting"
   # Expected: FAIL with "cargo fmt" suggestion

   # Fix
   cargo fmt
   git add src/lib.rs
   git commit -m "test: Formatting"
   # Expected: PASS
   ```

2. **Clippy warning**
   ```bash
   # Introduce warning
   echo "pub fn test() { let x = 1; }" >> src/lib.rs

   # Test verification
   ./scripts/check-quality.sh
   # Expected: FAIL with "unused variable" warning

   # Fix
   echo "pub fn test() { let _x = 1; }" > src/lib.rs
   ./scripts/check-quality.sh
   # Expected: PASS
   ```

3. **Vulnerability detection**
   ```bash
   # Add vulnerable dependency (test only!)
   cargo add [email protected]

   # Test security check
   ./scripts/check-security.sh
   # Expected: FAIL with RUSTSEC advisory

   # Fix
   cargo update -p tokio
   ./scripts/check-security.sh
   # Expected: PASS
   ```

4. **Secret detection**
   ```bash
   # Introduce secret (test only!)
   echo 'API_KEY="sk_live_test123"' > config.txt
   git add config.txt

   # Test pre-commit (secret scan)
   git commit -m "test: Secret"
   # Expected: WARNING or FAIL

   # Fix
   rm config.txt
   ```

### Integration Testing

**Full workflow test**:
```bash
# 1. Create feature branch
git checkout -b test-policy-enforcement

# 2. Make changes
echo "// Test" >> src/lib.rs

# 3. Test local validation
./scripts/pre-push.sh
# Expected: PASS

# 4. Push to remote
git push origin test-policy-enforcement

# 5. Create PR
gh pr create --title "Test: Policy enforcement" --body "Testing"

# 6. Wait for CI
gh pr checks
# Expected: All checks PASS

# 7. Merge PR
gh pr merge --squash

# 8. Cleanup
git checkout main
git pull
git branch -d test-policy-enforcement
```

## Maintenance

### Updating Pre-commit Hooks

```bash
# Update pre-commit framework
pip install --upgrade pre-commit

# Update hook versions
pre-commit autoupdate

# Test updated hooks
pre-commit run --all-files
```

### Updating Verification Scripts

```bash
# Edit script
vim scripts/check-quality.sh

# Make executable
chmod +x scripts/check-quality.sh

# Test
./scripts/check-quality.sh

# Commit
git add scripts/check-quality.sh
git commit -m "chore: Update check-quality script"
```

### Updating CI Workflows

```bash
# Edit workflow
vim .github/workflows/policy.yml

# Test locally with act (GitHub Actions local runner)
act pull_request -j quality-gates

# Commit and test on CI
git add .github/workflows/policy.yml
git commit -m "ci: Update policy workflow"
git push
```

### Updating Policy Configuration

```bash
# Update audit.toml or deny.toml
vim audit.toml

# Test locally
cargo audit
cargo deny check

# Commit
git add audit.toml
git commit -m "chore: Update audit config"
```

### Quarterly Review

**Checklist**:
- [ ] Review ignored advisories (still valid?)
- [ ] Check for new security tools
- [ ] Analyze policy violation trends
- [ ] Update documentation
- [ ] Benchmark performance (still meeting targets?)
- [ ] Review temporary policy overrides (can be removed?)

## Troubleshooting

### Pre-commit Hooks

**Problem**: Hooks not running

```bash
# Reinstall
pre-commit uninstall
pre-commit install
pre-commit install --hook-type commit-msg
```

**Problem**: Slow hook execution

```bash
# Check hook timing
time pre-commit run --all-files

# Consider disabling slow hooks locally
# (CI will still run them)
SKIP=cargo-clippy git commit
```

### Verification Scripts

**Problem**: Script not found

```bash
# Check permissions
ls -la scripts/
chmod +x scripts/*.sh
```

**Problem**: Tool not found

```bash
# Use Nix shell
nix develop --command ./scripts/check-quality.sh

# Or install manually
cargo install cargo-audit cargo-deny
```

### CI Workflows

**Problem**: Workflow not triggering

```bash
# Check workflow file syntax
yamllint .github/workflows/policy.yml

# Check branch protection rules
gh api repos/:owner/:repo/branches/main/protection
```

**Problem**: Slow CI execution

```bash
# Check job timing
gh run view <run-id> --log

# Consider:
# - Adding more caching
# - Parallelizing more jobs
# - Reducing test concurrency
```

**Problem**: Flaky tests

```bash
# Increase concurrency caps
# In workflow:
env:
  RUST_TEST_THREADS: 1  # More stable
  RAYON_NUM_THREADS: 2  # Reduce parallelism
```

## Performance Optimization

### Caching Strategies

**Cargo cache**:
```yaml
- name: Cache cargo registry
  uses: actions/cache@v3
  with:
    path: ~/.cargo/registry
    key: ${{ runner.os }}-cargo-registry-${{ hashFiles('**/Cargo.lock') }}
```

**Nix cache**:
```yaml
- name: Setup Nix cache
  uses: cachix/cachix-action@v15
  with:
    name: rust-sitter
    authToken: '${{ secrets.CACHIX_AUTH_TOKEN }}'
```

### Incremental Checks

**Only run on changed files** (pre-commit):
```yaml
- id: cargo-clippy
  pass_filenames: true  # Only check changed files
```

**Skip benchmarks on non-performance changes**:
```yaml
performance-gates:
  if: contains(github.event.pull_request.labels.*.name, 'performance')
```

### Parallel Execution

**Pre-commit parallel hooks**:
```bash
# Run hooks in parallel
pre-commit run --all-files --parallel
```

**CI parallel jobs**:
```yaml
jobs:
  quality-gates:
    # ...
  security-scanning:
    # Runs in parallel with quality-gates
```

## Related Documentation

- [POLICIES.md](../../POLICIES.md) - User-facing policy documentation
- [ADR-0010: Policy-as-Code](../adr/ADR-0010-POLICY-AS-CODE.md) - Architecture decision record
- [BDD Planning](../plans/BDD_POLICY_ENFORCEMENT.md) - BDD scenarios and acceptance criteria
- [Phase 1B Contract](../specs/POLICY_AS_CODE_CONTRACT.md) - Implementation contract

---

**Last Updated**: 2025-11-20
**Version**: 1.0.0 (Phase 1B complete)
**Maintained by**: rust-sitter maintainers
