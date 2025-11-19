# ADR 0003: Enum Variant Inlining for GLR Conflict Preservation

## Status

Proposed

## Context

### Problem Statement

The current enum variant expansion in `tool/src/expansion.rs` creates intermediate symbols for each variant, preventing the generation of shift/reduce conflicts needed for GLR parsing:

```rust
// Current behavior
enum Expr {
    Binary(Box<Expr>, String, Box<Expr>),
    Number(i32),
}

// Generates:
Expr → Expr_Binary
Expr → Expr_Number
Expr_Binary → Expr OP Expr  // 3 rules with intermediate symbols
Expr_Number → NUMBER
```

**Impact**: The intermediate symbols (`Expr_Binary`, `Expr_Number`) provide disambiguation points that allow the LR(1) parser to avoid shift/reduce conflicts, breaking GLR functionality.

**Expected Behavior** (from Grammar Extraction Contract):
```
Expr → Expr OP Expr  // 2 rules, direct
Expr → NUMBER
```

### Investigation Evidence

- **Test**: `test_contract_enum_grammar_extraction()` proves intermediate symbols are created
- **Comparison**: Manual grammar (2 rules) vs Enum grammar (7 rules)
- **Code Location**: `tool/src/expansion.rs:814-854`
- **Related Docs**: `docs/plans/ENUM_VARIANT_DISAMBIGUATION.md`, `docs/specs/GRAMMAR_EXTRACTION_CONTRACT.md`

## Decision

Implement **default variant inlining with opt-out attribute** for enum variants.

### Inlining Rules

Enum variants will be inlined directly into CHOICE members by default, eliminating intermediate symbols, when ALL of the following conditions are met:

1. **No precedence on variant**: Variant lacks `#[rust_sitter::prec]`, `#[rust_sitter::prec_left]`, or `#[rust_sitter::prec_right]`
2. **No explicit no_inline attribute**: Variant lacks `#[rust_sitter::no_inline]`
3. **Non-unit fields**: Variant has fields (not a unit variant)

### New Attribute: `#[rust_sitter::no_inline]`

Users can preserve the current intermediate symbol behavior for specific variants:

```rust
#[rust_sitter::language]
enum Expr {
    #[rust_sitter::no_inline]  // Keeps intermediate Expr_Binary symbol
    Binary(Box<Expr>, String, Box<Expr>),

    Number(i32),  // Inlined by default
}
```

**Use Cases for `no_inline`**:
- Complex AST node types that benefit from named intermediate rules
- Debugging: easier to trace specific variant in grammar
- Tree-sitter node naming: when you want `Expr_Binary` nodes in the CST

### Implementation Strategy

1. **Phase 1: Detection**
   - Add helper function to detect if variant should be inlined
   - Check for precedence and `no_inline` attributes
   - Default to `inline=true` for maximum GLR compatibility

2. **Phase 2: Inlining Logic**
   - When `inline=true`, expand variant fields directly into CHOICE member
   - Skip intermediate rule generation for inlined variants
   - Preserve field names and structure

3. **Phase 3: Backward Compatibility**
   - Variants with precedence keep intermediate symbols (existing behavior)
   - Unit variants keep intermediate symbols (existing behavior)
   - Explicit `#[no_inline]` keeps intermediate symbols

## Consequences

### Positive

1. **GLR Functionality**: Enum-based grammars can now generate shift/reduce conflicts
2. **Contract Compliance**: Meets Grammar Extraction Contract specification
3. **Backward Compatible**: Precedence-based grammars (arithmetic) unchanged
4. **User Control**: `#[no_inline]` provides escape hatch when needed
5. **Simpler Grammars**: Fewer intermediate rules for most cases

### Negative

1. **Breaking Change for Some**: Grammars without precedence will have different structure
   - **Mitigation**: Add `#[no_inline]` to preserve old behavior
   - Document in migration guide

2. **CST Node Names Change**: Some node types will change names
   - **Before**: `Expr_Binary`, `Expr_Number`
   - **After**: Both become `Expr` with variant field distinguisher
   - **Mitigation**: Use `#[no_inline]` if node names matter

3. **Increased Complexity**: Grammar generation has more conditional logic
   - **Mitigation**: Well-documented, thoroughly tested

### Neutral

1. **Documentation Updates Needed**: Must document inlining behavior and `#[no_inline]` attribute
2. **Migration Path**: Users with complex grammars may need to audit and add `#[no_inline]`

## Alternatives Considered

### Alternative 1: Always Inline (No Attribute)

**Rejected**: Too breaking, no escape hatch for users who need intermediate symbols.

### Alternative 2: Opt-In Inlining (`#[inline]` attribute)

**Rejected**: Requires users to understand the issue and opt-in. Default behavior would still prevent GLR, violating principle of least surprise for ambiguous grammars.

### Alternative 3: Separate Grammar Definition Syntax

**Rejected**: Too complex, creates fragmentation. The enum-based syntax should "just work" for ambiguous grammars.

### Alternative 4: Document Limitation Only

**Rejected**: Leaves GLR functionality broken for enum-based grammars, forcing users to manually build Grammar IR.

## Implementation Plan

### Phase 1: Specification & Tests (TDD)

1. Create `docs/specs/ENUM_VARIANT_INLINING.md` specification
2. Write failing tests in `tool/tests/test_grammar_extraction_contract.rs`:
   - `test_inlined_enum_matches_manual_grammar()`
   - `test_no_inline_attribute_preserves_intermediates()`
   - `test_precedence_preserves_intermediates()`
3. Run tests, verify they fail with clear messages

### Phase 2: Implementation

1. Add `should_inline_variant()` helper function in `expansion.rs`
2. Modify enum variant processing loop (lines 814-854)
3. Implement direct field expansion for inlined variants
4. Add `no_inline` attribute parsing

### Phase 3: Validation

1. Verify all contract tests pass
2. Run `test_contract_enum_grammar_extraction()` - should pass without violations
3. Ensure arithmetic and other precedence grammars unchanged
4. Add regression tests for edge cases

### Phase 4: Documentation

1. Update `CLAUDE.md` with inlining behavior
2. Document `#[rust_sitter::no_inline]` attribute
3. Create migration guide for breaking changes
4. Update tutorial examples

## Acceptance Criteria

- [ ] `test_contract_enum_grammar_extraction()` passes without contract violations
- [ ] Enum grammar generates same rule count as manual grammar (2 vs 2, not 7 vs 2)
- [ ] LR(1) tests detect conflicts in enum-based ambiguous grammar
- [ ] Arithmetic grammar (with precedence) still compiles and works
- [ ] `#[no_inline]` attribute preserves intermediate symbols
- [ ] All existing tests pass
- [ ] Documentation updated

## References

- Grammar Extraction Contract: `docs/specs/GRAMMAR_EXTRACTION_CONTRACT.md`
- Investigation: `docs/plans/ENUM_VARIANT_DISAMBIGUATION.md`
- Code Location: `tool/src/expansion.rs:814-854`
- Tests: `tool/tests/test_grammar_extraction_contract.rs`
- Related Issue: GLR conflict preservation validation

## Timeline

- **Decision**: 2025-11-19
- **Implementation Start**: 2025-11-19
- **Target Completion**: Within 2 days
- **Review & Merge**: Upon acceptance criteria completion
