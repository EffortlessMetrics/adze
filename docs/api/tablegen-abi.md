# Tablegen ABI

**Status:** Stabilizing
**Spec:** SUPPORT_TIERS.md (Tablegen TSLanguage ABI)
**Proof:** `cargo test -p adze-tablegen --all-features`

Tablegen generates compressed parse tables compatible with the Tree-sitter ABI.
This is the bridge between grammar IR and runtime decode.

## Generated output

Tablegen produces:

- Compressed parse table rows (state x symbol -> action).
- Field map metadata (field name -> child index).
- Alias sequences (production -> alias identity).
- Node type metadata (kind name, named flag, children, fields).
- External scanner metadata.
- Lex mode tables.

## ABI compatibility

Generated tables target a specific Tree-sitter ABI version. The current target
matches Tree-sitter ABI version 15.

## Validation

```bash
# Round-trip: compress -> decode -> verify
cargo test -p adze --features "pure-rust,glr,ts-compat" --test tablegen_abi_decode_roundtrip -- --nocapture
```

## Not promised

- Arbitrary internal compression implementation stability.
- ABI stability across major versions.
- Full Tree-sitter ABI feature coverage.
