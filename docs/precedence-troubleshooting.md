# Precedence Attribute Troubleshooting Guide

This guide helps you understand and resolve precedence attribute errors in rust-sitter grammars.

## Overview

Rust-sitter provides three precedence attributes to control parsing of ambiguous grammars:

- `#[rust_sitter::prec(n)]` - Non-associative precedence
- `#[rust_sitter::prec_left(n)]` - Left-associative precedence  
- `#[rust_sitter::prec_right(n)]` - Right-associative precedence

## Common Error Messages and Solutions

### Error: Multiple Precedence Attributes

**Error Message:**
```
only one of prec, prec_left, and prec_right can be specified, but found: prec, prec_left
```

**Problem:** You've applied multiple precedence attributes to the same rule.

**Bad Example:**
```rust
#[rust_sitter::prec(1)]
#[rust_sitter::prec_left(2)]
pub struct Conflict {
    // ...
}
```

**Solution:** Use only one precedence attribute per rule:
```rust
// Choose the appropriate associativity
#[rust_sitter::prec_left(2)]
pub struct Fixed {
    // ...
}
```

### Error: Non-Integer Precedence Value

**Error Message:**
```
Expected integer literal for precedence. Use #[rust_sitter::prec(123)] with a positive integer (0 to 4294967295).
```

**Problem:** The precedence value is not an integer literal.

**Common Bad Examples:**
```rust
// String instead of integer
#[rust_sitter::prec("high")]

// Float instead of integer
#[rust_sitter::prec_left(3.14)]

// Variable instead of literal
const HIGH_PREC: u32 = 10;
#[rust_sitter::prec(HIGH_PREC)]

// Boolean instead of integer
#[rust_sitter::prec_right(true)]
```

**Solution:** Use integer literals directly:
```rust
#[rust_sitter::prec(10)]           // ✅ Valid
#[rust_sitter::prec_left(20)]      // ✅ Valid
#[rust_sitter::prec_right(30)]     // ✅ Valid
```

### Error: Precedence Value Out of Range

**Error Messages:**
```
Invalid integer literal for precedence: number too large for type 'u32'
```

**Problem:** The precedence value is outside the u32 range (0 to 4294967295).

**Bad Examples:**
```rust
#[rust_sitter::prec(-1)]           // Negative number
#[rust_sitter::prec(4294967296)]   // Too large for u32
```

**Solution:** Use values within the valid range:
```rust
#[rust_sitter::prec(0)]            // ✅ Minimum value
#[rust_sitter::prec(100)]          // ✅ Common value
#[rust_sitter::prec(4294967295)]   // ✅ Maximum value
```

## Best Practices

### Precedence Value Guidelines

1. **Valid Range:** 0 to 4294967295 (u32)
2. **Zero is Valid:** `#[rust_sitter::prec(0)]` is the lowest precedence
3. **Use Meaningful Gaps:** Space values (10, 20, 30) for future expansion
4. **Higher Numbers Bind Tighter:** Multiplication (20) > Addition (10)

### Common Precedence Patterns

```rust
// Arithmetic operators (common pattern)
#[rust_sitter::prec_left(10)]  // Addition, subtraction
Add(Box<Expr>, (), Box<Expr>),

#[rust_sitter::prec_left(20)]  // Multiplication, division
Mul(Box<Expr>, (), Box<Expr>),

#[rust_sitter::prec_right(30)] // Exponentiation
Pow(Box<Expr>, (), Box<Expr>),

#[rust_sitter::prec(40)]       // Comparison (non-associative)
Compare(Box<Expr>, CompOp, Box<Expr>),
```

### Associativity Choices

- **Left Associative (`prec_left`)**: For operations like `1 - 2 - 3` → `(1 - 2) - 3`
  - Addition, subtraction, multiplication, division
  - Function calls, member access

- **Right Associative (`prec_right`)**: For operations like `2^3^4` → `2^(3^4)`
  - Exponentiation
  - Assignment operators
  - Some conditional operators

- **Non-Associative (`prec`)**: For operations that shouldn't chain
  - Comparison operators (`<`, `>`, `==`)
  - Type annotations

## Debugging Precedence Issues

### 1. Check for Attribute Conflicts

Search your grammar for multiple precedence attributes:
```bash
grep -n "prec\|precedence" your_grammar.rs
```

### 2. Validate Precedence Values

Ensure all precedence values are:
- Integer literals (not variables or expressions)
- Within u32 range (0 to 4294967295)
- Appropriate for the operator precedence hierarchy

### 3. Test Parsing Behavior

Create test cases to verify precedence:
```rust
#[test]
fn test_precedence() {
    // Test that multiplication binds tighter than addition
    assert_eq!(
        grammar::parse("1 + 2 * 3").unwrap(),
        Add(
            Box::new(Number(1)),
            (),
            Box::new(Mul(
                Box::new(Number(2)),
                (),
                Box::new(Number(3))
            ))
        )
    );
}
```

### 4. Review Error Context

Precedence errors include specific context:
- Which attributes were found
- The expected format for each attribute type
- The valid range for precedence values

## Integration with GLR Parsing

In rust-sitter's GLR mode:
- Multiple precedence conflicts are preserved for ambiguity handling
- Precedence helps order actions but doesn't eliminate them completely
- This enables parsing of inherently ambiguous grammars

## Migration from Tree-sitter

If migrating from Tree-sitter grammar.js:
```javascript
// Tree-sitter grammar.js
prec.left(1, seq($.expr, '+', $.expr))
prec.right(2, seq($.base, '^', $.exp))
prec(3, seq($.left, '==', $.right))
```

Becomes:
```rust
// rust-sitter
#[rust_sitter::prec_left(1)]
Add(Box<Expr>, (), Box<Expr>),

#[rust_sitter::prec_right(2)]
Pow(Box<Expr>, (), Box<Expr>),

#[rust_sitter::prec(3)]
Equal(Box<Expr>, (), Box<Expr>),
```

## Advanced Troubleshooting

### Build-Time vs Runtime Errors

- **Build-Time:** Precedence attribute validation happens during grammar processing
- **Scope:** Errors are caught before parser generation
- **IDE Support:** Error messages include source location information

### Performance Considerations

- Precedence attributes have zero runtime cost
- They only affect the generated parse tables
- GLR mode may create larger tables but maintains correctness

### Complex Grammar Interactions

If precedence errors occur in complex grammars:
1. Isolate the problematic rule in a minimal test case
2. Check for interactions with `#[rust_sitter::field]` and other attributes
3. Verify the rule structure matches the precedence attribute type

## Getting Help

If you encounter precedence errors not covered here:
1. Check the [FAQ](../book/src/appendix/faq.md) for common solutions
2. Review the [Grammar Definition Guide](../book/src/guide/grammar-definition.md)
3. File an issue with the specific error message and minimal reproduction case

## Examples

See the [arithmetic grammar](../example/src/arithmetic.rs) for a working example of precedence attributes in action.