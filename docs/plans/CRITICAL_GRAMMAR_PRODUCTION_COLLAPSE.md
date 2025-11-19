# CRITICAL FINDING: Grammar Production Collapsing

## Status: 🔴 CRITICAL BUG DISCOVERED

**Date**: 2025-11-19
**Impact**: High - Prevents GLR conflict generation testing
**Root Cause**: Grammar extraction is collapsing enum variant productions

## Executive Summary

**Problem**: All three test grammars (arithmetic, dangling-else, ambiguous_expr) are generating ZERO conflicts, even when intentionally designed to create conflicts.

**Root Cause Discovery**: Deep parse table inspection reveals that the grammar productions are being severely collapsed during extraction/generation. The ambiguous expression grammar shows:

**Expected Productions**:
```
Expr → Binary
Expr → Number
Binary → Expr Op Expr  # This creates left-recursion and conflicts
```

**Actual Generated Productions**:
```
Expr → <1 symbol>  # WRONG - Binary production is missing!
```

This explains why no conflicts are being generated: the recursive Binary production that creates the ambiguity has been eliminated entirely.

## Detailed Findings

### Test Case: Ambiguous Expression Grammar

**Grammar Definition** (example/src/ambiguous_expr.rs):
```rust
#[rust_sitter::language]
#[derive(PartialEq, Eq, Debug, Clone)]
pub enum Expr {
    /// Single binary operation variant - NO precedence!
    Binary(
        Box<Expr>,
        #[rust_sitter::leaf(pattern = r"[-+*/]")] String,
        Box<Expr>,
    ),
    Number(#[rust_sitter::leaf(pattern = r"\d+", transform = |v| v.parse().unwrap())] i32),
}
```

**Expected Grammar**:
- Inherent left-recursion: `Binary(Box<Expr>, String, Box<Expr>)`
- Should create shift/reduce conflicts after "Expr Op Expr" on operator lookahead
- NO precedence annotations to resolve conflicts
- Perfect test case for GLR

### Parse Table Analysis

**Generated Parse Table**:
- **9 states** (but most are empty!)
- **11 symbols**
- **8 rules**

**Critical Rules**:
```
Rule 0: _1 → <1 symbols>        # Operator pattern
Rule 1: _6 → <1 symbols>        # Unknown
Rule 2: _7 → <1 symbols>        # Unknown
Rule 3: Whitespace__whitespace → <1 symbols>
Rule 4: Whitespace__whitespace → <1 symbols>
Rule 5: Whitespace → <3 symbols>
Rule 6: source_file → <1 symbols>
Rule 7: Expr → <1 symbols>      # ❌ WRONG - Should be Expr → Binary or Expr → Number
```

**Action Table Structure**:
```
State 0: On symbol 0 (end): SHIFT to state 7
State 1: EMPTY
State 2: EMPTY
State 3: EMPTY
State 4: EMPTY
State 5: EMPTY
State 6: On symbol 6 (source_file): SHIFT to state 7
State 7: On symbol 7 (Expr): SHIFT to state 7
State 8: EMPTY
```

**Problem**: Most states are empty! This is not a valid parser structure. The Binary production is missing entirely.

### Comparison with Expected Structure

For a grammar with `Expr → Binary | Number` and `Binary → Expr Op Expr`, we would expect:

**Expected Productions**:
1. S' → Expr EOF
2. Expr → Binary
3. Expr → Number
4. Binary → Expr Op Expr

**Expected States** (simplified):
- State 0: Initial, shift Number/goto Expr
- State 1: After Expr, shift Op (potential conflict point!)
- State 2: After Expr Op, shift Number/goto Expr
- State 3: After Expr Op Expr, reduce Binary (conflict with shift!)
- ... more states

**Expected Conflicts**:
- State 1, symbol Op: SHIFT (to read operator) vs REDUCE (complete current Expr)
- This is the classic shift/reduce conflict that GLR should preserve!

## Root Cause Investigation Needed

### Hypothesis 1: Grammar Normalization Eliminates Productions

The grammar might be normalized/optimized before LR(1) construction, eliminating recursive structures:
- **Check**: tool/common grammar extraction code
- **Look for**: Production elimination or collapsing logic
- **File**: `common/src/grammar_extractor.rs` or similar

### Hypothesis 2: Enum Translation Doesn't Create Separate Productions

Rust enum variants might not be translating to separate grammar productions:
- **Check**: How `Binary` and `Number` variants map to productions
- **Look for**: Enum variant handling in macro expansion
- **File**: `macro/src/lib.rs` and `common/src/enum_handling.rs`

### Hypothesis 3: Inline Optimization Too Aggressive

Single-child productions might be automatically inlined:
- **Check**: Production inlining logic  - **Look for**: Optimizations that collapse `Expr → Binary` chains
- **File**: `ir/src/optimizer.rs`

## Evidence Summary

| Grammar | States | Symbols | Conflicts | Expected Conflicts |
|---------|--------|---------|-----------|-------------------|
| arithmetic | 10 | 12 | 0 | 0 (precedence resolved) |
| dangling-else | 36 | 21 | 0 | 1+ (shift/reduce on else) |
| ambiguous_expr | 9 (7 active) | 11 | 0 | **MANY** (all operators) |

All three grammars show zero conflicts, which is impossible for the ambiguous_expr case.

## Next Steps

### Immediate Actions

1. **Locate Grammar Extraction Code**
   - Find where Rust enums are converted to grammar productions
   - Identify the production generation logic

2. **Add Production Debugging**
   - Add logging to show all productions before optimization
   - Add logging to show productions after normalization
   - Compare before/after to find where Binary disappears

3. **Trace Binary Variant**
   - Track how `Expr::Binary` is processed
   - Verify it creates a production `Binary → Expr Op Expr`
   - Verify `Expr → Binary` production is created

4. **Check Grammar IR**
   - Examine the IR representation before table generation
   - Verify all expected productions are present

### Investigation Targets

**Files to Examine**:
1. `tool/src/main.rs` - Entry point for grammar processing
2. `common/src/grammar.rs` - Grammar extraction logic
3. `ir/src/optimizer.rs` - Grammar optimization
4. `glr-core/src/lib.rs` - LR(1) construction (already checked conflict resolution)

**Key Questions**:
1. Are enum variants creating separate productions?
2. Is there production inlining/collapsing happening?
3. Are recursive productions being eliminated?
4. Is the IR correct before table generation?

## Impact Assessment

**Severity**: 🔴 CRITICAL

**Impact**:
- **GLR Validation**: Cannot validate conflict preservation without actual conflicts
- **Test Grammars**: All three test grammars affected
- **Production Use**: Could affect real grammars if productions are being eliminated
- **GLR Runtime**: Cannot test fork/merge logic without conflicts

**Blocker For**:
- ✅ GLR conflict preservation validation
- ✅ GLR runtime testing
- ✅ BDD scenario execution (Scenarios 1-8)
- ✅ Production-ready GLR parser

## Timeline

**Discovered**: 2025-11-19
**Priority**: P0 - Must fix before continuing GLR validation
**Estimated Investigation**: 2-4 hours
**Estimated Fix**: 2-8 hours (depends on root cause)

## Related Documentation

- `docs/plans/BDD_GLR_CONFLICT_PRESERVATION.md` - Original BDD spec (blocked)
- `docs/plans/GLR_CONFLICT_INVESTIGATION_FINDINGS.md` - Initial findings
- `example/src/ambiguous_expr.rs` - Test grammar definition
- `runtime/tests/test_ambiguous_expr_table_decode.rs` - Deep table inspection

## Test Artifacts

**Generated Parse Table**:
- Location: `target/debug/build/rust-sitter-example-*/out/grammar_ambiguous_expr/parser_ambiguous_expr.rs`
- Analysis: See detailed table decode above

**Diagnostic Tests**:
- `test_ambiguous_expr_conflicts` - Shows zero conflicts
- `test_ambiguous_expr_table_decode` - Deep table inspection revealing production collapse

## Conclusion

The GLR conflict preservation fix in `glr-core/src/lib.rs` **cannot be validated** until the grammar production collapse bug is fixed. The fix itself is likely correct, but the grammars being tested are not generating the expected productions, which prevents conflicts from occurring in the first place.

**This is a prerequisite bug** that must be fixed before continuing with GLR validation and production readiness work.
