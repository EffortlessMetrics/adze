# Runtime2 GLR Examples

**Status**: Phase 3.3 Integration Testing
**Purpose**: Validate GLR runtime with example grammars
**Related**: [Phase 3.3 Specification](../../docs/specs/PHASE_3.3_INTEGRATION_TESTING.md)

---

## Overview

This directory contains example grammars specifically designed to test and demonstrate the **runtime2 GLR (Generalized LR) parsing runtime**. These examples validate that the pure-Rust GLR implementation works correctly with both ambiguous and unambiguous grammars.

### Why Separate Examples?

As documented in [ADR-0007](../../docs/adr/ADR-0007-RUNTIME2-GLR-INTEGRATION.md):
- **runtime2**: Production-ready GLR runtime (Phase 3.1-3.2 complete)
- **runtime**: Original runtime crate (used by main examples/)
- **Separation**: Allows GLR testing without affecting existing functionality

These examples will eventually merge into the main examples/ directory once runtime2 is integrated into runtime.

---

## Example Grammars

### Unambiguous Grammars (GLR = LR Parity)

These grammars have precedence and associativity annotations that resolve all conflicts. GLR should produce **identical results** to LR parsing.

#### `arithmetic.rs` ✅ Priority 1

**Grammar**: Simple arithmetic with precedence
```rust
Expression ::= Number
            | Expression - Expression  // precedence 1, left assoc
            | Expression * Expression  // precedence 2, left assoc
```

**Test Scenarios**:
- Precedence: `1 - 2 * 3` → `1 - (2 * 3)`
- Left associativity: `1 - 2 - 3` → `(1 - 2) - 3`
- Complex expressions: `1 - 2 * 3 - 4`

**Contract**: GLR output must match LR output exactly

### Ambiguous Grammars (GLR Specific)

These grammars intentionally have NO precedence annotations, creating conflicts that GLR must handle via forking and disambiguation.

#### `ambiguous_expr.rs` ✅ Priority 1

**Grammar**: Expression without precedence
```rust
Expression ::= Number
            | Expression + Expression  // NO precedence!
```

**Test Scenarios**:
- Multiple parse trees: `1 + 2 + 3` has 2 valid parses
- Disambiguation: GLR selects one valid interpretation
- Forest exploration: Verify both parses exist before selection

**Contract**: GLR must handle ambiguity without error

#### `dangling_else.rs` ✅ Priority 1

**Grammar**: Classic if/if-else ambiguity
```rust
Statement ::= if Expression Statement
           | if Expression Statement else Statement
```

**Test Scenarios**:
- Dangling else: `if a if b c else d`
  - Parse 1: `if a { if b c else d }` (else binds to inner if)
  - Parse 2: `if a { if b c } else d` (else binds to outer if)
- Unambiguous cases: `if a c else d`

**Contract**: GLR produces valid parse tree for both interpretations

---

## Running Examples

### Build with GLR Runtime

```bash
# From workspace root
cargo build -p rust-sitter-runtime --features pure-rust-glr --examples
```

### Run Specific Example

```bash
cargo run -p rust-sitter-runtime --features pure-rust-glr --example arithmetic
```

### Run All Example Tests

```bash
cargo test -p rust-sitter-runtime --features pure-rust-glr --examples
```

---

## Testing Strategy

### Unit Tests (Per Example)

Each example includes comprehensive unit tests:
- Precedence validation
- Associativity validation
- Error handling
- Edge cases

### Integration Tests (runtime2/tests/)

Cross-example integration tests:
- **GLR vs LR Parity**: Verify unambiguous grammars produce identical results
- **Performance**: Benchmark GLR overhead
- **Memory**: Profile memory usage
- **E2E**: End-to-end scenarios

---

## Example Structure

Each example follows this pattern:

```rust
//! Example: <name>
//!
//! Grammar: <description>
//! Contract: <test contract>
//! Phase: 3.3 Integration Testing

use rust_sitter_runtime::{Parser, Tree};
use rust_sitter_glr_core::{build_lr1_automaton, FirstFollowSets};
use rust_sitter_ir::Grammar;

// Grammar definition
fn create_grammar() -> Grammar {
    // ... grammar construction
}

// Helper: Create parser with GLR table
fn create_parser() -> Parser {
    let grammar = create_grammar();
    let ff = FirstFollowSets::compute(&grammar).unwrap();
    let table = build_lr1_automaton(&grammar, &ff).unwrap();

    let mut parser = Parser::new();
    parser.set_glr_table(Box::leak(Box::new(table))).unwrap();
    // ... set metadata and patterns

    parser
}

// Tests
#[cfg(test)]
mod tests {
    #[test]
    fn test_basic_parsing() {
        let parser = create_parser();
        let tree = parser.parse(b"input", None).unwrap();
        // ... assertions
    }
}
```

---

## Success Criteria (Phase 3.3)

Per [PHASE_3.3_INTEGRATION_TESTING.md](../../docs/specs/PHASE_3.3_INTEGRATION_TESTING.md):

### Functional
- [ ] All example grammars parse with `pure-rust-glr`
- [ ] Ambiguous grammars produce valid trees
- [ ] GLR matches LR for unambiguous grammars
- [ ] All integration tests pass

### Performance
- [ ] GLR ≤ 2x slower than LR (unambiguous)
- [ ] Large file (1MB) parses in < 5 seconds
- [ ] Memory usage ≤ 10x input size
- [ ] No memory leaks

### Quality
- [ ] Test coverage >80%
- [ ] All tests passing
- [ ] Documentation complete
- [ ] Performance baselines established

---

## Development Workflow

Following the **contract-first, spec-driven, test-driven** methodology:

### 1. Specification First
Before writing code, define:
- Grammar contract
- Test scenarios (BDD format)
- Success criteria
- Performance targets

### 2. Tests First (TDD)
Write tests before implementation:
- Unit tests for each scenario
- Integration tests for cross-cutting concerns
- Property tests for invariants

### 3. Implementation
Build to make tests pass:
- Grammar construction
- Parser setup
- Helper functions

### 4. Validation
Verify all contracts met:
- All tests passing
- Performance within targets
- Documentation complete

---

## References

- [Phase 3.3 Specification](../../docs/specs/PHASE_3.3_INTEGRATION_TESTING.md) - Complete integration testing plan
- [ADR-0007](../../docs/adr/ADR-0007-RUNTIME2-GLR-INTEGRATION.md) - Runtime2 integration decision
- [Phase 3 Overview](../../docs/specs/PHASE_3_PURE_RUST_GLR_RUNTIME.md) - GLR runtime architecture

---

## Future Work

**Post-Phase 3.4**: Once runtime2 is stable:
1. Merge runtime2 into runtime
2. Migrate these examples to main examples/
3. Deprecate runtime2 as separate package
4. Update all documentation

**Timeline**: Phase 4+

---

**Status**: Phase 3.3 in progress
**Last Updated**: 2025-11-19
**Next**: Implement arithmetic example
