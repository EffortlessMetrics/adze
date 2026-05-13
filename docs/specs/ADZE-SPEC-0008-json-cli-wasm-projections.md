# ADZE-SPEC-0008: JSON, CLI, and WASM projections

Status: accepted
Owner: cli/wasm/schema
Created: 2026-05-13
Linked proposal: ../proposals/ADZE-PROP-0002-api-foundation.md
Linked ADRs: ../adr/ADZE-ADR-0004-schema-versioned-projections.md
Linked plan: ../../plans/0.9.0/api-foundation.md
Linked issues:
Linked PRs:
Support-tier impact: ../status/SUPPORT_TIERS.md
Policy impact: ../../policy/doc-artifacts.toml

## Problem

CLI and WASM users need structured output, but serialized output becomes a
contract outside Rust semver. JSON, CLI, and WASM must serialize projections of
`AdzeDocument` with explicit schema families and versions.

## Behavior

### B1. Every serialized projection has a schema family

Examples:

- `adze.document.v1`
- `adze.tree.v1`
- `adze.diagnostics.v1`
- `adze.typed-cst.v1`
- `adze.ambiguity.v1`
- `adze.forest.v1`
- `adze.node-types.v1`

### B2. Document JSON includes a schema envelope

Document JSON must identify the schema, language, source encoding/length,
parse status, selected tree, diagnostics, and ambiguity summaries when present.

### B3. CLI emits projections, not new semantics

Target commands include:

```bash
adze parse file --output document-json
adze parse file --output tree-json
adze parse file --output diagnostics-json
adze parse file --output ambiguity-json
adze parse file --output sexp
adze node-types
adze schema --name adze.document.v1
```

### B4. WASM mirrors schema families

WASM should expose schema-versioned document output before stabilizing a rich JS
object API.

### B5. Version lines are separate

Adze tracks Rust semver, document schema versions, grammar fingerprints, and
Tree-sitter ABI compatibility separately.

## Non-Goals

- No stable WASM object API before JSON schema proof.
- No full forest JSON stability yet.
- No Tree-sitter query JSON parity.
- No guarantee that all output modes are implemented in 0.9.

## Required Evidence

- JSON schema snapshot.
- Document JSON roundtrip or validation canary.
- Diagnostics JSON includes byte and point ranges.
- Node-types JSON advisory snapshot.
- CLI smoke emits schema envelope.
- WASM compile/smoke emits schema envelope when implemented.

## Acceptance Examples

```json
{
  "schema": "adze.document.v1",
  "language": {
    "name": "arithmetic"
  },
  "status": "exact",
  "tree": {},
  "diagnostics": [],
  "ambiguities": []
}
```

## Test Mapping

- `runtime/tests/adze_document_json.rs`
- future CLI document JSON tests
- future WASM schema smoke tests
- node-types metadata tests

## Implementation Mapping

Primary implementation surfaces:

- `runtime` serialization feature;
- schema files under a future `schemas/` directory;
- CLI output modes;
- WASM bindings;
- node-types projection.

## CI Proof

```bash
cargo test -p adze --features "pure-rust,serialization" --test adze_document_json -- --nocapture
cargo test -p adze --features "pure-rust,serialization,glr" --test adze_document_json parse_document_json_serializes_glr_ambiguity_summary -- --exact --nocapture
git diff --check
```

## Metrics / Promotion Rule

Serialized projections remain advisory until schema snapshots, validation
tests, and release compatibility rules exist. Rust API stability does not imply
JSON/WASM schema stability.
