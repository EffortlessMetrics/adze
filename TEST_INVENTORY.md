# Test Inventory & Categorization Schema

**Purpose**: Systematic inventory of all tests to enable confident iteration and prevent regressions.

**Last Updated**: 2025-11-19
**Version**: v0.6.1-beta

---

## Test Categories (Schema)

### Category Definitions

```yaml
test_categories:
  - id: UNIT
    name: "Unit Tests"
    description: "Test individual functions/modules in isolation"
    confidence: "HIGH"
    coverage_target: 90%

  - id: INTEGRATION
    name: "Integration Tests"
    description: "Test interactions between components"
    confidence: "HIGH"
    coverage_target: 85%

  - id: E2E
    name: "End-to-End Tests"
    description: "Full workflow from grammar definition to parsing"
    confidence: "MEDIUM"
    coverage_target: 75%

  - id: PROPERTY
    name: "Property-Based Tests"
    description: "Generative testing with random inputs"
    confidence: "HIGH"
    coverage_target: 50%

  - id: REGRESSION
    name: "Regression Tests"
    description: "Tests for previously fixed bugs"
    confidence: "HIGH"
    coverage_target: 100%

  - id: SNAPSHOT
    name: "Snapshot Tests"
    description: "Output comparison tests (using insta)"
    confidence: "MEDIUM"
    coverage_target: 80%

test_states:
  - PASSING: "Test passes consistently"
  - IGNORED: "Test disabled with #[ignore], documented reason"
  - FLAKY: "Test sometimes fails, needs investigation"
  - BROKEN: "Test fails consistently, needs fix"
  - MISSING: "Test should exist but doesn't"
```

---

## Current Test Inventory

### ✅ Locked-In Tests (Passing)

#### Macro Generation (`rust-sitter-macro`)
- **Count**: 13/13 passing
- **Category**: E2E + INTEGRATION
- **Confidence**: HIGH
- **Coverage**: Grammar attribute expansion, validation
- **Files**:
  - `macro/tests/*.rs`

**Key Tests**:
1. `test_grammar_expansion` - Validates macro expansion
2. `test_language_attribute` - Tests #[language] processing
3. `test_precedence_attributes` - Tests #[prec_left], #[prec_right]
4. `test_leaf_patterns` - Tests #[leaf] with pattern/text
5. `test_repeat_delimited` - Tests Vec<> with delimiters

#### GLR Core (`rust-sitter-glr-core`)
- **Count**: 30/30 passing
- **Category**: UNIT + INTEGRATION
- **Confidence**: HIGH
- **Coverage**: Fork/merge logic, conflict resolution, precedence
- **Files**:
  - `glr-core/tests/test_*.rs`

**Key Tests**:
1. `test_shift_reduce_conflict_resolution` - Precedence handling
2. `test_reduce_reduce_conflict_resolution` - Associativity
3. `test_fork_merge_behavior` - GLR state splitting
4. `test_precedence_ordering` - Rule vs token precedence
5. `test_associativity_left` - Left-associative parsing
6. `test_associativity_right` - Right-associative parsing

#### Table Generation (`rust-sitter-tablegen`)
- **Count**: 8/8 passing
- **Category**: UNIT + INTEGRATION
- **Confidence**: HIGH
- **Coverage**: Compression, encoding, ABI compatibility
- **Files**:
  - `tablegen/tests/test_*.rs`

**Key Tests**:
1. `test_action_table_compression` - Verifies compression correctness
2. `test_accept_encoding` - Validates 0xFFFF encoding
3. `test_token_count_includes_eof` - EOF handling
4. `test_empty_tables` - Edge case handling
5. `test_abi_compatibility` - Tree-sitter ABI match

#### Runtime Basic (`rust-sitter`)
- **Count**: ~50 passing (need exact count)
- **Category**: UNIT + INTEGRATION
- **Confidence**: MEDIUM-HIGH
- **Coverage**: Basic parsing, tree API, error handling
- **Files**:
  - `runtime/tests/*.rs`

---

### ⚠️ Tests with #[ignore] (Documented)

#### Incremental Parsing
- **Count**: ~8 tests
- **Reason**: Feature not yet implemented
- **Priority**: v0.7.0
- **Tracked in**: GAPS.md Section 2

**Tests**:
1. `test_incremental_edit_simple` - Basic edit handling
2. `test_incremental_subtree_reuse` - Parse tree reuse
3. `test_incremental_range_update` - Range-based updates
4. `test_edit_overflow_protection` - Safety checks

#### Query Predicates
- **Count**: ~5 tests
- **Reason**: Partial implementation
- **Priority**: v0.7.0
- **Tracked in**: GAPS.md Section 3

**Tests**:
1. `test_query_predicate_eq` - #eq? predicate
2. `test_query_predicate_match` - #match? predicate
3. `test_query_predicate_validation` - Predicate parsing

#### External Scanners
- **Count**: ~3 tests
- **Reason**: Limited coverage
- **Priority**: v0.7.0
- **Tracked in**: GAPS.md Section 4

**Tests**:
1. `test_external_scanner_integration` - FFI integration
2. `test_python_indent_scanner` - Python indentation
3. `test_scanner_state_management` - State persistence

#### GLR Runtime Integration
- **Count**: ~4 tests
- **Reason**: Runtime wiring not complete
- **Priority**: v0.7.0 (HIGH)
- **Tracked in**: ARCHITECTURE_ISSUE_GLR_PARSER.md

**Tests**:
1. `test_arithmetic_associativity` - Left/right associativity
2. `test_glr_precedence_disambiguation` - Precedence handling
3. `test_python_grammar_parsing` - Complex grammar

---

### 🚧 Missing Tests (Should Exist)

#### Schema Validation
- [ ] `test_parse_table_schema_validation` - Validate table structure
- [ ] `test_ir_schema_validation` - Validate IR representation
- [ ] `test_grammar_json_schema` - Validate grammar.json format

#### Error Handling
- [ ] `test_malformed_grammar_error` - Error messages
- [ ] `test_circular_dependency_detection` - Cycle detection
- [ ] `test_invalid_precedence_error` - Precedence validation

#### Performance
- [ ] `test_large_grammar_compilation_time` - Build time benchmark
- [ ] `test_parse_speed_baseline` - Runtime performance
- [ ] `test_memory_usage_bounds` - Memory constraints

#### Safety
- [ ] `test_stack_overflow_protection` - Deep recursion handling
- [ ] `test_integer_overflow_protection` - Arithmetic safety
- [ ] `test_null_pointer_safety` - FFI safety

---

## Test Policy Enforcement

### CI Requirements

```yaml
test_policy:
  minimum_test_count:
    rust-sitter: 50
    rust-sitter-macro: 13
    rust-sitter-glr-core: 30
    rust-sitter-tablegen: 8
    rust-sitter-tool: 10

  ignored_tests:
    max_allowed: 25  # Current count, should decrease over time
    require_documentation: true
    require_tracking_issue: true

  coverage_targets:
    unit_tests: 90%
    integration_tests: 85%
    e2e_tests: 75%

  performance_gates:
    max_test_duration: 60s
    max_compilation_time: 120s
    max_memory_usage: 2GB
```

### Pre-Commit Hooks

```bash
# .githooks/pre-commit additions
- Check for new #[ignore] without documentation
- Verify test count hasn't decreased
- Run affected crate tests
- Validate test categorization tags
```

### CI Pipeline Gates

```yaml
# .github/workflows/test-policy.yml
jobs:
  validate_test_inventory:
    - Count tests per crate
    - Verify against minimums
    - Check for undocumented #[ignore]
    - Generate test coverage report
```

---

## Test Categorization Tags

Use doc attributes to categorize tests:

```rust
/// Category: UNIT
/// Confidence: HIGH
/// Related: glr-core/precedence
#[test]
fn test_precedence_ordering() {
    // ...
}

/// Category: INTEGRATION
/// Confidence: MEDIUM
/// Blocked-by: #123 (GLR runtime wiring)
#[test]
#[ignore = "Blocked by GLR runtime wiring"]
fn test_associativity_e2e() {
    // ...
}

/// Category: REGRESSION
/// Confidence: HIGH
/// Fixed-by: PR #45
/// Issue: #42
#[test]
fn test_issue_42_null_pointer_crash() {
    // ...
}
```

---

## BDD Scenario Template

```gherkin
Feature: GLR Parser Runtime
  As a rust-sitter user
  I want correct precedence and associativity
  So that my grammars parse expressions correctly

  Scenario: Left-associative operators
    Given a grammar with left-associative multiplication
    When I parse "1 * 2 * 3"
    Then the result should be ((1 * 2) * 3)
    And not (1 * (2 * 3))

  Scenario: Mixed precedence
    Given a grammar with + (prec 1) and * (prec 2)
    When I parse "1 + 2 * 3"
    Then the result should be (1 + (2 * 3))
```

---

## Schema Definitions

### Parse Table Schema (JSON Schema)

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "ParseTable",
  "type": "object",
  "required": ["states", "actions", "gotos", "symbols"],
  "properties": {
    "states": {
      "type": "array",
      "items": { "type": "integer", "minimum": 0 }
    },
    "actions": {
      "type": "array",
      "items": {
        "type": "object",
        "required": ["state", "symbol", "action"],
        "properties": {
          "state": { "type": "integer" },
          "symbol": { "type": "integer" },
          "action": {
            "oneOf": [
              { "type": "object", "properties": { "Shift": { "type": "integer" }}},
              { "type": "object", "properties": { "Reduce": { "type": "integer" }}},
              { "const": "Accept" },
              { "const": "Error" }
            ]
          }
        }
      }
    }
  }
}
```

---

## Next Steps

1. **Immediate** (This Session):
   - [ ] Generate automated test inventory from codebase
   - [ ] Add test policy enforcement to CI
   - [ ] Create schema validators for parse tables

2. **v0.7.0**:
   - [ ] Re-enable all ignored tests (tracked in GAPS.md)
   - [ ] Add missing safety/performance tests
   - [ ] Achieve 85%+ coverage on core paths

3. **v1.0**:
   - [ ] 95%+ test coverage
   - [ ] All tests passing
   - [ ] Zero ignored tests
   - [ ] Comprehensive BDD scenarios

---

**See Also**:
- [GAPS.md](./GAPS.md) - Task breakdown with test re-enablement
- [ARCHITECTURE_ISSUE_GLR_PARSER.md](./ARCHITECTURE_ISSUE_GLR_PARSER.md) - GLR runtime issue
- [CURRENT_STATUS_2025-11.md](./CURRENT_STATUS_2025-11.md) - Detailed status
