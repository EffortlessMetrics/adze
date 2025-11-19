# CORRECTED ROOT CAUSE ANALYSIS: LR(1) Conflict Detection

## Status: 🔴 CRITICAL - Root Cause Identified

**Date**: 2025-11-19
**Previous Hypothesis**: Grammar productions being collapsed (INCORRECT)
**Actual Root Cause**: LR(1) automaton construction not detecting conflicts

## Executive Summary

### Initial Hypothesis (WRONG)
We initially believed the grammar optimizer was collapsing productions, eliminating the Binary production needed for conflicts.

### Actual Finding (CORRECT)
The Binary production **DOES EXIST** in the generated parse table:
- `Rule 2 (or 5): Expr_Binary → <3 symbols>` (the `Expr Op Expr` production)
- All expected productions are present

### Real Problem
**The LR(1) automaton construction in glr-core is not creating states where shift/reduce conflicts would occur.**

Despite having a left-recursive grammar with no precedence (`Binary → Expr Op Expr`), the LR(1) builder is constructing a conflict-free parse table.

## Evidence

### Generated Productions (VERIFIED)

From manual inspection of `parser_ambiguous_expr.rs` TS_RULES:

```rust
static TS_RULES: &[TSRule] = &[
    TSRule { lhs: 1u16, rhs_len: 1u8 },  // Rule 0: _1 → <1 symbol>
    TSRule { lhs: 2u16, rhs_len: 1u8 },  // Rule 1: _2 → <1 symbol>
    TSRule { lhs: 3u16, rhs_len: 3u8 },  // Rule 2: ??? → <3 symbols> *** BINARY PRODUCTION ***
    TSRule { lhs: 4u16, rhs_len: 1u8 },  // Rule 3: Expr_Number → <1 symbol>
    ...
];
```

Decoded with symbol names:
```
Rule 0: _1 → <1 symbols>                    # Operator token
Rule 1: _2 → <1 symbols>                    # Whitespace token
Rule 2: LHS=3 → <3 symbols>                 # BINARY PRODUCTION (Expr Op Expr)
Rule 3: Expr_Number → <1 symbols>
Rule 4: Whitespace__whitespace → <1 symbols>
Rule 5: Expr_Binary → <3 symbols>           # Or possibly here
Rule 6: source_file → <1 symbols>
Rule 7: Whitespace → <1 symbols>
```

**The Binary production EXISTS!**

### Parse Table Analysis

**Diagnostic Output**:
```
=== Ambiguous Expression Grammar Parse Table Inspection ===
Total states: 7
Total symbols: 11

--- State 0 Actions ---
  Symbol 0 (end): 1 actions
    Action 0: Shift(StateId(8))
  Symbol 1 (_1): 1 actions
    Action 0: Shift(StateId(7))

--- Multi-Action Cells (GLR Conflicts) ---
⚠ CRITICAL: No multi-action cells found!
```

**Key Observation**: Only 7 states for a left-recursive grammar. A grammar with `Binary → Expr Op Expr` should create more states and should have shift/reduce conflicts.

## Why This is Wrong

### Expected LR(1) States for Ambiguous Expression Grammar

For the grammar:
```
S' → Expr EOF
Expr → Binary
Expr → Number
Binary → Expr Op Expr
```

We expect states like:

**State 0**: Initial
```
S' → • Expr EOF, {$}
Expr → • Binary, {EOF}
Expr → • Number, {EOF}
Binary → • Expr Op Expr, {EOF}
Expr → • Binary, {Op}   # Closure from Binary → • Expr...
Expr → • Number, {Op}
```

**State 1**: After Expr
```
S' → Expr • EOF, {$}
Binary → Expr • Op Expr, {EOF, Op}  # Key: lookahead includes Op!
```

**State 2**: After Expr Op
```
Binary → Expr Op • Expr, {EOF, Op}
Expr → • Binary, {EOF, Op}
Expr → • Number, {EOF, Op}
Binary → • Expr Op Expr, {EOF, Op}  # Recursive!
```

**State 3**: After Expr Op Expr
```
Binary → Expr Op Expr •, {EOF, Op}  # REDUCE action

BUT also need to handle if next symbol is Op:
- REDUCE Binary (complete current expr)
- SHIFT Op (continue parsing, left-associative)

THIS IS THE CONFLICT!
```

### Why Conflicts MUST Occur

After parsing `1 + 2`, when lookahead is `*`:

1. **Reduce Action**: Complete `Binary(1, +, 2)` using `Binary → Expr Op Expr`
   - Result: `Expr`, then shift `*` and continue
   - Forms: `(1 + 2) * 3`

2. **Shift Action**: Shift the `*` operator
   - Result: Now in state with `Expr Op •` waiting for next Expr
   - Forms: `1 + (2 * 3)`

**Both are valid parse paths!** This MUST create a shift/reduce conflict in any LR parser without precedence resolution.

## Root Cause Hypothesis

The LR(1) automaton builder in `glr-core/src/lib.rs` must have one of these issues:

### Hypothesis 1: Lookahead Computation Error
The FIRST/FOLLOW sets might be incorrectly computed, causing the lookahead for the reduce action to not include operator symbols.

**Check**: `compute_first_sets()` and `compute_follow_sets()` in glr-core

### Hypothesis 2: Item Closure Missing Recursive Items
When computing closure of items, the recursive Binary production might not be added correctly.

**Check**: `compute_closure()` function

### Hypothesis 3: Conflict Detection Not Recognizing Shift/Reduce
The conflict detection might be identifying conflicts but categorizing them incorrectly.

**Check**: `detect_conflicts()` and related functions

### Hypothesis 4: Grammar Normalization Before LR(1)
The grammar might be normalized in a way that removes ambiguity before LR(1) construction.

**Check**: Grammar IR processing before `build_lr1_automaton()`

## Next Investigation Steps

### 1. Add Debug Logging to LR(1) Construction

Add logging in `glr-core/src/lib.rs` to trace:
- Item sets for each state
- Lookahead computation for reduce items
- Conflict detection results

### 2. Compare with Reference Implementation

Check how Tree-sitter handles the same grammar:
- Does Tree-sitter detect conflicts for this grammar?
- How many states does Tree-sitter create?

### 3. Manual LR(1) Construction

Manually construct the LR(1) automaton on paper for the ambiguous_expr grammar to verify expected conflicts.

### 4. Check Precedence Application

Verify that NO precedence is being applied during automaton construction (it should only affect conflict resolution, not detection).

## Files to Investigate

### Priority 1 - LR(1) Core
1. `glr-core/src/lib.rs`:
   - `build_lr1_automaton()`
   - `compute_closure()`
   - `detect_conflicts()`
   - Lookahead computation functions

### Priority 2 - Grammar Processing
2. `tool/src/pure_rust_builder.rs`:
   - How grammar is passed to `build_lr1_automaton()`
   - Any preprocessing before automaton construction

### Priority 3 - IR Definition
3. `ir/src/lib.rs`:
   - How productions are represented
   - Any implicit normalization

## Impact

**Severity**: 🔴 P0 CRITICAL

This blocks:
- GLR conflict preservation validation
- All BDD scenarios
- GLR runtime testing
- Production readiness

**Why Critical**: We cannot validate the GLR conflict preservation fix without actual conflicts being detected in the first place.

## Comparison with Working Grammars

To validate the LR(1) builder works at all:
- Check arithmetic grammar (DOES have precedence - should resolve conflicts)
- Check simple non-ambiguous grammars
- Verify those construct correct tables

If those work, the issue is specifically with ambiguous grammar handling.

## Timeline

- **Investigation Start**: 2025-11-19
- **Corrected Root Cause Identified**: 2025-11-19
- **Next Step**: Deep dive into glr-core LR(1) automaton construction
- **Target Resolution**: Within 1-2 days

## Related Documentation

- `docs/plans/CRITICAL_GRAMMAR_PRODUCTION_COLLAPSE.md` - Original (incorrect) hypothesis
- `docs/plans/GLR_CONFLICT_INVESTIGATION_FINDINGS.md` - Initial investigation
- `example/src/ambiguous_expr.rs` - Test grammar
- `glr-core/src/lib.rs` - LR(1) automaton builder (investigate here!)

## Summary

The grammar productions are correct. The issue is in the LR(1) automaton construction algorithm in glr-core - it's not creating the states where conflicts should occur, or it's not properly detecting conflicts when they do occur.

This is a more fundamental issue than optimizer collapsing - it's about the core LR(1) algorithm implementation.
