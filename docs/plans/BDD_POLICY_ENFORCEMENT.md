# BDD Specifications: Policy-as-Code Enforcement

**Version**: 1.0.0
**Date**: 2025-11-20
**Status**: 📋 **PLANNED** (Phase 1B - Week 2)
**Related Contract**: [POLICY_AS_CODE_CONTRACT.md](../specs/POLICY_AS_CODE_CONTRACT.md)
**Related ADR**: [ADR-0010-POLICY-AS-CODE.md](../adr/ADR-0010-POLICY-AS-CODE.md)

---

## Executive Summary

This document defines **32 BDD scenarios** covering all 5 acceptance criteria for Policy-as-Code v1. Each scenario is written in Gherkin format and maps to specific contract requirements.

**Scenario Distribution**:
- **AC-P1** (Pre-commit Hooks): 8 scenarios
- **AC-P2** (CI Policy Enforcement): 10 scenarios
- **AC-P3** (Security Policies): 6 scenarios
- **AC-P4** (Quality Verification Scripts): 5 scenarios
- **AC-P5** (Documentation & Governance): 3 scenarios

**Total**: 32 scenarios covering 100% of contract acceptance criteria

---

## Table of Contents

1. [AC-P1: Pre-commit Hooks](#ac-p1-pre-commit-hooks) (8 scenarios)
2. [AC-P2: CI Policy Enforcement](#ac-p2-ci-policy-enforcement) (10 scenarios)
3. [AC-P3: Security Policies](#ac-p3-security-policies) (6 scenarios)
4. [AC-P4: Quality Verification Scripts](#ac-p4-quality-verification-scripts) (5 scenarios)
5. [AC-P5: Documentation & Governance](#ac-p5-documentation--governance) (3 scenarios)
6. [Test Implementation Guide](#test-implementation-guide)

---

## AC-P1: Pre-commit Hooks

**Requirement**: Local quality gates catch issues before commit.

### Scenario 1.1: Pre-commit Hook Installation (Automatic)

```gherkin
Feature: Pre-commit Hook Installation
  As a contributor
  I want pre-commit hooks installed automatically
  So that I don't need manual setup

Scenario: Hooks install automatically on Nix shell entry
  Given I have cloned the rust-sitter repository
  And I have Nix installed with flakes enabled
  When I run "nix develop"
  Then I see "✅ Pre-commit hooks installed"
  And the file ".git/hooks/pre-commit" exists
  And the file ".git/hooks/commit-msg" exists
  And "pre-commit --version" shows a version number
```

**Acceptance Criteria**:
- Hook installation automated in Nix shellHook
- No manual intervention required
- Clear confirmation message

**Test Implementation**:
```bash
# Test: pre-commit installed automatically
test_auto_install_hooks() {
    cd /tmp/rust-sitter-clone
    nix develop --command bash -c "test -f .git/hooks/pre-commit"
    nix develop --command bash -c "pre-commit --version"
}
```

---

### Scenario 1.2: Formatting Hook Blocks Bad Code

```gherkin
Feature: Formatting Enforcement
  As a contributor
  I want unformatted code to be rejected
  So that all code follows consistent style

Scenario: Pre-commit blocks unformatted Rust code
  Given I am in the rust-sitter repository
  And pre-commit hooks are installed
  And I have modified "src/parser.rs" with unformatted code:
    """
    fn foo(){return 1;}
    """
  When I run "git add src/parser.rs"
  And I run "git commit -m 'feat: new parser'"
  Then the commit is blocked
  And I see the error message:
    """
    ❌ Cargo Format Check
    Run: cargo fmt
    """
  And "src/parser.rs" is not committed

Scenario: Pre-commit allows formatted code
  Given the same setup as above
  When I run "cargo fmt"
  And I run "git add src/parser.rs"
  And I run "git commit -m 'feat: new parser'"
  Then the commit succeeds
  And "src/parser.rs" is committed
```

**Acceptance Criteria**:
- Unformatted code blocked with clear error
- Error message includes remediation command
- Formatted code passes without issue

**Test Implementation**:
```bash
# Test: formatting hook
test_formatting_hook() {
    echo "fn foo(){return 1;}" > src/test.rs
    git add src/test.rs
    ! git commit -m "test" 2>&1 | grep "Cargo Format Check"

    cargo fmt
    git add src/test.rs
    git commit -m "test"  # Should succeed
}
```

---

### Scenario 1.3: Clippy Hook Catches Warnings

```gherkin
Feature: Linting Enforcement
  As a contributor
  I want clippy warnings to be caught early
  So that code quality is maintained

Scenario: Pre-commit blocks code with clippy warnings
  Given I have modified "src/tree.rs" with code that triggers clippy:
    """
    pub fn unused_function() {
        let x = 5;  // unused variable
    }
    """
  When I stage the changes
  And I attempt to commit
  Then the commit is blocked
  And I see the clippy warning:
    """
    ❌ Cargo Clippy
    warning: unused variable `x`
     --> src/tree.rs:2:9
      |
    2 |     let x = 5;
      |         ^

    Run: cargo clippy --fix
    """

Scenario: Clippy-clean code passes
  Given I fix the warning by removing the unused variable
  When I commit the changes
  Then the commit succeeds
  And clippy reports "0 warnings"
```

**Acceptance Criteria**:
- Clippy warnings block commit
- Warning details shown (file, line, message)
- Remediation command provided

**Test Implementation**:
```bash
# Test: clippy hook
test_clippy_hook() {
    echo 'pub fn test() { let x = 5; }' > src/test.rs  # Unused variable
    git add src/test.rs
    ! git commit -m "test" 2>&1 | grep "unused variable"

    echo 'pub fn test() { println!("ok"); }' > src/test.rs
    git add src/test.rs
    git commit -m "test"  # Should succeed
}
```

---

### Scenario 1.4: Test Connectivity Hook Prevents Disconnection

```gherkin
Feature: Test Connectivity Safeguards
  As a contributor
  I want disabled tests to be blocked
  So that no tests are silently disconnected

Scenario: Pre-commit blocks .rs.disabled files
  Given I have renamed a test file to "test_parser.rs.disabled"
  When I attempt to stage the file
  And I attempt to commit
  Then the commit is blocked
  And I see the error:
    """
    ❌ Test Connectivity Check
    Found .rs.disabled file: tests/test_parser.rs.disabled

    Use #[ignore] attribute instead of renaming.
    """

Scenario: #[ignore] attribute passes
  Given I use "#[ignore]" on the test instead
  And I restore the filename to "test_parser.rs"
  When I commit the changes
  Then the commit succeeds
  And the test connectivity check passes
```

**Acceptance Criteria**:
- .rs.disabled files blocked with clear error
- Error message suggests #[ignore] alternative
- Properly ignored tests pass

**Test Implementation**:
```bash
# Test: test connectivity hook
test_test_connectivity_hook() {
    mv tests/example.rs tests/example.rs.disabled
    git add tests/example.rs.disabled
    ! git commit -m "test" 2>&1 | grep "Test Connectivity Check"

    git reset HEAD tests/example.rs.disabled
    mv tests/example.rs.disabled tests/example.rs
}
```

---

### Scenario 1.5: Commit Message Validation

```gherkin
Feature: Commit Message Validation
  As a contributor
  I want commit messages to follow conventions
  So that commit history is consistent and searchable

Scenario: Invalid commit message blocked
  Given I have staged changes
  When I commit with message "added stuff"
  Then the commit is blocked
  And I see the error:
    """
    ❌ Commit message does not follow Conventional Commits format

    Expected format:
      <type>(<scope>): <subject>

    Types: feat, fix, docs, style, refactor, perf, test, build, ci, chore, revert

    Examples:
      feat(parser): add incremental parsing support
      fix(glr): resolve shift/reduce conflict in state 0
      docs: update README with Nix installation
    """

Scenario: Valid conventional commit passes
  Given I have the same staged changes
  When I commit with message "feat(parser): add incremental parsing"
  Then the commit succeeds
  And the message is validated
```

**Acceptance Criteria**:
- Conventional Commits format enforced
- Clear error with format guide and examples
- Valid formats pass without issue

**Test Implementation**:
```bash
# Test: commit message validation
test_commit_msg_validation() {
    echo "test" > file.txt
    git add file.txt
    ! git commit -m "added stuff" 2>&1 | grep "Conventional Commits"

    git commit -m "feat: add feature"  # Should succeed
}
```

---

### Scenario 1.6: Large File Prevention

```gherkin
Feature: Large File Prevention
  As a contributor
  I want to be warned about large files
  So that repository size is controlled

Scenario: Warning on large file (>1MB)
  Given I have created a 2MB test file
  When I attempt to stage and commit it
  Then I see a warning:
    """
    ⚠️  Large file detected: test_data.bin (2.1 MB)

    Consider:
      - Using Git LFS for binary files
      - Compressing the file
      - Hosting externally and referencing by URL

    Continue? (y/N)
    """
  And I can choose to proceed or cancel

Scenario: Small files pass without warning
  Given I have created a 100KB test file
  When I commit the file
  Then no large file warning is shown
  And the commit succeeds
```

**Acceptance Criteria**:
- Files >1MB trigger warning
- Warning includes size and mitigation options
- Small files pass silently

**Test Implementation**:
```bash
# Test: large file prevention
test_large_file_check() {
    dd if=/dev/zero of=large.bin bs=1M count=2  # 2MB file
    git add large.bin
    git commit -m "test" 2>&1 | grep "Large file detected"

    rm large.bin
}
```

---

### Scenario 1.7: Hook Execution Performance

```gherkin
Feature: Pre-commit Hook Performance
  As a contributor
  I want hooks to run quickly
  So that commits are not slowed down significantly

Scenario: Typical commit completes in <5 seconds
  Given I have modified 3 Rust files (typical change)
  And all code is properly formatted and linted
  When I stage and commit the changes
  Then the pre-commit hooks complete in <5 seconds
  And I see individual hook timing:
    """
    Cargo Format Check........Passed (0.8s)
    Cargo Clippy...............Passed (2.9s)
    Test Connectivity Check....Passed (0.3s)
    Commit Message Validation..Passed (0.1s)
    Large File Check...........Passed (0.2s)

    Total: 4.3s
    """

Scenario: Large changeset completes in <10 seconds
  Given I have modified 20 Rust files (large refactor)
  When I commit the changes
  Then the hooks complete in <10 seconds
```

**Acceptance Criteria**:
- Typical commit: <5 seconds
- Large commit: <10 seconds
- Timing breakdown shown

**Test Implementation**:
```bash
# Test: hook performance
test_hook_performance() {
    # Modify 3 files
    for i in 1 2 3; do
        echo "pub fn test$i() {}" > src/test$i.rs
        git add src/test$i.rs
    done

    START=$(date +%s.%N)
    git commit -m "feat: test"
    END=$(date +%s.%N)

    DURATION=$(echo "$END - $START" | bc)
    [ $(echo "$DURATION < 5.0" | bc) -eq 1 ]
}
```

---

### Scenario 1.8: Hook Bypass Detection

```gherkin
Feature: Hook Bypass Detection
  As a maintainer
  I want to detect when hooks are bypassed
  So that policies are still enforced in CI

Scenario: Bypass with --no-verify is caught in CI
  Given I have unformatted code
  When I commit with "git commit --no-verify -m 'feat: test'"
  Then the commit succeeds locally
  But when I push and CI runs
  Then the CI policy workflow fails
  And I see:
    """
    ❌ Formatting Check Failed

    Pre-commit hook was bypassed locally.
    All policies are enforced in CI.

    Fix: cargo fmt && git commit --amend
    """

Scenario: Properly committed code passes CI
  Given I commit without --no-verify
  And all hooks pass
  When CI runs
  Then all CI policy checks pass
```

**Acceptance Criteria**:
- --no-verify bypass detected in CI
- Clear message indicating bypass occurred
- Remediation steps provided

**Test Implementation**:
```bash
# Test: bypass detection
test_bypass_detection() {
    echo "fn bad(){}" > src/test.rs
    git add src/test.rs
    git commit --no-verify -m "feat: test"  # Bypass locally

    # Simulate CI run
    ! cargo fmt --check  # Should fail
}
```

---

## AC-P2: CI Policy Enforcement

**Requirement**: CI automatically enforces all quality policies.

### Scenario 2.1: Quality Gates Workflow Execution

```gherkin
Feature: CI Quality Gates
  As a maintainer
  I want all quality policies enforced in CI
  So that PRs cannot merge without meeting standards

Scenario: Quality gates workflow runs on PR
  Given a PR is opened with code changes
  When GitHub Actions triggers
  Then the "Policy Enforcement" workflow runs
  And the following jobs execute in parallel:
    | Job Name               | Purpose                          |
    | quality-gates          | Formatting, linting, tests, docs |
    | security-scanning      | Vulnerabilities, licenses        |
    | performance-gates      | Regression detection             |
    | test-connectivity      | No .rs.disabled files            |
  And each job reports status to the PR

Scenario: All gates pass - PR can merge
  Given all policy jobs pass
  Then the PR shows "✅ All checks have passed"
  And the merge button is enabled
  And maintainers can merge

Scenario: Any gate fails - PR blocked
  Given the quality-gates job fails (clippy warning)
  Then the PR shows "❌ Some checks were not successful"
  And the merge button is disabled
  And I see a clear failure summary
```

**Acceptance Criteria**:
- Workflow runs on all PRs
- All jobs run in parallel (fast feedback)
- Merge blocked if any job fails

**Test Implementation**:
```yaml
# .github/workflows/test-policy.yml
name: Test Policy Enforcement

on: [pull_request]

jobs:
  test-gates:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Verify workflow exists
        run: test -f .github/workflows/policy.yml
      - name: Verify all jobs defined
        run: |
          grep -q "quality-gates:" .github/workflows/policy.yml
          grep -q "security-scanning:" .github/workflows/policy.yml
          grep -q "performance-gates:" .github/workflows/policy.yml
```

---

### Scenario 2.2: Formatting Check in CI

```gherkin
Feature: CI Formatting Enforcement
  As a maintainer
  I want unformatted code to fail CI
  So that formatting is always consistent

Scenario: Unformatted code fails CI
  Given a PR with unformatted Rust code
  When the quality-gates job runs
  Then "cargo fmt --check" fails
  And the job fails with:
    """
    ❌ Formatting Check Failed

    The following files are not formatted:
      src/parser.rs
      src/tree.rs

    Run: cargo fmt
    Then: git commit --amend && git push --force
    """

Scenario: Formatted code passes CI
  Given a PR with properly formatted code
  When the quality-gates job runs
  Then "cargo fmt --check" passes
  And the formatting check is green ✅
```

**Test Implementation**:
```bash
# Test: CI formatting
test_ci_formatting() {
    # Intentionally create unformatted code
    echo "fn bad(){}" > src/test.rs
    git add src/test.rs
    git commit -m "test"

    # Simulate CI
    ! cargo fmt --all -- --check
}
```

---

### Scenario 2.3: Clippy Zero Warnings Enforcement

```gherkin
Feature: CI Clippy Enforcement
  As a maintainer
  I want zero clippy warnings in merged code
  So that code quality is consistently high

Scenario: Clippy warnings fail CI
  Given a PR with code that has clippy warnings
  When the quality-gates job runs
  Then "cargo clippy -- -D warnings" fails
  And the output shows each warning:
    """
    warning: unused variable `x`
     --> src/parser.rs:123:9
      |
    123 |     let x = 5;
        |         ^

    warning: needless borrow
     --> src/tree.rs:456:18
      |
    456 |     foo(&mut &bar)
        |                  ^^^ help: change this to: `bar`

    ❌ Found 2 warnings (threshold: 0)

    Run: cargo clippy --fix
    """

Scenario: Clippy-clean code passes CI
  Given a PR with zero clippy warnings
  When the job runs
  Then clippy passes with "0 warnings"
  And the check is green ✅
```

**Test Implementation**:
```bash
# Test: CI clippy
test_ci_clippy() {
    # Code with warning
    echo 'pub fn test() { let x = 5; }' > src/test.rs
    ! cargo clippy --all-targets -- -D warnings

    # Fix and verify
    echo 'pub fn test() {}' > src/test.rs
    cargo clippy --all-targets -- -D warnings  # Should pass
}
```

---

### Scenario 2.4: Test Pass Rate Enforcement

```gherkin
Feature: CI Test Enforcement
  As a maintainer
  I want 100% test pass rate
  So that broken code doesn't merge

Scenario: Failing tests block CI
  Given a PR with a failing test
  When the quality-gates job runs
  Then "cargo test" fails
  And the output shows:
    """
    test tree::test_edit ... FAILED

    failures:

    ---- tree::test_edit stdout ----
    thread 'tree::test_edit' panicked at 'assertion failed: `(left == right)`
      left: `5`,
     right: `6`', src/tree.rs:789:5

    test result: FAILED. 143 passed; 1 failed; 0 ignored

    ❌ Tests failed (143/144 passed)
    """

Scenario: All tests pass
  Given a PR with all tests passing
  When the job runs
  Then "cargo test" succeeds
  And the summary shows "144/144 passed" ✅
```

**Test Implementation**:
```bash
# Test: CI test enforcement
test_ci_tests() {
    # All tests should pass
    cargo test --workspace -- --test-threads=2
    echo "$?" | grep "0"
}
```

---

### Scenario 2.5: Documentation Warning Enforcement

```gherkin
Feature: CI Documentation Enforcement
  As a maintainer
  I want zero rustdoc warnings
  So that documentation is complete and correct

Scenario: Missing documentation fails CI
  Given a PR with undocumented public API
  When the quality-gates job runs
  Then "cargo doc" with RUSTDOCFLAGS=-D warnings fails
  And the output shows:
    """
    warning: missing documentation for a function
     --> src/parser.rs:45:1
      |
    45 | pub fn parse_incremental(...) -> Result<Tree> {
       | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^

    ❌ Documentation warnings found (threshold: 0)

    Add documentation:
      /// Parses input incrementally using old tree.
      pub fn parse_incremental(...) -> Result<Tree>
    """

Scenario: Fully documented code passes
  Given all public APIs have rustdoc comments
  When the job runs
  Then "cargo doc" passes with 0 warnings ✅
```

**Test Implementation**:
```bash
# Test: CI documentation
test_ci_docs() {
    RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --all-features
}
```

---

### Scenario 2.6: Security Vulnerability Scanning

```gherkin
Feature: CI Security Scanning
  As a maintainer
  I want vulnerabilities detected automatically
  So that insecure dependencies don't merge

Scenario: Dependency with CVE fails CI
  Given a PR that updates a dependency to a vulnerable version
  When the security-scanning job runs
  Then "cargo audit" detects the vulnerability
  And the job fails with:
    """
    ❌ Security Vulnerability Detected

    Crate:     tokio v1.20.0
    ID:        RUSTSEC-2023-0001
    CVE:       CVE-2023-12345
    Severity:  HIGH
    Title:     Data race in task spawning

    Solution:
      Update to tokio v1.21.0 or later:
        cargo update -p tokio

    See: https://rustsec.org/advisories/RUSTSEC-2023-0001
    """

Scenario: No vulnerabilities - scan passes
  Given a PR with no vulnerable dependencies
  When the job runs
  Then "cargo audit" reports "Success No vulnerable packages found" ✅
```

**Test Implementation**:
```bash
# Test: CI security scanning
test_ci_security() {
    cargo audit  # Should pass if no vulnerabilities
}
```

---

### Scenario 2.7: License Compliance Checking

```gherkin
Feature: CI License Compliance
  As a maintainer
  I want only approved licenses
  So that we avoid legal issues

Scenario: GPL dependency fails CI
  Given a PR adds a dependency with GPL-3.0 license
  When the security-scanning job runs
  Then "cargo deny check licenses" fails
  And the output shows:
    """
    ❌ License Violation Detected

    Crate:    some-gpl-crate v1.0.0
    License:  GPL-3.0
    Status:   DENIED (copyleft not allowed)

    Allowed licenses: MIT, Apache-2.0, BSD-3-Clause
    Denied licenses:  GPL-2.0, GPL-3.0, AGPL-3.0

    Action: Remove dependency or find alternative
    """

Scenario: MIT-licensed dependencies pass
  Given all dependencies use approved licenses
  When the job runs
  Then "cargo deny check" passes ✅
```

**Test Implementation**:
```bash
# Test: CI license compliance
test_ci_licenses() {
    cargo install cargo-deny
    cargo deny check licenses
}
```

---

### Scenario 2.8: Performance Regression Detection

```gherkin
Feature: CI Performance Gates
  As a maintainer
  I want performance regressions caught
  So that performance doesn't degrade

Scenario: >5% regression fails CI
  Given a PR that causes a 10% slowdown in parsing
  When the performance-gates job runs
  Then benchmarks are run for both base and PR branches
  And the comparison shows:
    """
    📊 Performance Comparison (PR vs base)

    ❌ Regressions Detected:

    Benchmark: parse_python_large
      Base:   125.3 ms ± 2.1 ms
      PR:     138.1 ms ± 2.3 ms
      Change: +10.2% (threshold: 5%)

    Benchmark: parse_javascript_medium
      Base:   45.2 ms ± 0.8 ms
      PR:     47.5 ms ± 0.9 ms
      Change: +5.1% (threshold: 5%)

    2 benchmarks regressed. Fix performance or justify regression.
    """

Scenario: <5% variance passes
  Given a PR with <5% performance impact
  Then the performance gates pass ✅
```

**Test Implementation**:
```bash
# Test: CI performance gates
test_ci_performance() {
    # Run benchmarks
    cargo bench --workspace -- --save-baseline pr

    # Compare (needs baseline from base branch)
    ./scripts/check-perf-regression.sh base pr 5
}
```

---

### Scenario 2.9: Test Connectivity in CI

```gherkin
Feature: CI Test Connectivity
  As a maintainer
  I want to detect disconnected tests
  So that test coverage doesn't silently decrease

Scenario: .rs.disabled file fails CI
  Given a PR renames a test file to .rs.disabled
  When the test-connectivity job runs
  Then the check for .rs.disabled files fails
  And the output shows:
    """
    ❌ Disconnected Tests Detected

    Found .rs.disabled files:
      tests/parser_test.rs.disabled
      tests/tree_test.rs.disabled

    Use #[ignore] attribute instead of renaming files.
    This ensures tests remain connected and visible.
    """

Scenario: Non-zero test counts verified
  Given a PR adds a new crate
  But the crate has 0 tests discovered
  When the job runs
  Then the test count verification fails
  And the output shows:
    """
    ❌ Zero Tests Detected

    Crate: rust-sitter-new-feature
    Tests: 0 (expected: >0)

    Either:
      - Add tests to the crate
      - Or document why tests are not needed
    """

Scenario: All tests connected
  Given all test files have .rs extension
  And all crates have non-zero test counts
  Then the test-connectivity check passes ✅
```

**Test Implementation**:
```bash
# Test: CI test connectivity
test_ci_test_connectivity() {
    # Check for .rs.disabled
    ! find . -name "*.rs.disabled" | grep -q .

    # Check test counts
    ./scripts/check-test-connectivity.sh
}
```

---

### Scenario 2.10: PR Status and Merge Blocking

```gherkin
Feature: PR Status Integration
  As a contributor
  I want clear feedback on policy status
  So that I know what needs fixing

Scenario: Policy failures show in PR checks
  Given a PR with formatting, clippy, and security issues
  When CI completes
  Then the PR shows:
    """
    ❌ Policy Enforcement — Failed
      ❌ quality-gates — Failed
        ❌ Formatting check
        ❌ Clippy check
        ✅ Tests
        ✅ Documentation
      ❌ security-scanning — Failed
        ❌ Cargo audit (1 vulnerability)
        ✅ Cargo deny
      ✅ performance-gates — Passed
      ✅ test-connectivity — Passed
    """
  And the merge button shows "Merging is blocked"
  And I see "2 failing checks"

Scenario: All policies pass - merge enabled
  Given a PR with all policies passing
  Then the PR shows "✅ All checks have passed"
  And the merge button is enabled
  And maintainers can approve and merge
```

**Test Implementation**:
```yaml
# .github/workflows/test-pr-status.yml
- name: Verify Branch Protection
  run: |
    # Ensure policy workflow is required for merge
    gh api repos/${{ github.repository }}/branches/main/protection \
      | jq -r '.required_status_checks.contexts[]' \
      | grep "Policy Enforcement"
```

---

## AC-P3: Security Policies

**Requirement**: Automated security scanning prevents vulnerabilities.

### Scenario 3.1: Cargo Audit Vulnerability Detection

```gherkin
Feature: Vulnerability Scanning
  As a maintainer
  I want known CVEs to be blocked
  So that we don't ship vulnerable code

Scenario: High severity CVE blocks PR
  Given a dependency has a known high-severity CVE
  When cargo audit runs in CI
  Then the scan fails
  And the output includes:
    """
    Crate:     openssl v0.10.45
    ID:        RUSTSEC-2023-0072
    CVE:       CVE-2023-12345
    Severity:  HIGH
    Title:     Memory corruption in SSL handshake
    Date:      2023-08-15

    Solution:
      Upgrade to openssl v0.10.46 or later
        cargo update -p openssl

    URL: https://rustsec.org/advisories/RUSTSEC-2023-0072
    """
  And the PR is blocked

Scenario: Informational advisory passes with warning
  Given a dependency has an informational advisory
  When cargo audit runs
  Then the scan passes
  But a warning is shown for visibility
```

**Test Implementation**:
```bash
# Test: vulnerability detection
test_vulnerability_scan() {
    cargo audit
    # Should pass if no vulns, fail if vulns found
}
```

---

### Scenario 3.2: License Compliance Enforcement

```gherkin
Feature: License Compliance
  As a maintainer
  I want only approved licenses
  So that we comply with legal requirements

Scenario: Copyleft license blocked
  Given a PR adds dependency with GPL-3.0 license
  When cargo deny runs
  Then the check fails
  And the output shows:
    """
    ❌ License Denied: GPL-3.0

    Crate:   some-crate v1.0.0
    License: GPL-3.0
    Reason:  Copyleft licenses not allowed

    Allowed: MIT, Apache-2.0, BSD-3-Clause, ISC, Zlib
    Denied:  GPL-2.0, GPL-3.0, AGPL-3.0

    Find alternative with approved license.
    """

Scenario: Unlicensed crate blocked
  Given a dependency has no license
  Then cargo deny fails
  And suggests contacting the author

Scenario: MIT license passes
  Given all dependencies use approved licenses
  Then cargo deny passes ✅
```

**Test Implementation**:
```bash
# Test: license compliance
test_license_compliance() {
    cargo install cargo-deny
    cargo deny check licenses
}
```

---

### Scenario 3.3: Multiple Version Detection

```gherkin
Feature: Multiple Version Detection
  As a maintainer
  I want to minimize duplicate dependencies
  So that binary size and build time are optimized

Scenario: Multiple versions warning
  Given the dependency tree has 2 versions of syn
  When cargo deny runs
  Then a warning is shown:
    """
    ⚠️  Multiple Versions Detected

    Crate: syn
    Versions:
      - syn v1.0.109 (used by 12 crates)
      - syn v2.0.15 (used by 3 crates)

    Consider:
      - Update to single version if possible
      - Or accept if unavoidable (transitive deps)

    Impact: +450 KB binary size, +2s build time
    """
  But the check passes (warning only, not blocking)
```

**Test Implementation**:
```bash
# Test: multiple versions
test_multiple_versions() {
    cargo deny check bans 2>&1 | grep -i "multiple versions" || true
}
```

---

### Scenario 3.4: Secret Detection

```gherkin
Feature: Secret Detection
  As a maintainer
  I want credentials blocked from commits
  So that secrets don't leak

Scenario: API key detected in code
  Given a PR adds code with an API key:
    """
    const API_KEY: &str = "sk_live_1234567890abcdef";
    """
  When the secret detection workflow runs (TruffleHog)
  Then the scan detects the secret
  And the output shows:
    """
    ❌ Secret Detected

    Type:     API Key (Stripe)
    File:     src/config.rs
    Line:     23
    Match:    sk_live_********************

    Action:
      1. Remove secret from code
      2. Rotate the credential
      3. Use environment variable instead
      4. Add to .gitignore if config file
    """

Scenario: No secrets in code
  Given a PR with no hardcoded credentials
  Then the secret detection passes ✅
```

**Test Implementation**:
```yaml
# .github/workflows/secrets.yml
- name: TruffleHog Scan
  uses: trufflesecurity/trufflehog@main
  with:
    path: ./
```

---

### Scenario 3.5: SBOM Generation

```gherkin
Feature: Software Bill of Materials
  As a maintainer
  I want SBOM generated automatically
  So that we have supply chain visibility

Scenario: SBOM generated on release
  Given a new version is tagged
  When the release workflow runs
  Then cargo-sbom generates SBOM
  And the output includes:
    """
    {
      "bomFormat": "CycloneDX",
      "specVersion": "1.4",
      "version": 1,
      "components": [
        {
          "type": "library",
          "name": "rust-sitter",
          "version": "0.9.0",
          "licenses": ["MIT"]
        },
        {
          "type": "library",
          "name": "tokio",
          "version": "1.28.0",
          "licenses": ["MIT"]
        },
        ...
      ]
    }
    """
  And the SBOM is attached to the release

Scenario: SBOM uploaded to artifact storage
  Then the SBOM is available at:
    https://github.com/EffortlessMetrics/rust-sitter/releases/download/v0.9.0/sbom.json
```

**Test Implementation**:
```bash
# Test: SBOM generation
test_sbom_generation() {
    cargo install cargo-sbom
    cargo sbom > sbom.json
    jq '.components | length' sbom.json  # Should have entries
}
```

---

### Scenario 3.6: Security Advisory Monitoring

```gherkin
Feature: Security Advisory Monitoring
  As a maintainer
  I want to be notified of new vulnerabilities
  So that we can respond quickly

Scenario: Dependabot creates PR for vulnerability
  Given a new CVE is published for a dependency
  When Dependabot checks (daily)
  Then a PR is automatically created:
    """
    Title: Bump tokio from 1.28.0 to 1.28.1

    Bumps tokio from 1.28.0 to 1.28.1.

    **Security Advisory**
    CVE-2023-XXXX: High severity vulnerability in tokio

    This PR fixes:
    - RUSTSEC-2023-0072

    Release notes: https://github.com/tokio-rs/tokio/releases/tag/tokio-1.28.1
    """
  And maintainers are notified
  And the PR can be reviewed and merged
```

**Test Implementation**:
```yaml
# .github/dependabot.yml
version: 2
updates:
  - package-ecosystem: cargo
    directory: "/"
    schedule:
      interval: daily
    open-pull-requests-limit: 10
```

---

## AC-P4: Quality Verification Scripts

**Requirement**: Local scripts enable self-service quality validation.

### Scenario 4.1: check-quality.sh Comprehensive Validation

```gherkin
Feature: Local Quality Check
  As a contributor
  I want to validate quality locally
  So that I can fix issues before pushing

Scenario: All quality checks pass
  Given I have clean, well-formatted code
  When I run "./scripts/check-quality.sh"
  Then I see:
    """
    🔍 Checking Quality...

    ✅ Formatting (cargo fmt): PASS
    ✅ Clippy (cargo clippy): PASS
    ✅ Tests (cargo test): PASS (144 tests)
    ✅ Documentation (cargo doc): PASS
    ✅ Test Connectivity: PASS

    🎉 All quality checks passed!
    """
  And the script exits with code 0

Scenario: Quality checks find issues
  Given I have unformatted code and clippy warnings
  When I run the script
  Then I see:
    """
    🔍 Checking Quality...

    ❌ Formatting: FAIL
       Run: cargo fmt
    ❌ Clippy: FAIL (3 warnings)
       src/parser.rs:123: unused variable 'x'
       src/tree.rs:456: needless borrow
       src/edit.rs:789: redundant clone
       Run: cargo clippy --fix
    ⏭️  Tests: SKIPPED (fix above first)

    ❌ Quality checks failed.
    """
  And the script exits with code 1
  And subsequent checks are skipped (fail-fast)
```

**Test Implementation**:
```bash
# Test: check-quality.sh
test_check_quality_script() {
    ./scripts/check-quality.sh
    [ $? -eq 0 ]  # Should pass on clean code
}
```

---

### Scenario 4.2: check-security.sh Security Validation

```gherkin
Feature: Local Security Check
  As a contributor
  I want to validate security locally
  So that I can fix issues before pushing

Scenario: All security checks pass
  Given I have no vulnerable dependencies
  When I run "./scripts/check-security.sh"
  Then I see:
    """
    🔒 Checking Security...

    🔍 Vulnerability Scan (cargo audit): PASS
    📜 License Compliance (cargo deny): PASS
    🔐 Secret Detection: PASS

    🎉 All security checks passed!
    """
  And the script exits with code 0

Scenario: Vulnerability detected
  Given I have a dependency with a known CVE
  When I run the script
  Then I see:
    """
    🔒 Checking Security...

    ❌ Vulnerability Scan: FAIL

       Crate:     tokio v1.20.0
       CVE:       CVE-2023-XXXX
       Severity:  HIGH

       Run: cargo update -p tokio

    ❌ Security checks failed.
    """
  And the script exits with code 1
```

**Test Implementation**:
```bash
# Test: check-security.sh
test_check_security_script() {
    ./scripts/check-security.sh
    [ $? -eq 0 ]
}
```

---

### Scenario 4.3: pre-push.sh Comprehensive Pre-push Validation

```gherkin
Feature: Pre-push Validation
  As a contributor
  I want comprehensive validation before pushing
  So that I don't waste CI time on preventable failures

Scenario: All validations pass
  Given I have committed clean, tested code
  When I run "./scripts/pre-push.sh"
  Then quality checks run and pass
  And security checks run and pass
  And I see:
    """
    🚀 Pre-Push Validation...

    [... quality check output ...]
    [... security check output ...]

    ✅ Pre-push validation passed!
    Safe to push.
    """
  And the script exits with code 0

Scenario: Pushing to main triggers warning
  Given I am on the main branch
  When I run the script
  Then I see:
    """
    ⚠️  WARNING: Pushing directly to main
    Consider using a feature branch instead.
    Continue? (y/N)
    """
  And I can choose to proceed or abort
```

**Test Implementation**:
```bash
# Test: pre-push.sh
test_pre_push_script() {
    ./scripts/pre-push.sh
    [ $? -eq 0 ]
}
```

---

### Scenario 4.4: Script Performance

```gherkin
Feature: Verification Script Performance
  As a contributor
  I want scripts to run quickly
  So that local validation is practical

Scenario: check-quality.sh completes in <30 seconds
  Given a typical codebase state
  When I run "./scripts/check-quality.sh"
  Then the script completes in <30 seconds
  And I see timing breakdown:
    """
    Formatting:       0.8s
    Clippy:           12.3s
    Tests:            14.2s
    Documentation:    1.5s
    Test Connectivity: 0.3s

    Total: 29.1s
    """

Scenario: check-security.sh completes in <10 seconds
  Given cargo audit database is cached
  When I run "./scripts/check-security.sh"
  Then the script completes in <10 seconds
```

**Test Implementation**:
```bash
# Test: script performance
test_script_performance() {
    START=$(date +%s)
    ./scripts/check-quality.sh
    END=$(date +%s)
    DURATION=$((END - START))
    [ $DURATION -lt 30 ]
}
```

---

### Scenario 4.5: Git Hook Integration (Optional)

```gherkin
Feature: Git Hook Integration
  As a contributor
  I want to optionally use scripts as git hooks
  So that validation runs automatically

Scenario: Install pre-push hook
  Given I want automatic pre-push validation
  When I run "./scripts/install-hooks.sh"
  Then the pre-push hook is installed
  And I see:
    """
    ✅ Installed pre-push hook

    The following validations will run on git push:
      - Quality checks (formatting, clippy, tests)
      - Security checks (audit, deny)
      - Branch name check

    To skip: git push --no-verify
    """

Scenario: Pre-push hook runs automatically
  Given the hook is installed
  When I run "git push origin my-branch"
  Then "./scripts/pre-push.sh" runs automatically
  And if it fails, the push is aborted
```

**Test Implementation**:
```bash
# Test: hook installation
test_hook_installation() {
    ./scripts/install-hooks.sh
    test -f .git/hooks/pre-push
    grep -q "pre-push.sh" .git/hooks/pre-push
}
```

---

## AC-P5: Documentation & Governance

**Requirement**: Clear policy documentation and governance processes.

### Scenario 5.1: Policy Documentation Clarity

```gherkin
Feature: Policy Documentation
  As a contributor
  I want clear policy documentation
  So that I understand requirements and remediation

Scenario: New contributor reads POLICIES.md
  Given I am a new contributor
  When I read "POLICIES.md"
  Then I understand:
    | Policy                  | What                     | Why                        | How                      |
    | Formatting              | cargo fmt required       | Consistency                | Run: cargo fmt           |
    | Linting                 | Zero clippy warnings     | Code quality               | Run: cargo clippy --fix  |
    | Test Pass Rate          | 100% pass rate           | Reliability                | Run: cargo test          |
    | Vulnerability Scanning  | No known CVEs            | Security                   | Run: cargo update        |
    | License Compliance      | Approved licenses only   | Legal compliance           | Check: cargo deny        |
    | Performance             | <5% regression           | User experience            | Run: cargo bench         |
  And I see clear examples of violations and fixes
  And I know how to request exceptions

Scenario: Contributor finds remediation quickly
  Given I receive a policy failure
  When I search POLICIES.md for the error
  Then I find the relevant section
  And I see step-by-step remediation
  And I understand why the policy exists
```

**Test Implementation**:
```bash
# Test: documentation completeness
test_documentation_completeness() {
    grep -q "Formatting" POLICIES.md
    grep -q "Linting" POLICIES.md
    grep -q "Security" POLICIES.md
    grep -q "Override Procedures" POLICIES.md
}
```

---

### Scenario 5.2: Override Request Process

```gherkin
Feature: Policy Override Process
  As a contributor
  I want a clear process for exceptions
  So that I can proceed when policies are too strict

Scenario: Request override for false positive
  Given cargo deny flags an acceptable dependency
  When I need to override the policy
  Then I follow the process:
    """
    1. Open GitHub issue using template:
       .github/ISSUE_TEMPLATE/policy-override.md

    2. Provide details:
       - Policy violated: License Compliance
       - Dependency: some-crate v1.0.0
       - License: BSD-2-Clause (not in allowed list)
       - Justification: BSD-2-Clause compatible with MIT
       - Mitigation: Legal review confirms compatibility

    3. Get approval from 2+ maintainers

    4. Add exception to deny.toml:
       [licenses]
       allow = [..., "BSD-2-Clause"]  # Approved via issue #123

    5. Document in PR description
    """
  And maintainers review and approve
  And the exception is tracked

Scenario: Override for intentional violation
  Given I need to use a GPL library (accepted trade-off)
  Then the same process applies
  And justification explains business decision
  And legal approval is documented
```

**Test Implementation**:
```bash
# Test: override template exists
test_override_template() {
    test -f .github/ISSUE_TEMPLATE/policy-override.md
    grep -q "Policy violated" .github/ISSUE_TEMPLATE/policy-override.md
}
```

---

### Scenario 5.3: Policy Evolution and Versioning

```gherkin
Feature: Policy Evolution
  As a maintainer
  I want policies to evolve
  So that we can improve over time

Scenario: Policy v1.1 introduces code coverage
  Given Policy v1.0 is established
  When we decide to add code coverage enforcement
  Then the process is:
    """
    1. Draft ADR-0011: Code Coverage Policy
       - Rationale: Increase test quality
       - Threshold: 80% coverage
       - Exemptions: Generated code

    2. Update POLICY_AS_CODE_CONTRACT.md to v1.1
       - Add AC-P6: Code Coverage Enforcement

    3. Implement gradually:
       - Week 1: Measure baseline (informational)
       - Week 2: Warn on <80% (not blocking)
       - Week 3: Block on <80% (enforced)

    4. Update POLICIES.md with new policy

    5. Announce to team with migration guide
    """
  And the team has time to adapt
  And existing code is grandfathered

Scenario: Policy threshold tuned based on data
  Given we have 6 months of metrics
  And false positive rate is 15% (too high)
  When we review the policy
  Then we adjust the threshold:
    - Performance regression: 5% → 7% (fewer false alarms)
  And document the change in ADR-0010 update
  And communicate to team
```

**Test Implementation**:
```bash
# Test: policy versioning
test_policy_versioning() {
    grep -q "Version:" POLICIES.md
    grep -q "Last Updated:" POLICIES.md
}
```

---

## Test Implementation Guide

### Test Organization

**Directory Structure**:
```
tests/
├── policy/
│   ├── pre_commit_test.rs       # AC-P1 scenarios
│   ├── ci_policy_test.rs        # AC-P2 scenarios
│   ├── security_policy_test.rs  # AC-P3 scenarios
│   ├── verification_scripts_test.rs # AC-P4 scenarios
│   └── documentation_test.rs    # AC-P5 scenarios
└── integration/
    └── end_to_end_policy_test.rs  # Full workflow tests
```

### Test Execution

**Running BDD Tests**:
```bash
# Run all policy tests
cargo test --package rust-sitter --test policy

# Run specific AC tests
cargo test --test pre_commit_test
cargo test --test ci_policy_test
cargo test --test security_policy_test

# Run with output
cargo test --test policy -- --nocapture

# Run in CI (concurrency-capped)
RUST_TEST_THREADS=2 cargo test --test policy
```

### Test Data

**Fixtures** (`tests/fixtures/policy/`):
```
fixtures/policy/
├── unformatted_code.rs           # For formatting tests
├── clippy_warnings.rs            # For linting tests
├── vulnerable_Cargo.toml         # For security tests
├── gpl_dependency_Cargo.toml     # For license tests
└── slow_benchmark.rs             # For performance tests
```

### Mocking CI Environment

```bash
# Simulate CI locally
export CI=true
export GITHUB_ACTIONS=true
export GITHUB_EVENT_NAME=pull_request

# Run policy workflow locally (act)
act pull_request -W .github/workflows/policy.yml
```

### Test Coverage Tracking

```bash
# Generate coverage report for policy tests
cargo tarpaulin --test policy --out Html

# Verify all BDD scenarios have tests
./scripts/verify-bdd-coverage.sh
```

---

## Success Criteria

Policy BDD scenarios are **complete** when:

1. ✅ All 32 scenarios implemented as executable tests
2. ✅ All tests pass in CI
3. ✅ Test coverage >95% for policy code
4. ✅ Documentation links scenarios to code
5. ✅ Scenarios validated by external review

---

## Appendix: Scenario Mapping

| AC    | Scenario | Test File | Status |
|-------|----------|-----------|--------|
| AC-P1 | 1.1 - 1.8 | `pre_commit_test.rs` | Planned |
| AC-P2 | 2.1 - 2.10 | `ci_policy_test.rs` | Planned |
| AC-P3 | 3.1 - 3.6 | `security_policy_test.rs` | Planned |
| AC-P4 | 4.1 - 4.5 | `verification_scripts_test.rs` | Planned |
| AC-P5 | 5.1 - 5.3 | `documentation_test.rs` | Planned |

---

**Version**: 1.0.0
**Last Updated**: 2025-11-20
**Next Review**: After Phase 1B implementation
**Owner**: rust-sitter core team

---

END OF BDD SPECIFICATIONS
