# Specification: Enum Variant Inlining

## Version

1.0.0 (2025-11-19)

## Status

Implemented (2025-11-19)

## Purpose

Define the precise behavior of enum variant inlining in adze grammar generation to enable GLR conflict preservation while maintaining backward compatibility.

## Overview

Enum variant inlining controls whether enum variants generate intermediate grammar symbols or are expanded directly into the parent enum's CHOICE production.

## Terminology

- **Intermediate Symbol**: A named grammar rule created for an enum variant (e.g., `Expr_Binary`)
- **Inlined Variant**: Variant whose fields are expanded directly into the parent enum's CHOICE
- **Direct Production**: A grammar production without intermediate symbols (e.g., `Expr → Expr OP Expr`)

## Inlining Decision Algorithm

### Input

An enum variant with:
- Variant name (e.g., `Binary`)
- Parent enum name (e.g., `Expr`)
- Fields (named or unnamed)
- Attributes (precedence, no_inline, etc.)

### Output

Boolean: `should_inline` (true = inline, false = create intermediate symbol)

### Algorithm

```
should_inline_variant(variant) -> bool:
    // Rule 1: Explicit opt-out takes precedence
    if variant.has_attribute("adze::no_inline"):
        return false

    // Rule 2: Unit variants never inline (backward compatibility)
    if variant.is_unit():
        return false

    // Rule 3: Variants with precedence never inline (backward compatibility)
    if variant.has_precedence_attribute():
        return false

    // Rule 4: Default behavior - inline for GLR support
    return true
```

## Grammar Generation Behavior

### Inlined Variant (should_inline = true)

**Input Rust Code:**
```rust
#[adze::language]
enum Expr {
    Binary(Box<Expr>, String, Box<Expr>),
    Number(i32),
}
```

**Generated Grammar JSON:**
```json
{
  "rules": {
    "Expr": {
      "type": "CHOICE",
      "members": [
        {
          "type": "SEQ",
          "members": [
            {"type": "FIELD", "name": "Binary_0", "content": {"type": "SYMBOL", "name": "Expr"}},
            {"type": "FIELD", "name": "Binary_1", "content": {"type": "SYMBOL", "name": "OP"}},
            {"type": "FIELD", "name": "Binary_2", "content": {"type": "SYMBOL", "name": "Expr"}}
          ]
        },
        {
          "type": "PATTERN",
          "value": "\\d+"
        }
      ]
    }
  }
}
```

**Key Properties:**
- NO `Expr_Binary` or `Expr_Number` intermediate rules
- Variant fields expanded directly into CHOICE member
- Field names preserve variant context (`Binary_0`, `Binary_1`, etc.)
- Results in **direct productions**: `Expr → Expr OP Expr`, `Expr → NUMBER`

### Non-Inlined Variant (should_inline = false)

**Input Rust Code:**
```rust
#[adze::language]
enum Expr {
    #[adze::no_inline]
    Binary(Box<Expr>, String, Box<Expr>),

    Number(i32),
}
```

**Generated Grammar JSON:**
```json
{
  "rules": {
    "Expr": {
      "type": "CHOICE",
      "members": [
        {"type": "SYMBOL", "name": "Expr_Binary"},
        {"type": "PATTERN", "value": "\\d+"}
      ]
    },
    "Expr_Binary": {
      "type": "SEQ",
      "members": [
        {"type": "FIELD", "name": "Expr_Binary_0", "content": {"type": "SYMBOL", "name": "Expr"}},
        {"type": "FIELD", "name": "Expr_Binary_1", "content": {"type": "SYMBOL", "name": "OP"}},
        {"type": "FIELD", "name": "Expr_Binary_2", "content": {"type": "SYMBOL", "name": "Expr"}}
      ]
    }
  }
}
```

**Key Properties:**
- `Expr_Binary` intermediate rule created
- CHOICE references intermediate symbol
- Results in **indirect productions**: `Expr → Expr_Binary → Expr OP Expr`

## Attribute Specification

### `#[adze::no_inline]`

**Syntax:**
```rust
#[adze::no_inline]
variant_name(fields)
```

**Semantics:**
- Forces creation of intermediate symbol for this variant
- Overrides default inlining behavior
- Does not take parameters

**Valid Locations:**
- Enum variant declaration only
- Cannot be applied to enum itself or fields

**Example:**
```rust
#[adze::language]
enum Statement {
    // Complex variant - keep intermediate for clarity
    #[adze::no_inline]
    FunctionDecl {
        name: String,
        params: Vec<Param>,
        body: Block,
    },

    // Simple variant - inline by default
    Expression(Expr),
}
```

**Error Conditions:**
```rust
// ERROR: no_inline on enum itself
#[adze::no_inline]  // ❌ Error
enum Expr { ... }

// ERROR: no_inline on field
enum Expr {
    Binary(
        #[adze::no_inline]  // ❌ Error
        Box<Expr>,
        String,
        Box<Expr>
    ),
}
```

## Backward Compatibility

### Preserved Behaviors

1. **Precedence-based grammars** (arithmetic, expression with operators)
   - Variants with `#[prec]`, `#[prec_left]`, `#[prec_right]` keep intermediates
   - Grammar structure unchanged
   - No migration needed

2. **Unit variants**
   - Unit enum variants keep intermediate symbols
   - Example: `enum Token { Plus, Minus }` → `Token_Plus`, `Token_Minus`

3. **Explicit opt-out**
   - `#[no_inline]` provides escape hatch
   - Users can preserve old behavior per-variant

### Breaking Changes

**Grammars without precedence will change:**

**Before (v0.7.x):**
```
Expr → Expr_Binary
Expr → Expr_Number
Expr_Binary → Expr OP Expr
Expr_Number → NUMBER
```

**After (v0.8.0):**
```
Expr → Expr OP Expr
Expr → NUMBER
```

**Migration:**
- Add `#[adze::no_inline]` to variants if intermediate symbols needed
- Update CST traversal code if relying on `Expr_Binary` node names

## Field Naming in Inlined Variants

When a variant is inlined, field names must preserve the variant context to avoid collisions:

**Example:**
```rust
enum Node {
    Binary(Expr, Op, Expr),  // Fields: Binary_0, Binary_1, Binary_2
    Unary(Op, Expr),          // Fields: Unary_0, Unary_1
}
```

**Naming Rules:**
1. Named fields: `{VariantName}_{field_name}`
2. Unnamed fields: `{VariantName}_{index}`
3. Consistent across grammar generation

## Test Scenarios

### Scenario 1: Simple Ambiguous Grammar

**Input:**
```rust
#[adze::language]
enum Expr {
    Binary(Box<Expr>, String, Box<Expr>),
    Number(i32),
}
```

**Expected:**
- `should_inline(Binary) == true`
- `should_inline(Number) == true`
- Grammar has 2 direct productions
- LR(1) detects shift/reduce conflicts
- GLR can parse ambiguous expressions

### Scenario 2: Mixed Inlining

**Input:**
```rust
#[adze::language]
enum Stmt {
    #[adze::no_inline]
    FunctionDecl { name: String, body: Block },

    Expr(Expression),  // Inlined by default
}
```

**Expected:**
- `should_inline(FunctionDecl) == false`
- `should_inline(Expr) == true`
- `Stmt_FunctionDecl` intermediate exists
- `Expr` variant inlined into CHOICE

### Scenario 3: Precedence Preservation

**Input:**
```rust
#[adze::language]
enum Expr {
    #[adze::prec_left(1)]
    Add(Box<Expr>, Box<Expr>),

    #[adze::prec_left(2)]
    Mul(Box<Expr>, Box<Expr>),

    Number(i32),
}
```

**Expected:**
- `should_inline(Add) == false` (has precedence)
- `should_inline(Mul) == false` (has precedence)
- `should_inline(Number) == true`
- Precedence-based conflict resolution preserved
- Grammar structure unchanged from v0.7.x

### Scenario 4: Unit Variants

**Input:**
```rust
#[adze::language]
enum Token {
    Plus,
    Minus,
    Star,
}
```

**Expected:**
- `should_inline(Plus) == false` (unit variant)
- `should_inline(Minus) == false` (unit variant)
- `should_inline(Star) == false` (unit variant)
- Intermediates: `Token_Plus`, `Token_Minus`, `Token_Star`
- Behavior unchanged from v0.7.x

## Error Handling

### Invalid Attribute Usage

**Error 1: `no_inline` on enum**
```rust
#[adze::no_inline]  // ❌ Error
#[adze::language]
enum Expr { ... }
```
**Error Message:** "`no_inline` attribute can only be applied to enum variants, not the enum itself"

**Error 2: `no_inline` on field**
```rust
enum Expr {
    Binary(
        #[adze::no_inline] Box<Expr>,  // ❌ Error
        String
    )
}
```
**Error Message:** "`no_inline` attribute can only be applied to enum variants, not fields"

**Error 3: `no_inline` with parameters**
```rust
enum Expr {
    #[adze::no_inline(true)]  // ❌ Error
    Binary(Box<Expr>, String, Box<Expr>)
}
```
**Error Message:** "`no_inline` attribute does not accept parameters"

## Implementation Checklist

- [x] Add `no_inline` attribute parsing in `expansion.rs`
- [x] Implement `should_inline_variant()` helper function
- [x] Modify enum variant loop to check inlining condition
- [x] Implement direct field expansion for inlined variants
- [x] Preserve field naming with variant context
- [x] Add error handling for invalid attribute usage
- [x] Write unit tests for inlining decision algorithm
- [x] Write integration tests for grammar generation
- [x] Verify backward compatibility with precedence grammars
- [ ] Update documentation and migration guide (pending)

## References

- **ADR**: `docs/adr/0003-enum-variant-inlining-for-glr.md`
- **Contract**: `docs/specs/GRAMMAR_EXTRACTION_CONTRACT.md`
- **Investigation**: `docs/plans/ENUM_VARIANT_DISAMBIGUATION.md`
- **Tests**: `tool/tests/test_grammar_extraction_contract.rs`

## Changelog

- **1.0.0** (2025-11-19): Initial specification
