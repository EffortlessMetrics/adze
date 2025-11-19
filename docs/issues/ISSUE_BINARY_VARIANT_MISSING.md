# Issue: Binary Variant Missing from ambiguous_expr Grammar Generation

## Status
🔴 **CRITICAL BLOCKER** - Discovered 2025-11-19

## Summary

The `ambiguous_expr.rs` grammar's Binary variant is completely missing from the generated parse table, despite being correctly defined in the source code and the enum variant inlining implementation (ADR-0003) passing all contract tests.

## Impact

- **Severity**: CRITICAL
- **Scope**: Grammar extraction/generation pipeline
- **Affected**: All multi-field enum variants may be affected
- **Blocks**: E2E GLR validation, production deployment

## Evidence

### Source Definition (Correct)
```rust
// example/src/ambiguous_expr.rs
#[rust_sitter::language]
pub enum Expr {
    Binary(
        Box<Expr>,
        #[rust_sitter::leaf(pattern = r"[-+*/]")] String,
        Box<Expr>,
    ),
    Number(#[rust_sitter::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())] i32),
}
```

### Generated Grammar IR (Incorrect)
```json
// target/.../grammar_ambiguous_expr/grammar.ir.json
{
  "rules": {
    "1": [
      {
        "lhs": 1,  // Expr
        "rhs": [{"Terminal": 6}],  // Only Number production!
        "production_id": 0
      }
    ],
    // Binary production COMPLETELY MISSING
    "3": [
      {
        "lhs": 3,  // Expr_Binary_Expr_Binary_1 (operator fragment)
        "rhs": [{"Terminal": 8}],  // Operator terminal exists
        "production_id": 3
      }
    ]
  },
  "rule_names": {
    "1": "Expr",
    "3": "Expr_Binary_Expr_Binary_1"  // Orphaned operator symbol
  }
}
```

### Expected Grammar IR (From Specification)
```json
{
  "rules": {
    "Expr": [
      {
        "lhs": "Expr",
        "rhs": ["Expr", "OP", "Expr"],  // Binary production MISSING
        "production_id": 0
      },
      {
        "lhs": "Expr",
        "rhs": ["NUMBER"],  // Number production present
        "production_id": 1
      }
    ]
  }
}
```

### NODE_TYPES Confirms Missing Fields
```json
// NODE_TYPES.json shows Expr has NO fields at all
[
  {
    "type": "Expr",
    "named": true
    // NO FIELDS - should have Binary_0, Binary_1, Binary_2
  }
]
```

### E2E Test Results
```
test test_ambiguous_grammar_conflict_generation ... FAILED
  CONTRACT VIOLATION: Ambiguous grammar MUST generate GLR conflicts!
  Expected: At least 1 multi-action cell
  Actual: 0 conflicts

test test_ambiguous_grammar_glr_parsing ... FAILED
  ParseIntError: Empty (empty string passed to number transform)

test test_ambiguous_vs_arithmetic_comparison ... FAILED
  CONTRACT VIOLATION: Ambiguous grammar MUST have conflicts!
```

## Diagnostic Analysis

### What Works
✅ **Enum variant inlining code** - Contract tests pass (8/8)
✅ **Tool compilation** - No build errors
✅ **Grammar parsing** - ambiguous_expr.rs file is read correctly
✅ **Single-field variants** - Number variant extracts correctly

### What Fails
❌ **Multi-field variant generation** - Binary variant disappears
❌ **Orphaned symbols** - Operator field becomes standalone symbol
❌ **CHOICE members** - Binary not added to Expr CHOICE
❌ **Parse table** - No `Expr → Expr OP Expr` production

### Code Path Analysis

1. **tool/src/expansion.rs:873-897** - Enum variant loop
   - Correctly calls `should_inline_variant()` → returns `true`
   - Correctly calls `gen_struct_or_variant()` with `inline=true`
   - Should receive `Some(rule)` back with inlined Binary SEQ
   - Should add rule to `members` array

2. **tool/src/expansion.rs:794-799** - Inlining return path
   ```rust
   if inline {
       Ok(Some(rule))  // Should return Binary SEQ here
   } else {
       out.insert(path, rule);
       Ok(None)
   }
   ```

3. **Expected flow for Binary variant**:
   - `inline = true` (no precedence, 3 fields)
   - `gen_struct_or_variant()` generates SEQ:
     ```json
     {
       "type": "SEQ",
       "members": [
         {"type": "FIELD", "name": "Binary_0", "content": {"type": "SYMBOL", "name": "Expr"}},
         {"type": "FIELD", "name": "Binary_1", "content": {"type": "PATTERN", "value": "[-+*/]"}},
         {"type": "FIELD", "name": "Binary_2", "content": {"type": "SYMBOL", "name": "Expr"}}
       ]
     }
     ```
   - Returns `Ok(Some(seq))`
   - Enum handler adds seq to CHOICE members
   - Final: `Expr → CHOICE(Binary_SEQ, Number_PATTERN)`

4. **Actual result**:
   - Binary SEQ never appears in CHOICE
   - Only Number PATTERN in CHOICE
   - Operator field becomes orphaned symbol `Expr_Binary_Expr_Binary_1`

## Hypothesis

The Binary variant is being partially processed:
1. The operator field (field 1) is being extracted as a separate symbol
2. The full SEQ is NOT being returned from `gen_struct_or_variant()`
3. Likely returning `Ok(None)` instead of `Ok(Some(seq))`

**Possible causes**:
1. Early return path in `gen_struct_or_variant()` for single-leaf detection
2. Field iteration skipping non-leaf fields (Box<Expr>)
3. Error during field processing causing early exit
4. Silent failure in field generation returning None

## Reproduction Steps

1. Build example with GLR:
   ```bash
   cd example && cargo build --features glr
   ```

2. Examine generated grammar:
   ```bash
   cat target/debug/build/rust-sitter-example-*/out/grammar_ambiguous_expr/grammar.ir.json
   ```

3. Run E2E tests:
   ```bash
   cd /home/user/rust-sitter
   cargo test --manifest-path runtime/Cargo.toml --features glr --test test_e2e_ambiguous_grammar_glr -- --nocapture
   ```

## Next Steps

### Investigation Phase
- [ ] Add debug logging to `gen_struct_or_variant()` at line 506
- [ ] Trace return value for Binary variant specifically
- [ ] Check if `children` array is populated correctly for 3 fields
- [ ] Verify field iteration doesn't skip Box<T> types
- [ ] Check error handling - are errors being silently swallowed?

### Fix Phase
- [ ] Identify exact line where Binary variant processing fails
- [ ] Implement fix ensuring all fields are processed
- [ ] Verify SEQ is correctly returned for inline=true
- [ ] Add regression test to contract suite

### Validation Phase
- [ ] Re-run contract tests (should still pass)
- [ ] Re-run E2E tests (should now pass)
- [ ] Verify grammar IR contains Binary production
- [ ] Verify NODE_TYPES shows Expr fields
- [ ] Verify GLR conflicts are detected

## Related Files

- `example/src/ambiguous_expr.rs` - Source grammar definition
- `tool/src/expansion.rs:499-800` - `gen_struct_or_variant()` function
- `tool/src/expansion.rs:870-897` - Enum variant processing loop
- `tool/tests/test_grammar_extraction_contract.rs` - Contract tests (passing)
- `runtime/tests/test_e2e_ambiguous_grammar_glr.rs` - E2E tests (failing)
- `docs/specs/E2E_AMBIGUOUS_GRAMMAR_GLR_VALIDATION.md` - Test contract
- `docs/adr/0003-enum-variant-inlining-for-glr.md` - ADR for inlining

## References

- **Test Output**: Contract violation showing 0 conflicts
- **Contract**: `docs/specs/E2E_AMBIGUOUS_GRAMMAR_GLR_VALIDATION.md`
- **ADR**: `docs/adr/0003-enum-variant-inlining-for-glr.md`
- **Investigation**: `docs/plans/ENUM_VARIANT_DISAMBIGUATION.md` (RESOLVED status)

## Timeline

- **Discovered**: 2025-11-19 04:01 UTC (E2E test run)
- **Analyzed**: 2025-11-19 04:15 UTC (Grammar IR inspection)
- **Reported**: 2025-11-19 (this document)
- **Target Fix**: Next development session

---

## Contract Validation

This issue directly violates the contract defined in `E2E_AMBIGUOUS_GRAMMAR_GLR_VALIDATION.md`:

> **Contract Assertion 1: Multi-action cells exist**
> ```rust
> assert!(conflict_count > 0,
>     "Contract violation: Ambiguous grammar must generate GLR conflicts");
> ```
>
> **Status**: ❌ FAILED - 0 conflicts detected

The enum variant inlining implementation (ADR-0003) passed all unit/contract tests but fails in actual grammar build, suggesting a gap between test coverage and real-world usage that needs to be addressed.
