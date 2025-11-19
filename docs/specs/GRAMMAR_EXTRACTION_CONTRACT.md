# Grammar Extraction Contract Specification

## Version: 1.0.0
## Status: Draft
## Date: 2025-11-19

## Purpose

Define the contract for how Rust type definitions (enums, structs) should map to Grammar IR productions, ensuring predictable and testable behavior for both ambiguous and unambiguous grammars.

## Scope

This specification covers:
- Enum variant to production mapping
- Struct field to production mapping
- Precedence and associativity handling
- Recursive type handling
- Conflict generation expectations

## Contract: Enum Variant Mapping

### Given: A Rust enum with multiple variants

```rust
#[rust_sitter::language]
enum Expr {
    Binary(Box<Expr>, String, Box<Expr>),
    Number(i32),
}
```

### Expected Behavior

#### Requirement 1: Direct Production Mapping

**MUST**: Each enum variant MUST map to a production rule with the enum name as LHS

```ebnf
Expr → Binary-production
Expr → Number-production
```

#### Requirement 2: Variant Inlining for Simple Cases

**MUST**: Variants with inline fields MUST expand to direct symbol sequences

```ebnf
# For Binary variant:
Expr → Expr OP Expr

# For Number variant:
Expr → NUMBER
```

**MUST NOT**: Create intermediate non-terminals unless explicitly required for:
- Field names (when using `#[rust_sitter::field]`)
- Precedence grouping
- External scanner integration

#### Requirement 3: Recursion Preservation

**MUST**: Preserve direct left-recursion in variant definitions

```rust
Binary(Box<Expr>, ...) → Expr → ... (recursive)
```

**MUST**: Result in left-recursive production:
```ebnf
Expr → Expr ...
```

#### Requirement 4: Ambiguity Preservation

**MUST**: When NO precedence attributes are present, the grammar MUST remain ambiguous

**Test Case**:
```rust
// No #[rust_sitter::prec_left] or #[rust_sitter::prec_right]
Binary(Box<Expr>, String, Box<Expr>)
```

**Expected**: LR(1) automaton detects shift/reduce conflicts

**Current Behavior**: ZERO conflicts detected ❌

## Contract: Production Structure

### Symbol Naming Convention

**Terminal Symbols**:
- Literals: Use exact string (e.g., "+", "if")
- Patterns: Use generated name (e.g., "_1", "_2") with metadata

**Non-Terminal Symbols**:
- Enum types: Use enum name (e.g., "Expr")
- Variant-specific: ONLY when required for fields or precedence
- Avoid: `Expr_Binary_Expr_Binary_1` style names unless necessary

### Production Format

**MUST**: Use IR `Rule` structure:
```rust
Rule {
    lhs: SymbolId,          // The enum name
    rhs: Vec<Symbol>,       // Direct expansion of variant fields
    precedence: Option<i16>,
    associativity: Option<Associativity>,
    fields: Vec<FieldMapping>,
    production_id: ProductionId,
}
```

**MUST NOT**: Add implicit precedence when none specified

## Contract: Conflict Generation

### Scenario 1: Unambiguous Grammar with Precedence

**Given**:
```rust
#[rust_sitter::prec_left(1)]
Add(Box<Expr>, #[rust_sitter::leaf(text = "+")] (), Box<Expr>)

#[rust_sitter::prec_left(2)]
Mul(Box<Expr>, #[rust_sitter::leaf(text = "*")] (), Box<Expr>)
```

**Expected**: Conflicts detected then resolved by precedence

**Validation**: Parse table has multi-action cells with priority ordering

### Scenario 2: Ambiguous Grammar without Precedence

**Given**:
```rust
// NO precedence attributes
Binary(Box<Expr>, String, Box<Expr>)
```

**Expected**: Conflicts detected and preserved (GLR mode)

**Validation**: Parse table has multi-action cells, both actions valid

### Scenario 3: Non-Ambiguous Grammar

**Given**:
```rust
IfThen(/* distinct from IfThenElse through structure */)
IfThenElse(/* distinct through different token sequence */)
```

**Expected**: ZERO conflicts

**Validation**: All action cells have exactly 1 action

## Test Contracts

### Test 1: Production Count Equality

**Given**: Manual grammar and enum grammar defining same language

**When**: Both converted to Grammar IR

**Then**: Production counts MUST match

### Test 2: Symbol Table Equality

**Given**: Manual grammar and enum grammar defining same language

**When**: Both converted to Grammar IR

**Then**:
- Terminal count MUST match
- Non-terminal count MUST match
- Symbol names MUST correspond (allowing for generated names)

### Test 3: Recursion Pattern Equality

**Given**: Manual grammar with `rule("E", vec!["E", "+", "n"])`

**And**: Enum grammar with `Variant(Box<Enum>, ...)`

**When**: Both converted to Grammar IR

**Then**: Both MUST have left-recursive production with same RHS length

### Test 4: Conflict Detection Equality

**Given**: Manual ambiguous grammar generates N conflicts

**And**: Equivalent enum grammar with NO precedence

**When**: Both build LR(1) automata

**Then**: Enum grammar MUST also generate N conflicts

## Acceptance Criteria

### Phase 1: Investigation (Current)

- [x] Prove LR(1) builder works (manual grammar)
- [x] Prove enum grammar generates zero conflicts
- [ ] Extract production structures from both
- [ ] Compare symbol counts and production counts
- [ ] Identify where disambiguation occurs

### Phase 2: Root Cause

- [ ] Locate code that adds intermediate symbols
- [ ] Determine if inlining is incomplete
- [ ] Check for implicit precedence application
- [ ] Document exact transformation steps

### Phase 3: Solution Design

- [ ] Option A: Fix extraction to inline completely
- [ ] Option B: Add `#[rust_sitter::inline]` attribute
- [ ] Option C: Document as architectural limitation
- [ ] Choose approach based on impact analysis

### Phase 4: Implementation

- [ ] Implement chosen solution
- [ ] Add regression tests
- [ ] Update documentation
- [ ] Validate GLR conflict preservation

## Success Metrics

### Functional Requirements

✅ **FR-1**: Manual and enum grammars MUST generate equivalent productions
✅ **FR-2**: Ambiguous grammars MUST preserve conflicts
✅ **FR-3**: LR(1) builder MUST detect all conflicts
✅ **FR-4**: GLR preservation MUST maintain all valid parse paths

### Performance Requirements

✅ **PR-1**: Grammar extraction MUST complete in <1s
✅ **PR-2**: LR(1) construction MUST handle 1000+ states
✅ **PR-3**: Conflict detection MUST scale linearly

### Quality Requirements

✅ **QR-1**: All transformations MUST be deterministic
✅ **QR-2**: Error messages MUST identify exact source location
✅ **QR-3**: 100% test coverage for extraction logic

## Related Documents

- `docs/plans/ENUM_VARIANT_DISAMBIGUATION.md` - Investigation hypothesis
- `docs/plans/CORRECTED_ROOT_CAUSE_ANALYSIS.md` - Root cause findings
- `docs/plans/BDD_GLR_CONFLICT_PRESERVATION.md` - Original BDD spec

## Revision History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0.0 | 2025-11-19 | Claude | Initial contract specification |

## Approval

- [ ] Technical Review
- [ ] Architectural Review
- [ ] Security Review
- [ ] Implementation Ready
