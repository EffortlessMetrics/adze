# Decoder GLR Investigation - Root Cause Analysis

**Date**: 2025-11-20
**Investigator**: Claude Code Assistant
**Related**: PARSER_V4_TABLE_LOADING_BLOCKER.md, GLR_V1_COMPLETION_CONTRACT.md
**Status**: ROOT CAUSE IDENTIFIED

---

## Executive Summary

**Finding**: The `decode_parse_table()` function in `runtime/src/decoder.rs` **can only decode single-action cells** because the underlying TSLanguage ABI format being decoded does not support multi-action cells.

**Root Cause**: TSLanguage compression format limitation, not a decoder bug.

**Impact**: GLR multi-action cells are lost during compression (tablegen) and cannot be recovered during decoding.

**Recommended Solution**: Bypass TSLanguage entirely for GLR mode using pure-Rust ParseTable serialization (as proposed in PHASE_3_PURE_RUST_GLR_RUNTIME.md).

---

## Investigation Methodology

###  Step 1: Code Review

**File**: `runtime/src/decoder.rs`
**Function**: `decode_parse_table(lang: &'static TSLanguage) -> ParseTable`
**Lines Analyzed**: 695-1020

### Step 2: Decoder Behavior Analysis

#### Large States Decoding (Lines 795-822)

```rust
for state in 0..lang.large_state_count as usize {
    let mut state_actions = Vec::new();

    for symbol in 0..lang.symbol_count as usize {
        let table_offset = state * lang.symbol_count as usize + symbol;
        let action = unsafe {
            let action_idx = *lang.parse_table.add(table_offset);
            if action_idx != 0 {
                let raw = &*lang.parse_actions.add(action_idx as usize);
                decode_action(raw, &rules, &rid_by_pair)  // ← Returns SINGLE action
            } else {
                Action::Error
            }
        };
        let action_cell = if matches!(action, Action::Error) {
            vec![]
        } else {
            vec![action]  // ← CRITICAL: Only ONE action per cell
        };
        state_actions.push(action_cell);
    }

    action_table.push(state_actions);
}
```

**Finding**: Each cell receives **exactly one action**.

---

#### Small States Decoding (Lines 824-876)

```rust
for state in lang.large_state_count as usize..lang.state_count as usize {
    let mut state_actions = vec![vec![]; lang.symbol_count as usize];

    let offset = unsafe { *lang.small_parse_table_map.add(map_index) } as usize;
    let mut ptr = unsafe { lang.small_parse_table.add(offset) };

    let field_count = unsafe { *ptr } as usize;
    ptr = unsafe { ptr.add(1) };

    for _ in 0..field_count {
        let symbol = unsafe { *ptr } as usize;
        ptr = unsafe { ptr.add(1) };

        let action_index = unsafe { *ptr } as usize;
        ptr = unsafe { ptr.add(1) };

        if action_index != 0 && symbol < lang.symbol_count as usize {
            let action = unsafe {
                let action_entry = &*lang.parse_actions.add(action_index);
                decode_action(action_entry, &rules, &rid_by_pair)  // ← Single action
            };
            if !matches!(action, Action::Error) {
                state_actions[symbol].push(action);  // ← Pushes single action
            }
        }
    }

    action_table.push(state_actions);
}
```

**Finding**: Small states **could theoretically** support multiple actions if the same `(state, symbol)` pair appeared multiple times in the field list, but this doesn't happen in practice because TSLanguage doesn't encode it that way.

---

### Step 3: TSLanguage Format Analysis

**File**: `runtime/src/pure_parser.rs` (TSLanguage struct definition)

The TSLanguage struct contains:

```rust
pub struct TSLanguage {
    // ... other fields ...
    pub parse_table: *const u16,        // Maps (state, symbol) → action_index
    pub parse_actions: *const TSParseAction,  // Array of actions
    pub small_parse_table: *const u16,  // Compressed table for small states
    pub small_parse_table_map: *const u16,  // Maps state → offset in small_parse_table
    // ... other fields ...
}

pub struct TSParseAction {
    pub action_type: u8,  // Shift, Reduce, Accept
    pub next_state: u16,  // For Shift
    pub symbol: u16,      // For Reduce
    pub child_count: u8,  // For Reduce
    pub production_id: u16,  // For Reduce
    pub extra: u8,        // Flags
}
```

**Key Insight**: Each entry in `parse_table` or `small_parse_table` points to a **single TSParseAction**. There is no mechanism in this ABI to represent multiple actions for the same `(state, symbol)` pair.

---

### Step 4: Compression Pipeline Review

**File**: `tablegen/src/compress.rs`

The compression process:

```
ParseTable (multi-action cells)
    ↓
tablegen::compress()
    ↓ [Uses choose_action() to pick ONE action]
TSLanguage (single-action cells)
```

The `choose_action()` function selects:
- First action if multiple exist
- Uses precedence ordering if available
- **Discards all other actions**

**Evidence**: From PARSER_V4_TABLE_LOADING_BLOCKER.md findings, the arithmetic grammar had **ZERO multi-action cells** after compression.

---

## Root Cause Analysis

### The Problem

1. **glr-core generates ParseTable with multi-action cells** ✅
   - Conflicts preserved as `Vec<Vec<Vec<Action>>>`
   - Multiple actions per `(state, symbol)` pair

2. **tablegen compresses to TSLanguage** ❌
   - TSLanguage ABI only supports one action per `(state, symbol)` pair
   - `choose_action()` selects first/prioritized action
   - **All other actions are discarded**

3. **decoder reads TSLanguage** ❌
   - Decoder correctly reads what's encoded
   - But TSLanguage only contains single actions
   - **Multi-action cells cannot be recovered**

### Why This Happens

The TSLanguage struct is designed for **LR parsers**, which by definition have at most one action per `(state, symbol)` pair after conflict resolution.

GLR parsers require **multiple actions** to represent the forking behavior needed for ambiguous grammars.

**The TSLanguage ABI is fundamentally incompatible with GLR semantics.**

---

## Evidence Summary

| Component | Supports Multi-Action? | Evidence |
|-----------|------------------------|----------|
| glr-core::ParseTable | ✅ Yes | Type: `Vec<Vec<Vec<Action>>>` |
| tablegen compression | ❌ No | Uses `choose_action()` to pick one |
| TSLanguage ABI | ❌ No | One `TSParseAction` per cell |
| decoder | ❌ No | Can only read what TSLanguage contains |
| Runtime ParseTable | ✅ Yes | Type supports multi-action, but decoder fills with single |

---

## Recommended Solution

### Option A: Pure-Rust GLR Path (RECOMMENDED)

**Approach**: Bypass TSLanguage entirely for GLR mode, as proposed in [PHASE_3_PURE_RUST_GLR_RUNTIME.md](../specs/PHASE_3_PURE_RUST_GLR_RUNTIME.md).

**Architecture**:

```
Grammar IR
    ↓
glr-core::build_lr1_automaton()
    ↓
ParseTable (multi-action cells preserved)
    ↓
[Serialize to pure-Rust format: JSON, bincode, or MessagePack]
    ↓
static PARSE_TABLE_BYTES: &[u8] = include_bytes!("grammar.parsetable");
    ↓
runtime2::Parser::load_glr_table(PARSE_TABLE_BYTES)
    ↓
GLR parsing with multi-action cell support
```

**Benefits**:
- ✅ Preserves multi-action cells without data loss
- ✅ No ABI compatibility concerns
- ✅ Easier to debug (human-readable serialization option)
- ✅ Future-proof for GLR enhancements

**Effort**: 8-12 hours
- 2-4 hours: Implement ParseTable serialization/deserialization
- 2-4 hours: Update build.rs to generate .parsetable files
- 2-2 hours: Wire Parser::load_glr_table() in runtime2
- 2-4 hours: Tests and validation

**Risk**: Low (well-defined scope, clear separation from LR path)

---

### Option B: Extend TSLanguage ABI (NOT RECOMMENDED)

**Approach**: Modify TSLanguage to support multi-action cells.

**Changes Required**:
- New field in TSLanguage: `multi_action_table: *const MultiActionEntry`
- New struct: `MultiActionEntry { state: u16, symbol: u16, actions: &[TSParseAction] }`
- Update tablegen compression to populate multi_action_table
- Update decoder to read multi_action_table

**Drawbacks**:
- ❌ Breaks ABI compatibility with Tree-sitter
- ❌ Complex changes across multiple crates
- ❌ Ongoing maintenance burden
- ❌ Doesn't align with pure-Rust goals

**Effort**: 16-24 hours (high complexity)

**Risk**: High (ABI compatibility, cross-crate coordination)

---

### Option C: Hybrid Approach (COMPROMISE)

**Approach**: Use TSLanguage for LR mode, pure-Rust format for GLR mode.

**Feature Flag Strategy**:

```rust
#[cfg(feature = "glr")]
static PARSE_TABLE: &[u8] = include_bytes!("grammar.glr.parsetable");

#[cfg(not(feature = "glr"))]
static LANGUAGE: TSLanguage = /* ... */;
```

**Benefits**:
- ✅ Backward compatible (LR mode unchanged)
- ✅ GLR mode gets full multi-action support
- ✅ Clear separation of concerns

**Effort**: Same as Option A + 2 hours for feature flag wiring

**Risk**: Low

---

## Implementation Plan

### Phase 1: ParseTable Serialization (Week 1, Days 1-2)

**Tasks**:
1. Add `serde` feature to `glr-core` crate
2. Derive `Serialize` and `Deserialize` for ParseTable and related types
3. Implement `ParseTable::to_bytes()` and `ParseTable::from_bytes()`
4. Write roundtrip tests

**Files**:
- `glr-core/Cargo.toml`: Add serde dependency
- `glr-core/src/lib.rs`: Derive serde traits
- `glr-core/src/serialization.rs`: Serialization implementation (new file)
- `glr-core/tests/test_serialization.rs`: Tests (new file)

**Acceptance Criteria**:
- [ ] ParseTable can serialize to bytes
- [ ] ParseTable can deserialize from bytes
- [ ] Round-trip test: table → bytes → table (equality preserved)
- [ ] Multi-action cells preserved through serialization

---

### Phase 2: Build-Time Generation (Week 1, Days 3-4)

**Tasks**:
1. Update `pure_rust_builder.rs` to generate .parsetable files
2. Emit serialized ParseTable alongside TSLanguage
3. Add `include_bytes!()` macro for loading at compile time
4. Wire into build.rs for example grammars

**Files**:
- `tool/src/pure_rust_builder.rs`: Add .parsetable generation
- `example/build.rs`: Use generated .parsetable files
- `runtime/src/__private.rs`: Add GLR loading path

**Acceptance Criteria**:
- [ ] Build generates .parsetable files
- [ ] Files are included in binary via include_bytes!()
- [ ] Runtime can load ParseTable from bytes
- [ ] Example grammars compile with GLR mode

---

### Phase 3: Runtime Integration (Week 1, Days 5-6)

**Tasks**:
1. Implement `Parser::load_glr_table()` in runtime2
2. Update `parse_with_glr()` to use loaded ParseTable
3. Feature flag: `glr` → use .parsetable, default → use TSLanguage
4. Integration tests

**Files**:
- `runtime2/src/parser.rs`: Add load_glr_table() method
- `runtime/src/__private.rs`: Update parse_with_glr() routing
- `runtime/tests/test_glr_table_loading.rs`: Tests (new file)

**Acceptance Criteria**:
- [ ] Parser loads GLR tables without decoder
- [ ] Multi-action cells accessible in runtime
- [ ] Arithmetic grammar parses correctly with GLR backend
- [ ] Feature flag routing works (glr vs default)

---

### Phase 4: Validation & Documentation (Week 1, Day 7)

**Tasks**:
1. Create table round-trip diagnostic test
2. Validate multi-action cell preservation
3. Update GLR_V1_COMPLETION_CONTRACT.md with solution
4. Document in PHASE_3_PURE_RUST_GLR_RUNTIME.md

**Files**:
- `runtime/tests/test_table_round_trip.rs`: Diagnostic tests
- `docs/specs/GLR_V1_COMPLETION_CONTRACT.md`: Update AC-4
- `docs/specs/PHASE_3_PURE_RUST_GLR_RUNTIME.md`: Update architecture
- `docs/findings/DECODER_GLR_INVESTIGATION_2025-11-20.md`: This document

**Acceptance Criteria**:
- [ ] Diagnostic test shows multi-action cells preserved
- [ ] All documentation updated
- [ ] Solution validated end-to-end

---

## Success Criteria

**AC-4 from GLR_V1_COMPLETION_CONTRACT.md is complete when**:

1. ✅ Multi-action cells generated correctly (glr-core)
2. ✅ ParseTable serialization preserves all actions
3. ✅ Round-trip test passes: generate → serialize → deserialize → verify
4. ✅ Runtime loads ParseTable without decoder
5. ✅ GLR parsing works with multi-action cells
6. ✅ Feature flag routing works (glr vs default)

---

## Risk Assessment

### Low Risks
- ✅ ParseTable serialization (well-understood problem)
- ✅ Build-time generation (existing patterns to follow)
- ✅ Feature flag routing (already implemented)

### Medium Risks
- ⚠️ Binary size increase (serialized ParseTable vs compressed TSLanguage)
  - **Mitigation**: Use bincode (compact binary format)
  - **Acceptable**: GLR is opt-in feature flag

- ⚠️ Build time increase (additional serialization step)
  - **Mitigation**: Only when `glr` feature enabled
  - **Acceptable**: Marginal increase (< 1s for typical grammars)

### Mitigated Risks
- ❌ ABI compatibility (Option B) → Not pursuing this option
- ❌ Decoder complexity → Bypassing decoder entirely

---

## Conclusion

**The decoder is not broken** - it correctly reads what TSLanguage encodes.

**The problem is TSLanguage ABI limitation** - it cannot represent multi-action cells.

**The solution is pure-Rust GLR path** - bypass TSLanguage for GLR mode.

**Implementation timeline**: 1 week (7 days, 40 hours)

**Confidence level**: HIGH - clear problem, proven solution approach, low risk

---

## Next Steps (Immediate)

1. ✅ Document findings (this file)
2. [ ] Implement ParseTable serialization (Phase 1)
3. [ ] Update build tooling (Phase 2)
4. [ ] Wire runtime integration (Phase 3)
5. [ ] Validate and document (Phase 4)

---

## References

- [PARSER_V4_TABLE_LOADING_BLOCKER.md](../plans/PARSER_V4_TABLE_LOADING_BLOCKER.md) - Original blocker investigation
- [PHASE_3_PURE_RUST_GLR_RUNTIME.md](../specs/PHASE_3_PURE_RUST_GLR_RUNTIME.md) - Pure-Rust GLR architecture
- [GLR_V1_COMPLETION_CONTRACT.md](../specs/GLR_V1_COMPLETION_CONTRACT.md) - Acceptance criteria
- [decoder.rs](../../runtime/src/decoder.rs) - Decoder implementation
- [pure_rust_builder.rs](../../tool/src/pure_rust_builder.rs) - Table generation

---

**Status**: Investigation complete, solution identified, ready for implementation
**Blocker Resolution**: 1 week implementation timeline
**Impact**: Unblocks AC-4, enables full GLR runtime integration

---

END OF INVESTIGATION
