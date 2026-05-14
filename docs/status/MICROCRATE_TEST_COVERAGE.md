# Microcrate Test Coverage Analysis

**Generated:** 2026-03-26
**Last Updated:** 2026-03-27
**Total Crates:** 4

## Summary

| Category | Count | Percentage |
|----------|-------|------------|
| Complete (BDD + Property) | 4 | 100% |
| Contract Lock Tests | 30+ | 95%+ |

All 4 remaining tracked microcrates have comprehensive test coverage with both BDD tests and property-based tests.

## Complete Coverage (BDD + Property Tests)

All 4 remaining tracked crates have both BDD tests and property-based tests:

| Crate | BDD File | Property File | Contract Lock |
|-------|----------|---------------|---------------|
| `bdd-governance-core` | ✓ | ✓ | ✓ |
| `linecol-core` | ✓ | ✓ | ✓ |
| `parsetable-metadata` | ✓ | ✓ | ✓ |
| `ts-c-harness` | ✓ | ✓ | - |

## Contract Lock Files

The following 30+ remaining tracked crates have `contract_lock.rs` test files (contract verification):

- `bdd-governance-core`
- `linecol-core`
- `parsetable-metadata`

### Crates Without Contract Lock Tests

The following crates do not have contract lock tests (by design):

- `ts-c-harness` - FFI test harness (excluded from workspace)

## Test Coverage Milestones

| Date | Milestone |
|------|-----------|
| 2026-03-26 | Initial coverage analysis (20 complete, 23 partial, 4 missing) |
| 2026-03-27 | **100% BDD + Property coverage achieved** - All 47 crates now have both test types |
| 2026-03-27 | Contract lock tests expanded to 45+ crates |

## Overlapping Responsibilities Analysis

### Potential Consolidation Opportunities

1. **Runtime Governance Crates:**
   The runtime governance facade/matrix stack has been collapsed into runtime owner modules and `bdd-governance-core::runtime`.

2. **Concurrency Init Crates:**
   Remaining initialization crates should keep classifier and bootstrap helpers as SRP owner submodules instead of standalone migration targets.

## Documentation Status

All crates have proper module-level documentation (`//!` comments) except:

| Crate | Status |
|-------|--------|
| `ts-c-harness` | Missing documentation |

This is acceptable as `ts-c-harness` is an FFI test harness (excluded from workspace).

## Next Steps

1. ✅ ~~Add property tests to high-priority crates missing them~~ - **COMPLETE**
2. ✅ ~~Add BDD + Property tests to crates with no coverage~~ - **COMPLETE**
3. Review overlapping crates for potential consolidation
4. ✅ Documentation check complete - all workspace crates documented
5. Consider adding contract lock tests to fixture crates if applicable
