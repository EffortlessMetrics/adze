# ADR-007: BDD Framework for Parser Testing

## Status

Accepted

## Context

Parser development, particularly for GLR (Generalized LR) parsers, involves complex interactions between:
- Parse table generation with shift/reduce conflicts
- Runtime fork/merge operations on multiple parse stacks
- Incremental parsing with tree reuse heuristics
- Performance contracts that must be maintained across changes

Traditional unit testing approaches have limitations for these scenarios:
1. **Combinatorial explosion**: Testing all interaction paths is impractical
2. **Unclear intent**: Test names like `test_glr_conflict_3` don't communicate purpose
3. **Brittle tests**: Implementation changes break tests even when behavior is correct
4. **Poor documentation**: Tests don't serve as executable specifications

The project faced specific challenges with GLR conflict preservation:
- Conflicts must be preserved (not resolved away) for true GLR behavior
- Precedence ordering affects which parse tree is preferred
- Multi-action cells in parse tables require runtime forking

### Alternatives Considered

1. **Traditional unit tests only**: Already in use but insufficient for complex scenarios
2. **Property-based testing (proptest)**: Useful but doesn't capture behavioral intent
3. **Snapshot testing only**: Good for regression but poor for specification
4. **BDD framework**: Gherkin-style scenarios that serve as documentation and tests

## Decision

We adopted a **Behavior-Driven Development (BDD) framework** using Gherkin-style scenarios to drive development across:

1. **GLR conflict preservation**: Scenarios for shift/reduce conflict detection and ordering
2. **Incremental parsing**: Scenarios for tree reuse and invalidation heuristics
3. **Performance contracts**: Scenarios defining acceptable performance characteristics

### BDD Scenario Structure

Each scenario follows the Given-When-Then pattern:

```gherkin
Feature: GLR Conflict Detection and Preservation

  Scenario: Preserve Conflicts with Precedence Ordering (PreferShift)
    Given a shift/reduce conflict with precedence favoring shift
    When resolve_shift_reduce_conflict() is called
    Then both actions are preserved in order [shift, reduce]
    And the first action (shift) has higher runtime priority
```

### Implementation Approach

The BDD framework is implemented through:
- **Scenario files**: `docs/archive/plans/BDD_GLR_CONFLICT_PRESERVATION.md`
- **Test mapping**: Each scenario maps to acceptance criteria checkboxes
- **Living documentation**: Scenarios are updated as behavior evolves

### Key Benefits Observed

The BDD approach delivered **60% time savings** compared to traditional test-driven development:
- **Clearer requirements**: Scenarios forced explicit behavior specification
- **Faster debugging**: Failures point to specific behavioral expectations
- **Better communication**: Non-developers can understand parser behavior
- **Regression prevention**: Scenarios capture intent, not implementation

## Consequences

### Positive

- **Documentation as tests**: Scenarios serve dual purpose as specification and verification
- **Faster development cycles**: Clear acceptance criteria reduce back-and-forth
- **Improved communication**: Gherkin syntax is accessible to non-developers
- **Better coverage**: BDD thinking exposes edge cases that unit tests miss
- **60% time savings**: Reported efficiency gain from structured approach
- **Executable specifications**: Scenarios can be traced to test implementations

### Negative

- **Learning curve**: Team members need to learn Gherkin syntax and BDD mindset
- **Additional abstraction**: One more layer between code and tests
- **Maintenance overhead**: Scenarios must be kept in sync with implementation
- **Tooling gaps**: No direct Gherkin-to-Rust test automation (manual mapping)

### Neutral

- **Hybrid approach**: BDD complements, not replaces, unit and property tests
- **Documentation location**: Scenarios live in `docs/archive/plans/` for historical reasons
- **Acceptance criteria format**: Uses checkboxes in markdown rather than automated test generation

## Related

- Related ADRs: [ADR-001](001-pure-rust-glr-implementation.md), [ADR-005](005-incremental-parsing-architecture.md)
- Reference: [docs/archive/plans/BDD_GLR_CONFLICT_PRESERVATION.md](../archive/plans/BDD_GLR_CONFLICT_PRESERVATION.md)
- Reference: [docs/archive/specs/BASELINE_MANAGEMENT_SPEC.md](../archive/specs/BASELINE_MANAGEMENT_SPEC.md) - BDD scenarios for performance
