# Policy-as-Code Contract

**Version**: 1.0.0
**Date**: 2025-11-20
**Status**: 📋 **PLANNED** (Phase 1B - Week 2)
**Predecessor**: [NIX_CI_INTEGRATION_CONTRACT.md](./NIX_CI_INTEGRATION_CONTRACT.md) (80% complete)
**Target**: Infrastructure Phase I completion - Automated governance and quality gates
**Strategic Context**: [STRATEGIC_IMPLEMENTATION_PLAN.md Phase 1B](../plans/STRATEGIC_IMPLEMENTATION_PLAN.md)

---

## Executive Summary

This contract defines **Policy-as-Code v1** - automated enforcement of quality, security, and governance standards throughout the rust-sitter development lifecycle.

**Goal**: Eliminate manual quality checks through automated policy enforcement, ensuring consistent standards across all contributions while maintaining development velocity.

**Success Criteria**:
- Zero manual quality checks required for PRs
- 100% automated policy enforcement
- Clear failure reasons with actionable remediation
- No false positives blocking valid contributions

---

## I. Strategic Context

### The Quality Problem

**Current State** (Manual Quality Checks):
- Manual code review for formatting, linting
- Security vulnerabilities discovered late
- Test connectivity issues found in CI
- Performance regressions detected after merge
- Inconsistent standards across contributors

**Pain Points**:
- ⏱️ Time-consuming manual review
- 🐛 Issues slip through to main branch
- 😤 Frustration from inconsistent enforcement
- 🔄 Rework after merge
- 📉 Quality debt accumulation

### Policy-as-Code Solution

**Automated Enforcement**:
- ✅ Pre-commit hooks catch issues locally
- ✅ CI gates block problematic PRs
- ✅ Security scanning prevents vulnerabilities
- ✅ Performance gates prevent regressions
- ✅ Test connectivity ensures no silent failures

**Benefits**:
- 🚀 Faster development (catch issues early)
- 🛡️ Higher quality (automated standards)
- 😊 Better contributor experience (clear feedback)
- 📈 Reduced rework (issues caught before merge)
- 🔒 Improved security posture

---

## II. Scope Definition

### In Scope for Policy v1

1. **Pre-commit Hooks** (AC-P1)
   - Formatting enforcement (cargo fmt)
   - Linting enforcement (cargo clippy)
   - Test connectivity verification (no .rs.disabled)
   - Commit message validation
   - Large file prevention

2. **CI Policy Enforcement** (AC-P2)
   - Quality gates (zero warnings, 100% test pass)
   - Security scanning (cargo audit, cargo deny)
   - Performance regression detection (5% threshold)
   - Test connectivity safeguards (non-zero test counts)
   - Documentation coverage validation

3. **Security Policies** (AC-P3)
   - Dependency vulnerability scanning
   - License compliance checking
   - Secret detection (API keys, tokens)
   - SBOM generation (Software Bill of Materials)
   - Security advisory monitoring

4. **Quality Verification Scripts** (AC-P4)
   - `scripts/check-quality.sh` - Local quality verification
   - `scripts/check-security.sh` - Security posture check
   - `scripts/pre-push.sh` - Pre-push validation
   - Integration with pre-commit framework

5. **Documentation & Governance** (AC-P5)
   - Policy documentation (what, why, how)
   - Override procedures for exceptions
   - Policy versioning and evolution
   - ADR documenting policy decisions
   - Contributor guide updates

### Out of Scope for Policy v1

1. **Advanced Static Analysis** - Beyond clippy (deferred to v1.1)
2. **Code Coverage Enforcement** - Metrics only (deferred to v1.1)
3. **Automated Dependency Updates** - Dependabot/Renovate (separate concern)
4. **Advanced Security Scanning** - SAST/DAST tools (deferred)
5. **Policy as Configuration** - YAML/TOML policies (use code for v1)

---

## III. Acceptance Criteria

### AC-P1: Pre-commit Hooks

**Requirement**: Local quality gates catch issues before commit.

**Success Criteria**:
1. `.pre-commit-config.yaml` defines all hooks
2. Hooks install automatically on `nix develop` entry
3. Hooks run on `git commit` and block if failures
4. Clear failure messages with remediation steps
5. Fast execution (<5 seconds for typical commit)

**BDD Scenario**:
```gherkin
Scenario: Pre-commit hooks catch formatting issues
  Given I have modified Rust files
  And the files are not formatted correctly
  When I run "git commit -m 'feat: new feature'"
  Then the commit is blocked
  And I see a clear error: "Code not formatted. Run: cargo fmt"
  And the error includes affected files
  When I run "cargo fmt"
  And I run "git commit -m 'feat: new feature'"
  Then the commit succeeds
```

**Hooks Configuration**:
```yaml
# .pre-commit-config.yaml
repos:
  - repo: local
    hooks:
      # Formatting
      - id: cargo-fmt
        name: Cargo Format Check
        entry: cargo fmt --all -- --check
        language: system
        types: [rust]
        pass_filenames: false

      # Linting
      - id: cargo-clippy
        name: Cargo Clippy
        entry: cargo clippy --all-targets -- -D warnings
        language: system
        types: [rust]
        pass_filenames: false

      # Test connectivity
      - id: test-connectivity
        name: Test Connectivity Check
        entry: ./scripts/check-test-connectivity.sh
        language: system
        pass_filenames: false

      # Commit message
      - id: commit-msg
        name: Commit Message Validation
        entry: ./scripts/validate-commit-msg.sh
        language: system
        stages: [commit-msg]

      # Large files
      - id: large-files
        name: Prevent Large Files
        entry: ./scripts/check-large-files.sh
        language: system
```

**Installation**:
```bash
# Automatic in Nix shell (shellHook)
if [ -f .pre-commit-config.yaml ]; then
    if ! pre-commit --version >/dev/null 2>&1; then
        echo "Installing pre-commit..."
        pip install --user pre-commit
    fi
    pre-commit install
fi
```

**Deliverables**:
- [ ] `.pre-commit-config.yaml` with 5+ hooks
- [ ] `scripts/validate-commit-msg.sh` (conventional commits)
- [ ] `scripts/check-large-files.sh` (>1MB warning)
- [ ] Nix shell auto-installation
- [ ] Documentation: `docs/guides/PRE_COMMIT_HOOKS.md`

---

### AC-P2: CI Policy Enforcement

**Requirement**: CI automatically enforces all quality policies.

**Success Criteria**:
1. `.github/workflows/policy.yml` defines policy checks
2. Policy workflow runs on all PRs
3. Blocks merge if any policy violation
4. Clear failure summary in PR comments
5. No false positives (properly tuned thresholds)

**BDD Scenario**:
```gherkin
Scenario: CI policy workflow catches quality violations
  Given a PR with code changes
  And the code has clippy warnings
  When GitHub Actions CI runs
  Then the policy workflow fails
  And PR status shows "Policy checks failed"
  And PR comment shows:
    """
    ❌ Policy Violations Found:

    **Quality**:
    - Clippy warnings: 3 found (threshold: 0)
      - src/parser.rs:123: unused variable
      - src/tree.rs:456: needless borrow
      - src/edit.rs:789: redundant clone

    **Action Required**:
    Run `cargo clippy --fix` and push changes.
    """
  And the merge button is disabled
```

**Policy Workflow**:
```yaml
# .github/workflows/policy.yml
name: Policy Enforcement

on:
  pull_request:
  push:
    branches: [main, master]

jobs:
  quality-gates:
    name: Quality Gates
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Nix
        uses: cachix/install-nix-action@v27

      - name: Check Formatting
        run: nix develop --command cargo fmt --all -- --check

      - name: Check Clippy (Zero Warnings)
        run: nix develop --command cargo clippy --all-targets -- -D warnings

      - name: Check Test Pass Rate
        run: |
          nix develop --command cargo test --workspace -- --test-threads=2 > test_output.txt
          if ! grep -q "test result: ok" test_output.txt; then
            echo "❌ Tests failed"
            exit 1
          fi

      - name: Check Documentation
        run: nix develop --command cargo doc --no-deps --all-features
        env:
          RUSTDOCFLAGS: -D warnings

  security-scanning:
    name: Security Scanning
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Cargo Audit (Vulnerabilities)
        uses: actions-rs/audit-check@v1
        with:
          token: ${{ secrets.GITHUB_TOKEN }}

      - name: Cargo Deny (Licenses & Security)
        run: |
          cargo install cargo-deny
          cargo deny check

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

      - name: Benchmark Base Branch
        run: |
          git checkout ${{ github.base_ref }}
          nix develop --command cargo bench --workspace -- --save-baseline base

      - name: Compare Performance
        run: |
          ./scripts/check-perf-regression.sh base pr 5
          # Fails if any benchmark regresses >5%

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

      - name: Verify Non-Zero Test Counts
        run: |
          ./scripts/check-test-connectivity.sh
          # Ensures all crates have tests discovered
```

**Deliverables**:
- [ ] `.github/workflows/policy.yml` (complete workflow)
- [ ] `scripts/check-perf-regression.sh` (performance comparison)
- [ ] PR comment integration (failure summaries)
- [ ] Status badge for README
- [ ] Documentation: Policy enforcement guide

---

### AC-P3: Security Policies

**Requirement**: Automated security scanning prevents vulnerabilities.

**Success Criteria**:
1. `cargo audit` scans for known vulnerabilities
2. `cargo deny` enforces license compliance
3. Secret detection prevents credential leaks
4. SBOM generated for compliance
5. Security advisories monitored (GitHub Dependabot)

**BDD Scenario**:
```gherkin
Scenario: Security scanning catches vulnerability
  Given a dependency with a known CVE
  When GitHub Actions security scan runs
  Then the scan fails
  And the failure shows:
    """
    ❌ Security Vulnerability Detected:

    **Crate**: tokio v1.20.0
    **CVE**: CVE-2023-XXXX
    **Severity**: HIGH
    **Description**: Data race in task spawning

    **Action Required**:
    Update to tokio v1.21.0 or later:
      cargo update -p tokio
    """
  And the PR is blocked until fixed
```

**Security Configuration**:

1. **Cargo Audit** (`audit.toml`):
```toml
[advisories]
db-path = "~/.cargo/advisory-db"
db-urls = ["https://github.com/rustsec/advisory-db"]
vulnerability = "deny"
unmaintained = "warn"
unsound = "warn"
yanked = "deny"

[bans]
multiple-versions = "warn"
wildcards = "deny"
```

2. **Cargo Deny** (`deny.toml`):
```toml
[advisories]
vulnerability = "deny"
unmaintained = "warn"
notice = "warn"
ignore = []

[licenses]
unlicensed = "deny"
allow = ["MIT", "Apache-2.0", "BSD-3-Clause"]
deny = ["GPL-2.0", "GPL-3.0"]
copyleft = "deny"

[bans]
multiple-versions = "warn"
deny = []

[sources]
unknown-registry = "deny"
unknown-git = "deny"
```

3. **Secret Detection** (`.github/workflows/secrets.yml`):
```yaml
name: Secret Detection

on: [push, pull_request]

jobs:
  detect-secrets:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0

      - name: TruffleHog Scan
        uses: trufflesecurity/trufflehog@main
        with:
          path: ./
          base: ${{ github.event.repository.default_branch }}
          head: HEAD
```

4. **SBOM Generation**:
```bash
# scripts/generate-sbom.sh
#!/bin/bash
cargo install cargo-sbom
cargo sbom > sbom.json
# Upload to artifact storage
```

**Deliverables**:
- [ ] `audit.toml` configuration
- [ ] `deny.toml` configuration
- [ ] Secret detection workflow
- [ ] SBOM generation script
- [ ] Security policy document (`SECURITY.md`)
- [ ] Vulnerability disclosure process

---

### AC-P4: Quality Verification Scripts

**Requirement**: Local scripts enable self-service quality validation.

**Success Criteria**:
1. `scripts/check-quality.sh` runs all quality checks locally
2. `scripts/check-security.sh` runs security scans
3. `scripts/pre-push.sh` validates before pushing
4. Scripts provide clear pass/fail and remediation
5. Fast execution (<30 seconds for typical run)

**BDD Scenario**:
```gherkin
Scenario: Developer runs local quality check
  Given I have uncommitted changes
  When I run "./scripts/check-quality.sh"
  Then I see:
    """
    🔍 Checking Quality...

    ✅ Formatting (cargo fmt): PASS
    ✅ Clippy (cargo clippy): PASS
    ✅ Tests (cargo test): PASS (144/144)
    ✅ Documentation (cargo doc): PASS
    ✅ Test Connectivity: PASS

    🎉 All quality checks passed!
    """
  And the script exits with code 0

Scenario: Local quality check finds issues
  Given I have code with clippy warnings
  When I run "./scripts/check-quality.sh"
  Then I see:
    """
    🔍 Checking Quality...

    ✅ Formatting: PASS
    ❌ Clippy: FAIL (3 warnings)
       src/parser.rs:123: unused variable 'x'
       src/tree.rs:456: needless borrow
    ⏭️  Tests: SKIPPED (fix clippy first)

    ❌ Quality checks failed.

    To fix:
      cargo clippy --fix
    """
  And the script exits with code 1
```

**Script Implementations**:

1. **`scripts/check-quality.sh`**:
```bash
#!/usr/bin/env bash
set -euo pipefail

echo "🔍 Checking Quality..."
echo ""

# Formatting
echo -n "✅ Formatting (cargo fmt): "
if cargo fmt --all -- --check >/dev/null 2>&1; then
    echo "PASS"
else
    echo "FAIL"
    echo "  Run: cargo fmt"
    exit 1
fi

# Clippy
echo -n "✅ Clippy (cargo clippy): "
if cargo clippy --all-targets -- -D warnings 2>&1 | tee /tmp/clippy.log | grep -q "0 warnings"; then
    echo "PASS"
else
    echo "FAIL"
    grep "warning:" /tmp/clippy.log | head -5
    echo "  Run: cargo clippy --fix"
    exit 1
fi

# Tests
echo -n "✅ Tests (cargo test): "
if cargo test --workspace -- --test-threads=2 >/tmp/test.log 2>&1; then
    TEST_COUNT=$(grep -oP '\d+(?= passed)' /tmp/test.log | head -1)
    echo "PASS ($TEST_COUNT tests)"
else
    echo "FAIL"
    tail -20 /tmp/test.log
    exit 1
fi

# Documentation
echo -n "✅ Documentation (cargo doc): "
if RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --all-features >/dev/null 2>&1; then
    echo "PASS"
else
    echo "FAIL"
    echo "  Fix documentation warnings"
    exit 1
fi

# Test connectivity
echo -n "✅ Test Connectivity: "
if ./scripts/check-test-connectivity.sh >/dev/null 2>&1; then
    echo "PASS"
else
    echo "FAIL"
    echo "  Some tests may be disconnected"
    exit 1
fi

echo ""
echo "🎉 All quality checks passed!"
```

2. **`scripts/check-security.sh`**:
```bash
#!/usr/bin/env bash
set -euo pipefail

echo "🔒 Checking Security..."
echo ""

# Cargo audit
echo -n "🔍 Vulnerability Scan (cargo audit): "
if cargo audit >/dev/null 2>&1; then
    echo "PASS"
else
    echo "FAIL"
    cargo audit
    exit 1
fi

# Cargo deny
echo -n "📜 License Compliance (cargo deny): "
if cargo deny check >/dev/null 2>&1; then
    echo "PASS"
else
    echo "FAIL"
    cargo deny check
    exit 1
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
echo "🎉 All security checks passed!"
```

3. **`scripts/pre-push.sh`**:
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

**Deliverables**:
- [ ] `scripts/check-quality.sh` (comprehensive quality)
- [ ] `scripts/check-security.sh` (security scanning)
- [ ] `scripts/pre-push.sh` (pre-push validation)
- [ ] Git hook integration (optional)
- [ ] Documentation: `docs/guides/LOCAL_QUALITY_CHECKS.md`

---

### AC-P5: Documentation & Governance

**Requirement**: Clear policy documentation and governance processes.

**Success Criteria**:
1. Policy documentation explains what, why, how
2. Override procedures documented for exceptions
3. Policy versioning and evolution process defined
4. ADR documents policy architecture decisions
5. Contributor guide updated with policy info

**BDD Scenario**:
```gherkin
Scenario: Contributor understands policy requirements
  Given I am a new contributor
  When I read the policy documentation
  Then I understand:
    - What policies are enforced
    - Why each policy exists
    - How to satisfy each policy
    - How to request exceptions
  And I can run local checks before pushing
  And I know what CI will check
```

**Documentation Structure**:

1. **`POLICIES.md`** (Policy Overview):
```markdown
# rust-sitter Development Policies

## Quality Policies

### Formatting (Zero Tolerance)
**What**: All code must be formatted with `cargo fmt`
**Why**: Consistent formatting reduces review burden
**How**: Run `cargo fmt` before commit
**Enforcement**: Pre-commit hook + CI

### Linting (Zero Warnings)
**What**: All code must pass `cargo clippy` without warnings
**Why**: Clippy catches common bugs and anti-patterns
**How**: Run `cargo clippy --fix` to auto-fix
**Enforcement**: Pre-commit hook + CI

### Test Pass Rate (100%)
**What**: All tests must pass
**Why**: Broken tests indicate broken functionality
**How**: Run `cargo test` and fix failures
**Enforcement**: CI (blocking)

## Security Policies

### Vulnerability Scanning
**What**: No dependencies with known CVEs
**Why**: Security vulnerabilities put users at risk
**How**: Run `cargo audit` and update dependencies
**Enforcement**: CI (blocking)

### License Compliance
**What**: Only approved licenses (MIT, Apache-2.0, BSD-3)
**Why**: Legal compliance and compatibility
**How**: Check `cargo deny` output
**Enforcement**: CI (blocking)

## Performance Policies

### Regression Detection (5% Threshold)
**What**: Benchmarks must not regress >5%
**Why**: Performance is a feature
**How**: Run `cargo bench` and optimize
**Enforcement**: CI (on PR)

## Override Procedures

### When to Request Override
- False positive from automated check
- Intentional violation for valid reason
- External dependency issue blocking progress

### How to Request Override
1. Open GitHub issue explaining:
   - What policy is violated
   - Why override is needed
   - What mitigation is in place
2. Get approval from 2+ maintainers
3. Add exception to policy configuration
4. Document in PR description

### Example Exceptions
- Audit ignore: Known CVE with no fix, mitigation in place
- Clippy allow: Intentional pattern for performance
- Performance: Intentional trade-off (e.g., correctness over speed)
```

2. **`docs/guides/POLICY_ENFORCEMENT.md`** (Detailed Guide):
- Policy architecture overview
- Local workflow (pre-commit → pre-push → CI)
- Troubleshooting common policy failures
- Performance tuning (disabling checks locally)

3. **ADR-0010: Policy-as-Code Architecture**:
- Decision rationale (why automate policies)
- Trade-offs analyzed (strictness vs velocity)
- Tool selection (pre-commit, cargo-audit, cargo-deny)
- Evolution strategy (how policies change over time)

**Deliverables**:
- [ ] `POLICIES.md` (policy overview)
- [ ] `docs/guides/POLICY_ENFORCEMENT.md` (detailed guide)
- [ ] `docs/adr/ADR-0010-POLICY-AS-CODE.md` (architecture)
- [ ] `CONTRIBUTING.md` updated (policy section)
- [ ] Override request template (`.github/ISSUE_TEMPLATE/policy-override.md`)

---

## IV. Implementation Plan

### Week 2: Foundation (Days 1-3)

**Goal**: Establish core policy infrastructure

**Day 1: Pre-commit Setup**
- [ ] Create `.pre-commit-config.yaml`
- [ ] Install pre-commit in Nix shell
- [ ] Test formatting hook
- [ ] Test clippy hook

**Day 2: Verification Scripts**
- [ ] Implement `scripts/check-quality.sh`
- [ ] Implement `scripts/check-security.sh`
- [ ] Implement `scripts/pre-push.sh`
- [ ] Make scripts executable
- [ ] Test all scripts

**Day 3: CI Policy Workflow**
- [ ] Create `.github/workflows/policy.yml`
- [ ] Implement quality gates job
- [ ] Test workflow on test PR

**Deliverables**:
- Pre-commit hooks functional
- Verification scripts working
- CI policy workflow running

---

### Week 2: Security & Performance (Days 4-5)

**Goal**: Add security and performance policies

**Day 4: Security Scanning**
- [ ] Create `audit.toml` configuration
- [ ] Create `deny.toml` configuration
- [ ] Add security scanning to CI
- [ ] Test with intentional vulnerability
- [ ] Add secret detection workflow

**Day 5: Performance Gates**
- [ ] Implement `scripts/check-perf-regression.sh`
- [ ] Add performance job to CI
- [ ] Test with intentional regression
- [ ] Document performance policy

**Deliverables**:
- Security scanning complete
- Performance gates operational

---

### Week 2: Documentation (Day 5)

**Goal**: Complete policy documentation

**Tasks**:
- [ ] Write `POLICIES.md`
- [ ] Write `docs/guides/POLICY_ENFORCEMENT.md`
- [ ] Create `docs/adr/ADR-0010-POLICY-AS-CODE.md`
- [ ] Update `CONTRIBUTING.md`
- [ ] Create policy override template

**Deliverables**:
- Complete policy documentation
- Contributor guide updated

---

## V. Test Strategy

### Pre-commit Hook Tests

```bash
# Test formatting hook
echo "fn foo() {return 1;}" > src/test.rs  # Bad formatting
git add src/test.rs
git commit -m "test"  # Should fail
cargo fmt
git commit -m "test"  # Should succeed
```

### CI Policy Tests

Create test PRs with intentional violations:

1. **Bad Formatting PR**: Code not formatted
2. **Clippy Warnings PR**: Code with warnings
3. **Test Failure PR**: Failing test
4. **Security Issue PR**: Dependency with CVE
5. **Performance Regression PR**: 10% slowdown

Each should be blocked by CI with clear message.

### Verification Script Tests

```bash
# Test quality script
./scripts/check-quality.sh  # Should pass on clean code

# Introduce issue
echo "fn unused() {}" >> src/lib.rs  # Unused function
./scripts/check-quality.sh  # Should fail with clear message
```

---

## VI. Success Metrics

### Quantitative

| Metric | Target | Measurement |
|--------|--------|-------------|
| **Pre-commit Hook Success** | >95% | Commit attempts that pass |
| **CI Policy Pass Rate** | 100% (on main) | PRs merged without policy violations |
| **False Positive Rate** | <5% | Policy blocks on valid code |
| **Time to Policy Feedback** | <2 min | From commit to hook result |
| **Security Vulnerabilities** | 0 | Known CVEs in dependencies |

### Qualitative

| Metric | Success Criteria |
|--------|------------------|
| **Developer Experience** | "Policies help, don't hinder" |
| **Clear Feedback** | Developers know how to fix violations |
| **Low Friction** | Policies don't significantly slow development |
| **Trust** | Team trusts automated checks |

---

## VII. Risk Management

### High Risks

| Risk | Impact | Likelihood | Mitigation |
|------|--------|------------|------------|
| False positives block work | HIGH | MEDIUM | Well-tuned thresholds, override process |
| Slow pre-commit hooks | MEDIUM | MEDIUM | Optimize hooks, allow skip for WIP |
| Security tool failures | HIGH | LOW | Multiple tools, manual fallback |

### Mitigation Strategies

**False Positives**:
- Careful threshold tuning (start lenient, tighten)
- Clear override process
- Regular policy review

**Performance**:
- Optimize hook execution (<5s target)
- Allow `git commit --no-verify` for WIP
- Catch in CI if skipped locally

**Tool Reliability**:
- Multiple security tools (audit + deny)
- Fallback to manual review if tools fail
- Monitor tool health in CI

---

## VIII. Definition of Done

Policy-as-Code v1 is **DONE** when:

1. ✅ All acceptance criteria (AC-P1 through AC-P5) met
2. ✅ Pre-commit hooks installed and functional
3. ✅ CI policy workflow running on all PRs
4. ✅ Security scanning catching vulnerabilities
5. ✅ Performance gates preventing regressions
6. ✅ Verification scripts working locally
7. ✅ Documentation complete and reviewed
8. ✅ Test PRs validate each policy
9. ✅ Team trained on policy usage
10. ✅ Override process documented

---

## IX. Current Status

**Status**: 📋 **PLANNED** (not yet started)
**Prerequisite**: Phase 1A (Nix CI) 80% complete
**Estimated Duration**: 1 week (5 days)
**Target Start**: After Phase 1A AC-2,AC-3 verification
**Target Completion**: End of Week 2

**Readiness**:
- [x] Nix infrastructure in place ✅
- [x] CI workflows established ✅
- [x] Test connectivity safeguards exist ✅
- [ ] Pre-commit framework needed
- [ ] Security scanning tools needed

---

## X. References

### Related Contracts

- [NIX_CI_INTEGRATION_CONTRACT.md](./NIX_CI_INTEGRATION_CONTRACT.md) (80% complete)
- [GLR_V1_COMPLETION_CONTRACT.md](./GLR_V1_COMPLETION_CONTRACT.md) (100% complete)
- [GLR_INCREMENTAL_CONTRACT.md](./GLR_INCREMENTAL_CONTRACT.md) (planned)

### Architecture Decision Records

- [ADR-0008: Nix Development Environment](../adr/ADR-0008-NIX-DEVELOPMENT-ENVIRONMENT.md)
- [ADR-0010: Policy-as-Code Architecture](../adr/ADR-0010-POLICY-AS-CODE.md) (to be created)

### External References

- [Pre-commit Framework](https://pre-commit.com/)
- [Cargo Audit](https://github.com/rustsec/rustsec/tree/main/cargo-audit)
- [Cargo Deny](https://github.com/EmbarkStudios/cargo-deny)
- [GitHub Actions Best Practices](https://docs.github.com/en/actions/security-guides/security-hardening-for-github-actions)

---

**Contract Version**: 1.0.0
**Last Updated**: 2025-11-20
**Next Review**: After Phase 1A completion
**Owner**: rust-sitter core team

---

**Signatures** (for contract acceptance):

- [ ] Technical Lead: _______________ Date: ___________
- [ ] Security Lead: _______________ Date: ___________
- [ ] Quality Assurance: _______________ Date: ___________

---

END OF CONTRACT
