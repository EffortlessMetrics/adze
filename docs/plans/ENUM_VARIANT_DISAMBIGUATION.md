# HYPOTHESIS: Enum Variants Create Implicit Disambiguation

## Date: 2025-11-19

## Status: ✅ RESOLVED - IMPLEMENTATION COMPLETE (2025-11-19)

## Hypothesis

The way adze maps Rust enum variants to grammar productions **naturally creates unambiguous grammars** even when the user intends to create ambiguity.

## Background

We've proven that:
1. ✅ The LR(1) automaton builder works correctly
2. ✅ Manually-built ambiguous grammars generate conflicts
3. ❌ Enum-based grammars generate ZERO conflicts

## The Key Difference

### Manually-Built Ambiguous Grammar (WORKS)
Using `GrammarBuilder`:
```rust
let grammar = GrammarBuilder::new("ambiguous")
    .rule("expr", vec!["binary"])
    .rule("expr", vec!["NUMBER"])
    .rule("binary", vec!["expr", "OP", "expr"])  // Ambiguous!
    .build();
```

Productions:
```
Expr → Binary
Expr → Number
Binary → Expr Op Expr  # Creates recursion through Expr
```

**Result**: Shift/reduce conflict detected ✅

###Enum-Based Grammar (FAILS)
Using Rust enums:
```rust
#[adze::language]
enum Expr {
    Binary(Box<Expr>, String, Box<Expr>),
    Number(i32),
}
```

**How adze translates this**:
```
Expr → Expr_Binary       # Separate non-terminal for Binary variant
Expr → Expr_Number       # Separate non-terminal for Number variant
Expr_Binary → Expr Op Expr
Expr_Number → NUMBER
```

Or possibly inline to:
```
Expr → Expr Op Expr      # Inline Binary variant
Expr → NUMBER            # Inline Number variant
```

## Analysis

### If Variants Are Separate Non-Terminals

The LR(1) parser can use lookahead to distinguish:
- When it sees `NUMBER`, reduce to `Expr_Number`, then to `Expr`
- When it sees complex expression start, it eventually reduces to `Expr_Binary`, then to `Expr`

**Problem**: The extra layer of non-terminals provides disambiguation points!

### If Variants Are Inlined

This SHOULD create ambiguity:
```
Expr → Expr Op Expr
Expr → NUMBER
```

After parsing "Expr Op Expr", on lookahead "Op":
- **SHIFT**: Continue to parse → "(Expr Op Expr) Op ..."
- **REDUCE**: Complete the production → "Expr Op (Expr Op ...)"

This MUST create a conflict!

## Test Plan

1. **Examine Generated Grammar.json**
   - Look at the actual productions generated from `ambiguous_expr.rs`
   - Determine if variants are separate symbols or inlined
   - Check if there are hidden intermediate symbols

2. **Add Debug Logging to Extraction**
   - Instrument `tool/src/expansion.rs` to print productions
   - See exactly how enum variants are processed

3. **Compare Production Structures**
   - Manual grammar (works): Count productions and symbols
   - Enum grammar (fails): Count productions and symbols
   - Look for structural differences

## Expected Findings

### Scenario A: Variants Are Separate Symbols
If each variant becomes a separate non-terminal, then the enum-based approach **CANNOT** create truly ambiguous grammars. This would be a fundamental architectural limitation.

**Implication**: Need to allow users to write direct productions without enum variants, OR automatically inline variant productions for ambiguous cases.

### Scenario B: Variants Are Inlined
If variants are inlined directly into Expr productions, then something ELSE is adding disambiguation:
- Implicit precedence?
- Symbol ordering?
- Special handling for recursive variants?

## Next Steps

1. Enable grammar.json emission
2. Compare generated grammar structures
3. Trace through production generation in expansion.rs
4. Identify where disambiguation is introduced
5. Fix or document the limitation

## Related Files

- `example/src/ambiguous_expr.rs` - Enum-based grammar
- `glr-core/tests/test_ambiguous_expr_lr1_construction.rs` - Working manual grammar
- `tool/src/expansion.rs` - Enum to grammar conversion
- `target/debug/build/.../grammar_ambiguous_expr/` - Generated artifacts

## Timeline

- Investigation Start: 2025-11-19
- Hypothesis Formed: 2025-11-19
- Target Resolution: Within 1-2 days

## Impact

If enum variants inherently create disambiguation, this has major implications:
- May need alternative syntax for truly ambiguous grammars
- Documentation must explain this limitation
- GLR validation tests need manually-built grammars
- Users creating ambiguous grammars need different approach

---

## ✅ CONFIRMED FINDINGS (2025-11-19)

### Test Results

Created `test_contract_enum_grammar_extraction()` which proves the hypothesis:

**Enum-based grammar extraction creates:**
```
Expr → Expr_Binary      # Intermediate symbol!
Expr → Expr_Number      # Intermediate symbol!
Expr_Binary → Expr OP Expr
Expr_Number → NUMBER
```

**Evidence:**
- Has 'Expr_Binary' intermediate symbol: **true**
- Has 'Expr_Number' intermediate symbol: **true**
- Manual grammar: **2 rules**
- Enum grammar: **7 rules**

### Code Location

The issue is in `tool/src/expansion.rs` lines 814-854:

```rust
for v in e.variants.iter() {
    let variant_path = format!("{}_{}", e.ident, v.ident);  // ← Creates Expr_Binary

    gen_struct_or_variant(
        variant_path.clone(),  // ← Generates separate rule
        v.attrs.clone(),
        v.fields.clone(),
        &mut rules_map,
        &mut word_rule,
    )?;

    members.push(json!({
        "type": "SYMBOL",
        "name": variant_path.clone()  // ← References intermediate symbol
    }));
}

// Creates: Expr → CHOICE(Expr_Binary, Expr_Number)
rules_map.insert(e.ident.to_string(), rule);
```

### Why This Prevents Conflicts

The intermediate symbols give the LR(1) parser **disambiguation points**:

1. When parsing `Expr OP Expr`, on lookahead `OP`:
   - **With intermediates**: Parser knows the Expr came from `Expr_Binary` or `Expr_Number`
   - This extra context allows it to decide without creating a conflict

2. **Without intermediates** (manual grammar):
   - Parser only knows it has `Expr OP Expr`
   - Can't distinguish whether to shift or reduce
   - **MUST create shift/reduce conflict** → GLR fork point

### Next Steps

**Solution Design Options:**

1. **Option A: Direct Inlining** (Preferred)
   - Modify expansion.rs to inline simple enum variants directly
   - Conditions for inlining:
     - No precedence attributes on variant
     - Simple field structure (not too complex)
   - Result: `Expr → Expr OP Expr` (direct)

2. **Option B: Attribute Control**
   - Add `#[adze::inline]` attribute
   - User can control when to inline vs create intermediate
   - More flexible but requires user knowledge

3. **Option C: Document Limitation**
   - Keep current behavior
   - Document that enum variants inherently disambiguate
   - Require manual grammars for truly ambiguous cases
   - Least preferred - limits GLR functionality

**Recommended Approach:**

Implement **Option A with Option B as override**:
- Default: Inline simple enum variants (no intermediate symbols)
- Attribute: `#[adze::no_inline]` to keep intermediate when needed
- This preserves backward compatibility while enabling ambiguous grammars

### Implementation Plan

1. Add flag to track whether to inline variant
2. Modify enum variant processing in expansion.rs
3. Inline variant fields directly into CHOICE members when appropriate
4. Add tests to verify inlined structure matches contract
5. Update documentation with inlining behavior

**Acceptance Criteria:**
- `test_contract_enum_grammar_extraction()` passes without contract violation
- Enum grammar generates same number of rules as manual grammar
- LR(1) tests detect conflicts in enum-based ambiguous grammar
- No regression in existing grammars (arithmetic, etc.)

---

## ✅ RESOLUTION (2025-11-19)

### Implementation Complete

Successfully implemented enum variant inlining per ADR-0003 and specification.

**Key Changes:**
- Added `should_inline_variant()` helper in `tool/src/expansion.rs:462-497`
- Modified `gen_struct_or_variant()` to support inline flag (line 505)
- Updated enum variant loop to use inlining (lines 821-846)
- Preserved backward compatibility for precedence and unit variants

**Test Results:**
- ✅ All TDD tests passing (3/3 ignored tests now pass)
- ✅ Contract tests validate JSON structure (11/11 passing)
- ✅ Example grammars build successfully
- ✅ No regression in arithmetic grammar

**Impact:**
- Enum-based grammars now generate direct productions
- GLR conflict preservation works for enum definitions
- Backward compatible via `#[no_inline]` attribute
- Precedence-based grammars unchanged

**See:**
- ADR: `docs/adr/0003-enum-variant-inlining-for-glr.md`
- Spec: `docs/specs/ENUM_VARIANT_INLINING.md`
- Tests: `tool/tests/test_grammar_extraction_contract.rs`
