---
name: Policy Override Request
about: Request an exception to an automated policy check
title: 'policy: [Brief description of override needed]'
labels: ['policy-override', 'needs-review']
assignees: []
---

## Policy Override Request

**Policy being overridden**:
<!-- Select one or more: -->
- [ ] Code formatting (cargo fmt)
- [ ] Clippy warning
- [ ] Test failure
- [ ] Documentation warning
- [ ] Vulnerability (cargo audit)
- [ ] License compliance (cargo deny)
- [ ] Secret detection
- [ ] Performance regression
- [ ] Other: _____________

**Affected components**:
<!-- List files, modules, or crates affected -->

**Duration**:
- [ ] Temporary (expected fix date: YYYY-MM-DD)
- [ ] Permanent (requires architectural decision)

---

## Justification

### Technical Rationale
<!-- Explain WHY this override is necessary. Include:
- What problem are you solving?
- Why can't you fix the underlying issue?
- What alternatives did you consider?
-->

### Use Case
<!-- Describe the specific use case or scenario where this override is needed -->

### Risk Assessment
<!-- What are the risks of approving this override? -->

---

## Mitigation Plan

### Alternative Protections
<!-- What steps will you take to mitigate the risk? Examples:
- Manual review process
- Additional testing
- Documentation updates
- Code comments explaining the exception
-->

### Monitoring
<!-- How will you ensure this doesn't cause issues? Examples:
- Additional CI checks
- Runtime assertions
- Periodic review
-->

---

## Implementation Details

### Configuration Changes
<!-- Show the exact configuration changes needed. Examples:

For cargo-audit (audit.toml):
```toml
[advisories]
ignore = [
    "RUSTSEC-2023-0001",  # Brief reason
    # Override: Issue #XXX
    # Justification: ...
    # Mitigation: ...
    # Expected fix: YYYY-MM-DD
]
```

For clippy (in code):
```rust
#[allow(clippy::lint_name)]  // Override: Issue #XXX - Reason
fn example() { ... }
```

For cargo-deny (deny.toml):
```toml
[[licenses.exceptions]]
allow = ["LicenseRef-Custom"]
name = "crate-name"
# Override: Issue #XXX
# Justification: ...
```
-->

### Documentation Updates
<!-- List documentation that needs updating:
- [ ] Update POLICIES.md with exception
- [ ] Add code comments explaining override
- [ ] Update CONTRIBUTING.md if workflow changes
- [ ] Update ADR if architectural impact
-->

---

## Review Checklist

**For Maintainers**:

- [ ] Justification is technically sound
- [ ] Mitigation plan is adequate
- [ ] Risk is acceptable
- [ ] No better alternative exists
- [ ] Documentation is complete
- [ ] Configuration changes are correct
- [ ] Review period is appropriate (temporary) or ADR created (permanent)

**Approval Requirements**:
- Temporary override: 1 maintainer approval
- Permanent override: 2 maintainer approvals + ADR

---

## Additional Context

<!-- Add any other context, screenshots, logs, or references that support this request -->

**Related Issues/PRs**:
<!-- Link to related work -->

**References**:
- [POLICIES.md](../../POLICIES.md)
- [ADR-0010: Policy-as-Code](../../docs/adr/ADR-0010-POLICY-AS-CODE.md)
- [Policy Enforcement Guide](../../docs/guides/POLICY_ENFORCEMENT.md)
