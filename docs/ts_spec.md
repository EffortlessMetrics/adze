# Tree-sitter Format Specification

> **This spec governs encoder, decoder, runtime, tests, and CI. Any changes to tags/columns/reduce/goto/Accept must update all of: encoder, decoder, tests, `docs/ts_spec.md`, `docs/MERGE_CHECKLIST.md`.**

## Critical ABI Contract

**DO NOT CHANGE WITHOUT UPDATING BOTH ENCODER AND DECODER**

This document defines the exact binary format and invariants that must be maintained for Tree-sitter compatibility.

### Action Tag Constants
- `Error = 0` - Error action
- `Shift = 1` - Shift to state
- `Reduce = 3` - Reduce by rule (Note: 2 is Recover in Tree-sitter)
- `Accept = 4` - Accept the input

### Column Layout
- **Dense mapping**: Columns are 0..N-1 with no gaps
- **Token-first ordering**: 
  - Tokens occupy columns `[0..tcols)` where `tcols = token_count + external_token_count`
  - Non-terminals occupy columns `[tcols..N)`
- **External tokens**: Must be within the token band

### Action Encoding
- **Shift**: Encodes target state ID
- **Reduce**: Encodes rule ID (child_count derived from `rules[id].rhs_len`)
- **NT GOTO**: Represented as `Shift(next_state)` in NT columns
- **Accept**: Located at `GOTO(I0, start_symbol)` on EOF

### Table Structure
- **Action table**: 2D array `[state][symbol] -> Vec<Action>`
- **Symbol mapping**: `symbol_to_index` provides column for each symbol
- **Rules**: Each rule has `lhs` symbol and `rhs_len` 
- **Production LHS**: `production_lhs_index[i]` gives column index of rule i's LHS

### Decoder Requirements
- Must iterate **all** columns (not just token columns)
- Must handle multi-action cells (GLR)
- Must respect precedence/associativity ordering
- No sentinels (65535) in dense band

### External Scanner Integration
- `lex_modes`: Array of size `state_count`
- `external_token_count`: Number of external tokens
- External scanner results map to columns `[token_count..token_count+external_token_count)`

### Compression
- Small-table uses index pairs for state/symbol lookup
- Large states use full row encoding
- Actions compressed with variable-length encoding

### Invariants Enforced by Tests
1. Tag constants verified at compile time
2. Accept = GOTO(I0, start) shape preserved
3. No sentinel values in symbol tables
4. EOF within token band (typical case)
5. LHS/production agreement
6. External tokens in correct band
7. Normalization performance bounded

## Format Versions
- Current: Tree-sitter Language Version 15
- Minimum Compatible: Version 13

## ABI Stability
The GLR implementation maintains bit-for-bit compatibility with Tree-sitter's C runtime for all table formats and action encodings.