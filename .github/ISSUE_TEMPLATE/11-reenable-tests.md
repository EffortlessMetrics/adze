---
name: "Re-enable/modernize disabled tests"
about: "Port disabled tests to new APIs and re-enable"
title: "[TASK] Re-enable/modernize disabled tests"
labels: "testing, technical-debt"
assignees: ""
---

## Overview
8 test files disabled due to API changes. Need porting and re-enabling.

## Test Files to Re-enable

### Immediate (unblocked)
- [ ] `test_pure_rust_e2e.rs.disabled` - Port to unified_parser API
- [ ] `golden_tests.rs.disabled` - Update snapshot format
- [ ] `test_pure_rust_real_grammar.rs.disabled` - Fix parser initialization
- [ ] `parser_bench.rs.disabled` - Update benchmark harness

### After GLR fixes
- [ ] `test_glr_parsing.rs.disabled` - Update for new GLR API

### After field names (#1)
- [ ] `test_query_predicates.rs.disabled` - Needs field support

### After query engine (#10)
- [ ] `query_demo.rs.disabled` - Full query system needed

### After multiple features
- [ ] `test_complete_example.rs.disabled` - Needs fields + queries + incremental

## Porting Checklist

For each test file:
- [ ] Update imports to new module structure
- [ ] Replace old Parser with unified_parser::Parser
- [ ] Update API calls (parse → parse_utf8, etc.)
- [ ] Fix assertions for new tree structure
- [ ] Add feature gates if needed
- [ ] Remove `.disabled` suffix

## CI Integration
- [ ] Add matrix job: `--features "incremental_glr,queries,serialization"`
- [ ] Separate miri job for external scanners
- [ ] Coverage reporting

## Acceptance Criteria
- [x] All non-blocked tests passing
- [x] CI runs all test configurations
- [x] No test regressions
- [x] Coverage ≥ 80%

## Files to Modify
- `runtime/tests/*.rs.disabled` - Remove suffix after porting
- `.github/workflows/ci.yml` - Add feature matrix
- `runtime/Cargo.toml` - Update test dependencies