# ADZE-SPEC-0008: JSON CLI WASM projections

Status: proposed
Owner: Adze maintainers
Created: 2026-05-13
Linked proposal: ../proposals/ADZE-PROP-0002-api-foundation.md
Linked ADRs: ADZE-ADR-0001 AdzeDocument one parse truth
Linked plan:
Linked issues:
Linked PRs:
Support-tier impact: experimental
Policy impact: none

## Problem

CLI output, WASM bindings, and JSON serialization must represent the same
document facts as the Rust API. Without a clear projection contract, serialized
output could drift from native document data, schema versions could be undefined,
and consumers could not trust the output.

## Behavior

### Projections consume document facts

JSON, CLI, and WASM outputs are projections of `AdzeDocument`. They do not
re-parse, invent data, or depend on separate code paths.

### Separate version lines

Serialized output carries multiple version identifiers:

| Version | Meaning |
| --- | --- |
| Rust semver | The crate version that produced the output. |
| Document schema version | The schema of the serialized document (e.g. `adze.document.v1`). |
| Grammar fingerprint | A hash identifying the grammar that produced the parse table. |
| Tree-sitter ABI compatibility | The Tree-sitter ABI version the generated tables target. |

These are separate concerns and must not be conflated.

### Schema-versioned output

Every serialized output includes a schema version tag. Consumers can detect
schema changes and handle them explicitly.

```json
{
  "schema": "adze.document.v1",
  "version": "0.9.0",
  "grammar_fingerprint": "abc123",
  "ts_abi_version": 15,
  "source": "...",
  "tree": { ... },
  "diagnostics": [ ... ],
  "ambiguities": [ ... ],
  "metadata": { ... }
}
```

### JSON projection

`AdzeDocument::to_json_value()` emits a schema-tagged JSON envelope under the
`serialization` feature. This is experimental and not a stable CLI/WASM
contract.

### CLI projection

CLI output formats (when implemented) serialize the same document facts through
different presentation schemas (human-readable, machine-readable, S-expression).

### WASM projection

WASM bindings expose the same document facts as the Rust API, serialized
through `wasm-bindgen` types. The WASM surface is compile-check verified but
not runtime certified.

### Projections are lazy

JSON, CLI, and WASM projections are computed on demand. They do not add cost
to the common `parse()` path.

## Non-Goals

- Stable JSON schema.
- Stable CLI output format.
- WASM runtime certification.
- Binary serialization format.
- Streaming output.

## Required Evidence

- `to_json_value()` output includes schema tag and version identifiers.
- JSON round-trip preserves node identity, fields, diagnostics, and ambiguity
  summaries.
- JSON cross-check: serialized child/edge indexes match native `AdzeEdge`
  projection.
- WASM build succeeds for all core crates.

## Acceptance Examples

### JSON output has schema tag

```rust
let doc = grammar::parse_document("1 + 2")?;
let json = doc.to_json_value()?;
assert_eq!(json["schema"], "adze.document.v1");
assert!(json["version"].is_string());
```

### JSON round-trip preserves facts

```rust
let doc = grammar::parse_document("1 + 2")?;
let json = doc.to_json_value()?;
// Cross-check: serialized child indexes match native edges
for edge in doc.root().edges() {
    assert!(json["tree"]["children"].as_array().unwrap().len() > 0);
}
```

## Test Mapping

- `cargo test -p adze --features "pure-rust,serialization" --test adze_document_json`
- `cargo test -p adze --features "pure-rust,serialization,glr" --test adze_document_json`
- `cargo check -p adze --target wasm32-unknown-unknown --features pure-rust`

## Implementation Mapping

| Component | Owner |
| --- | --- |
| `to_json_value()` | `runtime/src/document.rs` |
| JSON schema | `runtime/src/document.rs` |
| WASM bindings | `runtime/src/wasm_support.rs` |
| CLI output | `cli/src/` |

## CI Proof

```bash
cargo test -p adze --features "pure-rust,serialization" --test adze_document_json -- --nocapture
cargo test -p adze --features "pure-rust,serialization,glr" --test adze_document_json parse_document_json_serializes_glr_ambiguity_summary -- --exact --nocapture
cargo check -p adze --target wasm32-unknown-unknown --features pure-rust
just ci-supported
```

## Metrics And Promotion Rule

Promotion from experimental to stabilizing requires:

- JSON schema is documented and versioned.
- Round-trip fidelity is proven for all supported grammar families.
- WASM bindings expose the same document fields as the Rust API.
- No schema change breaks existing consumers without a version bump.
