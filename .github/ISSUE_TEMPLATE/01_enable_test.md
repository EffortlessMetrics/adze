---
name: 🧪 Re-enable Ignored Test
about: Pick up a task to re-enable an ignored test
title: 'Re-enable test: [TEST_NAME]'
labels: ['good first issue', 'testing', 'v0.7.0']
assignees: ''
---

## Test Information

**Test Name**: `test_name_here`
**File**: `path/to/test/file.rs`
**Estimated Time**: X hours
**From**: [GAPS.md - Ignored Tests](../../../GAPS.md#ignored-tests-20-total)

## What This Test Checks

<!-- Describe what this test validates -->

## Why It's Currently Ignored

<!-- Explain why the test is currently ignored -->

## How to Fix

<!-- Step-by-step guidance from GAPS.md -->

1. Run the test: `cargo test test_name -- --ignored --nocapture`
2. Analyze the failure
3. Fix the underlying issue (usually in `file/path.rs`)
4. Remove the `#[ignore]` attribute
5. Verify test passes consistently

## Acceptance Criteria

- [ ] Test passes without `#[ignore]` attribute
- [ ] No new clippy warnings introduced
- [ ] Error messages are helpful and accurate (if applicable)
- [ ] Changes are minimal and focused

## Implementation Notes

<!-- Any additional context, code snippets, or guidance -->

## Related Issues

<!-- Link to related issues or PRs -->

---

**Ready to work on this?** Comment below to claim it!
**Need help?** Ask questions in this issue or in [GitHub Discussions](../../../discussions)
**See also**: [CONTRIBUTING.md](../../../CONTRIBUTING.md) for development setup
